<script lang="ts">
  import PasswordGenerator from './PasswordGenerator.svelte';
  import { clipboardCopy } from '$lib/utils/tauri';
  import { settings } from '$lib/stores/settings';

  interface Props {
    value: string;
    editable?: boolean;
    onchange?: (value: string) => void;
    oncopy?: () => void;
    sensitive?: boolean;
  }

  let { value, editable = false, onchange, oncopy, sensitive = false }: Props = $props();
  let visible = $state(false);
  let copied = $state(false);
  let showGenerator = $state(false);

  async function handleCopy() {
    await clipboardCopy(value, sensitive, $settings.clipboardClearSeconds);
    copied = true;
    oncopy?.();
    setTimeout(() => (copied = false), 2000);
  }

  function handleUsePassword(password: string) {
    onchange?.(password);
    showGenerator = false;
  }
</script>

<div class="relative">
  <div class="flex items-center gap-2">
    {#if editable}
      <input
        type={visible ? 'text' : 'password'}
        class="flex-1 rounded-md border border-line bg-card px-3 py-2 text-sm text-heading outline-none focus:border-accent"
        {value}
        oninput={(e) => onchange?.((e.target as HTMLInputElement).value)}
        placeholder="输入密码"
        autocomplete="off"
      />
      <button
        class="cursor-pointer rounded-md border border-line bg-card px-3 py-2 text-xs text-accent hover:bg-line"
        onclick={() => (visible = !visible)}
      >
        {visible ? '隐藏' : '显示'}
      </button>
      <button
        class="cursor-pointer rounded-md border border-line bg-card px-3 py-2 text-xs text-accent hover:bg-line"
        onclick={() => (showGenerator = !showGenerator)}
      >
        生成
      </button>
    {:else}
      <div
        class="flex flex-1 items-center justify-between rounded-md border border-line bg-card px-3 py-2 text-sm"
      >
        <span class="text-heading">
          {visible ? value : '••••••••••'}
        </span>
        <button
          class="ml-2 cursor-pointer text-hint hover:text-body"
          onclick={() => (visible = !visible)}
          aria-label={visible ? '隐藏密码' : '显示密码'}
        >
          {visible ? '隐藏' : '显示'}
        </button>
      </div>
    {/if}

    {#if !editable && value}
      <button
        class="cursor-pointer rounded-md px-3 py-2 text-xs text-accent hover:bg-card"
        onclick={handleCopy}
      >
        {copied ? '已复制' : '复制'}
      </button>
    {/if}
  </div>

  {#if showGenerator}
    <div class="absolute left-0 top-full z-50 mt-1">
      <PasswordGenerator onuse={handleUsePassword} />
    </div>
  {/if}
</div>

{#if showGenerator}
  <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
  <div
    class="fixed inset-0 z-40"
    role="presentation"
    onclick={() => (showGenerator = false)}
    onkeydown={() => {}}
  ></div>
{/if}
