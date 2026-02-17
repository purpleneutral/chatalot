<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { toastStore } from '$lib/stores/toast.svelte';
	import * as groupApi from '$lib/api/groups';
	import * as communityApi from '$lib/api/communities';

	type InviteData =
		| { type: 'group'; name: string; description: string | null; memberCount: number }
		| { type: 'community'; name: string; description: string | null; memberCount: number };

	let invite = $state<InviteData | null>(null);
	let loading = $state(true);
	let joining = $state(false);
	let error = $state('');

	let code = $derived($page.params.code ?? '');

	onMount(async () => {
		if (!code) { error = 'No invite code provided.'; loading = false; return; }
		// Try community invite first (more common entry point), then group invite
		try {
			const info = await communityApi.getInviteInfo(code);
			invite = {
				type: 'community',
				name: info.community_name,
				description: info.community_description,
				memberCount: info.member_count
			};
		} catch {
			try {
				const info = await groupApi.getInviteInfo(code);
				invite = {
					type: 'group',
					name: info.group_name,
					description: info.group_description,
					memberCount: info.member_count
				};
			} catch {
				error = 'This invite link is invalid, expired, or has reached its usage limit.';
			}
		}
		loading = false;
	});

	async function handleJoin() {
		if (!authStore.isAuthenticated) {
			goto(`/login?redirect=/invite/${encodeURIComponent(code)}`);
			return;
		}

		joining = true;
		try {
			if (invite?.type === 'community') {
				const result = await communityApi.acceptInvite(code);
				toastStore.success(`Joined ${result.community_name}!`);
			} else {
				const result = await groupApi.acceptInvite(code);
				toastStore.success(`Joined ${result.group_name}!`);
			}
			goto('/channels');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to join';
		} finally {
			joining = false;
		}
	}
</script>

<div class="flex min-h-screen items-center justify-center p-4">
	<div class="w-full max-w-md rounded-2xl bg-[var(--bg-secondary)] p-8 shadow-2xl">
		<h1 class="mb-6 text-center text-2xl font-bold text-[var(--text-primary)]">Chatalot</h1>

		{#if loading}
			<div class="flex flex-col items-center gap-4 py-8">
				<div
					class="h-8 w-8 animate-spin rounded-full border-2 border-[var(--text-secondary)] border-t-[var(--accent)]"
				></div>
				<p class="text-sm text-[var(--text-secondary)]">Loading invite...</p>
			</div>
		{:else if error}
			<div class="flex flex-col items-center gap-4 py-4">
				<div class="rounded-lg bg-red-500/10 p-4 text-center text-sm text-[var(--danger)]" role="alert">
					{error}
				</div>
				<a
					href="/channels"
					class="text-sm text-[var(--accent)] transition hover:text-[var(--accent-hover)]"
				>
					Go to Chatalot
				</a>
			</div>
		{:else if invite}
			<div class="flex flex-col items-center gap-6">
				<p class="text-sm text-[var(--text-secondary)]">You've been invited to join</p>

				<div class="flex flex-col items-center gap-2">
					<div
						class="flex h-16 w-16 items-center justify-center rounded-xl bg-[var(--accent)]/20 text-2xl font-bold text-[var(--accent)]"
					>
						{invite.name.charAt(0).toUpperCase()}
					</div>
					<h2 class="text-xl font-semibold text-[var(--text-primary)]">{invite.name}</h2>
					{#if invite.description}
						<p class="text-center text-sm text-[var(--text-secondary)]">
							{invite.description}
						</p>
					{/if}
					<p class="text-xs text-[var(--text-secondary)]">
						{invite.type === 'community' ? 'Community' : 'Group'} &middot; {invite.memberCount}
						{invite.memberCount === 1 ? 'member' : 'members'}
					</p>
				</div>

				<button
					onclick={handleJoin}
					disabled={joining}
					class="w-full rounded-lg bg-[var(--accent)] px-4 py-2.5 font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:cursor-not-allowed disabled:opacity-50"
				>
					{#if !authStore.isAuthenticated}
						Sign in to join
					{:else if joining}
						Joining...
					{:else}
						Accept Invite
					{/if}
				</button>

				{#if !authStore.isAuthenticated}
					<p class="text-center text-sm text-[var(--text-secondary)]">
						Don't have an account?
						<a
							href="/register"
							class="text-[var(--accent)] transition hover:text-[var(--accent-hover)]"
						>
							Create one
						</a>
					</p>
				{/if}
			</div>
		{/if}
	</div>
</div>
