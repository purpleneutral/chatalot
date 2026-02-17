<script lang="ts">
	import { userStore } from '$lib/stores/users.svelte';
	import { presenceStore } from '$lib/stores/presence.svelte';
	import { isTauri, getServerUrl } from '$lib/env';

	type Size = 'xs' | 'sm' | 'md' | 'lg';

	let {
		userId,
		size = 'md',
		showStatus = false,
		name = undefined,
		avatarUrl = undefined
	}: {
		userId?: string;
		size?: Size;
		showStatus?: boolean;
		name?: string;
		avatarUrl?: string | null;
	} = $props();

	let imgError = $state(false);
	let prevUrl = $state<string | null>(null);

	// Reset error state when the avatar URL changes (e.g. user updates their avatar)
	$effect(() => {
		if (url !== prevUrl) {
			prevUrl = url;
			imgError = false;
		}
	});

	const sizeClasses: Record<Size, string> = {
		xs: 'h-6 w-6',
		sm: 'h-7 w-7',
		md: 'h-9 w-9',
		lg: 'h-16 w-16'
	};

	const textSizes: Record<Size, string> = {
		xs: 'text-xs',
		sm: 'text-xs',
		md: 'text-sm',
		lg: 'text-2xl'
	};

	const dotSizes: Record<Size, string> = {
		xs: 'h-2 w-2 -right-0 -bottom-0',
		sm: 'h-2 w-2 -right-0 -bottom-0',
		md: 'h-2.5 w-2.5 -right-0.5 -bottom-0.5',
		lg: 'h-3.5 w-3.5 -right-0.5 -bottom-0.5'
	};

	const statusColors: Record<string, string> = {
		online: 'bg-[var(--success)]',
		idle: 'bg-yellow-400',
		dnd: 'bg-[var(--danger)]',
		invisible: 'bg-gray-500',
		offline: 'bg-gray-500'
	};

	let user = $derived(userId ? userStore.getUser(userId) : undefined);
	let displayName = $derived(name ?? user?.display_name ?? userId?.slice(0, 8) ?? '?');
	let initial = $derived(displayName[0]?.toUpperCase() ?? '?');
	let rawUrl = $derived(avatarUrl !== undefined ? avatarUrl : (user?.avatar_url ?? null));
	// Resolve relative avatar URLs for Tauri desktop mode
	let url = $derived(rawUrl && rawUrl.startsWith('/') && isTauri() ? `${getServerUrl() ?? ''}${rawUrl}` : rawUrl);
	let status = $derived(userId ? presenceStore.getStatus(userId) : 'offline');
	let customStatus = $derived(user?.custom_status ?? null);
</script>

<div
	class="relative inline-flex shrink-0 items-center justify-center rounded-full {sizeClasses[size]} {url && !imgError ? '' : 'bg-[var(--bg-tertiary)]'}"
	title={customStatus ?? undefined}
>
	{#if url && !imgError}
		<img
			src={url}
			alt={displayName}
			class="h-full w-full rounded-full object-cover"
			onerror={() => { imgError = true; }}
		/>
	{:else}
		<span class="{textSizes[size]} font-medium text-[var(--text-secondary)]">
			{initial}
		</span>
	{/if}

	{#if showStatus && userId}
		<span
			class="absolute rounded-full border-2 border-[var(--bg-secondary)] {dotSizes[size]} {statusColors[status] ?? statusColors.offline}"
		></span>
	{/if}
</div>
