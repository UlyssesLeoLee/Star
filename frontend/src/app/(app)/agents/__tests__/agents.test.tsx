/*
@cypher schema=1 source_sha256=e089a441ecedd4f841fff4eb2470522536a5829e29b63758b0db1f1dd23b7696
MERGE (self:File {path:"frontend/src/app/(app)/agents/__tests__/agents.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/app/(app)/agents/__tests__/agents.test.tsx::Agent 中心真实运行时 API",kind:"suite"})
MERGE (page:File {path:"frontend/src/app/(app)/agents/page.tsx"})
MERGE (api:File {path:"frontend/src/lib/agents/agentsApi.ts"})
MERGE (i18n:File {path:"frontend/src/lib/i18n"})
MERGE (server:File {path:"frontend/src/mocks/server.ts"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (self)-[:IMPORTS]->(page)
MERGE (self)-[:IMPORTS]->(api)
MERGE (self)-[:IMPORTS]->(i18n)
MERGE (self)-[:IMPORTS]->(server)
@endcypher
*/
// =====================================================================
// agents.test.tsx — Agent 中心真实运行时 API 行为测试 (per 2026-11-17)
//
// 覆盖 (任务书验收清单):
//   1. 初始 = 加载中态 (不预填 mock 数据)
//   2. 成功 → 渲染真实 agent + 数据源说明
//   3. 空数据 → 诚实空态 "暂无 agent"
//   4. 后端不可达 → "未接入" + Argo CLI 未接入, 且 mock agent 名字绝不出现
//   5. 401 → 未授权态
//   6. 未接入/加载中时按钮禁用; 已接入后可用
//   7. SYNC / RUN VERIFY 走真实动作并展示结果
//
// 方式: 页面 fetch("/api/v1/*") 由 MSW 拦截 (per 守门 #25: 测试不发真 HTTP);
//   3D 效果组件 (R3F/Three.js) 以 stub 替换 (jsdom 无 WebGL)。
// =====================================================================

import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { render, screen, cleanup, fireEvent, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { http, HttpResponse, delay } from "msw";

// ── 重 3D 组件 stub (jsdom 无 WebGL; 断言只关心页面数据流与禁用态) ──
vi.mock("@/components/effects/AnimeCelShaderCanvas", () => ({
  AnimeCelShaderCanvas: () => <div data-testid="agents-3d-stub" />,
}));
vi.mock("@/components/effects/TacticalCore3D", () => ({
  TacticalCore3D: () => <div data-testid="agents-core3d-stub" />,
}));
vi.mock("@/components/effects/CelCard3D", () => ({
  CelCard3D: ({ children, className }: { children?: ReactNode; className?: string }) => (
    <div className={className}>{children}</div>
  ),
}));
vi.mock("@/components/effects/Cel3DUI", () => ({
  CelButton3D: (props: { label: string; onClick?: () => void; disabled?: boolean }) => (
    <button type="button" disabled={props.disabled ?? false} onClick={props.onClick}>
      {props.label}
    </button>
  ),
  CelToggle3D: () => <div />,
  CelBeacon3D: () => <div />,
  CelDial3D: () => <div />,
}));

import AgentsPage from "../page";
import { I18nProvider } from "@/lib/i18n";
import { server } from "@/mocks/server";

function renderWithI18n(ui: ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

/** star-api-rest GET /api/v1/agents 的真实响应形状 (RestResponse 信封) */
const REAL_AGENTS_PAYLOAD = {
  data: {
    total: 2,
    agents: [
      {
        id: "00000000-0000-0000-0000-00000000a101",
        name: "SA-01 LLM 推理",
        avatar_url: null,
        role: "Worker",
        kind: "Sa01",
        domain: "Player",
        status: "Running",
        token_usage: 42000,
        token_budget: 1200000,
        parent_session_id: null,
        pipeline_agent_ids: [],
        started_at: "2026-11-17T02:00:00Z",
        tenant_id: "00000000-0000-0000-0000-000000000000",
        version: 3,
      },
      {
        id: "00000000-0000-0000-0000-00000000a105",
        name: "SA-05 评审审查",
        avatar_url: null,
        role: "Reviewer",
        kind: "Sa05",
        domain: "Social",
        status: "Stopped",
        token_usage: 0,
        token_budget: 1200000,
        parent_session_id: null,
        pipeline_agent_ids: [],
        started_at: "2026-11-17T01:30:00Z",
        tenant_id: "00000000-0000-0000-0000-000000000000",
        version: 2,
      },
    ],
    note: "内存样例, 宿主运行时接入后替换 (无 DB)",
  },
  meta: { request_id: "req_stub", timestamp: "2026-11-17T02:00:00Z", version: "v1" },
};

const ARGO_PAYLOAD = { data: { supported_cli: ["cargo", "git"] } };

/** 旧 mock fixture 里的 agent 名字 — 任何诚实态下都不得出现 */
const MOCK_AGENT_NAME_MARKERS = [
  /Ulysses-CLI/,
  /Physis-builder/,
  /Star-frontend/,
  /Doc-scribe/,
  /Review-bot/,
];

const ROW_1_ID = "00000000-0000-0000-0000-00000000a101";
const ROW_2_ID = "00000000-0000-0000-0000-00000000a105";

function stubOkEndpoints(agentsPayload: unknown = REAL_AGENTS_PAYLOAD) {
  server.use(
    http.get("/api/v1/agents", () => HttpResponse.json(agentsPayload)),
    http.get("/api/v1/argo/nodes", () => HttpResponse.json(ARGO_PAYLOAD)),
    http.get("/api/v1/health", () => HttpResponse.json({ status: "ok" })),
  );
}

describe("Agent 中心真实运行时 API", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });
  afterEach(() => {
    cleanup();
  });

  it("加载中态: 初始为 loading, 不预填 mock 数据", async () => {
    server.use(
      http.get("/api/v1/agents", async () => {
        await delay(120);
        return HttpResponse.json(REAL_AGENTS_PAYLOAD);
      }),
      http.get("/api/v1/argo/nodes", async () => {
        await delay(120);
        return HttpResponse.json(ARGO_PAYLOAD);
      }),
    );
    renderWithI18n(<AgentsPage />);
    // 初始态 = 加载中 (不是 mock 列表)
    expect(screen.getByTestId("status-notice-loading")).toBeInTheDocument();
    expect(screen.queryByTestId(`agent-row-${ROW_1_ID}`)).toBeNull();
    for (const marker of MOCK_AGENT_NAME_MARKERS) {
      expect(screen.queryByText(marker)).toBeNull();
    }
    // 请求完成后进入就绪态 (真实行出现)
    expect(await screen.findByTestId(`agent-row-${ROW_1_ID}`)).toBeInTheDocument();
  });

  it("成功: 渲染真实 agent + 数据源说明, 不出现 mock 名字", async () => {
    stubOkEndpoints();
    renderWithI18n(<AgentsPage />);

    expect(await screen.findByTestId(`agent-row-${ROW_1_ID}`)).toBeInTheDocument();
    expect(screen.getByTestId(`agent-row-${ROW_2_ID}`)).toBeInTheDocument();
    expect(screen.getByText("SA-01 LLM 推理")).toBeInTheDocument();
    expect(screen.getByText("SA-05 评审审查")).toBeInTheDocument();
    // 后端自带数据源说明原样展示 (诚实标注, 不吞掉)
    expect(screen.getByTestId("agents-data-note")).toHaveTextContent("内存样例, 宿主运行时接入后替换");
    // Argo CLI 徽章渲染真实 supported_cli
    expect(screen.getByTestId("agents-cli-badge")).toHaveTextContent("cargo");
    // 旧 mock 名字绝不出现
    for (const marker of MOCK_AGENT_NAME_MARKERS) {
      expect(screen.queryByText(marker)).toBeNull();
    }
    // SESSIONS 计数 = 真实行数 2 (不再硬编码 5)
    expect(screen.getByText("2 SESSIONS")).toBeInTheDocument();
  });

  it("空数据 → 诚实空态", async () => {
    stubOkEndpoints({
      data: { total: 0, agents: [], note: null },
      meta: { request_id: "req_stub", timestamp: "2026-11-17T02:00:00Z", version: "v1" },
    });
    renderWithI18n(<AgentsPage />);

    expect(await screen.findByTestId("status-notice-empty")).toBeInTheDocument();
    expect(screen.getByText("暂无 agent")).toBeInTheDocument();
    // 空态下不渲染表格
    expect(screen.queryByTestId(`agent-row-${ROW_1_ID}`)).toBeNull();
  });

  it("后端不可达 → 未接入, 且 mock agent 名字绝不出现", async () => {
    server.use(
      http.get("/api/v1/agents", () => HttpResponse.error()),
      http.get("/api/v1/argo/nodes", () => HttpResponse.error()),
    );
    renderWithI18n(<AgentsPage />);

    // 主列表未接入 + Argo CLI 未接入 (两处都诚实标注)
    expect(await screen.findByText("未接入 — 真实 Agent 运行时不可达")).toBeInTheDocument();
    expect(screen.getByTestId("agents-argo-unavailable")).toHaveTextContent("Argo CLI 未接入");
    expect(screen.getByTestId("agents-retry")).toBeInTheDocument();
    // 关键纪律: 失败绝不回落 mock 数据
    for (const marker of MOCK_AGENT_NAME_MARKERS) {
      expect(screen.queryByText(marker)).toBeNull();
    }
    expect(screen.queryByTestId(`agent-row-${ROW_1_ID}`)).toBeNull();
  });

  it("401 → 未授权态", async () => {
    server.use(
      http.get("/api/v1/agents", () =>
        HttpResponse.json({ error: "BEARER_MISSING_HEADER" }, { status: 401 }),
      ),
      http.get("/api/v1/argo/nodes", () => HttpResponse.json(ARGO_PAYLOAD)),
    );
    renderWithI18n(<AgentsPage />);

    expect(await screen.findByText("未授权 — Agent 运行时拒绝访问")).toBeInTheDocument();
    expect(screen.getByTestId("status-notice-unauthorized")).toBeInTheDocument();
  });

  it("未接入/加载中时按钮禁用; 已接入后可用", async () => {
    server.use(
      http.get("/api/v1/agents", () => HttpResponse.error()),
      http.get("/api/v1/argo/nodes", () => HttpResponse.error()),
    );
    const { unmount } = renderWithI18n(<AgentsPage />);
    await screen.findByText("未接入 — 真实 Agent 运行时不可达");
    // 运行时不可达 → 两个操作按钮禁用 + 原因说明
    expect(screen.getByText("01 // SYNC WORKTREE")).toBeDisabled();
    expect(screen.getByText("02 // RUN VERIFY")).toBeDisabled();
    expect(screen.getByTestId("agents-actions-disabled-hint")).toBeInTheDocument();
    unmount();

    // 已接入 → 按钮可用 (重新挂载, 模拟后端恢复)
    stubOkEndpoints();
    renderWithI18n(<AgentsPage />);
    await screen.findByTestId(`agent-row-${ROW_1_ID}`);
    expect(screen.getByText("01 // SYNC WORKTREE")).toBeEnabled();
    expect(screen.getByText("02 // RUN VERIFY")).toBeEnabled();
    expect(screen.queryByTestId("agents-actions-disabled-hint")).toBeNull();
  });

  it("SYNC 与 RUN VERIFY 走真实动作并展示结果", async () => {
    let agentsCalls = 0;
    let healthCalls = 0;
    server.use(
      http.get("/api/v1/agents", () => {
        agentsCalls += 1;
        return HttpResponse.json(REAL_AGENTS_PAYLOAD);
      }),
      http.get("/api/v1/argo/nodes", () => HttpResponse.json(ARGO_PAYLOAD)),
      http.get("/api/v1/health", () => {
        healthCalls += 1;
        return HttpResponse.json({ status: "ok" });
      }),
    );
    renderWithI18n(<AgentsPage />);
    await screen.findByTestId(`agent-row-${ROW_1_ID}`);
    expect(agentsCalls).toBe(1);

    // RUN VERIFY → ping /api/v1/health, 展示真实结果
    fireEvent.click(screen.getByText("02 // RUN VERIFY"));
    await waitFor(() => expect(screen.getByTestId("agents-action-result")).toHaveTextContent("健康检查通过"));
    expect(healthCalls).toBe(1);

    // SYNC → 重新拉取花名册 (第 2 次 GET /api/v1/agents)
    fireEvent.click(screen.getByText("01 // SYNC WORKTREE"));
    await waitFor(() => expect(screen.getByTestId("agents-action-result")).toHaveTextContent("2 个 agent"));
    expect(agentsCalls).toBe(2);
  });
});
