// 静的設定（スペック非依存のベストプラクティス）
//
// PCスペックに関係なく推奨される固定設定値
// 参考: obs_guide.md

// 将来のUI/API拡張用メソッドの警告を抑制
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// レート制御方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RateControl {
    /// 固定ビットレート（配信向け）
    Cbr,
    /// 品質ベース（録画向け）
    Cqp,
    /// 可変ビットレート
    Vbr,
}

impl RateControl {
    /// OBS設定値として出力
    pub fn as_obs_value(&self) -> &'static str {
        match self {
            Self::Cbr => "CBR",
            Self::Cqp => "CQP",
            Self::Vbr => "VBR",
        }
    }
}

/// カラーフォーマット
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ColorFormat {
    /// 4:2:0サブサンプリング（配信標準）
    Nv12,
    /// 4:4:4サブサンプリング（録画向け、CPU負荷+3-4%）
    I444,
    /// 10bit HDR
    P010,
}

impl ColorFormat {
    /// OBS設定値として出力
    pub fn as_obs_value(&self) -> &'static str {
        match self {
            Self::Nv12 => "NV12",
            Self::I444 => "I444",
            Self::P010 => "P010",
        }
    }
}

/// カラースペース
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ColorSpace {
    /// SDR標準（配信/ゲーム）
    Rec709,
    /// PCゲーム録画向け
    Srgb,
    /// HDR
    Rec2100Pq,
}

impl ColorSpace {
    /// OBS設定値として出力
    pub fn as_obs_value(&self) -> &'static str {
        match self {
            Self::Rec709 => "709",
            Self::Srgb => "sRGB",
            Self::Rec2100Pq => "2100PQ",
        }
    }
}

/// カラーレンジ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ColorRange {
    /// 部分レンジ（16-235、配信標準）
    Partial,
    /// フルレンジ（0-255）
    Full,
}

impl ColorRange {
    /// OBS設定値として出力
    pub fn as_obs_value(&self) -> &'static str {
        match self {
            Self::Partial => "Partial",
            Self::Full => "Full",
        }
    }
}

/// H.264プロファイル
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum H264Profile {
    /// 最高効率（推奨）
    High,
    /// 中間
    Main,
    /// 互換性重視（ビデオ会議用、品質低い）
    Baseline,
}

impl H264Profile {
    /// OBS設定値として出力
    pub fn as_obs_value(&self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Main => "main",
            Self::Baseline => "baseline",
        }
    }
}

/// スペック非依存の静的推奨設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticSettings {
    // 音声設定
    /// サンプルレート（Hz）
    pub sample_rate: u32,
    /// 音声ビットレート（kbps）
    pub audio_bitrate: u32,

    // 映像固定設定
    /// キーフレーム間隔（秒）
    pub keyframe_interval: u32,
    /// レート制御
    pub rate_control: RateControl,
    /// カラーフォーマット
    pub color_format: ColorFormat,
    /// カラースペース
    pub color_space: ColorSpace,
    /// カラーレンジ
    pub color_range: ColorRange,
    /// H.264プロファイル
    pub profile: H264Profile,

    // エンコーダ詳細設定
    /// Bフレーム数
    pub b_frames: u32,
    /// Look-ahead（配信ではOFF推奨）
    pub look_ahead: bool,
    /// Psycho Visual Tuning（配信ではON推奨）
    pub psycho_visual_tuning: bool,
}

impl Default for StaticSettings {
    fn default() -> Self {
        Self::for_streaming()
    }
}

impl StaticSettings {
    /// 配信向けデフォルト設定
    pub fn for_streaming() -> Self {
        Self {
            sample_rate: 48000,
            audio_bitrate: 160,
            keyframe_interval: 2,
            rate_control: RateControl::Cbr,
            color_format: ColorFormat::Nv12,
            color_space: ColorSpace::Rec709,
            color_range: ColorRange::Partial,
            profile: H264Profile::High,
            b_frames: 2,
            look_ahead: false,
            psycho_visual_tuning: true,
        }
    }

    /// 録画向け設定
    pub fn for_recording() -> Self {
        Self {
            sample_rate: 48000,
            audio_bitrate: 320,
            keyframe_interval: 2,
            rate_control: RateControl::Cqp,
            color_format: ColorFormat::Nv12,
            color_space: ColorSpace::Rec709,
            color_range: ColorRange::Partial,
            profile: H264Profile::High,
            b_frames: 4,
            look_ahead: true,
            psycho_visual_tuning: false,
        }
    }

    /// 設定理由を生成
    pub fn reasons(&self) -> Vec<StaticSettingReason> {
        vec![
            StaticSettingReason {
                setting: "サンプルレート".to_string(),
                value: format!("{} Hz", self.sample_rate),
                reason: "Windows/デバイスのデフォルト、リサンプル回避".to_string(),
            },
            StaticSettingReason {
                setting: "音声ビットレート".to_string(),
                value: format!("{} kbps", self.audio_bitrate),
                reason: "ゲーム配信に十分な音質".to_string(),
            },
            StaticSettingReason {
                setting: "キーフレーム間隔".to_string(),
                value: format!("{} 秒", self.keyframe_interval),
                reason: "プラットフォーム標準、視聴者が最大2秒で追いつける".to_string(),
            },
            StaticSettingReason {
                setting: "レート制御".to_string(),
                value: self.rate_control.as_obs_value().to_string(),
                reason: match self.rate_control {
                    RateControl::Cbr => "配信向け：一定ビットレートで安定配信".to_string(),
                    RateControl::Cqp => "録画向け：品質ベースでビットレート無駄なし".to_string(),
                    RateControl::Vbr => "可変ビットレート".to_string(),
                },
            },
            StaticSettingReason {
                setting: "カラーフォーマット".to_string(),
                value: self.color_format.as_obs_value().to_string(),
                reason: "配信プラットフォームで4:2:0が強制".to_string(),
            },
            StaticSettingReason {
                setting: "カラースペース".to_string(),
                value: self.color_space.as_obs_value().to_string(),
                reason: "SDR配信の標準規格".to_string(),
            },
            StaticSettingReason {
                setting: "カラーレンジ".to_string(),
                value: self.color_range.as_obs_value().to_string(),
                reason: "配信プラットフォームとの互換性".to_string(),
            },
            StaticSettingReason {
                setting: "プロファイル".to_string(),
                value: self.profile.as_obs_value().to_string(),
                reason: "ハードウェアアクセラレーション最大活用".to_string(),
            },
            StaticSettingReason {
                setting: "Bフレーム".to_string(),
                value: self.b_frames.to_string(),
                reason: match self.rate_control {
                    RateControl::Cbr => "配信向け：一定値推奨".to_string(),
                    _ => "録画向け：高品質".to_string(),
                },
            },
            StaticSettingReason {
                setting: "Look-ahead".to_string(),
                value: if self.look_ahead { "ON" } else { "OFF" }.to_string(),
                reason: match self.rate_control {
                    RateControl::Cbr => "配信向け：Bフレーム数固定推奨".to_string(),
                    _ => "録画向け：品質最適化".to_string(),
                },
            },
            StaticSettingReason {
                setting: "Psycho Visual".to_string(),
                value: if self.psycho_visual_tuning { "ON" } else { "OFF" }.to_string(),
                reason: match self.rate_control {
                    RateControl::Cbr => "配信向け：ビットレート制限下で有効".to_string(),
                    _ => "録画向け：CQPでは不要".to_string(),
                },
            },
        ]
    }
}

/// 静的設定の理由説明
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticSettingReason {
    /// 設定名
    pub setting: String,
    /// 設定値
    pub value: String,
    /// 理由
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_defaults() {
        let settings = StaticSettings::for_streaming();
        assert_eq!(settings.sample_rate, 48000);
        assert_eq!(settings.rate_control, RateControl::Cbr);
        assert!(!settings.look_ahead);
        assert!(settings.psycho_visual_tuning);
    }

    #[test]
    fn test_recording_settings() {
        let settings = StaticSettings::for_recording();
        assert_eq!(settings.rate_control, RateControl::Cqp);
        assert!(settings.look_ahead);
        assert!(!settings.psycho_visual_tuning);
        assert_eq!(settings.b_frames, 4);
    }

    #[test]
    fn test_obs_values() {
        assert_eq!(RateControl::Cbr.as_obs_value(), "CBR");
        assert_eq!(ColorFormat::Nv12.as_obs_value(), "NV12");
        assert_eq!(ColorSpace::Rec709.as_obs_value(), "709");
        assert_eq!(ColorRange::Partial.as_obs_value(), "Partial");
        assert_eq!(H264Profile::High.as_obs_value(), "high");
    }

    #[test]
    fn test_reasons_generation() {
        let settings = StaticSettings::for_streaming();
        let reasons = settings.reasons();
        assert!(!reasons.is_empty());
        assert!(reasons.iter().any(|r| r.setting == "サンプルレート"));
        assert!(reasons.iter().any(|r| r.setting == "レート制御"));
    }
}
