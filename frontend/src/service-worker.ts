/// <reference lib="webworker" />

import { build, files, version } from "$service-worker";

declare const self: ServiceWorkerGlobalScope;

const PRECACHE = `precache-${version}`;
const OFFLINE_FALLBACK = "index.html";

const PRECACHE_ASSETS = Array.from(
	new Set([...build, ...files, OFFLINE_FALLBACK]),
);

const PRECACHE_PATHS = new Set(PRECACHE_ASSETS.map(normalizePath));

// ── Install: cache everything upfront, skip waiting immediately ──────────────
self.addEventListener("install", (event) => {
	event.waitUntil(
		(async () => {
			const cache = await caches.open(PRECACHE);
			await cache.addAll(PRECACHE_ASSETS);
			await self.skipWaiting();
		})(),
	);
});

// ── Activate: purge old caches, claim clients, notify them to reload ─────────
self.addEventListener("activate", (event) => {
	event.waitUntil(
		(async () => {
			const keys = await caches.keys();
			await Promise.all(
				keys
					.filter((key) => key !== PRECACHE)
					.map((key) => caches.delete(key)),
			);
			await self.clients.claim();

			// Tell all open clients that a new version is ready.
			const allClients = await self.clients.matchAll({ type: "window" });
			for (const client of allClients) {
				client.postMessage({ type: "SW_UPDATED", version });
			}
		})(),
	);
});

// ── Fetch: fully offline-first (cache-first for everything) ──────────────────
self.addEventListener("fetch", (event) => {
	const request = event.request;

	if (request.method !== "GET") return;

	const url = new URL(request.url);

	// Only handle same-origin HTTP(S) requests.
	if (url.origin !== self.location.origin) return;
	if (url.protocol !== "http:" && url.protocol !== "https:") return;

	event.respondWith(offlineFirst(request, url));
});

async function offlineFirst(request: Request, url: URL): Promise<Response> {
	// 1. Precached asset → serve immediately from cache.
	if (PRECACHE_PATHS.has(normalizePath(url.pathname))) {
		const cached = await caches.match(request);
		if (cached) return cached;
	}

	// 2. Navigation or any other same-origin request → try cache first.
	const cached = await caches.match(request);
	if (cached) return cached;

	// 3. Nothing in cache yet (first visit before SW was active, or
	//    a resource added after install) → fetch from network and cache it.
	try {
		const response = await fetch(request);
		if (response.ok) {
			const cache = await caches.open(PRECACHE);
			cache.put(request, response.clone());
		}
		return response;
	} catch {
		// 4. Offline and no cache → return offline fallback for navigations.
		if (request.mode === "navigate") {
			const fallback =
				(await caches.match(OFFLINE_FALLBACK)) ??
				(await caches.match("/"));
			if (fallback) return fallback;
		}

		return new Response("Offline", {
			status: 503,
			statusText: "Service Unavailable",
			headers: { "content-type": "text/plain; charset=utf-8" },
		});
	}
}

function normalizePath(path: string): string {
	if (!path) return "/";
	let normalized = path.replace(/\/+/g, "/");
	if (!normalized.startsWith("/")) normalized = `/${normalized}`;
	return normalized;
}

export {};
