const CACHE = 'loom-v1';
self.addEventListener('install', (event) => {
  event.waitUntil(caches.open(CACHE).then((c) => c.addAll(['/','/search','/tags','/estate/tokens.css'])));
  self.skipWaiting();
});
self.addEventListener('activate', (event) => {
  event.waitUntil(caches.keys().then((keys) => Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k)))));
  self.clients.claim();
});
self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);
  if (event.request.method !== 'GET' || url.pathname.startsWith('/api/')) return;
  if (url.pathname.startsWith('/b/')) {
    event.respondWith(caches.open(CACHE).then(async (cache) => {
      const keys = await cache.keys();
      const bundlePages = keys.filter((r) => new URL(r.url).pathname.startsWith('/b/'));
      if (bundlePages.length >= 20) await cache.delete(bundlePages[0]);
      const response = await fetch(event.request);
      if (response.ok) await cache.put(event.request, response.clone());
      return response;
    }));
  }
});
