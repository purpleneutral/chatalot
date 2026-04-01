<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { handleOidcCallback } from '$lib/api/auth';
	import { authStore } from '$lib/stores/auth.svelte';
	import { onMount } from 'svelte';

	let error = $state('');
	let processing = $state(true);

	onMount(async () => {
		const code = $page.url.searchParams.get('code');
		const state = $page.url.searchParams.get('state');

		if (!code || !state) {
			error = 'Missing authorization parameters. Please try signing in again.';
			processing = false;
			return;
		}

		try {
			const response = await handleOidcCallback(code, state);
			authStore.setAuth(response.access_token, response.refresh_token, response.user);

			if (!response.keys_registered) {
				goto('/oidc/setup-keys');
			} else {
				goto('/channels');
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Authentication failed';
			processing = false;
		}
	});
</script>

<div class="flex min-h-screen items-center justify-center p-3 sm:p-4">
	<div class="w-full max-w-md rounded-2xl bg-[var(--bg-secondary)] p-5 sm:p-8 shadow-2xl">
		<h1 class="mb-2 text-center text-3xl font-bold text-[var(--text-primary)]">Chatalot</h1>

		{#if processing}
			<div class="flex flex-col items-center gap-4 py-8">
				<div class="h-8 w-8 animate-spin rounded-full border-3 border-[var(--accent)] border-t-transparent"></div>
				<p class="text-[var(--text-secondary)]">Completing sign in...</p>
			</div>
		{:else if error}
			<div class="py-4">
				<div class="mb-6 rounded-lg bg-red-500/10 p-4 text-center text-sm text-[var(--danger)]" role="alert">
					{error}
				</div>
				<a
					href="/login"
					class="block w-full rounded-xl bg-[var(--accent)] px-4 py-2.5 text-center font-medium text-white transition hover:bg-[var(--accent-hover)]"
				>
					Back to Sign In
				</a>
			</div>
		{/if}
	</div>
</div>
