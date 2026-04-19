import { writable } from 'svelte/store';

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

export const settings = writable<Settings>(DEFAULT_SETTINGS);
