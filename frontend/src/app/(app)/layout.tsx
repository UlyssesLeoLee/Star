// =====================================================================
// (app) layout — AppShell mount (per docs/frontend/design/ui-redesign-multica-style.md §3 + §8.1)
// =====================================================================
// - "use client" 因为 AppShell 包含 client AppHeader
// - 只对 (app) 路由组生效 — 不污染 /api / 22 旧路由
// - children 是 panel page (per §5 + §8.1)
// - AppHeader 已经处理 nav / ⌘K / workspace / user (per §3 + §6)
// - SubNav 槽位: U2 接管 (per §4 + 任务分工)
// - CommandBar 全局 ⌘K listener: P2 缺口 (per 任务说明) — 当前仅 ⌘K 按钮触发
// =====================================================================
"use client";

import { ReactNode } from "react";
import { AppShell } from "@/components/AppShell";

export default function AppLayout({ children }: { children: ReactNode }) {
  return <AppShell>{children}</AppShell>;
}
