// プロファイル管理コマンド

use crate::error::AppError;
use crate::storage::{
    SettingsProfile, ProfileSettings, ProfileSummary,
    get_profiles as storage_get_profiles,
    get_profile as storage_get_profile,
    save_profile as storage_save_profile,
    delete_profile as storage_delete_profile,
};
use crate::obs::{get_obs_client, get_obs_settings};
use crate::services::get_streaming_mode_service;

/// プロファイル一覧を取得
#[tauri::command]
pub async fn get_profiles() -> Result<Vec<ProfileSummary>, AppError> {
    storage_get_profiles()
}

/// プロファイルを取得
#[tauri::command]
pub async fn get_profile(profile_id: String) -> Result<SettingsProfile, AppError> {
    storage_get_profile(&profile_id)
}

/// プロファイルを保存
#[tauri::command]
pub async fn save_profile(profile: SettingsProfile) -> Result<(), AppError> {
    storage_save_profile(&profile)
}

/// プロファイルを削除
#[tauri::command]
pub async fn delete_profile(profile_id: String) -> Result<(), AppError> {
    storage_delete_profile(&profile_id)
}

/// プロファイルをOBSに適用
///
/// OBSに接続していない場合はエラーを返す
#[tauri::command]
pub async fn apply_profile(profile_id: String) -> Result<(), AppError> {
    // 配信中の場合は適用を拒否
    let streaming_service = get_streaming_mode_service();
    if streaming_service.is_streaming_mode().await {
        return Err(AppError::obs_state(
            "配信中のため設定を変更できません。配信を停止してから再度お試しください。"
        ));
    }

    // プロファイルを読み込み（将来のOBS設定適用で使用予定）
    let _profile = storage_get_profile(&profile_id)?;

    // OBS接続確認
    let client = get_obs_client();
    if !client.is_connected().await {
        return Err(AppError::obs_state("OBSに接続されていません"));
    }

    // TODO: Phase 2bでOBS設定適用APIを実装予定
    // 現時点ではプロファイル読み込みのみ実装
    // 将来的にobwsを使用して設定を適用

    Ok(())
}

/// 現在のOBS設定をプロファイルとして保存
#[tauri::command]
pub async fn save_current_settings_as_profile(
    name: String,
    description: String,
    platform: crate::storage::config::StreamingPlatform,
    style: crate::storage::config::StreamingStyle,
) -> Result<String, AppError> {
    // 現在のOBS設定を取得
    let current_settings = get_obs_settings().await?;

    // プロファイルIDを生成（UUID）
    let profile_id = uuid::Uuid::new_v4().to_string();

    // Unixタイムスタンプを取得
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| AppError::config_error(&format!("時刻の取得に失敗: {e}")))?
        .as_secs() as i64;

    // プロファイルを作成
    let profile = SettingsProfile {
        id: profile_id.clone(),
        name,
        description,
        platform,
        style,
        settings: ProfileSettings {
            video: crate::storage::profiles::VideoSettings {
                output_width: current_settings.video.output_width,
                output_height: current_settings.video.output_height,
                fps: current_settings.video.fps() as u32,
                downscale_filter: "Lanczos".to_string(),
            },
            audio: crate::storage::profiles::AudioSettings {
                sample_rate: current_settings.audio.sample_rate,
                bitrate_kbps: 160, // デフォルト値
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

    // プロファイルを保存
    storage_save_profile(&profile)?;

    Ok(profile_id)
}
