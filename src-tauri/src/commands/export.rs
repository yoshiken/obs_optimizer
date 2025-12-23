// データエクスポートコマンド
//
// セッションデータと診断レポートをエクスポートするTauriコマンド

use crate::error::AppError;
use crate::services::exporter::{ReportExporter, DiagnosticReport};
use crate::services::analyzer::ProblemAnalyzer;
use crate::storage::metrics_history::{SessionSummary, HistoricalMetrics};
use serde::Deserialize;

/// エクスポートリクエスト
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSessionRequest {
    /// セッションID
    pub session_id: String,
}

/// JSONエクスポートレスポンス
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportJsonResponse {
    /// JSONデータ
    pub data: String,
    /// ファイル名
    pub filename: String,
}

/// CSVエクスポートレスポンス
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportCsvResponse {
    /// CSVデータ
    pub data: String,
    /// ファイル名
    pub filename: String,
}

/// セッションをJSON形式でエクスポート
///
/// # Arguments
/// * `request` - エクスポートリクエスト
///
/// # Returns
/// JSON文字列とファイル名
#[tauri::command]
pub async fn export_session_json(request: ExportSessionRequest) -> Result<ExportJsonResponse, AppError> {
    let exporter = ReportExporter::new();

    // TODO: 実際のデータベースから取得
    // 現在はダミーデータを使用
    let session_summary = create_dummy_session_summary(&request.session_id);
    let metrics_history = create_dummy_metrics_history(&request.session_id);

    let json_data = exporter.export_session_json(&session_summary, &metrics_history)?;

    let filename = format!("obs_session_{}.json", request.session_id);

    Ok(ExportJsonResponse {
        data: json_data,
        filename,
    })
}

/// セッションをCSV形式でエクスポート
///
/// # Arguments
/// * `request` - エクスポートリクエスト
///
/// # Returns
/// CSV文字列とファイル名
#[tauri::command]
pub async fn export_session_csv(request: ExportSessionRequest) -> Result<ExportCsvResponse, AppError> {
    let exporter = ReportExporter::new();

    // TODO: 実際のデータベースから取得
    // 現在はダミーデータを使用
    let metrics_history = create_dummy_metrics_history(&request.session_id);

    let csv_data = exporter.export_session_csv(&metrics_history)?;

    let filename = format!("obs_session_{}.csv", request.session_id);

    Ok(ExportCsvResponse {
        data: csv_data,
        filename,
    })
}

/// 診断レポートを生成
///
/// # Returns
/// 診断レポート
#[tauri::command]
pub async fn generate_diagnostic_report() -> Result<DiagnosticReport, AppError> {
    let exporter = ReportExporter::new();
    let analyzer = ProblemAnalyzer::new();

    // TODO: 実際のデータを使用
    // 現在はダミーデータを使用
    let session_summary = create_dummy_session_summary("current");

    // ダミーの問題を生成（テスト用）
    let metrics_history = create_dummy_metrics_history("current");
    let problems = analyzer.analyze_frame_drops(&metrics_history.iter()
        .map(|m| m.system.clone())
        .collect::<Vec<_>>());

    let report = exporter.generate_diagnostic_report(&session_summary, &problems)?;

    Ok(report)
}

// ============================================================
// ダミーデータ生成（テスト用）
// ============================================================

fn create_dummy_session_summary(session_id: &str) -> SessionSummary {
    let now = chrono::Utc::now().timestamp();
    SessionSummary {
        session_id: session_id.to_string(),
        start_time: now - 3600,
        end_time: now,
        avg_cpu: 55.0,
        avg_gpu: 65.0,
        total_dropped_frames: 25,
        peak_bitrate: 6000,
        quality_score: 80.0,
    }
}

fn create_dummy_metrics_history(session_id: &str) -> Vec<HistoricalMetrics> {
    use crate::storage::metrics_history::{SystemMetricsSnapshot, ObsStatusSnapshot};

    let now = chrono::Utc::now().timestamp();

    vec![
        HistoricalMetrics {
            timestamp: now - 3600,
            session_id: session_id.to_string(),
            system: SystemMetricsSnapshot {
                cpu_usage: 50.0,
                memory_used: 8_000_000_000,
                memory_total: 16_000_000_000,
                gpu_usage: Some(60.0),
                gpu_memory_used: Some(4_000_000_000),
                network_upload: 800_000,
                network_download: 200_000,
            },
            obs: ObsStatusSnapshot {
                streaming: true,
                recording: false,
                fps: Some(60.0),
                render_dropped_frames: Some(10),
                output_dropped_frames: Some(5),
                stream_bitrate: Some(6000),
            },
        },
        HistoricalMetrics {
            timestamp: now - 1800,
            session_id: session_id.to_string(),
            system: SystemMetricsSnapshot {
                cpu_usage: 55.0,
                memory_used: 8_500_000_000,
                memory_total: 16_000_000_000,
                gpu_usage: Some(65.0),
                gpu_memory_used: Some(4_200_000_000),
                network_upload: 820_000,
                network_download: 220_000,
            },
            obs: ObsStatusSnapshot {
                streaming: true,
                recording: false,
                fps: Some(60.0),
                render_dropped_frames: Some(15),
                output_dropped_frames: Some(8),
                stream_bitrate: Some(5800),
            },
        },
        HistoricalMetrics {
            timestamp: now,
            session_id: session_id.to_string(),
            system: SystemMetricsSnapshot {
                cpu_usage: 60.0,
                memory_used: 9_000_000_000,
                memory_total: 16_000_000_000,
                gpu_usage: Some(70.0),
                gpu_memory_used: Some(4_500_000_000),
                network_upload: 850_000,
                network_download: 250_000,
            },
            obs: ObsStatusSnapshot {
                streaming: true,
                recording: false,
                fps: Some(60.0),
                render_dropped_frames: Some(20),
                output_dropped_frames: Some(12),
                stream_bitrate: Some(6100),
            },
        },
    ]
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_export_session_json() {
        let request = ExportSessionRequest {
            session_id: "test_session".to_string(),
        };

        let result = export_session_json(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("test_session"));
        assert!(response.filename.ends_with(".json"));
    }

    #[tokio::test]
    async fn test_export_session_csv() {
        let request = ExportSessionRequest {
            session_id: "test_session".to_string(),
        };

        let result = export_session_csv(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.contains("timestamp"));
        assert!(response.filename.ends_with(".csv"));
    }

    #[tokio::test]
    async fn test_generate_diagnostic_report() {
        let result = generate_diagnostic_report().await;
        assert!(result.is_ok());

        let report = result.unwrap();
        assert_eq!(report.session.session_id, "current");
        assert!(report.performance.overall_score >= 0.0);
        assert!(report.performance.overall_score <= 100.0);
    }
}
