import { writable } from 'svelte/store';
import { tagList, tagCreate, tagUpdate as tagUpdateApi, tagDelete as tagDeleteApi } from '$lib/utils/tauri';
import type { Tag } from '$lib/utils/tauri';

function createTagsStore() {
  const { subscribe, set, update } = writable<Tag[]>([]);

  return {
    subscribe,
    async load() {
      const list = await tagList();
      set(list);
    },
    async create(name: string, color: string | null = null) {
      const tag = await tagCreate(name, color);
      update((items) => [...items, tag]);
      return tag;
    },
    async update(id: string, name: string, color: string) {
      await tagUpdateApi(id, name, color);
      update((items) => items.map((t) => (t.id === id ? { ...t, name, color } : t)));
    },
    async remove(id: string) {
      await tagDeleteApi(id);
      update((items) => items.filter((t) => t.id !== id));
    }
  };
}

export const tags = createTagsStore();
