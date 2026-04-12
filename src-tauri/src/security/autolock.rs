use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tauri::Emitter;

const DEFAULT_TIMEOUT_SECS: u64 = 300; // 5 minutes

static LAST_ACTIVITY: AtomicU64 = AtomicU64::new(0);

pub fn update_activity() {
    LAST_ACTIVITY.store(Instant::now().elapsed().as_secs(), Ordering::SeqCst);
}

pub fn check_auto_lock(timeout_secs: u64) -> bool {
    let last = LAST_ACTIVITY.load(Ordering::SeqCst);
    let elapsed = Instant::now().elapsed().as_secs();
    elapsed - last > timeout_secs
}

pub fn reset_activity() {
    update_activity();
}

pub fn init_auto_lock(app: &tauri::App) {
    let app_handle = app.handle().clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(10));
            if check_auto_lock(DEFAULT_TIMEOUT_SECS) {
                // Trigger lock via Tauri event
                let _ = app_handle.emit("auto-lock", ());
            }
        }
    });
}