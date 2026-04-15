<script lang="ts">
  import { tags } from '$lib/stores/tags';

  interface Props {
    selectedTagIds: string[];
    onselect: (tagIds: string[]) => void;
  }

  let { selectedTagIds, onselect }: Props = $props();
  let newTagName = $state('');
  let showNewTag = $state(false);

  async function handleCreate() {
    if (!newTagName.trim()) return;
    await tags.create(newTagName.trim());
    newTagName = '';
    showNewTag = false;
  }

  function toggleTag(tagId: string) {
    if (selectedTagIds.includes(tagId)) {
      onselect(selectedTagIds.filter((id) => id !== tagId));
    } else {
      onselect([...selectedTagIds, tagId]);
    }
  }
</script>

<div class="border-b border-dark-border py-2">
  <div class="flex items-center justify-between px-3 pb-1">
    <span class="text-xs font-medium uppercase tracking-wide text-dark-muted">标签</span>
    <button
      class="cursor-pointer text-xs text-dark-muted hover:text-accent"
      onclick={() => (showNewTag = !showNewTag)}
    >
      + 新建
    </button>
  </div>

  {#if showNewTag}
    <div class="px-3 pb-1">
      <div class="flex gap-1">
        <input
          type="text"
          bind:value={newTagName}
          placeholder="标签名称"
          class="flex-1 rounded border border-dark-border bg-dark-card px-2 py-1 text-xs text-dark-text outline-none placeholder:text-dark-muted focus:border-accent"
          onkeydown={(e) => e.key === 'Enter' && handleCreate()}
        />
        <button
          class="cursor-pointer rounded bg-accent px-2 py-1 text-xs text-white hover:bg-accent-hover"
          onclick={handleCreate}
        >
          确定
        </button>
      </div>
    </div>
  {/if}

  <div class="flex flex-wrap gap-1 px-3 py-1">
    {#each $tags as tag (tag.id)}
      <button
        type="button"
        class="cursor-pointer rounded-full px-2 py-0.5 text-xs transition-colors {selectedTagIds.includes(tag.id)
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
