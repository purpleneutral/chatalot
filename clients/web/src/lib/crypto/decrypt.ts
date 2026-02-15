import { channelStore } from '$lib/stores/channels.svelte';
import { initCrypto, getSessionManager } from '$lib/crypto';

/**
 * Decrypt message ciphertext: E2E for DMs, plain UTF-8 for group channels.
 *
 * Used by both the WebSocket handler (real-time) and REST API consumers
 * (search, pagination, pinned messages).
 *
 * @param channelId  The channel the message belongs to
 * @param senderId   The sender's user ID
 * @param ciphertext Raw ciphertext bytes
 * @param messageId  Optional message ID for decrypted-message cache lookup
 * @param peerUserIdOverride  For own messages in DMs, pass the peer's user ID
 *                            (since the session is keyed by peer, not self)
 */
export async function decryptMessage(
	channelId: string,
	senderId: string,
	ciphertext: number[] | Uint8Array,
	messageId?: string,
	peerUserIdOverride?: string | null,
): Promise<string> {
	const channel = channelStore.channels.find((c) => c.id === channelId);
	const isDm = channel?.channel_type === 'dm';
	const bytes = ciphertext instanceof Uint8Array ? ciphertext : new Uint8Array(ciphertext);

	if (isDm) {
		try {
			await initCrypto();
			const sm = getSessionManager();
			const peerUserId = peerUserIdOverride !== undefined ? peerUserIdOverride : senderId;
			if (peerUserId) {
				return await sm.decryptOrFallback(peerUserId, bytes, messageId, channelId);
			}
		} catch (err) {
			console.error('DM decryption failed, falling back to UTF-8:', err);
		}
	} else {
		// Group channel — Sender Key decryption (falls back to UTF-8 for legacy plaintext)
		try {
			await initCrypto();
			const sm = getSessionManager();
			return await sm.decryptGroupMessage(channelId, senderId, bytes, messageId);
		} catch {
			// Expected for unencrypted messages — fall through to UTF-8
		}
	}

	return new TextDecoder().decode(bytes);
}
