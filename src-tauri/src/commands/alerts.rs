// アラート管理コマンド

use crate::error::AppError;
use crate::services::alerts::{get_alert_engine, Alert};

/// アクティブなアラート一覧を取得
#[tauri::command]
pub async fn get_active_alerts() -> Result<Vec<Alert>, AppError> {
    if let Some(engine_arc) = get_alert_engine().await {
        let engine_option = engine_arc.read().await;
        if let Some(engine) = engine_option.as_ref() {
            return Ok(engine.get_active_alerts().await);
        }
    }

    Ok(Vec::new())
}

/// すべてのアラートをクリア
#[tauri::command]
pub async fn clear_all_alerts() -> Result<(), AppError> {
    if let Some(engine_arc) = get_alert_engine().await {
        let engine_option = engine_arc.read().await;
        if let Some(engine) = engine_option.as_ref() {
            return engine.clear_all_alerts().await;
        }
    }

    Err(AppError::new(
        "ALERT_ENGINE_NOT_INITIALIZED",
        "アラートエンジンが初期化されていません",
    ))
}
