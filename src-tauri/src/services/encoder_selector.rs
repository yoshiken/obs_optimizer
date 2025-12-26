// エンコーダー選択サービス
//
// ハードウェア情報とユーザー設定からエンコーダーを選択
// obs_guide.mdの判定ロジックに基づく

use super::gpu_detection::{CpuTier, GpuEncoderCapability, GpuGeneration, get_encoder_capability};
use crate::storage::config::{StreamingPlatform, StreamingStyle};
use serde::{Deserialize, Serialize};

/// 推奨エンコーダー情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendedEncoder {
    /// OBSエンコーダーID（ffmpeg_nvenc, amd_amf_h264, obs_qsv11, obs_x264等）
    pub encoder_id: String,
    /// エンコーダー表示名
    pub display_name: String,
    /// プリセット（NVENCならP1-P7、x264ならultrafast-slow）
    pub preset: String,
    /// レート制御モード
    pub rate_control: String,
    /// Bフレーム設定（使用する場合の推奨値）
    pub b_frames: Option<u32>,
    /// Look-ahead有効化（NVENC/AMF）
    pub look_ahead: bool,
    /// Psycho Visual Tuning有効化（NVENC）
    pub psycho_visual_tuning: bool,
    /// マルチパスモード（NVENC: "disabled", "quarter_res", "full_res"）
    pub multipass_mode: String,
    /// チューニング設定（NVENC: "hq", "ll", "ull" / x264: "film", "animation"等）
    pub tuning: Option<String>,
    /// H.264プロファイル（"baseline", "main", "high"）
    pub profile: String,
    /// 選択理由
    pub reason: String,
}

/// エンコーダー選択コンテキスト
#[derive(Debug, Clone)]
pub struct EncoderSelectionContext {
    /// GPU世代
    pub gpu_generation: GpuGeneration,
    /// CPUティア
    pub cpu_tier: CpuTier,
    /// 配信プラットフォーム（将来のビットレート計算で使用予定）
    #[allow(dead_code)]
    pub platform: StreamingPlatform,
    /// 配信スタイル（将来のビットレート計算で使用予定）
    #[allow(dead_code)]
    pub style: StreamingStyle,
    /// ネットワーク速度（Mbps）（将来のビットレート計算で使用予定）
    #[allow(dead_code)]
    pub network_speed_mbps: f64,
}

/// エンコーダー選択エンジン
pub struct EncoderSelector;

impl EncoderSelector {
    /// 推奨エンコーダーを選択
    ///
    /// # Arguments
    /// * `context` - エンコーダー選択コンテキスト
    ///
    /// # Returns
    /// 推奨エンコーダー情報
    pub fn select_encoder(context: &EncoderSelectionContext) -> RecommendedEncoder {
        // プラットフォーム別の制約を確認
        let platform_supports_av1 = matches!(context.platform, StreamingPlatform::YouTube);
        let platform_supports_hevc = matches!(
            context.platform,
            StreamingPlatform::YouTube | StreamingPlatform::TwitCasting
        );

        // GPU世代に基づく判定
        match context.gpu_generation {
            GpuGeneration::NvidiaAda
            | GpuGeneration::NvidiaAmpere
            | GpuGeneration::NvidiaTuring => {
                // YouTube かつ AV1対応GPUの場合はAV1を優先検討
                if platform_supports_av1 && Self::gpu_supports_av1(context.gpu_generation) {
                    Self::select_av1_encoder(context)
                } else {
                    Self::select_nvenc_encoder(context)
                }
            }
            GpuGeneration::NvidiaPascal => {
                // Pascal世代は品質が低いため、CPUがハイエンドならx264も検討
                if matches!(context.cpu_tier, CpuTier::HighEnd) {
                    Self::select_x264_or_nvenc(context)
                } else {
                    Self::select_nvenc_encoder(context)
                }
            }
            GpuGeneration::AmdVcn4 | GpuGeneration::AmdVcn3 => {
                Self::select_amd_encoder(context)
            }
            GpuGeneration::IntelArc => {
                // Intel ArcもAV1対応だが、YouTubeの場合のみ
                if platform_supports_av1 {
                    Self::select_av1_encoder(context)
                } else {
                    Self::select_intel_arc_encoder(context)
                }
            }
            GpuGeneration::IntelQuickSync => Self::select_quicksync_encoder(context),
            GpuGeneration::Unknown | GpuGeneration::None => {
                // GPUがない、または不明の場合はCPUエンコード
                Self::select_x264_encoder(context)
            }
        }
    }

    /// GPUがAV1をサポートしているか確認
    fn gpu_supports_av1(generation: GpuGeneration) -> bool {
        if let Some(capability) = get_encoder_capability(generation) {
            capability.av1
        } else {
            false
        }
    }

    /// AV1 エンコーダーを選択
    fn select_av1_encoder(context: &EncoderSelectionContext) -> RecommendedEncoder {
        let encoder_id = match context.gpu_generation {
            GpuGeneration::NvidiaAda => "jim_av1_nvenc", // NVIDIA AV1
            GpuGeneration::IntelArc => "obs_qsv11_av1",  // Intel Arc AV1
            _ => "ffmpeg_nvenc", // フォールバック: H.264
        };

        let is_av1 = matches!(
            context.gpu_generation,
            GpuGeneration::NvidiaAda | GpuGeneration::IntelArc
        );

        if is_av1 {
            let reason = format!(
                "{}を検出。AV1エンコーダーはYouTubeで高画質・低ビットレートを実現します。H.264の30%程度のビットレートで同等画質を達成可能",
                Self::gpu_display_name(context.gpu_generation)
            );

            RecommendedEncoder {
                encoder_id: encoder_id.to_string(),
                display_name: "AV1 (Hardware)".to_string(),
                preset: "p7".to_string(), // AV1は高品質プリセット推奨
                rate_control: "CBR".to_string(),
                b_frames: Some(2),
                look_ahead: true,
                psycho_visual_tuning: true,
                multipass_mode: "quarter_res".to_string(),
                tuning: Some("hq".to_string()),
                profile: "main".to_string(), // AV1はmainプロファイル
                reason,
            }
        } else {
            // AV1非対応の場合はH.264にフォールバック
            Self::select_nvenc_encoder(context)
        }
    }

    /// NVENC エンコーダーを選択
    fn select_nvenc_encoder(context: &EncoderSelectionContext) -> RecommendedEncoder {
        // デフォルトのNVENC能力情報（フォールバック用）
        let default_capability = GpuEncoderCapability {
            generation: context.gpu_generation,
            h264: true,
            hevc: true,
            av1: false,
            b_frames: true,
            quality_equivalent: "medium",
            recommended_preset: "p5",
        };
        let capability = get_encoder_capability(context.gpu_generation)
            .unwrap_or(&default_capability);

        let b_frames = if capability.b_frames { Some(2) } else { None };

        // Turing以降は高品質機能を有効化
        let psycho_visual_tuning = matches!(
            context.gpu_generation,
            GpuGeneration::NvidiaTuring
                | GpuGeneration::NvidiaAmpere
                | GpuGeneration::NvidiaAda
        );

        let look_ahead = matches!(
            context.gpu_generation,
            GpuGeneration::NvidiaAmpere | GpuGeneration::NvidiaAda
        );

        // マルチパスモード: Turing以降は2パス(1/4解像度)で高品質化
        // 参考: https://castcraft.live/blog/178/
        let multipass_mode = match context.gpu_generation {
            GpuGeneration::NvidiaAda | GpuGeneration::NvidiaAmpere => "quarter_res".to_string(),
            GpuGeneration::NvidiaTuring => "quarter_res".to_string(),
            _ => "disabled".to_string(),
        };

        // チューニング: 高品質設定を推奨
        let tuning = match context.gpu_generation {
            GpuGeneration::NvidiaAda | GpuGeneration::NvidiaAmpere | GpuGeneration::NvidiaTuring => {
                Some("hq".to_string())
            }
            _ => None,
        };

        let reason = format!(
            "{}を検出。NVENCエンコーダーはCPU負荷をほぼゼロにし、{}相当の品質を実現します。マルチパス2パスで高画質化",
            Self::gpu_display_name(context.gpu_generation),
            capability.quality_equivalent
        );

        RecommendedEncoder {
            encoder_id: "ffmpeg_nvenc".to_string(),
            display_name: "NVIDIA NVENC H.264".to_string(),
            preset: capability.recommended_preset.to_string(),
            rate_control: "CBR".to_string(),
            b_frames,
            look_ahead,
            psycho_visual_tuning,
            multipass_mode,
            tuning,
            profile: "high".to_string(),
            reason,
        }
    }

    /// AMD AMF エンコーダーを選択
    fn select_amd_encoder(context: &EncoderSelectionContext) -> RecommendedEncoder {
        // デフォルトのAMD能力情報（フォールバック用）
        let default_capability = GpuEncoderCapability {
            generation: context.gpu_generation,
            h264: true,
            hevc: true,
            av1: false,
            b_frames: false,
            quality_equivalent: "fast",
            recommended_preset: "default",
        };
        let capability = get_encoder_capability(context.gpu_generation)
            .unwrap_or(&default_capability);

        // VCN 4.0はBフレームサポート
        let b_frames = if capability.b_frames { Some(2) } else { None };

        let reason = format!(
            "{}を検出。AMFエンコーダーはCPU負荷を軽減し、8Mbps以上では高品質です",
            Self::gpu_display_name(context.gpu_generation)
        );

        RecommendedEncoder {
            encoder_id: "amd_amf_h264".to_string(),
            display_name: "AMD AMF H.264".to_string(),
            preset: "quality".to_string(),
            rate_control: "CBR".to_string(),
            b_frames,
            look_ahead: false,
            psycho_visual_tuning: false,
            multipass_mode: "disabled".to_string(),
            tuning: None,
            profile: "high".to_string(),
            reason,
        }
    }

    /// Intel Arc エンコーダーを選択
    fn select_intel_arc_encoder(_context: &EncoderSelectionContext) -> RecommendedEncoder {
        RecommendedEncoder {
            encoder_id: "obs_qsv11".to_string(),
            display_name: "Intel QuickSync H.264".to_string(),
            preset: "balanced".to_string(),
            rate_control: "CBR".to_string(),
            b_frames: Some(2),
            look_ahead: true, // Intel Arcはlook-ahead対応
            psycho_visual_tuning: false,
            multipass_mode: "disabled".to_string(),
            tuning: None,
            profile: "high".to_string(),
            reason: "Intel Arcを検出。QuickSyncは低ビットレートで優秀な品質を発揮します"
                .to_string(),
        }
    }

    /// Intel QuickSync エンコーダーを選択
    fn select_quicksync_encoder(_context: &EncoderSelectionContext) -> RecommendedEncoder {
        RecommendedEncoder {
            encoder_id: "obs_qsv11".to_string(),
            display_name: "Intel QuickSync H.264".to_string(),
            preset: "balanced".to_string(),
            rate_control: "CBR".to_string(),
            b_frames: Some(2),
            look_ahead: false,
            psycho_visual_tuning: false,
            multipass_mode: "disabled".to_string(),
            tuning: None,
            profile: "main".to_string(), // 内蔵GPUは互換性重視でmain
            reason: "Intel内蔵GPUを検出。QuickSyncでCPU負荷を軽減できます".to_string(),
        }
    }

    /// x264 CPU エンコーダーを選択
    fn select_x264_encoder(context: &EncoderSelectionContext) -> RecommendedEncoder {
        let preset = Self::select_x264_preset(context.cpu_tier);

        let reason = match context.cpu_tier {
            CpuTier::Entry => {
                "GPUエンコーダーが利用できません。CPUエンコードは負荷が高いため、ハードウェアエンコーダー対応GPUの導入を推奨します".to_string()
            }
            CpuTier::Middle => {
                format!("CPUエンコード（{}プリセット）を使用。ゲームプレイ中の負荷が高くなる可能性があります", preset)
            }
            CpuTier::UpperMiddle | CpuTier::HighEnd => {
                format!("高性能CPUを検出。x264 {}プリセットで高品質配信が可能です", preset)
            }
        };

        // x264のチューニング: ゲーム配信向けにzerolatencyを検討するが
        // 品質重視の場合はNone（デフォルト）を使用
        // 参考: https://castcraft.live/blog/107/
        let tuning = match context.cpu_tier {
            CpuTier::Entry => Some("zerolatency".to_string()), // 低遅延優先
            _ => None, // 品質優先
        };

        RecommendedEncoder {
            encoder_id: "obs_x264".to_string(),
            display_name: "x264 (CPU)".to_string(),
            preset,
            rate_control: "CBR".to_string(),
            b_frames: Some(2), // x264はBフレーム使用可能
            look_ahead: false,
            psycho_visual_tuning: false,
            multipass_mode: "disabled".to_string(),
            tuning,
            profile: "high".to_string(),
            reason,
        }
    }

    /// x264プリセットを選択（CPUティアに基づく）
    fn select_x264_preset(cpu_tier: CpuTier) -> String {
        match cpu_tier {
            CpuTier::Entry => "ultrafast".to_string(),
            CpuTier::Middle => "veryfast".to_string(),
            CpuTier::UpperMiddle => "faster".to_string(),
            CpuTier::HighEnd => "fast".to_string(),
        }
    }

    /// x264とNVENCを比較して選択（Pascal世代用）
    fn select_x264_or_nvenc(context: &EncoderSelectionContext) -> RecommendedEncoder {
        // Pascalは品質が低いため、ハイエンドCPUならx264を優先
        if matches!(context.cpu_tier, CpuTier::HighEnd) {
            let mut encoder = Self::select_x264_encoder(context);
            encoder.reason = format!(
                "{}。ハイエンドCPUを活用してx264で高品質配信を行います",
                encoder.reason
            );
            encoder
        } else {
            Self::select_nvenc_encoder(context)
        }
    }

    /// GPU世代の表示名を取得
    fn gpu_display_name(generation: GpuGeneration) -> &'static str {
        match generation {
            GpuGeneration::NvidiaAda => "NVIDIA RTX 40シリーズ",
            GpuGeneration::NvidiaAmpere => "NVIDIA RTX 30シリーズ",
            GpuGeneration::NvidiaTuring => "NVIDIA RTX 20/GTX 16シリーズ",
            GpuGeneration::NvidiaPascal => "NVIDIA GTX 10シリーズ",
            GpuGeneration::AmdVcn4 => "AMD RX 7000シリーズ",
            GpuGeneration::AmdVcn3 => "AMD RX 6000シリーズ",
            GpuGeneration::IntelArc => "Intel Arc GPU",
            GpuGeneration::IntelQuickSync => "Intel内蔵GPU",
            GpuGeneration::Unknown => "不明なGPU",
            GpuGeneration::None => "GPU未検出",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context(
        gpu_gen: GpuGeneration,
        cpu_tier: CpuTier,
    ) -> EncoderSelectionContext {
        EncoderSelectionContext {
            gpu_generation: gpu_gen,
            cpu_tier,
            platform: StreamingPlatform::YouTube,
            style: StreamingStyle::Gaming,
            network_speed_mbps: 10.0,
        }
    }

    #[test]
    fn test_select_nvenc_ada() {
        let context = create_test_context(GpuGeneration::NvidiaAda, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "ffmpeg_nvenc");
        assert_eq!(encoder.preset, "p7");
        assert!(encoder.psycho_visual_tuning);
        assert!(encoder.look_ahead);
        assert_eq!(encoder.b_frames, Some(2));
    }

    #[test]
    fn test_select_nvenc_turing() {
        let context = create_test_context(GpuGeneration::NvidiaTuring, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "ffmpeg_nvenc");
        assert_eq!(encoder.preset, "p5");
        assert!(encoder.psycho_visual_tuning);
        assert!(!encoder.look_ahead); // Turingはlook-aheadなし
        assert_eq!(encoder.b_frames, Some(2));
    }

    #[test]
    fn test_select_nvenc_pascal() {
        let context = create_test_context(GpuGeneration::NvidiaPascal, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "ffmpeg_nvenc");
        assert_eq!(encoder.preset, "p4");
        assert!(!encoder.psycho_visual_tuning);
        assert_eq!(encoder.b_frames, None); // PascalはBフレームなし
    }

    #[test]
    fn test_select_x264_for_pascal_high_end_cpu() {
        let context = create_test_context(GpuGeneration::NvidiaPascal, CpuTier::HighEnd);
        let encoder = EncoderSelector::select_encoder(&context);

        // ハイエンドCPU + Pascalならx264を選択
        assert_eq!(encoder.encoder_id, "obs_x264");
        assert_eq!(encoder.preset, "fast");
    }

    #[test]
    fn test_select_amd_vcn4() {
        let context = create_test_context(GpuGeneration::AmdVcn4, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "amd_amf_h264");
        assert_eq!(encoder.b_frames, Some(2)); // VCN 4.0はBフレーム対応
    }

    #[test]
    fn test_select_amd_vcn3() {
        let context = create_test_context(GpuGeneration::AmdVcn3, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "amd_amf_h264");
        assert_eq!(encoder.b_frames, None); // VCN 3.0はBフレーム未対応
    }

    #[test]
    fn test_select_intel_arc() {
        let context = create_test_context(GpuGeneration::IntelArc, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "obs_qsv11");
        assert_eq!(encoder.preset, "balanced");
    }

    #[test]
    fn test_select_x264_entry_cpu() {
        let context = create_test_context(GpuGeneration::None, CpuTier::Entry);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "obs_x264");
        assert_eq!(encoder.preset, "ultrafast");
    }

    #[test]
    fn test_select_x264_middle_cpu() {
        let context = create_test_context(GpuGeneration::None, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "obs_x264");
        assert_eq!(encoder.preset, "veryfast");
    }

    #[test]
    fn test_select_x264_upper_middle_cpu() {
        let context = create_test_context(GpuGeneration::None, CpuTier::UpperMiddle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "obs_x264");
        assert_eq!(encoder.preset, "faster");
    }

    #[test]
    fn test_select_x264_high_end_cpu() {
        let context = create_test_context(GpuGeneration::None, CpuTier::HighEnd);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "obs_x264");
        assert_eq!(encoder.preset, "fast");
    }

    #[test]
    fn test_encoder_has_reason() {
        let context = create_test_context(GpuGeneration::NvidiaAmpere, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert!(!encoder.reason.is_empty());
        assert!(encoder.reason.contains("RTX 30"));
    }

    #[test]
    fn test_rate_control_is_cbr() {
        // すべてのエンコーダーでCBRを使用
        for gpu_gen in [
            GpuGeneration::NvidiaAda,
            GpuGeneration::NvidiaTuring,
            GpuGeneration::AmdVcn4,
            GpuGeneration::IntelArc,
            GpuGeneration::None,
        ] {
            let context = create_test_context(gpu_gen, CpuTier::Middle);
            let encoder = EncoderSelector::select_encoder(&context);
            assert_eq!(encoder.rate_control, "CBR");
        }
    }

    #[test]
    fn test_av1_encoder_for_youtube() {
        // YouTubeプラットフォームでRTX 40シリーズの場合はAV1推奨
        let mut context = create_test_context(GpuGeneration::NvidiaAda, CpuTier::Middle);
        context.platform = StreamingPlatform::YouTube;
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "jim_av1_nvenc");
        assert!(encoder.reason.contains("AV1"));
    }

    #[test]
    fn test_no_av1_for_twitch() {
        // TwitchではAV1非対応のためH.264を使用
        let mut context = create_test_context(GpuGeneration::NvidiaAda, CpuTier::Middle);
        context.platform = StreamingPlatform::Twitch;
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "ffmpeg_nvenc");
        assert!(!encoder.reason.contains("AV1"));
    }

    #[test]
    fn test_av1_for_intel_arc_youtube() {
        // Intel ArcでYouTubeの場合もAV1推奨
        let mut context = create_test_context(GpuGeneration::IntelArc, CpuTier::Middle);
        context.platform = StreamingPlatform::YouTube;
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "obs_qsv11_av1");
        assert!(encoder.reason.contains("AV1"));
    }

    #[test]
    fn test_h264_for_intel_arc_twitch() {
        // Intel ArcでTwitchの場合はH.264
        let mut context = create_test_context(GpuGeneration::IntelArc, CpuTier::Middle);
        context.platform = StreamingPlatform::Twitch;
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "obs_qsv11");
    }

    #[test]
    fn test_platform_constraints() {
        // プラットフォームごとのエンコーダー制約テスト
        let test_cases = vec![
            (StreamingPlatform::YouTube, GpuGeneration::NvidiaAda, "jim_av1_nvenc"),
            (StreamingPlatform::Twitch, GpuGeneration::NvidiaAda, "ffmpeg_nvenc"),
            (StreamingPlatform::NicoNico, GpuGeneration::NvidiaAda, "ffmpeg_nvenc"),
            (StreamingPlatform::TwitCasting, GpuGeneration::NvidiaAda, "ffmpeg_nvenc"),
            (StreamingPlatform::Other, GpuGeneration::NvidiaAda, "ffmpeg_nvenc"),
        ];

        for (platform, gpu_gen, expected_encoder) in test_cases {
            let mut context = create_test_context(gpu_gen, CpuTier::Middle);
            context.platform = platform;
            let encoder = EncoderSelector::select_encoder(&context);
            assert_eq!(
                encoder.encoder_id, expected_encoder,
                "{:?}プラットフォームでの推奨エンコーダー",
                platform
            );
        }
    }
}
