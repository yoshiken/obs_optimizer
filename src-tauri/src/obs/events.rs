// Tauriイベント発行ヘルパー
//
// OBSの状態変化をフロントエンドに通知するためのイベント発行機能

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use super::types::{ConnectionState, ObsStatus};

/// OBSイベント名の定数
pub mod event_names {
    /// 接続状態変化イベント
    pub const OBS_CONNECTION_CHANGED: &str = "obs:connection-changed";
    /// 配信状態変化イベント
    pub const OBS_STREAMING_CHANGED: &str = "obs:streaming-changed";
    /// 録画状態変化イベント
    pub const OBS_RECORDING_CHANGED: &str = "obs:recording-changed";
    /// ステータス更新イベント（将来使用予定）
    #[allow(dead_code)]
    pub const OBS_STATUS_UPDATE: &str = "obs:status-update";
    /// シーン変更イベント（将来使用予定）
    #[allow(dead_code)]
    pub const OBS_SCENE_CHANGED: &str = "obs:scene-changed";
    /// エラーイベント（将来使用予定）
    #[allow(dead_code)]
    pub const OBS_ERROR: &str = "obs:error";
}

/// 接続状態変化ペイロード
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionChangedPayload {
    /// 前の状態
    pub previous_state: ConnectionState,
    /// 現在の状態
    pub current_state: ConnectionState,
    /// 接続先情報 (接続時のみ)
    pub host: Option<String>,
    /// ポート (接続時のみ)
    pub port: Option<u16>,
}

/// 配信状態変化ペイロード
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingChangedPayload {
    /// 配信中かどうか
    pub is_streaming: bool,
    /// 配信開始時刻 (Unix timestamp、配信開始時のみ)
    pub started_at: Option<u64>,
}

/// 録画状態変化ペイロード
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingChangedPayload {
    /// 録画中かどうか
    pub is_recording: bool,
    /// 録画開始時刻 (Unix timestamp、録画開始時のみ)
    pub started_at: Option<u64>,
}

/// シーン変更ペイロード（将来使用予定）
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneChangedPayload {
    /// 前のシーン名
    pub previous_scene: Option<String>,
    /// 現在のシーン名
    pub current_scene: String,
}

/// エラーペイロード（将来使用予定）
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPayload {
    /// エラーコード
    pub code: String,
    /// エラーメッセージ
    pub message: String,
    /// 回復可能かどうか
    pub recoverable: bool,
}

/// OBSイベント発行器
///
/// Tauriのappハンドルを保持し、OBS関連のイベントをフロントエンドに発行する
#[derive(Clone)]
pub struct ObsEventEmitter {
    app_handle: AppHandle,
}

impl ObsEventEmitter {
    /// 新しいイベント発行器を作成
    ///
    /// # Arguments
    /// * `app_handle` - `TauriのAppHandle`
    pub const fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    /// `AppHandleから作成` (Manager traitを使用)（将来使用予定）
    #[allow(dead_code)]
    pub fn from_manager<M: Manager<tauri::Wry>>(manager: &M) -> Self {
        Self {
            app_handle: manager.app_handle().clone(),
        }
    }

    /// 接続状態変化を通知
    pub fn emit_connection_changed(&self, payload: ConnectionChangedPayload) -> Result<(), String> {
        self.app_handle
            .emit(event_names::OBS_CONNECTION_CHANGED, payload)
            .map_err(|e| format!("イベント発行エラー: {e}"))
    }

    /// 配信状態変化を通知
    pub fn emit_streaming_changed(&self, payload: StreamingChangedPayload) -> Result<(), String> {
        self.app_handle
            .emit(event_names::OBS_STREAMING_CHANGED, payload)
            .map_err(|e| format!("イベント発行エラー: {e}"))
    }

    /// 録画状態変化を通知
    pub fn emit_recording_changed(&self, payload: RecordingChangedPayload) -> Result<(), String> {
        self.app_handle
            .emit(event_names::OBS_RECORDING_CHANGED, payload)
            .map_err(|e| format!("イベント発行エラー: {e}"))
    }

    /// ステータス更新を通知（将来使用予定）
    #[allow(dead_code)]
    pub fn emit_status_update(&self, status: ObsStatus) -> Result<(), String> {
        self.app_handle
            .emit(event_names::OBS_STATUS_UPDATE, status)
            .map_err(|e| format!("イベント発行エラー: {e}"))
    }

    /// シーン変更を通知（将来使用予定）
    #[allow(dead_code)]
    pub fn emit_scene_changed(&self, payload: SceneChangedPayload) -> Result<(), String> {
        self.app_handle
            .emit(event_names::OBS_SCENE_CHANGED, payload)
            .map_err(|e| format!("イベント発行エラー: {e}"))
    }

    /// エラーを通知（将来使用予定）
    #[allow(dead_code)]
    pub fn emit_error(&self, payload: ErrorPayload) -> Result<(), String> {
        self.app_handle
            .emit(event_names::OBS_ERROR, payload)
            .map_err(|e| format!("イベント発行エラー: {e}"))
    }
}

/// 簡易的なイベント発行ヘルパー関数（将来使用予定）
///
/// グローバルなAppHandleを使用せずに、直接イベントを発行する場合に使用
#[allow(dead_code)]
pub fn emit_obs_event<T: Serialize + Clone>(
    app_handle: &AppHandle,
    event_name: &str,
    payload: T,
) -> Result<(), String> {
    app_handle
        .emit(event_name, payload)
        .map_err(|e| format!("イベント発行エラー: {e}"))
}

/// 現在時刻をUnix timestampで取得（将来使用予定）
#[allow(dead_code)]
pub fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_name_constants() {
        assert_eq!(event_names::OBS_CONNECTION_CHANGED, "obs:connection-changed");
        assert_eq!(event_names::OBS_STREAMING_CHANGED, "obs:streaming-changed");
        assert_eq!(event_names::OBS_RECORDING_CHANGED, "obs:recording-changed");
        assert_eq!(event_names::OBS_STATUS_UPDATE, "obs:status-update");
        assert_eq!(event_names::OBS_SCENE_CHANGED, "obs:scene-changed");
        assert_eq!(event_names::OBS_ERROR, "obs:error");
    }

    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        // 現在時刻は2020年以降であるべき
        assert!(ts > 1577836800); // 2020-01-01 00:00:00 UTC
    }

    #[test]
    fn test_payload_serialization() {
        let payload = ConnectionChangedPayload {
            previous_state: ConnectionState::Disconnected,
            current_state: ConnectionState::Connected,
            host: Some("localhost".to_string()),
            port: Some(4455),
        };

        let json = serde_json::to_string(&payload);
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("previousState"));
        assert!(json_str.contains("currentState"));
    }

    #[test]
    fn test_error_payload() {
        let payload = ErrorPayload {
            code: "OBS_CONNECTION".to_string(),
            message: "接続エラー".to_string(),
            recoverable: true,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("recoverable"));
    }
}
