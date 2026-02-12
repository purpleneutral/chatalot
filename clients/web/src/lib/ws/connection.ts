import { authStore } from '$lib/stores/auth.svelte';
import { wsUrl } from '$lib/env';
import type { ClientMessage, ServerMessage } from './types';

type MessageHandler = (msg: ServerMessage) => void;

class WebSocketClient {
	private ws: WebSocket | null = null;
	private reconnectAttempts = 0;
	private maxReconnectDelay = 30000;
	private heartbeatTimer: ReturnType<typeof setInterval> | null = null;
	private handlers: Set<MessageHandler> = new Set();
	private authenticatedCallback: (() => void) | null = null;
	private connected = false;

	get isConnected(): boolean {
		return this.connected;
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
			this.connected = false;
			this.stopHeartbeat();
			if (authStore.isAuthenticated) {
				this.scheduleReconnect();
			}
		};

		this.ws.onerror = () => {
			// onclose will fire after this
		};
	}

	disconnect() {
		this.stopHeartbeat();
		this.ws?.close();
		this.ws = null;
		this.connected = false;
	}

	send(msg: ClientMessage) {
		if (this.ws?.readyState === WebSocket.OPEN) {
			this.ws.send(JSON.stringify(msg));
		}
	}

	private dispatch(msg: ServerMessage) {
		if (msg.type === 'authenticated') {
			this.connected = true;
			this.startHeartbeat();
			this.authenticatedCallback?.();
		}

		for (const handler of this.handlers) {
			handler(msg);
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
