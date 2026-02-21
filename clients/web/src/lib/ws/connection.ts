import { authStore } from '$lib/stores/auth.svelte';
import { refreshToken } from '$lib/api/auth';
import { wsUrl } from '$lib/env';
import type { ClientMessage, ServerMessage } from './types';

declare const __APP_VERSION__: string;

type MessageHandler = (msg: ServerMessage) => void | Promise<void>;

// Message types that should be queued when offline (user-initiated actions)
const QUEUEABLE_TYPES = new Set([
	'send_message', 'edit_message', 'delete_message',
	'add_reaction', 'remove_reaction', 'mark_read', 'mark_all_read',
	'typing', 'stop_typing'
]);

const HEARTBEAT_INTERVAL_MS = 15_000;
const ZOMBIE_TIMEOUT_MS = 45_000; // 3 missed pongs → zombie
const MAX_RECONNECT_DELAY_MS = 30_000;
const MAX_OFFLINE_QUEUE_SIZE = 50;

class WebSocketClient {
	private ws: WebSocket | null = null;
	private reconnectAttempts = 0;
	private heartbeatTimer: ReturnType<typeof setInterval> | null = null;
	private zombieTimer: ReturnType<typeof setInterval> | null = null;
	private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
	private handlers: Set<MessageHandler> = new Set();
	private authenticatedCallback: (() => void) | null = null;
	private connected = false;
	private _reconnecting = false;
	private offlineQueue: ClientMessage[] = [];
	private lastPongTime = 0;

	get isConnected(): boolean {
		return this.connected;
	}

	get isReconnecting(): boolean {
		return this._reconnecting;
	}

	onMessage(handler: MessageHandler): () => void {
		this.handlers.add(handler);
		return () => this.handlers.delete(handler);
	}

	onAuthenticated(cb: () => void) {
		this.authenticatedCallback = cb;
	}

	connect() {
		const state = this.ws?.readyState;
		if (state === WebSocket.OPEN || state === WebSocket.CONNECTING || state === WebSocket.CLOSING) return;

		const url = wsUrl();
		if (!url) return;

		this.ws = new WebSocket(url);

		this.ws.onopen = () => {
			this.reconnectAttempts = 0;
			// Authenticate with JWT
			const token = authStore.accessToken;
			if (token) {
				this.send({ type: 'authenticate', token });
			}
		};

		this.ws.onmessage = (event) => {
			try {
				const msg = JSON.parse(event.data) as ServerMessage;
				this.dispatch(msg);
			} catch {
				console.error('Failed to parse WebSocket message');
			}
		};

		this.ws.onclose = () => {
			const wasConnected = this.connected;
			this.connected = false;
			this.stopHeartbeat();
			if (authStore.isAuthenticated) {
				this._reconnecting = true;
				if (wasConnected) {
					window.dispatchEvent(new CustomEvent('chatalot:connection', { detail: 'reconnecting' }));
				}
				this.scheduleReconnect();
			}
		};

		this.ws.onerror = () => {
			// onclose will fire after this
		};
	}

	disconnect() {
		this.stopHeartbeat();
		if (this.reconnectTimer) {
			clearTimeout(this.reconnectTimer);
			this.reconnectTimer = null;
		}
		this._reconnecting = false;
		this.offlineQueue.length = 0;
		this.ws?.close();
		this.ws = null;
		this.connected = false;
	}

	send(msg: ClientMessage): boolean {
		if (this.ws?.readyState === WebSocket.OPEN) {
			this.ws.send(JSON.stringify(msg));
			return true;
		}
		// Queue user-initiated messages for delivery on reconnect
		if (QUEUEABLE_TYPES.has(msg.type) && this.offlineQueue.length < MAX_OFFLINE_QUEUE_SIZE) {
			this.offlineQueue.push(msg);
		}
		return false;
	}

	private flushQueue() {
		if (this.offlineQueue.length === 0) return;
		const queued = this.offlineQueue.splice(0);
		for (const msg of queued) {
			// Drop stale typing indicators (only useful in real-time)
			if (msg.type === 'typing' || msg.type === 'stop_typing') continue;
			this.send(msg);
		}
	}

	private dispatch(msg: ServerMessage) {
		if (msg.type === 'pong') {
			this.lastPongTime = Date.now();
		}

		// Token rejected — refresh before the next reconnect attempt
		if (msg.type === 'error' && msg.code === 'unauthorized') {
			const rt = authStore.refreshToken;
			if (rt) {
				refreshToken(rt)
					.then((data) => {
						authStore.setTokens(data.access_token, data.refresh_token);
						console.info('Token refreshed after WS auth failure');
					})
					.catch(() => {
						// Refresh also failed — force re-login
						authStore.logout();
						window.location.href = '/login';
					});
			} else {
				authStore.logout();
				window.location.href = '/login';
			}
			return; // Don't dispatch to handlers (suppresses toast)
		}

		if (msg.type === 'authenticated') {
			const wasReconnecting = this._reconnecting;
			this.connected = true;
			this._reconnecting = false;
			this.startHeartbeat();
			this.authenticatedCallback?.();

			// Deliver any messages queued while offline
			this.flushQueue();

			if (wasReconnecting) {
				window.dispatchEvent(new CustomEvent('chatalot:connection', { detail: 'connected' }));
			}

			// Notify if the server was updated with a new client build.
			// Only suppress in native Tauri (top-level window with bundled assets).
			// In the iframe shell, the web app loads from the server, so reload works.
			if (
				msg.server_version &&
				msg.server_version !== 'unknown' &&
				msg.server_version !== __APP_VERSION__
			) {
				console.info(
					`Version mismatch: client=${__APP_VERSION__}, server=${msg.server_version}`,
				);
				const isNativeTauri = window.parent === window && '__TAURI_INTERNALS__' in window;
				// DEBUG: show what's happening in the title bar
				document.title = `UPDATE: ${__APP_VERSION__}→${msg.server_version} native=${isNativeTauri} parent=${window.parent === window}`;
				if (!isNativeTauri) {
					window.dispatchEvent(new CustomEvent('chatalot:update-available'));
				}
			}
		}

		for (const handler of this.handlers) {
			try {
				const result = handler(msg);
				if (result && typeof result === 'object' && 'catch' in result) {
					(result as Promise<void>).catch((err) =>
						console.error('Async handler error:', err),
					);
				}
			} catch (err) {
				console.error('Handler error:', err);
			}
		}
	}

	private startHeartbeat() {
		this.stopHeartbeat();
		this.lastPongTime = Date.now();
		this.heartbeatTimer = setInterval(() => {
			this.send({ type: 'ping', timestamp: Date.now() });
		}, HEARTBEAT_INTERVAL_MS);
		// Detect zombie connections (TCP open but no data flowing)
		this.zombieTimer = setInterval(() => {
			if (this.connected && Date.now() - this.lastPongTime > ZOMBIE_TIMEOUT_MS) {
				console.warn('Zombie connection detected — no pong in', ZOMBIE_TIMEOUT_MS, 'ms, forcing reconnect');
				this.ws?.close();
			}
		}, HEARTBEAT_INTERVAL_MS);
	}

	private stopHeartbeat() {
		if (this.heartbeatTimer) {
			clearInterval(this.heartbeatTimer);
			this.heartbeatTimer = null;
		}
		if (this.zombieTimer) {
			clearInterval(this.zombieTimer);
			this.zombieTimer = null;
		}
	}

	private scheduleReconnect() {
		if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
		const delay = Math.min(
			1000 * Math.pow(2, this.reconnectAttempts) + Math.random() * 1000,
			MAX_RECONNECT_DELAY_MS
		);
		this.reconnectAttempts++;
		this.reconnectTimer = setTimeout(() => {
			this.reconnectTimer = null;
			this.connect();
		}, delay);
	}
}

export const wsClient = new WebSocketClient();
