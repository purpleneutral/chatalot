import { api } from './client';
import type { Group } from './groups';

export interface Community {
	id: string;
	name: string;
	description: string | null;
	icon_url: string | null;
	owner_id: string;
	created_at: string;
	member_count: number;
	who_can_create_groups: string;
	who_can_create_invites: string;
	discoverable: boolean;
}

export interface CommunityMember {
	user_id: string;
	username: string;
	display_name: string;
	avatar_url: string | null;
	role: string;
	nickname: string | null;
	joined_at: string;
}

export interface CommunityInvite {
	id: string;
	code: string;
	community_id: string;
	max_uses: number | null;
	used_count: number;
	expires_at: string | null;
	created_at: string;
}

export interface CommunityInviteInfo {
	community_name: string;
	community_description: string | null;
	member_count: number;
	code: string;
}

export interface CommunityBan {
	user_id: string;
	username: string;
	display_name: string;
	reason: string | null;
	created_at: string;
}

// ── Community CRUD ──

export async function listCommunities(): Promise<Community[]> {
	return api.get<Community[]>('/communities');
}

export async function getCommunity(id: string): Promise<Community> {
	return api.get<Community>(`/communities/${id}`);
}

export async function createCommunity(name: string, description?: string): Promise<Community> {
	return api.post<Community>('/communities', { name, description: description ?? null });
}

export async function updateCommunity(
	id: string,
	updates: {
		name?: string;
		description?: string;
		iconUrl?: string;
		who_can_create_groups?: string;
		who_can_create_invites?: string;
		discoverable?: boolean;
	}
): Promise<Community> {
	const body: Record<string, string | boolean | null> = {};
	if (updates.name !== undefined) body.name = updates.name;
	if (updates.description !== undefined) body.description = updates.description;
	if (updates.iconUrl !== undefined) body.icon_url = updates.iconUrl;
	if (updates.who_can_create_groups !== undefined) body.who_can_create_groups = updates.who_can_create_groups;
	if (updates.who_can_create_invites !== undefined) body.who_can_create_invites = updates.who_can_create_invites;
	if (updates.discoverable !== undefined) body.discoverable = updates.discoverable;
	return api.patch<Community>(`/communities/${id}`, body);
}

export async function deleteCommunity(id: string): Promise<void> {
	await api.delete(`/communities/${id}`);
}

export async function transferOwnership(id: string, newOwnerId: string): Promise<void> {
	await api.post(`/communities/${id}/transfer-ownership`, { new_owner_id: newOwnerId });
}

export async function leaveCommunity(id: string): Promise<void> {
	await api.post(`/communities/${id}/leave`, {});
}

// ── Members ──

export async function listMembers(id: string): Promise<CommunityMember[]> {
	return api.get<CommunityMember[]>(`/communities/${id}/members`);
}

export async function setMemberRole(id: string, userId: string, role: string): Promise<void> {
	await api.put(`/communities/${id}/members/${userId}/role`, { role });
}

export async function setNickname(
	id: string,
	userId: string,
	nickname: string | null
): Promise<void> {
	await api.put(`/communities/${id}/members/${userId}/nickname`, { nickname });
}

export async function kickMember(id: string, userId: string): Promise<void> {
	await api.delete(`/communities/${id}/members/${userId}`);
}

// ── Bans ──

export async function listBans(id: string): Promise<CommunityBan[]> {
	return api.get<CommunityBan[]>(`/communities/${id}/bans`);
}

export async function banMember(id: string, userId: string, reason?: string): Promise<void> {
	await api.post(`/communities/${id}/bans/${userId}`, { reason: reason ?? null });
}

export async function unbanMember(id: string, userId: string): Promise<void> {
	await api.delete(`/communities/${id}/bans/${userId}`);
}

// ── Invites ──

export async function listInvites(id: string): Promise<CommunityInvite[]> {
	return api.get<CommunityInvite[]>(`/communities/${id}/invites`);
}

export async function createInvite(
	id: string,
	maxUses?: number,
	expiresInHours?: number
): Promise<CommunityInvite> {
	return api.post<CommunityInvite>(`/communities/${id}/invites`, {
		max_uses: maxUses ?? null,
		expires_in_hours: expiresInHours ?? null
	});
}

export async function deleteInvite(id: string, inviteId: string): Promise<void> {
	await api.delete(`/communities/${id}/invites/${inviteId}`);
}

export async function getInviteInfo(code: string): Promise<CommunityInviteInfo> {
	return api.get<CommunityInviteInfo>(`/community-invites/${code}`);
}

export async function acceptInvite(
	code: string
): Promise<{ community_id: string; community_name: string }> {
	return api.post<{ community_id: string; community_name: string }>(
		`/community-invites/${code}/accept`,
		{}
	);
}

// ── Groups within community ──

export async function listCommunityGroups(id: string): Promise<Group[]> {
	return api.get<Group[]>(`/communities/${id}/groups`);
}
