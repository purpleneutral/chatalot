<script lang="ts">
	import { goto } from '$app/navigation';
	import { fade, scale } from 'svelte/transition';
	import { authStore } from '$lib/stores/auth.svelte';
	import { toastStore } from '$lib/stores/toast.svelte';
	import {
		listUsers, suspendUser, unsuspendUser, deleteUser, setAdmin, resetUserPassword, type AdminUser,
		createRegistrationInvite, listRegistrationInvites, deleteRegistrationInvite, type RegistrationInvite,
		listFiles, adminDeleteFile, quarantineFile, unquarantineFile, type AdminFileEntry, type AdminFilesResponse,
		getStorageStats, type StorageStats,
		purgeMessage, purgeUserMessages, purgeChannel, type PurgeResult,
		quarantineMessage, unquarantineMessage,
		listBlockedHashes, addBlockedHash, removeBlockedHash, type BlockedHash,
		getAuditLog, type AuditLogEntry, type AuditLogResponse,
		listReports, reviewReport, type Report, type ReportsResponse,
		getInstanceSettings, updateInstanceSettings,
		listAllWebhooks, type AdminWebhook, type AdminWebhooksResponse
	} from '$lib/api/admin';
	import { createAnnouncement, listAllAnnouncements, type Announcement } from '$lib/api/announcements';
	import { getAuthenticatedThumbUrl } from '$lib/api/files';
	import { onMount, onDestroy } from 'svelte';

	type Tab = 'users' | 'invites' | 'files' | 'reports' | 'audit' | 'security' | 'announcements' | 'webhooks' | 'settings';
	let activeTab = $state<Tab>('users');

	// ── Users ──
	let users = $state<AdminUser[]>([]);
	let usersLoading = $state(false);
	let searchQuery = $state('');
	let searchTimeout: ReturnType<typeof setTimeout>;
	let resettingPasswordId = $state<string | null>(null);
	let resetPasswordUser = $state<AdminUser | null>(null);
	let resetPasswordInput = $state('');

	// ── Invites ──
	let invites = $state<RegistrationInvite[]>([]);
	let invitesLoading = $state(false);
	let showCreateInvite = $state(false);
	let newInviteMaxUses = $state('');
	let newInviteExpiresHours = $state('');
	let creatingInvite = $state(false);

	// ── Files ──
	let filesResponse = $state<AdminFilesResponse | null>(null);
	let filesLoading = $state(false);
	let filesPage = $state(1);
	let filesSort = $state('date');
	let storageStats = $state<StorageStats | null>(null);
	let filesViewMode = $state<'table' | 'grid'>('table');
	let filesSearch = $state('');
	let filesSearchTimeout: ReturnType<typeof setTimeout>;
	let filesContentType = $state('');
	let filesDateFrom = $state('');
	let filesDateTo = $state('');

	// ── Reports ──
	let reportsResponse = $state<ReportsResponse | null>(null);
	let reportsLoading = $state(false);
	let reportsPage = $state(1);
	let reportsStatusFilter = $state('');
	let reviewingReportId = $state<string | null>(null);
	let reviewStatus = $state('reviewed');
	let reviewNotes = $state('');

	// ── Audit Log ──
	let auditResponse = $state<AuditLogResponse | null>(null);
	let auditLoading = $state(false);
	let auditPage = $state(1);
	let auditActionFilter = $state('');
	let auditUserFilter = $state('');

	// ── Announcements ──
	let adminAnnouncements = $state<Announcement[]>([]);
	let announcementsLoading = $state(false);
	let newAnnouncementTitle = $state('');
	let newAnnouncementBody = $state('');
	let creatingAnnouncement = $state(false);

	// ── Webhooks ──
	let webhooksResponse = $state<AdminWebhooksResponse | null>(null);
	let webhooksLoading = $state(false);

	// ── Instance Settings ──
	let instanceSettings = $state<Record<string, string>>({});
	let settingsLoading = $state(false);
	let savingSettings = $state(false);
	let settingsMaxCache = $state('500');
	let settingsMaxPins = $state('50');
	let settingsE2eEnabled = $state('true');

	// ── Security (Blocked Hashes + Purge) ──
	let blockedHashes = $state<BlockedHash[]>([]);
	let hashesLoading = $state(false);
	let newHashValue = $state('');
	let newHashReason = $state('');
	let purgeType = $state<'message' | 'user' | 'channel'>('message');
	let purgeTargetId = $state('');
	let purgeBlockHashes = $state(false);
	let purging = $state(false);

	onMount(() => {
		if (!authStore.isAuthenticated || (!authStore.user?.is_admin && !authStore.user?.is_owner)) {
			goto('/channels');
			return;
		}
		loadUsers();
	});

	onDestroy(() => {
		clearTimeout(searchTimeout);
	});

	function switchTab(tab: Tab) {
		activeTab = tab;
		// Clear stale review form when leaving reports tab
		if (tab !== 'reports') reviewingReportId = null;
		if (tab === 'invites' && invites.length === 0) loadInvites();
		if (tab === 'files' && !filesResponse) { loadFiles(); loadStorageStats(); }
		if (tab === 'reports' && !reportsResponse) loadReports();
		if (tab === 'audit' && !auditResponse) loadAuditLog();
		if (tab === 'security' && blockedHashes.length === 0) loadBlockedHashes();
		if (tab === 'announcements' && adminAnnouncements.length === 0) loadAnnouncements();
		if (tab === 'webhooks' && !webhooksResponse) loadWebhooks();
		if (tab === 'settings' && Object.keys(instanceSettings).length === 0) loadSettings();
	}

	// ── Confirm dialog ──
	let confirmDialog = $state<{
		title: string;
		message: string;
		confirmLabel?: string;
		danger?: boolean;
		inputPlaceholder?: string;
		inputValue?: string;
		checkboxLabel?: string;
		checkboxValue?: boolean;
		onConfirm: (inputValue?: string, checkboxValue?: boolean) => void;
	} | null>(null);
	let confirmInput = $state('');
	let confirmCheckbox = $state(false);

	function showConfirmDialog(opts: NonNullable<typeof confirmDialog>) {
		confirmInput = opts.inputValue ?? '';
		confirmCheckbox = opts.checkboxValue ?? false;
		confirmDialog = opts;
	}

	// ── User handlers ──

	async function loadUsers() {
		usersLoading = true;
		try {
			users = await listUsers({ search: searchQuery || undefined, limit: 50 });
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to load users');
		} finally {
			usersLoading = false;
		}
	}

	function handleSearch() {
		clearTimeout(searchTimeout);
		searchTimeout = setTimeout(() => loadUsers(), 300);
	}

	function handleSuspend(user: AdminUser) {
		showConfirmDialog({
			title: `Suspend ${user.username}`,
			message: 'This will immediately block the user from accessing the platform.',
			confirmLabel: 'Suspend',
			danger: true,
			inputPlaceholder: 'Suspension reason (optional)',
			async onConfirm(reason) {
				try {
					await suspendUser(user.id, reason || undefined);
					toastStore.success(`Suspended ${user.username}`);
					await loadUsers();
				} catch (err) {
					toastStore.error(err instanceof Error ? err.message : 'Failed to suspend user');
				}
			}
		});
	}

	function handleUnsuspend(user: AdminUser) {
		showConfirmDialog({
			title: `Unsuspend ${user.username}?`,
			message: 'This will restore access for the user immediately.',
			confirmLabel: 'Unsuspend',
			async onConfirm() {
				try {
					await unsuspendUser(user.id);
					toastStore.success(`Unsuspended ${user.username}`);
					await loadUsers();
				} catch (err) {
					toastStore.error(err instanceof Error ? err.message : 'Failed to unsuspend user');
				}
			}
		});
	}

	function handleDelete(user: AdminUser) {
		showConfirmDialog({
			title: `Delete ${user.username}?`,
			message: 'This will permanently delete this user account. This cannot be undone.',
			confirmLabel: 'Delete',
			danger: true,
			async onConfirm() {
				try {
					await deleteUser(user.id);
					toastStore.success(`Deleted ${user.username}`);
					await loadUsers();
				} catch (err) {
					toastStore.error(err instanceof Error ? err.message : 'Failed to delete user');
				}
			}
		});
	}

	function handleToggleAdmin(user: AdminUser) {
		const newState = !user.is_admin;
		const action = newState ? 'Grant admin to' : 'Revoke admin from';
		showConfirmDialog({
			title: `${action} ${user.username}?`,
			message: newState
				? 'This user will have full administrative access to the platform.'
				: 'This user will lose administrative access.',
			confirmLabel: newState ? 'Grant Admin' : 'Revoke Admin',
			danger: !newState,
			async onConfirm() {
				try {
					await setAdmin(user.id, newState);
					toastStore.success(`${newState ? 'Granted' : 'Revoked'} admin for ${user.username}`);
					await loadUsers();
				} catch (err) {
					toastStore.error(err instanceof Error ? err.message : 'Failed to update admin status');
				}
			}
		});
	}

	function handleResetPassword(user: AdminUser) {
		resetPasswordUser = user;
		resetPasswordInput = '';
	}

	async function submitResetPassword() {
		if (!resetPasswordUser || !resetPasswordInput) return;
		resettingPasswordId = resetPasswordUser.id;
		try {
			await resetUserPassword(resetPasswordUser.id, resetPasswordInput);
			toastStore.success(`Password reset for ${resetPasswordUser.username}`);
			resetPasswordUser = null;
			resetPasswordInput = '';
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to reset password');
		} finally {
			resettingPasswordId = null;
		}
	}

	// ── Invite handlers ──

	async function loadInvites() {
		invitesLoading = true;
		try {
			invites = await listRegistrationInvites();
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to load invites');
		} finally {
			invitesLoading = false;
		}
	}

	async function handleCreateInvite() {
		creatingInvite = true;
		try {
			const params: { max_uses?: number; expires_in_hours?: number } = {};
			if (newInviteMaxUses) {
				const n = parseInt(newInviteMaxUses, 10);
				if (!Number.isFinite(n) || n < 1) { toastStore.error('Max uses must be a positive number'); creatingInvite = false; return; }
				params.max_uses = n;
			}
			if (newInviteExpiresHours) {
				const n = parseInt(newInviteExpiresHours, 10);
				if (!Number.isFinite(n) || n < 1) { toastStore.error('Expiry hours must be a positive number'); creatingInvite = false; return; }
				params.expires_in_hours = n;
			}
			const invite = await createRegistrationInvite(params);
			toastStore.success(`Invite code created: ${invite.code}`);
			showCreateInvite = false;
			newInviteMaxUses = '';
			newInviteExpiresHours = '';
			await loadInvites();
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to create invite');
		} finally {
			creatingInvite = false;
		}
	}

	function handleDeleteInvite(invite: RegistrationInvite) {
		showConfirmDialog({
			title: 'Delete invite?',
			message: `Delete invite code ${invite.code}?`,
			confirmLabel: 'Delete',
			danger: true,
			async onConfirm() {
				try {
					await deleteRegistrationInvite(invite.id);
					toastStore.success('Invite deleted');
					await loadInvites();
				} catch (err) {
					toastStore.error(err instanceof Error ? err.message : 'Failed to delete invite');
				}
			}
		});
	}

	async function copyText(text: string) {
		try {
			await navigator.clipboard.writeText(text);
			toastStore.success('Copied to clipboard');
		} catch {
			toastStore.error('Failed to copy');
		}
	}

	// ── File handlers ──

	async function loadFiles() {
		filesLoading = true;
		try {
			filesResponse = await listFiles({
				page: filesPage,
				per_page: filesViewMode === 'grid' ? 40 : 25,
				sort: filesSort,
				search: filesSearch || undefined,
				content_type: filesContentType || undefined,
				date_from: filesDateFrom ? new Date(filesDateFrom).toISOString() : undefined,
				date_to: filesDateTo ? new Date(filesDateTo + 'T23:59:59').toISOString() : undefined
			});
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to load files');
		} finally {
			filesLoading = false;
		}
	}

	function handleFilesSearch() {
		clearTimeout(filesSearchTimeout);
		filesSearchTimeout = setTimeout(() => { filesPage = 1; loadFiles(); }, 300);
	}

	function resetFilesFilters() {
		filesSearch = '';
		filesContentType = '';
		filesDateFrom = '';
		filesDateTo = '';
		filesPage = 1;
		loadFiles();
	}

	async function loadStorageStats() {
		try {
			storageStats = await getStorageStats();
		} catch { /* non-critical */ }
	}

	async function handleQuarantineFile(file: AdminFileEntry) {
		try {
			if (file.quarantined_at) {
				await unquarantineFile(file.id);
				toastStore.success('File unquarantined');
			} else {
				await quarantineFile(file.id);
				toastStore.success('File quarantined');
			}
			await loadFiles();
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to update quarantine');
		}
	}

	function handleDeleteFile(file: AdminFileEntry) {
		showConfirmDialog({
			title: 'Delete file?',
			message: `Delete file ${file.id.slice(0, 8)}...? This removes it from disk permanently.`,
			confirmLabel: 'Delete',
			danger: true,
			checkboxLabel: 'Also block this file hash to prevent re-upload',
			async onConfirm(_input, blockHash) {
				try {
					await adminDeleteFile(file.id, blockHash ?? false);
					toastStore.success('File deleted');
					await loadFiles();
					await loadStorageStats();
				} catch (err) {
					toastStore.error(err instanceof Error ? err.message : 'Failed to delete file');
				}
			}
		});
	}

	// ── Report handlers ──

	async function loadReports() {
		reportsLoading = true;
		try {
			reportsResponse = await listReports({
				status: reportsStatusFilter || undefined,
				page: reportsPage,
				per_page: 25
			});
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to load reports');
		} finally {
			reportsLoading = false;
		}
	}

	async function handleReviewReport() {
		if (!reviewingReportId) return;
		try {
			await reviewReport(reviewingReportId, reviewStatus, reviewNotes || undefined);
			toastStore.success('Report reviewed');
			reviewingReportId = null;
			reviewStatus = 'reviewed';
			reviewNotes = '';
			await loadReports();
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to review report');
		}
	}

	// ── Audit handlers ──

	async function loadAuditLog() {
		auditLoading = true;
		try {
			auditResponse = await getAuditLog({
				page: auditPage,
				per_page: 50,
				action: auditActionFilter || undefined,
				user_id: auditUserFilter || undefined
			});
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to load audit log');
		} finally {
			auditLoading = false;
		}
	}

	// ── Security handlers ──

	async function loadBlockedHashes() {
		hashesLoading = true;
		try {
			blockedHashes = await listBlockedHashes({ per_page: 100 });
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to load blocked hashes');
		} finally {
			hashesLoading = false;
		}
	}

	async function handleAddHash() {
		if (!newHashValue || newHashValue.length !== 64) {
			toastStore.error('Hash must be a 64-character hex SHA256');
			return;
		}
		try {
			await addBlockedHash(newHashValue, newHashReason || undefined);
			toastStore.success('Hash blocked');
			newHashValue = '';
			newHashReason = '';
			await loadBlockedHashes();
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to block hash');
		}
	}

	function handleRemoveHash(hash: BlockedHash) {
		showConfirmDialog({
			title: 'Unblock hash?',
			message: `Unblock hash ${hash.hash.slice(0, 16)}...? Files with this hash will be allowed again.`,
			confirmLabel: 'Unblock',
			async onConfirm() {
				try {
					await removeBlockedHash(hash.id);
					toastStore.success('Hash unblocked');
					await loadBlockedHashes();
				} catch (err) {
					toastStore.error(err instanceof Error ? err.message : 'Failed to unblock hash');
				}
			}
		});
	}

	function handlePurge() {
		if (!purgeTargetId.trim()) {
			toastStore.error('Enter a target ID');
			return;
		}
		const labels: Record<string, string> = {
			message: 'message',
			user: 'ALL messages and files from user',
			channel: 'ALL messages and files in channel'
		};
		const targetId = purgeTargetId.trim();
		const type = purgeType;
		const blockHashes = purgeBlockHashes;
		showConfirmDialog({
			title: 'Permanent Purge',
			message: `PERMANENTLY PURGE ${labels[type]} ${targetId}? This CANNOT be undone.`,
			confirmLabel: 'Purge',
			danger: true,
			async onConfirm() {
				purging = true;
				try {
					let result: PurgeResult;
					if (type === 'message') result = await purgeMessage(targetId, blockHashes);
					else if (type === 'user') result = await purgeUserMessages(targetId, blockHashes);
					else result = await purgeChannel(targetId, blockHashes);
					toastStore.success(`Purged: ${result.messages_deleted} messages, ${result.files_deleted} files, ${result.hashes_blocked} hashes blocked`);
					purgeTargetId = '';
				} catch (err) {
					toastStore.error(err instanceof Error ? err.message : 'Purge failed');
				} finally {
					purging = false;
				}
			}
		});
	}

	function formatBytes(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
		if (bytes < 1073741824) return `${(bytes / 1048576).toFixed(1)} MB`;
		return `${(bytes / 1073741824).toFixed(2)} GB`;
	}

	// ── Announcement handlers ──

	async function loadAnnouncements() {
		announcementsLoading = true;
		try {
			adminAnnouncements = await listAllAnnouncements();
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to load announcements');
		} finally {
			announcementsLoading = false;
		}
	}

	async function handleCreateAnnouncement() {
		if (!newAnnouncementTitle.trim() || !newAnnouncementBody.trim()) {
			toastStore.error('Title and body are required');
			return;
		}
		creatingAnnouncement = true;
		try {
			const ann = await createAnnouncement(newAnnouncementTitle.trim(), newAnnouncementBody.trim());
			adminAnnouncements = [ann, ...adminAnnouncements];
			newAnnouncementTitle = '';
			newAnnouncementBody = '';
			toastStore.success('Announcement published');
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to create announcement');
		} finally {
			creatingAnnouncement = false;
		}
	}

	async function loadWebhooks() {
		webhooksLoading = true;
		try {
			webhooksResponse = await listAllWebhooks({ per_page: 50 });
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to load webhooks');
		} finally {
			webhooksLoading = false;
		}
	}

	async function loadSettings() {
		settingsLoading = true;
		try {
			instanceSettings = await getInstanceSettings();
			settingsMaxCache = instanceSettings.max_messages_cache ?? '500';
			settingsMaxPins = instanceSettings.max_pins_per_channel ?? '50';
			settingsE2eEnabled = instanceSettings.e2e_enabled ?? 'true';
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to load settings');
		} finally {
			settingsLoading = false;
		}
	}

	async function saveSettings() {
		savingSettings = true;
		try {
			instanceSettings = await updateInstanceSettings({
				max_messages_cache: settingsMaxCache,
				max_pins_per_channel: settingsMaxPins,
				e2e_enabled: settingsE2eEnabled,
			});
			settingsMaxCache = instanceSettings.max_messages_cache ?? '500';
			settingsMaxPins = instanceSettings.max_pins_per_channel ?? '50';
			settingsE2eEnabled = instanceSettings.e2e_enabled ?? 'true';
			toastStore.success('Settings saved');
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to save settings');
		} finally {
			savingSettings = false;
		}
	}

	const tabs: { id: Tab; label: string }[] = [
		{ id: 'users', label: 'Users' },
		{ id: 'invites', label: 'Invites' },
		{ id: 'files', label: 'Files' },
		{ id: 'reports', label: 'Reports' },
		{ id: 'audit', label: 'Audit Log' },
		{ id: 'security', label: 'Security' },
		{ id: 'announcements', label: 'Announcements' },
		{ id: 'webhooks', label: 'Webhooks' },
		{ id: 'settings', label: 'Settings' }
	];
</script>

{#if authStore.isAuthenticated && (authStore.user?.is_admin || authStore.user?.is_owner)}
	<div class="min-h-screen bg-[var(--bg-primary)] text-[var(--text-primary)]">
		<div class="mx-auto max-w-5xl px-3 py-4 sm:px-6 sm:py-8">
			<!-- Header -->
			<div class="mb-6 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
				<h1 class="text-2xl font-bold">Admin Panel</h1>
				<div class="flex gap-2">
					<button onclick={() => goto('/settings')} class="rounded-lg border border-white/10 px-3 py-1.5 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]">Settings</button>
					<button onclick={() => goto('/channels')} class="rounded-lg border border-white/10 px-3 py-1.5 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]">Back to Chat</button>
				</div>
			</div>

			<!-- Tabs -->
			<div class="mb-6 flex gap-1 overflow-x-auto rounded-lg border border-white/10 bg-[var(--bg-secondary)] p-1" role="tablist" aria-label="Admin panel">
				{#each tabs as tab}
					<button
						onclick={() => switchTab(tab.id)}
						role="tab"
						aria-selected={activeTab === tab.id}
						class="whitespace-nowrap rounded-md px-4 py-2 text-sm font-medium transition {activeTab === tab.id ? 'bg-[var(--accent)] text-white' : 'text-[var(--text-secondary)] hover:bg-white/5 hover:text-[var(--text-primary)]'}"
					>
						{tab.label}
					</button>
				{/each}
			</div>

			<!-- ═══ USERS TAB ═══ -->
			{#if activeTab === 'users'}
				<section class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
					<div class="mb-4 flex items-center justify-between">
						<h2 class="text-lg font-semibold">User Management</h2>
						<span class="text-sm text-[var(--text-secondary)]">{users.length} users</span>
					</div>
					<div class="mb-4">
						<input type="text" bind:value={searchQuery} oninput={handleSearch} placeholder="Search by username, name, or email..." class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]" />
					</div>
					{#if usersLoading}
						<p class="text-sm text-[var(--text-secondary)]">Loading users...</p>
					{:else if users.length === 0}
						<p class="text-sm text-[var(--text-secondary)]">No users found.</p>
					{:else}
						<div class="overflow-x-auto">
							<table class="w-full text-sm">
								<thead>
									<tr class="border-b border-white/10 text-left text-[var(--text-secondary)]">
										<th class="py-2 pr-4 font-medium">User</th>
										<th class="py-2 pr-4 font-medium">Email</th>
										<th class="py-2 pr-4 font-medium">Memberships</th>
										<th class="py-2 pr-4 font-medium">Status</th>
										<th class="py-2 pr-4 font-medium">Joined</th>
										<th class="py-2 font-medium">Actions</th>
									</tr>
								</thead>
								<tbody>
									{#each users as user}
										<tr class="border-b border-white/5">
											<td class="py-3 pr-4">
												<div class="flex items-center gap-2">
													<div class="flex h-8 w-8 items-center justify-center rounded-full bg-[var(--accent)]">
														<span class="text-xs font-bold text-white">{user.display_name?.[0]?.toUpperCase() ?? '?'}</span>
													</div>
													<div>
														<div class="font-medium">
															{user.display_name}
															{#if user.is_owner}
																<span class="ml-1 rounded bg-amber-500/20 px-1.5 py-0.5 text-xs text-amber-400">owner</span>
															{:else if user.is_admin}
																<span class="ml-1 rounded bg-[var(--accent)]/20 px-1.5 py-0.5 text-xs text-[var(--accent)]">admin</span>
															{/if}
														</div>
														<div class="text-xs text-[var(--text-secondary)]">@{user.username}</div>
													</div>
												</div>
											</td>
											<td class="py-3 pr-4 text-[var(--text-secondary)]">{user.email}</td>
											<td class="py-3 pr-4">
												<div class="flex flex-wrap gap-1">
													{#each user.communities ?? [] as c}
														<span class="rounded bg-purple-500/10 px-1.5 py-0.5 text-xs text-purple-400" title={c.role}>{c.name}</span>
													{/each}
													{#each user.groups ?? [] as g}
														<span class="rounded bg-blue-500/10 px-1.5 py-0.5 text-xs text-blue-400" title={g.role}>{g.name}</span>
													{/each}
													{#if (user.communities?.length ?? 0) === 0 && (user.groups?.length ?? 0) === 0}
														<span class="text-xs text-[var(--text-secondary)]">None</span>
													{/if}
												</div>
											</td>
											<td class="py-3 pr-4">
												{#if user.suspended_at}
													<span class="rounded bg-red-500/10 px-2 py-0.5 text-xs text-red-400">Suspended</span>
												{:else}
													<span class="rounded bg-green-500/10 px-2 py-0.5 text-xs text-green-400">Active</span>
												{/if}
											</td>
											<td class="py-3 pr-4 text-[var(--text-secondary)]">{new Date(user.created_at).toLocaleDateString()}</td>
											<td class="py-3">
												{#if user.id !== authStore.user?.id}
													<div class="flex flex-wrap gap-1">
														{#if user.suspended_at}
															<button onclick={() => handleUnsuspend(user)} class="rounded px-2 py-1 text-xs text-green-400 transition hover:bg-green-500/10">Unsuspend</button>
														{:else}
															<button onclick={() => handleSuspend(user)} class="rounded px-2 py-1 text-xs text-yellow-400 transition hover:bg-yellow-500/10">Suspend</button>
														{/if}
														<button onclick={() => handleResetPassword(user)} disabled={resettingPasswordId === user.id} class="rounded px-2 py-1 text-xs text-orange-400 transition hover:bg-orange-500/10 disabled:opacity-50">{resettingPasswordId === user.id ? 'Resetting...' : 'Reset PW'}</button>
														<button onclick={() => handleToggleAdmin(user)} class="rounded px-2 py-1 text-xs text-[var(--accent)] transition hover:bg-[var(--accent)]/10">{user.is_admin ? 'Revoke Admin' : 'Grant Admin'}</button>
														<button onclick={() => handleDelete(user)} class="rounded px-2 py-1 text-xs text-red-400 transition hover:bg-red-500/10">Delete</button>
													</div>
												{:else}
													<span class="text-xs text-[var(--text-secondary)]">You</span>
												{/if}
											</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					{/if}
				</section>

			<!-- ═══ INVITES TAB ═══ -->
			{:else if activeTab === 'invites'}
				<section class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
					<div class="mb-4 flex items-center justify-between">
						<h2 class="text-lg font-semibold">Registration Invites</h2>
						<button onclick={() => (showCreateInvite = !showCreateInvite)} class="rounded-lg bg-[var(--accent)] px-3 py-1.5 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)]">
							{showCreateInvite ? 'Cancel' : 'Generate Invite'}
						</button>
					</div>
					<p class="mb-4 text-xs text-[var(--text-secondary)]">Set REGISTRATION_MODE=invite_only to require invite codes for new registrations.</p>
					{#if showCreateInvite}
						<div class="mb-4 rounded-lg border border-white/10 bg-[var(--bg-primary)] p-4">
							<div class="flex gap-3">
								<div class="flex-1">
									<label for="admin-invite-max-uses" class="mb-1 block text-xs text-[var(--text-secondary)]">Max Uses (blank = unlimited)</label>
									<input id="admin-invite-max-uses" type="number" bind:value={newInviteMaxUses} min="1" placeholder="Unlimited" class="w-full rounded border border-white/10 bg-[var(--bg-secondary)] px-3 py-1.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]" />
								</div>
								<div class="flex-1">
									<label for="admin-invite-expires" class="mb-1 block text-xs text-[var(--text-secondary)]">Expires In (hours, blank = never)</label>
									<input id="admin-invite-expires" type="number" bind:value={newInviteExpiresHours} min="1" placeholder="Never" class="w-full rounded border border-white/10 bg-[var(--bg-secondary)] px-3 py-1.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]" />
								</div>
								<div class="flex items-end">
									<button onclick={handleCreateInvite} disabled={creatingInvite} class="rounded-lg bg-[var(--accent)] px-4 py-1.5 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:opacity-50">
										{creatingInvite ? 'Creating...' : 'Create'}
									</button>
								</div>
							</div>
						</div>
					{/if}
					{#if invitesLoading}
						<p class="text-sm text-[var(--text-secondary)]">Loading invites...</p>
					{:else if invites.length === 0}
						<p class="text-sm text-[var(--text-secondary)]">No invite codes yet.</p>
					{:else}
						<div class="overflow-x-auto">
							<table class="w-full text-sm">
								<thead>
									<tr class="border-b border-white/10 text-left text-[var(--text-secondary)]">
										<th class="py-2 pr-4 font-medium">Code</th>
										<th class="py-2 pr-4 font-medium">Uses</th>
										<th class="py-2 pr-4 font-medium">Expires</th>
										<th class="py-2 pr-4 font-medium">Created</th>
										<th class="py-2 font-medium">Actions</th>
									</tr>
								</thead>
								<tbody>
									{#each invites as invite}
										<tr class="border-b border-white/5">
											<td class="py-3 pr-4"><code class="rounded bg-white/5 px-2 py-0.5 font-mono text-xs">{invite.code}</code></td>
											<td class="py-3 pr-4 text-[var(--text-secondary)]">{invite.used_count}{invite.max_uses !== null ? `/${invite.max_uses}` : ''}</td>
											<td class="py-3 pr-4 text-[var(--text-secondary)]">{invite.expires_at ? new Date(invite.expires_at).toLocaleString() : 'Never'}</td>
											<td class="py-3 pr-4 text-[var(--text-secondary)]">{new Date(invite.created_at).toLocaleDateString()}</td>
											<td class="py-3">
												<div class="flex gap-1">
													<button onclick={() => copyText(invite.code)} class="rounded px-2 py-1 text-xs text-[var(--accent)] transition hover:bg-[var(--accent)]/10">Copy</button>
													<button onclick={() => handleDeleteInvite(invite)} class="rounded px-2 py-1 text-xs text-red-400 transition hover:bg-red-500/10">Delete</button>
												</div>
											</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					{/if}
				</section>

			<!-- ═══ FILES TAB ═══ -->
			{:else if activeTab === 'files'}
				<!-- Storage Stats -->
				{#if storageStats}
					<div class="mb-4 grid grid-cols-3 gap-4">
						<div class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-4 text-center">
							<div class="text-2xl font-bold">{storageStats.total_files}</div>
							<div class="text-xs text-[var(--text-secondary)]">Total Files</div>
						</div>
						<div class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-4 text-center">
							<div class="text-2xl font-bold">{formatBytes(storageStats.total_bytes)}</div>
							<div class="text-xs text-[var(--text-secondary)]">Total Storage</div>
						</div>
						<div class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-4 text-center">
							<div class="text-2xl font-bold">{storageStats.per_user.length}</div>
							<div class="text-xs text-[var(--text-secondary)]">Uploaders</div>
						</div>
					</div>
				{/if}

				<section class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
					<!-- Header with view toggle -->
					<div class="mb-4 flex items-center justify-between">
						<h2 class="text-lg font-semibold">File Browser</h2>
						<div class="flex items-center gap-3">
							<!-- View toggle -->
							<div class="flex rounded border border-white/10">
								<button onclick={() => { filesViewMode = 'table'; filesPage = 1; loadFiles(); }} class="px-2.5 py-1.5 text-xs transition {filesViewMode === 'table' ? 'bg-white/10 text-[var(--text-primary)]' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}" title="Table view">
									<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/></svg>
								</button>
								<button onclick={() => { filesViewMode = 'grid'; filesPage = 1; loadFiles(); }} class="px-2.5 py-1.5 text-xs transition {filesViewMode === 'grid' ? 'bg-white/10 text-[var(--text-primary)]' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}" title="Grid view">
									<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1H5a1 1 0 01-1-1V5zm10 0a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1V5zM4 15a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1H5a1 1 0 01-1-1v-4zm10 0a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z"/></svg>
								</button>
							</div>
							<label for="files-sort" class="text-xs text-[var(--text-secondary)]">Sort:</label>
							<select id="files-sort" bind:value={filesSort} onchange={() => { filesPage = 1; loadFiles(); }} class="rounded border border-white/10 bg-[var(--bg-primary)] px-2 py-1 text-xs text-[var(--text-primary)]">
								<option value="date">Newest</option>
								<option value="size">Largest</option>
							</select>
						</div>
					</div>

					<!-- Search & Filters -->
					<div class="mb-4 flex flex-wrap items-end gap-3">
						<div class="flex-1" style="min-width: 180px;">
							<label for="files-search" class="mb-1 block text-xs text-[var(--text-secondary)]">Search filename</label>
							<input id="files-search" type="text" bind:value={filesSearch} oninput={handleFilesSearch} placeholder="Search..." class="w-full rounded border border-white/10 bg-[var(--bg-primary)] px-3 py-1.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-secondary)]/50 focus:border-blue-500/50 focus:outline-none" />
						</div>
						<div>
							<label for="files-type" class="mb-1 block text-xs text-[var(--text-secondary)]">Type</label>
							<select id="files-type" bind:value={filesContentType} onchange={() => { filesPage = 1; loadFiles(); }} class="rounded border border-white/10 bg-[var(--bg-primary)] px-2 py-1.5 text-xs text-[var(--text-primary)]">
								<option value="">All</option>
								<option value="image/">Images</option>
								<option value="video/">Video</option>
								<option value="audio/">Audio</option>
								<option value="application/pdf">PDF</option>
								<option value="application/">Documents</option>
								<option value="text/">Text</option>
							</select>
						</div>
						<div>
							<label for="files-date-from" class="mb-1 block text-xs text-[var(--text-secondary)]">From</label>
							<input id="files-date-from" type="date" bind:value={filesDateFrom} onchange={() => { filesPage = 1; loadFiles(); }} class="rounded border border-white/10 bg-[var(--bg-primary)] px-2 py-1 text-xs text-[var(--text-primary)] focus:border-blue-500/50 focus:outline-none" />
						</div>
						<div>
							<label for="files-date-to" class="mb-1 block text-xs text-[var(--text-secondary)]">To</label>
							<input id="files-date-to" type="date" bind:value={filesDateTo} onchange={() => { filesPage = 1; loadFiles(); }} class="rounded border border-white/10 bg-[var(--bg-primary)] px-2 py-1 text-xs text-[var(--text-primary)] focus:border-blue-500/50 focus:outline-none" />
						</div>
						{#if filesSearch || filesContentType || filesDateFrom || filesDateTo}
							<button onclick={resetFilesFilters} class="rounded px-2 py-1.5 text-xs text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]">Clear filters</button>
						{/if}
					</div>

					{#if filesLoading}
						<p class="text-sm text-[var(--text-secondary)]">Loading files...</p>
					{:else if !filesResponse || filesResponse.files.length === 0}
						<p class="text-sm text-[var(--text-secondary)]">No files found.</p>
					{:else if filesViewMode === 'grid'}
						<!-- Grid View -->
						<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
							{#each filesResponse.files as file}
								<div class="group relative overflow-hidden rounded-lg border border-white/10 bg-[var(--bg-primary)] transition hover:border-white/20">
									<!-- Thumbnail / Icon -->
									<div class="flex aspect-square items-center justify-center overflow-hidden bg-black/20">
										{#if file.has_thumbnail}
											{#await getAuthenticatedThumbUrl(file.id)}
												<div class="h-8 w-8 animate-pulse rounded bg-white/10"></div>
											{:then thumbUrl}
												<img src={thumbUrl} alt="" class="h-full w-full object-cover" />
											{:catch}
												<svg class="h-10 w-10 text-[var(--text-secondary)]/30" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"/></svg>
											{/await}
										{:else}
											<!-- File type icon -->
											{#if file.content_type?.startsWith('video/')}
												<svg class="h-10 w-10 text-[var(--text-secondary)]/30" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z"/></svg>
											{:else if file.content_type?.startsWith('audio/')}
												<svg class="h-10 w-10 text-[var(--text-secondary)]/30" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"/></svg>
											{:else if file.content_type === 'application/pdf'}
												<svg class="h-10 w-10 text-red-400/40" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"/></svg>
											{:else}
												<svg class="h-10 w-10 text-[var(--text-secondary)]/30" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"/></svg>
											{/if}
										{/if}
										<!-- Quarantine badge -->
										{#if file.quarantined_at}
											<div class="absolute left-1.5 top-1.5 rounded bg-orange-500/80 px-1.5 py-0.5 text-[10px] font-medium text-white">Quarantined</div>
										{/if}
									</div>
									<!-- Info -->
									<div class="p-2">
										<div class="truncate text-xs text-[var(--text-primary)]" title={file.encrypted_name}>{file.encrypted_name || '—'}</div>
										<div class="mt-0.5 flex items-center justify-between text-[10px] text-[var(--text-secondary)]">
											<span>{formatBytes(file.size_bytes)}</span>
											<span>{new Date(file.created_at).toLocaleDateString()}</span>
										</div>
										<div class="mt-0.5 text-[10px] text-[var(--text-secondary)]">{file.content_type ?? 'unknown'}</div>
									</div>
									<!-- Hover actions -->
									<div class="absolute inset-x-0 bottom-0 flex translate-y-full gap-1 border-t border-white/10 bg-[var(--bg-primary)]/95 p-1.5 backdrop-blur transition-transform group-hover:translate-y-0">
										<button onclick={() => copyText(file.id)} class="flex-1 rounded px-1.5 py-1 text-[10px] text-[var(--text-secondary)] transition hover:bg-white/10 hover:text-[var(--text-primary)]">Copy ID</button>
										<button onclick={() => handleQuarantineFile(file)} class="flex-1 rounded px-1.5 py-1 text-[10px] text-orange-400 transition hover:bg-orange-500/10">{file.quarantined_at ? 'Restore' : 'Quarantine'}</button>
										<button onclick={() => handleDeleteFile(file)} class="flex-1 rounded px-1.5 py-1 text-[10px] text-red-400 transition hover:bg-red-500/10">Delete</button>
									</div>
								</div>
							{/each}
						</div>
					{:else}
						<!-- Table View -->
						<div class="overflow-x-auto">
							<table class="w-full text-sm">
								<thead>
									<tr class="border-b border-white/10 text-left text-[var(--text-secondary)]">
										<th class="py-2 pr-3 font-medium">ID</th>
										<th class="py-2 pr-3 font-medium">Name</th>
										<th class="py-2 pr-3 font-medium">Type</th>
										<th class="py-2 pr-3 font-medium">Size</th>
										<th class="py-2 pr-3 font-medium">Uploader</th>
										<th class="py-2 pr-3 font-medium">Status</th>
										<th class="py-2 pr-3 font-medium">Uploaded</th>
										<th class="py-2 font-medium">Actions</th>
									</tr>
								</thead>
								<tbody>
									{#each filesResponse.files as file}
										<tr class="border-b border-white/5">
											<td class="py-3 pr-3">
												<button class="cursor-pointer rounded bg-white/5 px-1.5 py-0.5 font-mono text-xs hover:bg-white/10" title={file.id} onclick={() => copyText(file.id)}>{file.id.slice(0, 8)}</button>
											</td>
											<td class="py-3 pr-3 max-w-32 truncate text-xs text-[var(--text-secondary)]" title={file.encrypted_name}>{file.encrypted_name || '—'}</td>
											<td class="py-3 pr-3 text-xs text-[var(--text-secondary)]">{file.content_type ?? 'unknown'}</td>
											<td class="py-3 pr-3 text-xs">{formatBytes(file.size_bytes)}</td>
											<td class="py-3 pr-3">
												<button class="cursor-pointer font-mono text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)]" title={file.uploader_id} onclick={() => copyText(file.uploader_id)}>{file.uploader_id.slice(0, 8)}</button>
											</td>
											<td class="py-3 pr-3">
												{#if file.quarantined_at}
													<span class="rounded bg-orange-500/10 px-2 py-0.5 text-xs text-orange-400">Quarantined</span>
												{:else}
													<span class="rounded bg-green-500/10 px-2 py-0.5 text-xs text-green-400">Active</span>
												{/if}
											</td>
											<td class="py-3 pr-3 text-xs text-[var(--text-secondary)]">{new Date(file.created_at).toLocaleDateString()}</td>
											<td class="py-3">
												<div class="flex gap-1">
													<button onclick={() => handleQuarantineFile(file)} class="rounded px-2 py-1 text-xs text-orange-400 transition hover:bg-orange-500/10">
														{file.quarantined_at ? 'Unquarantine' : 'Quarantine'}
													</button>
													<button onclick={() => handleDeleteFile(file)} class="rounded px-2 py-1 text-xs text-red-400 transition hover:bg-red-500/10">Delete</button>
												</div>
											</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					{/if}
					<!-- Pagination -->
					{#if filesResponse && filesResponse.total > filesResponse.per_page}
						<div class="mt-4 flex items-center justify-between text-xs text-[var(--text-secondary)]">
							<span>Page {filesResponse.page} of {Math.ceil(filesResponse.total / filesResponse.per_page)} ({filesResponse.total} files)</span>
							<div class="flex gap-2">
								<button disabled={filesPage <= 1} onclick={() => { filesPage--; loadFiles(); }} class="rounded border border-white/10 px-3 py-1 transition hover:bg-white/5 disabled:opacity-30">Prev</button>
								<button disabled={filesPage >= Math.ceil(filesResponse.total / filesResponse.per_page)} onclick={() => { filesPage++; loadFiles(); }} class="rounded border border-white/10 px-3 py-1 transition hover:bg-white/5 disabled:opacity-30">Next</button>
							</div>
						</div>
					{/if}
				</section>

			<!-- ═══ REPORTS TAB ═══ -->
			{:else if activeTab === 'reports'}
				<section class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
					<div class="mb-4 flex items-center justify-between">
						<h2 class="text-lg font-semibold">Content Reports</h2>
						<div class="flex items-center gap-2">
							<label for="report-status" class="text-xs text-[var(--text-secondary)]">Filter:</label>
							<select id="report-status" bind:value={reportsStatusFilter} onchange={() => { reportsPage = 1; loadReports(); }} class="rounded border border-white/10 bg-[var(--bg-primary)] px-2 py-1 text-xs text-[var(--text-primary)]">
								<option value="">All</option>
								<option value="pending">Pending</option>
								<option value="reviewed">Reviewed</option>
								<option value="resolved">Resolved</option>
								<option value="dismissed">Dismissed</option>
							</select>
						</div>
					</div>

					{#if reportsLoading}
						<p class="text-sm text-[var(--text-secondary)]">Loading reports...</p>
					{:else if !reportsResponse || reportsResponse.reports.length === 0}
						<p class="text-sm text-[var(--text-secondary)]">No reports found.</p>
					{:else}
						<div class="space-y-3">
							{#each reportsResponse.reports as report}
								<div class="rounded-lg border border-white/5 bg-[var(--bg-primary)] p-4">
									<div class="mb-2 flex items-start justify-between">
										<div class="flex items-center gap-2">
											<span class="rounded px-2 py-0.5 text-xs font-medium {
												report.status === 'pending' ? 'bg-yellow-500/10 text-yellow-400' :
												report.status === 'reviewed' ? 'bg-blue-500/10 text-blue-400' :
												report.status === 'resolved' ? 'bg-green-500/10 text-green-400' :
												'bg-gray-500/10 text-gray-400'
											}">{report.status}</span>
											<span class="rounded bg-white/5 px-2 py-0.5 text-xs">{report.report_type}</span>
										</div>
										<span class="text-xs text-[var(--text-secondary)]">{new Date(report.created_at).toLocaleString()}</span>
									</div>
									<p class="mb-2 text-sm">{report.reason}</p>
									<div class="flex items-center gap-4 text-xs text-[var(--text-secondary)]">
										<span>Target: <button class="cursor-pointer font-mono hover:text-[var(--text-primary)]" onclick={() => copyText(report.target_id)}>{report.target_id.slice(0, 8)}</button></span>
										<span>Reporter: <button class="cursor-pointer font-mono hover:text-[var(--text-primary)]" onclick={() => copyText(report.reporter_id)}>{report.reporter_id.slice(0, 8)}</button></span>
										{#if report.admin_notes}
											<span>Notes: {report.admin_notes}</span>
										{/if}
									</div>
									{#if report.status === 'pending'}
										{#if reviewingReportId === report.id}
											<div class="mt-3 flex items-end gap-2 border-t border-white/5 pt-3">
												<div>
													<label for="review-status-{report.id}" class="mb-1 block text-xs text-[var(--text-secondary)]">Status</label>
													<select id="review-status-{report.id}" bind:value={reviewStatus} class="rounded border border-white/10 bg-[var(--bg-secondary)] px-2 py-1 text-xs text-[var(--text-primary)]">
														<option value="reviewed">Reviewed</option>
														<option value="resolved">Resolved</option>
														<option value="dismissed">Dismissed</option>
													</select>
												</div>
												<div class="flex-1">
													<label for="review-notes-{report.id}" class="mb-1 block text-xs text-[var(--text-secondary)]">Notes (optional)</label>
													<input id="review-notes-{report.id}" type="text" bind:value={reviewNotes} placeholder="Admin notes..." class="w-full rounded border border-white/10 bg-[var(--bg-secondary)] px-2 py-1 text-xs text-[var(--text-primary)] outline-none focus:border-[var(--accent)]" />
												</div>
												<button onclick={handleReviewReport} class="rounded bg-[var(--accent)] px-3 py-1 text-xs font-medium text-white transition hover:bg-[var(--accent-hover)]">Submit</button>
												<button onclick={() => { reviewingReportId = null; }} class="rounded border border-white/10 px-3 py-1 text-xs text-[var(--text-secondary)] transition hover:bg-white/5">Cancel</button>
											</div>
										{:else}
											<button onclick={() => { reviewingReportId = report.id; reviewStatus = 'reviewed'; reviewNotes = ''; }} class="mt-2 rounded px-2 py-1 text-xs text-[var(--accent)] transition hover:bg-[var(--accent)]/10">Review</button>
										{/if}
									{/if}
								</div>
							{/each}
						</div>
						<!-- Pagination -->
						{#if reportsResponse.total > reportsResponse.per_page}
							<div class="mt-4 flex items-center justify-between text-xs text-[var(--text-secondary)]">
								<span>Page {reportsResponse.page} of {Math.ceil(reportsResponse.total / reportsResponse.per_page)} ({reportsResponse.total} reports)</span>
								<div class="flex gap-2">
									<button disabled={reportsPage <= 1} onclick={() => { reportsPage--; loadReports(); }} class="rounded border border-white/10 px-3 py-1 transition hover:bg-white/5 disabled:opacity-30">Prev</button>
									<button disabled={reportsPage >= Math.ceil(reportsResponse.total / reportsResponse.per_page)} onclick={() => { reportsPage++; loadReports(); }} class="rounded border border-white/10 px-3 py-1 transition hover:bg-white/5 disabled:opacity-30">Next</button>
								</div>
							</div>
						{/if}
					{/if}
				</section>

			<!-- ═══ AUDIT LOG TAB ═══ -->
			{:else if activeTab === 'audit'}
				<section class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
					<div class="mb-4 flex items-center justify-between">
						<h2 class="text-lg font-semibold">Audit Log</h2>
						<div class="flex items-center gap-2">
							<input type="text" bind:value={auditActionFilter} placeholder="Filter action..." class="w-36 rounded border border-white/10 bg-[var(--bg-primary)] px-2 py-1 text-xs text-[var(--text-primary)] outline-none focus:border-[var(--accent)]" />
							<input type="text" bind:value={auditUserFilter} placeholder="User ID..." class="w-36 rounded border border-white/10 bg-[var(--bg-primary)] px-2 py-1 text-xs text-[var(--text-primary)] outline-none focus:border-[var(--accent)]" />
							<button onclick={() => { auditPage = 1; loadAuditLog(); }} class="rounded bg-[var(--accent)] px-3 py-1 text-xs font-medium text-white transition hover:bg-[var(--accent-hover)]">Filter</button>
						</div>
					</div>

					{#if auditLoading}
						<p class="text-sm text-[var(--text-secondary)]">Loading audit log...</p>
					{:else if !auditResponse || auditResponse.entries.length === 0}
						<p class="text-sm text-[var(--text-secondary)]">No audit entries found.</p>
					{:else}
						<div class="overflow-x-auto">
							<table class="w-full text-sm">
								<thead>
									<tr class="border-b border-white/10 text-left text-[var(--text-secondary)]">
										<th class="py-2 pr-3 font-medium">Time</th>
										<th class="py-2 pr-3 font-medium">Action</th>
										<th class="py-2 pr-3 font-medium">User</th>
										<th class="py-2 pr-3 font-medium">IP</th>
										<th class="py-2 font-medium">Details</th>
									</tr>
								</thead>
								<tbody>
									{#each auditResponse.entries as entry}
										<tr class="border-b border-white/5">
											<td class="py-2 pr-3 text-xs text-[var(--text-secondary)]">{new Date(entry.created_at).toLocaleString()}</td>
											<td class="py-2 pr-3">
												<span class="rounded bg-white/5 px-2 py-0.5 font-mono text-xs">{entry.action}</span>
											</td>
											<td class="py-2 pr-3">
												{#if entry.user_id}
													<button class="cursor-pointer font-mono text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)]" onclick={() => copyText(entry.user_id!)}>{entry.user_id.slice(0, 8)}</button>
												{:else}
													<span class="text-xs text-[var(--text-secondary)]">system</span>
												{/if}
											</td>
											<td class="py-2 pr-3 text-xs text-[var(--text-secondary)]">{entry.ip_address ?? '-'}</td>
											<td class="py-2 text-xs text-[var(--text-secondary)]">
												{#if entry.metadata}
													<code class="font-mono">{JSON.stringify(entry.metadata)}</code>
												{:else}
													-
												{/if}
											</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
						<!-- Pagination -->
						{#if auditResponse.total > auditResponse.per_page}
							<div class="mt-4 flex items-center justify-between text-xs text-[var(--text-secondary)]">
								<span>Page {auditResponse.page} of {Math.ceil(auditResponse.total / auditResponse.per_page)} ({auditResponse.total} entries)</span>
								<div class="flex gap-2">
									<button disabled={auditPage <= 1} onclick={() => { auditPage--; loadAuditLog(); }} class="rounded border border-white/10 px-3 py-1 transition hover:bg-white/5 disabled:opacity-30">Prev</button>
									<button disabled={auditPage >= Math.ceil(auditResponse.total / auditResponse.per_page)} onclick={() => { auditPage++; loadAuditLog(); }} class="rounded border border-white/10 px-3 py-1 transition hover:bg-white/5 disabled:opacity-30">Next</button>
								</div>
							</div>
						{/if}
					{/if}
				</section>

			<!-- ═══ SECURITY TAB ═══ -->
			{:else if activeTab === 'security'}
				<!-- Purge Tools -->
				<section class="mb-4 rounded-xl border border-red-500/20 bg-[var(--bg-secondary)] p-6">
					<h2 class="mb-1 text-lg font-semibold text-red-400">Purge Tools</h2>
					<p class="mb-4 text-xs text-[var(--text-secondary)]">Permanently hard-delete messages and files. This cannot be undone.</p>
					<div class="flex flex-wrap items-end gap-3">
						<div>
							<label for="purge-type" class="mb-1 block text-xs text-[var(--text-secondary)]">Target Type</label>
							<select id="purge-type" bind:value={purgeType} class="rounded border border-white/10 bg-[var(--bg-primary)] px-3 py-1.5 text-sm text-[var(--text-primary)]">
								<option value="message">Single Message</option>
								<option value="user">All from User</option>
								<option value="channel">All in Channel</option>
							</select>
						</div>
						<div class="flex-1">
							<label for="purge-id" class="mb-1 block text-xs text-[var(--text-secondary)]">
								{purgeType === 'message' ? 'Message' : purgeType === 'user' ? 'User' : 'Channel'} ID
							</label>
							<input id="purge-id" type="text" bind:value={purgeTargetId} placeholder="UUID..." class="w-full rounded border border-white/10 bg-[var(--bg-primary)] px-3 py-1.5 text-sm text-[var(--text-primary)] outline-none focus:border-red-400" />
						</div>
						<label class="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
							<input type="checkbox" bind:checked={purgeBlockHashes} class="rounded" />
							Block file hashes
						</label>
						<button onclick={handlePurge} disabled={purging} class="rounded-lg bg-red-600 px-4 py-1.5 text-sm font-medium text-white transition hover:bg-red-700 disabled:opacity-50">
							{purging ? 'Purging...' : 'Purge'}
						</button>
					</div>
				</section>

				<!-- Quarantine Tools -->
				<section class="mb-4 rounded-xl border border-orange-500/20 bg-[var(--bg-secondary)] p-6">
					<h2 class="mb-1 text-lg font-semibold text-orange-400">Quick Quarantine</h2>
					<p class="mb-4 text-xs text-[var(--text-secondary)]">Hide a message from view without deleting it. Use the Files tab to quarantine files.</p>
					<div class="flex items-end gap-3">
						<div class="flex-1">
							<label for="quarantine-msg-id" class="mb-1 block text-xs text-[var(--text-secondary)]">Message ID</label>
							<input id="quarantine-msg-id" type="text" placeholder="UUID..." class="w-full rounded border border-white/10 bg-[var(--bg-primary)] px-3 py-1.5 text-sm text-[var(--text-primary)] outline-none focus:border-orange-400" />
						</div>
						<button
							onclick={async () => {
								const input = document.getElementById('quarantine-msg-id') as HTMLInputElement;
								if (!input.value.trim()) { toastStore.error('Enter a message ID'); return; }
								try {
									await quarantineMessage(input.value.trim());
									toastStore.success('Message quarantined');
									input.value = '';
								} catch (err) {
									toastStore.error(err instanceof Error ? err.message : 'Failed to quarantine');
								}
							}}
							class="rounded-lg bg-orange-600 px-4 py-1.5 text-sm font-medium text-white transition hover:bg-orange-700"
						>
							Quarantine
						</button>
						<button
							onclick={async () => {
								const input = document.getElementById('quarantine-msg-id') as HTMLInputElement;
								if (!input.value.trim()) { toastStore.error('Enter a message ID'); return; }
								try {
									await unquarantineMessage(input.value.trim());
									toastStore.success('Message unquarantined');
									input.value = '';
								} catch (err) {
									toastStore.error(err instanceof Error ? err.message : 'Failed to unquarantine');
								}
							}}
							class="rounded-lg border border-orange-500/30 px-4 py-1.5 text-sm font-medium text-orange-400 transition hover:bg-orange-500/10"
						>
							Unquarantine
						</button>
					</div>
				</section>

				<!-- Blocked Hashes -->
				<section class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
					<h2 class="mb-1 text-lg font-semibold">Blocked File Hashes</h2>
					<p class="mb-4 text-xs text-[var(--text-secondary)]">SHA256 hashes of files that will be rejected on upload.</p>

					<!-- Add hash form -->
					<div class="mb-4 flex items-end gap-2">
						<div class="flex-1">
							<label for="new-hash" class="mb-1 block text-xs text-[var(--text-secondary)]">SHA256 Hash (64 hex chars)</label>
							<input id="new-hash" type="text" bind:value={newHashValue} placeholder="e.g. a1b2c3d4..." maxlength="64" class="w-full rounded border border-white/10 bg-[var(--bg-primary)] px-3 py-1.5 font-mono text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]" />
						</div>
						<div class="w-48">
							<label for="hash-reason" class="mb-1 block text-xs text-[var(--text-secondary)]">Reason (optional)</label>
							<input id="hash-reason" type="text" bind:value={newHashReason} placeholder="Reason..." class="w-full rounded border border-white/10 bg-[var(--bg-primary)] px-3 py-1.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]" />
						</div>
						<button onclick={handleAddHash} class="rounded-lg bg-[var(--accent)] px-4 py-1.5 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)]">Block</button>
					</div>

					{#if hashesLoading}
						<p class="text-sm text-[var(--text-secondary)]">Loading blocked hashes...</p>
					{:else if blockedHashes.length === 0}
						<p class="text-sm text-[var(--text-secondary)]">No blocked hashes.</p>
					{:else}
						<div class="overflow-x-auto">
							<table class="w-full text-sm">
								<thead>
									<tr class="border-b border-white/10 text-left text-[var(--text-secondary)]">
										<th class="py-2 pr-3 font-medium">Hash</th>
										<th class="py-2 pr-3 font-medium">Reason</th>
										<th class="py-2 pr-3 font-medium">Added</th>
										<th class="py-2 font-medium">Actions</th>
									</tr>
								</thead>
								<tbody>
									{#each blockedHashes as hash}
										<tr class="border-b border-white/5">
											<td class="py-2 pr-3">
												<button class="cursor-pointer font-mono text-xs hover:text-[var(--text-primary)]" title={hash.hash} onclick={() => copyText(hash.hash)}>{hash.hash.slice(0, 24)}...</button>
											</td>
											<td class="py-2 pr-3 text-xs text-[var(--text-secondary)]">{hash.reason ?? '-'}</td>
											<td class="py-2 pr-3 text-xs text-[var(--text-secondary)]">{new Date(hash.created_at).toLocaleDateString()}</td>
											<td class="py-2">
												<button onclick={() => handleRemoveHash(hash)} class="rounded px-2 py-1 text-xs text-red-400 transition hover:bg-red-500/10">Remove</button>
											</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					{/if}
				</section>
			{:else if activeTab === 'announcements'}
				<section class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
					<h2 class="mb-1 text-lg font-semibold">Server Announcements</h2>
					<p class="mb-4 text-xs text-[var(--text-secondary)]">Publish announcements visible to all users as banners in the chat view. Users can dismiss them individually.</p>

					<!-- Create form -->
					<div class="mb-6 space-y-3 rounded-lg border border-white/10 bg-[var(--bg-primary)] p-4">
						<div>
							<label for="ann-title" class="mb-1 block text-xs text-[var(--text-secondary)]">Title</label>
							<input id="ann-title" type="text" bind:value={newAnnouncementTitle} placeholder="Announcement title..." maxlength="200" class="w-full rounded border border-white/10 bg-[var(--bg-secondary)] px-3 py-1.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]" />
						</div>
						<div>
							<label for="ann-body" class="mb-1 block text-xs text-[var(--text-secondary)]">Body</label>
							<textarea id="ann-body" bind:value={newAnnouncementBody} placeholder="Announcement details..." rows="3" maxlength="5000" class="w-full resize-y rounded border border-white/10 bg-[var(--bg-secondary)] px-3 py-1.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"></textarea>
						</div>
						<button onclick={handleCreateAnnouncement} disabled={creatingAnnouncement || !newAnnouncementTitle.trim() || !newAnnouncementBody.trim()} class="rounded-lg bg-[var(--accent)] px-4 py-2 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:cursor-not-allowed disabled:opacity-50">
							{creatingAnnouncement ? 'Publishing...' : 'Publish Announcement'}
						</button>
					</div>

					<!-- Existing announcements -->
					{#if announcementsLoading}
						<p class="text-sm text-[var(--text-secondary)]">Loading announcements...</p>
					{:else if adminAnnouncements.length === 0}
						<p class="text-sm text-[var(--text-secondary)]">No announcements yet.</p>
					{:else}
						<div class="space-y-3">
							{#each adminAnnouncements as ann (ann.id)}
								<div class="rounded-lg border border-white/10 bg-[var(--bg-primary)] p-4">
									<div class="mb-1 flex items-baseline justify-between gap-2">
										<h3 class="text-sm font-semibold text-[var(--text-primary)]">{ann.title}</h3>
										<span class="shrink-0 text-xs text-[var(--text-secondary)]">{new Date(ann.created_at).toLocaleString()}</span>
									</div>
									<p class="text-sm text-[var(--text-secondary)]">{ann.body}</p>
								</div>
							{/each}
						</div>
					{/if}
				</section>
			{:else if activeTab === 'webhooks'}
				<section class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
					<h2 class="mb-1 text-lg font-semibold">Webhooks Overview</h2>
					<p class="mb-4 text-xs text-[var(--text-secondary)]">All webhooks across all channels. Manage individual webhooks from the channel settings.</p>

					{#if webhooksLoading}
						<p class="text-sm text-[var(--text-secondary)]">Loading webhooks...</p>
					{:else if !webhooksResponse || webhooksResponse.webhooks.length === 0}
						<p class="text-sm text-[var(--text-secondary)]">No webhooks configured.</p>
					{:else}
						<p class="mb-3 text-xs text-[var(--text-secondary)]">{webhooksResponse.total} webhook{webhooksResponse.total === 1 ? '' : 's'}</p>
						<div class="overflow-x-auto">
							<table class="w-full text-sm">
								<thead>
									<tr class="border-b border-white/10 text-left text-xs text-[var(--text-secondary)]">
										<th class="pb-2 pr-4">Name</th>
										<th class="pb-2 pr-4">Channel</th>
										<th class="pb-2 pr-4">Status</th>
										<th class="pb-2">Created</th>
									</tr>
								</thead>
								<tbody>
									{#each webhooksResponse.webhooks as wh (wh.id)}
										<tr class="border-b border-white/5">
											<td class="py-2 pr-4 font-medium text-[var(--text-primary)]">{wh.name}</td>
											<td class="py-2 pr-4 font-mono text-xs text-[var(--text-secondary)]">{wh.channel_id.slice(0, 8)}...</td>
											<td class="py-2 pr-4">
												<span class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {wh.active ? 'bg-green-500/10 text-green-400' : 'bg-red-500/10 text-red-400'}">
													{wh.active ? 'Active' : 'Inactive'}
												</span>
											</td>
											<td class="py-2 text-xs text-[var(--text-secondary)]">{new Date(wh.created_at).toLocaleDateString()}</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					{/if}
				</section>
			{:else if activeTab === 'settings'}
				<section class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
					<h2 class="mb-1 text-lg font-semibold">Instance Settings</h2>
					<p class="mb-4 text-xs text-[var(--text-secondary)]">Configure server-wide limits and defaults. Changes take effect immediately for new connections.</p>

					{#if settingsLoading}
						<p class="text-sm text-[var(--text-secondary)]">Loading settings...</p>
					{:else}
						<div class="space-y-4">
							<div>
								<label for="setting-cache" class="mb-1 block text-sm font-medium text-[var(--text-primary)]">Message cache per channel</label>
								<p class="mb-2 text-xs text-[var(--text-secondary)]">Max messages kept in memory per channel on each client. Pinned messages are always preserved. Range: 50-10,000.</p>
								<input id="setting-cache" type="number" bind:value={settingsMaxCache} min="50" max="10000" class="w-40 rounded border border-white/10 bg-[var(--bg-primary)] px-3 py-1.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]" />
							</div>
							<div>
								<label for="setting-pins" class="mb-1 block text-sm font-medium text-[var(--text-primary)]">Max pins per channel</label>
								<p class="mb-2 text-xs text-[var(--text-secondary)]">Maximum number of pinned messages allowed per channel. Range: 1-200.</p>
								<input id="setting-pins" type="number" bind:value={settingsMaxPins} min="1" max="200" class="w-40 rounded border border-white/10 bg-[var(--bg-primary)] px-3 py-1.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]" />
							</div>
							<div>
								<label for="setting-e2e" class="mb-1 block text-sm font-medium text-[var(--text-primary)]">End-to-end encryption</label>
								<p class="mb-2 text-xs text-[var(--text-secondary)]">When enabled, new messages are encrypted client-side using the Signal protocol (X3DH + Double Ratchet for DMs, Sender Keys for groups). Old plaintext messages remain readable.</p>
								<label class="inline-flex cursor-pointer items-center gap-2">
									<input id="setting-e2e" type="checkbox" checked={settingsE2eEnabled === 'true'} onchange={(e) => { settingsE2eEnabled = (e.target as HTMLInputElement).checked ? 'true' : 'false'; }} class="h-4 w-4 rounded border-white/20 bg-[var(--bg-primary)] accent-[var(--accent)]" />
									<span class="text-sm text-[var(--text-primary)]">{settingsE2eEnabled === 'true' ? 'Enabled' : 'Disabled'}</span>
								</label>
							</div>
							<button onclick={saveSettings} disabled={savingSettings} class="rounded-lg bg-[var(--accent)] px-4 py-2 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:cursor-not-allowed disabled:opacity-50">
								{savingSettings ? 'Saving...' : 'Save Settings'}
							</button>
						</div>
					{/if}
				</section>
			{/if}
		</div>
	</div>

	<!-- Reset Password Modal -->
	{#if resetPasswordUser}
		<!-- svelte-ignore a11y_no_noninteractive_element_interactions a11y_click_events_have_key_events -->
		<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60" role="dialog" aria-modal="true" aria-label="Reset password" onclick={() => { resetPasswordUser = null; }} onkeydown={(e) => { if (e.key === 'Escape') resetPasswordUser = null; }}>
			<!-- svelte-ignore a11y_no_noninteractive_element_interactions a11y_click_events_have_key_events -->
			<form onsubmit={(e) => { e.preventDefault(); submitResetPassword(); }} class="w-full max-w-sm rounded-2xl bg-[var(--bg-secondary)] p-6 shadow-xl" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
				<h3 class="mb-1 text-lg font-bold text-[var(--text-primary)]">Reset Password</h3>
				<p class="mb-4 text-sm text-[var(--text-secondary)]">Set a new password for <strong>{resetPasswordUser.username}</strong></p>
				<input
					type="password"
					bind:value={resetPasswordInput}
					placeholder="New password"
					required
					minlength="8"
					autofocus
					autocomplete="new-password"
					class="mb-2 w-full rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] px-4 py-2.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/30"
				/>
				<p class="mb-4 text-xs text-[var(--text-secondary)]">8+ chars, uppercase, lowercase, digit, special character</p>
				<div class="flex justify-end gap-2">
					<button type="button" onclick={() => { resetPasswordUser = null; }} class="rounded-lg px-4 py-2 text-sm text-[var(--text-secondary)] transition hover:bg-white/5">Cancel</button>
					<button type="submit" disabled={resettingPasswordId !== null || resetPasswordInput.length < 8} class="rounded-lg bg-orange-500 px-4 py-2 text-sm font-medium text-white transition hover:bg-orange-600 disabled:opacity-50">
						{resettingPasswordId ? 'Resetting...' : 'Reset Password'}
					</button>
				</div>
			</form>
		</div>
	{/if}

	<!-- Confirm Dialog Modal -->
	{#if confirmDialog}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
			role="dialog"
			tabindex="-1"
			aria-modal="true"
			aria-label={confirmDialog.title}
			transition:fade={{ duration: 150 }}
			onclick={() => confirmDialog = null}
			onkeydown={(e) => { if (e.key === 'Escape') confirmDialog = null; }}
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
						onkeydown={(e) => { if (e.key === 'Enter') { confirmDialog?.onConfirm(confirmInput, confirmCheckbox); confirmDialog = null; } }}
						autofocus
						class="mb-4 w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
					/>
				{/if}
				{#if confirmDialog.checkboxLabel}
					<label class="mb-4 flex items-center gap-2 text-sm text-[var(--text-secondary)] cursor-pointer">
						<input type="checkbox" bind:checked={confirmCheckbox} class="rounded" />
						{confirmDialog.checkboxLabel}
					</label>
				{/if}
				<div class="flex justify-end gap-2">
					<button
						onclick={() => confirmDialog = null}
						class="rounded-lg px-4 py-2 text-sm font-medium text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					>
						Cancel
					</button>
					<button
						onclick={() => { confirmDialog?.onConfirm(confirmInput, confirmCheckbox); confirmDialog = null; }}
						class="rounded-lg px-4 py-2 text-sm font-medium text-white transition {confirmDialog.danger ? 'bg-[var(--danger)] hover:bg-red-600' : 'bg-[var(--accent)] hover:bg-[var(--accent-hover)]'}"
					>
						{confirmDialog.confirmLabel ?? 'Confirm'}
					</button>
				</div>
			</div>
		</div>
	{/if}
{/if}
