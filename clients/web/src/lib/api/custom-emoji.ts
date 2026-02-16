import { api } from './client';
import { apiBase } from '$lib/env';
import { authStore } from '$lib/stores/auth.svelte';

export interface CustomEmoji {
	id: string;
	community_id: string;
	shortcode: string;
	url: string;
	content_type: string;
	uploaded_by: string;
	created_at: string;
}

export async function listCommunityEmojis(communityId: string): Promise<CustomEmoji[]> {
	return api.get(`/communities/${communityId}/emojis`);
}

export async function uploadEmoji(communityId: string, shortcode: string, file: File): Promise<CustomEmoji> {
	const formData = new FormData();
	formData.append('shortcode', shortcode);
	formData.append('file', file);

	const headers: Record<string, string> = {};
	const token = authStore.accessToken;
	if (token) {
		headers['Authorization'] = `Bearer ${token}`;
	}

	const response = await fetch(`${apiBase()}/communities/${communityId}/emojis`, {
		method: 'POST',
		headers,
		body: formData
	});

	if (!response.ok) {
		const body = await response.json().catch(() => null);
		throw new Error(body?.error?.message || `Upload failed: ${response.status}`);
	}

	return response.json();
}

export async function deleteEmoji(communityId: string, emojiId: string): Promise<void> {
	return api.delete(`/communities/${communityId}/emojis/${emojiId}`);
}
