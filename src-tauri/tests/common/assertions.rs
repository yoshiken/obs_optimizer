// 統合テスト用カスタムアサーション
//
// 統合テストで頻繁に使用されるアサーションパターンを提供する。

// =============================================================================
// 基本的なアサーション
// =============================================================================

/// 値が指定範囲内であることを確認
pub fn assert_in_range<T: PartialOrd + std::fmt::Display>(value: T, min: T, max: T) {
    assert!(
        value >= min && value <= max,
        "Expected value in range [{}, {}], got {}",
        min,
        max,
        value
    );
}

/// 文字列が空でないことを確認
pub fn assert_not_empty(s: &str) {
    assert!(!s.is_empty(), "Expected non-empty string");
}

/// ベクターが空でないことを確認
pub fn assert_vec_not_empty<T>(v: &[T]) {
    assert!(!v.is_empty(), "Expected non-empty vector");
}

/// ベクターの長さが期待値と一致することを確認
pub fn assert_vec_len<T>(v: &[T], expected_len: usize) {
    assert_eq!(
        v.len(),
        expected_len,
        "Expected vector length {}, got {}",
        expected_len,
        v.len()
    );
}

// =============================================================================
// Result関連アサーション
// =============================================================================

/// Resultがエラーであることを確認し、エラーを返す
pub fn assert_is_err<T, E: std::fmt::Debug>(result: Result<T, E>) -> E {
    match result {
        Ok(_) => panic!("Expected Err, but got Ok"),
        Err(e) => e,
    }
}

/// Resultがエラーであり、エラーメッセージに特定の文字列が含まれることを確認
pub fn assert_err_contains<T, E: std::fmt::Debug + std::fmt::Display>(
    result: Result<T, E>,
    expected_substring: &str,
) {
    let err = assert_is_err(result);
    let err_str = format!("{}", err);
    assert!(
        err_str.contains(expected_substring),
        "Expected error to contain '{}', got '{}'",
        expected_substring,
        err_str
    );
}

// =============================================================================
// 統計関連アサーション
// =============================================================================

/// 値の変動係数（CV）が閾値未満であることを確認
pub fn assert_cv_below(values: &[f64], max_cv_percent: f64) {
    if values.is_empty() {
        return;
    }

    let avg = values.iter().sum::<f64>() / values.len() as f64;
    if avg == 0.0 {
        return;
    }

    let variance = values
        .iter()
        .map(|&v| {
            let diff = v - avg;
            diff * diff
        })
        .sum::<f64>()
        / values.len() as f64;
    let std_dev = variance.sqrt();
    let cv = (std_dev / avg) * 100.0;

    assert!(
        cv < max_cv_percent,
        "Expected CV < {}%, got {:.2}%",
        max_cv_percent,
        cv
    );
}

/// 値の変動係数（CV）が閾値以上であることを確認
pub fn assert_cv_above(values: &[f64], min_cv_percent: f64) {
    if values.is_empty() {
        return;
    }

    let avg = values.iter().sum::<f64>() / values.len() as f64;
    if avg == 0.0 {
        return;
    }

    let variance = values
        .iter()
        .map(|&v| {
            let diff = v - avg;
            diff * diff
        })
        .sum::<f64>()
        / values.len() as f64;
    let std_dev = variance.sqrt();
    let cv = (std_dev / avg) * 100.0;

    assert!(
        cv >= min_cv_percent,
        "Expected CV >= {}%, got {:.2}%",
        min_cv_percent,
        cv
    );
}

/// 値の平均が期待範囲内であることを確認
pub fn assert_avg_in_range(values: &[f64], min: f64, max: f64) {
    if values.is_empty() {
        panic!("Cannot calculate average of empty slice");
    }

    let avg = values.iter().sum::<f64>() / values.len() as f64;
    assert!(
        avg >= min && avg <= max,
        "Expected average in range [{}, {}], got {}",
        min,
        max,
        avg
    );
}

// =============================================================================
// 時間関連アサーション
// =============================================================================

/// 処理時間が閾値未満であることを確認するためのタイマー
pub struct AssertionTimer {
    start: std::time::Instant,
    max_duration_ms: u64,
    operation_name: String,
}

impl AssertionTimer {
    /// 新しいタイマーを開始
    pub fn start(operation_name: &str, max_duration_ms: u64) -> Self {
        Self {
            start: std::time::Instant::now(),
            max_duration_ms,
            operation_name: operation_name.to_string(),
        }
    }

    /// タイマーを停止し、処理時間が閾値未満であることを確認
    pub fn stop_and_assert(self) {
        let elapsed = self.start.elapsed().as_millis() as u64;
        assert!(
            elapsed < self.max_duration_ms,
            "{} took {}ms, expected < {}ms",
            self.operation_name,
            elapsed,
            self.max_duration_ms
        );
    }

    /// 経過時間を取得（ミリ秒）
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
}

// =============================================================================
// ファイル関連アサーション
// =============================================================================

/// ファイルが存在することを確認
pub fn assert_file_exists(path: &std::path::Path) {
    assert!(
        path.exists(),
        "Expected file to exist: {}",
        path.display()
    );
}

/// ファイルが存在しないことを確認
pub fn assert_file_not_exists(path: &std::path::Path) {
    assert!(
        !path.exists(),
        "Expected file to not exist: {}",
        path.display()
    );
}

/// ディレクトリが存在することを確認
pub fn assert_dir_exists(path: &std::path::Path) {
    assert!(
        path.exists() && path.is_dir(),
        "Expected directory to exist: {}",
        path.display()
    );
}

/// ファイルの内容が期待値と一致することを確認
pub fn assert_file_content(path: &std::path::Path, expected_content: &str) {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Failed to read file: {}", path.display()));
    assert_eq!(
        content, expected_content,
        "File content mismatch for {}",
        path.display()
    );
}

/// ファイルの内容に特定の文字列が含まれることを確認
pub fn assert_file_contains(path: &std::path::Path, expected_substring: &str) {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Failed to read file: {}", path.display()));
    assert!(
        content.contains(expected_substring),
        "Expected file {} to contain '{}', but it doesn't",
        path.display(),
        expected_substring
    );
}
