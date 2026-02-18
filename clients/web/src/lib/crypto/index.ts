import { CryptoStorage } from './storage';
import { KeyManager } from './key-manager';
import { SessionManager } from './session-manager';

let _storage: CryptoStorage | null = null;
let _keyManager: KeyManager | null = null;
let _sessionManager: SessionManager | null = null;
let _initPromise: Promise<void> | null = null;

/** Initialize the crypto subsystem (IndexedDB + WASM). Safe to call multiple times. */
export async function initCrypto(): Promise<void> {
	if (_storage && _keyManager && _sessionManager) return;
	if (_initPromise) return _initPromise;

	_initPromise = (async () => {
		try {
			_storage = new CryptoStorage();
			await _storage.open();
			_keyManager = new KeyManager(_storage);
			_sessionManager = new SessionManager(_storage, _keyManager);
		} catch (err) {
			// Reset so future calls can retry instead of returning a stale rejected promise
			_storage = null;
			_keyManager = null;
			_sessionManager = null;
			_initPromise = null;
			throw err;
		}
	})();

	return _initPromise;
}

export function getKeyManager(): KeyManager {
	if (!_keyManager) throw new Error('Crypto not initialized — call initCrypto() first');
	return _keyManager;
}

export function getSessionManager(): SessionManager {
	if (!_sessionManager) throw new Error('Crypto not initialized — call initCrypto() first');
	return _sessionManager;
}

export function getCryptoStorage(): CryptoStorage {
	if (!_storage) throw new Error('Crypto not initialized — call initCrypto() first');
	return _storage;
}

/**
 * Derive personal key from password + userId and store in IndexedDB.
 * Called at login time when we still have the plaintext password.
 */
export async function storePersonalKey(password: string, userId: string): Promise<void> {
	const { derivePersonalKey } = await import('./personal-key');
	const storage = new CryptoStorage();
	await storage.open();
	const key = await derivePersonalKey(password, userId);
	await storage.setPersonalKey(key);
}

/** Get the personal key from IndexedDB (null if not yet derived, e.g. haven't logged in since feature was added). */
export async function getPersonalKey(): Promise<Uint8Array | null> {
	if (!_storage) return null;
	return _storage.getPersonalKey();
}

/** Wipe all crypto state from IndexedDB. Call on logout. */
export async function wipeCrypto(): Promise<void> {
	if (_storage) {
		try {
			await _storage.clear();
		} catch {
			// Best effort — DB might already be closed
		}
	}
	_storage = null;
	_keyManager = null;
	_sessionManager = null;
	_initPromise = null;
}
