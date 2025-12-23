// 最適化適用コマンド
//
// 推奨設定をOBSに一括適用する機能

use crate::error::AppError;
use crate::obs::{get_obs_client, get_obs_settings};
use crate::services::{get_streaming_mode_service, RecommendationEngine, HardwareInfo};
use crate::storage::config::{load_config, StreamingPlatform, StreamingStyle};
use crate::storage::{SettingsProfile, ProfileSettings, save_profile as storage_save_profile};
use crate::monitor::{get_cpu_core_count, get_memory_info};
use crate::monitor::gpu::get_gpu_info;
use serde::{Deserialize, Serialize};

/// 設定バックアップ（将来のバックアップ機能で使用予定）
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsBackup {
    /// バックアップID
    pub id: String,
    /// バックアップ日時
    pub timestamp: i64,
    /// バックアップした設定
    pub settings: crate::obs::ObsSettings,
}

/// 推奨設定を一括適用
///
/// 配信中は適用不可
#[tauri::command]
pub async fn apply_recommended_settings() -> Result<(), AppError> {
    // 配信中の場合は適用を拒否
    let streaming_service = get_streaming_mode_service();
    if streaming_service.is_streaming_mode().await {
        return Err(AppError::obs_state(
            "配信中のため設定を変更できません。配信を停止してから再度お試しください。"
        ));
    }

    // OBS接続確認
    let client = get_obs_client();
    if !client.is_connected().await {
        return Err(AppError::obs_state("OBSに接続されていません"));
    }

    // 現在の設定をバックアップ
    backup_current_settings().await?;

    // 推奨設定を計算
    let config = load_config()?;
    let current_settings = get_obs_settings().await?;
    let hardware = get_hardware_info().await;

    // 推奨設定を計算（将来のOBS設定適用で使用予定）
    let _recommendations = RecommendationEngine::calculate_recommendations(
        &hardware,
        &current_settings,
        config.streaming_mode.platform,
        config.streaming_mode.style,
        config.streaming_mode.network_speed_mbps,
    );

    // TODO: Phase 2bでOBS設定適用APIを実装予定
    // 現時点では推奨設定の計算のみ実装
    // 将来的にobwsを使用して設定を適用

    Ok(())
}

/// カスタム推奨設定を適用
#[tauri::command]
pub async fn apply_custom_settings(
    platform: StreamingPlatform,
    style: StreamingStyle,
    network_speed_mbps: f64,
) -> Result<(), AppError> {
    // 配信中の場合は適用を拒否
    let streaming_service = get_streaming_mode_service();
    if streaming_service.is_streaming_mode().await {
        return Err(AppError::obs_state(
            "配信中のため設定を変更できません。配信を停止してから再度お試しください。"
        ));
    }

    // OBS接続確認
    let client = get_obs_client();
    if !client.is_connected().await {
        return Err(AppError::obs_state("OBSに接続されていません"));
    }

    // 現在の設定をバックアップ
    backup_current_settings().await?;

    // 推奨設定を計算
    let current_settings = get_obs_settings().await?;
    let hardware = get_hardware_info().await;

    // 推奨設定を計算（将来のOBS設定適用で使用予定）
    let _recommendations = RecommendationEngine::calculate_recommendations(
        &hardware,
        &current_settings,
        platform,
        style,
        network_speed_mbps,
    );

    // TODO: Phase 2bでOBS設定適用APIを実装予定

    Ok(())
}

/// 現在の設定をバックアップ
#[tauri::command]
pub async fn backup_current_settings() -> Result<String, AppError> {
    // 現在のOBS設定を取得
    let current_settings = get_obs_settings().await?;

    // バックアップIDを生成
    let backup_id = uuid::Uuid::new_v4().to_string();

    // Unixタイムスタンプを取得
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| AppError::config_error(&format!("時刻の取得に失敗: {e}")))?
        .as_secs() as i64;

    // バックアップをプロファイルとして保存
    let backup_profile = SettingsProfile {
        id: backup_id.clone(),
        name: format!("バックアップ {}", chrono::DateTime::from_timestamp(now, 0)
            .unwrap_or(chrono::DateTime::UNIX_EPOCH)
            .format("%Y-%m-%d %H:%M:%S")),
        description: "自動バックアップ".to_string(),
        platform: StreamingPlatform::Other,
        style: StreamingStyle::Other,
        settings: ProfileSettings {
            video: crate::storage::profiles::VideoSettings {
                output_width: current_settings.video.output_width,
                output_height: current_settings.video.output_height,
                fps: current_settings.video.fps() as u32,
                downscale_filter: "Lanczos".to_string(),
            },
            audio: crate::storage::profiles::AudioSettings {
                sample_rate: current_settings.audio.sample_rate,
                bitrate_kbps: 160,
            },
            output: crate::storage::profiles::OutputSettings {
                encoder: current_settings.output.encoder,
                bitrate_kbps: current_settings.output.bitrate_kbps,
                keyframe_interval_secs: current_settings.output.keyframe_interval_secs,
                preset: current_settings.output.preset,
                rate_control: current_settings.output.rate_control.unwrap_or_else(|| "CBR".to_string()),
            },
        },
        created_at: now,
        updated_at: now,
    };

    storage_save_profile(&backup_profile)?;

    Ok(backup_id)
}

/// バックアップから復元
#[tauri::command]
pub async fn restore_backup(_backup_id: String) -> Result<(), AppError> {
    // 配信中の場合は復元を拒否
    let streaming_service = get_streaming_mode_service();
    if streaming_service.is_streaming_mode().await {
        return Err(AppError::obs_state(
            "配信中のため設定を変更できません。配信を停止してから再度お試しください。"
        ));
    }

    // OBS接続確認
    let client = get_obs_client();
    if !client.is_connected().await {
        return Err(AppError::obs_state("OBSに接続されていません"));
    }

    // TODO: Phase 2bでOBS設定適用APIを実装予定
    // _backup_idからプロファイルを読み込み、設定を復元

    Ok(())
}

/// ハードウェア情報を取得（内部関数）
async fn get_hardware_info() -> HardwareInfo {
    let cpu_cores = get_cpu_core_count().unwrap_or(4);
    let (_, total_memory) = get_memory_info().unwrap_or((0, 8_000_000_000)); // デフォルト8GB
    let total_memory_gb = total_memory as f64 / 1_000_000_000.0;
    let gpu_info = get_gpu_info().await;

    HardwareInfo {
        cpu_name: "CPU".to_string(), // TODO: 実際のCPU名を取得
        cpu_cores,
        total_memory_gb,
        gpu: gpu_info,
    }
}
