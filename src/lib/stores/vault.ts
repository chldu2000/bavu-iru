import { writable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import { vaultSetup, vaultUnlock, vaultLock, vaultStatus } from '$lib/utils/tauri';

export interface VaultState {
	isUnlocked: boolean;
	isInitialized: boolean;
}

function createVaultStore() {
	const { subscribe, set, update } = writable<VaultState>({
		isUnlocked: false,
		isInitialized: false
	});

	// Listen for vault-locked events from backend (e.g. system lock screen)
	if (typeof window !== 'undefined') {
		listen('vault-locked', () => {
			update((s) => ({ ...s, isUnlocked: false }));
		}).catch(() => {});
	}

	return {
		subscribe,
		async checkStatus() {
			const status = await vaultStatus();
			set({ isUnlocked: status, isInitialized: status });
		},
		async setup(password: string) {
			await vaultSetup(password);
			update(() => ({ isUnlocked: true, isInitialized: true }));
		},
		async unlock(password: string) {
			await vaultUnlock(password);
			update(() => ({ isUnlocked: true, isInitialized: true }));
		},
		async lock() {
			await vaultLock();
			update((s) => ({ ...s, isUnlocked: false }));
		},
		reset() {
			set({ isUnlocked: false, isInitialized: false });
		}
	};
}

export const vault = createVaultStore();
