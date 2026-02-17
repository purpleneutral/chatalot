<script lang="ts">
	import { fade, scale } from 'svelte/transition';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { groupStore } from '$lib/stores/groups.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { updateGroup as apiUpdateGroup, leaveGroup, deleteGroup, createInvite, uploadGroupIcon, uploadGroupBanner } from '$lib/api/groups';
	import type { Group } from '$lib/api/groups';

	let {
		group,
		myRole,
		anchorRect,
		onclose,
		ondeleted,
		onleft,
		oninvitecreated,
		isCommunityModerator = false,
		assignedMemberName
	}: {
		group: Group;
		myRole: string;
		anchorRect: { x: number; y: number };
		onclose: () => void;
		ondeleted?: () => void;
		onleft?: () => void;
		oninvitecreated?: (code: string) => void;
		isCommunityModerator?: boolean;
		assignedMemberName?: string;
	} = $props();

	const isAdmin = $derived(myRole === 'owner' || myRole === 'admin');

	let confirmDialog = $state<{ title: string; message: string; confirmLabel: string; danger?: boolean; onConfirm: () => void } | null>(null);
	const isOwner = $derived(myRole === 'owner');
	const isPersonal = $derived(!!group.assigned_member_id);
	const isAssignedMember = $derived(group.assigned_member_id === authStore.user?.id);
	// Assigned members can't delete; only community moderator+ can
	const canDelete = $derived(isPersonal ? isCommunityModerator : isOwner);
	// Assigned members can only invite if allow_invites is true
	const canInvite = $derived(isAdmin && (!isPersonal || group.allow_invites || isCommunityModerator));

	let editingName = $state(false);
	let editName = $state('');
	let editingDesc = $state(false);
	let editDesc = $state('');
	let saving = $state(false);
	let iconInputEl = $state<HTMLInputElement | null>(null);
	let bannerInputEl = $state<HTMLInputElement | null>(null);
	let iconUploading = $state(false);
	let bannerUploading = $state(false);
	let editingAccent = $state(false);
	let editAccentColor = $state('#5865f2');
	$effect(() => { editName = group.name; });
	$effect(() => { editDesc = group.description ?? ''; });
	$effect(() => { editAccentColor = group.accent_color ?? '#5865f2'; });

	// Position the card relative to the anchor, clamped to viewport
	let cardStyle = $derived.by(() => {
		const padding = 12;
		const estimatedW = 300;
		const estimatedH = 380;
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

	async function saveName() {
		if (!editName.trim() || editName.trim() === group.name) { editingName = false; return; }
		saving = true;
		try {
			const updated = await apiUpdateGroup(group.id, { name: editName.trim() });
			groupStore.updateGroup(group.id, updated);
			group = updated;
			editingName = false;
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to rename');
		} finally {
			saving = false;
		}
	}

	async function saveDescription() {
		saving = true;
		try {
			const updated = await apiUpdateGroup(group.id, { description: editDesc.trim() || undefined });
			groupStore.updateGroup(group.id, updated);
			group = updated;
			editingDesc = false;
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to update');
		} finally {
			saving = false;
		}
	}

	async function toggleDiscoverable() {
		saving = true;
		try {
			const updated = await apiUpdateGroup(group.id, { discoverable: !group.discoverable });
			groupStore.updateGroup(group.id, updated);
			group = updated;
			toastStore.success(updated.discoverable ? 'Group is now discoverable' : 'Group is now hidden');
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to update');
		} finally {
			saving = false;
		}
	}

	async function toggleVisibility() {
		const newVis = group.visibility === 'public' ? 'private' : 'public';
		saving = true;
		try {
			const updated = await apiUpdateGroup(group.id, { visibility: newVis });
			groupStore.updateGroup(group.id, updated);
			group = updated;
			toastStore.success(`Group is now ${newVis}`);
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to update');
		} finally {
			saving = false;
		}
	}

	async function toggleAllowInvites() {
		saving = true;
		try {
			const updated = await apiUpdateGroup(group.id, { allow_invites: !group.allow_invites });
			groupStore.updateGroup(group.id, updated);
			group = updated;
			toastStore.success(updated.allow_invites ? 'Member can now create invites' : 'Member can no longer create invites');
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to update');
		} finally {
			saving = false;
		}
	}

	async function handleCreateInvite() {
		try {
			const invite = await createInvite(group.id);
			await navigator.clipboard.writeText(invite.code);
			toastStore.success('Invite code copied to clipboard');
			oninvitecreated?.(invite.code);
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to create invite');
		}
	}

	async function handleCopyId() {
		try {
			await navigator.clipboard.writeText(group.id);
			toastStore.success('Group ID copied');
		} catch {
			toastStore.error('Failed to copy');
		}
	}

	async function handleIconUpload(e: Event) {
		const file = (e.target as HTMLInputElement).files?.[0];
		if (!file) return;
		iconUploading = true;
		try {
			const updated = await uploadGroupIcon(group.id, file);
			groupStore.updateGroup(group.id, updated);
			group = updated;
			toastStore.success('Group icon updated');
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to upload icon');
		} finally {
			iconUploading = false;
		}
	}

	async function handleBannerUpload(e: Event) {
		const file = (e.target as HTMLInputElement).files?.[0];
		if (!file) return;
		bannerUploading = true;
		try {
			const updated = await uploadGroupBanner(group.id, file);
			groupStore.updateGroup(group.id, updated);
			group = updated;
			toastStore.success('Group banner updated');
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to upload banner');
		} finally {
			bannerUploading = false;
		}
	}

	async function saveAccentColor() {
		saving = true;
		try {
			const updated = await apiUpdateGroup(group.id, { accent_color: editAccentColor });
			groupStore.updateGroup(group.id, updated);
			group = updated;
			editingAccent = false;
			toastStore.success('Accent color updated');
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to update accent color');
		} finally {
			saving = false;
		}
	}

	async function clearAccentColor() {
		saving = true;
		try {
			const updated = await apiUpdateGroup(group.id, { accent_color: '' });
			groupStore.updateGroup(group.id, updated);
			group = updated;
			editingAccent = false;
			toastStore.success('Accent color cleared');
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to clear accent color');
		} finally {
			saving = false;
		}
	}

	function handleLeave() {
		confirmDialog = {
			title: 'Leave group?',
			message: `Leave "${group.name}"? You can rejoin later if the group is still available.`,
			confirmLabel: 'Leave',
			danger: true,
			async onConfirm() {
				try {
					await leaveGroup(group.id);
					toastStore.success('Left group');
					onleft?.();
					onclose();
				} catch (err: any) {
					toastStore.error(err?.message ?? 'Failed to leave');
				}
			}
		};
	}

	function handleDelete() {
		confirmDialog = {
			title: `Delete "${group.name}"?`,
			message: 'This will permanently delete this group and all its channels. This cannot be undone.',
			confirmLabel: 'Delete',
			danger: true,
			async onConfirm() {
				try {
					await deleteGroup(group.id);
					toastStore.success('Group deleted');
					ondeleted?.();
					onclose();
				} catch (err: any) {
					toastStore.error(err?.message ?? 'Failed to delete');
				}
			}
		};
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
		class="fixed z-50 w-[300px] rounded-xl border border-white/10 bg-[var(--bg-secondary)] shadow-2xl"
		style={cardStyle}
		onclick={(e) => e.stopPropagation()}
		onkeydown={(e) => e.stopPropagation()}
		transition:scale={{ start: 0.9, duration: 150 }}
	>
		<!-- Banner -->
		<div class="relative h-20 overflow-hidden rounded-t-xl" style={group.accent_color ? `background: linear-gradient(135deg, ${group.accent_color}, ${group.accent_color}88)` : ''}>
			{#if group.banner_url}
				<img src={group.banner_url} alt="Group banner" class="h-full w-full object-cover" onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }} />
			{:else if !group.accent_color}
				<div class="h-full w-full bg-gradient-to-r from-[var(--accent)] to-[var(--accent-hover)]"></div>
			{/if}
			<!-- Group icon overlapping the banner -->
			<div class="absolute -bottom-5 left-4">
				{#if group.icon_url}
					<img src={group.icon_url} alt="Group icon" class="h-10 w-10 rounded-full border-2 border-[var(--bg-secondary)] object-cover" />
				{:else}
					<div class="flex h-10 w-10 items-center justify-center rounded-full border-2 border-[var(--bg-secondary)] bg-[var(--bg-tertiary)]">
						<span class="text-sm font-bold text-[var(--text-secondary)]">#</span>
					</div>
				{/if}
			</div>
		</div>

		<!-- Hidden file inputs -->
		<input bind:this={iconInputEl} type="file" accept="image/*" class="hidden" onchange={handleIconUpload} />
		<input bind:this={bannerInputEl} type="file" accept="image/*" class="hidden" onchange={handleBannerUpload} />

		<!-- Content -->
		<div class="px-4 pb-4 pt-7">
			<!-- Group Name -->
			{#if editingName && isAdmin}
				<div class="mb-2 flex gap-1">
					<input
						class="flex-1 rounded border border-white/20 bg-[var(--bg-primary)] px-2 py-1 text-sm text-[var(--text-primary)]"
						bind:value={editName}
						onkeydown={(e) => { if (e.key === 'Enter') saveName(); if (e.key === 'Escape') editingName = false; }}
					/>
					<button onclick={saveName} class="rounded bg-[var(--accent)] px-2 py-1 text-xs text-white disabled:opacity-50 disabled:cursor-not-allowed" disabled={saving}>Save</button>
				</div>
			{:else if isAdmin}
				<button class="mb-0.5 text-left text-base font-bold text-[var(--text-primary)] cursor-pointer hover:underline" onclick={() => { editingName = true; editName = group.name; }}>
					{group.name}
				</button>
			{:else}
				<h3 class="mb-0.5 text-base font-bold text-[var(--text-primary)]">{group.name}</h3>
			{/if}

			<!-- Meta line -->
			<p class="mb-2 text-xs text-[var(--text-secondary)]">
				{group.member_count} member{group.member_count !== 1 ? 's' : ''}
				<span class="mx-1">·</span>
				<span class="inline-flex items-center gap-0.5">
					{#if group.visibility === 'private'}
						<svg xmlns="http://www.w3.org/2000/svg" class="inline h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
						Private
					{:else}
						<svg xmlns="http://www.w3.org/2000/svg" class="inline h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
						Public
					{/if}
				</span>
			</p>

			<!-- Personal group indicator -->
			{#if isPersonal}
				<div class="mb-2 flex items-center gap-1.5 rounded-lg bg-[var(--accent)]/10 px-2.5 py-1.5 text-xs text-[var(--accent)]">
					<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
					Personal space{#if assignedMemberName}&nbsp;of <strong>{assignedMemberName}</strong>{/if}
				</div>
			{/if}

			<!-- Description -->
			{#if editingDesc && isAdmin}
				<div class="mb-2">
					<textarea
						class="w-full rounded border border-white/20 bg-[var(--bg-primary)] px-2 py-1 text-sm text-[var(--text-primary)]"
						rows="2"
						bind:value={editDesc}
						onkeydown={(e) => { if (e.key === 'Escape') editingDesc = false; }}
					></textarea>
					<div class="mt-1 flex justify-end gap-1">
						<button onclick={() => editingDesc = false} class="rounded px-2 py-0.5 text-xs text-[var(--text-secondary)] hover:bg-white/5">Cancel</button>
						<button onclick={saveDescription} class="rounded bg-[var(--accent)] px-2 py-0.5 text-xs text-white disabled:opacity-50 disabled:cursor-not-allowed" disabled={saving}>Save</button>
					</div>
				</div>
			{:else if group.description && isAdmin}
				<button class="mb-2 text-left text-sm text-[var(--text-secondary)] cursor-pointer hover:underline" onclick={() => { editingDesc = true; editDesc = group.description ?? ''; }}>
					{group.description}
				</button>
			{:else if group.description}
				<p class="mb-2 text-sm text-[var(--text-secondary)]">{group.description}</p>
			{:else if isAdmin}
				<button
					onclick={() => { editingDesc = true; editDesc = ''; }}
					class="mb-2 text-xs text-[var(--text-secondary)] hover:text-[var(--accent)]"
				>
					+ Add description
				</button>
			{/if}

			<!-- Customization (admin only) -->
			{#if isAdmin}
				<div class="mb-2 flex flex-wrap gap-1.5">
					<button
						onclick={() => iconInputEl?.click()}
						disabled={iconUploading}
						class="rounded-lg bg-white/5 px-2 py-1 text-xs text-[var(--text-secondary)] transition hover:bg-white/10 hover:text-[var(--text-primary)] disabled:opacity-50 disabled:cursor-not-allowed"
					>
						{iconUploading ? 'Uploading...' : 'Change Icon'}
					</button>
					<button
						onclick={() => bannerInputEl?.click()}
						disabled={bannerUploading}
						class="rounded-lg bg-white/5 px-2 py-1 text-xs text-[var(--text-secondary)] transition hover:bg-white/10 hover:text-[var(--text-primary)] disabled:opacity-50 disabled:cursor-not-allowed"
					>
						{bannerUploading ? 'Uploading...' : 'Change Banner'}
					</button>
					<button
						onclick={() => { editingAccent = !editingAccent; editAccentColor = group.accent_color ?? '#5865f2'; }}
						class="rounded-lg bg-white/5 px-2 py-1 text-xs text-[var(--text-secondary)] transition hover:bg-white/10 hover:text-[var(--text-primary)]"
					>
						Accent Color
					</button>
				</div>
				{#if editingAccent}
					<div class="mb-2 flex items-center gap-2 rounded-lg border border-white/10 bg-white/5 p-2">
						<input type="color" bind:value={editAccentColor} class="h-7 w-7 cursor-pointer rounded border-0 bg-transparent" aria-label="Accent color" />
						<span class="flex-1 text-xs text-[var(--text-secondary)]">{editAccentColor}</span>
						<button onclick={saveAccentColor} disabled={saving} class="rounded bg-[var(--accent)] px-2 py-0.5 text-xs text-white">Save</button>
						{#if group.accent_color}
							<button onclick={clearAccentColor} disabled={saving} class="rounded px-2 py-0.5 text-xs text-[var(--text-secondary)] hover:text-[var(--danger)]">Clear</button>
						{/if}
					</div>
				{/if}
			{/if}

			<div class="mb-2 border-t border-white/10"></div>

			<!-- Admin Controls -->
			{#if isAdmin}
				<!-- Visibility toggle -->
				<button
					onclick={toggleVisibility}
					class="flex w-full items-center gap-2 rounded-lg px-3 py-1.5 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					disabled={saving}
				>
					{#if group.visibility === 'public'}
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
						Make Private
					{:else}
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
						Make Public
					{/if}
				</button>

				<!-- Discoverable toggle -->
				<button
					onclick={toggleDiscoverable}
					class="flex w-full items-center justify-between rounded-lg px-3 py-1.5 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					disabled={saving}
				>
					<span class="flex items-center gap-2">
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
						Discoverable
					</span>
					<span class="rounded-full px-2 py-0.5 text-xs {group.discoverable ? 'bg-[var(--accent)]/20 text-[var(--accent)]' : 'bg-white/10 text-[var(--text-secondary)]'}">
						{group.discoverable ? 'ON' : 'OFF'}
					</span>
				</button>

				<!-- Allow Invites toggle (moderator-only, personal groups only) -->
				{#if isPersonal && isCommunityModerator}
					<button
						onclick={toggleAllowInvites}
						class="flex w-full items-center justify-between rounded-lg px-3 py-1.5 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
						disabled={saving}
					>
						<span class="flex items-center gap-2">
							<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="8.5" cy="7" r="4"/><line x1="20" y1="8" x2="20" y2="14"/><line x1="23" y1="11" x2="17" y2="11"/></svg>
							Allow Invites
						</span>
						<span class="rounded-full px-2 py-0.5 text-xs {group.allow_invites ? 'bg-[var(--accent)]/20 text-[var(--accent)]' : 'bg-white/10 text-[var(--text-secondary)]'}">
							{group.allow_invites ? 'ON' : 'OFF'}
						</span>
					</button>
				{/if}

				<!-- Create Invite (hidden for assigned member when allow_invites is off) -->
				{#if canInvite}
					<button
						onclick={handleCreateInvite}
						class="flex w-full items-center gap-2 rounded-lg px-3 py-1.5 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><polyline points="22,6 12,13 2,6"/></svg>
						Create Invite
					</button>
				{/if}
			{/if}

			<!-- Copy ID -->
			<button
				onclick={handleCopyId}
				class="flex w-full items-center gap-2 rounded-lg px-3 py-1.5 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
			>
				<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2" /><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" /></svg>
				Copy Group ID
			</button>

			<div class="mb-1 mt-1 border-t border-white/10"></div>

			<!-- Leave / Delete -->
			{#if !isOwner && !isAssignedMember}
				<button
					onclick={handleLeave}
					class="flex w-full items-center gap-2 rounded-lg px-3 py-1.5 text-sm text-[var(--danger)] transition hover:bg-[var(--danger)]/10"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16,17 21,12 16,7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>
					Leave Group
				</button>
			{/if}
			{#if canDelete}
				<button
					onclick={handleDelete}
					class="flex w-full items-center gap-2 rounded-lg px-3 py-1.5 text-sm text-[var(--danger)] transition hover:bg-[var(--danger)]/10"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3,6 5,6 21,6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
					Delete Group
				</button>
			{/if}
		</div>
	</div>
</div>

{#if confirmDialog}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="fixed inset-0 z-[60] flex items-center justify-center bg-black/60 p-4" role="dialog" tabindex="-1" aria-modal="true" aria-label={confirmDialog.title} transition:fade={{ duration: 150 }} onclick={() => confirmDialog = null} onkeydown={(e) => { if (e.key === 'Escape') confirmDialog = null; }}>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="w-full max-w-sm rounded-2xl bg-[var(--bg-secondary)] p-5 shadow-xl" transition:scale={{ start: 0.95, duration: 200 }} onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
			<h3 class="mb-2 text-base font-bold text-[var(--text-primary)]">{confirmDialog.title}</h3>
			<p class="mb-4 text-sm text-[var(--text-secondary)]">{confirmDialog.message}</p>
			<div class="flex justify-end gap-2">
				<button onclick={() => confirmDialog = null} class="rounded-lg px-4 py-2 text-sm font-medium text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]">Cancel</button>
				<button onclick={() => { confirmDialog?.onConfirm(); confirmDialog = null; }} class="rounded-lg px-4 py-2 text-sm font-medium text-white transition {confirmDialog.danger ? 'bg-[var(--danger)] hover:bg-red-600' : 'bg-[var(--accent)] hover:bg-[var(--accent-hover)]'}">{confirmDialog.confirmLabel}</button>
			</div>
		</div>
	</div>
{/if}
