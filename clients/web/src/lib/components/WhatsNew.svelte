<script lang="ts" module>
	declare const __APP_VERSION__: string;
</script>

<script lang="ts">
	import { fade, scale } from 'svelte/transition';

	const APP_VERSION = __APP_VERSION__;
	const STORAGE_KEY = 'chatalot-last-seen-version';

	interface ChangelogEntry {
		version: string;
		date: string;
		changes: string[];
	}

	const changelog: ChangelogEntry[] = [
		{
			version: '0.13.9',
			date: '2026-02-16',
			changes: [
				'Real-time: profile changes (avatar, name, status) now broadcast instantly to all connected users',
				'Real-time: channel settings, group settings, and community settings updates broadcast live',
				'Real-time: channel and group deletions reflected immediately without page reload',
				'UX: confirmation dialogs on invite and emoji deletion, disabled button styling',
				'Fix: admin page search timer cleanup on navigation, audio device error logging',
			]
		},
		{
			version: '0.13.8',
			date: '2026-02-16',
			changes: [
				'Accessibility: aria-labels on 25+ icon-only buttons, color inputs, and mod action buttons',
				'Accessibility: mobile member panel overlay now handles Escape key and has proper ARIA role',
				'UX: DM user search and GIF picker now show error messages on failure instead of empty results',
				'Server: invite expiration hours validated to 1-8760 range on community and admin endpoints',
				'Performance: batch group auto-join, optimized unread count query, community members index',
			]
		},
		{
			version: '0.13.7',
			date: '2026-02-16',
			changes: [
				'Error feedback: loading failures for messages, pins, and search now show error state with retry',
				'Performance: batch SQL operations for channel member inserts, sender key cleanup, and hash blocking',
				'WebSocket: idle broadcast channels cleaned up every 5 minutes to prevent memory growth',
				'WebSocket: malformed messages now return an error instead of being silently ignored',
				'Security: eliminated format!-based SQL construction in file listing',
				'Resilience: broken community/group icon images gracefully hidden instead of showing broken image',
			]
		},
		{
			version: '0.13.6',
			date: '2026-02-16',
			changes: [
				'Security: custom emoji URLs now validated to prevent XSS via data URI schemes',
				'Server hardening: invite code length limits, consistent error codes for expired invites, ban reason length cap',
				'Memory optimization: member and typing data now properly cleaned up when leaving channels',
				'UI: improved empty state contrast for groups, channels, DMs, and GIF picker',
				'UX: hover tooltips on all truncated text (group names, DM names, file names, poll options)',
				'UI consistency: skeleton loading backgrounds now use theme variables',
			]
		},
		{
			version: '0.13.5',
			date: '2026-02-16',
			changes: [
				'Security: invite deletion now validates ownership to prevent cross-group/community abuse',
				'Security: moderation actions (timeout/warn) now verify channel belongs to the community',
				'Performance: shared HTTP connection pool for server-side outbound requests (GIFs, link previews, feedback)',
				'Reliability: concurrent token refresh requests are now deduplicated to prevent auth race conditions',
				'Refactoring: centralized file upload helper (-130 lines of duplicated code across 5 API modules)',
			]
		},
		{
			version: '0.13.4',
			date: '2026-02-16',
			changes: [
				'Clipboard error handling: all copy actions now show error toasts instead of failing silently',
				'Accessibility: aria-labels on all icon-only buttons (feedback, sign out, scroll, video focus, close)',
				'Performance: DM list loading optimized from N+1 queries to batch fetch',
				'Performance: added database indexes for group invites, DM pairs, scheduled messages',
				'Race condition fix: channel member loading now discards stale results on fast switching',
				'Server refactoring: deduplicated message response building code',
			]
		},
		{
			version: '0.13.3',
			date: '2026-02-16',
			changes: [
				'Mobile responsive: settings, community, admin, login, and register pages now adapt to small screens',
				'Mobile tab bars: horizontal scrollable tabs replace sidebars on mobile devices',
				'WebSocket hardening: 1MB message size limit, subscription caps, better auth error messages',
				'Server validation: ban reason, welcome message, invite expiration, accent color format checks',
				'Modal safe areas: all dialogs now have proper padding on small screens',
				'Context menu: max-width capped to viewport to prevent overflow',
			]
		},
		{
			version: '0.13.2',
			date: '2026-02-16',
			changes: [
				'Accessibility: fixed 40 a11y warnings — proper keyboard navigation, focus states, aria labels',
				'Code quality: replaced all empty error catches with console.warn for easier debugging',
				'Removed all "as any" type assertions with proper TypeScript types',
				'Reactivity fixes: blocked status and edit forms now sync correctly when props change',
				'Formatting toolbar buttons now show focus indicators for keyboard users',
				'Character counters on display name and custom status inputs',
				'Escape key now closes channel/group creation forms',
				'Server hardening: pagination bounds, header injection prevention',
			]
		},
		{
			version: '0.13.1',
			date: '2026-02-16',
			changes: [
				'Mark All Read: button in sidebar + Shift+Esc shortcut to clear all unread badges at once',
				'Confirmation dialogs for destructive actions: session revocation, voice kick now require confirmation',
				'Loading states on session revoke and revoke-all buttons to prevent double-clicks',
				'Improved error messages: rate limit feedback with retry timer, better API error parsing',
				'Fixed broken toast notifications for WebSocket errors and timeouts',
				'Fixed channel load race condition when switching channels quickly',
				'Fixed 27 TypeScript type errors across the codebase',
			]
		},
		{
			version: '0.13.0',
			date: '2026-02-16',
			changes: [
				'Announcements: server-wide banners from admins with per-user dismissal',
				'Custom emoji: upload community emoji, use with :shortcode: in messages, autocomplete in composer',
				'Report messages: right-click any message to report it to admins',
				'Warning system: moderators can issue warnings to users in channels',
				'Auto-idle: status automatically set to idle after 5 minutes of inactivity',
				'Message "sending..." indicator for pending messages',
				'Copy message link from context menu',
				'Smooth scroll-to-bottom animation',
				'Empty channel state with friendly placeholder',
			]
		},
		{
			version: '0.12.0',
			date: '2026-02-16',
			changes: [
				'Polls: create polls in any channel with 2-10 options, multi-select, anonymous voting, and optional expiry',
				'Real-time poll updates: votes and poll closures broadcast instantly via WebSocket',
				'Webhook management: create, toggle, and delete webhooks from channel settings (admin only)',
				'Copy webhook URL with one click for easy integration with external services',
			]
		},
		{
			version: '0.11.0',
			date: '2026-02-15',
			changes: [
				'Profile banners: upload a banner image displayed on your profile card',
				'Community theming: admins can set accent colors, backgrounds, text colors, and custom CSS for their community',
				'Community welcome message: shown as a splash screen when members first visit',
				'Community icon and banner uploads with live preview',
				'Group customization: icon, banner, and accent color per group',
				'Voice call backgrounds: choose from 6 animated presets (Fireplace, Aurora, Rain, Sunset, Space, Cozy) or set a solid color, gradient, or custom image',
				'Per-channel voice ambiance: admins can set a shared background for voice channels',
				'Server-side CSS sanitizer with allowlisted properties to prevent XSS in community themes',
			]
		},
		{
			version: '0.10.0',
			date: '2026-02-14',
			changes: [
				'Group visibility: set groups to public or private (private groups require invite)',
				'Channel read-only mode: lock channels so only admins can post',
				'Channel slow mode: rate-limit messages (5s to 10m intervals)',
				'Community policies: configure who can create groups and invites',
				'Right-click groups and channels for settings popover',
				'Role permissions reference table in community settings',
				'Fixed: any community member could join any group (now respects visibility)',
			]
		},
		{
			version: '0.9.0',
			date: '2026-02-14',
			changes: [
				'Mobile-friendly layout: sidebar, header, and input area adapt to screen size',
				'Community rail hidden on mobile — compact horizontal switcher in sidebar instead',
				'Member panel slides in as overlay on mobile, stays as sidebar on desktop',
				'Responsive images, videos, and link previews scale to fit small screens',
				'Send button shows arrow icon on small screens, text on larger screens',
				'Context menu clamped to viewport so it never clips off-screen',
				'Touch-friendly tap targets throughout the UI',
			]
		},
		{
			version: '0.8.0',
			date: '2026-02-14',
			changes: [
				'Preset themes: Monokai, Dracula, Nord, Solarized, AMOLED, and Catppuccin color palettes',
				'Custom theme creator: pick your own colors for backgrounds, text, and accents',
				'Chat bubble style: switch between flat (Discord-style) and rounded bubbles (iMessage-style)',
				'Relative timestamps: show "5m ago" instead of exact times (toggle in Settings > Appearance)',
				'Animated accent: subtle color shift effect on accent elements',
				'Reduce motion: disable all animations for accessibility',
				'Invite links now use the server\'s public URL instead of local hostname',
			]
		},
		{
			version: '0.7.0',
			date: '2026-02-14',
			changes: [
				'Invite links: share clickable URLs instead of codes, paste links or codes in the join dialog',
				'Longer invite codes (12 chars) for better security',
				'Cross-tab logout detection — logging out in one tab logs out all tabs',
				'Fixed WebSocket subscription task cleanup on disconnect',
				'Message size validation (64 KiB limit)',
				'Server now reports errors on voice join failure',
				'Connection pool increased to 50 connections',
				'Zero accessibility warnings in build',
			]
		},
		{
			version: '0.6.0',
			date: '2026-02-14',
			changes: [
				'Tiling layout: stream takes main pane with participant tiles stacked alongside',
				'Focus mode: hide participant tiles to watch a stream full-width (button in top-right)',
				'Auto-focus setting: automatically enter focus mode when someone shares their screen',
			]
		},
		{
			version: '0.5.9',
			date: '2026-02-14',
			changes: [
				'Fixed screen shares not showing when joining a channel where someone is already streaming',
			]
		},
		{
			version: '0.5.8',
			date: '2026-02-14',
			changes: [
				'Screen share auto-captures system audio via PipeWire/PulseAudio monitor device on Linux',
				'Falls back gracefully if no monitor device found — shows "No audio" badge',
				'Right-click screen share for volume/mute controls',
			]
		},
		{
			version: '0.5.3',
			date: '2026-02-14',
			changes: [
				'Volume amplification now goes up to 500% using real audio processing',
				'Screen sharing now transmits audio along with video',
				'Fixed favicon 404 error in Chrome',
			]
		},
		{
			version: '0.5.2',
			date: '2026-02-14',
			changes: [
				'Right-click voice participants in sidebar for volume control and kick',
				'Scroll position preserved when switching between channels',
				'Voice calls now auto-rejoin after server restarts',
				'Fixed service worker serving stale code after deploys',
				'Fixed Chrome caching causing 404 errors on updates',
			]
		},
		{
			version: '0.5.1',
			date: '2026-02-14',
			changes: [
				'Fixed video cameras not showing for either party when toggled on',
				'Fixed audio pipeline rebuilds (device switch, noise suppression change) breaking video',
				'Mute state now preserved when switching audio devices or noise suppression',
			]
		},
		{
			version: '0.5.0',
			date: '2026-02-14',
			changes: [
				'Full voice settings: input/output device selection, mic test with level meter',
				'Mic volume control — right-click your own tile to adjust what others hear',
				'Master output volume and per-user volume sliders',
				'Advanced audio toggles: echo cancellation, auto gain control',
			]
		},
		{
			version: '0.4.2',
			date: '2026-02-14',
			changes: [
				'Participant name badges moved to top-right of video tiles',
				'Message input bar now has a solid background',
				'Fixed phantom unread badges sticking after selecting a channel',
			]
		},
		{
			version: '0.4.0',
			date: '2026-02-14',
			changes: [
				'Right-click a participant in voice to adjust their volume',
				'Fixed "Encrypted message" showing in group channels',
				'Voice controls persist in sidebar when browsing other channels',
			]
		},
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
	let showAll = $state(false);

	$effect(() => {
		const lastSeen = localStorage.getItem(STORAGE_KEY);
		if (lastSeen !== APP_VERSION) {
			show = true;
		}
	});

	export function open() {
		showAll = true;
		show = true;
	}

	function dismiss() {
		localStorage.setItem(STORAGE_KEY, APP_VERSION);
		show = false;
		showAll = false;
	}

	function handleBackdropKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') dismiss();
	}

	const currentEntry = changelog.find(e => e.version === APP_VERSION) ?? changelog[0];
</script>

{#if show}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-[200] flex items-center justify-center bg-black/60 p-4"
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

			{#if showAll}
				<div class="mb-6 max-h-[60vh] space-y-4 overflow-y-auto pr-1">
					{#each changelog as entry}
						<div>
							<div class="mb-1.5 flex items-baseline gap-2">
								<span class="text-sm font-semibold text-[var(--text-primary)]">v{entry.version}</span>
								<span class="text-xs text-[var(--text-secondary)]">{entry.date}</span>
							</div>
							<ul class="space-y-1.5 pl-1">
								{#each entry.changes as change}
									<li class="flex items-start gap-2.5 text-sm text-[var(--text-primary)]">
										<span class="mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full bg-[var(--accent)]"></span>
										<span>{change}</span>
									</li>
								{/each}
							</ul>
						</div>
					{/each}
				</div>
			{:else}
				<ul class="mb-6 space-y-2.5 pl-1">
					{#each currentEntry.changes as change}
						<li class="flex items-start gap-2.5 text-sm text-[var(--text-primary)]">
							<span class="mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full bg-[var(--accent)]"></span>
							<span>{change}</span>
						</li>
					{/each}
				</ul>
			{/if}

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
