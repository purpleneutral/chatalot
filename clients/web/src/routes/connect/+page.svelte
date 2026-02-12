<script lang="ts">
	import { goto } from '$app/navigation';
	import { isTauri, setServerUrl, getServerUrl } from '$lib/env';
	import { onMount } from 'svelte';

	let serverUrl = $state('');
	let error = $state('');
	let loading = $state(false);

	onMount(() => {
		// If not in Tauri mode, redirect to login
		if (!isTauri()) {
			goto('/login');
			return;
		}
		// If already have a server URL, go to login
		if (getServerUrl()) {
			goto('/login');
		}
	});

	async function handleConnect(e: SubmitEvent) {
		e.preventDefault();
		error = '';

		// Clean up the URL
		let url = serverUrl.trim().replace(/\/+$/, '');
		if (!url) {
			error = 'Please enter a server URL';
			return;
		}

		// Auto-add protocol if missing
		if (!url.startsWith('http://') && !url.startsWith('https://')) {
			url = `https://${url}`;
		}

		loading = true;

		try {
			// Validate by hitting the health endpoint
			const response = await fetch(`${url}/api/health`, {
				signal: AbortSignal.timeout(10000)
			});

			if (!response.ok) {
				throw new Error(`Server responded with ${response.status}`);
			}

			const data = await response.json();
			if (!data.status || data.status !== 'ok') {
				throw new Error('Invalid server response');
			}

			// Save and redirect
			setServerUrl(url);
			goto('/login');
		} catch (err) {
			if (err instanceof TypeError && err.message.includes('fetch')) {
				error = 'Could not reach server. Check the URL and try again.';
			} else if (err instanceof DOMException && err.name === 'TimeoutError') {
				error = 'Connection timed out. Check the URL and try again.';
			} else {
				error = err instanceof Error ? err.message : 'Failed to connect';
			}
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex min-h-screen items-center justify-center p-4">
	<div class="w-full max-w-md rounded-2xl bg-[var(--bg-secondary)] p-8 shadow-2xl">
		<h1 class="mb-2 text-center text-3xl font-bold text-[var(--text-primary)]">Chatalot</h1>
		<p class="mb-8 text-center text-[var(--text-secondary)]">Connect to a server</p>

		{#if error}
			<div class="mb-4 rounded-lg bg-red-500/10 p-3 text-sm text-[var(--danger)]">
				{error}
			</div>
		{/if}

		<form onsubmit={handleConnect} class="space-y-4">
			<div>
				<label for="server-url" class="mb-1 block text-sm font-medium text-[var(--text-secondary)]">
					Server URL
				</label>
				<input
					id="server-url"
					type="text"
					bind:value={serverUrl}
					required
					placeholder="https://chat.example.com"
					autofocus
					class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)]"
				/>
				<p class="mt-1 text-xs text-[var(--text-secondary)]">
					Enter the URL of the Chatalot server you want to connect to.
				</p>
			</div>

			<button
				type="submit"
				disabled={loading}
				class="w-full rounded-lg bg-[var(--accent)] px-4 py-2.5 font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:cursor-not-allowed disabled:opacity-50"
			>
				{loading ? 'Connecting...' : 'Connect'}
			</button>
		</form>
	</div>
</div>
