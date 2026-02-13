<script lang="ts">
	import { goto } from '$app/navigation';
	import { authStore } from '$lib/stores/auth.svelte';
	import { communityStore } from '$lib/stores/communities.svelte';
	import { toastStore } from '$lib/stores/toast.svelte';
	import {
		getCommunity,
		updateCommunity,
		deleteCommunity,
		leaveCommunity,
		transferOwnership,
		listMembers,
		setMemberRole,
		kickMember,
		banMember,
		unbanMember,
		listBans,
		listInvites,
		createInvite,
		deleteInvite,
		listCommunities,
		type Community,
		type CommunityMember,
		type CommunityInvite,
		type CommunityBan
	} from '$lib/api/communities';
	import Avatar from '$lib/components/Avatar.svelte';
	import { onMount } from 'svelte';

	let activeTab = $state<'overview' | 'members' | 'invites' | 'bans'>('overview');
	let community = $state<Community | null>(null);
	let members = $state<CommunityMember[]>([]);
	let invites = $state<CommunityInvite[]>([]);
	let bans = $state<CommunityBan[]>([]);
	let loading = $state(true);
	let myRole = $state<string>('member');

	// Edit state
	let editName = $state('');
	let editDescription = $state('');
	let saving = $state(false);

	// Invite creation
	let newInviteMaxUses = $state('');
	let newInviteExpiresHours = $state('');
	let creatingInvite = $state(false);

	let canManage = $derived(myRole === 'owner' || myRole === 'admin' || authStore.user?.is_admin);
	let isOwner = $derived(myRole === 'owner' || authStore.user?.is_admin);

	onMount(async () => {
		if (!authStore.isAuthenticated) {
			goto('/login');
			return;
		}
		if (!communityStore.activeCommunityId) {
			goto('/channels');
			return;
		}
		await loadData();
	});

	async function loadData() {
		loading = true;
		try {
			const cid = communityStore.activeCommunityId!;
			// Also load all communities to populate the store (needed for updates)
			const [c, m, allCommunities] = await Promise.all([
				getCommunity(cid),
				listMembers(cid),
				listCommunities()
			]);
			communityStore.setCommunities(allCommunities);
			community = c;
			members = m;
			editName = c.name;
			editDescription = c.description ?? '';

			// Find my role
			const me = m.find((mem) => mem.user_id === authStore.user?.id);
			myRole = me?.role ?? 'member';
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to load community');
			goto('/channels');
		} finally {
			loading = false;
		}
	}

	async function handleSave() {
		if (!community) return;
		const name = editName.trim();
		if (!name) {
			toastStore.error('Community name cannot be empty');
			return;
		}
		if (name.length > 64) {
			toastStore.error('Community name must be 64 characters or less');
			return;
		}
		saving = true;
		try {
			const updated = await updateCommunity(
				community.id,
				name,
				editDescription.trim()
			);
			community = updated;
			communityStore.updateCommunity(community.id, {
				name: updated.name,
				description: updated.description
			});
			toastStore.success('Community updated');
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to update');
		} finally {
			saving = false;
		}
	}

	async function handleCreateInvite() {
		if (!community) return;
		creatingInvite = true;
		try {
			const invite = await createInvite(
				community.id,
				newInviteMaxUses ? parseInt(newInviteMaxUses) : undefined,
				newInviteExpiresHours ? parseInt(newInviteExpiresHours) : undefined
			);
			invites = [invite, ...invites];
			await navigator.clipboard.writeText(invite.code);
			toastStore.success(`Invite code copied: ${invite.code}`);
			newInviteMaxUses = '';
			newInviteExpiresHours = '';
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to create invite');
		} finally {
			creatingInvite = false;
		}
	}

	async function loadInvites() {
		if (!community) return;
		try {
			invites = await listInvites(community.id);
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to load invites');
		}
	}

	async function loadBans() {
		if (!community) return;
		try {
			bans = await listBans(community.id);
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to load bans');
		}
	}

	async function handleDeleteInvite(inviteId: string) {
		if (!community) return;
		try {
			await deleteInvite(community.id, inviteId);
			invites = invites.filter((i) => i.id !== inviteId);
			toastStore.success('Invite deleted');
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to delete invite');
		}
	}

	async function handleRoleChange(userId: string, newRole: string) {
		if (!community) return;
		try {
			await setMemberRole(community.id, userId, newRole);
			members = members.map((m) => (m.user_id === userId ? { ...m, role: newRole } : m));
			toastStore.success('Role updated');
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to update role');
		}
	}

	async function handleKick(member: CommunityMember) {
		if (!community || !confirm(`Kick ${member.display_name} from ${community.name}?`)) return;
		try {
			await kickMember(community.id, member.user_id);
			members = members.filter((m) => m.user_id !== member.user_id);
			toastStore.success(`Kicked ${member.display_name}`);
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to kick member');
		}
	}

	async function handleBan(member: CommunityMember) {
		if (!community) return;
		const reason = prompt(`Ban ${member.display_name}? Enter an optional reason:`);
		if (reason === null) return;
		try {
			await banMember(community.id, member.user_id, reason || undefined);
			members = members.filter((m) => m.user_id !== member.user_id);
			toastStore.success(`Banned ${member.display_name}`);
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to ban member');
		}
	}

	async function handleUnban(userId: string) {
		if (!community) return;
		try {
			await unbanMember(community.id, userId);
			bans = bans.filter((b) => b.user_id !== userId);
			toastStore.success('User unbanned');
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to unban');
		}
	}

	async function handleDelete() {
		if (
			!community ||
			!confirm(
				`Delete "${community.name}"? This will permanently delete ALL groups, channels, and messages. This cannot be undone.`
			)
		)
			return;
		try {
			await deleteCommunity(community.id);
			communityStore.removeCommunity(community.id);
			const communities = await listCommunities();
			communityStore.setCommunities(communities);
			toastStore.success('Community deleted');
			goto('/channels');
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to delete community');
		}
	}

	async function handleLeave() {
		if (!community || !confirm(`Leave "${community.name}"?`)) return;
		try {
			await leaveCommunity(community.id);
			communityStore.removeCommunity(community.id);
			const communities = await listCommunities();
			communityStore.setCommunities(communities);
			toastStore.success(`Left "${community.name}"`);
			goto('/channels');
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to leave community');
		}
	}

	async function handleTransferOwnership() {
		if (!community) return;
		const newOwnerId = prompt('Enter the user ID of the new owner:');
		if (!newOwnerId) return;
		try {
			await transferOwnership(community.id, newOwnerId.trim());
			await loadData();
			toastStore.success('Ownership transferred');
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to transfer ownership');
		}
	}

	function switchTab(tab: typeof activeTab) {
		activeTab = tab;
		if (tab === 'invites' && invites.length === 0) loadInvites();
		if (tab === 'bans' && bans.length === 0) loadBans();
	}

	function roleLabel(role: string): string {
		return role.charAt(0).toUpperCase() + role.slice(1);
	}

	function roleBadgeColor(role: string): string {
		switch (role) {
			case 'owner':
				return 'bg-yellow-500/20 text-yellow-400';
			case 'admin':
				return 'bg-red-500/20 text-red-400';
			case 'moderator':
				return 'bg-blue-500/20 text-blue-400';
			default:
				return 'bg-white/10 text-[var(--text-secondary)]';
		}
	}
</script>

<div class="flex min-h-screen flex-col bg-[var(--bg-primary)]">
	<!-- Header -->
	<header class="flex h-14 items-center justify-between border-b border-white/10 px-6">
		<div class="flex items-center gap-3">
			<button onclick={() => goto('/channels')} class="text-[var(--text-secondary)] transition hover:text-[var(--text-primary)]">
				<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<line x1="19" y1="12" x2="5" y2="12" /><polyline points="12 19 5 12 12 5" />
				</svg>
			</button>
			<h1 class="text-lg font-bold text-[var(--text-primary)]">
				{community?.name ?? 'Community'} Settings
			</h1>
		</div>
	</header>

	{#if loading}
		<div class="flex flex-1 items-center justify-center">
			<div class="h-8 w-8 animate-spin rounded-full border-2 border-[var(--accent)] border-t-transparent"></div>
		</div>
	{:else if community}
		<div class="mx-auto flex w-full max-w-4xl flex-1 gap-6 p-6">
			<!-- Sidebar tabs -->
			<nav class="w-48 space-y-1">
				{#each [['overview', 'Overview'], ['members', 'Members'], ['invites', 'Invites'], ['bans', 'Bans']] as [tab, label]}
					<button
						onclick={() => switchTab(tab as typeof activeTab)}
						class="w-full rounded-lg px-3 py-2 text-left text-sm transition {activeTab === tab ? 'bg-white/10 text-[var(--text-primary)]' : 'text-[var(--text-secondary)] hover:bg-white/5 hover:text-[var(--text-primary)]'}"
					>
						{label}
					</button>
				{/each}

				<div class="my-3 h-px bg-white/10"></div>

				{#if isOwner}
					<button onclick={handleDelete} class="w-full rounded-lg px-3 py-2 text-left text-sm text-[var(--danger)] transition hover:bg-[var(--danger)]/10">
						Delete Community
					</button>
				{:else}
					<button onclick={handleLeave} class="w-full rounded-lg px-3 py-2 text-left text-sm text-[var(--danger)] transition hover:bg-[var(--danger)]/10">
						Leave Community
					</button>
				{/if}
			</nav>

			<!-- Content -->
			<div class="flex-1">
				{#if activeTab === 'overview'}
					<h2 class="mb-4 text-xl font-bold text-[var(--text-primary)]">Overview</h2>

					<div class="space-y-4 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<div>
							<label class="mb-1 block text-sm text-[var(--text-secondary)]">Community Name</label>
							<input
								type="text"
								bind:value={editName}
								maxlength="64"
								disabled={!canManage}
								class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)] disabled:opacity-50"
							/>
						</div>
						<div>
							<label class="mb-1 block text-sm text-[var(--text-secondary)]">Description</label>
							<textarea
								bind:value={editDescription}
								rows="3"
								disabled={!canManage}
								class="w-full resize-none rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)] disabled:opacity-50"
							></textarea>
						</div>
						<div class="flex items-center gap-4 text-sm text-[var(--text-secondary)]">
							<span>Owner: <strong class="text-[var(--text-primary)]">{members.find((m) => m.role === 'owner')?.display_name ?? 'Unknown'}</strong></span>
							<span>Members: <strong class="text-[var(--text-primary)]">{community.member_count}</strong></span>
							<span>Created: <strong class="text-[var(--text-primary)]">{new Date(community.created_at).toLocaleDateString()}</strong></span>
						</div>
						{#if canManage}
							<div class="flex gap-2">
								<button
									onclick={handleSave}
									disabled={saving}
									class="rounded-lg bg-[var(--accent)] px-4 py-2 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:opacity-50"
								>
									{saving ? 'Saving...' : 'Save Changes'}
								</button>
								{#if isOwner}
									<button
										onclick={handleTransferOwnership}
										class="rounded-lg border border-white/10 px-4 py-2 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
									>
										Transfer Ownership
									</button>
								{/if}
							</div>
						{/if}
					</div>

				{:else if activeTab === 'members'}
					<h2 class="mb-4 text-xl font-bold text-[var(--text-primary)]">Members ({members.length})</h2>

					<div class="space-y-1">
						{#each members as member (member.user_id)}
							<div class="flex items-center gap-3 rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-4 py-3">
								<Avatar userId={member.user_id} size="sm" />
								<div class="flex-1">
									<div class="flex items-center gap-2">
										<span class="text-sm font-medium text-[var(--text-primary)]">{member.display_name}</span>
										<span class="text-xs text-[var(--text-secondary)]">@{member.username}</span>
										<span class="rounded px-1.5 py-0.5 text-xs font-medium {roleBadgeColor(member.role)}">{roleLabel(member.role)}</span>
									</div>
									{#if member.nickname}
										<div class="text-xs text-[var(--text-secondary)]">Nickname: {member.nickname}</div>
									{/if}
								</div>
								{#if canManage && member.user_id !== authStore.user?.id && member.role !== 'owner'}
									<div class="flex items-center gap-1">
										<select
											value={member.role}
											onchange={(e) => handleRoleChange(member.user_id, (e.target as HTMLSelectElement).value)}
											class="rounded border border-white/10 bg-[var(--bg-primary)] px-2 py-1 text-xs text-[var(--text-primary)] outline-none"
										>
											<option value="member">Member</option>
											<option value="moderator">Moderator</option>
											{#if isOwner}
												<option value="admin">Admin</option>
											{/if}
										</select>
										<button
											onclick={() => handleKick(member)}
											class="rounded p-1 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--danger)]"
											title="Kick"
										>
											<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
												<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" /><polyline points="16 17 21 12 16 7" /><line x1="21" y1="12" x2="9" y2="12" />
											</svg>
										</button>
										<button
											onclick={() => handleBan(member)}
											class="rounded p-1 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--danger)]"
											title="Ban"
										>
											<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
												<circle cx="12" cy="12" r="10" /><line x1="4.93" y1="4.93" x2="19.07" y2="19.07" />
											</svg>
										</button>
									</div>
								{/if}
							</div>
						{/each}
					</div>

				{:else if activeTab === 'invites'}
					<h2 class="mb-4 text-xl font-bold text-[var(--text-primary)]">Invites</h2>

					{#if canManage}
						<div class="mb-4 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-4">
							<h3 class="mb-3 text-sm font-semibold text-[var(--text-primary)]">Create Invite</h3>
							<div class="flex gap-3">
								<div class="flex-1">
									<label class="mb-1 block text-xs text-[var(--text-secondary)]">Max Uses (blank = unlimited)</label>
									<input
										type="number"
										bind:value={newInviteMaxUses}
										min="1"
										class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
									/>
								</div>
								<div class="flex-1">
									<label class="mb-1 block text-xs text-[var(--text-secondary)]">Expires In Hours (blank = never)</label>
									<input
										type="number"
										bind:value={newInviteExpiresHours}
										min="1"
										class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
									/>
								</div>
							</div>
							<button
								onclick={handleCreateInvite}
								disabled={creatingInvite}
								class="mt-3 rounded-lg bg-[var(--accent)] px-4 py-2 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:opacity-50"
							>
								{creatingInvite ? 'Creating...' : 'Create Invite'}
							</button>
						</div>
					{/if}

					<div class="space-y-1">
						{#each invites as invite (invite.id)}
							<div class="flex items-center justify-between rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-4 py-3">
								<div>
									<code class="rounded bg-white/10 px-2 py-1 text-sm font-mono text-[var(--text-primary)]">{invite.code}</code>
									<div class="mt-1 flex gap-3 text-xs text-[var(--text-secondary)]">
										<span>Uses: {invite.used_count}{invite.max_uses ? `/${invite.max_uses}` : ''}</span>
										{#if invite.expires_at}
											<span>Expires: {new Date(invite.expires_at).toLocaleString()}</span>
										{:else}
											<span>Never expires</span>
										{/if}
									</div>
								</div>
								<div class="flex items-center gap-2">
									<button
										onclick={async () => { await navigator.clipboard.writeText(invite.code); toastStore.success('Copied!'); }}
										class="rounded px-2 py-1 text-xs text-[var(--accent)] transition hover:bg-[var(--accent)]/10"
									>
										Copy
									</button>
									{#if canManage}
										<button
											onclick={() => handleDeleteInvite(invite.id)}
											class="rounded px-2 py-1 text-xs text-[var(--danger)] transition hover:bg-[var(--danger)]/10"
										>
											Delete
										</button>
									{/if}
								</div>
							</div>
						{:else}
							<p class="text-sm text-[var(--text-secondary)]">No invites yet.</p>
						{/each}
					</div>

				{:else if activeTab === 'bans'}
					<h2 class="mb-4 text-xl font-bold text-[var(--text-primary)]">Bans</h2>

					<div class="space-y-1">
						{#each bans as ban (ban.user_id)}
							<div class="flex items-center justify-between rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-4 py-3">
								<div>
									<span class="text-sm font-medium text-[var(--text-primary)]">{ban.display_name}</span>
									<span class="ml-1 text-xs text-[var(--text-secondary)]">@{ban.username}</span>
									{#if ban.reason}
										<div class="mt-1 text-xs text-[var(--text-secondary)]">Reason: {ban.reason}</div>
									{/if}
								</div>
								{#if canManage}
									<button
										onclick={() => handleUnban(ban.user_id)}
										class="rounded px-3 py-1 text-xs text-[var(--accent)] transition hover:bg-[var(--accent)]/10"
									>
										Unban
									</button>
								{/if}
							</div>
						{:else}
							<p class="text-sm text-[var(--text-secondary)]">No bans.</p>
						{/each}
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>
