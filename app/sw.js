'use strict';

let preCache = [
	'./',
	'./favicon.png',
	'./main.js',
	'./manifest.json'
];

const CACHE = 'cache-__BUILD_DATE';

self.addEventListener('install', function (evt) {
	evt.waitUntil(caches.open(CACHE).then(function (cache) {
		cache.addAll(preCache);
	}));
});

self.addEventListener('fetch', function (evt) {
	evt.respondWith(fromCache(evt.request).then(function (match) {
		if (match) {
			let path = new URL(evt.request.url).pathname;
			if (path.startsWith('/v2/plan.json')) { evt.waitUntil(update(evt.request).then(refresh)); }
			return match;
		} else {
			return evt.request.method === 'GET' ? fromNetworkAndCache(evt.request) : fromNetwork(evt.request);
		}
	}));
});

self.addEventListener('message', function (evt) {
	let data = JSON.parse(evt.data);
	if (data.type === 'prerender') {
		evt.waitUntil(caches.open(CACHE).then(function (cache) {
			return cache.put('/', new Response(data.content, {
				headers: {
					'Content-Type': 'text/html; charset=UTF-8'
				}
			}));
		}));
	} else if (data.type === 'update') {
		evt.waitUntil(update(evt.request).then(refresh));
	}
});

self.addEventListener('activate', function (event) {
	event.waitUntil(caches.keys().then(function (cacheNames) {
		return Promise.all(cacheNames.filter(c => c !== CACHE).map(c => caches.delete(c)));
	}));
});

function fromNetwork (request) {
	return new Promise(function (resolve, reject) {
		var timeoutId = setTimeout(reject, 500);
		fetch(request).then(function (response) {
			clearTimeout(timeoutId);
			resolve(response);
		}, reject);
	});
}

function fromNetworkAndCache (request) {
	return caches.open(CACHE).then(function (cache) {
		return new Promise(function (resolve, reject) {
			var timeoutId = setTimeout(reject, 500);
			fetch(request).then(function (response) {
				cache.put(request, response.clone());
				clearTimeout(timeoutId);
				resolve(response);
			}, reject);
		});
	});
}

function fromCache (request) {
	return caches.open(CACHE).then(function (cache) {
		return cache.match(request);
	});
}

function update (request) {
	return caches.open(CACHE).then(function (cache) {
		return fetch(request).then(function (response) {
			return cache.put(request, response.clone()).then(function () {
				return response;
			});
		});
	});
}

function refresh (response) {
	return response.text().then(function (text) {
		return self.clients.matchAll().then(function (clients) {
			clients.forEach(function (client) {
				client.postMessage(text);
			});
		});
	});
}
