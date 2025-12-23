// カスタムテストアサーション
//
// ドメイン固有のアサーションマクロとヘルパー関数を提供する。
// 標準のassert!マクロを拡張し、より意味のあるエラーメッセージを出力する。

use crate::error::AppError;
use crate::services::alerts::AlertSeverity;
use crate::services::analyzer::ProblemReport;
use crate::storage::metrics_history::SystemMetricsSnapshot;

// =============================================================================
// メトリクス関連アサーション
// =============================================================================

/// CPU使用率が正常範囲内であることを確認
pub fn assert_cpu_usage_valid(usage: f32) {
    assert!(
        (0.0..=100.0).contains(&usage),
        "CPU usage should be between 0 and 100, got {}",
        usage
    );
}

/// メモリ使用量が妥当であることを確認
pub fn assert_memory_valid(used: u64, total: u64) {
    assert!(total > 0, "Total memory should be greater than 0");
    assert!(
        used <= total,
        "Used memory ({}) should be <= total memory ({})",
        used,
        total
    );
}

/// GPU使用率が正常範囲内であることを確認
pub fn assert_gpu_usage_valid(usage: Option<f32>) {
    if let Some(u) = usage {
        assert!(
            (0.0..=100.0).contains(&u),
            "GPU usage should be between 0 and 100, got {}",
            u
        );
    }
}

/// SystemMetricsSnapshotの全フィールドが妥当であることを確認
pub fn assert_metrics_snapshot_valid(metrics: &SystemMetricsSnapshot) {
    assert_cpu_usage_valid(metrics.cpu_usage);
    assert_memory_valid(metrics.memory_used, metrics.memory_total);
    assert_gpu_usage_valid(metrics.gpu_usage);
}

// =============================================================================
// AppError関連アサーション
// =============================================================================

/// エラーが特定のコードを持つことを確認
pub fn assert_error_code(error: &AppError, expected_code: &str) {
    assert_eq!(
        error.code(),
        expected_code,
        "Expected error code '{}', got '{}'",
        expected_code,
        error.code()
    );
}

/// エラーメッセージに特定の文字列が含まれることを確認
pub fn assert_error_contains(error: &AppError, expected_substring: &str) {
    assert!(
        error.message().contains(expected_substring),
        "Expected error message to contain '{}', got '{}'",
        expected_substring,
        error.message()
    );
}

/// Resultがエラーであり、特定のコードを持つことを確認
pub fn assert_result_error_code<T>(result: &Result<T, AppError>, expected_code: &str) {
    match result {
        Ok(_) => panic!("Expected error with code '{}', but got Ok", expected_code),
        Err(e) => assert_error_code(e, expected_code),
    }
}

// =============================================================================
// 問題レポート関連アサーション
// =============================================================================

/// 問題レポートリストに特定の重要度が含まれることを確認
pub fn assert_has_severity(reports: &[ProblemReport], severity: AlertSeverity) {
    assert!(
        reports.iter().any(|r| r.severity == severity),
        "Expected at least one problem with severity {:?}",
        severity
    );
}

/// 問題レポートリストにCriticalな問題が含まれることを確認
pub fn assert_has_critical_problem(reports: &[ProblemReport]) {
    assert_has_severity(reports, AlertSeverity::Critical);
}

/// 問題レポートリストにWarningな問題が含まれることを確認
pub fn assert_has_warning_problem(reports: &[ProblemReport]) {
    assert_has_severity(reports, AlertSeverity::Warning);
}

/// 問題レポートリストが空であることを確認
pub fn assert_no_problems(reports: &[ProblemReport]) {
    assert!(
        reports.is_empty(),
        "Expected no problems, but found {} problem(s): {:?}",
        reports.len(),
        reports.iter().map(|r| &r.title).collect::<Vec<_>>()
    );
}

/// 問題レポートの推奨アクションが空でないことを確認
pub fn assert_has_suggested_actions(report: &ProblemReport) {
    assert!(
        !report.suggested_actions.is_empty(),
        "Problem '{}' should have at least one suggested action",
        report.title
    );
}

// =============================================================================
// OBS設定関連アサーション
// =============================================================================

/// 解像度が妥当な値であることを確認
pub fn assert_resolution_valid(width: u32, height: u32) {
    assert!(width > 0 && width <= 7680, "Width should be between 1 and 7680, got {}", width);
    assert!(height > 0 && height <= 4320, "Height should be between 1 and 4320, got {}", height);
}

/// FPSが妥当な値であることを確認
pub fn assert_fps_valid(fps: u32) {
    assert!(
        fps > 0 && fps <= 240,
        "FPS should be between 1 and 240, got {}",
        fps
    );
}

/// ビットレートが妥当な値であることを確認
pub fn assert_bitrate_valid(bitrate_kbps: u32) {
    assert!(
        bitrate_kbps > 0 && bitrate_kbps <= 100000,
        "Bitrate should be between 1 and 100000 kbps, got {}",
        bitrate_kbps
    );
}

// =============================================================================
// 統合テスト用ユーティリティ
// =============================================================================

/// 非同期テストのタイムアウト付き実行
#[allow(dead_code)]
pub async fn with_timeout<F, T>(duration_ms: u64, future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    tokio::time::timeout(
        std::time::Duration::from_millis(duration_ms),
        future,
    )
    .await
    .expect("Test timed out")
}

/// テスト用の一時的な遅延
#[allow(dead_code)]
pub async fn test_delay(ms: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
}

// =============================================================================
// マクロ定義
// =============================================================================

/// 特定のエラーコードを持つResultを期待するマクロ
#[macro_export]
macro_rules! assert_err_code {
    ($result:expr, $code:expr) => {
        match &$result {
            Ok(_) => panic!("Expected error with code '{}', but got Ok", $code),
            Err(e) => {
                assert_eq!(
                    e.code(),
                    $code,
                    "Expected error code '{}', got '{}'",
                    $code,
                    e.code()
                );
            }
        }
    };
}

/// Okの結果を期待し、値を返すマクロ
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        match $result {
            Ok(v) => v,
            Err(e) => panic!("Expected Ok, but got Err: {}", e),
        }
    };
}

/// 範囲内の値を期待するマクロ
#[macro_export]
macro_rules! assert_in_range {
    ($value:expr, $min:expr, $max:expr) => {
        assert!(
            $value >= $min && $value <= $max,
            "Expected value in range [{}, {}], got {}",
            $min,
            $max,
            $value
        );
    };
}
