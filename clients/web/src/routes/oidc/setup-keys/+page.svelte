<script lang="ts">
	import { goto } from '$app/navigation';
	import { completeOidcSetup } from '$lib/api/auth';
	import { authStore } from '$lib/stores/auth.svelte';
	import { initCrypto, getKeyManager } from '$lib/crypto';
	import { onMount } from 'svelte';

	let error = $state('');
	let generating = $state(false);
	let success = $state(false);

	onMount(() => {
		if (!authStore.isAuthenticated) {
			goto('/login');
		}
	});

	async function handleGenerateKeys() {
		error = '';
		generating = true;

		try {
			await initCrypto();
			const keys = await getKeyManager().generateRegistrationKeys();

			await completeOidcSetup({
				identity_key: keys.identityKey,
				signed_prekey: keys.signedPrekey,
				one_time_prekeys: keys.oneTimePrekeys
			});

			success = true;
			setTimeout(() => goto('/channels'), 2000);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to generate encryption keys';
		} finally {
			generating = false;
		}
	}
</script>

<div class="flex min-h-screen items-center justify-center p-3 sm:p-4">
	<div class="w-full max-w-md rounded-2xl bg-[var(--bg-secondary)] p-5 sm:p-8 shadow-2xl">
		<div class="mb-6 flex justify-center">
			<div class="flex h-16 w-16 items-center justify-center rounded-2xl bg-[var(--accent)]/10">
				<svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8 text-[var(--accent)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4" />
				</svg>
			</div>
		</div>

		<h1 class="mb-2 text-center text-2xl font-bold text-[var(--text-primary)]">
			Set Up End-to-End Encryption
		</h1>
		<p class="mb-8 text-center text-sm text-[var(--text-secondary)]">
			Your messages are encrypted so only you and the recipient can read them. Generate your encryption keys to get started.
		</p>

		{#if error}
			<div class="mb-4 rounded-lg bg-red-500/10 p-3 text-sm text-[var(--danger)]" role="alert">
				{error}
			</div>
		{/if}

		{#if success}
			<div class="flex flex-col items-center gap-4 py-4">
				<div class="flex h-12 w-12 items-center justify-center rounded-full bg-green-500/10">
					<svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-green-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
						<polyline points="20 6 9 17 4 12" />
					</svg>
				</div>
				<p class="font-medium text-green-400">Encryption keys generated</p>
				<p class="text-sm text-[var(--text-secondary)]">Redirecting to Chatalot...</p>
			</div>
		{:else}
			<button
				onclick={handleGenerateKeys}
				disabled={generating}
				class="w-full rounded-xl bg-[var(--accent)] px-4 py-3 font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:cursor-not-allowed disabled:opacity-50"
			>
				{#if generating}
					<span class="inline-flex items-center justify-center gap-2">
						<svg class="h-5 w-5 animate-spin" viewBox="0 0 24 24" fill="none">
							<circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
							<path d="M4 12a8 8 0 018-8" stroke="currentColor" stroke-width="3" stroke-linecap="round" class="opacity-75" />
						</svg>
						Generating keys...
					</span>
				{:else}
					Generate Encryption Keys
				{/if}
			</button>

			<p class="mt-4 text-center text-xs text-[var(--text-secondary)]/70">
				This creates your unique encryption identity. Keys are stored securely in your browser.
			</p>
		{/if}
	</div>
</div>
