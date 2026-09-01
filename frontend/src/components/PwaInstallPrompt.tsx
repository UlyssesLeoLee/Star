// Star PWA Install Prompt (per 2026-09-01 PHASE-MOBILE-PWA v0.3)
//
// 浏览器原生: 监听 beforeinstallprompt (Chrome/Edge/Samsung), 阻止默认 banner,
// 自己弹一个更友好的底部 prompt 卡片。
//
// iOS Safari: 不支持 beforeinstallprompt, 仅检测 standalone 模式判断是否已安装,
// 未安装时显示"如何加到主屏"说明 modal (iOS 走 Safari 分享 → 添加到主屏)。
"use client";

import { useEffect, useState } from "react";
import { Download, Share, Plus, X, Smartphone } from "lucide-react";
import { clsx } from "clsx";

type State =
  | { kind: "hidden" }
  | { kind: "native"; deferred: BeforeInstallPromptEvent }
  | { kind: "ios-instructions" };

interface BeforeInstallPromptEvent extends Event {
  prompt(): Promise<void>;
  userChoice: Promise<{ outcome: "accepted" | "dismissed" }>;
}

const STORAGE_KEY = "star:pwa-install-dismissed";

export function PwaInstallPrompt() {
  const [state, setState] = useState<State>({ kind: "hidden" });

  useEffect(() => {
    // 已安装 (standalone mode) 不显示
    if (typeof window === "undefined") return;
    const isStandalone =
      window.matchMedia?.("(display-mode: standalone)").matches ||
      // iOS 旧属性
      (navigator as { standalone?: boolean }).standalone === true;
    if (isStandalone) return;

    // 用户已 dismiss 24h 内不再弹
    const lastDismissed = window.localStorage.getItem(STORAGE_KEY);
    if (lastDismissed) {
      const ts = Number(lastDismissed);
      if (Number.isFinite(ts) && Date.now() - ts < 24 * 60 * 60 * 1000) {
        return;
      }
    }

    // 检测 iOS (Safari 不支持 beforeinstallprompt)
    const isIOS = /iPad|iPhone|iPod/.test(navigator.userAgent) && !("MSStream" in window);

    const onBeforeInstall = (e: Event) => {
      e.preventDefault();
      setState({ kind: "native", deferred: e as BeforeInstallPromptEvent });
    };

    window.addEventListener("beforeinstallprompt", onBeforeInstall);

    // iOS: 延迟 3s 弹, 避免页面加载时即弹
    if (isIOS) {
      const t = setTimeout(() => setState({ kind: "ios-instructions" }), 3000);
      return () => {
        clearTimeout(t);
        window.removeEventListener("beforeinstallprompt", onBeforeInstall);
      };
    }

    return () => {
      window.removeEventListener("beforeinstallprompt", onBeforeInstall);
    };
  }, []);

  const dismiss = (permanent = false) => {
    if (permanent) {
      window.localStorage.setItem(STORAGE_KEY, String(Date.now()));
    }
    setState({ kind: "hidden" });
  };

  if (state.kind === "hidden") return null;

  // 通用底部 banner 容器样式
  return (
    <div
      data-testid="pwa-install-prompt"
      role="dialog"
      aria-label="Install Star App"
      className={clsx(
        "fixed z-40 rounded-2xl border border-accent/40 bg-bg-soft/95 backdrop-blur-xl shadow-[0_0_24px_rgba(0,240,255,0.18)]",
        // 移动端: 底部 16, 让出 bottom nav; 桌面: 右下角 24
        "bottom-20 left-3 right-3 md:bottom-6 md:left-auto md:right-6 md:max-w-sm",
      )}
      style={{ paddingBottom: "calc(0.75rem + env(safe-area-inset-bottom))" }}
    >
      <div className="p-3">
        {state.kind === "native" && (
          <>
            <div className="flex items-start gap-3">
              <div className="size-10 rounded-xl overflow-hidden border border-accent/40 shrink-0">
                <img
                  src="/icon-192.png"
                  alt=""
                  width={40}
                  height={40}
                  className="w-full h-full object-cover"
                />
              </div>
              <div className="flex-1 min-w-0">
                <div className="text-sm font-semibold text-ink">安装 Star App</div>
                <div className="text-[11px] text-ink-dim mt-0.5 leading-relaxed">
                  添加到主屏, 离线可用, 推送通知
                </div>
                <div className="flex items-center gap-2 mt-2.5">
                  <button
                    type="button"
                    onClick={async () => {
                      await state.deferred.prompt();
                      const choice = await state.deferred.userChoice;
                      if (choice.outcome === "accepted") {
                        dismiss(true);
                      } else {
                        dismiss();
                      }
                    }}
                    data-testid="pwa-install-accept"
                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-accent/20 border border-accent/40 text-accent text-xs font-semibold hover:bg-accent/30"
                  >
                    <Download size={11} />
                    安装
                  </button>
                  <button
                    type="button"
                    onClick={() => dismiss(true)}
                    data-testid="pwa-install-dismiss"
                    className="px-3 py-1.5 rounded-lg text-xs text-ink-dim hover:text-ink"
                  >
                    暂不
                  </button>
                </div>
              </div>
              <button
                type="button"
                onClick={() => dismiss()}
                aria-label="Dismiss"
                className="p-1 text-ink-mute hover:text-ink shrink-0"
              >
                <X size={13} />
              </button>
            </div>
          </>
        )}

        {state.kind === "ios-instructions" && (
          <>
            <div className="flex items-start gap-3 mb-2.5">
              <div className="size-10 rounded-xl overflow-hidden border border-accent/40 shrink-0">
                <img
                  src="/icon-192.png"
                  alt=""
                  width={40}
                  height={40}
                  className="w-full h-full object-cover"
                />
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-1.5 text-sm font-semibold text-ink">
                  <Smartphone size={13} className="text-accent" />
                  安装到主屏
                </div>
                <div className="text-[11px] text-ink-dim mt-0.5 leading-relaxed">
                  iOS 需手动添加, 3 步搞定
                </div>
            </div>
              <button
                type="button"
                onClick={() => dismiss(true)}
                aria-label="Dismiss"
                className="p-1 text-ink-mute hover:text-ink shrink-0"
              >
                <X size={13} />
              </button>
            </div>
            <ol className="text-[11px] text-ink-dim space-y-1.5 pl-1">
              <li className="flex items-start gap-2">
                <span className="font-mono text-accent shrink-0">1.</span>
                <span className="flex items-center gap-1.5">
                  点击底部分享按钮
                  <Share size={12} className="text-accent" />
                </span>
              </li>
              <li className="flex items-start gap-2">
                <span className="font-mono text-accent shrink-0">2.</span>
                <span className="flex items-center gap-1.5">
                  下滑找"添加到主屏幕"
                  <Plus size={11} className="text-accent" />
                </span>
              </li>
              <li className="flex items-start gap-2">
                <span className="font-mono text-accent shrink-0">3.</span>
                点击右上"添加"即可
              </li>
            </ol>
            <button
              type="button"
              onClick={() => dismiss(true)}
              data-testid="pwa-install-ios-dismiss"
              className="w-full mt-3 py-1.5 rounded-lg border border-line text-[11px] text-ink-dim hover:text-ink"
            >
              我知道了
            </button>
          </>
        )}
      </div>
    </div>
  );
}
