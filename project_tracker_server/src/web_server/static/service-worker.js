self.addEventListener("install", (event) => {
	event.waitUntil(
		caches.open("pwa-cache").then((cache) => {
			return cache.addAll([
				"/",
				"/static/index.html",
				"/static/style.css",
				"/static/script.js",
				"/static/service-worker.js",
				"/static/bar-chart-fill.svg",
				"/static/caret-down-fill.svg",
				"/static/cart-right-fill.svg",
				"/static/cpu.svg",
				"/static/favicon.ico",
				"/static/file-earmark-text.svg",
				"/static/globe.svg",
				"/static/hourglass-split.svg",
				"/static/house-fill.svg",
				"/static/icon_180x180.png",
				"/static/memory.svg",
				"/static/thermometer-half.svg",
				"/static/window.svg",
				"/static/admin/index.html",
				"/static/admin/script.js",
				"/static/admin/style.css",
				"/static/login/index.html",
				"/static/login/script.js",
				"/static/login/style.css",
			]);
		}),
	);
});

self.addEventListener("fetch", (event) => {
	event.respondWith(
		caches.match(event.request).then((response) => {
			return response || fetch(event.request);
		}),
	);
});
