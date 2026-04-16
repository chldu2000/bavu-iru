# Phase 3 Implementation Plan: Clipboard + Auto-Lock + System Tray

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add clipboard integration with auto-clear, idle/system/focus auto-lock, and system tray support to the password manager.

**Architecture:** Three subsystems implemented sequentially (clipboard → auto-lock → tray), followed by a settings page. Each subsystem is self-contained and testable independently. All clipboard operations go through Tauri IPC to Rust backend. Auto-lock uses frontend idle detection + backend system lock screen listener. System tray intercepts window close and provides quick-lock access.

**Tech Stack:** Tauri v2, Svelte 5, Rust, `tauri-plugin-clipboard-manager`, `tauri` tray-icon feature

---

## Task 1: Clipboard Rust Backend — `clipboard_copy` and `clipboard_clear` commands

**Files:**
- Modify: `src-tauri/src/commands/clipboard.rs` (currently placeholder)
- Modify: `src-tauri/src/error.rs` (add `Clipboard` variant)
- Modify: `src-tauri/src/lib.rs` (register plugin + commands)

- [ ] **Step 1: Add `Clipboard` error variant to `error.rs`**

Add after the `Base64` variant in `src-tauri/src/error.rs`:

```rust
#[error("Clipboard error: {0}")]
Clipboard(String),
```

- [ ] **Step 2: Implement `clipboard.rs`**

Replace the placeholder content in `src-tauri/src/commands/clipboard.rs` with:

```rust
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
```

- [ ] **Step 3: Add `tokio-util` dependency to `Cargo.toml`**

Add at the end of the `[dependencies]` section:

```toml
tokio-util = { version = "0.7", features = ["rt"] }
```

- [ ] **Step 4: Register clipboard plugin, state, and commands in `lib.rs`**

In `src-tauri/src/lib.rs`, make these changes:

Add import at top:
```rust
use commands::clipboard::ClipboardState;
```

In the `setup` closure, after `app.manage(keyring);`, add:
```rust
app.manage(ClipboardState::new());
```

Add `.plugin(tauri_plugin_clipboard_manager::init())` right after the log plugin line:
```rust
.plugin(tauri_plugin_clipboard_manager::init())
```

Add to `invoke_handler` array:
```rust
commands::clipboard::clipboard_copy,
commands::clipboard::clipboard_clear,
```

- [ ] **Step 5: Build and verify compilation**

Run: `cd /Users/chldu/Workspace/bavu-iru && cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -20`
Expected: Build succeeds with no errors.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands/clipboard.rs src-tauri/src/error.rs src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat: implement clipboard copy/clear Rust commands with auto-clear timer"
```

---

## Task 2: Clipboard Frontend — IPC wrappers and component integration

**Files:**
- Modify: `src/lib/utils/tauri.ts` (add clipboard IPC functions)
- Modify: `src/lib/components/EntryDetail.svelte` (replace navigator.clipboard)
- Modify: `src/lib/components/PasswordField.svelte` (replace navigator.clipboard)

- [ ] **Step 1: Add clipboard IPC wrappers to `tauri.ts`**

Append to end of `src/lib/utils/tauri.ts`:

```typescript
// --- Clipboard operations ---

export async function clipboardCopy(text: string, sensitive: boolean, clearSeconds?: number): Promise<void> {
  return invoke('clipboard_copy', { text, sensitive, clearSeconds });
}

export async function clipboardClear(): Promise<void> {
  return invoke('clipboard_clear');
}
```

- [ ] **Step 2: Update `EntryDetail.svelte` — replace copy logic**

In `src/lib/components/EntryDetail.svelte`:

Add import at top of `<script>`:
```typescript
import { clipboardCopy } from '$lib/utils/tauri';
import { settings } from '$lib/stores/settings';
```

Replace the `copyText` function (lines 37-45) with:
```typescript
async function copyText(text: string, field: string = '', sensitive: boolean = false) {
    await clipboardCopy(text, sensitive, $settings.clipboardClearSeconds);
    copiedField = field;
    showCopiedToast = true;
    setTimeout(() => {
        showCopiedToast = false;
        copiedField = '';
    }, 2000);
}
```

Update the username copy button's `onclick` to pass `sensitive: false`:
```svelte
onclick={() => copyText(entry.username!, 'username', false)}
```

Update the password field's `oncopy` to pass `sensitive: true`:
```svelte
<PasswordField value={entry.password ?? ''} oncopy={() => copyText(entry.password!, 'password', true)} />
```

- [ ] **Step 3: Update `PasswordField.svelte` — remove direct clipboard access**

In `src/lib/components/PasswordField.svelte`, the `handleCopy` function currently uses `navigator.clipboard.writeText(value)`. We need to change the component to delegate copy to the parent:

Change the `Props` interface to include a `sensitive` prop:
```typescript
interface Props {
  value: string;
  editable?: boolean;
  onchange?: (value: string) => void;
  oncopy?: () => void;
  sensitive?: boolean;
}
```

Update destructuring:
```typescript
let { value, editable = false, onchange, oncopy, sensitive = false }: Props = $props();
```

Add imports at top of `<script>`:
```typescript
import { clipboardCopy } from '$lib/utils/tauri';
import { settings } from '$lib/stores/settings';
```

Replace `handleCopy` function:
```typescript
async function handleCopy() {
  await clipboardCopy(value, sensitive, $settings.clipboardClearSeconds);
  copied = true;
  oncopy?.();
  setTimeout(() => (copied = false), 2000);
}
```

- [ ] **Step 4: Build and verify**

Run: `cd /Users/chldu/Workspace/bavu-iru && pnpm build 2>&1 | tail -20`
Expected: Build succeeds.

- [ ] **Step 5: Commit**

```bash
git add src/lib/utils/tauri.ts src/lib/components/EntryDetail.svelte src/lib/components/PasswordField.svelte
git commit -m "feat: integrate clipboard IPC in frontend, replace navigator.clipboard"
```

---

## Task 3: Clipboard Two-Stage Toast — listen for `clipboard-cleared` event

**Files:**
- Modify: `src/lib/components/EntryDetail.svelte` (add cleared toast)
- Modify: `src/routes/+page.svelte` (add global clipboard-cleared listener)

- [ ] **Step 1: Add global clipboard-cleared listener in `+page.svelte`**

In `src/routes/+page.svelte`, add import at top of `<script>`:

```typescript
import { listen } from '@tauri-apps/api/event';
import Toast from '$lib/components/Toast.svelte';
```

Add state variables after existing state declarations:
```typescript
let clipboardClearedToast = $state(false);
```

Add listener setup inside the `onMount` callback, after `vault.checkStatus()`:
```typescript
listen('clipboard-cleared', () => {
    clipboardClearedToast = true;
    setTimeout(() => (clipboardClearedToast = false), 2000);
}).catch(() => {});
```

Add the Toast component at the bottom of the template, after the closing `{/if}` of the main content, but inside the root fragment:
```svelte
{#if !$vault.isUnlocked}
    <LockScreen />
{:else}
    <div class="flex h-screen">
        <!-- ... existing content ... -->
    </div>
{/if}
<Toast message="剪贴板已清除" visible={clipboardClearedToast} />
```

- [ ] **Step 2: Build and verify**

Run: `pnpm build 2>&1 | tail -10`
Expected: Build succeeds.

- [ ] **Step 3: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat: add clipboard-cleared toast notification via Tauri event"
```

---

## Task 4: Auto-Lock — Frontend Idle Timer Module

**Files:**
- Create: `src/lib/lib/idleTimer.ts`

- [ ] **Step 1: Create `idleTimer.ts`**

Create file `src/lib/lib/idleTimer.ts` with:

```typescript
import { get } from 'svelte/store';
import { vault } from '$lib/stores/vault';
import { settings } from '$lib/stores/settings';

type TimerHandle = { stop: () => void };

/**
 * Start an idle timer that locks the vault after `minutes` of no user interaction.
 * Returns a handle with a `stop()` method to cancel the timer.
 */
export function startIdleTimer(): TimerHandle {
    let timeoutId: ReturnType<typeof setTimeout> | null = null;
    let active = true;

    function reset() {
        if (!active) return;
        if (timeoutId !== null) clearTimeout(timeoutId);

        const minutes = get(settings).autoLockMinutes;
        if (minutes <= 0) return; // disabled

        timeoutId = setTimeout(async () => {
            if (!get(vault).isUnlocked) return;
            try {
                await vault.lock();
            } catch (e) {
                console.error('Auto-lock failed:', e);
            }
        }, minutes * 60 * 1000);
    }

    function handler() {
        reset();
    }

    const events = ['mousemove', 'keydown', 'click', 'scroll', 'touchstart'] as const;
    for (const event of events) {
        window.addEventListener(event, handler, { passive: true });
    }
    reset();

    return {
        stop() {
            active = false;
            if (timeoutId !== null) clearTimeout(timeoutId);
            for (const event of events) {
                window.removeEventListener(event, handler);
            }
        },
    };
}

/**
 * Start a focus-loss timer that locks the vault after `minutes` of the window
 * being blurred / hidden. Only active when enabled in settings.
 */
export function startFocusLossTimer(): TimerHandle {
    let timeoutId: ReturnType<typeof setTimeout> | null = null;
    let active = true;

    function clearTimer() {
        if (timeoutId !== null) {
            clearTimeout(timeoutId);
            timeoutId = null;
        }
    }

    function onBlur() {
        if (!active || !get(vault).isUnlocked) return;
        clearTimer();

        // Focus loss timer is controlled by a setting; 0 means disabled
        // We'll use a separate setting field: focusLockMinutes
        const minutes = get(settings).focusLockMinutes ?? 0;
        if (minutes <= 0) return;

        timeoutId = setTimeout(async () => {
            if (!get(vault).isUnlocked) return;
            try {
                await vault.lock();
            } catch (e) {
                console.error('Focus-loss lock failed:', e);
            }
        }, minutes * 60 * 1000);
    }

    function onFocus() {
        if (!active) return;
        clearTimer();
    }

    window.addEventListener('blur', onBlur);
    window.addEventListener('focus', onFocus);
    document.addEventListener('visibilitychange', () => {
        if (document.hidden) {
            onBlur();
        } else {
            onFocus();
        }
    });

    return {
        stop() {
            active = false;
            clearTimer();
            window.removeEventListener('blur', onBlur);
            window.removeEventListener('focus', onFocus);
        },
    };
}
```

- [ ] **Step 2: Add `focusLockMinutes` to settings store**

In `src/lib/stores/settings.ts`, add the field to the interface and defaults:

```typescript
export interface Settings {
    autoLockMinutes: number;
    clipboardClearSeconds: number;
    focusLockMinutes: number; // 0 = disabled
    theme: 'light' | 'dark' | 'system';
}

const DEFAULT_SETTINGS: Settings = {
    autoLockMinutes: 5,
    clipboardClearSeconds: 30,
    focusLockMinutes: 0,
    theme: 'dark'
};
```

- [ ] **Step 3: Commit**

```bash
git add src/lib/lib/idleTimer.ts src/lib/stores/settings.ts
git commit -m "feat: add idle timer and focus-loss timer modules for auto-lock"
```

---

## Task 5: Auto-Lock — Frontend Timer Integration in `+page.svelte`

**Files:**
- Modify: `src/routes/+page.svelte`

- [ ] **Step 1: Add timer lifecycle to `+page.svelte`**

Add imports at top of `<script>` in `src/routes/+page.svelte`:

```typescript
import { startIdleTimer, startFocusLossTimer } from '$lib/lib/idleTimer';
import type { TimerHandle } from '$lib/lib/idleTimer';
```

Add state variables after existing declarations:

```typescript
let idleTimer: TimerHandle | null = $state(null);
let focusTimer: TimerHandle | null = $state(null);
```

Replace the existing `$effect` that loads data (line 36-42) with an expanded version:

```typescript
$effect(() => {
    if ($vault.isUnlocked) {
        entries.load();
        folders.load();
        tags.load();

        // Start auto-lock timers
        if (!idleTimer) {
            idleTimer = startIdleTimer();
        }
        if (!focusTimer) {
            focusTimer = startFocusLossTimer();
        }
    } else {
        // Stop timers when locked
        if (idleTimer) {
            idleTimer.stop();
            idleTimer = null;
        }
        if (focusTimer) {
            focusTimer.stop();
            focusTimer = null;
        }
    }
});
```

- [ ] **Step 2: Build and verify**

Run: `pnpm build 2>&1 | tail -10`
Expected: Build succeeds.

- [ ] **Step 3: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat: integrate idle and focus-loss auto-lock timers in main page"
```

---

## Task 6: Auto-Lock — Rust Backend System Lock Screen Listener (macOS)

**Files:**
- Modify: `src-tauri/src/security/autolock.rs` (currently placeholder)
- Modify: `src-tauri/src/security/mod.rs` (no change needed, already exports autolock)
- Modify: `src-tauri/src/lib.rs` (initialize listener in setup)

- [ ] **Step 1: Implement `autolock.rs`**

Replace placeholder content in `src-tauri/src/security/autolock.rs` with:

```rust
use std::sync::{Arc, Mutex};

/// Trait for listening to system lock-screen events.
/// Implementations are platform-specific.
pub trait LockScreenListener: Send + Sync {
    fn start_listening(&self, on_lock: Box<dyn Fn() + Send + Sync>);
    fn stop_listening(&self);
}

/// A no-op listener for platforms without an implementation yet.
pub struct NoOpLockScreenListener;

impl LockScreenListener for NoOpLockScreenListener {
    fn start_listening(&self, _on_lock: Box<dyn Fn() + Send + Sync>) {}
    fn stop_listening(&self) {}
}

#[cfg(target_os = "macos")]
pub mod platform {
    use super::LockScreenListener;
    use std::sync::{Arc, Mutex};

    pub struct MacOSLockScreenListener {
        running: Arc<Mutex<bool>>,
    }

    impl MacOSLockScreenListener {
        pub fn new() -> Self {
            Self {
                running: Arc::new(Mutex::new(false)),
            }
        }
    }

    impl LockScreenListener for MacOSLockScreenListener {
        fn start_listening(&self, on_lock: Box<dyn Fn() + Send + Sync>) {
            use std::ffi::c_void;

            let running = self.running.clone();
            *running.lock().unwrap() = true;

            // Wrap callback in double-box for raw pointer storage
            let callback: Box<Box<dyn Fn() + Send + Sync>> = Box::new(on_lock);
            let callback_ptr = Box::into_raw(callback) as *mut c_void;

            // Poll CGSessionCopyCurrentDictionary for screen lock state
            std::thread::spawn(move || {
                let mut was_locked = false;
                while *running.lock().unwrap() {
                    let is_locked = is_screen_locked();
                    if is_locked && !was_locked {
                        let cb = callback_ptr as *mut Box<dyn Fn() + Send + Sync>;
                        unsafe {
                            (*cb)();
                        }
                    }
                    was_locked = is_locked;
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }

                // Clean up callback
                unsafe {
                    let _ = Box::from_raw(callback_ptr as *mut Box<dyn Fn() + Send + Sync>);
                }
            });
        }

        fn stop_listening(&self) {
            *self.running.lock().unwrap() = false;
        }
    }

    fn is_screen_locked() -> bool {
        unsafe {
            let dict = CGSessionCopyCurrentDictionary();
            if dict.is_null() {
                return false;
            }

            let key = CFStringCreateWithCString(
                std::ptr::null(),
                "CGSSessionScreenIsLocked\0".as_ptr() as *const i8,
                0x08000100, // kCFStringEncodingUTF8
            );

            let value = CFDictionaryGetValue(dict, key as *const std::ffi::c_void);
            CFRelease(dict as *mut std::ffi::c_void);
            CFRelease(key as *mut std::ffi::c_void);

            !value.is_null()
        }
    }

    extern "C" {
        fn CGSessionCopyCurrentDictionary() -> *mut std::ffi::c_void;
        fn CFStringCreateWithCString(
            alloc: *mut std::ffi::c_void,
            c_str: *const i8,
            encoding: u32,
        ) -> *mut std::ffi::c_void;
        fn CFDictionaryGetValue(
            dict: *mut std::ffi::c_void,
            key: *const std::ffi::c_void,
        ) -> *const std::ffi::c_void;
        fn CFRelease(cf: *mut std::ffi::c_void);
    }
}

/// Create the platform-appropriate lock screen listener.
pub fn create_lock_screen_listener() -> Box<dyn LockScreenListener> {
    #[cfg(target_os = "macos")]
    {
        Box::new(platform::MacOSLockScreenListener::new())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Box::new(NoOpLockScreenListener)
    }
}
```

> **Note:** The macOS implementation uses raw FFI to CoreGraphics/CoreFoundation for screen lock detection via polling `CGSessionCopyCurrentDictionary`. No additional crate dependencies needed.

- [ ] **Step 2: Initialize lock screen listener in `lib.rs` setup**

In `src-tauri/src/lib.rs`, add import at top:

```rust
use security::autolock::create_lock_screen_listener;
use tauri::Emitter;
```

Inside the `setup` closure, after `app.manage(keyring);`, add:

```rust
// Start system lock screen listener
let lock_listener = create_lock_screen_listener();
let app_handle = app.handle().clone();
lock_listener.start_listening(Box::new(move || {
    let keyring = app_handle.state::<Keyring>();
    if keyring.is_unlocked() {
        keyring.lock();
        let _ = app_handle.emit("vault-locked", ());
    }
}));
```

- [ ] **Step 3: Build and verify**

Run: `cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -20`
Expected: Build succeeds. On non-macOS platforms, NoOpLockScreenListener is used.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/security/autolock.rs src-tauri/src/lib.rs
git commit -m "feat: implement lock screen listener with macOS support via polling"
```

---

## Task 7: Auto-Lock — Frontend `vault-locked` event listener

**Files:**
- Modify: `src/lib/stores/vault.ts` (listen for backend-emitted vault-locked event)

- [ ] **Step 1: Add `vault-locked` event listener to vault store**

In `src/lib/stores/vault.ts`, add import:

```typescript
import { listen } from '@tauri-apps/api/event';
```

Add inside `createVaultStore()`, after `reset()` method and before the closing `}`:

```typescript
// Listen for vault-locked events from backend (e.g. system lock screen)
if (typeof window !== 'undefined') {
    listen('vault-locked', () => {
        update((s) => ({ ...s, isUnlocked: false }));
    }).catch(() => {});
}
```

- [ ] **Step 2: Build and verify**

Run: `pnpm build 2>&1 | tail -10`
Expected: Build succeeds.

- [ ] **Step 3: Commit**

```bash
git add src/lib/stores/vault.ts
git commit -m "feat: listen for backend vault-locked events in vault store"
```

---

## Task 8: System Tray — Tray Icon Resources

**Files:**
- Create: `src-tauri/icons/tray-unlocked.png`
- Create: `src-tauri/icons/tray-locked.png`

- [ ] **Step 1: Generate tray icon PNGs**

We need two small PNG icons (16x16 or 32x32) for tray: one for locked, one for unlocked. Create simple monochrome icons using the existing app icon or use ImageMagick:

```bash
# Use existing icon as base, create simple status icons
# If ImageMagick is available:
convert src-tauri/icons/32x32.png -colorspace Gray -threshold 50% src-tauri/icons/tray-unlocked.png
convert src-tauri/icons/32x32.png -colorspace Gray -threshold 50% -negate src-tauri/icons/tray-locked.png
```

If ImageMagick is not available, copy the existing 32x32 icon as both and we'll differentiate by tooltip text instead:

```bash
cp src-tauri/icons/32x32.png src-tauri/icons/tray-unlocked.png
cp src-tauri/icons/32x32.png src-tauri/icons/tray-locked.png
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/icons/tray-unlocked.png src-tauri/icons/tray-locked.png
git commit -m "feat: add system tray icon resources"
```

---

## Task 9: System Tray — Rust Tray Module

**Files:**
- Create: `src-tauri/src/tray.rs`
- Modify: `src-tauri/src/lib.rs` (register tray, intercept window close)

- [ ] **Step 1: Create `tray.rs`**

Create `src-tauri/src/tray.rs` with:

```rust
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Manager, Runtime,
};

pub fn create_tray<R: Runtime>(app: &App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
    let lock = MenuItem::with_id(app, "lock", "立即锁定", true, None::<&str>)?;
    let status = MenuItem::with_id(app, "status", "保险库状态: 已锁定", false, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&status, &show, &lock, &sep, &quit])?;

    let icon_bytes = include_bytes!("../icons/32x32.png");
    let icon = tauri::image::Image::from_bytes(icon_bytes)?;

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
                    // Update tray state
                    update_tray_state(app, false);
                    // Hide window
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
    if let Some(tray) = app.tray_by_id("main-tray") {
        let menu = tray.menu().unwrap();

        // Update status text
        if let Some(item) = menu.get("status") {
            if let Some(menu_item) = item.as_menuitem() {
                let _ = menu_item.set_text(if unlocked {
                    "保险库状态: 已解锁"
                } else {
                    "保险库状态: 已锁定"
                });
            }
        }

        // Enable/disable lock button
        if let Some(item) = menu.get("lock") {
            if let Some(menu_item) = item.as_menuitem() {
                let _ = menu_item.set_enabled(unlocked);
            }
        }
    }
}
```

- [ ] **Step 2: Register tray and window close interception in `lib.rs`**

Add import at top of `src-tauri/src/lib.rs`:

```rust
mod tray;
```

In the `setup` closure, at the end (before `Ok(())`), add:

```rust
// Create system tray
tray::create_tray(app)?;

// Update tray state when vault unlocks
let app_handle = app.handle().clone();
app.listen("vault-unlocked", move |_| {
    tray::update_tray_state(&app_handle, true);
});
```

Also in `setup`, after `app.manage(keyring);`, add window close interception. We need to change `setup` to use `app` properly. Add a `on_window_event` handler:

Actually, we need to add the window close handler on the Builder, not in setup. Add before `.run()`:

```rust
.on_window_event(|window, event| {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        // Hide window instead of closing
        let _ = window.hide();
        api.prevent_close();
    }
})
```

- [ ] **Step 3: Update `vault_unlock` and `vault_setup` to emit `vault-unlocked` event**

In `src-tauri/src/commands/vault.rs`, add `app: tauri::AppHandle` parameter and emit event:

For `vault_setup`:
```rust
#[tauri::command]
pub fn vault_setup(
    password: String,
    app: tauri::AppHandle,
    db: State<'_, Database>,
    keyring: State<'_, Keyring>,
) -> Result<(), AppError> {
    // ... existing logic ...
    keyring.set(key);
    let _ = app.emit("vault-unlocked", ());
    Ok(())
}
```

For `vault_unlock`:
```rust
#[tauri::command]
pub fn vault_unlock(
    password: String,
    app: tauri::AppHandle,
    db: State<'_, Database>,
    keyring: State<'_, Keyring>,
) -> Result<(), AppError> {
    // ... existing logic ...
    keyring.set(key);
    let _ = app.emit("vault-unlocked", ());
    Ok(())
}
```

For `vault_lock`, also emit and update tray:
```rust
#[tauri::command]
pub fn vault_lock(app: tauri::AppHandle, keyring: State<'_, Keyring>) -> Result<(), AppError> {
    keyring.lock();
    let _ = app.emit("vault-locked", ());
    Ok(())
}
```

Add import in `vault.rs`:
```rust
use tauri::Emitter;
```

- [ ] **Step 4: Build and verify**

Run: `cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -20`
Expected: Build succeeds.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/tray.rs src-tauri/src/lib.rs src-tauri/src/commands/vault.rs
git commit -m "feat: add system tray with lock/show/quit menu and window close interception"
```

---

## Task 10: Settings Page — UI Component

**Files:**
- Create: `src/lib/components/Settings.svelte`
- Modify: `src/routes/+page.svelte` (add settings entry point)

- [ ] **Step 1: Create `Settings.svelte`**

Create `src/lib/components/Settings.svelte` with:

```svelte
<script lang="ts">
    import { settings } from '$lib/stores/settings';

    let localSettings = $state({ ...$settings });
    let saved = $state(false);

    function save() {
        $settings = { ...localSettings };
        saved = true;
        setTimeout(() => (saved = false), 2000);
    }

    interface Props {
        onclose: () => void;
    }

    let { onclose }: Props = $props();
</script>

<div class="flex h-full flex-col bg-dark-bg">
    <!-- Header -->
    <div class="flex items-center justify-between border-b border-dark-border px-5 py-3">
        <h2 class="text-lg font-bold text-dark-text">设置</h2>
        <button
            class="cursor-pointer text-dark-muted hover:text-dark-text"
            onclick={onclose}
        >
            ✕
        </button>
    </div>

    <!-- Settings form -->
    <div class="flex-1 overflow-y-auto p-5">
        <div class="flex flex-col gap-6">
            <!-- Auto-lock -->
            <section>
                <h3 class="mb-3 text-sm font-medium text-dark-text">自动锁定</h3>
                <div class="flex flex-col gap-3">
                    <label class="flex items-center justify-between">
                        <span class="text-sm text-dark-secondary">空闲自动锁定（分钟，0 = 关闭）</span>
                        <input
                            type="number"
                            class="w-20 rounded-md border border-dark-border bg-dark-card px-3 py-1.5 text-sm text-dark-text outline-none focus:border-accent"
                            bind:value={localSettings.autoLockMinutes}
                            min="0"
                            max="30"
                        />
                    </label>
                    <label class="flex items-center justify-between">
                        <span class="text-sm text-dark-secondary">窗口失焦锁定（分钟，0 = 关闭）</span>
                        <input
                            type="number"
                            class="w-20 rounded-md border border-dark-border bg-dark-card px-3 py-1.5 text-sm text-dark-text outline-none focus:border-accent"
                            bind:value={localSettings.focusLockMinutes}
                            min="0"
                            max="10"
                        />
                    </label>
                </div>
            </section>

            <!-- Clipboard -->
            <section>
                <h3 class="mb-3 text-sm font-medium text-dark-text">剪贴板</h3>
                <label class="flex items-center justify-between">
                    <span class="text-sm text-dark-secondary">自动清除时间（秒）</span>
                    <input
                        type="number"
                        class="w-20 rounded-md border border-dark-border bg-dark-card px-3 py-1.5 text-sm text-dark-text outline-none focus:border-accent"
                        bind:value={localSettings.clipboardClearSeconds}
                        min="10"
                        max="120"
                    />
                </label>
            </section>

            <!-- Theme -->
            <section>
                <h3 class="mb-3 text-sm font-medium text-dark-text">外观</h3>
                <label class="flex items-center justify-between">
                    <span class="text-sm text-dark-secondary">主题</span>
                    <select
                        class="rounded-md border border-dark-border bg-dark-card px-3 py-1.5 text-sm text-dark-text outline-none focus:border-accent"
                        bind:value={localSettings.theme}
                    >
                        <option value="dark">暗色</option>
                        <option value="light">亮色</option>
                        <option value="system">跟随系统</option>
                    </select>
                </label>
            </section>
        </div>
    </div>

    <!-- Footer -->
    <div class="flex items-center justify-end border-t border-dark-border px-5 py-3">
        <button
            class="cursor-pointer rounded-md bg-accent px-4 py-2 text-sm font-medium text-white hover:bg-accent-hover"
            onclick={save}
        >
            {saved ? '已保存' : '保存'}
        </button>
    </div>
</div>
```

- [ ] **Step 2: Add settings entry to `+page.svelte`**

In `src/routes/+page.svelte`:

Add import:
```typescript
import Settings from '$lib/components/Settings.svelte';
```

Add state:
```typescript
let showSettings = $state(false);
```

Add a settings button in the bottom bar area. Find the existing bottom bar `div` (around line 147-161) and add a settings button next to the lock button:

```svelte
<div class="flex items-center justify-between border-t border-dark-border p-2">
    <div class="flex items-center gap-1">
        <button
            class="cursor-pointer rounded-md px-2 py-1.5 text-xs text-dark-muted hover:text-accent"
            onclick={handleLock}
            title="锁定保险库"
        >
            🔒 锁定
        </button>
        <button
            class="cursor-pointer rounded-md px-2 py-1.5 text-xs text-dark-muted hover:text-accent"
            onclick={() => (showSettings = true)}
            title="设置"
        >
            ⚙ 设置
        </button>
    </div>
    <button
        class="cursor-pointer rounded-md bg-accent px-4 py-1.5 text-xs font-medium text-white hover:bg-accent-hover"
        onclick={startCreate}
    >
        + 新建条目
    </button>
</div>
```

Add settings view in the right panel. Update the right panel section to include settings:

```svelte
<!-- Right panel -->
<div class="flex-1 bg-dark-bg">
    {#if showSettings}
        <Settings onclose={() => (showSettings = false)} />
    {:else if viewMode === 'detail' && selectedEntry}
        <EntryDetail
            entry={selectedEntry}
            onedit={startEdit}
            ondelete={deleteEntry}
        />
    {:else if viewMode === 'edit' && selectedEntry}
        <EntryForm
            entry={selectedEntry}
            onsave={saveEntry}
            oncancel={cancelEdit}
        />
    {:else if viewMode === 'create'}
        <EntryForm
            onsave={saveEntry}
            oncancel={cancelEdit}
        />
    {:else}
        <div class="flex h-full items-center justify-center text-dark-muted">
            <div class="text-center">
                <div class="mb-2 text-3xl">📋</div>
                <p class="text-sm">选择一个条目查看详情</p>
                <p class="text-sm">或创建新条目</p>
            </div>
        </div>
    {/if}
</div>
```

- [ ] **Step 3: Build and verify**

Run: `pnpm build 2>&1 | tail -10`
Expected: Build succeeds.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/Settings.svelte src/routes/+page.svelte
git commit -m "feat: add settings page with auto-lock and clipboard configuration"
```

---

## Task 11: End-to-End Verification

**Files:** None (testing only)

- [ ] **Step 1: Run full Rust build**

Run: `cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10`
Expected: Build succeeds.

- [ ] **Step 2: Run frontend build**

Run: `pnpm build 2>&1 | tail -10`
Expected: Build succeeds.

- [ ] **Step 3: Run `pnpm tauri dev` and manually verify**

1. Launch the app with `pnpm tauri dev`
2. Set up master password or unlock vault
3. **Clipboard**: Click copy on a password field → verify "已复制" toast appears → wait 30s → verify "剪贴板已清除" toast
4. **Auto-lock**: Wait 5 minutes (or change setting to 1 min for testing) without interaction → verify vault locks
5. **System tray**: Close window → verify app stays in tray → click tray icon → verify window reappears → right-click tray → verify "立即锁定" works
6. **Settings**: Click settings button → change values → save → verify settings persist

- [ ] **Step 4: Final commit — mark Phase 3 tasks in PLAN.md**

Update `PLAN.md` Phase 3 section, change all `- [ ]` to `- [x]`:

```bash
git add PLAN.md
git commit -m "docs: mark Phase 3 as complete in PLAN.md"
```
