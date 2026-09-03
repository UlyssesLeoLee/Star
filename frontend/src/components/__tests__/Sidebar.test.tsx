// =====================================================================
// Sidebar.test.tsx — Sidebar 折叠 + scope toggle 测试 (per 2026-09-03 12:36 JST 拍板)
// =====================================================================
// 8 个测试 (覆盖 4 项拍板 + 守门 #11 缺标比错标):
//   1. 默认 expanded + scope=main, 渲染 4 个核心 sidebar item
//   2. fold toggle 切换 w-64 ↔ w-16
//   3. 折叠态下隐藏 label/count/footer
//   4. scope toggle 切换 main ↔ project
//   5. project scope 在 /projects 路径下渲染 5 个 subnav item
//   6. project scope 在 /inbox 等非 /projects 路径下 disabled
//   7. data-testid="sidebar-fold-toggle" 始终存在
//   8. data-fold / data-scope 属性正确反映 store 状态
// =====================================================================

import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup, within } from "@testing-library/react";
import type { ReactNode } from "react";

// ---- mock next/navigation ----
const mockUsePathname = vi.fn(() => "/inbox");
const mockUseSearchParams = vi.fn(() => new URLSearchParams(""));
vi.mock("next/navigation", () => ({
  usePathname: () => mockUsePathname(),
  useSearchParams: () => mockUseSearchParams(),
  useRouter: () => ({ replace: vi.fn(), push: vi.fn() }),
}));

import { Sidebar } from "../Sidebar";
import { useNavStore } from "@/lib/nav/navStore";
import { I18nProvider } from "@/lib/i18n";

function renderWithI18n(ui: ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

const resetNavStore = () => {
  useNavStore.setState({
    sidebarItemIds: ["inbox", "issues", "projects", "agents"],
    pinnedViewIds: ["kanban", "timeline"],
    headerTabIds: ["inbox", "issues", "projects", "agents", "analytics"],
    sidebarFold: "expanded",
    sidebarScope: "main",
    selectedProjectId: "",
    isMatrixOpen: false,
  });
};

describe("Sidebar — fold + scope", () => {
  beforeEach(() => {
    cleanup();
    mockUsePathname.mockReturnValue("/inbox");
    mockUseSearchParams.mockReturnValue(new URLSearchParams(""));
    resetNavStore();
    if (typeof window !== "undefined") {
      window.localStorage.clear();
    }
  });

  afterEach(() => {
    resetNavStore();
  });

  it("default expanded + scope=main, 渲染 4 个核心 sidebar item", () => {
    renderWithI18n(<Sidebar />);
    const sidebar = screen.getByTestId("app-sidebar");
    expect(sidebar.getAttribute("data-fold")).toBe("expanded");
    expect(sidebar.getAttribute("data-scope")).toBe("main");
    // 4 个核心模块
    expect(screen.getByTestId("sidebar-item-inbox")).toBeInTheDocument();
    expect(screen.getByTestId("sidebar-item-issues")).toBeInTheDocument();
    expect(screen.getByTestId("sidebar-item-projects")).toBeInTheDocument();
    expect(screen.getByTestId("sidebar-item-agents")).toBeInTheDocument();
  });

  it("fold toggle: 点击切换 w-64 → w-16", () => {
    renderWithI18n(<Sidebar />);
    const sidebar = screen.getByTestId("app-sidebar");
    expect(sidebar.className).toMatch(/w-64/);
    const toggle = screen.getByTestId("sidebar-fold-toggle");
    fireEvent.click(toggle);
    expect(sidebar.getAttribute("data-fold")).toBe("collapsed");
    expect(sidebar.className).toMatch(/w-16/);
  });

  it("折叠态: 不显示 footer (Tactical HUD) 和 scope toggle", () => {
    renderWithI18n(<Sidebar />);
    fireEvent.click(screen.getByTestId("sidebar-fold-toggle"));
    // scope toggle 应该不在 DOM 中
    expect(screen.queryByTestId("sidebar-scope-toggle")).toBeNull();
    // footer 中的 "ALL GREEN" 文本不出现
    expect(screen.queryByText("ALL GREEN")).toBeNull();
  });

  it("scope toggle: 点击 Project 切换到 project scope", () => {
    mockUsePathname.mockReturnValue("/projects");
    renderWithI18n(<Sidebar />);
    const mainBtn = screen.getByTestId("sidebar-scope-main");
    const projBtn = screen.getByTestId("sidebar-scope-project");
    expect(mainBtn.getAttribute("aria-selected")).toBe("true");
    expect(projBtn.getAttribute("aria-selected")).toBe("false");
    fireEvent.click(projBtn);
    const sidebar = screen.getByTestId("app-sidebar");
    expect(sidebar.getAttribute("data-scope")).toBe("project");
  });

  it("project scope 在 /projects 路径下渲染 5 个 subnav item", () => {
    mockUsePathname.mockReturnValue("/projects");
    mockUseSearchParams.mockReturnValue(new URLSearchParams("tab=kanban"));
    renderWithI18n(<Sidebar />);
    fireEvent.click(screen.getByTestId("sidebar-scope-project"));
    // 5 个 project view
    expect(screen.getByTestId("sidebar-subnav-item-kanban")).toBeInTheDocument();
    expect(screen.getByTestId("sidebar-subnav-item-timeline")).toBeInTheDocument();
    expect(screen.getByTestId("sidebar-subnav-item-backlog")).toBeInTheDocument();
    expect(screen.getByTestId("sidebar-subnav-item-agents")).toBeInTheDocument();
    expect(screen.getByTestId("sidebar-subnav-item-worktrees")).toBeInTheDocument();
    // kanban 应该是 active
    const kanban = screen.getByTestId("sidebar-subnav-item-kanban");
    expect(kanban.getAttribute("data-active")).toBe("true");
  });

  it("project scope 在 /inbox 路径下 disabled (aria-disabled=true)", () => {
    mockUsePathname.mockReturnValue("/inbox");
    renderWithI18n(<Sidebar />);
    const projBtn = screen.getByTestId("sidebar-scope-project");
    expect(projBtn.getAttribute("aria-disabled")).toBe("true");
    // 点击不切换
    fireEvent.click(projBtn);
    expect(screen.getByTestId("app-sidebar").getAttribute("data-scope")).toBe("main");
  });

  it("scope 自动 fallback: pathname 离开 /projects 时, project scope 强制回 main", () => {
    mockUsePathname.mockReturnValue("/projects");
    renderWithI18n(<Sidebar />);
    // 切到 project scope
    fireEvent.click(screen.getByTestId("sidebar-scope-project"));
    expect(screen.getByTestId("app-sidebar").getAttribute("data-scope")).toBe("project");
    // 切到 /inbox, 触发 useEffect 自动 fallback
    mockUsePathname.mockReturnValue("/inbox");
    // 重渲染以触发 useEffect
    cleanup();
    renderWithI18n(<Sidebar />);
    // 因为 store 持久化到了 localStorage, 重新读 store
    // 实际上 useEffect 在新一次 mount 时也会跑, 验证 store 已经回到 main
    expect(useNavStore.getState().sidebarScope).toBe("main");
  });

  it("sidebar-fold-toggle 始终存在 (折叠 + 展开态都可见)", () => {
    renderWithI18n(<Sidebar />);
    expect(screen.getByTestId("sidebar-fold-toggle")).toBeInTheDocument();
    fireEvent.click(screen.getByTestId("sidebar-fold-toggle"));
    // 折叠态下 toggle 仍在
    expect(screen.getByTestId("sidebar-fold-toggle")).toBeInTheDocument();
  });
});
