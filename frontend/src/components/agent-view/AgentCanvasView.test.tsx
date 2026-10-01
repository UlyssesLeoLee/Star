// =====================================================================
// AgentCanvasView.test.tsx — smoke 测试
// =====================================================================
// 覆盖:
//   1. 渲染空 worktree 场景 (无 worktree 节点, 只画 agent)
//   2. 渲染有 worktree 场景
//   3. minimap / toolbar / status bar 渲染
//   4. zoom 数字显示
//   5. 节点 testid 出现
// =====================================================================

import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { AgentCanvasView } from "./AgentCanvasView";
import { useStore } from "@/lib/store";
import { I18nProvider } from "@/lib/i18n";

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider>{ui}</I18nProvider>);
import type { AgentSession, Worktree, WorkItem, AgentCanvas } from "@/types/ids";
import type { AgentCanvas as AgentCanvasType } from "@/lib/agent-view/types";

// ---- mock next/navigation ----
vi.mock("next/navigation", () => ({
  usePathname: () => "/agent-view",
  useSearchParams: () => new URLSearchParams(""),
  useRouter: () => ({ replace: vi.fn(), push: vi.fn() }),
}));

const baseAgent: AgentSession = {
  id: "ag-001",
  tenant_id: "ten-acme",
  project_id: "prj-physis",
  worktree_id: "wt-001",
  agent_kind: "claude-sonnet",
  status: "executing",
  current_step: "tool.call:grep",
  token_usage: { input: 1000, output: 200, total: 1200 },
  cost_summary: { usd: 0.5, budget_usd: 5.0 },
  started_at: "2026-09-05T10:00:00Z",
};

const baseWorktree: Worktree = {
  id: "wt-001",
  tenant_id: "ten-acme",
  project_id: "prj-physis",
  name: "wt-1",
  branch: "feat/test",
  base_branch: "main",
  status: "active",
  lock_version: 0,
  last_event_at: "2026-09-05T10:00:00Z",
  created_at: "2026-09-05T09:00:00Z",
};

const baseWorkItems: WorkItem[] = [
  {
    id: "wi-001",
    tenant_id: "ten-acme",
    project_id: "prj-physis",
    key: "PHYSIS-1",
    title: "Task 1",
    description: "",
    kind: "task",
    status: "in_progress",
    priority: "p0",
    reporter_id: "usr-001",
    labels: [],
    workflow_id: "wf-default",
    worktree_id: "wt-001",
    created_at: "2026-09-05T08:00:00Z",
    updated_at: "2026-09-05T08:00:00Z",
  },
];

beforeEach(() => {
  // 重置 store 状态到 seed
  useStore.setState({ workItems: baseWorkItems });
});

describe("AgentCanvasView", () => {
  it("渲染空 worktree 场景: 仅 agent 节点, 0 connector", () => {
    const canvas: AgentCanvasType = {
      agentId: "ag-001",
      nodes: [
        { id: "n-agent-ag-001", kind: "agent", x: 0, y: 0, width: 220, height: 110, ref: { kind: "agent", agentId: "ag-001" } },
      ],
      connectors: [],
      viewport: { x: 0, y: 0, zoom: 1 },
      derivedAt: "2026-09-05T11:00:00Z",
    };
    renderWithI18n(<AgentCanvasView canvas={canvas} agent={baseAgent} worktree={null} />);
    expect(screen.getByTestId("agent-canvas-container")).toBeTruthy();
    expect(screen.getByTestId("agent-canvas-svg")).toBeTruthy();
    expect(screen.getByTestId("agent-canvas-toolbar")).toBeTruthy();
    expect(screen.getByTestId("agent-canvas-minimap")).toBeTruthy();
    expect(screen.getByTestId("agent-canvas-statusbar")).toBeTruthy();
    expect(screen.getByTestId("agent-canvas-node-n-agent-ag-001")).toBeTruthy();
  });

  it("渲染完整场景: agent + worktree + 1 wi + 2 connector", () => {
    const canvas: AgentCanvasType = {
      agentId: "ag-001",
      nodes: [
        { id: "n-agent-ag-001", kind: "agent", x: 0, y: 0, width: 220, height: 110, ref: { kind: "agent", agentId: "ag-001" } },
        { id: "n-wt-wt-001", kind: "worktree", x: 300, y: 0, width: 240, height: 80, ref: { kind: "worktree", worktreeId: "wt-001" } },
        { id: "n-wi-wi-001", kind: "work_item", x: 100, y: 300, width: 180, height: 64, ref: { kind: "work_item", workItemId: "wi-001" } },
      ],
      connectors: [
        { id: "c1", fromNodeId: "n-agent-ag-001", toNodeId: "n-wt-wt-001", color: "#2f81f7", label: "executes on" },
        { id: "c2", fromNodeId: "n-wt-wt-001", toNodeId: "n-wi-wi-001", color: "#2f81f7", label: "in_progress" },
      ],
      viewport: { x: 0, y: 0, zoom: 1 },
      derivedAt: "2026-09-05T11:00:00Z",
    };
    renderWithI18n(<AgentCanvasView canvas={canvas} agent={baseAgent} worktree={baseWorktree} />);
    expect(screen.getByTestId("agent-canvas-node-n-agent-ag-001")).toBeTruthy();
    expect(screen.getByTestId("agent-canvas-node-n-wt-wt-001")).toBeTruthy();
    expect(screen.getByTestId("agent-canvas-node-n-wi-wi-001")).toBeTruthy();
    expect(screen.getByTestId("agent-canvas-connector-c1")).toBeTruthy();
    expect(screen.getByTestId("agent-canvas-connector-c2")).toBeTruthy();
  });

  it("zoom 数字显示在 toolbar", () => {
    const canvas: AgentCanvasType = {
      agentId: "ag-001",
      nodes: [],
      connectors: [],
      viewport: { x: 0, y: 0, zoom: 0.8 },
      derivedAt: "2026-09-05T11:00:00Z",
    };
    renderWithI18n(<AgentCanvasView canvas={canvas} agent={baseAgent} worktree={null} />);
    expect(screen.getByTestId("agent-canvas-zoom").textContent).toBe("80%");
  });

  it("status bar 显示节点数", () => {
    const canvas: AgentCanvasType = {
      agentId: "ag-001",
      nodes: [
        { id: "n-agent-ag-001", kind: "agent", x: 0, y: 0, width: 220, height: 110, ref: { kind: "agent", agentId: "ag-001" } },
      ],
      connectors: [],
      viewport: { x: 0, y: 0, zoom: 1 },
      derivedAt: "2026-09-05T11:00:00Z",
    };
    renderWithI18n(<AgentCanvasView canvas={canvas} agent={baseAgent} worktree={null} />);
    const statusbar = screen.getByTestId("agent-canvas-statusbar");
    expect(statusbar.textContent).toContain("nodes 1");
    expect(statusbar.textContent).toContain("connectors 0");
  });


describe("AgentCanvasView 注释层 (per 2026-10-01 OOB 恢复无限画布画笔)", () => {
  const emptyCanvas: AgentCanvasType = {
    agentId: "ag-001",
    nodes: [
      { id: "n-agent-ag-001", kind: "agent", x: 0, y: 0, width: 220, height: 110, ref: { kind: "agent", agentId: "ag-001" } },
    ],
    connectors: [],
    viewport: { x: 0, y: 0, zoom: 1 },
    derivedAt: "2026-10-01T00:00:00Z",
  };
  const noop = async () => undefined;

  it("E. readOnly=true 默认无 sticky/text/shape/brush 按钮", () => {
    renderWithI18n(<AgentCanvasView canvas={emptyCanvas} agent={baseAgent} worktree={null} />);
    expect(screen.queryByTestId("agent-canvas-tool-sticky")).toBeNull();
    expect(screen.queryByTestId("agent-canvas-tool-text")).toBeNull();
    expect(screen.queryByTestId("agent-canvas-tool-shape")).toBeNull();
    expect(screen.queryByTestId("agent-canvas-tool-brush")).toBeNull();
    expect(screen.queryByTestId("agent-canvas-tool-connector")).toBeNull();
  });

  it("F. readOnly=false + 提供回调 时 sticky/text/shape/brush/connector 按钮出现", () => {
    const noop = async () => undefined;
    renderWithI18n(
      <AgentCanvasView
        canvas={emptyCanvas}
        agent={baseAgent}
        worktree={null}
        readOnly={false}
        onCreateAnnotation={noop}
        onDeleteAnnotation={noop}
        onCreateFreeConnector={noop}
      />,
    );
    expect(screen.getByTestId("agent-canvas-tool-sticky")).toBeTruthy();
    expect(screen.getByTestId("agent-canvas-tool-text")).toBeTruthy();
    expect(screen.getByTestId("agent-canvas-tool-shape")).toBeTruthy();
    expect(screen.getByTestId("agent-canvas-tool-brush")).toBeTruthy();
    expect(screen.getByTestId("agent-canvas-tool-connector")).toBeTruthy();
  });

  it("G. brush tool 选中时 palette / size 控件展开", () => {
    const noop = async () => undefined;
    renderWithI18n(
      <AgentCanvasView
        canvas={emptyCanvas}
        agent={baseAgent}
        worktree={null}
        readOnly={false}
        onCreateAnnotation={noop}
      />,
    );
    // Brush 工具未选时不应有 palette/size
    expect(screen.queryByTestId("agent-canvas-brush-palette")).toBeNull();
    expect(screen.queryByTestId("agent-canvas-brush-size")).toBeNull();
  });

  it("H. annotation 入参可被 SVG 渲染为 annotation 元素", () => {
    const annotations = [
      {
        id: "ann-001",
        kind: "sticky_note" as const,
        x: 200,
        y: 200,
        width: 180,
        height: 100,
        created_at: "2026-10-01T00:00:00Z",
        created_by: "usr-001",
        content: { color: "#f9d77e", text: "Memo" },
      },
    ];
    renderWithI18n(
      <AgentCanvasView
        canvas={emptyCanvas}
        agent={baseAgent}
        worktree={null}
        annotations={annotations}
      />,
    );
    expect(screen.getByTestId("annotation-ann-001")).toBeTruthy();
  });

  it("I. free connector 入参可被 SVG 渲染", () => {
    const annotations = [
      {
        id: "ann-001",
        kind: "sticky_note" as const,
        x: 100,
        y: 100,
        width: 80,
        height: 60,
        created_at: "2026-10-01T00:00:00Z",
        created_by: "usr-001",
        content: { color: "#f9d77e", text: "" },
      },
      {
        id: "ann-002",
        kind: "sticky_note" as const,
        x: 400,
        y: 400,
        width: 80,
        height: 60,
        created_at: "2026-10-01T00:00:00Z",
        created_by: "usr-001",
        content: { color: "#a3d9ff", text: "" },
      },
    ];
    const freeConnectors = [
      { id: "fc-001", fromAnnotationId: "ann-001", toAnnotationId: "ann-002", color: "#00f0ff" },
    ];
    renderWithI18n(
      <AgentCanvasView
        canvas={emptyCanvas}
        agent={baseAgent}
        worktree={null}
        annotations={annotations}
        freeConnectors={freeConnectors}
      />,
    );
    expect(screen.getByTestId("agent-canvas-free-connector-fc-001")).toBeTruthy();
  });
});

  it("J. minimap 显示 annotation 信标 (per 任务 #7)", () => {
    const annotations = [
      {
        id: "ann-stick",
        kind: "sticky_note" as const,
        x: 100,
        y: 100,
        width: 80,
        height: 60,
        created_at: "2026-10-01T00:00:00Z",
        created_by: "usr-001",
        content: { color: "#f9d77e", text: "" },
      },
      {
        id: "ann-text",
        kind: "text" as const,
        x: 400,
        y: 400,
        width: 100,
        height: 40,
        created_at: "2026-10-01T00:00:00Z",
        created_by: "usr-001",
        content: { text: "Note" },
      },
      {
        id: "ann-path",
        kind: "path" as const,
        x: 200,
        y: 300,
        width: 60,
        height: 40,
        created_at: "2026-10-01T00:00:00Z",
        created_by: "usr-001",
        content: { path_data: "M 0 0 L 30 30", brush_size: 4, brush_color: "#00f0ff" },
      },
    ];
    const canvas2: AgentCanvasType = {
      agentId: "ag-002",
      nodes: [
        { id: "n-agent-ag-002", kind: "agent", x: 0, y: 0, width: 220, height: 110, ref: { kind: "agent", agentId: "ag-002" } },
      ],
      connectors: [],
      viewport: { x: 0, y: 0, zoom: 1 },
      derivedAt: "2026-10-01T00:00:00Z",
    };
    renderWithI18n(
      <AgentCanvasView
        canvas={canvas2}
        agent={baseAgent}
        worktree={null}
        annotations={annotations}
      />,
    );
    expect(screen.getByTestId("agent-canvas-minimap-ann-ann-stick")).toBeTruthy();
    expect(screen.getByTestId("agent-canvas-minimap-ann-ann-text")).toBeTruthy();
    expect(screen.getByTestId("agent-canvas-minimap-ann-ann-path")).toBeTruthy();
  });

  it("K. eraser 工具按钮仅在 write mode + 有回调时显示 (per 任务 #6)", () => {
    const canvas2: AgentCanvasType = {
      agentId: "ag-003",
      nodes: [
        { id: "n-agent-ag-003", kind: "agent", x: 0, y: 0, width: 220, height: 110, ref: { kind: "agent", agentId: "ag-003" } },
      ],
      connectors: [],
      viewport: { x: 0, y: 0, zoom: 1 },
      derivedAt: "2026-10-01T00:00:00Z",
    };
    renderWithI18n(
      <AgentCanvasView
        canvas={canvas2}
        agent={baseAgent}
        worktree={null}
        readOnly={false}
        onCreateAnnotation={async () => undefined}
        onDeleteAnnotation={async () => undefined}
      />,
    );
    expect(screen.getByTestId("agent-canvas-tool-eraser")).toBeTruthy();
  });
});
