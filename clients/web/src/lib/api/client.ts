import { authStore } from '$lib/stores/auth.svelte';
import { apiBase } from '$lib/env';

const MAX_RETRIES = 2;
const RETRY_STATUSES = new Set([502, 503, 504]);

class ApiClient {
	private refreshPromise: Promise<boolean> | null = null;

	/** Fetch with exponential backoff on transient failures (502/503/504 or network error). */
	private async fetchWithRetry(url: string, init: RequestInit): Promise<Response> {
		let lastError: unknown;
		for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
			try {
				const response = await fetch(url, init);
				if (attempt < MAX_RETRIES && RETRY_STATUSES.has(response.status)) {
					await this.delay(300 * 2 ** attempt);
					continue;
				}
				return response;
			} catch (err) {
				lastError = err;
				if (attempt < MAX_RETRIES) {
					await this.delay(300 * 2 ** attempt);
					continue;
				}
			}
		}
		throw lastError;
	}

	private delay(ms: number): Promise<void> {
		return new Promise(r => setTimeout(r, ms));
	}

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
		const response = await this.fetchWithRetry(`${base}${path}`, {
			...options,
			headers
		});

		if (response.status === 401 && token) {
			// Try to refresh the token
			const refreshed = await this.refreshToken();
			if (refreshed) {
				headers['Authorization'] = `Bearer ${authStore.accessToken}`;
				const retryResponse = await this.fetchWithRetry(`${base}${path}`, {
					...options,
					headers
				});
				if (!retryResponse.ok) {
					throw await this.parseError(retryResponse);
				}
				const retryText = await retryResponse.text();
				if (!retryText) return undefined as T;
				try { return JSON.parse(retryText); } catch { return retryText as T; }
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
		try { return JSON.parse(text); } catch { return text as T; }
	}

	private async parseError(response: Response): Promise<Error> {
		if (response.status === 429) {
			const retryAfter = response.headers.get('Retry-After');
			const secs = retryAfter ? parseInt(retryAfter, 10) : 0;
			return new Error(secs > 0 ? `Too many requests — try again in ${secs}s` : 'Too many requests — slow down');
		}
		try {
			const body = await response.json();
			return new Error(body.error?.message || body.message || `HTTP ${response.status}`);
		} catch {
			return new Error(`HTTP ${response.status}`);
		}
	}

	private async refreshToken(): Promise<boolean> {
		// Deduplicate concurrent refresh attempts (e.g. multiple 401s at once)
		if (this.refreshPromise) return this.refreshPromise;
		this.refreshPromise = this.doRefreshToken();
		try {
			return await this.refreshPromise;
		} finally {
			this.refreshPromise = null;
		}
	}

	private async doRefreshToken(): Promise<boolean> {
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

	/** Upload a file via multipart/form-data. */
	async upload<T>(path: string, fieldName: string, file: File | Blob, extraFields?: Record<string, string>): Promise<T> {
		const buildFormData = () => {
			const fd = new FormData();
			if (extraFields) {
				for (const [key, value] of Object.entries(extraFields)) {
					fd.append(key, value);
				}
			}
			fd.append(fieldName, file, file instanceof File ? file.name : 'cropped.webp');
			return fd;
		};

		const headers: Record<string, string> = {};
		const token = authStore.accessToken;
		if (token) headers['Authorization'] = `Bearer ${token}`;

		const base = apiBase();
		const response = await this.fetchWithRetry(`${base}${path}`, {
			method: 'POST',
			headers,
			body: buildFormData(),
		});

		if (response.status === 401 && token) {
			const refreshed = await this.refreshToken();
			if (refreshed) {
				headers['Authorization'] = `Bearer ${authStore.accessToken}`;
				const retryResponse = await this.fetchWithRetry(`${base}${path}`, {
					method: 'POST',
					headers,
					body: buildFormData(),
				});
				if (!retryResponse.ok) throw await this.parseError(retryResponse);
				return retryResponse.json();
			}
			authStore.logout();
			throw new Error('Session expired');
		}

		if (!response.ok) throw await this.parseError(response);
		return response.json();
	}
}

export const api = new ApiClient();
