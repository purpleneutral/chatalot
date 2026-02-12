<script lang="ts">
	import { goto } from '$app/navigation';
	import { login } from '$lib/api/auth';
	import { authStore } from '$lib/stores/auth.svelte';
	import { isTauri, getServerUrl, clearServerUrl } from '$lib/env';

	let serverUrl = $derived(isTauri() ? getServerUrl() : null);

	function changeServer() {
		clearServerUrl();
		authStore.logout();
		goto('/connect');
	}

	let username = $state('');
	let password = $state('');
	let totpCode = $state('');
	let showTotp = $state(false);
	let error = $state('');
	let loading = $state(false);

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		error = '';
		loading = true;

		try {
			const response = await login(username, password, showTotp ? totpCode : undefined);
			authStore.setAuth(response.access_token, response.refresh_token, response.user);
			goto('/channels');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Login failed';
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex min-h-screen items-center justify-center p-4">
	<div class="w-full max-w-md rounded-2xl bg-[var(--bg-secondary)] p-8 shadow-2xl">
		<h1 class="mb-2 text-center text-3xl font-bold text-[var(--text-primary)]">Chatalot</h1>
		<p class="mb-8 text-center text-[var(--text-secondary)]">Welcome back</p>

		{#if serverUrl}
			<div class="mb-4 flex items-center justify-center gap-2 text-xs text-[var(--text-secondary)]">
				<span>Connected to {serverUrl}</span>
				<button onclick={changeServer} class="text-[var(--accent)] hover:underline">Change</button>
			</div>
		{/if}

		{#if error}
			<div class="mb-4 rounded-lg bg-red-500/10 p-3 text-sm text-[var(--danger)]">
				{error}
			</div>
		{/if}

		<form onsubmit={handleSubmit} class="space-y-4">
			<div>
				<label for="username" class="mb-1 block text-sm font-medium text-[var(--text-secondary)]">
					Username
				</label>
				<input
					id="username"
					type="text"
					bind:value={username}
					required
					autocomplete="username"
					class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)]"
				/>
			</div>

			<div>
				<label for="password" class="mb-1 block text-sm font-medium text-[var(--text-secondary)]">
					Password
				</label>
				<input
					id="password"
					type="password"
					bind:value={password}
					required
					autocomplete="current-password"
					class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)]"
				/>
			</div>

			{#if showTotp}
				<div>
					<label for="totp" class="mb-1 block text-sm font-medium text-[var(--text-secondary)]">
						2FA Code
					</label>
					<input
						id="totp"
						type="text"
						bind:value={totpCode}
						inputmode="numeric"
						pattern="[0-9]{6}"
						maxlength="6"
						autocomplete="one-time-code"
						class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 text-center font-mono text-lg tracking-widest text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)]"
					/>
				</div>
			{/if}

			<button
				type="submit"
				disabled={loading}
				class="w-full rounded-lg bg-[var(--accent)] px-4 py-2.5 font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:cursor-not-allowed disabled:opacity-50"
			>
				{loading ? 'Signing in...' : 'Sign In'}
			</button>
		</form>

		<div class="mt-4 flex items-center justify-between text-sm">
			<button
				onclick={() => (showTotp = !showTotp)}
				class="text-[var(--text-secondary)] transition hover:text-[var(--text-primary)]"
			>
				{showTotp ? 'Hide 2FA' : 'Have a 2FA code?'}
			</button>
			<a href="/register" class="text-[var(--accent)] transition hover:text-[var(--accent-hover)]">
				Create account
			</a>
		</div>
	</div>
</div>
