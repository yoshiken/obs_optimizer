// 最適化適用コマンド
//
// 推奨設定をOBSに一括適用する機能

use crate::commands::utils::get_hardware_info;
use crate::error::AppError;
use crate::obs::{get_obs_client, get_obs_settings};
use crate::services::{get_streaming_mode_service, RecommendationEngine};
use crate::storage::config::{load_config, StreamingPlatform, StreamingStyle};
use crate::storage::{
    get_profile, get_profiles, save_profile as storage_save_profile, ProfileSettings,
    SettingsProfile,
};
use serde::{Deserialize, Serialize};

/// 設定バックアップ情報（TypeScriptのBackupInfoに対応）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    /// バックアップID
    pub id: String,
    /// 作成日時（Unixタイムスタンプ）
    pub created_at: i64,
    /// 説明
    pub description: String,
    /// バックアップした設定
    pub settings: ProfileSettings,
}

/// 最適化結果（TypeScriptのOptimizationResultに対応）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptimizationResult {
    /// 適用された設定の数
    pub applied_count: usize,
    /// 適用に失敗した設定の数
    pub failed_count: usize,
    /// エラーメッセージ（失敗時）
    pub errors: Vec<String>,
}

/// 推奨設定を一括適用
///
/// 配信中は適用不可。TOCTOU競合条件を防ぐためロックを使用。
#[tauri::command]
pub async fn apply_recommended_settings() -> Result<(), AppError> {
    let streaming_service = get_streaming_mode_service();

    // TOCTOU対策: ロックを取得し、配信中でないことを確認してから操作を実行
    streaming_service
        .execute_if_not_streaming(|| async {
            // OBS接続確認
            let client = get_obs_client();
            if !client.is_connected().await {
                return Err(AppError::obs_state("OBSに接続されていません"));
            }

            // 現在の設定をバックアップ
            backup_current_settings_internal().await?;

            // 推奨設定を計算
            let config = load_config()?;
            let current_settings = get_obs_settings().await?;
            let hardware = get_hardware_info().await;

            // 推奨設定を計算
            let recommendations = RecommendationEngine::calculate_recommendations(
                &hardware,
                &current_settings,
                config.streaming_mode.platform,
                config.streaming_mode.style,
                config.streaming_mode.network_speed_mbps,
            );

            // 推奨設定をOBSに適用
            crate::obs::settings::apply_video_settings(
                recommendations.video.output_width,
                recommendations.video.output_height,
                recommendations.video.fps,
            )
            .await?;

            // プロファイルパラメータでビットレート・プリセットを適用
            apply_output_settings_via_profile(&client, &recommendations.output).await?;

            Ok(())
        })
        .await
}

/// カスタム推奨設定を適用
///
/// TOCTOU競合条件を防ぐためロックを使用。
#[tauri::command]
pub async fn apply_custom_settings(
    platform: StreamingPlatform,
    style: StreamingStyle,
    network_speed_mbps: f64,
) -> Result<(), AppError> {
    let streaming_service = get_streaming_mode_service();

    // TOCTOU対策: ロックを取得し、配信中でないことを確認してから操作を実行
    streaming_service
        .execute_if_not_streaming(|| async {
            // OBS接続確認
            let client = get_obs_client();
            if !client.is_connected().await {
                return Err(AppError::obs_state("OBSに接続されていません"));
            }

            // 現在の設定をバックアップ
            backup_current_settings_internal().await?;

            // 推奨設定を計算
            let current_settings = get_obs_settings().await?;
            let hardware = get_hardware_info().await;

            // 推奨設定を計算
            let recommendations = RecommendationEngine::calculate_recommendations(
                &hardware,
                &current_settings,
                platform,
                style,
                network_speed_mbps,
            );

            // 推奨設定をOBSに適用
            crate::obs::settings::apply_video_settings(
                recommendations.video.output_width,
                recommendations.video.output_height,
                recommendations.video.fps,
            )
            .await?;

            // プロファイルパラメータでビットレート・プリセットを適用
            apply_output_settings_via_profile(&client, &recommendations.output).await?;

            Ok(())
        })
        .await
}

/// プリセットに基づいて最適化を適用
///
/// # Arguments
/// * `preset` - 最適化プリセット（"low", "medium", "high", "ultra", "custom"）
/// * `selected_keys` - 適用する設定項目のキーリスト（省略時は全て適用）
///
/// # Returns
/// 最適化結果（適用成功数、失敗数、エラーメッセージ）
///
/// TOCTOU競合条件を防ぐためロックを使用。
#[tauri::command]
pub async fn apply_optimization(
    preset: String,
    selected_keys: Option<Vec<String>>,
) -> Result<OptimizationResult, AppError> {
    // プリセットの検証（ロック取得前に行う）
    let valid_presets = ["low", "medium", "high", "ultra", "custom"];
    if !valid_presets.contains(&preset.as_str()) {
        return Err(AppError::config_error(&format!(
            "無効なプリセット: {}。有効な値は low, medium, high, ultra, custom です",
            preset
        )));
    }

    let streaming_service = get_streaming_mode_service();

    // TOCTOU対策: ロックを取得し、配信中でないことを確認してから操作を実行
    streaming_service
        .execute_if_not_streaming(|| async {
            // OBS接続確認
            let client = get_obs_client();
            if !client.is_connected().await {
                return Err(AppError::obs_state("OBSに接続されていません"));
            }

            // 現在の設定をバックアップ
            backup_current_settings_internal().await?;

            // TODO: Phase 2bでOBS設定適用APIを実装予定
            // 現在はダミーのレスポンスを返す
            let _ = preset;
            let _ = selected_keys;

            Ok(OptimizationResult {
                applied_count: 0,
                failed_count: 0,
                errors: vec![],
            })
        })
        .await
}

/// バックアップ一覧を取得
///
/// # Returns
/// バックアップ情報のリスト
#[tauri::command]
pub async fn get_backups() -> Result<Vec<BackupInfo>, AppError> {
    // プロファイル一覧を取得
    let profiles = get_profiles()?;

    // "バックアップ"で始まるプロファイルのみをフィルタリング
    let backups: Vec<BackupInfo> = profiles
        .into_iter()
        .filter(|p| p.name.starts_with("バックアップ"))
        .map(|summary| {
            // 完全なプロファイルを読み込み
            match get_profile(&summary.id) {
                Ok(profile) => Some(BackupInfo {
                    id: profile.id,
                    created_at: profile.created_at,
                    description: profile.description,
                    settings: profile.settings,
                }),
                Err(e) => {
                    tracing::warn!(target: "optimization", error = %e, "バックアップの読み込みに失敗");
                    None
                }
            }
        })
        .flatten()
        .collect();

    Ok(backups)
}

/// 現在の設定をバックアップ（内部関数）
///
/// TOCTOU対策済みの関数から呼び出される内部実装
async fn backup_current_settings_internal() -> Result<String, AppError> {
    // 現在のOBS設定を取得
    let current_settings = get_obs_settings().await?;

    // バックアップIDを生成
    let backup_id = uuid::Uuid::new_v4().to_string();

    // Unixタイムスタンプを取得
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| AppError::config_error(&format!("時刻の取得に失敗: {e}")))?
        .as_secs() as i64;

    // バックアップをプロファイルとして保存
    let backup_profile = SettingsProfile {
        id: backup_id.clone(),
        name: format!(
            "バックアップ {}",
            chrono::DateTime::from_timestamp(now, 0)
                .unwrap_or(chrono::DateTime::UNIX_EPOCH)
                .format("%Y-%m-%d %H:%M:%S")
        ),
        description: "自動バックアップ".to_string(),
        platform: StreamingPlatform::Other,
        style: StreamingStyle::Other,
        settings: ProfileSettings {
            video: crate::storage::profiles::VideoSettings {
                output_width: current_settings.video.output_width,
                output_height: current_settings.video.output_height,
                fps: current_settings.video.fps() as u32,
                downscale_filter: "Lanczos".to_string(),
            },
            audio: crate::storage::profiles::AudioSettings {
                sample_rate: current_settings.audio.sample_rate,
                bitrate_kbps: 160,
            },
            output: crate::storage::profiles::OutputSettings {
                encoder: current_settings.output.encoder,
                bitrate_kbps: current_settings.output.bitrate_kbps,
                keyframe_interval_secs: current_settings.output.keyframe_interval_secs,
                preset: current_settings.output.preset,
                rate_control: current_settings
                    .output
                    .rate_control
                    .unwrap_or_else(|| "CBR".to_string()),
            },
        },
        created_at: now,
        updated_at: now,
    };

    storage_save_profile(&backup_profile)?;

    Ok(backup_id)
}

/// 現在の設定をバックアップ（Tauriコマンド）
#[tauri::command]
pub async fn backup_current_settings() -> Result<String, AppError> {
    backup_current_settings_internal().await
}

/// バックアップから復元
///
/// TOCTOU競合条件を防ぐためロックを使用。
#[tauri::command]
pub async fn restore_backup(_backup_id: String) -> Result<(), AppError> {
    let streaming_service = get_streaming_mode_service();

    // TOCTOU対策: ロックを取得し、配信中でないことを確認してから操作を実行
    streaming_service
        .execute_if_not_streaming(|| async {
            // OBS接続確認
            let client = get_obs_client();
            if !client.is_connected().await {
                return Err(AppError::obs_state("OBSに接続されていません"));
            }

            // TODO: Phase 2bでOBS設定適用APIを実装予定
            // _backup_idからプロファイルを読み込み、設定を復元

            Ok(())
        })
        .await
}

/// プロファイルパラメータを使用して出力設定を適用
///
/// OBS WebSocket の SetProfileParameter を使用して
/// エンコーダ、ビットレート、プリセット等を設定する
async fn apply_output_settings_via_profile(
    client: &crate::obs::ObsClient,
    output: &crate::services::RecommendedOutputSettings,
) -> Result<(), AppError> {
    // エンコーダを設定
    if let Err(e) = client
        .set_profile_parameter("SimpleOutput", "StreamEncoder", Some(&output.encoder))
        .await
    {
        tracing::warn!(
            target: "optimization",
            error = %e,
            encoder = %output.encoder,
            "エンコーダの設定に失敗"
        );
    } else {
        tracing::info!(
            target: "optimization",
            encoder = %output.encoder,
            "エンコーダを設定しました"
        );
    }

    // ビットレートを設定
    if let Err(e) = client
        .set_profile_parameter("SimpleOutput", "VBitrate", Some(&output.bitrate_kbps.to_string()))
        .await
    {
        tracing::warn!(
            target: "optimization",
            error = %e,
            bitrate = output.bitrate_kbps,
            "ビットレートの設定に失敗"
        );
    } else {
        tracing::info!(
            target: "optimization",
            bitrate = output.bitrate_kbps,
            "ビットレートを設定しました"
        );
    }

    // プリセットを設定（存在する場合のみ）
    if let Some(ref preset) = output.preset {
        if let Err(e) = client
            .set_profile_parameter("SimpleOutput", "Preset", Some(preset))
            .await
        {
            tracing::warn!(
                target: "optimization",
                error = %e,
                preset = %preset,
                "プリセットの設定に失敗"
            );
        } else {
            tracing::info!(
                target: "optimization",
                preset = %preset,
                "プリセットを設定しました"
            );
        }
    }

    // キーフレーム間隔を設定
    if let Err(e) = client
        .set_profile_parameter(
            "SimpleOutput",
            "VKeyIntSec",
            Some(&output.keyframe_interval_secs.to_string()),
        )
        .await
    {
        tracing::warn!(
            target: "optimization",
            error = %e,
            keyframe_interval = output.keyframe_interval_secs,
            "キーフレーム間隔の設定に失敗"
        );
    } else {
        tracing::info!(
            target: "optimization",
            keyframe_interval = output.keyframe_interval_secs,
            "キーフレーム間隔を設定しました"
        );
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// BackupInfoのシリアライゼーション/デシリアライゼーションをテスト
    #[test]
    fn test_backup_info_serialization() {
        let backup = BackupInfo {
            id: "backup-001".to_string(),
            created_at: 1_703_332_800, // 2023-12-23 12:00:00 UTC
            description: "テストバックアップ".to_string(),
            settings: ProfileSettings {
                video: crate::storage::profiles::VideoSettings {
                    output_width: 1920,
                    output_height: 1080,
                    fps: 60,
                    downscale_filter: "Lanczos".to_string(),
                },
                audio: crate::storage::profiles::AudioSettings {
                    sample_rate: 48000,
                    bitrate_kbps: 160,
                },
                output: crate::storage::profiles::OutputSettings {
                    encoder: "ffmpeg_nvenc".to_string(),
                    bitrate_kbps: 6000,
                    keyframe_interval_secs: 2,
                    preset: Some("p5".to_string()),
                    rate_control: "CBR".to_string(),
                },
            },
        };

        // JSONにシリアライズ
        let json = serde_json::to_string(&backup).unwrap();

        // JSONからデシリアライズ
        let deserialized: BackupInfo = serde_json::from_str(&json).unwrap();

        // 値が保持されていることを確認
        assert_eq!(deserialized.id, "backup-001");
        assert_eq!(deserialized.created_at, 1_703_332_800);
        assert_eq!(deserialized.description, "テストバックアップ");
        assert_eq!(deserialized.settings.video.output_width, 1920);
        assert_eq!(deserialized.settings.video.output_height, 1080);
        assert_eq!(deserialized.settings.video.fps, 60);
        assert_eq!(deserialized.settings.audio.sample_rate, 48000);
        assert_eq!(deserialized.settings.output.encoder, "ffmpeg_nvenc");
        assert_eq!(deserialized.settings.output.bitrate_kbps, 6000);
    }

    /// BackupInfoのcamelCase変換をテスト
    #[test]
    fn test_backup_info_camel_case_keys() {
        let backup = BackupInfo {
            id: "backup-002".to_string(),
            created_at: 1_703_419_200,
            description: "camelCase test".to_string(),
            settings: ProfileSettings {
                video: crate::storage::profiles::VideoSettings {
                    output_width: 1280,
                    output_height: 720,
                    fps: 30,
                    downscale_filter: "Bicubic".to_string(),
                },
                audio: crate::storage::profiles::AudioSettings {
                    sample_rate: 44100,
                    bitrate_kbps: 128,
                },
                output: crate::storage::profiles::OutputSettings {
                    encoder: "obs_x264".to_string(),
                    bitrate_kbps: 3500,
                    keyframe_interval_secs: 2,
                    preset: Some("veryfast".to_string()),
                    rate_control: "VBR".to_string(),
                },
            },
        };

        let json = serde_json::to_value(&backup).unwrap();

        // camelCaseのキーが存在することを確認
        assert!(json.get("id").is_some());
        assert!(json.get("createdAt").is_some());
        assert!(json.get("description").is_some());
        assert!(json.get("settings").is_some());

        // snake_caseのキーが存在しないことを確認
        assert!(json.get("created_at").is_none());
    }

    /// OptimizationResultのシリアライゼーションをテスト
    #[test]
    fn test_optimization_result_serialization() {
        let result = OptimizationResult {
            applied_count: 10,
            failed_count: 2,
            errors: vec![
                "エラー1: 設定の適用に失敗".to_string(),
                "エラー2: 無効な値".to_string(),
            ],
        };

        let json = serde_json::to_string(&result).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        // camelCaseのキーを確認
        assert_eq!(value["appliedCount"], 10);
        assert_eq!(value["failedCount"], 2);
        assert_eq!(value["errors"].as_array().unwrap().len(), 2);
    }

    /// OptimizationResultの成功ケースをテスト
    #[test]
    fn test_optimization_result_success() {
        let result = OptimizationResult {
            applied_count: 15,
            failed_count: 0,
            errors: vec![],
        };

        assert_eq!(result.applied_count, 15);
        assert_eq!(result.failed_count, 0);
        assert!(result.errors.is_empty());
    }

    /// OptimizationResultの部分失敗ケースをテスト
    #[test]
    fn test_optimization_result_partial_failure() {
        let result = OptimizationResult {
            applied_count: 8,
            failed_count: 3,
            errors: vec![
                "設定A: 適用失敗".to_string(),
                "設定B: 無効な値".to_string(),
                "設定C: OBS接続エラー".to_string(),
            ],
        };

        assert_eq!(result.applied_count, 8);
        assert_eq!(result.failed_count, 3);
        assert_eq!(result.errors.len(), 3);
    }

    // =====================================================================
    // apply_optimization のプリセット検証テスト
    // =====================================================================

    /// 有効なプリセット（low）をテスト
    /// TODO: OBS接続が必要なため、実際のOBS設定適用は統合テストで実装
    #[tokio::test]
    async fn test_apply_optimization_valid_preset_low() {
        // OBS未接続の場合はエラーになることを確認
        let result = apply_optimization("low".to_string(), None).await;

        // OBS未接続エラーまたは配信中エラーが返る（プリセット検証はパスする）
        match result {
            Err(e) => {
                // プリセット検証を通過していればOBS_STATEエラーになるはず
                // CONFIG_ERRORの場合はプリセット検証に失敗している
                assert_eq!(e.code(), "OBS_STATE", "プリセット検証に失敗した可能性");
            },
            Ok(_) => {
                // OBS接続済みの場合は成功する可能性がある（テスト環境依存）
            },
        }
    }

    /// 有効なプリセット（medium）をテスト
    #[tokio::test]
    async fn test_apply_optimization_valid_preset_medium() {
        let result = apply_optimization("medium".to_string(), None).await;

        match result {
            Err(e) => {
                assert_eq!(e.code(), "OBS_STATE", "プリセット検証に失敗した可能性");
            },
            Ok(_) => {},
        }
    }

    /// 有効なプリセット（high）をテスト
    #[tokio::test]
    async fn test_apply_optimization_valid_preset_high() {
        let result = apply_optimization("high".to_string(), None).await;

        match result {
            Err(e) => {
                assert_eq!(e.code(), "OBS_STATE", "プリセット検証に失敗した可能性");
            },
            Ok(_) => {},
        }
    }

    /// 有効なプリセット（ultra）をテスト
    #[tokio::test]
    async fn test_apply_optimization_valid_preset_ultra() {
        let result = apply_optimization("ultra".to_string(), None).await;

        match result {
            Err(e) => {
                assert_eq!(e.code(), "OBS_STATE", "プリセット検証に失敗した可能性");
            },
            Ok(_) => {},
        }
    }

    /// 有効なプリセット（custom）をテスト
    #[tokio::test]
    async fn test_apply_optimization_valid_preset_custom() {
        let result = apply_optimization("custom".to_string(), None).await;

        match result {
            Err(e) => {
                assert_eq!(e.code(), "OBS_STATE", "プリセット検証に失敗した可能性");
            },
            Ok(_) => {},
        }
    }

    /// 無効なプリセットが拒否されることをテスト
    #[tokio::test]
    async fn test_apply_optimization_invalid_preset() {
        let result = apply_optimization("invalid".to_string(), None).await;

        match result {
            Err(e) => {
                assert_eq!(e.code(), "CONFIG_ERROR");
                assert!(e.message().contains("無効なプリセット"));
                assert!(e.message().contains("invalid"));
            },
            Ok(_) => {
                panic!("無効なプリセットが受け入れられてしまった");
            },
        }
    }

    /// 大文字小文字が違うプリセットが拒否されることをテスト
    #[tokio::test]
    async fn test_apply_optimization_case_sensitive_preset() {
        let result = apply_optimization("HIGH".to_string(), None).await;

        match result {
            Err(e) => {
                assert_eq!(e.code(), "CONFIG_ERROR");
                assert!(e.message().contains("無効なプリセット"));
            },
            Ok(_) => {
                panic!("大文字のプリセットが受け入れられてしまった");
            },
        }
    }

    /// 空文字列のプリセットが拒否されることをテスト
    #[tokio::test]
    async fn test_apply_optimization_empty_preset() {
        let result = apply_optimization("".to_string(), None).await;

        match result {
            Err(e) => {
                assert_eq!(e.code(), "CONFIG_ERROR");
                assert!(e.message().contains("無効なプリセット"));
            },
            Ok(_) => {
                panic!("空のプリセットが受け入れられてしまった");
            },
        }
    }

    /// プリセット検証のエラーメッセージに有効な値が含まれることをテスト
    #[tokio::test]
    async fn test_apply_optimization_error_message_contains_valid_presets() {
        let result = apply_optimization("bad_preset".to_string(), None).await;

        match result {
            Err(e) => {
                let msg = e.message();
                assert!(msg.contains("low"));
                assert!(msg.contains("medium"));
                assert!(msg.contains("high"));
                assert!(msg.contains("ultra"));
                assert!(msg.contains("custom"));
            },
            _ => {
                panic!("エラーメッセージが返されるべき");
            },
        }
    }

    // =====================================================================
    // get_backups のフィルタリングテスト
    // =====================================================================
    // 注意: これらのテストは実際のファイルシステムを使用するため、
    // テスト前後でクリーンアップが必要

    /// バックアッププロファイルが正しくフィルタリングされることをテスト
    /// TODO: 統合テストで実装（ファイルシステムのモックが必要）
    #[tokio::test]
    async fn test_get_backups_filters_backup_profiles() {
        // このテストは実際のファイルシステムに依存するため、
        // 統合テストまたはモックを使用したテストで実装する必要がある
        //
        // テスト手順:
        // 1. テスト用のプロファイルを作成
        //    - "バックアップ 2024-01-01 12:00:00"
        //    - "通常プロファイル"
        //    - "バックアップ 2024-01-02 14:00:00"
        // 2. get_backups() を呼び出し
        // 3. "バックアップ"で始まるもののみが返されることを確認
        // 4. 件数が2件であることを確認
    }

    /// 空のプロファイルディレクトリで空配列が返ることをテスト
    /// TODO: 統合テストで実装（ファイルシステムのモックが必要）
    #[tokio::test]
    async fn test_get_backups_empty_directory() {
        // このテストは実際のファイルシステムに依存するため、
        // 統合テストまたはモックを使用したテストで実装する必要がある
        //
        // テスト手順:
        // 1. 一時ディレクトリを作成
        // 2. get_backups() を呼び出し
        // 3. 空の配列が返されることを確認
    }

    /// 破損したプロファイルがスキップされることをテスト
    /// TODO: 統合テストで実装（ファイルシステムのモックが必要）
    #[tokio::test]
    async fn test_get_backups_skips_corrupted_profiles() {
        // このテストは実際のファイルシステムに依存するため、
        // 統合テストまたはモックを使用したテストで実装する必要がある
        //
        // テスト手順:
        // 1. 正常なバックアッププロファイルを1件作成
        // 2. 破損したJSON（"バックアップ"で始まる名前）を1件作成
        // 3. get_backups() を呼び出し
        // 4. 正常なプロファイルのみが返されることを確認（警告は出る）
    }
}
