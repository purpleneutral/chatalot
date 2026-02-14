<script lang="ts">
	import { voiceStore } from '$lib/stores/voice.svelte';

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
</script>

{#if voiceStore.isInCall}
	{#each remoteEntries as [userId] (userId)}
		<!-- svelte-ignore element_invalid_self_closing_tag -->
		<audio autoplay use:bindAudio={userId} class="hidden"></audio>
	{/each}
{/if}
