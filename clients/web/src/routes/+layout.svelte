<script lang="ts">
	import '../app.css';
	import Toast from '$lib/components/Toast.svelte';
	import { themeStore } from '$lib/stores/theme.svelte';
	import { onMount } from 'svelte';

	let { children } = $props();

	onMount(() => {
		// Ensure theme is applied on mount
		document.documentElement.setAttribute('data-theme', themeStore.resolved);
	});
</script>

<svelte:head>
	<title>Chatalot</title>
</svelte:head>

{@render children()}
<Toast />
