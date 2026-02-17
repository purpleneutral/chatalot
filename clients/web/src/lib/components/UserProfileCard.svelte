<script lang="ts">
	import { scale } from 'svelte/transition';
	import Avatar from './Avatar.svelte';
	import { userStore } from '$lib/stores/users.svelte';
	import { presenceStore } from '$lib/stores/presence.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { communityMemberStore } from '$lib/stores/communityMembers.svelte';
	import { memberStore } from '$lib/stores/members.svelte';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { blockUser, unblockUser } from '$lib/api/users';
	import { createTimeout } from '$lib/api/communities';

	let {
		userId,
		communityId = undefined,
		channelId = undefined,
		anchorRect,
		onclose,
		onstartdm,
		blockedIds = [],
		canModerate = false
	}: {
		userId: string;
		communityId?: string;
		channelId?: string;
		anchorRect: { x: number; y: number };
		onclose: () => void;
		onstartdm?: (userId: string) => void;
		blockedIds?: string[];
		canModerate?: boolean;
	} = $props();

	const user = $derived(userStore.getUser(userId));
	const status = $derived(presenceStore.getStatus(userId));
	const nickname = $derived(
		communityId ? communityMemberStore.getNickname(communityId, userId) : null
	);
	const channelMember = $derived(
		channelId
			? memberStore.getMembers(channelId).find((m) => m.user_id === userId)
			: undefined
	);
	const role = $derived(channelMember?.role ?? 'member');
	const isOwnProfile = $derived(userId === authStore.user?.id);
	let isBlocked = $state(false);
	$effect(() => { isBlocked = blockedIds.includes(userId); });
	let blockLoading = $state(false);
	let showTimeoutPicker = $state(false);
	let timeoutLoading = $state(false);

	const statusLabels: Record<string, string> = {
		online: 'Online',
		idle: 'Idle',
		dnd: 'Do Not Disturb',
		invisible: 'Invisible',
		offline: 'Offline'
	};

	const statusColors: Record<string, string> = {
		online: 'bg-[var(--success)]',
		idle: 'bg-yellow-400',
		dnd: 'bg-[var(--danger)]',
		invisible: 'bg-gray-500',
		offline: 'bg-gray-500'
	};

	const roleColors: Record<string, string> = {
		owner: 'bg-yellow-500/20 text-yellow-400',
		admin: 'bg-blue-500/20 text-blue-400',
		member: 'bg-white/10 text-[var(--text-secondary)]'
	};

	// Position the card relative to the anchor, clamped to viewport
	let cardEl = $state<HTMLDivElement | null>(null);
	let cardStyle = $derived.by(() => {
		const padding = 12;
		const estimatedW = 280;
		const estimatedH = 300;
		let x = anchorRect.x + padding;
		let y = anchorRect.y;

		if (typeof window !== 'undefined') {
			if (x + estimatedW > window.innerWidth - padding) {
				x = anchorRect.x - estimatedW - padding;
			}
			if (y + estimatedH > window.innerHeight - padding) {
				y = window.innerHeight - estimatedH - padding;
			}
			if (y < padding) y = padding;
			if (x < padding) x = padding;
		}

		return `left: ${x}px; top: ${y}px;`;
	});

	const joinedDate = $derived.by(() => {
		if (!user?.created_at) return null;
		const d = new Date(user.created_at);
		return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });
	});

	async function handleCopyId() {
		try {
			await navigator.clipboard.writeText(userId);
			toastStore.success('User ID copied');
		} catch {
			toastStore.error('Failed to copy');
		}
	}

	function handleSendMessage() {
		if (onstartdm && !isOwnProfile) {
			onstartdm(userId);
			onclose();
		}
	}

	const timeoutOptions = [
		{ label: '1 minute', seconds: 60 },
		{ label: '5 minutes', seconds: 300 },
		{ label: '15 minutes', seconds: 900 },
		{ label: '1 hour', seconds: 3600 },
		{ label: '1 day', seconds: 86400 },
		{ label: '1 week', seconds: 604800 }
	];

	async function handleTimeout(seconds: number) {
		if (!communityId || !channelId) return;
		timeoutLoading = true;
		try {
			await createTimeout(communityId, channelId, userId, seconds);
			toastStore.success('User timed out');
			showTimeoutPicker = false;
			onclose();
		} catch (e: any) {
			toastStore.error(e?.message ?? 'Failed to timeout user');
		} finally {
			timeoutLoading = false;
		}
	}

	async function handleToggleBlock() {
		blockLoading = true;
		try {
			if (isBlocked) {
				await unblockUser(userId);
				isBlocked = false;
				toastStore.success('User unblocked');
			} else {
				await blockUser(userId);
				isBlocked = true;
				toastStore.success('User blocked');
			}
			window.dispatchEvent(new CustomEvent('chatalot:blocks-changed'));
		} catch (e: any) {
			toastStore.error(e?.message ?? 'Failed to update block');
		} finally {
			blockLoading = false;
		}
	}
</script>

<!-- Backdrop -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed inset-0 z-50"
	onclick={onclose}
	onkeydown={(e) => { if (e.key === 'Escape') onclose(); }}
>
	<!-- Card -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		bind:this={cardEl}
		class="fixed z-50 w-[280px] rounded-xl border border-white/10 bg-[var(--bg-secondary)] shadow-2xl"
		style={cardStyle}
		onclick={(e) => e.stopPropagation()}
		onkeydown={(e) => e.stopPropagation()}
		transition:scale={{ start: 0.9, duration: 150 }}
	>
		<!-- Banner / header area -->
		<div class="relative h-24 overflow-hidden rounded-t-xl">
			{#if user?.banner_url}
				<img src={user.banner_url} alt="User banner" class="h-full w-full object-cover" />
			{:else}
				<div class="h-full w-full bg-gradient-to-r from-[var(--accent)] to-[var(--accent-hover)]"></div>
			{/if}
			<!-- Avatar overlapping the banner -->
			<div class="absolute -bottom-8 left-4">
				<div class="rounded-full border-4 border-[var(--bg-secondary)]">
					<Avatar {userId} size="lg" showStatus />
				</div>
			</div>
		</div>

		<!-- Content area -->
		<div class="px-4 pb-4 pt-10">
			<!-- Name + username -->
			<div class="mb-1">
				<h3 class="text-base font-bold text-[var(--text-primary)]">
					{user?.display_name ?? userId.slice(0, 8)}
					{#if user?.pronouns}
						<span class="ml-1.5 text-xs font-normal text-[var(--text-secondary)]">({user.pronouns})</span>
					{/if}
				</h3>
				<p class="text-sm text-[var(--text-secondary)]">
					@{user?.username ?? '...'}
				</p>
			</div>

			<!-- Community nickname (if set and different from display name) -->
			{#if nickname && nickname !== user?.display_name}
				<p class="mb-1 text-xs text-[var(--text-secondary)]">
					Nickname: <span class="font-medium text-[var(--text-primary)]">{nickname}</span>
				</p>
			{/if}

			<!-- Status -->
			<div class="mb-3 flex items-center gap-1.5">
				<span class="h-2 w-2 rounded-full {statusColors[status] ?? statusColors.offline}"></span>
				<span class="text-xs text-[var(--text-secondary)]">{statusLabels[status] ?? 'Offline'}</span>
				{#if user?.custom_status}
					<span class="text-xs text-[var(--text-secondary)]">— {user.custom_status}</span>
				{/if}
			</div>

			<!-- Bio -->
			{#if user?.bio}
				<p class="mb-3 text-sm text-[var(--text-primary)] leading-relaxed">{user.bio}</p>
			{/if}

			<!-- Role badge -->
			<div class="mb-3 flex flex-wrap gap-1.5">
				<span class="rounded-full px-2 py-0.5 text-xs font-medium {roleColors[role] ?? roleColors.member}">
					{role.charAt(0).toUpperCase() + role.slice(1)}
				</span>
				{#if user?.is_owner}
					<span class="rounded-full bg-amber-500/20 px-2 py-0.5 text-xs font-medium text-amber-400">
						Instance Owner
					</span>
				{:else if user?.is_admin}
					<span class="rounded-full bg-red-500/20 px-2 py-0.5 text-xs font-medium text-red-400">
						Server Admin
					</span>
				{/if}
			</div>

			<!-- Member since -->
			{#if joinedDate}
				<div class="mb-3 text-xs text-[var(--text-secondary)]">
					<span class="font-medium uppercase tracking-wider">Member Since</span>
					<p class="mt-0.5 text-[var(--text-primary)]">{joinedDate}</p>
				</div>
			{/if}

			<!-- Separator -->
			<div class="mb-3 border-t border-white/10"></div>

			<!-- Actions -->
			<div class="flex flex-col gap-1.5">
				{#if !isOwnProfile && onstartdm}
					<button
						onclick={handleSendMessage}
						class="flex w-full items-center gap-2 rounded-lg bg-[var(--accent)] px-3 py-1.5 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)]"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
						</svg>
						Send Message
					</button>
				{/if}
				<button
					onclick={handleCopyId}
					class="flex w-full items-center gap-2 rounded-lg px-3 py-1.5 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<rect x="9" y="9" width="13" height="13" rx="2" ry="2" /><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
					</svg>
					Copy User ID
				</button>
				{#if !isOwnProfile && canModerate && communityId && channelId}
					{#if showTimeoutPicker}
						<div class="flex flex-col gap-1 rounded-lg border border-white/10 bg-white/5 p-2">
							<p class="mb-1 text-xs font-medium text-[var(--text-secondary)]">Timeout duration:</p>
							{#each timeoutOptions as opt}
								<button
									onclick={() => handleTimeout(opt.seconds)}
									disabled={timeoutLoading}
									class="rounded px-2 py-1 text-left text-xs text-[var(--text-primary)] transition hover:bg-white/10"
								>
									{opt.label}
								</button>
							{/each}
							<button
								onclick={() => showTimeoutPicker = false}
								class="mt-1 text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
							>
								Cancel
							</button>
						</div>
					{:else}
						<button
							onclick={() => showTimeoutPicker = true}
							class="flex w-full items-center gap-2 rounded-lg px-3 py-1.5 text-sm text-yellow-400 transition hover:bg-yellow-500/10"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" />
							</svg>
							Timeout
						</button>
					{/if}
				{/if}
				{#if !isOwnProfile}
					<button
						onclick={handleToggleBlock}
						disabled={blockLoading}
						class="flex w-full items-center gap-2 rounded-lg px-3 py-1.5 text-sm transition {isBlocked ? 'text-[var(--text-secondary)] hover:bg-white/5' : 'text-[var(--danger)] hover:bg-[var(--danger)]/10'}"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<circle cx="12" cy="12" r="10" /><line x1="4.93" y1="4.93" x2="19.07" y2="19.07" />
						</svg>
						{isBlocked ? 'Unblock User' : 'Block User'}
					</button>
				{/if}
			</div>
		</div>
	</div>
</div>
