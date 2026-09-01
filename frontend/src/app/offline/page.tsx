// Star 离线 fallback 页面 (per 2026-09-01 PHASE-MOBILE-PWA)
// 当 navigation 网络挂 + cache miss 时由 Service Worker 返回。
"use client";

import Link from "next/link";
import { CloudOff, RefreshCw } from "lucide-react";

export default function OfflinePage() {
  return (
    <div
      data-testid="offline-page"
      className="min-h-[60vh] flex flex-col items-center justify-center px-6 py-12 text-center"
    >
      <div className="size-20 rounded-2xl bg-err/10 border border-err/30 grid place-items-center mb-6">
        <CloudOff size={36} className="text-err" aria-hidden="true" />
      </div>
      <h1 className="text-2xl font-black tracking-tight text-ink mb-2">
        离线模式
      </h1>
      <p className="text-sm text-ink-dim max-w-sm mb-8 leading-relaxed">
        网络连接已断开。请检查 Wi-Fi / 移动数据后重试,或返回首页继续使用上次看过的页面。
      </p>
      <div className="flex flex-col sm:flex-row gap-3">
        <button
          type="button"
          onClick={() => {
            if (typeof window !== "undefined") {
              window.location.reload();
            }
          }}
          className="inline-flex items-center justify-center gap-2 px-5 py-2.5 rounded-xl border border-accent/40 bg-accent/10 text-accent text-sm font-semibold hover:bg-accent/20 transition-colors"
        >
          <RefreshCw size={15} />
          重新加载
        </button>
        <Link
          href="/"
          className="inline-flex items-center justify-center gap-2 px-5 py-2.5 rounded-xl border border-line bg-bg-soft text-ink-dim text-sm font-medium hover:text-ink hover:border-accent/40 transition-colors"
        >
          返回首页
        </Link>
      </div>
      <p className="text-[10px] font-mono text-ink-mute mt-12 tracking-widest uppercase">
        Star PWA · offline · 2026-09-01
      </p>
    </div>
  );
}
