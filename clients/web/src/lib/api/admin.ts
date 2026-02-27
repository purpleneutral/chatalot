import { api } from './client';

export interface AdminUserMembership {
	id: string;
	name: string;
	role: string;
}

export interface AdminUser {
	id: string;
	username: string;
	display_name: string;
	email: string;
	avatar_url: string | null;
	is_admin: boolean;
	is_owner: boolean;
	suspended_at: string | null;
	suspended_reason: string | null;
	created_at: string;
	groups: AdminUserMembership[];
	communities: AdminUserMembership[];
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

export async function resetUserPassword(
	userId: string,
	newPassword: string
): Promise<void> {
	return api.put(`/admin/users/${userId}/password`, { new_password: newPassword });
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

// ── File Management ──

export interface AdminFileEntry {
	id: string;
	uploader_id: string;
	encrypted_name: string;
	size_bytes: number;
	content_type: string | null;
	checksum: string;
	channel_id: string | null;
	quarantined_at: string | null;
	quarantined_by: string | null;
	created_at: string;
	has_thumbnail: boolean;
}

export interface AdminFilesResponse {
	files: AdminFileEntry[];
	total: number;
	page: number;
	per_page: number;
}

export async function listFiles(params?: {
	page?: number;
	per_page?: number;
	user_id?: string;
	sort?: string;
	search?: string;
	content_type?: string;
	date_from?: string;
	date_to?: string;
}): Promise<AdminFilesResponse> {
	const query = new URLSearchParams();
	if (params?.page) query.set('page', String(params.page));
	if (params?.per_page) query.set('per_page', String(params.per_page));
	if (params?.user_id) query.set('user_id', params.user_id);
	if (params?.sort) query.set('sort', params.sort);
	if (params?.search) query.set('search', params.search);
	if (params?.content_type) query.set('content_type', params.content_type);
	if (params?.date_from) query.set('date_from', params.date_from);
	if (params?.date_to) query.set('date_to', params.date_to);
	const qs = query.toString();
	return api.get(`/admin/files${qs ? `?${qs}` : ''}`);
}

export async function adminDeleteFile(fileId: string, blockHash = false): Promise<void> {
	const qs = blockHash ? '?block_hashes=true' : '';
	return api.delete(`/admin/files/${fileId}${qs}`);
}

export async function quarantineFile(fileId: string): Promise<void> {
	return api.post(`/admin/files/${fileId}/quarantine`, {});
}

export async function unquarantineFile(fileId: string): Promise<void> {
	return api.post(`/admin/files/${fileId}/unquarantine`, {});
}

// ── Storage Stats ──

export interface UserStorageStat {
	user_id: string;
	file_count: number;
	total_bytes: number;
}

export interface StorageStats {
	total_files: number;
	total_bytes: number;
	per_user: UserStorageStat[];
}

export async function getStorageStats(): Promise<StorageStats> {
	return api.get('/admin/storage-stats');
}

// ── Purge Endpoints ──

export interface PurgeResult {
	messages_deleted: number;
	files_deleted: number;
	hashes_blocked: number;
}

export async function purgeMessage(messageId: string, blockHashes = false): Promise<PurgeResult> {
	const qs = blockHashes ? '?block_hashes=true' : '';
	return api.post(`/admin/purge/message/${messageId}${qs}`, {});
}

export async function purgeUserMessages(userId: string, blockHashes = false): Promise<PurgeResult> {
	const qs = blockHashes ? '?block_hashes=true' : '';
	return api.post(`/admin/purge/user/${userId}/messages${qs}`, {});
}

export async function purgeChannel(channelId: string, blockHashes = false): Promise<PurgeResult> {
	const qs = blockHashes ? '?block_hashes=true' : '';
	return api.post(`/admin/purge/channel/${channelId}${qs}`, {});
}

// ── Message Quarantine ──

export async function quarantineMessage(messageId: string): Promise<void> {
	return api.post(`/admin/messages/${messageId}/quarantine`, {});
}

export async function unquarantineMessage(messageId: string): Promise<void> {
	return api.post(`/admin/messages/${messageId}/unquarantine`, {});
}

// ── Blocked Hashes ──

export interface BlockedHash {
	id: string;
	hash: string;
	reason: string | null;
	blocked_by: string;
	created_at: string;
}

export async function listBlockedHashes(params?: {
	page?: number;
	per_page?: number;
}): Promise<BlockedHash[]> {
	const query = new URLSearchParams();
	if (params?.page) query.set('page', String(params.page));
	if (params?.per_page) query.set('per_page', String(params.per_page));
	const qs = query.toString();
	return api.get(`/admin/blocked-hashes${qs ? `?${qs}` : ''}`);
}

export async function addBlockedHash(hash: string, reason?: string): Promise<BlockedHash> {
	return api.post('/admin/blocked-hashes', { hash, reason: reason ?? null });
}

export async function removeBlockedHash(id: string): Promise<void> {
	return api.delete(`/admin/blocked-hashes/${id}`);
}

// ── Audit Log ──

export interface AuditLogEntry {
	id: string;
	user_id: string | null;
	action: string;
	ip_address: string | null;
	user_agent: string | null;
	metadata: Record<string, unknown> | null;
	created_at: string;
}

export interface AuditLogResponse {
	entries: AuditLogEntry[];
	total: number;
	page: number;
	per_page: number;
}

export async function getAuditLog(params?: {
	page?: number;
	per_page?: number;
	action?: string;
	user_id?: string;
}): Promise<AuditLogResponse> {
	const query = new URLSearchParams();
	if (params?.page) query.set('page', String(params.page));
	if (params?.per_page) query.set('per_page', String(params.per_page));
	if (params?.action) query.set('action', params.action);
	if (params?.user_id) query.set('user_id', params.user_id);
	const qs = query.toString();
	return api.get(`/admin/audit-log${qs ? `?${qs}` : ''}`);
}

// ── Reports ──

export interface Report {
	id: string;
	reporter_id: string;
	report_type: string;
	target_id: string;
	reason: string;
	status: string;
	reviewed_by: string | null;
	reviewed_at: string | null;
	admin_notes: string | null;
	created_at: string;
}

export interface ReportsResponse {
	reports: Report[];
	total: number;
	page: number;
	per_page: number;
}

export async function listReports(params?: {
	status?: string;
	page?: number;
	per_page?: number;
}): Promise<ReportsResponse> {
	const query = new URLSearchParams();
	if (params?.status) query.set('status', params.status);
	if (params?.page) query.set('page', String(params.page));
	if (params?.per_page) query.set('per_page', String(params.per_page));
	const qs = query.toString();
	return api.get(`/admin/reports${qs ? `?${qs}` : ''}`);
}

export async function reviewReport(
	reportId: string,
	status: string,
	adminNotes?: string
): Promise<Report> {
	return api.post(`/admin/reports/${reportId}/review`, {
		status,
		admin_notes: adminNotes ?? null
	});
}

// ── Instance Settings ──

export async function getInstanceSettings(): Promise<Record<string, string>> {
	return api.get('/admin/settings');
}

export async function updateInstanceSettings(updates: Record<string, string>): Promise<Record<string, string>> {
	return api.put('/admin/settings', updates);
}

// ── Webhooks Overview ──

export interface AdminWebhook {
	id: string;
	channel_id: string;
	name: string;
	active: boolean;
	created_by: string;
	created_at: string;
}

export interface AdminWebhooksResponse {
	webhooks: AdminWebhook[];
	total: number;
}

export async function listAllWebhooks(params?: {
	page?: number;
	per_page?: number;
}): Promise<AdminWebhooksResponse> {
	const query = new URLSearchParams();
	if (params?.page) query.set('page', String(params.page));
	if (params?.per_page) query.set('per_page', String(params.per_page));
	const qs = query.toString();
	return api.get(`/admin/webhooks${qs ? `?${qs}` : ''}`);
}
