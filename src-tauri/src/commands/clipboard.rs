use std::sync::Mutex;
use tauri::Emitter;
use tauri::State;

use crate::error::AppError;

/// Tracks the active clipboard clear timer so we can cancel it if user copies again.
pub struct ClipboardState {
    cancel_token: Mutex<Option<tokio_util::sync::CancellationToken>>,
}

impl ClipboardState {
    pub fn new() -> Self {
        Self {
            cancel_token: Mutex::new(None),
        }
    }
}

#[tauri::command]
pub async fn clipboard_copy(
    app: tauri::AppHandle,
    state: State<'_, ClipboardState>,
    text: String,
    sensitive: bool,
    clear_seconds: Option<u64>,
) -> Result<(), AppError> {
    // Cancel any existing timer
    if let Some(token) = state.cancel_token.lock().unwrap().take() {
        token.cancel();
    }

    // Write to clipboard via the plugin
    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard()
        .write_text(&text)
        .map_err(|e| AppError::Clipboard(e.to_string()))?;

    // If sensitive, start auto-clear timer
    if sensitive {
        let seconds = clear_seconds.unwrap_or(30);
        let app_clone = app.clone();
        let token = tokio_util::sync::CancellationToken::new();
        let token_clone = token.clone();

        *state.cancel_token.lock().unwrap() = Some(token);

        tauri::async_runtime::spawn(async move {
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(seconds)) => {
                    let _ = app_clone.clipboard()
                        .write_text("")
                        .map_err(|e| AppError::Clipboard(e.to_string()));
                    let _ = app_clone.emit("clipboard-cleared", ());
                }
                _ = token_clone.cancelled() => {
                    // Timer was cancelled (new copy happened)
                }
            }
        });
    }

    Ok(())
}

#[tauri::command]
pub async fn clipboard_clear(
    app: tauri::AppHandle,
    state: State<'_, ClipboardState>,
) -> Result<(), AppError> {
    // Cancel any existing timer
    if let Some(token) = state.cancel_token.lock().unwrap().take() {
        token.cancel();
    }

    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard()
        .write_text("")
        .map_err(|e| AppError::Clipboard(e.to_string()))?;

    let _ = app.emit("clipboard-cleared", ());
    Ok(())
}
