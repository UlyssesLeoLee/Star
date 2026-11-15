/*
@cypher schema=1 source_sha256=141eee8f3657d1ae2714de1e5cae5f89078f2ecf71d1ded247fd5f842a090c0b
MERGE (self:File {path:"frontend/src/lib/redirects.shim.cjs"})
@endcypher
*/
// =====================================================================
// frontend/src/lib/redirects.shim.cjs
//
// CommonJS bridge that next.config.js (which must be .js, not .ts) can
// require() to obtain the canonical redirect list without TypeScript
// compile-time overhead at Next.js config load.
//
// The 27 entries below MUST stay in sync with src/lib/redirects.ts.
// The vitest spec at e2e/redirects.spec.ts imports the .ts version
// and asserts count + specific entries are identical, so any drift
// between the two surfaces will fail the test suite.
//
// IMPORTANT: Do NOT Object.freeze() the entries. Next.js's
// load-custom-routes processRoutes() normalises each entry in-place
// (e.g. lower-casing the source path on Windows). Frozen objects throw
// "Cannot assign to read only property" during the build.
// =====================================================================

"use strict";

/** @type {Array<{source: string, destination: string, permanent: boolean}>} */
const LEGACY_REDIRECTS = [
  // ── /projects sink ─────────────────────────────────────────────────
  { source: "/workspace", destination: "/projects", permanent: false },
  { source: "/workspace/:id", destination: "/projects/:id", permanent: false },
  { source: "/project", destination: "/projects", permanent: false },
  { source: "/board", destination: "/projects?tab=kanban", permanent: false },
  { source: "/planning", destination: "/projects?tab=timeline", permanent: false },
  { source: "/scm", destination: "/projects?tab=worktrees", permanent: false },
  { source: "/collaboration", destination: "/projects?tab=worktrees", permanent: false },
  // /workflow: no redirect — 独立页 src/app/workflow/page.tsx 直接可达 (2026-11-15 canvas 自审)
  { source: "/relation", destination: "/projects?tab=worktrees", permanent: false },
  // /canvas/:id 不在 legacy redirect 列表 (per 2026-09-04 canvas e2e 守门 prerequisite):
  //   app/canvas/[id]/page.tsx (CanvasView 详情页) 是设计文档意图主入口.
  //   跟 src/lib/redirects.ts 注释一致, 同步删除.

  // ── /sprint sink (per 2026-09-05 19:13 JST 拍板: /issues 重命名 /sprint) ──
  { source: "/issues", destination: "/sprint?view=sprint", permanent: false },
  { source: "/work-item", destination: "/sprint?view=list", permanent: false },

  // ── /agent-view sink (per 2026-09-05 19:45 JST 拍板: /agents -> /agent-view 307 redirect) ──
  { source: "/agent", destination: "/canvas/canvas-001?tab=agent", permanent: false },
  { source: "/agents", destination: "/canvas/canvas-001?tab=agent", permanent: false },
  // /agent-view: no redirect — Agent 中心独立页 (非无限画布) 直接可达 (2026-11-15)
  { source: "/validation", destination: "/canvas/canvas-001?tab=agent", permanent: false },
  { source: "/automation", destination: "/canvas/canvas-001?tab=agent", permanent: false },
  { source: "/development", destination: "/canvas/canvas-001?tab=agent", permanent: false },
  { source: "/local-runtime", destination: "/canvas/canvas-001?tab=agent", permanent: false },

  // ── /inbox sink ─────────────────────────────────────────────────────
  { source: "/notification", destination: "/inbox", permanent: false },
  { source: "/comment", destination: "/inbox?type=comment", permanent: false },
  { source: "/audit", destination: "/inbox?type=audit", permanent: false },
  { source: "/feedback", destination: "/inbox?type=feedback", permanent: false },
  { source: "/search", destination: "/inbox?type=search", permanent: false },
  { source: "/context", destination: "/inbox?type=context", permanent: false },

  // ── /settings sink ──────────────────────────────────────────────────
  { source: "/permission", destination: "/settings?tab=permissions", permanent: false },
  { source: "/identity", destination: "/settings?tab=members", permanent: false },
  { source: "/tenant", destination: "/settings?tab=workspace", permanent: false },
  { source: "/integration", destination: "/settings?tab=integrations", permanent: false },
];

module.exports = { LEGACY_REDIRECTS };
