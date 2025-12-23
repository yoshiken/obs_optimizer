// 設定管理コマンド

use crate::error::AppError;
use crate::storage::config::AppConfig;
use crate::storage::{load_config, save_config};

/// 設定を取得
#[tauri::command]
pub async fn get_config() -> Result<AppConfig, AppError> {
    load_config()
}

/// 設定を保存
#[tauri::command]
pub async fn save_app_config(config: AppConfig) -> Result<(), AppError> {
    save_config(&config)
}
