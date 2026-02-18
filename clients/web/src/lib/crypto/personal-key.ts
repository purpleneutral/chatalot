/**
 * Personal encryption key derived from the user's password.
 * Used to encrypt data the user wants accessible on any device
 * (e.g. scheduled message previews) without the server being able to read it.
 *
 * Flow:
 * 1. At login, derive AES-256-GCM key from PBKDF2(password, userId)
 * 2. Store the raw key bytes in IndexedDB
 * 3. Use for encrypt/decrypt of personal data
 */

const PBKDF2_ITERATIONS = 100_000;
const KEY_USAGE: KeyUsage[] = ['encrypt', 'decrypt'];

/** Derive an AES-256-GCM key from password + userId using PBKDF2. */
export async function derivePersonalKey(password: string, userId: string): Promise<Uint8Array> {
	const enc = new TextEncoder();
	const keyMaterial = await crypto.subtle.importKey(
		'raw',
		enc.encode(password),
		'PBKDF2',
		false,
		['deriveKey']
	);
	const salt = enc.encode(`chatalot-personal-${userId}`);
	const aesKey = await crypto.subtle.deriveKey(
		{ name: 'PBKDF2', salt, iterations: PBKDF2_ITERATIONS, hash: 'SHA-256' },
		keyMaterial,
		{ name: 'AES-GCM', length: 256 },
		true, // extractable so we can export raw bytes for storage
		KEY_USAGE
	);
	const raw = await crypto.subtle.exportKey('raw', aesKey);
	return new Uint8Array(raw);
}

/** Encrypt plaintext with a personal key. Returns base64(iv + ciphertext). */
export async function personalEncrypt(keyBytes: Uint8Array, plaintext: string): Promise<string> {
	const key = await crypto.subtle.importKey('raw', keyBytes, 'AES-GCM', false, ['encrypt']);
	const iv = crypto.getRandomValues(new Uint8Array(12));
	const enc = new TextEncoder();
	const ciphertext = await crypto.subtle.encrypt(
		{ name: 'AES-GCM', iv },
		key,
		enc.encode(plaintext)
	);
	// Concatenate iv + ciphertext and base64-encode
	const combined = new Uint8Array(iv.length + ciphertext.byteLength);
	combined.set(iv);
	combined.set(new Uint8Array(ciphertext), iv.length);
	return btoa(String.fromCharCode(...combined));
}

/** Decrypt base64(iv + ciphertext) with a personal key. Returns plaintext or null on failure. */
export async function personalDecrypt(keyBytes: Uint8Array, encoded: string): Promise<string | null> {
	try {
		const key = await crypto.subtle.importKey('raw', keyBytes, 'AES-GCM', false, ['decrypt']);
		const combined = Uint8Array.from(atob(encoded), c => c.charCodeAt(0));
		const iv = combined.slice(0, 12);
		const ciphertext = combined.slice(12);
		const plaintext = await crypto.subtle.decrypt(
			{ name: 'AES-GCM', iv },
			key,
			ciphertext
		);
		return new TextDecoder().decode(plaintext);
	} catch {
		return null;
	}
}
