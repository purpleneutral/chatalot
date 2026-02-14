const CACHE_NAME = 'chatalot-v2';
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
