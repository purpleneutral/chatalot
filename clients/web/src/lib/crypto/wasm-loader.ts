import type * as WasmModule from './wasm/chatalot_crypto_wasm';

let wasmModule: typeof WasmModule | null = null;
let initPromise: Promise<typeof WasmModule> | null = null;

/** Lazy-load and initialize the WASM crypto module. Cached after first call. */
export async function getCrypto(): Promise<typeof WasmModule> {
	if (wasmModule) return wasmModule;
	if (initPromise) return initPromise;

	initPromise = (async () => {
		const mod = await import('./wasm/chatalot_crypto_wasm');
		await mod.default();
		wasmModule = mod;
		return mod;
	})();

	return initPromise;
}
