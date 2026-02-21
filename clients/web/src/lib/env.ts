import { isTauriIframe } from '$lib/utils/tauri-bridge';

const SERVER_URL_KEY = 'chatalot_server_url';

export function isTauri(): boolean {
	return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/** True if running in direct Tauri mode OR inside Tauri's iframe shell. */
export function isTauriEnv(): boolean {
	return isTauri() || isTauriIframe();
}

/** True if inside Tauri but navigated directly to the server origin.
 *  In this mode, relative URLs work (same as regular web) and the page
 *  can reload to pick up new assets (unlike bundled Tauri mode). */
export function isTauriDirectNav(): boolean {
	return isTauri() && typeof window !== 'undefined' && window.location.protocol.startsWith('http');
}

export function getServerUrl(): string | null {
	return localStorage.getItem(SERVER_URL_KEY);
}

export function setServerUrl(url: string) {
	localStorage.setItem(SERVER_URL_KEY, url);
}

export function clearServerUrl() {
	localStorage.removeItem(SERVER_URL_KEY);
}

export function apiBase(): string {
	if (isTauri() && !isTauriDirectNav()) {
		const server = getServerUrl();
		if (server) return `${server}/api`;
	}
	// Direct nav, iframe, or regular web: relative URL (same origin)
	return '/api';
}

export function wsUrl(): string {
	if (isTauri()) {
		const server = getServerUrl();
		if (server) {
			const url = new URL(server);
			const protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
			return `${protocol}//${url.host}/ws`;
		}
		// Direct navigation mode: already on the server origin, fall through
	}
	// In iframe mode, regular web, or direct navigation: use current host
	const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
	return `${protocol}//${window.location.host}/ws`;
}
