<script lang="ts">
	import type { Entry } from '$lib/stores/entries';
	import PasswordField from './PasswordField.svelte';

	interface Props {
		entry?: Entry;
		onsave: (entry: Entry) => void;
		oncancel: () => void;
	}

	let { entry, onsave, oncancel }: Props = $props();

	let title = $state(entry?.title ?? '');
	let username = $state(entry?.username ?? '');
	let password = $state(entry?.password ?? '');
	let url = $state(entry?.url ?? '');
	let notes = $state(entry?.notes ?? '');

	function generatePassword() {
		const chars = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*';
		const bytes = crypto.getRandomValues(new Uint8Array(16));
		password = Array.from(bytes)
			.map((b) => chars[b % chars.length])
			.join('');
	}

	function handleSubmit() {
		if (!title.trim()) return;
		const now = new Date().toISOString();
		onsave({
			id: entry?.id ?? crypto.randomUUID(),
			folder_id: entry?.folder_id ?? null,
			title: title.trim(),
			username: username || null,
			password: password || null,
			url: url || null,
			notes: notes || null,
			custom_fields: entry?.custom_fields ?? null,
			tags: entry?.tags ?? null,
			strength: entry?.strength ?? null,
			expires_at: entry?.expires_at ?? null,
			created_at: entry?.created_at ?? now,
			updated_at: now
		});
	}
</script>

<div class="flex h-full flex-col p-5">
	<h2 class="mb-5 text-lg font-bold text-dark-text">
		{entry ? '编辑条目' : '新建条目'}
	</h2>

	<div class="flex flex-1 flex-col gap-3 overflow-y-auto">
		<div>
			<span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">标题 *</span>
			<input
				type="text"
				bind:value={title}
				placeholder="例如：GitHub"
				class="w-full rounded-md border border-dark-border bg-dark-card px-3 py-2 text-sm text-dark-text outline-none placeholder:text-dark-muted focus:border-accent"
			/>
		</div>

		<div>
			<span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">用户名</span>
			<input
				type="text"
				bind:value={username}
				placeholder="user@example.com"
				class="w-full rounded-md border border-dark-border bg-dark-card px-3 py-2 text-sm text-dark-text outline-none placeholder:text-dark-muted focus:border-accent"
			/>
		</div>

		<div>
			<span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">密码</span>
			<PasswordField
				value={password}
				editable={true}
				onchange={(v) => (password = v)}
				ongenerate={generatePassword}
			/>
		</div>

		<div>
			<span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">网址</span>
			<input
				type="text"
				bind:value={url}
				placeholder="https://"
				class="w-full rounded-md border border-dark-border bg-dark-card px-3 py-2 text-sm text-dark-text outline-none placeholder:text-dark-muted focus:border-accent"
			/>
		</div>

		<div>
			<span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">备注</span>
			<textarea
				bind:value={notes}
				placeholder="可选备注..."
				rows="3"
				class="w-full resize-y rounded-md border border-dark-border bg-dark-card px-3 py-2 text-sm text-dark-text outline-none placeholder:text-dark-muted focus:border-accent"
			></textarea>
		</div>
	</div>

	<!-- 操作按钮 -->
	<div class="flex justify-end gap-2 pt-4">
		<button
			class="cursor-pointer rounded-md px-4 py-2 text-sm text-dark-muted hover:text-dark-text"
			onclick={oncancel}
		>
			取消
		</button>
		<button
			class="cursor-pointer rounded-md bg-accent px-4 py-2 text-sm font-medium text-white hover:bg-accent-hover disabled:cursor-not-allowed disabled:opacity-50"
			onclick={handleSubmit}
			disabled={!title.trim()}
		>
			保存
		</button>
	</div>
</div>
