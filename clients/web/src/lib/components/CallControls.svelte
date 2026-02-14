<script lang="ts">
	import { voiceStore } from '$lib/stores/voice.svelte';
	import { webrtcManager } from '$lib/webrtc/manager';
	import { authStore } from '$lib/stores/auth.svelte';

	interface Props {
		channelId: string;
		channelType: string;
	}

	let { channelId, channelType }: Props = $props();

	let voiceParticipants = $derived(voiceStore.getChannelParticipants(channelId));
	let isInThisCall = $derived(voiceStore.activeCall?.channelId === channelId);
	let callActive = $derived(voiceParticipants.length > 0);

	async function joinCall(withVideo: boolean) {
		try {
			await webrtcManager.joinCall(channelId, withVideo);
		} catch (err) {
			console.error('Failed to join call:', err);
		}
	}

	async function leaveCall() {
		await webrtcManager.leaveCall();
	}
</script>

{#if channelType === 'voice' || (channelType === 'text' && (callActive || isInThisCall))}
	<div class="flex items-center gap-2">
		{#if callActive && !isInThisCall}
			<div class="flex items-center gap-2">
				<span class="flex h-2 w-2 rounded-full bg-green-400 animate-pulse"></span>
				<span class="text-xs text-green-400">{voiceParticipants.length} in call</span>
			</div>
		{/if}

		{#if isInThisCall}
			<!-- Call controls -->
			<div class="flex items-center gap-1">
				<!-- Mute/unmute -->
				<button
					onclick={() => webrtcManager.toggleAudio()}
					class="rounded-lg p-2 text-sm transition {voiceStore.activeCall?.audioEnabled ? 'bg-white/5 text-[var(--text-primary)]' : 'bg-red-500/20 text-red-400'}"
					title={voiceStore.activeCall?.audioEnabled ? 'Mute' : 'Unmute'}
				>
					{#if voiceStore.activeCall?.audioEnabled}
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
							<path d="M19 10v2a7 7 0 0 1-14 0v-2" /><line x1="12" y1="19" x2="12" y2="23" />
							<line x1="8" y1="23" x2="16" y2="23" />
						</svg>
					{:else}
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<line x1="1" y1="1" x2="23" y2="23" />
							<path d="M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6" />
							<path d="M17 16.95A7 7 0 0 1 5 12v-2m14 0v2c0 .4-.03.8-.1 1.17" />
							<line x1="12" y1="19" x2="12" y2="23" /><line x1="8" y1="23" x2="16" y2="23" />
						</svg>
					{/if}
				</button>

				<!-- Video toggle -->
				<button
					onclick={() => webrtcManager.toggleVideo()}
					class="rounded-lg p-2 text-sm transition {voiceStore.activeCall?.videoEnabled ? 'bg-white/5 text-[var(--text-primary)]' : 'bg-white/5 text-[var(--text-secondary)]'}"
					title={voiceStore.activeCall?.videoEnabled ? 'Turn off camera' : 'Turn on camera'}
				>
					{#if voiceStore.activeCall?.videoEnabled}
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<polygon points="23 7 16 12 23 17 23 7" /><rect x="1" y="5" width="15" height="14" rx="2" ry="2" />
						</svg>
					{:else}
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<path d="M16 16v1a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2h2m5.66 0H14a2 2 0 0 1 2 2v3.34l1 1L23 7v10" />
							<line x1="1" y1="1" x2="23" y2="23" />
						</svg>
					{/if}
				</button>

				<!-- Screen share -->
				<button
					onclick={() => webrtcManager.toggleScreenShare()}
					class="rounded-lg p-2 text-sm transition {voiceStore.activeCall?.screenSharing ? 'bg-[var(--accent)]/20 text-[var(--accent)]' : 'bg-white/5 text-[var(--text-secondary)]'}"
					title={voiceStore.activeCall?.screenSharing ? 'Stop sharing' : 'Share screen'}
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<rect x="2" y="3" width="20" height="14" rx="2" ry="2" /><line x1="8" y1="21" x2="16" y2="21" />
						<line x1="12" y1="17" x2="12" y2="21" />
					</svg>
				</button>

				<!-- Leave call -->
				<button
					onclick={leaveCall}
					class="rounded-lg bg-red-500 p-2 text-sm text-white transition hover:bg-red-600"
					title="Leave call"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<path d="M10.68 13.31a16 16 0 0 0 3.41 2.6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7 2 2 0 0 1 1.72 2v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91" />
						<line x1="23" y1="1" x2="1" y2="23" />
					</svg>
				</button>
			</div>
		{:else}
			<!-- Join buttons -->
			<button
				onclick={() => joinCall(false)}
				class="rounded-lg bg-green-600 px-3 py-1.5 text-xs font-medium text-white transition hover:bg-green-700"
				title="Join voice call"
			>
				Join Voice
			</button>
			<button
				onclick={() => joinCall(true)}
				class="rounded-lg border border-white/10 px-3 py-1.5 text-xs font-medium text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
				title="Join with video"
			>
				Video
			</button>
		{/if}
	</div>
{/if}
