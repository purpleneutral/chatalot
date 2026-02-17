import type { UserPublic } from '$lib/api/auth';
import { wipeCrypto } from '$lib/crypto';
import { bookmarkStore } from '$lib/stores/bookmarks.svelte';
import { channelStore } from '$lib/stores/channels.svelte';
import { communityStore } from '$lib/stores/communities.svelte';
import { communityMemberStore } from '$lib/stores/communityMembers.svelte';
import { groupStore } from '$lib/stores/groups.svelte';
import { memberStore } from '$lib/stores/members.svelte';
import { messageStore } from '$lib/stores/messages.svelte';
import { preferencesStore } from '$lib/stores/preferences.svelte';
import { presenceStore } from '$lib/stores/presence.svelte';
import { readReceiptStore } from '$lib/stores/readReceipts.svelte';
import { userStore } from '$lib/stores/users.svelte';
import { voiceStore } from '$lib/stores/voice.svelte';
import { wsClient } from '$lib/ws/connection';
import { clearMarkReadTimer } from '$lib/ws/handler';

const TOKEN_KEY = 'chatalot_access_token';
const REFRESH_KEY = 'chatalot_refresh_token';
const USER_KEY = 'chatalot_user';

class AuthStore {
	accessToken = $state<string | null>(null);
	refreshToken = $state<string | null>(null);
	user = $state<UserPublic | null>(null);

	get isAuthenticated(): boolean {
		return this.accessToken !== null;
	}

	constructor() {
		if (typeof window !== 'undefined') {
			this.accessToken = localStorage.getItem(TOKEN_KEY);
			this.refreshToken = localStorage.getItem(REFRESH_KEY);
			const userJson = localStorage.getItem(USER_KEY);
			if (userJson) {
				try {
					this.user = JSON.parse(userJson);
				} catch {
					this.user = null;
				}
			}

			// Detect cross-tab logout/login via storage events
			window.addEventListener('storage', (e) => {
				if (e.key === TOKEN_KEY) {
					if (e.newValue === null) {
						// Another tab logged out — clear local state and redirect
						this.accessToken = null;
						this.refreshToken = null;
						this.user = null;
						window.location.href = '/login';
					} else {
						// Another tab logged in or refreshed tokens
						this.accessToken = e.newValue;
						this.refreshToken = localStorage.getItem(REFRESH_KEY);
						const u = localStorage.getItem(USER_KEY);
						if (u) try { this.user = JSON.parse(u); } catch { /* ignore */ }
					}
				}
			});
		}
	}

	setAuth(accessToken: string, refreshToken: string, user: UserPublic) {
		this.accessToken = accessToken;
		this.refreshToken = refreshToken;
		this.user = user;
		localStorage.setItem(TOKEN_KEY, accessToken);
		localStorage.setItem(REFRESH_KEY, refreshToken);
		localStorage.setItem(USER_KEY, JSON.stringify(user));
	}

	setTokens(accessToken: string, refreshToken: string) {
		this.accessToken = accessToken;
		this.refreshToken = refreshToken;
		localStorage.setItem(TOKEN_KEY, accessToken);
		localStorage.setItem(REFRESH_KEY, refreshToken);
	}

	updateUser(updates: Partial<UserPublic>) {
		if (this.user) {
			this.user = { ...this.user, ...updates };
			localStorage.setItem(USER_KEY, JSON.stringify(this.user));
		}
	}

	logout() {
		this.accessToken = null;
		this.refreshToken = null;
		this.user = null;
		localStorage.removeItem(TOKEN_KEY);
		localStorage.removeItem(REFRESH_KEY);
		localStorage.removeItem(USER_KEY);
		wsClient.disconnect();
		clearMarkReadTimer();
		preferencesStore.cancelPendingSync();
		voiceStore.reset();
		presenceStore.reset();
		messageStore.clear();
		channelStore.clear();
		groupStore.clear();
		communityStore.clear();
		communityMemberStore.clear();
		memberStore.clear();
		userStore.clear();
		bookmarkStore.clear();
		readReceiptStore.clear();
		wipeCrypto().catch((err) => console.warn('Failed to wipe crypto state:', err));
	}
}

export const authStore = new AuthStore();
