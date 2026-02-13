<script lang="ts">
	import { goto } from '$app/navigation';
	import { register, getServerConfig } from '$lib/api/auth';
	import { authStore } from '$lib/stores/auth.svelte';
	import { isTauri, getServerUrl, clearServerUrl } from '$lib/env';
	import { onMount } from 'svelte';
	import { initCrypto, getKeyManager } from '$lib/crypto';

	let serverUrl = $derived(isTauri() ? getServerUrl() : null);

	function changeServer() {
		clearServerUrl();
		authStore.logout();
		goto('/connect');
	}

	let username = $state('');
	let email = $state('');
	let displayName = $state('');
	let password = $state('');
	let confirmPassword = $state('');
	let inviteCode = $state('');
	let error = $state('');
	let loading = $state(false);
	let registrationMode = $state('open');
	let configLoading = $state(true);

	// Password strength checks
	let pwHasLength = $derived(password.length >= 8);
	let pwHasUpper = $derived(/[A-Z]/.test(password));
	let pwHasLower = $derived(/[a-z]/.test(password));
	let pwHasDigit = $derived(/[0-9]/.test(password));
	let pwHasSpecial = $derived(/[^A-Za-z0-9]/.test(password));
	let pwAllMet = $derived(pwHasLength && pwHasUpper && pwHasLower && pwHasDigit && pwHasSpecial);

	onMount(async () => {
		try {
			const config = await getServerConfig();
			registrationMode = config.registration_mode;
		} catch {
			// If we can't fetch config, assume open
		} finally {
			configLoading = false;
		}
	});

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		error = '';

		if (!pwAllMet) {
			error = 'Password does not meet all requirements';
			return;
		}
		if (password !== confirmPassword) {
			error = 'Passwords do not match';
			return;
		}

		loading = true;

		try {
			// Generate real E2E encryption keys via WASM crypto module
			await initCrypto();
			const keys = await getKeyManager().generateRegistrationKeys();

			const response = await register({
				username,
				email,
				password,
				display_name: displayName || username,
				identity_key: keys.identityKey,
				signed_prekey: keys.signedPrekey,
				one_time_prekeys: keys.oneTimePrekeys,
				invite_code: registrationMode === 'invite_only' ? inviteCode : undefined
			});

			authStore.setAuth(response.access_token, response.refresh_token, response.user);
			goto('/channels');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Registration failed';
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex min-h-screen items-center justify-center p-4">
	<div class="w-full max-w-md rounded-2xl bg-[var(--bg-secondary)] p-8 shadow-2xl">
		<h1 class="mb-2 text-center text-3xl font-bold text-[var(--text-primary)]">Chatalot</h1>

		{#if serverUrl}
			<div class="mb-4 flex items-center justify-center gap-2 text-xs text-[var(--text-secondary)]">
				<span>Connected to {serverUrl}</span>
				<button onclick={changeServer} class="text-[var(--accent)] hover:underline">Change</button>
			</div>
		{/if}

		{#if configLoading}
			<div class="flex justify-center py-8">
				<div class="h-6 w-6 animate-spin rounded-full border-2 border-[var(--accent)] border-t-transparent"></div>
			</div>
		{:else if registrationMode === 'closed'}
			<p class="mb-4 text-center text-[var(--text-secondary)]">Registration is currently closed</p>
			<p class="text-center text-sm text-[var(--text-secondary)]">
				Contact the server administrator for access.
			</p>
			<p class="mt-4 text-center text-sm text-[var(--text-secondary)]">
				Already have an account?
				<a href="/login" class="text-[var(--accent)] transition hover:text-[var(--accent-hover)]">
					Sign in
				</a>
			</p>
		{:else}
			<p class="mb-8 text-center text-[var(--text-secondary)]">Create your account</p>

			{#if error}
				<div class="mb-4 rounded-lg bg-red-500/10 p-3 text-sm text-[var(--danger)]">
					{error}
				</div>
			{/if}

			<form onsubmit={handleSubmit} class="space-y-4">
				{#if registrationMode === 'invite_only'}
					<div>
						<label for="invite-code" class="mb-1 block text-sm font-medium text-[var(--text-secondary)]">
							Invite Code
						</label>
						<input
							id="invite-code"
							type="text"
							bind:value={inviteCode}
							required
							placeholder="Enter your invite code"
							class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 font-mono tracking-wider text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)]"
						/>
						<p class="mt-1 text-xs text-[var(--text-secondary)]">
							An invite code is required to register on this server.
						</p>
					</div>
				{/if}

				<div>
					<label for="username" class="mb-1 block text-sm font-medium text-[var(--text-secondary)]">
						Username
					</label>
					<input
						id="username"
						type="text"
						bind:value={username}
						required
						minlength="3"
						maxlength="32"
						autocomplete="username"
						class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)]"
					/>
				</div>

				<div>
					<label for="display-name" class="mb-1 block text-sm font-medium text-[var(--text-secondary)]">
						Display Name <span class="text-[var(--text-secondary)]/50">(optional)</span>
					</label>
					<input
						id="display-name"
						type="text"
						bind:value={displayName}
						maxlength="64"
						placeholder={username || 'Your display name'}
						class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)]"
					/>
				</div>

				<div>
					<label for="email" class="mb-1 block text-sm font-medium text-[var(--text-secondary)]">
						Email
					</label>
					<input
						id="email"
						type="email"
						bind:value={email}
						required
						autocomplete="email"
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
						minlength="8"
						autocomplete="new-password"
						class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)]"
					/>
					{#if password.length > 0}
						<div class="mt-2 space-y-1 text-xs">
							<div class={pwHasLength ? 'text-green-400' : 'text-[var(--text-secondary)]'}>
								{pwHasLength ? '✓' : '✗'} At least 8 characters
							</div>
							<div class={pwHasUpper ? 'text-green-400' : 'text-[var(--text-secondary)]'}>
								{pwHasUpper ? '✓' : '✗'} One uppercase letter
							</div>
							<div class={pwHasLower ? 'text-green-400' : 'text-[var(--text-secondary)]'}>
								{pwHasLower ? '✓' : '✗'} One lowercase letter
							</div>
							<div class={pwHasDigit ? 'text-green-400' : 'text-[var(--text-secondary)]'}>
								{pwHasDigit ? '✓' : '✗'} One digit
							</div>
							<div class={pwHasSpecial ? 'text-green-400' : 'text-[var(--text-secondary)]'}>
								{pwHasSpecial ? '✓' : '✗'} One special character
							</div>
						</div>
					{/if}
				</div>

				<div>
					<label for="confirm-password" class="mb-1 block text-sm font-medium text-[var(--text-secondary)]">
						Confirm Password
					</label>
					<input
						id="confirm-password"
						type="password"
						bind:value={confirmPassword}
						required
						autocomplete="new-password"
						class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)]"
					/>
				</div>

				<button
					type="submit"
					disabled={loading}
					class="w-full rounded-lg bg-[var(--accent)] px-4 py-2.5 font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:cursor-not-allowed disabled:opacity-50"
				>
					{loading ? 'Creating account...' : 'Create Account'}
				</button>
			</form>

			<p class="mt-4 text-center text-sm text-[var(--text-secondary)]">
				Already have an account?
				<a href="/login" class="text-[var(--accent)] transition hover:text-[var(--accent-hover)]">
					Sign in
				</a>
			</p>
		{/if}
	</div>
</div>
