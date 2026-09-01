// Star Web Push 客户端 (per 2026-09-01 PHASE-MOBILE-PWA v0.4)
//
// 真实推送: 后端 VAPID + push 端点 + pushManager.subscribe, server 端存 subscription
// MVP 阶段: 用 Notification API + SW push event 本地模拟, 数据存 localStorage

const PUSH_ENABLED_KEY = "star:push-enabled";
const VAPID_PUBLIC_KEY = process.env.NEXT_PUBLIC_VAPID_PUBLIC_KEY || "";

/** Notification.permission 状态 */
export type PushPermission = "default" | "granted" | "denied" | "unsupported";

export function getPushPermission(): PushPermission {
  if (typeof window === "undefined") return "unsupported";
  if (!("Notification" in window)) return "unsupported";
  return Notification.permission as PushPermission;
}

export function isPushSubscribed(): boolean {
  if (typeof window === "undefined") return false;
  return window.localStorage.getItem(PUSH_ENABLED_KEY) === "1";
}

/**
 * 请求通知权限 + 注册 push subscription
 *
 * - 浏览器不支持 Notification API → 直接 return false
 * - 权限 denied → return false
 * - 权限 granted → 尝试 pushManager.subscribe(VAPID), 失败降级 (只用 Notification 模拟)
 */
export async function requestPushSubscription(): Promise<boolean> {
  if (typeof window === "undefined") return false;
  if (!("Notification" in window)) return false;

  if (Notification.permission === "denied") return false;

  const permission = Notification.permission === "granted"
    ? "granted"
    : await Notification.requestPermission();
  if (permission !== "granted") return false;

  // 尝试 pushManager.subscribe (需后端 VAPID public key)
  if ("serviceWorker" in navigator && VAPID_PUBLIC_KEY) {
    try {
      const reg = await navigator.serviceWorker.ready;
      let sub = await reg.pushManager.getSubscription();
      if (!sub) {
        sub = await reg.pushManager.subscribe({
          userVisibleOnly: true,
          applicationServerKey: urlBase64ToUint8Array(VAPID_PUBLIC_KEY),
        });
      }
      // 真实场景: POST 到 /v1/push/subscribe (等 BFF 端点)
      // MVP: 存 localStorage
      window.localStorage.setItem("star:push-subscription", JSON.stringify(sub.toJSON()));
    } catch (e) {
      // push 不可用 (如 Firefox 私有模式), 降级到只本地通知
      console.warn("pushManager.subscribe failed, fallback to local:", e);
    }
  }

  window.localStorage.setItem(PUSH_ENABLED_KEY, "1");
  return true;
}

/** 取消订阅 */
export async function unsubscribePush(): Promise<boolean> {
  if (typeof window === "undefined") return false;
  if ("serviceWorker" in navigator) {
    try {
      const reg = await navigator.serviceWorker.ready;
      const sub = await reg.pushManager.getSubscription();
      if (sub) {
        await sub.unsubscribe();
        // 真实: POST /v1/push/unsubscribe
      }
    } catch {
      // ignore
    }
  }
  window.localStorage.removeItem(PUSH_ENABLED_KEY);
  window.localStorage.removeItem("star:push-subscription");
  return true;
}

/**
 * 模拟"推送测试" — 走 SW push event (不依赖后端 VAPID)
 * 真实生产: 由后端 push 端点触发
 */
export async function simulateLocalPush(payload: {
  title: string;
  body: string;
  url?: string;
  tag?: string;
}): Promise<void> {
  if (typeof window === "undefined") return;
  if (!("serviceWorker" in navigator)) return;

  const reg = await navigator.serviceWorker.ready;
  // 通过 postMessage 让 SW 显示通知 (模拟 push event)
  reg.active?.postMessage({
    type: "SIMULATE_PUSH",
    payload: {
      title: payload.title,
      body: payload.body,
      icon: "/icon-192.png",
      data: { url: payload.url || "/" },
      tag: payload.tag || "star-test",
    },
  });
}

// VAPID 公钥转 Uint8Array (Push API 需求)
function urlBase64ToUint8Array(base64String: string): Uint8Array {
  const padding = "=".repeat((4 - (base64String.length % 4)) % 4);
  const base64 = (base64String + padding).replace(/-/g, "+").replace(/_/g, "/");
  const rawData = atob(base64);
  const output = new Uint8Array(rawData.length);
  for (let i = 0; i < rawData.length; ++i) {
    output[i] = rawData.charCodeAt(i);
  }
  return output;
}
