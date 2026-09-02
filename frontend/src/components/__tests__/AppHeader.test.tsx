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
import type { ReactNode } from "react";

// ---- mock next/navigation ----
const mockUsePathname = vi.fn(() => "/inbox");
vi.mock("next/navigation", () => ({
  usePathname: () => mockUsePathname(),
  useRouter: () => ({ replace: vi.fn(), push: vi.fn() }),
}));

import { AppHeader } from "../AppHeader";
import { useCommandBarStore } from "@/lib/commandBarStore";
import { I18nProvider } from "@/lib/i18n";

// per 2026-08-31 i18n 实装: AppHeader 内 useTranslation() 必须包 I18nProvider
function renderWithI18n(ui: ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

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
    // 清掉可能跨测试残留的 localStorage
    if (typeof window !== "undefined") {
      window.localStorage.clear();
    }
  });

  afterEach(() => {
    resetCommandBarStore();
  });

  it("renders 5 primary tabs + Settings 齿轮 (per §3 5 视图 tab)", () => {
    renderWithI18n(<AppHeader />);
    expect(screen.getByTestId("tab-inbox")).toBeInTheDocument();
    expect(screen.getByTestId("tab-issues")).toBeInTheDocument();
    expect(screen.getByTestId("tab-projects")).toBeInTheDocument();
    expect(screen.getByTestId("tab-agents")).toBeInTheDocument();
    expect(screen.getByTestId("tab-analytics")).toBeInTheDocument();
    expect(screen.getByTestId("settings-gear")).toBeInTheDocument();
  });

  it("marks the active tab by pathname + applies category color (per 2026-09-02 18:16 JST Jira 风格)", () => {
    mockUsePathname.mockReturnValue("/issues");
    renderWithI18n(<AppHeader />);
    const issuesTab = screen.getByTestId("tab-issues");
    const inboxTab = screen.getByTestId("tab-inbox");
    expect(issuesTab.getAttribute("data-active")).toBe("true");
    expect(inboxTab.getAttribute("data-active")).toBe("false");
    // issues 是 work 域, active 用 blue-900 + border-blue-500
    // (per 2026-09-02 18:16 JST 推: 顶栏 active 改域色, 替换旧 accent)
    expect(issuesTab.className).toMatch(/text-blue-900/);
    expect(issuesTab.className).toMatch(/border-blue-500/);
  });

  it("applies different category colors per tab (per 2026-09-02 18:16 JST 推 Jira 风格)", () => {
    // inbox=core (cyan), issues=work (blue), agents=agent (emerald), settings=system (amber)
    mockUsePathname.mockReturnValue("/inbox");
    renderWithI18n(<AppHeader />);
    // inbox active = cyan
    expect(screen.getByTestId("tab-inbox").className).toMatch(/text-cyan-900/);
    // 其他 tab inactive 用 ink-dim, 不应该有 cyan/blue/emerald/amber 域色
    expect(screen.getByTestId("tab-issues").className).toMatch(/text-ink-dim/);
  });

  it("clicking ⌘K trigger calls commandBarStore.open() (per §6 + §3 右栏搜索)", () => {
    renderWithI18n(<AppHeader />);
    const trigger = screen.getByTestId("command-bar-trigger");
    expect(useCommandBarStore.getState().isOpen).toBe(false);
    fireEvent.click(trigger);
    expect(useCommandBarStore.getState().isOpen).toBe(true);
  });

  it("notifications badge shows count (per §3 🔔 通知 badge)", () => {
    renderWithI18n(<AppHeader />);
    const badge = screen.getByTestId("notifications-badge");
    expect(badge).toBeInTheDocument();
    expect(badge.textContent?.trim()).toBe("3");
  });

  it("realtime status shows ok dot (per §3 🟢 Realtime status)", () => {
    // aria-label 走 i18n (zh-CN "实时同步: 在线") 不硬编码 'online',
    // 改为验证 data-testid 存在 + bg-ok 圆点
    renderWithI18n(<AppHeader />);
    const status = screen.getByTestId("realtime-status");
    expect(status).toBeInTheDocument();
    const dot = status.querySelector("span.bg-ok");
    expect(dot).not.toBeNull();
  });
});
