use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

static CLIPBOARD_CLEAR_SCHEDULED: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub fn copy_to_clipboard(text: &str, app: AppHandle) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;

    // Schedule clipboard clear after 30 seconds
    if !CLIPBOARD_CLEAR_SCHEDULED.swap(true, Ordering::SeqCst) {
        let app_clone = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(30));
            if let Err(e) = app_clone.clipboard().write_text("") {
                log::error!("Failed to clear clipboard: {}", e);
            }
            CLIPBOARD_CLEAR_SCHEDULED.store(false, Ordering::SeqCst);
        });
    }

    Ok(())
}