import { api } from './client';
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
	return api.upload('/account/avatar', 'avatar', file);
}

export async function uploadBanner(file: File): Promise<UserPublic> {
	return api.upload('/account/banner', 'banner', file);
}

export async function uploadVoiceBackground(file: File): Promise<{ url: string }> {
	return api.upload('/account/voice-background', 'background', file);
}
