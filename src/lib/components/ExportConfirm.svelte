<script lang="ts">
	import { exportVault } from '$lib/utils/tauri';
	import type { ExportData } from '$lib/utils/tauri';

	interface Props {
		onclose: () => void;
		onexport: (data: ExportData) => void;
	}

	let { onclose, onexport }: Props = $props();

	let format = $state('json');
	let masterPassword = $state('');
	let exportPassword = $state('');
	let exportPasswordConfirm = $state('');
	let error = $state('');
	let loading = $state(false);

	function triggerDownload(data: ExportData) {
		const blob = new Blob([data.data], { type: 'text/plain' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = data.filename;
		a.click();
		URL.revokeObjectURL(url);
	}

	async function handleExport() {
		error = '';
		if (!masterPassword) {
			error = '请输入主密码以验证身份';
			return;
		}
		if (format === 'encrypted') {
			if (!exportPassword || exportPassword.length < 8) {
				error = '导出密码至少需要 8 个字符';
				return;
			}
			if (exportPassword !== exportPasswordConfirm) {
				error = '两次输入的导出密码不一致';
				return;
			}
		}

		loading = true;
		try {
			const data = await exportVault(format, masterPassword, format === 'encrypted' ? exportPassword : undefined);
			triggerDownload(data);
			onexport(data);
		} catch (e: unknown) {
			error = String(e);
		} finally {
			loading = false;
		}
	}
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={onclose}>
	<div class="w-full max-w-md rounded-xl bg-card p-6 shadow-xl" onclick={(e) => e.stopPropagation()}>
		<h2 class="mb-4 text-lg font-bold text-heading">导出保险库</h2>

		{#if error}
			<div class="mb-3 rounded-md bg-red-500/10 px-3 py-2 text-sm text-red-400">{error}</div>
		{/if}

		<div class="flex flex-col gap-4">
			<label class="flex flex-col gap-1">
				<span class="text-sm text-body">导出格式</span>
				<select
					class="rounded-md border border-line bg-sidebar px-3 py-2 text-sm text-heading outline-none focus:border-accent"
					bind:value={format}
				>
					<option value="json">JSON（无损）</option>
					<option value="csv">CSV（通用）</option>
					<option value="encrypted">加密备份 (.bvault)</option>
				</select>
			</label>

			<label class="flex flex-col gap-1">
				<span class="text-sm text-body">主密码（验证身份）</span>
				<input
					type="password"
					class="rounded-md border border-line bg-sidebar px-3 py-2 text-sm text-heading outline-none focus:border-accent"
					bind:value={masterPassword}
					placeholder="输入主密码"
				/>
			</label>

			{#if format === 'encrypted'}
				<label class="flex flex-col gap-1">
					<span class="text-sm text-body">导出密码</span>
					<input
						type="password"
						class="rounded-md border border-line bg-sidebar px-3 py-2 text-sm text-heading outline-none focus:border-accent"
						bind:value={exportPassword}
						placeholder="设置导出文件密码"
					/>
				</label>
				<label class="flex flex-col gap-1">
					<span class="text-sm text-body">确认导出密码</span>
					<input
						type="password"
						class="rounded-md border border-line bg-sidebar px-3 py-2 text-sm text-heading outline-none focus:border-accent"
						bind:value={exportPasswordConfirm}
						placeholder="再次输入导出密码"
					/>
				</label>
			{/if}
		</div>

		<div class="mt-5 flex justify-end gap-2">
			<button
				class="cursor-pointer rounded-md border border-line px-4 py-2 text-sm text-body hover:bg-sidebar"
				onclick={onclose}
			>
				取消
			</button>
			<button
				class="cursor-pointer rounded-md bg-accent px-4 py-2 text-sm font-medium text-white hover:bg-accent-hover disabled:opacity-50"
				onclick={handleExport}
				disabled={loading}
			>
				{loading ? '导出中...' : '导出'}
			</button>
		</div>
	</div>
</div>
