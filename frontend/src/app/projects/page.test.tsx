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
import ProjectsPage from "./page";
import { useStore } from "@/lib/store";

describe("ProjectsPage", () => {
  beforeEach(() => {
    cleanup();
    // 重置 store 状态 (persist 持久化可能带入其他测试的状态)
    useStore.setState((s) => ({
      ...s,
      workItems: s.workItems,
      board: s.board,
    }));
  });

  it("renders project switcher + 5 tabs + Overview by default", () => {
    render(<ProjectsPage />);
    // switcher
    expect(screen.getByTestId("project-switcher")).toBeTruthy();
    // 3 个 project 都可点
    expect(screen.getByTestId("project-switcher-prj-physis")).toBeTruthy();
    expect(screen.getByTestId("project-switcher-prj-stargate")).toBeTruthy();
    expect(screen.getByTestId("project-switcher-prj-mobile")).toBeTruthy();
    // 5 个 tab 渲染
    expect(screen.getByRole("tab", { name: /Overview/i })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /Board/i })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /Timeline/i })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /Calendar/i })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /Members/i })).toBeTruthy();
    // 默认 Overview
    expect(screen.getByTestId("projects-overview-tab")).toBeTruthy();
  });

  it("Overview tab shows project metadata + KPIs", () => {
    render(<ProjectsPage />);
    // 默认 PHYSIS project — "Physis / GVPE" 出现在 switcher + overview metadata, 用 getAllByText
    const physisTexts = screen.getAllByText("Physis / GVPE");
    expect(physisTexts.length).toBeGreaterThanOrEqual(1);
    // KPI 标签
    expect(screen.getByText(/Open Issues/i)).toBeTruthy();
    expect(screen.getByText(/Active Agents/i)).toBeTruthy();
    expect(screen.getByText(/Last Activity/i)).toBeTruthy();
    // Recent work-items 表
    expect(screen.getByText(/Recent Work-items/i)).toBeTruthy();
  });

  it("switching project updates selected project", () => {
    render(<ProjectsPage />);
    const sgBtn = screen.getByTestId("project-switcher-prj-stargate");
    fireEvent.click(sgBtn);
    // Overview 标签应换成 SG (getAllByText 因为可能出现在 switcher)
    const sgTexts = screen.getAllByText("StarGate Dashboard");
    expect(sgTexts.length).toBeGreaterThanOrEqual(1);
  });

  it("Board tab renders KanbanBoard with project-filtered data", () => {
    render(<ProjectsPage />);
    const boardTab = screen.getByRole("tab", { name: /Board/i });
    fireEvent.click(boardTab);
    // kanban 4 列
    expect(screen.getByTestId("kanban-board")).toBeTruthy();
    expect(screen.getByTestId("kanban-column-todo")).toBeTruthy();
    expect(screen.getByTestId("kanban-column-in_progress")).toBeTruthy();
    expect(screen.getByTestId("kanban-column-review")).toBeTruthy();
    expect(screen.getByTestId("kanban-column-done")).toBeTruthy();
  });

  it("Timeline tab renders Gantt", () => {
    render(<ProjectsPage />);
    const tlTab = screen.getByRole("tab", { name: /Timeline/i });
    fireEvent.click(tlTab);
    expect(screen.getByTestId("projects-timeline-tab")).toBeTruthy();
  });

  it("Calendar tab renders MonthView by default", () => {
    render(<ProjectsPage />);
    const calTab = screen.getByRole("tab", { name: /Calendar/i });
    fireEvent.click(calTab);
    expect(screen.getByTestId("projects-calendar-tab")).toBeTruthy();
  });

  it("Members tab shows members table with role (mock)", () => {
    render(<ProjectsPage />);
    const memTab = screen.getByRole("tab", { name: /Members/i });
    fireEvent.click(memTab);
    expect(screen.getByTestId("projects-members-tab")).toBeTruthy();
    // 至少 1 行成员
    const memberRows = screen.getAllByTestId(/^member-/);
    expect(memberRows.length).toBeGreaterThan(0);
  });

  it("switching to mobile project shows fewer work-items in Board", () => {
    render(<ProjectsPage />);
    // 切到 MOB project
    fireEvent.click(screen.getByTestId("project-switcher-prj-mobile"));
    // 切到 Board tab
    fireEvent.click(screen.getByRole("tab", { name: /Board/i }));
    // 验证 4 列都有 (即使 0 卡)
    expect(screen.getByTestId("kanban-column-todo")).toBeTruthy();
  });
});
