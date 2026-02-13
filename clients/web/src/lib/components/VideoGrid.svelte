<script lang="ts">
	import { voiceStore } from '$lib/stores/voice.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { userStore } from '$lib/stores/users.svelte';

	let localVideoEl: HTMLVideoElement | undefined = $state();
	let screenVideoEl: HTMLVideoElement | undefined = $state();
	let remoteVideoEls = $state<Map<string, HTMLVideoElement>>(new Map());

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

	let hasVideo = $derived(voiceStore.activeCall?.videoEnabled ?? false);
	let remoteEntries = $derived([...voiceStore.remoteStreams]);
	let totalParticipants = $derived(1 + remoteEntries.length);
	let gridCols = $derived(
		totalParticipants <= 1 ? 'grid-cols-1' :
		totalParticipants <= 4 ? 'grid-cols-2' :
		'grid-cols-3'
	);
</script>

{#if voiceStore.isInCall}
	<div class="border-b border-white/10 bg-[var(--bg-secondary)]">
		<div class="grid {gridCols} gap-1 p-2" style="max-height: 400px;">
			<!-- Local video/avatar -->
			<div class="relative flex items-center justify-center rounded-lg bg-[var(--bg-tertiary)] overflow-hidden" style="aspect-ratio: 16/9; min-height: 120px;">
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
					<div class="flex h-16 w-16 items-center justify-center rounded-full bg-[var(--accent)]">
						<span class="text-2xl font-bold text-white">
							{authStore.user?.display_name?.[0]?.toUpperCase() ?? '?'}
						</span>
					</div>
				{/if}
				<div class="absolute bottom-1 left-1 rounded bg-black/60 px-2 py-0.5 text-xs text-white">
					You {voiceStore.activeCall?.audioEnabled ? '' : '(muted)'}
				</div>
			</div>

			<!-- Remote participants -->
			{#each remoteEntries as [userId, _stream] (userId)}
				<div class="relative flex items-center justify-center rounded-lg bg-[var(--bg-tertiary)] overflow-hidden" style="aspect-ratio: 16/9; min-height: 120px;">
					<video
						autoplay
						playsinline
						class="h-full w-full object-cover"
						use:bindRemoteVideo={userId}
					></video>
					<div class="absolute bottom-1 left-1 rounded bg-black/60 px-2 py-0.5 text-xs text-white">
						{userStore.getDisplayName(userId)}
					</div>
				</div>
			{/each}
		</div>

		{#if voiceStore.activeCall?.screenSharing}
			<div class="border-t border-white/10 p-2">
				<video
					bind:this={screenVideoEl}
					autoplay
					muted
					playsinline
					class="w-full rounded-lg"
					style="max-height: 300px;"
				></video>
			</div>
		{/if}
	</div>
{/if}
