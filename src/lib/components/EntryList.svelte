<script lang="ts">
	import type { Entry } from '$lib/stores/entries';
	import { entries } from '$lib/stores/entries';

	interface Props {
		entries: Entry[];
		selectedId: string | null;
		filterFolderId: string | null;
		filterTagId: string | null;
		onselect: (id: string) => void;
		oncreate: () => void;
		onlock: () => void;
	}

	let { entries: allEntries, selectedId, filterFolderId, filterTagId, onselect, oncreate, onlock }: Props = $props();

	let filtered = $derived.by(() => {
		let result = allEntries;

		if (filterFolderId !== null) {
			result = result.filter((e) => e.folder_id === filterFolderId);
		}

		// Note: filterTagId filtering requires entry-tag association via the entry_tags table.
		// For now, tags stored in entry.tags JSON field are used. This will be enhanced later.

		return result;
	});

	async function handleToggleFavorite(id: string, e: Event) {
		e.stopPropagation();
		await entries.toggleFavorite(id);
	}
</script>

<div class="flex flex-col">
	<div class="flex-1 overflow-y-auto">
		{#if filtered.length === 0}
			<div class="flex flex-col items-center justify-center py-12 text-dark-muted">
				<div class="mb-2 text-2xl">🔒</div>
				<p class="text-xs">{allEntries.length === 0 ? '还没有条目' : '没有匹配结果'}</p>
			</div>
		{:else}
			{#each filtered as entry (entry.id)}
				<button
					class="group w-full cursor-pointer border-l-3 px-3 py-2 text-left transition-colors {selectedId === entry.id
						? 'border-l-accent bg-dark-card'
						: 'border-l-transparent hover:bg-dark-card/50'}"
					onclick={() => onselect(entry.id)}
				>
					<div class="flex items-center gap-1">
						<span
							role="button"
							tabindex="0"
							class="cursor-pointer text-xs {entry.is_favorite
								? 'text-accent opacity-100'
								: 'opacity-0 transition-opacity group-hover:opacity-100'}"
							onclick={(e) => handleToggleFavorite(entry.id, e)}
							onkeydown={(e) => e.key === 'Enter' && handleToggleFavorite(entry.id, e)}
							title={entry.is_favorite ? '取消收藏' : '收藏'}
						>
							{entry.is_favorite ? '★' : '☆'}
						</span>
						<span
							class="truncate text-sm font-medium {selectedId === entry.id
								? 'text-dark-text'
								: 'text-dark-secondary'}"
						>
							{entry.title || '无标题'}
						</span>
					</div>
					<div class="truncate pl-4 text-xs text-dark-muted">
						{entry.username ?? ''}
					</div>
				</button>
			{/each}
		{/if}
	</div>

	<div class="flex items-center justify-between border-t border-dark-border p-2">
		<button
			class="cursor-pointer rounded-md px-2 py-1.5 text-xs text-dark-muted hover:text-accent"
			onclick={onlock}
			title="锁定保险库"
		>
			🔒 锁定
		</button>
		<button
			class="cursor-pointer rounded-md bg-accent px-4 py-1.5 text-xs font-medium text-white hover:bg-accent-hover"
			onclick={oncreate}
		>
			+ 新建条目
		</button>
	</div>
</div>
