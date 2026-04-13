import { writable } from 'svelte/store';

export interface VaultState {
	isUnlocked: boolean;
	isInitialized: boolean;
}

function createVaultStore() {
	const { subscribe, set, update } = writable<VaultState>({
		isUnlocked: false,
		isInitialized: false
	});

	return {
		subscribe,
		initialize: () => update((s) => ({ ...s, isInitialized: true })),
		unlock: () => update((s) => ({ ...s, isUnlocked: true, isInitialized: true })),
		lock: () => update((s) => ({ ...s, isUnlocked: false })),
		reset: () => set({ isUnlocked: false, isInitialized: false })
	};
}

export const vault = createVaultStore();
