// 最適化エンジンコマンド

use crate::error::AppError;
use crate::obs::get_obs_settings;
use crate::monitor::{get_cpu_core_count, get_cpu_name, get_memory_info};
use crate::monitor::gpu::get_gpu_info;
use crate::services::optimizer::{HardwareInfo, RecommendationEngine, RecommendedSettings};
use crate::storage::config::{load_config, StreamingPlatform, StreamingStyle};

/// OBS設定を取得
#[tauri::command]
pub async fn get_obs_settings_command() -> Result<crate::obs::ObsSettings, AppError> {
    get_obs_settings().await
}

/// 推奨設定を計算
#[tauri::command]
pub async fn calculate_recommendations() -> Result<RecommendedSettings, AppError> {
    // 設定を読み込み
    let config = load_config()?;

    // 現在のOBS設定を取得
    let current_settings = get_obs_settings().await?;

    // ハードウェア情報を収集
    let cpu_name = get_cpu_name().unwrap_or_else(|_| "Unknown CPU".to_string());
    let cpu_cores = get_cpu_core_count().unwrap_or(4);
    let (_, total_memory) = get_memory_info().unwrap_or((0, 8_000_000_000)); // デフォルト8GB
    let total_memory_gb = total_memory as f64 / 1_000_000_000.0;
    let gpu_info = get_gpu_info().await;

    let hardware = HardwareInfo {
        cpu_name,
        cpu_cores,
        total_memory_gb,
        gpu: gpu_info,
    };

    // 推奨設定を算出
    let recommendations = RecommendationEngine::calculate_recommendations(
        &hardware,
        &current_settings,
        config.streaming_mode.platform,
        config.streaming_mode.style,
        config.streaming_mode.network_speed_mbps,
    );

    Ok(recommendations)
}

/// 推奨設定をカスタムパラメーターで計算
#[tauri::command]
pub async fn calculate_custom_recommendations(
    platform: StreamingPlatform,
    style: StreamingStyle,
    network_speed_mbps: f64,
) -> Result<RecommendedSettings, AppError> {
    // 現在のOBS設定を取得
    let current_settings = get_obs_settings().await?;

    // ハードウェア情報を収集
    let cpu_name = get_cpu_name().unwrap_or_else(|_| "Unknown CPU".to_string());
    let cpu_cores = get_cpu_core_count().unwrap_or(4);
    let (_, total_memory) = get_memory_info().unwrap_or((0, 8_000_000_000)); // デフォルト8GB
    let total_memory_gb = total_memory as f64 / 1_000_000_000.0;
    let gpu_info = get_gpu_info().await;

    let hardware = HardwareInfo {
        cpu_name,
        cpu_cores,
        total_memory_gb,
        gpu: gpu_info,
    };

    // 推奨設定を算出
    let recommendations = RecommendationEngine::calculate_recommendations(
        &hardware,
        &current_settings,
        platform,
        style,
        network_speed_mbps,
    );

    Ok(recommendations)
}
