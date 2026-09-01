// PWA Service Worker 注册 (per 2026-09-01 PHASE-MOBILE-PWA 实装)
"use client";

import { useEffect } from "react";

/**
 * Star PWA Service Worker 注册 hook
 *
 * - 仅在 production 注册 (避免 dev 热更新卡住)
 * - 静默失败: SW 不支持时不影响 web 行为
 * - 提供更新提示: 发现新 SW 时 dispatch 事件给 UI
 *
 * 升级 SW: 改 public/sw.js 的 SW_VERSION 即可。
 */
export function usePwaRegister() {
  useEffect(() => {
    if (typeof window === "undefined") return;
    if (process.env.NODE_ENV !== "production") return;
    if (!("serviceWorker" in navigator)) return;

    const onLoad = () => {
      navigator.serviceWorker
        .register("/sw.js", { scope: "/" })
        .then((reg) => {
          // 检查更新
          reg.update().catch(() => {});
          reg.addEventListener("updatefound", () => {
            const newSw = reg.installing;
            if (!newSw) return;
            newSw.addEventListener("statechange", () => {
              if (newSw.state === "installed" && navigator.serviceWorker.controller) {
                // 新 SW 安装好,提示用户刷新
                window.dispatchEvent(new CustomEvent("star:pwa-updated", { detail: { reg } }));
              }
            });
          });
        })
        .catch(() => {
          // 静默失败: SW 注册失败不应影响 web
        });
    };

    if (document.readyState === "complete") {
      onLoad();
    } else {
      window.addEventListener("load", onLoad, { once: true });
    }
  }, []);
}
