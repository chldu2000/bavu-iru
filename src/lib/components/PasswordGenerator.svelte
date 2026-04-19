<script lang="ts">
  import {
    generatePassword,
    evaluatePasswordStrength,
    type StrengthResult,
    type GeneratorOptions
  } from '$lib/utils/tauri';
  import PasswordStrength from './PasswordStrength.svelte';

  interface Props {
    onuse: (password: string) => void;
  }

  let { onuse }: Props = $props();

  let options = $state<GeneratorOptions>({
    length: 16,
    uppercase: true,
    lowercase: true,
    digits: true,
    special: true,
    exclude_chars: ''
  });
  let generated = $state('');
  let strength = $state<StrengthResult | null>(null);

  async function regenerate() {
    try {
      const result = await generatePassword(options);
      generated = result.password;
      strength = await evaluatePasswordStrength(generated);
    } catch {
      generated = '';
      strength = null;
    }
  }

  $effect(() => {
    regenerate();
  });
</script>

<div class="w-72 rounded-lg border border-line bg-card p-3 shadow-xl">
  <div class="mb-3 flex items-center justify-between">
    <span class="text-xs font-medium text-body">密码生成器</span>
  </div>

  <!-- Generated password preview -->
  <div class="mb-3 rounded-md border border-line bg-page px-2 py-1.5 font-mono text-xs break-all text-heading">
    {generated || '...'}
  </div>

  <!-- Strength bar -->
  <div class="mb-3">
    <PasswordStrength result={strength} />
  </div>

  <!-- Length slider -->
  <div class="mb-3">
    <div class="mb-1 flex items-center justify-between">
      <span class="text-xs text-hint">长度</span>
      <span class="text-xs text-heading">{options.length}</span>
    </div>
    <input
      type="range"
      min="8"
      max="64"
      bind:value={options.length}
      class="w-full accent-accent"
    />
  </div>

  <!-- Character toggles -->
  <div class="mb-3 grid grid-cols-2 gap-2">
    <label class="flex cursor-pointer items-center gap-1.5 text-xs text-body">
      <input type="checkbox" bind:checked={options.uppercase} class="accent-accent" />
      大写字母
    </label>
    <label class="flex cursor-pointer items-center gap-1.5 text-xs text-body">
      <input type="checkbox" bind:checked={options.lowercase} class="accent-accent" />
      小写字母
    </label>
    <label class="flex cursor-pointer items-center gap-1.5 text-xs text-body">
      <input type="checkbox" bind:checked={options.digits} class="accent-accent" />
      数字
    </label>
    <label class="flex cursor-pointer items-center gap-1.5 text-xs text-body">
      <input type="checkbox" bind:checked={options.special} class="accent-accent" />
      特殊字符
    </label>
  </div>

  <!-- Exclude chars -->
  <div class="mb-3">
    <span class="mb-1 block text-xs text-hint">排除字符</span>
    <input
      type="text"
      bind:value={options.exclude_chars}
      placeholder="例如: 0OlI1"
      class="w-full rounded-md border border-line bg-page px-2 py-1 text-xs text-heading outline-none placeholder:text-hint focus:border-accent"
    />
  </div>

  <!-- Action buttons -->
  <div class="flex gap-2">
    <button
      class="flex-1 cursor-pointer rounded-md border border-line bg-page px-3 py-1.5 text-xs text-body hover:text-heading"
      onclick={regenerate}
    >
      重新生成
    </button>
    <button
      class="flex-1 cursor-pointer rounded-md bg-accent px-3 py-1.5 text-xs font-medium text-white hover:bg-accent-hover"
      onclick={() => onuse(generated)}
    >
      使用
    </button>
  </div>
</div>
