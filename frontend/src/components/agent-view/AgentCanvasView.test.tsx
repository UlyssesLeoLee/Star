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
});
