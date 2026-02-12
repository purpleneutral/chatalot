import { api } from './client';

export interface AdminUser {
	id: string;
	username: string;
	display_name: string;
	email: string;
	avatar_url: string | null;
	is_admin: boolean;
	suspended_at: string | null;
	suspended_reason: string | null;
	created_at: string;
}

export async function listUsers(params?: {
	search?: string;
	limit?: number;
	offset?: number;
}): Promise<AdminUser[]> {
	const query = new URLSearchParams();
	if (params?.search) query.set('search', params.search);
	if (params?.limit) query.set('limit', String(params.limit));
	if (params?.offset) query.set('offset', String(params.offset));
	const qs = query.toString();
	return api.get(`/admin/users${qs ? `?${qs}` : ''}`);
}

export async function suspendUser(
	userId: string,
	reason?: string
): Promise<void> {
	return api.post(`/admin/users/${userId}/suspend`, { reason: reason ?? null });
}

export async function unsuspendUser(userId: string): Promise<void> {
	return api.post(`/admin/users/${userId}/unsuspend`, {});
}

export async function deleteUser(userId: string): Promise<void> {
	return api.delete(`/admin/users/${userId}`);
}

export async function setAdmin(
	userId: string,
	isAdmin: boolean
): Promise<void> {
	return api.put(`/admin/users/${userId}/admin`, { is_admin: isAdmin });
}

// ── Registration Invites ──

export interface RegistrationInvite {
	id: string;
	code: string;
	created_by: string;
	max_uses: number | null;
	used_count: number;
	expires_at: string | null;
	created_at: string;
}

export async function createRegistrationInvite(params?: {
	max_uses?: number;
	expires_in_hours?: number;
}): Promise<RegistrationInvite> {
	return api.post('/admin/invites', {
		max_uses: params?.max_uses ?? null,
		expires_in_hours: params?.expires_in_hours ?? null
	});
}

export async function listRegistrationInvites(): Promise<RegistrationInvite[]> {
	return api.get('/admin/invites');
}

export async function deleteRegistrationInvite(id: string): Promise<void> {
	return api.delete(`/admin/invites/${id}`);
}
