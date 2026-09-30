/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.test.tsx",type:"file",language:"tsx"}),
  (suite:Function {name:"HookPolicyPage unauthenticated state tests",type:"function",signature:"describe('HookPolicyPage', ...)",visibility:"private",complexity:"simple"}),
  (failClosed:Function {name:"unauthenticated Hook page case",type:"function",signature:"it('does not show seeded policies without a host session', ...)",visibility:"private",complexity:"simple"}),
  (render:Function {name:"Testing Library render",type:"function",signature:"render(<HookPolicyPage />)",visibility:"private",complexity:"simple"}),
  (file)-[:CONTAINS]->(suite),(suite)-[:CONTAINS]->(failClosed),(failClosed)-[:CALLS]->(render);
*/

import { fireEvent, render, screen, waitFor } from "@testing-library/react";
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
    const summary = {
      metric_version: "hook_execution_summary_v2",
      window: { days: 30, from: "2026-09-01T00:00:00.000Z", to: "2026-10-01T00:00:00.000Z" },
      observed_event_count: 1,
      run_linked_event_count: 0,
      run_state_joined_event_count: 0,
      excluded_incomplete_run_event_count: 0,
      timeout_count: 0,
      duration_total_ms: 2,
      groups: [],
      coverage: {
        scope: "hook_execution_event_and_task_execution_run_event",
        status: "partial",
        reported_percentage: null,
        instrumented_phases: ["worktree_archive"],
        not_yet_instrumented_phases: ["run_admission", "tool"],
        run_state_join: "no_samples",
        note: "Only archive is instrumented.",
      },
      formulas: {},
    };
    const fetcher = vi.fn<typeof fetch>().mockImplementation(async (input) => {
      const url = new URL(String(input), "http://localhost");
      if (url.pathname === "/api/v1/projects") {
        return new Response(JSON.stringify({ projects: [{ project_id: "11111111-1111-4111-8111-111111111111", role: "project_admin" }] }), { status: 200 });
      }
      if (url.pathname.endsWith("/hook-events/summary")) {
        return new Response(JSON.stringify(summary), { status: 200 });
      }
      if (url.pathname.endsWith("/hook-events")) {
        return new Response(JSON.stringify({ events: [event], next_cursor: null, coverage }), { status: 200 });
      }
      if (url.pathname.endsWith("/hook-policy")) {
        return new Response(JSON.stringify({ producer_capabilities: { run_admission: false }, policy_set_id: null, policy_document: null, effective_policy_document: null, draft: null, audit: [] }), { status: 200 });
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

  it("enables Run admission phase only when the service reports an installed producer", async () => {
    const projectId = "11111111-1111-4111-8111-111111111111";
    const policyDocument = {
      schema_version: 1,
      evaluator_api_version: 1,
      tenant_id: Array(16).fill(1),
      project_id: Array(16).fill(2),
      worktree_id: null,
      project_version: 1,
      worktree_version: null,
      digest: Array(32).fill(0),
      project_rules: [{
        rule_id: Array(16).fill(3),
        priority: 1,
        enabled: true,
        decision: "Deny",
        reason_code: "RuleDenied",
        conditions: [],
      }],
      worktree_rules: [],
    };
    let runAdmissionAvailable = false;
    const fetcher = vi.fn<typeof fetch>().mockImplementation(async (input) => {
      const url = new URL(String(input), "http://localhost");
      if (url.pathname === "/api/v1/projects") {
        return new Response(JSON.stringify({ projects: [{ project_id: projectId, role: "project_admin" }] }), { status: 200 });
      }
      if (url.pathname.endsWith("/hook-policy")) {
        return new Response(JSON.stringify({ producer_capabilities: { run_admission: runAdmissionAvailable }, policy_set_id: "policy-1", policy_document: policyDocument, effective_policy_document: policyDocument, draft: null, audit: [] }), { status: 200 });
      }
      if (url.pathname.endsWith("/hook-events/summary")) {
        const instrumentedPhases = runAdmissionAvailable ? ["worktree_archive", "run_admission"] : ["worktree_archive"];
        const pendingPhases = runAdmissionAvailable ? ["tool"] : ["run_admission", "tool"];
        return new Response(JSON.stringify({
          metric_version: "hook_execution_summary_v2",
          window: { days: 30, from: "2026-09-01T00:00:00.000Z", to: "2026-10-01T00:00:00.000Z" },
          observed_event_count: 0,
          run_linked_event_count: 0,
          run_state_joined_event_count: 0,
          excluded_incomplete_run_event_count: 0,
          timeout_count: 0,
          duration_total_ms: 0,
          groups: [],
          coverage: { scope: "hook_execution_event_and_task_execution_run_event", status: "partial", reported_percentage: null, instrumented_phases: instrumentedPhases, not_yet_instrumented_phases: pendingPhases, run_state_join: "no_samples", note: "Other producers remain pending." },
          formulas: {},
        }), { status: 200 });
      }
      if (url.pathname.endsWith("/worktrees")) return new Response(JSON.stringify({ worktrees: [] }), { status: 200 });
      const instrumentedPhases = runAdmissionAvailable ? ["worktree_archive", "run_admission"] : ["worktree_archive"];
      const pendingPhases = runAdmissionAvailable ? ["tool"] : ["run_admission", "tool"];
      return new Response(JSON.stringify({ events: [], next_cursor: null, coverage: { status: "partial", reported_percentage: null, instrumented_phases: instrumentedPhases, not_yet_instrumented_phases: pendingPhases, note: "Other producers remain pending." } }), { status: 200 });
    });

    render(
      <WorktreeGroupApiProvider getAccessToken={async () => "user-jwt"} sessionKey="session-1" fetcher={fetcher}>
        <HookPolicyPage />
      </WorktreeGroupApiProvider>,
    );

    expect(await screen.findByLabelText("Hook 适用阶段")).toHaveValue("BeforeWorktreeArchiveCleanup");
    expect(screen.getByRole("option", { name: "Run admission（producer 未安装）" })).toBeDisabled();
    expect(screen.getByText(/Run admission 的 Rust phase contract 已定义/)).toBeInTheDocument();

    runAdmissionAvailable = true;
    fireEvent.click(screen.getByRole("button", { name: "刷新", exact: true }));
    await waitFor(() => expect(screen.getByRole("option", { name: "Run admission" })).toBeEnabled());
    expect(screen.getByText(/Run admission producer 已由服务端报告可用/)).toBeInTheDocument();
    await waitFor(() => expect(screen.getByTestId("hook-event-coverage")).toHaveTextContent("run_admission"));
  });
});

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (suite:Function {name:"HookPolicyPage unauthenticated state tests"}),
      (render:Function {name:"Testing Library render"});
CREATE (eventPanel:Function {name:"Hook execution event panel UI case",type:"function",language:"typescript",visibility:"private",complexity:"moderate"});
CREATE (phaseEditor:Function {name:"Run admission phase remains unavailable UI case",type:"function",language:"typescript",visibility:"private",complexity:"moderate"});
CREATE (producerCoverage:Function {name:"Run admission producer capability refresh case",type:"function",language:"typescript",visibility:"private",complexity:"moderate"});
CREATE (suite)-[:CONTAINS]->(eventPanel),(eventPanel)-[:CALLS]->(render),
       (suite)-[:CONTAINS]->(phaseEditor),(phaseEditor)-[:CALLS]->(render);
CREATE (suite)-[:CONTAINS]->(producerCoverage),(producerCoverage)-[:CALLS]->(render);
*/
