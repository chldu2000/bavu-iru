<script lang="ts">
	import type { Entry } from '$lib/stores/entries';
	import PasswordField from './PasswordField.svelte';

	interface Props {
		entry: Entry;
		onedit: () => void;
		ondelete: () => void;
	}

	let { entry, onedit, ondelete }: Props = $props();

	async function copyText(text: string) {
		await navigator.clipboard.writeText(text);
	}
</script>

<div class="flex h-full flex-col p-5">
	<!-- 顶栏 -->
	<div class="mb-5 flex items-center justify-between">
		<h2 class="text-lg font-bold text-dark-text">{entry.title || '无标题'}</h2>
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
		{#if entry.username}
			<div>
				<span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">用户名</span>
				<div class="flex items-center justify-between rounded-md border border-dark-border bg-dark-card px-3 py-2 text-sm text-dark-text">
					<span>{entry.username}</span>
					<button
						class="cursor-pointer text-xs text-accent hover:underline"
						onclick={() => copyText(entry.username!)}
					>
						复制
					</button>
				</div>
			</div>
		{/if}

		{#if entry.password !== null}
			<div>
				<span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">密码</span>
				<PasswordField value={entry.password ?? ''} />
			</div>
		{/if}

		{#if entry.url}
			<div>
				<span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">网址</span>
				<div class="rounded-md border border-dark-border bg-dark-card px-3 py-2 text-sm text-accent">
					{entry.url}
				</div>
			</div>
		{/if}

		{#if entry.notes}
			<div>
				<span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">备注</span>
				<div class="whitespace-pre-wrap rounded-md border border-dark-border bg-dark-card px-3 py-2 text-sm text-dark-secondary">
					{entry.notes}
				</div>
			</div>
		{/if}
	</div>
</div>
