// Clippy pedantic lint allowances for pragmatic development
// These lints are overly strict for this project's use cases
#![allow(clippy::cast_possible_wrap)]        // Unix timestamps won't overflow i64
#![allow(clippy::cast_possible_truncation)]  // Known-safe float to int casts
#![allow(clippy::cast_sign_loss)]            // Known-positive values
#![allow(clippy::cast_precision_loss)]       // Acceptable for display purposes
#![allow(clippy::cast_lossless)]             // f32 to f64 is fine with `as`
#![allow(clippy::missing_const_for_fn)]      // Const fn is optional optimization
#![allow(clippy::unnecessary_wraps)]         // Result types for future-proofing
#![allow(clippy::uninlined_format_args)]     // Style preference
#![allow(clippy::unnecessary_debug_formatting)] // Debug prints are intentional
#![allow(clippy::float_cmp)]                 // Test assertions with known values
#![allow(clippy::case_sensitive_file_extension_comparisons)] // Tests use known lowercase
#![allow(clippy::match_same_arms)]           // Intentional for future expansion
#![allow(clippy::manual_string_new)]         // "".to_string() is readable
#![allow(clippy::format_push_string)]        // format! + push_str is clear
#![allow(clippy::single_match_else)]         // match is clearer in some contexts
#![allow(clippy::manual_range_contains)]     // Comparison operators are readable
#![allow(clippy::module_inception)]          // Nested test module is standard

mod error;
mod commands;
mod obs;
mod monitor;
mod services;
mod storage;
mod tray;

// テストユーティリティモジュール
// - ユニットテスト（#[cfg(test)]）時にコンパイル
// - 統合テスト実行時は --features testing でコンパイル
#[cfg(any(test, feature = "testing"))]
pub mod testing;

pub use error::AppError;

// サービス層の公開API
// 統合テストや外部クレートからのアクセスを許可
pub use services::{
    // 問題分析エンジン
    ProblemAnalyzer,
    ProblemReport,
    ProblemCategory,
    // その他のサービス
    RecommendationEngine,
    HardwareInfo,
    RecommendedSettings,
};

// ストレージ層の公開API
// メトリクス履歴など、テストや外部クレートで必要な型を公開
pub use storage::{
    SystemMetricsSnapshot,
    ObsStatusSnapshot,
    MetricsHistoryStore,
    HistoricalMetrics,
    SessionSummary,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // トレーシングサブスクライバーの初期化
    // RUST_LOG環境変数でログレベルを制御可能（例: RUST_LOG=debug,obs_optimizer=trace）
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // システム監視コマンド
            commands::get_system_metrics,
            commands::get_process_metrics,
            commands::get_legacy_system_metrics,
            // OBS接続コマンド
            commands::connect_obs,
            commands::disconnect_obs,
            commands::get_obs_status,
            commands::get_saved_connection,
            // OBSシーン操作コマンド
            commands::get_scene_list,
            commands::set_current_scene,
            // OBS配信・録画コマンド
            commands::start_streaming,
            commands::stop_streaming,
            commands::start_recording,
            commands::stop_recording,
            // OBSプロファイルパラメータ操作（テスト用）
            commands::get_obs_profile_parameter,
            commands::set_obs_profile_parameter,
            commands::get_current_obs_profile,
            commands::get_obs_profile_list,
            // 設定管理コマンド
            commands::get_config,
            commands::save_app_config,
            // 最適化エンジンコマンド
            commands::get_obs_settings_command,
            commands::calculate_recommendations,
            commands::calculate_custom_recommendations,
            // アラート管理コマンド
            commands::get_active_alerts,
            commands::clear_all_alerts,
            // Phase 2a: プロファイル管理コマンド
            commands::get_profiles,
            commands::get_profile,
            commands::save_profile,
            commands::delete_profile,
            commands::apply_profile,
            commands::save_current_settings_as_profile,
            // Phase 2a: 最適化適用コマンド
            commands::apply_recommended_settings,
            commands::apply_custom_settings,
            commands::backup_current_settings,
            commands::restore_backup,
            commands::get_backups,
            commands::apply_optimization,
            // Phase 2a: 配信中モード管理コマンド
            commands::set_streaming_mode,
            commands::get_streaming_mode,
            // Phase 2b: 問題分析コマンド
            commands::analyze_problems,
            commands::analyze_settings,
            commands::get_problem_history,
            // Phase 2b: エクスポートコマンド
            commands::export_session_json,
            commands::export_session_csv,
            commands::generate_diagnostic_report,
            // Phase 2b: セッション履歴コマンド
            commands::get_sessions,
            commands::get_metrics_range,
        ])
        .setup(|app| {
            // システムトレイのセットアップ
            if let Err(e) = tray::setup_tray(app.handle()) {
                tracing::warn!(target: "tray", "システムトレイの初期化に失敗: {e}");
                // トレイの初期化失敗は致命的ではないため、アプリケーションは継続
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            // エラー詳細をログ出力してから終了
            tracing::error!(target: "app", "Failed to run Tauri application");
            tracing::error!(target: "app", "Error: {e}");
            tracing::error!(target: "app", "Error type: {}", std::any::type_name_of_val(&e));
            tracing::error!(target: "app", "Terminating process with exit code 1");
            std::process::exit(1);
        });
}
