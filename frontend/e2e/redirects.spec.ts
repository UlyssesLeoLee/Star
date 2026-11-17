/*
@cypher schema=1 source_sha256=3e35bd77fb6454780bd10e4a08ef261a1e7216ab0fd334abd6de7bf0d38fff54
MERGE (self:File {path:"frontend/e2e/redirects.spec.ts"})
MERGE (redirects:File {path:"frontend/src/lib/redirects.ts"})
MERGE (shim:File {path:"frontend/src/lib/redirects.shim.cjs"})
MERGE (legacyRedirects:Symbol {id:"frontend/src/lib/redirects.ts::LEGACY_REDIRECTS",kind:"const"})
MERGE (self)-[:IMPORTS]->(redirects)
MERGE (self)-[:IMPORTS]->(shim)
MERGE (self)-[:TESTS]->(legacyRedirects)
@endcypher
*/
// =====================================================================
// frontend/e2e/redirects.spec.ts — U5 (multica-style route consolidation)
//
// Unit-level tests for the legacy-route redirect map and retained first-class routes.
//
// These are NOT a Playwright/browser e2e suite. We don't have @playwright
// installed in this worktree. Instead we validate the redirect config at
// the same layer Next.js uses: a typed object that next.config.js passes
// to its redirects() function. A real server start is exercised by the
// worker via `next start -p 3130` + curl in the verification log.
//
// Why validate the config and not a running server?
//   - next.config.js is loaded by Next.js at boot, and the redirect() output
//     is a plain JSON-serializable object. We can import the same module
//     that next.config.js requires (src/lib/redirects.ts) and assert on
//     its content.
//   - This catches drift between the .ts source and the .cjs shim that
//     next.config.js loads, and between the spec and either source.
//
// The 8 spec cases the parent brief calls for:
//   1. /workspace       → 307/308 → /projects
//   2. /work-item       → /issues?view=kanban
//   3. /board           → /projects?tab=board
//   4. /audit           → /inbox?type=audit
//   5. /planning        → /projects?tab=gantt
//   6. /workflow        → /projects?tab=workflow
//   7. /workspace/123   → /projects/123   (path-param variant)
//   8. /  is NOT in the next.config.js redirect list
//      (so it falls through to app/page.tsx, which has its own redirect)
// =====================================================================

import { describe, it, expect } from "vitest";
import { LEGACY_REDIRECTS, REDIRECTS_BY_SOURCE } from "../src/lib/redirects";
import { LEGACY_REDIRECTS as SHIM_REDIRECTS } from "../src/lib/redirects.shim.cjs";

const findRedirect = (source: string) => {
  // Exact match first (handles /workspace)
  const exact = REDIRECTS_BY_SOURCE.get(source);
  if (exact) return exact;
  // Path-param match (handles /workspace/123 → /workspace/:id)
  for (const r of LEGACY_REDIRECTS) {
    if (r.source.includes(":")) {
      const pattern = new RegExp(
        "^" + r.source.replace(/:[A-Za-z_]+/g, "[^/]+") + "$",
      );
      if (pattern.test(source)) return r;
    }
  }
  return undefined;
};

describe("redirects: legacy 20 routes → 4 panels", () => {
  it("contains 19 entries (per 2026-11-17 去重: /planning 恢复独立页, /board 改指 /kanban)", () => {
    // /worktree is not redirected because it is the Project Worktree Index route.
    // per 2026-11-16 10 条第 4 条: agent-sink 7 routes removed (不再固定指向 canvas-001).
    // per 2026-11-17 去重: /planning redirect 删除 (独立页可达, 同 /workflow 先例),
    //   /board 目的地改 /kanban (任务看板合并进 /kanban)。
    // Total: 19 = 8 project routes + 2 sprint routes + 5 inbox + 4 settings.
    expect(LEGACY_REDIRECTS.length).toBe(19);
  });

  it("/workspace → /projects (case 1)", () => {
    const r = findRedirect("/workspace");
    expect(r?.destination).toBe("/projects");
    expect(r?.permanent).toBe(false);
  });

  it("/work-item → /sprint?view=list (case 2, per 2026-09-05 19:13 JST: /issues 重命名 /sprint + Kanban 已删)", () => {
    const r = findRedirect("/work-item");
    expect(r?.destination).toBe("/sprint?view=list");
  });

  it("/worktree remains the Project Worktree Index route", () => {
    expect(findRedirect("/worktree")).toBeUndefined();
  });

  it("/board → /kanban (case 3, per 2026-11-17 去重: /projects?tab=kanban 合并进 /kanban)", () => {
    // per 2026-08-31 12:42 JST DRIFT-α-003: 旧 tab=board 不存在, 改 kanban;
    // per 2026-11-17 去重: /projects 的 kanban tab 合并进 /kanban 顶层路由, 目的地随之更新。
    const r = findRedirect("/board");
    expect(r?.destination).toBe("/kanban");
  });

  it("/audit → /inbox?type=audit (case 4)", () => {
    const r = findRedirect("/audit");
    expect(r?.destination).toBe("/inbox?type=audit");
  });

  it("/planning 无 redirect — 独立页 src/app/planning/page.tsx 直接可达 (2026-11-17 去重, 同 /workflow 先例)", () => {
    const r = findRedirect("/planning");
    expect(r).toBeUndefined();
  });

  it("/workflow 无 redirect — 独立页 src/app/workflow/page.tsx 直接可达 (2026-11-15 canvas 自审)", () => {
    const r = findRedirect("/workflow");
    expect(r).toBeUndefined();
  });

  it("/agent /agents /agent-view /validation /automation /development /local-runtime 不再指向 canvas-001 (per 2026-11-16 10 条第 4 条: /canvas 入口按 Worktree/Lane 上下文解析)", () => {
    for (const src of [
      "/agent",
      "/agents",
      "/agent-view",
      "/validation",
      "/automation",
      "/development",
      "/local-runtime",
    ]) {
      expect(findRedirect(src), `${src} should not be redirected`).toBeUndefined();
    }
  });

  it("/workspace/:id → /projects/:id (case 7, path-param variant)", () => {
    // The destination stored in next.config.js is a TEMPLATE ("/projects/:id"),
    // not the resolved value. Next.js substitutes :id at request time. So
    // we assert on the template, and on a regex-based simulation of the
    // runtime substitution.
    const r = REDIRECTS_BY_SOURCE.get("/workspace/:id");
    expect(r?.destination).toBe("/projects/:id");
    // Local template-to-path substitution (mirror of what Next.js does).
    const live = findRedirect("/workspace/123");
    expect(live?.destination).toBe("/projects/:id");
    const resolved = live!.destination.replace(/:id/g, "123");
    expect(resolved).toBe("/projects/123");
  });

  it("/ is NOT in the next.config.js redirect list (case 8, default)", () => {
    // The root path is handled by app/page.tsx (redirect → /inbox), not
    // by next.config.js. Asserting the absence here is what makes the
    // "default" branch of the redirect map well-defined.
    expect(REDIRECTS_BY_SOURCE.has("/")).toBe(false);
    expect(findRedirect("/")).toBeUndefined();
  });

  it("shim (cjs) stays in sync with .ts source", () => {
    // next.config.js loads the cjs shim; vitest loads the .ts. Both
    // must agree entry-for-entry. A drift here means a 307 in production
    // differs from the documented spec.
    expect(SHIM_REDIRECTS.length).toBe(LEGACY_REDIRECTS.length);
    SHIM_REDIRECTS.forEach((s, i) => {
      expect(s.source).toBe(LEGACY_REDIRECTS[i].source);
      expect(s.destination).toBe(LEGACY_REDIRECTS[i].destination);
      expect(s.permanent).toBe(LEGACY_REDIRECTS[i].permanent);
    });
  });

  it("every destination targets one of the 4 new panel routes", () => {
    const PANEL_PREFIXES = [
      "/inbox",
      "/sprint",
      "/projects",
      "/settings",
      "/kanban",
    ];
    for (const r of LEGACY_REDIRECTS) {
      const ok = PANEL_PREFIXES.some(
        (p) =>
          r.destination === p || r.destination.startsWith(p + "?") || r.destination.startsWith(p + "/"),
      );
      // /projects?canvas=:id → starts with /projects?
      // /projects/:id → starts with /projects/
      expect(
        ok,
        `redirect ${r.source} → ${r.destination} does not target a known panel`,
      ).toBe(true);
    }
  });

  it("no duplicate source paths", () => {
    const seen = new Set<string>();
    for (const r of LEGACY_REDIRECTS) {
      expect(seen.has(r.source), `duplicate source: ${r.source}`).toBe(false);
      seen.add(r.source);
    }
  });

  it("all 20 entries use 307 (permanent: false) for client-friendly nav", () => {
    for (const r of LEGACY_REDIRECTS) {
      expect(r.permanent, `${r.source} should not be permanent`).toBe(false);
    }
  });
});
