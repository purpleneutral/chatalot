import { authStore } from '$lib/stores/auth.svelte';
import { apiBase } from '$lib/env';

export interface FeedbackResponse {
	success: boolean;
	issue_number?: number;
	message: string;
}

async function doFeedbackRequest(form: FormData): Promise<Response> {
	const headers: Record<string, string> = {};
	const token = authStore.accessToken;
	if (token) {
		headers['Authorization'] = `Bearer ${token}`;
	}

	return fetch(`${apiBase()}/feedback`, {
		method: 'POST',
		headers,
		body: form
	});
}

export async function submitFeedback(params: {
	title: string;
	description: string;
	category: string;
	screenshot?: File | null;
}): Promise<FeedbackResponse> {
	const form = new FormData();
	form.append('title', params.title);
	form.append('description', params.description);
	form.append('category', params.category);
	if (params.screenshot) {
		form.append('screenshot', params.screenshot);
	}

	let response = await doFeedbackRequest(form);

	// Handle expired token — refresh and retry
	if (response.status === 401) {
		const refreshToken = authStore.refreshToken;
		if (refreshToken) {
			const refreshResp = await fetch(`${apiBase()}/auth/refresh`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ refresh_token: refreshToken })
			});
			if (refreshResp.ok) {
				const data = await refreshResp.json();
				authStore.setTokens(data.access_token, data.refresh_token);
				// Rebuild form since the original body stream is consumed
				const retryForm = new FormData();
				retryForm.append('title', params.title);
				retryForm.append('description', params.description);
				retryForm.append('category', params.category);
				if (params.screenshot) {
					retryForm.append('screenshot', params.screenshot);
				}
				response = await doFeedbackRequest(retryForm);
			} else {
				authStore.logout();
				throw new Error('Session expired');
			}
		}
	}

	if (!response.ok) {
		const body = await response.json().catch(() => ({}));
		throw new Error(body.error?.message || `HTTP ${response.status}`);
	}

	return response.json();
}
