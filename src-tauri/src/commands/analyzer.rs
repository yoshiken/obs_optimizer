// 問題分析コマンド
//
// システムメトリクスとOBS統計を分析して問題を検出するTauriコマンド

use crate::error::AppError;
use crate::services::analyzer::{ProblemAnalyzer, ProblemReport};
use crate::services::system::system_monitor_service;
use crate::storage::metrics_history::SystemMetricsSnapshot;
use serde::{Deserialize, Serialize};

/// 問題分析リクエスト
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeProblemsRequest {
    /// エンコーダータイプ
    pub encoder_type: String,
    /// 目標ビットレート（kbps）
    pub target_bitrate: u64,
}

/// 問題分析結果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeProblemsResponse {
    /// 検出された問題
    pub problems: Vec<ProblemReport>,
    /// 総合評価スコア（0-100）
    pub overall_score: f64,
}

/// 現在の問題を分析
///
/// システムメトリクスとOBS状態を分析して、パフォーマンス問題を検出する
///
/// # Arguments
/// * `request` - 分析リクエスト（エンコーダータイプ、目標ビットレート）
///
/// # Returns
/// 検出された問題のリスト
#[tauri::command]
pub async fn analyze_problems(request: AnalyzeProblemsRequest) -> Result<AnalyzeProblemsResponse, AppError> {
    let analyzer = ProblemAnalyzer::new();
    let service = system_monitor_service();

    // 現在のシステムメトリクスを取得
    let cpu_usage = service.get_cpu_usage()?;
    let (memory_used, memory_total) = service.get_memory_info()?;
    let gpu_metrics = service.get_gpu_metrics()?;
    let network_metrics = service.get_network_metrics()?;

    // スナップショットを作成
    let current_snapshot = SystemMetricsSnapshot::from_metrics(
        cpu_usage,
        memory_used,
        memory_total,
        gpu_metrics.as_ref(),
        &network_metrics,
    );

    // 履歴データ（現在は単一スナップショット）
    let metrics_history = vec![current_snapshot];

    // ビットレート履歴（ダミーデータ - 将来的には実データを使用）
    let bitrate_history: Vec<u64> = vec![request.target_bitrate];

    // 総合分析を実行
    let problems = analyzer.analyze_comprehensive(
        &metrics_history,
        &bitrate_history,
        request.target_bitrate,
        &request.encoder_type,
    );

    // スコアを計算（問題の数と重要度から）
    let overall_score = calculate_overall_score(&problems);

    Ok(AnalyzeProblemsResponse {
        problems,
        overall_score,
    })
}

/// 問題履歴を取得
///
/// 過去に検出された問題の履歴を取得する
///
/// # Arguments
/// * `limit` - 取得する問題の最大数
///
/// # Returns
/// 問題履歴のリスト
#[tauri::command]
pub async fn get_problem_history(limit: usize) -> Result<Vec<ProblemReport>, AppError> {
    // TODO: 実際の履歴データベースから取得
    // 現在は空のリストを返す
    let _ = limit; // 未使用警告を回避
    Ok(Vec::new())
}

/// スコアを計算
///
/// 問題の数と重要度から総合スコアを算出
fn calculate_overall_score(problems: &[ProblemReport]) -> f64 {
    if problems.is_empty() {
        return 100.0;
    }

    let mut score: f64 = 100.0;

    for problem in problems {
        let penalty = match problem.severity {
            crate::services::alerts::AlertSeverity::Critical => 20.0,
            crate::services::alerts::AlertSeverity::Warning => 10.0,
            crate::services::alerts::AlertSeverity::Info => 5.0,
            crate::services::alerts::AlertSeverity::Tips => 2.0,
        };
        score -= penalty;
    }

    score.clamp(0.0, 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_overall_score_no_problems() {
        let problems = vec![];
        let score = calculate_overall_score(&problems);
        assert_eq!(score, 100.0);
    }

    #[test]
    fn test_calculate_overall_score_with_problems() {
        use crate::services::alerts::{AlertSeverity, MetricType};
        use crate::services::analyzer::ProblemCategory;

        let problems = vec![
            ProblemReport {
                id: "test-1".to_string(),
                category: ProblemCategory::Resource,
                severity: AlertSeverity::Critical,
                title: "Test".to_string(),
                description: "Test".to_string(),
                suggested_actions: vec![],
                affected_metric: MetricType::CpuUsage,
                detected_at: 0,
            },
            ProblemReport {
                id: "test-2".to_string(),
                category: ProblemCategory::Network,
                severity: AlertSeverity::Warning,
                title: "Test".to_string(),
                description: "Test".to_string(),
                suggested_actions: vec![],
                affected_metric: MetricType::NetworkBandwidth,
                detected_at: 0,
            },
        ];

        let score = calculate_overall_score(&problems);
        assert_eq!(score, 70.0); // 100 - 20 - 10
    }
}
