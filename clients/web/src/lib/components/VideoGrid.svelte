<script lang="ts">
	import { voiceStore } from '$lib/stores/voice.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { userStore } from '$lib/stores/users.svelte';
	import Avatar from '$lib/components/Avatar.svelte';

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
</script>

{#if voiceStore.isInCall}
	<div class="border-b border-white/10 bg-[var(--bg-secondary)]">
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
		<div class="grid {gridCols} gap-1 p-2" style="max-height: {hasAnyScreenShare ? '150px' : '400px'};">
			<!-- Local video/avatar -->
			<div class="relative flex items-center justify-center rounded-lg bg-[var(--bg-tertiary)] overflow-hidden transition-shadow duration-200 {voiceStore.isSpeaking(authStore.user?.id ?? '') ? 'ring-2 ring-[var(--success)] shadow-[0_0_8px_var(--success)]' : ''}" style="aspect-ratio: 16/9; min-height: {hasAnyScreenShare ? '80px' : '120px'};">
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
				<div class="absolute bottom-1 left-1 rounded bg-black/60 px-2 py-0.5 text-xs text-white">
					You {voiceStore.activeCall?.audioEnabled ? '' : '(muted)'}
				</div>
				{#if voiceStore.activeCall?.screenSharing}
					<div class="absolute top-1 right-1 flex items-center gap-1 rounded bg-red-500/80 px-1.5 py-0.5 text-[10px] text-white">
						<span class="h-1.5 w-1.5 rounded-full bg-white animate-pulse"></span>
						LIVE
					</div>
				{/if}
			</div>

			<!-- Remote participants -->
			{#each remoteEntries as [userId, _stream] (userId)}
				<div class="relative flex items-center justify-center rounded-lg bg-[var(--bg-tertiary)] overflow-hidden transition-shadow duration-200 {voiceStore.isSpeaking(userId) ? 'ring-2 ring-[var(--success)] shadow-[0_0_8px_var(--success)]' : ''}" style="aspect-ratio: 16/9; min-height: {hasAnyScreenShare ? '80px' : '120px'};">
					{#if !voiceStore.hasRemoteVideo(userId)}
						<Avatar {userId} size="lg" />
					{/if}
					<video
						autoplay
						playsinline
						class="{voiceStore.hasRemoteVideo(userId) ? 'h-full w-full object-cover' : 'absolute h-0 w-0 opacity-0'}"
						use:bindRemoteVideo={userId}
					></video>
					<div class="absolute bottom-1 left-1 rounded bg-black/60 px-2 py-0.5 text-xs text-white">
						{userStore.getDisplayName(userId)}
					</div>
				</div>
			{/each}
		</div>
	</div>
{/if}
