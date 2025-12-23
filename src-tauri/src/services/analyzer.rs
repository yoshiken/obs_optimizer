// 問題分析エンジン
//
// システムメトリクスとOBS統計を分析し、パフォーマンス問題を検出する
// フレームドロップ、ビットレート変動、リソース不足などを診断

use crate::services::alerts::{AlertSeverity, MetricType};
use crate::storage::metrics_history::SystemMetricsSnapshot;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// AppErrorは将来の拡張用にコメントアウト
// use crate::error::AppError;

/// 問題カテゴリー
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProblemCategory {
    /// エンコーディング関連
    Encoding,
    /// ネットワーク関連
    Network,
    /// リソース不足
    Resource,
    /// 設定問題
    Settings,
}

/// 問題レポート
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemReport {
    /// 一意識別子
    pub id: String,
    /// カテゴリー
    pub category: ProblemCategory,
    /// 重要度
    pub severity: AlertSeverity,
    /// タイトル
    pub title: String,
    /// 詳細説明
    pub description: String,
    /// 推奨される対処方法
    pub suggested_actions: Vec<String>,
    /// 影響を受けるメトリクス
    pub affected_metric: MetricType,
    /// 検出時刻（UNIX epoch秒）
    pub detected_at: i64,
}

/// 問題分析エンジン
pub struct ProblemAnalyzer;

impl ProblemAnalyzer {
    /// 新しいアナライザーを作成
    pub fn new() -> Self {
        Self
    }

    /// フレームドロップの原因分析
    ///
    /// # Arguments
    /// * `metrics_history` - メトリクス履歴
    ///
    /// # Returns
    /// 検出された問題のリスト
    pub fn analyze_frame_drops(&self, metrics_history: &[SystemMetricsSnapshot]) -> Vec<ProblemReport> {
        let mut problems = Vec::new();

        if metrics_history.is_empty() {
            return problems;
        }

        // CPU使用率の平均を計算
        let avg_cpu = metrics_history.iter()
            .map(|m| m.cpu_usage as f64)
            .sum::<f64>() / metrics_history.len() as f64;

        // GPU使用率の平均を計算
        let avg_gpu = metrics_history.iter()
            .filter_map(|m| m.gpu_usage.map(|u| u as f64))
            .sum::<f64>() / metrics_history.len() as f64;

        // CPU過負荷の検出
        if avg_cpu > 85.0 {
            problems.push(ProblemReport {
                id: Uuid::new_v4().to_string(),
                category: ProblemCategory::Resource,
                severity: AlertSeverity::Critical,
                title: "CPU負荷が高すぎます".to_string(),
                description: format!(
                    "平均CPU使用率が {:.1}% に達しています。エンコーダー設定を軽量化する必要があります。",
                    avg_cpu
                ),
                suggested_actions: vec![
                    "エンコーダープリセットを「faster」または「veryfast」に変更".to_string(),
                    "配信解像度を下げる（例: 1080p → 720p）".to_string(),
                    "フレームレートを下げる（例: 60fps → 30fps）".to_string(),
                    "他のアプリケーションを終了してCPUリソースを確保".to_string(),
                ],
                affected_metric: MetricType::CpuUsage,
                detected_at: chrono::Utc::now().timestamp(),
            });
        }

        // GPU過負荷の検出
        if avg_gpu > 90.0 {
            problems.push(ProblemReport {
                id: Uuid::new_v4().to_string(),
                category: ProblemCategory::Encoding,
                severity: AlertSeverity::Critical,
                title: "GPU負荷が高すぎます".to_string(),
                description: format!(
                    "平均GPU使用率が {:.1}% に達しています。GPUエンコーダーが過負荷状態です。",
                    avg_gpu
                ),
                suggested_actions: vec![
                    "配信解像度を下げる".to_string(),
                    "ビットレートを下げる".to_string(),
                    "ゲームのグラフィック設定を下げる".to_string(),
                ],
                affected_metric: MetricType::GpuUsage,
                detected_at: chrono::Utc::now().timestamp(),
            });
        }

        // メモリ使用率の確認
        let avg_memory_usage = metrics_history.iter()
            .map(|m| (m.memory_used as f64 / m.memory_total as f64) * 100.0)
            .sum::<f64>() / metrics_history.len() as f64;

        if avg_memory_usage > 90.0 {
            problems.push(ProblemReport {
                id: Uuid::new_v4().to_string(),
                category: ProblemCategory::Resource,
                severity: AlertSeverity::Warning,
                title: "メモリ使用率が高い".to_string(),
                description: format!(
                    "平均メモリ使用率が {:.1}% です。メモリ不足によりパフォーマンスが低下する可能性があります。",
                    avg_memory_usage
                ),
                suggested_actions: vec![
                    "不要なアプリケーションを終了".to_string(),
                    "ブラウザのタブを減らす".to_string(),
                    "OBSのシーンを簡素化（ソース数を減らす）".to_string(),
                ],
                affected_metric: MetricType::MemoryUsage,
                detected_at: chrono::Utc::now().timestamp(),
            });
        }

        problems
    }

    /// ビットレート変動の原因分析
    ///
    /// # Arguments
    /// * `bitrate_history` - ビットレート履歴（kbps）
    /// * `target_bitrate` - 目標ビットレート（kbps）
    pub fn analyze_bitrate_issues(
        &self,
        bitrate_history: &[u64],
        target_bitrate: u64,
    ) -> Vec<ProblemReport> {
        let mut problems = Vec::new();

        if bitrate_history.len() < 10 {
            return problems; // データ不足
        }

        // ビットレートの変動係数を計算
        let avg = bitrate_history.iter().sum::<u64>() as f64 / bitrate_history.len() as f64;
        let variance = bitrate_history.iter()
            .map(|&b| {
                let diff = b as f64 - avg;
                diff * diff
            })
            .sum::<f64>() / bitrate_history.len() as f64;
        let std_dev = variance.sqrt();
        let cv = (std_dev / avg) * 100.0; // 変動係数（%）

        // 変動が大きい場合
        if cv > 15.0 {
            problems.push(ProblemReport {
                id: Uuid::new_v4().to_string(),
                category: ProblemCategory::Network,
                severity: AlertSeverity::Warning,
                title: "ビットレートが不安定".to_string(),
                description: format!(
                    "ビットレートの変動が大きいです（変動係数: {:.1}%）。ネットワークが不安定な可能性があります。",
                    cv
                ),
                suggested_actions: vec![
                    "有線LAN接続に変更（Wi-Fiを使用している場合）".to_string(),
                    "他のネットワーク利用を制限（動画視聴、ダウンロードなど）".to_string(),
                    "ビットレートを下げて安定性を優先".to_string(),
                    "レート制御を「CBR」に変更".to_string(),
                ],
                affected_metric: MetricType::NetworkBandwidth,
                detected_at: chrono::Utc::now().timestamp(),
            });
        }

        // 目標に達していない場合
        if avg < target_bitrate as f64 * 0.8 {
            problems.push(ProblemReport {
                id: Uuid::new_v4().to_string(),
                category: ProblemCategory::Network,
                severity: AlertSeverity::Critical,
                title: "帯域不足".to_string(),
                description: format!(
                    "実際のビットレート（{:.0} kbps）が目標（{} kbps）を大きく下回っています。",
                    avg, target_bitrate
                ),
                suggested_actions: vec![
                    "目標ビットレートを下げる".to_string(),
                    "インターネット回線を確認".to_string(),
                    "配信サーバーを変更（近い場所のサーバーを選択）".to_string(),
                ],
                affected_metric: MetricType::NetworkBandwidth,
                detected_at: chrono::Utc::now().timestamp(),
            });
        }

        problems
    }

    /// エンコーダー負荷分析
    ///
    /// # Arguments
    /// * `encoder_usage` - エンコーダー使用率（%）
    /// * `encoder_type` - エンコーダータイプ（"nvenc", "x264", etc.）
    pub fn analyze_encoder_load(
        &self,
        encoder_usage: f32,
        encoder_type: &str,
    ) -> Vec<ProblemReport> {
        let mut problems = Vec::new();

        // ハードウェアエンコーダーの過負荷
        if (encoder_type.contains("nvenc") || encoder_type.contains("qsv") || encoder_type.contains("vce"))
            && encoder_usage > 95.0
        {
            problems.push(ProblemReport {
                id: Uuid::new_v4().to_string(),
                category: ProblemCategory::Encoding,
                severity: AlertSeverity::Critical,
                title: "ハードウェアエンコーダーが過負荷".to_string(),
                description: format!(
                    "{}エンコーダーの使用率が {:.1}% に達しています。",
                    encoder_type, encoder_usage
                ),
                suggested_actions: vec![
                    "解像度を下げる".to_string(),
                    "フレームレートを下げる".to_string(),
                    "ビットレートを下げる".to_string(),
                    "2パスエンコードを無効化".to_string(),
                ],
                affected_metric: MetricType::GpuUsage,
                detected_at: chrono::Utc::now().timestamp(),
            });
        }

        // ソフトウェアエンコーダーの過負荷
        if encoder_type.contains("x264") && encoder_usage > 85.0 {
            problems.push(ProblemReport {
                id: Uuid::new_v4().to_string(),
                category: ProblemCategory::Encoding,
                severity: AlertSeverity::Critical,
                title: "ソフトウェアエンコーダーが過負荷".to_string(),
                description: format!(
                    "x264エンコーダーのCPU使用率が {:.1}% に達しています。",
                    encoder_usage
                ),
                suggested_actions: vec![
                    "プリセットを「faster」または「veryfast」に変更".to_string(),
                    "ハードウェアエンコーダー（NVENC/QuickSync）に変更".to_string(),
                    "解像度またはフレームレートを下げる".to_string(),
                ],
                affected_metric: MetricType::CpuUsage,
                detected_at: chrono::Utc::now().timestamp(),
            });
        }

        problems
    }

    /// 総合的な問題分析
    ///
    /// すべての分析を統合して実行
    pub fn analyze_comprehensive(
        &self,
        metrics_history: &[SystemMetricsSnapshot],
        bitrate_history: &[u64],
        target_bitrate: u64,
        encoder_type: &str,
    ) -> Vec<ProblemReport> {
        let mut all_problems = Vec::new();

        // フレームドロップ分析
        all_problems.extend(self.analyze_frame_drops(metrics_history));

        // ビットレート分析
        all_problems.extend(self.analyze_bitrate_issues(bitrate_history, target_bitrate));

        // エンコーダー負荷分析
        if let Some(latest) = metrics_history.last() {
            let encoder_usage = if encoder_type.contains("nvenc") || encoder_type.contains("qsv") {
                latest.gpu_usage.unwrap_or(0.0)
            } else {
                latest.cpu_usage
            };
            all_problems.extend(self.analyze_encoder_load(encoder_usage, encoder_type));
        }

        // 重要度順にソート
        all_problems.sort_by(|a, b| {
            let severity_order = |s: &AlertSeverity| match s {
                AlertSeverity::Critical => 0,
                AlertSeverity::Warning => 1,
                AlertSeverity::Info => 2,
                AlertSeverity::Tips => 3,
            };
            severity_order(&a.severity).cmp(&severity_order(&b.severity))
        });

        all_problems
    }
}

impl Default for ProblemAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metrics(cpu: f32, gpu: f32, memory_percent: f32) -> SystemMetricsSnapshot {
        let total_memory = 16_000_000_000u64;
        let used_memory = (total_memory as f32 * memory_percent / 100.0) as u64;

        SystemMetricsSnapshot {
            cpu_usage: cpu,
            memory_used: used_memory,
            memory_total: total_memory,
            gpu_usage: Some(gpu),
            gpu_memory_used: Some(4_000_000_000),
            network_upload: 1_000_000,
            network_download: 500_000,
        }
    }

    #[test]
    fn test_cpu_overload_detection() {
        let analyzer = ProblemAnalyzer::new();
        let metrics = vec![
            create_test_metrics(90.0, 50.0, 60.0),
            create_test_metrics(92.0, 50.0, 60.0),
            create_test_metrics(88.0, 50.0, 60.0),
        ];

        let problems = analyzer.analyze_frame_drops(&metrics);
        assert!(!problems.is_empty());
        assert!(problems.iter().any(|p| p.category == ProblemCategory::Resource));
    }

    #[test]
    fn test_bitrate_instability_detection() {
        let analyzer = ProblemAnalyzer::new();
        let bitrates = vec![6000, 5500, 4000, 6500, 3500, 6000, 4500, 5000, 3000, 6000];

        let problems = analyzer.analyze_bitrate_issues(&bitrates, 6000);
        assert!(!problems.is_empty());
        assert!(problems.iter().any(|p| p.category == ProblemCategory::Network));
    }

    #[test]
    fn test_encoder_overload_detection() {
        let analyzer = ProblemAnalyzer::new();
        let problems = analyzer.analyze_encoder_load(96.0, "nvenc_h264");

        assert!(!problems.is_empty());
        assert!(problems[0].severity == AlertSeverity::Critical);
    }

    #[test]
    fn test_no_problems_when_healthy() {
        let analyzer = ProblemAnalyzer::new();
        let metrics = vec![
            create_test_metrics(50.0, 60.0, 50.0),
            create_test_metrics(52.0, 62.0, 51.0),
        ];

        let problems = analyzer.analyze_frame_drops(&metrics);
        assert!(problems.is_empty());
    }

    // === 追加のエッジケーステスト ===

    #[test]
    fn test_empty_metrics_history() {
        let analyzer = ProblemAnalyzer::new();
        let empty_metrics: Vec<SystemMetricsSnapshot> = vec![];

        // 空の履歴でもクラッシュしない
        let problems = analyzer.analyze_frame_drops(&empty_metrics);
        assert!(problems.is_empty(), "空の履歴では問題なし");

        let bitrate_problems = analyzer.analyze_bitrate_issues(&[], 6000);
        assert!(bitrate_problems.is_empty(), "空のビットレート履歴では問題なし");
    }

    #[test]
    fn test_single_metric_entry() {
        let analyzer = ProblemAnalyzer::new();
        let single = vec![create_test_metrics(95.0, 95.0, 95.0)];

        // 1つだけのエントリでも処理可能
        let problems = analyzer.analyze_frame_drops(&single);
        assert!(!problems.is_empty(), "1つのエントリでも問題検出");
    }

    #[test]
    fn test_cpu_boundary_85_percent() {
        let analyzer = ProblemAnalyzer::new();

        // ちょうど85.0%（境界値）
        let at_boundary = vec![
            create_test_metrics(85.0, 50.0, 60.0),
            create_test_metrics(85.0, 50.0, 60.0),
        ];
        let problems_at = analyzer.analyze_frame_drops(&at_boundary);
        assert!(problems_at.is_empty(), "85.0%ではまだ問題なし");

        // 85.1%（境界値を超える）
        let above_boundary = vec![
            create_test_metrics(85.1, 50.0, 60.0),
            create_test_metrics(85.1, 50.0, 60.0),
        ];
        let problems_above = analyzer.analyze_frame_drops(&above_boundary);
        assert!(!problems_above.is_empty(), "85.1%では問題検出");
    }

    #[test]
    fn test_gpu_boundary_90_percent() {
        let analyzer = ProblemAnalyzer::new();

        // 90.0%（境界値の直下）
        let below = vec![
            create_test_metrics(50.0, 90.0, 50.0),
            create_test_metrics(50.0, 90.0, 50.0),
        ];
        let problems_below = analyzer.analyze_frame_drops(&below);
        assert!(problems_below.is_empty(), "90.0%ではまだ問題なし");

        // 90.1%（境界値を超える）
        let above = vec![
            create_test_metrics(50.0, 90.1, 50.0),
            create_test_metrics(50.0, 90.1, 50.0),
        ];
        let problems_above = analyzer.analyze_frame_drops(&above);
        assert!(!problems_above.is_empty(), "90.1%では問題検出");
    }

    #[test]
    fn test_memory_boundary_90_percent() {
        let analyzer = ProblemAnalyzer::new();

        // 89.9%（境界値の直下）
        let below = vec![
            create_test_metrics(50.0, 50.0, 89.9),
            create_test_metrics(50.0, 50.0, 89.9),
        ];
        let problems_below = analyzer.analyze_frame_drops(&below);
        assert!(problems_below.is_empty(), "89.9%では問題なし");

        // 90.1%（境界値を超える）
        let above = vec![
            create_test_metrics(50.0, 50.0, 90.1),
            create_test_metrics(50.0, 50.0, 90.1),
        ];
        let problems_above = analyzer.analyze_frame_drops(&above);
        assert!(!problems_above.is_empty(), "90.1%では問題検出");
    }

    #[test]
    fn test_extreme_values_100_percent() {
        let analyzer = ProblemAnalyzer::new();
        let maxed_out = vec![
            create_test_metrics(100.0, 100.0, 100.0),
            create_test_metrics(100.0, 100.0, 100.0),
        ];

        let problems = analyzer.analyze_frame_drops(&maxed_out);
        assert!(!problems.is_empty(), "100%使用率では問題検出");
        assert!(problems.len() >= 2, "CPU、GPU、メモリで複数の問題検出");
    }

    #[test]
    fn test_extreme_values_zero_percent() {
        let analyzer = ProblemAnalyzer::new();
        let zero = vec![
            create_test_metrics(0.0, 0.0, 0.0),
            create_test_metrics(0.0, 0.0, 0.0),
        ];

        let problems = analyzer.analyze_frame_drops(&zero);
        assert!(problems.is_empty(), "0%使用率では問題なし");
    }

    #[test]
    fn test_gpu_usage_none() {
        let analyzer = ProblemAnalyzer::new();
        let mut metrics = vec![
            create_test_metrics(50.0, 50.0, 50.0),
            create_test_metrics(50.0, 50.0, 50.0),
        ];

        // GPU情報をNoneに設定
        for m in &mut metrics {
            m.gpu_usage = None;
            m.gpu_memory_used = None;
        }

        // GPUなしでもクラッシュしない
        let problems = analyzer.analyze_frame_drops(&metrics);
        assert!(problems.is_empty(), "GPU情報がなくても処理可能");
    }

    #[test]
    fn test_bitrate_insufficient_data() {
        let analyzer = ProblemAnalyzer::new();

        // 10未満のデータ（データ不足）
        let few_data = vec![6000, 5900, 6100];
        let problems = analyzer.analyze_bitrate_issues(&few_data, 6000);
        assert!(problems.is_empty(), "データ不足では分析しない");
    }

    #[test]
    fn test_bitrate_stable() {
        let analyzer = ProblemAnalyzer::new();

        // 非常に安定したビットレート
        let stable = vec![6000; 20];
        let problems = analyzer.analyze_bitrate_issues(&stable, 6000);
        assert!(problems.is_empty(), "安定したビットレートでは問題なし");
    }

    #[test]
    fn test_bitrate_high_variation() {
        let analyzer = ProblemAnalyzer::new();

        // 変動が激しいビットレート
        let unstable = vec![
            6000, 3000, 8000, 2000, 7000, 4000, 9000, 1000, 5000, 6500,
            6000, 3000, 8000, 2000, 7000, 4000, 9000, 1000, 5000, 6500,
        ];
        let problems = analyzer.analyze_bitrate_issues(&unstable, 6000);
        assert!(!problems.is_empty(), "変動が激しい場合は問題検出");
        assert!(
            problems.iter().any(|p| p.title.contains("不安定")),
            "不安定に関する問題が含まれる"
        );
    }

    #[test]
    fn test_bitrate_below_target() {
        let analyzer = ProblemAnalyzer::new();

        // 目標の80%未満（帯域不足）
        let low = vec![4000; 20]; // 目標6000の約67%
        let problems = analyzer.analyze_bitrate_issues(&low, 6000);
        assert!(!problems.is_empty(), "目標未達では問題検出");
        assert!(
            problems.iter().any(|p| p.title.contains("帯域不足")),
            "帯域不足の問題が含まれる"
        );
    }

    #[test]
    fn test_bitrate_boundary_80_percent() {
        let analyzer = ProblemAnalyzer::new();

        // ちょうど80%
        let at_80 = vec![4800; 20]; // 6000 * 0.8
        let problems_at = analyzer.analyze_bitrate_issues(&at_80, 6000);
        // 80%ちょうどでは問題検出されないはず
        assert!(
            !problems_at.iter().any(|p| p.title.contains("帯域不足")),
            "80%ちょうどでは帯域不足にならない"
        );

        // 79.9%（境界値を下回る）
        let below_80 = vec![4794; 20]; // 6000 * 0.799
        let problems_below = analyzer.analyze_bitrate_issues(&below_80, 6000);
        assert!(
            problems_below.iter().any(|p| p.title.contains("帯域不足")),
            "80%未満では帯域不足検出"
        );
    }

    #[test]
    fn test_encoder_nvenc_overload() {
        let analyzer = ProblemAnalyzer::new();

        let problems = analyzer.analyze_encoder_load(96.0, "nvenc_h264");
        assert!(!problems.is_empty(), "NVENC過負荷検出");
        assert_eq!(problems[0].severity, AlertSeverity::Critical);
        assert!(problems[0].title.contains("ハードウェアエンコーダー"));
    }

    #[test]
    fn test_encoder_qsv_overload() {
        let analyzer = ProblemAnalyzer::new();

        let problems = analyzer.analyze_encoder_load(97.0, "obs_qsv11");
        assert!(!problems.is_empty(), "QuickSync過負荷検出");
        assert!(problems[0].affected_metric == MetricType::GpuUsage);
    }

    #[test]
    fn test_encoder_vce_overload() {
        let analyzer = ProblemAnalyzer::new();

        let problems = analyzer.analyze_encoder_load(98.0, "amd_vce");
        assert!(!problems.is_empty(), "VCE過負荷検出");
    }

    #[test]
    fn test_encoder_x264_overload() {
        let analyzer = ProblemAnalyzer::new();

        let problems = analyzer.analyze_encoder_load(90.0, "obs_x264");
        assert!(!problems.is_empty(), "x264過負荷検出");
        assert!(problems[0].title.contains("ソフトウェアエンコーダー"));
        assert!(problems[0].affected_metric == MetricType::CpuUsage);
    }

    #[test]
    fn test_encoder_below_threshold() {
        let analyzer = ProblemAnalyzer::new();

        // NVENC 94%（95%未満）
        let nvenc_ok = analyzer.analyze_encoder_load(94.0, "nvenc_h264");
        assert!(nvenc_ok.is_empty(), "95%未満では問題なし");

        // x264 84%（85%未満）
        let x264_ok = analyzer.analyze_encoder_load(84.0, "obs_x264");
        assert!(x264_ok.is_empty(), "85%未満では問題なし");
    }

    #[test]
    fn test_comprehensive_analysis() {
        let analyzer = ProblemAnalyzer::new();

        let metrics = vec![
            create_test_metrics(95.0, 95.0, 95.0),
            create_test_metrics(96.0, 96.0, 96.0),
        ];
        let bitrates = vec![4000; 20];

        let all_problems = analyzer.analyze_comprehensive(
            &metrics,
            &bitrates,
            6000,
            "nvenc_h264",
        );

        // 複数の問題が検出される
        assert!(!all_problems.is_empty(), "総合分析で複数の問題検出");

        // 重要度順にソートされている
        if all_problems.len() > 1 {
            for i in 0..all_problems.len() - 1 {
                let current_severity = match all_problems[i].severity {
                    AlertSeverity::Critical => 0,
                    AlertSeverity::Warning => 1,
                    AlertSeverity::Info => 2,
                    AlertSeverity::Tips => 3,
                };
                let next_severity = match all_problems[i + 1].severity {
                    AlertSeverity::Critical => 0,
                    AlertSeverity::Warning => 1,
                    AlertSeverity::Info => 2,
                    AlertSeverity::Tips => 3,
                };
                assert!(
                    current_severity <= next_severity,
                    "重要度順にソートされている"
                );
            }
        }
    }

    #[test]
    fn test_problem_report_fields() {
        let analyzer = ProblemAnalyzer::new();
        let metrics = vec![
            create_test_metrics(95.0, 50.0, 50.0),
            create_test_metrics(96.0, 50.0, 50.0),
        ];

        let problems = analyzer.analyze_frame_drops(&metrics);
        assert!(!problems.is_empty());

        let problem = &problems[0];
        assert!(!problem.id.is_empty(), "IDが設定されている");
        assert!(!problem.title.is_empty(), "タイトルが設定されている");
        assert!(!problem.description.is_empty(), "説明が設定されている");
        assert!(!problem.suggested_actions.is_empty(), "推奨アクションが設定されている");
        assert!(problem.detected_at > 0, "検出時刻が設定されている");
    }

    #[test]
    fn test_suggested_actions_not_empty() {
        let analyzer = ProblemAnalyzer::new();

        // 各問題タイプで推奨アクションが提供されることを確認
        let cpu_problems = analyzer.analyze_frame_drops(&vec![
            create_test_metrics(95.0, 50.0, 50.0),
            create_test_metrics(95.0, 50.0, 50.0),
        ]);
        if let Some(p) = cpu_problems.first() {
            assert!(p.suggested_actions.len() >= 2, "CPU問題には複数の推奨アクションがある");
        }

        let bitrate_problems = analyzer.analyze_bitrate_issues(&vec![4000; 20], 6000);
        if let Some(p) = bitrate_problems.first() {
            assert!(p.suggested_actions.len() >= 2, "ビットレート問題には複数の推奨アクションがある");
        }

        let encoder_problems = analyzer.analyze_encoder_load(96.0, "nvenc_h264");
        if let Some(p) = encoder_problems.first() {
            assert!(p.suggested_actions.len() >= 2, "エンコーダー問題には複数の推奨アクションがある");
        }
    }
}
