// OBS設定読み取り機能
//
// OBS WebSocket経由で現在の設定を取得し、
// 最適化エンジンで使用する構造化データに変換する

use crate::error::AppError;
use crate::obs::get_obs_client;
use serde::{Deserialize, Serialize};

/// OBSの現在の設定全体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsSettings {
    /// ビデオ設定
    pub video: VideoSettings,
    /// 音声設定
    pub audio: AudioSettings,
    /// 出力設定
    pub output: OutputSettings,
}

/// ビデオ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoSettings {
    /// 基本解像度（幅）
    pub base_width: u32,
    /// 基本解像度（高さ）
    pub base_height: u32,
    /// 出力解像度（幅）
    pub output_width: u32,
    /// 出力解像度（高さ）
    pub output_height: u32,
    /// フレームレート（分子）
    pub fps_numerator: u32,
    /// フレームレート（分母）
    pub fps_denominator: u32,
}

#[allow(dead_code)]
impl VideoSettings {
    /// フレームレートを計算
    pub fn fps(&self) -> f64 {
        if self.fps_denominator == 0 {
            return 0.0;
        }
        f64::from(self.fps_numerator) / f64::from(self.fps_denominator)
    }

    /// 解像度を文字列で取得（例: "1920x1080"）
    pub fn resolution_string(&self) -> String {
        format!("{}x{}", self.output_width, self.output_height)
    }

    /// ダウンスケールの有無を判定
    pub fn is_downscaled(&self) -> bool {
        self.base_width != self.output_width || self.base_height != self.output_height
    }

    /// ダウンスケール比率を計算（%）
    pub fn downscale_ratio(&self) -> f64 {
        if self.base_width == 0 || self.base_height == 0 {
            return 100.0;
        }
        let base_pixels = f64::from(self.base_width * self.base_height);
        let output_pixels = f64::from(self.output_width * self.output_height);
        (output_pixels / base_pixels) * 100.0
    }
}

/// 音声設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSettings {
    /// サンプルレート（Hz）
    pub sample_rate: u32,
    /// チャンネル数（1=モノラル、2=ステレオ）
    pub channels: u32,
}

#[allow(dead_code)]
impl AudioSettings {
    /// チャンネル設定を文字列で取得
    pub fn channel_string(&self) -> String {
        match self.channels {
            1 => "モノラル".to_string(),
            2 => "ステレオ".to_string(),
            n => format!("{n}チャンネル"),
        }
    }
}

/// 出力設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputSettings {
    /// エンコーダー名
    pub encoder: String,
    /// ビットレート（kbps）
    pub bitrate_kbps: u32,
    /// キーフレーム間隔（秒）
    pub keyframe_interval_secs: u32,
    /// プリセット（x264/x265の場合）
    pub preset: Option<String>,
    /// レート制御モード（CBR/VBR/CQP等）
    pub rate_control: Option<String>,
}

impl OutputSettings {
    /// エンコーダータイプを判定
    pub fn encoder_type(&self) -> EncoderType {
        let encoder_lower = self.encoder.to_lowercase();

        if encoder_lower.contains("nvenc") || encoder_lower.contains("nvidia") {
            EncoderType::NvencH264
        } else if encoder_lower.contains("qsv") {
            EncoderType::QuickSync
        } else if encoder_lower.contains("amd") || encoder_lower.contains("vce") {
            EncoderType::AmdVce
        } else if encoder_lower.contains("x264") {
            EncoderType::X264
        } else if encoder_lower.contains("x265") || encoder_lower.contains("hevc") {
            EncoderType::X265
        } else {
            EncoderType::Other
        }
    }

    /// ハードウェアエンコーダーを使用しているか
    pub fn is_hardware_encoder(&self) -> bool {
        matches!(
            self.encoder_type(),
            EncoderType::NvencH264 | EncoderType::QuickSync | EncoderType::AmdVce
        )
    }
}

/// エンコーダータイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EncoderType {
    /// NVIDIA NVENC (H.264)
    NvencH264,
    /// Intel QuickSync
    QuickSync,
    /// AMD VCE
    AmdVce,
    /// ソフトウェア x264
    X264,
    /// ソフトウェア x265 (HEVC)
    X265,
    /// その他
    Other,
}

/// 配信出力のエンコーダー設定を取得するための構造体
#[derive(Debug, Clone, Deserialize)]
struct StreamEncoderSettings {
    /// ビットレート (kbps)
    #[serde(default)]
    bitrate: Option<u32>,
    /// レート制御方式
    #[serde(default)]
    rate_control: Option<String>,
    /// プリセット
    #[serde(default)]
    preset: Option<String>,
    /// キーフレーム間隔
    #[serde(default, alias = "keyint_sec")]
    keyframe_interval: Option<u32>,
}

/// OBSの現在の設定を取得
///
/// # Returns
/// OBS設定全体。接続されていない場合はエラー。
pub async fn get_obs_settings() -> Result<ObsSettings, AppError> {
    let client = get_obs_client();

    if !client.is_connected().await {
        return Err(AppError::obs_state("OBSに接続されていません"));
    }

    // obws APIを使用して実際のOBS設定を取得
    let video_settings = get_video_settings_from_obs(&client).await?;
    let audio_settings = get_audio_settings_from_obs()?;
    let output_settings = get_output_settings_from_obs(&client).await?;

    Ok(ObsSettings {
        video: video_settings,
        audio: audio_settings,
        output: output_settings,
    })
}

/// ビデオ設定をOBSから取得
async fn get_video_settings_from_obs(client: &super::ObsClient) -> Result<VideoSettings, AppError> {
    // ObsClient の専用メソッドを使用
    let video = client.get_video_settings().await?;

    Ok(VideoSettings {
        base_width: video.base_width,
        base_height: video.base_height,
        output_width: video.output_width,
        output_height: video.output_height,
        fps_numerator: video.fps_numerator,
        fps_denominator: video.fps_denominator,
    })
}

/// 音声設定を取得（OBS WebSocket APIでは直接取得が制限されている）
fn get_audio_settings_from_obs() -> Result<AudioSettings, AppError> {
    // OBS WebSocket 5.x では音声設定の直接取得APIがないため、
    // 一般的なデフォルト値を返す（Windowsのデフォルトは48kHz）
    // TODO: OBS設定ファイルから読み取ることを検討
    Ok(AudioSettings {
        sample_rate: 48000,
        channels: 2,
    })
}

/// 出力設定をOBSから取得
///
/// 注意: OBS WebSocket の outputs().list() は NDI 等のプラグインがある環境で
/// obs_output_get_width でクラッシュするバグがある（OBS Issue #11645）
/// そのため、取得を試みて失敗した場合はデフォルト値にフォールバックする
async fn get_output_settings_from_obs(client: &super::ObsClient) -> Result<OutputSettings, AppError> {
    // 出力一覧の取得を試みる（NDI等のプラグインがあるとクラッシュする可能性あり）
    let outputs_result = client.get_output_list().await;

    match outputs_result {
        Ok(outputs) => {
            // ストリーム出力を探す
            let stream_output = outputs.iter()
                .find(|o| o.name.contains("stream") || o.name.contains("streaming"))
                .or_else(|| outputs.first());

            if let Some(output) = stream_output {
                // 出力の設定を取得
                let settings_result: Result<StreamEncoderSettings, _> =
                    client.get_output_settings(&output.name).await;

                if let Ok(settings) = settings_result {
                    return Ok(OutputSettings {
                        encoder: output.name.clone(),
                        bitrate_kbps: settings.bitrate.unwrap_or(6000),
                        keyframe_interval_secs: settings.keyframe_interval.unwrap_or(2),
                        preset: settings.preset,
                        rate_control: settings.rate_control,
                    });
                }
            }

            // 出力が見つからない場合はデフォルト値
            Ok(default_output_settings())
        }
        Err(e) => {
            // GetOutputList が失敗した場合（NDIプラグイン等によるクラッシュ回避後）
            // ログを出力してデフォルト値を返す
            eprintln!("[WARNING] outputs().list() failed (possibly due to plugin conflict): {}", e);
            Ok(default_output_settings())
        }
    }
}

/// デフォルトの出力設定
fn default_output_settings() -> OutputSettings {
    OutputSettings {
        encoder: "unknown".to_string(),
        bitrate_kbps: 6000,
        keyframe_interval_secs: 2,
        preset: None,
        rate_control: Some("CBR".to_string()),
    }
}

/// 推奨ビデオ設定をOBSに適用
///
/// # Arguments
/// * `output_width` - 出力解像度の幅
/// * `output_height` - 出力解像度の高さ
/// * `fps` - フレームレート
pub async fn apply_video_settings(
    output_width: u32,
    output_height: u32,
    fps: u32,
) -> Result<(), AppError> {
    let client = get_obs_client();

    if !client.is_connected().await {
        return Err(AppError::obs_state("OBSに接続されていません"));
    }

    // 現在のビデオ設定を取得してベース解像度を維持
    let current = client.get_video_settings().await?;

    // obws の SetVideoSettings を構築
    use obws::requests::config::SetVideoSettings;
    let settings = SetVideoSettings {
        fps_numerator: Some(fps),
        fps_denominator: Some(1),
        base_width: Some(current.base_width), // ベース解像度は維持
        base_height: Some(current.base_height),
        output_width: Some(output_width),
        output_height: Some(output_height),
    };

    client.set_video_settings(settings).await?;

    Ok(())
}

/// 推奨設定をまとめてOBSに適用
///
/// # Arguments
/// * `video` - ビデオ設定（解像度、FPS）
/// * `encoder_name` - 使用するエンコーダー名（省略時は現在のまま）
/// * `bitrate_kbps` - ビットレート（省略時は現在のまま）
#[allow(dead_code)]
pub async fn apply_recommended_settings_to_obs(
    video: Option<(u32, u32, u32)>, // (width, height, fps)
    _encoder_name: Option<&str>,
    _bitrate_kbps: Option<u32>,
) -> Result<ApplyResult, AppError> {
    let client = get_obs_client();

    if !client.is_connected().await {
        return Err(AppError::obs_state("OBSに接続されていません"));
    }

    let mut result = ApplyResult::default();

    // ビデオ設定を適用
    if let Some((width, height, fps)) = video {
        match apply_video_settings(width, height, fps).await {
            Ok(()) => {
                result.applied.push("video".to_string());
            }
            Err(e) => {
                result.failed.push(format!("video: {}", e));
            }
        }
    }

    // エンコーダー・ビットレートの設定は OBS WebSocket 5.x では
    // 直接設定するAPIが制限されているため、将来の実装とする
    // TODO: OBS設定ファイル経由での設定変更を検討

    Ok(result)
}

/// 設定適用結果
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResult {
    /// 適用成功した設定項目
    pub applied: Vec<String>,
    /// 適用失敗した設定項目とエラーメッセージ
    pub failed: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_settings_fps() {
        let settings = VideoSettings {
            base_width: 1920,
            base_height: 1080,
            output_width: 1920,
            output_height: 1080,
            fps_numerator: 60,
            fps_denominator: 1,
        };
        assert_eq!(settings.fps(), 60.0);
    }

    #[test]
    fn test_video_settings_fps_fractional() {
        let settings = VideoSettings {
            base_width: 1920,
            base_height: 1080,
            output_width: 1920,
            output_height: 1080,
            fps_numerator: 30000,
            fps_denominator: 1001,
        };
        // 29.97 FPS (NTSC)
        assert!((settings.fps() - 29.97).abs() < 0.01);
    }

    #[test]
    fn test_video_settings_fps_zero_denominator() {
        let settings = VideoSettings {
            base_width: 1920,
            base_height: 1080,
            output_width: 1920,
            output_height: 1080,
            fps_numerator: 60,
            fps_denominator: 0,
        };
        // ゼロ除算を回避
        assert_eq!(settings.fps(), 0.0);
    }

    #[test]
    fn test_video_settings_resolution_string() {
        let settings = VideoSettings {
            base_width: 1920,
            base_height: 1080,
            output_width: 1280,
            output_height: 720,
            fps_numerator: 30,
            fps_denominator: 1,
        };
        assert_eq!(settings.resolution_string(), "1280x720");
    }

    #[test]
    fn test_video_settings_resolution_string_4k() {
        let settings = VideoSettings {
            base_width: 3840,
            base_height: 2160,
            output_width: 3840,
            output_height: 2160,
            fps_numerator: 60,
            fps_denominator: 1,
        };
        assert_eq!(settings.resolution_string(), "3840x2160");
    }

    #[test]
    fn test_video_settings_downscale() {
        let settings = VideoSettings {
            base_width: 1920,
            base_height: 1080,
            output_width: 1280,
            output_height: 720,
            fps_numerator: 30,
            fps_denominator: 1,
        };
        assert!(settings.is_downscaled());
        assert!((settings.downscale_ratio() - 44.44).abs() < 0.1);
    }

    #[test]
    fn test_video_settings_no_downscale() {
        let settings = VideoSettings {
            base_width: 1920,
            base_height: 1080,
            output_width: 1920,
            output_height: 1080,
            fps_numerator: 60,
            fps_denominator: 1,
        };
        assert!(!settings.is_downscaled());
        assert_eq!(settings.downscale_ratio(), 100.0);
    }

    #[test]
    fn test_video_settings_downscale_ratio_zero_base() {
        let settings = VideoSettings {
            base_width: 0,
            base_height: 0,
            output_width: 1920,
            output_height: 1080,
            fps_numerator: 60,
            fps_denominator: 1,
        };
        // ゼロ除算を回避
        assert_eq!(settings.downscale_ratio(), 100.0);
    }

    #[test]
    fn test_audio_settings_channel_string() {
        let mono = AudioSettings {
            sample_rate: 48000,
            channels: 1,
        };
        assert_eq!(mono.channel_string(), "モノラル");

        let stereo = AudioSettings {
            sample_rate: 48000,
            channels: 2,
        };
        assert_eq!(stereo.channel_string(), "ステレオ");
    }

    #[test]
    fn test_audio_settings_channel_string_surround() {
        let surround = AudioSettings {
            sample_rate: 48000,
            channels: 6,
        };
        assert_eq!(surround.channel_string(), "6チャンネル");
    }

    #[test]
    fn test_audio_settings_sample_rates() {
        let rates = vec![44100, 48000, 96000];
        for rate in rates {
            let settings = AudioSettings {
                sample_rate: rate,
                channels: 2,
            };
            assert_eq!(settings.sample_rate, rate);
        }
    }

    #[test]
    fn test_encoder_type_detection() {
        let nvenc = OutputSettings {
            encoder: "ffmpeg_nvenc".to_string(),
            bitrate_kbps: 6000,
            keyframe_interval_secs: 2,
            preset: None,
            rate_control: None,
        };
        assert_eq!(nvenc.encoder_type(), EncoderType::NvencH264);
        assert!(nvenc.is_hardware_encoder());

        let x264 = OutputSettings {
            encoder: "obs_x264".to_string(),
            bitrate_kbps: 6000,
            keyframe_interval_secs: 2,
            preset: Some("veryfast".to_string()),
            rate_control: Some("CBR".to_string()),
        };
        assert_eq!(x264.encoder_type(), EncoderType::X264);
        assert!(!x264.is_hardware_encoder());
    }

    #[test]
    fn test_encoder_type_nvenc_nvidia() {
        let encoder = OutputSettings {
            encoder: "nvidia_h264".to_string(),
            bitrate_kbps: 6000,
            keyframe_interval_secs: 2,
            preset: None,
            rate_control: None,
        };
        assert_eq!(encoder.encoder_type(), EncoderType::NvencH264);
    }

    #[test]
    fn test_encoder_type_quicksync() {
        let encoder = OutputSettings {
            encoder: "obs_qsv11".to_string(),
            bitrate_kbps: 6000,
            keyframe_interval_secs: 2,
            preset: None,
            rate_control: None,
        };
        assert_eq!(encoder.encoder_type(), EncoderType::QuickSync);
        assert!(encoder.is_hardware_encoder());
    }

    #[test]
    fn test_encoder_type_amd() {
        let encoder = OutputSettings {
            encoder: "amd_amf_h264".to_string(),
            bitrate_kbps: 6000,
            keyframe_interval_secs: 2,
            preset: None,
            rate_control: None,
        };
        assert_eq!(encoder.encoder_type(), EncoderType::AmdVce);
        assert!(encoder.is_hardware_encoder());
    }

    #[test]
    fn test_encoder_type_x265() {
        let encoder = OutputSettings {
            encoder: "obs_x265".to_string(),
            bitrate_kbps: 4000,
            keyframe_interval_secs: 2,
            preset: Some("medium".to_string()),
            rate_control: Some("CRF".to_string()),
        };
        assert_eq!(encoder.encoder_type(), EncoderType::X265);
        assert!(!encoder.is_hardware_encoder());
    }

    #[test]
    fn test_encoder_type_hevc() {
        let encoder = OutputSettings {
            encoder: "ffmpeg_hevc".to_string(),
            bitrate_kbps: 4000,
            keyframe_interval_secs: 2,
            preset: None,
            rate_control: None,
        };
        assert_eq!(encoder.encoder_type(), EncoderType::X265);
    }

    #[test]
    fn test_encoder_type_other() {
        let encoder = OutputSettings {
            encoder: "unknown_encoder".to_string(),
            bitrate_kbps: 6000,
            keyframe_interval_secs: 2,
            preset: None,
            rate_control: None,
        };
        assert_eq!(encoder.encoder_type(), EncoderType::Other);
        assert!(!encoder.is_hardware_encoder());
    }

    #[test]
    fn test_encoder_type_case_insensitive() {
        let encoder = OutputSettings {
            encoder: "NVENC_H264".to_string(),
            bitrate_kbps: 6000,
            keyframe_interval_secs: 2,
            preset: None,
            rate_control: None,
        };
        assert_eq!(encoder.encoder_type(), EncoderType::NvencH264);
    }

    #[test]
    fn test_obs_settings_serialization() {
        let settings = ObsSettings {
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
        };

        let json = serde_json::to_string(&settings).expect("serialization failed");
        assert!(json.contains("video"));
        assert!(json.contains("audio"));
        assert!(json.contains("output"));
    }

    #[test]
    fn test_encoder_type_serialization() {
        let encoder_type = EncoderType::NvencH264;
        let json = serde_json::to_string(&encoder_type).expect("serialization failed");
        assert!(json.contains("nvencH264"));

        let deserialized: EncoderType = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(deserialized, EncoderType::NvencH264);
    }
}
