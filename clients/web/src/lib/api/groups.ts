import { api } from './client';
import type { Channel } from './channels';

export interface Group {
	id: string;
	name: string;
	description: string | null;
	owner_id: string;
	created_at: string;
	member_count: number;
}

export interface GroupMember {
	user_id: string;
	username: string;
	display_name: string;
	avatar_url: string | null;
	role: string;
	joined_at: string;
}

export async function listGroups(): Promise<Group[]> {
	return api.get<Group[]>('/groups');
}

export async function discoverGroups(): Promise<Group[]> {
	return api.get<Group[]>('/groups/discover');
}

export async function createGroup(name: string, description?: string): Promise<Group> {
	return api.post<Group>('/groups', { name, description: description ?? null });
}

export async function updateGroup(
	id: string,
	name?: string,
	description?: string
): Promise<Group> {
	return api.patch<Group>(`/groups/${id}`, {
		name: name ?? null,
		description: description ?? null
	});
}

export async function deleteGroup(id: string): Promise<void> {
	await api.delete(`/groups/${id}`);
}

export async function joinGroup(id: string): Promise<void> {
	await api.post(`/groups/${id}/join`, {});
}

export async function leaveGroup(id: string): Promise<void> {
	await api.post(`/groups/${id}/leave`, {});
}

export async function listGroupMembers(id: string): Promise<GroupMember[]> {
	return api.get<GroupMember[]>(`/groups/${id}/members`);
}

export async function listGroupChannels(id: string): Promise<Channel[]> {
	return api.get<Channel[]>(`/groups/${id}/channels`);
}

export async function createGroupChannel(
	groupId: string,
	name: string,
	channelType: string,
	topic?: string
): Promise<Channel> {
	return api.post<Channel>(`/groups/${groupId}/channels`, {
		name,
		channel_type: channelType,
		topic: topic ?? null
	});
}

export async function updateChannel(
	groupId: string,
	channelId: string,
	name?: string,
	topic?: string
): Promise<Channel> {
	return api.patch<Channel>(`/groups/${groupId}/channels/${channelId}`, {
		name: name ?? null,
		topic: topic ?? null
	});
}

export async function deleteChannel(groupId: string, channelId: string): Promise<void> {
	await api.delete(`/groups/${groupId}/channels/${channelId}`);
}

// ── Invites ──

export interface GroupInvite {
	id: string;
	code: string;
	group_id: string;
	max_uses: number | null;
	used_count: number;
	expires_at: string | null;
	created_at: string;
}

export interface InviteInfo {
	group_name: string;
	group_description: string | null;
	member_count: number;
	code: string;
}

export async function createInvite(
	groupId: string,
	maxUses?: number,
	expiresInHours?: number
): Promise<GroupInvite> {
	return api.post<GroupInvite>(`/groups/${groupId}/invites`, {
		max_uses: maxUses ?? null,
		expires_in_hours: expiresInHours ?? null
	});
}

export async function listInvites(groupId: string): Promise<GroupInvite[]> {
	return api.get<GroupInvite[]>(`/groups/${groupId}/invites`);
}

export async function deleteInvite(groupId: string, inviteId: string): Promise<void> {
	await api.delete(`/groups/${groupId}/invites/${inviteId}`);
}

export async function getInviteInfo(code: string): Promise<InviteInfo> {
	return api.get<InviteInfo>(`/invites/${code}`);
}

export async function acceptInvite(code: string): Promise<{ group_id: string; group_name: string }> {
	return api.post<{ group_id: string; group_name: string }>(`/invites/${code}/accept`, {});
}
