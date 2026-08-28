// =====================================================================
// AppHeader.test.tsx — 顶栏测试 (per design §3 + §8.1)
// =====================================================================
// 5 个测试 (per 任务要求 ≥2):
//   1. 渲染 5 tab + Settings 齿轮
//   2. active tab (pathname=/issues) 标 data-active="true"
//   3. 点击 ⌘K 触发 commandBarStore.open
//   4. 通知 badge 显示 mock count (3)
//   5. realtime status 显示 online + size-2 ok 圆点
// =====================================================================

import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";

// ---- mock next/navigation ----
const mockUsePathname = vi.fn(() => "/inbox");
vi.mock("next/navigation", () => ({
  usePathname: () => mockUsePathname(),
  useRouter: () => ({ replace: vi.fn(), push: vi.fn() }),
}));

import { AppHeader } from "../AppHeader";
import { useCommandBarStore } from "@/lib/commandBarStore";

const resetCommandBarStore = () => {
  useCommandBarStore.setState({
    isOpen: false,
    query: "",
    recent: [],
  });
};

describe("AppHeader", () => {
  beforeEach(() => {
    cleanup();
    mockUsePathname.mockReturnValue("/inbox");
    resetCommandBarStore();
  });

  afterEach(() => {
    resetCommandBarStore();
  });

  it("renders 5 primary tabs + Settings 齿轮 (per §3 5 视图 tab)", () => {
    render(<AppHeader />);
    expect(screen.getByTestId("tab-inbox")).toBeInTheDocument();
    expect(screen.getByTestId("tab-issues")).toBeInTheDocument();
    expect(screen.getByTestId("tab-projects")).toBeInTheDocument();
    expect(screen.getByTestId("tab-agents")).toBeInTheDocument();
    expect(screen.getByTestId("tab-analytics")).toBeInTheDocument();
    expect(screen.getByTestId("settings-gear")).toBeInTheDocument();
  });

  it("marks the active tab by pathname (per §3 active 状态)", () => {
    mockUsePathname.mockReturnValue("/issues");
    render(<AppHeader />);
    const issuesTab = screen.getByTestId("tab-issues");
    const inboxTab = screen.getByTestId("tab-inbox");
    expect(issuesTab.getAttribute("data-active")).toBe("true");
    expect(inboxTab.getAttribute("data-active")).toBe("false");
    // active tab 应该是 accent 色 + accent border
    expect(issuesTab.className).toMatch(/text-accent/);
    expect(issuesTab.className).toMatch(/border-accent/);
  });

  it("clicking ⌘K trigger calls commandBarStore.open() (per §6 + §3 右栏搜索)", () => {
    render(<AppHeader />);
    const trigger = screen.getByTestId("command-bar-trigger");
    expect(useCommandBarStore.getState().isOpen).toBe(false);
    fireEvent.click(trigger);
    expect(useCommandBarStore.getState().isOpen).toBe(true);
  });

  it("notifications badge shows count (per §3 🔔 通知 badge)", () => {
    render(<AppHeader />);
    const badge = screen.getByTestId("notifications-badge");
    expect(badge).toBeInTheDocument();
    expect(badge.textContent?.trim()).toBe("3");
  });

  it("realtime status shows online + size-2 ok 圆点 (per §3 🟢 Realtime status)", () => {
    render(<AppHeader />);
    const status = screen.getByTestId("realtime-status");
    expect(status).toBeInTheDocument();
    expect(status.getAttribute("aria-label")).toMatch(/online/);
    // 含 size-2 rounded-full bg-ok 的圆点
    const dot = status.querySelector("span.bg-ok");
    expect(dot).not.toBeNull();
  });
});
