import { api } from './client';
import type { Channel } from './channels';

export interface Group {
	id: string;
	name: string;
	description: string | null;
	icon_url: string | null;
	banner_url: string | null;
	accent_color: string | null;
	owner_id: string;
	community_id: string;
	created_at: string;
	member_count: number;
	visibility: string;
	discoverable: boolean;
	assigned_member_id: string | null;
	allow_invites: boolean;
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

export async function createGroup(
	communityId: string,
	name: string,
	description?: string,
	visibility?: string,
	assignedMemberId?: string
): Promise<Group> {
	return api.post<Group>('/groups', {
		community_id: communityId,
		name,
		description: description ?? null,
		visibility: visibility ?? null,
		assigned_member_id: assignedMemberId ?? null
	});
}

export async function updateGroup(
	id: string,
	updates: {
		name?: string;
		description?: string;
		visibility?: string;
		discoverable?: boolean;
		allow_invites?: boolean;
		icon_url?: string;
		banner_url?: string;
		accent_color?: string;
	}
): Promise<Group> {
	const body: Record<string, string | boolean | null> = {};
	if (updates.name !== undefined) body.name = updates.name;
	if (updates.description !== undefined) body.description = updates.description;
	if (updates.visibility !== undefined) body.visibility = updates.visibility;
	if (updates.discoverable !== undefined) body.discoverable = updates.discoverable;
	if (updates.allow_invites !== undefined) body.allow_invites = updates.allow_invites;
	if (updates.icon_url !== undefined) body.icon_url = updates.icon_url;
	if (updates.banner_url !== undefined) body.banner_url = updates.banner_url;
	if (updates.accent_color !== undefined) body.accent_color = updates.accent_color;
	return api.patch<Group>(`/groups/${id}`, body);
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
	updates: { name?: string; topic?: string; read_only?: boolean; slow_mode_seconds?: number; discoverable?: boolean; archived?: boolean; voice_background?: string | null }
): Promise<Channel> {
	const body: Record<string, string | boolean | number | null> = {};
	if (updates.name !== undefined) body.name = updates.name;
	if (updates.topic !== undefined) body.topic = updates.topic;
	if (updates.read_only !== undefined) body.read_only = updates.read_only;
	if (updates.slow_mode_seconds !== undefined) body.slow_mode_seconds = updates.slow_mode_seconds;
	if (updates.discoverable !== undefined) body.discoverable = updates.discoverable;
	if (updates.archived !== undefined) body.archived = updates.archived;
	if (updates.voice_background !== undefined) body.voice_background = updates.voice_background;
	return api.patch<Channel>(`/groups/${groupId}/channels/${channelId}`, body);
}

export async function deleteChannel(groupId: string, channelId: string): Promise<void> {
	await api.delete(`/groups/${groupId}/channels/${channelId}`);
}

// ── Group Assets ──

export async function uploadGroupIcon(id: string, file: File | Blob): Promise<Group> {
	return api.upload(`/groups/${id}/icon`, 'icon', file);
}

export async function uploadGroupBanner(id: string, file: File | Blob): Promise<Group> {
	return api.upload(`/groups/${id}/banner`, 'banner', file);
}

// ── Channel Voice Background ──

export async function uploadChannelVoiceBackground(groupId: string, channelId: string, file: File | Blob): Promise<Channel> {
	return api.upload(`/groups/${groupId}/channels/${channelId}/voice-background`, 'background', file);
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
