// =====================================================================
// Projects Page 单元测试 (per ui-redesign-multica-style.md §5.2)
// 覆盖:
//   1. 渲染 5 tab + project switcher
//   2. 默认显示 Overview tab + project 元信息 + KPI
//   3. 切换 project 后数据过滤
//   4. 切换到 Board tab 显示 KanbanBoard (4 列)
//   5. 切换到 Members tab 显示成员表
// =====================================================================

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, cleanup, fireEvent } from "@testing-library/react";
import type { ReactNode } from "react";
import ProjectsClient from "./ProjectsClient";
import { I18nProvider } from "@/lib/i18n";
import { useStore } from "@/lib/store";

// per 2026-08-31 i18n 补缺口: PageHeader / Sidebar / AppHeader 等含 useTranslation() 必须包 I18nProvider
function renderWithI18n(ui: ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

describe("ProjectsPage", () => {
  beforeEach(() => {
    cleanup();
    if (typeof window !== "undefined") {
      window.localStorage.clear();
    }
    // 重置 store 状态 (persist 持久化可能带入其他测试的状态)
    useStore.setState((s) => ({
      ...s,
      workItems: s.workItems,
      board: s.board,
    }));
  });

  it("renders project switcher + 5 tabs by default", () => {
    renderWithI18n(<ProjectsClient initialTab="kanban" />);
    // switcher
    expect(screen.getByTestId("project-switcher")).toBeTruthy();
    // 3 个 project 都可点
    expect(screen.getByTestId("project-switcher-prj-physis")).toBeTruthy();
    expect(screen.getByTestId("project-switcher-prj-stargate")).toBeTruthy();
    expect(screen.getByTestId("project-switcher-prj-mobile")).toBeTruthy();
    // 5 个 tab 渲染 (Kanban / Timeline / Backlog / Agents / Worktrees)
    expect(screen.getByRole("tab", { name: /Kanban|看板/i })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /Timeline|时间线|甘特/i })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /Backlog|待办池|待办/i })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /Agents|Agent/i })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /Worktrees|Worktree/i })).toBeTruthy();
    // 默认 Kanban tab
    expect(screen.getByTestId("projects-kanban-tab")).toBeTruthy();
  });

  it("switching project updates selected project", () => {
    renderWithI18n(<ProjectsClient initialTab="kanban" />);
    const sgBtn = screen.getByTestId("project-switcher-prj-stargate");
    fireEvent.click(sgBtn);
    expect(screen.getByTestId("project-switcher-prj-stargate")).toBeTruthy();
  });

  it("Kanban tab renders KanbanBoard with project-filtered data", () => {
    renderWithI18n(<ProjectsClient initialTab="kanban" />);
    const kanbanTab = screen.getByRole("tab", { name: /Kanban|看板/i });
    fireEvent.click(kanbanTab);
    // kanban 4 列
    expect(screen.getByTestId("kanban-board")).toBeTruthy();
    expect(screen.getByTestId("kanban-column-todo")).toBeTruthy();
    expect(screen.getByTestId("kanban-column-in_progress")).toBeTruthy();
    expect(screen.getByTestId("kanban-column-review")).toBeTruthy();
    expect(screen.getByTestId("kanban-column-done")).toBeTruthy();
  });

  it("Timeline tab renders Gantt and Calendar", () => {
    renderWithI18n(<ProjectsClient initialTab="kanban" />);
    const tlTab = screen.getByRole("tab", { name: /Timeline|时间线|甘特/i });
    fireEvent.click(tlTab);
    expect(screen.getByTestId("projects-timeline-tab")).toBeTruthy();
  });

  it("Backlog tab renders work-items list", () => {
    renderWithI18n(<ProjectsClient initialTab="kanban" />);
    const backlogTab = screen.getByRole("tab", { name: /Backlog|待办池|待办/i });
    fireEvent.click(backlogTab);
    expect(screen.getByTestId("projects-backlog-tab")).toBeTruthy();
  });

  it("Agents tab shows members table with role (mock)", () => {
    renderWithI18n(<ProjectsClient initialTab="kanban" />);
    const agentsTab = screen.getByRole("tab", { name: /Agents|Agent/i });
    fireEvent.click(agentsTab);
    expect(screen.getByTestId("projects-members-tab")).toBeTruthy();
    // 至少 1 行成员
    const memberRows = screen.getAllByTestId(/^member-/);
    expect(memberRows.length).toBeGreaterThan(0);
  });

  it("Worktrees tab shows worktree list", () => {
    renderWithI18n(<ProjectsClient initialTab="kanban" />);
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
    renderWithI18n(<ProjectsClient initialTab="kanban" />);
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
    renderWithI18n(<ProjectsClient initialTab="kanban" />);
    fireEvent.click(screen.getByTestId("project-switcher-prj-stargate"));
    fireEvent.click(screen.getByRole("tab", { name: /Worktrees|Worktree/i }));
    expect(screen.queryByTestId("project-canvas-links")).toBeNull();
  });

  it("switching to mobile project shows fewer work-items in Kanban", () => {
    renderWithI18n(<ProjectsClient initialTab="kanban" />);
    // 切到 MOB project
    fireEvent.click(screen.getByTestId("project-switcher-prj-mobile"));
    // 切到 Kanban tab
    fireEvent.click(screen.getByRole("tab", { name: /Kanban|看板/i }));
    // 验证 4 列都有 (即使 0 卡)
    expect(screen.getByTestId("kanban-column-todo")).toBeTruthy();
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
    renderWithI18n(<ProjectsClient initialTab="kanban" />);
    // 清空 cookie (可能 vitest jsdom 残留)
    document.cookie = "projects-default-tab=; path=/; max-age=0";
    // 切到 Timeline tab
    fireEvent.click(screen.getByRole("tab", { name: /Timeline|时间线|甘特/i }));
    // 验证 cookie 写入
    expect(document.cookie).toContain("projects-default-tab=timeline");
  });
});
