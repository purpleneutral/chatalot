<script lang="ts">
	import { goto } from '$app/navigation';
	import { authStore } from '$lib/stores/auth.svelte';
	import { toastStore } from '$lib/stores/toast.svelte';
	import {
		listUsers, suspendUser, unsuspendUser, deleteUser, setAdmin, resetUserPassword, type AdminUser,
		createRegistrationInvite, listRegistrationInvites, deleteRegistrationInvite,
		type RegistrationInvite
	} from '$lib/api/admin';
	import { onMount } from 'svelte';

	let users = $state<AdminUser[]>([]);
	let loading = $state(false);
	let searchQuery = $state('');
	let searchTimeout: ReturnType<typeof setTimeout>;

	// Invite codes
	let invites = $state<RegistrationInvite[]>([]);
	let invitesLoading = $state(false);
	let showCreateInvite = $state(false);
	let newInviteMaxUses = $state('');
	let newInviteExpiresHours = $state('');
	let creatingInvite = $state(false);

	onMount(() => {
		if (!authStore.isAuthenticated || (!authStore.user?.is_admin && !authStore.user?.is_owner)) {
			goto('/channels');
			return;
		}
		loadUsers();
		loadInvites();
	});

	async function loadUsers() {
		loading = true;
		try {
			users = await listUsers({ search: searchQuery || undefined, limit: 50 });
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to load users');
		} finally {
			loading = false;
		}
	}

	function handleSearch() {
		clearTimeout(searchTimeout);
		searchTimeout = setTimeout(() => loadUsers(), 300);
	}

	async function handleSuspend(user: AdminUser) {
		const reason = prompt('Suspension reason (optional):');
		if (reason === null) return;
		try {
			await suspendUser(user.id, reason || undefined);
			toastStore.success(`Suspended ${user.username}`);
			await loadUsers();
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to suspend user');
		}
	}

	async function handleUnsuspend(user: AdminUser) {
		try {
			await unsuspendUser(user.id);
			toastStore.success(`Unsuspended ${user.username}`);
			await loadUsers();
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to unsuspend user');
		}
	}

	async function handleDelete(user: AdminUser) {
		if (!confirm(`Are you sure you want to delete ${user.username}? This cannot be undone.`)) return;
		try {
			await deleteUser(user.id);
			toastStore.success(`Deleted ${user.username}`);
			await loadUsers();
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to delete user');
		}
	}

	async function handleToggleAdmin(user: AdminUser) {
		const newState = !user.is_admin;
		const action = newState ? 'grant admin to' : 'revoke admin from';
		if (!confirm(`Are you sure you want to ${action} ${user.username}?`)) return;
		try {
			await setAdmin(user.id, newState);
			toastStore.success(`${newState ? 'Granted' : 'Revoked'} admin for ${user.username}`);
			await loadUsers();
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to update admin status');
		}
	}

	async function handleResetPassword(user: AdminUser) {
		const newPassword = prompt(
			`Reset password for ${user.username}\n\nRequirements: 8+ chars, 1 uppercase, 1 lowercase, 1 digit, 1 special character\n\nEnter new password:`
		);
		if (!newPassword) return;
		try {
			await resetUserPassword(user.id, newPassword);
			toastStore.success(`Password reset for ${user.username} — they will need to log in again`);
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to reset password');
		}
	}

	// ── Invite Code Management ──

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
			if (newInviteMaxUses) params.max_uses = parseInt(newInviteMaxUses);
			if (newInviteExpiresHours) params.expires_in_hours = parseInt(newInviteExpiresHours);

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

	async function handleDeleteInvite(invite: RegistrationInvite) {
		if (!confirm(`Delete invite code ${invite.code}?`)) return;
		try {
			await deleteRegistrationInvite(invite.id);
			toastStore.success('Invite deleted');
			await loadInvites();
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to delete invite');
		}
	}

	function copyCode(code: string) {
		navigator.clipboard.writeText(code);
		toastStore.success('Copied to clipboard');
	}
</script>

{#if authStore.isAuthenticated && (authStore.user?.is_admin || authStore.user?.is_owner)}
	<div class="min-h-screen bg-[var(--bg-primary)] text-[var(--text-primary)]">
		<div class="mx-auto max-w-4xl px-6 py-8">
			<!-- Header -->
			<div class="mb-8 flex items-center justify-between">
				<h1 class="text-2xl font-bold">Admin Panel</h1>
				<div class="flex gap-2">
					<button
						onclick={() => goto('/settings')}
						class="rounded-lg border border-white/10 px-4 py-2 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					>
						Settings
					</button>
					<button
						onclick={() => goto('/channels')}
						class="rounded-lg border border-white/10 px-4 py-2 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					>
						Back to Chat
					</button>
				</div>
			</div>

			<!-- User Management -->
			<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
				<div class="mb-4 flex items-center justify-between">
					<h2 class="text-lg font-semibold">User Management</h2>
					<span class="text-sm text-[var(--text-secondary)]">{users.length} users</span>
				</div>

				<!-- Search -->
				<div class="mb-4">
					<input
						type="text"
						bind:value={searchQuery}
						oninput={handleSearch}
						placeholder="Search by username, name, or email..."
						class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-4 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
					/>
				</div>

				{#if loading}
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
													<span class="text-xs font-bold text-white">
														{user.display_name?.[0]?.toUpperCase() ?? '?'}
													</span>
												</div>
												<div>
													<div class="font-medium">
														{user.display_name}
														{#if user.is_admin}
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
													<span class="rounded bg-purple-500/10 px-1.5 py-0.5 text-xs text-purple-400" title="{c.role}">
														{c.name}
													</span>
												{/each}
												{#each user.groups ?? [] as g}
													<span class="rounded bg-blue-500/10 px-1.5 py-0.5 text-xs text-blue-400" title="{g.role}">
														{g.name}
													</span>
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
										<td class="py-3 pr-4 text-[var(--text-secondary)]">
											{new Date(user.created_at).toLocaleDateString()}
										</td>
										<td class="py-3">
											{#if user.id !== authStore.user?.id}
												<div class="flex gap-1">
													{#if user.suspended_at}
														<button
															onclick={() => handleUnsuspend(user)}
															class="rounded px-2 py-1 text-xs text-green-400 transition hover:bg-green-500/10"
														>
															Unsuspend
														</button>
													{:else}
														<button
															onclick={() => handleSuspend(user)}
															class="rounded px-2 py-1 text-xs text-yellow-400 transition hover:bg-yellow-500/10"
														>
															Suspend
														</button>
													{/if}
													<button
														onclick={() => handleResetPassword(user)}
														class="rounded px-2 py-1 text-xs text-orange-400 transition hover:bg-orange-500/10"
													>
														Reset Password
													</button>
													<button
														onclick={() => handleToggleAdmin(user)}
														class="rounded px-2 py-1 text-xs text-[var(--accent)] transition hover:bg-[var(--accent)]/10"
													>
														{user.is_admin ? 'Revoke Admin' : 'Grant Admin'}
													</button>
													<button
														onclick={() => handleDelete(user)}
														class="rounded px-2 py-1 text-xs text-red-400 transition hover:bg-red-500/10"
													>
														Delete
													</button>
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

			<!-- Invite Codes -->
			<section class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
				<div class="mb-4 flex items-center justify-between">
					<h2 class="text-lg font-semibold">Registration Invites</h2>
					<button
						onclick={() => (showCreateInvite = !showCreateInvite)}
						class="rounded-lg bg-[var(--accent)] px-3 py-1.5 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)]"
					>
						{showCreateInvite ? 'Cancel' : 'Generate Invite'}
					</button>
				</div>

				<p class="mb-4 text-xs text-[var(--text-secondary)]">
					Set REGISTRATION_MODE=invite_only to require invite codes for new registrations.
				</p>

				{#if showCreateInvite}
					<div class="mb-4 rounded-lg border border-white/10 bg-[var(--bg-primary)] p-4">
						<div class="flex gap-3">
							<div class="flex-1">
								<label for="admin-invite-max-uses" class="mb-1 block text-xs text-[var(--text-secondary)]">Max Uses (blank = unlimited)</label>
								<input
									id="admin-invite-max-uses"
									type="number"
									bind:value={newInviteMaxUses}
									min="1"
									placeholder="Unlimited"
									class="w-full rounded border border-white/10 bg-[var(--bg-secondary)] px-3 py-1.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
								/>
							</div>
							<div class="flex-1">
								<label for="admin-invite-expires" class="mb-1 block text-xs text-[var(--text-secondary)]">Expires In (hours, blank = never)</label>
								<input
									id="admin-invite-expires"
									type="number"
									bind:value={newInviteExpiresHours}
									min="1"
									placeholder="Never"
									class="w-full rounded border border-white/10 bg-[var(--bg-secondary)] px-3 py-1.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
								/>
							</div>
							<div class="flex items-end">
								<button
									onclick={handleCreateInvite}
									disabled={creatingInvite}
									class="rounded-lg bg-[var(--accent)] px-4 py-1.5 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:opacity-50"
								>
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
										<td class="py-3 pr-4">
											<code class="rounded bg-white/5 px-2 py-0.5 font-mono text-xs">{invite.code}</code>
										</td>
										<td class="py-3 pr-4 text-[var(--text-secondary)]">
											{invite.used_count}{invite.max_uses !== null ? `/${invite.max_uses}` : ''}
										</td>
										<td class="py-3 pr-4 text-[var(--text-secondary)]">
											{invite.expires_at ? new Date(invite.expires_at).toLocaleString() : 'Never'}
										</td>
										<td class="py-3 pr-4 text-[var(--text-secondary)]">
											{new Date(invite.created_at).toLocaleDateString()}
										</td>
										<td class="py-3">
											<div class="flex gap-1">
												<button
													onclick={() => copyCode(invite.code)}
													class="rounded px-2 py-1 text-xs text-[var(--accent)] transition hover:bg-[var(--accent)]/10"
												>
													Copy
												</button>
												<button
													onclick={() => handleDeleteInvite(invite)}
													class="rounded px-2 py-1 text-xs text-red-400 transition hover:bg-red-500/10"
												>
													Delete
												</button>
											</div>
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
			</section>
		</div>
	</div>
{/if}
