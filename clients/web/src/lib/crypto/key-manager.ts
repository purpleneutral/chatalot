import { getCrypto } from './wasm-loader';
import type { CryptoStorage } from './storage';
import { getPrekeyCount, registerKeys, uploadOneTimePrekeys } from '$lib/api/keys';

const INITIAL_OTP_COUNT = 100;
const OTP_REPLENISH_THRESHOLD = 25;
const OTP_REPLENISH_BATCH = 100;

/**
 * Bumped when E2E key registration changes require re-registration.
 * v2: Fix stale OTP key mismatch after re-registration.
 */
const KEY_VERSION = 2;

export class KeyManager {
	constructor(private storage: CryptoStorage) {}

	/**
	 * Generate all keys needed for registration.
	 * Stores private keys in IndexedDB, returns public data for the RegisterRequest.
	 */
	async generateRegistrationKeys(): Promise<{
		identityKey: number[];
		signedPrekey: { key_id: number; public_key: number[]; signature: number[] };
		oneTimePrekeys: { key_id: number; public_key: number[] }[];
	}> {
		const crypto = await getCrypto();

		// Generate Ed25519 identity keypair
		const identity = crypto.generate_identity_key() as {
			signing_key: number[];
			verifying_key: number[];
		};
		await this.storage.setIdentity({
			signingKey: new Uint8Array(identity.signing_key),
			verifyingKey: new Uint8Array(identity.verifying_key),
		});

		// Generate signed prekey (key_id = 1)
		const spk = crypto.generate_signed_prekey(
			new Uint8Array(identity.signing_key),
			1,
		) as {
			key_id: number;
			public_key: number[];
			private_key: number[];
			signature: number[];
		};
		await this.storage.setSignedPrekey({
			keyId: spk.key_id,
			publicKey: new Uint8Array(spk.public_key),
			privateKey: new Uint8Array(spk.private_key),
		});

		// Generate initial batch of one-time prekeys
		const otps = crypto.generate_one_time_prekeys(1, INITIAL_OTP_COUNT) as {
			key_id: number;
			public_key: number[];
			private_key: number[];
		}[];
		await this.storage.setOneTimePrekeys(
			otps.map((otp) => ({
				keyId: otp.key_id,
				publicKey: new Uint8Array(otp.public_key),
				privateKey: new Uint8Array(otp.private_key),
			})),
		);

		return {
			identityKey: identity.verifying_key,
			signedPrekey: {
				key_id: spk.key_id,
				public_key: spk.public_key,
				signature: spk.signature,
			},
			oneTimePrekeys: otps.map((otp) => ({
				key_id: otp.key_id,
				public_key: otp.public_key,
			})),
		};
	}

	/**
	 * For users who registered before E2E was active: generate keys,
	 * upload them to the server, and store them locally.
	 * Also re-registers if the key version has changed (e.g., to fix
	 * stale OTPs from a previous key generation).
	 */
	async ensureKeysRegistered(): Promise<void> {
		const existing = await this.storage.getIdentity();
		const storedVersion = await this.storage.getKeyVersion();

		if (existing && storedVersion === KEY_VERSION) return; // Up to date

		if (existing && storedVersion !== KEY_VERSION) {
			console.info(`E2E key version mismatch (have ${storedVersion}, need ${KEY_VERSION}) — re-registering...`);
			// Wipe all crypto state so we start fresh.
			// This also clears stale sessions that used the old keys.
			await this.storage.clear();
		} else {
			console.info('No E2E keys found — generating and registering...');
		}

		const keys = await this.generateRegistrationKeys();
		await registerKeys({
			identity_key: keys.identityKey,
			signed_prekey: keys.signedPrekey,
			one_time_prekeys: keys.oneTimePrekeys,
		});
		await this.storage.setKeyVersion(KEY_VERSION);
		console.info('E2E key registration complete (version', KEY_VERSION, ')');
	}

	/**
	 * Check OTP count on server and replenish if below threshold.
	 */
	async replenishPrekeys(): Promise<void> {
		try {
			const count = await getPrekeyCount();
			if (count >= OTP_REPLENISH_THRESHOLD) return;

			const crypto = await getCrypto();
			const maxKeyId = await this.storage.getMaxOtpKeyId();
			const startId = maxKeyId + 1;

			const otps = crypto.generate_one_time_prekeys(startId, OTP_REPLENISH_BATCH) as {
				key_id: number;
				public_key: number[];
				private_key: number[];
			}[];

			// Store private keys locally
			await this.storage.setOneTimePrekeys(
				otps.map((otp) => ({
					keyId: otp.key_id,
					publicKey: new Uint8Array(otp.public_key),
					privateKey: new Uint8Array(otp.private_key),
				})),
			);

			// Upload public keys to server
			await uploadOneTimePrekeys(
				otps.map((otp) => ({
					key_id: otp.key_id,
					public_key: otp.public_key,
				})),
			);

			console.info(`Replenished ${OTP_REPLENISH_BATCH} one-time prekeys (starting at ${startId})`);
		} catch (err) {
			console.error('Failed to replenish prekeys:', err);
		}
	}

	/** Get our identity signing key from storage. */
	async getSigningKey(): Promise<Uint8Array> {
		const identity = await this.storage.getIdentity();
		if (!identity) throw new Error('No identity key found — not registered?');
		return identity.signingKey;
	}

	/** Get our identity verifying key (public). */
	async getVerifyingKey(): Promise<Uint8Array> {
		const identity = await this.storage.getIdentity();
		if (!identity) throw new Error('No identity key found — not registered?');
		return identity.verifyingKey;
	}
}
