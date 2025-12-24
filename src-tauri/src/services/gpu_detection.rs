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
}
