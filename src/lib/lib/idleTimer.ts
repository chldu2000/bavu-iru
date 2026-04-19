import { get } from 'svelte/store';
import { vault } from '$lib/stores/vault';
import { settings } from '$lib/stores/settings';

export type TimerHandle = { stop: () => void };

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
