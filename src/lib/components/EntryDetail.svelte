<script lang="ts">
	import type { Entry } from '$lib/stores/entries';
	import { entries } from '$lib/stores/entries';
	import { folders } from '$lib/stores/folders';
	import { tags } from '$lib/stores/tags';
	import PasswordField from './PasswordField.svelte';
	import Toast from './Toast.svelte';

	interface Props {
		entry: Entry;
		onedit: () => void;
		ondelete: () => void;
	}

	let { entry, onedit, ondelete }: Props = $props();

	let folderName = $derived.by(() => {
		if (!entry.folder_id) return null;
		return $folders.find((f) => f.id === entry.folder_id)?.name ?? null;
	});

	let entryTags = $derived.by(() => {
		if (!entry.tags) return [];
		try {
			const ids: string[] = JSON.parse(entry.tags);
			return ids
				.map((id) => $tags.find((t) => t.id === id))
				.filter((t): t is NonNullable<typeof t> => t !== undefined);
		} catch {
			return [];
		}
	});

	let copiedField = $state('');
	let showCopiedToast = $state(false);

	async function copyText(text: string, field: string = '') {
		await navigator.clipboard.writeText(text);
		copiedField = field;
		showCopiedToast = true;
		setTimeout(() => {
			showCopiedToast = false;
			copiedField = '';
		}, 2000);
	}

	async function handleToggleFavorite() {
		await entries.toggleFavorite(entry.id);
	}
</script>

<div class="flex h-full flex-col p-5">
	<!-- 顶栏 -->
	<div class="mb-5 flex items-center justify-between">
		<div class="flex items-center gap-2">
			<button
				class="cursor-pointer text-lg {entry.is_favorite
					? 'text-accent'
					: 'text-dark-muted hover:text-accent'}"
				onclick={handleToggleFavorite}
				title={entry.is_favorite ? '取消收藏' : '收藏'}
			>
				{entry.is_favorite ? '★' : '☆'}
			</button>
			<h2 class="text-lg font-bold text-dark-text">{entry.title || '无标题'}</h2>
		</div>
		<div class="flex gap-3">
			<button
				class="cursor-pointer text-xs text-accent hover:underline"
				onclick={onedit}
			>
				编辑
			</button>
			<button
				class="cursor-pointer text-xs text-danger hover:underline"
				onclick={ondelete}
			>
				删除
			</button>
		</div>
	</div>

	<!-- 字段 -->
	<div class="flex flex-col gap-4">
		{#if folderName}
			<div>
				<span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">文件夹</span>
				<div
					class="rounded-md border border-dark-border bg-dark-card px-3 py-2 text-sm text-dark-text"
				>
					{folderName}
				</div>
			</div>
		{/if}

		{#if entryTags.length > 0}
			<div>
				<span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">标签</span>
				<div class="flex flex-wrap gap-1">
					{#each entryTags as tag (tag.id)}
						<span
							class="rounded-full px-2 py-0.5 text-xs text-white ring-1 ring-white/20"
							style:background-color={tag.color}
						>
							{tag.name}
						</span>
					{/each}
				</div>
			</div>
		{/if}

		{#if entry.username}
			<div>
				<span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">用户名</span>
				<div
					class="flex items-center justify-between rounded-md border border-dark-border bg-dark-card px-3 py-2 text-sm text-dark-text"
				>
					<span>{entry.username}</span>
					<button
						class="cursor-pointer text-xs text-accent hover:underline"
						onclick={() => copyText(entry.username!, 'username')}
					>
						{copiedField === 'username' ? '已复制' : '复制'}
					</button>
				</div>
			</div>
		{/if}

		{#if entry.password !== null}
			<div>
				<span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">密码</span>
				<PasswordField value={entry.password ?? ''} oncopy={() => copyText(entry.password!, 'password')} />
			</div>
		{/if}

		{#if entry.url}
			<div>
				<span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">网址</span>
				<div
					class="rounded-md border border-dark-border bg-dark-card px-3 py-2 text-sm text-accent"
				>
					{entry.url}
				</div>
			</div>
		{/if}

		{#if entry.notes}
			<div>
				<span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">备注</span>
				<div
					class="whitespace-pre-wrap rounded-md border border-dark-border bg-dark-card px-3 py-2 text-sm text-dark-secondary"
				>
					{entry.notes}
				</div>
			</div>
		{/if}
	</div>

	<Toast message="已复制到剪贴板" visible={showCopiedToast} />
</div>
