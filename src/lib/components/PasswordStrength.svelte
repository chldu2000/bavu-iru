<script lang="ts">
  import type { StrengthResult } from '$lib/utils/tauri';

  interface Props {
    result: StrengthResult | null;
  }

  let { result }: Props = $props();

  const colors = ['#cc4444', '#e87a2e', '#e8c72e', '#4ade80', '#22c55e'];
</script>

{#if result}
  <div class="flex items-center gap-2">
    <div class="flex gap-0.5">
      {#each Array(5) as _, i}
        <div
          class="h-1.5 w-5 rounded-sm transition-colors"
          style:background-color={i < result.score ? (colors[result.score - 1] ?? '#7a7568') : '#2a2822'}
        ></div>
      {/each}
    </div>
    <span class="text-xs" style:color={colors[result.score - 1] ?? '#7a7568'}>
      {result.label}
    </span>
  </div>
{/if}
