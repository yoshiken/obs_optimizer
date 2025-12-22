// OBS WebSocket関連のTauriコマンド
//
// フロントエンドから呼び出されるOBS操作コマンド

use serde::Deserialize;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;
use tauri::AppHandle;

use crate::error::AppError;
use crate::obs::{
    ConnectionConfig, ConnectionState, ObsEventEmitter, ObsStatus,
    ConnectionChangedPayload,
};
use crate::services::obs_service;

/// レート制限付きログ出力
///
/// 同一メッセージの連続出力を抑制し、ログ肥大化を防止する
mod rate_limited_log {
    use super::*;

    /// レート制限状態
    struct RateLimitState {
        last_log_time: Option<Instant>,
        last_message: String,
    }

    /// グローバルレート制限状態（Mutexで保護）
    static RATE_LIMIT_STATE: Lazy<Mutex<RateLimitState>> = Lazy::new(|| {
        Mutex::new(RateLimitState {
            last_log_time: None,
            last_message: String::new(),
        })
    });

    /// 最小ログ間隔（5秒）
    const MIN_LOG_INTERVAL: Duration = Duration::from_secs(5);

    /// レート制限付きでエラーログを出力
    ///
    /// 同一メッセージの場合、MIN_LOG_INTERVAL以内の再出力を抑制
    pub fn log_error(message: &str) {
        // Mutex poisoned時でもログは出力する（デバッグ情報の消失を防ぐ）
        let mut state = match RATE_LIMIT_STATE.lock() {
            Ok(s) => s,
            Err(poisoned) => {
                // Mutexがpoisonedでも、ログは出力してからリカバリ
                eprintln!("[WARN] Rate limiter mutex poisoned, logging anyway: {}", message);
                poisoned.into_inner()
            }
        };

        let now = Instant::now();

        // 前回ログと同一メッセージかチェック
        let is_same_message = state.last_message == message;

        // ログ出力判定
        let should_log = match state.last_log_time {
            Some(last) if is_same_message => {
                // 同一メッセージ: 5秒以上経過していれば出力
                now.duration_since(last) >= MIN_LOG_INTERVAL
            }
            _ => {
                // 初回 or 異なるメッセージ: 即座に出力
                true
            }
        };

        if should_log {
            eprintln!("{}", message);
            state.last_log_time = Some(now);
            state.last_message = message.to_string();
        }
    }
}

/// OBS接続パラメータ (フロントエンドからの入力)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsConnectionParams {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
}

impl From<ObsConnectionParams> for ConnectionConfig {
    fn from(params: ObsConnectionParams) -> Self {
        ConnectionConfig {
            host: params.host,
            port: params.port,
            password: params.password,
        }
    }
}

/// OBS WebSocketサーバーに接続
///
/// # Arguments
/// * `app_handle` - Tauriアプリケーションハンドル (イベント発行用)
/// * `params` - 接続パラメータ (ホスト、ポート、パスワード)
///
/// # Returns
/// 成功時はOk(()), 失敗時はAppError
#[tauri::command]
pub async fn connect_obs(
    app_handle: AppHandle,
    params: ObsConnectionParams,
) -> Result<(), AppError> {
    let config: ConnectionConfig = params.into();
    let service = obs_service();

    // 前の状態を取得
    let previous_state = service.connection_state().await;

    // 接続実行（サービス層経由）
    service.connect(config.clone()).await?;

    // 接続成功イベントを発行
    let emitter = ObsEventEmitter::new(app_handle);
    if let Err(e) = emitter.emit_connection_changed(ConnectionChangedPayload {
        previous_state,
        current_state: ConnectionState::Connected,
        host: Some(config.host),
        port: Some(config.port),
    }) {
        rate_limited_log::log_error(&format!("Failed to emit connection_changed event: {}", e));
    }

    Ok(())
}

/// OBS WebSocketサーバーから切断
///
/// # Arguments
/// * `app_handle` - Tauriアプリケーションハンドル (イベント発行用)
///
/// # Returns
/// 成功時はOk(()), 失敗時はAppError
#[tauri::command]
pub async fn disconnect_obs(app_handle: AppHandle) -> Result<(), AppError> {
    let service = obs_service();

    // 前の状態を取得
    let previous_state = service.connection_state().await;

    // 切断実行（サービス層経由）
    service.disconnect().await?;

    // 切断イベントを発行
    let emitter = ObsEventEmitter::new(app_handle);
    if let Err(e) = emitter.emit_connection_changed(ConnectionChangedPayload {
        previous_state,
        current_state: ConnectionState::Disconnected,
        host: None,
        port: None,
    }) {
        rate_limited_log::log_error(&format!("Failed to emit connection_changed event: {}", e));
    }

    Ok(())
}

/// OBSの現在のステータスを取得
///
/// 接続されていない場合は disconnected ステータスを返す
///
/// # Returns
/// OBSの現在のステータス
#[tauri::command]
pub async fn get_obs_status() -> Result<ObsStatus, AppError> {
    let service = obs_service();

    // サービス層経由でステータスを取得（未接続時の処理も含む）
    service.get_status().await
}

/// シーンリストを取得
///
/// # Returns
/// シーン名の配列
#[tauri::command]
pub async fn get_scene_list() -> Result<Vec<String>, AppError> {
    let service = obs_service();
    service.get_scene_list().await
}

/// 現在のシーンを変更
///
/// # Arguments
/// * `scene_name` - 切り替え先のシーン名
#[tauri::command]
pub async fn set_current_scene(scene_name: String) -> Result<(), AppError> {
    let service = obs_service();
    service.set_current_scene(&scene_name).await
}

/// 配信を開始
#[tauri::command]
pub async fn start_streaming(app_handle: AppHandle) -> Result<(), AppError> {
    let service = obs_service();
    service.start_streaming().await?;

    // 配信開始イベントを発行
    let emitter = ObsEventEmitter::new(app_handle);
    if let Err(e) = emitter.emit_streaming_changed(crate::obs::StreamingChangedPayload {
        is_streaming: true,
        started_at: Some(crate::obs::events::current_timestamp()),
    }) {
        rate_limited_log::log_error(&format!("Failed to emit streaming_changed event: {}", e));
    }

    Ok(())
}

/// 配信を停止
#[tauri::command]
pub async fn stop_streaming(app_handle: AppHandle) -> Result<(), AppError> {
    let service = obs_service();
    service.stop_streaming().await?;

    // 配信停止イベントを発行
    let emitter = ObsEventEmitter::new(app_handle);
    if let Err(e) = emitter.emit_streaming_changed(crate::obs::StreamingChangedPayload {
        is_streaming: false,
        started_at: None,
    }) {
        rate_limited_log::log_error(&format!("Failed to emit streaming_changed event: {}", e));
    }

    Ok(())
}

/// 録画を開始
#[tauri::command]
pub async fn start_recording(app_handle: AppHandle) -> Result<(), AppError> {
    let service = obs_service();
    service.start_recording().await?;

    // 録画開始イベントを発行
    let emitter = ObsEventEmitter::new(app_handle);
    if let Err(e) = emitter.emit_recording_changed(crate::obs::RecordingChangedPayload {
        is_recording: true,
        started_at: Some(crate::obs::events::current_timestamp()),
    }) {
        rate_limited_log::log_error(&format!("Failed to emit recording_changed event: {}", e));
    }

    Ok(())
}

/// 録画を停止
///
/// # Returns
/// 録画ファイルのパス
#[tauri::command]
pub async fn stop_recording(app_handle: AppHandle) -> Result<String, AppError> {
    let service = obs_service();
    let path = service.stop_recording().await?;

    // 録画停止イベントを発行
    let emitter = ObsEventEmitter::new(app_handle);
    if let Err(e) = emitter.emit_recording_changed(crate::obs::RecordingChangedPayload {
        is_recording: false,
        started_at: None,
    }) {
        rate_limited_log::log_error(&format!("Failed to emit recording_changed event: {}", e));
    }

    Ok(path)
}
