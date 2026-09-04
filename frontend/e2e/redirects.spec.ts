// =====================================================================
// frontend/e2e/redirects.spec.ts — U5 (multica-style route consolidation)
//
// Unit-level tests for the 26-entry legacy-route → 6-panel redirect map.
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

describe("redirects: legacy 22 routes → 6 panels", () => {
  it("contains 26 entries (9 projects-sink + 1 /workspace/:id param + 2 issues-sink + 5 agents-sink + 6 inbox-sink + 3 settings-sink)", () => {
    // per 2026-09-04 canvas e2e 守门 prerequisite: /canvas/:id 移出 legacy redirect 列表
    //   (app/canvas/[id]/page.tsx 才是设计文档意图的 CanvasView Miro 详情页主入口).
    //   从 27 entries 减到 26 entries.
    expect(LEGACY_REDIRECTS.length).toBe(26);
  });

  it("/workspace → /projects (case 1)", () => {
    const r = findRedirect("/workspace");
    expect(r?.destination).toBe("/projects");
    expect(r?.permanent).toBe(false);
  });

  it("/work-item → /issues?view=kanban (case 2)", () => {
    const r = findRedirect("/work-item");
    expect(r?.destination).toBe("/issues?view=kanban");
  });

  it("/board → /projects?tab=kanban (case 3, per 2026-08-31 12:42 JST DRIFT-α-003 修复)", () => {
    // per handoff 兜底 + 5 tab 拍板 (kanban/timeline/backlog/agents/worktrees):
    //   /board 旧 redirect tab=board 不存在, 改 tab=kanban (5 tab 之一)
    const r = findRedirect("/board");
    expect(r?.destination).toBe("/projects?tab=kanban");
  });

  it("/audit → /inbox?type=audit (case 4)", () => {
    const r = findRedirect("/audit");
    expect(r?.destination).toBe("/inbox?type=audit");
  });

  it("/planning → /projects?tab=timeline (case 5)", () => {
    const r = findRedirect("/planning");
    expect(r?.destination).toBe("/projects?tab=timeline");
  });

  it("/workflow → /projects?tab=worktrees (case 6, per 2026-08-31 12:42 JST DRIFT-α-004 修复)", () => {
    // per handoff 兜底 + 5 tab 拍板:
    //   /scm /collaboration /workflow /relation 4 redirect 旧 tab=workflow/relations 不存在
    //   全部改 tab=worktrees (5 tab 之一, worktree domain 入口)
    const r = findRedirect("/workflow");
    expect(r?.destination).toBe("/projects?tab=worktrees");
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

  it("every destination targets one of the 6 new panel routes", () => {
    const PANEL_PREFIXES = [
      "/inbox",
      "/issues",
      "/projects",
      "/agents",
      "/settings",
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

  it("all 26 entries use 307 (permanent: false) for client-friendly nav", () => {
    for (const r of LEGACY_REDIRECTS) {
      expect(r.permanent, `${r.source} should not be permanent`).toBe(false);
    }
  });
});
