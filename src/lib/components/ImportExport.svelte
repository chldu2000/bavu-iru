<script lang="ts">
	import { entries } from '$lib/stores/entries';
	import { previewImport, checkIntegrity } from '$lib/utils/tauri';
	import type { PreviewResult, IntegrityResult, ExportData } from '$lib/utils/tauri';
	import ExportConfirm from './ExportConfirm.svelte';
	import ImportPreview from './ImportPreview.svelte';

	interface Props {
		onclose: () => void;
	}

	let { onclose }: Props = $props();

	type Tab = 'import' | 'export' | 'integrity';
	let tab = $state<Tab>('import');

	// Import state
	let importFormat = $state('json');
	let fileContent = $state('');
	let fileName = $state('');
	let importPassword = $state('');
	let importLoading = $state(false);
	let importError = $state('');
	let previewResult = $state<PreviewResult | null>(null);

	// Export state
	let showExportConfirm = $state(false);

	// Integrity state
	let integrityResult = $state<IntegrityResult | null>(null);
	let integrityLoading = $state(false);

	async function handleFileSelect(e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;
		fileName = file.name;
		const text = await file.text();
		fileContent = text;
	}

	async function handlePreview() {
		importError = '';
		if (!fileContent) {
			importError = '请先选择文件';
			return;
		}
		importLoading = true;
		try {
			previewResult = await previewImport(
				importFormat,
				fileContent,
				importFormat === 'encrypted' ? importPassword : undefined
			);
		} catch (e: unknown) {
			importError = String(e);
		} finally {
			importLoading = false;
		}
	}

	async function handleImportComplete() {
		previewResult = null;
		fileContent = '';
		fileName = '';
		await entries.load();
	}

	function handleExportComplete(_data: ExportData) {
		showExportConfirm = false;
	}

	async function handleIntegrityCheck() {
		integrityLoading = true;
		try {
			integrityResult = await checkIntegrity();
		} catch (e: unknown) {
			integrityResult = {
				status: 'error',
				issues: [{ severity: 'error', message: String(e) }]
			};
		} finally {
			integrityLoading = false;
		}
	}

	function severityColor(severity: string): string {
		switch (severity) {
			case 'error': return 'text-red-400';
			case 'warning': return 'text-yellow-400';
			default: return 'text-hint';
		}
	}
</script>

<div class="flex h-full flex-col bg-page">
	<!-- Header -->
	<div class="flex items-center justify-between border-b border-line px-5 py-3">
		<h2 class="text-lg font-bold text-heading">导入 / 导出</h2>
		<button class="cursor-pointer text-hint hover:text-heading" onclick={onclose}>✕</button>
	</div>

	<!-- Tabs -->
	<div class="flex border-b border-line">
		<button
			class="flex-1 cursor-pointer px-4 py-2.5 text-sm {tab === 'import' ? 'border-b-2 border-accent font-medium text-accent' : 'text-hint hover:text-heading'}"
			onclick={() => (tab = 'import')}
		>
			导入
		</button>
		<button
			class="flex-1 cursor-pointer px-4 py-2.5 text-sm {tab === 'export' ? 'border-b-2 border-accent font-medium text-accent' : 'text-hint hover:text-heading'}"
			onclick={() => (tab = 'export')}
		>
			导出
		</button>
		<button
			class="flex-1 cursor-pointer px-4 py-2.5 text-sm {tab === 'integrity' ? 'border-b-2 border-accent font-medium text-accent' : 'text-hint hover:text-heading'}"
			onclick={() => (tab = 'integrity')}
		>
			完整性检查
		</button>
	</div>

	<!-- Content -->
	<div class="flex-1 overflow-y-auto">
		{#if tab === 'import'}
			{#if previewResult}
				<ImportPreview
					preview={previewResult}
					format={importFormat}
					content={fileContent}
					password={importFormat === 'encrypted' ? importPassword : undefined}
					oncomplete={handleImportComplete}
					oncancel={() => (previewResult = null)}
				/>
			{:else}
				<div class="p-5">
					<div class="flex flex-col gap-4">
						<label class="flex flex-col gap-1">
							<span class="text-sm text-body">导入格式</span>
							<select
								class="rounded-md border border-line bg-card px-3 py-2 text-sm text-heading outline-none focus:border-accent"
								bind:value={importFormat}
							>
								<option value="json">JSON（本应用格式）</option>
								<option value="csv">CSV（通用）</option>
								<option value="bitwarden">Bitwarden JSON</option>
								<option value="keepass">KeePass CSV</option>
								<option value="chrome">Chrome / Firefox CSV</option>
								<option value="encrypted">加密备份 (.bvault)</option>
							</select>
						</label>

						<label class="flex flex-col gap-1">
							<span class="text-sm text-body">选择文件</span>
							<input
								type="file"
								class="text-sm text-body file:mr-3 file:rounded-md file:border file:border-line file:bg-card file:px-3 file:py-1.5 file:text-sm file:text-heading file:cursor-pointer"
								accept=".csv,.json,.bvault,.txt"
								onchange={handleFileSelect}
							/>
							{#if fileName}
								<span class="text-xs text-hint">已选择：{fileName}</span>
							{/if}
						</label>

						{#if importFormat === 'encrypted'}
							<label class="flex flex-col gap-1">
								<span class="text-sm text-body">导出密码（解密用）</span>
								<input
									type="password"
									class="rounded-md border border-line bg-card px-3 py-2 text-sm text-heading outline-none focus:border-accent"
									bind:value={importPassword}
									placeholder="输入该备份的导出密码"
								/>
							</label>
						{/if}

						{#if importError}
							<div class="rounded-md bg-red-500/10 px-3 py-2 text-sm text-red-400">{importError}</div>
						{/if}

						<button
							class="cursor-pointer rounded-md bg-accent px-4 py-2 text-sm font-medium text-white hover:bg-accent-hover disabled:opacity-50"
							onclick={handlePreview}
							disabled={importLoading}
						>
							{importLoading ? '解析中...' : '预览导入'}
						</button>
					</div>
				</div>
			{/if}
		{:else if tab === 'export'}
			<div class="flex h-full items-center justify-center p-5">
				<div class="w-full max-w-sm text-center">
					<div class="mb-4 text-3xl">📦</div>
					<p class="mb-4 text-sm text-body">导出所有密码条目到文件。导出时需要验证主密码。</p>
					<button
						class="cursor-pointer rounded-md bg-accent px-6 py-2 text-sm font-medium text-white hover:bg-accent-hover"
						onclick={() => (showExportConfirm = true)}
					>
						开始导出
					</button>
				</div>
			</div>
		{:else}
			<!-- Integrity check -->
			<div class="p-5">
				<div class="mb-4 text-center">
					<p class="text-sm text-body">检查数据库完整性和加密数据一致性。</p>
					<button
						class="mt-3 cursor-pointer rounded-md bg-accent px-6 py-2 text-sm font-medium text-white hover:bg-accent-hover disabled:opacity-50"
						onclick={handleIntegrityCheck}
						disabled={integrityLoading}
					>
						{integrityLoading ? '检查中...' : '开始检查'}
					</button>
				</div>

				{#if integrityResult}
					<div class="mt-4">
						<div class="mb-3 flex items-center gap-2">
							<span class="text-sm font-medium text-heading">状态：</span>
							<span class="rounded-full px-2 py-0.5 text-xs font-medium
								{integrityResult.status === 'ok' ? 'bg-green-500/20 text-green-400' : ''}
								{integrityResult.status === 'warning' ? 'bg-yellow-500/20 text-yellow-400' : ''}
								{integrityResult.status === 'error' ? 'bg-red-500/20 text-red-400' : ''}
							">
								{integrityResult.status === 'ok' ? '正常' : integrityResult.status === 'warning' ? '警告' : '错误'}
							</span>
						</div>
						{#each integrityResult.issues as issue}
							<div class="mb-2 rounded-md border border-line bg-card px-4 py-2.5">
								<span class="text-xs font-medium {severityColor(issue.severity)}">[{issue.severity.toUpperCase()}]</span>
								<span class="ml-2 text-sm text-body">{issue.message}</span>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		{/if}
	</div>
</div>

{#if showExportConfirm}
	<ExportConfirm
		onclose={() => (showExportConfirm = false)}
		onexport={handleExportComplete}
	/>
{/if}
