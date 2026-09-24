"use client";

// =====================================================================
// /terminal-stack-demo — Demo page for Terminal Stack E2E tests (per T23.5)
// (per PR #98.5 + PR #109 + terminal-ws-integration E2E + p1e-end-to-end E2E)
// =====================================================================
// Purpose: 提供 Playwright E2E 测试所需的 demo 页面 + URL 参数 + DOM testids.
//
// URL params:
//   ?sessionId=<uuid>  → 真实 WS 模式 (useTerminalStackWs hook 推 wsClient.connect)
//   no sessionId         → mock 模式 (直接 setWsConnected(true))
//
// E2E 测试 (per e2e/terminal-ws-integration.spec.ts + e2e/p1e-end-to-end.spec.ts):
//   - data-testid="terminal-stack-container"
//   - data-testid="ws-debug"
//   - data-testid="pane-count"
//   - data-testid="split-vertical-btn"
//   - data-testid="terminal-pane-{paneId}"
//
// 不在本 page 范围:
//   ❌ 真实 xterm.js mount (per ULYS-232 §3 Step 1 — T23 followup)
//   ❌ ANSI escape 解析 (per ULYS-200 §7 — P2 followup)
//   ❌ 真实 PTY subprocess (per ULYS-200 §5 — P2 followup)
// =====================================================================

import { Suspense } from "react";
import { useSearchParams } from "next/navigation";
import { TerminalStackContainer } from "@/components/terminal/TerminalStackContainer";
import { useTerminalStackStore } from "@/components/terminal/terminalStackStore";

export default function TerminalStackDemoPage() {
  return (
    <Suspense fallback={<div className="p-4" data-testid="terminal-stack-demo-loading">Loading...</div>}>
      <TerminalStackDemoContent />
    </Suspense>
  );
}

function TerminalStackDemoContent() {
  const searchParams = useSearchParams();
  const sessionId = searchParams?.get("sessionId") ?? null;
  const tree = useTerminalStackStore((s) => s.tree);

  return (
    <div
      className="terminal-stack-demo-page min-h-screen p-4 bg-bg-base text-ink"
      data-testid="terminal-stack-demo-page"
    >
      <h1 className="text-lg font-semibold mb-3">
        Terminal Stack Demo (T23.5 E2E 守门)
        {sessionId ? (
          <span
            className="ml-2 text-xs text-accent"
            data-testid="session-id-display"
          >
            sessionId={sessionId}
          </span>
        ) : (
          <span
            className="ml-2 text-xs text-ink-dim"
            data-testid="session-id-display"
          >
            mock mode
          </span>
        )}
      </h1>
      <TerminalStackContainer sessionId={sessionId} />
      <p
        className="mt-3 text-xs text-ink-dim"
        data-testid="tree-summary"
      >
        tree.kind={tree.kind}, root.panes={JSON.stringify(
          tree.kind === "single"
            ? [tree.root.kind === "pane" ? tree.root.pane.id : tree.root.id]
            : tree.root.kind === "split"
            ? tree.root.children.map((c) =>
                c.kind === "pane" ? c.pane.id : c.id,
              )
            : [],
        )}
      </p>
    </div>
  );
}