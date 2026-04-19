import { writable } from 'svelte/store';
import { settingsGet, settingsSet } from '$lib/utils/tauri';

export interface Settings {
	autoLockMinutes: number;
	clipboardClearSeconds: number;
	focusLockMinutes: number;
	theme: 'light' | 'dark' | 'system';
}

const DEFAULT_SETTINGS: Settings = {
	autoLockMinutes: 5,
	clipboardClearSeconds: 30,
	focusLockMinutes: 0,
	theme: 'dark'
};

export const settings = writable<Settings>(DEFAULT_SETTINGS);

let loaded = false;

export async function loadSettings() {
	if (loaded) return;
	try {
		const json = await settingsGet();
		if (json && json !== '{}') {
			const saved = JSON.parse(json) as Partial<Settings>;
			settings.set({ ...DEFAULT_SETTINGS, ...saved });
		}
	} catch {
		// Tauri not available (dev in browser)
	}
	loaded = true;
}

export async function saveSettings(s: Settings) {
	settings.set(s);
	try {
		await settingsSet(JSON.stringify(s));
	} catch {
		// Tauri not available
	}
}
