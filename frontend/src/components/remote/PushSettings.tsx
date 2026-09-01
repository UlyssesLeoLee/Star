// Star Push Settings (per 2026-09-01 PHASE-MOBILE-PWA v0.4)
// 放在 /remote 顶部, 集中管理通知权限
"use client";

import { useEffect, useState } from "react";
import { Bell, BellOff, Send, Check } from "lucide-react";
import {
  getPushPermission,
  isPushSubscribed,
  requestPushSubscription,
  unsubscribePush,
  simulateLocalPush,
  type PushPermission,
} from "@/lib/pushClient";

export function PushSettings() {
  const [perm, setPerm] = useState<PushPermission>("default");
  const [subscribed, setSubscribed] = useState(false);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    setPerm(getPushPermission());
    setSubscribed(isPushSubscribed());
  }, []);

  const enable = async () => {
    setBusy(true);
    const ok = await requestPushSubscription();
    setBusy(false);
    if (ok) {
      setPerm("granted");
      setSubscribed(true);
    } else {
      setPerm(getPushPermission());
    }
  };

  const disable = async () => {
    setBusy(true);
    await unsubscribePush();
    setBusy(false);
    setSubscribed(false);
  };

  const testPush = async () => {
    await simulateLocalPush({
      title: "🔔 Star 推送测试",
      body: "这是一条本地模拟推送, 真实生产由 BFF push 端点触发",
      url: "/remote",
      tag: "star-push-test",
    });
  };

  // 状态描述
  const statusText = (() => {
    if (perm === "unsupported") return "浏览器不支持";
    if (perm === "denied") return "权限被拒, 请在浏览器设置中开启";
    if (subscribed) return "已订阅";
    return "未启用";
  })();

  const statusColor = (() => {
    if (perm === "unsupported" || perm === "denied") return "text-err";
    if (subscribed) return "text-ok";
    return "text-ink-dim";
  })();

  return (
    <div
      data-testid="push-settings"
      className="card flex items-center gap-3"
    >
      <div
        className={
          "size-10 rounded-xl border grid place-items-center shrink-0 " +
          (subscribed
            ? "border-ok/40 bg-ok/10"
            : "border-line bg-bg-soft/50")
        }
      >
        {subscribed ? (
          <Bell size={18} className="text-ok" />
        ) : (
          <BellOff size={18} className="text-ink-dim" />
        )}
      </div>
      <div className="flex-1 min-w-0">
        <div className="text-sm font-semibold text-ink flex items-center gap-2">
          推送通知
          <span className={`text-[10px] font-mono ${statusColor}`}>
            · {statusText}
          </span>
        </div>
        <div className="text-[11px] text-ink-dim mt-0.5">
          路上收 Worktree / Agent / Feedback 实时通知
        </div>
      </div>
      <div className="flex items-center gap-1.5">
        {subscribed ? (
          <>
            <button
              type="button"
              onClick={testPush}
              data-testid="push-test"
              className="flex items-center gap-1 px-2.5 py-1.5 rounded-lg border border-line text-[11px] text-ink-dim hover:text-ink"
            >
              <Send size={11} />
              测试
            </button>
            <button
              type="button"
              onClick={disable}
              disabled={busy}
              data-testid="push-disable"
              className="flex items-center gap-1 px-2.5 py-1.5 rounded-lg border border-err/30 text-err text-[11px] hover:bg-err/10"
            >
              <BellOff size={11} />
              关闭
            </button>
          </>
        ) : (
          <button
            type="button"
            onClick={enable}
            disabled={busy || perm === "denied" || perm === "unsupported"}
            data-testid="push-enable"
            className={
              "flex items-center gap-1 px-3 py-1.5 rounded-lg text-[11px] font-semibold transition-colors " +
              (perm === "denied" || perm === "unsupported"
                ? "border border-line text-ink-mute cursor-not-allowed"
                : "border border-accent/40 bg-accent/10 text-accent hover:bg-accent/20")
            }
          >
            {perm === "granted" ? <Check size={11} /> : <Bell size={11} />}
            {busy ? "请求中..." : "启用"}
          </button>
        )}
      </div>
    </div>
  );
}
