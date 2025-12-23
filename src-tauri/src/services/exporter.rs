// データエクスポート機能
//
// セッションデータ、診断レポートをJSON/CSV形式でエクスポート

use crate::error::AppError;
use crate::services::analyzer::ProblemReport;
use crate::storage::metrics_history::{HistoricalMetrics, SessionSummary};
use serde::{Deserialize, Serialize};

/// 診断レポート
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticReport {
    /// レポート生成日時（UNIX epoch秒）
    pub generated_at: i64,
    /// セッション情報
    pub session: SessionInfo,
    /// システム情報
    pub system_info: SystemInfo,
    /// 検出された問題
    pub problems: Vec<ProblemReport>,
    /// パフォーマンス評価
    pub performance: PerformanceEvaluation,
    /// 推奨事項サマリー
    pub recommendations_summary: String,
}

/// セッション情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    /// セッションID
    pub session_id: String,
    /// 配信時間（秒）
    pub duration_secs: i64,
    /// 配信開始時刻
    pub started_at: i64,
    /// 配信終了時刻
    pub ended_at: i64,
}

/// システム情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    /// OS情報
    pub os: String,
    /// CPUモデル
    pub cpu_model: String,
    /// 総メモリ（MB）
    pub total_memory_mb: u64,
    /// GPUモデル
    pub gpu_model: Option<String>,
}

/// パフォーマンス評価
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceEvaluation {
    /// 総合スコア（0-100）
    pub overall_score: f64,
    /// CPU評価（0-100）
    pub cpu_score: f64,
    /// GPU評価（0-100）
    pub gpu_score: f64,
    /// ネットワーク評価（0-100）
    pub network_score: f64,
    /// 安定性評価（0-100）
    pub stability_score: f64,
}

/// レポートエクスポーター
pub struct ReportExporter;

impl ReportExporter {
    /// 新しいエクスポーターを作成
    pub fn new() -> Self {
        Self
    }

    /// セッションデータをJSON形式でエクスポート
    ///
    /// # Arguments
    /// * `session_summary` - セッションサマリー
    /// * `metrics_history` - メトリクス履歴
    ///
    /// # Returns
    /// JSON文字列
    pub fn export_session_json(
        &self,
        session_summary: &SessionSummary,
        metrics_history: &[HistoricalMetrics],
    ) -> Result<String, AppError> {
        let export_data = serde_json::json!({
            "version": "1.0",
            "exported_at": chrono::Utc::now().timestamp(),
            "session": session_summary,
            "metrics": metrics_history,
        });

        serde_json::to_string_pretty(&export_data)
            .map_err(|e| AppError::export_error(&format!("Failed to serialize JSON: {e}")))
    }

    /// セッションデータをCSV形式でエクスポート
    ///
    /// # Arguments
    /// * `metrics_history` - メトリクス履歴
    ///
    /// # Returns
    /// CSV文字列
    pub fn export_session_csv(&self, metrics_history: &[HistoricalMetrics]) -> Result<String, AppError> {
        let mut csv = String::new();

        // ヘッダー
        csv.push_str("timestamp,session_id,cpu_usage,memory_used_mb,memory_total_mb,gpu_usage,network_upload_mbps,network_download_mbps,streaming,recording,fps,dropped_frames\n");

        // データ行
        for metrics in metrics_history {
            csv.push_str(&format!(
                "{},{},{:.2},{},{},{:.2},{:.2},{:.2},{},{},{:.2},{}\n",
                metrics.timestamp,
                metrics.session_id,
                metrics.system.cpu_usage,
                metrics.system.memory_used / 1024 / 1024,
                metrics.system.memory_total / 1024 / 1024,
                metrics.system.gpu_usage.unwrap_or(0.0),
                metrics.system.network_upload as f64 / 1_000_000.0 * 8.0, // バイト/秒 → Mbps
                metrics.system.network_download as f64 / 1_000_000.0 * 8.0,
                metrics.obs.streaming,
                metrics.obs.recording,
                metrics.obs.fps.unwrap_or(0.0),
                metrics.obs.output_dropped_frames.unwrap_or(0),
            ));
        }

        Ok(csv)
    }

    /// 診断レポートを生成
    ///
    /// # Arguments
    /// * `session_summary` - セッションサマリー
    /// * `problems` - 検出された問題
    ///
    /// # Returns
    /// 診断レポート
    pub fn generate_diagnostic_report(
        &self,
        session_summary: &SessionSummary,
        problems: &[ProblemReport],
    ) -> Result<DiagnosticReport, AppError> {
        // システム情報の取得
        let system_info = self.get_system_info();

        // パフォーマンス評価の計算
        let performance = self.calculate_performance_evaluation(session_summary, problems);

        // 推奨事項サマリーの生成
        let recommendations_summary = self.generate_recommendations_summary(problems);

        let report = DiagnosticReport {
            generated_at: chrono::Utc::now().timestamp(),
            session: SessionInfo {
                session_id: session_summary.session_id.clone(),
                duration_secs: session_summary.end_time - session_summary.start_time,
                started_at: session_summary.start_time,
                ended_at: session_summary.end_time,
            },
            system_info,
            problems: problems.to_vec(),
            performance,
            recommendations_summary,
        };

        Ok(report)
    }

    /// システム情報を取得
    fn get_system_info(&self) -> SystemInfo {
        // TODO: 実際のシステム情報を取得
        // 現在はダミーデータ
        SystemInfo {
            os: std::env::consts::OS.to_string(),
            cpu_model: "Unknown CPU".to_string(),
            total_memory_mb: 16384,
            gpu_model: Some("Unknown GPU".to_string()),
        }
    }

    /// パフォーマンス評価を計算
    fn calculate_performance_evaluation(
        &self,
        session_summary: &SessionSummary,
        problems: &[ProblemReport],
    ) -> PerformanceEvaluation {
        // CPU評価: 平均CPU使用率から算出
        let cpu_score = (100.0 - session_summary.avg_cpu).clamp(0.0, 100.0);

        // GPU評価: 平均GPU使用率から算出
        let gpu_score = (100.0 - session_summary.avg_gpu).clamp(0.0, 100.0);

        // ネットワーク評価: ピークビットレートが目標に達しているかで算出
        let network_score = if session_summary.peak_bitrate >= 6000 {
            90.0
        } else if session_summary.peak_bitrate >= 4000 {
            70.0
        } else {
            50.0
        };

        // 安定性評価: ドロップフレーム数から算出
        let stability_score = if session_summary.total_dropped_frames == 0 {
            100.0
        } else if session_summary.total_dropped_frames < 100 {
            80.0
        } else if session_summary.total_dropped_frames < 500 {
            60.0
        } else {
            40.0
        };

        // クリティカル問題がある場合はペナルティ
        let critical_penalty = problems.iter()
            .filter(|p| matches!(p.severity, crate::services::alerts::AlertSeverity::Critical))
            .count() as f64 * 10.0;

        // 総合スコア
        let overall_score = ((cpu_score + gpu_score + network_score + stability_score) / 4.0
            - critical_penalty)
            .clamp(0.0, 100.0);

        PerformanceEvaluation {
            overall_score,
            cpu_score,
            gpu_score,
            network_score,
            stability_score,
        }
    }

    /// 推奨事項サマリーを生成
    fn generate_recommendations_summary(&self, problems: &[ProblemReport]) -> String {
        if problems.is_empty() {
            return "問題は検出されませんでした。現在の設定は良好です。".to_string();
        }

        let critical_count = problems.iter()
            .filter(|p| matches!(p.severity, crate::services::alerts::AlertSeverity::Critical))
            .count();

        let warning_count = problems.iter()
            .filter(|p| matches!(p.severity, crate::services::alerts::AlertSeverity::Warning))
            .count();

        let mut summary = format!(
            "{}個の問題が検出されました（クリティカル: {}, 警告: {}）。\n\n",
            problems.len(),
            critical_count,
            warning_count
        );

        if critical_count > 0 {
            summary.push_str("優先対応:\n");
            for (i, problem) in problems.iter()
                .filter(|p| matches!(p.severity, crate::services::alerts::AlertSeverity::Critical))
                .take(3)
                .enumerate()
            {
                summary.push_str(&format!("{}. {}\n", i + 1, problem.title));
            }
        }

        summary
    }
}

impl Default for ReportExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::services::alerts::{AlertSeverity, MetricType};
    use crate::services::analyzer::ProblemCategory;
    use crate::storage::metrics_history::{SystemMetricsSnapshot, ObsStatusSnapshot};

    fn create_test_session_summary() -> SessionSummary {
        SessionSummary {
            session_id: "test_session".to_string(),
            start_time: 1_000_000,
            end_time: 1_003_600,
            avg_cpu: 65.0,
            avg_gpu: 70.0,
            total_dropped_frames: 50,
            peak_bitrate: 6000,
            quality_score: 75.0,
        }
    }

    fn create_test_problem() -> ProblemReport {
        ProblemReport {
            id: "test-problem-1".to_string(),
            category: ProblemCategory::Resource,
            severity: AlertSeverity::Warning,
            title: "Test Problem".to_string(),
            description: "Test description".to_string(),
            suggested_actions: vec!["Action 1".to_string()],
            affected_metric: MetricType::CpuUsage,
            detected_at: 1_000_000,
        }
    }

    #[test]
    fn test_export_json() {
        let exporter = ReportExporter::new();
        let summary = create_test_session_summary();
        let metrics = vec![HistoricalMetrics {
            timestamp: 1_000_000,
            session_id: "test".to_string(),
            system: SystemMetricsSnapshot {
                cpu_usage: 50.0,
                memory_used: 8_000_000_000,
                memory_total: 16_000_000_000,
                gpu_usage: Some(60.0),
                gpu_memory_used: Some(4_000_000_000),
                network_upload: 1_000_000,
                network_download: 500_000,
            },
            obs: ObsStatusSnapshot::empty(),
        }];

        let result = exporter.export_session_json(&summary, &metrics);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("test_session"));
    }

    #[test]
    fn test_export_csv() {
        let exporter = ReportExporter::new();
        let metrics = vec![HistoricalMetrics {
            timestamp: 1_000_000,
            session_id: "test".to_string(),
            system: SystemMetricsSnapshot {
                cpu_usage: 50.0,
                memory_used: 8_000_000_000,
                memory_total: 16_000_000_000,
                gpu_usage: Some(60.0),
                gpu_memory_used: Some(4_000_000_000),
                network_upload: 1_000_000,
                network_download: 500_000,
            },
            obs: ObsStatusSnapshot::empty(),
        }];

        let result = exporter.export_session_csv(&metrics);
        assert!(result.is_ok());
        let csv = result.unwrap();
        assert!(csv.contains("timestamp,session_id"));
        assert!(csv.contains("50.00")); // CPU usage
    }

    #[test]
    fn test_generate_diagnostic_report() {
        let exporter = ReportExporter::new();
        let summary = create_test_session_summary();
        let problems = vec![create_test_problem()];

        let result = exporter.generate_diagnostic_report(&summary, &problems);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report.session.session_id, "test_session");
        assert_eq!(report.problems.len(), 1);
    }

    #[test]
    fn test_performance_evaluation() {
        let exporter = ReportExporter::new();
        let summary = create_test_session_summary();
        let problems = vec![];

        let eval = exporter.calculate_performance_evaluation(&summary, &problems);
        assert!(eval.overall_score > 0.0);
        assert!(eval.overall_score <= 100.0);
        assert!(eval.cpu_score > 0.0);
    }

    #[test]
    fn test_performance_evaluation_with_critical_problems() {
        let exporter = ReportExporter::new();
        let summary = create_test_session_summary();
        let problems = vec![
            ProblemReport {
                id: "crit-1".to_string(),
                category: ProblemCategory::Resource,
                severity: AlertSeverity::Critical,
                title: "Critical Problem".to_string(),
                description: "Critical issue".to_string(),
                suggested_actions: vec![],
                affected_metric: MetricType::CpuUsage,
                detected_at: 1_000_000,
            },
        ];

        let eval = exporter.calculate_performance_evaluation(&summary, &problems);
        // クリティカル問題がある場合はペナルティが適用される
        assert!(eval.overall_score < 100.0);
    }

    #[test]
    fn test_performance_evaluation_perfect_session() {
        let exporter = ReportExporter::new();
        let summary = SessionSummary {
            session_id: "perfect".to_string(),
            start_time: 1_000_000,
            end_time: 1_003_600,
            avg_cpu: 10.0, // 低いCPU使用率
            avg_gpu: 15.0, // 低いGPU使用率
            total_dropped_frames: 0, // ドロップフレームなし
            peak_bitrate: 6000,
            quality_score: 100.0,
        };

        let eval = exporter.calculate_performance_evaluation(&summary, &[]);
        assert!(eval.overall_score > 80.0);
        assert!(eval.stability_score == 100.0);
    }

    #[test]
    fn test_performance_evaluation_poor_session() {
        let exporter = ReportExporter::new();
        let summary = SessionSummary {
            session_id: "poor".to_string(),
            start_time: 1_000_000,
            end_time: 1_003_600,
            avg_cpu: 95.0, // 高いCPU使用率
            avg_gpu: 98.0, // 高いGPU使用率
            total_dropped_frames: 1000, // 多くのドロップフレーム
            peak_bitrate: 2000, // 低いビットレート
            quality_score: 20.0,
        };

        let eval = exporter.calculate_performance_evaluation(&summary, &[]);
        assert!(eval.overall_score < 50.0);
        assert!(eval.cpu_score < 20.0);
        assert!(eval.gpu_score < 20.0);
    }

    #[test]
    fn test_recommendations_summary_no_problems() {
        let exporter = ReportExporter::new();
        let summary = exporter.generate_recommendations_summary(&[]);
        assert!(summary.contains("問題は検出されませんでした"));
    }

    #[test]
    fn test_recommendations_summary_with_problems() {
        let exporter = ReportExporter::new();
        let problems = vec![
            create_test_problem(),
            ProblemReport {
                id: "crit-1".to_string(),
                category: ProblemCategory::Network,
                severity: AlertSeverity::Critical,
                title: "Network Problem".to_string(),
                description: "Network issue".to_string(),
                suggested_actions: vec![],
                affected_metric: MetricType::NetworkBandwidth,
                detected_at: 1_000_000,
            },
        ];

        let summary = exporter.generate_recommendations_summary(&problems);
        assert!(summary.contains("2個の問題"));
        assert!(summary.contains("クリティカル: 1"));
        assert!(summary.contains("警告: 1"));
    }

    #[test]
    fn test_csv_export_empty_data() {
        let exporter = ReportExporter::new();
        let result = exporter.export_session_csv(&[]);
        assert!(result.is_ok());
        let csv = result.unwrap();
        // ヘッダーのみ含まれる
        assert!(csv.contains("timestamp,session_id"));
        assert_eq!(csv.lines().count(), 1); // ヘッダー行のみ
    }

    #[test]
    fn test_csv_export_multiple_metrics() {
        let exporter = ReportExporter::new();
        let metrics = vec![
            HistoricalMetrics {
                timestamp: 1_000_000,
                session_id: "test1".to_string(),
                system: SystemMetricsSnapshot {
                    cpu_usage: 50.0,
                    memory_used: 8_000_000_000,
                    memory_total: 16_000_000_000,
                    gpu_usage: Some(60.0),
                    gpu_memory_used: Some(4_000_000_000),
                    network_upload: 1_000_000,
                    network_download: 500_000,
                },
                obs: ObsStatusSnapshot::empty(),
            },
            HistoricalMetrics {
                timestamp: 1_000_001,
                session_id: "test2".to_string(),
                system: SystemMetricsSnapshot {
                    cpu_usage: 55.0,
                    memory_used: 9_000_000_000,
                    memory_total: 16_000_000_000,
                    gpu_usage: None,
                    gpu_memory_used: None,
                    network_upload: 2_000_000,
                    network_download: 1_000_000,
                },
                obs: ObsStatusSnapshot::empty(),
            },
        ];

        let result = exporter.export_session_csv(&metrics);
        assert!(result.is_ok());
        let csv = result.unwrap();
        assert_eq!(csv.lines().count(), 3); // ヘッダー + 2データ行
    }

    #[test]
    fn test_json_export_structure() {
        let exporter = ReportExporter::new();
        let summary = create_test_session_summary();
        let metrics = vec![];

        let result = exporter.export_session_json(&summary, &metrics);
        assert!(result.is_ok());
        let json_str = result.unwrap();

        // JSONとしてパースできることを確認
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert!(parsed.get("version").is_some());
        assert!(parsed.get("exported_at").is_some());
        assert!(parsed.get("session").is_some());
        assert!(parsed.get("metrics").is_some());
    }

    #[test]
    fn test_system_info_retrieval() {
        let exporter = ReportExporter::new();
        let info = exporter.get_system_info();
        // OSは必ず取得できる
        assert!(!info.os.is_empty());
    }

    #[test]
    fn test_default_implementation() {
        let exporter = ReportExporter::default();
        let summary = create_test_session_summary();
        let result = exporter.export_session_csv(&[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_diagnostic_report_timestamps() {
        let exporter = ReportExporter::new();
        let summary = create_test_session_summary();
        let problems = vec![];

        let result = exporter.generate_diagnostic_report(&summary, &problems);
        assert!(result.is_ok());
        let report = result.unwrap();

        // タイムスタンプが有効範囲
        assert!(report.generated_at > 1_000_000);
        assert_eq!(report.session.duration_secs, 3600);
    }
}
