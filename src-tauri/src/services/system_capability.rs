// システム能力統合評価サービス
//
// GPU、CPU、メモリの各ティアを統合し、
// システム全体の配信能力を評価する

use super::gpu_detection::{CpuTier, EffectiveTier, MemoryTier};
use serde::{Deserialize, Serialize};

/// システム全体の総合ティア
///
/// GPU、CPU、メモリのティアを統合した最終評価
/// 最も低いコンポーネントのスコアに合わせる
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OverallTier {
    /// 最高性能 - 1440p60/4K対応、AV1余裕
    Ultra,
    /// 高性能 - 1080p60余裕、1440p可能
    High,
    /// 中位 - 1080p60可能
    Medium,
    /// 下位 - 720p60または1080p30
    Low,
    /// 最低 - 720p30推奨
    Minimal,
}

impl OverallTier {
    /// スコア（1-6）からティアを判定
    pub fn from_score(score: u8) -> Self {
        match score {
            6 => Self::Ultra,
            5 => Self::High,
            4 => Self::High,
            3 => Self::Medium,
            2 => Self::Low,
            _ => Self::Minimal,
        }
    }

    /// ティアのスコアを取得
    pub fn score(&self) -> u8 {
        match self {
            Self::Ultra => 6,
            Self::High => 5,
            Self::Medium => 4,
            Self::Low => 3,
            Self::Minimal => 2,
        }
    }

    /// ティアの表示ラベルを取得
    pub fn display_label(&self) -> &'static str {
        match self {
            Self::Ultra => "最高性能",
            Self::High => "高性能",
            Self::Medium => "中位",
            Self::Low => "下位",
            Self::Minimal => "エントリー",
        }
    }
}

/// ボトルネック要因
///
/// システムの配信能力を制限しているコンポーネントを特定
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BottleneckFactor {
    /// ボトルネックなし（バランスの取れた構成）
    None,
    /// GPU性能がボトルネック
    Gpu,
    /// CPU性能がボトルネック
    Cpu,
    /// メモリ容量がボトルネック
    Memory,
}

impl BottleneckFactor {
    /// ボトルネックの表示ラベルを取得
    pub fn display_label(&self) -> &'static str {
        match self {
            Self::None => "なし",
            Self::Gpu => "GPU",
            Self::Cpu => "CPU",
            Self::Memory => "メモリ",
        }
    }
}

/// システム能力統合評価
///
/// GPU、CPU、メモリの個別評価と、それらを統合した総合評価を保持
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemCapability {
    /// GPU性能ティア
    pub gpu_tier: EffectiveTier,
    /// GPU名
    pub gpu_name: String,
    /// CPUティア
    pub cpu_tier: CpuTier,
    /// CPUコア数
    pub cpu_cores: usize,
    /// メモリティア
    pub memory_tier: MemoryTier,
    /// メモリ容量（GB）
    pub memory_gb: f64,
    /// 総合ティア（最も低いコンポーネントに合わせる）
    pub overall_tier: OverallTier,
    /// ボトルネック要因
    pub bottleneck: BottleneckFactor,
    /// 能力の説明文
    pub description: String,
}

impl SystemCapability {
    /// 新しいSystemCapabilityを作成
    ///
    /// 各コンポーネントのティアから総合評価を計算する
    pub fn new(
        gpu_tier: EffectiveTier,
        gpu_name: String,
        cpu_tier: CpuTier,
        cpu_cores: usize,
        memory_tier: MemoryTier,
        memory_gb: f64,
    ) -> Self {
        let gpu_score = gpu_tier.score();
        let cpu_score = cpu_tier.score();
        let mem_score = memory_tier.score();

        // 最も低いスコアで総合評価を決定
        let min_score = gpu_score.min(cpu_score).min(mem_score);
        let overall_tier = OverallTier::from_score(min_score);

        // ボトルネック検出
        let bottleneck = Self::detect_bottleneck(gpu_score, cpu_score, mem_score);

        // 説明文生成
        let description = Self::generate_description(overall_tier, bottleneck);

        Self {
            gpu_tier,
            gpu_name,
            cpu_tier,
            cpu_cores,
            memory_tier,
            memory_gb,
            overall_tier,
            bottleneck,
            description,
        }
    }

    /// ボトルネック要因を検出
    ///
    /// 最も低いスコアのコンポーネントがボトルネック
    /// 複数が同点の場合はボトルネックなしとみなす
    fn detect_bottleneck(gpu: u8, cpu: u8, mem: u8) -> BottleneckFactor {
        let min = gpu.min(cpu).min(mem);

        // 2番目に低いスコアを取得
        let second = if min == gpu {
            cpu.min(mem)
        } else if min == cpu {
            gpu.min(mem)
        } else {
            gpu.min(cpu)
        };

        // 最小値が2番目より明確に低い場合のみボトルネック
        if min < second {
            if min == gpu {
                BottleneckFactor::Gpu
            } else if min == cpu {
                BottleneckFactor::Cpu
            } else {
                BottleneckFactor::Memory
            }
        } else {
            BottleneckFactor::None
        }
    }

    /// システム能力の説明文を生成
    fn generate_description(tier: OverallTier, bottleneck: BottleneckFactor) -> String {
        let (resolution, fps) = match tier {
            OverallTier::Ultra => ("1440p", 60),
            OverallTier::High => ("1080p", 60),
            OverallTier::Medium => ("1080p", 60),
            OverallTier::Low => ("720p", 60),
            OverallTier::Minimal => ("720p", 30),
        };

        let quality = match tier {
            OverallTier::Ultra => "余裕を持って",
            OverallTier::High => "高品質で",
            OverallTier::Medium => "安定して",
            OverallTier::Low => "基本的な",
            OverallTier::Minimal => "軽量設定で",
        };

        let bottleneck_note = match bottleneck {
            BottleneckFactor::None => String::new(),
            BottleneckFactor::Gpu => "（GPU性能が制限要因）".to_string(),
            BottleneckFactor::Cpu => "（CPU性能が制限要因）".to_string(),
            BottleneckFactor::Memory => "（メモリ容量が制限要因）".to_string(),
        };

        format!(
            "{} {}fps配信が{}可能です{}",
            resolution, fps, quality, bottleneck_note
        )
    }

    /// GPUの能力リストを取得
    pub fn gpu_capabilities(&self) -> Vec<&'static str> {
        match self.gpu_tier {
            EffectiveTier::TierS => vec!["AV1", "HEVC", "H.264", "1440p+", "4K対応"],
            EffectiveTier::TierA => vec!["AV1", "HEVC", "H.264", "1080p60余裕"],
            EffectiveTier::TierB => vec!["HEVC", "H.264", "1080p60可能"],
            EffectiveTier::TierC => vec!["HEVC", "H.264", "1080p可能"],
            EffectiveTier::TierD => vec!["H.264", "720p推奨"],
            EffectiveTier::TierE => vec!["H.264", "720p30"],
        }
    }

    /// CPUの能力リストを取得
    pub fn cpu_capabilities(&self) -> Vec<&'static str> {
        match self.cpu_tier {
            CpuTier::HighEnd => vec!["x264 medium可", "マルチタスク余裕"],
            CpuTier::UpperMiddle => vec!["x264 veryfast可", "ゲーム+配信可能"],
            CpuTier::Middle => vec!["NVENC推奨", "軽いゲーム+配信"],
            CpuTier::Entry => vec!["NVENC必須", "単一タスク推奨"],
        }
    }

    /// メモリの能力リストを取得
    pub fn memory_capabilities(&self) -> Vec<&'static str> {
        match self.memory_tier {
            MemoryTier::Abundant => vec!["1440p/4K対応", "複数アプリ余裕"],
            MemoryTier::Adequate => vec!["1080p60標準", "ゲーム+配信+Discord"],
            MemoryTier::Standard => vec!["1080p30可能", "ブラウザソース制限"],
            MemoryTier::Entry => vec!["720p推奨", "最小構成"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overall_tier_from_score() {
        assert_eq!(OverallTier::from_score(6), OverallTier::Ultra);
        assert_eq!(OverallTier::from_score(5), OverallTier::High);
        assert_eq!(OverallTier::from_score(4), OverallTier::High);
        assert_eq!(OverallTier::from_score(3), OverallTier::Medium);
        assert_eq!(OverallTier::from_score(2), OverallTier::Low);
        assert_eq!(OverallTier::from_score(1), OverallTier::Minimal);
        assert_eq!(OverallTier::from_score(0), OverallTier::Minimal);
    }

    #[test]
    fn test_bottleneck_detection_gpu() {
        // GPU: TierD(2), CPU: HighEnd(5), Memory: Abundant(5)
        let cap = SystemCapability::new(
            EffectiveTier::TierD,
            "GTX 1050".to_string(),
            CpuTier::HighEnd,
            12,
            MemoryTier::Abundant,
            32.0,
        );
        assert_eq!(cap.bottleneck, BottleneckFactor::Gpu);
        assert_eq!(cap.overall_tier, OverallTier::Low);
    }

    #[test]
    fn test_bottleneck_detection_cpu() {
        // GPU: TierA(5), CPU: Entry(2), Memory: Adequate(4)
        let cap = SystemCapability::new(
            EffectiveTier::TierA,
            "RTX 4070".to_string(),
            CpuTier::Entry,
            4,
            MemoryTier::Adequate,
            16.0,
        );
        assert_eq!(cap.bottleneck, BottleneckFactor::Cpu);
        assert_eq!(cap.overall_tier, OverallTier::Low);
    }

    #[test]
    fn test_bottleneck_detection_memory() {
        // GPU: TierA(5), CPU: HighEnd(5), Memory: Entry(2)
        let cap = SystemCapability::new(
            EffectiveTier::TierA,
            "RTX 4070".to_string(),
            CpuTier::HighEnd,
            12,
            MemoryTier::Entry,
            4.0,
        );
        assert_eq!(cap.bottleneck, BottleneckFactor::Memory);
        assert_eq!(cap.overall_tier, OverallTier::Low);
    }

    #[test]
    fn test_balanced_system_no_bottleneck() {
        // GPU: TierA(5), CPU: HighEnd(5), Memory: Adequate(4)
        // CPUとGPUが同点、メモリが少し低いがスコア差1なので許容
        let cap = SystemCapability::new(
            EffectiveTier::TierB,
            "RTX 4060".to_string(),
            CpuTier::UpperMiddle,
            8,
            MemoryTier::Adequate,
            16.0,
        );
        // GPU(4), CPU(4), Mem(4) → 全て同点 → ボトルネックなし
        assert_eq!(cap.bottleneck, BottleneckFactor::None);
        assert_eq!(cap.overall_tier, OverallTier::High);
    }

    #[test]
    fn test_high_end_system() {
        let cap = SystemCapability::new(
            EffectiveTier::TierS,
            "RTX 4090".to_string(),
            CpuTier::HighEnd,
            16,
            MemoryTier::Abundant,
            64.0,
        );
        // GPU(6), CPU(5), Mem(5) → GPU > CPU = Mem → ボトルネックなし
        assert_eq!(cap.overall_tier, OverallTier::High);
        assert!(cap.description.contains("1080p"));
    }

    #[test]
    fn test_ultra_tier_system() {
        let cap = SystemCapability::new(
            EffectiveTier::TierS,
            "RTX 5090".to_string(),
            CpuTier::HighEnd,
            24,
            MemoryTier::Abundant,
            128.0,
        );
        // GPU(6), CPU(5), Mem(5) → min=5 → High
        // TierSだけでは Ultra にならない（CPUとメモリもスコア6が必要）
        assert_eq!(cap.overall_tier, OverallTier::High);
    }

    #[test]
    fn test_capabilities_lists() {
        let cap = SystemCapability::new(
            EffectiveTier::TierS,
            "RTX 4090".to_string(),
            CpuTier::HighEnd,
            16,
            MemoryTier::Abundant,
            64.0,
        );

        let gpu_caps = cap.gpu_capabilities();
        assert!(gpu_caps.contains(&"AV1"));
        assert!(gpu_caps.contains(&"4K対応"));

        let cpu_caps = cap.cpu_capabilities();
        assert!(cpu_caps.contains(&"x264 medium可"));

        let mem_caps = cap.memory_capabilities();
        assert!(mem_caps.contains(&"1440p/4K対応"));
    }

    #[test]
    fn test_description_generation() {
        let cap = SystemCapability::new(
            EffectiveTier::TierD,
            "GTX 1050".to_string(),
            CpuTier::HighEnd,
            12,
            MemoryTier::Abundant,
            32.0,
        );
        assert!(cap.description.contains("GPU性能が制限要因"));
    }

    #[test]
    fn test_display_labels() {
        assert_eq!(OverallTier::Ultra.display_label(), "最高性能");
        assert_eq!(OverallTier::High.display_label(), "高性能");
        assert_eq!(OverallTier::Medium.display_label(), "中位");
        assert_eq!(BottleneckFactor::Gpu.display_label(), "GPU");
        assert_eq!(BottleneckFactor::None.display_label(), "なし");
    }
}
