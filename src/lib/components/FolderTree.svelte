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

<div class="border-b border-line py-2">
	<div class="flex items-center justify-between px-3 pb-1">
		<span class="text-xs font-medium uppercase tracking-wide text-hint">文件夹</span>
		<button
			class="cursor-pointer text-xs text-hint hover:text-accent"
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
					class="flex-1 rounded border border-line bg-card px-2 py-1 text-xs text-heading outline-none placeholder:text-hint focus:border-accent"
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
				? 'bg-card text-heading'
				: 'text-body hover:bg-card-subtle'}"
			onclick={() => onselect(null)}
		>
			所有条目
		</button>

		{#each $folders as folder (folder.id)}
			<button
				class="w-full cursor-pointer px-3 py-1 text-left text-xs transition-colors {selectedFolderId === folder.id
					? 'bg-card text-heading'
					: 'text-body hover:bg-card-subtle'}"
				onclick={() => onselect(folder.id)}
			>
				{folder.name}
			</button>
		{/each}
	</div>
</div>
