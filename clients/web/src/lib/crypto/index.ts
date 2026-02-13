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
		_storage = new CryptoStorage();
		await _storage.open();
		_keyManager = new KeyManager(_storage);
		_sessionManager = new SessionManager(_storage, _keyManager);
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
