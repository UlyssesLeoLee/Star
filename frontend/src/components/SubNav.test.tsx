// =====================================================================
// SubNav.test.tsx — U2 SubNav 4 测试 (per ui-redesign-multica-style.md §4)
// =====================================================================
// 4 个测试 (per spec):
//   1. 渲染所有 items (label + href)
//   2. active 状态: bg-accent/[0.12] + border-l-accent
//   3. count badge 显示
//   4. 点击触发导航 (default <Link> 行为)
//
// 已知缺口 (per 缺标比错标):
//   - 同 active 路径子路径匹配 (startsWith) — 已实现,不在此测
//   - 折叠/展开 (Phase 2+) — 不测
// =====================================================================

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import { SubNav, type SubNavItem } from "./SubNav";

// ---- mock next/navigation ----
const mockPathname = vi.fn(() => "/issues");
vi.mock("next/navigation", () => ({
  usePathname: () => mockPathname(),
  useRouter: () => ({ push: vi.fn(), replace: vi.fn(), prefetch: vi.fn() }),
  useSearchParams: () => new URLSearchParams(),
}));

const items: SubNavItem[] = [
  { id: "kanban", label: "Kanban", href: "/issues?view=kanban" },
  { id: "list",   label: "List",   href: "/issues?view=list",   count: 30 },
  { id: "tree",   label: "Tree",   href: "/issues?view=tree" },
  { id: "sprint", label: "Sprint", href: "/issues?view=sprint", count: 4 },
];

describe("SubNav (U2)", () => {
  beforeEach(() => {
    mockPathname.mockReturnValue("/issues");
    cleanup();
  });
  afterEach(() => vi.clearAllMocks());

  // ---- Test 1: 渲染所有 items ----
  it("renders all items with label", () => {
    render(<SubNav items={items} activeId="kanban" />);
    // 4 个 item 全部渲染
    for (const item of items) {
      const el = screen.getByTestId(`subnav-item-${item.id}`);
      expect(el).toBeTruthy();
      expect(el.textContent).toContain(item.label);
      // href 正确
      expect(el.getAttribute("href")).toBe(item.href);
    }
    // subnav 容器 testid 存在
    expect(screen.getByTestId("subnav")).toBeTruthy();
  });

  // ---- Test 2: active 状态 ----
  it("applies active styling (bg-accent/12 + border-l-accent) to the active item", () => {
    render(<SubNav items={items} activeId="list" />);
    const active = screen.getByTestId("subnav-item-list");
    const inactive = screen.getByTestId("subnav-item-kanban");

    // active
    expect(active.getAttribute("data-active")).toBe("true");
    expect(active.className).toMatch(/bg-accent/);
    expect(active.className).toMatch(/border-l-accent/);
    expect(active.getAttribute("aria-current")).toBe("page");

    // inactive
    expect(inactive.getAttribute("data-active")).toBe("false");
    expect(inactive.className).not.toMatch(/border-l-accent/);
    expect(inactive.getAttribute("aria-current")).toBeNull();
  });

  // ---- Test 3: count badge 显示 ----
  it("renders count badge when item.count is defined, hides when undefined", () => {
    render(<SubNav items={items} activeId="kanban" />);

    // list count=30, sprint count=4 — 应该有 count badge
    expect(screen.getByTestId("subnav-count-list")).toBeTruthy();
    expect(screen.getByTestId("subnav-count-list").textContent).toBe("30");
    expect(screen.getByTestId("subnav-count-sprint")).toBeTruthy();
    expect(screen.getByTestId("subnav-count-sprint").textContent).toBe("4");

    // kanban / tree 没 count — 不渲染
    expect(screen.queryByTestId("subnav-count-kanban")).toBeNull();
    expect(screen.queryByTestId("subnav-count-tree")).toBeNull();
  });

  // ---- Test 4: 点击触发导航 (Link 行为) ----
  it("renders each item as an <a> link that navigates on click", () => {
    render(<SubNav items={items} activeId="tree" />);
    // 点击 list item — 默认 Next.js Link 行为 (测试中走 native navigation)
    const listItem = screen.getByTestId("subnav-item-list");
    expect(listItem.tagName.toLowerCase()).toBe("a");
    // href 包含 view=list query
    expect(listItem.getAttribute("href")).toBe("/issues?view=list");

    // 点击不报错
    fireEvent.click(listItem);
  });
});
