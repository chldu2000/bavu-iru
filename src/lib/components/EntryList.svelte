<script lang="ts">
	import type { Entry } from '$lib/stores/entries';

	interface Props {
		entries: Entry[];
		selectedId: string | null;
		onselect: (id: string) => void;
		oncreate: () => void;
	}

	let { entries, selectedId, onselect, oncreate }: Props = $props();
	let query = $state('');

	let filtered = $derived(
		entries.filter((e) => {
			if (!query) return true;
			const q = query.toLowerCase();
			return (
				e.title.toLowerCase().includes(q) ||
				(e.username ?? '').toLowerCase().includes(q) ||
				(e.url ?? '').toLowerCase().includes(q)
			);
		})
	);
</script>

<div class="flex h-full flex-col bg-dark-sidebar">
	<!-- 搜索框 -->
	<div class="border-b border-dark-border p-3">
		<input
			type="text"
			bind:value={query}
			placeholder="搜索条目..."
			class="w-full rounded-md border border-dark-border bg-dark-card px-3 py-2 text-sm text-dark-text outline-none placeholder:text-dark-muted focus:border-accent"
		/>
	</div>

	<!-- 条目列表 -->
	<div class="flex-1 overflow-y-auto">
		{#if filtered.length === 0}
			<div class="flex flex-col items-center justify-center py-12 text-dark-muted">
				<div class="mb-2 text-2xl">🔒</div>
				<p class="text-xs">{entries.length === 0 ? '还没有条目' : '没有匹配结果'}</p>
			</div>
		{:else}
			{#each filtered as entry (entry.id)}
				<button
					class="w-full cursor-pointer border-l-3 px-3 py-2.5 text-left transition-colors {selectedId === entry.id
						? 'border-l-accent bg-dark-card'
						: 'border-l-transparent hover:bg-dark-card/50'}"
					onclick={() => onselect(entry.id)}
				>
					<div class="truncate text-sm font-medium {selectedId === entry.id ? 'text-dark-text' : 'text-dark-secondary'}">
						{entry.title || '无标题'}
					</div>
					<div class="truncate text-xs text-dark-muted">
						{entry.username ?? ''}
					</div>
				</button>
			{/each}
		{/if}
	</div>

	<!-- 新建按钮 -->
	<div class="border-t border-dark-border p-2 text-center">
		<button
			class="cursor-pointer rounded-md bg-accent px-4 py-1.5 text-xs font-medium text-white hover:bg-accent-hover"
			onclick={oncreate}
		>
			+ 新建条目
		</button>
	</div>
</div>
