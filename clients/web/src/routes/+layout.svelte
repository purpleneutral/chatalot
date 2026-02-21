<script lang="ts">
	import '../app.css';
	import Toast from '$lib/components/Toast.svelte';
	import PersistentAudio from '$lib/components/PersistentAudio.svelte';

	let { children } = $props();
	let runtimeError = $state<Error | null>(null);

	function handleError(error: unknown) {
		console.error('Runtime error caught by boundary:', error);
		runtimeError = error instanceof Error ? error : new Error(String(error));
	}

	function handleReload() {
		window.location.reload();
	}
</script>

<svelte:head>
	<title>Chatalot</title>
</svelte:head>

{#if runtimeError}
	<div class="flex min-h-screen items-center justify-center p-4">
		<div class="w-full max-w-md rounded-2xl bg-[var(--bg-secondary)] p-8 text-center shadow-2xl">
			<h1 class="mb-2 text-2xl font-bold text-[var(--text-primary)]">Something went wrong</h1>
			<p class="mb-4 text-sm text-[var(--text-secondary)]">
				An unexpected error occurred. Try reloading the page.
			</p>
			<pre class="mb-6 max-h-32 overflow-auto rounded-lg bg-[var(--bg-primary)] p-3 text-left text-xs text-[var(--danger)]">{runtimeError.message}</pre>
			<button
				onclick={handleReload}
				class="rounded-xl bg-[var(--accent)] px-6 py-2.5 font-medium text-white transition hover:bg-[var(--accent-hover)]"
			>
				Reload Page
			</button>
		</div>
	</div>
{:else}
	<svelte:boundary onerror={handleError}>
		{@render children()}
	</svelte:boundary>
{/if}
<Toast />
<PersistentAudio />
