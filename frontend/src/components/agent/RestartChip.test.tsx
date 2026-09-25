// =====================================================================
// RestartChip.test.tsx — ULYS-214 RestartChip 组件测试
//
// 4 个测试:
//   1. available=false 时不渲染
//   2. result 注入式: 用 injected mock result 渲染 chip
//   3. 点击触发 restart() (从 useRestart 取) — 注入式走 result prop 时同样触发
//   4. isRestarting=true 时不调用 onClick (防御性)
// =====================================================================
import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RestartChip } from "./RestartChip";
import type { UseRestartResult } from "@/lib/hooks/use-restart";
import * as React from "react";

const makeWrapper = (client: QueryClient) =>
  ({ children }: { children: React.ReactNode }) =>
    React.createElement(QueryClientProvider, { client }, children);

function availableMock(available = true): UseRestartResult {
  return {
    restartAvailable: available,
    lastCheckedAt: Date.now(),
    isLoading: false,
    restart: vi.fn(),
    isRestarting: false,
    error: null,
    refetch: vi.fn(),
  };
}

describe("RestartChip (ULYS-214)", () => {
  beforeEach(() => {
    cleanup();
  });

  it("available=false 时不渲染", () => {
    const { container } = render(
      <RestartChip sessionId="sess-1" available={false} />,
    );
    expect(container.firstChild).toBeNull();
  });

  it("注入 result 时渲染 chip (data-testid=restart-chip + Restart 文案)", () => {
    render(<RestartChip sessionId="sess-1" result={availableMock(true)} />);
    const btn = screen.getByTestId("restart-chip");
    expect(btn).toBeTruthy();
    expect(btn.tagName).toBe("BUTTON");
    expect(btn.getAttribute("data-state")).toBe("ready");
    expect(btn.textContent).toMatch(/Restart/);
    expect(btn.getAttribute("aria-busy")).toBe("false");
  });

  it("注入不可用 result 时不渲染", () => {
    const { container } = render(
      <RestartChip sessionId="sess-1" result={availableMock(false)} />,
    );
    expect(container.firstChild).toBeNull();
  });

  it("点击 button 触发注入的 result.restart()", () => {
    const restart = vi.fn();
    const injected: UseRestartResult = {
      ...availableMock(true),
      restart,
    };

    render(<RestartChip sessionId="sess-1" result={injected} />);
    const btn = screen.getByTestId("restart-chip");

    fireEvent.click(btn);
    expect(restart).toHaveBeenCalledTimes(1);
  });

  it("isRestarting=true 时显示 aria-busy 且不调用 onClick (防御性)", () => {
    const restart = vi.fn();
    const injected: UseRestartResult = {
      ...availableMock(true),
      isRestarting: true, // 关键: 在转
      restart,
    };

    render(<RestartChip sessionId="sess-1" result={injected} />);
    const btn = screen.getByTestId("restart-chip");
    expect(btn.getAttribute("data-state")).toBe("restarting");
    expect(btn.getAttribute("aria-busy")).toBe("true");

    fireEvent.click(btn);
    expect(restart).not.toHaveBeenCalled();
  });

  it("自管 (无 result/available) 在 QueryClientProvider 内不渲染 chip — useRestart 默认值是 false", async () => {
    const client = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });

    // 没传 available 也没传 result → 走"自管"分支调 useRestart;
    // useRestart 默认 fetcher (本期会 404) → query.data.restart_available 始终 false
    // 此时 chip 不渲染
    render(
      <QueryClientProvider client={client}>
        <RestartChip sessionId="sess-x-no-injection" />
      </QueryClientProvider>,
    );

    // 等一个 tick 后 chip 不会渲染 (default false)
    await new Promise((r) => setTimeout(r, 50));
    expect(screen.queryByTestId("restart-chip")).toBeNull();
  });
});
