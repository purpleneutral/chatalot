<script lang="ts">
	import { goto } from '$app/navigation';
	import { authStore } from '$lib/stores/auth.svelte';
	import { isTauri, getServerUrl } from '$lib/env';
	import { onMount } from 'svelte';

	onMount(() => {
		// Desktop mode: need a server URL first
		if (isTauri() && !getServerUrl()) {
			goto('/connect');
			return;
		}

		if (authStore.isAuthenticated) {
			goto('/channels');
		} else {
			goto('/login');
		}
	});
</script>

<div class="flex h-screen flex-col items-center justify-center gap-3">
	<div class="h-8 w-8 animate-spin rounded-full border-2 border-[var(--accent)] border-t-transparent"></div>
	<div class="text-sm text-[var(--text-secondary)]">Loading Chatalot...</div>
</div>
