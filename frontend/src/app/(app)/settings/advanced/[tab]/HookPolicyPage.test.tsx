/*
@cypher schema=1 source_sha256=2b4290c300fc7e09933512dd5a571dd45c4b4ecd575698b4e7d140fb92997d93
MERGE (self:File {path:"frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.test.tsx::HookPolicyPage tests",kind:"suite"})
MERGE (expirationCase:Symbol {id:"frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.test.tsx::clears policy UI when the host session expires case",kind:"test"})
MERGE (unsavedCase:Symbol {id:"frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.test.tsx::warns before discarding local policy edits case",kind:"test"})
MERGE (provider:Symbol {id:"frontend/src/lib/group/groupProjection.tsx::WorktreeGroupApiProvider",kind:"function"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.tsx::HookPolicyPage",kind:"function"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:DEFINES]->(expirationCase)
MERGE (suite)-[:DEFINES]->(unsavedCase)
MERGE (expirationCase)-[:CALLS]->(provider)
MERGE (expirationCase)-[:CALLS]->(page)
MERGE (expirationCase)-[:TESTS]->(provider)
MERGE (expirationCase)-[:TESTS]->(page)
MERGE (unsavedCase)-[:CALLS]->(provider)
MERGE (unsavedCase)-[:CALLS]->(page)
MERGE (unsavedCase)-[:TESTS]->(page)
@endcypher
*/
/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.test.tsx",type:"file",language:"tsx"}),
  (suite:Function {name:"HookPolicyPage unauthenticated state tests",type:"function",signature:"describe('HookPolicyPage', ...)",visibility:"private",complexity:"simple"}),
  (page:Function {name:"HookPolicyPage",type:"function",signature:"HookPolicyPage()",visibility:"public",complexity:"complex"}),
  (failClosed:Function {name:"unauthenticated Hook page case",type:"function",signature:"it('does not show seeded policies without a host session', ...)",visibility:"private",complexity:"simple"}),
  (unsavedChanges:Function {name:"unsaved policy changes case",type:"function",signature:"it('warns before discarding local policy edits', ...)",visibility:"private",complexity:"moderate"}),
  (render:Function {name:"Testing Library render",type:"function",signature:"render(<HookPolicyPage />)",visibility:"private",complexity:"simple"}),
  (file)-[:CONTAINS]->(suite),(file)-[:DEPENDS_ON]->(page),(suite)-[:CONTAINS]->(failClosed),(failClosed)-[:CALLS]->(render),
  (suite)-[:CONTAINS]->(unsavedChanges),(unsavedChanges)-[:CALLS]->(render),
  (unsavedChanges)-[:TESTS]->(page);
*/

import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { WorktreeGroupApiProvider } from "@/lib/group/groupProjection";
import HookPolicyPage from "./HookPolicyPage";

const navigation = vi.hoisted(() => ({ query: "", router: { replace: vi.fn() } }));
vi.mock("next/navigation", () => ({
  useRouter: () => navigation.router,
  usePathname: () => "/settings/advanced/hooks",
  useSearchParams: () => new URLSearchParams(navigation.query),
}));
beforeEach(() => { navigation.query = ""; navigation.router.replace.mockClear(); });
afterEach(() => vi.unstubAllGlobals());

describe("HookPolicyPage", () => {
  it("does not show seeded policies without a host session", () => {
    render(<HookPolicyPage />);

    expect(screen.getByTestId("hook-auth-required")).toBeInTheDocument();
    expect(screen.getByText(/当前不显示策略样例/)).toBeInTheDocument();
    expect(screen.queryByTestId("hook-policy-page")).not.toBeInTheDocument();
    expect(screen.queryByTestId("hook-execution-events")).not.toBeInTheDocument();
  });

  it("clears the policy UI when the host session expires", async () => {
    let token: string | null = "user-jwt";
    const projectId = "11111111-1111-4111-8111-111111111111";
    const fetcher = vi.fn<typeof fetch>().mockImplementation(async (input) => {
      const url = new URL(String(input), "http://localhost");
      if (url.pathname === "/api/v1/projects") {
        return new Response(JSON.stringify({ projects: [{ project_id: projectId, role: "project_admin" }], next_cursor: null }), { status: 200 });
      }
      if (url.pathname.endsWith("/hook-events/summary")) {
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
          coverage: { scope: "hook_execution_event_and_task_execution_run_event", status: "partial", reported_percentage: null, instrumented_phases: [], not_yet_instrumented_phases: ["run_admission", "tool"], run_state_join: "no_samples", note: "No producer evidence." },
          formulas: {},
        }), { status: 200 });
      }
      if (url.pathname.endsWith("/hook-events")) {
        return new Response(JSON.stringify({ events: [], next_cursor: null, coverage: { scope: "hook_execution_event_ledger", status: "partial", reported_percentage: null, instrumented_phases: [], not_yet_instrumented_phases: ["run_admission", "tool"], note: "No producer evidence." } }), { status: 200 });
      }
      if (url.pathname.endsWith("/hook-policy")) {
        return new Response(JSON.stringify({ producer_capabilities: { run_admission: false }, policy_set_id: null, policy_document: null, effective_policy_document: null, draft: null, audit: [] }), { status: 200 });
      }
      if (url.pathname.endsWith("/worktrees")) return new Response(JSON.stringify({ worktrees: [], next_cursor: null }), { status: 200 });
      return new Response(JSON.stringify({}), { status: 200 });
    });

    render(
      <WorktreeGroupApiProvider getAccessToken={async () => token} sessionKey="host-session-1" fetcher={fetcher}>
        <HookPolicyPage />
      </WorktreeGroupApiProvider>,
    );

    expect(await screen.findByTestId("hook-policy-page")).toBeInTheDocument();
    await waitFor(() => expect(fetcher.mock.calls.some(([input]) => String(input).includes("/hook-policy"))).toBe(true));

    token = null;
    const refreshEvents = screen.getByRole("button", { name: "刷新摘要与事件" });
    await waitFor(() => expect(refreshEvents).toBeEnabled());
    fireEvent.click(refreshEvents);

    expect(await screen.findByTestId("hook-auth-required")).toBeInTheDocument();
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
        return new Response(JSON.stringify({ projects: [{ project_id: "11111111-1111-4111-8111-111111111111", role: "project_admin" }], next_cursor: null }), { status: 200 });
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
      if (url.pathname.endsWith("/worktrees")) return new Response(JSON.stringify({ worktrees: [], next_cursor: null }), { status: 200 });
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
        return new Response(JSON.stringify({ projects: [{ project_id: projectId, role: "project_admin" }], next_cursor: null }), { status: 200 });
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
      if (url.pathname.endsWith("/worktrees")) return new Response(JSON.stringify({ worktrees: [], next_cursor: null }), { status: 200 });
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

  it("warns before discarding local policy edits and keeps them when the user cancels", async () => {
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
        phase: "BeforeWorktreeArchiveCleanup",
        priority: 1,
        enabled: true,
        decision: "Deny",
        reason_code: "RuleDenied",
        conditions: [{ field: "ActiveRunCount", operator: "GreaterThan", expected: { Count: 0 } }],
      }],
      worktree_rules: [],
    };
    const fetcher = vi.fn<typeof fetch>().mockImplementation(async (input) => {
      const url = new URL(String(input), "http://localhost");
      if (url.pathname === "/api/v1/projects") {
        return jsonResponse({ projects: [{ project_id: projectId, role: "project_admin" }], next_cursor: null });
      }
      if (url.pathname.endsWith("/hook-policy")) {
        return jsonResponse({
          scope: { kind: "project", project_id: projectId, worktree_id: null },
          producer_capabilities: { run_admission: false },
          policy_set_id: "policy-1",
          policy_document: policyDocument,
          effective_policy_document: policyDocument,
          inherited_project_policy_set_id: null,
          draft: {
            draft_id: "draft-1",
            draft_version: 2,
            base_policy_set_id: "policy-1",
            inherited_project_policy_set_id: null,
            expires_at: "2026-10-22T00:00:00.000Z",
            is_stale: false,
            policy_document: policyDocument,
          },
          audit: [],
        });
      }
      if (url.pathname.endsWith("/hook-events/summary")) {
        return jsonResponse({
          metric_version: "hook_execution_summary_v2",
          window: { days: 30, from: "2026-09-01T00:00:00.000Z", to: "2026-10-01T00:00:00.000Z" },
          observed_event_count: 0,
          run_linked_event_count: 0,
          run_state_joined_event_count: 0,
          excluded_incomplete_run_event_count: 0,
          timeout_count: 0,
          duration_total_ms: 0,
          groups: [],
          coverage: { scope: "hook_execution_event_and_task_execution_run_event", status: "partial", reported_percentage: null, instrumented_phases: [], not_yet_instrumented_phases: ["run_admission", "tool"], run_state_join: "no_samples", note: "No producer evidence." },
          formulas: {},
        });
      }
      if (url.pathname.endsWith("/hook-events")) {
        return jsonResponse({ events: [], next_cursor: null, coverage: { scope: "hook_execution_event_ledger", status: "partial", reported_percentage: null, instrumented_phases: [], not_yet_instrumented_phases: ["run_admission", "tool"], note: "No producer evidence." } });
      }
      if (url.pathname.endsWith("/worktrees")) return jsonResponse({ worktrees: [], next_cursor: null });
      return jsonResponse({});
    });
    const confirm = vi.fn().mockReturnValue(false);
    vi.stubGlobal("confirm", confirm);

    render(
      <WorktreeGroupApiProvider getAccessToken={getTestAccessToken} sessionKey="draft-session" fetcher={fetcher}>
        <HookPolicyPage />
      </WorktreeGroupApiProvider>,
    );

    await screen.findByLabelText("Hook 适用阶段");
    fireEvent.click(screen.getByRole("button", { name: "新增" }));
    expect(await screen.findByTestId("hook-unsaved-draft")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "发布版本" })).toBeDisabled();

    fireEvent.change(screen.getByLabelText("策略范围"), { target: { value: "worktree" } });
    expect(confirm).toHaveBeenCalledOnce();
    expect(screen.getByLabelText("策略范围")).toHaveValue("project");
    expect(screen.getByTestId("hook-unsaved-draft")).toBeInTheDocument();

    const beforeUnload = new Event("beforeunload", { cancelable: true });
    window.dispatchEvent(beforeUnload);
    expect(beforeUnload.defaultPrevented).toBe(true);

    fireEvent.click(screen.getByRole("button", { name: "放弃本地更改" }));
    expect(screen.queryByTestId("hook-unsaved-draft")).not.toBeInTheDocument();
    expect(screen.getByText("本地未保存修改已放弃；服务端草稿和生效策略未改变。")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "发布版本" })).toBeEnabled();
  });
});

const hintedProjectId = "22222222-2222-4222-8222-222222222222";
const hintedWorktreeId = "33333333-3333-4333-8333-333333333333";
const otherProjectId = "11111111-1111-4111-8111-111111111111";
const getTestAccessToken = async () => "test-user-jwt";

function jsonResponse(value: unknown): Response {
  return new Response(JSON.stringify(value), { status: 200 });
}

function createScopeFetcher(override?: (url: URL, init?: RequestInit) => Response | Promise<Response> | undefined) {
  return vi.fn<typeof fetch>().mockImplementation(async (input, init) => {
    const url = new URL(String(input), "http://localhost");
    const response = override?.(url, init);
    if (response) return response;
    if (url.pathname === "/api/v1/projects") return jsonResponse({ projects: [{ project_id: hintedProjectId, role: "project_admin" }], next_cursor: null });
    if (url.pathname.endsWith("/worktrees")) return jsonResponse({ worktrees: [{ id: hintedWorktreeId, name: "Authorized checkout" }], next_cursor: null });
    if (/\/hook-policy(?:\/effective)?$/.test(url.pathname)) return jsonResponse({ producer_capabilities: { run_admission: false }, policy_set_id: null, policy_document: null, effective_policy_document: null, draft: null, audit: [] });
    return new Response("unavailable", { status: 503 });
  });
}

function authenticatedPage(fetcher: typeof fetch, sessionKey = "deep-link-session") {
  return <WorktreeGroupApiProvider getAccessToken={getTestAccessToken} sessionKey={sessionKey} fetcher={fetcher}><HookPolicyPage /></WorktreeGroupApiProvider>;
}

function scopeReadCalls(fetcher: ReturnType<typeof createScopeFetcher>) {
  return fetcher.mock.calls.filter(([input]) => /\/hook-(policy|events)/.test(String(input)));
}

describe("HookPolicyPage Run deep links", () => {
  it.each([
    `project_id=${hintedProjectId}`,
    `project_id=bad&worktree_id=${hintedWorktreeId}`,
    `project_id=${hintedProjectId}&project_id=${otherProjectId}&worktree_id=${hintedWorktreeId}`,
    `project_id=${hintedProjectId}&worktree_id=bad`,
  ])("blocks malformed or ambiguous hints without reading a fallback scope: %s", async (query) => {
    navigation.query = query;
    const fetcher = createScopeFetcher();
    render(authenticatedPage(fetcher));
    expect(screen.getByTestId("hook-deep-link-scope")).toHaveTextContent(/阻断/);
    await act(async () => {});
    expect(fetcher).not.toHaveBeenCalled();
    expect(screen.queryByRole("button", { name: "保存草稿" })).not.toBeInTheDocument();
  });

  it("loads only the authorized hinted worktree after both paginated directories match", async () => {
    navigation.query = `project_id=${hintedProjectId}&worktree_id=${hintedWorktreeId}`;
    let resolveWorktrees!: (response: Response) => void;
    const worktreeResponse = new Promise<Response>((resolve) => { resolveWorktrees = resolve; });
    const fetcher = createScopeFetcher((url) => {
      if (url.pathname === "/api/v1/projects" && !url.searchParams.has("cursor")) return jsonResponse({ projects: [{ project_id: otherProjectId, role: "project_admin" }], next_cursor: "page-2" });
      if (url.pathname.endsWith("/worktrees")) return worktreeResponse;
    });
    render(authenticatedPage(fetcher));
    await waitFor(() => expect(fetcher.mock.calls.some(([input]) => String(input).includes(`/projects/${hintedProjectId}/worktrees`))).toBe(true));
    expect(scopeReadCalls(fetcher)).toHaveLength(0);
    expect(screen.queryByRole("button", { name: "保存草稿" })).not.toBeInTheDocument();
    await act(async () => { resolveWorktrees(jsonResponse({ worktrees: [{ id: hintedWorktreeId, name: "Authorized checkout" }], next_cursor: null })); });
    await waitFor(() => expect(fetcher.mock.calls.some(([input]) => String(input).endsWith(`/worktrees/${hintedWorktreeId}/hook-policy/effective`))).toBe(true));
    expect(screen.getByLabelText("策略范围")).toHaveValue("worktree");
    expect(screen.getByTestId("hook-deep-link-scope")).toHaveTextContent("Authorized checkout");
    expect(scopeReadCalls(fetcher).every(([input]) => !String(input).includes(otherProjectId))).toBe(true);
    expect(scopeReadCalls(fetcher).some(([input]) => String(input).includes(`/projects/${hintedProjectId}/hook-policy`))).toBe(false);
    expect(fetcher.mock.calls.every(([, init]) => !init?.method || init.method === "GET")).toBe(true);
    const directoryCalls = fetcher.mock.calls.filter(([input]) => /\/projects\?|\/worktrees\?/.test(String(input)));
    expect(directoryCalls.every(([input]) => new URL(String(input), "http://localhost").searchParams.get("limit") === "200")).toBe(true);
  });

  it.each(["project", "worktree"])("blocks a hint absent from its authorized %s directory", async (missing) => {
    navigation.query = `project_id=${hintedProjectId}&worktree_id=${hintedWorktreeId}`;
    const fetcher = createScopeFetcher((url) => {
      if (missing === "project" && url.pathname === "/api/v1/projects") return jsonResponse({ projects: [{ project_id: otherProjectId, role: "project_admin" }], next_cursor: null });
      if (missing === "worktree" && url.pathname.endsWith("/worktrees")) return jsonResponse({ worktrees: [], next_cursor: null });
    });
    render(authenticatedPage(fetcher));
    await waitFor(() => expect(screen.getByTestId("hook-deep-link-scope")).toHaveTextContent("不在"));
    expect(scopeReadCalls(fetcher)).toHaveLength(0);
  });

  it("revalidates hints against a replacement session before further policy or event reads", async () => {
    navigation.query = `project_id=${hintedProjectId}&worktree_id=${hintedWorktreeId}`;
    let sessionChanged = false;
    let resolveProjects!: (response: Response) => void;
    const nextProjects = new Promise<Response>((resolve) => { resolveProjects = resolve; });
    const fetcher = createScopeFetcher((url) => sessionChanged && url.pathname === "/api/v1/projects" ? nextProjects : undefined);
    const view = render(authenticatedPage(fetcher));
    await waitFor(() => expect(screen.getByTestId("hook-deep-link-scope")).toHaveTextContent("已通过"));
    await waitFor(() => expect(scopeReadCalls(fetcher)).toHaveLength(3));
    fetcher.mockClear();
    sessionChanged = true;
    view.rerender(authenticatedPage(fetcher, "replacement-session"));
    await waitFor(() => expect(fetcher).toHaveBeenCalled());
    expect(screen.getByTestId("hook-deep-link-scope")).toHaveTextContent("正在");
    expect(scopeReadCalls(fetcher)).toHaveLength(0);
    await act(async () => { resolveProjects(jsonResponse({ projects: [], next_cursor: null })); });
    await waitFor(() => expect(screen.getByTestId("hook-deep-link-scope")).toHaveTextContent("不在"));
    expect(scopeReadCalls(fetcher)).toHaveLength(0);
  });

  it("stops at twenty directory pages and keeps scope reads blocked", async () => {
    navigation.query = `project_id=${hintedProjectId}&worktree_id=${hintedWorktreeId}`;
    let pages = 0;
    const fetcher = createScopeFetcher((url) => url.pathname === "/api/v1/projects" ? jsonResponse({ projects: [], next_cursor: `page-${++pages}` }) : undefined);
    render(authenticatedPage(fetcher));
    await waitFor(() => expect(screen.getByTestId("hook-deep-link-scope")).toHaveTextContent("尚未覆盖"));
    expect(pages).toBe(20);
    expect(scopeReadCalls(fetcher)).toHaveLength(0);
  });

  it("ignores a late lookup page and sends no next page after hint cleanup", async () => {
    navigation.query = `project_id=${hintedProjectId}&worktree_id=${hintedWorktreeId}`;
    let resolveProjects!: (response: Response) => void;
    const oldPage = new Promise<Response>((resolve) => { resolveProjects = resolve; });
    const fetcher = createScopeFetcher((url) => url.pathname === "/api/v1/projects" ? oldPage : undefined);
    const view = render(authenticatedPage(fetcher));
    await waitFor(() => expect(fetcher).toHaveBeenCalledTimes(1));
    navigation.query = `project_id=bad&worktree_id=${hintedWorktreeId}`;
    view.rerender(authenticatedPage(fetcher));
    await act(async () => { resolveProjects(jsonResponse({ projects: [], next_cursor: "late-next-page" })); });
    expect(fetcher).toHaveBeenCalledTimes(1);
    expect(scopeReadCalls(fetcher)).toHaveLength(0);
    expect(screen.getByTestId("hook-deep-link-scope")).toHaveTextContent("格式无效");
  });

  it("requires explicit hint clearing before restoring ordinary authorized navigation", async () => {
    navigation.query = `project_id=bad&worktree_id=${hintedWorktreeId}&view=visual`;
    const fetcher = createScopeFetcher();
    const view = render(authenticatedPage(fetcher));
    fireEvent.click(screen.getByRole("button", { name: "清除深链并按授权目录手动选择" }));
    expect(navigation.router.replace).toHaveBeenCalledWith("/settings/advanced/hooks?view=visual", { scroll: false });
    await waitFor(() => expect(fetcher.mock.calls.some(([input]) => String(input).endsWith(`/projects/${hintedProjectId}/hook-policy`))).toBe(true));
    expect(screen.queryByTestId("hook-deep-link-scope")).not.toBeInTheDocument();
    navigation.query = "view=visual";
    view.rerender(authenticatedPage(fetcher));
    fetcher.mockClear();
    navigation.query = `project_id=bad&worktree_id=${hintedWorktreeId}&view=visual`;
    view.rerender(authenticatedPage(fetcher));
    await act(async () => {});
    expect(screen.getByTestId("hook-deep-link-scope")).toHaveTextContent("格式无效");
    expect(scopeReadCalls(fetcher)).toHaveLength(0);
  });

  it("does not send a draft when hints become unresolved while its digest is pending", async () => {
    navigation.query = `project_id=${hintedProjectId}&worktree_id=${hintedWorktreeId}`;
    const policyDocument = {
      schema_version: 1, evaluator_api_version: 2, tenant_id: Array(16).fill(1), project_id: Array(16).fill(2),
      worktree_id: null, project_version: 1, worktree_version: null, digest: Array(32).fill(0), project_rules: [], worktree_rules: [],
    };
    let resolveDigest!: (digest: ArrayBuffer) => void;
    const digest = vi.fn(() => new Promise<ArrayBuffer>((resolve) => { resolveDigest = resolve; }));
    vi.stubGlobal("crypto", { subtle: { digest }, randomUUID: () => "44444444-4444-4444-8444-444444444444" });
    const fetcher = createScopeFetcher((url) => url.pathname.endsWith("/hook-policy/effective")
      ? jsonResponse({ producer_capabilities: { run_admission: false }, policy_set_id: "policy-1", policy_document: null, effective_policy_document: policyDocument, draft: null, audit: [] })
      : undefined);
    const view = render(authenticatedPage(fetcher));
    const save = await screen.findByRole("button", { name: "保存草稿" });
    fireEvent.click(save);
    await waitFor(() => expect(digest).toHaveBeenCalledTimes(1));
    navigation.query = `project_id=bad&worktree_id=${hintedWorktreeId}`;
    view.rerender(authenticatedPage(fetcher));
    await act(async () => { resolveDigest(new Uint8Array(32).buffer); });
    expect(fetcher.mock.calls.some(([, init]) => init?.method === "PUT")).toBe(false);
    expect(screen.getByTestId("hook-deep-link-scope")).toHaveTextContent("格式无效");
    expect(screen.queryByRole("button", { name: "保存草稿" })).not.toBeInTheDocument();
  });
});

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (suite:Function {name:"HookPolicyPage unauthenticated state tests"}),
      (render:Function {name:"Testing Library render"})
CREATE (eventPanel:Function {name:"Hook execution event panel UI case",type:"function",language:"typescript",visibility:"private",complexity:"moderate"})
CREATE (phaseEditor:Function {name:"Run admission phase remains unavailable UI case",type:"function",language:"typescript",visibility:"private",complexity:"moderate"})
CREATE (producerCoverage:Function {name:"Run admission producer capability refresh case",type:"function",language:"typescript",visibility:"private",complexity:"moderate"})
CREATE (suite)-[:CONTAINS]->(eventPanel),(eventPanel)-[:CALLS]->(render),
       (suite)-[:CONTAINS]->(phaseEditor),(phaseEditor)-[:CALLS]->(render)
CREATE (suite)-[:CONTAINS]->(producerCoverage),(producerCoverage)-[:CALLS]->(render);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (file:File {name:"frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.test.tsx"})
CREATE (json:Function {name:"jsonResponse",type:"function",signature:"jsonResponse(value): Response"}),
       (fetcher:Function {name:"createScopeFetcher",type:"function",signature:"createScopeFetcher(override)"}),
       (page:Function {name:"authenticatedPage",type:"function",signature:"authenticatedPage(fetcher, sessionKey)"}),
       (reads:Function {name:"scopeReadCalls",type:"function",signature:"scopeReadCalls(fetcher)"}),
       (token:Function {name:"getTestAccessToken",type:"function",signature:"getTestAccessToken(): Promise<string>"}),
       (navigation:Variable {name:"navigation",type:"variable"}),
       (suite:Function {name:"HookPolicyPage Run deep links",type:"function",signature:"describe('HookPolicyPage Run deep links')"}),
       (file)-[:CONTAINS]->(json),(file)-[:CONTAINS]->(fetcher),(file)-[:CONTAINS]->(page),
       (file)-[:CONTAINS]->(reads),(file)-[:CONTAINS]->(token),(file)-[:CONTAINS]->(navigation),(file)-[:CONTAINS]->(suite),
       (fetcher)-[:CALLS]->(json),(suite)-[:CALLS]->(fetcher),(suite)-[:CALLS]->(page),
       (suite)-[:CALLS]->(reads),(suite)-[:CALLS]->(json),(suite)-[:USES]->(navigation);
*/
