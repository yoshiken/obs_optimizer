// 配信中モード管理コマンド

use crate::error::AppError;
use crate::services::get_streaming_mode_service;

/// 配信中モードを設定
#[tauri::command]
pub async fn set_streaming_mode(enabled: bool) -> Result<(), AppError> {
    let service = get_streaming_mode_service();
    service.set_streaming_mode(enabled).await;
    Ok(())
}

/// 配信中モードを取得
#[tauri::command]
pub async fn get_streaming_mode() -> Result<bool, AppError> {
    let service = get_streaming_mode_service();
    Ok(service.is_streaming_mode().await)
}
