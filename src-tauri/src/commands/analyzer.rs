// 問題分析コマンド
//
// システムメトリクスとOBS統計を分析して問題を検出するTauriコマンド

use crate::error::AppError;
use crate::services::analyzer::{ProblemAnalyzer, ProblemReport};
use crate::services::system::system_monitor_service;
use crate::services::optimizer::RecommendationEngine;
use crate::storage::metrics_history::SystemMetricsSnapshot;
use crate::monitor::get_memory_info;
use crate::obs::get_obs_settings;
use crate::storage::config::load_config;
use crate::commands::utils::get_hardware_info;
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

/// OBS設定分析結果（analyze_settings用）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisResult {
    /// 品質スコア（0-100）
    pub quality_score: u8,
    /// 検出された問題の数
    pub issue_count: usize,
    /// 推奨設定変更リスト
    pub recommendations: Vec<ObsSetting>,
    /// システム環境情報
    pub system_info: SystemInfo,
    /// 分析日時（Unixタイムスタンプ）
    pub analyzed_at: i64,
}

/// OBS設定項目
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsSetting {
    /// 設定項目キー
    pub key: String,
    /// 表示名
    pub display_name: String,
    /// 現在の値
    pub current_value: serde_json::Value,
    /// 推奨値
    pub recommended_value: serde_json::Value,
    /// 変更理由
    pub reason: String,
    /// 優先度
    pub priority: String, // "critical" | "recommended" | "optional"
}

/// システム環境情報
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    /// CPUモデル名
    pub cpu_model: String,
    /// GPUモデル名
    pub gpu_model: Option<String>,
    /// 総メモリ容量（MB）
    pub total_memory_mb: u64,
    /// 利用可能メモリ（MB）
    pub available_memory_mb: u64,
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

/// OBS設定を分析して推奨事項を返す
///
/// # Returns
/// 分析結果（品質スコア、推奨設定、システム情報）
#[tauri::command]
pub async fn analyze_settings() -> Result<AnalysisResult, AppError> {
    // 現在のOBS設定を取得
    let obs_settings = get_obs_settings().await?;

    // システム情報を取得
    let hardware_info = get_hardware_info().await;

    // アプリケーション設定を取得
    let app_config = load_config()?;

    // 推奨設定を計算
    let recommendations = RecommendationEngine::calculate_recommendations(
        &hardware_info,
        &obs_settings,
        app_config.streaming_mode.platform,
        app_config.streaming_mode.style,
        app_config.streaming_mode.network_speed_mbps,
    );

    // 推奨事項リストを構築
    let mut recommendation_list = Vec::new();

    // 解像度の推奨
    if obs_settings.video.output_width != recommendations.video.output_width
        || obs_settings.video.output_height != recommendations.video.output_height {
        recommendation_list.push(ObsSetting {
            key: "video.resolution".to_string(),
            display_name: "出力解像度".to_string(),
            current_value: serde_json::json!(format!(
                "{}x{}",
                obs_settings.video.output_width,
                obs_settings.video.output_height
            )),
            recommended_value: serde_json::json!(format!(
                "{}x{}",
                recommendations.video.output_width,
                recommendations.video.output_height
            )),
            reason: "現在の設定はシステム性能に最適化されていません".to_string(),
            priority: "recommended".to_string(),
        });
    }

    // FPSの推奨
    let current_fps = obs_settings.video.fps() as u32;
    if current_fps != recommendations.video.fps {
        recommendation_list.push(ObsSetting {
            key: "video.fps".to_string(),
            display_name: "FPS".to_string(),
            current_value: serde_json::json!(current_fps),
            recommended_value: serde_json::json!(recommendations.video.fps),
            reason: "配信スタイルに適したFPSに変更することを推奨します".to_string(),
            priority: if current_fps > recommendations.video.fps { "recommended" } else { "optional" }.to_string(),
        });
    }

    // ビットレートの推奨
    let bitrate_diff = (obs_settings.output.bitrate_kbps as i32
        - recommendations.output.bitrate_kbps as i32).abs();
    if bitrate_diff > 500 {
        recommendation_list.push(ObsSetting {
            key: "output.bitrate".to_string(),
            display_name: "ビットレート".to_string(),
            current_value: serde_json::json!(obs_settings.output.bitrate_kbps),
            recommended_value: serde_json::json!(recommendations.output.bitrate_kbps),
            reason: format!(
                "ネットワーク速度とプラットフォームに最適化されたビットレートは{}kbpsです",
                recommendations.output.bitrate_kbps
            ),
            priority: if bitrate_diff > 2000 { "critical" } else { "recommended" }.to_string(),
        });
    }

    // エンコーダーの推奨
    if obs_settings.output.encoder != recommendations.output.encoder {
        let priority = if !obs_settings.output.is_hardware_encoder() && hardware_info.gpu.is_some() {
            "critical"
        } else {
            "recommended"
        };

        recommendation_list.push(ObsSetting {
            key: "output.encoder".to_string(),
            display_name: "エンコーダー".to_string(),
            current_value: serde_json::json!(obs_settings.output.encoder),
            recommended_value: serde_json::json!(recommendations.output.encoder),
            reason: "ハードウェアエンコーダーの使用を推奨します（CPU負荷軽減のため）".to_string(),
            priority: priority.to_string(),
        });
    }

    // システム情報を構築
    let (memory_used, memory_total) = get_memory_info().unwrap_or((0, 8_000_000_000));
    let system_info = SystemInfo {
        cpu_model: hardware_info.cpu_name.clone(),
        gpu_model: hardware_info.gpu.as_ref().map(|g| g.name.clone()),
        total_memory_mb: memory_total / 1_048_576,
        available_memory_mb: (memory_total - memory_used) / 1_048_576,
    };

    // 品質スコアを取得
    let quality_score = recommendations.overall_score;

    Ok(AnalysisResult {
        quality_score,
        issue_count: recommendation_list.len(),
        recommendations: recommendation_list,
        system_info,
        analyzed_at: chrono::Utc::now().timestamp(),
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
