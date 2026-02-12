const SERVER_URL_KEY = 'chatalot_server_url';

export function isTauri(): boolean {
	return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
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
	if (isTauri()) {
		const server = getServerUrl();
		if (!server) return '/api';
		return `${server}/api`;
	}
	return '/api';
}

export function wsUrl(): string {
	if (isTauri()) {
		const server = getServerUrl();
		if (!server) return '';
		const url = new URL(server);
		const protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
		return `${protocol}//${url.host}/ws`;
	}
	const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
	return `${protocol}//${window.location.host}/ws`;
}
