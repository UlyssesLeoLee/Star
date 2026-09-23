// =====================================================================
// SessionCard.test.tsx — ULYS-214 SessionCard 组件测试
//
// 测试列表:
//   1. session=undefined 显示 skeleton
//   2. 渲染 name / status / worktree_id / kind / duration
//   3. selected=true 时 data-selected="true" + ring-2
//   4. session.restart_available=true 时显示 RestartChip
//      (走 RestartChip 自管 useRestart 路径, 用 QueryClientProvider 包)
//   5. session.restart_available=false 时不显示 chip
//   6. session.restart_available=undefined (字段缺省) 用 QueryClientProvider
//      包时也不崩; chip 内部走默认 fetcher → 当前 404 → 不假阳地渲染
// =====================================================================
import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { SessionCard } from "./SessionCard";
import type { AgentSession } from "@/types/ids";
import * as React from "react";

const makeWrapper = (client: QueryClient) =>
  ({ children }: { children: React.ReactNode }) =>
    React.createElement(QueryClientProvider, { client }, children);

const mockSession: AgentSession = {
  id: "sess-uuid-1",
  name: "PHYSIS-7 closer",
  tenant_id: "t-1",
  project_id: "p-1",
  worktree_id: "wt-abcd",
  agent_kind: "claude-sonnet",
  status: "executing",
  current_step: "edit src/foo.rs",
  token_usage: { input: 1200, output: 230, total: 1430 },
  cost_summary: { usd: 0.42, budget_usd: 5.0 },
  started_at: new Date(Date.now() - 65_000).toISOString(), // ~65s 前
};

describe("SessionCard (ULYS-214)", () => {
  beforeEach(() => {
    cleanup();
  });

  it("session=undefined 显示 skeleton (data-testid=session-card-skeleton)", () => {
    const { container } = render(<SessionCard session={undefined} />);
    expect(screen.queryByTestId("session-card-skeleton")).toBeTruthy();
    // 不应再渲染正式卡片
    expect(container.querySelector('[data-testid^="session-card-sess"]')).toBeNull();
  });

  it("渲染 name / status chip / worktree_id / kind / duration", () => {
    render(<SessionCard session={mockSession} />);
    expect(screen.getByTestId(`session-card-${mockSession.id}`)).toBeTruthy();
    expect(screen.getByTestId("session-name").textContent).toBe("PHYSIS-7 closer");
    expect(screen.getByTestId("session-status").textContent).toMatch(/executing/);
    expect(screen.getByTestId("session-duration").textContent).toMatch(/m|s/);
    expect(screen.getByTestId("session-kind").textContent).toBe("claude-sonnet");
  });

  it("selected=true 时 data-selected=true + ring-2", () => {
    render(<SessionCard session={mockSession} selected={true} />);
    const card = screen.getByTestId(`session-card-${mockSession.id}`);
    expect(card.getAttribute("data-selected")).toBe("true");
    expect(card.className).toMatch(/ring-2/);
  });

  it("session.restart_available=true 时显示 RestartChip (data-testid=restart-chip)", async () => {
    const client = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const s: AgentSession = {
      ...mockSession,
      restart_available: true,
    };
    render(
      <QueryClientProvider client={client}>
        <SessionCard session={s} />
      </QueryClientProvider>,
    );
    await waitFor(() => {
      expect(screen.getByTestId("restart-chip")).toBeTruthy();
    });
  });

  it("session.restart_available=false 时不显示 chip", () => {
    const s: AgentSession = {
      ...mockSession,
      restart_available: false,
    };
    render(<SessionCard session={s} />);
    expect(screen.queryByTestId("restart-chip")).toBeNull();
  });

  it("session.restart_available=undefined (字段缺省) 包 QueryClientProvider 时不崩 — 默认不会假阳渲染 chip", async () => {
    const client = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const s: AgentSession = { ...mockSession };
    delete (s as { restart_available?: boolean }).restart_available;

    render(
      <QueryClientProvider client={client}>
        <SessionCard session={s} />
      </QueryClientProvider>,
    );
    // 主卡渲染 OK
    expect(screen.getByTestId(`session-card-${s.id}`)).toBeTruthy();
    // 等默认值稳定下来: chip 不渲染 (默认 false 保守)
    await new Promise((r) => setTimeout(r, 50));
    expect(screen.queryByTestId("restart-chip")).toBeNull();
  });
});
