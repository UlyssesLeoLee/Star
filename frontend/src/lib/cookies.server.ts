// =====================================================================
// cookies.server.ts — server-only cookies 工具 (per 2026-09-04 canvas e2e 守门 prerequisite)
//
// 用途: 拆分 cookies.ts 让 client component (ProjectsClient.tsx) 能 import
//   client-safe 部分 (PROJECTS_DEFAULT_TAB_COOKIE / writeProjectsDefaultTabCookie
//   / isValidProjectsTab / resolveInitialTab) 而不触发 next/headers 报错
//   "You're importing a component that needs next/headers" (Next 14 App Router
//   server component only, pages directory 禁).
//
// 设计: import "server-only" 强制只在 server 调用, 客户端 import 直接 build fail
//   (per server-only 0.0.1 Next 14 实证).
//
// 触发: 2026-09-04 19:30 JST pnpm test:e2e -- canvas-view 9/9 fail
//   "WebServer Error: ./src/lib/cookies.ts import next/headers not allowed in
//    client bundle" → 拆 server-only 模块修 baseline
// =====================================================================

import "server-only";
import { cookies } from "next/headers";
import { PROJECTS_DEFAULT_TAB_COOKIE, isValidProjectsTab, type ProjectsTabId } from "./cookies";

/**
 * server component 读 cookie (per Next.js 14 next/headers).
 * 在 server 渲染时同步可读, SSR HTML 已经包含正确的 default tab.
 */
export function readProjectsDefaultTabFromCookie(): ProjectsTabId | null {
  try {
    const c = cookies().get(PROJECTS_DEFAULT_TAB_COOKIE)?.value;
    return isValidProjectsTab(c) ? c : null;
  } catch {
    // 边界: middleware 上下文外调用 cookies() 会抛 — fallback null
    return null;
  }
}
