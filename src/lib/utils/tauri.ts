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

// --- Folder operations ---

export interface Folder {
  id: string;
  name: string;
  parent_id: string | null;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export async function folderCreate(name: string, parentId: string | null): Promise<Folder> {
  return invoke('folder_create', { name, parentId });
}

export async function folderRename(id: string, name: string): Promise<boolean> {
  return invoke('folder_rename', { id, name });
}

export async function folderDelete(id: string): Promise<boolean> {
  return invoke('folder_delete', { id });
}

export async function folderList(): Promise<Folder[]> {
  return invoke('folder_list');
}

// --- Tag operations ---

export interface Tag {
  id: string;
  name: string;
  color: string;
  created_at: string;
  updated_at: string;
}

export async function tagCreate(name: string, color: string | null): Promise<Tag> {
  return invoke('tag_create', { name, color });
}

export async function tagUpdate(id: string, name: string, color: string): Promise<boolean> {
  return invoke('tag_update', { id, name, color });
}

export async function tagDelete(id: string): Promise<boolean> {
  return invoke('tag_delete', { id });
}

export async function tagList(): Promise<Tag[]> {
  return invoke('tag_list');
}

export async function tagAddToEntry(entryId: string, tagId: string): Promise<void> {
  return invoke('tag_add_to_entry', { entryId, tagId });
}

export async function tagRemoveFromEntry(entryId: string, tagId: string): Promise<void> {
  return invoke('tag_remove_from_entry', { entryId, tagId });
}

// --- Favorite operations ---

export async function toggleFavorite(id: string): Promise<boolean> {
  return invoke('toggle_favorite', { id });
}

// --- Clipboard operations ---

export async function clipboardCopy(text: string, sensitive: boolean, clearSeconds?: number): Promise<void> {
  return invoke('clipboard_copy', { text, sensitive, clearSeconds });
}

export async function clipboardClear(): Promise<void> {
  return invoke('clipboard_clear');
}

// --- Settings operations ---

export async function settingsGet(): Promise<string> {
  return invoke('settings_get');
}

export async function settingsSet(settings: string): Promise<void> {
  return invoke('settings_set', { settings });
}
