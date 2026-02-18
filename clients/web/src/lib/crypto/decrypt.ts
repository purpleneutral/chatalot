import { channelStore } from '$lib/stores/channels.svelte';
import { initCrypto, getSessionManager } from '$lib/crypto';

export interface DecryptResult {
	content: string;
	encrypted: boolean;
}

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
): Promise<DecryptResult> {
	const channel = channelStore.channels.find((c) => c.id === channelId);
	const isDm = channel?.channel_type === 'dm';
	const bytes = ciphertext instanceof Uint8Array ? ciphertext : new Uint8Array(ciphertext);

	if (isDm) {
		try {
			await initCrypto();
			const sm = getSessionManager();
			const peerUserId = peerUserIdOverride !== undefined ? peerUserIdOverride : senderId;
			if (peerUserId) {
				const content = await sm.decryptOrFallback(peerUserId, bytes, messageId, channelId);
				// Check if it was actually E2E encrypted (v:1 wire format)
				const encrypted = isEncryptedWireFormat(bytes);
				return { content, encrypted };
			}
		} catch (err) {
			console.error('DM decryption failed, falling back to UTF-8:', err);
		}
	} else {
		// Group channel — Sender Key decryption (falls back to UTF-8 for legacy plaintext)
		try {
			await initCrypto();
			const sm = getSessionManager();
			const content = await sm.decryptGroupMessage(channelId, senderId, bytes, messageId);
			const encrypted = isEncryptedWireFormat(bytes);
			return { content, encrypted };
		} catch {
			// Expected for unencrypted messages — fall through to UTF-8
		}
	}

	return { content: new TextDecoder().decode(bytes), encrypted: false };
}

/** Check if ciphertext bytes contain our v:1 encrypted wire format. */
function isEncryptedWireFormat(bytes: Uint8Array): boolean {
	try {
		const text = new TextDecoder().decode(bytes);
		const parsed = JSON.parse(text);
		return parsed?.v === 1;
	} catch {
		return false;
	}
}
