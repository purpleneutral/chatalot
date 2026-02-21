<script lang="ts">
	import '../app.css';
	import Toast from '$lib/components/Toast.svelte';
	import PersistentAudio from '$lib/components/PersistentAudio.svelte';

	let { children } = $props();
	void children;
</script>

<svelte:head>
	<title>Chatalot</title>
</svelte:head>

<h1 style="color: white; background: #1a1a2e; padding: 20px; font-family: sans-serif;">Layout with stores works!</h1>
<Toast />
<PersistentAudio />
