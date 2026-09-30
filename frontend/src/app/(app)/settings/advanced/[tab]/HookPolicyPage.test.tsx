/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.test.tsx",type:"file",language:"tsx"}),
  (suite:Function {name:"HookPolicyPage unauthenticated state tests",type:"function",signature:"describe('HookPolicyPage', ...)",visibility:"private",complexity:"simple"}),
  (failClosed:Function {name:"unauthenticated Hook page case",type:"function",signature:"it('does not show seeded policies without a host session', ...)",visibility:"private",complexity:"simple"}),
  (render:Function {name:"Testing Library render",type:"function",signature:"render(<HookPolicyPage />)",visibility:"private",complexity:"simple"}),
  (file)-[:CONTAINS]->(suite),(suite)-[:CONTAINS]->(failClosed),(failClosed)-[:CALLS]->(render);
*/

import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { WorktreeGroupApiProvider } from "@/lib/group/groupProjection";
import HookPolicyPage from "./HookPolicyPage";

describe("HookPolicyPage", () => {
  it("does not show seeded policies without a host session", () => {
    render(<HookPolicyPage />);

    expect(screen.getByTestId("hook-auth-required")).toBeInTheDocument();
    expect(screen.getByText(/当前不显示策略样例/)).toBeInTheDocument();
    expect(screen.queryByTestId("hook-policy-page")).not.toBeInTheDocument();
    expect(screen.queryByTestId("hook-execution-events")).not.toBeInTheDocument();
  });

  it("shows only authorized execution events and makes partial coverage explicit", async () => {
    const event = {
      event_id: "event-1",
      worktree_id: "worktree-1",
      work_item_id: null,
      run_id: null,
      actor_id: "actor-1",
      correlation_id: "correlation-1",
      source_kind: "worktree_lifecycle",
      hook_phase: "worktree_archive",
      hook_decision: "deny",
      hook_reason_code: "active_run_present",
      matched_rule_id: null,
      project_policy_version: 1,
      worktree_policy_version: null,
      evaluator_api_version: 1,
      policy_digest: null,
      evaluated_condition_count: 0,
      duration_ms: 2,
      timed_out: false,
      occurred_at: "2026-10-01T00:00:00.000Z",
    };
    const coverage = {
      scope: "hook_execution_event_ledger",
      status: "partial",
      reported_percentage: null,
      instrumented_phases: ["worktree_archive"],
      not_yet_instrumented_phases: ["run_admission", "tool"],
      note: "Only archive is instrumented.",
    };
    const fetcher = vi.fn<typeof fetch>().mockImplementation(async (input) => {
      const url = new URL(String(input), "http://localhost");
      if (url.pathname === "/api/v1/projects") {
        return new Response(JSON.stringify({ projects: [{ project_id: "11111111-1111-4111-8111-111111111111", role: "project_admin" }] }), { status: 200 });
      }
      if (url.pathname.endsWith("/hook-events")) {
        return new Response(JSON.stringify({ events: [event], next_cursor: null, coverage }), { status: 200 });
      }
      if (url.pathname.endsWith("/hook-policy")) {
        return new Response(JSON.stringify({ policy_set_id: null, policy_document: null, effective_policy_document: null, draft: null, audit: [] }), { status: 200 });
      }
      if (url.pathname.endsWith("/worktrees")) return new Response(JSON.stringify({ worktrees: [] }), { status: 200 });
      return new Response(JSON.stringify({}), { status: 200 });
    });

    render(
      <WorktreeGroupApiProvider getAccessToken={async () => "user-jwt"} sessionKey="session-1" fetcher={fetcher}>
        <HookPolicyPage />
      </WorktreeGroupApiProvider>,
    );

    expect(await screen.findByText("拒绝 · worktree_archive")).toBeInTheDocument();
    expect(screen.getByTestId("hook-event-coverage")).toHaveTextContent("部分接入");
    expect(screen.getByTestId("hook-event-coverage")).toHaveTextContent("覆盖比例未知");
    expect(screen.getByTestId("hook-event-coverage")).toHaveTextContent("未接入范围不按 0 次处理");
    expect(fetcher.mock.calls.some(([input]) => String(input).includes("/api/v1/projects/11111111-1111-4111-8111-111111111111/hook-events?limit=30"))).toBe(true);
  });
});

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (suite:Function {name:"HookPolicyPage unauthenticated state tests"}),
      (render:Function {name:"Testing Library render"});
CREATE (eventPanel:Function {name:"Hook execution event panel UI case",type:"function",language:"typescript",visibility:"private",complexity:"moderate"});
CREATE (suite)-[:CONTAINS]->(eventPanel),(eventPanel)-[:CALLS]->(render);
*/
