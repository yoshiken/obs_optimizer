use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};

use crate::error::AppError;

/// システムトレイアイコンのセットアップ
///
/// # Arguments
/// * `app` - Tauriアプリケーションハンドル
///
/// # Returns
/// * `Ok(())` - セットアップ成功
/// * `Err(AppError)` - セットアップ失敗
pub fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    // トレイメニューの作成
    let show_item = MenuItem::with_id(app, "show", "表示", true, None::<&str>)
        .map_err(|e| AppError::tray_error(&format!("メニュー項目の作成に失敗: {}", e)))?;

    let quit_item = MenuItem::with_id(app, "quit", "終了", true, None::<&str>)
        .map_err(|e| AppError::tray_error(&format!("メニュー項目の作成に失敗: {}", e)))?;

    let menu = Menu::with_items(app, &[&show_item, &quit_item])
        .map_err(|e| AppError::tray_error(&format!("メニューの作成に失敗: {}", e)))?;

    // トレイアイコンの作成
    let icon = app.default_window_icon()
        .ok_or_else(|| AppError::tray_error("デフォルトウィンドウアイコンが見つかりません"))?
        .clone();

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .icon(icon)
        .tooltip("OBS配信最適化ツール")
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "show" => {
                    if let Err(e) = toggle_window_visibility(app) {
                        eprintln!("ウィンドウの表示切替に失敗: {}", e);
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray: &TrayIcon<R>, event| {
            // 左クリックでウィンドウの表示/非表示をトグル
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Err(e) = toggle_window_visibility(app) {
                    eprintln!("ウィンドウの表示切替に失敗: {}", e);
                }
            }
        })
        .build(app)
        .map_err(|e| AppError::tray_error(&format!("トレイアイコンの作成に失敗: {}", e)))?;

    Ok(())
}

/// ウィンドウの表示/非表示をトグル
///
/// # Arguments
/// * `app` - Tauriアプリケーションハンドル
///
/// # Returns
/// * `Ok(())` - トグル成功
/// * `Err(AppError)` - トグル失敗
fn toggle_window_visibility<R: Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible()
            .map_err(|e| AppError::window_error(&format!("ウィンドウの可視性確認に失敗: {}", e)))?
        {
            // 表示中の場合は非表示にする
            window.hide()
                .map_err(|e| AppError::window_error(&format!("ウィンドウの非表示に失敗: {}", e)))?;
        } else {
            // 非表示の場合は表示して前面に持ってくる
            window.show()
                .map_err(|e| AppError::window_error(&format!("ウィンドウの表示に失敗: {}", e)))?;
            window.set_focus()
                .map_err(|e| AppError::window_error(&format!("ウィンドウのフォーカスに失敗: {}", e)))?;
        }
        Ok(())
    } else {
        Err(AppError::window_error("メインウィンドウが見つかりません"))
    }
}
