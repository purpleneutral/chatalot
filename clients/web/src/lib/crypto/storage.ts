const DB_NAME = 'chatalot-crypto';
const DB_VERSION = 1;

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
		return this.get('identity', 'self');
	}

	async setIdentity(keys: IdentityKeys): Promise<void> {
		return this.put('identity', 'self', keys);
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

	// ─── Wipe ─────────────────────────────────────────────────────

	async clear(): Promise<void> {
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
			tx.objectStore(storeName).put(value, key);
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
