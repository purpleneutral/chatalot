<script lang="ts">
	import { voiceStore } from '$lib/stores/voice.svelte';
	import { audioDeviceStore } from '$lib/stores/audioDevices.svelte';
	import { preferencesStore } from '$lib/stores/preferences.svelte';

	// Web Audio API gain nodes for per-user volume amplification (beyond 100%)
	let audioCtx: AudioContext | null = null;
	let userGains = new Map<string, { source: MediaStreamAudioSourceNode; gain: GainNode }>();
	let screenGains = new Map<string, { source: MediaStreamAudioSourceNode; gain: GainNode }>();

	// Hidden <audio> elements for output device routing (setSinkId)
	let audioEls = $state<Map<string, HTMLAudioElement>>(new Map());
	let screenAudioEls = $state<Map<string, HTMLAudioElement>>(new Map());
	let remoteEntries = $derived([...voiceStore.remoteStreams]);
	let remoteScreenEntries = $derived([...voiceStore.remoteScreenStreams]);

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
			(node as any).setSinkId(sinkId).catch(() => {});
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

	function bindScreenAudio(node: HTMLAudioElement, userId: string) {
		const next = new Map(screenAudioEls);
		next.set(userId, node);
		screenAudioEls = next;

		const stream = voiceStore.remoteScreenStreams.get(userId);
		if (stream) setupScreenGainNode(userId, stream, node);

		const sinkId = audioDeviceStore.selectedOutputId;
		if (sinkId && 'setSinkId' in node) {
			(node as any).setSinkId(sinkId).catch(() => {});
		}

		return {
			destroy() {
				cleanupScreenGainNode(userId);
				const next = new Map(screenAudioEls);
				next.delete(userId);
				screenAudioEls = next;
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

	function setupScreenGainNode(userId: string, stream: MediaStream, audioEl: HTMLAudioElement) {
		cleanupScreenGainNode(userId);

		// Only set up if the stream has audio tracks
		if (stream.getAudioTracks().length === 0) return;

		const ctx = getAudioContext();
		const source = ctx.createMediaStreamSource(stream);
		const gain = ctx.createGain();

		const masterVol = preferencesStore.preferences.outputVolume / 100;
		const screenVol = voiceStore.getScreenShareVolume(userId) / 100;
		const muted = voiceStore.isScreenShareMuted(userId);
		gain.gain.value = muted ? 0 : screenVol * masterVol;

		const destination = ctx.createMediaStreamDestination();
		source.connect(gain);
		gain.connect(destination);

		audioEl.srcObject = destination.stream;
		screenGains.set(userId, { source, gain });
	}

	function cleanupGainNode(userId: string) {
		const existing = userGains.get(userId);
		if (existing) {
			existing.source.disconnect();
			existing.gain.disconnect();
			userGains.delete(userId);
		}
	}

	function cleanupScreenGainNode(userId: string) {
		const existing = screenGains.get(userId);
		if (existing) {
			existing.source.disconnect();
			existing.gain.disconnect();
			screenGains.delete(userId);
		}
	}

	// Reconnect voice streams when they change
	$effect(() => {
		for (const [userId, stream] of voiceStore.remoteStreams) {
			const el = audioEls.get(userId);
			const existing = userGains.get(userId);
			if (el && (!existing || el.srcObject !== userGains.get(userId)?.gain)) {
				setupGainNode(userId, stream, el);
			}
		}
	});

	// Reconnect screen share streams when they change
	$effect(() => {
		for (const [userId, stream] of voiceStore.remoteScreenStreams) {
			const el = screenAudioEls.get(userId);
			const existing = screenGains.get(userId);
			if (el && (!existing || el.srcObject !== screenGains.get(userId)?.gain)) {
				setupScreenGainNode(userId, stream, el);
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

	// Apply screen share volume × master volume, respecting mute
	$effect(() => {
		const masterVol = preferencesStore.preferences.outputVolume / 100;
		for (const [userId] of voiceStore.remoteScreenStreams) {
			const entry = screenGains.get(userId);
			if (entry) {
				const muted = voiceStore.isScreenShareMuted(userId);
				const screenVol = voiceStore.getScreenShareVolume(userId) / 100;
				entry.gain.gain.value = muted ? 0 : screenVol * masterVol;
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
		for (const [, el] of screenAudioEls) {
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
	{#each remoteScreenEntries as [userId] (userId)}
		<!-- svelte-ignore element_invalid_self_closing_tag -->
		<audio autoplay use:bindScreenAudio={userId} class="hidden"></audio>
	{/each}
{/if}
