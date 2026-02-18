<script lang="ts">
	import { goto } from '$app/navigation';
	import { fade, scale } from 'svelte/transition';
	import { authStore } from '$lib/stores/auth.svelte';
	import { communityStore } from '$lib/stores/communities.svelte';
	import { channelStore } from '$lib/stores/channels.svelte';
	import { groupStore } from '$lib/stores/groups.svelte';
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
		uploadCommunityIcon,
		uploadCommunityBanner,
		type Community,
		type CommunityMember,
		type CommunityInvite,
		type CommunityBan
	} from '$lib/api/communities';
	import { getPublicUrl } from '$lib/api/auth';
	import { listCommunityEmojis, uploadEmoji, deleteEmoji, type CustomEmoji } from '$lib/api/custom-emoji';
	import Avatar from '$lib/components/Avatar.svelte';
	import { onMount } from 'svelte';

	let activeTab = $state<'overview' | 'members' | 'invites' | 'bans' | 'settings' | 'theme' | 'emoji'>('overview');
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

	// Icon/banner uploads
	let iconInputEl: HTMLInputElement | undefined = $state();
	let bannerInputEl: HTMLInputElement | undefined = $state();
	let iconUploading = $state(false);
	let bannerUploading = $state(false);

	// Welcome message
	let editWelcomeMessage = $state('');

	// Policy state
	let policyGroups = $state('admin');
	let policyInvites = $state('admin');
	let communityDiscoverable = $state(true);
	let savingPolicies = $state(false);

	// Custom emoji state
	let communityEmojis = $state<CustomEmoji[]>([]);
	let emojisLoading = $state(false);
	let newEmojiShortcode = $state('');
	let emojiFileInputEl: HTMLInputElement | undefined = $state();
	let uploadingEmoji = $state(false);

	// Theme state
	let themeAccent = $state('');
	let themeAccentHover = $state('');
	let themeBgPrimary = $state('');
	let themeBgSecondary = $state('');
	let themeBgTertiary = $state('');
	let themeTextPrimary = $state('');
	let themeTextSecondary = $state('');
	let themeCustomCss = $state('');
	let savingTheme = $state(false);

	let canManage = $derived(myRole === 'owner' || myRole === 'admin' || authStore.user?.is_owner);
	let isOwner = $derived(myRole === 'owner' || authStore.user?.is_owner);

	// Check community policies for who can create invites/groups (mirrors server meets_policy)
	function meetsPolicy(role: string | null, policy: string): boolean {
		if (!role) return false;
		const level = role === 'owner' ? 3 : role === 'admin' ? 2 : role === 'moderator' ? 1 : 0;
		const required = policy === 'everyone' ? 0 : policy === 'moderator' ? 1 : 2;
		return level >= required || !!authStore.user?.is_owner;
	}
	let canCreateInvites = $derived(meetsPolicy(myRole, community?.who_can_create_invites ?? 'admin'));

	// Confirm dialog
	let confirmDialog = $state<{
		title: string; message: string; confirmLabel: string; danger?: boolean;
		inputPlaceholder?: string;
		onConfirm: (inputValue?: string) => void;
	} | null>(null);
	let confirmInput = $state('');

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
			policyGroups = c.who_can_create_groups ?? 'admin';
			policyInvites = c.who_can_create_invites ?? 'admin';
			communityDiscoverable = c.discoverable ?? true;
			editWelcomeMessage = c.welcome_message ?? '';

			// Load theme values
			const t = c.community_theme;
			themeAccent = t?.accent ?? '';
			themeAccentHover = t?.accentHover ?? '';
			themeBgPrimary = t?.bgPrimary ?? '';
			themeBgSecondary = t?.bgSecondary ?? '';
			themeBgTertiary = t?.bgTertiary ?? '';
			themeTextPrimary = t?.textPrimary ?? '';
			themeTextSecondary = t?.textSecondary ?? '';
			themeCustomCss = t?.customCss ?? '';

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
			const updated = await updateCommunity(community.id, {
				name,
				description: editDescription.trim()
			});
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
			const link = `${getPublicUrl()}/invite/${invite.code}`;
			await navigator.clipboard.writeText(link);
			toastStore.success('Invite link copied!');
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

	function handleDeleteInvite(inviteId: string) {
		if (!community) return;
		confirmDialog = {
			title: 'Delete invite?',
			message: 'Delete this invite link? This cannot be undone.',
			confirmLabel: 'Delete',
			danger: true,
			async onConfirm() {
				try {
					await deleteInvite(community!.id, inviteId);
					invites = invites.filter((i) => i.id !== inviteId);
					toastStore.success('Invite deleted');
				} catch (err: any) {
					toastStore.error(err?.message || 'Failed to delete invite');
				}
			}
		};
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

	function handleKick(member: CommunityMember) {
		if (!community) return;
		confirmDialog = {
			title: `Kick ${member.display_name}?`,
			message: `Remove ${member.display_name} from ${community.name}? They can rejoin via invite.`,
			confirmLabel: 'Kick',
			danger: true,
			async onConfirm() {
				try {
					await kickMember(community!.id, member.user_id);
					members = members.filter((m) => m.user_id !== member.user_id);
					toastStore.success(`Kicked ${member.display_name}`);
				} catch (err: any) {
					toastStore.error(err?.message || 'Failed to kick member');
				}
			}
		};
	}

	function handleBan(member: CommunityMember) {
		if (!community) return;
		confirmInput = '';
		confirmDialog = {
			title: `Ban ${member.display_name}?`,
			message: `Ban ${member.display_name} from ${community.name}? They will not be able to rejoin.`,
			confirmLabel: 'Ban',
			danger: true,
			inputPlaceholder: 'Ban reason (optional)',
			async onConfirm(reason) {
				try {
					await banMember(community!.id, member.user_id, reason || undefined);
					members = members.filter((m) => m.user_id !== member.user_id);
					toastStore.success(`Banned ${member.display_name}`);
				} catch (err: any) {
					toastStore.error(err?.message || 'Failed to ban member');
				}
			}
		};
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

	function handleDelete() {
		if (!community) return;
		confirmDialog = {
			title: `Delete "${community.name}"?`,
			message: 'This will permanently delete ALL groups, channels, and messages. This cannot be undone.',
			confirmLabel: 'Delete',
			danger: true,
			async onConfirm() {
				try {
					const cid = community!.id;
					await deleteCommunity(cid);
					// Clean up groups and channels for this community from local stores
					for (const g of groupStore.groups.filter(g => g.community_id === cid)) {
						channelStore.removeChannelsForGroup(g.id);
						groupStore.removeGroup(g.id);
					}
					communityStore.removeCommunity(cid);
					const communities = await listCommunities();
					communityStore.setCommunities(communities);
					toastStore.success('Community deleted');
					goto('/channels');
				} catch (err: any) {
					toastStore.error(err?.message || 'Failed to delete community');
				}
			}
		};
	}

	function handleLeave() {
		if (!community) return;
		confirmDialog = {
			title: `Leave "${community.name}"?`,
			message: 'You will need a new invite to rejoin this community.',
			confirmLabel: 'Leave',
			danger: true,
			async onConfirm() {
				try {
					const cid = community!.id;
					await leaveCommunity(cid);
					// Clean up groups and channels for this community from local stores
					for (const g of groupStore.groups.filter(g => g.community_id === cid)) {
						channelStore.removeChannelsForGroup(g.id);
						groupStore.removeGroup(g.id);
					}
					communityStore.removeCommunity(cid);
					const communities = await listCommunities();
					communityStore.setCommunities(communities);
					toastStore.success(`Left "${community!.name}"`);
					goto('/channels');
				} catch (err: any) {
					toastStore.error(err?.message || 'Failed to leave community');
				}
			}
		};
	}

	function handleTransferOwnership() {
		if (!community) return;
		confirmInput = '';
		confirmDialog = {
			title: 'Transfer ownership',
			message: 'Enter the user ID of the new owner. This will make them the owner and demote you to admin.',
			confirmLabel: 'Transfer',
			danger: true,
			inputPlaceholder: 'User ID',
			async onConfirm(newOwnerId) {
				if (!newOwnerId) return;
				try {
					await transferOwnership(community!.id, newOwnerId.trim());
					await loadData();
					toastStore.success('Ownership transferred');
				} catch (err: any) {
					toastStore.error(err?.message || 'Failed to transfer ownership');
				}
			}
		};
	}

	async function handleIconUpload(e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file || !community) return;
		iconUploading = true;
		try {
			const updated = await uploadCommunityIcon(community.id, file);
			community = updated;
			communityStore.updateCommunity(community.id, { icon_url: updated.icon_url });
			toastStore.success('Icon updated');
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to upload icon');
		} finally {
			iconUploading = false;
			if (iconInputEl) iconInputEl.value = '';
		}
	}

	async function handleBannerUpload(e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file || !community) return;
		bannerUploading = true;
		try {
			const updated = await uploadCommunityBanner(community.id, file);
			community = updated;
			communityStore.updateCommunity(community.id, { banner_url: updated.banner_url });
			toastStore.success('Banner updated');
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to upload banner');
		} finally {
			bannerUploading = false;
			if (bannerInputEl) bannerInputEl.value = '';
		}
	}

	async function handleSavePolicies() {
		if (!community) return;
		savingPolicies = true;
		try {
			const updated = await updateCommunity(community.id, {
				who_can_create_groups: policyGroups,
				who_can_create_invites: policyInvites,
				discoverable: communityDiscoverable
			});
			community = updated;
			communityStore.updateCommunity(community.id, {
				who_can_create_groups: updated.who_can_create_groups,
				who_can_create_invites: updated.who_can_create_invites,
				discoverable: updated.discoverable
			});
			toastStore.success('Settings updated');
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to update policies');
		} finally {
			savingPolicies = false;
		}
	}

	async function handleSaveTheme() {
		if (!community) return;
		savingTheme = true;
		try {
			const theme: Record<string, string> = {};
			if (themeAccent) theme.accent = themeAccent;
			if (themeAccentHover) theme.accentHover = themeAccentHover;
			if (themeBgPrimary) theme.bgPrimary = themeBgPrimary;
			if (themeBgSecondary) theme.bgSecondary = themeBgSecondary;
			if (themeBgTertiary) theme.bgTertiary = themeBgTertiary;
			if (themeTextPrimary) theme.textPrimary = themeTextPrimary;
			if (themeTextSecondary) theme.textSecondary = themeTextSecondary;
			if (themeCustomCss.trim()) theme.customCss = themeCustomCss.trim();

			const updated = await updateCommunity(community.id, {
				community_theme: Object.keys(theme).length > 0 ? theme : null
			});
			community = updated;
			communityStore.updateCommunity(community.id, { community_theme: updated.community_theme });
			communityStore.applyCommunityTheme(updated.community_theme);
			toastStore.success('Theme saved');
		} catch (err: any) {
			toastStore.error(err?.message || 'Failed to save theme');
		} finally {
			savingTheme = false;
		}
	}

	function handleResetTheme() {
		themeAccent = '';
		themeAccentHover = '';
		themeBgPrimary = '';
		themeBgSecondary = '';
		themeBgTertiary = '';
		themeTextPrimary = '';
		themeTextSecondary = '';
		themeCustomCss = '';
	}

	async function loadEmojis() {
		if (!community) return;
		emojisLoading = true;
		try {
			communityEmojis = await listCommunityEmojis(community.id);
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to load emojis');
		} finally {
			emojisLoading = false;
		}
	}

	async function handleUploadEmoji() {
		if (!community || !emojiFileInputEl?.files?.[0] || !newEmojiShortcode.trim()) return;
		const file = emojiFileInputEl.files[0];
		if (file.size > 256 * 1024) {
			toastStore.error('Emoji must be under 256 KB');
			return;
		}
		uploadingEmoji = true;
		try {
			const emoji = await uploadEmoji(community.id, newEmojiShortcode.trim(), file);
			communityEmojis = [...communityEmojis, emoji];
			newEmojiShortcode = '';
			emojiFileInputEl.value = '';
			toastStore.success(`Added :${emoji.shortcode}:`);
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to upload emoji');
		} finally {
			uploadingEmoji = false;
		}
	}

	function handleDeleteEmoji(emoji: CustomEmoji) {
		if (!community) return;
		confirmDialog = {
			title: `Delete :${emoji.shortcode}:?`,
			message: 'This custom emoji will be removed. This cannot be undone.',
			confirmLabel: 'Delete',
			danger: true,
			async onConfirm() {
				try {
					await deleteEmoji(community!.id, emoji.id);
					communityEmojis = communityEmojis.filter(e => e.id !== emoji.id);
					toastStore.success(`Removed :${emoji.shortcode}:`);
				} catch (err) {
					toastStore.error(err instanceof Error ? err.message : 'Failed to delete emoji');
				}
			}
		};
	}

	function switchTab(tab: typeof activeTab) {
		activeTab = tab;
		// Always reload data when switching to these tabs for freshness
		if (tab === 'invites') loadInvites();
		if (tab === 'bans') loadBans();
		if (tab === 'emoji') loadEmojis();
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
			<button aria-label="Back to channels" onclick={() => goto('/channels')} class="text-[var(--text-secondary)] transition hover:text-[var(--text-primary)]">
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
		<div class="mx-auto flex w-full max-w-4xl flex-1 flex-col md:flex-row gap-4 md:gap-6 p-4 md:p-6">
			<!-- Mobile tab bar -->
			<div class="flex gap-1 overflow-x-auto pb-1 md:hidden">
				{#each [['overview', 'Overview'], ['members', 'Members'], ['invites', 'Invites'], ['bans', 'Bans'], ...(canManage ? [['settings', 'Settings'], ['theme', 'Theme'], ['emoji', 'Emoji']] : [])] as [tab, label]}
					<button
						onclick={() => switchTab(tab as typeof activeTab)}
						class="shrink-0 rounded-lg px-3 py-1.5 text-xs font-medium transition {activeTab === tab ? 'bg-white/10 text-[var(--text-primary)]' : 'text-[var(--text-secondary)] hover:bg-white/5'}"
					>
						{label}
					</button>
				{/each}
			</div>
			<!-- Sidebar tabs (desktop only) -->
			<nav class="hidden md:block w-48 shrink-0 space-y-1">
				{#each [['overview', 'Overview'], ['members', 'Members'], ['invites', 'Invites'], ['bans', 'Bans'], ...(canManage ? [['settings', 'Settings'], ['theme', 'Theme'], ['emoji', 'Emoji']] : [])] as [tab, label]}
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
							<label for="edit-community-name" class="mb-1 block text-sm text-[var(--text-secondary)]">Community Name</label>
							<input id="edit-community-name"
								type="text"
								bind:value={editName}
								maxlength="64"
								disabled={!canManage}
								class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)] disabled:opacity-50"
							/>
						</div>
						<div>
							<label for="edit-community-desc" class="mb-1 block text-sm text-[var(--text-secondary)]">Description</label>
							<textarea id="edit-community-desc"
								bind:value={editDescription}
								rows="3"
								maxlength={2048}
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

					{#if canManage}
						<!-- Icon & Banner -->
						<div class="mt-6 grid grid-cols-1 sm:grid-cols-2 gap-4">
							<!-- Icon upload -->
							<div class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-4">
								<h3 class="mb-3 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Community Icon</h3>
								<div class="group relative mx-auto h-20 w-20 overflow-hidden rounded-full border border-white/10">
									{#if community.icon_url}
										<img src={community.icon_url} alt="Community icon" class="h-full w-full object-cover" />
									{:else}
										<div class="flex h-full w-full items-center justify-center bg-[var(--bg-primary)] text-2xl font-bold text-[var(--text-secondary)]">
											{community.name.charAt(0).toUpperCase()}
										</div>
									{/if}
									<button
										aria-label="Upload community icon"
										onclick={() => iconInputEl?.click()}
										disabled={iconUploading}
										class="absolute inset-0 flex items-center justify-center bg-black/50 opacity-0 transition group-hover:opacity-100 disabled:cursor-wait"
									>
										<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
											<path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z" /><circle cx="12" cy="13" r="4" />
										</svg>
									</button>
									<input bind:this={iconInputEl} type="file" accept="image/png,image/jpeg,image/webp,image/gif" onchange={handleIconUpload} class="hidden" />
								</div>
								<p class="mt-2 text-center text-xs text-[var(--text-secondary)]">Max 2 MB</p>
							</div>

							<!-- Banner upload -->
							<div class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-4">
								<h3 class="mb-3 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Community Banner</h3>
								<div class="group relative aspect-[3/1] w-full overflow-hidden rounded-lg border border-white/10">
									{#if community.banner_url}
										<img src={community.banner_url} alt="Community banner" class="h-full w-full object-cover" />
									{:else}
										<div class="flex h-full w-full items-center justify-center bg-gradient-to-r from-[var(--accent)] to-[var(--accent-hover)]">
											<span class="text-xs text-white/50">No banner</span>
										</div>
									{/if}
									<button
										aria-label="Upload community banner"
										onclick={() => bannerInputEl?.click()}
										disabled={bannerUploading}
										class="absolute inset-0 flex items-center justify-center bg-black/50 opacity-0 transition group-hover:opacity-100 disabled:cursor-wait"
									>
										<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
											<path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z" /><circle cx="12" cy="13" r="4" />
										</svg>
									</button>
									<input bind:this={bannerInputEl} type="file" accept="image/png,image/jpeg,image/webp,image/gif" onchange={handleBannerUpload} class="hidden" />
								</div>
								<p class="mt-2 text-xs text-[var(--text-secondary)]">Max 5 MB</p>
							</div>
						</div>

						<!-- Welcome Message -->
						<div class="mt-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
							<h3 class="mb-3 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Welcome Message</h3>
							<p class="mb-3 text-xs text-[var(--text-secondary)]">Shown to new members when they first join. Leave empty to disable.</p>
							<textarea
								bind:value={editWelcomeMessage}
								rows="3"
								maxlength="2000"
								placeholder="Welcome to our community!"
								class="w-full resize-none rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
							></textarea>
							<button
								onclick={async () => {
									if (!community) return;
									saving = true;
									try {
										const updated = await updateCommunity(community.id, {
											welcome_message: editWelcomeMessage.trim()
										});
										community = updated;
										toastStore.success('Welcome message updated');
									} catch (err) {
										toastStore.error(err instanceof Error ? err.message : 'Failed to update');
									} finally {
										saving = false;
									}
								}}
								disabled={saving}
								class="mt-2 rounded-lg bg-[var(--accent)] px-4 py-2 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:opacity-50"
							>
								{saving ? 'Saving...' : 'Save Welcome Message'}
							</button>
						</div>
					{/if}

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
										aria-label="Kick member"
										>
											<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
												<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" /><polyline points="16 17 21 12 16 7" /><line x1="21" y1="12" x2="9" y2="12" />
											</svg>
										</button>
										<button
											onclick={() => handleBan(member)}
											class="rounded p-1 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--danger)]"
											title="Ban"
											aria-label="Ban member"
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

					{#if canCreateInvites}
						<div class="mb-4 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-4">
							<h3 class="mb-3 text-sm font-semibold text-[var(--text-primary)]">Create Invite</h3>
							<div class="flex gap-3">
								<div class="flex-1">
									<label for="community-invite-max-uses" class="mb-1 block text-xs text-[var(--text-secondary)]">Max Uses (blank = unlimited)</label>
									<input id="community-invite-max-uses"
										type="number"
										bind:value={newInviteMaxUses}
										min="1"
										class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
									/>
								</div>
								<div class="flex-1">
									<label for="community-invite-expires" class="mb-1 block text-xs text-[var(--text-secondary)]">Expires In Hours (blank = never)</label>
									<input id="community-invite-expires"
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
								<div class="min-w-0 flex-1">
									<code class="rounded bg-white/10 px-2 py-1 text-sm font-mono text-[var(--text-primary)]">{getPublicUrl()}/invite/{invite.code}</code>
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
										onclick={async () => { try { await navigator.clipboard.writeText(`${getPublicUrl()}/invite/${invite.code}`); toastStore.success('Link copied!'); } catch { toastStore.error('Failed to copy'); } }}
										class="rounded px-2 py-1 text-xs text-[var(--accent)] transition hover:bg-[var(--accent)]/10"
									>
										Copy Link
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

				{:else if activeTab === 'settings'}
					<h2 class="mb-4 text-xl font-bold text-[var(--text-primary)]">Settings</h2>

					<div class="space-y-6">
						<!-- Discoverability -->
						<div class="rounded-lg border border-white/10 bg-[var(--bg-secondary)] p-4">
							<div class="flex items-center justify-between">
								<div>
									<p class="text-sm font-medium text-[var(--text-primary)]">Discoverable</p>
									<p class="mt-1 text-xs text-[var(--text-secondary)]">When disabled, your community name and description are hidden from invite previews. Members can still join via direct invite links.</p>
								</div>
								<button
									aria-label="Toggle discoverability"
									onclick={() => communityDiscoverable = !communityDiscoverable}
									class="ml-4 flex h-6 w-11 shrink-0 items-center rounded-full transition-colors {communityDiscoverable ? 'bg-[var(--accent)]' : 'bg-white/20'}"
								>
									<span class="h-4 w-4 rounded-full bg-white shadow transition-transform {communityDiscoverable ? 'translate-x-6' : 'translate-x-1'}"></span>
								</button>
							</div>
						</div>

						<!-- Who can create groups -->
						<div class="rounded-lg border border-white/10 bg-[var(--bg-secondary)] p-4">
							<label for="policy-groups" class="mb-2 block text-sm font-medium text-[var(--text-primary)]">Who can create groups</label>
							<p class="mb-3 text-xs text-[var(--text-secondary)]">Controls which community members can create new groups.</p>
							<select
								id="policy-groups"
								bind:value={policyGroups}
								class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
							>
								<option value="admin">Admins & Owners</option>
								<option value="moderator">Moderators & Above</option>
								<option value="everyone">Everyone</option>
							</select>
						</div>

						<!-- Who can create invites -->
						<div class="rounded-lg border border-white/10 bg-[var(--bg-secondary)] p-4">
							<label for="policy-invites" class="mb-2 block text-sm font-medium text-[var(--text-primary)]">Who can create invites</label>
							<p class="mb-3 text-xs text-[var(--text-secondary)]">Controls which community members can create invite links.</p>
							<select
								id="policy-invites"
								bind:value={policyInvites}
								class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
							>
								<option value="admin">Admins & Owners</option>
								<option value="moderator">Moderators & Above</option>
								<option value="everyone">Everyone</option>
							</select>
						</div>

						<button
							onclick={handleSavePolicies}
							disabled={savingPolicies}
							class="rounded-lg bg-[var(--accent)] px-4 py-2 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:opacity-50"
						>
							{savingPolicies ? 'Saving...' : 'Save Settings'}
						</button>

						<!-- Role Permissions Reference -->
						<div class="rounded-lg border border-white/10 bg-[var(--bg-secondary)] p-4">
							<h3 class="mb-3 text-sm font-medium text-[var(--text-primary)]">Role Permissions Reference</h3>
							<div class="overflow-x-auto">
								<table class="w-full text-left text-xs">
									<thead>
										<tr class="border-b border-white/10 text-[var(--text-secondary)]">
											<th class="pb-2 pr-4">Permission</th>
											<th class="pb-2 px-2 text-center">Owner</th>
											<th class="pb-2 px-2 text-center">Admin</th>
											<th class="pb-2 px-2 text-center">Mod</th>
											<th class="pb-2 px-2 text-center">Member</th>
										</tr>
									</thead>
									<tbody class="text-[var(--text-primary)]">
										<tr class="border-b border-white/5">
											<td class="py-1.5 pr-4 text-[var(--text-secondary)]">Manage community</td>
											<td class="py-1.5 px-2 text-center text-green-400">Yes</td>
											<td class="py-1.5 px-2 text-center text-green-400">Yes</td>
											<td class="py-1.5 px-2 text-center text-red-400">No</td>
											<td class="py-1.5 px-2 text-center text-red-400">No</td>
										</tr>
										<tr class="border-b border-white/5">
											<td class="py-1.5 pr-4 text-[var(--text-secondary)]">Manage members</td>
											<td class="py-1.5 px-2 text-center text-green-400">Yes</td>
											<td class="py-1.5 px-2 text-center text-green-400">Yes</td>
											<td class="py-1.5 px-2 text-center text-red-400">No</td>
											<td class="py-1.5 px-2 text-center text-red-400">No</td>
										</tr>
										<tr class="border-b border-white/5">
											<td class="py-1.5 pr-4 text-[var(--text-secondary)]">Create groups</td>
											<td class="py-1.5 px-2 text-center text-green-400">Yes</td>
											<td class="py-1.5 px-2 text-center text-green-400">Yes</td>
											<td class="py-1.5 px-2 text-center {policyGroups === 'moderator' || policyGroups === 'everyone' ? 'text-green-400' : 'text-red-400'}">{policyGroups === 'moderator' || policyGroups === 'everyone' ? 'Yes' : 'No'}</td>
											<td class="py-1.5 px-2 text-center {policyGroups === 'everyone' ? 'text-green-400' : 'text-red-400'}">{policyGroups === 'everyone' ? 'Yes' : 'No'}</td>
										</tr>
										<tr class="border-b border-white/5">
											<td class="py-1.5 pr-4 text-[var(--text-secondary)]">Create invites</td>
											<td class="py-1.5 px-2 text-center text-green-400">Yes</td>
											<td class="py-1.5 px-2 text-center text-green-400">Yes</td>
											<td class="py-1.5 px-2 text-center {policyInvites === 'moderator' || policyInvites === 'everyone' ? 'text-green-400' : 'text-red-400'}">{policyInvites === 'moderator' || policyInvites === 'everyone' ? 'Yes' : 'No'}</td>
											<td class="py-1.5 px-2 text-center {policyInvites === 'everyone' ? 'text-green-400' : 'text-red-400'}">{policyInvites === 'everyone' ? 'Yes' : 'No'}</td>
										</tr>
										<tr class="border-b border-white/5">
											<td class="py-1.5 pr-4 text-[var(--text-secondary)]">Delete community</td>
											<td class="py-1.5 px-2 text-center text-green-400">Yes</td>
											<td class="py-1.5 px-2 text-center text-red-400">No</td>
											<td class="py-1.5 px-2 text-center text-red-400">No</td>
											<td class="py-1.5 px-2 text-center text-red-400">No</td>
										</tr>
										<tr>
											<td class="py-1.5 pr-4 text-[var(--text-secondary)]">Transfer ownership</td>
											<td class="py-1.5 px-2 text-center text-green-400">Yes</td>
											<td class="py-1.5 px-2 text-center text-red-400">No</td>
											<td class="py-1.5 px-2 text-center text-red-400">No</td>
											<td class="py-1.5 px-2 text-center text-red-400">No</td>
										</tr>
									</tbody>
								</table>
							</div>
						</div>
					</div>

				{:else if activeTab === 'theme'}
					<h2 class="mb-4 text-xl font-bold text-[var(--text-primary)]">Community Theme</h2>

					<div class="space-y-6">
						<!-- Live Preview -->
						<div class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-4">
							<h3 class="mb-3 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Preview</h3>
							<div class="overflow-hidden rounded-lg border border-white/10" style="background: {themeBgPrimary || 'var(--bg-primary)'}">
								<div class="flex h-8 items-center gap-2 px-3" style="background: {themeBgSecondary || 'var(--bg-secondary)'}">
									<div class="h-3 w-3 rounded-full" style="background: {themeAccent || 'var(--accent)'}"></div>
									<span class="text-xs font-medium" style="color: {themeTextPrimary || 'var(--text-primary)'}">Channel Name</span>
								</div>
								<div class="space-y-2 p-3">
									<div class="flex items-start gap-2">
										<div class="h-6 w-6 rounded-full" style="background: {themeBgTertiary || 'var(--bg-tertiary)'}"></div>
										<div>
											<span class="text-xs font-medium" style="color: {themeAccent || 'var(--accent)'}">User</span>
											<p class="text-xs" style="color: {themeTextPrimary || 'var(--text-primary)'}">Hello, this is a preview message!</p>
										</div>
									</div>
									<div class="flex items-start gap-2">
										<div class="h-6 w-6 rounded-full" style="background: {themeBgTertiary || 'var(--bg-tertiary)'}"></div>
										<div>
											<span class="text-xs font-medium" style="color: {themeTextSecondary || 'var(--text-secondary)'}">Another User</span>
											<p class="text-xs" style="color: {themeTextPrimary || 'var(--text-primary)'}">Looking good!</p>
										</div>
									</div>
								</div>
								<div class="flex h-8 items-center px-3" style="background: {themeBgTertiary || 'var(--bg-tertiary)'}">
									<span class="text-xs" style="color: {themeTextSecondary || 'var(--text-secondary)'}">Type a message...</span>
								</div>
							</div>
						</div>

						<!-- Color Pickers -->
						<div class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-4">
							<h3 class="mb-3 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Colors</h3>
							<div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
								{#each [
									['Accent', themeAccent, (v: string) => themeAccent = v] as [string, string, (v: string) => string],
									['Accent Hover', themeAccentHover, (v: string) => themeAccentHover = v] as [string, string, (v: string) => string],
									['Background Primary', themeBgPrimary, (v: string) => themeBgPrimary = v] as [string, string, (v: string) => string],
									['Background Secondary', themeBgSecondary, (v: string) => themeBgSecondary = v] as [string, string, (v: string) => string],
									['Background Tertiary', themeBgTertiary, (v: string) => themeBgTertiary = v] as [string, string, (v: string) => string],
									['Text Primary', themeTextPrimary, (v: string) => themeTextPrimary = v] as [string, string, (v: string) => string],
									['Text Secondary', themeTextSecondary, (v: string) => themeTextSecondary = v] as [string, string, (v: string) => string],
								] as [label, value, setter]}
									<div class="flex items-center gap-3">
										<input
											type="color"
											value={value || '#000000'}
											oninput={(e) => setter((e.target as HTMLInputElement).value)}
											class="h-8 w-8 cursor-pointer rounded border border-white/10 bg-transparent"
											aria-label="{label} color"
										/>
										<div class="flex-1">
											<span class="block text-xs font-medium text-[var(--text-primary)]">{label}</span>
											<div class="flex items-center gap-1">
												<input
													type="text"
													value={value}
													oninput={(e) => setter((e.target as HTMLInputElement).value)}
													placeholder="Default"
													class="w-24 rounded border border-white/10 bg-[var(--bg-primary)] px-2 py-1 text-xs text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
												/>
												{#if value}
													<button
														onclick={() => setter('')}
														class="text-xs text-[var(--text-secondary)] hover:text-[var(--danger)]"
														title="Clear"
														aria-label="Clear field"
													>&times;</button>
												{/if}
											</div>
										</div>
									</div>
								{/each}
							</div>
						</div>

						<!-- Custom CSS -->
						<div class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-4">
							<h3 class="mb-2 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Custom CSS</h3>
							<p class="mb-3 text-xs text-[var(--text-secondary)]">
								Allowed properties: color, background, border, font, spacing, shadows, opacity, text-decoration.
								No url() (except /api/ paths), no position, no @import.
								Max 4 KB.
							</p>
							<textarea
								bind:value={themeCustomCss}
								rows="6"
								maxlength="4096"
								placeholder="/* e.g. --my-custom-var: #ff0000; */"
								spellcheck="false"
								class="w-full resize-y rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2.5 font-mono text-xs text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
							></textarea>
						</div>

						<!-- Actions -->
						<div class="flex gap-2">
							<button
								onclick={handleSaveTheme}
								disabled={savingTheme}
								class="rounded-lg bg-[var(--accent)] px-4 py-2 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:opacity-50"
							>
								{savingTheme ? 'Saving...' : 'Save Theme'}
							</button>
							<button
								onclick={handleResetTheme}
								class="rounded-lg border border-white/10 px-4 py-2 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
							>
								Reset to Defaults
							</button>
						</div>
					</div>
				{:else if activeTab === 'emoji'}
					<h2 class="mb-4 text-xl font-bold text-[var(--text-primary)]">Custom Emoji</h2>
					<p class="mb-4 text-sm text-[var(--text-secondary)]">Upload custom emojis for this community. Members can use them with <code class="rounded bg-white/10 px-1">:shortcode:</code> syntax. Max 50 emojis, 256 KB each.</p>

					<!-- Upload form -->
					{#if canManage}
						<div class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-4">
							<h3 class="mb-3 text-sm font-semibold text-[var(--text-primary)]">Add Emoji</h3>
							<div class="flex flex-wrap items-end gap-3">
								<div>
									<label for="emoji-shortcode" class="mb-1 block text-xs text-[var(--text-secondary)]">Shortcode</label>
									<input
										id="emoji-shortcode"
										type="text"
										bind:value={newEmojiShortcode}
										placeholder="my_emoji"
										maxlength="32"
										pattern="[a-zA-Z0-9_]+"
										class="w-48 rounded border border-white/10 bg-[var(--bg-primary)] px-3 py-1.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
									/>
									<p class="mt-0.5 text-xs text-[var(--text-secondary)] opacity-60">2-32 chars, letters/numbers/underscores</p>
								</div>
								<div>
									<label for="emoji-file" class="mb-1 block text-xs text-[var(--text-secondary)]">Image</label>
									<input
										id="emoji-file"
										type="file"
										bind:this={emojiFileInputEl}
										accept="image/png,image/gif,image/webp"
										class="text-sm text-[var(--text-secondary)] file:mr-2 file:rounded file:border-0 file:bg-[var(--accent)] file:px-3 file:py-1 file:text-sm file:text-white file:cursor-pointer hover:file:bg-[var(--accent-hover)]"
									/>
								</div>
								<button
									onclick={handleUploadEmoji}
									disabled={uploadingEmoji || !newEmojiShortcode.trim() || !emojiFileInputEl?.files?.length}
									class="rounded-lg bg-[var(--accent)] px-4 py-1.5 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:cursor-not-allowed disabled:opacity-50"
								>
									{uploadingEmoji ? 'Uploading...' : 'Upload'}
								</button>
							</div>
						</div>
					{/if}

					<!-- Emoji grid -->
					{#if emojisLoading}
						<p class="text-sm text-[var(--text-secondary)]">Loading emojis...</p>
					{:else if communityEmojis.length === 0}
						<div class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-8 text-center">
							<p class="text-sm text-[var(--text-secondary)]">No custom emojis yet</p>
							{#if canManage}
								<p class="mt-1 text-xs text-[var(--text-secondary)] opacity-60">Upload your first emoji above!</p>
							{/if}
						</div>
					{:else}
						<div class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-4">
							<p class="mb-3 text-xs text-[var(--text-secondary)]">{communityEmojis.length}/50 emojis</p>
							<div class="grid grid-cols-2 gap-2 sm:grid-cols-3 md:grid-cols-4">
								{#each communityEmojis as emoji (emoji.id)}
									<div class="group flex items-center gap-3 rounded-lg border border-white/5 bg-[var(--bg-primary)] px-3 py-2">
										<img src={emoji.url} alt={emoji.shortcode} class="h-8 w-8 object-contain" loading="lazy" />
										<div class="min-w-0 flex-1">
											<p class="truncate text-sm font-medium text-[var(--text-primary)]">:{emoji.shortcode}:</p>
										</div>
										{#if canManage}
											<button
												onclick={() => handleDeleteEmoji(emoji)}
												class="hidden shrink-0 rounded p-1 text-[var(--text-secondary)] transition hover:bg-red-500/10 hover:text-red-400 group-hover:block"
												title="Delete emoji"
												aria-label="Delete emoji"
											>
												<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
											</button>
										{/if}
									</div>
								{/each}
							</div>
						</div>
					{/if}
				{/if}
			</div>
		</div>
	{/if}

	{#if confirmDialog}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
			role="dialog"
			tabindex="-1"
			aria-modal="true"
			aria-label={confirmDialog.title}
			transition:fade={{ duration: 150 }}
			onclick={() => { confirmDialog = null; confirmInput = ''; }}
			onkeydown={(e) => { if (e.key === 'Escape') { confirmDialog = null; confirmInput = ''; } }}
		>
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="w-full max-w-sm rounded-2xl bg-[var(--bg-secondary)] p-5 shadow-xl"
				transition:scale={{ start: 0.95, duration: 200 }}
				onclick={(e) => e.stopPropagation()}
				onkeydown={(e) => e.stopPropagation()}
			>
				<h3 class="mb-2 text-base font-bold text-[var(--text-primary)]">{confirmDialog.title}</h3>
				<p class="mb-4 text-sm text-[var(--text-secondary)]">{confirmDialog.message}</p>
				{#if confirmDialog.inputPlaceholder}
					<input
						type="text"
						bind:value={confirmInput}
						placeholder={confirmDialog.inputPlaceholder}
						onkeydown={(e) => { if (e.key === 'Enter') { confirmDialog?.onConfirm(confirmInput); confirmDialog = null; confirmInput = ''; } }}
						autofocus
						class="mb-4 w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
					/>
				{/if}
				<div class="flex justify-end gap-2">
					<button
						onclick={() => { confirmDialog = null; confirmInput = ''; }}
						class="rounded-lg px-4 py-2 text-sm font-medium text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					>
						Cancel
					</button>
					<button
						onclick={() => { confirmDialog?.onConfirm(confirmInput); confirmDialog = null; confirmInput = ''; }}
						class="rounded-lg px-4 py-2 text-sm font-medium text-white transition {confirmDialog.danger ? 'bg-[var(--danger)] hover:bg-red-600' : 'bg-[var(--accent)] hover:bg-[var(--accent-hover)]'}"
					>
						{confirmDialog.confirmLabel ?? 'Confirm'}
					</button>
				</div>
			</div>
		</div>
	{/if}
</div>
