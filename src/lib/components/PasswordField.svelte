<script lang="ts">
	interface Props {
		value: string;
		editable?: boolean;
		onchange?: (value: string) => void;
		ongenerate?: () => void;
	}

	let { value, editable = false, onchange, ongenerate }: Props = $props();
	let visible = $state(false);
	let copied = $state(false);

	async function handleCopy() {
		await navigator.clipboard.writeText(value);
		copied = true;
		setTimeout(() => (copied = false), 2000);
	}
</script>

<div class="flex items-center gap-2">
	{#if editable}
		<input
			type={visible ? 'text' : 'password'}
			class="flex-1 rounded-md border border-dark-border bg-dark-card px-3 py-2 text-sm text-dark-text outline-none focus:border-accent"
			{value}
			oninput={(e) => onchange?.((e.target as HTMLInputElement).value)}
			placeholder="输入密码"
			autocomplete="off"
		/>
	{:else}
		<div
			class="flex flex-1 items-center justify-between rounded-md border border-dark-border bg-dark-card px-3 py-2 text-sm"
		>
			<span class="text-dark-text">
				{visible ? value : '••••••••••'}
			</span>
			<button
				class="ml-2 cursor-pointer text-dark-muted hover:text-dark-secondary"
				onclick={() => (visible = !visible)}
				aria-label={visible ? '隐藏密码' : '显示密码'}
			>
				{visible ? '隐藏' : '显示'}
			</button>
		</div>
	{/if}

	{#if editable}
		<button
			class="cursor-pointer rounded-md border border-dark-border bg-dark-card px-3 py-2 text-xs text-accent hover:bg-dark-border"
			onclick={() => (visible = !visible)}
		>
			{visible ? '隐藏' : '显示'}
		</button>
		{#if ongenerate}
			<button
				class="cursor-pointer rounded-md border border-dark-border bg-dark-card px-3 py-2 text-xs text-accent hover:bg-dark-border"
				onclick={ongenerate}
			>
				生成
			</button>
		{/if}
	{:else if value}
		<button
			class="cursor-pointer rounded-md px-3 py-2 text-xs text-accent hover:bg-dark-card"
			onclick={handleCopy}
		>
			{copied ? '已复制' : '复制'}
		</button>
	{/if}
</div>
