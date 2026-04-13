import { writable } from 'svelte/store';
import { entryList, entryCreate, entryUpdate, entryDelete } from '$lib/utils/tauri';

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

function createEntriesStore() {
	const { subscribe, set, update } = writable<Entry[]>([]);

	return {
		subscribe,
		async load() {
			const list = await entryList();
			set(list);
		},
		async create(entry: Entry) {
			await entryCreate(entry);
			update((items) => [entry, ...items]);
		},
		async save(entry: Entry) {
			await entryUpdate(entry);
			update((items) => items.map((e) => (e.id === entry.id ? entry : e)));
		},
		async remove(id: string) {
			await entryDelete(id);
			update((items) => items.filter((e) => e.id !== id));
		}
	};
}

export const entries = createEntriesStore();
