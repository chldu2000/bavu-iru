<script lang="ts">
	import { settings, saveSettings } from '$lib/stores/settings';

	interface Props {
		onclose: () => void;
	}

	let { onclose }: Props = $props();

	let localSettings = $state({ ...$settings });
	let saved = $state(false);

	async function save() {
		await saveSettings({ ...localSettings });
		saved = true;
		setTimeout(() => (saved = false), 2000);
	}
</script>

<div class="flex h-full flex-col bg-dark-bg">
	<!-- Header -->
	<div class="flex items-center justify-between border-b border-dark-border px-5 py-3">
		<h2 class="text-lg font-bold text-dark-text">设置</h2>
		<button
			class="cursor-pointer text-dark-muted hover:text-dark-text"
			onclick={onclose}
		>
			✕
		</button>
	</div>

	<!-- Settings form -->
	<div class="flex-1 overflow-y-auto p-5">
		<div class="flex flex-col gap-6">
			<!-- Auto-lock -->
			<section>
				<h3 class="mb-3 text-sm font-medium text-dark-text">自动锁定</h3>
				<div class="flex flex-col gap-3">
					<label class="flex items-center justify-between">
						<span class="text-sm text-dark-secondary">空闲自动锁定（分钟，0 = 关闭）</span>
						<input
							type="number"
							class="w-20 rounded-md border border-dark-border bg-dark-card px-3 py-1.5 text-sm text-dark-text outline-none focus:border-accent"
							bind:value={localSettings.autoLockMinutes}
							min="0"
							max="30"
						/>
					</label>
					<label class="flex items-center justify-between">
						<span class="text-sm text-dark-secondary">窗口失焦锁定（分钟，0 = 关闭）</span>
						<input
							type="number"
							class="w-20 rounded-md border border-dark-border bg-dark-card px-3 py-1.5 text-sm text-dark-text outline-none focus:border-accent"
							bind:value={localSettings.focusLockMinutes}
							min="0"
							max="10"
						/>
					</label>
				</div>
			</section>

			<!-- Clipboard -->
			<section>
				<h3 class="mb-3 text-sm font-medium text-dark-text">剪贴板</h3>
				<label class="flex items-center justify-between">
					<span class="text-sm text-dark-secondary">自动清除时间（秒）</span>
					<input
						type="number"
						class="w-20 rounded-md border border-dark-border bg-dark-card px-3 py-1.5 text-sm text-dark-text outline-none focus:border-accent"
						bind:value={localSettings.clipboardClearSeconds}
						min="10"
						max="120"
					/>
				</label>
			</section>

			<!-- Theme -->
			<section>
				<h3 class="mb-3 text-sm font-medium text-dark-text">外观</h3>
				<label class="flex items-center justify-between">
					<span class="text-sm text-dark-secondary">主题</span>
					<select
						class="rounded-md border border-dark-border bg-dark-card px-3 py-1.5 text-sm text-dark-text outline-none focus:border-accent"
						bind:value={localSettings.theme}
					>
						<option value="dark">暗色</option>
						<option value="light">亮色</option>
						<option value="system">跟随系统</option>
					</select>
				</label>
			</section>
		</div>
	</div>

	<!-- Footer -->
	<div class="flex items-center justify-end border-t border-dark-border px-5 py-3">
		<button
			class="cursor-pointer rounded-md bg-accent px-4 py-2 text-sm font-medium text-white hover:bg-accent-hover"
			onclick={save}
		>
			{saved ? '已保存' : '保存'}
		</button>
	</div>
</div>
