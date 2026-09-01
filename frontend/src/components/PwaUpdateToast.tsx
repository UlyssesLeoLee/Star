// Star PWA 更新提示 Toast (per 2026-09-01 PHASE-MOBILE-PWA v0.3)
//
// 监听 PwaBoot 通过 window.dispatchEvent('star:pwa-updated') 派发的事件,
// 弹一个底部 toast, 让用户感知"新版本已就绪, 立即刷新"。
//
// 注意: Service Worker "新 SW installed 但旧 SW 仍控制页面" 时,
// 只有 SKIP_WAITING 才能让新 SW 接管, SKIP_WAITING 后需 reload。
"use client";

import { useEffect, useState } from "react";
import { RefreshCw, X } from "lucide-react";
import { useRouter } from "next/navigation";

interface UpdateEvent extends CustomEvent {
  detail: { reg: ServiceWorkerRegistration };
}

export function PwaUpdateToast() {
  const router = useRouter();
  const [pendingReg, setPendingReg] = useState<ServiceWorkerRegistration | null>(null);

  useEffect(() => {
    const onUpdated = (e: Event) => {
      const ce = e as UpdateEvent;
      setPendingReg(ce.detail?.reg ?? null);
    };
    window.addEventListener("star:pwa-updated", onUpdated);
    return () => window.removeEventListener("star:pwa-updated", onUpdated);
  }, []);

  if (!pendingReg) return null;

  const apply = () => {
    // 让新 SW 立即激活 + reload
    pendingReg.waiting?.postMessage({ type: "SKIP_WAITING" });
    setTimeout(() => {
      // router.refresh() 不够, 必须 reload 走完整 SW 控制切换
      window.location.reload();
    }, 200);
  };

  const dismiss = () => {
    setPendingReg(null);
    // 下次打开自动应用
  };

  return (
    <div
      data-testid="pwa-update-toast"
      role="status"
      aria-live="polite"
      className="fixed bottom-20 md:bottom-6 left-3 right-3 md:left-auto md:right-6 md:max-w-sm rounded-2xl border border-accent/40 bg-bg-soft/95 backdrop-blur-xl p-3 shadow-[0_0_24px_rgba(0,240,255,0.18)]"
      style={{
        zIndex: 9997,
        paddingBottom: "calc(0.75rem + env(safe-area-inset-bottom))",
      }}
    >
      <div className="flex items-start gap-3">
        <div className="size-8 rounded-lg border border-accent/40 bg-accent/10 grid place-items-center shrink-0">
          <RefreshCw size={15} className="text-accent" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="text-sm font-semibold text-ink">新版本已就绪</div>
          <div className="text-[11px] text-ink-dim mt-0.5 leading-relaxed">
            点击刷新以加载最新功能 (无需重新安装)
          </div>
          <div className="flex items-center gap-2 mt-2.5">
            <button
              type="button"
              onClick={apply}
              data-testid="pwa-update-apply"
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-accent/20 border border-accent/40 text-accent text-xs font-semibold hover:bg-accent/30 transition-colors"
            >
              <RefreshCw size={11} />
              立即刷新
            </button>
            <button
              type="button"
              onClick={dismiss}
              data-testid="pwa-update-dismiss"
              className="px-3 py-1.5 rounded-lg text-xs text-ink-dim hover:text-ink"
            >
              稍后
            </button>
          </div>
        </div>
        <button
          type="button"
          onClick={dismiss}
          aria-label="Dismiss"
          className="p-1 text-ink-mute hover:text-ink shrink-0"
        >
          <X size={13} />
        </button>
      </div>
    </div>
  );
}
