// =====================================================================
// PanelPlaceholder — U1 占位卡片 (per design §5 + §8.1 + 任务要求)
// =====================================================================
// - 6 路由 /inbox /issues /projects /agents /analytics /settings 共享的 placeholder 样式
// - 显示 panel 名称 + "Pending implementation by U2/U3/U4" 文案
// - 使用 U5 dark token (bg/border/ink/accent) — 不改 tailwind.config.ts
// - 由 U2/U3/U4 实装时整段替换为真正的 panel 实现
// =====================================================================
"use client";

import Link from "next/link";
import { ReactNode } from "react";

export type PanelOwner = "U2" | "U3" | "U4";

export interface PanelPlaceholderProps {
  title: string;
  description: string;
  owner: PanelOwner;
  children?: ReactNode;
}

/** Owner worker 简表 — 标在 placeholder 卡片上,方便 DDD Review 找责任人 */
const OWNER_NOTE: Record<PanelOwner, string> = {
  U2: "Pending implementation by U2 (SubNav + Issues 主面板)",
  U3: "Pending implementation by U3 (Projects 多 panel)",
  U4: "Pending implementation by U4 (Agents / Analytics / Inbox / Settings)",
};

export function PanelPlaceholder({
  title,
  description,
  owner,
  children,
}: PanelPlaceholderProps) {
  return (
    <div
      data-testid={`panel-placeholder-${title.toLowerCase()}`}
      data-owner={owner}
      className="max-w-3xl"
    >
      <header className="mb-6">
        <div className="flex items-center gap-2 mb-2">
          <span className="text-[10px] uppercase tracking-wider font-mono text-ink-mute">
            Panel
          </span>
          <span className="text-[10px] font-mono text-warn border border-warn/40 rounded px-1.5 py-0.5">
            {owner} · PENDING
          </span>
        </div>
        <h1
          data-testid="panel-title"
          className="text-2xl font-semibold text-ink leading-tight"
        >
          {title}
        </h1>
        <p className="text-sm text-ink-dim mt-2 leading-relaxed">{description}</p>
      </header>

      <div className="rounded-md border border-line bg-bg-soft/40 p-5 mb-4">
        <div className="text-xs uppercase tracking-wider text-ink-mute font-mono mb-3">
          {OWNER_NOTE[owner]}
        </div>
        <div className="text-sm text-ink-dim leading-relaxed">
          This panel is a <span className="text-accent font-mono">U1 placeholder</span>.
          Real implementation is owned by{" "}
          <span className="text-ink font-mono">{owner}</span> (per design{" "}
          <span className="font-mono">ui-redesign-multica-style.md §8.1</span>).
          SubNav 180px sticky sidebar will appear here for{" "}
          <span className="font-mono">/projects /agents /analytics</span> per §4.
        </div>
        {children}
      </div>

      <div className="flex items-center gap-3 text-xs text-ink-mute">
        <Link
          href="/"
          className="hover:text-accent transition-colors"
          data-testid="panel-back-to-root"
        >
          ← Back to root
        </Link>
        <span aria-hidden="true">·</span>
        <span className="font-mono">AppShell mounted by (app)/layout.tsx</span>
      </div>
    </div>
  );
}
