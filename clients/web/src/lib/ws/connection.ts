import { authStore } from '$lib/stores/auth.svelte';
import { wsUrl } from '$lib/env';
import type { ClientMessage, ServerMessage } from './types';

declare const __APP_VERSION__: string;

type MessageHandler = (msg: ServerMessage) => void | Promise<void>;

class WebSocketClient {
	private ws: WebSocket | null = null;
	private reconnectAttempts = 0;
	private maxReconnectDelay = 30000;
	private heartbeatTimer: ReturnType<typeof setInterval> | null = null;
	private handlers: Set<MessageHandler> = new Set();
	private authenticatedCallback: (() => void) | null = null;
	private connected = false;
	private _reconnecting = false;

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
		if (this.ws?.readyState === WebSocket.OPEN) return;

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
		this._reconnecting = false;
		this.ws?.close();
		this.ws = null;
		this.connected = false;
	}

	send(msg: ClientMessage): boolean {
		if (this.ws?.readyState === WebSocket.OPEN) {
			this.ws.send(JSON.stringify(msg));
			return true;
		}
		return false;
	}

	private dispatch(msg: ServerMessage) {
		if (msg.type === 'authenticated') {
			const wasReconnecting = this._reconnecting;
			this.connected = true;
			this._reconnecting = false;
			this.startHeartbeat();
			this.authenticatedCallback?.();

			if (wasReconnecting) {
				window.dispatchEvent(new CustomEvent('chatalot:connection', { detail: 'connected' }));
			}

			// Auto-reload if the server was updated with a new client build
			if (
				msg.server_version &&
				msg.server_version !== 'unknown' &&
				msg.server_version !== __APP_VERSION__
			) {
				console.info(
					`Version mismatch: client=${__APP_VERSION__}, server=${msg.server_version}`,
				);
				window.dispatchEvent(new CustomEvent('chatalot:update-available'));
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
		this.heartbeatTimer = setInterval(() => {
			this.send({ type: 'ping', timestamp: Date.now() });
		}, 30000);
	}

	private stopHeartbeat() {
		if (this.heartbeatTimer) {
			clearInterval(this.heartbeatTimer);
			this.heartbeatTimer = null;
		}
	}

	private scheduleReconnect() {
		const delay = Math.min(
			1000 * Math.pow(2, this.reconnectAttempts) + Math.random() * 1000,
			this.maxReconnectDelay
		);
		this.reconnectAttempts++;
		setTimeout(() => this.connect(), delay);
	}
}

export const wsClient = new WebSocketClient();
