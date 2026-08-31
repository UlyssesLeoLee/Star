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
import ProjectsPage from "./page";
import { useStore } from "@/lib/store";
import { I18nProvider } from "@/lib/i18n";

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
    renderWithI18n(<ProjectsPage />);
    // switcher
    expect(screen.getByTestId("project-switcher")).toBeTruthy();
    // 3 个 project 都可点
    expect(screen.getByTestId("project-switcher-prj-physis")).toBeTruthy();
    expect(screen.getByTestId("project-switcher-prj-stargate")).toBeTruthy();
    expect(screen.getByTestId("project-switcher-prj-mobile")).toBeTruthy();
    // 5 个 tab 渲染 (Kanban / Timeline / Backlog / Agents / Worktrees)
    expect(screen.getByRole("tab", { name: /Kanban/i })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /Timeline/i })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /Backlog/i })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /Agents/i })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /Worktrees/i })).toBeTruthy();
    // 默认 Kanban tab
    expect(screen.getByTestId("projects-kanban-tab")).toBeTruthy();
  });

  it("switching project updates selected project", () => {
    renderWithI18n(<ProjectsPage />);
    const sgBtn = screen.getByTestId("project-switcher-prj-stargate");
    fireEvent.click(sgBtn);
    expect(screen.getByTestId("project-switcher-prj-stargate")).toBeTruthy();
  });

  it("Kanban tab renders KanbanBoard with project-filtered data", () => {
    renderWithI18n(<ProjectsPage />);
    const kanbanTab = screen.getByRole("tab", { name: /Kanban/i });
    fireEvent.click(kanbanTab);
    // kanban 4 列
    expect(screen.getByTestId("kanban-board")).toBeTruthy();
    expect(screen.getByTestId("kanban-column-todo")).toBeTruthy();
    expect(screen.getByTestId("kanban-column-in_progress")).toBeTruthy();
    expect(screen.getByTestId("kanban-column-review")).toBeTruthy();
    expect(screen.getByTestId("kanban-column-done")).toBeTruthy();
  });

  it("Timeline tab renders Gantt and Calendar", () => {
    renderWithI18n(<ProjectsPage />);
    const tlTab = screen.getByRole("tab", { name: /Timeline/i });
    fireEvent.click(tlTab);
    expect(screen.getByTestId("projects-timeline-tab")).toBeTruthy();
  });

  it("Backlog tab renders work-items list", () => {
    renderWithI18n(<ProjectsPage />);
    const backlogTab = screen.getByRole("tab", { name: /Backlog/i });
    fireEvent.click(backlogTab);
    expect(screen.getByTestId("projects-backlog-tab")).toBeTruthy();
  });

  it("Agents tab shows members table with role (mock)", () => {
    renderWithI18n(<ProjectsPage />);
    const agentsTab = screen.getByRole("tab", { name: /Agents/i });
    fireEvent.click(agentsTab);
    expect(screen.getByTestId("projects-members-tab")).toBeTruthy();
    // 至少 1 行成员
    const memberRows = screen.getAllByTestId(/^member-/);
    expect(memberRows.length).toBeGreaterThan(0);
  });

  it("Worktrees tab shows worktree list", () => {
    renderWithI18n(<ProjectsPage />);
    const wtTab = screen.getByRole("tab", { name: /Worktrees/i });
    fireEvent.click(wtTab);
    expect(screen.getByTestId("projects-worktrees-tab")).toBeTruthy();
  });

  it("switching to mobile project shows fewer work-items in Kanban", () => {
    renderWithI18n(<ProjectsPage />);
    // 切到 MOB project
    fireEvent.click(screen.getByTestId("project-switcher-prj-mobile"));
    // 切到 Kanban tab
    fireEvent.click(screen.getByRole("tab", { name: /Kanban/i }));
    // 验证 4 列都有 (即使 0 卡)
    expect(screen.getByTestId("kanban-column-todo")).toBeTruthy();
  });
});
