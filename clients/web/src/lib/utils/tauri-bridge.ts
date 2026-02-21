const BRIDGE_TIMEOUT_MS = 10_000;

let messageId = 0;
const pendingRequests = new Map<number, { resolve: (v: unknown) => void; reject: (e: Error) => void }>();
let listenerAttached = false;

/** Detect if we're running inside the Tauri shell iframe. */
export function isTauriIframe(): boolean {
	if (typeof window === 'undefined') return false;
	if (window.parent === window) return false;
	if ('__TAURI_INTERNALS__' in window) return false;

	// Check for shell responses as a runtime marker — but optimistically
	// detect based on the iframe-in-parent pattern. The shell will validate
	// origin on its side.
	return true;
}

function ensureListener(): void {
	if (listenerAttached) return;
	listenerAttached = true;

	window.addEventListener('message', (event) => {
		const msg = event.data;
		if (!msg || typeof msg !== 'object' || msg.source !== 'chatalot-shell') return;

		const pending = pendingRequests.get(msg.id);
		if (!pending) return;
		pendingRequests.delete(msg.id);

		if (msg.error) {
			pending.reject(new Error(msg.error));
		} else {
			pending.resolve(msg.result);
		}
	});
}

/** Send a command to the Tauri shell via postMessage bridge. */
export function bridgeInvoke(action: string, payload?: Record<string, unknown>): Promise<unknown> {
	ensureListener();

	const id = ++messageId;

	return new Promise((resolve, reject) => {
		pendingRequests.set(id, { resolve, reject });

		const timeout = setTimeout(() => {
			pendingRequests.delete(id);
			reject(new Error(`Bridge timeout for action: ${action}`));
		}, BRIDGE_TIMEOUT_MS);

		// Clear timeout when resolved
		const origResolve = resolve;
		const origReject = reject;
		pendingRequests.set(id, {
			resolve: (v) => { clearTimeout(timeout); origResolve(v); },
			reject: (e) => { clearTimeout(timeout); origReject(e); }
		});

		window.parent.postMessage({
			source: 'chatalot-bridge',
			id,
			action,
			payload: payload ?? {}
		}, '*');
	});
}
