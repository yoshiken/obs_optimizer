// アプリケーション設定ファイル管理
//
// %APPDATA%/obs-optimizer/config.json に保存
// デフォルト値を提供し、存在しない場合は自動作成

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const APP_NAME: &str = "obs-optimizer";
const CONFIG_FILE_NAME: &str = "config.json";

/// アプリケーション設定全体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    /// 設定ファイルバージョン
    pub version: String,
    /// OBS接続設定
    pub connection: ConnectionConfig,
    /// 監視設定
    pub monitoring: MonitoringConfig,
    /// アラート設定
    pub alerts: AlertConfig,
    /// 表示設定
    pub display: DisplayConfig,
    /// 配信モード設定
    pub streaming_mode: StreamingModeConfig,
}

/// OBS接続設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionConfig {
    /// 最後に接続したホスト
    pub last_host: String,
    /// 最後に接続したポート
    pub last_port: u16,
    /// パスワードを保存するか（OSのキーリングに保存）
    pub save_password: bool,
    /// 起動時に自動接続するか
    pub auto_connect_on_startup: bool,
    /// 接続タイムアウト（秒）
    pub connection_timeout_secs: u64,
    /// 【移行用】旧プレーンテキストパスワード
    /// 読み込み時に検出された場合、キーリングに移行して削除
    #[serde(default, skip_serializing_if = "Option::is_none")]
    saved_password: Option<String>,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            last_host: "localhost".to_string(),
            last_port: 4455,
            save_password: false,
            auto_connect_on_startup: false,
            connection_timeout_secs: 10,
            saved_password: None,
        }
    }
}

impl ConnectionConfig {
    /// 旧プレーンテキストパスワードを取得（移行用）
    ///
    /// 設定ファイルにプレーンテキストで保存されていたパスワードを取得。
    /// キーリングへの移行処理で使用する。
    pub fn get_legacy_password(&self) -> Option<&str> {
        self.saved_password.as_deref()
    }

    /// 旧プレーンテキストパスワードをクリア（移行後）
    ///
    /// キーリングへの移行が完了した後、設定ファイルから
    /// プレーンテキストパスワードを削除する。
    pub fn clear_legacy_password(&mut self) {
        self.saved_password = None;
    }

    /// 旧プレーンテキストパスワードが存在するか
    pub fn has_legacy_password(&self) -> bool {
        self.saved_password.as_ref().is_some_and(|p| !p.is_empty())
    }
}

/// 監視設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitoringConfig {
    /// メトリクス更新間隔（ミリ秒）
    pub update_interval_ms: u64,
    /// システムメトリクスを収集するか
    pub collect_system_metrics: bool,
    /// GPUメトリクスを収集するか（NVIDIA専用）
    pub collect_gpu_metrics: bool,
    /// OBSプロセスメトリクスを収集するか
    pub collect_process_metrics: bool,
    /// メトリクス履歴を保存するか
    pub save_metrics_history: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            update_interval_ms: 1000, // 1秒
            collect_system_metrics: true,
            collect_gpu_metrics: true,
            collect_process_metrics: true,
            save_metrics_history: true,
        }
    }
}

/// アラート設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertConfig {
    /// アラートを有効にするか
    pub enabled: bool,
    /// CPU使用率警告閾値（%）
    pub cpu_warning_threshold: f64,
    /// CPU使用率クリティカル閾値（%）
    pub cpu_critical_threshold: f64,
    /// GPU使用率警告閾値（%）
    pub gpu_warning_threshold: f64,
    /// GPU使用率クリティカル閾値（%）
    pub gpu_critical_threshold: f64,
    /// フレームドロップ率警告閾値（%）
    pub frame_drop_warning_threshold: f64,
    /// フレームドロップ率クリティカル閾値（%）
    pub frame_drop_critical_threshold: f64,
    /// アラート判定に必要な継続時間（秒）
    pub alert_duration_secs: u64,
    /// アラート音を鳴らすか
    pub play_sound: bool,
    /// デスクトップ通知を表示するか
    pub show_notification: bool,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cpu_warning_threshold: 90.0,
            cpu_critical_threshold: 95.0,
            gpu_warning_threshold: 90.0,
            gpu_critical_threshold: 95.0,
            frame_drop_warning_threshold: 0.5,
            frame_drop_critical_threshold: 2.0,
            alert_duration_secs: 5,
            play_sound: true,
            show_notification: true,
        }
    }
}

/// 表示設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayConfig {
    /// ダークモードを使用するか
    pub dark_mode: bool,
    /// メトリクスグラフの履歴表示時間（秒）
    pub graph_history_duration_secs: u64,
    /// コンパクト表示モード
    pub compact_mode: bool,
    /// 常に最前面に表示
    pub always_on_top: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            dark_mode: true,
            graph_history_duration_secs: 60, // 1分
            compact_mode: false,
            always_on_top: false,
        }
    }
}

/// 配信モード設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingModeConfig {
    /// 配信プラットフォーム
    pub platform: StreamingPlatform,
    /// 配信スタイル
    pub style: StreamingStyle,
    /// ネットワーク速度（Mbps）
    pub network_speed_mbps: f64,
    /// 画質優先モード
    pub quality_priority: bool,
}

impl Default for StreamingModeConfig {
    fn default() -> Self {
        Self {
            platform: StreamingPlatform::YouTube,
            style: StreamingStyle::Gaming,
            network_speed_mbps: 10.0,
            quality_priority: false,
        }
    }
}

/// 配信プラットフォーム
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StreamingPlatform {
    /// YouTube
    YouTube,
    /// Twitch
    Twitch,
    /// ニコニコ生放送
    NicoNico,
    /// ツイキャス
    TwitCasting,
    /// その他
    Other,
}

/// 配信スタイル
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StreamingStyle {
    /// 雑談・トーク
    Talk,
    /// ゲーム実況
    Gaming,
    /// 歌・演奏
    Music,
    /// お絵描き・制作
    Art,
    /// その他
    Other,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            connection: ConnectionConfig::default(),
            monitoring: MonitoringConfig::default(),
            alerts: AlertConfig::default(),
            display: DisplayConfig::default(),
            streaming_mode: StreamingModeConfig::default(),
        }
    }
}

/// 設定ファイルのパスを取得
///
/// Windows: %APPDATA%/obs-optimizer/config.json
/// Linux: ~/.config/obs-optimizer/config.json
/// macOS: ~/Library/Application Support/obs-optimizer/config.json
fn get_config_path() -> Result<PathBuf, AppError> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| AppError::new("CONFIG_ERROR", "設定ディレクトリを取得できませんでした"))?;

    let app_config_dir = config_dir.join(APP_NAME);
    let config_path = app_config_dir.join(CONFIG_FILE_NAME);

    Ok(config_path)
}

/// 設定ディレクトリを作成
fn ensure_config_dir() -> Result<PathBuf, AppError> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| AppError::new("CONFIG_ERROR", "設定ディレクトリを取得できませんでした"))?;

    let app_config_dir = config_dir.join(APP_NAME);

    if !app_config_dir.exists() {
        std::fs::create_dir_all(&app_config_dir)?;
    }

    Ok(app_config_dir)
}

/// 設定ファイルを読み込む
///
/// ファイルが存在しない場合はデフォルト値を返す。
/// プレーンテキストパスワードが検出された場合は、キーリングへの移行を試行する。
pub fn load_config() -> Result<AppConfig, AppError> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        // ファイルが存在しない場合はデフォルト値を返す
        return Ok(AppConfig::default());
    }

    let content = std::fs::read_to_string(&config_path)?;
    let mut config: AppConfig = serde_json::from_str(&content)?;

    // プレーンテキストパスワードの移行処理
    if config.connection.has_legacy_password() {
        migrate_legacy_password(&mut config);
    }

    Ok(config)
}

/// プレーンテキストパスワードをキーリングに移行
///
/// 移行成功時は設定ファイルからプレーンテキストを削除して保存。
/// 移行失敗時は警告を出力するが、アプリは続行する。
fn migrate_legacy_password(config: &mut AppConfig) {
    use super::credentials::migrate_from_plaintext;

    let legacy_password = config.connection.get_legacy_password();

    match migrate_from_plaintext(legacy_password) {
        Ok(true) => {
            // 移行成功: プレーンテキストを削除して保存
            config.connection.clear_legacy_password();
            if let Err(e) = save_config(config) {
                tracing::warn!(
                    target: "config",
                    error = %e,
                    "設定ファイルの更新に失敗（パスワードは移行済み）"
                );
            } else {
                tracing::info!(
                    target: "config",
                    "プレーンテキストパスワードを設定ファイルから削除しました"
                );
            }
        },
        Ok(false) => {
            // キーリングが利用できない場合
            // パスワードは設定ファイルに残る（後方互換性）
        },
        Err(e) => {
            tracing::warn!(target: "config", error = %e, "パスワード移行エラー");
        },
    }
}

/// 設定ファイルを保存する
pub fn save_config(config: &AppConfig) -> Result<(), AppError> {
    ensure_config_dir()?;
    let config_path = get_config_path()?;

    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(&config_path, content)?;

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.connection.last_host, "localhost");
        assert_eq!(config.connection.last_port, 4455);
        assert!(config.alerts.enabled);
        assert_eq!(config.alerts.cpu_warning_threshold, 90.0);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.version, deserialized.version);
        assert_eq!(
            config.connection.last_host,
            deserialized.connection.last_host
        );
    }

    #[test]
    fn test_streaming_platform_serialization() {
        let platform = StreamingPlatform::YouTube;
        let json = serde_json::to_string(&platform).unwrap();
        assert_eq!(json, r#""youTube""#);
    }

    #[test]
    fn test_streaming_style_serialization() {
        let style = StreamingStyle::Gaming;
        let json = serde_json::to_string(&style).unwrap();
        assert_eq!(json, r#""gaming""#);
    }

    // === 追加のエッジケーステスト ===

    #[test]
    fn test_all_streaming_platforms_serialization() {
        // すべてのプラットフォームがシリアライズ可能
        for platform in [
            StreamingPlatform::YouTube,
            StreamingPlatform::Twitch,
            StreamingPlatform::NicoNico,
            StreamingPlatform::TwitCasting,
            StreamingPlatform::Other,
        ] {
            let json = serde_json::to_string(&platform).unwrap();
            let deserialized: StreamingPlatform = serde_json::from_str(&json).unwrap();
            assert_eq!(platform, deserialized);
        }
    }

    #[test]
    fn test_all_streaming_styles_serialization() {
        // すべてのスタイルがシリアライズ可能
        for style in [
            StreamingStyle::Talk,
            StreamingStyle::Gaming,
            StreamingStyle::Music,
            StreamingStyle::Art,
            StreamingStyle::Other,
        ] {
            let json = serde_json::to_string(&style).unwrap();
            let deserialized: StreamingStyle = serde_json::from_str(&json).unwrap();
            assert_eq!(style, deserialized);
        }
    }

    #[test]
    fn test_config_default_values() {
        let config = AppConfig::default();

        // ConnectionConfig デフォルト値
        assert_eq!(config.connection.last_host, "localhost");
        assert_eq!(config.connection.last_port, 4455);
        assert!(
            !config.connection.save_password,
            "デフォルトではパスワード保存しない"
        );
        assert!(!config.connection.auto_connect_on_startup);
        assert_eq!(config.connection.connection_timeout_secs, 10);

        // MonitoringConfig デフォルト値
        assert_eq!(config.monitoring.update_interval_ms, 1000);
        assert!(config.monitoring.collect_system_metrics);
        assert!(config.monitoring.collect_gpu_metrics);
        assert!(config.monitoring.collect_process_metrics);
        assert!(config.monitoring.save_metrics_history);

        // AlertConfig デフォルト値
        assert!(config.alerts.enabled);
        assert_eq!(config.alerts.cpu_warning_threshold, 90.0);
        assert_eq!(config.alerts.cpu_critical_threshold, 95.0);
        assert_eq!(config.alerts.gpu_warning_threshold, 90.0);
        assert_eq!(config.alerts.gpu_critical_threshold, 95.0);
        assert_eq!(config.alerts.frame_drop_warning_threshold, 0.5);
        assert_eq!(config.alerts.frame_drop_critical_threshold, 2.0);
        assert_eq!(config.alerts.alert_duration_secs, 5);

        // DisplayConfig デフォルト値
        assert!(config.display.dark_mode, "デフォルトはダークモード");
        assert_eq!(config.display.graph_history_duration_secs, 60);
        assert!(!config.display.compact_mode);
        assert!(!config.display.always_on_top);

        // StreamingModeConfig デフォルト値
        assert_eq!(config.streaming_mode.platform, StreamingPlatform::YouTube);
        assert_eq!(config.streaming_mode.style, StreamingStyle::Gaming);
        assert_eq!(config.streaming_mode.network_speed_mbps, 10.0);
        assert!(!config.streaming_mode.quality_priority);
    }

    #[test]
    fn test_config_round_trip_serialization() {
        let config = AppConfig::default();

        // JSON化して戻す
        let json = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

        // 主要フィールドが一致
        assert_eq!(config.version, deserialized.version);
        assert_eq!(
            config.connection.last_host,
            deserialized.connection.last_host
        );
        assert_eq!(
            config.alerts.cpu_warning_threshold,
            deserialized.alerts.cpu_warning_threshold
        );
        assert_eq!(
            config.streaming_mode.platform,
            deserialized.streaming_mode.platform
        );
    }

    #[test]
    fn test_config_partial_json() {
        // 一部のフィールドが欠けているJSONからデシリアライズ
        let partial_json = r#"{
            "version": "1.0.0",
            "connection": {
                "lastHost": "192.168.1.1",
                "lastPort": 4455,
                "savePassword": false,
                "autoConnectOnStartup": true,
                "connectionTimeoutSecs": 10
            },
            "monitoring": {
                "updateIntervalMs": 1000,
                "collectSystemMetrics": true,
                "collectGpuMetrics": true,
                "collectProcessMetrics": true,
                "saveMetricsHistory": true
            },
            "alerts": {
                "enabled": true,
                "cpuWarningThreshold": 90.0,
                "cpuCriticalThreshold": 95.0,
                "gpuWarningThreshold": 90.0,
                "gpuCriticalThreshold": 95.0,
                "frameDropWarningThreshold": 0.5,
                "frameDropCriticalThreshold": 2.0,
                "alertDurationSecs": 5,
                "playSound": true,
                "showNotification": true
            },
            "display": {
                "darkMode": true,
                "graphHistoryDurationSecs": 60,
                "compactMode": false,
                "alwaysOnTop": false
            },
            "streamingMode": {
                "platform": "youTube",
                "style": "gaming",
                "networkSpeedMbps": 10.0,
                "qualityPriority": false
            }
        }"#;

        let config: Result<AppConfig, _> = serde_json::from_str(partial_json);
        assert!(config.is_ok(), "部分的なJSONでもデシリアライズ可能");

        let config = config.unwrap();
        assert_eq!(config.connection.last_host, "192.168.1.1");
        assert!(config.connection.auto_connect_on_startup);
    }

    #[test]
    fn test_invalid_json_format() {
        let invalid_json = r#"{ "version": "1.0.0", invalid syntax }"#;
        let result: Result<AppConfig, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err(), "不正なJSONはエラーになる");
    }

    #[test]
    fn test_unknown_enum_values() {
        // 未知のenum値を含むJSON
        let json_with_unknown = r#"{
            "version": "1.0.0",
            "connection": {
                "lastHost": "localhost",
                "lastPort": 4455,
                "savePassword": false,
                "autoConnectOnStartup": false,
                "connectionTimeoutSecs": 10
            },
            "monitoring": {
                "updateIntervalMs": 1000,
                "collectSystemMetrics": true,
                "collectGpuMetrics": true,
                "collectProcessMetrics": true,
                "saveMetricsHistory": true
            },
            "alerts": {
                "enabled": true,
                "cpuWarningThreshold": 90.0,
                "cpuCriticalThreshold": 95.0,
                "gpuWarningThreshold": 90.0,
                "gpuCriticalThreshold": 95.0,
                "frameDropWarningThreshold": 0.5,
                "frameDropCriticalThreshold": 2.0,
                "alertDurationSecs": 5,
                "playSound": true,
                "showNotification": true
            },
            "display": {
                "darkMode": true,
                "graphHistoryDurationSecs": 60,
                "compactMode": false,
                "alwaysOnTop": false
            },
            "streamingMode": {
                "platform": "unknownPlatform",
                "style": "gaming",
                "networkSpeedMbps": 10.0,
                "qualityPriority": false
            }
        }"#;

        let result: Result<AppConfig, _> = serde_json::from_str(json_with_unknown);
        assert!(result.is_err(), "未知のenum値はエラーになる");
    }

    #[test]
    fn test_boundary_values_for_thresholds() {
        let mut config = AppConfig::default();

        // 境界値：0%
        config.alerts.cpu_warning_threshold = 0.0;
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.alerts.cpu_warning_threshold, 0.0);

        // 境界値：100%
        config.alerts.cpu_critical_threshold = 100.0;
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.alerts.cpu_critical_threshold, 100.0);

        // 境界値：200%（異常だがシリアライズ可能）
        config.alerts.gpu_critical_threshold = 200.0;
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.alerts.gpu_critical_threshold, 200.0);
    }

    #[test]
    fn test_extreme_network_speed_values() {
        let mut config = AppConfig::default();

        // 極端に低い速度
        config.streaming_mode.network_speed_mbps = 0.1;
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.streaming_mode.network_speed_mbps, 0.1);

        // 極端に高い速度
        config.streaming_mode.network_speed_mbps = 10000.0;
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.streaming_mode.network_speed_mbps, 10000.0);

        // 0
        config.streaming_mode.network_speed_mbps = 0.0;
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.streaming_mode.network_speed_mbps, 0.0);
    }

    #[test]
    fn test_extreme_timeout_values() {
        let mut config = AppConfig::default();

        // 非常に短いタイムアウト
        config.connection.connection_timeout_secs = 1;
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.connection.connection_timeout_secs, 1);

        // 非常に長いタイムアウト
        config.connection.connection_timeout_secs = 3600;
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.connection.connection_timeout_secs, 3600);
    }

    #[test]
    fn test_extreme_update_interval() {
        let mut config = AppConfig::default();

        // 非常に短い更新間隔（100ms）
        config.monitoring.update_interval_ms = 100;
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.monitoring.update_interval_ms, 100);

        // 非常に長い更新間隔（1分）
        config.monitoring.update_interval_ms = 60000;
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.monitoring.update_interval_ms, 60000);
    }

    #[test]
    fn test_config_with_all_boolean_false() {
        let mut config = AppConfig::default();

        // すべてのブール値をfalseに
        config.connection.save_password = false;
        config.connection.auto_connect_on_startup = false;
        config.monitoring.collect_system_metrics = false;
        config.monitoring.collect_gpu_metrics = false;
        config.monitoring.collect_process_metrics = false;
        config.monitoring.save_metrics_history = false;
        config.alerts.enabled = false;
        config.alerts.play_sound = false;
        config.alerts.show_notification = false;
        config.display.dark_mode = false;
        config.display.compact_mode = false;
        config.display.always_on_top = false;
        config.streaming_mode.quality_priority = false;

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

        assert!(!deserialized.connection.save_password);
        assert!(!deserialized.monitoring.collect_system_metrics);
        assert!(!deserialized.alerts.enabled);
        assert!(!deserialized.display.dark_mode);
    }

    #[test]
    fn test_config_with_all_boolean_true() {
        let mut config = AppConfig::default();

        // すべてのブール値をtrueに
        config.connection.save_password = true;
        config.connection.auto_connect_on_startup = true;
        config.monitoring.collect_system_metrics = true;
        config.monitoring.collect_gpu_metrics = true;
        config.monitoring.collect_process_metrics = true;
        config.monitoring.save_metrics_history = true;
        config.alerts.enabled = true;
        config.alerts.play_sound = true;
        config.alerts.show_notification = true;
        config.display.dark_mode = true;
        config.display.compact_mode = true;
        config.display.always_on_top = true;
        config.streaming_mode.quality_priority = true;

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

        assert!(deserialized.connection.save_password);
        assert!(deserialized.monitoring.collect_system_metrics);
        assert!(deserialized.alerts.enabled);
        assert!(deserialized.display.dark_mode);
    }

    #[test]
    fn test_config_json_format() {
        let config = AppConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();

        // camelCase形式であることを確認
        assert!(json.contains("lastHost"), "camelCase形式");
        assert!(json.contains("updateIntervalMs"), "camelCase形式");
        assert!(json.contains("cpuWarningThreshold"), "camelCase形式");
        assert!(!json.contains("last_host"), "snake_caseではない");
    }

    #[test]
    fn test_special_characters_in_host() {
        let mut config = AppConfig::default();

        // 特殊文字を含むホスト名
        config.connection.last_host = "obs-server.local".to_string();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.connection.last_host, "obs-server.local");

        // IPv6アドレス
        config.connection.last_host = "::1".to_string();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.connection.last_host, "::1");
    }

    #[test]
    fn test_port_boundary_values() {
        let mut config = AppConfig::default();

        // 最小ポート
        config.connection.last_port = 1;
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.connection.last_port, 1);

        // 最大ポート
        config.connection.last_port = 65535;
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.connection.last_port, 65535);

        // 一般的なポート
        config.connection.last_port = 4455;
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.connection.last_port, 4455);
    }

    // === レガシーパスワード移行テスト ===

    #[test]
    fn test_legacy_password_detection() {
        // デフォルトではレガシーパスワードなし
        let config = ConnectionConfig::default();
        assert!(!config.has_legacy_password());
        assert!(config.get_legacy_password().is_none());
    }

    #[test]
    fn test_legacy_password_from_json() {
        // 旧形式のJSONからレガシーパスワードを読み込む
        let json_with_password = r#"{
            "lastHost": "localhost",
            "lastPort": 4455,
            "savePassword": true,
            "savedPassword": "secret123",
            "autoConnectOnStartup": false,
            "connectionTimeoutSecs": 10
        }"#;

        let config: ConnectionConfig = serde_json::from_str(json_with_password).unwrap();
        assert!(config.has_legacy_password());
        assert_eq!(config.get_legacy_password(), Some("secret123"));
    }

    #[test]
    fn test_legacy_password_clear() {
        let json_with_password = r#"{
            "lastHost": "localhost",
            "lastPort": 4455,
            "savePassword": true,
            "savedPassword": "secret123",
            "autoConnectOnStartup": false,
            "connectionTimeoutSecs": 10
        }"#;

        let mut config: ConnectionConfig = serde_json::from_str(json_with_password).unwrap();
        assert!(config.has_legacy_password());

        config.clear_legacy_password();
        assert!(!config.has_legacy_password());
        assert!(config.get_legacy_password().is_none());
    }

    #[test]
    fn test_legacy_password_empty_string() {
        // 空文字列はレガシーパスワードとして扱わない
        let json_with_empty = r#"{
            "lastHost": "localhost",
            "lastPort": 4455,
            "savePassword": true,
            "savedPassword": "",
            "autoConnectOnStartup": false,
            "connectionTimeoutSecs": 10
        }"#;

        let config: ConnectionConfig = serde_json::from_str(json_with_empty).unwrap();
        assert!(!config.has_legacy_password());
    }

    #[test]
    fn test_legacy_password_not_serialized_when_none() {
        // レガシーパスワードがNoneの場合、JSONには出力されない
        let config = ConnectionConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();

        // savedPasswordフィールドが含まれていないこと
        assert!(
            !json.contains("savedPassword"),
            "Noneのsaved_passwordはシリアライズされない"
        );
    }
}
