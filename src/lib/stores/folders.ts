import { writable } from 'svelte/store';
import { folderList, folderCreate, folderRename as folderRenameApi, folderDelete as folderDeleteApi } from '$lib/utils/tauri';
import type { Folder } from '$lib/utils/tauri';

function createFoldersStore() {
  const { subscribe, set, update } = writable<Folder[]>([]);

  return {
    subscribe,
    async load() {
      const list = await folderList();
      set(list);
    },
    async create(name: string, parentId: string | null = null) {
      const folder = await folderCreate(name, parentId);
      update((items) => [...items, folder]);
      return folder;
    },
    async rename(id: string, name: string) {
      await folderRenameApi(id, name);
      update((items) => items.map((f) => (f.id === id ? { ...f, name } : f)));
    },
    async remove(id: string) {
      await folderDeleteApi(id);
      update((items) => items.filter((f) => f.id !== id));
    }
  };
}

export const folders = createFoldersStore();
