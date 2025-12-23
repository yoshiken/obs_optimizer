// セッション履歴コマンド
//
// メトリクス履歴とセッション情報を管理するTauriコマンド

use crate::error::AppError;
use crate::storage::metrics_history::{HistoricalMetrics, SessionSummary};
use serde::Deserialize;

/// メトリクス取得リクエスト
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMetricsRangeRequest {
    /// セッションID
    pub session_id: String,
    /// 開始時刻（Unixタイムスタンプ）
    pub from: i64,
    /// 終了時刻（Unixタイムスタンプ）
    pub to: i64,
}

/// セッション一覧を取得
///
/// # Returns
/// セッションサマリーのリスト
#[tauri::command]
pub async fn get_sessions() -> Result<Vec<SessionSummary>, AppError> {
    // TODO: 実際のデータベースから取得
    // 現在はダミーデータを返す
    let now = chrono::Utc::now().timestamp();

    Ok(vec![
        SessionSummary {
            session_id: "demo-session-1".to_string(),
            start_time: now - 7200, // 2時間前
            end_time: now - 3600,   // 1時間前
            avg_cpu: 45.5,
            avg_gpu: 62.3,
            total_dropped_frames: 15,
            peak_bitrate: 6200,
            quality_score: 85.5,
        },
        SessionSummary {
            session_id: "demo-session-2".to_string(),
            start_time: now - 14400, // 4時間前
            end_time: now - 10800,   // 3時間前
            avg_cpu: 52.1,
            avg_gpu: 68.7,
            total_dropped_frames: 42,
            peak_bitrate: 6500,
            quality_score: 78.2,
        },
    ])
}

/// 指定期間のメトリクスを取得
///
/// # Arguments
/// * `request` - セッションIDと期間の指定
///
/// # Returns
/// 履歴メトリクスのリスト
#[tauri::command]
pub async fn get_metrics_range(
    request: GetMetricsRangeRequest,
) -> Result<Vec<HistoricalMetrics>, AppError> {
    // TODO: 実際のデータベースから取得
    // 現在は空のリストを返す

    // パラメータを使用して警告を回避
    let _ = request.session_id;
    let _ = request.from;
    let _ = request.to;

    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_sessions() {
        let result = get_sessions().await;
        assert!(result.is_ok());

        let sessions = result.expect("Failed to get sessions in test");
        assert!(!sessions.is_empty());

        // 最初のセッションの検証
        let first = &sessions[0];
        assert_eq!(first.session_id, "demo-session-1");
        assert!(first.avg_cpu > 0.0);
        assert!(first.quality_score > 0.0);
        assert!(first.quality_score <= 100.0);
    }

    #[tokio::test]
    async fn test_get_metrics_range() {
        let request = GetMetricsRangeRequest {
            session_id: "test-session".to_string(),
            from: 1000000,
            to: 2000000,
        };

        let result = get_metrics_range(request).await;
        assert!(result.is_ok());

        let metrics = result.expect("Failed to get metrics range in test");
        // 現在は空のリストを返す実装
        assert!(metrics.is_empty());
    }
}
