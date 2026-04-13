import { writable } from 'svelte/store';

export interface Settings {
	autoLockMinutes: number;
	clipboardClearSeconds: number;
	theme: 'light' | 'dark' | 'system';
}

const DEFAULT_SETTINGS: Settings = {
	autoLockMinutes: 5,
	clipboardClearSeconds: 30,
	theme: 'dark'
};

export const settings = writable<Settings>(DEFAULT_SETTINGS);
