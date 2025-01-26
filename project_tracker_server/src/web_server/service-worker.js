self.addEventListener("install", (event) => {
	event.waitUntil(
		caches.open("pwa-cache").then((cache) => {
			return cache.addAll([
				"/",
				"/index.html",
				"/style.css",
				"/script.js",
				"/service-worker.js",
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
				"/static/apple-touch-icon.png",
				"/static/memory.svg",
				"/static/thermometer-half.svg",
				"/static/window.svg",
				"/admin/index.html",
				"/admin/script.js",
				"/admin/style.css",
				"/login/index.html",
				"/login/script.js",
				"/login/style.css",
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
