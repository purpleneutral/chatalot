<script lang="ts">
	import { goto } from '$app/navigation';
	import { recoverAccount } from '$lib/api/auth';

	let username = $state('');
	let recoveryCode = $state('');
	let newPassword = $state('');
	let confirmPassword = $state('');
	let error = $state('');
	let loading = $state(false);
	let showNewPassword = $state(false);
	let showConfirmPassword = $state(false);
	let newRecoveryCode = $state('');
	let showSuccess = $state(false);
	let copiedRecovery = $state(false);

	let pwMatch = $derived(confirmPassword.length > 0 && newPassword === confirmPassword);
	let pwMismatch = $derived(confirmPassword.length > 0 && newPassword !== confirmPassword);

	// Password strength checks
	let pwHasLength = $derived(newPassword.length >= 8);
	let pwHasUpper = $derived(/[A-Z]/.test(newPassword));
	let pwHasLower = $derived(/[a-z]/.test(newPassword));
	let pwHasDigit = $derived(/[0-9]/.test(newPassword));
	let pwHasSpecial = $derived(/[^A-Za-z0-9]/.test(newPassword));
	let pwAllMet = $derived(pwHasLength && pwHasUpper && pwHasLower && pwHasDigit && pwHasSpecial);

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		error = '';

		if (!pwAllMet) {
			error = 'Password does not meet all requirements';
			return;
		}
		if (newPassword !== confirmPassword) {
			error = 'Passwords do not match';
			return;
		}

		loading = true;

		try {
			const normalizedCode = recoveryCode.trim().toUpperCase().replace(/\s+/g, '-');
			const response = await recoverAccount(username.trim(), normalizedCode, newPassword);
			newRecoveryCode = response.recovery_code;
			showSuccess = true;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Recovery failed';
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex min-h-screen items-center justify-center p-3 sm:p-4">
	<div class="w-full max-w-md rounded-2xl bg-[var(--bg-secondary)] p-5 sm:p-8 shadow-2xl">
		<h1 class="mb-2 text-center text-3xl font-bold text-[var(--text-primary)]">Chatalot</h1>
		<p class="mb-8 text-center text-[var(--text-secondary)]">Account Recovery</p>

		{#if showSuccess}
			<div class="space-y-4">
				<div class="rounded-xl border border-green-500/20 bg-green-500/10 px-4 py-3 text-sm text-green-400">
					Password reset successfully! All sessions have been revoked.
				</div>

				<div class="rounded-xl bg-[var(--bg-primary)] p-4">
					<p class="mb-2 text-sm font-medium text-[var(--text-primary)]">Your New Recovery Code</p>
					<p class="mb-3 text-xs text-[var(--text-secondary)]">
						Save this code — it replaces your previous one and will not be shown again.
					</p>
					<div class="mb-3 rounded-xl bg-[var(--bg-secondary)] p-3 text-center">
						<code class="select-all font-mono text-lg font-bold tracking-wider text-[var(--accent)]">
							{newRecoveryCode}
						</code>
					</div>
					<button
						onclick={() => {
							navigator.clipboard.writeText(newRecoveryCode);
							copiedRecovery = true;
							setTimeout(() => (copiedRecovery = false), 2000);
						}}
						class="w-full rounded-xl border border-[var(--border)] px-4 py-2 text-sm text-[var(--text-primary)] transition hover:bg-white/5"
					>
						{copiedRecovery ? 'Copied!' : 'Copy Code'}
					</button>
				</div>

				<a
					href="/login"
					class="block w-full rounded-xl bg-[var(--accent)] px-4 py-2.5 text-center text-sm font-medium text-white transition hover:bg-[var(--accent-hover)]"
				>
					Go to Login
				</a>
			</div>
		{:else}
			<p class="mb-6 text-sm text-[var(--text-secondary)]">
				Enter your username and recovery code to reset your password.
				Your recovery code was shown when you created your account.
			</p>

			{#if error}
				<div class="mb-4 rounded-xl bg-red-500/10 p-3 text-sm text-[var(--danger)]" role="alert">
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
					<label for="recovery-code" class="mb-1 block text-sm font-medium text-[var(--text-secondary)]">
						Recovery Code
					</label>
					<input
						id="recovery-code"
						type="text"
						bind:value={recoveryCode}
						required
						maxlength={40}
						placeholder="XXXX-XXXX-XXXX-XXXX"
						class="w-full rounded-xl border border-[var(--border)] bg-[var(--bg-primary)] px-4 py-2.5 font-mono tracking-wider text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/30"
					/>
				</div>

				<div>
					<label for="new-password" class="mb-1 block text-sm font-medium text-[var(--text-secondary)]">
						New Password
					</label>
					<div class="relative">
						<input
							id="new-password"
							type={showNewPassword ? 'text' : 'password'}
							bind:value={newPassword}
							required
							minlength="8"
							maxlength={128}
							autocomplete="new-password"
							class="w-full rounded-xl border border-[var(--border)] bg-[var(--bg-primary)] px-4 py-2.5 pr-10 text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/30"
						/>
						<button
							type="button"
							onclick={() => showNewPassword = !showNewPassword}
							class="absolute right-3 top-1/2 -translate-y-1/2 text-[var(--text-secondary)] transition hover:text-[var(--text-primary)]"
							aria-label={showNewPassword ? 'Hide password' : 'Show password'}
						>
							{#if showNewPassword}
								<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/><line x1="1" y1="1" x2="23" y2="23"/></svg>
							{:else}
								<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
							{/if}
						</button>
					</div>
					{#if newPassword.length > 0}
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
						Confirm New Password
					</label>
					<div class="relative">
						<input
							id="confirm-password"
							type={showConfirmPassword ? 'text' : 'password'}
							bind:value={confirmPassword}
							required
							maxlength={128}
							autocomplete="new-password"
							class="w-full rounded-xl border border-[var(--border)] bg-[var(--bg-primary)] px-4 py-2.5 pr-10 text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/30"
						/>
						<button
							type="button"
							onclick={() => showConfirmPassword = !showConfirmPassword}
							class="absolute right-3 top-1/2 -translate-y-1/2 text-[var(--text-secondary)] transition hover:text-[var(--text-primary)]"
							aria-label={showConfirmPassword ? 'Hide password' : 'Show password'}
						>
							{#if showConfirmPassword}
								<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/><line x1="1" y1="1" x2="23" y2="23"/></svg>
							{:else}
								<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
							{/if}
						</button>
					</div>
					{#if pwMatch}
						<p class="mt-1 text-xs text-green-400">Passwords match</p>
					{:else if pwMismatch}
						<p class="mt-1 text-xs text-[var(--danger)]">Passwords do not match</p>
					{/if}
				</div>

				<button
					type="submit"
					disabled={loading}
					class="w-full rounded-xl bg-[var(--accent)] px-4 py-2.5 font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:cursor-not-allowed disabled:opacity-50"
				>
					{#if loading}
						<span class="inline-flex items-center gap-2">
							<svg class="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25"/><path d="M4 12a8 8 0 018-8" stroke="currentColor" stroke-width="3" stroke-linecap="round" class="opacity-75"/></svg>
							Recovering...
						</span>
					{:else}
						Reset Password
					{/if}
				</button>
			</form>

			<p class="mt-4 text-center text-sm text-[var(--text-secondary)]">
				Remember your password?
				<a href="/login" class="text-[var(--accent)] transition hover:text-[var(--accent-hover)]">
					Sign in
				</a>
			</p>
		{/if}
	</div>
</div>
