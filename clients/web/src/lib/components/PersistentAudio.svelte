<script lang="ts">
	import { onDestroy } from 'svelte';
	import { voiceStore } from '$lib/stores/voice.svelte';
	import { audioDeviceStore } from '$lib/stores/audioDevices.svelte';
	import { preferencesStore } from '$lib/stores/preferences.svelte';

	interface AudioElementWithSinkId extends HTMLAudioElement {
		setSinkId(sinkId: string): Promise<void>;
	}

	// Web Audio API gain nodes for per-user volume amplification (beyond 100%)
	let audioCtx: AudioContext | null = null;
	let userGains = new Map<string, { source: MediaStreamAudioSourceNode; gain: GainNode }>();

	// Hidden <audio> elements for output device routing (setSinkId)
	let audioEls = $state<Map<string, HTMLAudioElement>>(new Map());
	let remoteEntries = $derived([...voiceStore.remoteStreams]);

	function getAudioContext(): AudioContext {
		if (!audioCtx) {
			audioCtx = new AudioContext();
		}
		return audioCtx;
	}

	function bindAudio(node: HTMLAudioElement, userId: string) {
		const next = new Map(audioEls);
		next.set(userId, node);
		audioEls = next;

		const stream = voiceStore.remoteStreams.get(userId);
		if (stream) setupGainNode(userId, stream, node);

		// Set output device if supported
		const sinkId = audioDeviceStore.selectedOutputId;
		if (sinkId && 'setSinkId' in node) {
			(node as AudioElementWithSinkId).setSinkId(sinkId).catch((e) => console.warn('setSinkId failed:', e));
		}

		return {
			destroy() {
				cleanupGainNode(userId);
				const next = new Map(audioEls);
				next.delete(userId);
				audioEls = next;
			}
		};
	}

	function setupGainNode(userId: string, stream: MediaStream, audioEl: HTMLAudioElement) {
		cleanupGainNode(userId);

		const ctx = getAudioContext();
		const source = ctx.createMediaStreamSource(stream);
		const gain = ctx.createGain();

		const masterVol = preferencesStore.preferences.outputVolume / 100;
		const userVol = voiceStore.getUserVolume(userId) / 100;
		gain.gain.value = userVol * masterVol;

		// Route through gain node to a new MediaStream for the <audio> element
		const destination = ctx.createMediaStreamDestination();
		source.connect(gain);
		gain.connect(destination);

		audioEl.srcObject = destination.stream;
		userGains.set(userId, { source, gain });
	}

	function cleanupGainNode(userId: string) {
		const existing = userGains.get(userId);
		if (existing) {
			existing.source.disconnect();
			existing.gain.disconnect();
			userGains.delete(userId);
		}
	}

	// Reconnect streams when they change
	$effect(() => {
		for (const [userId, stream] of voiceStore.remoteStreams) {
			const el = audioEls.get(userId);
			const existing = userGains.get(userId);
			// Reconnect if no gain node yet (stream changed or first time)
			if (el && !existing) {
				setupGainNode(userId, stream, el);
			}
		}
	});

	// Apply per-user volume × master output volume via GainNode
	$effect(() => {
		const masterVol = preferencesStore.preferences.outputVolume / 100;
		for (const [userId] of voiceStore.remoteStreams) {
			const entry = userGains.get(userId);
			if (entry) {
				const userVol = voiceStore.getUserVolume(userId) / 100;
				entry.gain.gain.value = userVol * masterVol;
			}
		}
	});

	onDestroy(() => {
		for (const userId of userGains.keys()) {
			cleanupGainNode(userId);
		}
		if (audioCtx) {
			audioCtx.close();
			audioCtx = null;
		}
	});

	// Apply output device changes to all audio elements
	$effect(() => {
		const sinkId = audioDeviceStore.selectedOutputId;
		for (const [, el] of audioEls) {
			if (el && 'setSinkId' in el) {
				(el as AudioElementWithSinkId).setSinkId(sinkId).catch((e) => console.warn('setSinkId failed:', e));
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
