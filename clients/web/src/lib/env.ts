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

/** crypto.randomUUID() polyfill — WebKitGTK with self-signed certs
 *  may not expose it (requires secure context). */
export function randomUUID(): string {
	if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
		return crypto.randomUUID();
	}
	// Fallback using crypto.getRandomValues (available in all contexts)
	const bytes = new Uint8Array(16);
	crypto.getRandomValues(bytes);
	bytes[6] = (bytes[6] & 0x0f) | 0x40; // version 4
	bytes[8] = (bytes[8] & 0x3f) | 0x80; // variant 1
	const hex = [...bytes].map(b => b.toString(16).padStart(2, '0')).join('');
	return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
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
