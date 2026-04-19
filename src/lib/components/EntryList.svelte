<script lang="ts">
	import type { Entry } from '$lib/stores/entries';
	import { entries } from '$lib/stores/entries';

	interface Props {
		entries: Entry[];
		selectedId: string | null;
		searchQuery: string;
		filterFolderId: string | null;
		filterTagIds: string[];
		onselect: (id: string) => void;
	}

	let { entries: allEntries, selectedId, searchQuery, filterFolderId, filterTagIds, onselect }: Props = $props();

	let filtered = $derived.by(() => {
		let result = allEntries;

		// Filter by folder
		if (filterFolderId !== null) {
			result = result.filter((e) => e.folder_id === filterFolderId);
		}

		// Filter by tags — match entries whose tags JSON includes any selected tag
		if (filterTagIds.length > 0) {
			result = result.filter((e) => {
				if (!e.tags) return false;
				try {
					const ids: string[] = JSON.parse(e.tags);
					return filterTagIds.some((tagId) => ids.includes(tagId));
				} catch {
					return false;
				}
			});
		}

		// Filter by search query
		if (searchQuery) {
			const q = searchQuery.toLowerCase();
			result = result.filter(
				(e) =>
					e.title.toLowerCase().includes(q) ||
					(e.username ?? '').toLowerCase().includes(q) ||
					(e.url ?? '').toLowerCase().includes(q)
			);
		}

		return result;
	});

	async function handleToggleFavorite(id: string, e: Event) {
		e.stopPropagation();
		await entries.toggleFavorite(id);
		// Reload to get correct sort order (favorites pinned to top)
		await entries.load();
	}
</script>

{#if filtered.length === 0}
	<div class="flex flex-col items-center justify-center py-12 text-hint">
		<div class="mb-2 text-2xl">🔒</div>
		<p class="text-xs">{allEntries.length === 0 ? '还没有条目' : '没有匹配结果'}</p>
	</div>
{:else}
	{#each filtered as entry (entry.id)}
		<button
			class="group w-full cursor-pointer border-l-3 px-3 py-2 text-left transition-colors {selectedId === entry.id
				? 'border-l-accent bg-card'
				: 'border-l-transparent hover:bg-card-subtle'}"
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
						? 'text-heading'
						: 'text-body'}"
				>
					{entry.title || '无标题'}
				</span>
			</div>
			<div class="truncate pl-4 text-xs text-hint">
				{entry.username ?? ''}
			</div>
		</button>
	{/each}
{/if}
