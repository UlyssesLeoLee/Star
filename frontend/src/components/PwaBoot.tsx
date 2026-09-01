// PWA 引导组件: 注册 Service Worker + 监听更新事件 (per 2026-09-01 PHASE-MOBILE-PWA)
"use client";

import { usePwaRegister } from "@/lib/pwaRegister";

export function PwaBoot() {
  usePwaRegister();
  return null;
}
