<script lang="ts">
	import { voiceStore } from '$lib/stores/voice.svelte';
	import { audioDeviceStore } from '$lib/stores/audioDevices.svelte';
	import { preferencesStore } from '$lib/stores/preferences.svelte';

	// Hidden <audio> elements for remote streams, persists across route changes.
	// VideoGrid handles the visual part; this ensures audio never drops on navigation.
	let audioEls = $state<Map<string, HTMLAudioElement>>(new Map());
	let remoteEntries = $derived([...voiceStore.remoteStreams]);

	function bindAudio(node: HTMLAudioElement, userId: string) {
		const next = new Map(audioEls);
		next.set(userId, node);
		audioEls = next;

		const stream = voiceStore.remoteStreams.get(userId);
		if (stream) node.srcObject = stream;

		// Set output device if supported
		const sinkId = audioDeviceStore.selectedOutputId;
		if (sinkId && 'setSinkId' in node) {
			(node as any).setSinkId(sinkId).catch(() => {});
		}

		return {
			destroy() {
				const next = new Map(audioEls);
				next.delete(userId);
				audioEls = next;
			}
		};
	}

	$effect(() => {
		for (const [userId, stream] of voiceStore.remoteStreams) {
			const el = audioEls.get(userId);
			if (el && el.srcObject !== stream) {
				el.srcObject = stream;
			}
		}
	});

	// Apply per-user volume × master output volume
	$effect(() => {
		const masterVol = preferencesStore.preferences.outputVolume / 100;
		for (const [userId] of voiceStore.remoteStreams) {
			const el = audioEls.get(userId);
			if (el) {
				const userVol = voiceStore.getUserVolume(userId) / 100;
				el.volume = Math.min(1, userVol * masterVol);
			}
		}
	});

	// Apply output device changes to all audio elements
	$effect(() => {
		const sinkId = audioDeviceStore.selectedOutputId;
		for (const [, el] of audioEls) {
			if (el && 'setSinkId' in el) {
				(el as any).setSinkId(sinkId).catch(() => {});
			}
		}
	});
</script>

{#if voiceStore.isInCall}
	{#each remoteEntries as [userId] (userId)}
		<!-- svelte-ignore element_invalid_self_closing_tag -->
		<audio autoplay use:bindAudio={userId} class="hidden"></audio>
	{/each}
{/if}
