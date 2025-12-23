// Tauriイベント発行ヘルパー
//
// OBSの状態変化をフロントエンドに通知するためのイベント発行機能

use serde::{Deserialize, Serialize};
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[allow(clippy::unwrap_used)]
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
        assert!(ts > 1_577_836_800); // 2020-01-01 00:00:00 UTC

        // 2回呼び出して単調増加を確認
        let ts1 = current_timestamp();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let ts2 = current_timestamp();
        assert!(ts2 >= ts1);
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
    fn test_connection_changed_payload_all_states() {
        // すべての状態遷移をテスト
        let payloads = vec![
            ConnectionChangedPayload {
                previous_state: ConnectionState::Disconnected,
                current_state: ConnectionState::Connecting,
                host: Some("localhost".to_string()),
                port: Some(4455),
            },
            ConnectionChangedPayload {
                previous_state: ConnectionState::Connecting,
                current_state: ConnectionState::Connected,
                host: Some("localhost".to_string()),
                port: Some(4455),
            },
            ConnectionChangedPayload {
                previous_state: ConnectionState::Connected,
                current_state: ConnectionState::Disconnected,
                host: None,
                port: None,
            },
            ConnectionChangedPayload {
                previous_state: ConnectionState::Connected,
                current_state: ConnectionState::Error,
                host: None,
                port: None,
            },
        ];

        for payload in payloads {
            let json = serde_json::to_string(&payload).unwrap();
            assert!(!json.is_empty());
        }
    }

    #[test]
    fn test_streaming_changed_payload() {
        let payload = StreamingChangedPayload {
            is_streaming: true,
            started_at: Some(1_000_000),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("isStreaming"));
        assert!(json.contains("startedAt"));
        assert!(json.contains("1000000"));
    }

    #[test]
    fn test_streaming_changed_payload_stopped() {
        let payload = StreamingChangedPayload {
            is_streaming: false,
            started_at: None,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("isStreaming"));
        assert!(json.contains("false"));
    }

    #[test]
    fn test_recording_changed_payload() {
        let payload = RecordingChangedPayload {
            is_recording: true,
            started_at: Some(2_000_000),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("isRecording"));
        assert!(json.contains("startedAt"));
    }

    #[test]
    fn test_recording_changed_payload_stopped() {
        let payload = RecordingChangedPayload {
            is_recording: false,
            started_at: None,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("false"));
    }

    #[test]
    fn test_scene_changed_payload() {
        let payload = SceneChangedPayload {
            previous_scene: Some("Scene 1".to_string()),
            current_scene: "Scene 2".to_string(),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("previousScene"));
        assert!(json.contains("currentScene"));
        assert!(json.contains("Scene 1"));
        assert!(json.contains("Scene 2"));
    }

    #[test]
    fn test_scene_changed_payload_no_previous() {
        let payload = SceneChangedPayload {
            previous_scene: None,
            current_scene: "Initial Scene".to_string(),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("currentScene"));
        assert!(json.contains("Initial Scene"));
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
        assert!(json.contains("OBS_CONNECTION"));
        assert!(json.contains("接続エラー"));
    }

    #[test]
    fn test_error_payload_not_recoverable() {
        let payload = ErrorPayload {
            code: "FATAL_ERROR".to_string(),
            message: "致命的エラー".to_string(),
            recoverable: false,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("false"));
    }

    #[test]
    fn test_payload_deserialization() {
        // シリアライゼーション後にデシリアライゼーションできることを確認
        let original = ConnectionChangedPayload {
            previous_state: ConnectionState::Disconnected,
            current_state: ConnectionState::Connected,
            host: Some("test.local".to_string()),
            port: Some(1234),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ConnectionChangedPayload = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.host, original.host);
        assert_eq!(deserialized.port, original.port);
    }

    #[test]
    fn test_obs_status_serialization() {
        let status = ObsStatus {
            connected: true,
            streaming: true,
            recording: false,
            virtual_cam_active: false,
            current_scene: Some("Test Scene".to_string()),
            obs_version: Some("30.0.0".to_string()),
            websocket_version: Some("5.0.0".to_string()),
            stream_timecode: None,
            record_timecode: None,
            stream_bitrate: Some(6000),
            record_bitrate: None,
            fps: Some(60.0),
            render_dropped_frames: Some(10),
            output_dropped_frames: Some(5),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("connected"));
        assert!(json.contains("streaming"));
        assert!(json.contains("Test Scene"));
    }
}
