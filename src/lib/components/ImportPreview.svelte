<script lang="ts">
	import type { PreviewResult } from '$lib/utils/tauri';
	import { importVault } from '$lib/utils/tauri';

	interface Props {
		preview: PreviewResult;
		format: string;
		content: string;
		password?: string;
		oncomplete: () => void;
		oncancel: () => void;
	}

	let { preview, format, content, password, oncomplete, oncancel }: Props = $props();

	let resolutions = $state<Record<string, string>>({});
	for (const dup of preview.duplicates) {
		resolutions[dup.existing_id] = 'keep';
	}

	let loading = $state(false);
	let error = $state('');

	async function handleImport() {
		loading = true;
		error = '';
		try {
			const result = await importVault(format, content, resolutions, password);
			alert(`导入完成：新增 ${result.imported} 条，跳过 ${result.skipped} 条，替换 ${result.replaced} 条`);
			oncomplete();
		} catch (e: unknown) {
			error = String(e);
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex h-full flex-col bg-page">
	<!-- Header -->
	<div class="flex items-center justify-between border-b border-line px-5 py-3">
		<h2 class="text-lg font-bold text-heading">导入预览</h2>
		<button class="cursor-pointer text-hint hover:text-heading" onclick={oncancel}>✕</button>
	</div>

	<!-- Summary -->
	<div class="border-b border-line px-5 py-3">
		<div class="flex gap-6 text-sm">
			<span class="text-body">共 <span class="font-medium text-heading">{preview.total}</span> 条</span>
			{#if preview.duplicates.length > 0}
				<span class="text-yellow-400">重复 <span class="font-medium">{preview.duplicates.length}</span> 条</span>
			{:else}
				<span class="text-green-400">无重复</span>
			{/if}
		</div>
	</div>

	{#if error}
		<div class="mx-5 mt-3 rounded-md bg-red-500/10 px-3 py-2 text-sm text-red-400">{error}</div>
	{/if}

	<!-- Duplicate list -->
	{#if preview.duplicates.length > 0}
		<div class="flex-1 overflow-y-auto p-5">
			<h3 class="mb-3 text-sm font-medium text-heading">重复条目处理</h3>
			<div class="flex flex-col gap-2">
				{#each preview.duplicates as dup}
					<div class="flex items-center justify-between rounded-lg border border-line bg-card px-4 py-3">
						<div class="min-w-0 flex-1">
							<div class="text-sm font-medium text-heading">{dup.imported_title}</div>
							<div class="text-xs text-hint">用户名：{dup.imported_username || '(空)'}</div>
						</div>
						<select
							class="ml-3 rounded-md border border-line bg-sidebar px-2 py-1 text-xs text-heading outline-none focus:border-accent"
							bind:value={resolutions[dup.existing_id]}
						>
							<option value="keep">保留两者</option>
							<option value="skip">跳过</option>
							<option value="replace">覆盖现有</option>
						</select>
					</div>
				{/each}
			</div>
		</div>
	{:else}
		<div class="flex flex-1 items-center justify-center text-hint">
			<div class="text-center">
				<p class="text-sm">无重复条目，可直接导入</p>
			</div>
		</div>
	{/if}

	<!-- Footer -->
	<div class="flex items-center justify-end border-t border-line px-5 py-3">
		<button
			class="mr-2 cursor-pointer rounded-md border border-line px-4 py-2 text-sm text-body hover:bg-sidebar"
			onclick={oncancel}
		>
			取消
		</button>
		<button
			class="cursor-pointer rounded-md bg-accent px-4 py-2 text-sm font-medium text-white hover:bg-accent-hover disabled:opacity-50"
			onclick={handleImport}
			disabled={loading}
		>
			{loading ? '导入中...' : '确认导入'}
		</button>
	</div>
</div>
