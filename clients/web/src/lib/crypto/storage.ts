import { isTauri } from '$lib/env';

const DB_NAME = 'chatalot-crypto';
const DB_VERSION = 2;

// ─── OS Keychain helpers (Tauri desktop only) ────────────────────
// Stores the identity signing key in the OS keychain for stronger
// at-rest protection. Falls back gracefully if keychain is unavailable.

async function keychainStore(name: string, value: string): Promise<boolean> {
	if (!isTauri()) return false;
	try {
		const { invoke } = await import('@tauri-apps/api/core');
		await invoke('store_key', { keyName: name, value });
		return true;
	} catch { return false; }
}

async function keychainGet(name: string): Promise<string | null> {
	if (!isTauri()) return null;
	try {
		const { invoke } = await import('@tauri-apps/api/core');
		return await invoke<string | null>('get_key', { keyName: name });
	} catch { return null; }
}

async function keychainDelete(name: string): Promise<void> {
	if (!isTauri()) return;
	try {
		const { invoke } = await import('@tauri-apps/api/core');
		await invoke('delete_key', { keyName: name });
	} catch { /* best-effort */ }
}

const KEYCHAIN_SIGNING_KEY = 'identity-signing-key';
const KEYCHAIN_VERIFYING_KEY = 'identity-verifying-key';

function uint8ToBase64(bytes: Uint8Array): string {
	let binary = '';
	for (const b of bytes) binary += String.fromCharCode(b);
	return btoa(binary);
}

function base64ToUint8(b64: string): Uint8Array {
	const binary = atob(b64);
	const bytes = new Uint8Array(binary.length);
	for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
	return bytes;
}

export interface IdentityKeys {
	signingKey: Uint8Array;
	verifyingKey: Uint8Array;
}

export interface SignedPrekeyPrivate {
	keyId: number;
	publicKey: Uint8Array;
	privateKey: Uint8Array;
}

export interface OtpPrivate {
	keyId: number;
	publicKey: Uint8Array;
	privateKey: Uint8Array;
}

export interface DecryptedMessageEntry {
	messageId: string;
	content: string;
	channelId: string;
}

export class CryptoStorage {
	private db: IDBDatabase | null = null;

	async open(): Promise<void> {
		if (this.db) return;

		return new Promise((resolve, reject) => {
			const request = indexedDB.open(DB_NAME, DB_VERSION);

			request.onupgradeneeded = () => {
				const db = request.result;
				if (!db.objectStoreNames.contains('identity')) {
					db.createObjectStore('identity');
				}
				if (!db.objectStoreNames.contains('signedPrekeys')) {
					db.createObjectStore('signedPrekeys', { keyPath: 'keyId' });
				}
				if (!db.objectStoreNames.contains('oneTimePrekeys')) {
					db.createObjectStore('oneTimePrekeys', { keyPath: 'keyId' });
				}
				if (!db.objectStoreNames.contains('sessions')) {
					db.createObjectStore('sessions');
				}
				if (!db.objectStoreNames.contains('peerIdentities')) {
					db.createObjectStore('peerIdentities');
				}
				if (!db.objectStoreNames.contains('decryptedMessages')) {
					db.createObjectStore('decryptedMessages');
				}
				// v2: Sender Keys for group E2E
				if (!db.objectStoreNames.contains('senderKeyStates')) {
					db.createObjectStore('senderKeyStates');
				}
				if (!db.objectStoreNames.contains('receiverKeyStates')) {
					db.createObjectStore('receiverKeyStates');
				}
			};

			request.onsuccess = () => {
				this.db = request.result;
				resolve();
			};

			request.onerror = () => reject(request.error);
		});
	}

	// ─── Identity ─────────────────────────────────────────────────

	async getIdentity(): Promise<IdentityKeys | null> {
		// In Tauri, try OS keychain first (stronger at-rest protection)
		const sigB64 = await keychainGet(KEYCHAIN_SIGNING_KEY);
		const verB64 = await keychainGet(KEYCHAIN_VERIFYING_KEY);
		if (sigB64 && verB64) {
			return {
				signingKey: base64ToUint8(sigB64),
				verifyingKey: base64ToUint8(verB64),
			};
		}
		// Fall back to IndexedDB
		const idbKeys: IdentityKeys | null = await this.get('identity', 'self');
		// If found in IDB but not keychain (e.g. first run after upgrade), migrate
		if (idbKeys && isTauri()) {
			await keychainStore(KEYCHAIN_SIGNING_KEY, uint8ToBase64(idbKeys.signingKey));
			await keychainStore(KEYCHAIN_VERIFYING_KEY, uint8ToBase64(idbKeys.verifyingKey));
		}
		return idbKeys;
	}

	async setIdentity(keys: IdentityKeys): Promise<void> {
		// Store in OS keychain if available (Tauri desktop)
		await keychainStore(KEYCHAIN_SIGNING_KEY, uint8ToBase64(keys.signingKey));
		await keychainStore(KEYCHAIN_VERIFYING_KEY, uint8ToBase64(keys.verifyingKey));
		// Always also store in IndexedDB as fallback
		return this.put('identity', 'self', keys);
	}

	// ─── Key Version ──────────────────────────────────────────────

	async getKeyVersion(): Promise<number> {
		const v: number | null = await this.get('identity', 'keyVersion');
		return v ?? 0;
	}

	async setKeyVersion(version: number): Promise<void> {
		return this.put('identity', 'keyVersion', version);
	}

	// ─── Personal Key ─────────────────────────────────────────────

	async getPersonalKey(): Promise<Uint8Array | null> {
		return this.get('identity', 'personalKey');
	}

	async setPersonalKey(key: Uint8Array): Promise<void> {
		return this.put('identity', 'personalKey', key);
	}

	// ─── Signed Prekeys ───────────────────────────────────────────

	async getSignedPrekey(keyId: number): Promise<SignedPrekeyPrivate | null> {
		return this.get('signedPrekeys', keyId);
	}

	async setSignedPrekey(prekey: SignedPrekeyPrivate): Promise<void> {
		return this.put('signedPrekeys', prekey.keyId, prekey);
	}

	async getLatestSignedPrekey(): Promise<SignedPrekeyPrivate | null> {
		return new Promise((resolve, reject) => {
			const tx = this.db!.transaction('signedPrekeys', 'readonly');
			const store = tx.objectStore('signedPrekeys');
			const request = store.openCursor(null, 'prev');
			request.onsuccess = () => {
				const cursor = request.result;
				resolve(cursor ? cursor.value : null);
			};
			request.onerror = () => reject(request.error);
		});
	}

	// ─── One-Time Prekeys ─────────────────────────────────────────

	async getOneTimePrekey(keyId: number): Promise<OtpPrivate | null> {
		return this.get('oneTimePrekeys', keyId);
	}

	async setOneTimePrekeys(prekeys: OtpPrivate[]): Promise<void> {
		return new Promise((resolve, reject) => {
			const tx = this.db!.transaction('oneTimePrekeys', 'readwrite');
			const store = tx.objectStore('oneTimePrekeys');
			for (const pk of prekeys) {
				store.put(pk);
			}
			tx.oncomplete = () => resolve();
			tx.onerror = () => reject(tx.error);
		});
	}

	async deleteOneTimePrekey(keyId: number): Promise<void> {
		return this.delete('oneTimePrekeys', keyId);
	}

	async getMaxOtpKeyId(): Promise<number> {
		return new Promise((resolve, reject) => {
			const tx = this.db!.transaction('oneTimePrekeys', 'readonly');
			const store = tx.objectStore('oneTimePrekeys');
			const request = store.openCursor(null, 'prev');
			request.onsuccess = () => {
				const cursor = request.result;
				resolve(cursor ? (cursor.value as OtpPrivate).keyId : 0);
			};
			request.onerror = () => reject(request.error);
		});
	}

	// ─── Sessions ─────────────────────────────────────────────────

	async getSession(peerUserId: string): Promise<string | null> {
		return this.get('sessions', peerUserId);
	}

	async setSession(peerUserId: string, sessionJson: string): Promise<void> {
		return this.put('sessions', peerUserId, sessionJson);
	}

	async deleteSession(peerUserId: string): Promise<void> {
		return this.delete('sessions', peerUserId);
	}

	// ─── Peer Identities ──────────────────────────────────────────

	async getPeerIdentity(peerUserId: string): Promise<Uint8Array | null> {
		return this.get('peerIdentities', peerUserId);
	}

	async setPeerIdentity(peerUserId: string, identityKey: Uint8Array): Promise<void> {
		return this.put('peerIdentities', peerUserId, identityKey);
	}

	// ─── Decrypted Message Cache ──────────────────────────────────

	async getDecryptedMessage(messageId: string): Promise<string | null> {
		const entry: DecryptedMessageEntry | null = await this.get('decryptedMessages', messageId);
		return entry?.content ?? null;
	}

	async setDecryptedMessage(messageId: string, content: string, channelId: string): Promise<void> {
		return this.put('decryptedMessages', messageId, { messageId, content, channelId });
	}

	// ─── Sender Key States (our keys, one per channel) ──────────

	async getSenderKeyState(channelId: string): Promise<string | null> {
		return this.get('senderKeyStates', channelId);
	}

	async setSenderKeyState(channelId: string, stateJson: string): Promise<void> {
		return this.put('senderKeyStates', channelId, stateJson);
	}

	async deleteSenderKeyState(channelId: string): Promise<void> {
		return this.delete('senderKeyStates', channelId);
	}

	// ─── Receiver Key States (other members' keys) ───────────────

	async getReceiverKeyState(channelId: string, senderId: string): Promise<string | null> {
		return this.get('receiverKeyStates', `${channelId}:${senderId}`);
	}

	async setReceiverKeyState(channelId: string, senderId: string, stateJson: string): Promise<void> {
		return this.put('receiverKeyStates', `${channelId}:${senderId}`, stateJson);
	}

	async deleteAllReceiverKeyStatesForChannel(channelId: string): Promise<void> {
		return new Promise((resolve, reject) => {
			const tx = this.db!.transaction('receiverKeyStates', 'readwrite');
			const store = tx.objectStore('receiverKeyStates');
			const request = store.openCursor();
			request.onsuccess = () => {
				const cursor = request.result;
				if (cursor) {
					if (typeof cursor.key === 'string' && cursor.key.startsWith(`${channelId}:`)) {
						cursor.delete();
					}
					cursor.continue();
				}
			};
			tx.oncomplete = () => resolve();
			tx.onerror = () => reject(tx.error);
		});
	}

	// ─── Wipe ─────────────────────────────────────────────────────

	async clear(): Promise<void> {
		// Clear OS keychain identity keys
		await keychainDelete(KEYCHAIN_SIGNING_KEY);
		await keychainDelete(KEYCHAIN_VERIFYING_KEY);

		if (!this.db) return;
		const storeNames = Array.from(this.db.objectStoreNames);
		return new Promise((resolve, reject) => {
			const tx = this.db!.transaction(storeNames, 'readwrite');
			for (const name of storeNames) {
				tx.objectStore(name).clear();
			}
			tx.oncomplete = () => resolve();
			tx.onerror = () => reject(tx.error);
		});
	}

	// ─── Generic helpers ──────────────────────────────────────────

	private get<T>(storeName: string, key: IDBValidKey): Promise<T | null> {
		return new Promise((resolve, reject) => {
			const tx = this.db!.transaction(storeName, 'readonly');
			const request = tx.objectStore(storeName).get(key);
			request.onsuccess = () => resolve(request.result ?? null);
			request.onerror = () => reject(request.error);
		});
	}

	private put(storeName: string, key: IDBValidKey, value: unknown): Promise<void> {
		return new Promise((resolve, reject) => {
			const tx = this.db!.transaction(storeName, 'readwrite');
			const store = tx.objectStore(storeName);
			// Stores with a keyPath derive the key from the value — passing
			// an explicit key throws DataError per the IndexedDB spec.
			if (store.keyPath != null) {
				store.put(value);
			} else {
				store.put(value, key);
			}
			tx.oncomplete = () => resolve();
			tx.onerror = () => reject(tx.error);
		});
	}

	private delete(storeName: string, key: IDBValidKey): Promise<void> {
		return new Promise((resolve, reject) => {
			const tx = this.db!.transaction(storeName, 'readwrite');
			tx.objectStore(storeName).delete(key);
			tx.oncomplete = () => resolve();
			tx.onerror = () => reject(tx.error);
		});
	}
}
