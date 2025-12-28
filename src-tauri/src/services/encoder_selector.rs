// エンコーダー選択サービス
//
// ハードウェア情報とユーザー設定からエンコーダーを選択
// obs_guide.mdの判定ロジックに基づく

use super::gpu_detection::{
    CpuTier, EffectiveTier, GpuEncoderCapability, GpuGeneration, GpuGrade,
    adjust_preset_for_effective_tier, calculate_effective_tier, get_encoder_capability,
    should_enable_multipass,
};
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
    /// GPU世代（アーキテクチャ）
    pub gpu_generation: GpuGeneration,
    /// GPU性能グレード（xx90/xx80/xx70等）
    pub gpu_grade: GpuGrade,
    /// CPUティア
    pub cpu_tier: CpuTier,
    /// 配信プラットフォーム
    pub platform: StreamingPlatform,
    /// 配信スタイル
    #[allow(dead_code)]
    pub style: StreamingStyle,
    /// ネットワーク速度（Mbps）
    #[allow(dead_code)]
    pub network_speed_mbps: f64,
}

impl EncoderSelectionContext {
    /// 統合ティアを計算
    pub fn effective_tier(&self) -> EffectiveTier {
        calculate_effective_tier(self.gpu_generation, self.gpu_grade)
    }
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
        // HEVC対応プラットフォーム（将来の拡張用）
        let _platform_supports_hevc = matches!(
            context.platform,
            StreamingPlatform::YouTube | StreamingPlatform::TwitCasting
        );

        // GPU世代に基づく判定
        match context.gpu_generation {
            GpuGeneration::NvidiaBlackwell
            | GpuGeneration::NvidiaAda
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
            GpuGeneration::NvidiaBlackwell | GpuGeneration::NvidiaAda => "jim_av1_nvenc", // NVIDIA AV1
            GpuGeneration::IntelArc => "obs_qsv11_av1",  // Intel Arc AV1
            _ => "ffmpeg_nvenc", // フォールバック: H.264
        };

        let is_av1 = matches!(
            context.gpu_generation,
            GpuGeneration::NvidiaBlackwell | GpuGeneration::NvidiaAda | GpuGeneration::IntelArc
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

        // 統合ティアを算出
        let effective_tier = context.effective_tier();

        let b_frames = if capability.b_frames { Some(2) } else { None };

        // Turing以降は高品質機能を有効化
        let psycho_visual_tuning = matches!(
            context.gpu_generation,
            GpuGeneration::NvidiaTuring
                | GpuGeneration::NvidiaAmpere
                | GpuGeneration::NvidiaAda
                | GpuGeneration::NvidiaBlackwell
        );

        let look_ahead = matches!(
            context.gpu_generation,
            GpuGeneration::NvidiaAmpere | GpuGeneration::NvidiaAda | GpuGeneration::NvidiaBlackwell
        );

        // マルチパスモード: 統合ティアに応じて調整
        // TierS/A/B: quarter_res（高品質）
        // TierC以下: disabled（負荷軽減）
        let multipass_mode = if should_enable_multipass(effective_tier) {
            "quarter_res".to_string()
        } else {
            "disabled".to_string()
        };

        // チューニング: 高品質設定を推奨
        let tuning = match context.gpu_generation {
            GpuGeneration::NvidiaBlackwell | GpuGeneration::NvidiaAda | GpuGeneration::NvidiaAmpere | GpuGeneration::NvidiaTuring => {
                Some("hq".to_string())
            }
            _ => None,
        };

        // プリセットを統合ティアに応じて調整
        let base_preset: u8 = capability.recommended_preset
            .trim_start_matches('p')
            .parse()
            .unwrap_or(5);
        let adjusted_preset = adjust_preset_for_effective_tier(base_preset, effective_tier);
        let preset_string = format!("p{}", adjusted_preset);

        // ティア情報を理由に追加
        let tier_note = match effective_tier {
            EffectiveTier::TierS => "（最高性能）".to_string(),
            EffectiveTier::TierA => "（高性能）".to_string(),
            EffectiveTier::TierB => "（中上位、プリセット1段階調整）".to_string(),
            EffectiveTier::TierC => "（中位、プリセット1段階調整）".to_string(),
            EffectiveTier::TierD => "（下位、プリセット2段階調整）".to_string(),
            EffectiveTier::TierE => "（エントリー、プリセット3段階調整）".to_string(),
        };

        let reason = format!(
            "{}（{}グレード）を検出。NVENCはCPU負荷ゼロで{}相当の品質{}",
            Self::gpu_display_name(context.gpu_generation),
            Self::grade_display_name(context.gpu_grade),
            capability.quality_equivalent,
            tier_note
        );

        RecommendedEncoder {
            encoder_id: "ffmpeg_nvenc".to_string(),
            display_name: "NVIDIA NVENC H.264".to_string(),
            preset: preset_string,
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

    /// グレードの表示名を取得
    fn grade_display_name(grade: GpuGrade) -> &'static str {
        match grade {
            GpuGrade::Flagship => "フラグシップ",
            GpuGrade::HighEnd => "ハイエンド",
            GpuGrade::UpperMid => "アッパーミドル",
            GpuGrade::Mid => "ミドル",
            GpuGrade::Entry => "エントリー",
            GpuGrade::Unknown => "不明",
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
            GpuGeneration::NvidiaBlackwell => "NVIDIA RTX 50シリーズ",
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
            gpu_grade: GpuGrade::HighEnd, // デフォルトはハイエンド
            cpu_tier,
            platform: StreamingPlatform::YouTube,
            style: StreamingStyle::Gaming,
            network_speed_mbps: 10.0,
        }
    }

    fn create_test_context_with_grade(
        gpu_gen: GpuGeneration,
        gpu_grade: GpuGrade,
        cpu_tier: CpuTier,
    ) -> EncoderSelectionContext {
        EncoderSelectionContext {
            gpu_generation: gpu_gen,
            gpu_grade,
            cpu_tier,
            platform: StreamingPlatform::YouTube,
            style: StreamingStyle::Gaming,
            network_speed_mbps: 10.0,
        }
    }

    #[test]
    fn test_select_nvenc_ada() {
        // Ada + HighEnd(デフォルト) = TierS → AV1エンコーダが選択される
        let context = create_test_context(GpuGeneration::NvidiaAda, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "jim_av1_nvenc"); // Ada世代はAV1対応
        assert_eq!(encoder.preset, "p7");
        assert!(encoder.psycho_visual_tuning);
        assert!(encoder.look_ahead);
        assert_eq!(encoder.b_frames, Some(2));
    }

    #[test]
    fn test_select_av1_blackwell_youtube() {
        // Blackwell + YouTube = AV1エンコーダが選択される
        let context = create_test_context(GpuGeneration::NvidiaBlackwell, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "jim_av1_nvenc", "Blackwell + YouTube must select AV1");
        assert_eq!(encoder.preset, "p7");
        assert!(encoder.psycho_visual_tuning);
        assert!(encoder.look_ahead);
        assert_eq!(encoder.b_frames, Some(2));
        assert!(encoder.reason.contains("AV1"), "Reason should mention AV1");
    }

    #[test]
    fn test_select_nvenc_blackwell_twitch() {
        // Blackwell + Twitch = H.264（TwitchはAV1非対応）
        let mut context = create_test_context(GpuGeneration::NvidiaBlackwell, CpuTier::Middle);
        context.platform = StreamingPlatform::Twitch;
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "ffmpeg_nvenc", "Blackwell + Twitch should use H.264");
        assert!(encoder.reason.contains("RTX 50"), "Reason should mention RTX 50");
    }

    #[test]
    fn test_select_nvenc_turing() {
        // Turing + HighEnd(デフォルト) = TierB → プリセット-1
        let context = create_test_context(GpuGeneration::NvidiaTuring, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "ffmpeg_nvenc");
        assert_eq!(encoder.preset, "p4"); // TierB: p5→p4
        assert!(encoder.psycho_visual_tuning);
        assert!(!encoder.look_ahead); // Turingはlook-aheadなし
        assert_eq!(encoder.b_frames, Some(2));
    }

    #[test]
    fn test_select_nvenc_pascal() {
        // Pascal + HighEnd(デフォルト) = TierC → プリセット-1
        let context = create_test_context(GpuGeneration::NvidiaPascal, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "ffmpeg_nvenc");
        assert_eq!(encoder.preset, "p3"); // TierC: p4→p3
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
        // Intel Arc + HighEnd(デフォルト) = TierA → AV1エンコーダが選択される
        let context = create_test_context(GpuGeneration::IntelArc, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "obs_qsv11_av1"); // Intel ArcはAV1対応
        assert_eq!(encoder.preset, "p7");
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
            (StreamingPlatform::YouTube, GpuGeneration::NvidiaBlackwell, "jim_av1_nvenc"),
            (StreamingPlatform::YouTube, GpuGeneration::NvidiaAda, "jim_av1_nvenc"),
            (StreamingPlatform::Twitch, GpuGeneration::NvidiaBlackwell, "ffmpeg_nvenc"),
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

    // === 統合ティア別テスト ===

    #[test]
    fn test_effective_tier_s_flagship_ada() {
        // RTX 4090（Ada + Flagship）= TierS → P7のまま
        let mut context = create_test_context_with_grade(
            GpuGeneration::NvidiaAda,
            GpuGrade::Flagship,
            CpuTier::Middle,
        );
        context.platform = StreamingPlatform::Twitch; // H.264を使用するためTwitch
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(context.effective_tier(), EffectiveTier::TierS);
        assert_eq!(encoder.preset, "p7", "TierSはP7のまま");
        assert_eq!(encoder.multipass_mode, "quarter_res", "TierSはマルチパス有効");
    }

    #[test]
    fn test_effective_tier_a_mid_ada() {
        // RTX 4060（Ada + Mid）= TierA → P7のまま
        let mut context = create_test_context_with_grade(
            GpuGeneration::NvidiaAda,
            GpuGrade::Mid,
            CpuTier::Middle,
        );
        context.platform = StreamingPlatform::Twitch;
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(context.effective_tier(), EffectiveTier::TierA);
        assert_eq!(encoder.preset, "p7", "TierAはP7のまま");
        assert_eq!(encoder.multipass_mode, "quarter_res", "TierAはマルチパス有効");
    }

    #[test]
    fn test_effective_tier_b_entry_ada() {
        // RTX 4050（Ada + Entry）= TierB → P6に調整
        let mut context = create_test_context_with_grade(
            GpuGeneration::NvidiaAda,
            GpuGrade::Entry,
            CpuTier::Middle,
        );
        context.platform = StreamingPlatform::Twitch;
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(context.effective_tier(), EffectiveTier::TierB);
        assert_eq!(encoder.preset, "p6", "TierBはP7→P6に調整");
        assert_eq!(encoder.multipass_mode, "quarter_res", "TierBはマルチパス有効");
    }

    #[test]
    fn test_effective_tier_c_entry_ampere() {
        // RTX 3050（Ampere + Entry）= TierC → P5に調整
        let mut context = create_test_context_with_grade(
            GpuGeneration::NvidiaAmpere,
            GpuGrade::Entry,
            CpuTier::Middle,
        );
        context.platform = StreamingPlatform::Twitch;
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(context.effective_tier(), EffectiveTier::TierC);
        assert_eq!(encoder.preset, "p5", "TierCはP6→P5に調整");
        assert_eq!(encoder.multipass_mode, "disabled", "TierCはマルチパス無効");
    }

    #[test]
    fn test_effective_tier_comparison_cross_generation() {
        // RTX 3090（Ampere Flagship）とRTX 4060（Ada Mid）は同じTierA
        let ampere_flagship = create_test_context_with_grade(
            GpuGeneration::NvidiaAmpere,
            GpuGrade::Flagship, // RTX 3090
            CpuTier::Middle,
        );
        let ada_mid = create_test_context_with_grade(
            GpuGeneration::NvidiaAda,
            GpuGrade::Mid, // RTX 4060
            CpuTier::Middle,
        );

        // 両方ともTierA
        assert_eq!(ampere_flagship.effective_tier(), EffectiveTier::TierA);
        assert_eq!(ada_mid.effective_tier(), EffectiveTier::TierA);
    }

    #[test]
    fn test_effective_tier_comparison_same_generation() {
        // 同一世代でもグレードが異なれば統合ティアが異なる
        let flagship = create_test_context_with_grade(
            GpuGeneration::NvidiaAmpere,
            GpuGrade::Flagship, // RTX 3090 → TierA
            CpuTier::Middle,
        );
        let entry = create_test_context_with_grade(
            GpuGeneration::NvidiaAmpere,
            GpuGrade::Entry, // RTX 3050 → TierC
            CpuTier::Middle,
        );

        let mut flagship_ctx = flagship;
        flagship_ctx.platform = StreamingPlatform::Twitch;
        let mut entry_ctx = entry;
        entry_ctx.platform = StreamingPlatform::Twitch;

        let flagship_encoder = EncoderSelector::select_encoder(&flagship_ctx);
        let entry_encoder = EncoderSelector::select_encoder(&entry_ctx);

        // 両方ともNVENCだが、プリセットが異なる
        assert_eq!(flagship_encoder.encoder_id, "ffmpeg_nvenc");
        assert_eq!(entry_encoder.encoder_id, "ffmpeg_nvenc");
        assert_ne!(flagship_encoder.preset, entry_encoder.preset,
            "RTX 3090とRTX 3050ではプリセットが異なる");
    }

    #[test]
    fn test_preset_minimum_clamp() {
        // TierE（Pascal + Entry）でもP1まで下がる
        let context = create_test_context_with_grade(
            GpuGeneration::NvidiaPascal, // 基本P4
            GpuGrade::Entry,              // TierE: -3調整
            CpuTier::Middle,
        );
        let mut ctx = context;
        ctx.platform = StreamingPlatform::Twitch;

        assert_eq!(ctx.effective_tier(), EffectiveTier::TierE);
        let encoder = EncoderSelector::select_encoder(&ctx);
        assert_eq!(encoder.preset, "p1", "P4-3=P1に調整（P1未満にはならない）");
    }

    // === AV1選択テスト ===

    #[test]
    fn test_av1_selection_youtube_ada_all_grades() {
        // Ada世代の全グレードでYouTubeならAV1を選択
        for grade in [GpuGrade::Flagship, GpuGrade::HighEnd, GpuGrade::UpperMid, GpuGrade::Mid, GpuGrade::Entry] {
            let context = create_test_context_with_grade(
                GpuGeneration::NvidiaAda,
                grade,
                CpuTier::Middle,
            );
            let encoder = EncoderSelector::select_encoder(&context);

            assert_eq!(encoder.encoder_id, "jim_av1_nvenc",
                "Ada {:?} + YouTube should select AV1", grade);
            assert_eq!(encoder.display_name, "AV1 (Hardware)");
            assert!(encoder.reason.contains("AV1"));
        }
    }

    #[test]
    fn test_av1_selection_youtube_blackwell_all_grades() {
        // Blackwell世代の全グレードでYouTubeならAV1を選択
        for grade in [GpuGrade::Flagship, GpuGrade::HighEnd, GpuGrade::UpperMid, GpuGrade::Mid, GpuGrade::Entry] {
            let context = create_test_context_with_grade(
                GpuGeneration::NvidiaBlackwell,
                grade,
                CpuTier::Middle,
            );
            let encoder = EncoderSelector::select_encoder(&context);

            assert_eq!(encoder.encoder_id, "jim_av1_nvenc",
                "Blackwell {:?} + YouTube should select AV1", grade);
            assert_eq!(encoder.display_name, "AV1 (Hardware)");
        }
    }

    #[test]
    fn test_av1_selection_intel_arc_youtube() {
        // Intel ArcでYouTubeならAV1を選択
        let context = create_test_context(GpuGeneration::IntelArc, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "obs_qsv11_av1");
        assert_eq!(encoder.display_name, "AV1 (Hardware)");
        assert_eq!(encoder.preset, "p7");
        assert!(encoder.reason.contains("AV1"));
    }

    #[test]
    fn test_no_av1_for_ampere() {
        // Ampere世代はAV1非対応なのでH.264を使用
        let context = create_test_context(GpuGeneration::NvidiaAmpere, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "ffmpeg_nvenc");
        assert!(!encoder.reason.contains("AV1"));
    }

    #[test]
    fn test_no_av1_for_turing() {
        // Turing世代はAV1非対応なのでH.264を使用
        let context = create_test_context(GpuGeneration::NvidiaTuring, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "ffmpeg_nvenc");
        assert!(!encoder.reason.contains("AV1"));
    }

    // === NVENCフォールバックテスト ===

    #[test]
    fn test_nvenc_fallback_when_not_youtube() {
        // AV1対応GPUでもYouTube以外ではH.264にフォールバック
        let platforms = vec![
            StreamingPlatform::Twitch,
            StreamingPlatform::NicoNico,
            StreamingPlatform::TwitCasting,
            StreamingPlatform::Other,
        ];

        for platform in platforms {
            let mut context = create_test_context(GpuGeneration::NvidiaAda, CpuTier::Middle);
            context.platform = platform;
            let encoder = EncoderSelector::select_encoder(&context);

            assert_eq!(encoder.encoder_id, "ffmpeg_nvenc",
                "{:?} should use H.264, not AV1", platform);
            assert_eq!(encoder.display_name, "NVIDIA NVENC H.264");
        }
    }

    #[test]
    fn test_nvenc_fallback_blackwell_non_youtube() {
        // Blackwell世代でもYouTube以外ではH.264
        let mut context = create_test_context(GpuGeneration::NvidiaBlackwell, CpuTier::Middle);
        context.platform = StreamingPlatform::NicoNico;
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "ffmpeg_nvenc");
        assert!(encoder.reason.contains("RTX 50"));
        assert!(!encoder.reason.contains("AV1"));
    }

    // === AMDエンコーダー選択テスト ===

    #[test]
    fn test_amd_vcn4_encoder_selection() {
        // VCN 4.0（RX 7000シリーズ）の選択
        let context = create_test_context(GpuGeneration::AmdVcn4, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "amd_amf_h264");
        assert_eq!(encoder.display_name, "AMD AMF H.264");
        assert_eq!(encoder.preset, "quality");
        assert_eq!(encoder.b_frames, Some(2), "VCN 4.0 supports B-frames");
        assert_eq!(encoder.rate_control, "CBR");
        assert!(!encoder.look_ahead);
        assert!(!encoder.psycho_visual_tuning);
        assert_eq!(encoder.multipass_mode, "disabled");
        assert!(encoder.reason.contains("RX 7000"));
    }

    #[test]
    fn test_amd_vcn3_encoder_selection() {
        // VCN 3.0（RX 6000シリーズ）の選択
        let context = create_test_context(GpuGeneration::AmdVcn3, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "amd_amf_h264");
        assert_eq!(encoder.display_name, "AMD AMF H.264");
        assert_eq!(encoder.preset, "quality");
        assert_eq!(encoder.b_frames, None, "VCN 3.0 does not support B-frames");
        assert!(encoder.reason.contains("RX 6000"));
    }

    #[test]
    fn test_amd_vcn4_all_platforms() {
        // AMD VCN 4.0は全プラットフォームでH.264を使用（AV1非対応）
        let platforms = vec![
            StreamingPlatform::YouTube,
            StreamingPlatform::Twitch,
            StreamingPlatform::NicoNico,
            StreamingPlatform::TwitCasting,
        ];

        for platform in platforms {
            let mut context = create_test_context(GpuGeneration::AmdVcn4, CpuTier::Middle);
            context.platform = platform;
            let encoder = EncoderSelector::select_encoder(&context);

            assert_eq!(encoder.encoder_id, "amd_amf_h264",
                "AMD VCN 4.0 on {:?} should use H.264", platform);
        }
    }

    // === Intel QuickSyncテスト ===

    #[test]
    fn test_intel_arc_h264_when_not_youtube() {
        // Intel ArcでYouTube以外ならH.264を使用
        let mut context = create_test_context(GpuGeneration::IntelArc, CpuTier::Middle);
        context.platform = StreamingPlatform::Twitch;
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "obs_qsv11");
        assert_eq!(encoder.display_name, "Intel QuickSync H.264");
        assert_eq!(encoder.preset, "balanced");
        assert_eq!(encoder.b_frames, Some(2));
        assert!(encoder.look_ahead, "Intel Arc supports look-ahead");
        assert_eq!(encoder.profile, "high");
        assert!(encoder.reason.contains("Intel Arc"));
    }

    #[test]
    fn test_intel_quicksync_integrated_gpu() {
        // Intel内蔵GPUの選択
        let context = create_test_context(GpuGeneration::IntelQuickSync, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "obs_qsv11");
        assert_eq!(encoder.display_name, "Intel QuickSync H.264");
        assert_eq!(encoder.preset, "balanced");
        assert_eq!(encoder.b_frames, Some(2));
        assert!(!encoder.look_ahead, "Integrated GPU does not have look-ahead");
        assert_eq!(encoder.profile, "main", "Integrated GPU uses 'main' profile for compatibility");
        assert!(encoder.reason.contains("内蔵GPU"));
    }

    #[test]
    fn test_intel_quicksync_all_platforms() {
        // Intel QuickSyncは全プラットフォームで同じ設定
        for platform in [StreamingPlatform::YouTube, StreamingPlatform::Twitch, StreamingPlatform::NicoNico] {
            let mut context = create_test_context(GpuGeneration::IntelQuickSync, CpuTier::Middle);
            context.platform = platform;
            let encoder = EncoderSelector::select_encoder(&context);

            assert_eq!(encoder.encoder_id, "obs_qsv11");
            assert_eq!(encoder.preset, "balanced");
        }
    }

    // === CPUフォールバックテスト ===

    #[test]
    fn test_cpu_fallback_no_gpu() {
        // GPU未検出時はx264を使用
        let context = create_test_context(GpuGeneration::None, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "obs_x264");
        assert_eq!(encoder.display_name, "x264 (CPU)");
        assert_eq!(encoder.preset, "veryfast");
        assert_eq!(encoder.rate_control, "CBR");
        assert_eq!(encoder.b_frames, Some(2));
        assert!(!encoder.look_ahead);
        assert_eq!(encoder.profile, "high");
    }

    #[test]
    fn test_cpu_fallback_unknown_gpu() {
        // GPU不明時もx264を使用
        let context = create_test_context(GpuGeneration::Unknown, CpuTier::UpperMiddle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "obs_x264");
        assert_eq!(encoder.preset, "faster");
    }

    #[test]
    fn test_x264_all_cpu_tiers() {
        // 全CPUティアでのx264プリセット確認
        let test_cases = vec![
            (CpuTier::Entry, "ultrafast", Some("zerolatency")),
            (CpuTier::Middle, "veryfast", None),
            (CpuTier::UpperMiddle, "faster", None),
            (CpuTier::HighEnd, "fast", None),
        ];

        for (cpu_tier, expected_preset, expected_tuning) in test_cases {
            let context = create_test_context(GpuGeneration::None, cpu_tier);
            let encoder = EncoderSelector::select_encoder(&context);

            assert_eq!(encoder.encoder_id, "obs_x264");
            assert_eq!(encoder.preset, expected_preset,
                "CPU tier {:?} should use preset {}", cpu_tier, expected_preset);
            assert_eq!(encoder.tuning.as_deref(), expected_tuning,
                "CPU tier {:?} tuning mismatch", cpu_tier);
        }
    }

    #[test]
    fn test_x264_entry_cpu_has_zerolatency() {
        // エントリーCPUではzerolatencyチューニングを使用
        let context = create_test_context(GpuGeneration::None, CpuTier::Entry);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.tuning, Some("zerolatency".to_string()),
            "Entry CPU should use zerolatency tuning for low latency");
        assert!(encoder.reason.contains("負荷が高い") || encoder.reason.contains("推奨"));
    }

    #[test]
    fn test_x264_high_end_cpu_no_tuning() {
        // ハイエンドCPUではチューニングなし（品質優先）
        let context = create_test_context(GpuGeneration::None, CpuTier::HighEnd);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.tuning, None,
            "High-end CPU should not use tuning for quality priority");
        assert!(encoder.reason.contains("高性能CPU") || encoder.reason.contains("高品質"));
    }

    // === ティア別プリセット調整テスト ===

    #[test]
    fn test_preset_adjustment_tier_s() {
        // TierS: プリセット調整なし、マルチパス有効
        let context = create_test_context_with_grade(
            GpuGeneration::NvidiaAda,
            GpuGrade::Flagship,
            CpuTier::Middle,
        );
        let mut ctx = context;
        ctx.platform = StreamingPlatform::Twitch; // H.264を使用

        assert_eq!(ctx.effective_tier(), EffectiveTier::TierS);
        let encoder = EncoderSelector::select_encoder(&ctx);

        assert_eq!(encoder.preset, "p7", "TierS should keep p7");
        assert_eq!(encoder.multipass_mode, "quarter_res", "TierS should enable multipass");
        assert!(encoder.reason.contains("最高性能"));
    }

    #[test]
    fn test_preset_adjustment_tier_a() {
        // TierA: プリセット調整なし、マルチパス有効
        let context = create_test_context_with_grade(
            GpuGeneration::NvidiaAmpere,
            GpuGrade::Flagship,
            CpuTier::Middle,
        );
        let mut ctx = context;
        ctx.platform = StreamingPlatform::Twitch;

        assert_eq!(ctx.effective_tier(), EffectiveTier::TierA);
        let encoder = EncoderSelector::select_encoder(&ctx);

        assert_eq!(encoder.preset, "p6", "TierA Ampere should keep p6");
        assert_eq!(encoder.multipass_mode, "quarter_res", "TierA should enable multipass");
        assert!(encoder.reason.contains("高性能"));
    }

    #[test]
    fn test_preset_adjustment_tier_b() {
        // TierB: -1段階調整、マルチパス有効
        let context = create_test_context_with_grade(
            GpuGeneration::NvidiaAmpere,
            GpuGrade::HighEnd,
            CpuTier::Middle,
        );
        let mut ctx = context;
        ctx.platform = StreamingPlatform::Twitch;

        assert_eq!(ctx.effective_tier(), EffectiveTier::TierB);
        let encoder = EncoderSelector::select_encoder(&ctx);

        assert_eq!(encoder.preset, "p5", "TierB should adjust p6 to p5");
        assert_eq!(encoder.multipass_mode, "quarter_res", "TierB should enable multipass");
        assert!(encoder.reason.contains("中上位") && encoder.reason.contains("1段階"));
    }

    #[test]
    fn test_preset_adjustment_tier_c() {
        // TierC: -1段階調整、マルチパス無効
        let context = create_test_context_with_grade(
            GpuGeneration::NvidiaAmpere,
            GpuGrade::Entry,
            CpuTier::Middle,
        );
        let mut ctx = context;
        ctx.platform = StreamingPlatform::Twitch;

        assert_eq!(ctx.effective_tier(), EffectiveTier::TierC);
        let encoder = EncoderSelector::select_encoder(&ctx);

        assert_eq!(encoder.preset, "p5", "TierC should adjust p6 to p5");
        assert_eq!(encoder.multipass_mode, "disabled", "TierC should disable multipass");
        assert!(encoder.reason.contains("中位") && encoder.reason.contains("1段階"));
    }

    #[test]
    fn test_preset_adjustment_tier_d() {
        // TierD: -2段階調整、マルチパス無効
        let context = create_test_context_with_grade(
            GpuGeneration::NvidiaTuring,
            GpuGrade::Entry,
            CpuTier::Middle,
        );
        let mut ctx = context;
        ctx.platform = StreamingPlatform::Twitch;

        assert_eq!(ctx.effective_tier(), EffectiveTier::TierD);
        let encoder = EncoderSelector::select_encoder(&ctx);

        assert_eq!(encoder.preset, "p3", "TierD should adjust p5 to p3");
        assert_eq!(encoder.multipass_mode, "disabled", "TierD should disable multipass");
        assert!(encoder.reason.contains("下位") && encoder.reason.contains("2段階"));
    }

    #[test]
    fn test_preset_adjustment_tier_e() {
        // TierE: -3段階調整、マルチパス無効
        let context = create_test_context_with_grade(
            GpuGeneration::NvidiaPascal,
            GpuGrade::Entry,
            CpuTier::Middle,
        );
        let mut ctx = context;
        ctx.platform = StreamingPlatform::Twitch;

        assert_eq!(ctx.effective_tier(), EffectiveTier::TierE);
        let encoder = EncoderSelector::select_encoder(&ctx);

        assert_eq!(encoder.preset, "p1", "TierE should adjust p4 to p1");
        assert_eq!(encoder.multipass_mode, "disabled", "TierE should disable multipass");
        assert!(encoder.reason.contains("エントリー") && encoder.reason.contains("3段階"));
    }

    // === 機能テスト（Psycho Visual Tuning, Look-ahead, B-frames） ===

    #[test]
    fn test_psycho_visual_tuning_enabled_for_turing_and_newer() {
        // Turing以降はPsycho Visual Tuning有効
        let turing_ctx = create_test_context_with_grade(
            GpuGeneration::NvidiaTuring,
            GpuGrade::HighEnd,
            CpuTier::Middle,
        );
        let mut turing = turing_ctx;
        turing.platform = StreamingPlatform::Twitch;

        let turing_encoder = EncoderSelector::select_encoder(&turing);
        assert!(turing_encoder.psycho_visual_tuning, "Turing should enable psycho visual tuning");

        // Ampere, Ada, Blackwellも確認
        for gen in [GpuGeneration::NvidiaAmpere, GpuGeneration::NvidiaAda, GpuGeneration::NvidiaBlackwell] {
            let mut ctx = create_test_context(gen, CpuTier::Middle);
            ctx.platform = StreamingPlatform::Twitch;
            let encoder = EncoderSelector::select_encoder(&ctx);
            assert!(encoder.psycho_visual_tuning, "{:?} should enable psycho visual tuning", gen);
        }
    }

    #[test]
    fn test_psycho_visual_tuning_disabled_for_pascal() {
        // PascalはPsycho Visual Tuning無効
        let mut context = create_test_context(GpuGeneration::NvidiaPascal, CpuTier::Middle);
        context.platform = StreamingPlatform::Twitch;
        let encoder = EncoderSelector::select_encoder(&context);

        assert!(!encoder.psycho_visual_tuning, "Pascal should not enable psycho visual tuning");
    }

    #[test]
    fn test_look_ahead_enabled_for_ampere_and_newer() {
        // Ampere以降はLook-ahead有効
        for gen in [GpuGeneration::NvidiaAmpere, GpuGeneration::NvidiaAda, GpuGeneration::NvidiaBlackwell] {
            let mut ctx = create_test_context(gen, CpuTier::Middle);
            ctx.platform = StreamingPlatform::Twitch;
            let encoder = EncoderSelector::select_encoder(&ctx);
            assert!(encoder.look_ahead, "{:?} should enable look-ahead", gen);
        }
    }

    #[test]
    fn test_look_ahead_disabled_for_turing_and_older() {
        // Turing以前はLook-ahead無効
        for gen in [GpuGeneration::NvidiaTuring, GpuGeneration::NvidiaPascal] {
            let mut ctx = create_test_context(gen, CpuTier::Middle);
            ctx.platform = StreamingPlatform::Twitch;
            let encoder = EncoderSelector::select_encoder(&ctx);
            assert!(!encoder.look_ahead, "{:?} should not enable look-ahead", gen);
        }
    }

    #[test]
    fn test_b_frames_supported_pascal_and_newer() {
        // Pascal以降はBフレーム対応（Pascalを除く）
        let test_cases = vec![
            (GpuGeneration::NvidiaTuring, Some(2)),
            (GpuGeneration::NvidiaAmpere, Some(2)),
            (GpuGeneration::NvidiaAda, Some(2)),
            (GpuGeneration::NvidiaBlackwell, Some(2)),
            (GpuGeneration::NvidiaPascal, None), // Pascalは非対応
        ];

        for (gen, expected_b_frames) in test_cases {
            let mut ctx = create_test_context(gen, CpuTier::Middle);
            ctx.platform = StreamingPlatform::Twitch;
            let encoder = EncoderSelector::select_encoder(&ctx);
            assert_eq!(encoder.b_frames, expected_b_frames,
                "{:?} B-frames expectation", gen);
        }
    }

    // === エッジケーステスト ===

    #[test]
    fn test_pascal_high_end_cpu_prefers_x264() {
        // Pascal + ハイエンドCPUならx264を優先
        let context = create_test_context(GpuGeneration::NvidiaPascal, CpuTier::HighEnd);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "obs_x264",
            "Pascal with high-end CPU should prefer x264");
        assert_eq!(encoder.preset, "fast");
        assert!(encoder.reason.contains("ハイエンドCPU") || encoder.reason.contains("x264"));
    }

    #[test]
    fn test_pascal_non_high_end_cpu_uses_nvenc() {
        // Pascal + 非ハイエンドCPUならNVENCを使用
        for cpu_tier in [CpuTier::Entry, CpuTier::Middle, CpuTier::UpperMiddle] {
            let context = create_test_context(GpuGeneration::NvidiaPascal, cpu_tier);
            let encoder = EncoderSelector::select_encoder(&context);

            assert_eq!(encoder.encoder_id, "ffmpeg_nvenc",
                "Pascal with {:?} CPU should use NVENC", cpu_tier);
        }
    }

    #[test]
    fn test_all_encoders_have_non_empty_reason() {
        // すべてのエンコーダー選択結果に理由が記載されている
        let test_cases = vec![
            (GpuGeneration::NvidiaBlackwell, StreamingPlatform::YouTube),
            (GpuGeneration::NvidiaAda, StreamingPlatform::Twitch),
            (GpuGeneration::NvidiaAmpere, StreamingPlatform::NicoNico),
            (GpuGeneration::NvidiaTuring, StreamingPlatform::YouTube),
            (GpuGeneration::NvidiaPascal, StreamingPlatform::Twitch),
            (GpuGeneration::AmdVcn4, StreamingPlatform::YouTube),
            (GpuGeneration::AmdVcn3, StreamingPlatform::Twitch),
            (GpuGeneration::IntelArc, StreamingPlatform::YouTube),
            (GpuGeneration::IntelQuickSync, StreamingPlatform::Twitch),
            (GpuGeneration::None, StreamingPlatform::YouTube),
            (GpuGeneration::Unknown, StreamingPlatform::Twitch),
        ];

        for (gpu_gen, platform) in test_cases {
            let mut ctx = create_test_context(gpu_gen, CpuTier::Middle);
            ctx.platform = platform;
            let encoder = EncoderSelector::select_encoder(&ctx);

            assert!(!encoder.reason.is_empty(),
                "{:?} on {:?} must have a reason", gpu_gen, platform);
        }
    }

    #[test]
    fn test_all_encoders_use_cbr() {
        // すべてのエンコーダーでCBRレート制御を使用
        let all_generations = vec![
            GpuGeneration::NvidiaBlackwell,
            GpuGeneration::NvidiaAda,
            GpuGeneration::NvidiaAmpere,
            GpuGeneration::NvidiaTuring,
            GpuGeneration::NvidiaPascal,
            GpuGeneration::AmdVcn4,
            GpuGeneration::AmdVcn3,
            GpuGeneration::IntelArc,
            GpuGeneration::IntelQuickSync,
            GpuGeneration::None,
        ];

        for gpu_gen in all_generations {
            let context = create_test_context(gpu_gen, CpuTier::Middle);
            let encoder = EncoderSelector::select_encoder(&context);

            assert_eq!(encoder.rate_control, "CBR",
                "{:?} should use CBR rate control", gpu_gen);
        }
    }

    #[test]
    fn test_av1_encoder_settings() {
        // AV1エンコーダーの詳細設定確認
        let context = create_test_context(GpuGeneration::NvidiaAda, CpuTier::Middle);
        let encoder = EncoderSelector::select_encoder(&context);

        assert_eq!(encoder.encoder_id, "jim_av1_nvenc");
        assert_eq!(encoder.preset, "p7", "AV1 should use high quality preset");
        assert_eq!(encoder.b_frames, Some(2));
        assert!(encoder.look_ahead, "AV1 should enable look-ahead");
        assert!(encoder.psycho_visual_tuning, "AV1 should enable psycho visual tuning");
        assert_eq!(encoder.multipass_mode, "quarter_res", "AV1 should use multipass");
        assert_eq!(encoder.tuning, Some("hq".to_string()), "AV1 should use HQ tuning");
        assert_eq!(encoder.profile, "main", "AV1 should use main profile");
    }

    #[test]
    fn test_encoder_display_names() {
        // エンコーダーの表示名が正しいことを確認
        let test_cases = vec![
            (GpuGeneration::NvidiaAda, StreamingPlatform::YouTube, "AV1 (Hardware)"),
            (GpuGeneration::NvidiaAda, StreamingPlatform::Twitch, "NVIDIA NVENC H.264"),
            (GpuGeneration::AmdVcn4, StreamingPlatform::YouTube, "AMD AMF H.264"),
            (GpuGeneration::IntelArc, StreamingPlatform::YouTube, "AV1 (Hardware)"),
            (GpuGeneration::IntelArc, StreamingPlatform::Twitch, "Intel QuickSync H.264"),
            (GpuGeneration::IntelQuickSync, StreamingPlatform::YouTube, "Intel QuickSync H.264"),
            (GpuGeneration::None, StreamingPlatform::YouTube, "x264 (CPU)"),
        ];

        for (gpu_gen, platform, expected_name) in test_cases {
            let mut ctx = create_test_context(gpu_gen, CpuTier::Middle);
            ctx.platform = platform;
            let encoder = EncoderSelector::select_encoder(&ctx);

            assert_eq!(encoder.display_name, expected_name,
                "{:?} on {:?} display name mismatch", gpu_gen, platform);
        }
    }

    #[test]
    fn test_tuning_values() {
        // チューニング値の確認
        let mut nvenc_ctx = create_test_context(GpuGeneration::NvidiaAmpere, CpuTier::Middle);
        nvenc_ctx.platform = StreamingPlatform::Twitch;
        let nvenc = EncoderSelector::select_encoder(&nvenc_ctx);
        assert_eq!(nvenc.tuning, Some("hq".to_string()), "NVENC should use HQ tuning");

        let av1_ctx = create_test_context(GpuGeneration::NvidiaAda, CpuTier::Middle);
        let av1 = EncoderSelector::select_encoder(&av1_ctx);
        assert_eq!(av1.tuning, Some("hq".to_string()), "AV1 should use HQ tuning");

        let x264_entry = create_test_context(GpuGeneration::None, CpuTier::Entry);
        let x264_e = EncoderSelector::select_encoder(&x264_entry);
        assert_eq!(x264_e.tuning, Some("zerolatency".to_string()),
            "x264 Entry should use zerolatency");

        let x264_high = create_test_context(GpuGeneration::None, CpuTier::HighEnd);
        let x264_h = EncoderSelector::select_encoder(&x264_high);
        assert_eq!(x264_h.tuning, None, "x264 HighEnd should not use tuning");
    }

    #[test]
    fn test_profile_settings() {
        // プロファイル設定の確認
        let test_cases = vec![
            (GpuGeneration::NvidiaAda, StreamingPlatform::YouTube, "main"), // AV1
            (GpuGeneration::NvidiaAmpere, StreamingPlatform::Twitch, "high"), // NVENC H.264
            (GpuGeneration::IntelQuickSync, StreamingPlatform::YouTube, "main"), // 内蔵GPU
            (GpuGeneration::IntelArc, StreamingPlatform::Twitch, "high"), // Arc H.264
            (GpuGeneration::None, StreamingPlatform::YouTube, "high"), // x264
        ];

        for (gpu_gen, platform, expected_profile) in test_cases {
            let mut ctx = create_test_context(gpu_gen, CpuTier::Middle);
            ctx.platform = platform;
            let encoder = EncoderSelector::select_encoder(&ctx);

            assert_eq!(encoder.profile, expected_profile,
                "{:?} on {:?} profile mismatch", gpu_gen, platform);
        }
    }
}
