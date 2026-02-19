<script lang="ts">
	import { voiceStore } from '$lib/stores/voice.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { userStore } from '$lib/stores/users.svelte';
	import { preferencesStore, voiceBackgroundStyle } from '$lib/stores/preferences.svelte';
	import { webrtcManager } from '$lib/webrtc/manager';
	import Avatar from '$lib/components/Avatar.svelte';

	let { height = null, canKick = false, onKickFromVoice, channelVoiceBackground = null }: {
		height?: number | null;
		canKick?: boolean;
		onKickFromVoice?: (userId: string) => void;
		channelVoiceBackground?: string | null;
	} = $props();

	let expanded = $derived(height == null);

	let localVideoEl: HTMLVideoElement | undefined = $state();
	let screenVideoEl: HTMLVideoElement | undefined = $state();
	let remoteVideoEls = $state<Map<string, HTMLVideoElement>>(new Map());
	let remoteScreenEls = $state<Map<string, HTMLVideoElement>>(new Map());

	// Hidden audio elements for screen share audio (separate from muted video)
	let screenAudioEls = $state<Map<string, HTMLAudioElement>>(new Map());

	// Focus mode: hide participant tiles, show only stream
	let focusStream = $state(false);

	// Attach local stream to video element
	$effect(() => {
		if (localVideoEl && voiceStore.activeCall?.localStream) {
			localVideoEl.srcObject = voiceStore.activeCall.localStream;
		}
	});

	// Attach screen share stream
	$effect(() => {
		if (screenVideoEl && voiceStore.activeCall?.screenStream) {
			screenVideoEl.srcObject = voiceStore.activeCall.screenStream;
		}
	});

	// Reactively attach remote streams to video elements whenever either changes
	$effect(() => {
		for (const [userId, stream] of voiceStore.remoteStreams) {
			const el = remoteVideoEls.get(userId);
			if (el && el.srcObject !== stream) {
				el.srcObject = stream;
			}
		}
	});

	// Attach remote screen share streams to video elements
	$effect(() => {
		for (const [userId, stream] of voiceStore.remoteScreenStreams) {
			const el = remoteScreenEls.get(userId);
			if (el && el.srcObject !== stream) {
				el.srcObject = stream;
			}
		}
	});

	// Attach remote screen share streams to audio elements and apply volume/mute
	$effect(() => {
		for (const [userId, stream] of voiceStore.remoteScreenStreams) {
			const el = screenAudioEls.get(userId);
			if (el) {
				if (el.srcObject !== stream) {
					el.srcObject = stream;
				}
				el.muted = voiceStore.isScreenShareMuted(userId);
				el.volume = voiceStore.getScreenShareVolume(userId) / 100;
			}
		}
	});

	// Auto-enter focus mode when preference is on and a screen share appears
	$effect(() => {
		if (hasAnyScreenShare && preferencesStore.preferences.autoHideParticipantsOnStream) {
			focusStream = true;
		}
		if (!hasAnyScreenShare) {
			focusStream = false;
		}
	});

	function bindRemoteVideo(node: HTMLVideoElement, userId: string) {
		const next = new Map(remoteVideoEls);
		next.set(userId, node);
		remoteVideoEls = next;

		// Try to attach stream immediately
		const stream = voiceStore.remoteStreams.get(userId);
		if (stream) node.srcObject = stream;

		return {
			destroy() {
				const next = new Map(remoteVideoEls);
				next.delete(userId);
				remoteVideoEls = next;
			}
		};
	}

	function bindRemoteScreen(node: HTMLVideoElement, userId: string) {
		const next = new Map(remoteScreenEls);
		next.set(userId, node);
		remoteScreenEls = next;

		const stream = voiceStore.remoteScreenStreams.get(userId);
		if (stream) node.srcObject = stream;

		return {
			destroy() {
				const next = new Map(remoteScreenEls);
				next.delete(userId);
				remoteScreenEls = next;
			}
		};
	}

	function bindScreenAudio(node: HTMLAudioElement, userId: string) {
		const next = new Map(screenAudioEls);
		next.set(userId, node);
		screenAudioEls = next;

		const stream = voiceStore.remoteScreenStreams.get(userId);
		if (stream) {
			node.srcObject = stream;
			node.muted = voiceStore.isScreenShareMuted(userId);
			node.volume = voiceStore.getScreenShareVolume(userId) / 100;
		}

		return {
			destroy() {
				const next = new Map(screenAudioEls);
				next.delete(userId);
				screenAudioEls = next;
			}
		};
	}

	// Check which remote screen shares have audio tracks
	let screenShareHasAudio = $derived(() => {
		const result = new Map<string, boolean>();
		for (const [userId, stream] of voiceStore.remoteScreenStreams) {
			result.set(userId, stream.getAudioTracks().length > 0);
		}
		return result;
	});

	let hasVideo = $derived(voiceStore.activeCall?.videoEnabled ?? false);
	let remoteEntries = $derived([...voiceStore.remoteStreams]);
	let remoteScreenEntries = $derived([...voiceStore.remoteScreenStreams]);
	let hasAnyScreenShare = $derived(voiceStore.activeCall?.screenSharing || remoteScreenEntries.length > 0);
	let totalParticipants = $derived(1 + remoteEntries.length);
	let gridCols = $derived(
		totalParticipants <= 1 ? 'grid-cols-1' :
		totalParticipants <= 4 ? 'grid-cols-1 sm:grid-cols-2' :
		totalParticipants <= 9 ? 'grid-cols-2 sm:grid-cols-3' :
		totalParticipants <= 16 ? 'grid-cols-3 sm:grid-cols-4' :
		'grid-cols-4 sm:grid-cols-5'
	);

	// All participant IDs (for focus mode avatar strip)
	let allParticipantIds = $derived([
		authStore.user?.id ?? '',
		...remoteEntries.map(([id]) => id)
	]);

	// Voice background for local tile
	let localBgStyle = $derived(voiceBackgroundStyle(preferencesStore.preferences.voiceBackground));

	// Channel ambiance background for the grid container (sanitized to prevent CSS injection)
	let channelAmbianceStyle = $derived.by(() => {
		if (!channelVoiceBackground) return '';
		// Block characters that can break out of url("...") or inject CSS
		if (/[;'"\\(){}]/.test(channelVoiceBackground)) return '';
		return `background: url("${channelVoiceBackground}") center/cover no-repeat;`;
	});

	// Compute CSS background style for a remote user's voice tile
	function remoteVoiceBgStyle(userId: string): string {
		const user = userStore.getUser(userId);
		const url = user?.voice_background_url;
		if (!url) return '';
		// Block characters that can break out of url("...") or inject CSS
		if (/[;'"\\(){}]/.test(url)) return '';
		return `background: url("${url}") center/cover no-repeat;`;
	}

	// Context menu state
	let menuUserId = $state<string | null>(null);
	let menuPos = $state({ x: 0, y: 0 });
	let menuType = $state<'self' | 'remote' | 'screen'>('remote');

	function openVolumeMenu(e: MouseEvent, userId: string) {
		e.preventDefault();
		menuUserId = userId;
		menuPos = { x: e.clientX, y: e.clientY };
		menuType = userId === authStore.user?.id ? 'self' : 'remote';
	}

	function openScreenMenu(e: MouseEvent, userId: string) {
		e.preventDefault();
		menuUserId = userId;
		menuPos = { x: e.clientX, y: e.clientY };
		menuType = 'screen';
	}

	function closeMenu() {
		menuUserId = null;
	}
</script>

{#snippet audioLevelMeter()}
	<div class="absolute bottom-0 left-0 right-0 h-1 overflow-hidden rounded-b-lg bg-black/20 pointer-events-none">
		<div
			class="h-full transition-[width] duration-75 {voiceStore.localAudioLevel > 70 ? 'bg-yellow-400/70' : 'bg-green-400/70'}"
			style="width: {voiceStore.localAudioLevel}%"
		></div>
	</div>
{/snippet}

{#if voiceStore.isInCall}
	<div class="{expanded ? 'flex-1 min-h-0 flex flex-col' : 'shrink-0'} overflow-hidden"
		style="{!expanded && height != null ? `height: ${height}px;` : ''} {channelAmbianceStyle || 'background: var(--bg-secondary);'}">

		{#if hasAnyScreenShare && !focusStream}
			<!-- ═══ TILED MODE: Stream as master pane + participant tiles stacked on right ═══ -->
			<div class="relative flex h-full gap-1 overflow-hidden p-2 {expanded ? 'flex-1 min-h-0' : ''}">
				<!-- Master pane: screen shares -->
				<div class="flex min-w-0 flex-1 flex-col gap-1 min-h-0">
					{#if voiceStore.activeCall?.screenSharing}
						<div class="relative min-h-0 flex-1 overflow-hidden">
							<video
								bind:this={screenVideoEl}
								autoplay
								muted
								playsinline
								class="h-full w-full rounded-lg object-contain"
							></video>
							<div class="absolute top-2 left-2 flex items-center gap-1.5 rounded bg-red-500/90 px-2 py-1 text-xs font-medium text-white">
								<span class="h-2 w-2 rounded-full bg-white animate-pulse"></span>
								You are sharing your screen
							</div>
							{#if voiceStore.activeCall?.screenStream && voiceStore.activeCall.screenStream.getAudioTracks().length === 0}
								<div class="absolute top-2 right-2 rounded bg-black/60 px-2 py-1 text-xs text-yellow-300">
									No audio — share a tab for sound
								</div>
							{/if}
						</div>
					{/if}

					{#each remoteScreenEntries as [userId, _stream] (userId)}
						<div class="relative min-h-0 flex-1 overflow-hidden">
							<video
								autoplay
								muted
								playsinline
								class="h-full w-full rounded-lg object-contain"
								use:bindRemoteScreen={userId}
							></video>
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div
								class="absolute inset-0 rounded-lg"
								oncontextmenu={(e) => openScreenMenu(e, userId)}
							></div>
							<div class="absolute top-2 left-2 flex items-center gap-1.5 rounded bg-[var(--accent)]/90 px-2 py-1 text-xs font-medium text-white pointer-events-none">
								<span class="h-2 w-2 rounded-full bg-white animate-pulse"></span>
								{userStore.getDisplayName(userId)} is sharing their screen
							</div>
							{#if !screenShareHasAudio().get(userId)}
								<div class="absolute top-2 right-2 rounded bg-black/60 px-2 py-1 text-xs text-yellow-300 pointer-events-none">
									No audio
								</div>
							{:else if voiceStore.isScreenShareMuted(userId)}
								<div class="absolute top-2 right-2 rounded bg-black/60 px-2 py-1 text-xs text-white pointer-events-none">
									Audio muted
								</div>
							{:else if voiceStore.getScreenShareVolume(userId) !== 100}
								<div class="absolute top-2 right-2 rounded bg-black/60 px-2 py-1 text-xs text-white pointer-events-none">
									{voiceStore.getScreenShareVolume(userId)}%
								</div>
							{/if}
							<!-- svelte-ignore element_invalid_self_closing_tag -->
							<audio autoplay use:bindScreenAudio={userId} class="hidden"></audio>
						</div>
					{/each}
				</div>

				<!-- Participant tiles stacked on the right -->
				<div class="flex w-44 shrink-0 flex-col gap-1 overflow-y-auto">
					<!-- Local tile -->
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div
						class="relative flex items-center justify-center rounded-lg overflow-hidden transition-shadow duration-200 {voiceStore.isSpeaking(authStore.user?.id ?? '') ? 'ring-2 ring-[var(--success)] shadow-[0_0_8px_var(--success)]' : ''}"
						style="aspect-ratio: 16/9; {!hasVideo && localBgStyle ? localBgStyle : 'background: var(--bg-tertiary);'}"
						oncontextmenu={(e) => openVolumeMenu(e, authStore.user?.id ?? '')}
					>
						{#if hasVideo}
							<!-- svelte-ignore element_invalid_self_closing_tag -->
							<video
								bind:this={localVideoEl}
								autoplay
								muted
								playsinline
								class="h-full w-full object-cover"
							></video>
						{:else}
							{#if authStore.user}
								<Avatar userId={authStore.user.id} size="md" />
							{/if}
						{/if}
						<div class="absolute top-0.5 right-0.5 rounded bg-black/60 px-1.5 py-0.5 text-[10px] text-white">
							You {voiceStore.activeCall?.audioEnabled ? '' : '(muted)'}
						</div>
						{#if voiceStore.activeCall?.screenSharing}
							<div class="absolute top-0.5 left-0.5 flex items-center gap-0.5 rounded bg-red-500/80 px-1 py-0.5 text-[9px] text-white">
								<span class="h-1.5 w-1.5 rounded-full bg-white animate-pulse"></span>
								LIVE
							</div>
						{/if}
						{@render audioLevelMeter()}
					</div>

					<!-- Remote participant tiles -->
					{#each remoteEntries as [userId, _stream] (userId)}
						{@const remoteBg = remoteVoiceBgStyle(userId)}
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div
							class="relative flex items-center justify-center rounded-lg overflow-hidden transition-shadow duration-200 {voiceStore.isSpeaking(userId) ? 'ring-2 ring-[var(--success)] shadow-[0_0_8px_var(--success)]' : ''}"
							style="aspect-ratio: 16/9; {remoteBg || 'background: var(--bg-tertiary);'}"
							oncontextmenu={(e) => openVolumeMenu(e, userId)}
						>
							{#if !voiceStore.hasRemoteVideo(userId)}
								<Avatar {userId} size="md" />
							{/if}
							<video
								autoplay
								muted
								playsinline
								class="{voiceStore.hasRemoteVideo(userId) ? 'h-full w-full object-cover' : 'absolute h-0 w-0 opacity-0'}"
								use:bindRemoteVideo={userId}
							></video>
							<div class="absolute top-0.5 right-0.5 rounded bg-black/60 px-1.5 py-0.5 text-[10px] text-white">
								{userStore.getDisplayName(userId)}
								{#if voiceStore.getUserVolume(userId) !== 100}
									<span class="ml-0.5 opacity-70">{voiceStore.getUserVolume(userId)}%</span>
								{/if}
							</div>
						</div>
					{/each}
				</div>

				<!-- Focus toggle button -->
				<button
					onclick={() => focusStream = true}
					class="absolute top-3 right-3 z-10 rounded-lg bg-black/50 p-1.5 text-white/70 transition hover:bg-black/70 hover:text-white"
					title="Focus on stream (hide participants)"
					aria-label="Focus on stream"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<path d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3" />
					</svg>
				</button>
			</div>

		{:else if hasAnyScreenShare && focusStream}
			<!-- ═══ FOCUS MODE: Stream full-width, participant avatars in strip below ═══ -->
			<div class="relative flex h-full flex-col {expanded ? 'flex-1' : ''} p-2">
				<!-- Screen shares -->
				<div class="flex-1">
					{#if voiceStore.activeCall?.screenSharing}
						<div class="relative">
							<video
								bind:this={screenVideoEl}
								autoplay
								muted
								playsinline
								class="w-full rounded-lg"
								style="{expanded ? '' : 'max-height: 70vh;'}"
							></video>
							<div class="absolute top-2 left-2 flex items-center gap-1.5 rounded bg-red-500/90 px-2 py-1 text-xs font-medium text-white">
								<span class="h-2 w-2 rounded-full bg-white animate-pulse"></span>
								You are sharing your screen
							</div>
							{#if voiceStore.activeCall?.screenStream && voiceStore.activeCall.screenStream.getAudioTracks().length === 0}
								<div class="absolute top-2 right-2 rounded bg-black/60 px-2 py-1 text-xs text-yellow-300">
									No audio — share a tab for sound
								</div>
							{/if}
						</div>
					{/if}

					{#each remoteScreenEntries as [userId, _stream] (userId)}
						<div class="relative mt-1">
							<video
								autoplay
								muted
								playsinline
								class="w-full rounded-lg"
								style="{expanded ? '' : 'max-height: 70vh;'}"
								use:bindRemoteScreen={userId}
							></video>
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div
								class="absolute inset-0 rounded-lg"
								oncontextmenu={(e) => openScreenMenu(e, userId)}
							></div>
							<div class="absolute top-2 left-2 flex items-center gap-1.5 rounded bg-[var(--accent)]/90 px-2 py-1 text-xs font-medium text-white pointer-events-none">
								<span class="h-2 w-2 rounded-full bg-white animate-pulse"></span>
								{userStore.getDisplayName(userId)} is sharing their screen
							</div>
							{#if !screenShareHasAudio().get(userId)}
								<div class="absolute top-2 right-2 rounded bg-black/60 px-2 py-1 text-xs text-yellow-300 pointer-events-none">
									No audio
								</div>
							{:else if voiceStore.isScreenShareMuted(userId)}
								<div class="absolute top-2 right-2 rounded bg-black/60 px-2 py-1 text-xs text-white pointer-events-none">
									Audio muted
								</div>
							{:else if voiceStore.getScreenShareVolume(userId) !== 100}
								<div class="absolute top-2 right-2 rounded bg-black/60 px-2 py-1 text-xs text-white pointer-events-none">
									{voiceStore.getScreenShareVolume(userId)}%
								</div>
							{/if}
							<!-- svelte-ignore element_invalid_self_closing_tag -->
							<audio autoplay use:bindScreenAudio={userId} class="hidden"></audio>
						</div>
					{/each}
				</div>

				<!-- Participant avatar strip -->
				<div class="mt-1 flex items-center gap-2 rounded-lg bg-[var(--bg-tertiary)]/80 px-3 py-1.5">
					{#each allParticipantIds as userId (userId)}
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div
							class="relative shrink-0"
							oncontextmenu={(e) => openVolumeMenu(e, userId)}
							title={userId === authStore.user?.id ? 'You' : userStore.getDisplayName(userId)}
						>
							<div class="h-7 w-7 rounded-full overflow-hidden transition-shadow {voiceStore.isSpeaking(userId) ? 'ring-2 ring-[var(--success)] shadow-[0_0_6px_var(--success)]' : ''}">
								<Avatar {userId} size="sm" />
							</div>
							{#if userId === authStore.user?.id && !voiceStore.activeCall?.audioEnabled}
								<div class="absolute -bottom-0.5 -right-0.5 h-3 w-3 rounded-full bg-[var(--bg-secondary)] flex items-center justify-center">
									<svg xmlns="http://www.w3.org/2000/svg" class="h-2 w-2 text-[var(--danger)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
										<line x1="1" y1="1" x2="23" y2="23" /><path d="M9 9v3a3 3 0 0 0 5.12 2.12" />
									</svg>
								</div>
							{/if}
						</div>
					{/each}
					<div class="ml-auto text-[10px] text-[var(--text-secondary)]">
						{totalParticipants} in call
					</div>
				</div>

				<!-- Unfocus toggle button -->
				<button
					onclick={() => focusStream = false}
					class="absolute top-3 right-3 z-10 rounded-lg bg-black/50 p-1.5 text-white/70 transition hover:bg-black/70 hover:text-white"
					title="Show participants (tiled view)"
					aria-label="Show participants"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<rect x="3" y="3" width="7" height="7" /><rect x="14" y="3" width="7" height="7" /><rect x="14" y="14" width="7" height="7" /><rect x="3" y="14" width="7" height="7" />
					</svg>
				</button>
			</div>

			<!-- Hidden video elements to keep remote streams attached (audio handled by PersistentAudio) -->
			{#each remoteEntries as [userId, _stream] (userId)}
				<video
					autoplay
					muted
					playsinline
					class="absolute h-0 w-0 opacity-0"
					use:bindRemoteVideo={userId}
				></video>
			{/each}

		{:else}
			<!-- ═══ STANDARD MODE: No screen share — normal participant grid ═══ -->
			<div class="grid {gridCols} gap-1 overflow-hidden p-2 {expanded ? 'flex-1' : 'h-full'}" style="grid-auto-rows: 1fr;">
				<!-- Local video/avatar -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					class="relative flex min-h-0 items-center justify-center rounded-lg overflow-hidden transition-shadow duration-200 {voiceStore.isSpeaking(authStore.user?.id ?? '') ? 'ring-2 ring-[var(--success)] shadow-[0_0_8px_var(--success)]' : ''}"
					style="{!hasVideo && localBgStyle ? localBgStyle : 'background: var(--bg-tertiary);'}"
					oncontextmenu={(e) => openVolumeMenu(e, authStore.user?.id ?? '')}
				>
					{#if hasVideo}
						<!-- svelte-ignore element_invalid_self_closing_tag -->
						<video
							bind:this={localVideoEl}
							autoplay
							muted
							playsinline
							class="h-full w-full object-cover"
						></video>
					{:else}
						{#if authStore.user}
							<Avatar userId={authStore.user.id} size="lg" />
						{/if}
					{/if}
					<div class="absolute top-1 right-1 rounded bg-black/60 px-2 py-0.5 text-xs text-white">
						You {voiceStore.activeCall?.audioEnabled ? '' : '(muted)'}
						{#if preferencesStore.preferences.inputGain !== 100}
							<span class="ml-1 opacity-70">{preferencesStore.preferences.inputGain}%</span>
						{/if}
					</div>
					{@render audioLevelMeter()}
				</div>

				<!-- Remote participants -->
				{#each remoteEntries as [userId, _stream] (userId)}
					{@const remoteBg = remoteVoiceBgStyle(userId)}
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div
						class="relative flex min-h-0 items-center justify-center rounded-lg overflow-hidden transition-shadow duration-200 {voiceStore.isSpeaking(userId) ? 'ring-2 ring-[var(--success)] shadow-[0_0_8px_var(--success)]' : ''}"
						style="{remoteBg || 'background: var(--bg-tertiary);'}"
						oncontextmenu={(e) => openVolumeMenu(e, userId)}
					>
						{#if !voiceStore.hasRemoteVideo(userId)}
							<Avatar {userId} size="lg" />
						{/if}
						<video
							autoplay
							muted
							playsinline
							class="{voiceStore.hasRemoteVideo(userId) ? 'h-full w-full object-cover' : 'absolute h-0 w-0 opacity-0'}"
							use:bindRemoteVideo={userId}
						></video>
						<div class="absolute top-1 right-1 rounded bg-black/60 px-2 py-0.5 text-xs text-white">
							{userStore.getDisplayName(userId)}
							{#if voiceStore.getUserVolume(userId) !== 100}
								<span class="ml-1 opacity-70">{voiceStore.getUserVolume(userId)}%</span>
							{/if}
						</div>
					</div>
				{/each}
			</div>
		{/if}

		<!-- Context menu -->
		{#if menuUserId}
			<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
			<div
				class="fixed inset-0 z-40"
				onclick={closeMenu} onkeydown={(e) => { if (e.key === "Escape") closeMenu(); }} role="presentation"
				oncontextmenu={(e) => { e.preventDefault(); closeMenu(); }}
			></div>
			<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
			<div role="menu" tabindex="-1"
				class="fixed z-50 w-56 rounded-lg border border-white/10 bg-[var(--bg-secondary)] p-3 shadow-xl"
				style="left: {menuPos.x}px; top: {menuPos.y}px;"
				onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}
			>
				{#if menuType === 'self'}
					<!-- Self: mic gain control -->
					<div class="mb-2 flex items-center gap-2 text-xs font-medium text-[var(--text-primary)]">
						<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" /><path d="M19 10v2a7 7 0 0 1-14 0v-2" /><line x1="12" y1="19" x2="12" y2="23" /><line x1="8" y1="23" x2="16" y2="23" />
						</svg>
						Mic Volume
					</div>
					<div class="mb-1 text-[10px] text-[var(--text-secondary)]">What others hear from you</div>
					<div class="flex items-center gap-2">
						<input
							type="range"
							min="0"
							max="200"
							value={preferencesStore.preferences.inputGain}
							oninput={(e) => webrtcManager.setMicGain(parseInt(e.currentTarget.value))}
							aria-label="Microphone volume"
							class="h-1.5 w-full cursor-pointer appearance-none rounded-full bg-white/10 accent-[var(--accent)]"
						/>
						<span class="w-10 text-right text-xs font-medium text-[var(--text-secondary)]">
							{preferencesStore.preferences.inputGain}%
						</span>
					</div>
					{#if preferencesStore.preferences.inputGain !== 100}
						<button
							onclick={() => webrtcManager.setMicGain(100)}
							class="mt-2 w-full rounded px-2 py-1 text-xs text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
						>
							Reset to 100%
						</button>
					{/if}
				{:else if menuType === 'screen'}
					<!-- Screen share: audio volume + mute -->
					<div class="mb-2 flex items-center gap-2 text-xs font-medium text-[var(--text-primary)]">
						<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<rect x="2" y="3" width="20" height="14" rx="2" ry="2" /><line x1="8" y1="21" x2="16" y2="21" /><line x1="12" y1="17" x2="12" y2="21" />
						</svg>
						{userStore.getDisplayName(menuUserId)}'s Stream
					</div>
					<div class="mb-2 flex items-center gap-2">
						<input
							type="range"
							min="0"
							max="100"
							value={voiceStore.isScreenShareMuted(menuUserId) ? 0 : voiceStore.getScreenShareVolume(menuUserId)}
							disabled={voiceStore.isScreenShareMuted(menuUserId)}
							oninput={(e) => { if (menuUserId) voiceStore.setScreenShareVolume(menuUserId, parseInt(e.currentTarget.value)); }}
							aria-label="Screen share volume"
							class="h-1.5 w-full cursor-pointer appearance-none rounded-full bg-white/10 accent-[var(--accent)] disabled:opacity-40"
						/>
						<span class="w-10 text-right text-xs font-medium text-[var(--text-secondary)]">
							{voiceStore.isScreenShareMuted(menuUserId) ? '---' : `${voiceStore.getScreenShareVolume(menuUserId)}%`}
						</span>
					</div>
					<button
						onclick={() => { if (menuUserId) voiceStore.toggleScreenShareMute(menuUserId); }}
						class="w-full rounded px-2 py-1.5 text-left text-xs transition hover:bg-white/5 {voiceStore.isScreenShareMuted(menuUserId) ? 'text-[var(--danger)]' : 'text-[var(--text-secondary)]'}"
					>
						{#if voiceStore.isScreenShareMuted(menuUserId)}
							<span class="flex items-center gap-2">
								<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" /><line x1="23" y1="9" x2="17" y2="15" /><line x1="17" y1="9" x2="23" y2="15" />
								</svg>
								Unmute stream audio
							</span>
						{:else}
							<span class="flex items-center gap-2">
								<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" /><path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07" />
								</svg>
								Mute stream audio
							</span>
						{/if}
					</button>
					{#if voiceStore.getScreenShareVolume(menuUserId) !== 100}
						<button
							onclick={() => { if (menuUserId) voiceStore.setScreenShareVolume(menuUserId, 100); }}
							class="w-full rounded px-2 py-1 text-xs text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
						>
							Reset to 100%
						</button>
					{/if}
				{:else}
					<!-- Remote: playback volume control -->
					<div class="mb-2 flex items-center gap-2 text-xs font-medium text-[var(--text-primary)]">
						<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" /><path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07" />
						</svg>
						{userStore.getDisplayName(menuUserId)}
					</div>
					<div class="flex items-center gap-2">
						<input
							type="range"
							min="0"
							max="500"
							value={voiceStore.getUserVolume(menuUserId)}
							oninput={(e) => { if (menuUserId) voiceStore.setUserVolume(menuUserId, parseInt(e.currentTarget.value)); }}
							aria-label="User volume"
							class="h-1.5 w-full cursor-pointer appearance-none rounded-full bg-white/10 accent-[var(--accent)]"
						/>
						<span class="w-10 text-right text-xs font-medium text-[var(--text-secondary)]">
							{voiceStore.getUserVolume(menuUserId)}%
						</span>
					</div>
					{#if voiceStore.getUserVolume(menuUserId) !== 100}
						<button
							onclick={() => { if (menuUserId) voiceStore.setUserVolume(menuUserId, 100); }}
							class="mt-2 w-full rounded px-2 py-1 text-xs text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
						>
							Reset to 100%
						</button>
					{/if}
					{#if canKick && onKickFromVoice}
						<div class="my-1.5 border-t border-white/10"></div>
						<button
							onclick={() => { if (menuUserId && onKickFromVoice) { onKickFromVoice(menuUserId); closeMenu(); } }}
							class="w-full rounded px-2 py-1 text-left text-xs text-[var(--danger)] transition hover:bg-white/5"
						>
							Kick from voice
						</button>
					{/if}
				{/if}
			</div>
		{/if}
	</div>
{/if}
