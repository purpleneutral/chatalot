import { api } from './client';

export interface Channel {
	id: string;
	name: string | null;
	channel_type: string;
	topic: string | null;
	created_by: string | null;
	created_at: string;
	group_id: string | null;
	read_only: boolean;
	slow_mode_seconds: number;
	discoverable: boolean;
	archived: boolean;
	voice_background: string | null;
}

export interface ReactionInfo {
	emoji: string;
	user_ids: string[];
}

export interface Message {
	id: string;
	channel_id: string;
	sender_id: string;
	ciphertext: number[];
	nonce: number[];
	message_type: string;
	reply_to_id: string | null;
	sender_key_id: string | null;
	edited_at: string | null;
	created_at: string;
	reactions?: ReactionInfo[];
}

export async function listChannels(): Promise<Channel[]> {
	return api.get<Channel[]>('/channels');
}

export async function createChannel(name: string, channelType: string, topic?: string): Promise<Channel> {
	return api.post<Channel>('/channels', {
		name,
		channel_type: channelType,
		topic: topic ?? null
	});
}

export async function joinChannel(channelId: string): Promise<void> {
	await api.post(`/channels/${channelId}/join`, {});
}

export interface ChannelMember {
	user_id: string;
	username: string;
	display_name: string;
	avatar_url: string | null;
	role: string;
	joined_at: string;
}

export async function getChannelMembers(channelId: string): Promise<ChannelMember[]> {
	return api.get<ChannelMember[]>(`/channels/${channelId}/members`);
}

export async function updateMemberRole(
	channelId: string,
	userId: string,
	role: string
): Promise<void> {
	await api.patch(`/channels/${channelId}/members/${userId}/role`, { role });
}

export async function kickMember(channelId: string, userId: string): Promise<void> {
	await api.post(`/channels/${channelId}/members/${userId}/kick`, {});
}

export async function banMember(
	channelId: string,
	userId: string,
	reason?: string
): Promise<void> {
	await api.post(`/channels/${channelId}/members/${userId}/ban`, { reason: reason ?? null });
}

export async function unbanMember(channelId: string, userId: string): Promise<void> {
	await api.post(`/channels/${channelId}/members/${userId}/unban`, {});
}

export async function getMessages(channelId: string, before?: string, limit?: number): Promise<Message[]> {
	const params = new URLSearchParams();
	if (before) params.set('before', before);
	if (limit) params.set('limit', String(limit));
	const query = params.toString();
	return api.get<Message[]>(`/channels/${channelId}/messages${query ? '?' + query : ''}`);
}

export async function searchMessages(channelId: string, query: string, limit?: number): Promise<Message[]> {
	const params = new URLSearchParams({ q: query });
	if (limit) params.set('limit', String(limit));
	return api.get<Message[]>(`/channels/${channelId}/messages/search?${params.toString()}`);
}

export async function searchMessagesGlobal(query: string, limit?: number): Promise<Message[]> {
	const params = new URLSearchParams({ q: query });
	if (limit) params.set('limit', String(limit));
	return api.get<Message[]>(`/messages/search?${params.toString()}`);
}

// ── Pinned Messages ──

export interface PinnedMessage {
	id: string;
	channel_id: string;
	sender_id: string | null;
	ciphertext: number[];
	nonce: number[];
	message_type: string;
	reply_to_id: string | null;
	sender_key_id: string | null;
	edited_at: string | null;
	created_at: string;
	pinned_by: string;
	pinned_at: string;
}

export async function getPinnedMessages(channelId: string): Promise<PinnedMessage[]> {
	return api.get<PinnedMessage[]>(`/channels/${channelId}/pins`);
}

export async function pinMessage(channelId: string, messageId: string): Promise<void> {
	await api.post(`/channels/${channelId}/pins/${messageId}`, {});
}

export async function unpinMessage(channelId: string, messageId: string): Promise<void> {
	await api.delete(`/channels/${channelId}/pins/${messageId}`);
}
