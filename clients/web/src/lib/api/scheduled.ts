import { api } from './client';

export interface ScheduledMessage {
	id: string;
	channel_id: string;
	scheduled_for: string;
	created_at: string;
	/** Client-only: plaintext cached locally (not from server — E2E safe) */
	content?: string;
}

export async function scheduleMessage(
	channelId: string,
	ciphertext: string,
	nonce: string,
	scheduledFor: string
): Promise<ScheduledMessage> {
	return api.post('/messages/schedule', {
		channel_id: channelId,
		ciphertext,
		nonce,
		scheduled_for: scheduledFor
	});
}

export async function listScheduledMessages(): Promise<ScheduledMessage[]> {
	return api.get('/messages/scheduled');
}

export async function cancelScheduledMessage(id: string): Promise<void> {
	return api.delete(`/messages/scheduled/${id}`);
}
