import { getCrypto } from './wasm-loader';
import type { CryptoStorage } from './storage';
import type { KeyManager } from './key-manager';
import { getKeyBundle } from '$lib/api/keys';
import { uploadSenderKey, getSenderKeys } from '$lib/api/sender-keys';
import { authStore } from '$lib/stores/auth.svelte';

/**
 * Wire format for encrypted DM messages.
 * Embedded as JSON bytes in the ciphertext field of WebSocket messages.
 */
export interface WireMessage {
	v: 1;
	x3dh?: {
		identity_key: number[];
		ephemeral_key: number[];
		signed_prekey_id: number;
		one_time_prekey_id: number | null;
	};
	header: {
		ratchet_key: number[];
		previous_chain_length: number;
		message_number: number;
	};
	ciphertext: number[];
	nonce: number[];
}

export class SessionManager {
	constructor(
		private storage: CryptoStorage,
		private keyManager: KeyManager,
	) {}

	/**
	 * Store a peer's identity key, comparing with any previously stored key.
	 * If the key has changed (TOFU violation), dispatches a custom event.
	 */
	private async storePeerIdentity(peerUserId: string, identityKey: Uint8Array): Promise<void> {
		const existing = await this.storage.getPeerIdentity(peerUserId);
		if (existing && !arraysEqual(existing, identityKey)) {
			// Identity key changed — possible MITM or device change
			if (typeof window !== 'undefined') {
				window.dispatchEvent(new CustomEvent('chatalot:identity-key-changed', {
					detail: { userId: peerUserId },
				}));
			}
		}
		await this.storage.setPeerIdentity(peerUserId, identityKey);
	}

	/**
	 * Encrypt a plaintext string for a DM peer.
	 * If no session exists, performs X3DH first (fetches their key bundle).
	 * Returns the wire-format bytes to put in the WS ciphertext field.
	 */
	async encryptForPeer(
		peerUserId: string,
		plaintext: string,
	): Promise<{ ciphertext: number[]; nonce: number[] }> {
		const crypto = await getCrypto();
		const signingKey = await this.keyManager.getSigningKey();

		let sessionJson = await this.storage.getSession(peerUserId);
		let x3dhHeader: WireMessage['x3dh'] | undefined;

		// If no session, perform X3DH to establish one
		if (!sessionJson) {
			const bundle = await getKeyBundle(peerUserId);

			const result = crypto.x3dh_initiate(
				signingKey,
				JSON.stringify(bundle),
			) as {
				session_json: string;
				ephemeral_public_key: number[];
				associated_data: number[];
			};

			sessionJson = result.session_json;

			// Store peer identity (trust on first use, detect key changes)
			await this.storePeerIdentity(peerUserId, new Uint8Array(bundle.identity_key));

			x3dhHeader = {
				identity_key: Array.from(await this.keyManager.getVerifyingKey()),
				ephemeral_key: result.ephemeral_public_key,
				signed_prekey_id: bundle.signed_prekey.key_id,
				one_time_prekey_id: bundle.one_time_prekey?.key_id ?? null,
			};
		}

		// Encrypt with Double Ratchet
		const encResult = crypto.ratchet_encrypt(
			sessionJson,
			new TextEncoder().encode(plaintext),
		) as {
			session_json: string;
			encrypted: {
				header: { ratchet_key: number[]; previous_chain_length: number; message_number: number };
				ciphertext: number[];
				nonce: number[];
			};
		};

		// Persist updated session
		await this.storage.setSession(peerUserId, encResult.session_json);

		// Build wire message
		const wire: WireMessage = {
			v: 1,
			...(x3dhHeader ? { x3dh: x3dhHeader } : {}),
			header: encResult.encrypted.header,
			ciphertext: encResult.encrypted.ciphertext,
			nonce: encResult.encrypted.nonce,
		};

		const wireBytes = new TextEncoder().encode(JSON.stringify(wire));
		return {
			ciphertext: Array.from(wireBytes),
			nonce: encResult.encrypted.nonce,
		};
	}

	/**
	 * Decrypt a WireMessage from a DM peer.
	 * If the message contains an X3DH header, initializes a responder session.
	 */
	async decryptFromPeer(
		peerUserId: string,
		ciphertextBytes: Uint8Array,
	): Promise<string> {
		const crypto = await getCrypto();
		const text = new TextDecoder().decode(ciphertextBytes);
		const wire: WireMessage = JSON.parse(text);

		if (wire.v !== 1) {
			throw new Error(`Unsupported wire message version: ${wire.v}`);
		}

		let sessionJson = await this.storage.getSession(peerUserId);

		// Handle X3DH header (first message from a new peer)
		if (wire.x3dh) {
			const signingKey = await this.keyManager.getSigningKey();

			// Look up our private keys for the prekeys they used
			const spk = await this.storage.getSignedPrekey(wire.x3dh.signed_prekey_id);
			if (!spk) {
				throw new Error(`Signed prekey ${wire.x3dh.signed_prekey_id} not found locally`);
			}

			let otpPrivate: Uint8Array | null = null;
			if (wire.x3dh.one_time_prekey_id != null) {
				const otp = await this.storage.getOneTimePrekey(wire.x3dh.one_time_prekey_id);
				if (otp) {
					otpPrivate = otp.privateKey;
					// Consume the OTP (it's single-use)
					await this.storage.deleteOneTimePrekey(wire.x3dh.one_time_prekey_id);
				}
			}

			const result = crypto.x3dh_respond(
				signingKey,
				spk.privateKey,
				otpPrivate,
				new Uint8Array(wire.x3dh.identity_key),
				new Uint8Array(wire.x3dh.ephemeral_key),
			) as {
				session_json: string;
				associated_data: number[];
			};

			sessionJson = result.session_json;

			// Store peer identity (trust on first use, detect key changes)
			await this.storePeerIdentity(peerUserId, new Uint8Array(wire.x3dh.identity_key));
		}

		if (!sessionJson) {
			throw new Error(`No session found for peer ${peerUserId} and no X3DH header`);
		}

		// Build the encrypted message JSON for the WASM decrypt function
		const encryptedMsg = {
			header: wire.header,
			ciphertext: wire.ciphertext,
			nonce: wire.nonce,
		};

		const decResult = crypto.ratchet_decrypt(
			sessionJson,
			JSON.stringify(encryptedMsg),
		) as {
			session_json: string;
			plaintext: number[];
		};

		// Persist updated session
		await this.storage.setSession(peerUserId, decResult.session_json);

		return new TextDecoder().decode(new Uint8Array(decResult.plaintext));
	}

	// Track peers that have already logged a decryption error (avoid console spam)
	private _decryptErrorLogged = new Set<string>();

	/**
	 * Try to decrypt as a WireMessage; fall back to UTF-8 for legacy messages.
	 */
	async decryptOrFallback(
		peerUserId: string | null,
		ciphertextBytes: Uint8Array,
		messageId?: string,
		channelId?: string,
	): Promise<string> {
		// No peer = non-DM channel, just decode as UTF-8
		if (!peerUserId) {
			return new TextDecoder().decode(ciphertextBytes);
		}

		// Check local cache first (for messages we've already decrypted)
		if (messageId) {
			const cached = await this.storage.getDecryptedMessage(messageId);
			if (cached !== null) return cached;
		}

		try {
			const text = new TextDecoder().decode(ciphertextBytes);
			const parsed = JSON.parse(text);

			if (parsed && parsed.v === 1) {
				const plaintext = await this.decryptFromPeer(peerUserId, ciphertextBytes);

				// Cache the decrypted message
				if (messageId && channelId) {
					await this.storage.setDecryptedMessage(messageId, plaintext, channelId);
				}

				return plaintext;
			}
		} catch (err) {
			if (!this._decryptErrorLogged.has(peerUserId)) {
				this._decryptErrorLogged.add(peerUserId);
				console.warn('[E2E] Decryption failed for peer', peerUserId, '(further errors for this peer suppressed):', err);
			}
		}

		// Legacy message: raw UTF-8 plaintext
		return new TextDecoder().decode(ciphertextBytes);
	}

	/** Check if a Double Ratchet session exists for a peer. */
	async hasSession(peerUserId: string): Promise<boolean> {
		const session = await this.storage.getSession(peerUserId);
		return session !== null;
	}

	/** Delete a session (e.g., for re-keying). */
	async deleteSession(peerUserId: string): Promise<void> {
		await this.storage.deleteSession(peerUserId);
	}

	// ─── Sender Keys (Group E2E) ──────────────────────────────────

	/**
	 * Encrypt a plaintext string for a group channel using Sender Keys.
	 * If no sender key exists for this channel, generates one and distributes it.
	 */
	async encryptForGroup(
		channelId: string,
		plaintext: string,
	): Promise<{ ciphertext: number[]; nonce: number[] }> {
		const crypto = await getCrypto();
		const userId = authStore.user?.id;
		if (!userId) throw new Error('Not logged in');

		let stateJson = await this.storage.getSenderKeyState(channelId);

		if (!stateJson) {
			// Generate a new sender key
			const result = crypto.sender_key_generate(
				new TextEncoder().encode(userId),
			) as { state_json: string; distribution_json: string };

			// Upload distribution to server FIRST (broadcasts to other members via WS)
			// Only persist state after successful upload so we don't encrypt with
			// a key that was never distributed to other members.
			const distribution = JSON.parse(result.distribution_json);
			await uploadSenderKey(channelId, distribution.chain_id, distribution);

			stateJson = result.state_json;
			await this.storage.setSenderKeyState(channelId, stateJson);
		}

		const encResult = crypto.sender_key_encrypt(
			stateJson,
			new TextEncoder().encode(plaintext),
		) as { state_json: string; message_json: string };

		// Persist updated state
		await this.storage.setSenderKeyState(channelId, encResult.state_json);

		// Wrap in a SenderKeyWireMessage
		const wireMessage: SenderKeyWireMessage = {
			v: 1,
			sk: true,
			message: JSON.parse(encResult.message_json),
		};

		const wireBytes = new TextEncoder().encode(JSON.stringify(wireMessage));
		return {
			ciphertext: Array.from(wireBytes),
			nonce: wireMessage.message.nonce,
		};
	}

	/**
	 * Decrypt a group message using the sender's receiver key state.
	 */
	async decryptGroupMessage(
		channelId: string,
		senderId: string,
		ciphertextBytes: Uint8Array,
		messageId?: string,
	): Promise<string> {
		// Check decrypted message cache first
		if (messageId) {
			const cached = await this.storage.getDecryptedMessage(messageId);
			if (cached !== null) return cached;
		}

		const text = new TextDecoder().decode(ciphertextBytes);

		try {
			const parsed = JSON.parse(text);
			if (parsed?.v === 1 && parsed?.sk === true) {
				const crypto = await getCrypto();
				const message = parsed.message;

				let receiverStateJson = await this.storage.getReceiverKeyState(channelId, senderId);

				if (!receiverStateJson) {
					// We don't have the sender's key yet -- fetch from server
					const distributions = await getSenderKeys(channelId);
					const dist = distributions.find((d) => d.user_id === senderId);
					if (!dist) {
						throw new Error(`No sender key for ${senderId} in channel ${channelId}`);
					}
					receiverStateJson = crypto.sender_key_from_distribution(
						JSON.stringify(dist.distribution),
					);
				}

				const decResult = crypto.sender_key_decrypt(
					receiverStateJson,
					JSON.stringify(message),
				) as { state_json: string; plaintext: number[] };

				// Persist updated receiver state
				await this.storage.setReceiverKeyState(channelId, senderId, decResult.state_json);

				const plaintext = new TextDecoder().decode(new Uint8Array(decResult.plaintext));

				// Cache decrypted content
				if (messageId) {
					await this.storage.setDecryptedMessage(messageId, plaintext, channelId);
				}

				return plaintext;
			}
		} catch {
			// Not a SenderKeyWireMessage or decryption failed — fall through to UTF-8
		}

		// Legacy message: raw UTF-8 plaintext
		return new TextDecoder().decode(ciphertextBytes);
	}

	/**
	 * Handle sender key rotation: delete our sender key state and all receiver states
	 * for the channel. Next message send will generate a new key.
	 */
	async rotateSenderKeys(channelId: string): Promise<void> {
		await this.storage.deleteSenderKeyState(channelId);
		await this.storage.deleteAllReceiverKeyStatesForChannel(channelId);
	}

	/**
	 * Process a received sender key distribution (from WS SenderKeyUpdated event).
	 */
	async processSenderKeyDistribution(
		channelId: string,
		senderId: string,
		distributionJson: string,
	): Promise<void> {
		const crypto = await getCrypto();
		const receiverStateJson = crypto.sender_key_from_distribution(distributionJson);
		await this.storage.setReceiverKeyState(channelId, senderId, receiverStateJson);
	}
}

/** Compare two Uint8Arrays for equality. */
function arraysEqual(a: Uint8Array, b: Uint8Array): boolean {
	if (a.length !== b.length) return false;
	for (let i = 0; i < a.length; i++) {
		if (a[i] !== b[i]) return false;
	}
	return true;
}

/** Wire format for Sender Key encrypted group messages. */
export interface SenderKeyWireMessage {
	v: 1;
	sk: true;
	message: {
		chain_id: number;
		iteration: number;
		ciphertext: number[];
		nonce: number[];
	};
}
