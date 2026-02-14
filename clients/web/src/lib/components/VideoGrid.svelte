<script lang="ts">
	import { voiceStore } from '$lib/stores/voice.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { userStore } from '$lib/stores/users.svelte';
	import Avatar from '$lib/components/Avatar.svelte';

	let { expanded = false }: { expanded?: boolean } = $props();

	let localVideoEl: HTMLVideoElement | undefined = $state();
	let screenVideoEl: HTMLVideoElement | undefined = $state();
	let remoteVideoEls = $state<Map<string, HTMLVideoElement>>(new Map());
	let remoteScreenEls = $state<Map<string, HTMLVideoElement>>(new Map());

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

	// Attach remote screen share streams
	$effect(() => {
		for (const [userId, stream] of voiceStore.remoteScreenStreams) {
			const el = remoteScreenEls.get(userId);
			if (el && el.srcObject !== stream) {
				el.srcObject = stream;
			}
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

	let hasVideo = $derived(voiceStore.activeCall?.videoEnabled ?? false);
	let remoteEntries = $derived([...voiceStore.remoteStreams]);
	let remoteScreenEntries = $derived([...voiceStore.remoteScreenStreams]);
	let hasAnyScreenShare = $derived(voiceStore.activeCall?.screenSharing || remoteScreenEntries.length > 0);
	let totalParticipants = $derived(1 + remoteEntries.length);
	let gridCols = $derived(
		totalParticipants <= 1 ? 'grid-cols-1' :
		totalParticipants <= 4 ? 'grid-cols-2' :
		'grid-cols-3'
	);

	// Per-user volume context menu
	let volumeMenuUserId = $state<string | null>(null);
	let volumeMenuPos = $state({ x: 0, y: 0 });

	function openVolumeMenu(e: MouseEvent, userId: string) {
		e.preventDefault();
		volumeMenuUserId = userId;
		volumeMenuPos = { x: e.clientX, y: e.clientY };
	}

	function closeVolumeMenu() {
		volumeMenuUserId = null;
	}
</script>

{#if voiceStore.isInCall}
	<div class="{expanded ? 'flex-1' : ''} border-b border-white/10 bg-[var(--bg-secondary)] {expanded ? 'flex flex-col' : ''}">
		<!-- Screen share area (local or remote) — shown prominently above participant tiles -->
		{#if hasAnyScreenShare}
			<div class="p-2">
				{#if voiceStore.activeCall?.screenSharing}
					<div class="relative">
						<video
							bind:this={screenVideoEl}
							autoplay
							muted
							playsinline
							class="w-full rounded-lg"
							style="max-height: 400px;"
						></video>
						<div class="absolute top-2 left-2 flex items-center gap-1.5 rounded bg-red-500/90 px-2 py-1 text-xs font-medium text-white">
							<span class="h-2 w-2 rounded-full bg-white animate-pulse"></span>
							You are sharing your screen
						</div>
					</div>
				{/if}

				{#each remoteScreenEntries as [userId, _stream] (userId)}
					<div class="relative mt-1">
						<video
							autoplay
							playsinline
							class="w-full rounded-lg"
							style="max-height: 400px;"
							use:bindRemoteScreen={userId}
						></video>
						<div class="absolute top-2 left-2 flex items-center gap-1.5 rounded bg-[var(--accent)]/90 px-2 py-1 text-xs font-medium text-white">
							<span class="h-2 w-2 rounded-full bg-white animate-pulse"></span>
							{userStore.getDisplayName(userId)} is sharing their screen
						</div>
					</div>
				{/each}
			</div>
		{/if}

		<!-- Participant tiles -->
		<div class="grid {gridCols} gap-1 p-2 {expanded ? 'flex-1' : ''}" style="{expanded ? '' : `max-height: ${hasAnyScreenShare ? '150px' : '400px'};`}">
			<!-- Local video/avatar -->
			<div class="relative flex items-center justify-center rounded-lg bg-[var(--bg-tertiary)] overflow-hidden transition-shadow duration-200 {voiceStore.isSpeaking(authStore.user?.id ?? '') ? 'ring-2 ring-[var(--success)] shadow-[0_0_8px_var(--success)]' : ''}" style="aspect-ratio: 16/9; min-height: {hasAnyScreenShare ? '80px' : expanded ? '200px' : '120px'};">
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
				</div>
				{#if voiceStore.activeCall?.screenSharing}
					<div class="absolute top-1 left-1 flex items-center gap-1 rounded bg-red-500/80 px-1.5 py-0.5 text-[10px] text-white">
						<span class="h-1.5 w-1.5 rounded-full bg-white animate-pulse"></span>
						LIVE
					</div>
				{/if}
			</div>

			<!-- Remote participants -->
			{#each remoteEntries as [userId, _stream] (userId)}
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					class="relative flex items-center justify-center rounded-lg bg-[var(--bg-tertiary)] overflow-hidden transition-shadow duration-200 {voiceStore.isSpeaking(userId) ? 'ring-2 ring-[var(--success)] shadow-[0_0_8px_var(--success)]' : ''}"
					style="aspect-ratio: 16/9; min-height: {hasAnyScreenShare ? '80px' : expanded ? '200px' : '120px'};"
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

		<!-- Per-user volume context menu -->
		{#if volumeMenuUserId}
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="fixed inset-0 z-40"
				onclick={closeVolumeMenu}
				oncontextmenu={(e) => { e.preventDefault(); closeVolumeMenu(); }}
			></div>
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="fixed z-50 w-52 rounded-lg border border-white/10 bg-[var(--bg-secondary)] p-3 shadow-xl"
				style="left: {volumeMenuPos.x}px; top: {volumeMenuPos.y}px;"
				onclick={(e) => e.stopPropagation()}
			>
				<div class="mb-2 flex items-center gap-2 text-xs font-medium text-[var(--text-primary)]">
					<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" /><path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07" />
					</svg>
					{userStore.getDisplayName(volumeMenuUserId)}
				</div>
				<div class="flex items-center gap-2">
					<input
						type="range"
						min="0"
						max="200"
						value={voiceStore.getUserVolume(volumeMenuUserId)}
						oninput={(e) => { if (volumeMenuUserId) voiceStore.setUserVolume(volumeMenuUserId, parseInt(e.currentTarget.value)); }}
						class="h-1.5 w-full cursor-pointer appearance-none rounded-full bg-white/10 accent-[var(--accent)]"
					/>
					<span class="w-10 text-right text-xs font-medium text-[var(--text-secondary)]">
						{voiceStore.getUserVolume(volumeMenuUserId)}%
					</span>
				</div>
				{#if voiceStore.getUserVolume(volumeMenuUserId) !== 100}
					<button
						onclick={() => { if (volumeMenuUserId) voiceStore.setUserVolume(volumeMenuUserId, 100); }}
						class="mt-2 w-full rounded px-2 py-1 text-xs text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					>
						Reset to 100%
					</button>
				{/if}
			</div>
		{/if}
	</div>
{/if}
