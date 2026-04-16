<script lang="ts">
	import { onMount } from 'svelte';
	import { listen } from '@tauri-apps/api/event';
	import { vault } from '$lib/stores/vault';
	import { entries } from '$lib/stores/entries';
	import { folders } from '$lib/stores/folders';
	import { tags } from '$lib/stores/tags';
	import LockScreen from '$lib/components/LockScreen.svelte';
	import EntryList from '$lib/components/EntryList.svelte';
	import EntryDetail from '$lib/components/EntryDetail.svelte';
	import EntryForm from '$lib/components/EntryForm.svelte';
	import FolderTree from '$lib/components/FolderTree.svelte';
	import TagCloud from '$lib/components/TagCloud.svelte';
	import Toast from '$lib/components/Toast.svelte';

	import type { Entry } from '$lib/stores/entries';

	type ViewMode = 'empty' | 'detail' | 'edit' | 'create';

	let selectedId: string | null = $state(null);
	let viewMode: ViewMode = $state('empty');
	let searchQuery = $state('');
	let filterFolderId: string | null = $state(null);
	let filterTagIds: string[] = $state([]);
	let clipboardClearedToast = $state(false);

	let selectedEntry = $derived(
		selectedId ? $entries.find((e) => e.id === selectedId) ?? null : null
	);

	onMount(async () => {
		try {
			await vault.checkStatus();
		} catch {
			// Tauri not available (dev in browser)
		}
		listen('clipboard-cleared', () => {
			clipboardClearedToast = true;
			setTimeout(() => (clipboardClearedToast = false), 2000);
		}).catch(() => {});
	});

	$effect(() => {
		if ($vault.isUnlocked) {
			entries.load();
			folders.load();
			tags.load();
		}
	});

	function selectEntry(id: string) {
		selectedId = id;
		viewMode = 'detail';
	}

	function startCreate() {
		selectedId = null;
		viewMode = 'create';
	}

	function startEdit() {
		viewMode = 'edit';
	}

	function cancelEdit() {
		if (selectedId) {
			viewMode = 'detail';
		} else {
			viewMode = 'empty';
		}
	}

	async function saveEntry(entry: Entry) {
		try {
			if (viewMode === 'create') {
				await entries.create(entry);
				selectedId = entry.id;
			} else {
				await entries.save(entry);
			}
			viewMode = 'detail';
		} catch (e) {
			console.error('保存失败:', e);
		}
	}

	async function deleteEntry() {
		if (!selectedEntry) return;
		const confirmed = window.confirm(`确定要删除「${selectedEntry.title}」吗？`);
		if (!confirmed) return;
		try {
			await entries.remove(selectedEntry.id);
			selectedId = null;
			viewMode = 'empty';
		} catch (e) {
			console.error('删除失败:', e);
		}
	}

	async function handleLock() {
		await vault.lock();
		selectedId = null;
		viewMode = 'empty';
	}
</script>

{#if !$vault.isUnlocked}
	<LockScreen />
{:else}
	<div class="flex h-screen">
		<!-- Left sidebar -->
		<div class="flex w-[35%] min-w-0 flex-col bg-dark-sidebar">
			<!-- Search bar (fixed at top) -->
			<div class="border-b border-dark-border p-3">
				<input
					type="text"
					bind:value={searchQuery}
					placeholder="搜索条目..."
					class="w-full rounded-md border border-dark-border bg-dark-card px-3 py-2 text-sm text-dark-text outline-none placeholder:text-dark-muted focus:border-accent"
				/>
			</div>

			<!-- Folder navigation -->
			<FolderTree
				selectedFolderId={filterFolderId}
				onselect={(id) => {
					filterFolderId = id;
					filterTagIds = [];
				}}
			/>

			<!-- Tag cloud (multi-select) -->
			<TagCloud
				selectedTagIds={filterTagIds}
				onselect={(ids) => {
					filterTagIds = ids;
					filterFolderId = null;
				}}
			/>

			<!-- Scrollable entry list -->
			<div class="min-h-0 flex-1 overflow-y-auto">
				<EntryList
					entries={$entries}
					{selectedId}
					{searchQuery}
					{filterFolderId}
					{filterTagIds}
					onselect={selectEntry}
				/>
			</div>

			<!-- Bottom bar (fixed at bottom) -->
			<div class="flex items-center justify-between border-t border-dark-border p-2">
				<button
					class="cursor-pointer rounded-md px-2 py-1.5 text-xs text-dark-muted hover:text-accent"
					onclick={handleLock}
					title="锁定保险库"
				>
					🔒 锁定
				</button>
				<button
					class="cursor-pointer rounded-md bg-accent px-4 py-1.5 text-xs font-medium text-white hover:bg-accent-hover"
					onclick={startCreate}
				>
					+ 新建条目
				</button>
			</div>
		</div>

		<!-- Right panel -->
		<div class="flex-1 bg-dark-bg">
			{#if viewMode === 'detail' && selectedEntry}
				<EntryDetail
					entry={selectedEntry}
					onedit={startEdit}
					ondelete={deleteEntry}
				/>
			{:else if viewMode === 'edit' && selectedEntry}
				<EntryForm
					entry={selectedEntry}
					onsave={saveEntry}
					oncancel={cancelEdit}
				/>
			{:else if viewMode === 'create'}
				<EntryForm
					onsave={saveEntry}
					oncancel={cancelEdit}
				/>
			{:else}
				<div class="flex h-full items-center justify-center text-dark-muted">
					<div class="text-center">
						<div class="mb-2 text-3xl">📋</div>
						<p class="text-sm">选择一个条目查看详情</p>
						<p class="text-sm">或创建新条目</p>
					</div>
				</div>
			{/if}
		</div>
	</div>
{/if}

<Toast message="剪贴板已清除" visible={clipboardClearedToast} />
