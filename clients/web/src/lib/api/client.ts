import { authStore } from '$lib/stores/auth.svelte';
import { apiBase } from '$lib/env';

class ApiClient {
	private async request<T>(path: string, options: RequestInit = {}): Promise<T> {
		const headers: Record<string, string> = {
			'Content-Type': 'application/json',
			...(options.headers as Record<string, string> || {})
		};

		const token = authStore.accessToken;
		if (token) {
			headers['Authorization'] = `Bearer ${token}`;
		}

		const base = apiBase();
		const response = await fetch(`${base}${path}`, {
			...options,
			headers
		});

		if (response.status === 401 && token) {
			// Try to refresh the token
			const refreshed = await this.refreshToken();
			if (refreshed) {
				headers['Authorization'] = `Bearer ${authStore.accessToken}`;
				const retryResponse = await fetch(`${base}${path}`, {
					...options,
					headers
				});
				if (!retryResponse.ok) {
					throw await this.parseError(retryResponse);
				}
				const retryText = await retryResponse.text();
				if (!retryText) return undefined as T;
				return JSON.parse(retryText);
			}
			// Refresh failed, logout
			authStore.logout();
			throw new Error('Session expired');
		}

		if (!response.ok) {
			throw await this.parseError(response);
		}

		// Handle empty responses (204 or empty body) for void endpoints
		const text = await response.text();
		if (!text) return undefined as T;
		return JSON.parse(text);
	}

	private async parseError(response: Response): Promise<Error> {
		try {
			const body = await response.json();
			return new Error(body.error?.message || `HTTP ${response.status}`);
		} catch {
			return new Error(`HTTP ${response.status}`);
		}
	}

	private async refreshToken(): Promise<boolean> {
		const refreshToken = authStore.refreshToken;
		if (!refreshToken) return false;

		try {
			const response = await fetch(`${apiBase()}/auth/refresh`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ refresh_token: refreshToken })
			});

			if (!response.ok) return false;

			const data = await response.json();
			authStore.setTokens(data.access_token, data.refresh_token);
			return true;
		} catch {
			return false;
		}
	}

	async get<T>(path: string): Promise<T> {
		return this.request<T>(path, { method: 'GET' });
	}

	async post<T>(path: string, body: unknown): Promise<T> {
		return this.request<T>(path, {
			method: 'POST',
			body: JSON.stringify(body)
		});
	}

	async patch<T>(path: string, body: unknown): Promise<T> {
		return this.request<T>(path, {
			method: 'PATCH',
			body: JSON.stringify(body)
		});
	}

	async put<T>(path: string, body: unknown): Promise<T> {
		return this.request<T>(path, {
			method: 'PUT',
			body: JSON.stringify(body)
		});
	}

	async delete<T>(path: string, body?: unknown): Promise<T> {
		const options: RequestInit = { method: 'DELETE' };
		if (body !== undefined) {
			options.body = JSON.stringify(body);
		}
		return this.request<T>(path, options);
	}
}

export const api = new ApiClient();
