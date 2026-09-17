// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/mocks/handlers/worktree-canvas.ts
//
// MSW handlers for /api/worktree-canvas/* (per ULYS-57.4 T13 + DD §20-§21 +
// WORKTREE-CANVAS-IMPL-PLAN-001 §4.13). Stage 1 stub, real impl per §3.
//
// 阶段 1 stub: 0 真实业务, mock 数据 + 占位 401 守门. 阶段 2 实装接 BFF + graph-core.

import { http, HttpResponse } from "msw";

const MOCK_REPOS = [
  { id: "11111111-0000-0000-0000-000000000001", name: "demo-repo-1", default_branch: "main", worktree_count: 3 },
];

const MOCK_WORKTREES = [
  {
    id: "22222222-0000-0000-0000-000000000001",
    repo_id: "11111111-0000-0000-0000-000000000001",
    name: "feature/payment-fix",
    branch: "feature/payment-fix",
    human_state: "RUNNING",
    machine_state: "git_clean",
    ahead: 5,
    behind: 2,
    dirty: false,
    health_score: { value: 85, deductions: [], computed_at: "2026-09-17T00:00:00Z" },
    last_activity: "2026-09-17T00:00:00Z",
    last_activity_relative: "5m ago",
    agent: { id: "ag-001", agent_type: "claude-code", model: "claude-sonnet-4" },
    task: { id: "task-001", title: "Fix payment retry crash" },
    risk_count: 0,
    test_state: "PASSED",
    locked: false,
    archived: false,
    created_at: "2026-09-15T00:00:00Z",
    merged_at: null,
  },
];

export const worktreeCanvasHandlers = [
  http.get("/api/worktree-canvas/repositories", () => HttpResponse.json(MOCK_REPOS)),
  http.get("/api/worktree-canvas/worktrees", () =>
    HttpResponse.json({ items: MOCK_WORKTREES, total: MOCK_WORKTREES.length, next_cursor: null })
  ),
  http.get("/api/worktree-canvas/worktrees/:id", ({ params }) => {
    const wt = MOCK_WORKTREES.find((w) => w.id === params.id);
    return wt ? HttpResponse.json(wt) : HttpResponse.json({ code: "NOT_FOUND" }, { status: 404 });
  }),
  http.post("/api/worktree-canvas/worktrees", () =>
    HttpResponse.json(
      {
        success: true,
        worktree_id: "33333333-0000-0000-0000-000000000099",
        new_state: "RUNNING",
        duration_ms: 0,
        warnings: [],
        errors: [],
      },
      { status: 201 }
    )
  ),
  http.post("/api/worktree-canvas/worktrees/:id/actions/:actionType", () =>
    HttpResponse.json({
      success: true,
      worktree_id: "22222222-0000-0000-0000-000000000001",
      new_state: "RUNNING",
      duration_ms: 0,
      warnings: [],
      errors: [],
    })
  ),
  http.get("/api/worktree-canvas/risks", () => HttpResponse.json([])),
  http.get("/api/worktree-canvas/health", () =>
    HttpResponse.json({ overall_score: 85, healthy: 1, at_risk: 0, diverged: 0, conflict: 0, stale: 0 })
  ),
  http.post("/api/worktree-canvas/search", () =>
    HttpResponse.json({ worktree_ids: MOCK_WORKTREES.map((w) => w.id), total: 1, duration_ms: 5 })
  ),
  http.post("/api/worktree-canvas/nl-query", () =>
    HttpResponse.json({
      worktree_ids: [],
      translated_query: "",
      explanation: "[stage 1 mock] NL query",
    })
  ),
  http.get("/api/worktree-canvas/bff-health", () =>
    HttpResponse.json({ status: "ok", version: "0.2.0", uptime_seconds: 0, worktree_canvas_enabled: true })
  ),
];
