// 最適化適用コマンド
//
// 推奨設定をOBSに一括適用する機能

use crate::error::AppError;
use crate::obs::{get_obs_client, get_obs_settings};
use crate::services::{get_streaming_mode_service, RecommendationEngine};
use crate::storage::config::{load_config, StreamingPlatform, StreamingStyle};
use crate::storage::{
    SettingsProfile, ProfileSettings, save_profile as storage_save_profile,
    get_profile, get_profiles,
};
use crate::commands::utils::get_hardware_info;
use serde::{Deserialize, Serialize};

/// 設定バックアップ情報（TypeScriptのBackupInfoに対応）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    /// バックアップID
    pub id: String,
    /// 作成日時（Unixタイムスタンプ）
    pub created_at: i64,
    /// 説明
    pub description: String,
    /// バックアップした設定
    pub settings: ProfileSettings,
}

/// 最適化結果（TypeScriptのOptimizationResultに対応）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptimizationResult {
    /// 適用された設定の数
    pub applied_count: usize,
    /// 適用に失敗した設定の数
    pub failed_count: usize,
    /// エラーメッセージ（失敗時）
    pub errors: Vec<String>,
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

    // 推奨設定を計算
    let recommendations = RecommendationEngine::calculate_recommendations(
        &hardware,
        &current_settings,
        config.streaming_mode.platform,
        config.streaming_mode.style,
        config.streaming_mode.network_speed_mbps,
    );

    // 推奨設定をOBSに適用
    crate::obs::settings::apply_video_settings(
        recommendations.video.output_width,
        recommendations.video.output_height,
        recommendations.video.fps,
    ).await?;

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

    // 推奨設定を計算
    let recommendations = RecommendationEngine::calculate_recommendations(
        &hardware,
        &current_settings,
        platform,
        style,
        network_speed_mbps,
    );

    // 推奨設定をOBSに適用
    crate::obs::settings::apply_video_settings(
        recommendations.video.output_width,
        recommendations.video.output_height,
        recommendations.video.fps,
    ).await?;

    Ok(())
}

/// プリセットに基づいて最適化を適用
///
/// # Arguments
/// * `preset` - 最適化プリセット（"low", "medium", "high", "ultra", "custom"）
/// * `selected_keys` - 適用する設定項目のキーリスト（省略時は全て適用）
///
/// # Returns
/// 最適化結果（適用成功数、失敗数、エラーメッセージ）
#[tauri::command]
pub async fn apply_optimization(
    preset: String,
    selected_keys: Option<Vec<String>>,
) -> Result<OptimizationResult, AppError> {
    // プリセットの検証
    let valid_presets = ["low", "medium", "high", "ultra", "custom"];
    if !valid_presets.contains(&preset.as_str()) {
        return Err(AppError::config_error(
            &format!("無効なプリセット: {}。有効な値は low, medium, high, ultra, custom です", preset)
        ));
    }

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

    // TODO: Phase 2bでOBS設定適用APIを実装予定
    // 現在はダミーのレスポンスを返す
    let _ = preset;
    let _ = selected_keys;

    Ok(OptimizationResult {
        applied_count: 0,
        failed_count: 0,
        errors: vec![],
    })
}

/// バックアップ一覧を取得
///
/// # Returns
/// バックアップ情報のリスト
#[tauri::command]
pub async fn get_backups() -> Result<Vec<BackupInfo>, AppError> {
    // プロファイル一覧を取得
    let profiles = get_profiles()?;

    // "バックアップ"で始まるプロファイルのみをフィルタリング
    let backups: Vec<BackupInfo> = profiles
        .into_iter()
        .filter(|p| p.name.starts_with("バックアップ"))
        .map(|summary| {
            // 完全なプロファイルを読み込み
            match get_profile(&summary.id) {
                Ok(profile) => Some(BackupInfo {
                    id: profile.id,
                    created_at: profile.created_at,
                    description: profile.description,
                    settings: profile.settings,
                }),
                Err(e) => {
                    eprintln!("[WARNING] バックアップの読み込みに失敗: {}", e);
                    None
                }
            }
        })
        .flatten()
        .collect();

    Ok(backups)
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
