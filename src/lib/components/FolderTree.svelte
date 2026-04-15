<script lang="ts">
	import { folders } from '$lib/stores/folders';

	interface Props {
		selectedFolderId: string | null;
		onselect: (folderId: string | null) => void;
	}

	let { selectedFolderId, onselect }: Props = $props();
	let newFolderName = $state('');
	let showNewFolder = $state(false);

	async function handleCreate() {
		if (!newFolderName.trim()) return;
		await folders.create(newFolderName.trim());
		newFolderName = '';
		showNewFolder = false;
	}
</script>

<div class="border-b border-dark-border py-2">
	<div class="flex items-center justify-between px-3 pb-1">
		<span class="text-xs font-medium uppercase tracking-wide text-dark-muted">文件夹</span>
		<button
			class="cursor-pointer text-xs text-dark-muted hover:text-accent"
			onclick={() => (showNewFolder = !showNewFolder)}
		>
			+ 新建
		</button>
	</div>

	{#if showNewFolder}
		<div class="px-3 pb-1">
			<div class="flex gap-1">
				<input
					type="text"
					bind:value={newFolderName}
					placeholder="文件夹名称"
					class="flex-1 rounded border border-dark-border bg-dark-card px-2 py-1 text-xs text-dark-text outline-none placeholder:text-dark-muted focus:border-accent"
					onkeydown={(e) => e.key === 'Enter' && handleCreate()}
				/>
				<button
					class="cursor-pointer rounded bg-accent px-2 py-1 text-xs text-white hover:bg-accent-hover"
					onclick={handleCreate}
				>
					确定
				</button>
			</div>
		</div>
	{/if}

	<div class="flex flex-col">
		<button
			class="w-full cursor-pointer px-3 py-1 text-left text-xs transition-colors {selectedFolderId === null
				? 'bg-dark-card text-dark-text'
				: 'text-dark-secondary hover:bg-dark-card/50'}"
			onclick={() => onselect(null)}
		>
			所有条目
		</button>

		{#each $folders as folder (folder.id)}
			<button
				class="w-full cursor-pointer px-3 py-1 text-left text-xs transition-colors {selectedFolderId === folder.id
					? 'bg-dark-card text-dark-text'
					: 'text-dark-secondary hover:bg-dark-card/50'}"
				onclick={() => onselect(folder.id)}
			>
				{folder.name}
			</button>
		{/each}
	</div>
</div>
