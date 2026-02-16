import { api } from './client';
import { apiBase } from '$lib/env';
import { authStore } from '$lib/stores/auth.svelte';
import type { UserPublic } from './users';

export async function getMe(): Promise<UserPublic> {
	return api.get('/account/me');
}

export interface SessionInfo {
	id: string;
	device_name: string | null;
	ip_address: string | null;
	created_at: string;
	expires_at: string;
}

export async function changePassword(
	currentPassword: string,
	newPassword: string
): Promise<void> {
	return api.put('/account/password', {
		current_password: currentPassword,
		new_password: newPassword
	});
}

export async function updateProfile(updates: {
	display_name?: string;
	avatar_url?: string | null;
	banner_url?: string | null;
	custom_status?: string | null;
}): Promise<UserPublic> {
	return api.put('/account/profile', updates);
}

export async function deleteAccount(password: string): Promise<void> {
	return api.delete('/account', { password });
}

export async function logoutAll(): Promise<{ revoked_count: number }> {
	return api.post('/account/logout-all', {});
}

export async function listSessions(): Promise<SessionInfo[]> {
	return api.get('/account/sessions');
}

export async function revokeSession(sessionId: string): Promise<void> {
	return api.delete(`/account/sessions/${sessionId}`);
}

export async function uploadAvatar(file: File): Promise<UserPublic> {
	const formData = new FormData();
	formData.append('avatar', file);

	const headers: Record<string, string> = {};
	const token = authStore.accessToken;
	if (token) {
		headers['Authorization'] = `Bearer ${token}`;
	}

	const response = await fetch(`${apiBase()}/account/avatar`, {
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

export async function uploadBanner(file: File): Promise<UserPublic> {
	const formData = new FormData();
	formData.append('banner', file);

	const headers: Record<string, string> = {};
	const token = authStore.accessToken;
	if (token) {
		headers['Authorization'] = `Bearer ${token}`;
	}

	const response = await fetch(`${apiBase()}/account/banner`, {
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
