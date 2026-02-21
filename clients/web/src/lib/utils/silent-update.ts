import { voiceStore } from '$lib/stores/voice.svelte';

const IDLE_THRESHOLD_MS = 5_000;
const FALLBACK_TIMEOUT_MS = 120_000;
const DRAFT_KEY = 'chatalot:silent-reload-draft';
const RELOAD_FLAG = 'chatalot:silent-reload';

let lastTypingTime = 0;

/** Call from message input handler to track when the user last typed. */
export function markTyping(): void {
	lastTypingTime = Date.now();
}

/** Trigger the silent update flow: precache → wait for idle → reload. */
export function startSilentUpdate(): void {
	if (!navigator.serviceWorker?.controller) {
		// No SW (e.g. Tauri webview, HTTP) — skip precache, just wait for idle and reload
		waitForIdleThenReload();
		return;
	}

	// Ask SW to precache new assets
	navigator.serviceWorker.controller.postMessage({ type: 'precache-update' });

	// Listen for SW response
	const onMessage = (event: MessageEvent) => {
		if (event.data?.type !== 'update-ready') return;
		navigator.serviceWorker.removeEventListener('message', onMessage);
		waitForIdleThenReload();
	};
	navigator.serviceWorker.addEventListener('message', onMessage);
}

function waitForIdleThenReload(): void {
	const started = Date.now();

	const interval = setInterval(() => {
		const idle = Date.now() - lastTypingTime > IDLE_THRESHOLD_MS;
		const inCall = voiceStore.isInCall;

		if (idle && !inCall) {
			clearInterval(interval);
			saveDraftAndReload();
			return;
		}

		// Fallback: if never idle after 2 minutes, show toast
		if (Date.now() - started > FALLBACK_TIMEOUT_MS) {
			clearInterval(interval);
			window.dispatchEvent(new CustomEvent('chatalot:update-show-toast'));
		}
	}, 1_000);
}

function saveDraftAndReload(): void {
	// Save any in-progress draft from the message textarea
	const textarea = document.querySelector<HTMLTextAreaElement>('[data-message-input]');
	if (textarea?.value) {
		sessionStorage.setItem(DRAFT_KEY, textarea.value);
	}
	sessionStorage.setItem(RELOAD_FLAG, '1');
	location.reload();
}

/** Call in onMount to restore state after a silent reload. */
export function restoreAfterSilentReload(): void {
	if (!sessionStorage.getItem(RELOAD_FLAG)) return;
	sessionStorage.removeItem(RELOAD_FLAG);

	const draft = sessionStorage.getItem(DRAFT_KEY);
	if (draft) {
		sessionStorage.removeItem(DRAFT_KEY);
		// Retry until the textarea mounts (channel data loads async after onMount)
		let attempts = 0;
		const tryRestore = () => {
			const textarea = document.querySelector<HTMLTextAreaElement>('[data-message-input]');
			if (textarea) {
				textarea.value = draft;
				textarea.dispatchEvent(new Event('input', { bubbles: true }));
				return;
			}
			if (++attempts < 50) setTimeout(tryRestore, 100); // up to 5s
		};
		tryRestore();
	}
}
