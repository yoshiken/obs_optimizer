// 問題分析エンジン統合テスト
//
// ProblemAnalyzerの各分析機能を統合的にテストする。
// 複数の分析を組み合わせた総合分析の動作を検証する。

mod common;

use obs_optimizer_app_lib::testing::builders::SystemMetricsBuilder;
use obs_optimizer_app_lib::testing::fixtures::{
    critical_system_metrics, healthy_system_metrics, high_load_system_metrics,
    stable_bitrate_history, unstable_bitrate_history,
};

// 公開されたProblemAnalyzerをインポート
use obs_optimizer_app_lib::{ProblemAnalyzer, ProblemCategory};

/// テスト用のメトリクス履歴を生成
fn create_high_cpu_metrics_history() -> Vec<obs_optimizer_app_lib::testing::fixtures::SystemMetricsSnapshot> {
    use obs_optimizer_app_lib::testing::builders::build_increasing_cpu_metrics;
    build_increasing_cpu_metrics(10, 80.0, 95.0)
}

/// テスト用のメトリクス履歴を生成（正常）
fn create_healthy_metrics_history() -> Vec<obs_optimizer_app_lib::testing::fixtures::SystemMetricsSnapshot> {
    use obs_optimizer_app_lib::testing::fixtures::generate_metrics_history;
    generate_metrics_history(10, healthy_system_metrics())
}

// =============================================================================
// フレームドロップ分析テスト
// =============================================================================

#[test]
fn test_frame_drop_analysis_detects_cpu_overload() {
    // 高CPU使用率のメトリクス履歴で問題が検出されることを確認
    let metrics = create_high_cpu_metrics_history();

    // メトリクスの状態を確認
    let avg_cpu: f32 = metrics.iter().map(|m| m.cpu_usage).sum::<f32>() / metrics.len() as f32;
    assert!(avg_cpu > 85.0, "Test setup: CPU average should be > 85%");

    // ProblemAnalyzerで問題を検出
    let analyzer = ProblemAnalyzer::new();
    let problems = analyzer.analyze_frame_drops(&metrics);

    // CPU過負荷の問題が検出されることを確認
    assert!(!problems.is_empty(), "Should detect CPU overload problems");
    assert!(
        problems.iter().any(|p| p.category == ProblemCategory::Resource),
        "Should detect resource-related problems"
    );
    assert!(
        problems.iter().any(|p| p.title.contains("CPU")),
        "Should detect CPU-related problems"
    );
}

#[test]
fn test_frame_drop_analysis_no_issues_when_healthy() {
    let metrics = create_healthy_metrics_history();

    // 正常なメトリクスでは問題が検出されないことを確認
    let avg_cpu: f32 = metrics.iter().map(|m| m.cpu_usage).sum::<f32>() / metrics.len() as f32;
    assert!(avg_cpu < 85.0, "Test setup: CPU average should be < 85% for healthy metrics");

    // ProblemAnalyzerで分析
    let analyzer = ProblemAnalyzer::new();
    let problems = analyzer.analyze_frame_drops(&metrics);

    // 正常な状態では問題が検出されないことを確認
    assert!(problems.is_empty(), "Should not detect problems for healthy metrics");
}

// =============================================================================
// ビットレート分析テスト
// =============================================================================

#[test]
fn test_bitrate_analysis_detects_instability() {
    let bitrates = unstable_bitrate_history();

    // 変動係数を計算
    let avg = bitrates.iter().sum::<u64>() as f64 / bitrates.len() as f64;
    let variance = bitrates
        .iter()
        .map(|&b| {
            let diff = b as f64 - avg;
            diff * diff
        })
        .sum::<f64>()
        / bitrates.len() as f64;
    let std_dev = variance.sqrt();
    let cv = (std_dev / avg) * 100.0;

    // 変動係数が15%を超えていることを確認（問題検出の閾値）
    assert!(cv > 15.0, "Test setup: CV should be > 15% for unstable bitrates, got {:.2}%", cv);

    // ProblemAnalyzerでビットレート不安定を検出
    let analyzer = ProblemAnalyzer::new();
    let problems = analyzer.analyze_bitrate_issues(&bitrates, 6000);

    // ネットワーク関連の問題が検出されることを確認
    assert!(!problems.is_empty(), "Should detect bitrate instability");
    assert!(
        problems.iter().any(|p| p.category == ProblemCategory::Network),
        "Should detect network-related problems"
    );
}

#[test]
fn test_bitrate_analysis_stable_when_consistent() {
    let bitrates = stable_bitrate_history();

    // 変動係数を計算
    let avg = bitrates.iter().sum::<u64>() as f64 / bitrates.len() as f64;
    let variance = bitrates
        .iter()
        .map(|&b| {
            let diff = b as f64 - avg;
            diff * diff
        })
        .sum::<f64>()
        / bitrates.len() as f64;
    let std_dev = variance.sqrt();
    let cv = (std_dev / avg) * 100.0;

    // 変動係数が5%未満であることを確認
    assert!(cv < 5.0, "Test setup: CV should be < 5% for stable bitrates, got {:.2}%", cv);

    // ProblemAnalyzerで分析
    let analyzer = ProblemAnalyzer::new();
    let problems = analyzer.analyze_bitrate_issues(&bitrates, 6000);

    // 安定したビットレートでは変動に関する問題が検出されないことを確認
    assert!(
        !problems.iter().any(|p| p.title.contains("不安定")),
        "Should not detect instability for stable bitrates"
    );
}

// =============================================================================
// メトリクスビルダーテスト
// =============================================================================

#[test]
fn test_system_metrics_builder() {
    let metrics = SystemMetricsBuilder::new()
        .cpu_usage(75.0)
        .memory_percent(60.0)
        .gpu_usage(Some(50.0))
        .build();

    assert!((74.0..=76.0).contains(&metrics.cpu_usage), "CPU usage should be ~75%");
    assert!(metrics.gpu_usage.is_some(), "GPU usage should be set");

    let memory_percent = (metrics.memory_used as f64 / metrics.memory_total as f64) * 100.0;
    assert!((59.0..=61.0).contains(&memory_percent), "Memory usage should be ~60%");
}

#[test]
fn test_system_metrics_builder_no_gpu() {
    let metrics = SystemMetricsBuilder::new()
        .no_gpu()
        .build();

    assert!(metrics.gpu_usage.is_none(), "GPU usage should be None");
    assert!(metrics.gpu_memory_used.is_none(), "GPU memory should be None");
}

// =============================================================================
// フィクスチャテスト
// =============================================================================

#[test]
fn test_healthy_system_metrics_fixture() {
    let metrics = healthy_system_metrics();

    // 正常範囲内であることを確認
    assert!(metrics.cpu_usage < 50.0, "Healthy CPU should be < 50%");
    assert!(metrics.gpu_usage.unwrap_or(0.0) < 50.0, "Healthy GPU should be < 50%");

    let memory_percent = (metrics.memory_used as f64 / metrics.memory_total as f64) * 100.0;
    assert!(memory_percent < 50.0, "Healthy memory usage should be < 50%");
}

#[test]
fn test_high_load_system_metrics_fixture() {
    let metrics = high_load_system_metrics();

    // 高負荷であることを確認
    assert!(metrics.cpu_usage > 80.0, "High load CPU should be > 80%");
    assert!(metrics.gpu_usage.unwrap_or(0.0) > 80.0, "High load GPU should be > 80%");
}

#[test]
fn test_critical_system_metrics_fixture() {
    let metrics = critical_system_metrics();

    // クリティカルな状態であることを確認
    assert!(metrics.cpu_usage > 95.0, "Critical CPU should be > 95%");
    assert!(metrics.gpu_usage.unwrap_or(0.0) > 95.0, "Critical GPU should be > 95%");

    let memory_percent = (metrics.memory_used as f64 / metrics.memory_total as f64) * 100.0;
    assert!(memory_percent > 95.0, "Critical memory usage should be > 95%");
}

// =============================================================================
// 複合シナリオテスト
// =============================================================================

#[test]
fn test_scenario_streaming_session_degradation() {
    // 配信セッション中にパフォーマンスが徐々に悪化するシナリオ
    use obs_optimizer_app_lib::testing::builders::build_increasing_cpu_metrics;

    // 開始時は正常、終了時は高負荷
    let session_metrics = build_increasing_cpu_metrics(20, 40.0, 95.0);

    // 前半は正常範囲
    let first_half_avg: f32 = session_metrics[..10]
        .iter()
        .map(|m| m.cpu_usage)
        .sum::<f32>()
        / 10.0;
    assert!(first_half_avg < 70.0, "First half should be < 70% CPU");

    // 後半は高負荷
    let second_half_avg: f32 = session_metrics[10..]
        .iter()
        .map(|m| m.cpu_usage)
        .sum::<f32>()
        / 10.0;
    assert!(second_half_avg > 80.0, "Second half should be > 80% CPU");
}
