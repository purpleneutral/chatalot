<script lang="ts">
	import { goto } from '$app/navigation';
	import { authStore } from '$lib/stores/auth.svelte';
	import { themeStore } from '$lib/stores/theme.svelte';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { soundStore } from '$lib/stores/sound.svelte';
	import { notificationStore } from '$lib/stores/notification.svelte';
	import { preferencesStore, ACCENT_COLORS, FONT_SIZES, PRESET_THEMES, VOICE_BG_PRESETS, voiceBackgroundStyle, type AccentColor, type NoiseSuppression, type PresetTheme, type VoiceBackgroundType } from '$lib/stores/preferences.svelte';
	import { webrtcManager } from '$lib/webrtc/manager';
	import { voiceStore } from '$lib/stores/voice.svelte';
	import { audioDeviceStore } from '$lib/stores/audioDevices.svelte';
	import { setupTotp, verifyTotp, disableTotp, regenerateBackupCodes, type TotpSetup } from '$lib/api/totp';
	import { changePassword, updateProfile, uploadAvatar, uploadBanner, uploadVoiceBackground, deleteAccount, logoutAll, listSessions, revokeSession, regenerateRecoveryCode, type SessionInfo } from '$lib/api/account';
	import { isTauri, getServerUrl, clearServerUrl } from '$lib/env';
	import Avatar from '$lib/components/Avatar.svelte';
	import { onMount, onDestroy } from 'svelte';

	type Tab = 'profile' | 'appearance' | 'notifications' | 'chat' | 'voice' | 'security' | 'account';
	let activeTab = $state<Tab>('profile');

	let isDesktop = $derived(isTauri());
	let serverUrl = $derived(isDesktop ? getServerUrl() : null);

	let totpSetup = $state<TotpSetup | null>(null);
	let totpCode = $state('');
	let disableCode = $state('');
	let totpMessage = $state('');
	let totpError = $state('');
	let showTotpSetup = $state(false);
	let showTotpDisable = $state(false);
	let backupCodes = $state<string[]>([]);
	let showBackupCodes = $state(false);
	let backupCodeTotpInput = $state('');
	let backupCodeError = $state('');

	// Recovery code
	let recoveryCode = $state('');
	let showRecoveryCode = $state(false);
	let recoveryLoading = $state(false);
	let recoveryError = $state('');
	let copiedRecovery = $state(false);

	// Profile editing
	let editDisplayName = $state(authStore.user?.display_name ?? '');
	let editCustomStatus = $state(authStore.user?.custom_status ?? '');
	let editBio = $state(authStore.user?.bio ?? '');
	let editPronouns = $state(authStore.user?.pronouns ?? '');
	let profileSaving = $state(false);
	let profileMessage = $state('');
	let profileError = $state('');

	// Avatar upload
	let avatarInputEl: HTMLInputElement | undefined = $state();
	let avatarUploading = $state(false);

	// Banner upload
	let bannerInputEl: HTMLInputElement | undefined = $state();
	let bannerUploading = $state(false);

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
	let revokingSessionId = $state<string | null>(null);
	let revokingAll = $state(false);

	// Delete account
	let showDeleteConfirm = $state(false);
	let deletePassword = $state('');
	let deleteError = $state('');
	let deleting = $state(false);

	const tabs: { id: Tab; label: string; icon: string }[] = [
		{ id: 'profile', label: 'Profile', icon: 'M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2M12 3a4 4 0 1 0 0 8 4 4 0 0 0 0-8z' },
		{ id: 'appearance', label: 'Appearance', icon: 'M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 1 1-8 0 4 4 0 0 1 8 0z' },
		{ id: 'notifications', label: 'Notifications', icon: 'M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9M13.73 21a2 2 0 0 1-3.46 0' },
		{ id: 'chat', label: 'Chat', icon: 'M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z' },
		{ id: 'voice', label: 'Voice', icon: 'M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z M19 10v2a7 7 0 0 1-14 0v-2 M12 19v4 M8 23h8' },
		{ id: 'security', label: 'Security', icon: 'M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z' },
		{ id: 'account', label: 'Account', icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 0 0 2.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 0 0 1.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 0 0-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 0 0-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 0 0-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 0 0-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 0 0 1.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0z' },
	];

	// Mic test state
	let testStream: MediaStream | null = null;
	let testAudioCtx: AudioContext | null = null;
	let testAnalyser: AnalyserNode | null = null;
	let testLevel = $state(0);
	let testActive = $state(false);
	let testRafId = 0;

	// Voice background state
	let voiceBgType = $state<VoiceBackgroundType>(preferencesStore.preferences.voiceBackground.type);
	let voiceBgColor = $state(preferencesStore.preferences.voiceBackground.color ?? '#1a1a2e');
	let voiceBgGradFrom = $state(preferencesStore.preferences.voiceBackground.gradientFrom ?? '#ff6b2b');
	let voiceBgGradTo = $state(preferencesStore.preferences.voiceBackground.gradientTo ?? '#6c3483');
	let voiceBgGradAngle = $state(preferencesStore.preferences.voiceBackground.gradientAngle ?? 135);
	let voiceBgPresetId = $state(preferencesStore.preferences.voiceBackground.presetId ?? 'fireplace');
	let voiceBgCustomUrl = $state(preferencesStore.preferences.voiceBackground.customUrl ?? '');
	let voiceBgUploading = $state(false);
	let voiceBgInputEl = $state<HTMLInputElement | null>(null);

	function applyVoiceBg() {
		const bg = {
			type: voiceBgType,
			color: voiceBgColor,
			gradientFrom: voiceBgGradFrom,
			gradientTo: voiceBgGradTo,
			gradientAngle: voiceBgGradAngle,
			presetId: voiceBgPresetId,
			customUrl: voiceBgCustomUrl,
		};
		preferencesStore.set('voiceBackground', bg);
	}

	async function handleVoiceBgUpload(e: Event) {
		const file = (e.target as HTMLInputElement).files?.[0];
		if (!file) return;
		voiceBgUploading = true;
		try {
			const result = await uploadVoiceBackground(file);
			voiceBgCustomUrl = result.url;
			voiceBgType = 'custom';
			applyVoiceBg();
			toastStore.success('Background uploaded');
		} catch (err: any) {
			toastStore.error(err?.message ?? 'Upload failed');
		} finally {
			voiceBgUploading = false;
		}
	}

	async function startMicTest() {
		try {
			const constraints: MediaTrackConstraints = {
				noiseSuppression: false,
				echoCancellation: preferencesStore.preferences.echoCancellation,
				autoGainControl: preferencesStore.preferences.autoGainControl,
			};
			const inputId = audioDeviceStore.selectedInputId;
			if (inputId) constraints.deviceId = { exact: inputId };

			testStream = await navigator.mediaDevices.getUserMedia({ audio: constraints });
			// Re-enumerate now that we have permission (labels become available)
			audioDeviceStore.enumerateDevices();

			testAudioCtx = new AudioContext({ sampleRate: 48000 });
			const source = testAudioCtx.createMediaStreamSource(testStream);
			testAnalyser = testAudioCtx.createAnalyser();
			testAnalyser.fftSize = 256;
			testAnalyser.smoothingTimeConstant = 0.5;
			source.connect(testAnalyser);
			testActive = true;
			pollLevel();
		} catch (err) {
			console.error('Mic test failed:', err);
			toastStore.error('Could not access microphone');
		}
	}

	function pollLevel() {
		if (!testActive || !testAnalyser) return;
		const buffer = new Uint8Array(testAnalyser.frequencyBinCount);
		testAnalyser.getByteTimeDomainData(buffer);
		let sum = 0;
		for (let i = 0; i < buffer.length; i++) {
			const val = buffer[i] - 128;
			sum += val * val;
		}
		const rms = Math.sqrt(sum / buffer.length);
		testLevel = Math.min(100, Math.round(rms * 3));
		testRafId = requestAnimationFrame(pollLevel);
	}

	function stopMicTest() {
		testActive = false;
		cancelAnimationFrame(testRafId);
		testStream?.getTracks().forEach(t => t.stop());
		testStream = null;
		testAudioCtx?.close();
		testAudioCtx = null;
		testAnalyser = null;
		testLevel = 0;
	}

	// Clean up mic test when leaving voice tab or unmounting
	$effect(() => {
		if (activeTab !== 'voice') stopMicTest();
		return () => stopMicTest();
	});

	// Enumerate devices on mount
	onMount(() => { audioDeviceStore.enumerateDevices(); });
	onDestroy(() => { stopMicTest(); });

	const nsLevels: { id: NoiseSuppression; label: string; desc: string; cpu: string }[] = [
		{ id: 'off', label: 'Off', desc: 'No noise processing', cpu: '' },
		{ id: 'noise-gate', label: 'Noise Gate', desc: 'Silences audio below a volume threshold', cpu: 'Minimal CPU' },
		{ id: 'standard', label: 'Standard', desc: 'DSP-based noise reduction (Speex)', cpu: 'Low CPU' },
		{ id: 'maximum', label: 'Maximum', desc: 'ML-powered noise removal (RNNoise)', cpu: 'Moderate CPU' }
	];

	const accentColorList: { id: AccentColor; label: string }[] = [
		{ id: 'blue', label: 'Blue' },
		{ id: 'purple', label: 'Purple' },
		{ id: 'green', label: 'Green' },
		{ id: 'orange', label: 'Orange' },
		{ id: 'red', label: 'Red' },
		{ id: 'pink', label: 'Pink' },
		{ id: 'teal', label: 'Teal' },
		{ id: 'cyan', label: 'Cyan' },
	];

	const presetThemeList = (Object.keys(PRESET_THEMES) as PresetTheme[]).filter(k => k !== 'custom');

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

	async function handleBannerUpload(e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;

		bannerUploading = true;
		profileError = '';
		try {
			const updated = await uploadBanner(file);
			authStore.updateUser(updated);
			profileMessage = 'Banner updated.';
		} catch (err) {
			profileError = err instanceof Error ? err.message : 'Failed to upload banner';
		} finally {
			bannerUploading = false;
			if (bannerInputEl) bannerInputEl.value = '';
		}
	}

	async function handleRemoveBanner() {
		profileSaving = true;
		profileError = '';
		try {
			const updated = await updateProfile({ banner_url: null });
			authStore.updateUser(updated);
			profileMessage = 'Banner removed.';
		} catch (err) {
			profileError = err instanceof Error ? err.message : 'Failed to remove banner';
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
				custom_status: editCustomStatus || undefined,
				bio: editBio || null,
				pronouns: editPronouns || null
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
		if (revokingSessionId) return;
		revokingSessionId = id;
		try {
			await revokeSession(id);
			sessions = sessions.filter(s => s.id !== id);
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to revoke session');
		} finally {
			revokingSessionId = null;
		}
	}

	async function handleLogoutAll() {
		if (revokingAll) return;
		if (!confirm('Revoke all sessions? You will be logged out everywhere.')) return;
		revokingAll = true;
		try {
			await logoutAll();
			authStore.logout();
			goto('/login');
		} catch (err) {
			toastStore.error(err instanceof Error ? err.message : 'Failed to logout');
		} finally {
			revokingAll = false;
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
			const result = await verifyTotp(totpCode);
			totpMessage = '2FA enabled successfully!';
			showTotpSetup = false;
			totpSetup = null;
			totpCode = '';
			if (result.backup_codes?.length) {
				backupCodes = result.backup_codes;
				showBackupCodes = true;
			}
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
			backupCodes = [];
			showBackupCodes = false;
		} catch (err) {
			totpError = err instanceof Error ? err.message : 'Invalid code';
		}
	}

	async function handleRegenerateRecoveryCode() {
		recoveryLoading = true;
		recoveryError = '';
		try {
			const result = await regenerateRecoveryCode();
			recoveryCode = result.recovery_code;
			showRecoveryCode = true;
		} catch (err) {
			recoveryError = err instanceof Error ? err.message : 'Failed to regenerate recovery code';
		} finally {
			recoveryLoading = false;
		}
	}

	async function handleRegenerateBackupCodes(e: SubmitEvent) {
		e.preventDefault();
		backupCodeError = '';
		try {
			const result = await regenerateBackupCodes(backupCodeTotpInput);
			backupCodes = result.backup_codes;
			showBackupCodes = true;
			backupCodeTotpInput = '';
		} catch (err) {
			backupCodeError = err instanceof Error ? err.message : 'Invalid TOTP code';
		}
	}
</script>

{#if authStore.isAuthenticated}
	<div class="flex min-h-screen bg-[var(--bg-primary)] text-[var(--text-primary)]">
		<!-- Sidebar navigation (hidden on mobile) -->
		<nav class="hidden md:flex w-56 shrink-0 flex-col border-r border-white/10 bg-[var(--bg-secondary)] p-4">
			<div class="mb-6 flex items-center justify-between">
				<h1 class="text-lg font-bold">Settings</h1>
				<button
					onclick={() => goto('/channels')}
					class="rounded-lg p-1.5 text-[var(--text-secondary)] transition hover:bg-white/5 hover:text-[var(--text-primary)]"
					title="Back to chat"
					aria-label="Back to chat"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
					</svg>
				</button>
			</div>

			<div class="flex flex-col gap-1">
				{#each tabs as tab}
					<button
						onclick={() => (activeTab = tab.id)}
						class="flex items-center gap-3 rounded-lg px-3 py-2 text-left text-sm transition
							{activeTab === tab.id
								? 'bg-[var(--accent)]/15 text-[var(--accent)] font-medium'
								: 'text-[var(--text-secondary)] hover:bg-white/5 hover:text-[var(--text-primary)]'}"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<path d={tab.icon} />
						</svg>
						{tab.label}
					</button>
				{/each}
			</div>

			<div class="mt-auto pt-4">
				{#if authStore.user?.is_admin || authStore.user?.is_owner}
					<button
						onclick={() => goto('/admin')}
						class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-sm text-[var(--accent)] transition hover:bg-white/5"
					>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
						</svg>
						Admin Panel
					</button>
				{/if}
			</div>
		</nav>

		<!-- Content area -->
		<div class="flex-1 overflow-y-auto">
			<!-- Mobile header + tab bar -->
			<div class="sticky top-0 z-10 flex items-center gap-2 border-b border-white/10 bg-[var(--bg-secondary)] px-4 py-2 md:hidden">
				<button
					onclick={() => goto('/channels')}
					class="shrink-0 rounded-lg p-1.5 text-[var(--text-secondary)] transition hover:bg-white/5"
					title="Back to chat"
					aria-label="Back to chat"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<polyline points="15 18 9 12 15 6" />
					</svg>
				</button>
				<div class="flex flex-1 gap-1 overflow-x-auto">
					{#each tabs as tab}
						<button
							onclick={() => (activeTab = tab.id)}
							class="shrink-0 rounded-lg px-2.5 py-1.5 text-xs font-medium transition
								{activeTab === tab.id
									? 'bg-[var(--accent)]/15 text-[var(--accent)]'
									: 'text-[var(--text-secondary)] hover:bg-white/5'}"
						>
							{tab.label}
						</button>
					{/each}
				</div>
			</div>
			<div class="mx-auto max-w-2xl px-4 py-4 sm:px-6 sm:py-6 md:px-8 md:py-8">

				<!-- ══════════════════ PROFILE TAB ══════════════════ -->
				{#if activeTab === 'profile'}
					<h2 class="mb-6 text-xl font-bold">Profile</h2>

					{#if profileMessage}
						<div class="mb-4 rounded-lg border border-green-500/20 bg-green-500/10 px-4 py-3 text-sm text-green-400">
							{profileMessage}
						</div>
					{/if}
					{#if profileError}
						<div class="mb-4 rounded-lg border border-red-500/20 bg-red-500/10 px-4 py-3 text-sm text-red-400">
							{profileError}
						</div>
					{/if}

					<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Avatar</h3>
						<div class="flex items-center gap-4">
							<div class="group relative">
								{#if authStore.user}
									<Avatar userId={authStore.user.id} size="lg" />
								{/if}
								<button
									onclick={() => avatarInputEl?.click()}
									disabled={avatarUploading}
									class="absolute inset-0 flex items-center justify-center rounded-full bg-black/50 opacity-0 transition group-hover:opacity-100 disabled:cursor-wait"
									title="Change avatar"
									aria-label="Change avatar"
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
					</section>

					<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Profile Banner</h3>
						<div class="group relative aspect-[3/1] w-full overflow-hidden rounded-lg border border-white/10">
							{#if authStore.user?.banner_url}
								<img
									src={authStore.user.banner_url}
									alt="Profile banner"
									class="h-full w-full object-cover"
								/>
							{:else}
								<div class="flex h-full w-full items-center justify-center bg-gradient-to-r from-[var(--accent)] to-[var(--accent-hover)]">
									<span class="text-sm text-white/50">No banner set</span>
								</div>
							{/if}
							<button
								onclick={() => bannerInputEl?.click()}
								disabled={bannerUploading}
								class="absolute inset-0 flex items-center justify-center bg-black/50 opacity-0 transition group-hover:opacity-100 disabled:cursor-wait"
								title="Change banner"
								aria-label="Change banner"
							>
								<svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z" /><circle cx="12" cy="13" r="4" />
								</svg>
							</button>
							<input
								bind:this={bannerInputEl}
								type="file"
								accept="image/png,image/jpeg,image/webp,image/gif"
								onchange={handleBannerUpload}
								class="hidden"
							/>
						</div>
						<div class="mt-2 flex items-center justify-between">
							<span class="text-xs text-[var(--text-secondary)]">Recommended: 1200x400, max 5 MB</span>
							{#if authStore.user?.banner_url}
								<button
									onclick={handleRemoveBanner}
									class="text-xs text-[var(--danger)] hover:underline"
								>
									Remove banner
								</button>
							{/if}
						</div>
					</section>

					<section class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Personal Info</h3>
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
								<p class="mt-1 text-right text-[10px] text-[var(--text-secondary)]/50">{editDisplayName.length}/64</p>
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
								<p class="mt-1 text-right text-[10px] text-[var(--text-secondary)]/50">{editCustomStatus.length}/128</p>
							</div>
							<div>
								<label for="pronouns" class="mb-1 block text-sm font-medium">Pronouns</label>
								<input
									id="pronouns"
									type="text"
									bind:value={editPronouns}
									maxlength="50"
									placeholder="e.g. they/them"
									class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
								/>
								<p class="mt-1 text-right text-[10px] text-[var(--text-secondary)]/50">{editPronouns.length}/50</p>
							</div>
							<div>
								<label for="bio" class="mb-1 block text-sm font-medium">Bio</label>
								<textarea
									id="bio"
									bind:value={editBio}
									maxlength="500"
									rows="3"
									placeholder="Tell others about yourself..."
									class="w-full resize-none rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
								></textarea>
								<p class="mt-1 text-right text-[10px] text-[var(--text-secondary)]/50">{editBio.length}/500</p>
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

				<!-- ══════════════════ APPEARANCE TAB ══════════════════ -->
				{:else if activeTab === 'appearance'}
					<h2 class="mb-6 text-xl font-bold">Appearance</h2>

					<!-- Theme -->
					<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Theme</h3>
						<div class="flex items-center justify-between">
							<div>
								<div class="font-medium">Color scheme</div>
								<div class="text-sm text-[var(--text-secondary)]">
									{themeStore.current === 'system' ? `System (${themeStore.resolved === 'dark' ? 'Dark' : 'Light'})` : themeStore.resolved === 'dark' ? 'Dark mode' : 'Light mode'}
								</div>
							</div>
							<div class="flex overflow-hidden rounded-lg border border-white/10">
								<button
									onclick={() => themeStore.set('dark')}
									class="px-3 py-1.5 text-sm transition {themeStore.current === 'dark' ? 'bg-[var(--accent)] text-white' : 'text-[var(--text-secondary)] hover:bg-white/5'}"
									title="Dark"
									aria-label="Dark theme"
								>
									<svg xmlns="http://www.w3.org/2000/svg" class="inline h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
										<path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
									</svg>
								</button>
								<button
									onclick={() => themeStore.set('light')}
									class="border-x border-white/10 px-3 py-1.5 text-sm transition {themeStore.current === 'light' ? 'bg-[var(--accent)] text-white' : 'text-[var(--text-secondary)] hover:bg-white/5'}"
									title="Light"
									aria-label="Light theme"
								>
									<svg xmlns="http://www.w3.org/2000/svg" class="inline h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
										<circle cx="12" cy="12" r="5" /><line x1="12" y1="1" x2="12" y2="3" /><line x1="12" y1="21" x2="12" y2="23" /><line x1="4.22" y1="4.22" x2="5.64" y2="5.64" /><line x1="18.36" y1="18.36" x2="19.78" y2="19.78" /><line x1="1" y1="12" x2="3" y2="12" /><line x1="21" y1="12" x2="23" y2="12" /><line x1="4.22" y1="19.78" x2="5.64" y2="18.36" /><line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
									</svg>
								</button>
								<button
									onclick={() => themeStore.set('system')}
									class="px-3 py-1.5 text-sm transition {themeStore.current === 'system' ? 'bg-[var(--accent)] text-white' : 'text-[var(--text-secondary)] hover:bg-white/5'}"
									title="System"
									aria-label="System theme"
								>
									<svg xmlns="http://www.w3.org/2000/svg" class="inline h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
										<rect x="2" y="3" width="20" height="14" rx="2" ry="2" /><line x1="8" y1="21" x2="16" y2="21" /><line x1="12" y1="17" x2="12" y2="21" />
									</svg>
								</button>
							</div>
						</div>
					</section>

					<!-- Preset Themes -->
					<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Color Palette</h3>
						<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2">
							{#each presetThemeList as themeId}
								{@const t = PRESET_THEMES[themeId]}
								<button
									onclick={() => preferencesStore.set('presetTheme', themeId)}
									class="group flex flex-col items-center gap-1.5 rounded-lg border p-2 transition
										{preferencesStore.preferences.presetTheme === themeId
											? 'border-[var(--accent)] bg-[var(--accent)]/10'
											: 'border-white/10 hover:border-white/20'}"
								>
									<div class="flex h-8 w-full overflow-hidden rounded">
										<div class="flex-1" style="background: {t.colors.dark.bgPrimary}"></div>
										<div class="flex-1" style="background: {t.colors.dark.bgTertiary}"></div>
										<div class="flex-1" style="background: {t.colors.dark.accent}"></div>
									</div>
									<span class="text-xs text-[var(--text-secondary)]">{t.label}</span>
								</button>
							{/each}
							<!-- Custom theme button -->
							<button
								onclick={() => preferencesStore.set('presetTheme', 'custom')}
								class="group flex flex-col items-center gap-1.5 rounded-lg border p-2 transition
									{preferencesStore.preferences.presetTheme === 'custom'
										? 'border-[var(--accent)] bg-[var(--accent)]/10'
										: 'border-white/10 hover:border-white/20'}"
							>
								<div class="flex h-8 w-full items-center justify-center rounded bg-white/5">
									<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-[var(--text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
										<circle cx="13.5" cy="6.5" r="2.5" /><circle cx="19" cy="11.5" r="2.5" /><circle cx="6" cy="12.5" r="2.5" /><circle cx="17" cy="18.5" r="2.5" /><circle cx="8.5" cy="18.5" r="2.5" />
									</svg>
								</div>
								<span class="text-xs text-[var(--text-secondary)]">Custom</span>
							</button>
						</div>

						<!-- Custom theme color pickers -->
						{#if preferencesStore.preferences.presetTheme === 'custom'}
							<div class="mt-4 grid grid-cols-2 gap-3 border-t border-white/10 pt-4">
								{#each [
									{ key: 'bgPrimary', label: 'Background' },
									{ key: 'bgSecondary', label: 'Surface' },
									{ key: 'bgTertiary', label: 'Elevated' },
									{ key: 'textPrimary', label: 'Text' },
									{ key: 'textSecondary', label: 'Muted text' },
									{ key: 'accent', label: 'Accent' },
									{ key: 'accentHover', label: 'Accent hover' }
								] as field}
									<label class="flex items-center gap-2">
										<input
											type="color"
											value={preferencesStore.preferences.customThemeColors[field.key as keyof typeof preferencesStore.preferences.customThemeColors]}
											oninput={(e) => {
												const updated = { ...preferencesStore.preferences.customThemeColors, [field.key]: (e.target as HTMLInputElement).value };
												preferencesStore.set('customThemeColors', updated);
											}}
											class="h-8 w-8 cursor-pointer rounded border border-white/10 bg-transparent"
										/>
										<span class="text-sm text-[var(--text-secondary)]">{field.label}</span>
									</label>
								{/each}
							</div>
						{/if}
					</section>

					<!-- Accent Color (only shown for default theme) -->
					{#if preferencesStore.preferences.presetTheme === 'default'}
						<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
							<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Accent Color</h3>
							<div class="flex flex-wrap gap-3">
								{#each accentColorList as color}
									<button
										onclick={() => preferencesStore.set('accentColor', color.id)}
										class="group flex flex-col items-center gap-1.5"
										title={color.label}
									>
										<div
											class="flex h-10 w-10 items-center justify-center rounded-full transition-transform hover:scale-110
												{preferencesStore.preferences.accentColor === color.id ? 'ring-2 ring-white ring-offset-2 ring-offset-[var(--bg-secondary)]' : ''}"
											style="background-color: {ACCENT_COLORS[color.id].main};"
										>
											{#if preferencesStore.preferences.accentColor === color.id}
												<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
													<polyline points="20 6 9 17 4 12" />
												</svg>
											{/if}
										</div>
										<span class="text-xs text-[var(--text-secondary)]">{color.label}</span>
									</button>
								{/each}
							</div>
						</section>
					{/if}

					<!-- Message Density -->
					<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Message Display</h3>

						<div class="mb-4">
							<div class="mb-2 font-medium">Density</div>
							<div class="flex gap-3">
								<button
									onclick={() => preferencesStore.set('messageDensity', 'cozy')}
									class="flex-1 rounded-lg border p-3 text-left transition
										{preferencesStore.preferences.messageDensity === 'cozy'
											? 'border-[var(--accent)] bg-[var(--accent)]/10'
											: 'border-white/10 hover:border-white/20'}"
								>
									<div class="mb-1 text-sm font-medium">Cozy</div>
									<div class="text-xs text-[var(--text-secondary)]">Avatars and spacious layout</div>
								</button>
								<button
									onclick={() => preferencesStore.set('messageDensity', 'compact')}
									class="flex-1 rounded-lg border p-3 text-left transition
										{preferencesStore.preferences.messageDensity === 'compact'
											? 'border-[var(--accent)] bg-[var(--accent)]/10'
											: 'border-white/10 hover:border-white/20'}"
								>
									<div class="mb-1 text-sm font-medium">Compact</div>
									<div class="text-xs text-[var(--text-secondary)]">No avatars, tighter spacing</div>
								</button>
							</div>
						</div>

						<!-- Chat Bubble Style -->
						<div class="mb-4">
							<div class="mb-2 font-medium">Bubble style</div>
							<div class="flex gap-3">
								<button
									onclick={() => preferencesStore.set('chatBubbleStyle', 'flat')}
									class="flex-1 rounded-lg border p-3 text-left transition
										{preferencesStore.preferences.chatBubbleStyle === 'flat'
											? 'border-[var(--accent)] bg-[var(--accent)]/10'
											: 'border-white/10 hover:border-white/20'}"
								>
									<div class="mb-1 text-sm font-medium">Flat</div>
									<div class="text-xs text-[var(--text-secondary)]">Discord-style with color border</div>
								</button>
								<button
									onclick={() => preferencesStore.set('chatBubbleStyle', 'bubbles')}
									class="flex-1 rounded-lg border p-3 text-left transition
										{preferencesStore.preferences.chatBubbleStyle === 'bubbles'
											? 'border-[var(--accent)] bg-[var(--accent)]/10'
											: 'border-white/10 hover:border-white/20'}"
								>
									<div class="mb-1 text-sm font-medium">Bubbles</div>
									<div class="text-xs text-[var(--text-secondary)]">iMessage-style rounded bubbles</div>
								</button>
							</div>
						</div>

						<!-- Time Format -->
						<div class="mb-4 flex items-center justify-between">
							<div>
								<div class="font-medium">Time format</div>
								<div class="text-sm text-[var(--text-secondary)]">How timestamps are displayed</div>
							</div>
							<div class="flex overflow-hidden rounded-lg border border-white/10">
								<button
									onclick={() => preferencesStore.set('timeFormat', '12h')}
									class="px-3 py-1.5 text-sm transition {preferencesStore.preferences.timeFormat === '12h' ? 'bg-[var(--accent)] text-white' : 'text-[var(--text-secondary)] hover:bg-white/5'}"
								>
									12h
								</button>
								<button
									onclick={() => preferencesStore.set('timeFormat', '24h')}
									class="border-l border-white/10 px-3 py-1.5 text-sm transition {preferencesStore.preferences.timeFormat === '24h' ? 'bg-[var(--accent)] text-white' : 'text-[var(--text-secondary)] hover:bg-white/5'}"
								>
									24h
								</button>
							</div>
						</div>

						<!-- Relative Timestamps -->
						<div class="mb-4 flex items-center justify-between">
							<div>
								<div class="font-medium">Relative timestamps</div>
								<div class="text-sm text-[var(--text-secondary)]">Show "5m ago" instead of exact time</div>
							</div>
							<button
								onclick={() => preferencesStore.set('relativeTimestamps', !preferencesStore.preferences.relativeTimestamps)}
								class="relative h-8 w-14 rounded-full bg-[var(--bg-tertiary)] transition"
								aria-label="Toggle relative timestamps"
							>
								<span class="absolute left-1 top-1 h-6 w-6 rounded-full transition-transform {preferencesStore.preferences.relativeTimestamps ? 'translate-x-6 bg-[var(--accent)]' : 'bg-[var(--text-secondary)]'}"></span>
							</button>
						</div>

						<!-- Font Size -->
						<div class="flex items-center justify-between">
							<div>
								<div class="font-medium">Font size</div>
								<div class="text-sm text-[var(--text-secondary)]">{FONT_SIZES[preferencesStore.preferences.fontSize]}</div>
							</div>
							<div class="flex overflow-hidden rounded-lg border border-white/10">
								{#each (['small', 'medium', 'large'] as const) as size}
									<button
										onclick={() => preferencesStore.set('fontSize', size)}
										class="px-3 py-1.5 text-sm transition {size !== 'small' ? 'border-l border-white/10' : ''}
											{preferencesStore.preferences.fontSize === size ? 'bg-[var(--accent)] text-white' : 'text-[var(--text-secondary)] hover:bg-white/5'}"
									>
										{size.charAt(0).toUpperCase() + size.slice(1)}
									</button>
								{/each}
							</div>
						</div>
					</section>

					<!-- Accessibility -->
					<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Accessibility</h3>

						<!-- Reduce Motion -->
						<div class="mb-4 flex items-center justify-between">
							<div>
								<div class="font-medium">Reduce motion</div>
								<div class="text-sm text-[var(--text-secondary)]">Disable animations and transitions</div>
							</div>
							<button
								onclick={() => preferencesStore.set('reduceMotion', !preferencesStore.preferences.reduceMotion)}
								class="relative h-8 w-14 rounded-full bg-[var(--bg-tertiary)] transition"
								aria-label="Toggle reduce motion"
							>
								<span class="absolute left-1 top-1 h-6 w-6 rounded-full transition-transform {preferencesStore.preferences.reduceMotion ? 'translate-x-6 bg-[var(--accent)]' : 'bg-[var(--text-secondary)]'}"></span>
							</button>
						</div>

						<!-- Animated Accent -->
						<div class="flex items-center justify-between">
							<div>
								<div class="font-medium">Animated accent</div>
								<div class="text-sm text-[var(--text-secondary)]">Subtle color shift on accent elements</div>
							</div>
							<button
								onclick={() => preferencesStore.set('animatedAccent', !preferencesStore.preferences.animatedAccent)}
								class="relative h-8 w-14 rounded-full bg-[var(--bg-tertiary)] transition"
								aria-label="Toggle animated accent"
							>
								<span class="absolute left-1 top-1 h-6 w-6 rounded-full transition-transform {preferencesStore.preferences.animatedAccent ? 'translate-x-6 bg-[var(--accent)]' : 'bg-[var(--text-secondary)]'}"></span>
							</button>
						</div>
					</section>

				<!-- ══════════════════ NOTIFICATIONS TAB ══════════════════ -->
				{:else if activeTab === 'notifications'}
					<h2 class="mb-6 text-xl font-bold">Notifications</h2>

					<!-- Sounds -->
					<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Sounds</h3>
						<div class="space-y-4">
							<div class="flex items-center justify-between">
								<div>
									<div class="font-medium">DM message sound</div>
									<div class="text-sm text-[var(--text-secondary)]">Play a sound for direct messages</div>
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
										<span class="absolute left-1 top-1 h-6 w-6 rounded-full transition-transform {soundStore.preferences.dmMessage ? 'translate-x-6 bg-[var(--accent)]' : 'bg-[var(--text-secondary)]'}"></span>
									</button>
								</div>
							</div>

							<div class="flex items-center justify-between">
								<div>
									<div class="font-medium">Channel message sound</div>
									<div class="text-sm text-[var(--text-secondary)]">Play a sound for channel messages</div>
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
										<span class="absolute left-1 top-1 h-6 w-6 rounded-full transition-transform {soundStore.preferences.channelMessage ? 'translate-x-6 bg-[var(--accent)]' : 'bg-[var(--text-secondary)]'}"></span>
									</button>
								</div>
							</div>

							<div class="flex items-center justify-between">
								<div>
									<div class="font-medium">Mention sound</div>
									<div class="text-sm text-[var(--text-secondary)]">Distinct sound when @mentioned</div>
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
										<span class="absolute left-1 top-1 h-6 w-6 rounded-full transition-transform {soundStore.preferences.mentionMessage ? 'translate-x-6 bg-[var(--accent)]' : 'bg-[var(--text-secondary)]'}"></span>
									</button>
								</div>
							</div>

							<div class="flex items-center justify-between">
								<div>
									<div class="font-medium">Voice join/leave sounds</div>
									<div class="text-sm text-[var(--text-secondary)]">When someone joins or leaves voice</div>
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
										<span class="absolute left-1 top-1 h-6 w-6 rounded-full transition-transform {soundStore.preferences.voiceJoin ? 'translate-x-6 bg-[var(--accent)]' : 'bg-[var(--text-secondary)]'}"></span>
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
						</div>
					</section>

					<!-- Desktop Notifications -->
					<section class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Desktop Notifications</h3>
						<div class="space-y-4">
							<div class="flex items-center justify-between">
								<div>
									<div class="font-medium">Desktop notifications</div>
									<div class="text-sm text-[var(--text-secondary)]">Show OS-level notifications</div>
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
											<span class="absolute left-1 top-1 h-6 w-6 rounded-full transition-transform {notificationStore.preferences.desktopEnabled ? 'translate-x-6 bg-[var(--accent)]' : 'bg-[var(--text-secondary)]'}"></span>
										</button>
									{/if}
								</div>
							</div>

							<div class="flex items-center justify-between">
								<div>
									<div class="font-medium">Default channel notifications</div>
									<div class="text-sm text-[var(--text-secondary)]">Notification level for channels</div>
								</div>
								<select
									value={notificationStore.preferences.defaultChannelLevel}
									onchange={(e) => {
										notificationStore.preferences = { ...notificationStore.preferences, defaultChannelLevel: e.currentTarget.value as 'all' | 'mentions' | 'nothing' };
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

				<!-- ══════════════════ CHAT TAB ══════════════════ -->
				{:else if activeTab === 'chat'}
					<h2 class="mb-6 text-xl font-bold">Chat</h2>

					<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Input</h3>

						<!-- Send behavior -->
						<div class="mb-4 flex items-center justify-between">
							<div>
								<div class="font-medium">Send messages with</div>
								<div class="text-sm text-[var(--text-secondary)]">Choose how to send a message</div>
							</div>
							<div class="flex overflow-hidden rounded-lg border border-white/10">
								<button
									onclick={() => preferencesStore.set('sendBehavior', 'enter')}
									class="px-3 py-1.5 text-sm transition {preferencesStore.preferences.sendBehavior === 'enter' ? 'bg-[var(--accent)] text-white' : 'text-[var(--text-secondary)] hover:bg-white/5'}"
								>
									Enter
								</button>
								<button
									onclick={() => preferencesStore.set('sendBehavior', 'ctrl-enter')}
									class="border-l border-white/10 px-3 py-1.5 text-sm transition {preferencesStore.preferences.sendBehavior === 'ctrl-enter' ? 'bg-[var(--accent)] text-white' : 'text-[var(--text-secondary)] hover:bg-white/5'}"
								>
									Ctrl+Enter
								</button>
							</div>
						</div>

						<!-- Formatting toolbar -->
						<div class="flex items-center justify-between">
							<div>
								<div class="font-medium">Formatting toolbar</div>
								<div class="text-sm text-[var(--text-secondary)]">Show Bold, Italic, etc. below input</div>
							</div>
							<button
								onclick={() => preferencesStore.set('showFormattingToolbar', !preferencesStore.preferences.showFormattingToolbar)}
								class="relative h-8 w-14 rounded-full bg-[var(--bg-tertiary)] transition"
								aria-label="Toggle formatting toolbar"
							>
								<span class="absolute left-1 top-1 h-6 w-6 rounded-full transition-transform {preferencesStore.preferences.showFormattingToolbar ? 'translate-x-6 bg-[var(--accent)]' : 'bg-[var(--text-secondary)]'}"></span>
							</button>
						</div>
					</section>

					<section class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Display</h3>

						<!-- Link Previews -->
						<div class="flex items-center justify-between">
							<div>
								<div class="font-medium">Link previews</div>
								<div class="text-sm text-[var(--text-secondary)]">Show website previews for links</div>
							</div>
							<button
								onclick={() => preferencesStore.set('showLinkPreviews', !preferencesStore.preferences.showLinkPreviews)}
								class="relative h-8 w-14 rounded-full bg-[var(--bg-tertiary)] transition"
								aria-label="Toggle link previews"
							>
								<span class="absolute left-1 top-1 h-6 w-6 rounded-full transition-transform {preferencesStore.preferences.showLinkPreviews ? 'translate-x-6 bg-[var(--accent)]' : 'bg-[var(--text-secondary)]'}"></span>
							</button>
						</div>
					</section>

				<!-- ══════════════════ VOICE TAB ══════════════════ -->
				{:else if activeTab === 'voice'}
					<h2 class="mb-6 text-xl font-bold">Voice & Audio</h2>

					<!-- ── Input Device ── -->
					<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Input Device</h3>

						<div class="mb-4">
							<label for="mic-select" class="mb-1.5 block text-sm font-medium text-[var(--text-primary)]">Microphone</label>
							<select id="mic-select"
								value={audioDeviceStore.selectedInputId}
								onchange={(e) => {
									const id = e.currentTarget.value;
									audioDeviceStore.setInputDevice(id);
									if (voiceStore.isInCall) webrtcManager.switchInputDevice(id);
									if (testActive) { stopMicTest(); startMicTest(); }
								}}
								class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)]"
							>
								<option value="">System Default</option>
								{#each audioDeviceStore.inputDevices as device}
									<option value={device.deviceId}>{device.label}</option>
								{/each}
							</select>
						</div>

						<div class="mb-4">
							<div class="mb-2 flex items-center justify-between">
								<span class="text-sm font-medium text-[var(--text-primary)]">Mic Test</span>
								<button
									onclick={() => testActive ? stopMicTest() : startMicTest()}
									class="rounded-lg border border-white/10 px-3 py-1.5 text-xs font-medium text-[var(--text-primary)] transition hover:bg-white/5 {testActive ? 'border-red-500/50 text-red-400' : ''}"
								>
									{testActive ? 'Stop Test' : 'Test Microphone'}
								</button>
							</div>
							<div class="h-2.5 rounded-full bg-white/10 overflow-hidden">
								<div
									class="h-full rounded-full transition-all duration-75 {testLevel > 70 ? 'bg-yellow-400' : 'bg-green-400'}"
									style="width: {testActive ? testLevel : 0}%"
								></div>
							</div>
						</div>

						<div>
							<div class="mb-2 flex items-center justify-between">
								<span class="text-sm font-medium text-[var(--text-primary)]">Input Volume</span>
								<span class="text-sm text-[var(--text-secondary)]">{preferencesStore.preferences.inputGain}%</span>
							</div>
							<input
								type="range"
								min="0"
								max="200"
								value={preferencesStore.preferences.inputGain}
								oninput={(e) => webrtcManager.setMicGain(parseInt(e.currentTarget.value))}
								class="h-1.5 w-full cursor-pointer appearance-none rounded-full bg-white/10 accent-[var(--accent)]"
							/>
							<div class="mt-1 flex justify-between text-[10px] text-[var(--text-secondary)]">
								<span>0%</span>
								<span>100%</span>
								<span>200%</span>
							</div>
						</div>
					</section>

					<!-- ── Output Device ── -->
					<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Output Device</h3>

						<div class="mb-4">
							<label for="speaker-select" class="mb-1.5 block text-sm font-medium text-[var(--text-primary)]">Speaker</label>
							{#if audioDeviceStore.supportsOutputSelection}
								<select id="speaker-select"
									value={audioDeviceStore.selectedOutputId}
									onchange={(e) => audioDeviceStore.setOutputDevice(e.currentTarget.value)}
									class="w-full rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none transition focus:border-[var(--accent)]"
								>
									<option value="">System Default</option>
									{#each audioDeviceStore.outputDevices as device}
										<option value={device.deviceId}>{device.label}</option>
									{/each}
								</select>
							{:else}
								<p class="text-sm text-[var(--text-secondary)]">
									Your browser does not support output device selection. Use your system audio settings instead.
								</p>
							{/if}
						</div>

						<div>
							<div class="mb-2 flex items-center justify-between">
								<span class="text-sm font-medium text-[var(--text-primary)]">Output Volume</span>
								<span class="text-sm text-[var(--text-secondary)]">{preferencesStore.preferences.outputVolume}%</span>
							</div>
							<input
								type="range"
								min="0"
								max="200"
								value={preferencesStore.preferences.outputVolume}
								oninput={(e) => preferencesStore.set('outputVolume', parseInt(e.currentTarget.value))}
								class="h-1.5 w-full cursor-pointer appearance-none rounded-full bg-white/10 accent-[var(--accent)]"
							/>
							<div class="mt-1 flex justify-between text-[10px] text-[var(--text-secondary)]">
								<span>0%</span>
								<span>100%</span>
								<span>200%</span>
							</div>
						</div>
					</section>

					<!-- ── Noise Suppression ── -->
					<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Noise Suppression</h3>
						<p class="mb-4 text-sm text-[var(--text-secondary)]">
							Reduce background noise during voice calls. Higher levels use more CPU but produce cleaner audio. All processing happens on your device.
						</p>

						<div class="grid grid-cols-2 gap-3">
							{#each nsLevels as level}
								<button
									onclick={async () => {
										preferencesStore.set('noiseSuppression', level.id);
										if (voiceStore.isInCall) {
											await webrtcManager.setNoiseSuppressionLevel(level.id);
										}
									}}
									class="rounded-lg border p-4 text-left transition
										{preferencesStore.preferences.noiseSuppression === level.id
											? 'border-[var(--accent)] bg-[var(--accent)]/10'
											: 'border-white/10 hover:border-white/20'}"
								>
									<div class="mb-1 text-sm font-medium">{level.label}</div>
									<div class="text-xs text-[var(--text-secondary)]">{level.desc}</div>
									{#if level.cpu}
										<div class="mt-1.5 inline-block rounded-full bg-white/5 px-2 py-0.5 text-[10px] text-[var(--text-secondary)]">{level.cpu}</div>
									{/if}
								</button>
							{/each}
						</div>

						{#if voiceStore.isInCall}
							<div class="mt-3 flex items-center gap-2 text-xs text-[var(--accent)]">
								<span class="h-1.5 w-1.5 rounded-full bg-[var(--accent)] animate-pulse"></span>
								Changes apply immediately to your active call
							</div>
						{/if}
					</section>

					<!-- ── Advanced ── -->
					<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Advanced</h3>

						<div class="mb-4 flex items-center justify-between">
							<div>
								<div class="text-sm font-medium text-[var(--text-primary)]">Echo Cancellation</div>
								<div class="text-xs text-[var(--text-secondary)]">Removes echo from speakers feeding back into your mic</div>
							</div>
							<button aria-label="Toggle echo cancellation"
								onclick={() => preferencesStore.set('echoCancellation', !preferencesStore.preferences.echoCancellation)}
								class="relative h-8 w-14 rounded-full bg-[var(--bg-tertiary)] transition"
							>
								<span class="absolute left-1 top-1 h-6 w-6 rounded-full transition-transform {preferencesStore.preferences.echoCancellation ? 'translate-x-6 bg-[var(--accent)]' : 'bg-[var(--text-secondary)]'}"></span>
							</button>
						</div>

						<div class="flex items-center justify-between">
							<div>
								<div class="text-sm font-medium text-[var(--text-primary)]">Auto Gain Control</div>
								<div class="text-xs text-[var(--text-secondary)]">Automatically adjusts mic sensitivity</div>
							</div>
							<button aria-label="Toggle auto gain control"
								onclick={() => preferencesStore.set('autoGainControl', !preferencesStore.preferences.autoGainControl)}
								class="relative h-8 w-14 rounded-full bg-[var(--bg-tertiary)] transition"
							>
								<span class="absolute left-1 top-1 h-6 w-6 rounded-full transition-transform {preferencesStore.preferences.autoGainControl ? 'translate-x-6 bg-[var(--accent)]' : 'bg-[var(--text-secondary)]'}"></span>
							</button>
						</div>

						<p class="mt-3 text-xs text-[var(--text-secondary)]">
							These settings take effect on your next call, or when switching input devices.
						</p>
					</section>

					<!-- ── Stream Focus ── -->
					<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Stream Focus</h3>

						<div class="flex items-center justify-between">
							<div>
								<div class="text-sm font-medium text-[var(--text-primary)]">Auto-focus streams</div>
								<div class="text-xs text-[var(--text-secondary)]">Hide participant tiles when someone shares their screen</div>
							</div>
							<button aria-label="Toggle auto-focus streams"
								onclick={() => preferencesStore.set('autoHideParticipantsOnStream', !preferencesStore.preferences.autoHideParticipantsOnStream)}
								class="relative h-8 w-14 rounded-full bg-[var(--bg-tertiary)] transition"
							>
								<span class="absolute left-1 top-1 h-6 w-6 rounded-full transition-transform {preferencesStore.preferences.autoHideParticipantsOnStream ? 'translate-x-6 bg-[var(--accent)]' : 'bg-[var(--text-secondary)]'}"></span>
							</button>
						</div>

						<p class="mt-3 text-xs text-[var(--text-secondary)]">
							You can always toggle between focused and tiled view using the button in the top-right corner of the video area.
						</p>
					</section>

					<!-- ── Call Background ── -->
					<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Call Background</h3>
						<p class="mb-4 text-sm text-[var(--text-secondary)]">
							Set a background for your video tile when your camera is off.
						</p>

						<!-- Live preview -->
						<div
							class="mb-4 flex items-center justify-center overflow-hidden rounded-lg"
							style="aspect-ratio: 16/9; max-width: 240px; {voiceBackgroundStyle(preferencesStore.preferences.voiceBackground) || 'background: var(--bg-tertiary);'}"
						>
							<div class="flex h-12 w-12 items-center justify-center rounded-full bg-black/30">
								<svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-white/60" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
							</div>
						</div>

						<!-- Type selector -->
						<div class="mb-4 flex flex-wrap gap-2">
							{#each [
								{ id: 'none', label: 'None' },
								{ id: 'solid', label: 'Solid' },
								{ id: 'gradient', label: 'Gradient' },
								{ id: 'preset', label: 'Preset' },
								{ id: 'custom', label: 'Image' }
							] as opt}
								<button
									onclick={() => { voiceBgType = opt.id as VoiceBackgroundType; applyVoiceBg(); }}
									class="rounded-lg px-3 py-1.5 text-xs font-medium transition {voiceBgType === opt.id ? 'bg-[var(--accent)] text-white' : 'bg-white/5 text-[var(--text-secondary)] hover:bg-white/10'}"
								>
									{opt.label}
								</button>
							{/each}
						</div>

						<!-- Per-type controls -->
						{#if voiceBgType === 'solid'}
							<div class="flex items-center gap-3">
								<input type="color" bind:value={voiceBgColor} onchange={applyVoiceBg} class="h-8 w-8 cursor-pointer rounded border-0 bg-transparent" aria-label="Background color" />
								<span class="text-sm text-[var(--text-secondary)]">{voiceBgColor}</span>
							</div>
						{:else if voiceBgType === 'gradient'}
							<div class="flex flex-col gap-3">
								<div class="flex items-center gap-3">
									<span class="text-xs text-[var(--text-secondary)]">From</span>
									<input type="color" bind:value={voiceBgGradFrom} onchange={applyVoiceBg} class="h-7 w-7 cursor-pointer rounded border-0 bg-transparent" aria-label="Gradient start color" />
									<span class="text-xs text-[var(--text-secondary)]">To</span>
									<input type="color" bind:value={voiceBgGradTo} onchange={applyVoiceBg} class="h-7 w-7 cursor-pointer rounded border-0 bg-transparent" aria-label="Gradient end color" />
								</div>
								<div class="flex items-center gap-2">
									<span class="text-xs text-[var(--text-secondary)]">Angle</span>
									<input type="range" min="0" max="360" bind:value={voiceBgGradAngle} oninput={applyVoiceBg}
										class="h-1.5 w-32 cursor-pointer appearance-none rounded-full bg-white/10 accent-[var(--accent)]" aria-label="Gradient angle" />
									<span class="text-xs text-[var(--text-secondary)]">{voiceBgGradAngle}°</span>
								</div>
							</div>
						{:else if voiceBgType === 'preset'}
							<div class="grid grid-cols-2 sm:grid-cols-3 gap-2">
								{#each Object.entries(VOICE_BG_PRESETS) as [id, preset]}
									<button
										onclick={() => { voiceBgPresetId = id; applyVoiceBg(); }}
										class="overflow-hidden rounded-lg border-2 transition {voiceBgPresetId === id ? 'border-[var(--accent)]' : 'border-transparent hover:border-white/20'}"
									>
										<div class="flex items-center justify-center rounded" style="aspect-ratio: 16/9; background: {preset.css};">
											<span class="rounded bg-black/40 px-2 py-0.5 text-[10px] font-medium text-white">{preset.label}</span>
										</div>
									</button>
								{/each}
							</div>
						{:else if voiceBgType === 'custom'}
							<div class="flex items-center gap-3">
								<input bind:this={voiceBgInputEl} type="file" accept="image/*" class="hidden" onchange={handleVoiceBgUpload} />
								<button
									onclick={() => voiceBgInputEl?.click()}
									disabled={voiceBgUploading}
									class="rounded-lg bg-white/5 px-3 py-1.5 text-sm text-[var(--text-secondary)] transition hover:bg-white/10"
								>
									{voiceBgUploading ? 'Uploading...' : voiceBgCustomUrl ? 'Change Image' : 'Upload Image'}
								</button>
								{#if voiceBgCustomUrl}
									<span class="text-xs text-[var(--text-secondary)]">Image set</span>
								{/if}
							</div>
							<p class="mt-2 text-xs text-[var(--text-secondary)]">Max 2MB. PNG, JPEG, WebP, or GIF.</p>
						{/if}
					</section>

				{:else if activeTab === 'security'}
					<h2 class="mb-6 text-xl font-bold">Security</h2>

					<!-- 2FA -->
					<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Two-Factor Authentication</h3>
						<p class="mb-4 text-sm text-[var(--text-secondary)]">
							Add an extra layer of security with TOTP-based 2FA.
						</p>

						{#if totpMessage}
							<div class="mb-4 rounded-lg border border-green-500/20 bg-green-500/10 px-4 py-3 text-sm text-green-400">
								{totpMessage}
							</div>
						{/if}
						{#if totpError}
							<div class="mb-4 rounded-lg border border-red-500/20 bg-red-500/10 px-4 py-3 text-sm text-red-400">
								{totpError}
							</div>
						{/if}

						{#if showTotpSetup && totpSetup}
							<div class="rounded-lg border border-white/10 bg-[var(--bg-primary)] p-4">
								<p class="mb-3 text-sm">Scan with your authenticator app, or enter the secret manually:</p>
								<div class="mb-3 rounded-lg bg-white p-4 text-center">
									<p class="text-xs text-gray-500">QR Code for: {totpSetup.otpauth_url}</p>
								</div>
								<div class="mb-4">
									<span class="mb-1 block text-xs text-[var(--text-secondary)]">Manual entry secret</span>
									<code class="block select-all rounded bg-[var(--bg-tertiary)] px-3 py-2 font-mono text-sm">
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

					<!-- Recovery Code -->
					<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Recovery Code</h3>
						<p class="mb-4 text-sm text-[var(--text-secondary)]">
							Your recovery code lets you reset your password if you forget it, without needing an admin. Generate a new one if you've lost it.
						</p>

						{#if recoveryError}
							<div class="mb-4 rounded-lg border border-red-500/20 bg-red-500/10 px-4 py-3 text-sm text-red-400">
								{recoveryError}
							</div>
						{/if}

						{#if showRecoveryCode}
							<div class="mb-4 rounded-lg bg-[var(--bg-primary)] p-4">
								<p class="mb-2 text-sm font-medium text-[var(--text-primary)]">Your New Recovery Code</p>
								<p class="mb-3 text-xs text-[var(--text-secondary)]">
									Save this code somewhere safe. It replaces your previous code and will not be shown again.
								</p>
								<div class="mb-3 rounded-lg bg-[var(--bg-tertiary,var(--bg-primary))] p-3 text-center">
									<code class="select-all font-mono text-lg font-bold tracking-wider text-[var(--accent)]">
										{recoveryCode}
									</code>
								</div>
								<div class="flex gap-2">
									<button
										onclick={() => {
											navigator.clipboard.writeText(recoveryCode);
											copiedRecovery = true;
											setTimeout(() => (copiedRecovery = false), 2000);
										}}
										class="flex-1 rounded-lg border border-white/10 px-4 py-2 text-sm text-[var(--text-primary)] transition hover:bg-white/5"
									>
										{copiedRecovery ? 'Copied!' : 'Copy Code'}
									</button>
									<button
										onclick={() => { showRecoveryCode = false; recoveryCode = ''; }}
										class="rounded-lg border border-white/10 px-4 py-2 text-sm text-[var(--text-secondary)] transition hover:bg-white/5"
									>
										Done
									</button>
								</div>
							</div>
						{:else}
							<button
								onclick={handleRegenerateRecoveryCode}
								disabled={recoveryLoading}
								class="rounded-lg bg-[var(--accent)] px-4 py-2 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)] disabled:opacity-50"
							>
								{recoveryLoading ? 'Generating...' : 'Generate New Recovery Code'}
							</button>
						{/if}
					</section>

					<!-- 2FA Backup Codes -->
					{#if showBackupCodes && backupCodes.length > 0}
						<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
							<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">2FA Backup Codes</h3>
							<p class="mb-4 text-sm text-[var(--text-secondary)]">
								Each code can only be used once. Save these in case you lose your authenticator device.
							</p>
							<div class="mb-4 grid grid-cols-2 gap-2">
								{#each backupCodes as code}
									<code class="select-all rounded bg-[var(--bg-primary)] px-3 py-2 text-center font-mono text-sm tracking-wider text-[var(--text-primary)]">
										{code}
									</code>
								{/each}
							</div>
							<div class="flex gap-2">
								<button
									onclick={() => {
										navigator.clipboard.writeText(backupCodes.join('\n'));
										toastStore.success('Backup codes copied');
									}}
									class="rounded-lg border border-white/10 px-4 py-2 text-sm text-[var(--text-primary)] transition hover:bg-white/5"
								>
									Copy All
								</button>
								<button
									onclick={() => { showBackupCodes = false; backupCodes = []; }}
									class="rounded-lg border border-white/10 px-4 py-2 text-sm text-[var(--text-secondary)] transition hover:bg-white/5"
								>
									Done
								</button>
							</div>
						</section>
					{:else}
						<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
							<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">2FA Backup Codes</h3>
							<p class="mb-4 text-sm text-[var(--text-secondary)]">
								Regenerate backup codes if you've used some or lost them. Requires a valid 2FA code.
							</p>
							{#if backupCodeError}
								<div class="mb-4 rounded-lg border border-red-500/20 bg-red-500/10 px-4 py-3 text-sm text-red-400">
									{backupCodeError}
								</div>
							{/if}
							<form onsubmit={handleRegenerateBackupCodes} class="flex gap-2">
								<input
									type="text"
									bind:value={backupCodeTotpInput}
									placeholder="Enter 2FA code"
									maxlength="6"
									pattern="[0-9]{6}"
									class="flex-1 rounded-lg border border-white/10 bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] outline-none focus:border-[var(--accent)]"
								/>
								<button
									type="submit"
									class="rounded-lg bg-[var(--accent)] px-4 py-2 text-sm font-medium text-white transition hover:bg-[var(--accent-hover)]"
								>
									Regenerate
								</button>
							</form>
						</section>
					{/if}

					<!-- Sessions -->
					<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<div class="mb-4 flex items-center justify-between">
							<h3 class="text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Active Sessions</h3>
							<button
								onclick={handleLogoutAll}
								disabled={revokingAll}
								class="rounded-lg border border-red-500/20 px-3 py-1.5 text-xs font-medium text-red-400 transition hover:bg-red-500/10 disabled:opacity-50"
							>
								{revokingAll ? 'Revoking...' : 'Revoke All'}
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
								{#each sessions as session}
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
												<div class="text-sm font-medium">{session.device_name ?? 'Unknown device'}</div>
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
											disabled={revokingSessionId === session.id}
											class="rounded px-2 py-1 text-xs text-red-400 transition hover:bg-red-500/10 disabled:opacity-50"
										>
											{revokingSessionId === session.id ? 'Revoking...' : 'Revoke'}
										</button>
									</div>
								{/each}
							</div>
						{/if}
					</section>

					<!-- Encryption info -->
					<section class="rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Encryption</h3>
						<div class="space-y-3 text-sm text-[var(--text-secondary)]">
							<div class="flex items-center justify-between">
								<span>End-to-end encryption</span>
								<span class="rounded-full bg-green-500/10 px-3 py-1 text-xs text-green-400">Active</span>
							</div>
							<div class="flex items-center justify-between">
								<span>Identity key fingerprint</span>
								<code class="font-mono text-xs text-[var(--text-secondary)]">Stored on device</code>
							</div>
						</div>
					</section>

				<!-- ══════════════════ ACCOUNT TAB ══════════════════ -->
				{:else if activeTab === 'account'}
					<h2 class="mb-6 text-xl font-bold">Account</h2>

					<!-- Change Password -->
					<section class="mb-6 rounded-xl border border-white/10 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-[var(--text-secondary)]">Change Password</h3>
						<p class="mb-4 text-sm text-[var(--text-secondary)]">
							Changing your password will sign you out of all devices.
						</p>

						{#if passwordError}
							<div class="mb-4 rounded-lg border border-red-500/20 bg-red-500/10 px-4 py-3 text-sm text-red-400">
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
										<div class={pwHasLength ? 'text-green-400' : 'text-[var(--text-secondary)]'}>{pwHasLength ? '\u2713' : '\u2717'} At least 8 characters</div>
										<div class={pwHasUpper ? 'text-green-400' : 'text-[var(--text-secondary)]'}>{pwHasUpper ? '\u2713' : '\u2717'} One uppercase letter</div>
										<div class={pwHasLower ? 'text-green-400' : 'text-[var(--text-secondary)]'}>{pwHasLower ? '\u2713' : '\u2717'} One lowercase letter</div>
										<div class={pwHasDigit ? 'text-green-400' : 'text-[var(--text-secondary)]'}>{pwHasDigit ? '\u2713' : '\u2717'} One digit</div>
										<div class={pwHasSpecial ? 'text-green-400' : 'text-[var(--text-secondary)]'}>{pwHasSpecial ? '\u2713' : '\u2717'} One special character</div>
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

					<!-- Danger Zone -->
					<section class="rounded-xl border border-red-500/20 bg-[var(--bg-secondary)] p-6">
						<h3 class="mb-4 text-sm font-semibold uppercase tracking-wider text-red-400">Danger Zone</h3>
						<div class="flex flex-wrap gap-3">
							<button
								onclick={() => { authStore.logout(); goto('/login'); }}
								class="rounded-lg border border-red-500/20 bg-red-500/10 px-4 py-2 text-sm font-medium text-red-400 transition hover:bg-red-500/20"
							>
								Sign Out
							</button>
							<button
								onclick={() => { showDeleteConfirm = true; }}
								class="rounded-lg border border-red-500/20 bg-red-500/10 px-4 py-2 text-sm font-medium text-red-400 transition hover:bg-red-500/20"
							>
								Delete Account
							</button>
							{#if isDesktop && serverUrl}
								<button
									onclick={() => { clearServerUrl(); authStore.logout(); goto('/connect'); }}
									class="rounded-lg border border-red-500/20 bg-red-500/10 px-4 py-2 text-sm font-medium text-red-400 transition hover:bg-red-500/20"
								>
									Disconnect from Server
								</button>
							{/if}
						</div>

						{#if showDeleteConfirm}
							<div class="mt-4 rounded-lg border border-red-500/20 bg-[var(--bg-primary)] p-4">
								<p class="mb-3 text-sm text-red-400">
									This action is permanent and cannot be undone. All your data will be deleted.
								</p>
								{#if deleteError}
									<div class="mb-3 rounded-lg border border-red-500/20 bg-red-500/10 px-4 py-3 text-sm text-red-400">
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
				{/if}
			</div>
		</div>
	</div>
{/if}
