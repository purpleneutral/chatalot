const CACHE_NAME = 'chatalot-v10';
const STATIC_ASSETS = [
	'/',
	'/manifest.json',
	'/icon-192.png',
	'/icon-512.png'
];

self.addEventListener('install', (event) => {
	event.waitUntil(
		caches.open(CACHE_NAME).then((cache) => cache.addAll(STATIC_ASSETS))
	);
	self.skipWaiting();
});

self.addEventListener('activate', (event) => {
	event.waitUntil(
		caches.keys().then((keys) =>
			Promise.all(keys.filter((k) => k !== CACHE_NAME).map((k) => caches.delete(k)))
		)
	);
	self.clients.claim();
});

self.addEventListener('fetch', (event) => {
	const url = new URL(event.request.url);

	// Only handle requests from our own origin
	if (url.origin !== self.location.origin) return;

	// Don't cache API or WebSocket requests
	if (url.pathname.startsWith('/api') || url.pathname === '/ws') return;

	// Don't cache Vite's hashed chunks — browser HTTP cache handles these
	// via content-hash filenames. SW caching them causes stale code after deploys.
	if (url.pathname.startsWith('/_app/')) return;

	event.respondWith(
		fetch(event.request)
			.then((response) => {
				if (event.request.method === 'GET' && response.status === 200) {
					const clone = response.clone();
					caches.open(CACHE_NAME).then((cache) => cache.put(event.request, clone));
				}
				return response;
			})
			.catch(() => {
				return caches.match(event.request).then((cached) => {
					return cached || caches.match('/');
				});
			})
	);
});

// ── Precache Update (silent reload) ──

self.addEventListener('message', (event) => {
	if (event.data?.type !== 'precache-update') return;

	event.waitUntil(
		(async () => {
			try {
				// Fetch fresh index.html
				const response = await fetch('/', { cache: 'no-cache' });
				if (!response.ok) throw new Error(`fetch / returned ${response.status}`);

				const html = await response.text();

				// Parse immutable asset URLs from the HTML
				const urls = [];
				const pattern = /(?:src|href)=["']((\/_app\/immutable\/[^"']+))["']/g;
				let match;
				while ((match = pattern.exec(html)) !== null) {
					urls.push(match[1]);
				}

				// Prefetch all immutable assets
				await Promise.all(
					urls.map((url) =>
						fetch(url, { cache: 'no-cache' }).catch(() => {
							/* best-effort */
						})
					)
				);

				// Update cached index.html
				const cache = await caches.open(CACHE_NAME);
				await cache.put('/', new Response(html, {
					headers: response.headers
				}));
			} catch (err) {
				console.warn('[SW] Precache update error:', err);
			}

			// Signal all clients that update is ready (even on error — reload still works)
			const clients = await self.clients.matchAll({ type: 'window' });
			for (const client of clients) {
				client.postMessage({ type: 'update-ready' });
			}
		})()
	);
});

// ── Web Push ──

self.addEventListener('push', (event) => {
	if (!event.data) return;

	let data;
	try {
		data = event.data.json();
	} catch {
		return;
	}

	const title = data.sender_name || 'Chatalot';
	const body = data.notification_type === 'dm'
		? 'Sent you a message'
		: 'New message';

	event.waitUntil(
		self.registration.showNotification(title, {
			body,
			icon: '/icon-192.png',
			badge: '/icon-192.png',
			tag: data.channel_id || 'chatalot',
			data: { channelId: data.channel_id }
		})
	);
});

self.addEventListener('notificationclick', (event) => {
	event.notification.close();

	const channelId = event.notification.data?.channelId;
	const url = channelId ? `/channels?id=${channelId}` : '/channels';

	event.waitUntil(
		self.clients.matchAll({ type: 'window', includeUncontrolled: true }).then((clients) => {
			// Focus an existing window if one is open
			for (const client of clients) {
				if (new URL(client.url).origin === self.location.origin) {
					client.focus();
					client.postMessage({
						type: 'navigate-channel',
						channelId
					});
					return;
				}
			}
			// Otherwise open a new window
			return self.clients.openWindow(url);
		})
	);
});
