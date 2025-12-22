mod error;
mod commands;
mod obs;
mod monitor;
mod services;
mod tray;

pub use error::AppError;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
            // OBSシーン操作コマンド
            commands::get_scene_list,
            commands::set_current_scene,
            // OBS配信・録画コマンド
            commands::start_streaming,
            commands::stop_streaming,
            commands::start_recording,
            commands::stop_recording,
        ])
        .setup(|app| {
            // システムトレイのセットアップ
            if let Err(e) = tray::setup_tray(app.handle()) {
                eprintln!("[WARNING] システムトレイの初期化に失敗: {e}");
                // トレイの初期化失敗は致命的ではないため、アプリケーションは継続
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            // エラー詳細をログ出力してから終了
            eprintln!("[FATAL] Failed to run Tauri application");
            eprintln!("Error: {e}");
            eprintln!("Error type: {}", std::any::type_name_of_val(&e));
            eprintln!("Terminating process with exit code 1");
            std::process::exit(1);
        });
}
