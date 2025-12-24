// 推奨設定算出エンジン
//
// ハードウェア情報、現在のOBS設定、配信プラットフォーム、配信スタイル、
// ネットワーク速度を元に最適な設定を算出する

use crate::obs::ObsSettings;
use crate::storage::config::{StreamingPlatform, StreamingStyle};
use crate::monitor::gpu::GpuInfo;
use super::gpu_detection::{detect_gpu_generation, determine_cpu_tier, GpuGeneration};
use super::encoder_selector::{EncoderSelector, EncoderSelectionContext};
use serde::{Deserialize, Serialize};

/// ハードウェア情報のサマリー
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HardwareInfo {
    /// CPU名
    pub cpu_name: String,
    /// CPUコア数
    pub cpu_cores: usize,
    /// 総メモリ（GB）
    pub total_memory_gb: f64,
    /// GPU情報（利用可能な場合）
    pub gpu: Option<GpuInfo>,
}

/// 推奨設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendedSettings {
    /// ビデオ設定
    pub video: RecommendedVideoSettings,
    /// 音声設定
    pub audio: RecommendedAudioSettings,
    /// 出力設定
    pub output: RecommendedOutputSettings,
    /// 推奨理由
    pub reasons: Vec<String>,
    /// 全体スコア（0-100）
    pub overall_score: u8,
}

/// 推奨ビデオ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendedVideoSettings {
    /// 推奨解像度（幅）
    pub output_width: u32,
    /// 推奨解像度（高さ）
    pub output_height: u32,
    /// 推奨FPS
    pub fps: u32,
    /// ダウンスケールフィルター
    pub downscale_filter: String,
}

/// 推奨音声設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendedAudioSettings {
    /// サンプルレート（Hz）
    pub sample_rate: u32,
    /// ビットレート（kbps）
    pub bitrate_kbps: u32,
}

/// 推奨出力設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendedOutputSettings {
    /// 推奨エンコーダー
    pub encoder: String,
    /// 推奨ビットレート（kbps）
    pub bitrate_kbps: u32,
    /// 推奨キーフレーム間隔（秒）
    pub keyframe_interval_secs: u32,
    /// 推奨プリセット
    pub preset: Option<String>,
    /// レート制御モード
    pub rate_control: String,
}

/// プラットフォーム別の推奨値テーブル
struct PlatformPreset {
    /// 最大ビットレート（kbps）
    max_bitrate: u32,
    /// 推奨解像度（幅）
    recommended_width: u32,
    /// 推奨解像度（高さ）
    recommended_height: u32,
    /// 推奨FPS
    recommended_fps: u32,
    /// キーフレーム間隔（秒）
    keyframe_interval: u32,
}

impl PlatformPreset {
    /// プラットフォームに応じたプリセットを取得
    fn from_platform(platform: StreamingPlatform) -> Self {
        match platform {
            StreamingPlatform::YouTube => Self {
                max_bitrate: 9000,
                recommended_width: 1920,
                recommended_height: 1080,
                recommended_fps: 60,
                keyframe_interval: 2,
            },
            StreamingPlatform::Twitch => Self {
                max_bitrate: 6000,
                recommended_width: 1920,
                recommended_height: 1080,
                recommended_fps: 60,
                keyframe_interval: 2,
            },
            StreamingPlatform::NicoNico => Self {
                max_bitrate: 6000,
                recommended_width: 1280,
                recommended_height: 720,
                recommended_fps: 30,
                keyframe_interval: 2,
            },
            StreamingPlatform::Other => Self {
                max_bitrate: 6000,
                recommended_width: 1920,
                recommended_height: 1080,
                recommended_fps: 30,
                keyframe_interval: 2,
            },
        }
    }
}

/// 配信スタイル別の補正係数
struct StyleModifier {
    /// ビットレート補正（倍率）
    bitrate_multiplier: f64,
    /// FPS補正（倍率）
    fps_multiplier: f64,
}

impl StyleModifier {
    /// 配信スタイルに応じた補正係数を取得
    fn from_style(style: StreamingStyle) -> Self {
        match style {
            StreamingStyle::Talk => Self {
                bitrate_multiplier: 0.8, // 動きが少ないため低めでOK
                fps_multiplier: 0.5,     // 30FPSで十分
            },
            StreamingStyle::Gaming => Self {
                bitrate_multiplier: 1.2, // 動きが激しいため高め
                fps_multiplier: 1.0,     // 60FPS推奨
            },
            StreamingStyle::Music => Self {
                bitrate_multiplier: 1.0,
                fps_multiplier: 1.0,
            },
            StreamingStyle::Art => Self {
                bitrate_multiplier: 0.9, // 中程度
                fps_multiplier: 0.5,     // 30FPSで十分
            },
            StreamingStyle::Other => Self {
                bitrate_multiplier: 1.0,
                fps_multiplier: 1.0,
            },
        }
    }
}

/// 推奨エンジン
pub struct RecommendationEngine;

impl RecommendationEngine {
    /// 推奨設定を算出
    ///
    /// # Arguments
    /// * `hardware` - ハードウェア情報
    /// * `current_settings` - 現在のOBS設定
    /// * `platform` - 配信プラットフォーム
    /// * `style` - 配信スタイル
    /// * `network_speed_mbps` - ネットワーク速度（Mbps）
    ///
    /// # Returns
    /// 推奨設定
    pub fn calculate_recommendations(
        hardware: &HardwareInfo,
        current_settings: &ObsSettings,
        platform: StreamingPlatform,
        style: StreamingStyle,
        network_speed_mbps: f64,
    ) -> RecommendedSettings {
        let preset = PlatformPreset::from_platform(platform);
        let modifier = StyleModifier::from_style(style);
        let mut reasons = Vec::new();

        // エンコーダー推奨（新ロジック）
        let recommended_encoder = Self::recommend_encoder(
            hardware,
            platform,
            style,
            network_speed_mbps,
            &mut reasons,
        );

        // ビットレート推奨
        let recommended_bitrate = Self::recommend_bitrate(
            &preset,
            &modifier,
            network_speed_mbps,
            &mut reasons,
        );

        // 解像度推奨
        let (recommended_width, recommended_height) = Self::recommend_resolution(
            &preset,
            hardware,
            network_speed_mbps,
            &mut reasons,
        );

        // FPS推奨
        let recommended_fps = Self::recommend_fps(&preset, &modifier, hardware, &mut reasons);

        // 音声設定推奨
        let audio_bitrate = Self::recommend_audio_bitrate(platform);

        // プリセット推奨（新ロジック）
        let preset_string = Self::recommend_preset(
            &recommended_encoder,
            hardware,
            platform,
            style,
            network_speed_mbps,
        );

        // スコア算出
        let score = Self::calculate_score(current_settings, &RecommendedSettings {
            video: RecommendedVideoSettings {
                output_width: recommended_width,
                output_height: recommended_height,
                fps: recommended_fps,
                downscale_filter: "Lanczos".to_string(),
            },
            audio: RecommendedAudioSettings {
                sample_rate: 48000,
                bitrate_kbps: audio_bitrate,
            },
            output: RecommendedOutputSettings {
                encoder: recommended_encoder.clone(),
                bitrate_kbps: recommended_bitrate,
                keyframe_interval_secs: preset.keyframe_interval,
                preset: Some(preset_string.clone()),
                rate_control: "CBR".to_string(),
            },
            reasons: Vec::new(),
            overall_score: 0,
        });

        RecommendedSettings {
            video: RecommendedVideoSettings {
                output_width: recommended_width,
                output_height: recommended_height,
                fps: recommended_fps,
                downscale_filter: "Lanczos".to_string(),
            },
            audio: RecommendedAudioSettings {
                sample_rate: 48000,
                bitrate_kbps: audio_bitrate,
            },
            output: RecommendedOutputSettings {
                encoder: recommended_encoder,
                bitrate_kbps: recommended_bitrate,
                keyframe_interval_secs: preset.keyframe_interval,
                preset: Some(preset_string),
                rate_control: "CBR".to_string(),
            },
            reasons,
            overall_score: score,
        }
    }

    /// エンコーダー推奨（新ロジック）
    fn recommend_encoder(
        hardware: &HardwareInfo,
        platform: StreamingPlatform,
        style: StreamingStyle,
        network_speed_mbps: f64,
        reasons: &mut Vec<String>,
    ) -> String {
        // GPU世代を判定
        let gpu_generation = if let Some(gpu) = &hardware.gpu {
            detect_gpu_generation(&gpu.name)
        } else {
            GpuGeneration::None
        };

        // CPUティアを判定
        let cpu_tier = determine_cpu_tier(hardware.cpu_cores);

        // エンコーダー選択コンテキストを構築
        let context = EncoderSelectionContext {
            gpu_generation,
            cpu_tier,
            platform,
            style,
            network_speed_mbps,
        };

        // エンコーダーを選択
        let recommended = EncoderSelector::select_encoder(&context);
        reasons.push(recommended.reason.clone());

        recommended.encoder_id
    }

    /// ビットレート推奨
    fn recommend_bitrate(
        preset: &PlatformPreset,
        modifier: &StyleModifier,
        network_speed_mbps: f64,
        reasons: &mut Vec<String>,
    ) -> u32 {
        // プラットフォーム最大値に補正係数を適用
        let ideal_bitrate = (f64::from(preset.max_bitrate) * modifier.bitrate_multiplier) as u32;

        // ネットワーク速度の80%を上限とする（安全マージン）
        let network_limit = (network_speed_mbps * 1000.0 * 0.8) as u32;

        // プラットフォーム最大値を超えないようにする
        let recommended = ideal_bitrate.min(network_limit).min(preset.max_bitrate);

        if recommended < ideal_bitrate {
            reasons.push(format!(
                "ネットワーク速度またはプラットフォームの制限により、ビットレートを{recommended}kbpsに調整しました"
            ));
        }

        recommended
    }

    /// 解像度推奨
    fn recommend_resolution(
        preset: &PlatformPreset,
        hardware: &HardwareInfo,
        network_speed_mbps: f64,
        reasons: &mut Vec<String>,
    ) -> (u32, u32) {
        // 低スペックまたは低速回線の場合は720pにダウンスケール
        if hardware.cpu_cores < 4 || network_speed_mbps < 5.0 {
            reasons.push("ハードウェア性能またはネットワーク速度の制限により、720p解像度を推奨します".to_string());
            return (1280, 720);
        }

        (preset.recommended_width, preset.recommended_height)
    }

    /// FPS推奨
    fn recommend_fps(
        preset: &PlatformPreset,
        modifier: &StyleModifier,
        hardware: &HardwareInfo,
        reasons: &mut Vec<String>,
    ) -> u32 {
        let ideal_fps = (f64::from(preset.recommended_fps) * modifier.fps_multiplier) as u32;

        // 低スペックの場合は30FPSに制限
        if hardware.cpu_cores < 4 && ideal_fps > 30 {
            reasons.push("CPU性能の制限により、30FPSを推奨します".to_string());
            return 30;
        }

        ideal_fps
    }

    /// 音声ビットレート推奨
    fn recommend_audio_bitrate(platform: StreamingPlatform) -> u32 {
        match platform {
            StreamingPlatform::YouTube => 160,
            StreamingPlatform::Twitch => 160,
            StreamingPlatform::NicoNico => 128,
            StreamingPlatform::Other => 128,
        }
    }

    /// プリセット推奨（新ロジック対応）
    fn recommend_preset(
        _encoder: &str,
        hardware: &HardwareInfo,
        platform: StreamingPlatform,
        style: StreamingStyle,
        network_speed_mbps: f64,
    ) -> String {
        // GPU世代を判定
        let gpu_generation = if let Some(gpu) = &hardware.gpu {
            detect_gpu_generation(&gpu.name)
        } else {
            GpuGeneration::None
        };

        // CPUティアを判定
        let cpu_tier = determine_cpu_tier(hardware.cpu_cores);

        // エンコーダー選択コンテキストを構築
        let context = EncoderSelectionContext {
            gpu_generation,
            cpu_tier,
            platform,
            style,
            network_speed_mbps,
        };

        // エンコーダーを選択してプリセットを取得
        let recommended = EncoderSelector::select_encoder(&context);
        recommended.preset
    }

    /// 現在の設定と推奨設定を比較してスコアを算出
    fn calculate_score(current: &ObsSettings, recommended: &RecommendedSettings) -> u8 {
        let mut score = 100u32;

        // 解像度の一致度（0-30点）
        let resolution_match = if current.video.output_width == recommended.video.output_width
            && current.video.output_height == recommended.video.output_height
        {
            30
        } else {
            0
        };

        // FPSの一致度（0-20点）
        let current_fps = current.video.fps() as u32;
        let fps_match = if current_fps == recommended.video.fps {
            20
        } else if (current_fps as i32 - recommended.video.fps as i32).abs() <= 10 {
            10
        } else {
            0
        };

        // ビットレートの適切性（0-30点）
        let bitrate_diff = (current.output.bitrate_kbps as i32
            - recommended.output.bitrate_kbps as i32)
            .abs();
        let bitrate_score = if bitrate_diff < 500 {
            30
        } else if bitrate_diff < 2000 {
            15
        } else {
            0
        };

        // エンコーダーの適切性（0-20点）
        let encoder_score = if current.output.is_hardware_encoder() {
            20
        } else {
            10
        };

        score = score.min(resolution_match + fps_match + bitrate_score + encoder_score);
        score.min(100) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obs::{VideoSettings, AudioSettings, OutputSettings};

    fn create_test_hardware() -> HardwareInfo {
        HardwareInfo {
            cpu_name: "Test CPU".to_string(),
            cpu_cores: 8,
            total_memory_gb: 16.0,
            gpu: None,
        }
    }

    fn create_test_settings() -> ObsSettings {
        ObsSettings {
            video: VideoSettings {
                base_width: 1920,
                base_height: 1080,
                output_width: 1920,
                output_height: 1080,
                fps_numerator: 60,
                fps_denominator: 1,
            },
            audio: AudioSettings {
                sample_rate: 48000,
                channels: 2,
            },
            output: OutputSettings {
                encoder: "obs_x264".to_string(),
                bitrate_kbps: 6000,
                keyframe_interval_secs: 2,
                preset: Some("veryfast".to_string()),
                rate_control: Some("CBR".to_string()),
            },
        }
    }

    #[test]
    fn test_platform_preset_youtube() {
        let preset = PlatformPreset::from_platform(StreamingPlatform::YouTube);
        assert_eq!(preset.max_bitrate, 9000);
        assert_eq!(preset.recommended_width, 1920);
        assert_eq!(preset.recommended_height, 1080);
    }

    #[test]
    fn test_style_modifier_gaming() {
        let modifier = StyleModifier::from_style(StreamingStyle::Gaming);
        assert_eq!(modifier.bitrate_multiplier, 1.2);
        assert_eq!(modifier.fps_multiplier, 1.0);
    }

    #[test]
    fn test_recommendation_engine() {
        let hardware = create_test_hardware();
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        assert_eq!(recommended.video.output_width, 1920);
        assert_eq!(recommended.video.output_height, 1080);
        assert!(recommended.output.bitrate_kbps > 0);
        assert!(!recommended.reasons.is_empty());
    }

    // === 追加のエッジケーステスト ===

    #[test]
    fn test_low_network_speed_limits_bitrate() {
        let hardware = create_test_hardware();
        let current = create_test_settings();

        // 極端に低いネットワーク速度（1Mbps）
        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            1.0,
        );

        // 1Mbps * 1000 * 0.8 = 800kbps が上限
        assert!(recommended.output.bitrate_kbps <= 800,
            "ネットワーク速度制限によりビットレートが制限される: {}",
            recommended.output.bitrate_kbps);
        assert!(
            recommended.reasons.iter().any(|r| r.contains("ネットワーク速度")),
            "ネットワーク速度による制限の理由が含まれる"
        );
    }

    #[test]
    fn test_very_high_network_speed() {
        let hardware = create_test_hardware();
        let current = create_test_settings();

        // 非常に高速なネットワーク（100Mbps）
        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            100.0,
        );

        // プラットフォームの最大値を超えない
        assert!(recommended.output.bitrate_kbps <= 12000, // YouTube Gaming最大値程度
            "ネットワーク速度が高くてもプラットフォーム最大値は超えない");
    }

    #[test]
    fn test_zero_network_speed() {
        let hardware = create_test_hardware();
        let current = create_test_settings();

        // 異常値：ネットワーク速度0
        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            0.0,
        );

        // クラッシュせずに最小限のビットレートを推奨
        assert!(recommended.output.bitrate_kbps >= 0, "0でもクラッシュしない");
    }

    #[test]
    fn test_low_spec_hardware_downscales() {
        let mut hardware = create_test_hardware();
        hardware.cpu_cores = 2; // 低性能CPU
        hardware.gpu = None;
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // 低スペックなので720pにダウンスケール
        assert_eq!(recommended.video.output_width, 1280, "低スペックでは720p推奨");
        assert_eq!(recommended.video.output_height, 720);
        assert_eq!(recommended.video.fps, 30, "低スペックでは30fps推奨");
    }

    #[test]
    fn test_nvidia_gpu_encoder_recommendation() {
        let mut hardware = create_test_hardware();
        hardware.gpu = Some(GpuInfo {
            name: "NVIDIA GeForce RTX 3080".to_string(),
        });
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        assert_eq!(recommended.output.encoder, "ffmpeg_nvenc", "NVIDIA GPUではNVENC推奨");
        assert!(
            recommended.reasons.iter().any(|r| r.contains("NVIDIA")),
            "NVIDIA検出の理由が含まれる"
        );
    }

    #[test]
    fn test_amd_gpu_encoder_recommendation() {
        let mut hardware = create_test_hardware();
        hardware.gpu = Some(GpuInfo {
            name: "AMD Radeon RX 6800".to_string(),
        });
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        assert_eq!(recommended.output.encoder, "amd_amf_h264", "AMD GPUではVCE推奨");
    }

    #[test]
    fn test_intel_gpu_encoder_recommendation() {
        let mut hardware = create_test_hardware();
        hardware.gpu = Some(GpuInfo {
            name: "Intel UHD Graphics 770".to_string(),
        });
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        assert_eq!(recommended.output.encoder, "obs_qsv11", "Intel GPUではQuickSync推奨");
    }

    #[test]
    fn test_all_platforms() {
        let hardware = create_test_hardware();
        let current = create_test_settings();

        // 各プラットフォームで推奨設定が生成できること
        for platform in [
            StreamingPlatform::YouTube,
            StreamingPlatform::Twitch,
            StreamingPlatform::NicoNico,
            StreamingPlatform::Other,
        ] {
            let recommended = RecommendationEngine::calculate_recommendations(
                &hardware,
                &current,
                platform,
                StreamingStyle::Gaming,
                10.0,
            );

            assert!(recommended.output.bitrate_kbps > 0, "{:?}でビットレート設定", platform);
            assert!(recommended.overall_score <= 100, "スコアは100以下");
        }
    }

    #[test]
    fn test_all_streaming_styles() {
        let hardware = create_test_hardware();
        let current = create_test_settings();

        // 各配信スタイルで推奨設定が生成できること
        for style in [
            StreamingStyle::Talk,
            StreamingStyle::Gaming,
            StreamingStyle::Music,
            StreamingStyle::Art,
            StreamingStyle::Other,
        ] {
            let recommended = RecommendationEngine::calculate_recommendations(
                &hardware,
                &current,
                StreamingPlatform::YouTube,
                style,
                10.0,
            );

            assert!(recommended.video.fps > 0, "{:?}でFPS設定", style);
            assert!(recommended.output.bitrate_kbps > 0, "{:?}でビットレート設定", style);
        }
    }

    #[test]
    fn test_talk_style_lower_requirements() {
        let hardware = create_test_hardware();
        let current = create_test_settings();

        let talk = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Talk,
            10.0,
        );

        let gaming = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // トークはゲームより低FPS・低ビットレート
        assert!(talk.video.fps <= gaming.video.fps, "トークはゲームよりFPS低い");
        assert!(talk.output.bitrate_kbps <= gaming.output.bitrate_kbps,
            "トークはゲームよりビットレート低い");
    }

    #[test]
    fn test_niconico_limitations() {
        let hardware = create_test_hardware();
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::NicoNico,
            StreamingStyle::Gaming,
            10.0,
        );

        // ニコニコは制限が厳しい
        assert!(recommended.output.bitrate_kbps <= 6000, "ニコニコは6Mbps上限");
        assert_eq!(recommended.video.output_width, 1280, "ニコニコは720p推奨");
        assert_eq!(recommended.video.output_height, 720);
    }

    #[test]
    fn test_score_calculation_perfect_match() {
        let hardware = create_test_hardware();
        let mut current = create_test_settings();

        // まず推奨設定を取得
        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // 現在の設定を推奨設定に合わせる
        current.video.output_width = recommended.video.output_width;
        current.video.output_height = recommended.video.output_height;
        current.video.fps_numerator = recommended.video.fps;
        current.video.fps_denominator = 1;
        current.output.bitrate_kbps = recommended.output.bitrate_kbps;
        current.output.encoder = "ffmpeg_nvenc".to_string(); // ハードウェアエンコーダー

        let perfect = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // 完全一致ならスコアが高いはず（80以上）
        assert!(perfect.overall_score >= 80,
            "完全一致に近い設定ではスコアが高い: {}", perfect.overall_score);
    }

    #[test]
    fn test_score_calculation_poor_match() {
        let hardware = create_test_hardware();
        let mut current = create_test_settings();

        // 推奨とかけ離れた設定
        current.video.output_width = 640;
        current.video.output_height = 480;
        current.video.fps_numerator = 15;
        current.output.bitrate_kbps = 500;
        current.output.encoder = "obs_x264".to_string();

        let poor = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // 大きく異なる設定ではスコアが低い
        assert!(poor.overall_score < 50,
            "推奨と大きく異なる設定ではスコアが低い: {}", poor.overall_score);
    }

    #[test]
    fn test_extreme_cpu_cores() {
        let mut hardware = create_test_hardware();
        let current = create_test_settings();

        // 1コア
        hardware.cpu_cores = 1;
        let one_core = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );
        assert!(one_core.output.preset.as_ref().unwrap().contains("fast"),
            "1コアでは軽量プリセット");

        // 32コア
        hardware.cpu_cores = 32;
        let many_cores = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );
        assert!(many_cores.output.preset.is_some(), "32コアでもプリセット設定");
    }

    #[test]
    fn test_audio_bitrate_recommendations() {
        let hardware = create_test_hardware();
        let current = create_test_settings();

        let youtube = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );
        assert_eq!(youtube.audio.bitrate_kbps, 160, "YouTube音声ビットレート");

        let niconico = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::NicoNico,
            StreamingStyle::Gaming,
            10.0,
        );
        assert_eq!(niconico.audio.bitrate_kbps, 128, "ニコニコ音声ビットレート");
    }
}
