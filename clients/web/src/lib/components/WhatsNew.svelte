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
			version: '0.18.12',
			date: '2026-02-17',
			changes: [
				'Fix: reconnect message reload now properly decrypts messages (was showing raw bytes)',
				'Fix: global search now correctly decrypts DM messages from non-active channels',
				'Fix: message delete now checks connection status (no more phantom deletions when offline)',
				'Fix: file size display now supports GB for large files',
				'Fix: download links now show "Download unavailable" on error instead of silently failing',
				'Fix: link preview cache now limited to 200 entries (prevents unbounded memory growth)',
				'Fix: channel switch no longer incorrectly auto-scrolls to bottom (respects saved scroll position)',
				'Fix: profile success messages now auto-dismiss after 4 seconds',
			]
		},
		{
			version: '0.18.11',
			date: '2026-02-17',
			changes: [
				'Edit messages now uses a multi-line textarea (preserves line breaks in multi-line messages)',
				'Community tabs (invites, bans, emoji) now always refresh when switching to them',
				'Community description field now enforces 2048-character limit client-side',
				'Confirm dialog input now properly resets when cancelled or dismissed',
			]
		},
		{
			version: '0.18.10',
			date: '2026-02-17',
			changes: [
				'Security: cross-group channel delete/update bypass fixed (now verifies channel belongs to the group)',
				'Security: cross-community emoji deletion fixed (now verifies emoji belongs to the community)',
				'Security: webhook execute endpoint now validates avatar_url (prevents XSS/SSRF)',
				'Security: poll remove_vote now checks channel membership (matches vote endpoint)',
				'Security: group image uploads now validate magic bytes (matches community upload security)',
				'Fix: poll duplicate vote on multi-select no longer causes server 500 error (graceful no-op)',
				'Fix: scheduled messages that fail delivery for >5 min are now dropped (prevents infinite retry)',
				'Fix: file uploads no longer duplicate messages (optimistic add now happens before WS send)',
				'Fix: editing state now cleared when another user deletes the message being edited',
				'Fix: kicked/banned users now fully removed from sidebar and voice calls (not just hidden)',
				'Fix: community invite creation now respects who_can_create_invites policy (not just admin-only)',
				'Fix: webhook create now trims name (matches update behavior)',
				'Voice: eliminated duplicate audio playback (per-user volume now works correctly)',
				'Voice: screen share audio no longer corrupted when changing mic device or noise suppression',
				'Voice: peers in transient "disconnected" state no longer prematurely torn down',
			]
		},
		{
			version: '0.18.9',
			date: '2026-02-17',
			changes: [
				'Fix: anonymous poll votes no longer incorrectly track voter IDs client-side (was causing UI inconsistency)',
				'Fix: typing indicator now cleared when switching channels (no more ghost "is typing..." on old channel)',
				'Fix: polls with passed expiry can no longer be voted on client-side (matches server-side rejection)',
				'Fix: Escape key now properly closes confirmation dialogs and report modals before other overlays',
				'Fix: confirmation dialog input now autofocuses and resets on cancel (no stale text on reopen)',
				'Fix: long filenames in file messages now truncate instead of overflowing the message bubble',
				'Fix: typing indicators from blocked users are now hidden',
				'Fix: username/email trimmed on registration (prevents whitespace-padded accounts)',
				'Fix: recovery code input normalized (uppercase, trim) for more forgiving entry',
				'Fix: file upload retry after token refresh no longer sends empty body',
				'Fix: recovery code modal can no longer be accidentally dismissed with Escape key',
				'Fix: login auto-shows 2FA field when server requires it (instead of showing error)',
				'Fix: voice background upload input now resets after upload (allows re-uploading same file)',
				'Security: recovery code validation tightened to exact 19-character format',
			]
		},
		{
			version: '0.18.8',
			date: '2026-02-17',
			changes: [
				'All native confirm/prompt dialogs replaced with styled themed modals (admin, settings, community, channel/group settings)',
				'Push-to-Talk and Toggle Mute voice activation modes with configurable keybinds',
				'In-call audio level meter on local video tile (green/yellow bar)',
				'Security: scheduled messages now check timeout, read-only, and archived status at delivery time',
				'Security: timed-out users can no longer edit messages',
				'Fix: PTT state now properly resets when leaving a call while holding the key',
				'Fix: DM message edits from yourself now decrypt correctly (was showing "[Failed to decrypt]")',
				'Fix: mention autocomplete now places cursor after the inserted @mention (not at end of input)',
				'Fix: video track properly released on camera enable failure (prevents stuck camera light)',
				'Fix: noise suppression destination node properly disconnected on cleanup',
			]
		},
		{
			version: '0.18.7',
			date: '2026-02-17',
			changes: [
				'Security: only the instance owner can now promote/demote admins (previously any admin could)',
				'Security: RemoveReaction now checks channel membership (prevents ex-members removing reactions)',
				'Security: join_channel now verifies community membership for group channels',
				'Security: ChannelUpdated broadcasts only to channel members, not all users',
				'Security: poll vote removal now checks expiry (matching vote addition)',
				'Security: bookmark creation now verifies channel membership',
				'Security: timeout/warning actions now verify target user is a channel member',
				'Security: voice session creation is now atomic (prevents duplicate sessions from race condition)',
				'Security: community image uploads now validate magic bytes match content type',
				'Security: TOTP secrets now encrypted with ChaCha20-Poly1305 (was XOR); existing secrets auto-migrate',
				'Security: rate limiter now uses real connection IP (proxy headers only trusted from known proxies)',
				'Security: webhook execute endpoint now rate-limited to 1 msg/sec per webhook',
				'Security: suspended users are now immediately blocked (no more JWT validity window)',
				'Fix: IMAGE_URL_REGEX /g flag caused alternating link preview failures',
				'Fix: file uploads now send to the correct channel even if you switch channels during upload',
				'Fix: feedback screenshot blob URL properly revoked on navigation',
				'Security: files without a channel are now only accessible to the uploader',
				'Security: create_channel now verifies community membership when assigning to a group',
				'Security: link preview SSRF protection now validates resolved IP after DNS (blocks DNS rebinding)',
				'Decryption failures now show "[Failed to decrypt]" instead of silently dropping messages',
				'Added error boundary: unhandled rendering errors show a friendly page with reload button',
				'Failed initial data load now shows an error state with retry button instead of empty UI',
				'Logout now fully clears all stores (messages, channels, groups, communities, members, bookmarks, read receipts)',
				'Message cache capped at 500 per channel to prevent unbounded memory growth',
				'Added maxlength limits to login, recover, and message inputs',
				'Performance: markdown rendering now cached (avoids re-parsing unchanged messages)',
				'Performance: mention autocomplete results properly memoized',
				'Fix: GIF picker no longer overflows viewport on narrow screens',
				'Escape key now closes feedback, welcome, and community modals',
				'Unknown WebSocket message types now logged for easier debugging',
			]
		},
		{
			version: '0.18.6',
			date: '2026-02-17',
			changes: [
				'Security: recovery code comparison now uses constant-time comparison (prevents timing attacks)',
				'Security: encryption failures now warn the user instead of silently falling back to plaintext',
				'Fix: loading older messages no longer mixes channels if you switch during scroll-load',
				'Fix: thread replies now show an error if the connection is lost instead of silently dropping',
				'Fix: WebSocket connect guard prevents duplicate connections during CLOSING/CONNECTING states',
				'Fix: unhandled async WebRTC calls now properly catch and log errors',
				'Register & recover pages: password visibility toggles, loading spinners, real-time confirm match feedback',
				'Accessibility: role="alert" on all error messages, aria-label on all modals, descriptive alt text on images',
				'SEO: added meta description, OG tags, fixed theme-color to match actual brand colors',
				'Settings: autocomplete hints on all password fields for better browser autofill',
			]
		},
		{
			version: '0.18.5',
			date: '2026-02-17',
			changes: [
				'Login: password visibility toggle, autofocus, loading spinner, focus rings',
				'Fix: logout now fully cleans up WebSocket, voice, preferences timers, and mark-read state',
				'Settings: all 14 toggle switches now have keyboard focus indicators',
				'Login: TOTP field is now required when 2FA section is open',
			]
		},
		{
			version: '0.18.4',
			date: '2026-02-17',
			changes: [
				'Keyboard shortcuts now show Cmd on Mac, Ctrl on other platforms',
				'Admin: password reset uses a proper modal form instead of browser prompt',
				'Fix: reconnect message reload guards against stale channel',
				'GIFs freeze when window loses focus, resume on return',
				'Fix: edit/reply state properly reset when switching channels',
			]
		},
		{
			version: '0.18.2',
			date: '2026-02-16',
			changes: [
				'GIF picker: auto-sends when clicked instead of pasting URL into input',
				'GIF picker: compact width, left-aligned near GIF button',
			]
		},
		{
			version: '0.18.1',
			date: '2026-02-16',
			changes: [
				'Fix: voice mesh race condition — join_voice now sent before subscribe to prevent premature peer connections during WebSocket reconnect',
				'Fix: server sends voice state directly to joining user, ensuring mesh setup even before channel subscription is active',
				'Fix: stale voice sessions auto-cleaned when server shows you as participant but you have no active call',
				'Fix: voice call properly leaves on page close/refresh instead of waiting for 15s grace period',
			]
		},
		{
			version: '0.18.0',
			date: '2026-02-16',
			changes: [
				'Push-to-Talk: three voice activation modes — Open Mic, Push to Talk (hold key to transmit), and Toggle Mute (press key to toggle)',
				'Configurable keybinds: set your PTT or toggle-mute key in Settings > Voice & Audio > Voice Activation',
				'In-call audio level meter: real-time input level bar on your local tile — works even when muted to show what your mic picks up',
				'PTT auto-mutes on join so you start silent, and re-mutes if you Alt+Tab while holding the key',
				'Keybind hint shown next to the mute button during calls (e.g., "Hold Space" or "M to mute")',
			]
		},
		{
			version: '0.17.1',
			date: '2026-02-16',
			changes: [
				'Fix: voice calls now automatically reconnect when WebSocket drops — dead peer connections are cleaned up and the mesh is re-established',
				'Fix: reconnecting users no longer trigger the join sound for others already in the call',
				'Server: voice cleanup grace period now uses timestamp-based logic to preserve sessions rejoined during reconnection',
			]
		},
		{
			version: '0.17.0',
			date: '2026-02-16',
			changes: [
				'Thread panel polish: messages now render with full markdown, syntax highlighting, and emoji support',
				'Thread panel: file attachments (images, videos, audio) display with previews and lightbox',
				'Thread panel: link previews and inline image URLs rendered like main chat',
				'Thread panel: hover actions — react, edit, and delete thread replies',
				'Thread panel: reaction pills with add/remove and full emoji picker',
				'Thread panel: edit messages inline with save/cancel and "(edited)" indicator',
				'Thread composer upgraded to multi-line textarea with auto-resize',
				'Fix: optimistic thread messages now properly confirm (no more stuck "sending...")',
				'Fix: thread messages update in real-time when edited or deleted by others',
				'Server: bookmarks and communities list endpoints now support pagination',
			]
		},
		{
			version: '0.16.0',
			date: '2026-02-16',
			changes: [
				'Read receipts: see when your messages are read in DMs (checkmark + "Read 5m ago") and who has read up to each message in channels (avatar stack)',
				'Read receipts opt-out: toggle "Send read receipts" in Settings > Chat to stop broadcasting your read position',
				'Threaded replies: click "Reply in Thread" on any message to start a focused conversation — thread panel opens on the right with its own composer',
				'Thread badges: messages with replies show a clickable "N replies — Last reply 5m ago" badge for easy thread discovery',
				'Thread icon added to message hover bar and right-click context menu for quick access',
			]
		},
		{
			version: '0.15.0',
			date: '2026-02-16',
			changes: [
				'User profiles: bio and pronouns now displayed on profile cards and editable in settings',
				'Saved Items: browse your bookmarked messages in a dedicated panel — click to jump, one-click remove',
				'Scheduled messages: pick a date & time to send a message later, manage pending messages in a panel',
				'Search filters: narrow results by sender, date range, and whether messages contain files',
				'Message edit history: click "(edited)" on any edited message to view all previous versions',
			]
		},
		{
			version: '0.14.1',
			date: '2026-02-16',
			changes: [
				'Account recovery: self-service password reset using recovery codes (no admin needed)',
				'Recovery codes shown at registration — save them to recover your account if you forget your password',
				'2FA backup codes: 8 single-use codes generated when enabling TOTP, usable as login fallback',
				'Regenerate recovery codes and backup codes from Settings > Security',
				'Privacy policy and terms of service pages with customizable defaults',
				'New /recover page for password reset using recovery code',
				'"Forgot password?" link on login page',
			]
		},
		{
			version: '0.14.0',
			date: '2026-02-16',
			changes: [
				'Security: typing indicator broadcasts throttled (3s cooldown) to prevent spam',
				'Security: reaction additions rate-limited (200ms cooldown) per connection',
				'Security: text field length validation on webhook names, admin reasons, and timeouts',
				'Keyboard: Home key scrolls to top of messages, Ctrl+T focuses message input',
			]
		},
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
