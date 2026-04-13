<script lang="ts">
	import { onMount } from 'svelte';
	import { vault } from '$lib/stores/vault';
	import { entries } from '$lib/stores/entries';
	import LockScreen from '$lib/components/LockScreen.svelte';
	import EntryList from '$lib/components/EntryList.svelte';
	import EntryDetail from '$lib/components/EntryDetail.svelte';
	import EntryForm from '$lib/components/EntryForm.svelte';

	import type { Entry } from '$lib/stores/entries';

	type ViewMode = 'empty' | 'detail' | 'edit' | 'create';

	let selectedId: string | null = $state(null);
	let viewMode: ViewMode = $state('empty');

	let selectedEntry = $derived(
		selectedId ? $entries.find((e) => e.id === selectedId) ?? null : null
	);

	onMount(async () => {
		try {
			await vault.checkStatus();
		} catch {
			// Tauri not available (dev in browser)
		}
	});

	$effect(() => {
		if ($vault.isUnlocked) {
			entries.load();
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
		<!-- 左侧列表 -->
		<div class="w-[35%] min-w-0">
			<EntryList
				entries={$entries}
				{selectedId}
				onselect={selectEntry}
				oncreate={startCreate}
			/>
		</div>

		<!-- 右侧面板 -->
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

	<!-- 锁定按钮 -->
	<button
		class="fixed right-3 top-3 cursor-pointer rounded-md px-2 py-1 text-xs text-dark-muted hover:text-accent"
		onclick={handleLock}
		title="锁定保险库"
	>
		🔒 锁定
	</button>
{/if}
