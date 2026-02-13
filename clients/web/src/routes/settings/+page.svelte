<script lang="ts">
	import { goto } from '$app/navigation';
	import { authStore } from '$lib/stores/auth.svelte';
	import { themeStore } from '$lib/stores/theme.svelte';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { soundStore } from '$lib/stores/sound.svelte';
	import { notificationStore } from '$lib/stores/notification.svelte';
	import { setupTotp, verifyTotp, disableTotp, type TotpSetup } from '$lib/api/totp';
	import { changePassword, updateProfile, uploadAvatar, deleteAccount, logoutAll, listSessions, revokeSession, type SessionInfo } from '$lib/api/account';
	import { isTauri, getServerUrl, clearServerUrl } from '$lib/env';
	import Avatar from '$lib/components/Avatar.svelte';
	import { onMount } from 'svelte';

	let isDesktop = $derived(isTauri());
	let serverUrl = $derived(isDesktop ? getServerUrl() : null);

	let totpSetup = $state<TotpSetup | null>(null);
	let totpCode = $state('');
	let disableCode = $state('');
	let totpMessage = $state('');
	let totpError = $state('');
	let showTotpSetup = $state(false);
	let showTotpDisable = $state(false);

	// Profile editing
	let editDisplayName = $state(authStore.user?.display_name ?? '');
	let editCustomStatus = $state(authStore.user?.custom_status ?? '');
	let profileSaving = $state(false);
	let profileMessage = $state('');
	let profileError = $state('');

	// Avatar upload
	let avatarInputEl: HTMLInputElement | undefined = $state();
	let avatarUploading = $state(false);

	// Change password
	let currentPassword = $state('');
	let newPassword = $state('');
	let confirmPassword = $state('');
	let passwordSaving = $state(false);
	let passwordError = $state('');

	// Password strength checks
	let pwHasLength = $derived(newPassword.length >= 8);
	let pwHasUpper = $derived(/[A-Z]/.test(newPassword));
	let pwHasLower = $derived(/[a-z]/.test(newPassword));
	let pwHasDigit = $derived(/[0-9]/.test(newPassword));
	let pwHasSpecial = $derived(/[^A-Za-z0-9]/.test(newPassword));
	let pwAllMet = $derived(pwHasLength && pwHasUpper && pwHasLower && pwHasDigit && pwHasSpecial);

	// Sessions
	let sessions = $state<SessionInfo[]>([]);
	let sessionsLoading = $state(false);
	let sessionsError = $state('');

	// Delete account
	let showDeleteConfirm = $state(false);
	let deletePassword = $state('');
	let deleteError = $state('');
	let deleting = $state(false);

	onMount(async () => {
		if (!authStore.isAuthenticated) {
			goto('/login');
			return;
		}
		await loadSessions();
	});

	async function loadSessions() {
		sessionsLoading = true;
		sessionsError = '';
		try {
			sessions = await listSessions();
		} catch (err) {
			sessionsError = err instanceof Error ? err.message : 'Failed to load sessions';
		} finally {
			sessionsLoading = false;
		}
	}

	async function handleAvatarUpload(e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;

		avatarUploading = true;
		profileError = '';
		try {
			const updated = await uploadAvatar(file);
			authStore.updateUser(updated);
			profileMessage = 'Avatar updated.';
		} catch (err) {
			profileError = err instanceof Error ? err.message : 'Failed to upload avatar';
		} finally {
			avatarUploading = false;
			if (avatarInputEl) avatarInputEl.value = '';
		}
	}

	async function handleRemoveAvatar() {
		profileSaving = true;
		profileError = '';
		try {
			const updated = await updateProfile({ avatar_url: null });
			authStore.updateUser(updated);
			profileMessage = 'Avatar removed.';
		} catch (err) {
			profileError = err instanceof Error ? err.message : 'Failed to remove avatar';
		} finally {
			profileSaving = false;
		}
	}

	async function handleProfileSave() {
		profileSaving = true;
		profileError = '';
		profileMessage = '';
		try {
			const updated = await updateProfile({
				display_name: editDisplayName || undefined,
				custom_status: editCustomStatus || undefined
			});
			authStore.updateUser(updated);
			profileMessage = 'Profile updated.';
		} catch (err) {
			profileError = err instanceof Error ? err.message : 'Failed to update profile';
		} finally {
			profileSaving = false;
		}
	}

	async function handleChangePassword(e: SubmitEvent) {
		e.preventDefault();
		passwordError = '';

		if (newPassword !== confirmPassword) {
			passwordError = 'Passwords do not match.';
			return;
		}
		if (!pwAllMet) {
			passwordError = 'Password does not meet all requirements.';
			return;
		}

		passwordSaving = true;
		try {
			await changePassword(currentPassword, newPassword);
			authStore.logout();
			goto('/login');
		} catch (err) {
			passwordError = err instanceof Error ? err.message : 'Failed to change password';
		} finally {
			passwordSaving = false;
		}
	}

	async function handleRevokeSession(id: string) {
		try {
			await revokeSession(id);
			sessions = sessions.filter(s => s.id !== id);
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to revoke session');
		}
	}

	async function handleLogoutAll() {
		try {
			await logoutAll();
			authStore.logout();
			goto('/login');
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to logout');
		}
	}

	async function handleDeleteAccount(e: SubmitEvent) {
		e.preventDefault();
		deleteError = '';
		deleting = true;
		try {
			await deleteAccount(deletePassword);
			authStore.logout();
			goto('/login');
		} catch (err) {
			deleteError = err instanceof Error ? err.message : 'Failed to delete account';
		} finally {
			deleting = false;
		}
	}

	async function handleSetupTotp() {
		totpError = '';
		totpMessage = '';
		try {
			totpSetup = await setupTotp();
			showTotpSetup = true;
		} catch (err) {
			totpError = err instanceof Error ? err.message : 'Failed to setup 2FA';
		}
	}

	async function handleVerifyTotp(e: SubmitEvent) {
		e.preventDefault();
		totpError = '';
		try {
			await verifyTotp(totpCode);
			totpMessage = '2FA enabled successfully!';
			showTotpSetup = false;
			totpSetup = null;
			totpCode = '';
		} catch (err) {
			totpError = err instanceof Error ? err.message : 'Invalid code';
		}
	}

	async function handleDisableTotp(e: SubmitEvent) {
		e.preventDefault();
		totpError = '';
		try {
			await disableTotp(disableCode);
			totpMessage = '2FA disabled.';
			showTotpDisable = false;
			disableCode = '';
		} catch (err) {
			totpError = err instanceof Error ? err.message : 'Invalid code';
		}
	}
</script>

{#if authStore.isAuthenticated}
	<div class="min-h-screen bg-[var(--bg-primary)] text-[var(--text-primary)]">
		<div class="mx-auto max-w-2xl px-6 py-8">
			<!-- Header -->
			<div class="mb-8 flex items-center justify-between">
				<h1 class="text-2xl font-bold">Settings</h1>
				<div class="flex gap-2">
					{#if authStore.user?.is_admin}
						<button
							onclick={() => goto('/admin')}
							class="rounded-lg border border-white/10 px-4 py-2 text-sm text-[var(--accent)] transition hover:bg-white/5"
						>
							Admin Panel
						</button>
					{/if}
					<button
						onclick={() => goto('/channels')}
						class="rounded-lg border border-white/10 px-4 py-2 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					>
						Back to Chat
					</button>
				</div>
			</div>

			<!-- Profile Section (editable) -->
			<section class="mb-8 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
				<h2 class="mb-4 text-lg font-semibold">Profile</h2>

				{#if profileMessage}
					<div class="mb-4 rounded-lg bg-green-500/10 border border-green-500/20 px-4 py-3 text-sm text-green-400">
						{profileMessage}
					</div>
				{/if}
				{#if profileError}
					<div class="mb-4 rounded-lg bg-red-500/10 border border-red-500/20 px-4 py-3 text-sm text-red-400">
						{profileError}
					</div>
				{/if}

				<div class="mb-4 flex items-center gap-4">
					<div class="group relative">
						{#if authStore.user}
							<Avatar userId={authStore.user.id} size="lg" />
						{/if}
						<button
							onclick={() => avatarInputEl?.click()}
							disabled={avatarUploading}
							class="absolute inset-0 flex items-center justify-center rounded-full bg-black/50 opacity-0 transition group-hover:opacity-100 disabled:cursor-wait"
							title="Change avatar"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z" /><circle cx="12" cy="13" r="4" />
							</svg>
						</button>
						<input
							bind:this={avatarInputEl}
							type="file"
							accept="image/png,image/jpeg,image/webp,image/gif"
							onchange={handleAvatarUpload}
							class="hidden"
						/>
					</div>
					<div>
						<div class="text-sm text-[var(--text-secondary)]">@{authStore.user?.username}</div>
						{#if authStore.user?.avatar_url}
							<button
								onclick={handleRemoveAvatar}
								class="mt-1 text-xs text-[var(--danger)] hover:underline"
							>
								Remove avatar
							</button>
						{/if}
					</div>
				</div>

				<div class="space-y-4">
					<div>
						<label for="displayName" class="mb-1 block text-sm font-medium">Display Name</label>
						<input
							id="displayName"
							type="text"
							bind:value={editDisplayName}
							maxlength="64"
							class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
						/>
					</div>
					<div>
						<label for="customStatus" class="mb-1 block text-sm font-medium">Custom Status</label>
						<input
							id="customStatus"
							type="text"
							bind:value={editCustomStatus}
							maxlength="128"
							placeholder="What's on your mind?"
							class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
						/>
					</div>
					<button
						onclick={handleProfileSave}
						disabled={profileSaving}
						class="rounded-lg bg-[var(--accent)] px-4 py-2 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:opacity-50"
					>
						{profileSaving ? 'Saving...' : 'Save Profile'}
					</button>
				</div>
			</section>

			<!-- Change Password -->
			<section class="mb-8 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
				<h2 class="mb-4 text-lg font-semibold">Change Password</h2>
				<p class="mb-4 text-sm text-[var(--text-secondary)]">
					Changing your password will sign you out of all devices.
				</p>

				{#if passwordError}
					<div class="mb-4 rounded-lg bg-red-500/10 border border-red-500/20 px-4 py-3 text-sm text-red-400">
						{passwordError}
					</div>
				{/if}

				<form onsubmit={handleChangePassword} class="space-y-4">
					<div>
						<label for="currentPw" class="mb-1 block text-sm font-medium">Current Password</label>
						<input
							id="currentPw"
							type="password"
							bind:value={currentPassword}
							required
							class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
						/>
					</div>
					<div>
						<label for="newPw" class="mb-1 block text-sm font-medium">New Password</label>
						<input
							id="newPw"
							type="password"
							bind:value={newPassword}
							required
							class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
						/>
						{#if newPassword.length > 0}
							<div class="mt-2 space-y-1 text-xs">
								<div class={pwHasLength ? 'text-green-400' : 'text-[var(--text-secondary)]'}>
									{pwHasLength ? '\u2713' : '\u2717'} At least 8 characters
								</div>
								<div class={pwHasUpper ? 'text-green-400' : 'text-[var(--text-secondary)]'}>
									{pwHasUpper ? '\u2713' : '\u2717'} One uppercase letter
								</div>
								<div class={pwHasLower ? 'text-green-400' : 'text-[var(--text-secondary)]'}>
									{pwHasLower ? '\u2713' : '\u2717'} One lowercase letter
								</div>
								<div class={pwHasDigit ? 'text-green-400' : 'text-[var(--text-secondary)]'}>
									{pwHasDigit ? '\u2713' : '\u2717'} One digit
								</div>
								<div class={pwHasSpecial ? 'text-green-400' : 'text-[var(--text-secondary)]'}>
									{pwHasSpecial ? '\u2713' : '\u2717'} One special character
								</div>
							</div>
						{/if}
					</div>
					<div>
						<label for="confirmPw" class="mb-1 block text-sm font-medium">Confirm New Password</label>
						<input
							id="confirmPw"
							type="password"
							bind:value={confirmPassword}
							required
							class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
						/>
					</div>
					<button
						type="submit"
						disabled={passwordSaving || !pwAllMet || !currentPassword}
						class="rounded-lg bg-[var(--accent)] px-4 py-2 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:opacity-50"
					>
						{passwordSaving ? 'Changing...' : 'Change Password'}
					</button>
				</form>
			</section>

			<!-- Appearance -->
			<section class="mb-8 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
				<h2 class="mb-4 text-lg font-semibold">Appearance</h2>
				<div class="flex items-center justify-between">
					<div>
						<div class="font-medium">Theme</div>
						<div class="text-sm text-[var(--text-secondary)]">
							{themeStore.current === 'system' ? `System (${themeStore.resolved === 'dark' ? 'Dark' : 'Light'})` : themeStore.resolved === 'dark' ? 'Dark mode' : 'Light mode'}
						</div>
					</div>
					<div class="flex rounded-lg border border-white/10 overflow-hidden">
						<button
							onclick={() => themeStore.set('dark')}
							class="px-3 py-1.5 text-sm transition {themeStore.current === 'dark' ? 'bg-[var(--accent)] text-white' : 'text-[var(--text-secondary)] hover:bg-white/5'}"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="inline h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
							</svg>
						</button>
						<button
							onclick={() => themeStore.set('light')}
							class="border-x border-white/10 px-3 py-1.5 text-sm transition {themeStore.current === 'light' ? 'bg-[var(--accent)] text-white' : 'text-[var(--text-secondary)] hover:bg-white/5'}"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="inline h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<circle cx="12" cy="12" r="5" /><line x1="12" y1="1" x2="12" y2="3" /><line x1="12" y1="21" x2="12" y2="23" /><line x1="4.22" y1="4.22" x2="5.64" y2="5.64" /><line x1="18.36" y1="18.36" x2="19.78" y2="19.78" /><line x1="1" y1="12" x2="3" y2="12" /><line x1="21" y1="12" x2="23" y2="12" /><line x1="4.22" y1="19.78" x2="5.64" y2="18.36" /><line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
							</svg>
						</button>
						<button
							onclick={() => themeStore.set('system')}
							class="px-3 py-1.5 text-sm transition {themeStore.current === 'system' ? 'bg-[var(--accent)] text-white' : 'text-[var(--text-secondary)] hover:bg-white/5'}"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="inline h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<rect x="2" y="3" width="20" height="14" rx="2" ry="2" /><line x1="8" y1="21" x2="16" y2="21" /><line x1="12" y1="17" x2="12" y2="21" />
							</svg>
						</button>
					</div>
				</div>
			</section>

			<!-- Notifications -->
			<section class="mb-8 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
				<h2 class="mb-4 text-lg font-semibold">Notifications</h2>
				<div class="space-y-4">
					<div class="flex items-center justify-between">
						<div>
							<div class="font-medium">DM message sound</div>
							<div class="text-sm text-[var(--text-secondary)]">Play a sound when you receive a direct message</div>
						</div>
						<div class="flex items-center gap-2">
							<button
								onclick={() => soundStore.playDmNotification()}
								class="rounded px-2 py-1 text-xs text-[var(--accent)] transition hover:bg-white/5"
							>Test</button>
							<button
								onclick={() => { soundStore.preferences.dmMessage = !soundStore.preferences.dmMessage; soundStore.save(); }}
								class="relative h-8 w-14 rounded-full bg-[var(--bg-tertiary)] transition"
								aria-label="Toggle DM sound"
							>
								<span class="absolute top-1 left-1 h-6 w-6 rounded-full transition-transform {soundStore.preferences.dmMessage ? 'translate-x-6 bg-[var(--accent)]' : 'bg-[var(--text-secondary)]'}"></span>
							</button>
						</div>
					</div>

					<div class="flex items-center justify-between">
						<div>
							<div class="font-medium">Channel message sound</div>
							<div class="text-sm text-[var(--text-secondary)]">Play a sound for new messages in channels</div>
						</div>
						<div class="flex items-center gap-2">
							<button
								onclick={() => soundStore.playChannelNotification()}
								class="rounded px-2 py-1 text-xs text-[var(--accent)] transition hover:bg-white/5"
							>Test</button>
							<button
								onclick={() => { soundStore.preferences.channelMessage = !soundStore.preferences.channelMessage; soundStore.save(); }}
								class="relative h-8 w-14 rounded-full bg-[var(--bg-tertiary)] transition"
								aria-label="Toggle channel sound"
							>
								<span class="absolute top-1 left-1 h-6 w-6 rounded-full transition-transform {soundStore.preferences.channelMessage ? 'translate-x-6 bg-[var(--accent)]' : 'bg-[var(--text-secondary)]'}"></span>
							</button>
						</div>
					</div>

					<div class="flex items-center justify-between">
						<div>
							<div class="font-medium">Voice join/leave sounds</div>
							<div class="text-sm text-[var(--text-secondary)]">Play a sound when someone joins or leaves voice</div>
						</div>
						<div class="flex items-center gap-2">
							<button
								onclick={() => soundStore.playVoiceJoin()}
								class="rounded px-2 py-1 text-xs text-[var(--accent)] transition hover:bg-white/5"
							>Test</button>
							<button
								onclick={() => { soundStore.preferences.voiceJoin = !soundStore.preferences.voiceJoin; soundStore.preferences.voiceLeave = !soundStore.preferences.voiceLeave; soundStore.save(); }}
								class="relative h-8 w-14 rounded-full bg-[var(--bg-tertiary)] transition"
								aria-label="Toggle voice sounds"
							>
								<span class="absolute top-1 left-1 h-6 w-6 rounded-full transition-transform {soundStore.preferences.voiceJoin ? 'translate-x-6 bg-[var(--accent)]' : 'bg-[var(--text-secondary)]'}"></span>
							</button>
						</div>
					</div>

					<div>
						<div class="mb-2 flex items-center justify-between">
							<div class="font-medium">Volume</div>
							<span class="text-sm text-[var(--text-secondary)]">{Math.round(soundStore.preferences.volume * 100)}%</span>
						</div>
						<input
							type="range"
							min="0"
							max="1"
							step="0.05"
							bind:value={soundStore.preferences.volume}
							oninput={() => soundStore.save()}
							class="w-full accent-[var(--accent)]"
						/>
					</div>
					<!-- Mention sound -->
					<div class="flex items-center justify-between">
						<div>
							<div class="font-medium">Mention sound</div>
							<div class="text-sm text-[var(--text-secondary)]">Play a distinct sound when you are @mentioned</div>
						</div>
						<div class="flex items-center gap-2">
							<button
								onclick={() => soundStore.playMentionNotification()}
								class="rounded px-2 py-1 text-xs text-[var(--accent)] transition hover:bg-white/5"
							>Test</button>
							<button
								onclick={() => { soundStore.preferences.mentionMessage = !soundStore.preferences.mentionMessage; soundStore.save(); }}
								class="relative h-8 w-14 rounded-full bg-[var(--bg-tertiary)] transition"
								aria-label="Toggle mention sound"
							>
								<span class="absolute top-1 left-1 h-6 w-6 rounded-full transition-transform {soundStore.preferences.mentionMessage ? 'translate-x-6 bg-[var(--accent)]' : 'bg-[var(--text-secondary)]'}"></span>
							</button>
						</div>
					</div>

					<div class="my-4 border-t border-white/10"></div>

					<h3 class="mb-3 text-sm font-semibold text-[var(--text-secondary)] uppercase tracking-wider">Desktop Notifications</h3>

					<!-- Enable desktop notifications -->
					<div class="flex items-center justify-between">
						<div>
							<div class="font-medium">Desktop notifications</div>
							<div class="text-sm text-[var(--text-secondary)]">Show OS-level notifications for new messages</div>
						</div>
						<div class="flex items-center gap-2">
							{#if notificationStore.permissionState === 'unsupported'}
								<span class="text-xs text-[var(--text-secondary)]">Not supported</span>
							{:else if notificationStore.permissionState === 'denied'}
								<span class="text-xs text-red-400">Blocked by browser</span>
							{:else}
								<button
									onclick={async () => {
										if (notificationStore.preferences.desktopEnabled) {
											notificationStore.preferences = { ...notificationStore.preferences, desktopEnabled: false };
											notificationStore.save();
										} else {
											await notificationStore.requestPermission();
										}
									}}
									class="relative h-8 w-14 rounded-full bg-[var(--bg-tertiary)] transition"
									aria-label="Toggle desktop notifications"
								>
									<span class="absolute top-1 left-1 h-6 w-6 rounded-full transition-transform {notificationStore.preferences.desktopEnabled ? 'translate-x-6 bg-[var(--accent)]' : 'bg-[var(--text-secondary)]'}"></span>
								</button>
							{/if}
						</div>
					</div>

					<!-- Default notification level -->
					<div class="mt-4 flex items-center justify-between">
						<div>
							<div class="font-medium">Default channel notifications</div>
							<div class="text-sm text-[var(--text-secondary)]">Notification level for channels without a custom setting</div>
						</div>
						<select
							value={notificationStore.preferences.defaultChannelLevel}
							onchange={(e) => {
								notificationStore.preferences = { ...notificationStore.preferences, defaultChannelLevel: e.currentTarget.value as any };
								notificationStore.save();
							}}
							class="rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-1.5 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
						>
							<option value="all">All messages</option>
							<option value="mentions">Only @mentions</option>
							<option value="nothing">Nothing</option>
						</select>
					</div>
				</div>
			</section>

			<!-- Two-Factor Authentication -->
			<section class="mb-8 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
				<h2 class="mb-4 text-lg font-semibold">Two-Factor Authentication</h2>
				<p class="mb-4 text-sm text-[var(--text-secondary)]">
					Add an extra layer of security to your account with TOTP-based 2FA.
				</p>

				{#if totpMessage}
					<div class="mb-4 rounded-lg bg-green-500/10 border border-green-500/20 px-4 py-3 text-sm text-green-400">
						{totpMessage}
					</div>
				{/if}
				{#if totpError}
					<div class="mb-4 rounded-lg bg-red-500/10 border border-red-500/20 px-4 py-3 text-sm text-red-400">
						{totpError}
					</div>
				{/if}

				{#if showTotpSetup && totpSetup}
					<div class="rounded-lg border border-white/10 bg-[var(--bg-primary)] p-4">
						<p class="mb-3 text-sm">
							Scan this QR code with your authenticator app, or enter the secret manually:
						</p>
						<div class="mb-3 rounded-lg bg-white p-4 text-center">
							<!-- In production, render QR code from totpSetup.otpauth_url -->
							<p class="text-xs text-gray-500">QR Code for: {totpSetup.otpauth_url}</p>
						</div>
						<div class="mb-4">
							<span class="mb-1 block text-xs text-[var(--text-secondary)]">Manual entry secret</span>
							<code class="block rounded bg-[var(--bg-tertiary)] px-3 py-2 text-sm font-mono select-all">
								{totpSetup.secret}
							</code>
						</div>
						<form onsubmit={handleVerifyTotp} class="flex gap-2">
							<input
								type="text"
								bind:value={totpCode}
								placeholder="Enter 6-digit code"
								maxlength="6"
								pattern="[0-9]{6}"
								class="flex-1 rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
							/>
							<button
								type="submit"
								class="rounded-lg bg-[var(--accent)] px-4 py-2 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)]"
							>
								Verify & Enable
							</button>
						</form>
					</div>
				{:else if showTotpDisable}
					<form onsubmit={handleDisableTotp} class="flex gap-2">
						<input
							type="text"
							bind:value={disableCode}
							placeholder="Enter 2FA code to disable"
							maxlength="6"
							pattern="[0-9]{6}"
							class="flex-1 rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
						/>
						<button
							type="submit"
							class="rounded-lg bg-[var(--danger)] px-4 py-2 text-sm font-medium text-white transition hover:opacity-90"
						>
							Disable 2FA
						</button>
						<button
							type="button"
							onclick={() => { showTotpDisable = false; disableCode = ''; }}
							class="rounded-lg border border-white/10 px-4 py-2 text-sm text-[var(--text-secondary)] transition hover:bg-white/5"
						>
							Cancel
						</button>
					</form>
				{:else}
					<div class="flex gap-3">
						<button
							onclick={handleSetupTotp}
							class="rounded-lg bg-[var(--accent)] px-4 py-2 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)]"
						>
							Enable 2FA
						</button>
						<button
							onclick={() => (showTotpDisable = true)}
							class="rounded-lg border border-white/10 px-4 py-2 text-sm text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
						>
							Disable 2FA
						</button>
					</div>
				{/if}
			</section>

			<!-- Active Sessions -->
			<section class="mb-8 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
				<div class="mb-4 flex items-center justify-between">
					<h2 class="text-lg font-semibold">Active Sessions</h2>
					<button
						onclick={handleLogoutAll}
						class="rounded-lg border border-red-500/20 px-3 py-1.5 text-xs font-medium text-red-400 transition hover:bg-red-500/10"
					>
						Revoke All
					</button>
				</div>

				{#if sessionsLoading}
					<p class="text-sm text-[var(--text-secondary)]">Loading sessions...</p>
				{:else if sessionsError}
					<p class="text-sm text-red-400">{sessionsError}</p>
				{:else if sessions.length === 0}
					<p class="text-sm text-[var(--text-secondary)]">No active sessions.</p>
				{:else}
					<div class="space-y-3">
						{#each sessions as session, i}
							{@const isExpired = new Date(session.expires_at) < new Date()}
							{@const expiresIn = Math.max(0, Math.round((new Date(session.expires_at).getTime() - Date.now()) / (1000 * 60 * 60 * 24)))}
							<div class="flex items-center justify-between rounded-lg border border-white/5 bg-[var(--bg-primary)] px-4 py-3 {isExpired ? 'opacity-50' : ''}">
								<div class="flex items-center gap-3">
									<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-white/5 text-[var(--text-secondary)]">
										{#if session.device_name?.includes('Desktop')}
											<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2" /><line x1="8" y1="21" x2="16" y2="21" /><line x1="12" y1="17" x2="12" y2="21" /></svg>
										{:else if session.device_name?.includes('Android') || session.device_name?.includes('iOS')}
											<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="5" y="2" width="14" height="20" rx="2" /><line x1="12" y1="18" x2="12.01" y2="18" /></svg>
										{:else}
											<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10" /><line x1="2" y1="12" x2="22" y2="12" /><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" /></svg>
										{/if}
									</div>
									<div>
										<div class="text-sm font-medium">
											{session.device_name ?? 'Unknown device'}
										</div>
										<div class="text-xs text-[var(--text-secondary)]">
											{session.ip_address ?? 'Unknown IP'} &middot; {new Date(session.created_at).toLocaleDateString()}
											{#if isExpired}
												&middot; <span class="text-red-400">Expired</span>
											{:else}
												&middot; Expires in {expiresIn}d
											{/if}
										</div>
									</div>
								</div>
								<button
									onclick={() => handleRevokeSession(session.id)}
									class="rounded px-2 py-1 text-xs text-red-400 transition hover:bg-red-500/10"
								>
									Revoke
								</button>
							</div>
						{/each}
					</div>
				{/if}
			</section>

			<!-- Security Info -->
			<section class="mb-8 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
				<h2 class="mb-4 text-lg font-semibold">Security</h2>
				<div class="space-y-3 text-sm text-[var(--text-secondary)]">
					<div class="flex items-center justify-between">
						<span>End-to-end encryption</span>
						<span class="rounded-full bg-green-500/10 px-3 py-1 text-xs text-green-400">Active</span>
					</div>
					<div class="flex items-center justify-between">
						<span>Identity key fingerprint</span>
						<code class="text-xs font-mono text-[var(--text-secondary)]">
							Stored on device
						</code>
					</div>
				</div>
			</section>

			<!-- Danger Zone -->
			<section class="rounded-xl border border-red-500/20 bg-[var(--bg-secondary)] p-6">
				<h2 class="mb-4 text-lg font-semibold text-red-400">Danger Zone</h2>
				<div class="flex flex-wrap gap-3">
					<button
						onclick={() => { authStore.logout(); goto('/login'); }}
						class="rounded-lg bg-red-500/10 border border-red-500/20 px-4 py-2 text-sm font-medium text-red-400 transition hover:bg-red-500/20"
					>
						Sign Out
					</button>
					<button
						onclick={() => { showDeleteConfirm = true; }}
						class="rounded-lg bg-red-500/10 border border-red-500/20 px-4 py-2 text-sm font-medium text-red-400 transition hover:bg-red-500/20"
					>
						Delete Account
					</button>
					{#if isDesktop && serverUrl}
						<button
							onclick={() => { clearServerUrl(); authStore.logout(); goto('/connect'); }}
							class="rounded-lg bg-red-500/10 border border-red-500/20 px-4 py-2 text-sm font-medium text-red-400 transition hover:bg-red-500/20"
						>
							Disconnect from Server
						</button>
					{/if}
				</div>

				{#if showDeleteConfirm}
					<div class="mt-4 rounded-lg border border-red-500/20 bg-[var(--bg-primary)] p-4">
						<p class="mb-3 text-sm text-red-400">
							This action is permanent and cannot be undone. All your data will be deleted.
							If you own any groups, you must transfer ownership or delete them first.
						</p>
						{#if deleteError}
							<div class="mb-3 rounded-lg bg-red-500/10 border border-red-500/20 px-4 py-3 text-sm text-red-400">
								{deleteError}
							</div>
						{/if}
						<form onsubmit={handleDeleteAccount} class="flex gap-2">
							<input
								type="password"
								bind:value={deletePassword}
								placeholder="Confirm your password"
								required
								class="flex-1 rounded-lg border border-white/10 bg-[var(--bg-secondary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-red-500/50"
							/>
							<button
								type="submit"
								disabled={deleting}
								class="rounded-lg bg-red-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-red-700 disabled:opacity-50"
							>
								{deleting ? 'Deleting...' : 'Delete Forever'}
							</button>
							<button
								type="button"
								onclick={() => { showDeleteConfirm = false; deletePassword = ''; deleteError = ''; }}
								class="rounded-lg border border-white/10 px-4 py-2 text-sm text-[var(--text-secondary)] transition hover:bg-white/5"
							>
								Cancel
							</button>
						</form>
					</div>
				{/if}
			</section>
		</div>
	</div>
{/if}
