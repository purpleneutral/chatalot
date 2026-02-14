import { authStore } from '$lib/stores/auth.svelte';
import { apiBase } from '$lib/env';

export interface FeedbackResponse {
	success: boolean;
	issue_number?: number;
	message: string;
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

	const headers: Record<string, string> = {};
	const token = authStore.accessToken;
	if (token) {
		headers['Authorization'] = `Bearer ${token}`;
	}

	const response = await fetch(`${apiBase()}/feedback`, {
		method: 'POST',
		headers,
		body: form
	});

	if (!response.ok) {
		const body = await response.json().catch(() => ({}));
		throw new Error(body.error?.message || `HTTP ${response.status}`);
	}

	return response.json();
}
