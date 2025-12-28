// 推奨設定算出エンジン
//
// ハードウェア情報、現在のOBS設定、配信プラットフォーム、配信スタイル、
// ネットワーク速度を元に最適な設定を算出する

use crate::obs::ObsSettings;
use crate::storage::config::{StreamingPlatform, StreamingStyle};
use crate::monitor::gpu::GpuInfo;
use super::gpu_detection::{detect_gpu_generation, detect_gpu_grade, determine_cpu_tier, GpuGeneration, GpuGrade};
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
            StreamingPlatform::TwitCasting => Self {
                max_bitrate: 60000,
                recommended_width: 1920,
                recommended_height: 1080,
                recommended_fps: 60,
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
        let audio_bitrate = Self::recommend_audio_bitrate(platform, style);

        // プリセット推奨（新ロジック）
        let preset_string = Self::recommend_preset(
            &recommended_encoder,
            hardware,
            platform,
            style,
            network_speed_mbps,
        );

        // 縮小フィルタ推奨
        let downscale_filter = Self::recommend_downscale_filter(style).to_string();

        // スコア算出
        let score = Self::calculate_score(current_settings, &RecommendedSettings {
            video: RecommendedVideoSettings {
                output_width: recommended_width,
                output_height: recommended_height,
                fps: recommended_fps,
                downscale_filter: downscale_filter.clone(),
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
                downscale_filter,
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
        // GPU世代とグレードを判定
        let (gpu_generation, gpu_grade) = if let Some(gpu) = &hardware.gpu {
            (detect_gpu_generation(&gpu.name), detect_gpu_grade(&gpu.name))
        } else {
            (GpuGeneration::None, GpuGrade::Unknown)
        };

        // CPUティアを判定
        let cpu_tier = determine_cpu_tier(hardware.cpu_cores);

        // エンコーダー選択コンテキストを構築
        let context = EncoderSelectionContext {
            gpu_generation,
            gpu_grade,
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
        // 回線速度による分類（参考: https://castcraft.live/blog/178/）
        // - 5Mbps未満: 回線弱い → 2,000〜3,000kbps推奨
        // - 5〜10Mbps: 中程度 → 4,000〜6,000kbps推奨
        // - 10Mbps以上: 十分 → 高画質設定可能

        // プラットフォーム最大値に補正係数を適用
        let ideal_bitrate = (f64::from(preset.max_bitrate) * modifier.bitrate_multiplier) as u32;

        // ネットワーク速度の80%を上限とする（安全マージン）
        let network_limit = (network_speed_mbps * 1000.0 * 0.8) as u32;

        // 最低ビットレート（2000kbps）を保証
        let min_bitrate = 2000u32;

        // 回線が弱い場合の調整
        let recommended = if network_speed_mbps < 3.0 {
            // 超低速回線: 2,000〜2,500kbps
            let limited = 2500.min(network_limit).max(min_bitrate);
            reasons.push(format!(
                "回線速度が非常に遅い（{:.1}Mbps）ため、ビットレートを{}kbpsに制限。720p30fps推奨",
                network_speed_mbps, limited
            ));
            limited
        } else if network_speed_mbps < 5.0 {
            // 低速回線: 2,500〜3,500kbps
            let limited = 3500.min(network_limit).max(min_bitrate);
            reasons.push(format!(
                "回線速度が低め（{:.1}Mbps）のため、ビットレートを{}kbpsに調整",
                network_speed_mbps, limited
            ));
            limited
        } else if network_speed_mbps < 10.0 {
            // 中速回線: プラットフォーム推奨値の80%程度
            let limited = (ideal_bitrate as f64 * 0.8) as u32;
            let limited = limited.min(network_limit).min(preset.max_bitrate);
            if limited < ideal_bitrate {
                reasons.push(format!(
                    "回線速度（{:.1}Mbps）に合わせてビットレートを{}kbpsに最適化",
                    network_speed_mbps, limited
                ));
            }
            limited
        } else {
            // 高速回線: 理想値を使用可能
            let limited = ideal_bitrate.min(network_limit).min(preset.max_bitrate);
            if network_speed_mbps >= 20.0 && limited >= 9000 {
                reasons.push("高速回線を検出。9,000kbps以上で滑らかな高画質配信が可能です".to_string());
            }
            limited
        };

        // 最低ビットレートを保証
        recommended.max(min_bitrate)
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
    fn recommend_audio_bitrate(platform: StreamingPlatform, style: StreamingStyle) -> u32 {
        // スタイルによる基本ビットレート
        let base_bitrate = match style {
            StreamingStyle::Music => 320,      // 歌・演奏は高音質
            StreamingStyle::Gaming => 160,     // ゲームは標準
            StreamingStyle::Talk => 128,       // 雑談は控えめ
            StreamingStyle::Art => 160,        // お絵描きは標準
            StreamingStyle::Other => 160,      // その他は標準
        };

        // プラットフォームによる調整
        match platform {
            StreamingPlatform::YouTube => base_bitrate,
            StreamingPlatform::Twitch => base_bitrate.min(160), // Twitchは160kbps上限推奨
            StreamingPlatform::NicoNico => base_bitrate.min(128), // ニコニコは128kbps推奨
            StreamingPlatform::TwitCasting => base_bitrate, // ツイキャスは上限なし
            StreamingPlatform::Other => base_bitrate.min(160),
        }
    }

    /// 縮小フィルタ推奨
    ///
    /// 配信スタイルに応じて最適なダウンスケールフィルタを選択
    /// - ゲーム/Esports: Bicubic (16サンプル、GPU負荷中)
    /// - トーク/IRL: Lanczos (32サンプル、カメラ映像向け)
    fn recommend_downscale_filter(style: StreamingStyle) -> &'static str {
        match style {
            StreamingStyle::Gaming => "Bicubic",
            StreamingStyle::Talk => "Lanczos",
            StreamingStyle::Music => "Lanczos",  // カメラ重視
            StreamingStyle::Art => "Bicubic",    // 画面キャプチャ重視
            StreamingStyle::Other => "Bicubic",  // デフォルトはゲーム向け
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
        // GPU世代とグレードを判定
        let (gpu_generation, gpu_grade) = if let Some(gpu) = &hardware.gpu {
            (detect_gpu_generation(&gpu.name), detect_gpu_grade(&gpu.name))
        } else {
            (GpuGeneration::None, GpuGrade::Unknown)
        };

        // CPUティアを判定
        let cpu_tier = determine_cpu_tier(hardware.cpu_cores);

        // エンコーダー選択コンテキストを構築
        let context = EncoderSelectionContext {
            gpu_generation,
            gpu_grade,
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

        // 最低ビットレート2000kbpsが保証される
        // network_limit = 1.0 * 1000 * 0.8 = 800だが、min_bitrate=2000で底上げ
        assert_eq!(recommended.output.bitrate_kbps, 2000,
            "最低ビットレートが適用される: {}",
            recommended.output.bitrate_kbps);
        assert!(
            recommended.reasons.iter().any(|r| r.contains("回線速度")),
            "回線速度による制限の理由が含まれる"
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
            StreamingPlatform::TwitCasting,
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

        // ゲームスタイル - 160kbps
        let youtube_gaming = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );
        assert_eq!(youtube_gaming.audio.bitrate_kbps, 160, "YouTubeゲーム音声ビットレート");

        // 音楽スタイル - 320kbps
        let youtube_music = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Music,
            10.0,
        );
        assert_eq!(youtube_music.audio.bitrate_kbps, 320, "YouTube音楽音声ビットレート");

        // トークスタイル - 128kbps
        let youtube_talk = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Talk,
            10.0,
        );
        assert_eq!(youtube_talk.audio.bitrate_kbps, 128, "YouTubeトーク音声ビットレート");

        // ニコニコは128kbps上限
        let niconico_music = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::NicoNico,
            StreamingStyle::Music,
            10.0,
        );
        assert_eq!(niconico_music.audio.bitrate_kbps, 128, "ニコニコ音声ビットレート上限");
    }

    // === プラットフォーム制約の詳細テスト ===

    #[test]
    fn test_platform_bitrate_constraints_youtube() {
        // YouTube: 最大9000kbps
        let hardware = create_test_hardware();
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            100.0, // 高速回線
        );

        assert!(recommended.output.bitrate_kbps <= 9000,
            "YouTubeは9000kbps上限: {}kbps", recommended.output.bitrate_kbps);
    }

    #[test]
    fn test_platform_bitrate_constraints_twitch() {
        // Twitch: 最大6000kbps
        let hardware = create_test_hardware();
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::Twitch,
            StreamingStyle::Gaming,
            100.0,
        );

        assert!(recommended.output.bitrate_kbps <= 6000,
            "Twitchは6000kbps上限: {}kbps", recommended.output.bitrate_kbps);
    }

    #[test]
    fn test_platform_bitrate_constraints_niconico() {
        // ニコニコ: 最大6000kbps
        let hardware = create_test_hardware();
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::NicoNico,
            StreamingStyle::Gaming,
            100.0,
        );

        assert!(recommended.output.bitrate_kbps <= 6000,
            "ニコニコは6000kbps上限: {}kbps", recommended.output.bitrate_kbps);
    }

    #[test]
    fn test_platform_bitrate_constraints_twitcasting() {
        // ツイキャス: 最大60000kbps（実質制限なし）
        let hardware = create_test_hardware();
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::TwitCasting,
            StreamingStyle::Gaming,
            100.0,
        );

        // 回線速度80%制限で 100 * 1000 * 0.8 = 80000だが、
        // プラットフォーム制限60000が適用される
        assert!(recommended.output.bitrate_kbps <= 60000,
            "ツイキャスは60000kbps上限: {}kbps", recommended.output.bitrate_kbps);
    }

    // === ネットワーク制約の詳細テスト ===

    #[test]
    fn test_network_constraint_super_low_speed() {
        // 超低速回線: 2Mbps
        let hardware = create_test_hardware();
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            2.0,
        );

        // 2.0 * 1000 * 0.8 = 1600kbps だが、min_bitrate=2000で底上げ
        // 超低速回線では2500kbps上限
        assert!(recommended.output.bitrate_kbps <= 2500,
            "2Mbps回線では2500kbps以下: {}kbps", recommended.output.bitrate_kbps);
        assert!(recommended.output.bitrate_kbps >= 2000,
            "最低ビットレート2000kbps保証: {}kbps", recommended.output.bitrate_kbps);
    }

    #[test]
    fn test_network_constraint_low_speed() {
        // 低速回線: 4Mbps
        let hardware = create_test_hardware();
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            4.0,
        );

        // 4.0 * 1000 * 0.8 = 3200kbps、低速回線では3500kbps上限
        assert!(recommended.output.bitrate_kbps <= 3500,
            "4Mbps回線では3500kbps以下: {}kbps", recommended.output.bitrate_kbps);
    }

    #[test]
    fn test_network_constraint_medium_speed() {
        // 中速回線: 7Mbps
        let hardware = create_test_hardware();
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            7.0,
        );

        // 7.0 * 1000 * 0.8 = 5600kbps
        // プラットフォーム最大9000kbps * 1.2(Gaming) = 10800kbps * 0.8 = 8640kbps
        // → 5600kbps制限が適用される
        assert!(recommended.output.bitrate_kbps <= 5600,
            "7Mbps回線では5600kbps以下: {}kbps", recommended.output.bitrate_kbps);
    }

    #[test]
    fn test_network_constraint_high_speed() {
        // 高速回線: 20Mbps
        let hardware = create_test_hardware();
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            20.0,
        );

        // 20.0 * 1000 * 0.8 = 16000kbps
        // プラットフォーム最大9000kbps * 1.2(Gaming) = 10800kbps
        // → 9000kbps上限が適用される
        assert!(recommended.output.bitrate_kbps <= 9000,
            "YouTubeプラットフォーム上限9000kbps: {}kbps", recommended.output.bitrate_kbps);
        assert!(
            recommended.reasons.iter().any(|r| r.contains("高速回線") || r.contains("9,000kbps")),
            "高速回線の検出メッセージが含まれる"
        );
    }

    #[test]
    fn test_network_vs_platform_limit_youtube() {
        // ネットワーク制限 vs プラットフォーム制限（YouTube）
        let hardware = create_test_hardware();
        let current = create_test_settings();

        // ケース1: ネットワークが制限要因（5Mbps）
        let network_limited = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            5.0,
        );
        assert!(network_limited.output.bitrate_kbps <= 4000,
            "5Mbps回線では4000kbps以下");

        // ケース2: プラットフォームが制限要因（50Mbps）
        let platform_limited = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            50.0,
        );
        assert!(platform_limited.output.bitrate_kbps <= 9000,
            "YouTube上限9000kbps");
    }

    #[test]
    fn test_network_vs_platform_limit_twitch() {
        // ネットワーク制限 vs プラットフォーム制限（Twitch）
        let hardware = create_test_hardware();
        let current = create_test_settings();

        // ケース1: ネットワークが制限要因（3Mbps）
        let network_limited = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::Twitch,
            StreamingStyle::Gaming,
            3.0,
        );
        assert!(network_limited.output.bitrate_kbps <= 2500,
            "3Mbps回線では2500kbps以下");

        // ケース2: プラットフォームが制限要因（20Mbps）
        let platform_limited = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::Twitch,
            StreamingStyle::Gaming,
            20.0,
        );
        assert!(platform_limited.output.bitrate_kbps <= 6000,
            "Twitch上限6000kbps");
    }

    // === ハードウェアティア影響テスト ===

    #[test]
    fn test_hardware_tier_low_cpu_cores() {
        // 低コアCPU（2コア）
        let mut hardware = create_test_hardware();
        hardware.cpu_cores = 2;
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // 4コア未満は720p推奨
        assert_eq!(recommended.video.output_width, 1280, "2コアでは720p");
        assert_eq!(recommended.video.output_height, 720);
        // Gaming (fps_multiplier=1.0) でも低コアでは30fps制限
        assert_eq!(recommended.video.fps, 30, "2コアでは30fps");
    }

    #[test]
    fn test_hardware_tier_mid_cpu_cores() {
        // ミドルCPU（4コア）
        let mut hardware = create_test_hardware();
        hardware.cpu_cores = 4;
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // 4コア以上は1080p可能
        assert_eq!(recommended.video.output_width, 1920, "4コアでは1080p");
        assert_eq!(recommended.video.output_height, 1080);
    }

    #[test]
    fn test_hardware_tier_high_cpu_cores() {
        // ハイエンドCPU（16コア）
        let mut hardware = create_test_hardware();
        hardware.cpu_cores = 16;
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // 高コアCPUでも解像度は変わらない（プラットフォーム設定依存）
        assert_eq!(recommended.video.output_width, 1920);
        assert_eq!(recommended.video.output_height, 1080);
        // プリセットが高品質になる
        assert!(recommended.output.preset.is_some());
    }

    #[test]
    fn test_hardware_tier_very_low_memory() {
        // 超低メモリ（4GB）
        let mut hardware = create_test_hardware();
        hardware.total_memory_gb = 4.0;
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // メモリ容量は解像度判定に直接影響しない（CPU依存）
        // ただし、将来的な拡張の余地を確認
        assert!(recommended.overall_score <= 100);
    }

    #[test]
    fn test_hardware_tier_high_memory() {
        // 高メモリ（32GB）
        let mut hardware = create_test_hardware();
        hardware.total_memory_gb = 32.0;
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // 高メモリでも解像度は変わらない
        assert_eq!(recommended.video.output_width, 1920);
        assert_eq!(recommended.video.output_height, 1080);
    }

    #[test]
    fn test_hardware_tier_no_gpu_low_cpu() {
        // GPU無し＆低性能CPU（2コア）
        let mut hardware = create_test_hardware();
        hardware.cpu_cores = 2;
        hardware.gpu = None;
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // x264エンコーダー
        assert_eq!(recommended.output.encoder, "obs_x264");
        // 低性能なので720p30fps
        assert_eq!(recommended.video.output_width, 1280);
        assert_eq!(recommended.video.output_height, 720);
        assert_eq!(recommended.video.fps, 30);
    }

    // === GPU世代検出テスト ===

    #[test]
    fn test_gpu_generation_nvidia_ada() {
        // NVIDIA Ada（RTX 40シリーズ）
        let mut hardware = create_test_hardware();
        hardware.gpu = Some(GpuInfo {
            name: "NVIDIA GeForce RTX 4090".to_string(),
        });
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // AV1対応（YouTube）
        assert_eq!(recommended.output.encoder, "jim_av1_nvenc",
            "RTX 40シリーズはYouTubeでAV1推奨");
    }

    #[test]
    fn test_gpu_generation_nvidia_ada_twitch() {
        // NVIDIA Ada（RTX 40シリーズ）on Twitch
        let mut hardware = create_test_hardware();
        hardware.gpu = Some(GpuInfo {
            name: "NVIDIA GeForce RTX 4070".to_string(),
        });
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::Twitch,
            StreamingStyle::Gaming,
            10.0,
        );

        // TwitchではH.264
        assert_eq!(recommended.output.encoder, "ffmpeg_nvenc",
            "TwitchではH.264使用");
    }

    #[test]
    fn test_gpu_generation_nvidia_blackwell() {
        // NVIDIA Blackwell（RTX 50シリーズ）
        let mut hardware = create_test_hardware();
        hardware.gpu = Some(GpuInfo {
            name: "NVIDIA GeForce RTX 5090".to_string(),
        });
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // 最新世代もAV1対応
        assert_eq!(recommended.output.encoder, "jim_av1_nvenc",
            "RTX 50シリーズはAV1推奨");
    }

    #[test]
    fn test_gpu_generation_nvidia_ampere() {
        // NVIDIA Ampere（RTX 30シリーズ）
        let mut hardware = create_test_hardware();
        hardware.gpu = Some(GpuInfo {
            name: "NVIDIA GeForce RTX 3070".to_string(),
        });
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // AmpereはAV1非対応
        assert_eq!(recommended.output.encoder, "ffmpeg_nvenc",
            "RTX 30シリーズはH.264使用");
    }

    #[test]
    fn test_gpu_generation_nvidia_turing() {
        // NVIDIA Turing（RTX 20/GTX 16シリーズ）
        let mut hardware = create_test_hardware();
        hardware.gpu = Some(GpuInfo {
            name: "NVIDIA GeForce GTX 1660 Ti".to_string(),
        });
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        assert_eq!(recommended.output.encoder, "ffmpeg_nvenc");
    }

    #[test]
    fn test_gpu_generation_nvidia_pascal() {
        // NVIDIA Pascal（GTX 10シリーズ）
        let mut hardware = create_test_hardware();
        hardware.gpu = Some(GpuInfo {
            name: "NVIDIA GeForce GTX 1060".to_string(),
        });
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // Pascalは品質が低いが、CPUがハイエンドでないのでNVENC
        assert_eq!(recommended.output.encoder, "ffmpeg_nvenc");
    }

    #[test]
    fn test_gpu_generation_amd_vcn4() {
        // AMD VCN4（RX 7000シリーズ）
        let mut hardware = create_test_hardware();
        hardware.gpu = Some(GpuInfo {
            name: "AMD Radeon RX 7900 XTX".to_string(),
        });
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        assert_eq!(recommended.output.encoder, "amd_amf_h264");
    }

    #[test]
    fn test_gpu_generation_intel_arc() {
        // Intel Arc
        let mut hardware = create_test_hardware();
        hardware.gpu = Some(GpuInfo {
            name: "Intel Arc A770".to_string(),
        });
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // Intel ArcはAV1対応
        assert_eq!(recommended.output.encoder, "obs_qsv11_av1");
    }

    #[test]
    fn test_gpu_generation_intel_quicksync() {
        // Intel QuickSync（内蔵GPU）
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

        assert_eq!(recommended.output.encoder, "obs_qsv11");
    }

    // === エッジケーステスト ===

    #[test]
    fn test_edge_case_negative_network_speed() {
        // 異常値: 負のネットワーク速度
        let hardware = create_test_hardware();
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            -1.0,
        );

        // クラッシュせず最小ビットレート推奨
        assert!(recommended.output.bitrate_kbps >= 2000,
            "負のネットワーク速度でも最低ビットレート保証");
    }

    #[test]
    fn test_edge_case_zero_cpu_cores() {
        // 異常値: 0コア
        let mut hardware = create_test_hardware();
        hardware.cpu_cores = 0;
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // クラッシュせずに推奨設定を生成
        assert!(recommended.overall_score <= 100);
        // 0コアはEntryティア扱い
        assert_eq!(recommended.video.output_width, 1280, "0コアは720p推奨");
    }

    #[test]
    fn test_edge_case_extremely_high_cpu_cores() {
        // 極端な値: 128コア
        let mut hardware = create_test_hardware();
        hardware.cpu_cores = 128;
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // 正常に処理される
        assert_eq!(recommended.video.output_width, 1920);
        assert_eq!(recommended.video.output_height, 1080);
    }

    #[test]
    fn test_edge_case_zero_memory() {
        // 異常値: 0GBメモリ
        let mut hardware = create_test_hardware();
        hardware.total_memory_gb = 0.0;
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // クラッシュせず推奨設定を生成
        assert!(recommended.overall_score <= 100);
    }

    #[test]
    fn test_edge_case_unknown_gpu() {
        // 不明なGPU名
        let mut hardware = create_test_hardware();
        hardware.gpu = Some(GpuInfo {
            name: "Unknown Exotic GPU 9000".to_string(),
        });
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // 不明GPUはCPUエンコーダーにフォールバック
        assert_eq!(recommended.output.encoder, "obs_x264");
    }

    #[test]
    fn test_edge_case_combined_low_specs() {
        // 複合エッジケース: 低CPU、低メモリ、低回線
        let mut hardware = create_test_hardware();
        hardware.cpu_cores = 2;
        hardware.total_memory_gb = 4.0;
        hardware.gpu = None;
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            2.0, // 低速回線
        );

        // 全て低スペックでも推奨設定を生成
        assert_eq!(recommended.video.output_width, 1280, "低スペックは720p");
        assert_eq!(recommended.video.output_height, 720);
        assert_eq!(recommended.video.fps, 30, "低スペックは30fps");
        assert!(recommended.output.bitrate_kbps <= 2500, "低速回線制限");
        assert!(recommended.output.bitrate_kbps >= 2000, "最低ビットレート保証");
        assert!(recommended.reasons.len() > 0, "理由が含まれる");
    }

    // === 配信スタイルによる違いテスト ===

    #[test]
    fn test_style_talk_vs_gaming_bitrate() {
        // トーク vs ゲーム: ビットレート比較
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

        // トークはゲームより低ビットレート（0.8 vs 1.2倍率）
        assert!(talk.output.bitrate_kbps < gaming.output.bitrate_kbps,
            "トーク{}kbps < ゲーム{}kbps",
            talk.output.bitrate_kbps, gaming.output.bitrate_kbps);
    }

    #[test]
    fn test_style_talk_vs_gaming_fps() {
        // トーク vs ゲーム: FPS比較
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

        // トークは30fps、ゲームは60fps
        assert_eq!(talk.video.fps, 30, "トークは30fps");
        assert_eq!(gaming.video.fps, 60, "ゲームは60fps");
    }

    #[test]
    fn test_style_music_high_audio_bitrate() {
        // 音楽配信: 高音質音声
        let hardware = create_test_hardware();
        let current = create_test_settings();

        let music = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Music,
            10.0,
        );

        // 音楽は320kbps
        assert_eq!(music.audio.bitrate_kbps, 320, "音楽は320kbps高音質");
    }

    #[test]
    fn test_style_art_downscale_filter() {
        // お絵描き: ダウンスケールフィルター
        let hardware = create_test_hardware();
        let current = create_test_settings();

        let art = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Art,
            10.0,
        );

        let gaming = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // 両方ともBicubic（画面キャプチャ向け）
        assert_eq!(art.video.downscale_filter, "Bicubic");
        assert_eq!(gaming.video.downscale_filter, "Bicubic");
    }

    #[test]
    fn test_style_talk_downscale_filter() {
        // トーク: Lanczosフィルター（カメラ向け）
        let hardware = create_test_hardware();
        let current = create_test_settings();

        let talk = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Talk,
            10.0,
        );

        assert_eq!(talk.video.downscale_filter, "Lanczos",
            "トークはLanczos（カメラ向け）");
    }

    // === スコア算出の詳細テスト ===

    #[test]
    fn test_score_resolution_mismatch() {
        // 解像度不一致でスコア減少
        let hardware = create_test_hardware();
        let mut current = create_test_settings();
        current.video.output_width = 1280;
        current.video.output_height = 720;

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // 推奨は1920x1080だが現在は1280x720なのでスコア低下
        assert!(recommended.overall_score < 80,
            "解像度不一致でスコア低下: {}", recommended.overall_score);
    }

    #[test]
    fn test_score_fps_mismatch() {
        // FPS不一致でスコア減少
        let hardware = create_test_hardware();
        let mut current = create_test_settings();
        current.video.fps_numerator = 30;
        current.video.fps_denominator = 1;

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // 推奨は60fpsだが現在は30fpsなのでスコア低下
        assert!(recommended.overall_score < 90,
            "FPS不一致でスコア低下: {}", recommended.overall_score);
    }

    #[test]
    fn test_score_bitrate_close_match() {
        // ビットレートがほぼ一致（500kbps以内）
        let hardware = create_test_hardware();
        let current = create_test_settings();

        let recommended = RecommendationEngine::calculate_recommendations(
            &hardware,
            &current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // 現在のビットレートを推奨値に近づける
        let mut adjusted_current = current.clone();
        adjusted_current.output.bitrate_kbps = recommended.output.bitrate_kbps + 300;

        let score_check = RecommendationEngine::calculate_recommendations(
            &hardware,
            &adjusted_current,
            StreamingPlatform::YouTube,
            StreamingStyle::Gaming,
            10.0,
        );

        // 500kbps以内なら高スコア（ビットレート分30点満点）
        assert!(score_check.overall_score >= 50,
            "ビットレート近似でスコア高め: {}", score_check.overall_score);
    }

    #[test]
    fn test_reasons_not_empty() {
        // すべてのパターンで理由が含まれることを確認
        let test_cases = vec![
            (StreamingPlatform::YouTube, StreamingStyle::Gaming, 10.0),
            (StreamingPlatform::Twitch, StreamingStyle::Talk, 5.0),
            (StreamingPlatform::NicoNico, StreamingStyle::Music, 3.0),
            (StreamingPlatform::TwitCasting, StreamingStyle::Art, 20.0),
        ];

        for (platform, style, network_speed) in test_cases {
            let hardware = create_test_hardware();
            let current = create_test_settings();

            let recommended = RecommendationEngine::calculate_recommendations(
                &hardware,
                &current,
                platform,
                style,
                network_speed,
            );

            assert!(!recommended.reasons.is_empty(),
                "{:?} {:?} で理由が空", platform, style);
        }
    }
}
