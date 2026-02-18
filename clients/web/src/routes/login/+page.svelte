<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { login } from '$lib/api/auth';
	import { authStore } from '$lib/stores/auth.svelte';
	import { storePersonalKey } from '$lib/crypto';
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
	let showPassword = $state(false);
	let error = $state('');
	let loading = $state(false);

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		error = '';
		loading = true;

		try {
			const response = await login(username.trim(), password, showTotp ? totpCode.trim() : undefined);
			authStore.setAuth(response.access_token, response.refresh_token, response.user);
			// Derive personal encryption key from password before we lose access to it
			storePersonalKey(password, response.user.id).catch(err =>
				console.warn('Failed to derive personal key:', err)
			);
			const redirect = $page.url.searchParams.get('redirect');
			goto(redirect && redirect.startsWith('/') ? redirect : '/channels');
		} catch (err) {
			const msg = err instanceof Error ? err.message : 'Login failed';
			error = msg;
			// Auto-show TOTP field if the server says 2FA is required
			if (msg.toLowerCase().includes('2fa code required') || msg.toLowerCase().includes('2fa')) {
				if (!showTotp) showTotp = true;
				totpCode = '';
				error = msg.toLowerCase().includes('2fa code required') ? '' : msg;
				// Focus the TOTP input after the DOM updates
				setTimeout(() => document.getElementById('totp')?.focus(), 50);
			}
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex min-h-screen items-center justify-center p-3 sm:p-4">
	<div class="w-full max-w-md rounded-2xl bg-[var(--bg-secondary)] p-5 sm:p-8 shadow-2xl">
		<h1 class="mb-2 text-center text-3xl font-bold text-[var(--text-primary)]">Chatalot</h1>
		<p class="mb-8 text-center text-[var(--text-secondary)]">Welcome back</p>

		{#if serverUrl}
			<div class="mb-4 flex items-center justify-center gap-2 text-xs text-[var(--text-secondary)]">
				<span>Connected to {serverUrl}</span>
				<button onclick={changeServer} class="text-[var(--accent)] hover:underline">Change</button>
			</div>
		{/if}

		{#if error}
			<div class="mb-4 rounded-lg bg-red-500/10 p-3 text-sm text-[var(--danger)]" role="alert">
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
					autofocus
					maxlength={32}
					autocomplete="username"
					class="w-full rounded-xl border border-[var(--border)] bg-[var(--bg-primary)] px-4 py-2.5 text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/30"
				/>
			</div>

			<div>
				<label for="password" class="mb-1 block text-sm font-medium text-[var(--text-secondary)]">
					Password
				</label>
				<div class="relative">
					<input
						id="password"
						type={showPassword ? 'text' : 'password'}
						bind:value={password}
						required
						maxlength={128}
						autocomplete="current-password"
						class="w-full rounded-xl border border-[var(--border)] bg-[var(--bg-primary)] px-4 py-2.5 pr-10 text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/30"
					/>
					<button
						type="button"
						onclick={() => showPassword = !showPassword}
						class="absolute right-3 top-1/2 -translate-y-1/2 text-[var(--text-secondary)] transition hover:text-[var(--text-primary)]"
						aria-label={showPassword ? 'Hide password' : 'Show password'}
					>
						{#if showPassword}
							<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/><line x1="1" y1="1" x2="23" y2="23"/></svg>
						{:else}
							<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
						{/if}
					</button>
				</div>
			</div>

			{#if showTotp}
				<div>
					<label for="totp" class="mb-1 block text-sm font-medium text-[var(--text-secondary)]">
						2FA Code or Backup Code
					</label>
					<input
						id="totp"
						type="text"
						bind:value={totpCode}
						required
						autofocus
						autocomplete="one-time-code"
						placeholder="123456 or XXXX-XXXX"
						class="w-full rounded-xl border border-[var(--border)] bg-[var(--bg-primary)] px-4 py-2.5 text-center font-mono text-lg tracking-widest text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/30"
					/>
					<p class="mt-1 text-xs text-[var(--text-secondary)]/70">
						Enter your 6-digit TOTP code, or a backup code if you lost your device.
					</p>
				</div>
			{/if}

			<button
				type="submit"
				disabled={loading}
				class="w-full rounded-xl bg-[var(--accent)] px-4 py-2.5 font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:cursor-not-allowed disabled:opacity-50"
			>
				{#if loading}
					<span class="inline-flex items-center gap-2">
						<svg class="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25"/><path d="M4 12a8 8 0 018-8" stroke="currentColor" stroke-width="3" stroke-linecap="round" class="opacity-75"/></svg>
						Signing in...
					</span>
				{:else}
					Sign In
				{/if}
			</button>
		</form>

		<div class="mt-4 flex items-center justify-between text-sm">
			<button
				onclick={() => { showTotp = !showTotp; if (!showTotp) totpCode = ''; }}
				class="text-[var(--text-secondary)] transition hover:text-[var(--text-primary)]"
			>
				{showTotp ? 'Hide 2FA' : 'Have a 2FA code?'}
			</button>
			<div class="flex gap-3">
				<a href="/recover" class="text-[var(--text-secondary)] transition hover:text-[var(--text-primary)]">
					Forgot password?
				</a>
				<a href="/register" class="text-[var(--accent)] transition hover:text-[var(--accent-hover)]">
					Create account
				</a>
			</div>
		</div>
	</div>
</div>
