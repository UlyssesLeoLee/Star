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
import { usePathname } from "next/navigation";
import { AppShell } from "@/components/AppShell";

// 无限画布类页面 — 跳过 main 的 padding/max-width (per fullBleed 设计)
const FULL_BLEED_PREFIXES = ["/agent-view"];

export default function AppLayout({ children }: { children: ReactNode }) {
  const pathname = usePathname();
  const fullBleed = FULL_BLEED_PREFIXES.some((p) => pathname?.startsWith(p));
  return <AppShell fullBleed={fullBleed}>{children}</AppShell>;
}
