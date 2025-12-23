// アラート監視エンジン
//
// システムメトリクスを監視し、閾値を超えた場合にアラートを発行する
// Tauriイベントシステムを使用してフロントエンドに通知

use crate::error::AppError;
use crate::storage::config::AlertConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// アラート重要度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AlertSeverity {
    /// クリティカル（即座に対処が必要）
    Critical,
    /// 警告（注意が必要）
    Warning,
    /// 情報（参考情報）
    Info,
    /// ヒント（改善提案）
    Tips,
}

/// メトリクス種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MetricType {
    /// CPU使用率
    CpuUsage,
    /// GPU使用率
    GpuUsage,
    /// メモリ使用率
    MemoryUsage,
    /// フレームドロップ率
    FrameDropRate,
    /// ネットワーク帯域
    NetworkBandwidth,
}

/// アラートルール（将来の動的アラート機能で使用予定）
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AlertRule {
    /// メトリクス種別
    pub metric: MetricType,
    /// 閾値
    pub threshold: f64,
    /// 継続時間（秒）
    pub duration_secs: u64,
    /// 重要度
    pub severity: AlertSeverity,
}

/// アラート情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    /// アラートID
    pub id: String,
    /// メトリクス種別
    pub metric: MetricType,
    /// 現在の値
    pub current_value: f64,
    /// 閾値
    pub threshold: f64,
    /// 重要度
    pub severity: AlertSeverity,
    /// メッセージ
    pub message: String,
    /// 発生時刻（UNIX timestamp）
    pub timestamp: u64,
    /// アクティブかどうか
    pub active: bool,
}

/// メトリクスの状態追跡（将来の動的アラート機能で使用予定）
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct MetricState {
    /// 閾値超過の開始時刻
    started_at: Option<Instant>,
    /// 最後の値
    last_value: f64,
    /// アラートが発火済みか
    alert_triggered: bool,
}

/// アラートエンジン（将来の動的アラート機能で使用予定）
#[allow(dead_code)]
pub struct AlertEngine {
    /// アラートルール
    rules: Vec<AlertRule>,
    /// メトリクス状態（キーはMetricType + AlertSeverityの組み合わせ）
    states: Arc<RwLock<HashMap<(MetricType, AlertSeverity), MetricState>>>,
    /// アクティブなアラート
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,
}

#[allow(dead_code)]
impl AlertEngine {
    /// 新しいアラートエンジンを作成
    ///
    /// # Arguments
    /// * `config` - アラート設定
    pub fn new(config: &AlertConfig) -> Self {
        let mut rules = Vec::new();

        if config.enabled {
            // CPU警告ルール
            rules.push(AlertRule {
                metric: MetricType::CpuUsage,
                threshold: config.cpu_warning_threshold,
                duration_secs: config.alert_duration_secs,
                severity: AlertSeverity::Warning,
            });

            // CPUクリティカルルール
            rules.push(AlertRule {
                metric: MetricType::CpuUsage,
                threshold: config.cpu_critical_threshold,
                duration_secs: config.alert_duration_secs,
                severity: AlertSeverity::Critical,
            });

            // GPU警告ルール
            rules.push(AlertRule {
                metric: MetricType::GpuUsage,
                threshold: config.gpu_warning_threshold,
                duration_secs: config.alert_duration_secs,
                severity: AlertSeverity::Warning,
            });

            // GPUクリティカルルール
            rules.push(AlertRule {
                metric: MetricType::GpuUsage,
                threshold: config.gpu_critical_threshold,
                duration_secs: config.alert_duration_secs,
                severity: AlertSeverity::Critical,
            });

            // フレームドロップ警告ルール
            rules.push(AlertRule {
                metric: MetricType::FrameDropRate,
                threshold: config.frame_drop_warning_threshold,
                duration_secs: config.alert_duration_secs,
                severity: AlertSeverity::Warning,
            });

            // フレームドロップクリティカルルール
            rules.push(AlertRule {
                metric: MetricType::FrameDropRate,
                threshold: config.frame_drop_critical_threshold,
                duration_secs: config.alert_duration_secs,
                severity: AlertSeverity::Critical,
            });
        }

        Self {
            rules,
            states: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// メトリクスを更新してアラートをチェック
    ///
    /// # Arguments
    /// * `metric` - メトリクス種別
    /// * `value` - 現在の値
    ///
    /// # Returns
    /// 新しく発火したアラートのリスト
    pub async fn update_metric(&self, metric: MetricType, value: f64) -> Vec<Alert> {
        let mut new_alerts = Vec::new();

        for rule in &self.rules {
            if rule.metric != metric {
                continue;
            }

            if let Some(alert) = self.check_rule(rule, value).await {
                new_alerts.push(alert);
            }
        }

        new_alerts
    }

    /// ルールをチェックしてアラートを生成
    async fn check_rule(&self, rule: &AlertRule, value: f64) -> Option<Alert> {
        let mut states = self.states.write().await;
        let state_key = (rule.metric, rule.severity);
        let state = states.entry(state_key).or_insert(MetricState {
            started_at: None,
            last_value: 0.0,
            alert_triggered: false,
        });

        state.last_value = value;

        // 閾値を超えているか
        let exceeds_threshold = value >= rule.threshold;

        if exceeds_threshold {
            // 閾値超過の開始時刻を記録
            if state.started_at.is_none() {
                state.started_at = Some(Instant::now());
            }

            // 継続時間をチェック
            if let Some(started) = state.started_at {
                let elapsed = started.elapsed();
                if elapsed >= Duration::from_secs(rule.duration_secs) && !state.alert_triggered {
                    // アラート発火
                    state.alert_triggered = true;
                    let alert = self.create_alert(rule, value).await;
                    return Some(alert);
                }
            }
        } else {
            // 閾値を下回った場合、状態をリセット
            if state.alert_triggered {
                // アラート解決
                self.resolve_alert(rule.metric, rule.severity).await;
            }
            state.started_at = None;
            state.alert_triggered = false;
        }

        None
    }

    /// アラートを作成
    async fn create_alert(&self, rule: &AlertRule, value: f64) -> Alert {
        let alert_id = format!("{:?}_{:?}", rule.metric, rule.severity);
        let message = self.generate_message(rule.metric, rule.severity, value, rule.threshold);

        let alert = Alert {
            id: alert_id.clone(),
            metric: rule.metric,
            current_value: value,
            threshold: rule.threshold,
            severity: rule.severity,
            message,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            active: true,
        };

        // アクティブアラートに追加
        let mut active = self.active_alerts.write().await;
        active.insert(alert_id, alert.clone());

        alert
    }

    /// アラートを解決
    async fn resolve_alert(&self, metric: MetricType, severity: AlertSeverity) {
        let alert_id = format!("{metric:?}_{severity:?}");
        let mut active = self.active_alerts.write().await;

        if let Some(alert) = active.get_mut(&alert_id) {
            alert.active = false;
        }

        active.remove(&alert_id);
    }

    /// アラートメッセージを生成
    fn generate_message(
        &self,
        metric: MetricType,
        severity: AlertSeverity,
        value: f64,
        threshold: f64,
    ) -> String {
        let severity_text = match severity {
            AlertSeverity::Critical => "クリティカル",
            AlertSeverity::Warning => "警告",
            AlertSeverity::Info => "情報",
            AlertSeverity::Tips => "ヒント",
        };

        match metric {
            MetricType::CpuUsage => {
                format!(
                    "[{severity_text}] CPU使用率が高い状態が続いています（{value:.1}% > {threshold:.1}%）"
                )
            }
            MetricType::GpuUsage => {
                format!(
                    "[{severity_text}] GPU使用率が高い状態が続いています（{value:.1}% > {threshold:.1}%）"
                )
            }
            MetricType::MemoryUsage => {
                format!(
                    "[{severity_text}] メモリ使用率が高い状態が続いています（{value:.1}% > {threshold:.1}%）"
                )
            }
            MetricType::FrameDropRate => {
                format!(
                    "[{severity_text}] フレームドロップが発生しています（{value:.2}% > {threshold:.2}%）"
                )
            }
            MetricType::NetworkBandwidth => {
                format!(
                    "[{severity_text}] ネットワーク帯域が不足しています（{value:.1} Mbps）"
                )
            }
        }
    }

    /// アクティブなアラート一覧を取得
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let active = self.active_alerts.read().await;
        active.values().cloned().collect()
    }

    /// すべてのアラートをクリア
    pub async fn clear_all_alerts(&self) -> Result<(), AppError> {
        let mut active = self.active_alerts.write().await;
        active.clear();

        let mut states = self.states.write().await;
        states.clear();

        Ok(())
    }
}

/// グローバルアラートエンジンインスタンス
static ALERT_ENGINE: once_cell::sync::Lazy<Arc<RwLock<Option<AlertEngine>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(None)));

/// アラートエンジンを初期化（将来の動的アラート機能で使用予定）
#[allow(dead_code)]
pub async fn initialize_alert_engine(config: &AlertConfig) {
    let engine = AlertEngine::new(config);
    let mut global = ALERT_ENGINE.write().await;
    *global = Some(engine);
}

/// グローバルアラートエンジンを取得
pub async fn get_alert_engine() -> Option<Arc<RwLock<Option<AlertEngine>>>> {
    let global = ALERT_ENGINE.read().await;
    if global.is_some() {
        Some(ALERT_ENGINE.clone())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> AlertConfig {
        AlertConfig {
            enabled: true,
            cpu_warning_threshold: 90.0,
            cpu_critical_threshold: 95.0,
            gpu_warning_threshold: 90.0,
            gpu_critical_threshold: 95.0,
            frame_drop_warning_threshold: 0.5,
            frame_drop_critical_threshold: 2.0,
            alert_duration_secs: 1, // テスト用に1秒に短縮
            play_sound: false,
            show_notification: false,
        }
    }

    #[tokio::test]
    async fn test_alert_engine_creation() {
        let config = create_test_config();
        let engine = AlertEngine::new(&config);

        assert_eq!(engine.rules.len(), 6); // CPU x2, GPU x2, FrameDrop x2
    }

    #[tokio::test]
    async fn test_alert_not_triggered_below_threshold() {
        let config = create_test_config();
        let engine = AlertEngine::new(&config);

        let alerts = engine.update_metric(MetricType::CpuUsage, 80.0).await;
        assert!(alerts.is_empty());
    }

    #[tokio::test]
    async fn test_alert_triggered_above_threshold() {
        let mut config = create_test_config();
        config.alert_duration_secs = 0; // 継続時間チェックを即座にパス
        let engine = AlertEngine::new(&config);

        // 閾値を超えて即座にアラート発火
        let alerts = engine.update_metric(MetricType::CpuUsage, 92.0).await;
        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].severity, AlertSeverity::Warning);
    }

    #[tokio::test]
    async fn test_alert_resolution() {
        let mut config = create_test_config();
        config.alert_duration_secs = 0; // 継続時間チェックを即座にパス
        let engine = AlertEngine::new(&config);

        // アラート発火
        engine.update_metric(MetricType::CpuUsage, 92.0).await;

        // 閾値を下回る
        engine.update_metric(MetricType::CpuUsage, 50.0).await;

        // アクティブアラートが空になっているはず
        let active = engine.get_active_alerts().await;
        assert!(active.is_empty() || !active[0].active);
    }

    // === 追加のエッジケーステスト ===

    #[tokio::test]
    async fn test_alert_boundary_values() {
        let mut config = create_test_config();
        config.alert_duration_secs = 0; // 継続時間チェックを即座にパス
        let engine = AlertEngine::new(&config);

        // 境界値：閾値ちょうど（90.0）
        let alerts = engine.update_metric(MetricType::CpuUsage, 90.0).await;
        assert!(!alerts.is_empty(), "境界値では即座にアラート発火（閾値以上）");

        // 境界値：閾値の直下（89.9）
        let alerts = engine.update_metric(MetricType::CpuUsage, 89.9).await;
        assert!(alerts.is_empty(), "閾値未満ではアラート発火しない");

        // 境界値：100%
        let alerts = engine.update_metric(MetricType::GpuUsage, 100.0).await;
        assert!(!alerts.is_empty(), "100%でもアラート発火すべき");
    }

    #[tokio::test]
    async fn test_alert_zero_and_negative_values() {
        let config = create_test_config();
        let engine = AlertEngine::new(&config);

        // 0%の値
        let alerts = engine.update_metric(MetricType::CpuUsage, 0.0).await;
        assert!(alerts.is_empty(), "0%ではアラート発火しない");

        // 負の値（異常だが処理すべき）
        let alerts = engine.update_metric(MetricType::CpuUsage, -5.0).await;
        assert!(alerts.is_empty(), "負の値でもクラッシュせずに処理");
    }

    #[tokio::test]
    async fn test_alert_disabled_config() {
        let mut config = create_test_config();
        config.enabled = false;
        let engine = AlertEngine::new(&config);

        // 無効化された設定ではルールが作成されない
        assert_eq!(engine.rules.len(), 0, "無効化された設定ではルールが0");

        // メトリクス更新してもアラートは発火しない
        let alerts = engine.update_metric(MetricType::CpuUsage, 99.0).await;
        assert!(alerts.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_alerts_same_metric() {
        let mut config = create_test_config();
        config.alert_duration_secs = 0; // 継続時間チェックを即座にパス
        let engine = AlertEngine::new(&config);

        // CPUを95%にして、WarningとCritical両方の閾値を超える
        let alerts = engine.update_metric(MetricType::CpuUsage, 96.0).await;

        // Warning(90%)とCritical(95%)の両方が発火すべき
        assert!(alerts.len() >= 1, "少なくとも1つのアラートが発火");
        assert!(
            alerts.iter().any(|a| a.severity == AlertSeverity::Critical),
            "Criticalアラートが含まれる"
        );
    }

    #[tokio::test]
    async fn test_alert_different_metrics_simultaneously() {
        let mut config = create_test_config();
        config.alert_duration_secs = 0; // 継続時間チェックを即座にパス
        let engine = AlertEngine::new(&config);

        // 異なるメトリクスを同時に閾値超え
        let cpu_alerts = engine.update_metric(MetricType::CpuUsage, 92.0).await;
        let gpu_alerts = engine.update_metric(MetricType::GpuUsage, 92.0).await;
        let frame_alerts = engine.update_metric(MetricType::FrameDropRate, 1.0).await;

        assert!(!cpu_alerts.is_empty(), "CPUアラート発火");
        assert!(!gpu_alerts.is_empty(), "GPUアラート発火");
        assert!(!frame_alerts.is_empty(), "フレームドロップアラート発火");
    }

    #[tokio::test]
    async fn test_alert_flapping_prevention() {
        let mut config = create_test_config();
        config.alert_duration_secs = 0; // 継続時間チェックを即座にパス
        let engine = AlertEngine::new(&config);

        // 閾値を超える
        let alerts1 = engine.update_metric(MetricType::CpuUsage, 92.0).await;
        assert!(!alerts1.is_empty(), "最初のアラート発火");

        // すぐに閾値を超え続ける（フラッピング）
        let alerts2 = engine.update_metric(MetricType::CpuUsage, 92.0).await;
        let alerts3 = engine.update_metric(MetricType::CpuUsage, 92.0).await;

        // すでに発火済みなので新しいアラートは出ない
        assert!(alerts2.is_empty(), "既に発火済みなので新規アラートなし");
        assert!(alerts3.is_empty(), "既に発火済みなので新規アラートなし");
    }

    #[tokio::test]
    async fn test_alert_message_generation() {
        let config = create_test_config();
        let engine = AlertEngine::new(&config);

        // 各メトリクスタイプのメッセージ生成をテスト
        let cpu_msg = engine.generate_message(MetricType::CpuUsage, AlertSeverity::Warning, 92.0, 90.0);
        assert!(cpu_msg.contains("CPU"));
        assert!(cpu_msg.contains("92.0"));
        assert!(cpu_msg.contains("警告"));

        let gpu_msg = engine.generate_message(MetricType::GpuUsage, AlertSeverity::Critical, 97.0, 95.0);
        assert!(gpu_msg.contains("GPU"));
        assert!(gpu_msg.contains("クリティカル"));

        let frame_msg = engine.generate_message(MetricType::FrameDropRate, AlertSeverity::Warning, 1.5, 0.5);
        assert!(frame_msg.contains("フレームドロップ"));

        let memory_msg = engine.generate_message(MetricType::MemoryUsage, AlertSeverity::Info, 80.0, 75.0);
        assert!(memory_msg.contains("メモリ"));
        assert!(memory_msg.contains("情報"));

        let network_msg = engine.generate_message(MetricType::NetworkBandwidth, AlertSeverity::Tips, 5.0, 10.0);
        assert!(network_msg.contains("ネットワーク"));
        assert!(network_msg.contains("ヒント"));
    }

    #[tokio::test]
    async fn test_clear_all_alerts() {
        let mut config = create_test_config();
        config.alert_duration_secs = 0; // 継続時間チェックを即座にパス
        let engine = AlertEngine::new(&config);

        // 複数のアラートを発火
        engine.update_metric(MetricType::CpuUsage, 92.0).await;
        engine.update_metric(MetricType::GpuUsage, 92.0).await;

        let active_before = engine.get_active_alerts().await;
        assert!(!active_before.is_empty(), "アラートが発火している");

        // すべてクリア
        let result = engine.clear_all_alerts().await;
        assert!(result.is_ok());

        let active_after = engine.get_active_alerts().await;
        assert!(active_after.is_empty(), "すべてのアラートがクリアされた");
    }

    #[tokio::test]
    async fn test_alert_duration_edge_cases() {
        let mut config = create_test_config();
        config.alert_duration_secs = 0; // 即座に発火
        let engine = AlertEngine::new(&config);

        // 継続時間0秒なので即座に発火すべき
        let alerts = engine.update_metric(MetricType::CpuUsage, 92.0).await;
        assert!(!alerts.is_empty(), "継続時間0秒では即座に発火");
    }

    #[tokio::test]
    async fn test_extreme_threshold_values() {
        let mut config = create_test_config();
        config.cpu_warning_threshold = 0.0; // 極端に低い閾値
        config.cpu_critical_threshold = 200.0; // 極端に高い閾値（超えることはない）
        config.alert_duration_secs = 0; // 継続時間チェックを即座にパス
        let engine = AlertEngine::new(&config);

        let alerts = engine.update_metric(MetricType::CpuUsage, 50.0).await;

        // Warning閾値（0.0）は超えるがCritical（200.0）は超えない
        assert!(
            alerts.iter().any(|a| a.severity == AlertSeverity::Warning),
            "Warning閾値0.0を超える"
        );
        assert!(
            !alerts.iter().any(|a| a.severity == AlertSeverity::Critical),
            "Critical閾値200.0は超えない"
        );
    }
}
