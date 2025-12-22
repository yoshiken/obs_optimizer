// OBS WebSocket連携に使用する型定義
//
// フロントエンドとの通信に使用される型は serde の rename_all = "camelCase" を使用

use serde::{Deserialize, Serialize};

/// OBS `WebSocket接続設定`
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionConfig {
    /// OBS `WebSocketサーバーのホスト` (例: "localhost")
    pub host: String,
    /// OBS `WebSocketサーバーのポート` (デフォルト: 4455)
    pub port: u16,
    /// 認証パスワード (OBS設定で有効化している場合に必要)
    pub password: Option<String>,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 4455,
            password: None,
        }
    }
}

impl ConnectionConfig {
    /// WebSocket接続URLを生成（将来使用予定）
    #[allow(dead_code)]
    pub fn to_url(&self) -> String {
        format!("ws://{}:{}", self.host, self.port)
    }

    /// 設定の妥当性を検証
    pub fn validate(&self) -> Result<(), String> {
        // ホスト名の検証
        if self.host.is_empty() {
            return Err("ホストが指定されていません".to_string());
        }
        if self.host.trim().is_empty() {
            return Err("ホストに空白文字のみが指定されています".to_string());
        }

        // ポート番号の検証（1024-65535の範囲）
        // Well-known ports（1-1023）はシステム予約のため除外
        if self.port < 1024 {
            return Err("ポート番号は1024以上である必要があります".to_string());
        }

        Ok(())
    }
}

/// 再接続設定（将来の自動再接続機能で使用予定）
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// 自動再接続を有効にするか
    pub enabled: bool,
    /// 再接続の最大試行回数 (`unlimited_retries=true` の場合は無視)
    pub max_attempts: u32,
    /// 無制限再試行を有効にするか (手動停止まで継続)
    pub unlimited_retries: bool,
    /// 再接続間隔 (ミリ秒)
    pub interval_ms: u64,
    /// 指数バックオフを使用するか
    pub exponential_backoff: bool,
    /// 最大バックオフ間隔 (ミリ秒)
    pub max_interval_ms: u64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 5,
            unlimited_retries: true, // requirements_v2.md 仕様: 無制限
            interval_ms: 1000,
            exponential_backoff: true,
            max_interval_ms: 30000,
        }
    }
}

#[allow(dead_code)]
impl ReconnectConfig {
    /// 指定された試行回数に対する待機時間を計算
    ///
    /// `requirements_v2.md` 仕様:
    /// - 初回失敗: 即座に再試行 (attempt=0)
    /// - 1回目: 1秒後, 2回目: 2秒後, 3回目: 4秒後, 4回目: 8秒後
    /// - 5回目以降: 30秒間隔
    pub fn calculate_delay(&self, attempt: u32) -> u64 {
        // 初回は即座に再試行
        if attempt == 0 {
            return 0;
        }

        if !self.exponential_backoff {
            return self.interval_ms;
        }

        // 1回目以降は指数バックオフ
        // attempt=1 -> 1秒, attempt=2 -> 2秒, attempt=3 -> 4秒, attempt=4 -> 8秒
        // checked_sub でアンダーフロー防止（attempt >= 1 が保証されているが明示的に）
        let exponent = attempt.saturating_sub(1);
        let delay = self.interval_ms * 2u64.saturating_pow(exponent);
        delay.min(self.max_interval_ms)
    }

    /// 再試行を続けるべきかどうかを判定
    pub const fn should_retry(&self, attempt: u32) -> bool {
        if !self.enabled {
            return false;
        }
        if self.unlimited_retries {
            return true;
        }
        attempt < self.max_attempts
    }
}

/// OBSの現在の状態
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ObsStatus {
    /// OBS `WebSocketに接続しているか`
    pub connected: bool,
    /// 配信中か
    pub streaming: bool,
    /// 録画中か
    pub recording: bool,
    /// 仮想カメラが有効か
    pub virtual_cam_active: bool,
    /// 現在のシーン名
    pub current_scene: Option<String>,
    /// 接続先のOBS情報
    pub obs_version: Option<String>,
    /// `WebSocketサーバーのバージョン`
    pub websocket_version: Option<String>,
    /// 配信時間 (秒)
    pub stream_timecode: Option<u64>,
    /// 録画時間 (秒)
    pub record_timecode: Option<u64>,
    /// 配信のビットレート (kbps)
    pub stream_bitrate: Option<u32>,
    /// 録画のビットレート (kbps)
    pub record_bitrate: Option<u32>,
    /// フレームレート
    pub fps: Option<f64>,
    /// ドロップフレーム数
    pub render_dropped_frames: Option<u32>,
    /// 出力ドロップフレーム数
    pub output_dropped_frames: Option<u32>,
}

impl ObsStatus {
    /// 接続状態のみを含むステータスを作成
    pub fn disconnected() -> Self {
        Self {
            connected: false,
            ..Default::default()
        }
    }

    /// 接続済みの初期ステータスを作成（将来使用予定）
    #[allow(dead_code)]
    pub fn connected_initial() -> Self {
        Self {
            connected: true,
            ..Default::default()
        }
    }
}

/// 接続状態の変化を表す型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum ConnectionState {
    /// 未接続
    #[default]
    Disconnected,
    /// 接続中
    Connecting,
    /// 接続済み
    Connected,
    /// 再接続中（将来使用予定）
    #[allow(dead_code)]
    Reconnecting,
    /// エラー状態
    Error,
}


/// シーン情報（将来使用予定）
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneInfo {
    /// シーン名
    pub name: String,
    /// シーンのインデックス
    pub index: usize,
}

/// ソース情報（将来使用予定）
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
    /// ソース名
    pub name: String,
    /// ソースタイプ
    pub source_type: String,
    /// 表示されているか
    pub visible: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_config_default() {
        let config = ConnectionConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 4455);
        assert!(config.password.is_none());
    }

    #[test]
    fn test_connection_config_to_url() {
        let config = ConnectionConfig {
            host: "192.168.1.100".to_string(),
            port: 4455,
            password: None,
        };
        assert_eq!(config.to_url(), "ws://192.168.1.100:4455");
    }

    #[test]
    fn test_connection_config_validate() {
        let valid_config = ConnectionConfig::default();
        assert!(valid_config.validate().is_ok());

        let empty_host = ConnectionConfig {
            host: "".to_string(),
            port: 4455,
            password: None,
        };
        assert!(empty_host.validate().is_err());

        let whitespace_host = ConnectionConfig {
            host: "   ".to_string(),
            port: 4455,
            password: None,
        };
        assert!(whitespace_host.validate().is_err());

        let zero_port = ConnectionConfig {
            host: "localhost".to_string(),
            port: 0,
            password: None,
        };
        assert!(zero_port.validate().is_err());

        let low_port = ConnectionConfig {
            host: "localhost".to_string(),
            port: 80,
            password: None,
        };
        assert!(low_port.validate().is_err());
    }

    #[test]
    fn test_reconnect_config_calculate_delay() {
        let config = ReconnectConfig::default();

        // 初回: 即座に再試行 (0ms)
        assert_eq!(config.calculate_delay(0), 0);
        // 1回目: 1秒後
        assert_eq!(config.calculate_delay(1), 1000);
        // 2回目: 2秒後
        assert_eq!(config.calculate_delay(2), 2000);
        // 3回目: 4秒後
        assert_eq!(config.calculate_delay(3), 4000);
        // 4回目: 8秒後
        assert_eq!(config.calculate_delay(4), 8000);
        // 5回目以降: 30秒間隔 (上限)
        assert_eq!(config.calculate_delay(5), 16000);
        assert_eq!(config.calculate_delay(10), 30000);
    }

    #[test]
    fn test_reconnect_config_no_backoff() {
        let config = ReconnectConfig {
            exponential_backoff: false,
            interval_ms: 5000,
            ..Default::default()
        };

        // 初回は即座 (バックオフ関係なく)
        assert_eq!(config.calculate_delay(0), 0);
        // 1回目以降は固定間隔
        assert_eq!(config.calculate_delay(1), 5000);
        assert_eq!(config.calculate_delay(5), 5000);
    }

    #[test]
    fn test_reconnect_config_should_retry() {
        // 無制限再試行
        let unlimited = ReconnectConfig::default();
        assert!(unlimited.should_retry(0));
        assert!(unlimited.should_retry(100));
        assert!(unlimited.should_retry(1000));

        // 制限付き再試行
        let limited = ReconnectConfig {
            unlimited_retries: false,
            max_attempts: 5,
            ..Default::default()
        };
        assert!(limited.should_retry(0));
        assert!(limited.should_retry(4));
        assert!(!limited.should_retry(5));

        // 無効化
        let disabled = ReconnectConfig {
            enabled: false,
            ..Default::default()
        };
        assert!(!disabled.should_retry(0));
    }

    #[test]
    fn test_obs_status_disconnected() {
        let status = ObsStatus::disconnected();
        assert!(!status.connected);
        assert!(!status.streaming);
        assert!(!status.recording);
    }

    #[test]
    fn test_obs_status_connected_initial() {
        let status = ObsStatus::connected_initial();
        assert!(status.connected);
        assert!(!status.streaming);
        assert!(!status.recording);
    }
}
