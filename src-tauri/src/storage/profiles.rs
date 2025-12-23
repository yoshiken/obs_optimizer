// プロファイル管理ストレージ
//
// 設定プロファイルをJSONファイルとして永続化
// 将来的にSQLiteに移行する可能性あり

use crate::error::AppError;
use crate::storage::config::{StreamingPlatform, StreamingStyle};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const APP_NAME: &str = "obs-optimizer";
const PROFILES_DIR: &str = "profiles";

/// 設定プロファイル
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsProfile {
    /// プロファイルID（UUID）
    pub id: String,
    /// プロファイル名
    pub name: String,
    /// 説明
    pub description: String,
    /// 配信プラットフォーム
    pub platform: StreamingPlatform,
    /// 配信スタイル
    pub style: StreamingStyle,
    /// 設定内容
    pub settings: ProfileSettings,
    /// 作成日時（Unixタイムスタンプ）
    pub created_at: i64,
    /// 更新日時（Unixタイムスタンプ）
    pub updated_at: i64,
}

/// プロファイル設定内容
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSettings {
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
    /// 出力解像度（幅）
    pub output_width: u32,
    /// 出力解像度（高さ）
    pub output_height: u32,
    /// FPS
    pub fps: u32,
    /// ダウンスケールフィルター
    pub downscale_filter: String,
}

/// 音声設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSettings {
    /// サンプルレート（Hz）
    pub sample_rate: u32,
    /// ビットレート（kbps）
    pub bitrate_kbps: u32,
}

/// 出力設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputSettings {
    /// エンコーダー
    pub encoder: String,
    /// ビットレート（kbps）
    pub bitrate_kbps: u32,
    /// キーフレーム間隔（秒）
    pub keyframe_interval_secs: u32,
    /// プリセット
    pub preset: Option<String>,
    /// レート制御モード
    pub rate_control: String,
}

/// プロファイル一覧の概要（一覧表示用）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSummary {
    /// プロファイルID
    pub id: String,
    /// プロファイル名
    pub name: String,
    /// 説明
    pub description: String,
    /// 配信プラットフォーム
    pub platform: StreamingPlatform,
    /// 配信スタイル
    pub style: StreamingStyle,
    /// 作成日時
    pub created_at: i64,
    /// 更新日時
    pub updated_at: i64,
}

impl From<&SettingsProfile> for ProfileSummary {
    fn from(profile: &SettingsProfile) -> Self {
        Self {
            id: profile.id.clone(),
            name: profile.name.clone(),
            description: profile.description.clone(),
            platform: profile.platform,
            style: profile.style,
            created_at: profile.created_at,
            updated_at: profile.updated_at,
        }
    }
}

/// プロファイルディレクトリのパスを取得
fn get_profiles_dir() -> Result<PathBuf, AppError> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| AppError::config_error("設定ディレクトリを取得できませんでした"))?;

    let profiles_dir = config_dir.join(APP_NAME).join(PROFILES_DIR);

    if !profiles_dir.exists() {
        std::fs::create_dir_all(&profiles_dir)?;
    }

    Ok(profiles_dir)
}

/// プロファイルファイルのパスを取得
fn get_profile_path(profile_id: &str) -> Result<PathBuf, AppError> {
    let profiles_dir = get_profiles_dir()?;
    Ok(profiles_dir.join(format!("{profile_id}.json")))
}

/// プロファイル一覧を取得
pub fn get_profiles() -> Result<Vec<ProfileSummary>, AppError> {
    let profiles_dir = get_profiles_dir()?;

    let mut summaries = Vec::new();

    // プロファイルディレクトリ内のJSONファイルを読み込み
    let entries = std::fs::read_dir(profiles_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // .jsonファイルのみ処理
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        // JSONファイルを読み込み
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str::<SettingsProfile>(&content) {
                    Ok(profile) => {
                        summaries.push(ProfileSummary::from(&profile));
                    }
                    Err(e) => {
                        // パースエラーは警告として出力し、スキップ
                        eprintln!("[WARNING] プロファイルのパースに失敗: {:?}, エラー: {}", path, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("[WARNING] プロファイルファイルの読み込みに失敗: {:?}, エラー: {}", path, e);
            }
        }
    }

    // 更新日時の降順でソート
    summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    Ok(summaries)
}

/// プロファイルを取得
pub fn get_profile(profile_id: &str) -> Result<SettingsProfile, AppError> {
    let path = get_profile_path(profile_id)?;

    if !path.exists() {
        return Err(AppError::config_error(&format!(
            "プロファイルが見つかりません: {profile_id}"
        )));
    }

    let content = std::fs::read_to_string(&path)?;
    let profile: SettingsProfile = serde_json::from_str(&content)?;

    Ok(profile)
}

/// プロファイルを保存
pub fn save_profile(profile: &SettingsProfile) -> Result<(), AppError> {
    let path = get_profile_path(&profile.id)?;

    let content = serde_json::to_string_pretty(profile)?;
    std::fs::write(&path, content)?;

    Ok(())
}

/// プロファイルを削除
pub fn delete_profile(profile_id: &str) -> Result<(), AppError> {
    let path = get_profile_path(profile_id)?;

    if !path.exists() {
        return Err(AppError::config_error(&format!(
            "プロファイルが見つかりません: {profile_id}"
        )));
    }

    std::fs::remove_file(&path)?;

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn create_test_profile() -> SettingsProfile {
        SettingsProfile {
            id: "test-profile-001".to_string(),
            name: "テストプロファイル".to_string(),
            description: "テスト用のプロファイル".to_string(),
            platform: StreamingPlatform::YouTube,
            style: StreamingStyle::Gaming,
            settings: ProfileSettings {
                video: VideoSettings {
                    output_width: 1920,
                    output_height: 1080,
                    fps: 60,
                    downscale_filter: "Lanczos".to_string(),
                },
                audio: AudioSettings {
                    sample_rate: 48000,
                    bitrate_kbps: 160,
                },
                output: OutputSettings {
                    encoder: "ffmpeg_nvenc".to_string(),
                    bitrate_kbps: 6000,
                    keyframe_interval_secs: 2,
                    preset: Some("p5".to_string()),
                    rate_control: "CBR".to_string(),
                },
            },
            created_at: 1_703_332_800, // 2023-12-23 12:00:00 UTC
            updated_at: 1_703_332_800,
        }
    }

    #[test]
    fn test_profile_serialization() {
        let profile = create_test_profile();
        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: SettingsProfile = serde_json::from_str(&json).unwrap();

        assert_eq!(profile.id, deserialized.id);
        assert_eq!(profile.name, deserialized.name);
        assert_eq!(profile.settings.video.output_width, deserialized.settings.video.output_width);
    }

    #[test]
    fn test_profile_summary_conversion() {
        let profile = create_test_profile();
        let summary = ProfileSummary::from(&profile);

        assert_eq!(summary.id, profile.id);
        assert_eq!(summary.name, profile.name);
        assert_eq!(summary.description, profile.description);
        assert_eq!(summary.platform, profile.platform);
        assert_eq!(summary.style, profile.style);
    }
}
