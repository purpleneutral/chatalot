<script lang="ts">
	import { goto } from '$app/navigation';
	import { recoverAccount } from '$lib/api/auth';

	let username = $state('');
	let recoveryCode = $state('');
	let newPassword = $state('');
	let confirmPassword = $state('');
	let error = $state('');
	let loading = $state(false);
	let newRecoveryCode = $state('');
	let showSuccess = $state(false);
	let copiedRecovery = $state(false);

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
			const response = await recoverAccount(username, recoveryCode, newPassword);
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
				<div class="rounded-lg border border-green-500/20 bg-green-500/10 px-4 py-3 text-sm text-green-400">
					Password reset successfully! All sessions have been revoked.
				</div>

				<div class="rounded-lg bg-[var(--bg-primary)] p-4">
					<p class="mb-2 text-sm font-medium text-[var(--text-primary)]">Your New Recovery Code</p>
					<p class="mb-3 text-xs text-[var(--text-secondary)]">
						Save this code — it replaces your previous one and will not be shown again.
					</p>
					<div class="mb-3 rounded-lg bg-[var(--bg-secondary)] p-3 text-center">
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
						class="w-full rounded-lg border border-white/10 px-4 py-2 text-sm text-[var(--text-primary)] transition hover:bg-white/5"
					>
						{copiedRecovery ? 'Copied!' : 'Copy Code'}
					</button>
				</div>

				<a
					href="/login"
					class="block w-full rounded-lg bg-[var(--accent)] px-4 py-2.5 text-center text-sm font-medium text-white transition hover:bg-[var(--accent-hover)]"
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
						class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/50"
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
						placeholder="XXXX-XXXX-XXXX-XXXX"
						class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 font-mono tracking-wider text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/50"
					/>
				</div>

				<div>
					<label for="new-password" class="mb-1 block text-sm font-medium text-[var(--text-secondary)]">
						New Password
					</label>
					<input
						id="new-password"
						type="password"
						bind:value={newPassword}
						required
						minlength="8"
						autocomplete="new-password"
						class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/50"
					/>
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
					<input
						id="confirm-password"
						type="password"
						bind:value={confirmPassword}
						required
						autocomplete="new-password"
						class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/50"
					/>
				</div>

				<button
					type="submit"
					disabled={loading}
					class="w-full rounded-lg bg-[var(--accent)] px-4 py-2.5 font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:cursor-not-allowed disabled:opacity-50"
				>
					{loading ? 'Recovering...' : 'Reset Password'}
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
