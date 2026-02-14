<script lang="ts">
	import { fade, scale } from 'svelte/transition';

	declare const __APP_VERSION__: string;

	const APP_VERSION = __APP_VERSION__;
	const STORAGE_KEY = 'chatalot-last-seen-version';

	interface ChangelogEntry {
		version: string;
		date: string;
		changes: string[];
	}

	const changelog: ChangelogEntry[] = [
		{
			version: '0.3.3',
			date: '2026-02-14',
			changes: [
				'Messages during a deploy no longer get stuck as greyed-out',
				'Update banner now appears immediately on reconnect',
			]
		},
		{
			version: '0.3.2',
			date: '2026-02-14',
			changes: [
				'Fixed phantom unread badges on Channels tab',
				'Unread counts now skip your own messages',
			]
		},
		{
			version: '0.3.1',
			date: '2026-02-14',
			changes: [
				'Fixed group isolation — new community members no longer auto-join all groups',
				'Fixed unread badge counts showing inflated numbers',
			]
		},
		{
			version: '0.3.0',
			date: '2026-02-14',
			changes: [
				'GIF search now powered by GIPHY',
				'Non-intrusive update banner — you choose when to refresh',
			]
		},
		{
			version: '0.2.0',
			date: '2026-02-14',
			changes: [
				'Auto-update: app reloads automatically when a new version is deployed',
				'Collapsible chat during voice calls (button next to Send)',
				'Per-user colored message bubbles',
				'Unread badges on Channels/DMs tabs',
				'Collapsible member panel',
				'What\'s New changelog on each update',
			]
		},
		{
			version: '0.1.0',
			date: '2026-02-12',
			changes: [
				'Voice calls now support 3+ participants (full mesh)',
				'Feedback form with screenshot support',
				'Instance owner role & community isolation',
				'PWA support for mobile install'
			]
		}
	];

	let show = $state(false);

	$effect(() => {
		const lastSeen = localStorage.getItem(STORAGE_KEY);
		if (lastSeen !== APP_VERSION) {
			show = true;
		}
	});

	function dismiss() {
		localStorage.setItem(STORAGE_KEY, APP_VERSION);
		show = false;
	}

	function handleBackdropKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') dismiss();
	}

	const currentEntry = changelog.find(e => e.version === APP_VERSION) ?? changelog[0];
</script>

{#if show}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
		transition:fade={{ duration: 150 }}
		onclick={dismiss}
		onkeydown={handleBackdropKeydown}
	>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="w-full max-w-md rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6 shadow-2xl"
			transition:scale={{ start: 0.95, duration: 200 }}
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
		>
			<div class="mb-4 flex items-center gap-3">
				<div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-[var(--accent)]/20">
					<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-[var(--accent)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
					</svg>
				</div>
				<div>
					<h2 class="text-lg font-bold text-[var(--text-primary)]">What's New in Chatalot</h2>
					<p class="text-sm text-[var(--text-secondary)]">Version {currentEntry.version}</p>
				</div>
			</div>

			<ul class="mb-6 space-y-2.5 pl-1">
				{#each currentEntry.changes as change}
					<li class="flex items-start gap-2.5 text-sm text-[var(--text-primary)]">
						<span class="mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full bg-[var(--accent)]"></span>
						<span>{change}</span>
					</li>
				{/each}
			</ul>

			<div class="flex justify-end">
				<button
					onclick={dismiss}
					class="rounded-lg bg-[var(--accent)] px-5 py-2 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)]"
				>
					Got it
				</button>
			</div>
		</div>
	</div>
{/if}
