// =====================================================================
// cookies.ts — 共享 cookie 工具 (client-safe, server-only 函数拆到 cookies.server.ts)
//
// 用途: projects default tab 持久化 (per 2026-09-01 16:41 JST "界面迁移全面完善"
//   拍板 cookie-default 修法: SSR 通过 server component cookies() 读 cookie,
//   client 切 tab 时写 cookie, 下次 SSR 拿 cookie 渲染正确默认 tab, 避免
//   "SSR 默认 kanban → client hydration 闪一下变 timeline" 的视觉跳变)
//
// 设计 (per 2026-09-04 baseline fix):
//   1. PROJECTS_DEFAULT_TAB_COOKIE = "projects-default-tab" (唯一来源)
//   2. VALID_TABS = 5 个合法 tab id (server 校验防注入)
//   3. readServerCookie: 拆到 cookies.server.ts (server-only), import "server-only"
//   4. writeClientCookie: client component 用 document.cookie (本文件)
//   5. path=/; max-age=1y; SameSite=Lax (防 CSRF, 保持跨页一致)
//
// 拆分原因: Next.js 14 App Router 不允许 client component (ProjectsClient.tsx)
//   引用任何 import "next/headers" 的文件, 即使实际不调用 server 函数. 拆出
//   cookies.server.ts 加 import "server-only" 让 client bundle 完全不含 next/headers.
// =====================================================================

export const PROJECTS_DEFAULT_TAB_COOKIE = "projects-default-tab";

export const VALID_PROJECTS_TABS = [
  "kanban",
  "timeline",
  "backlog",
  "agents",
  "worktrees",
] as const;

export type ProjectsTabId = (typeof VALID_PROJECTS_TABS)[number];

export function isValidProjectsTab(t: string | undefined | null): t is ProjectsTabId {
  return !!t && (VALID_PROJECTS_TABS as readonly string[]).includes(t);
}

/**
 * client 写 cookie (per document.cookie).
 * tab change 时调用, 1 年有效期, 跨 path.
 */
export function writeProjectsDefaultTabCookie(tab: ProjectsTabId): void {
  if (typeof document === "undefined") return;
  // 1y 有效期, path=/, SameSite=Lax (防 CSRF + 跨页可用)
  const oneYear = 60 * 60 * 24 * 365;
  document.cookie = `${PROJECTS_DEFAULT_TAB_COOKIE}=${tab}; path=/; max-age=${oneYear}; SameSite=Lax`;
}

/**
 * 解析 URL ?tab= / ?canvas= 推断 default tab (server-side, 无 client hook 依赖).
 * 优先级: URL ?tab=X > URL ?canvas= → backlog > cookie > "kanban"
 */
export function resolveInitialTab(
  urlSearchParams: Record<string, string | string[] | undefined> | undefined,
  cookieTab: ProjectsTabId | null,
): ProjectsTabId {
  if (urlSearchParams) {
    const tabParam = pickString(urlSearchParams.tab);
    if (isValidProjectsTab(tabParam)) return tabParam;
    // ?canvas=:id → backlog tab (per 2026-08-31 12:42 JST DRIFT-α-005 修复)
    if (pickString(urlSearchParams.canvas)) return "backlog";
  }
  return cookieTab ?? "kanban";
}

function pickString(v: string | string[] | undefined): string | undefined {
  if (Array.isArray(v)) return v[0];
  return v;
}
