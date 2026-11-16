/*
@cypher schema=1 source_sha256=9c464ca208b812920c77471e7947ff1d4188b079fc918738fda8365aaffcbf7f
MERGE (self:File {path:"frontend/src/lib/redirects.ts"})
@endcypher
*/
// =====================================================================
// frontend/src/lib/redirects.ts — U5 (consolidated route structure)
//
// Single source of truth for legacy routes redirected to consolidated app
// routes. /worktree is an active Project Worktree Index route, not a redirect.
//
// 20 domain routes are absorbed into 4 new panel routes (20 total
// entries because /workspace has both an exact-match and a /:id
// path-param variant for deep links):
//   /inbox      ← notification / comment / audit / feedback / search / context
//   /sprint     ← work-item
//   /projects   ← project / workspace / board / planning / scm / collaboration
//                 / relation
//   /settings   ← tenant / identity / permission / integration
//
//   /canvas/:id 是画布本体路由, 不在 redirect 列表 (per 2026-09-04 canvas e2e
//   守门 prerequisite); per 2026-11-16 用户发令 10 条第 4 条, agent 7 路由
//   (/agent /agents /agent-view /validation /automation /development
//   /local-runtime) 不再固定指向 canvas-001, 由 /canvas 入口页
//   (src/app/canvas/page.tsx) 按当前 Worktree / Lane 上下文动态解析。
//
// Notes:
//   - We keep the path-param variants (/workspace/:id) so deep
//     links from emails, docs and old bookmarks survive the migration.
//   - The root path / is NOT in this list — its redirect is handled by
//     app/page.tsx (redirect → /inbox) so it can be unit-tested at the
//     page level rather than the routing-config level.
//   - This module is imported by:
//        1. next.config.js (rewrites the redirects() function)
//        2. e2e/redirects.spec.ts (vitest spec validating structure)
//   - permanent: false → 307 (preserves HTTP method, correct for GET nav)
//   - 308 is the Next.js fallback for permanent: true; we prefer 307 for
//     user-facing nav (avoids caching stale redirects on the client).
// =====================================================================

import type { NextRedirect } from "./redirects.types";

/**
 * Ordered list of legacy-route redirects.
 *
 * Order matters only for human readability — Next.js matches on first
 * hit regardless. The test suite asserts on the *set*, not the order.
 */
export const LEGACY_REDIRECTS: ReadonlyArray<NextRedirect> = [
  // ── /projects sink ─────────────────────────────────────────────────
  { source: "/workspace", destination: "/projects", permanent: false },
  {
    source: "/workspace/:id",
    destination: "/projects/:id",
    permanent: false,
  },
  { source: "/project", destination: "/projects", permanent: false },
  // per 2026-08-31 12:42 JST DRIFT-α-003/004 修复 (handoff 兜底):
  //   5 tab 拍板 (kanban/timeline/backlog/agents/worktrees) → redirect 4 tab 死链全改 worktrees
  { source: "/board", destination: "/projects?tab=kanban", permanent: false },
  {
    source: "/planning",
    destination: "/projects?tab=timeline",
    permanent: false,
  },
  {
    source: "/scm",
    destination: "/projects?tab=worktrees",
    permanent: false,
  },
  {
    source: "/collaboration",
    destination: "/projects?tab=worktrees",
    permanent: false,
  },
  // /workflow 无 redirect (per 2026-11-15 canvas 自审):
  //   workflow 已恢复独立页 (src/app/workflow/page.tsx), redirect 规则在路由层
  //   抢先拦截会让独立页永远不可达 — 与二轮拍板语义矛盾, 删除规则。
  {
    source: "/relation",
    destination: "/projects?tab=worktrees",
    permanent: false,
  },
  // /canvas/:id 不在 legacy redirect 列表 (per 2026-09-04 canvas e2e 守门 prerequisite):
  //   app/canvas/[id]/page.tsx (CanvasView 详情页, per docs/frontend-canvas-design.md §2.3)
  //   是设计文档意图的主入口, next.config.js redirects 抢先生效会让 page.tsx 永远不到达.
  //   原 redirect "/canvas/:id" → "/projects?canvas=:id" 跟设计矛盾, 删.
  //   (per LEGACY_REDIRECTS 注释 /workspace/:id 同样保留原因: app/workspace 目录不存在, 必须 redirect)

  // ── /sprint sink (per 2026-09-05 19:13 JST 拍板: /issues 重命名 /sprint) ──
  // 老链接兜底, 新主入口 /sprint 默认 view=sprint
  { source: "/issues", destination: "/sprint?view=sprint", permanent: false },
  {
    source: "/work-item",
    destination: "/sprint?view=list",
    permanent: false,
  },

  // ── Agent 中心 sink 已删除 (per 2026-11-16 用户发令 10 条第 4 条) ──
  //   /agent /agents /agent-view /validation /automation /development
  //   /local-runtime 不再 307 固定指向 canvas-001 — /canvas 入口
  //   (src/app/canvas/page.tsx) 按当前 Worktree / Lane 上下文动态解析画布,
  //   旧书签直接访问 /canvas 由入口页承载; 画布本体仍走 /canvas/:id。

  // ── /inbox sink ─────────────────────────────────────────────────────
  { source: "/notification", destination: "/inbox", permanent: false },
  {
    source: "/comment",
    destination: "/inbox?type=comment",
    permanent: false,
  },
  { source: "/audit", destination: "/inbox?type=audit", permanent: false },
  {
    source: "/feedback",
    destination: "/inbox?type=feedback",
    permanent: false,
  },
  { source: "/search", destination: "/inbox?type=search", permanent: false },
  {
    source: "/context",
    destination: "/inbox?type=context",
    permanent: false,
  },

  // ── /settings sink ──────────────────────────────────────────────────
  {
    source: "/permission",
    destination: "/settings?tab=permissions",
    permanent: false,
  },
  {
    source: "/identity",
    destination: "/settings?tab=members",
    permanent: false,
  },
  {
    source: "/tenant",
    destination: "/settings?tab=workspace",
    permanent: false,
  },
  {
    source: "/integration",
    destination: "/settings?tab=integrations",
    permanent: false,
  },
];

/**
 * Build a lookup map for fast tests and runtime introspection.
 * Multiple rules can share the same source only via path params (e.g.
 * /workspace vs /workspace/:id), so the LAST write wins for exact
 * matches but a Map preserves insertion order.
 */
export const REDIRECTS_BY_SOURCE: ReadonlyMap<string, NextRedirect> = new Map(
  LEGACY_REDIRECTS.map((r) => [r.source, r]),
);

/**
 * The 6 panel target routes used by redirects. Pages that don't exist
 * yet will 404, but the redirect itself is still served (307).
 *
 * Note: this is informational only; the actual page files are produced
 * by U2/U3/U4. U5 does NOT create them.
 */
export const NEW_PANEL_ROUTES = [
  "/inbox",
  "/issues",
  "/projects",
  "/agents",
  "/analytics",
  "/settings",
] as const;

export type NewPanelRoute = (typeof NEW_PANEL_ROUTES)[number];
