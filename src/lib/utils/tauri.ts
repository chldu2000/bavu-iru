import { invoke } from '@tauri-apps/api/core';

export async function vaultSetup(password: string): Promise<void> {
	return invoke('vault_setup', { password });
}

export async function vaultUnlock(password: string): Promise<void> {
	return invoke('vault_unlock', { password });
}

export async function vaultLock(): Promise<void> {
	return invoke('vault_lock');
}

export async function vaultStatus(): Promise<boolean> {
	return invoke('vault_status');
}
