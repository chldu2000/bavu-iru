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

import type { Entry } from '$lib/stores/entries';

export async function entryList(): Promise<Entry[]> {
	return invoke('entry_list');
}

export async function entryGet(id: string): Promise<Entry | null> {
	return invoke('entry_get', { id });
}

export async function entryCreate(entry: Entry): Promise<void> {
	return invoke('entry_create', { entry });
}

export async function entryUpdate(entry: Entry): Promise<boolean> {
	return invoke('entry_update', { entry });
}

export async function entryDelete(id: string): Promise<boolean> {
	return invoke('entry_delete', { id });
}

export interface GeneratorOptions {
  length: number;
  uppercase: boolean;
  lowercase: boolean;
  digits: boolean;
  special: boolean;
  exclude_chars: string;
}

export interface GeneratedPassword {
  password: string;
}

export interface StrengthResult {
  score: number;
  label: string;
  feedback: string;
}

export async function generatePassword(options: GeneratorOptions): Promise<GeneratedPassword> {
  return invoke('generate_password', { options });
}

export async function evaluatePasswordStrength(password: string): Promise<StrengthResult> {
  return invoke('evaluate_password_strength', { password });
}
