import { api } from './client';

export interface ScheduledMessage {
	id: string;
	channel_id: string;
	scheduled_for: string;
	created_at: string;
	/** Encrypted preview stored on server (opaque blob, decrypted client-side with personal key) */
	content_preview?: string;
	/** Client-only: decrypted plaintext for display */
	content?: string;
}

export async function scheduleMessage(
	channelId: string,
	ciphertext: string,
	nonce: string,
	scheduledFor: string,
	contentPreview?: string
): Promise<ScheduledMessage> {
	return api.post('/messages/schedule', {
		channel_id: channelId,
		ciphertext,
		nonce,
		scheduled_for: scheduledFor,
		content_preview: contentPreview
	});
}

export async function listScheduledMessages(): Promise<ScheduledMessage[]> {
	return api.get('/messages/scheduled');
}

export async function cancelScheduledMessage(id: string): Promise<void> {
	return api.delete(`/messages/scheduled/${id}`);
}
