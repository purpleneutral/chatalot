import type { UserPublic } from '$lib/api/auth';
import { wipeCrypto } from '$lib/crypto';

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
		wipeCrypto().catch(() => {});
	}
}

export const authStore = new AuthStore();
