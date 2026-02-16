<script lang="ts">
	import { scale, slide } from 'svelte/transition';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { channelStore } from '$lib/stores/channels.svelte';
	import { updateChannel as apiUpdateChannel, deleteChannel } from '$lib/api/groups';
	import { createWebhook, listWebhooks, updateWebhook, deleteWebhook, type Webhook } from '$lib/api/webhooks';
	import { getPublicUrl } from '$lib/api/auth';
	import type { Channel } from '$lib/api/channels';

	let {
		channel,
		groupId,
		myRole,
		anchorRect,
		onclose,
		ondeleted,
		onupdated
	}: {
		channel: Channel;
		groupId: string;
		myRole: string;
		anchorRect: { x: number; y: number };
		onclose: () => void;
		ondeleted?: () => void;
		onupdated?: (ch: Channel) => void;
	} = $props();

	const isAdmin = $derived(myRole === 'owner' || myRole === 'admin');

	let editingName = $state(false);
	let editName = $state('');
	let editingTopic = $state(false);
	let editTopic = $state('');
	let saving = $state(false);
	$effect(() => { editName = channel.name ?? ''; });
	$effect(() => { editTopic = channel.topic ?? ''; });

	// Webhook state
	let showWebhooks = $state(false);
	let webhooks = $state<Webhook[]>([]);
	let loadingWebhooks = $state(false);
	let newWebhookName = $state('');
	let creatingWebhook = $state(false);
	let copiedTokenId = $state<string | null>(null);

	const slowModeOptions = [
		{ label: 'Off', value: 0 },
		{ label: '5s', value: 5 },
		{ label: '10s', value: 10 },
		{ label: '30s', value: 30 },
		{ label: '1m', value: 60 },
		{ label: '5m', value: 300 },
		{ label: '10m', value: 600 }
	];

	// Position the card
	let cardStyle = $derived.by(() => {
		const padding = 12;
		const estimatedW = 300;
		const estimatedH = 500;
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
		if (!editName.trim() || editName.trim() === channel.name) { editingName = false; return; }
		saving = true;
		try {
			const updated = await apiUpdateChannel(groupId, channel.id, { name: editName.trim() });
			channelStore.updateChannel(updated);
			channel = updated;
			onupdated?.(updated);
			editingName = false;
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to rename');
		} finally {
			saving = false;
		}
	}

	async function saveTopic() {
		saving = true;
		try {
			const updated = await apiUpdateChannel(groupId, channel.id, { topic: editTopic.trim() || undefined });
			channelStore.updateChannel(updated);
			channel = updated;
			onupdated?.(updated);
			editingTopic = false;
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to update');
		} finally {
			saving = false;
		}
	}

	async function toggleArchived() {
		saving = true;
		try {
			const updated = await apiUpdateChannel(groupId, channel.id, { archived: !channel.archived });
			channelStore.updateChannel(updated);
			channel = updated;
			onupdated?.(updated);
			toastStore.success(updated.archived ? 'Channel archived' : 'Channel unarchived');
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to update');
		} finally {
			saving = false;
		}
	}

	async function toggleDiscoverable() {
		saving = true;
		try {
			const updated = await apiUpdateChannel(groupId, channel.id, { discoverable: !channel.discoverable });
			channelStore.updateChannel(updated);
			channel = updated;
			onupdated?.(updated);
			toastStore.success(updated.discoverable ? 'Channel is now discoverable' : 'Channel is now hidden');
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to update');
		} finally {
			saving = false;
		}
	}

	async function toggleReadOnly() {
		saving = true;
		try {
			const updated = await apiUpdateChannel(groupId, channel.id, { read_only: !channel.read_only });
			channelStore.updateChannel(updated);
			channel = updated;
			onupdated?.(updated);
			toastStore.success(updated.read_only ? 'Channel is now read-only' : 'Channel is now writable');
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to update');
		} finally {
			saving = false;
		}
	}

	async function setSlowMode(seconds: number) {
		saving = true;
		try {
			const updated = await apiUpdateChannel(groupId, channel.id, { slow_mode_seconds: seconds });
			channelStore.updateChannel(updated);
			channel = updated;
			onupdated?.(updated);
			toastStore.success(seconds > 0 ? `Slow mode: ${slowModeOptions.find(o => o.value === seconds)?.label}` : 'Slow mode off');
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to update');
		} finally {
			saving = false;
		}
	}

	async function handleCopyId() {
		try {
			await navigator.clipboard.writeText(channel.id);
			toastStore.success('Channel ID copied');
		} catch {
			toastStore.error('Failed to copy');
		}
	}

	async function handleDelete() {
		if (!confirm(`Delete "#${channel.name}"? This cannot be undone.`)) return;
		try {
			await deleteChannel(groupId, channel.id);
			toastStore.success('Channel deleted');
			ondeleted?.();
			onclose();
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to delete');
		}
	}

	// ── Webhook functions ──

	async function toggleWebhooks() {
		showWebhooks = !showWebhooks;
		if (showWebhooks && webhooks.length === 0) {
			loadingWebhooks = true;
			try {
				webhooks = await listWebhooks(channel.id);
			} catch (err: any) {
				toastStore.error(err?.message ?? 'Failed to load webhooks');
			} finally {
				loadingWebhooks = false;
			}
		}
	}

	async function handleCreateWebhook() {
		const name = newWebhookName.trim();
		if (!name) return;
		creatingWebhook = true;
		try {
			const webhook = await createWebhook(channel.id, name);
			webhooks = [...webhooks, webhook];
			newWebhookName = '';
			toastStore.success('Webhook created');
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to create webhook');
		} finally {
			creatingWebhook = false;
		}
	}

	async function handleToggleWebhook(webhook: Webhook) {
		try {
			const updated = await updateWebhook(webhook.id, { active: !webhook.active });
			webhooks = webhooks.map(w => w.id === webhook.id ? updated : w);
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to update webhook');
		}
	}

	async function handleDeleteWebhook(webhook: Webhook) {
		if (!confirm(`Delete webhook "${webhook.name}"?`)) return;
		try {
			await deleteWebhook(webhook.id);
			webhooks = webhooks.filter(w => w.id !== webhook.id);
			toastStore.success('Webhook deleted');
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Failed to delete webhook');
		}
	}

	async function copyWebhookUrl(webhook: Webhook) {
		if (!webhook.token) {
			toastStore.error('Token only visible at creation');
			return;
		}
		const base = getPublicUrl() || window.location.origin;
		try {
			await navigator.clipboard.writeText(`${base}/api/webhooks/execute/${webhook.token}`);
			copiedTokenId = webhook.id;
			setTimeout(() => { copiedTokenId = null; }, 2000);
			toastStore.success('Webhook URL copied');
		} catch {
			toastStore.error('Failed to copy');
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
		class="fixed z-50 w-[300px] max-h-[80vh] overflow-y-auto rounded-xl border border-white/10 bg-[var(--bg-secondary)] shadow-2xl"
		style={cardStyle}
		onclick={(e) => e.stopPropagation()}
		onkeydown={(e) => e.stopPropagation()}
		transition:scale={{ start: 0.9, duration: 150 }}
	>
		<!-- Banner -->
		<div class="relative h-12 rounded-t-xl bg-gradient-to-r from-[var(--accent)] to-[var(--accent-hover)]">
			<div class="absolute bottom-2 left-4">
				<span class="text-sm font-bold text-white/90">
					{#if channel.channel_type === 'voice'}
						<svg xmlns="http://www.w3.org/2000/svg" class="inline h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/><path d="M19 10v2a7 7 0 0 1-14 0v-2"/><line x1="12" y1="19" x2="12" y2="23"/><line x1="8" y1="23" x2="16" y2="23"/></svg>
					{:else}
						#
					{/if}
				</span>
			</div>
		</div>

		<!-- Content -->
		<div class="px-4 pb-4 pt-3">
			<!-- Channel Name -->
			{#if editingName && isAdmin}
				<div class="mb-2 flex gap-1">
					<input
						class="flex-1 rounded border border-white/20 bg-[var(--bg-primary)] px-2 py-1 text-sm text-[var(--text-primary)]"
						bind:value={editName}
						onkeydown={(e) => { if (e.key === 'Enter') saveName(); if (e.key === 'Escape') editingName = false; }}
					/>
					<button onclick={saveName} class="rounded bg-[var(--accent)] px-2 py-0.5 text-xs text-white" disabled={saving}>Save</button>
				</div>
			{:else if isAdmin}
				<button class="mb-0.5 text-left text-base font-bold text-[var(--text-primary)] cursor-pointer hover:underline" onclick={() => { editingName = true; editName = channel.name ?? ''; }}>
					{channel.name ?? 'Unnamed'}
				</button>
			{:else}
				<h3 class="mb-0.5 text-base font-bold text-[var(--text-primary)]">{channel.name ?? 'Unnamed'}</h3>
			{/if}

			<p class="mb-2 text-xs text-[var(--text-secondary)]">
				{channel.channel_type === 'voice' ? 'Voice' : 'Text'} channel
				{#if channel.read_only}
					<span class="ml-1 inline-flex items-center gap-0.5 rounded bg-yellow-500/20 px-1.5 py-0.5 text-yellow-400">
						<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
						Read-only
					</span>
				{/if}
				{#if channel.archived}
					<span class="ml-1 inline-flex items-center gap-0.5 rounded bg-orange-500/20 px-1.5 py-0.5 text-orange-400">
						Archived
					</span>
				{/if}
			</p>

			<!-- Topic -->
			{#if editingTopic && isAdmin}
				<div class="mb-2">
					<textarea
						class="w-full rounded border border-white/20 bg-[var(--bg-primary)] px-2 py-1 text-sm text-[var(--text-primary)]"
						rows="2"
						bind:value={editTopic}
						onkeydown={(e) => { if (e.key === 'Escape') editingTopic = false; }}
					></textarea>
					<div class="mt-1 flex justify-end gap-1">
						<button onclick={() => editingTopic = false} class="rounded px-2 py-0.5 text-xs text-[var(--text-secondary)] hover:bg-white/5">Cancel</button>
						<button onclick={saveTopic} class="rounded bg-[var(--accent)] px-2 py-0.5 text-xs text-white" disabled={saving}>Save</button>
					</div>
				</div>
			{:else if channel.topic && isAdmin}
				<button class="mb-2 text-left text-sm text-[var(--text-secondary)] cursor-pointer hover:underline" onclick={() => { editingTopic = true; editTopic = channel.topic ?? ''; }}>
					{channel.topic}
				</button>
			{:else if channel.topic}
				<p class="mb-2 text-sm text-[var(--text-secondary)]">{channel.topic}</p>
			{:else if isAdmin}
				<button
					onclick={() => { editingTopic = true; editTopic = ''; }}
					class="mb-2 text-xs text-[var(--text-secondary)] hover:text-[var(--accent)]"
				>
					+ Set topic
				</button>
			{/if}

			<div class="mb-2 border-t border-white/10"></div>

			<!-- Admin Controls -->
			{#if isAdmin}
				<!-- Read-only toggle -->
				<button
					onclick={toggleReadOnly}
					class="flex w-full items-center justify-between rounded-lg px-3 py-1.5 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					disabled={saving}
				>
					<span class="flex items-center gap-2">
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
						Read-only
					</span>
					<span class="rounded-full px-2 py-0.5 text-xs {channel.read_only ? 'bg-[var(--accent)]/20 text-[var(--accent)]' : 'bg-white/10 text-[var(--text-secondary)]'}">
						{channel.read_only ? 'ON' : 'OFF'}
					</span>
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
					<span class="rounded-full px-2 py-0.5 text-xs {channel.discoverable ? 'bg-[var(--accent)]/20 text-[var(--accent)]' : 'bg-white/10 text-[var(--text-secondary)]'}">
						{channel.discoverable ? 'ON' : 'OFF'}
					</span>
				</button>

				<!-- Archive toggle -->
				<button
					onclick={toggleArchived}
					class="flex w-full items-center justify-between rounded-lg px-3 py-1.5 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					disabled={saving}
				>
					<span class="flex items-center gap-2">
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="21,8 21,21 3,21 3,8"/><rect x="1" y="3" width="22" height="5"/><line x1="10" y1="12" x2="14" y2="12"/></svg>
						Archive
					</span>
					<span class="rounded-full px-2 py-0.5 text-xs {channel.archived ? 'bg-yellow-500/20 text-yellow-400' : 'bg-white/10 text-[var(--text-secondary)]'}">
						{channel.archived ? 'ON' : 'OFF'}
					</span>
				</button>

				<!-- Slow mode -->
				<div class="px-3 py-1.5">
					<p class="mb-1.5 flex items-center gap-2 text-sm text-[var(--text-secondary)]">
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12,6 12,12 16,14"/></svg>
						Slow mode
					</p>
					<div class="flex flex-wrap gap-1">
						{#each slowModeOptions as opt}
							<button
								onclick={() => setSlowMode(opt.value)}
								class="rounded px-2 py-0.5 text-xs transition {channel.slow_mode_seconds === opt.value ? 'bg-[var(--accent)] text-white' : 'bg-white/10 text-[var(--text-secondary)] hover:bg-white/20'}"
								disabled={saving}
							>
								{opt.label}
							</button>
						{/each}
					</div>
				</div>

				<!-- Webhooks -->
				<div class="mb-1 mt-1 border-t border-white/10"></div>
				<button
					onclick={toggleWebhooks}
					class="flex w-full items-center justify-between rounded-lg px-3 py-1.5 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
				>
					<span class="flex items-center gap-2">
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
						Webhooks
					</span>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 transition-transform {showWebhooks ? 'rotate-90' : ''}" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5l8 7-8 7z"/></svg>
				</button>
				{#if showWebhooks}
					<div class="px-3 py-1.5 space-y-2" transition:slide={{ duration: 150 }}>
						{#if loadingWebhooks}
							<p class="text-xs text-[var(--text-secondary)]">Loading...</p>
						{:else}
							{#each webhooks as webhook (webhook.id)}
								<div class="rounded-lg bg-white/5 px-2.5 py-2">
									<div class="flex items-center justify-between">
										<span class="text-xs font-medium text-[var(--text-primary)] truncate flex-1" title={webhook.name}>{webhook.name}</span>
										<div class="flex items-center gap-1 shrink-0">
											<button
												onclick={() => handleToggleWebhook(webhook)}
												class="rounded px-1.5 py-0.5 text-[10px] transition {webhook.active ? 'bg-emerald-500/20 text-emerald-400' : 'bg-white/10 text-[var(--text-secondary)]'}"
												title={webhook.active ? 'Disable' : 'Enable'}
											>
												{webhook.active ? 'ON' : 'OFF'}
											</button>
											<button
												onclick={() => handleDeleteWebhook(webhook)}
												class="rounded p-0.5 text-[var(--text-secondary)] transition hover:text-[var(--danger)]"
												title="Delete"
												aria-label="Delete webhook"
											>
												<svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
											</button>
										</div>
									</div>
									{#if webhook.token}
										<button
											onclick={() => copyWebhookUrl(webhook)}
											class="mt-1 w-full rounded bg-white/5 px-2 py-1 text-left text-[10px] font-mono text-[var(--text-secondary)] transition hover:bg-white/10 truncate"
											title="Copy webhook URL"
										>
											{copiedTokenId === webhook.id ? 'Copied!' : 'Click to copy URL'}
										</button>
									{/if}
								</div>
							{/each}
							{#if webhooks.length === 0}
								<p class="text-[10px] text-[var(--text-secondary)]">No webhooks yet. Create one to post messages from external services.</p>
							{/if}
							<!-- Create webhook -->
							<div class="flex gap-1">
								<input
									type="text"
									bind:value={newWebhookName}
									placeholder="Webhook name..."
									maxlength="64"
									class="flex-1 rounded border border-white/10 bg-[var(--bg-primary)] px-2 py-1 text-xs text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
									onkeydown={(e) => { if (e.key === 'Enter') handleCreateWebhook(); }}
								/>
								<button
									onclick={handleCreateWebhook}
									disabled={creatingWebhook || !newWebhookName.trim()}
									class="shrink-0 rounded bg-[var(--accent)] px-2 py-1 text-xs text-white transition hover:bg-[var(--accent-hover)] disabled:opacity-50"
								>
									{creatingWebhook ? '...' : 'Add'}
								</button>
							</div>
						{/if}
					</div>
				{/if}

				<div class="mb-1 mt-1 border-t border-white/10"></div>
			{/if}

			<!-- Copy ID -->
			<button
				onclick={handleCopyId}
				class="flex w-full items-center gap-2 rounded-lg px-3 py-1.5 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
			>
				<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2" /><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" /></svg>
				Copy Channel ID
			</button>

			<!-- Delete (admin only) -->
			{#if isAdmin}
				<button
					onclick={handleDelete}
					class="flex w-full items-center gap-2 rounded-lg px-3 py-1.5 text-sm text-[var(--danger)] transition hover:bg-[var(--danger)]/10"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3,6 5,6 21,6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
					Delete Channel
				</button>
			{/if}
		</div>
	</div>
</div>
