import { getVapidKey, subscribePush, unsubscribePush } from '$lib/api/push';

function urlBase64ToUint8Array(base64String: string): Uint8Array {
	const padding = '='.repeat((4 - (base64String.length % 4)) % 4);
	const base64 = (base64String + padding).replace(/-/g, '+').replace(/_/g, '/');
	const raw = atob(base64);
	const arr = new Uint8Array(raw.length);
	for (let i = 0; i < raw.length; i++) arr[i] = raw.charCodeAt(i);
	return arr;
}

function uint8ArrayToBase64Url(arr: Uint8Array): string {
	let binary = '';
	for (const byte of arr) binary += String.fromCharCode(byte);
	return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

class PushStore {
	/** Whether the server supports push (VAPID key configured). */
	supported = $state(false);
	/** Whether this browser currently has an active push subscription. */
	enabled = $state(false);
	/** Loading state for subscribe/unsubscribe operations. */
	loading = $state(false);

	private vapidKey: string | null = null;

	/** Call after auth to check if push is available. */
	async init() {
		if (typeof navigator === 'undefined' || !('serviceWorker' in navigator)) return;
		if (!('PushManager' in window)) return;

		this.vapidKey = await getVapidKey();
		if (!this.vapidKey) {
			this.supported = false;
			return;
		}

		this.supported = true;

		// Check if we already have an active subscription
		try {
			const reg = await navigator.serviceWorker.ready;
			const sub = await reg.pushManager.getSubscription();
			this.enabled = !!sub;
		} catch {
			this.enabled = false;
		}
	}

	async subscribe(): Promise<boolean> {
		if (!this.vapidKey || this.loading) return false;
		this.loading = true;

		try {
			const permission = await Notification.requestPermission();
			if (permission !== 'granted') {
				this.loading = false;
				return false;
			}

			const reg = await navigator.serviceWorker.ready;
			const sub = await reg.pushManager.subscribe({
				userVisibleOnly: true,
				applicationServerKey: urlBase64ToUint8Array(this.vapidKey)
			});

			const p256dh = sub.getKey('p256dh');
			const auth = sub.getKey('auth');
			if (!p256dh || !auth) throw new Error('Missing push keys');

			await subscribePush(
				sub.endpoint,
				uint8ArrayToBase64Url(new Uint8Array(p256dh)),
				uint8ArrayToBase64Url(new Uint8Array(auth))
			);

			this.enabled = true;
			return true;
		} catch (err) {
			console.error('Failed to subscribe to push:', err);
			return false;
		} finally {
			this.loading = false;
		}
	}

	async unsubscribe(): Promise<boolean> {
		if (this.loading) return false;
		this.loading = true;

		try {
			const reg = await navigator.serviceWorker.ready;
			const sub = await reg.pushManager.getSubscription();
			if (sub) {
				await unsubscribePush(sub.endpoint);
				await sub.unsubscribe();
			}
			this.enabled = false;
			return true;
		} catch (err) {
			console.error('Failed to unsubscribe from push:', err);
			return false;
		} finally {
			this.loading = false;
		}
	}
}

export const pushStore = new PushStore();
