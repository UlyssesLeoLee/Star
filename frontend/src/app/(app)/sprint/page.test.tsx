// =====================================================================
// Issues Page Test (per U2 任务 — 8 测试)
// =====================================================================
// 8 个测试 (per spec):
//   1. 3 view 切换 (Sprint | List | Tree) — 3 个独立 test (per 2026-09-05 19:13 JST 拍板: Kanban 已删)
//   2. "+ New issue" button
//   3. "🔍" 搜索 button
//   4. 详情侧栏 (选中 work-item)
//
// 已知缺口 (per 缺标比错标):
//   - 创建表单 (Phase 2+ 接后端) — 仅测 ?new=true banner 显示
//   - 搜索过滤 (Phase 2+ 接 U1 CommandBar) — 仅测 toggle
//   - Tree 视图展开 (Phase 2+ 接 relations) — 不测
//   - Sprint 视图拖动 (G3 a11y) — Phase Mobile
// =====================================================================

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup, within } from "@testing-library/react";
import type { ReactNode } from "react";
import IssuesPage from "./page";
import { I18nProvider } from "@/lib/i18n";

// per 2026-08-31 i18n 补缺口: PageHeader / Sidebar 等含 useTranslation() 必须包 I18nProvider
function renderWithI18n(ui: ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

// ---- mock next/navigation ----
const mockPush = vi.fn();
const mockPathname = vi.fn(() => "/sprint");
const mockSearchParamsGet = vi.fn();

const mockSearchParams = {
  get: (k: string) => mockSearchParamsGet(k),
  toString: () => {
    // 简化: 假定 get 已调用, 我们用最近一次 get 重建
    // Per 2026-09-05 19:13 JST 拍板: default = sprint (kanban 已删除)
    return "view=sprint";
  },
  has: () => false,
  entries: () => [],
  keys: [] as any,
  values: [] as any,
  forEach: () => {},
  [Symbol.iterator]: function* () {},
};

vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: mockPush, replace: vi.fn(), prefetch: vi.fn() }),
  usePathname: () => mockPathname(),
  useSearchParams: () => mockSearchParams,
}));

// ---- mock zustand store (per useStore 全局) ----
// 引入真实 store 即可, store 是单例 zustand; 我们 reset 状态
import { useStore } from "@/lib/store";
import * as seed from "@/lib/seed";

const resetStore = () => {
  if (typeof window !== "undefined") {
    window.localStorage.clear();
  }
  useStore.setState({
    workItems: seed.workItems,
    board: seed.board,
    sprints: seed.sprints,
    identities: seed.identities,
  } as any);
};

describe("IssuesPage (U2)", () => {
  beforeEach(() => {
    mockPush.mockClear();
    mockPathname.mockReturnValue("/sprint");
    mockSearchParamsGet.mockImplementation((k: string) => (k === "view" ? "sprint" : null));
    mockSearchParams.toString = () => "view=sprint";
    resetStore();
    cleanup();
  });
  afterEach(() => {
    vi.clearAllMocks();
  });

  // ---- Test 1: view 切换 (Sprint default, per 2026-09-05 19:13 JST 拍板) ----
  it("renders Sprint view by default with 3 view tabs (no Kanban)", () => {
    renderWithI18n(<IssuesPage />);
    // 3 tabs 渲染 (Kanban 已删除 per 2026-09-05 19:13 JST)
    expect(screen.getByTestId("issues-view-tab-sprint")).toBeTruthy();
    expect(screen.getByTestId("issues-view-tab-list")).toBeTruthy();
    expect(screen.getByTestId("issues-view-tab-tree")).toBeTruthy();
    // Kanban 不应再存在
    expect(screen.queryByTestId("issues-view-tab-kanban")).toBeNull();

    // default Sprint tab aria-selected=true
    const sprintTab = screen.getByTestId("issues-view-tab-sprint");
    expect(sprintTab.getAttribute("aria-selected")).toBe("true");

    // Sprint view 渲染 (per 2026-09-05 19:13 JST 拍板: 默认 sprint)
    expect(screen.getByTestId("issues-view-sprint")).toBeTruthy();
    // KanbanBoard 不应再存在
    expect(screen.queryByTestId("kanban-board")).toBeNull();
  });

  // ---- Test 2: view 切换 (List) ----
  it("renders List view with table when view=list", () => {
    mockSearchParamsGet.mockImplementation((k: string) => (k === "view" ? "list" : null));
    mockSearchParams.toString = () => "view=list";
    renderWithI18n(<IssuesPage />);
    expect(screen.getByTestId("issues-view-list")).toBeTruthy();
    expect(screen.getByTestId("issues-list-table")).toBeTruthy();
  });

  // ---- Test 3: view 切换 (Tree) ----
  it("renders Tree view with hierarchical table when view=tree", () => {
    mockSearchParamsGet.mockImplementation((k: string) => (k === "view" ? "tree" : null));
    mockSearchParams.toString = () => "view=tree";
    renderWithI18n(<IssuesPage />);
    expect(screen.getByTestId("issues-view-tree")).toBeTruthy();
    expect(screen.getByTestId("issues-tree")).toBeTruthy();
  });

  // ---- Test 4: view 切换 (Sprint) ----
  it("renders Sprint view grouped by sprint_id when view=sprint", () => {
    mockSearchParamsGet.mockImplementation((k: string) => (k === "view" ? "sprint" : null));
    mockSearchParams.toString = () => "view=sprint";
    renderWithI18n(<IssuesPage />);
    expect(screen.getByTestId("issues-view-sprint")).toBeTruthy();
    expect(screen.getByTestId("issues-sprint-list")).toBeTruthy();
  });

  // ---- Test 5: "+ New issue" button ----
  it("renders '+ New issue' button with primary styling", () => {
    renderWithI18n(<IssuesPage />);
    const btn = screen.getByTestId("issues-new-button");
    expect(btn).toBeTruthy();
    expect(btn.textContent).toContain("New");
    // 按钮可点 (不报错)
    fireEvent.click(btn);
  });

  // ---- Test 6: "🔍" 搜索 button ----
  it("renders search button that toggles search bar", () => {
    renderWithI18n(<IssuesPage />);
    const btn = screen.getByTestId("issues-search-button");
    expect(btn).toBeTruthy();

    // 默认 search bar 不在
    expect(screen.queryByTestId("issues-search-bar")).toBeNull();

    // 点击后 search bar 出现
    fireEvent.click(btn);
    expect(screen.getByTestId("issues-search-bar")).toBeTruthy();
    expect(screen.getByTestId("issues-search-input")).toBeTruthy();

    // 再点折叠
    fireEvent.click(btn);
    expect(screen.queryByTestId("issues-search-bar")).toBeNull();
  });

  // ---- Test 7: 详情侧栏 (选中 work-item) ----
  it("shows 320px detail sidebar when a work-item row is clicked in List view", () => {
    mockSearchParamsGet.mockImplementation((k: string) => (k === "view" ? "list" : null));
    mockSearchParams.toString = () => "view=list";
    renderWithI18n(<IssuesPage />);

    // 默认 detail sidebar 不在
    expect(screen.queryByTestId("issues-detail-sidebar")).toBeNull();

    // 点击第一行 (wi-001 在 todo 列, 应该出现在 list 中)
    const row = screen.getByTestId("issues-list-row-wi-001");
    fireEvent.click(row);

    // 详情侧栏出现 + 含 transition 按钮
    const sidebar = screen.getByTestId("issues-detail-sidebar");
    expect(sidebar).toBeTruthy();
    expect(within(sidebar).getByTestId("issues-detail-transitions")).toBeTruthy();

    // 关闭按钮可点
    fireEvent.click(within(sidebar).getByTestId("issues-detail-close"));
    expect(screen.queryByTestId("issues-detail-sidebar")).toBeNull();
  });

  // ---- Test 8: 新建模式 ?new=true 触发 banner ----
  it("shows new-issue banner when ?new=true is present in URL", () => {
    mockSearchParamsGet.mockImplementation((k: string) => {
      // Per 2026-09-05 19:13 JST: view 默认 sprint, kanban 已删
      if (k === "view") return "sprint";
      if (k === "new") return "true";
      return null;
    });
    mockSearchParams.toString = () => "view=sprint&new=true";
    renderWithI18n(<IssuesPage />);
    // banner 出现
    expect(screen.getByTestId("issues-new-banner")).toBeTruthy();
    expect(screen.getByTestId("issues-new-title-input")).toBeTruthy();
  });
});
