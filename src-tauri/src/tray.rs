use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Emitter, Listener, Manager, Runtime,
};

pub fn create_tray<R: Runtime>(app: &App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let status = MenuItem::with_id(app, "status", "保险库状态: 已锁定", false, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
    let lock = MenuItem::with_id(app, "lock", "立即锁定", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&status, &show, &lock, &sep, &quit])?;

    let icon_bytes = include_bytes!("../icons/32x32.png");
    let icon = tauri::image::Image::new_owned(icon_bytes.to_vec(), 32, 32);

    // Store a cloned menu reference for dynamic updates
    // Menu is Arc-based internally so clone is cheap
    let menu_for_state = menu.clone();
    app.manage(Arc::new(menu_for_state));

    let _tray = TrayIconBuilder::with_id("main-tray")
        .tooltip("Bavu-Iru 密码管理器")
        .icon(icon)
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "lock" => {
                let keyring = app.state::<crate::crypto::keyring::Keyring>();
                if keyring.is_unlocked() {
                    keyring.lock();
                    let _ = app.emit("vault-locked", ());
                    update_tray_state(app, false);
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                    }
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

pub fn update_tray_state<R: Runtime>(app: &tauri::AppHandle<R>, unlocked: bool) {
    let menu = app.state::<Arc<Menu<R>>>();
    update_menu_items(&*menu, unlocked);
}

fn update_menu_items<R: Runtime>(menu: &Menu<R>, unlocked: bool) {
    if let Some(item) = menu.get("status") {
        if let tauri::menu::MenuItemKind::MenuItem(ref mi) = item {
            let _ = mi.set_text(if unlocked {
                "保险库状态: 已解锁"
            } else {
                "保险库状态: 已锁定"
            });
        }
    }

    if let Some(item) = menu.get("lock") {
        if let tauri::menu::MenuItemKind::MenuItem(ref mi) = item {
            let _ = mi.set_enabled(unlocked);
        }
    }
}
