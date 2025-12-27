// GPU世代判定サービス
//
// GPU名から世代を判定し、エンコーダー能力を提供する
// 判定ロジックは変更しやすいようテーブル駆動で実装

use serde::{Deserialize, Serialize};

/// GPU世代の分類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GpuGeneration {
    /// NVIDIA Pascal世代（GTX 10シリーズ）
    NvidiaPascal,
    /// NVIDIA Turing世代（GTX 16/RTX 20シリーズ）
    NvidiaTuring,
    /// NVIDIA Ampere世代（RTX 30シリーズ）
    NvidiaAmpere,
    /// NVIDIA Ada Lovelace世代（RTX 40シリーズ）
    NvidiaAda,
    /// AMD RX 6000シリーズ（VCN 3.0）
    AmdVcn3,
    /// AMD RX 7000シリーズ（VCN 4.0）
    AmdVcn4,
    /// Intel Arc GPU
    IntelArc,
    /// Intel QuickSync（内蔵GPU）
    IntelQuickSync,
    /// 世代不明のGPU
    Unknown,
    /// GPUなし
    None,
}

/// CPUのティア分類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CpuTier {
    /// エントリークラス（4コア未満）
    Entry,
    /// ミドルクラス（4-7コア）
    Middle,
    /// アッパーミドル（8-11コア）
    UpperMiddle,
    /// ハイエンド（12コア以上）
    HighEnd,
}

/// GPUの性能グレード（同一世代内での性能差）
///
/// xx90, xx80などの型番による分類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GpuGrade {
    /// フラグシップ（xx90, Titan, x900等）
    Flagship,
    /// ハイエンド（xx80, x800等）
    HighEnd,
    /// アッパーミドル（xx70, x700等）
    UpperMid,
    /// ミドル（xx60, x600等）
    Mid,
    /// エントリー（xx50, x500等）
    Entry,
    /// 不明
    Unknown,
}

/// 統合ティア（世代×グレードの総合評価）
///
/// 世代の新しさとグレードを組み合わせた最終的な性能ティア
/// 例: RTX 3090（旧世代フラグシップ）≒ RTX 4070（新世代アッパーミドル）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EffectiveTier {
    /// 最高性能（RTX 4090/4080, RX 7900等）
    TierS,
    /// 高性能（RTX 4070, 3090/3080, RX 7800等）
    TierA,
    /// 中上位（RTX 4060, 3070, 2080, RX 7700/6900等）
    TierB,
    /// 中位（RTX 3060, 2070, 1080, RX 6800等）
    TierC,
    /// 下位（RTX 3050, GTX 1660, RX 6600等）
    TierD,
    /// 最低（GTX 1050, RX 6500, 内蔵GPU等）
    TierE,
}

// 後方互換性のためのエイリアス（テストで使用）
#[allow(dead_code)]
pub type GpuTier = GpuGrade;

/// GPU世代ごとのエンコーダー能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuEncoderCapability {
    /// 世代
    pub generation: GpuGeneration,
    /// H.264サポート
    pub h264: bool,
    /// HEVCサポート
    pub hevc: bool,
    /// AV1サポート
    pub av1: bool,
    /// Bフレームサポート
    pub b_frames: bool,
    /// x264換算の品質等価（veryfast/fast/medium/slow/veryslow）
    pub quality_equivalent: &'static str,
    /// 推奨NVENCプリセット（P1-P7）
    pub recommended_preset: &'static str,
}

/// GPU世代判定パターンテーブル
///
/// 変更しやすさのため、判定ルールをテーブルで管理
struct GpuDetectionPattern {
    /// 検索キーワード（大文字小文字を区別しない）
    keywords: &'static [&'static str],
    /// 除外キーワード（これが含まれる場合は除外）
    exclude_keywords: &'static [&'static str],
    /// 判定される世代
    generation: GpuGeneration,
}

/// GPU判定パターン定義テーブル
const GPU_PATTERNS: &[GpuDetectionPattern] = &[
    // NVIDIA Blackwell (RTX 50シリーズ) - Ada以上の能力を持つためAdaとして扱う
    GpuDetectionPattern {
        keywords: &["rtx 50", "rtx50", "5090", "5080", "5070", "5060"],
        exclude_keywords: &[],
        generation: GpuGeneration::NvidiaAda, // Blackwellは Ada 以上の能力
    },
    // NVIDIA Ada Lovelace (RTX 40シリーズ)
    GpuDetectionPattern {
        keywords: &["rtx 40", "rtx40", "4090", "4080", "4070", "4060"],
        exclude_keywords: &[],
        generation: GpuGeneration::NvidiaAda,
    },
    // NVIDIA Ampere (RTX 30シリーズ)
    GpuDetectionPattern {
        keywords: &["rtx 30", "rtx30", "3090", "3080", "3070", "3060", "3050"],
        exclude_keywords: &[],
        generation: GpuGeneration::NvidiaAmpere,
    },
    // NVIDIA Turing (RTX 20 / GTX 16シリーズ)
    GpuDetectionPattern {
        keywords: &["rtx 20", "rtx20", "2080", "2070", "2060"],
        exclude_keywords: &[],
        generation: GpuGeneration::NvidiaTuring,
    },
    GpuDetectionPattern {
        keywords: &["gtx 16", "gtx16", "1660", "1650"],
        exclude_keywords: &[],
        generation: GpuGeneration::NvidiaTuring,
    },
    // NVIDIA Pascal (GTX 10シリーズ)
    GpuDetectionPattern {
        keywords: &["gtx 10", "gtx10", "1080", "1070", "1060", "1050"],
        exclude_keywords: &["ti"],  // 1050 Tiなど正確な検出のため除外
        generation: GpuGeneration::NvidiaPascal,
    },
    GpuDetectionPattern {
        keywords: &["gtx 1080 ti", "gtx 1070 ti", "gtx 1050 ti"],
        exclude_keywords: &[],
        generation: GpuGeneration::NvidiaPascal,
    },
    // AMD RX 7000シリーズ (VCN 4.0)
    GpuDetectionPattern {
        keywords: &["rx 7", "rx7", "7900", "7800", "7700", "7600"],
        exclude_keywords: &[],
        generation: GpuGeneration::AmdVcn4,
    },
    // AMD RX 6000シリーズ (VCN 3.0)
    GpuDetectionPattern {
        keywords: &["rx 6", "rx6", "6900", "6800", "6700", "6600", "6500"],
        exclude_keywords: &[],
        generation: GpuGeneration::AmdVcn3,
    },
    // Intel Arc
    GpuDetectionPattern {
        keywords: &["arc a", "arc"],
        exclude_keywords: &[],
        generation: GpuGeneration::IntelArc,
    },
    // Intel QuickSync (内蔵GPU)
    GpuDetectionPattern {
        keywords: &["intel uhd", "intel iris", "intel hd"],
        exclude_keywords: &[],
        generation: GpuGeneration::IntelQuickSync,
    },
];

/// GPU世代別のエンコーダー能力テーブル
///
/// 変更しやすさのため、能力情報をテーブルで管理
const GPU_CAPABILITIES: &[GpuEncoderCapability] = &[
    GpuEncoderCapability {
        generation: GpuGeneration::NvidiaAda,
        h264: true,
        hevc: true,
        av1: true,
        b_frames: true,
        quality_equivalent: "slow",
        recommended_preset: "p7",
    },
    GpuEncoderCapability {
        generation: GpuGeneration::NvidiaAmpere,
        h264: true,
        hevc: true,
        av1: false,
        b_frames: true,
        quality_equivalent: "medium",
        recommended_preset: "p6",
    },
    GpuEncoderCapability {
        generation: GpuGeneration::NvidiaTuring,
        h264: true,
        hevc: true,
        av1: false,
        b_frames: true,
        quality_equivalent: "medium",
        recommended_preset: "p5",
    },
    GpuEncoderCapability {
        generation: GpuGeneration::NvidiaPascal,
        h264: true,
        hevc: false,
        av1: false,
        b_frames: false,
        quality_equivalent: "veryfast",
        recommended_preset: "p4",
    },
    GpuEncoderCapability {
        generation: GpuGeneration::AmdVcn4,
        h264: true,
        hevc: true,
        av1: false,
        b_frames: true,
        quality_equivalent: "fast",
        recommended_preset: "default",
    },
    GpuEncoderCapability {
        generation: GpuGeneration::AmdVcn3,
        h264: true,
        hevc: true,
        av1: false,
        b_frames: false,
        quality_equivalent: "veryfast",
        recommended_preset: "default",
    },
    GpuEncoderCapability {
        generation: GpuGeneration::IntelArc,
        h264: true,
        hevc: true,
        av1: true,
        b_frames: true,
        quality_equivalent: "medium",
        recommended_preset: "balanced",
    },
    GpuEncoderCapability {
        generation: GpuGeneration::IntelQuickSync,
        h264: true,
        hevc: true,
        av1: false,
        b_frames: true,
        quality_equivalent: "fast",
        recommended_preset: "balanced",
    },
];

/// GPU名から世代を判定
///
/// # Arguments
/// * `gpu_name` - GPU名称（例: "NVIDIA GeForce RTX 3060"）
///
/// # Returns
/// 判定されたGPU世代
pub fn detect_gpu_generation(gpu_name: &str) -> GpuGeneration {
    let gpu_name_lower = gpu_name.to_lowercase();

    for pattern in GPU_PATTERNS {
        // キーワードマッチをチェック
        let has_keyword = pattern
            .keywords
            .iter()
            .any(|kw| gpu_name_lower.contains(kw));

        // 除外キーワードチェック
        let has_exclude = pattern
            .exclude_keywords
            .iter()
            .any(|kw| gpu_name_lower.contains(kw));

        if has_keyword && !has_exclude {
            return pattern.generation;
        }
    }

    GpuGeneration::Unknown
}

/// GPU世代からエンコーダー能力を取得
///
/// # Arguments
/// * `generation` - GPU世代
///
/// # Returns
/// エンコーダー能力情報（見つからない場合はNone）
pub fn get_encoder_capability(generation: GpuGeneration) -> Option<&'static GpuEncoderCapability> {
    GPU_CAPABILITIES
        .iter()
        .find(|cap| cap.generation == generation)
}

/// CPUコア数からティアを判定
///
/// # Arguments
/// * `cpu_cores` - CPUコア数
///
/// # Returns
/// CPUティア
pub fn determine_cpu_tier(cpu_cores: usize) -> CpuTier {
    match cpu_cores {
        0..=3 => CpuTier::Entry,
        4..=7 => CpuTier::Middle,
        8..=11 => CpuTier::UpperMiddle,
        _ => CpuTier::HighEnd,
    }
}

/// GPUグレード判定パターン
struct GpuGradePattern {
    /// 検索キーワード（大文字小文字を区別しない）
    keywords: &'static [&'static str],
    /// 判定されるグレード
    grade: GpuGrade,
}

/// GPUグレード判定パターン定義テーブル
/// 順序が重要：より具体的なパターンを先に配置
const GPU_GRADE_PATTERNS: &[GpuGradePattern] = &[
    // === NVIDIA Flagship (xx90, Titan) ===
    GpuGradePattern {
        keywords: &["5090", "4090", "3090", "2080 ti", "1080 ti", "titan"],
        grade: GpuGrade::Flagship,
    },
    // === NVIDIA HighEnd (xx80) ===
    GpuGradePattern {
        keywords: &["5080", "4080", "3080", "2080", "1080"],
        grade: GpuGrade::HighEnd,
    },
    // === NVIDIA UpperMid (xx70) ===
    GpuGradePattern {
        keywords: &["5070", "4070", "3070", "2070", "1070"],
        grade: GpuGrade::UpperMid,
    },
    // === NVIDIA Mid (xx60) ===
    GpuGradePattern {
        keywords: &["5060", "4060", "3060", "2060", "1660", "1060"],
        grade: GpuGrade::Mid,
    },
    // === NVIDIA Entry (xx50) ===
    GpuGradePattern {
        keywords: &["5050", "4050", "3050", "1650", "1050"],
        grade: GpuGrade::Entry,
    },
    // === AMD Flagship (x900) ===
    GpuGradePattern {
        keywords: &["7900", "6900"],
        grade: GpuGrade::Flagship,
    },
    // === AMD HighEnd (x800) ===
    GpuGradePattern {
        keywords: &["7800", "6800"],
        grade: GpuGrade::HighEnd,
    },
    // === AMD UpperMid (x700) ===
    GpuGradePattern {
        keywords: &["7700", "6700"],
        grade: GpuGrade::UpperMid,
    },
    // === AMD Mid (x600) ===
    GpuGradePattern {
        keywords: &["7600", "6600"],
        grade: GpuGrade::Mid,
    },
    // === AMD Entry (x500) ===
    GpuGradePattern {
        keywords: &["6500", "6400"],
        grade: GpuGrade::Entry,
    },
    // === Intel Arc HighEnd ===
    GpuGradePattern {
        keywords: &["a770"],
        grade: GpuGrade::HighEnd,
    },
    // === Intel Arc UpperMid ===
    GpuGradePattern {
        keywords: &["a750"],
        grade: GpuGrade::UpperMid,
    },
    // === Intel Arc Mid ===
    GpuGradePattern {
        keywords: &["a580"],
        grade: GpuGrade::Mid,
    },
    // === Intel Arc Entry ===
    GpuGradePattern {
        keywords: &["a380", "a310"],
        grade: GpuGrade::Entry,
    },
];

/// GPU名から性能グレードを判定
///
/// # Arguments
/// * `gpu_name` - GPU名称（例: "NVIDIA GeForce RTX 3060"）
///
/// # Returns
/// 判定されたGPUグレード
pub fn detect_gpu_grade(gpu_name: &str) -> GpuGrade {
    let gpu_name_lower = gpu_name.to_lowercase();

    for pattern in GPU_GRADE_PATTERNS {
        let has_keyword = pattern
            .keywords
            .iter()
            .any(|kw| gpu_name_lower.contains(kw));

        if has_keyword {
            return pattern.grade;
        }
    }

    GpuGrade::Unknown
}

/// 後方互換性のためのエイリアス（テストで使用）
#[allow(dead_code)]
pub fn detect_gpu_tier(gpu_name: &str) -> GpuTier {
    detect_gpu_grade(gpu_name)
}

/// 世代とグレードから統合ティアを算出
///
/// 世代の新しさとグレードを組み合わせて最終的な性能ティアを決定
///
/// # 統合ティアマトリクス
/// ```text
///              | Flagship | HighEnd | UpperMid | Mid  | Entry |
/// Ada (40/50)  |    S     |    S    |    A     |  A   |   B   |
/// Ampere (30)  |    A     |    A    |    B     |  B   |   C   |
/// Turing (20)  |    B     |    B    |    C     |  C   |   D   |
/// Pascal (10)  |    C     |    C    |    D     |  D   |   E   |
/// VCN4 (7000)  |    A     |    A    |    B     |  B   |   C   |
/// VCN3 (6000)  |    B     |    B    |    C     |  C   |   D   |
/// Intel Arc   |    -     |    A    |    B     |  C   |   D   |
/// QuickSync   |    -     |    -    |    -     |  D   |   E   |
/// ```
pub fn calculate_effective_tier(generation: GpuGeneration, grade: GpuGrade) -> EffectiveTier {
    // マトリクス通りの直接マッピング
    match (generation, grade) {
        // === NVIDIA Ada (RTX 40/50シリーズ) ===
        (GpuGeneration::NvidiaAda, GpuGrade::Flagship | GpuGrade::HighEnd) => EffectiveTier::TierS,
        (GpuGeneration::NvidiaAda, GpuGrade::UpperMid | GpuGrade::Mid) => EffectiveTier::TierA,
        (GpuGeneration::NvidiaAda, GpuGrade::Entry) => EffectiveTier::TierB,

        // === NVIDIA Ampere (RTX 30シリーズ) ===
        (GpuGeneration::NvidiaAmpere, GpuGrade::Flagship | GpuGrade::HighEnd) => EffectiveTier::TierA,
        (GpuGeneration::NvidiaAmpere, GpuGrade::UpperMid | GpuGrade::Mid) => EffectiveTier::TierB,
        (GpuGeneration::NvidiaAmpere, GpuGrade::Entry) => EffectiveTier::TierC,

        // === NVIDIA Turing (RTX 20/GTX 16シリーズ) ===
        (GpuGeneration::NvidiaTuring, GpuGrade::Flagship | GpuGrade::HighEnd) => EffectiveTier::TierB,
        (GpuGeneration::NvidiaTuring, GpuGrade::UpperMid | GpuGrade::Mid) => EffectiveTier::TierC,
        (GpuGeneration::NvidiaTuring, GpuGrade::Entry) => EffectiveTier::TierD,

        // === NVIDIA Pascal (GTX 10シリーズ) ===
        (GpuGeneration::NvidiaPascal, GpuGrade::Flagship | GpuGrade::HighEnd) => EffectiveTier::TierC,
        (GpuGeneration::NvidiaPascal, GpuGrade::UpperMid | GpuGrade::Mid) => EffectiveTier::TierD,
        (GpuGeneration::NvidiaPascal, GpuGrade::Entry) => EffectiveTier::TierE,

        // === AMD VCN4 (RX 7000シリーズ) ===
        (GpuGeneration::AmdVcn4, GpuGrade::Flagship | GpuGrade::HighEnd) => EffectiveTier::TierA,
        (GpuGeneration::AmdVcn4, GpuGrade::UpperMid | GpuGrade::Mid) => EffectiveTier::TierB,
        (GpuGeneration::AmdVcn4, GpuGrade::Entry) => EffectiveTier::TierC,

        // === AMD VCN3 (RX 6000シリーズ) ===
        (GpuGeneration::AmdVcn3, GpuGrade::Flagship | GpuGrade::HighEnd) => EffectiveTier::TierB,
        (GpuGeneration::AmdVcn3, GpuGrade::UpperMid | GpuGrade::Mid) => EffectiveTier::TierC,
        (GpuGeneration::AmdVcn3, GpuGrade::Entry) => EffectiveTier::TierD,

        // === Intel Arc ===
        (GpuGeneration::IntelArc, GpuGrade::HighEnd) => EffectiveTier::TierA,
        (GpuGeneration::IntelArc, GpuGrade::UpperMid) => EffectiveTier::TierB,
        (GpuGeneration::IntelArc, GpuGrade::Mid) => EffectiveTier::TierC,
        (GpuGeneration::IntelArc, GpuGrade::Entry | GpuGrade::Flagship) => EffectiveTier::TierD,

        // === Intel QuickSync (内蔵GPU) ===
        (GpuGeneration::IntelQuickSync, GpuGrade::Mid | GpuGrade::UpperMid | GpuGrade::HighEnd | GpuGrade::Flagship) => EffectiveTier::TierD,
        (GpuGeneration::IntelQuickSync, GpuGrade::Entry) => EffectiveTier::TierE,

        // === Unknown / None / その他 ===
        (_, GpuGrade::Unknown) => EffectiveTier::TierD, // 不明時は保守的に
        (GpuGeneration::Unknown | GpuGeneration::None, _) => EffectiveTier::TierE,
    }
}

/// 統合ティアに基づくプリセット調整値を取得
///
/// # Arguments
/// * `base_preset` - 世代ごとの基本プリセット番号（例: P7 → 7）
/// * `effective_tier` - 統合ティア
///
/// # Returns
/// 調整後のプリセット番号
pub fn adjust_preset_for_effective_tier(base_preset: u8, effective_tier: EffectiveTier) -> u8 {
    let adjustment: i8 = match effective_tier {
        EffectiveTier::TierS => 0,   // 最高性能、調整なし
        EffectiveTier::TierA => 0,   // 高性能、調整なし
        EffectiveTier::TierB => -1,  // 1段階下げる
        EffectiveTier::TierC => -1,  // 1段階下げる
        EffectiveTier::TierD => -2,  // 2段階下げる
        EffectiveTier::TierE => -3,  // 3段階下げる
    };

    // P1-P7の範囲に収める
    let adjusted = (base_preset as i8 + adjustment).clamp(1, 7);
    adjusted as u8
}

/// 統合ティアに基づくマルチパスモード判定
pub fn should_enable_multipass(effective_tier: EffectiveTier) -> bool {
    matches!(effective_tier, EffectiveTier::TierS | EffectiveTier::TierA | EffectiveTier::TierB)
}

/// 後方互換性のための旧API（テストで使用）
#[allow(dead_code)]
pub fn adjust_preset_for_tier(base_preset: u8, tier: GpuTier) -> u8 {
    // 旧APIでは世代情報がないため、Adaと仮定して計算
    let effective = calculate_effective_tier(GpuGeneration::NvidiaAda, tier);
    adjust_preset_for_effective_tier(base_preset, effective)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_nvidia_ada() {
        assert_eq!(
            detect_gpu_generation("NVIDIA GeForce RTX 4090"),
            GpuGeneration::NvidiaAda
        );
        assert_eq!(
            detect_gpu_generation("RTX 4080"),
            GpuGeneration::NvidiaAda
        );
    }

    #[test]
    fn test_detect_nvidia_blackwell() {
        // RTX 50シリーズ（Blackwell）はAda相当として扱う
        assert_eq!(
            detect_gpu_generation("NVIDIA GeForce RTX 5090"),
            GpuGeneration::NvidiaAda
        );
        assert_eq!(
            detect_gpu_generation("RTX 5080"),
            GpuGeneration::NvidiaAda
        );
        assert_eq!(
            detect_gpu_generation("GeForce RTX 5070 Ti"),
            GpuGeneration::NvidiaAda
        );
    }

    #[test]
    fn test_detect_nvidia_ampere() {
        assert_eq!(
            detect_gpu_generation("NVIDIA GeForce RTX 3060"),
            GpuGeneration::NvidiaAmpere
        );
        assert_eq!(
            detect_gpu_generation("RTX 3080 Ti"),
            GpuGeneration::NvidiaAmpere
        );
    }

    #[test]
    fn test_detect_nvidia_turing() {
        assert_eq!(
            detect_gpu_generation("NVIDIA GeForce RTX 2070"),
            GpuGeneration::NvidiaTuring
        );
        assert_eq!(
            detect_gpu_generation("GTX 1660 Super"),
            GpuGeneration::NvidiaTuring
        );
    }

    #[test]
    fn test_detect_nvidia_pascal() {
        assert_eq!(
            detect_gpu_generation("NVIDIA GeForce GTX 1080"),
            GpuGeneration::NvidiaPascal
        );
        assert_eq!(
            detect_gpu_generation("GTX 1060 6GB"),
            GpuGeneration::NvidiaPascal
        );
    }

    #[test]
    fn test_detect_amd_vcn4() {
        assert_eq!(
            detect_gpu_generation("AMD Radeon RX 7900 XT"),
            GpuGeneration::AmdVcn4
        );
    }

    #[test]
    fn test_detect_amd_vcn3() {
        assert_eq!(
            detect_gpu_generation("AMD Radeon RX 6800"),
            GpuGeneration::AmdVcn3
        );
    }

    #[test]
    fn test_detect_intel_arc() {
        assert_eq!(
            detect_gpu_generation("Intel Arc A770"),
            GpuGeneration::IntelArc
        );
    }

    #[test]
    fn test_detect_intel_quicksync() {
        assert_eq!(
            detect_gpu_generation("Intel UHD Graphics 770"),
            GpuGeneration::IntelQuickSync
        );
        assert_eq!(
            detect_gpu_generation("Intel Iris Xe Graphics"),
            GpuGeneration::IntelQuickSync
        );
    }

    #[test]
    fn test_detect_unknown_gpu() {
        assert_eq!(
            detect_gpu_generation("Unknown GPU Model"),
            GpuGeneration::Unknown
        );
    }

    #[test]
    fn test_get_encoder_capability_ada() {
        let cap = get_encoder_capability(GpuGeneration::NvidiaAda);
        assert!(cap.is_some());
        let cap = cap.unwrap();
        assert!(cap.h264);
        assert!(cap.hevc);
        assert!(cap.av1);
        assert!(cap.b_frames);
        assert_eq!(cap.quality_equivalent, "slow");
        assert_eq!(cap.recommended_preset, "p7");
    }

    #[test]
    fn test_get_encoder_capability_turing() {
        let cap = get_encoder_capability(GpuGeneration::NvidiaTuring);
        assert!(cap.is_some());
        let cap = cap.unwrap();
        assert!(cap.h264);
        assert!(cap.hevc);
        assert!(!cap.av1);
        assert!(cap.b_frames);
        assert_eq!(cap.quality_equivalent, "medium");
        assert_eq!(cap.recommended_preset, "p5");
    }

    #[test]
    fn test_get_encoder_capability_pascal() {
        let cap = get_encoder_capability(GpuGeneration::NvidiaPascal);
        assert!(cap.is_some());
        let cap = cap.unwrap();
        assert!(!cap.b_frames); // Pascal does not support B-frames
        assert_eq!(cap.quality_equivalent, "veryfast");
    }

    #[test]
    fn test_determine_cpu_tier() {
        assert_eq!(determine_cpu_tier(2), CpuTier::Entry);
        assert_eq!(determine_cpu_tier(4), CpuTier::Middle);
        assert_eq!(determine_cpu_tier(6), CpuTier::Middle);
        assert_eq!(determine_cpu_tier(8), CpuTier::UpperMiddle);
        assert_eq!(determine_cpu_tier(12), CpuTier::HighEnd);
        assert_eq!(determine_cpu_tier(16), CpuTier::HighEnd);
    }

    #[test]
    fn test_case_insensitive_detection() {
        assert_eq!(
            detect_gpu_generation("nvidia geforce rtx 3060"),
            GpuGeneration::NvidiaAmpere
        );
        assert_eq!(
            detect_gpu_generation("NVIDIA GEFORCE RTX 3060"),
            GpuGeneration::NvidiaAmpere
        );
    }

    #[test]
    fn test_gpu_capability_table_completeness() {
        // すべての世代に能力情報が定義されているか（NoneとUnknownを除く）
        for generation in [
            GpuGeneration::NvidiaPascal,
            GpuGeneration::NvidiaTuring,
            GpuGeneration::NvidiaAmpere,
            GpuGeneration::NvidiaAda,
            GpuGeneration::AmdVcn3,
            GpuGeneration::AmdVcn4,
            GpuGeneration::IntelArc,
            GpuGeneration::IntelQuickSync,
        ] {
            assert!(
                get_encoder_capability(generation).is_some(),
                "Capability for {:?} should be defined",
                generation
            );
        }
    }

    // === GPUティア判定テスト ===

    #[test]
    fn test_detect_gpu_tier_nvidia_flagship() {
        assert_eq!(detect_gpu_tier("NVIDIA GeForce RTX 5090"), GpuTier::Flagship);
        assert_eq!(detect_gpu_tier("RTX 4090"), GpuTier::Flagship);
        assert_eq!(detect_gpu_tier("RTX 3090"), GpuTier::Flagship);
        assert_eq!(detect_gpu_tier("GTX 2080 Ti"), GpuTier::Flagship);
        assert_eq!(detect_gpu_tier("GTX 1080 Ti"), GpuTier::Flagship);
        assert_eq!(detect_gpu_tier("TITAN RTX"), GpuTier::Flagship);
    }

    #[test]
    fn test_detect_gpu_tier_nvidia_highend() {
        assert_eq!(detect_gpu_tier("RTX 5080"), GpuTier::HighEnd);
        assert_eq!(detect_gpu_tier("RTX 4080"), GpuTier::HighEnd);
        assert_eq!(detect_gpu_tier("RTX 3080"), GpuTier::HighEnd);
        assert_eq!(detect_gpu_tier("RTX 2080"), GpuTier::HighEnd);
        assert_eq!(detect_gpu_tier("GTX 1080"), GpuTier::HighEnd);
    }

    #[test]
    fn test_detect_gpu_tier_nvidia_upper_mid() {
        assert_eq!(detect_gpu_tier("RTX 5070 Ti"), GpuTier::UpperMid);
        assert_eq!(detect_gpu_tier("RTX 4070"), GpuTier::UpperMid);
        assert_eq!(detect_gpu_tier("RTX 3070"), GpuTier::UpperMid);
        assert_eq!(detect_gpu_tier("RTX 2070 Super"), GpuTier::UpperMid);
        assert_eq!(detect_gpu_tier("GTX 1070"), GpuTier::UpperMid);
    }

    #[test]
    fn test_detect_gpu_tier_nvidia_mid() {
        assert_eq!(detect_gpu_tier("RTX 5060"), GpuTier::Mid);
        assert_eq!(detect_gpu_tier("RTX 4060 Ti"), GpuTier::Mid);
        assert_eq!(detect_gpu_tier("RTX 3060"), GpuTier::Mid);
        assert_eq!(detect_gpu_tier("RTX 2060"), GpuTier::Mid);
        assert_eq!(detect_gpu_tier("GTX 1660 Super"), GpuTier::Mid);
        assert_eq!(detect_gpu_tier("GTX 1060"), GpuTier::Mid);
    }

    #[test]
    fn test_detect_gpu_tier_nvidia_entry() {
        assert_eq!(detect_gpu_tier("RTX 4050"), GpuTier::Entry);
        assert_eq!(detect_gpu_tier("RTX 3050"), GpuTier::Entry);
        assert_eq!(detect_gpu_tier("GTX 1650 Super"), GpuTier::Entry);
        assert_eq!(detect_gpu_tier("GTX 1050 Ti"), GpuTier::Entry);
    }

    #[test]
    fn test_detect_gpu_tier_amd() {
        assert_eq!(detect_gpu_tier("AMD Radeon RX 7900 XT"), GpuTier::Flagship);
        assert_eq!(detect_gpu_tier("AMD Radeon RX 6900 XT"), GpuTier::Flagship);
        assert_eq!(detect_gpu_tier("AMD Radeon RX 7800 XT"), GpuTier::HighEnd);
        assert_eq!(detect_gpu_tier("AMD Radeon RX 6800"), GpuTier::HighEnd);
        assert_eq!(detect_gpu_tier("AMD Radeon RX 7700 XT"), GpuTier::UpperMid);
        assert_eq!(detect_gpu_tier("AMD Radeon RX 6700 XT"), GpuTier::UpperMid);
        assert_eq!(detect_gpu_tier("AMD Radeon RX 7600"), GpuTier::Mid);
        assert_eq!(detect_gpu_tier("AMD Radeon RX 6600"), GpuTier::Mid);
        assert_eq!(detect_gpu_tier("AMD Radeon RX 6500 XT"), GpuTier::Entry);
    }

    #[test]
    fn test_detect_gpu_tier_intel_arc() {
        assert_eq!(detect_gpu_tier("Intel Arc A770"), GpuTier::HighEnd);
        assert_eq!(detect_gpu_tier("Intel Arc A750"), GpuTier::UpperMid);
        assert_eq!(detect_gpu_tier("Intel Arc A580"), GpuTier::Mid);
        assert_eq!(detect_gpu_tier("Intel Arc A380"), GpuTier::Entry);
        assert_eq!(detect_gpu_tier("Intel Arc A310"), GpuTier::Entry);
    }

    #[test]
    fn test_detect_gpu_tier_unknown() {
        assert_eq!(detect_gpu_tier("Intel UHD Graphics 770"), GpuTier::Unknown);
        assert_eq!(detect_gpu_tier("Some Unknown GPU"), GpuTier::Unknown);
    }

    #[test]
    fn test_adjust_preset_for_tier() {
        // 後方互換API（NvidiaAda世代を仮定）
        // Ada + Flagship/HighEnd = TierS → 調整なし
        assert_eq!(adjust_preset_for_tier(7, GpuTier::Flagship), 7);
        assert_eq!(adjust_preset_for_tier(7, GpuTier::HighEnd), 7);

        // Ada + UpperMid/Mid = TierA → 調整なし
        assert_eq!(adjust_preset_for_tier(7, GpuTier::UpperMid), 7);
        assert_eq!(adjust_preset_for_tier(7, GpuTier::Mid), 7);
        assert_eq!(adjust_preset_for_tier(6, GpuTier::Mid), 6);

        // Ada + Entry = TierB → -1
        assert_eq!(adjust_preset_for_tier(7, GpuTier::Entry), 6);
        assert_eq!(adjust_preset_for_tier(6, GpuTier::Entry), 5);
        assert_eq!(adjust_preset_for_tier(4, GpuTier::Entry), 3);

        // 最小値のクランプ（P1未満にはならない）
        assert_eq!(adjust_preset_for_tier(2, GpuTier::Entry), 1);
        assert_eq!(adjust_preset_for_tier(1, GpuTier::Entry), 1);
    }

    // === 統合ティア算出テスト ===

    #[test]
    fn test_effective_tier_nvidia_ada() {
        // Ada（RTX 40/50）の統合ティア
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaAda, GpuGrade::Flagship),
            EffectiveTier::TierS
        );
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaAda, GpuGrade::HighEnd),
            EffectiveTier::TierS
        );
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaAda, GpuGrade::UpperMid),
            EffectiveTier::TierA
        );
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaAda, GpuGrade::Mid),
            EffectiveTier::TierA
        );
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaAda, GpuGrade::Entry),
            EffectiveTier::TierB
        );
    }

    #[test]
    fn test_effective_tier_nvidia_ampere() {
        // Ampere（RTX 30）の統合ティア
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaAmpere, GpuGrade::Flagship),
            EffectiveTier::TierA
        );
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaAmpere, GpuGrade::HighEnd),
            EffectiveTier::TierA
        );
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaAmpere, GpuGrade::UpperMid),
            EffectiveTier::TierB
        );
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaAmpere, GpuGrade::Mid),
            EffectiveTier::TierB
        );
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaAmpere, GpuGrade::Entry),
            EffectiveTier::TierC
        );
    }

    #[test]
    fn test_effective_tier_nvidia_turing() {
        // Turing（RTX 20/GTX 16）の統合ティア
        // マトリクス: Flagship/HighEnd=B, UpperMid/Mid=C, Entry=D
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaTuring, GpuGrade::Flagship),
            EffectiveTier::TierB
        );
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaTuring, GpuGrade::HighEnd),
            EffectiveTier::TierB
        );
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaTuring, GpuGrade::UpperMid),
            EffectiveTier::TierC
        );
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaTuring, GpuGrade::Mid),
            EffectiveTier::TierC
        );
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaTuring, GpuGrade::Entry),
            EffectiveTier::TierD
        );
    }

    #[test]
    fn test_effective_tier_nvidia_pascal() {
        // Pascal（GTX 10）の統合ティア
        // マトリクス: Flagship/HighEnd=C, UpperMid/Mid=D, Entry=E
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaPascal, GpuGrade::Flagship),
            EffectiveTier::TierC
        );
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaPascal, GpuGrade::HighEnd),
            EffectiveTier::TierC
        );
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaPascal, GpuGrade::UpperMid),
            EffectiveTier::TierD
        );
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaPascal, GpuGrade::Mid),
            EffectiveTier::TierD
        );
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaPascal, GpuGrade::Entry),
            EffectiveTier::TierE
        );
    }

    #[test]
    fn test_effective_tier_cross_generation_equivalence() {
        // 世代をまたいだ同等性テスト
        // RTX 3090（Ampere Flagship）≒ RTX 4060（Ada Mid）
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaAmpere, GpuGrade::Flagship),
            calculate_effective_tier(GpuGeneration::NvidiaAda, GpuGrade::Mid)
        );

        // RTX 2080 Ti（Turing Flagship）≒ RTX 3070（Ampere UpperMid）
        assert_eq!(
            calculate_effective_tier(GpuGeneration::NvidiaTuring, GpuGrade::Flagship),
            calculate_effective_tier(GpuGeneration::NvidiaAmpere, GpuGrade::UpperMid)
        );
    }

    #[test]
    fn test_adjust_preset_for_effective_tier() {
        // TierS/A: 調整なし
        assert_eq!(adjust_preset_for_effective_tier(7, EffectiveTier::TierS), 7);
        assert_eq!(adjust_preset_for_effective_tier(7, EffectiveTier::TierA), 7);

        // TierB/C: -1
        assert_eq!(adjust_preset_for_effective_tier(7, EffectiveTier::TierB), 6);
        assert_eq!(adjust_preset_for_effective_tier(7, EffectiveTier::TierC), 6);

        // TierD: -2
        assert_eq!(adjust_preset_for_effective_tier(7, EffectiveTier::TierD), 5);

        // TierE: -3
        assert_eq!(adjust_preset_for_effective_tier(7, EffectiveTier::TierE), 4);

        // 最小値のクランプ
        assert_eq!(adjust_preset_for_effective_tier(2, EffectiveTier::TierE), 1);
    }

    #[test]
    fn test_should_enable_multipass() {
        // TierS/A/B: マルチパス有効
        assert!(should_enable_multipass(EffectiveTier::TierS));
        assert!(should_enable_multipass(EffectiveTier::TierA));
        assert!(should_enable_multipass(EffectiveTier::TierB));

        // TierC以下: マルチパス無効
        assert!(!should_enable_multipass(EffectiveTier::TierC));
        assert!(!should_enable_multipass(EffectiveTier::TierD));
        assert!(!should_enable_multipass(EffectiveTier::TierE));
    }
}
