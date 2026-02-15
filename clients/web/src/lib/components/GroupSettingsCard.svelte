<script lang="ts">
	import { scale } from 'svelte/transition';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { groupStore } from '$lib/stores/groups.svelte';
	import { updateGroup as apiUpdateGroup, leaveGroup, deleteGroup, createInvite } from '$lib/api/groups';
	import type { Group } from '$lib/api/groups';

	let {
		group,
		myRole,
		anchorRect,
		onclose,
		ondeleted,
		onleft,
		oninvitecreated
	}: {
		group: Group;
		myRole: string;
		anchorRect: { x: number; y: number };
		onclose: () => void;
		ondeleted?: () => void;
		onleft?: () => void;
		oninvitecreated?: (code: string) => void;
	} = $props();

	const isAdmin = $derived(myRole === 'owner' || myRole === 'admin');
	const isOwner = $derived(myRole === 'owner');

	let editingName = $state(false);
	let editName = $state(group.name);
	let editingDesc = $state(false);
	let editDesc = $state(group.description ?? '');
	let saving = $state(false);

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

	function handleCopyId() {
		navigator.clipboard.writeText(group.id);
		toastStore.success('Group ID copied');
	}

	async function handleLeave() {
		if (!confirm('Leave this group?')) return;
		try {
			await leaveGroup(group.id);
			toastStore.success('Left group');
			onleft?.();
			onclose();
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to leave');
		}
	}

	async function handleDelete() {
		if (!confirm(`Delete "${group.name}"? This cannot be undone.`)) return;
		try {
			await deleteGroup(group.id);
			toastStore.success('Group deleted');
			ondeleted?.();
			onclose();
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to delete');
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
		class="fixed z-50 w-[300px] rounded-xl border border-white/10 bg-[var(--bg-secondary)] shadow-2xl"
		style={cardStyle}
		onclick={(e) => e.stopPropagation()}
		onkeydown={(e) => e.stopPropagation()}
		transition:scale={{ start: 0.9, duration: 150 }}
	>
		<!-- Banner -->
		<div class="relative h-14 rounded-t-xl bg-gradient-to-r from-[var(--accent)] to-[var(--accent-hover)]">
			<div class="absolute bottom-2 left-4">
				<span class="text-lg font-bold text-white/90">#</span>
			</div>
		</div>

		<!-- Content -->
		<div class="px-4 pb-4 pt-3">
			<!-- Group Name -->
			{#if editingName && isAdmin}
				<div class="mb-2 flex gap-1">
					<input
						class="flex-1 rounded border border-white/20 bg-[var(--bg-primary)] px-2 py-1 text-sm text-[var(--text-primary)]"
						bind:value={editName}
						onkeydown={(e) => { if (e.key === 'Enter') saveName(); if (e.key === 'Escape') editingName = false; }}
					/>
					<button onclick={saveName} class="rounded bg-[var(--accent)] px-2 py-1 text-xs text-white" disabled={saving}>Save</button>
				</div>
			{:else}
				<h3
					class="mb-0.5 text-base font-bold text-[var(--text-primary)] {isAdmin ? 'cursor-pointer hover:underline' : ''}"
					onclick={() => { if (isAdmin) { editingName = true; editName = group.name; } }}
				>
					{group.name}
				</h3>
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
						<button onclick={saveDescription} class="rounded bg-[var(--accent)] px-2 py-0.5 text-xs text-white" disabled={saving}>Save</button>
					</div>
				</div>
			{:else if group.description}
				<p
					class="mb-2 text-sm text-[var(--text-secondary)] {isAdmin ? 'cursor-pointer hover:underline' : ''}"
					onclick={() => { if (isAdmin) { editingDesc = true; editDesc = group.description ?? ''; } }}
				>
					{group.description}
				</p>
			{:else if isAdmin}
				<button
					onclick={() => { editingDesc = true; editDesc = ''; }}
					class="mb-2 text-xs text-[var(--text-secondary)] hover:text-[var(--accent)]"
				>
					+ Add description
				</button>
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

				<!-- Create Invite -->
				<button
					onclick={handleCreateInvite}
					class="flex w-full items-center gap-2 rounded-lg px-3 py-1.5 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><polyline points="22,6 12,13 2,6"/></svg>
					Create Invite
				</button>
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
			{#if !isOwner}
				<button
					onclick={handleLeave}
					class="flex w-full items-center gap-2 rounded-lg px-3 py-1.5 text-sm text-[var(--danger)] transition hover:bg-[var(--danger)]/10"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16,17 21,12 16,7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>
					Leave Group
				</button>
			{/if}
			{#if isOwner}
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
