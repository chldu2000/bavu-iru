<script lang="ts">
	import { vault } from '$lib/stores/vault';

	let password = $state('');
	let confirmPassword = $state('');
	let error = $state('');
	let isSetup = $state(false);
	let loading = $state(false);

	async function handleSubmit() {
		if (!password) return;

		if (isSetup && password !== confirmPassword) {
			error = '两次输入的密码不一致';
			return;
		}
		if (password.length < 8) {
			error = '密码至少需要 8 个字符';
			return;
		}

		error = '';
		loading = true;
		try {
			if (isSetup) {
				await vault.setup(password);
			} else {
				await vault.unlock(password);
			}
		} catch (e: unknown) {
			const msg = e instanceof Error ? e.message : String(e);
			if (msg.includes('VaultNotFound') || msg.includes('not found')) {
				error = '未找到保险库，请先创建';
			} else if (msg.includes('InvalidPassword') || msg.includes('invalid')) {
				error = '密码错误';
			} else if (msg.includes('AlreadyExists')) {
				error = '保险库已存在，请切换到解锁模式';
			} else if (msg.includes('AlreadyUnlocked')) {
				error = '保险库已解锁';
			} else {
				error = msg;
			}
		} finally {
			loading = false;
		}
	}

	function toggleMode() {
		isSetup = !isSetup;
		error = '';
		password = '';
		confirmPassword = '';
	}
</script>

<div class="flex h-screen items-center justify-center bg-dark-bg">
	<div class="w-80 text-center">
		<h1 class="mb-1 text-3xl font-bold text-dark-text">Bavu-Iru</h1>
		<p class="mb-8 text-sm text-dark-muted">密码管理器</p>

		<form onsubmit|preventDefault={handleSubmit} class="flex flex-col gap-3">
			{#if error}
				<div class="rounded-md bg-danger/10 px-3 py-2 text-sm text-danger">{error}</div>
			{/if}

			<input
				type="password"
				bind:value={password}
				placeholder="输入主密码"
				class="w-full rounded-md border border-dark-border bg-dark-card px-4 py-3 text-sm text-dark-text outline-none placeholder:text-dark-muted focus:border-accent"
				autocomplete="off"
				autofocus
			/>

			{#if isSetup}
				<input
					type="password"
					bind:value={confirmPassword}
					placeholder="确认主密码"
					class="w-full rounded-md border border-dark-border bg-dark-card px-4 py-3 text-sm text-dark-text outline-none placeholder:text-dark-muted focus:border-accent"
					autocomplete="off"
				/>
			{/if}

			<button
				type="submit"
				disabled={loading}
				class="w-full cursor-pointer rounded-md bg-accent py-3 text-sm font-medium text-white hover:bg-accent-hover disabled:cursor-not-allowed disabled:opacity-50"
			>
				{loading ? '处理中...' : (isSetup ? '创建保险库' : '解锁保险库')}
			</button>
		</form>

		<button
			onclick={toggleMode}
			class="mt-4 cursor-pointer text-sm text-dark-muted hover:text-accent"
		>
			{isSetup ? '已有保险库？点击解锁' : '没有保险库？点击创建'}
		</button>
	</div>
</div>
