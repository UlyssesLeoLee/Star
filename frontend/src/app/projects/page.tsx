// =====================================================================
// Projects Page — server wrapper (per 2026-09-01 16:41 JST "界面迁移全面完善" 拍板)
// =====================================================================
// 职责:
//   1. server component 读 cookies() + searchParams 解析 initialTab
//      优先级: URL ?tab=X > URL ?canvas= → backlog > cookie > "kanban"
//   2. 渲染 <ProjectsClient initialTab={...} />, SSR HTML 已包含正确 default tab
//   3. 避免之前 "SSR 默认 kanban → client hydration 闪一下变 timeline" 的视觉跳变
//
// 配套文件:
//   - ProjectsClient.tsx — 客户端实现, 接受 initialTab prop, 切 tab 写 cookie
//   - src/lib/cookies.ts — 共享 cookie 工具 + ProjectsTabId 类型
//
// 历史 (per 守门 #1 禁回溯叙事, 只列 commit 实证):
//   - 2026-08-29 7d85c34 — 5 tab 命名实装 (Kanban / Timeline / Backlog / Agents / Worktrees)
//   - 2026-08-29 per AGENTS v0.11 已知缺口 #1 — useSearchParams 客户端生效, SSR 仍走默认
//   - 2026-09-01 16:41 JST cookie-default 拍板 → 本 commit 落地 server wrapper 修 SSR bug
// =====================================================================

import ProjectsClient from "./ProjectsClient";
import {
  readProjectsDefaultTabFromCookie,
  resolveInitialTab,
} from "@/lib/cookies";

// Next.js 14 强制 page-level dynamic 防止静态化时拿不到 cookie
export const dynamic = "force-dynamic";

export default function ProjectsPage({
  searchParams,
}: {
  searchParams?: Record<string, string | string[] | undefined>;
}) {
  // 1) 读 cookie (server side, SSR 同步)
  const cookieTab = readProjectsDefaultTabFromCookie();
  // 2) 合并 cookie + URL, 计算 initialTab
  const initialTab = resolveInitialTab(searchParams, cookieTab);

  return <ProjectsClient initialTab={initialTab} />;
}
