import { api } from './client';

export interface UserPublic {
	id: string;
	username: string;
	display_name: string;
	avatar_url: string | null;
	banner_url: string | null;
	status: string;
	custom_status: string | null;
	is_admin?: boolean;
	is_owner?: boolean;
	created_at?: string;
}

export async function searchUsers(query: string): Promise<UserPublic[]> {
	return api.get<UserPublic[]>(`/users/search?q=${encodeURIComponent(query)}`);
}

export async function getUser(userId: string): Promise<UserPublic> {
	return api.get<UserPublic>(`/users/${userId}`);
}

// ── Blocking ──

export interface BlockedUser {
	blocked_id: string;
	created_at: string;
}

export async function blockUser(userId: string): Promise<void> {
	await api.post('/users/block', { user_id: userId });
}

export async function unblockUser(userId: string): Promise<void> {
	await api.post(`/users/unblock/${userId}`, {});
}

export async function listBlockedUsers(): Promise<BlockedUser[]> {
	return api.get<BlockedUser[]>('/users/blocked');
}

// ── Reports ──

export async function createReport(reportType: string, targetId: string, reason: string): Promise<void> {
	await api.post('/reports', { report_type: reportType, target_id: targetId, reason });
}
