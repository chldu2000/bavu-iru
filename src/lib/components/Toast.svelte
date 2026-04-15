<script lang="ts">
	interface Props {
		message: string;
		visible: boolean;
	}

	let { message, visible }: Props = $props();
</script>

{#if visible}
	<div
		class="fixed bottom-4 right-4 z-50 rounded-lg border border-dark-border bg-dark-card px-4 py-2 text-sm text-dark-text shadow-lg"
	>
		{message}
	</div>
{/if}
