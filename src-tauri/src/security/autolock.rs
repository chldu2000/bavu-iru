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
            let running = self.running.clone();
            *running.lock().unwrap() = true;

            // Wrap callback for cross-thread use
            let callback: Arc<dyn Fn() + Send + Sync> = Arc::new(on_lock);

            // Poll CGSessionCopyCurrentDictionary for screen lock state
            std::thread::spawn(move || {
                let mut was_locked = false;
                while *running.lock().unwrap() {
                    let is_locked = is_screen_locked();
                    if is_locked && !was_locked {
                        callback();
                    }
                    was_locked = is_locked;
                    std::thread::sleep(std::time::Duration::from_secs(2));
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
                std::ptr::null_mut(),
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
