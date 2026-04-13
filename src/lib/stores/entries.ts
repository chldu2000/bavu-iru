import { writable } from 'svelte/store';

export interface Entry {
	id: string;
	folder_id: string | null;
	title: string;
	username: string | null;
	password: string | null;
	url: string | null;
	notes: string | null;
	custom_fields: string | null;
	tags: string | null;
	strength: number | null;
	expires_at: string | null;
	created_at: string;
	updated_at: string;
}

export const entries = writable<Entry[]>([]);
