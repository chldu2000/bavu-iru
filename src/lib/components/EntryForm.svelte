<script lang="ts">
	import type { Entry } from '$lib/stores/entries';
	import { folders } from '$lib/stores/folders';
	import { tags } from '$lib/stores/tags';
	import PasswordField from './PasswordField.svelte';

	interface Props {
		entry?: Entry;
		onsave: (entry: Entry) => void;
		oncancel: () => void;
	}

	let { entry, onsave, oncancel }: Props = $props();

	// These initialize from the entry prop. Svelte warns (state_referenced_locally) but this
	// is intentional: form fields must be mutable via bind:value while keeping their initial
	// value from the prop. The entry prop does not change after mount in the current flow.
	let title = $state(entry?.title ?? '');
	let username = $state(entry?.username ?? '');
	let password = $state(entry?.password ?? '');
	let url = $state(entry?.url ?? '');
	let notes = $state(entry?.notes ?? '');
	let folderId = $state(entry?.folder_id ?? '');
	let selectedTagIds = $state<string[]>(parseTagIds(entry?.tags));

	function parseTagIds(tagsJson: string | null | undefined): string[] {
		if (!tagsJson) return [];
		try {
			const parsed = JSON.parse(tagsJson);
			return Array.isArray(parsed) ? parsed : [];
		} catch {
			return [];
		}
	}

	function toggleTag(tagId: string) {
		if (selectedTagIds.includes(tagId)) {
			selectedTagIds = selectedTagIds.filter((id) => id !== tagId);
		} else {
			selectedTagIds = [...selectedTagIds, tagId];
		}
	}

	function handleSubmit() {
		if (!title.trim()) return;
		const now = new Date().toISOString();
		onsave({
			id: entry?.id ?? crypto.randomUUID(),
			folder_id: folderId || null,
			title: title.trim(),
			username: username || null,
			password: password || null,
			url: url || null,
			notes: notes || null,
			custom_fields: entry?.custom_fields ?? null,
			tags: selectedTagIds.length > 0 ? JSON.stringify(selectedTagIds) : null,
			strength: entry?.strength ?? null,
			expires_at: entry?.expires_at ?? null,
			is_favorite: entry?.is_favorite ?? false,
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
			<span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">文件夹</span>
			<select
				bind:value={folderId}
				class="w-full rounded-md border border-dark-border bg-dark-card px-3 py-2 text-sm text-dark-text outline-none focus:border-accent"
			>
				<option value="">无文件夹</option>
				{#each $folders as folder (folder.id)}
					<option value={folder.id}>{folder.name}</option>
				{/each}
			</select>
		</div>

		<div>
			<span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">标签</span>
			<div class="flex flex-wrap gap-1 rounded-md border border-dark-border bg-dark-card p-2">
				{#each $tags as tag (tag.id)}
					<button
						type="button"
						class="cursor-pointer rounded-full px-2 py-0.5 text-xs transition-colors {selectedTagIds.includes(
							tag.id
						)
							? 'text-white ring-1 ring-white/20'
							: 'text-dark-secondary hover:text-dark-text'}"
						style:background-color={selectedTagIds.includes(tag.id) ? tag.color : 'transparent'}
						style:border="1px solid {tag.color}"
						onclick={() => toggleTag(tag.id)}
					>
						{tag.name}
					</button>
				{/each}
				{#if $tags.length === 0}
					<span class="text-xs text-dark-muted">暂无标签</span>
				{/if}
			</div>
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
