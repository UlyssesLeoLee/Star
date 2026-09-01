/* Star PWA Service Worker (MVP v1) — 2026-09-01
 *
 * 策略:
 *   navigation (HTML):     network-first → /offline fallback
 *   /worktree /work-item  stale-while-revalidate (offline read 重点)
 *   /agent /feedback /notification /projects stale-while-revalidate
 *   静态资源 (icon / _next/static): cache-first
 *   API 调用: network-only (不缓存,业务数据由前端 store 负责)
 *
 * 升级: 发新版本时改 SW_VERSION 即可,旧 SW 会在下次 activate 替换。
 */

const SW_VERSION = "star-pwa-v1";
const STATIC_CACHE = `${SW_VERSION}-static`;
const RUNTIME_CACHE = `${SW_VERSION}-runtime`;
const OFFLINE_URL = "/offline";

// 关键页面: 用户期望"上次看过"的页面,网络挂时立即返 cache
const CORE_ROUTES = [
  "/worktree",
  "/work-item",
  "/agent",
  "/feedback",
  "/notification",
  "/projects",
];

// 安装时 precache 关键资源
self.addEventListener("install", (event) => {
  event.waitUntil(
    (async () => {
      const cache = await caches.open(STATIC_CACHE);
      // 单个失败不阻塞整体安装
      await Promise.allSettled([
        cache.addAll(["/", OFFLINE_URL, "/manifest.json", "/icon-192.png", "/icon-512.png", "/apple-touch-icon.png", "/favicon.ico"]),
      ]);
      // 立即接管页面,不等待旧 SW 终止
      await self.skipWaiting();
    })(),
  );
});

// activate: 清理旧版本 cache
self.addEventListener("activate", (event) => {
  event.waitUntil(
    (async () => {
      const keys = await caches.keys();
      await Promise.all(
        keys
          .filter((k) => !k.startsWith(SW_VERSION))
          .map((k) => caches.delete(k)),
      );
      await self.clients.claim();
    })(),
  );
});

// fetch: 三类策略分发
self.addEventListener("fetch", (event) => {
  const req = event.request;

  // 只处理 GET
  if (req.method !== "GET") return;

  const url = new URL(req.url);

  // 同源 only
  if (url.origin !== self.location.origin) return;

  // API 调用: 不缓存,直接穿透 (业务数据由前端 store 负责)
  if (url.pathname.startsWith("/api/") || url.pathname.startsWith("/v1/")) {
    return; // 默认 network
  }

  // 1) navigation: network-first → /offline fallback
  if (req.mode === "navigate") {
    event.respondWith(networkFirstNav(req));
    return;
  }

  // 2) core routes 内的静态资源请求(非 navigation,例如 prefetch)
  //    暂不特殊处理,走默认 network

  // 3) _next/static 静态资产: cache-first
  if (url.pathname.startsWith("/_next/static/") || url.pathname.startsWith("/static/")) {
    event.respondWith(cacheFirstStatic(req));
    return;
  }

  // 4) icon / favicon / manifest: cache-first
  if (/\.(png|ico|svg|webp|woff2?|css|js)$/.test(url.pathname)) {
    event.respondWith(cacheFirstStatic(req));
    return;
  }
});

async function networkFirstNav(req) {
  try {
    const fresh = await fetch(req);
    // 同步缓存当前 HTML (offline fallback)
    const cache = await caches.open(RUNTIME_CACHE);
    cache.put(req, fresh.clone()).catch(() => {});
    return fresh;
  } catch (e) {
    // 网络挂: 走 cache,最后 fallback /offline
    const cache = await caches.open(RUNTIME_CACHE);
    const cached = await cache.match(req);
    if (cached) return cached;
    const offline = await caches.match(OFFLINE_URL);
    if (offline) return offline;
    return new Response("Offline", { status: 503, statusText: "Offline" });
  }
}

async function cacheFirstStatic(req) {
  const cache = await caches.open(STATIC_CACHE);
  const cached = await cache.match(req);
  if (cached) return cached;
  try {
    const fresh = await fetch(req);
    if (fresh.ok) cache.put(req, fresh.clone()).catch(() => {});
    return fresh;
  } catch (e) {
    return cached || new Response("Static fetch failed", { status: 504 });
  }
}

// 接收客户端消息: SKIP_WAITING (强制激活新 SW) + SIMULATE_PUSH (本地推送测试)
self.addEventListener("message", (event) => {
  if (!event.data) return;
  if (event.data.type === "SKIP_WAITING") {
    self.skipWaiting();
    return;
  }
  if (event.data.type === "SIMULATE_PUSH" && event.data.payload) {
    // 模拟收到 push 事件
    const p = event.data.payload;
    self.registration.showNotification(p.title || "Star", {
      body: p.body,
      icon: p.icon || "/icon-192.png",
      badge: "/icon-192.png",
      tag: p.tag || "star-test",
      data: p.data || {},
    });
  }
});

// =====================================================================
// Push 事件 (per 2026-09-01 PHASE-MOBILE-PWA v0.4)
// 真实推送需后端 VAPID + push 端点, MVP 阶段供"推送测试"按钮触发
// =====================================================================
self.addEventListener("push", (event) => {
  if (!event.data) return;

  let payload;
  try {
    payload = event.data.json();
  } catch {
    payload = { title: "Star", body: event.data.text() };
  }

  const title = payload.title || "Star";
  const options = {
    body: payload.body || "",
    icon: payload.icon || "/icon-192.png",
    badge: "/icon-192.png",
    data: payload.data || {},
    tag: payload.tag || "star-default",
    requireInteraction: payload.requireInteraction || false,
    actions: payload.actions || [
      { action: "open", title: "查看" },
      { action: "dismiss", title: "稍后" },
    ],
  };

  event.waitUntil(self.registration.showNotification(title, options));
});

// 用户点击通知 → 打开或聚焦 app
self.addEventListener("notificationclick", (event) => {
  event.notification.close();
  const targetUrl = event.notification.data?.url || "/";
  event.waitUntil(
    (async () => {
      const allClients = await self.clients.matchAll({ type: "window", includeUncontrolled: true });
      for (const client of allClients) {
        const url = new URL(client.url);
        if (url.pathname === targetUrl && "focus" in client) {
          return client.focus();
        }
      }
      return self.clients.openWindow(targetUrl);
    })(),
  );
});
