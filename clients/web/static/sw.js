const CACHE_NAME = 'chatalot-v1';
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

	// Don't cache API or WebSocket requests
	if (url.pathname.startsWith('/api') || url.pathname === '/ws') {
		return;
	}

	event.respondWith(
		fetch(event.request)
			.then((response) => {
				// Cache successful GET responses
				if (event.request.method === 'GET' && response.status === 200) {
					const clone = response.clone();
					caches.open(CACHE_NAME).then((cache) => cache.put(event.request, clone));
				}
				return response;
			})
			.catch(() => {
				// Serve from cache when offline
				return caches.match(event.request).then((cached) => {
					return cached || caches.match('/');
				});
			})
	);
});
