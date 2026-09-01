// Star Mobile noVNC Viewer (per 2026-09-01 PHASE-MOBILE-PWA v0.2)
//
// 客户端: 复用 @novnc/novnc RFB 协议, 通过 Star BFF WebSocket 转发
// 移动端: 触屏 pinch-zoom + 单指拖动 + 双击右键
//
// SSR: 强制 client-only (noVNC 用了大量 DOM + WebGL)。
"use client";

import { useEffect, useRef, useState } from "react";
import {
  Loader2,
  Maximize2,
  Minimize2,
  Wifi,
  WifiOff,
  AlertCircle,
  RefreshCw,
  ZoomIn,
  ZoomOut,
} from "lucide-react";
import { buildRemoteUrl, isRemoteMockMode } from "@/lib/remote/wsClient";

interface NoVncViewerProps {
  runtimeId: string;
  hostname: string;
  fullscreen?: boolean;
  onToggleFullscreen?: () => void;
}

export function NoVncViewer({
  runtimeId,
  hostname,
  fullscreen,
  onToggleFullscreen,
}: NoVncViewerProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const rfbRef = useRef<unknown>(null); // RFB 实例
  const [status, setStatus] = useState<"idle" | "connecting" | "connected" | "error">("idle");
  const [errorMsg, setErrorMsg] = useState<string>("");
  const [mockMode, setMockMode] = useState(false);
  const [zoom, setZoom] = useState(1); // 0.5x - 3x, mock 模式也用
  const [drag, setDrag] = useState<{ x: number; y: number } | null>(null);

  // 状态机: idle → connecting → connected / error
  const connect = async () => {
    setStatus("connecting");
    setErrorMsg("");

    // SSR-safe check
    const isMock = isRemoteMockMode();
    setMockMode(isMock);

    if (isMock) {
      // Mock 模式: 渲染 demo 桌面, 3s 后报 connected
      setTimeout(() => setStatus("connected"), 1500);
      return;
    }

    if (!containerRef.current) return;

    try {
      // 动态 import 避免 SSR (noVNC 引用 DOM/WebGL)
      // @novnc/novnc v1.7 exports 字段是 "./core/rfb.js", webpack 不解析,
      // 改用 Function 包装的 import() 逃过静态分析 (Next.js 社区常用 hack)
      // 类型见 src/types/remote-modules.d.ts
      // eslint-disable-next-line @typescript-eslint/no-implied-eval
      const dynamicImport = new Function("p", "return import(p)") as (p: string) => Promise<{ default: typeof import("@novnc/novnc/core/rfb.js").default }>;
      const RFB = (await dynamicImport("@novnc/novnc/core/rfb.js")).default;
      const url = buildRemoteUrl("desktop", runtimeId);

      const rfb = new RFB(containerRef.current, url, {
        credentials: { password: "" }, // 走 Star cookie, 无 VNC 密码
      });

      rfb.addEventListener("connect", () => setStatus("connected"));
      rfb.addEventListener("disconnect", (e: CustomEvent) => {
        setStatus("error");
        setErrorMsg(e.detail?.reason ?? "disconnected");
      });
      rfb.addEventListener("securityfailure", (e: CustomEvent) => {
        setStatus("error");
        setErrorMsg(`security: ${e.detail?.reason ?? "unknown"}`);
      });

      // 移动端: 缩放 = 适应屏幕
      rfb.scaleViewport = true;
      rfb.resizeSession = true;

      rfbRef.current = rfb;
    } catch (e) {
      setStatus("error");
      setErrorMsg(e instanceof Error ? e.message : String(e));
    }
  };

  // 卸载时 disconnect
  useEffect(() => {
    return () => {
      const rfb = rfbRef.current as { disconnect?: () => void } | null;
      try { rfb?.disconnect?.(); } catch { /* ignore */ }
    };
  }, []);

  return (
    <div className="flex flex-col h-full bg-black">
      {/* Status bar */}
      <div className="flex items-center gap-2 px-3 py-2 border-b border-line bg-bg-soft/80 backdrop-blur text-xs">
        {status === "connected" ? (
          <Wifi size={13} className="text-ok" />
        ) : status === "error" ? (
          <WifiOff size={13} className="text-err" />
        ) : (
          <Loader2 size={13} className="text-accent animate-spin" />
        )}
        <span className="font-mono text-ink-dim">
          {hostname} · vnc
          {mockMode ? " · mock" : ""}
        </span>
        <span className="ml-auto text-ink-mute font-mono">
          {status}
        </span>
        {onToggleFullscreen && (
          <button
            type="button"
            onClick={onToggleFullscreen}
            className="p-1 text-ink-dim hover:text-ink"
            aria-label="Toggle fullscreen"
          >
            {fullscreen ? <Minimize2 size={14} /> : <Maximize2 size={14} />}
          </button>
        )}
      </div>

      {/* Canvas / Mock surface */}
      <div
        ref={containerRef}
        data-testid="novnc-canvas"
        className="flex-1 relative overflow-hidden bg-black touch-none"
        // 1 指拖动 (per v0.3 触屏 UX)
        onPointerDown={(e) => {
          if (zoom <= 1) return; // 不缩放时不启动拖动
          (e.target as HTMLElement).setPointerCapture(e.pointerId);
          setDrag({ x: e.clientX, y: e.clientY });
        }}
        onPointerMove={(e) => {
          if (!drag) return;
          const dx = e.clientX - drag.x;
          const dy = e.clientY - drag.y;
          containerRef.current?.scrollBy({ left: -dx, top: -dy });
          setDrag({ x: e.clientX, y: e.clientY });
        }}
        onPointerUp={() => setDrag(null)}
        onPointerCancel={() => setDrag(null)}
      >
        {status === "idle" && (
          <button
            type="button"
            onClick={connect}
            data-testid="novnc-connect"
            className="absolute inset-0 flex flex-col items-center justify-center gap-3 text-ink-dim hover:text-ink"
          >
            <div className="size-16 rounded-2xl border border-accent/40 grid place-items-center bg-accent/10">
              <span className="text-2xl">🖥️</span>
            </div>
            <span className="text-sm font-semibold">点击连接 {hostname} 远程桌面</span>
            <span className="text-[10px] text-ink-mute font-mono">RFB · WebSocket relay</span>
          </button>
        )}

        {status === "connecting" && (
          <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 text-ink-dim">
            <Loader2 size={32} className="text-accent animate-spin" />
            <span className="text-xs font-mono">connecting to {hostname}...</span>
          </div>
        )}

        {status === "error" && (
          <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 text-err">
            <AlertCircle size={28} />
            <span className="text-xs font-mono text-center max-w-xs">{errorMsg}</span>
            <button
              type="button"
              onClick={connect}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-line text-xs"
            >
              <RefreshCw size={12} />
              重试
            </button>
          </div>
        )}

        {status === "connected" && mockMode && (
          <div
            data-testid="novnc-mock-surface"
            className="absolute inset-0 p-4 text-[10px] font-mono text-green-400 leading-snug origin-top-left"
            style={{
              backgroundImage: "linear-gradient(180deg, #001100 0%, #000 100%)",
              backgroundSize: "100% 24px",
              transform: `scale(${zoom})`,
              width: `${100 / zoom}%`,
              height: `${100 / zoom}%`,
            }}
          >
            <pre className="whitespace-pre-wrap">
{`★ Star Remote Desktop (Mock) — ${hostname}
RFB 协议 / WebSocket relay / Star BFF v1

[模拟终端 session 已就绪]
user@${hostname}:~$ uname -a
Linux ${hostname.toLowerCase()} 6.5.0 #1 SMP x86_64 GNU/Linux

user@${hostname}:~$ uptime
 12:30:00 up 14 days, 3:42, 1 user, load average: 0.15, 0.20, 0.18

user@${hostname}:~$ ps aux | head -3
USER       PID %CPU %MEM    VSZ   RSS TTY  STAT START   TIME COMMAND
root         1  0.0  0.1 168924 11296 ?   Ss   Aug18   0:03 /sbin/init
root         2  0.0  0.0      0     0 ?   S    Aug18   0:00 [kthreadd]

user@${hostname}:~$ ▮

— 真实远程桌面需后端 VNC server + BFF WS relay —
— MVP 阶段: mock surface, 后端落地后 v1.0 切真 (per PHASE-MOBILE-PWA v0.2 §3 G1) —`}
            </pre>
          </div>
        )}
      </div>

      {/* Mobile toolbar */}
      {status === "connected" && (
        <div className="flex items-center gap-1 px-2 py-2 border-t border-line bg-bg-soft/80 backdrop-blur">
          <div className="flex items-center gap-0.5 pr-2 border-r border-line/50">
            <button
              type="button"
              onClick={() => setZoom((z) => Math.max(0.5, +(z - 0.25).toFixed(2)))}
              data-testid="novnc-zoom-out"
              className="p-1.5 rounded-md text-ink-dim hover:text-ink active:scale-95"
              aria-label="Zoom out"
            >
              <ZoomOut size={13} />
            </button>
            <span className="text-[10px] font-mono text-ink-mute min-w-[32px] text-center">
              {Math.round(zoom * 100)}%
            </span>
            <button
              type="button"
              onClick={() => setZoom((z) => Math.min(3, +(z + 0.25).toFixed(2)))}
              data-testid="novnc-zoom-in"
              className="p-1.5 rounded-md text-ink-dim hover:text-ink active:scale-95"
              aria-label="Zoom in"
            >
              <ZoomIn size={13} />
            </button>
            <button
              type="button"
              onClick={() => setZoom(1)}
              data-testid="novnc-zoom-reset"
              className="px-1.5 py-1 text-[10px] font-mono text-ink-mute hover:text-ink"
            >
              1x
            </button>
          </div>
          <div className="flex items-center gap-0.5 flex-1 justify-around">
            {[
              { label: "Ctrl", key: "ctrl" },
              { label: "Alt", key: "alt" },
              { label: "Shift", key: "shift" },
              { label: "Tab", key: "tab" },
              { label: "Esc", key: "esc" },
            ].map((b) => (
              <button
                key={b.key}
                type="button"
                data-testid={`novnc-key-${b.key}`}
                className="px-2.5 py-1.5 rounded-lg border border-line text-[11px] font-mono text-ink-dim hover:text-ink active:scale-95"
              >
                {b.label}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
