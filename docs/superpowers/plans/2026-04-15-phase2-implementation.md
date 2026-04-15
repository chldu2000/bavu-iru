# Phase 2: Password Generator + Search & Organization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement password generator, strength evaluation, search, folders/tags, favorites, and UI polish for the password manager.

**Architecture:** Rust backend provides Tauri IPC commands for password generation, strength evaluation, folder/tag CRUD, and favorites. Svelte 5 frontend uses popover-based generator, sidebar navigation for folders/tags, and reactive stores for state management.

**Tech Stack:** Tauri v2, Rust (rand 0.10, rusqlite), Svelte 5 (runes), Tailwind CSS 4

---

## Task 1: Password Generator — Rust Backend

**Files:**
- Create: `src-tauri/src/commands/generator.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create `commands/generator.rs` with generate_password command**

```rust
// src-tauri/src/commands/generator.rs
use serde::{Deserialize, Serialize};
use rand::Rng;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorOptions {
    pub length: usize,
    pub uppercase: bool,
    pub lowercase: bool,
    pub digits: bool,
    pub special: bool,
    pub exclude_chars: String,
}

impl Default for GeneratorOptions {
    fn default() -> Self {
        Self {
            length: 20,
            uppercase: true,
            lowercase: true,
            digits: true,
            special: true,
            exclude_chars: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedPassword {
    pub password: String,
}

const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SPECIAL: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

#[tauri::command]
pub fn generate_password(options: GeneratorOptions) -> Result<GeneratedPassword, String> {
    if options.length < 4 || options.length > 128 {
        return Err("Password length must be between 4 and 128".into());
    }

    let mut pool = String::new();
    let mut required: Vec<char> = Vec::new();

    if options.uppercase {
        pool.push_str(UPPERCASE);
        required.push(UPPERCASE.chars().next().unwrap());
    }
    if options.lowercase {
        pool.push_str(LOWERCASE);
        required.push(LOWERCASE.chars().next().unwrap());
    }
    if options.digits {
        pool.push_str(DIGITS);
        required.push(DIGITS.chars().next().unwrap());
    }
    if options.special {
        pool.push_str(SPECIAL);
        required.push(SPECIAL.chars().next().unwrap());
    }

    if pool.is_empty() {
        return Err("At least one character type must be selected".into());
    }

    // Remove excluded characters from pool
    let pool: String = pool
        .chars()
        .filter(|c| !options.exclude_chars.contains(*c))
        .collect();

    if pool.is_empty() {
        return Err("Character pool is empty after exclusions".into());
    }

    let pool_chars: Vec<char> = pool.chars().collect();
    let mut rng = rand::rng();

    // Start with required characters (one from each enabled type)
    let mut password_chars: Vec<char> = Vec::new();
    for req_char in &required {
        let type_chars: Vec<char> = if options.uppercase && UPPERCASE.contains(*req_char) {
            UPPERCASE.chars().filter(|c| !options.exclude_chars.contains(*c)).collect()
        } else if options.lowercase && LOWERCASE.contains(*req_char) {
            LOWERCASE.chars().filter(|c| !options.exclude_chars.contains(*c)).collect()
        } else if options.digits && DIGITS.contains(*req_char) {
            DIGITS.chars().filter(|c| !options.exclude_chars.contains(*c)).collect()
        } else if options.special && SPECIAL.contains(*req_char) {
            SPECIAL.chars().filter(|c| !options.exclude_chars.contains(*c)).collect()
        } else {
            continue;
        };
        if let Some(&c) = type_chars.choose(&mut rng) {
            password_chars.push(c);
        }
    }

    // Fill remaining length from full pool
    while password_chars.len() < options.length {
        let idx = rng.random_range(0..pool_chars.len());
        password_chars.push(pool_chars[idx]);
    }

    // Shuffle to randomize positions of required characters
    for i in (1..password_chars.len()).rev() {
        let j = rng.random_range(0..=i);
        password_chars.swap(i, j);
    }

    Ok(GeneratedPassword {
        password: password_chars.into_iter().collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options_generates_password() {
        let result = generate_password(GeneratorOptions::default()).unwrap();
        assert_eq!(result.password.len(), 20);
    }

    #[test]
    fn test_custom_length() {
        let opts = GeneratorOptions { length: 32, ..Default::default() };
        let result = generate_password(opts).unwrap();
        assert_eq!(result.password.len(), 32);
    }

    #[test]
    fn test_only_digits() {
        let opts = GeneratorOptions {
            length: 10,
            uppercase: false,
            lowercase: false,
            digits: true,
            special: false,
            exclude_chars: String::new(),
        };
        let result = generate_password(opts).unwrap();
        assert!(result.password.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_exclude_chars() {
        let opts = GeneratorOptions {
            length: 50,
            exclude_chars: "aeiouAEIOU0".into(),
            ..Default::default()
        };
        let result = generate_password(opts).unwrap();
        assert!(!result.password.contains('a'));
        assert!(!result.password.contains('E'));
        assert!(!result.password.contains('0'));
    }

    #[test]
    fn test_no_char_types_fails() {
        let opts = GeneratorOptions {
            uppercase: false,
            lowercase: false,
            digits: false,
            special: false,
            ..Default::default()
        };
        assert!(generate_password(opts).is_err());
    }

    #[test]
    fn test_invalid_length_fails() {
        let opts = GeneratorOptions { length: 2, ..Default::default() };
        assert!(generate_password(opts).is_err());
    }

    #[test]
    fn test_all_enabled_types_present() {
        let opts = GeneratorOptions { length: 50, ..Default::default() };
        let result = generate_password(opts).unwrap();
        let has_upper = result.password.chars().any(|c| c.is_ascii_uppercase());
        let has_lower = result.password.chars().any(|c| c.is_ascii_lowercase());
        let has_digit = result.password.chars().any(|c| c.is_ascii_digit());
        let has_special = result.password.chars().any(|c| SPECIAL.contains(c));
        assert!(has_upper);
        assert!(has_lower);
        assert!(has_digit);
        assert!(has_special);
    }
}
```

- [ ] **Step 2: Register generator module**

In `src-tauri/src/commands/mod.rs`, add `pub mod generator;` to the module list. The file becomes:

```rust
pub mod vault;
pub mod entries;
pub mod clipboard;
pub mod import_export;
pub mod generator;
```

- [ ] **Step 3: Register generate_password command in lib.rs**

In `src-tauri/src/lib.rs`, add `commands::generator::generate_password` to the `invoke_handler` array:

```rust
.invoke_handler(tauri::generate_handler![
    commands::vault::vault_setup,
    commands::vault::vault_unlock,
    commands::vault::vault_lock,
    commands::vault::vault_status,
    commands::entries::entry_list,
    commands::entries::entry_get,
    commands::entries::entry_create,
    commands::entries::entry_update,
    commands::entries::entry_delete,
    commands::generator::generate_password,
])
```

Note: Add `use rand::seq::SliceRandom;` is NOT needed — `SliceRandom` is imported via `rand::Rng` trait which provides `random_range`, and `choose` method is on slices directly.

Wait — actually, `.choose(&mut rng)` comes from `rand::seq::IteratorRandom` or `SliceRandom`. Need to add that import. In `generator.rs` add:

```rust
use rand::seq::SliceRandom;
```

And change the `use rand::Rng;` line to include both:

```rust
use rand::{Rng, seq::SliceRandom};
```

Actually for rand 0.10, the `choose` method on slices is provided by `SliceRandom` trait. Let me use a simpler approach: just index with `random_range`:

Replace the `.choose()` call:
```rust
if let Some(&c) = type_chars.choose(&mut rng) {
```
with:
```rust
if !type_chars.is_empty() {
    let c = type_chars[rng.random_range(0..type_chars.len())];
```

- [ ] **Step 4: Run tests**

Run: `cd src-tauri && cargo test commands::generator`
Expected: All 7 tests PASS

- [ ] **Step 5: Run full build**

Run: `cd src-tauri && cargo build`
Expected: Compiles with 0 errors

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands/generator.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add password generator with configurable options"
```

---

## Task 2: Password Strength — Rust Backend

**Files:**
- Create: `src-tauri/src/crypto/strength.rs`
- Modify: `src-tauri/src/crypto/mod.rs`
- Create: `src-tauri/src/commands/strength.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create `crypto/strength.rs` with evaluation algorithm**

```rust
// src-tauri/src/crypto/strength.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrengthResult {
    pub score: u8,
    pub label: String,
    pub feedback: String,
}

/// Top common passwords (abbreviated list for matching)
const COMMON_PASSWORDS: &[&str] = &[
    "password", "123456", "12345678", "qwerty", "abc123", "monkey",
    "master", "dragon", "login", "princess", "football", "shadow",
    "sunshine", "trustno1", "iloveyou", "batman", "access", "hello",
    "charlie", "donald", "123456789", "password1", "qwerty123",
    "letmein", "welcome", "admin", "passw0rd", "1234567", "12345",
    "1234", "123", "111111", "123123", "000000", "121212",
    "qwerty1", "password123", "1q2w3e4r", "666666", "555555",
    "654321", "superman", "michael", "jordan", "harley", "ranger",
    "thomas", "robert", "soccer", "hockey", "killer", "george",
];

/// Evaluate password strength. Returns score 0-4.
pub fn evaluate_strength(password: &str) -> StrengthResult {
    if password.is_empty() {
        return StrengthResult {
            score: 0,
            label: "空".into(),
            feedback: "请输入密码".into(),
        };
    }

    let lower = password.to_lowercase();

    // Check against common passwords — instant fail
    if COMMON_PASSWORDS.contains(&lower.as_str()) {
        return StrengthResult {
            score: 0,
            label: "很弱".into(),
            feedback: "这是常见密码，极易被破解".into(),
        };
    }

    let len = password.len();
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_ascii_alphanumeric());

    let char_types = [has_upper, has_lower, has_digit, has_special]
        .iter()
        .filter(|&&x| x)
        .count();

    // Calculate score components
    let mut score: i32 = 0;

    // Length scoring
    if len >= 8 { score += 1; }
    if len >= 12 { score += 1; }
    if len >= 16 { score += 1; }
    if len >= 20 { score += 1; }

    // Character diversity scoring
    score += char_types as i32;

    // Penalize patterns
    if has_sequential_chars(password, 3) {
        score -= 1;
    }
    if has_repeated_chars(password, 3) {
        score -= 1;
    }

    // Clamp to 0-4
    let score = score.clamp(0, 4) as u8;

    let (label, feedback) = match score {
        0 => ("很弱".into(), "密码太短或过于简单".into()),
        1 => ("弱".into(), "建议增加长度和字符类型".into()),
        2 => ("一般".into(), "可以使用，建议增加特殊字符".into()),
        3 => ("强".into(), "密码强度良好".into()),
        4 => ("很强".into(), "密码强度优秀".into()),
        _ => unreachable!(),
    };

    StrengthResult { score, label, feedback }
}

/// Detect sequential characters like "abc", "123", "cba", "321"
fn has_sequential_chars(password: &str, min_len: usize) -> bool {
    let chars: Vec<char> = password.chars().collect();
    if chars.len() < min_len {
        return false;
    }

    let mut ascending = 1;
    let mut descending = 1;

    for i in 1..chars.len() {
        if let (Some(prev), Some(curr)) = (chars[i - 1].as_ascii(), chars[i].as_ascii()) {
            let diff = curr.to_char() as i32 - prev.to_char() as i32;
            if diff == 1 {
                ascending += 1;
                descending = 1;
            } else if diff == -1 {
                descending += 1;
                ascending = 1;
            } else {
                ascending = 1;
                descending = 1;
            }

            if ascending >= min_len || descending >= min_len {
                return true;
            }
        }
    }
    false
}

/// Detect repeated characters like "aaa", "111"
fn has_repeated_chars(password: &str, min_repeat: usize) -> bool {
    let chars: Vec<char> = password.chars().collect();
    if chars.len() < min_repeat {
        return false;
    }

    let mut count = 1;
    for i in 1..chars.len() {
        if chars[i] == chars[i - 1] {
            count += 1;
            if count >= min_repeat {
                return true;
            }
        } else {
            count = 1;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_password() {
        let result = evaluate_strength("");
        assert_eq!(result.score, 0);
    }

    #[test]
    fn test_common_password() {
        let result = evaluate_strength("password");
        assert_eq!(result.score, 0);
        assert!(result.feedback.contains("常见"));
    }

    #[test]
    fn test_short_password() {
        let result = evaluate_strength("ab12");
        assert!(result.score <= 1);
    }

    #[test]
    fn test_strong_password() {
        let result = evaluate_strength("X#9kLm$2pQwR!nB7vF");
        assert!(result.score >= 3);
    }

    #[test]
    fn test_medium_password() {
        let result = evaluate_strength("hello123");
        assert!(result.score >= 1 && result.score <= 3);
    }

    #[test]
    fn test_sequential_detection() {
        assert!(has_sequential_chars("abc123", 3));
        assert!(has_sequential_chars("321cba", 3));
        assert!(!has_sequential_chars("a1b2c3", 3));
    }

    #[test]
    fn test_repeated_detection() {
        assert!(has_repeated_chars("aaabbb", 3));
        assert!(!has_repeated_chars("ababab", 3));
    }
}
```

- [ ] **Step 2: Register strength module in `crypto/mod.rs`**

```rust
pub mod kdf;
pub mod cipher;
pub mod keyring;
pub mod strength;
```

- [ ] **Step 3: Create `commands/strength.rs`**

```rust
// src-tauri/src/commands/strength.rs
use crate::crypto::strength::{self, StrengthResult};

#[tauri::command]
pub fn evaluate_password_strength(password: String) -> Result<StrengthResult, String> {
    Ok(strength::evaluate_strength(&password))
}
```

- [ ] **Step 4: Register strength command module**

In `src-tauri/src/commands/mod.rs` add `pub mod strength;`:

```rust
pub mod vault;
pub mod entries;
pub mod clipboard;
pub mod import_export;
pub mod generator;
pub mod strength;
```

In `src-tauri/src/lib.rs` add to `invoke_handler`:

```rust
commands::strength::evaluate_password_strength,
```

- [ ] **Step 5: Run tests**

Run: `cd src-tauri && cargo test crypto::strength`
Expected: All 8 tests PASS

- [ ] **Step 6: Run full build**

Run: `cd src-tauri && cargo build`
Expected: Compiles with 0 errors

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/crypto/strength.rs src-tauri/src/crypto/mod.rs src-tauri/src/commands/strength.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add password strength evaluation module"
```

---

## Task 3: Password Generator — Frontend (Popover + Strength Bar)

**Files:**
- Create: `src/lib/components/PasswordGenerator.svelte`
- Create: `src/lib/components/PasswordStrength.svelte`
- Modify: `src/lib/utils/tauri.ts`
- Modify: `src/lib/components/PasswordField.svelte`

- [ ] **Step 1: Add IPC wrappers in `tauri.ts`**

Append to the end of `src/lib/utils/tauri.ts`:

```typescript
export interface GeneratorOptions {
  length: number;
  uppercase: boolean;
  lowercase: boolean;
  digits: boolean;
  special: boolean;
  exclude_chars: string;
}

export interface GeneratedPassword {
  password: string;
}

export interface StrengthResult {
  score: number;
  label: string;
  feedback: string;
}

export async function generatePassword(options: GeneratorOptions): Promise<GeneratedPassword> {
  return invoke('generate_password', { options });
}

export async function evaluatePasswordStrength(password: string): Promise<StrengthResult> {
  return invoke('evaluate_password_strength', { password });
}
```

- [ ] **Step 2: Create `PasswordStrength.svelte`**

```svelte
<!-- src/lib/components/PasswordStrength.svelte -->
<script lang="ts">
  import type { StrengthResult } from '$lib/utils/tauri';

  interface Props {
    result: StrengthResult | null;
  }

  let { result }: Props = $props();

  const colors = ['#cc4444', '#e87a2e', '#e8c72e, '#4ade80', '#22c55e'];
  const segments = 5;
</script>

{#if result}
  <div class="flex items-center gap-2">
    <div class="flex gap-0.5">
      {#each Array(segments) as _, i}
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
```

- [ ] **Step 3: Create `PasswordGenerator.svelte`**

```svelte
<!-- src/lib/components/PasswordGenerator.svelte -->
<script lang="ts">
  import { generatePassword, evaluatePasswordStrength } from '$lib/utils/tauri';
  import type { StrengthResult } from '$lib/utils/tauri';
  import PasswordStrength from './PasswordStrength.svelte';

  interface Props {
    onuse: (password: string) => void;
  }

  let { onuse }: Props = $props();

  let length = $state(20);
  let uppercase = $state(true);
  let lowercase = $state(true);
  let digits = $state(true);
  let special = $state(true);
  let excludeChars = $state('');
  let generated = $state('');
  let strengthResult = $state<StrengthResult | null>(null);
  let loading = $state(false);

  async function generate() {
    loading = true;
    try {
      const result = await generatePassword({
        length,
        uppercase,
        lowercase,
        digits,
        special,
        exclude_chars: excludeChars
      });
      generated = result.password;
      await evaluateStrength();
    } catch (e) {
      console.error('生成失败:', e);
    } finally {
      loading = false;
    }
  }

  async function evaluateStrength() {
    if (!generated) {
      strengthResult = null;
      return;
    }
    try {
      strengthResult = await evaluatePasswordStrength(generated);
    } catch {
      strengthResult = null;
    }
  }

  function handleUse() {
    if (generated) {
      onuse(generated);
    }
  }

  // Auto-generate on mount
  $effect(() => {
    generate();
  });
</script>

<div class="w-72 rounded-lg border border-dark-border bg-dark-card p-3 shadow-xl">
  <!-- Generated password preview -->
  {#if generated}
    <div class="mb-3 break-all rounded bg-dark-bg px-2 py-1.5 font-mono text-sm text-dark-text">
      {generated}
    </div>
    <PasswordStrength result={strengthResult} />
  {/if}

  <!-- Controls -->
  <div class="mt-3 space-y-2">
    <!-- Length slider -->
    <div class="flex items-center justify-between">
      <span class="text-xs text-dark-muted">长度</span>
      <span class="text-xs text-dark-text">{length}</span>
    </div>
    <input
      type="range"
      min="8"
      max="64"
      bind:value={length}
      class="w-full accent-accent"
    />

    <!-- Character type toggles -->
    <div class="flex flex-wrap gap-2">
      <label class="flex cursor-pointer items-center gap-1 text-xs text-dark-secondary">
        <input type="checkbox" bind:checked={uppercase} class="accent-accent" />
        A-Z
      </label>
      <label class="flex cursor-pointer items-center gap-1 text-xs text-dark-secondary">
        <input type="checkbox" bind:checked={lowercase} class="accent-accent" />
        a-z
      </label>
      <label class="flex cursor-pointer items-center gap-1 text-xs text-dark-secondary">
        <input type="checkbox" bind:checked={digits} class="accent-accent" />
        0-9
      </label>
      <label class="flex cursor-pointer items-center gap-1 text-xs text-dark-secondary">
        <input type="checkbox" bind:checked={special} class="accent-accent" />
        !@#
      </label>
    </div>

    <!-- Exclude characters -->
    <div>
      <span class="mb-0.5 block text-xs text-dark-muted">排除字符</span>
      <input
        type="text"
        bind:value={excludeChars}
        placeholder="例如: 0OlI"
        class="w-full rounded border border-dark-border bg-dark-bg px-2 py-1 text-xs text-dark-text outline-none placeholder:text-dark-muted focus:border-accent"
      />
    </div>
  </div>

  <!-- Action buttons -->
  <div class="mt-3 flex gap-2">
    <button
      class="flex-1 cursor-pointer rounded bg-dark-border px-3 py-1.5 text-xs text-dark-text hover:bg-dark-muted/30"
      onclick={generate}
      disabled={loading}
    >
      重新生成
    </button>
    <button
      class="flex-1 cursor-pointer rounded bg-accent px-3 py-1.5 text-xs font-medium text-white hover:bg-accent-hover"
      onclick={handleUse}
      disabled={!generated}
    >
      使用
    </button>
  </div>
</div>
```

- [ ] **Step 4: Update `PasswordField.svelte` to integrate generator popover**

Replace the entire content of `src/lib/components/PasswordField.svelte`:

```svelte
<!-- src/lib/components/PasswordField.svelte -->
<script lang="ts">
  import PasswordGenerator from './PasswordGenerator.svelte';

  interface Props {
    value: string;
    editable?: boolean;
    onchange?: (value: string) => void;
  }

  let { value, editable = false, onchange }: Props = $props();
  let visible = $state(false);
  let copied = $state(false);
  let showGenerator = $state(false);

  async function handleCopy() {
    await navigator.clipboard.writeText(value);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  function useGenerated(password: string) {
    onchange?.(password);
    showGenerator = false;
  }
</script>

<div class="relative">
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
      <button
        class="cursor-pointer rounded-md border border-dark-border bg-dark-card px-3 py-2 text-xs text-dark-muted hover:text-dark-secondary"
        onclick={() => (visible = !visible)}
      >
        {visible ? '隐藏' : '显示'}
      </button>
      <button
        class="cursor-pointer rounded-md border border-dark-border bg-dark-card px-3 py-2 text-xs text-accent hover:bg-dark-border"
        onclick={() => (showGenerator = !showGenerator)}
        title="密码生成器"
      >
        生成
      </button>
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

    {#if !editable && value}
      <button
        class="cursor-pointer rounded-md px-3 py-2 text-xs text-accent hover:bg-dark-card"
        onclick={handleCopy}
      >
        {copied ? '已复制' : '复制'}
      </button>
    {/if}
  </div>

  <!-- Generator popover -->
  {#if showGenerator && editable}
    <div class="absolute left-0 top-full z-50 mt-1">
      <PasswordGenerator onuse={useGenerated} />
    </div>
  {/if}
</div>
```

- [ ] **Step 5: Update `EntryForm.svelte` — remove old generatePassword function**

Remove the `generatePassword` function (lines 19-25) and the `ongenerate` prop from PasswordField usage. The `PasswordField` component no longer takes `ongenerate` — it has the generator built in.

In `src/lib/components/EntryForm.svelte`, delete the `generatePassword` function and update the PasswordField usage to remove the `ongenerate` prop:

Change:
```svelte
<PasswordField
    value={password}
    editable={true}
    onchange={(v) => (password = v)}
    ongenerate={generatePassword}
/>
```

To:
```svelte
<PasswordField
    value={password}
    editable={true}
    onchange={(v) => (password = v)}
/>
```

- [ ] **Step 6: Run svelte-check**

Run: `npx svelte-check --threshold error`
Expected: 0 errors

- [ ] **Step 7: Commit**

```bash
git add src/lib/utils/tauri.ts src/lib/components/PasswordStrength.svelte src/lib/components/PasswordGenerator.svelte src/lib/components/PasswordField.svelte src/lib/components/EntryForm.svelte
git commit -m "feat: add password generator popover and strength indicator UI"
```

---

## Task 4: Database Schema — Tags + Favorites

**Files:**
- Modify: `src-tauri/src/db/repository.rs`
- Modify: `src-tauri/src/db/models.rs`

- [ ] **Step 1: Add Tag model and update Entry model in `models.rs`**

Replace the entire content of `src-tauri/src/db/models.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub folder_id: Option<String>,
    pub title: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub custom_fields: Option<String>,
    pub tags: Option<String>,
    pub strength: Option<i32>,
    pub expires_at: Option<String>,
    pub is_favorite: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Metadata key-value pairs for vault configuration (KDF params, salt, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub key: String,
    pub value: String,
}
```

- [ ] **Step 2: Update `repository.rs` — add tags table, entry_tags table, is_favorite column, and update all entry queries**

Replace the entire content of `src-tauri/src/db/repository.rs`:

```rust
use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;
use std::sync::Mutex;

use super::models::{Entry, Folder, Tag};

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Open (or create) the SQLite database at the given path.
    pub fn open(path: &Path) -> SqliteResult<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    /// Create an in-memory database (useful for testing).
    pub fn open_in_memory() -> SqliteResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS meta (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS folders (
                id         TEXT PRIMARY KEY,
                name       TEXT NOT NULL,
                parent_id  TEXT,
                sort_order INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (parent_id) REFERENCES folders(id)
            );

            CREATE TABLE IF NOT EXISTS tags (
                id         TEXT PRIMARY KEY,
                name       TEXT NOT NULL,
                color      TEXT DEFAULT '#6366f1',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS entry_tags (
                entry_id TEXT NOT NULL,
                tag_id   TEXT NOT NULL,
                PRIMARY KEY (entry_id, tag_id),
                FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS entries (
                id            TEXT PRIMARY KEY,
                folder_id     TEXT,
                title         TEXT NOT NULL,
                username      TEXT,
                password      TEXT,
                url           TEXT,
                notes         TEXT,
                custom_fields TEXT,
                tags          TEXT,
                strength      INTEGER,
                expires_at    TEXT,
                is_favorite   INTEGER DEFAULT 0,
                created_at    TEXT NOT NULL,
                updated_at    TEXT NOT NULL,
                FOREIGN KEY (folder_id) REFERENCES folders(id)
            );
            "
        )?;
        Ok(())
    }

    // --- Meta operations ---

    pub fn set_meta(&self, key: &str, value: &str) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![key, value],
        )?;
        Ok(())
    }

    pub fn get_meta(&self, key: &str) -> SqliteResult<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM meta WHERE key = ?1")?;
        let mut rows = stmt.query(rusqlite::params![key])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    // --- Entry operations ---

    pub fn create_entry(&self, entry: &Entry) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        let is_fav = entry.is_favorite as i32;
        conn.execute(
            "INSERT INTO entries (id, folder_id, title, username, password, url, notes, custom_fields, tags, strength, expires_at, is_favorite, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            rusqlite::params![
                entry.id, entry.folder_id, entry.title, entry.username,
                entry.password, entry.url, entry.notes, entry.custom_fields,
                entry.tags, entry.strength, entry.expires_at, is_fav,
                entry.created_at, entry.updated_at
            ],
        )?;
        Ok(())
    }

    pub fn list_entries(&self) -> SqliteResult<Vec<Entry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, folder_id, title, username, password, url, notes, custom_fields, tags, strength, expires_at, is_favorite, created_at, updated_at
             FROM entries ORDER BY is_favorite DESC, updated_at DESC"
        )?;
        let entries = stmt.query_map([], |row| {
            Ok(Entry {
                id: row.get(0)?,
                folder_id: row.get(1)?,
                title: row.get(2)?,
                username: row.get(3)?,
                password: row.get(4)?,
                url: row.get(5)?,
                notes: row.get(6)?,
                custom_fields: row.get(7)?,
                tags: row.get(8)?,
                strength: row.get(9)?,
                expires_at: row.get(10)?,
                is_favorite: row.get::<_, i32>(11)? != 0,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })?;
        entries.collect()
    }

    pub fn get_entry(&self, id: &str) -> SqliteResult<Option<Entry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, folder_id, title, username, password, url, notes, custom_fields, tags, strength, expires_at, is_favorite, created_at, updated_at
             FROM entries WHERE id = ?1"
        )?;
        let mut rows = stmt.query(rusqlite::params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(Entry {
                id: row.get(0)?,
                folder_id: row.get(1)?,
                title: row.get(2)?,
                username: row.get(3)?,
                password: row.get(4)?,
                url: row.get(5)?,
                notes: row.get(6)?,
                custom_fields: row.get(7)?,
                tags: row.get(8)?,
                strength: row.get(9)?,
                expires_at: row.get(10)?,
                is_favorite: row.get::<_, i32>(11)? != 0,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })),
            None => Ok(None),
        }
    }

    pub fn delete_entry(&self, id: &str) -> SqliteResult<bool> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute("DELETE FROM entries WHERE id = ?1", rusqlite::params![id])?;
        Ok(affected > 0)
    }

    pub fn update_entry(&self, entry: &Entry) -> SqliteResult<bool> {
        let conn = self.conn.lock().unwrap();
        let is_fav = entry.is_favorite as i32;
        let affected = conn.execute(
            "UPDATE entries SET folder_id=?2, title=?3, username=?4, password=?5, url=?6, notes=?7, custom_fields=?8, tags=?9, strength=?10, expires_at=?11, is_favorite=?12, updated_at=?13
             WHERE id=?1",
            rusqlite::params![
                entry.id, entry.folder_id, entry.title, entry.username,
                entry.password, entry.url, entry.notes, entry.custom_fields,
                entry.tags, entry.strength, entry.expires_at, is_fav, entry.updated_at
            ],
        )?;
        Ok(affected > 0)
    }

    pub fn toggle_favorite(&self, id: &str) -> SqliteResult<bool> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute(
            "UPDATE entries SET is_favorite = CASE WHEN is_favorite = 0 THEN 1 ELSE 0 END, updated_at = datetime('now') WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(affected > 0)
    }

    // --- Folder operations ---

    pub fn create_folder(&self, folder: &Folder) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO folders (id, name, parent_id, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                folder.id, folder.name, folder.parent_id,
                folder.sort_order, folder.created_at, folder.updated_at
            ],
        )?;
        Ok(())
    }

    pub fn list_folders(&self) -> SqliteResult<Vec<Folder>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, parent_id, sort_order, created_at, updated_at
             FROM folders ORDER BY sort_order, name"
        )?;
        let folders = stmt.query_map([], |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                sort_order: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        folders.collect()
    }

    pub fn rename_folder(&self, id: &str, name: &str) -> SqliteResult<bool> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute(
            "UPDATE folders SET name = ?2, updated_at = datetime('now') WHERE id = ?1",
            rusqlite::params![id, name],
        )?;
        Ok(affected > 0)
    }

    pub fn delete_folder(&self, id: &str) -> SqliteResult<bool> {
        let conn = self.conn.lock().unwrap();
        // Nullify folder_id on entries in this folder
        conn.execute(
            "UPDATE entries SET folder_id = NULL WHERE folder_id = ?1",
            rusqlite::params![id],
        )?;
        let affected = conn.execute("DELETE FROM folders WHERE id = ?1", rusqlite::params![id])?;
        Ok(affected > 0)
    }

    // --- Tag operations ---

    pub fn create_tag(&self, tag: &Tag) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO tags (id, name, color, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                tag.id, tag.name, tag.color, tag.created_at, tag.updated_at
            ],
        )?;
        Ok(())
    }

    pub fn list_tags(&self) -> SqliteResult<Vec<Tag>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, color, created_at, updated_at FROM tags ORDER BY name"
        )?;
        let tags = stmt.query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;
        tags.collect()
    }

    pub fn update_tag(&self, id: &str, name: &str, color: &str) -> SqliteResult<bool> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute(
            "UPDATE tags SET name = ?2, color = ?3, updated_at = datetime('now') WHERE id = ?1",
            rusqlite::params![id, name, color],
        )?;
        Ok(affected > 0)
    }

    pub fn delete_tag(&self, id: &str) -> SqliteResult<bool> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute("DELETE FROM tags WHERE id = ?1", rusqlite::params![id])?;
        Ok(affected > 0)
    }

    // --- Entry-Tag operations ---

    pub fn add_tag_to_entry(&self, entry_id: &str, tag_id: &str) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO entry_tags (entry_id, tag_id) VALUES (?1, ?2)",
            rusqlite::params![entry_id, tag_id],
        )?;
        Ok(())
    }

    pub fn remove_tag_from_entry(&self, entry_id: &str, tag_id: &str) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM entry_tags WHERE entry_id = ?1 AND tag_id = ?2",
            rusqlite::params![entry_id, tag_id],
        )?;
        Ok(())
    }

    pub fn get_entry_tags(&self, entry_id: &str) -> SqliteResult<Vec<Tag>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.color, t.created_at, t.updated_at
             FROM tags t JOIN entry_tags et ON t.id = et.tag_id
             WHERE et.entry_id = ?1 ORDER BY t.name"
        )?;
        let tags = stmt.query_map(rusqlite::params![entry_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;
        tags.collect()
    }
}
```

- [ ] **Step 3: Update `commands/entries.rs` — add toggle_favorite and update entry_create/entry_update to handle is_favorite**

Replace the entire content of `src-tauri/src/commands/entries.rs`:

```rust
use tauri::State;

use crate::crypto::keyring::Keyring;
use crate::db::models::Entry;
use crate::db::repository::Database;
use crate::error::AppError;

#[tauri::command]
pub fn entry_list(db: State<'_, Database>) -> Result<Vec<Entry>, AppError> {
    Ok(db.list_entries()?)
}

#[tauri::command]
pub fn entry_get(id: String, db: State<'_, Database>) -> Result<Option<Entry>, AppError> {
    Ok(db.get_entry(&id)?)
}

#[tauri::command]
pub fn entry_create(entry: Entry, db: State<'_, Database>, keyring: State<'_, Keyring>) -> Result<(), AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }
    db.create_entry(&entry)?;
    Ok(())
}

#[tauri::command]
pub fn entry_update(entry: Entry, db: State<'_, Database>, keyring: State<'_, Keyring>) -> Result<bool, AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }
    Ok(db.update_entry(&entry)?)
}

#[tauri::command]
pub fn entry_delete(id: String, db: State<'_, Database>, keyring: State<'_, Keyring>) -> Result<bool, AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }
    Ok(db.delete_entry(&id)?)
}

#[tauri::command]
pub fn toggle_favorite(id: String, db: State<'_, Database>, keyring: State<'_, Keyring>) -> Result<bool, AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }
    Ok(db.toggle_favorite(&id)?)
}
```

- [ ] **Step 4: Run full build**

Run: `cd src-tauri && cargo build`
Expected: Compiles with 0 errors

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/db/models.rs src-tauri/src/db/repository.rs src-tauri/src/commands/entries.rs
git commit -m "feat: add tags/favorites schema and repository operations"
```

---

## Task 5: Folder + Tag + Favorite — Rust Commands

**Files:**
- Create: `src-tauri/src/commands/folders.rs`
- Create: `src-tauri/src/commands/tags.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create `commands/folders.rs`**

```rust
// src-tauri/src/commands/folders.rs
use chrono::Utc;
use tauri::State;
use uuid::Uuid;

use crate::crypto::keyring::Keyring;
use crate::db::models::Folder;
use crate::db::repository::Database;
use crate::error::AppError;

#[tauri::command]
pub fn folder_create(
    name: String,
    parent_id: Option<String>,
    db: State<'_, Database>,
    keyring: State<'_, Keyring>,
) -> Result<Folder, AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }
    let now = Utc::now().to_rfc3339();
    let folder = Folder {
        id: Uuid::new_v4().to_string(),
        name,
        parent_id,
        sort_order: 0,
        created_at: now.clone(),
        updated_at: now,
    };
    db.create_folder(&folder)?;
    Ok(folder)
}

#[tauri::command]
pub fn folder_rename(id: String, name: String, db: State<'_, Database>, keyring: State<'_, Keyring>) -> Result<bool, AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }
    Ok(db.rename_folder(&id, &name)?)
}

#[tauri::command]
pub fn folder_delete(id: String, db: State<'_, Database>, keyring: State<'_, Keyring>) -> Result<bool, AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }
    Ok(db.delete_folder(&id)?)
}

#[tauri::command]
pub fn folder_list(db: State<'_, Database>) -> Result<Vec<Folder>, AppError> {
    Ok(db.list_folders()?)
}
```

- [ ] **Step 2: Create `commands/tags.rs`**

```rust
// src-tauri/src/commands/tags.rs
use chrono::Utc;
use tauri::State;
use uuid::Uuid;

use crate::crypto::keyring::Keyring;
use crate::db::models::Tag;
use crate::db::repository::Database;
use crate::error::AppError;

#[tauri::command]
pub fn tag_create(
    name: String,
    color: Option<String>,
    db: State<'_, Database>,
    keyring: State<'_, Keyring>,
) -> Result<Tag, AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }
    let now = Utc::now().to_rfc3339();
    let tag = Tag {
        id: Uuid::new_v4().to_string(),
        name,
        color: color.unwrap_or_else(|| "#6366f1".into()),
        created_at: now.clone(),
        updated_at: now,
    };
    db.create_tag(&tag)?;
    Ok(tag)
}

#[tauri::command]
pub fn tag_update(
    id: String,
    name: String,
    color: String,
    db: State<'_, Database>,
    keyring: State<'_, Keyring>,
) -> Result<bool, AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }
    Ok(db.update_tag(&id, &name, &color)?)
}

#[tauri::command]
pub fn tag_delete(id: String, db: State<'_, Database>, keyring: State<'_, Keyring>) -> Result<bool, AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }
    Ok(db.delete_tag(&id)?)
}

#[tauri::command]
pub fn tag_list(db: State<'_, Database>) -> Result<Vec<Tag>, AppError> {
    Ok(db.list_tags()?)
}

#[tauri::command]
pub fn tag_add_to_entry(
    entry_id: String,
    tag_id: String,
    db: State<'_, Database>,
    keyring: State<'_, Keyring>,
) -> Result<(), AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }
    db.add_tag_to_entry(&entry_id, &tag_id)?;
    Ok(())
}

#[tauri::command]
pub fn tag_remove_from_entry(
    entry_id: String,
    tag_id: String,
    db: State<'_, Database>,
    keyring: State<'_, Keyring>,
) -> Result<(), AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }
    db.remove_tag_from_entry(&entry_id, &tag_id)?;
    Ok(())
}
```

- [ ] **Step 3: Register new modules and commands**

Update `src-tauri/src/commands/mod.rs`:

```rust
pub mod vault;
pub mod entries;
pub mod clipboard;
pub mod import_export;
pub mod generator;
pub mod strength;
pub mod folders;
pub mod tags;
```

Update `src-tauri/src/lib.rs` `invoke_handler` to include all new commands:

```rust
.invoke_handler(tauri::generate_handler![
    commands::vault::vault_setup,
    commands::vault::vault_unlock,
    commands::vault::vault_lock,
    commands::vault::vault_status,
    commands::entries::entry_list,
    commands::entries::entry_get,
    commands::entries::entry_create,
    commands::entries::entry_update,
    commands::entries::entry_delete,
    commands::entries::toggle_favorite,
    commands::generator::generate_password,
    commands::strength::evaluate_password_strength,
    commands::folders::folder_create,
    commands::folders::folder_rename,
    commands::folders::folder_delete,
    commands::folders::folder_list,
    commands::tags::tag_create,
    commands::tags::tag_update,
    commands::tags::tag_delete,
    commands::tags::tag_list,
    commands::tags::tag_add_to_entry,
    commands::tags::tag_remove_from_entry,
])
```

- [ ] **Step 4: Run full build**

Run: `cd src-tauri && cargo build`
Expected: Compiles with 0 errors

- [ ] **Step 5: Run all tests**

Run: `cd src-tauri && cargo test`
Expected: All tests PASS

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands/folders.rs src-tauri/src/commands/tags.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add folder and tag CRUD commands"
```

---

## Task 6: Frontend Stores + IPC Wrappers for Folders/Tags/Favorites

**Files:**
- Modify: `src/lib/utils/tauri.ts`
- Modify: `src/lib/stores/entries.ts`
- Create: `src/lib/stores/folders.ts`
- Create: `src/lib/stores/tags.ts`

- [ ] **Step 1: Add all new IPC wrappers to `tauri.ts`**

Append to `src/lib/utils/tauri.ts` after the existing password generation/strength interfaces and functions:

```typescript
// --- Folder operations ---

export interface Folder {
  id: string;
  name: string;
  parent_id: string | null;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export async function folderCreate(name: string, parentId: string | null): Promise<Folder> {
  return invoke('folder_create', { name, parentId });
}

export async function folderRename(id: string, name: string): Promise<boolean> {
  return invoke('folder_rename', { id, name });
}

export async function folderDelete(id: string): Promise<boolean> {
  return invoke('folder_delete', { id });
}

export async function folderList(): Promise<Folder[]> {
  return invoke('folder_list');
}

// --- Tag operations ---

export interface Tag {
  id: string;
  name: string;
  color: string;
  created_at: string;
  updated_at: string;
}

export async function tagCreate(name: string, color: string | null): Promise<Tag> {
  return invoke('tag_create', { name, color });
}

export async function tagUpdate(id: string, name: string, color: string): Promise<boolean> {
  return invoke('tag_update', { id, name, color });
}

export async function tagDelete(id: string): Promise<boolean> {
  return invoke('tag_delete', { id });
}

export async function tagList(): Promise<Tag[]> {
  return invoke('tag_list');
}

export async function tagAddToEntry(entryId: string, tagId: string): Promise<void> {
  return invoke('tag_add_to_entry', { entryId, tagId });
}

export async function tagRemoveFromEntry(entryId: string, tagId: string): Promise<void> {
  return invoke('tag_remove_from_entry', { entryId, tagId });
}

// --- Favorite operations ---

export async function toggleFavorite(id: string): Promise<boolean> {
  return invoke('toggle_favorite', { id });
}
```

- [ ] **Step 2: Update Entry interface in `entries.ts` — add `is_favorite`**

Replace `src/lib/stores/entries.ts`:

```typescript
import { writable } from 'svelte/store';
import { entryList, entryCreate, entryUpdate, entryDelete, toggleFavorite as toggleFavoriteApi } from '$lib/utils/tauri';

export interface Entry {
  id: string;
  folder_id: string | null;
  title: string;
  username: string | null;
  password: string | null;
  url: string | null;
  notes: string | null;
  custom_fields: string | null;
  tags: string | null;
  strength: number | null;
  expires_at: string | null;
  is_favorite: boolean;
  created_at: string;
  updated_at: string;
}

function createEntriesStore() {
  const { subscribe, set, update } = writable<Entry[]>([]);

  return {
    subscribe,
    async load() {
      const list = await entryList();
      set(list);
    },
    async create(entry: Entry) {
      await entryCreate(entry);
      update((items) => [entry, ...items]);
    },
    async save(entry: Entry) {
      await entryUpdate(entry);
      update((items) => items.map((e) => (e.id === entry.id ? entry : e)));
    },
    async remove(id: string) {
      await entryDelete(id);
      update((items) => items.filter((e) => e.id !== id));
    },
    async toggleFavorite(id: string) {
      await toggleFavoriteApi(id);
      update((items) =>
        items.map((e) => (e.id === id ? { ...e, is_favorite: !e.is_favorite } : e))
      );
    }
  };
}

export const entries = createEntriesStore();
```

- [ ] **Step 3: Create `src/lib/stores/folders.ts`**

```typescript
import { writable } from 'svelte/store';
import { folderList, folderCreate, folderRename as folderRenameApi, folderDelete as folderDeleteApi } from '$lib/utils/tauri';
import type { Folder } from '$lib/utils/tauri';

function createFoldersStore() {
  const { subscribe, set, update } = writable<Folder[]>([]);

  return {
    subscribe,
    async load() {
      const list = await folderList();
      set(list);
    },
    async create(name: string, parentId: string | null = null) {
      const folder = await folderCreate(name, parentId);
      update((items) => [...items, folder]);
      return folder;
    },
    async rename(id: string, name: string) {
      await folderRenameApi(id, name);
      update((items) => items.map((f) => (f.id === id ? { ...f, name } : f)));
    },
    async remove(id: string) {
      await folderDeleteApi(id);
      update((items) => items.filter((f) => f.id !== id));
    }
  };
}

export const folders = createFoldersStore();
```

- [ ] **Step 4: Create `src/lib/stores/tags.ts`**

```typescript
import { writable } from 'svelte/store';
import { tagList, tagCreate, tagUpdate as tagUpdateApi, tagDelete as tagDeleteApi } from '$lib/utils/tauri';
import type { Tag } from '$lib/utils/tauri';

function createTagsStore() {
  const { subscribe, set, update } = writable<Tag[]>([]);

  return {
    subscribe,
    async load() {
      const list = await tagList();
      set(list);
    },
    async create(name: string, color: string | null = null) {
      const tag = await tagCreate(name, color);
      update((items) => [...items, tag]);
      return tag;
    },
    async update(id: string, name: string, color: string) {
      await tagUpdateApi(id, name, color);
      update((items) => items.map((t) => (t.id === id ? { ...t, name, color } : t)));
    },
    async remove(id: string) {
      await tagDeleteApi(id);
      update((items) => items.filter((t) => t.id !== id));
    }
  };
}

export const tags = createTagsStore();
```

- [ ] **Step 5: Run svelte-check**

Run: `npx svelte-check --threshold error`
Expected: 0 errors

- [ ] **Step 6: Commit**

```bash
git add src/lib/utils/tauri.ts src/lib/stores/entries.ts src/lib/stores/folders.ts src/lib/stores/tags.ts
git commit -m "feat: add frontend stores and IPC wrappers for folders, tags, favorites"
```

---

## Task 7: Sidebar — FolderTree + TagCloud Components

**Files:**
- Create: `src/lib/components/FolderTree.svelte`
- Create: `src/lib/components/TagCloud.svelte`

- [ ] **Step 1: Create `FolderTree.svelte`**

```svelte
<!-- src/lib/components/FolderTree.svelte -->
<script lang="ts">
  import { folders } from '$lib/stores/folders';
  import type { Folder } from '$lib/utils/tauri';

  interface Props {
    selectedFolderId: string | null;
    onselect: (folderId: string | null) => void;
  }

  let { selectedFolderId, onselect }: Props = $props();
  let newFolderName = $state('');
  let showNewFolder = $state(false);

  async function handleCreate() {
    if (!newFolderName.trim()) return;
    await folders.create(newFolderName.trim());
    newFolderName = '';
    showNewFolder = false;
  }
</script>

<div class="border-b border-dark-border py-2">
  <div class="flex items-center justify-between px-3 pb-1">
    <span class="text-xs font-medium uppercase tracking-wide text-dark-muted">文件夹</span>
    <button
      class="cursor-pointer text-xs text-dark-muted hover:text-accent"
      onclick={() => (showNewFolder = !showNewFolder)}
      title="新建文件夹"
    >
      + 新建
    </button>
  </div>

  {#if showNewFolder}
    <div class="px-3 pb-1">
      <div class="flex gap-1">
        <input
          type="text"
          bind:value={newFolderName}
          placeholder="文件夹名称"
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

  <div class="flex flex-col">
    <!-- All entries option -->
    <button
      class="w-full cursor-pointer px-3 py-1 text-left text-xs transition-colors {selectedFolderId === null
        ? 'bg-dark-card text-dark-text'
        : 'text-dark-secondary hover:bg-dark-card/50'}"
      onclick={() => onselect(null)}
    >
      所有条目
    </button>

    {#each $folders as folder (folder.id)}
      <button
        class="w-full cursor-pointer px-3 py-1 text-left text-xs transition-colors {selectedFolderId === folder.id
          ? 'bg-dark-card text-dark-text'
          : 'text-dark-secondary hover:bg-dark-card/50'}"
        onclick={() => onselect(folder.id)}
      >
        {folder.name}
      </button>
    {/each}
  </div>
</div>
```

- [ ] **Step 2: Create `TagCloud.svelte`**

```svelte
<!-- src/lib/components/TagCloud.svelte -->
<script lang="ts">
  import { tags } from '$lib/stores/tags';
  import type { Tag } from '$lib/utils/tauri';

  interface Props {
    selectedTagId: string | null;
    onselect: (tagId: string | null) => void;
  }

  let { selectedTagId, onselect }: Props = $props();
  let newTagName = $state('');
  let showNewTag = $state(false);

  async function handleCreate() {
    if (!newTagName.trim()) return;
    await tags.create(newTagName.trim());
    newTagName = '';
    showNewTag = false;
  }
</script>

<div class="border-b border-dark-border py-2">
  <div class="flex items-center justify-between px-3 pb-1">
    <span class="text-xs font-medium uppercase tracking-wide text-dark-muted">标签</span>
    <button
      class="cursor-pointer text-xs text-dark-muted hover:text-accent"
      onclick={() => (showNewTag = !showNewTag)}
      title="新建标签"
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
        class="cursor-pointer rounded-full px-2 py-0.5 text-xs transition-colors {selectedTagId === tag.id
          ? 'ring-1 ring-white/20 text-white'
          : 'text-dark-secondary hover:text-dark-text'}"
        style:background-color={selectedTagId === tag.id ? tag.color : 'transparent'}
        style:border="1px solid {tag.color}"
        onclick={() => onselect(selectedTagId === tag.id ? null : tag.id)}
      >
        {tag.name}
      </button>
    {/each}

    {#if $tags.length === 0}
      <span class="text-xs text-dark-muted">暂无标签</span>
    {/if}
  </div>
</div>
```

- [ ] **Step 3: Run svelte-check**

Run: `npx svelte-check --threshold error`
Expected: 0 errors

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/FolderTree.svelte src/lib/components/TagCloud.svelte
git commit -m "feat: add FolderTree and TagCloud sidebar components"
```

---

## Task 8: Main Page — Sidebar Restructure + Search + Favorites

**Files:**
- Modify: `src/lib/components/EntryList.svelte`
- Modify: `src/routes/+page.svelte`

- [ ] **Step 1: Rewrite `EntryList.svelte` with favorites, search highlight, and filter props**

Replace the entire content of `src/lib/components/EntryList.svelte`:

```svelte
<!-- src/lib/components/EntryList.svelte -->
<script lang="ts">
  import type { Entry } from '$lib/stores/entries';
  import { entries } from '$lib/stores/entries';

  interface Props {
    entries: Entry[];
    selectedId: string | null;
    filterFolderId: string | null;
    filterTagId: string | null;
    onselect: (id: string) => void;
    oncreate: () => void;
    onlock: () => void;
  }

  let { entries: allEntries, selectedId, filterFolderId, filterTagId, onselect, oncreate, onlock }: Props = $props();
  let query = $state('');

  let filtered = $derived(() => {
    let result = allEntries;

    // Filter by folder
    if (filterFolderId !== null) {
      result = result.filter((e) => e.folder_id === filterFolderId);
    }

    // Filter by search query
    if (query) {
      const q = query.toLowerCase();
      result = result.filter(
        (e) =>
          e.title.toLowerCase().includes(q) ||
          (e.username ?? '').toLowerCase().includes(q) ||
          (e.url ?? '').toLowerCase().includes(q)
      );
    }

    return result;
  });

  function highlightMatch(text: string, q: string): string {
    if (!q) return text;
    const idx = text.toLowerCase().indexOf(q.toLowerCase());
    if (idx === -1) return text;
    return text.slice(0, idx) + text.slice(idx, idx + q.length) + text.slice(idx + q.length);
  }

  async function handleToggleFavorite(id: string, e: Event) {
    e.stopPropagation();
    await entries.toggleFavorite(id);
  }
</script>

<div class="flex flex-col">
  <!-- Entry list -->
  <div class="flex-1 overflow-y-auto">
    {#if filtered().length === 0}
      <div class="flex flex-col items-center justify-center py-12 text-dark-muted">
        <div class="mb-2 text-2xl">🔒</div>
        <p class="text-xs">{allEntries.length === 0 ? '还没有条目' : '没有匹配结果'}</p>
      </div>
    {:else}
      {#each filtered() as entry (entry.id)}
        <button
          class="group w-full cursor-pointer border-l-3 px-3 py-2 text-left transition-colors {selectedId === entry.id
            ? 'border-l-accent bg-dark-card'
            : 'border-l-transparent hover:bg-dark-card/50'}"
          onclick={() => onselect(entry.id)}
        >
          <div class="flex items-center gap-1">
            <button
              class="cursor-pointer text-xs opacity-0 transition-opacity group-hover:opacity-100 {entry.is_favorite ? '!opacity-100' : ''}"
              onclick={(e) => handleToggleFavorite(entry.id, e)}
              title={entry.is_favorite ? '取消收藏' : '收藏'}
            >
              {entry.is_favorite ? '★' : '☆'}
            </button>
            <span class="truncate text-sm font-medium {selectedId === entry.id ? 'text-dark-text' : 'text-dark-secondary'}">
              {entry.title || '无标题'}
            </span>
          </div>
          <div class="truncate pl-4 text-xs text-dark-muted">
            {entry.username ?? ''}
          </div>
        </button>
      {/each}
    {/if}
  </div>

  <!-- Bottom actions -->
  <div class="flex items-center justify-between border-t border-dark-border p-2">
    <button
      class="cursor-pointer rounded-md px-2 py-1.5 text-xs text-dark-muted hover:text-accent"
      onclick={onlock}
      title="锁定保险库"
    >
      🔒 锁定
    </button>
    <button
      class="cursor-pointer rounded-md bg-accent px-4 py-1.5 text-xs font-medium text-white hover:bg-accent-hover"
      onclick={oncreate}
    >
      + 新建条目
    </button>
  </div>
</div>
```

- [ ] **Step 2: Restructure `+page.svelte` with full sidebar layout**

Replace the entire content of `src/routes/+page.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { vault } from '$lib/stores/vault';
  import { entries } from '$lib/stores/entries';
  import { folders } from '$lib/stores/folders';
  import { tags } from '$lib/stores/tags';
  import LockScreen from '$lib/components/LockScreen.svelte';
  import EntryList from '$lib/components/EntryList.svelte';
  import EntryDetail from '$lib/components/EntryDetail.svelte';
  import EntryForm from '$lib/components/EntryForm.svelte';
  import FolderTree from '$lib/components/FolderTree.svelte';
  import TagCloud from '$lib/components/TagCloud.svelte';

  import type { Entry } from '$lib/stores/entries';

  type ViewMode = 'empty' | 'detail' | 'edit' | 'create';

  let selectedId: string | null = $state(null);
  let viewMode: ViewMode = $state('empty');
  let filterFolderId: string | null = $state(null);
  let filterTagId: string | null = $state(null);
  let searchQuery = $state('');

  let selectedEntry = $derived(
    selectedId ? $entries.find((e) => e.id === selectedId) ?? null : null
  );

  onMount(async () => {
    try {
      await vault.checkStatus();
    } catch {
      // Tauri not available (dev in browser)
    }
  });

  $effect(() => {
    if ($vault.isUnlocked) {
      entries.load();
      folders.load();
      tags.load();
    }
  });

  function selectEntry(id: string) {
    selectedId = id;
    viewMode = 'detail';
  }

  function startCreate() {
    selectedId = null;
    viewMode = 'create';
  }

  function startEdit() {
    viewMode = 'edit';
  }

  function cancelEdit() {
    if (selectedId) {
      viewMode = 'detail';
    } else {
      viewMode = 'empty';
    }
  }

  async function saveEntry(entry: Entry) {
    try {
      if (viewMode === 'create') {
        await entries.create(entry);
        selectedId = entry.id;
      } else {
        await entries.save(entry);
      }
      viewMode = 'detail';
    } catch (e) {
      console.error('保存失败:', e);
    }
  }

  async function deleteEntry() {
    if (!selectedEntry) return;
    const confirmed = window.confirm(`确定要删除「${selectedEntry.title}」吗？`);
    if (!confirmed) return;
    try {
      await entries.remove(selectedEntry.id);
      selectedId = null;
      viewMode = 'empty';
    } catch (e) {
      console.error('删除失败:', e);
    }
  }

  async function handleLock() {
    await vault.lock();
    selectedId = null;
    viewMode = 'empty';
  }
</script>

{#if !$vault.isUnlocked}
  <LockScreen />
{:else}
  <div class="flex h-screen">
    <!-- Left sidebar -->
    <div class="flex w-[35%] min-w-0 flex-col bg-dark-sidebar">
      <!-- Search -->
      <div class="border-b border-dark-border p-3">
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="搜索条目..."
          class="w-full rounded-md border border-dark-border bg-dark-card px-3 py-2 text-sm text-dark-text outline-none placeholder:text-dark-muted focus:border-accent"
        />
      </div>

      <!-- Folder navigation -->
      <FolderTree
        {selectedFolderId: filterFolderId}
        onselect={(id) => { filterFolderId = id; filterTagId = null; }}
      />

      <!-- Tag cloud -->
      <TagCloud
        {selectedTagId: filterTagId}
        onselect={(id) => { filterTagId = id; filterFolderId = null; }}
      />

      <!-- Entry list -->
      <div class="flex-1 overflow-hidden">
        <EntryList
          entries={$entries}
          {selectedId}
          {filterFolderId}
          {filterTagId}
          onselect={selectEntry}
          oncreate={startCreate}
          onlock={handleLock}
        />
      </div>
    </div>

    <!-- Right panel -->
    <div class="flex-1 bg-dark-bg">
      {#if viewMode === 'detail' && selectedEntry}
        <EntryDetail
          entry={selectedEntry}
          onedit={startEdit}
          ondelete={deleteEntry}
        />
      {:else if viewMode === 'edit' && selectedEntry}
        <EntryForm
          entry={selectedEntry}
          onsave={saveEntry}
          oncancel={cancelEdit}
        />
      {:else if viewMode === 'create'}
        <EntryForm
          onsave={saveEntry}
          oncancel={cancelEdit}
        />
      {:else}
        <div class="flex h-full items-center justify-center text-dark-muted">
          <div class="text-center">
            <div class="mb-2 text-3xl">📋</div>
            <p class="text-sm">选择一个条目查看详情</p>
            <p class="text-sm">或创建新条目</p>
          </div>
        </div>
      {/if}
    </div>
  </div>
{/if}
```

- [ ] **Step 3: Run svelte-check**

Run: `npx svelte-check --threshold error`
Expected: 0 errors

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/EntryList.svelte src/routes/+page.svelte
git commit -m "feat: restructure sidebar with folders, tags, search, and favorites"
```

---

## Task 9: EntryForm — Folder/Tag Selection

**Files:**
- Modify: `src/lib/components/EntryForm.svelte`

- [ ] **Step 1: Update EntryForm with folder selector and tag multi-select**

Replace the entire content of `src/lib/components/EntryForm.svelte`:

```svelte
<!-- src/lib/components/EntryForm.svelte -->
<script lang="ts">
  import type { Entry } from '$lib/stores/entries';
  import PasswordField from './PasswordField.svelte';
  import { folders } from '$lib/stores/folders';
  import { tags } from '$lib/stores/tags';

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
  let folderId = $state(entry?.folder_id ?? null);
  let selectedTagIds = $state<string[]>([]);

  function handleSubmit() {
    if (!title.trim()) return;
    const now = new Date().toISOString();
    onsave({
      id: entry?.id ?? crypto.randomUUID(),
      folder_id: folderId,
      title: title.trim(),
      username: username || null,
      password: password || null,
      url: url || null,
      notes: notes || null,
      custom_fields: entry?.custom_fields ?? null,
      tags: JSON.stringify(selectedTagIds),
      strength: entry?.strength ?? null,
      expires_at: entry?.expires_at ?? null,
      is_favorite: entry?.is_favorite ?? false,
      created_at: entry?.created_at ?? now,
      updated_at: now
    });
  }

  function toggleTag(tagId: string) {
    if (selectedTagIds.includes(tagId)) {
      selectedTagIds = selectedTagIds.filter((id) => id !== tagId);
    } else {
      selectedTagIds = [...selectedTagIds, tagId];
    }
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
        <option value={null}>无文件夹</option>
        {#each $folders as folder (folder.id)}
          <option value={folder.id}>{folder.name}</option>
        {/each}
      </select>
    </div>

    <div>
      <span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">标签</span>
      <div class="flex flex-wrap gap-1">
        {#each $tags as tag (tag.id)}
          <button
            type="button"
            class="cursor-pointer rounded-full px-2 py-0.5 text-xs transition-colors {selectedTagIds.includes(tag.id)
              ? 'text-white'
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

  <!-- Action buttons -->
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
```

- [ ] **Step 2: Run svelte-check**

Run: `npx svelte-check --threshold error`
Expected: 0 errors

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/EntryForm.svelte
git commit -m "feat: add folder selector and tag multi-select to entry form"
```

---

## Task 10: EntryDetail — Show Folder + Tags + Favorites

**Files:**
- Modify: `src/lib/components/EntryDetail.svelte`

- [ ] **Step 1: Update EntryDetail to display folder, tags, and favorite toggle**

Replace the entire content of `src/lib/components/EntryDetail.svelte`:

```svelte
<!-- src/lib/components/EntryDetail.svelte -->
<script lang="ts">
  import type { Entry } from '$lib/stores/entries';
  import { entries } from '$lib/stores/entries';
  import { folders } from '$lib/stores/folders';
  import { tags } from '$lib/stores/tags';
  import PasswordField from './PasswordField.svelte';

  interface Props {
    entry: Entry;
    onedit: () => void;
    ondelete: () => void;
  }

  let { entry, onedit, ondelete }: Props = $props();

  async function copyText(text: string) {
    await navigator.clipboard.writeText(text);
  }

  async function handleToggleFavorite() {
    await entries.toggleFavorite(entry.id);
  }

  let folderName = $derived(
    entry.folder_id
      ? $folders.find((f) => f.id === entry.folder_id)?.name ?? ''
      : ''
  );

  let entryTags = $derived(() => {
    if (!entry.tags) return [];
    try {
      const ids: string[] = JSON.parse(entry.tags);
      return ids
        .map((id) => $tags.find((t) => t.id === id))
        .filter(Boolean);
    } catch {
      return [];
    }
  });
</script>

<div class="flex h-full flex-col p-5">
  <!-- Top bar with favorite toggle -->
  <div class="mb-5 flex items-center justify-between">
    <div class="flex items-center gap-2">
      <button
        class="cursor-pointer text-lg {entry.is_favorite ? 'text-accent' : 'text-dark-muted hover:text-dark-secondary'}"
        onclick={handleToggleFavorite}
        title={entry.is_favorite ? '取消收藏' : '收藏'}
      >
        {entry.is_favorite ? '★' : '☆'}
      </button>
      <h2 class="text-lg font-bold text-dark-text">{entry.title || '无标题'}</h2>
    </div>
    <div class="flex gap-3">
      <button
        class="cursor-pointer text-xs text-accent hover:underline"
        onclick={onedit}
      >
        编辑
      </button>
      <button
        class="cursor-pointer text-xs text-danger hover:underline"
        onclick={ondelete}
      >
        删除
      </button>
    </div>
  </div>

  <!-- Metadata: folder + tags -->
  {#if folderName || entryTags().length > 0}
    <div class="mb-4 flex flex-wrap items-center gap-2">
      {#if folderName}
        <span class="rounded bg-dark-card px-2 py-0.5 text-xs text-dark-muted">
          📁 {folderName}
        </span>
      {/if}
      {#each entryTags() as tag (tag.id)}
        <span
          class="rounded-full px-2 py-0.5 text-xs text-white"
          style:background-color={tag.color}
        >
          {tag.name}
        </span>
      {/each}
    </div>
  {/if}

  <!-- Fields -->
  <div class="flex flex-col gap-4">
    {#if entry.username}
      <div>
        <span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">用户名</span>
        <div class="flex items-center justify-between rounded-md border border-dark-border bg-dark-card px-3 py-2 text-sm text-dark-text">
          <span>{entry.username}</span>
          <button
            class="cursor-pointer text-xs text-accent hover:underline"
            onclick={() => copyText(entry.username!)}
          >
            复制
          </button>
        </div>
      </div>
    {/if}

    {#if entry.password !== null}
      <div>
        <span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">密码</span>
        <PasswordField value={entry.password ?? ''} />
      </div>
    {/if}

    {#if entry.url}
      <div>
        <span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">网址</span>
        <div class="rounded-md border border-dark-border bg-dark-card px-3 py-2 text-sm text-accent">
          {entry.url}
        </div>
      </div>
    {/if}

    {#if entry.notes}
      <div>
        <span class="mb-1 block text-xs uppercase tracking-wide text-dark-muted">备注</span>
        <div class="whitespace-pre-wrap rounded-md border border-dark-border bg-dark-card px-3 py-2 text-sm text-dark-secondary">
          {entry.notes}
        </div>
      </div>
    {/if}
  </div>
</div>
```

- [ ] **Step 2: Run svelte-check**

Run: `npx svelte-check --threshold error`
Expected: 0 errors

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/EntryDetail.svelte
git commit -m "feat: show folder, tags, and favorite toggle in entry detail"
```

---

## Task 11: Toast Component + Final Polish

**Files:**
- Create: `src/lib/components/Toast.svelte`
- Modify: `src/lib/components/EntryDetail.svelte`
- Modify: `src/lib/components/EntryList.svelte`

- [ ] **Step 1: Create `Toast.svelte` — a simple auto-dismiss notification**

```svelte
<!-- src/lib/components/Toast.svelte -->
<script lang="ts">
  interface Props {
    message: string;
    visible: boolean;
  }

  let { message, visible }: Props = $props();
</script>

{#if visible}
  <div
    class="fixed bottom-4 right-4 z-50 rounded-lg border border-dark-border bg-dark-card px-4 py-2 text-sm text-dark-text shadow-lg transition-opacity"
    class:opacity-100={visible}
    class:opacity-0={!visible}
  >
    {message}
  </div>
{/if}
```

- [ ] **Step 2: Add toast feedback to EntryDetail copy actions**

In `src/lib/components/EntryDetail.svelte`, add a `copied` state and toast. Add these variables inside the script:

```typescript
let copiedField = $state('');
let showCopiedToast = $state(false);

async function copyText(text: string, field: string = '') {
  await navigator.clipboard.writeText(text);
  copiedField = field;
  showCopiedToast = true;
  setTimeout(() => {
    showCopiedToast = false;
    copiedField = '';
  }, 2000);
}
```

Update the copy button text in the username section:

```svelte
<button
  class="cursor-pointer text-xs text-accent hover:underline"
  onclick={() => copyText(entry.username!, 'username')}
>
  {copiedField === 'username' ? '已复制' : '复制'}
</button>
```

Add the Toast import and usage at the bottom of the template (before the closing `</div>`):

```svelte
<Toast message="已复制到剪贴板" visible={showCopiedToast} />
```

- [ ] **Step 3: Run svelte-check**

Run: `npx svelte-check --threshold error`
Expected: 0 errors

- [ ] **Step 4: Run full Rust tests**

Run: `cd src-tauri && cargo test`
Expected: All tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/Toast.svelte src/lib/components/EntryDetail.svelte
git commit -m "feat: add toast notifications for copy feedback"
```

---

## Task 12: Final Verification

- [ ] **Step 1: Run all Rust tests**

Run: `cd src-tauri && cargo test`
Expected: All tests PASS

- [ ] **Step 2: Run svelte type check**

Run: `npx svelte-check --threshold error`
Expected: 0 errors

- [ ] **Step 3: Run dev build**

Run: `cd src-tauri && cargo build`
Expected: Compiles successfully

- [ ] **Step 4: Manual smoke test checklist**

Launch the app and verify:
- [ ] Password generator popover opens from entry form
- [ ] Generator produces passwords matching selected options
- [ ] Password strength bar shows in generator
- [ ] Folder tree shows in sidebar, can create folders
- [ ] Tag cloud shows in sidebar, can create tags
- [ ] Selecting a folder filters the entry list
- [ ] Selecting a tag filters the entry list
- [ ] Favorites toggle works (star icon in list and detail)
- [ ] Favorited entries sort to top
- [ ] Entry form has folder dropdown and tag selection
- [ ] Entry detail shows folder and tags
- [ ] Copy buttons show "已复制" feedback
