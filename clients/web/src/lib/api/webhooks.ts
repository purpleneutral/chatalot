import { api } from './client';

export interface Webhook {
	id: string;
	channel_id: string;
	name: string;
	token?: string;
	avatar_url: string | null;
	active: boolean;
	created_at: string;
}

export async function createWebhook(channelId: string, name: string, avatarUrl?: string): Promise<Webhook> {
	return api.post(`/channels/${channelId}/webhooks`, { name, avatar_url: avatarUrl ?? null });
}

export async function listWebhooks(channelId: string): Promise<Webhook[]> {
	return api.get(`/channels/${channelId}/webhooks`);
}

export async function updateWebhook(id: string, updates: { name?: string; avatar_url?: string | null; active?: boolean }): Promise<Webhook> {
	return api.patch(`/webhooks/${id}`, updates);
}

export async function deleteWebhook(id: string): Promise<void> {
	return api.delete(`/webhooks/${id}`);
}
