/*
@cypher schema=1 source_sha256=df4da2b171d4cbf347ed9f58734a6e9b8e923bbea16718e05df4febbdaf0c5ef
MERGE (self:File {path:"frontend/src/app/(app)/projects/page.test.tsx"})
@endcypher
*/
// =====================================================================
// Projects Page 单元测试 (per 前端布局设计说明 §5.2)
// 覆盖:
//   1. 渲染 5 tab + project switcher
//   2. 默认显示 Overview tab + project 元信息 + KPI
//   3. 切换 project 后数据过滤
//   4. 切换到 Board tab 显示 KanbanBoard (4 列)
//   5. 切换到 Members tab 显示成员表
// =====================================================================

import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, cleanup, fireEvent } from "@testing-library/react";
import type { ReactNode } from "react";

// per 2026-11-12 导航切换修复: ProjectsClient 用 useRouter 写回 URL ?tab=
// per 2026-11-17 去重: legacy ?tab=kanban 客户端转发 /kanban, 断言 replace 调用需要 hoisted mock
const { mockReplace } = vi.hoisted(() => ({ mockReplace: vi.fn() }));

vi.mock("next/navigation", () => ({
  useRouter: () => ({ replace: mockReplace, push: vi.fn() }),
}));

// per 2026-11-16 清 seed/mock 显示入口: ProjectsClient 用 Group API 会话判定宿主接入 —
// 测试默认模拟"宿主已接入" (既有用例断言 seed fixture 渲染),
// 无宿主用例切换 client=null 验证诚实未接入态。
const groupMockState = vi.hoisted(() => ({
  client: null as unknown,
  status: "unavailable" as "unavailable" | "ready" | "invalidated",
}));

vi.mock("@/lib/group/groupProjection", () => ({
  useWorktreeGroupApi: () => groupMockState.client,
  useWorktreeGroupSessionState: () => ({ status: groupMockState.status }),
}));

import ProjectsClient from "./ProjectsClient";
import { I18nProvider } from "@/lib/i18n";
import { useStore } from "@/lib/store";
import * as seed from "@/lib/seed";

// per 2026-08-31 i18n 补缺口: PageHeader / Sidebar / AppHeader 等含 useTranslation() 必须包 I18nProvider
function renderWithI18n(ui: ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

// per 2026-11-16 store 初始 state 改为诚实空状态 (不喂 seed 兜底) —
// 本文件用例断言 seed fixture 渲染, beforeEach 显式注入 (pattern 同 sprint/page.test.tsx)。
function resetStore() {
  if (typeof window !== "undefined") {
    window.localStorage.clear();
  }
  useStore.setState({
    projects: seed.projects,
    workItems: seed.workItems,
    sprints: seed.sprints,
    milestones: seed.milestones,
    board: seed.board,
    identities: seed.identities,
    workspaces: seed.workspaces,
    canvases: seed.canvases,
    worktrees: seed.worktrees,
    relations: seed.relations,
    agentSessions: seed.agentSessions,
    changeSets: seed.changeSets,
    repositories: seed.repositories,
    pullRequests: seed.pullRequests,
  } as never);
}

describe("ProjectsPage", () => {
  beforeEach(() => {
    cleanup();
    mockReplace.mockClear();
    // 默认模拟宿主已接入 (per 2026-11-16: 既有用例断言 seed fixture 渲染)
    groupMockState.client = {};
    groupMockState.status = "ready";
    resetStore();
  });

  it("renders project switcher + 5 tabs by default (per 2026-11-17 去重: Kanban 已移入 /kanban)", () => {
    renderWithI18n(<ProjectsClient initialTab="backlog" />);
    // switcher
    expect(screen.getByTestId("project-switcher")).toBeTruthy();
    // 3 个 project 都可点
    expect(screen.getByTestId("project-switcher-prj-physis")).toBeTruthy();
    expect(screen.getByTestId("project-switcher-prj-stargate")).toBeTruthy();
    expect(screen.getByTestId("project-switcher-prj-mobile")).toBeTruthy();
    // 5 个 tab 渲染 (Timeline / Backlog / Agents / Worktrees / IPA 技能)
    expect(screen.getByRole("tab", { name: /Timeline|时间线|甘特/i })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /Backlog|待办池|待办/i })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /Agents|Agent/i })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /Worktrees|Worktree/i })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /IPA/i })).toBeTruthy();
    // per 2026-11-17 去重: Kanban tab 不再渲染
    expect(screen.queryByRole("tab", { name: /Kanban|看板/i })).toBeNull();
    // 默认 backlog tab
    expect(screen.getByTestId("projects-backlog-tab")).toBeTruthy();
  });

  it("switching project updates selected project", () => {
    renderWithI18n(<ProjectsClient initialTab="backlog" />);
    const sgBtn = screen.getByTestId("project-switcher-prj-stargate");
    fireEvent.click(sgBtn);
    expect(screen.getByTestId("project-switcher-prj-stargate")).toBeTruthy();
  });

  it("legacy initialTab=kanban 客户端转发 /kanban (per 2026-11-17 去重, 老链接不 404)", () => {
    renderWithI18n(<ProjectsClient initialTab="kanban" />);
    expect(mockReplace).toHaveBeenCalledWith("/kanban");
  });

  it("Timeline tab renders Gantt and Calendar", () => {
    renderWithI18n(<ProjectsClient initialTab="backlog" />);
    const tlTab = screen.getByRole("tab", { name: /Timeline|时间线|甘特/i });
    fireEvent.click(tlTab);
    expect(screen.getByTestId("projects-timeline-tab")).toBeTruthy();
  });

  it("Backlog tab renders work-items list", () => {
    renderWithI18n(<ProjectsClient initialTab="backlog" />);
    const backlogTab = screen.getByRole("tab", { name: /Backlog|待办池|待办/i });
    fireEvent.click(backlogTab);
    expect(screen.getByTestId("projects-backlog-tab")).toBeTruthy();
  });

  it("Agents tab shows members table with role (mock)", () => {
    renderWithI18n(<ProjectsClient initialTab="backlog" />);
    const agentsTab = screen.getByRole("tab", { name: /Agents|Agent/i });
    fireEvent.click(agentsTab);
    expect(screen.getByTestId("projects-members-tab")).toBeTruthy();
    // 至少 1 行成员
    const memberRows = screen.getAllByTestId(/^member-/);
    expect(memberRows.length).toBeGreaterThan(0);
  });

  it("Worktrees tab shows worktree list", () => {
    renderWithI18n(<ProjectsClient initialTab="backlog" />);
    const wtTab = screen.getByRole("tab", { name: /Worktrees|Worktree/i });
    fireEvent.click(wtTab);
    expect(screen.getByTestId("projects-worktrees-tab")).toBeTruthy();
  });

  // per DRIFT-α-009 修复: canvas/[id] 之前无任何可达入口 (原 /collaboration
  // 已被 legacy redirect 拦截), 补一个真实可达的 project-scoped 链接。
  // 默认选中的 prj-physis 关联 seed 里唯一的 ref_kind="project" canvas
  // (canvas-001), 这条测试锁的是"确实渲染出一条可点的 /canvas/canvas-001 链接",
  // 而不只是代码路径存在但从不命中数据。
  it("Worktrees tab shows a working canvas link for the selected project (DRIFT-α-009)", () => {
    renderWithI18n(<ProjectsClient initialTab="backlog" />);
    // navStore.selectedProjectId 跨测试持久化 (per 2026-09-03 拍板), 显式选回
    // prj-physis 而非依赖默认值, 避免前面测试切过 project 导致这条测试脆弱
    fireEvent.click(screen.getByTestId("project-switcher-prj-physis"));
    fireEvent.click(screen.getByRole("tab", { name: /Worktrees|Worktree/i }));
    const links = screen.getByTestId("project-canvas-links");
    const link = screen.getByTestId("project-canvas-link-canvas-001");
    expect(links).toBeTruthy();
    expect(link.getAttribute("href")).toBe("/canvas/canvas-001");
  });

  it("Worktrees tab hides the canvas link block for a project with no linked canvas", () => {
    renderWithI18n(<ProjectsClient initialTab="backlog" />);
    fireEvent.click(screen.getByTestId("project-switcher-prj-stargate"));
    fireEvent.click(screen.getByRole("tab", { name: /Worktrees|Worktree/i }));
    expect(screen.queryByTestId("project-canvas-links")).toBeNull();
  });

  it("switching to mobile project keeps backlog tab rendered (per 2026-11-17 去重)", () => {
    renderWithI18n(<ProjectsClient initialTab="backlog" />);
    // 切到 MOB project
    fireEvent.click(screen.getByTestId("project-switcher-prj-mobile"));
    expect(screen.getByTestId("projects-backlog-tab")).toBeTruthy();
  });

  // =====================================================================
  // per 2026-09-01 16:41 JST "界面迁移全面完善" 拍板 cookie-default 修法
  //   server wrapper (page.tsx) 读 cookies + searchParams → 传 initialTab
  //   ProjectsClient 用 initialTab 做 useState 初始值, SSR HTML 已正确,
  //   避免 "闪一下"。下面 2 个测试锁这个行为。
  // =====================================================================

  it("SSR initialTab=timeline renders timeline tab content on first render (no flash)", () => {
    // 模拟 server wrapper 传 initialTab="timeline" (用户 cookie 持久化)
    renderWithI18n(<ProjectsClient initialTab="timeline" />);
    // SSR 阶段就应渲染 timeline tab content, 不应等 client useEffect 同步
    expect(screen.getByTestId("projects-timeline-tab")).toBeTruthy();
  });

  it("SSR initialTab=backlog renders backlog tab on first render", () => {
    renderWithI18n(<ProjectsClient initialTab="backlog" />);
    expect(screen.getByTestId("projects-backlog-tab")).toBeTruthy();
  });

  it("switching tab writes cookie for next SSR (cookie-default 持久化)", () => {
    renderWithI18n(<ProjectsClient initialTab="backlog" />);
    // 清空 cookie (可能 vitest jsdom 残留)
    document.cookie = "projects-default-tab=; path=/; max-age=0";
    // 切到 Timeline tab
    fireEvent.click(screen.getByRole("tab", { name: /Timeline|时间线|甘特/i }));
    // 验证 cookie 写入
    expect(document.cookie).toContain("projects-default-tab=timeline");
  });

  // per 2026-11-12 用户发令: sidebar Timeline/Backlog 跳本页不同 ?tab= 时必须
  // 正确切换 ("timeline 打开是看板标签页" 根因修复回归测试)
  it("sidebar jump re-render with new initialTab switches the active tab", () => {
    const { rerender } = renderWithI18n(<ProjectsClient initialTab="backlog" />);
    expect(screen.getByTestId("projects-backlog-tab")).toBeTruthy();
    expect(screen.queryByTestId("projects-timeline-tab")).toBeNull();

    // 模拟 server wrapper 因 URL ?tab=timeline 重渲染传新 initialTab
    rerender(
      <I18nProvider initialLanguage="zh-CN">
        <ProjectsClient initialTab="timeline" />
      </I18nProvider>,
    );
    expect(screen.getByTestId("projects-timeline-tab")).toBeTruthy();
    expect(screen.queryByTestId("projects-backlog-tab")).toBeNull();
  });

  // =====================================================================
  // per 2026-11-16 清 seed/mock 显示入口: 无宿主会话 (网页预览模式) 时
  // Kanban/Backlog 不渲染本地 seed 任务卡, 改诚实未接入态 + 预览模式徽标。
  // =====================================================================

  // per 2026-11-17 去重: 看板诚实态随任务看板移入 /kanban 页 (KanbanBoard 的
  // unavailableState prop 承接), 本页只剩 legacy ?tab=kanban 转发行为。
  it("无宿主会话: legacy ?tab=kanban 仍客户端转发 /kanban (不渲染死内容)", () => {
    groupMockState.client = null;
    groupMockState.status = "unavailable";
    renderWithI18n(<ProjectsClient initialTab="kanban" />);
    expect(mockReplace).toHaveBeenCalledWith("/kanban");
  });

  it("无宿主会话: Backlog tab 显示诚实未接入态, 不渲染 seed 任务列表", () => {
    groupMockState.client = null;
    groupMockState.status = "unavailable";
    renderWithI18n(<ProjectsClient initialTab="backlog" />);
    expect(screen.getByTestId("projects-backlog-tab")).toBeTruthy();
    expect(screen.getByTestId("status-notice-disconnected")).toBeTruthy();
    expect(screen.queryByTestId("backlog-item")).toBeNull();
  });
});
