/*
@cypher schema=1 source_sha256=41ede5a09499adc7ff86a9aa508b19b60418d6ec411499d79720ed2741fbbd6a
MERGE (self:File {path:"frontend/src/components/__tests__/AppHeader.test.tsx"})
@endcypher
*/
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
const mockUseSearchParams = vi.fn(() => new URLSearchParams(""));
vi.mock("next/navigation", () => ({
  usePathname: () => mockUsePathname(),
  useSearchParams: () => mockUseSearchParams(),
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

  it("renders 5 Branch 功能 tabs + Settings 齿轮 (per 2026-11-14 artifact 顶栏设计)", () => {
    renderWithI18n(<AppHeader />);
    // per artifact: 顶栏 = 任务(kanban)/画布(canvas)/协同(nema)/沙盒(sandbox)/原型(prototype)
    expect(screen.getByTestId("tab-kanban")).toBeInTheDocument();
    expect(screen.getByTestId("tab-canvas")).toBeInTheDocument();
    expect(screen.getByTestId("tab-nema")).toBeInTheDocument();
    expect(screen.getByTestId("tab-sandbox")).toBeInTheDocument();
    expect(screen.getByTestId("tab-prototype")).toBeInTheDocument();
    expect(screen.getByTestId("settings-gear")).toBeInTheDocument();
  });

  it("marks the active tab by pathname + applies category color (per 2026-09-02 18:16 JST)", () => {
    mockUsePathname.mockReturnValue("/kanban");
    renderWithI18n(<AppHeader />);
    const kanbanTab = screen.getByTestId("tab-kanban");
    const canvasTab = screen.getByTestId("tab-canvas");
    expect(kanbanTab.getAttribute("data-active")).toBe("true");
    expect(canvasTab.getAttribute("data-active")).toBe("false");
    // kanban = work 域, active 用 blue-900 + border-blue-500
    expect(kanbanTab.className).toMatch(/text-blue-900/);
    expect(kanbanTab.className).toMatch(/border-blue-500/);
  });

  it("applies different category colors per tab (per 2026-09-02 18:16 JST 推)", () => {
    // kanban=work (blue), canvas=agent (emerald), nema=agent (emerald), sandbox=system (amber), prototype=agent (violet)
    mockUsePathname.mockReturnValue("/nema");
    renderWithI18n(<AppHeader />);
    // nema active = agent 域 = emerald
    expect(screen.getByTestId("tab-nema").className).toMatch(/text-emerald-900/);
    // 其他 tab inactive 用 ink-dim
    expect(screen.getByTestId("tab-canvas").className).toMatch(/text-ink-dim/);
  });

  it("renders 5 tab icon tiles with category color when active (per 2026-09-02 18:23 JST 推 icon)", () => {
    mockUsePathname.mockReturnValue("/canvas");
    renderWithI18n(<AppHeader />);
    // canvas 是 agent 域, active icon tile 用 emerald-500/20 + border-emerald-500/50
    const canvasIcon = screen.getByTestId("tab-icon-tile-canvas");
    expect(canvasIcon.className).toMatch(/bg-emerald-500\/20/);
    expect(canvasIcon.className).toMatch(/border-emerald-500\/50/);
    expect(canvasIcon.className).toMatch(/text-emerald-900/);

    // inactive tab icon tile 应该 transparent (不染域色)
    const kanbanIcon = screen.getByTestId("tab-icon-tile-kanban");
    expect(kanbanIcon.className).not.toMatch(/bg-blue-500/);
    expect(kanbanIcon.className).toMatch(/border-transparent/);
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
