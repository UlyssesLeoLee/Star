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

  // ---- Test 2: active 状态 (per 2026-09-02 16:13 JST Jira 风格扩展) ----
  it("applies category-colored active styling (default category=core → cyan-500)", () => {
    render(<SubNav items={items} activeId="list" />);
    const active = screen.getByTestId("subnav-item-list");
    const inactive = screen.getByTestId("subnav-item-kanban");

    // active: 默认 category=core → cyan-500/20 bg + cyan-500/50 border
    expect(active.getAttribute("data-active")).toBe("true");
    expect(active.className).toMatch(/bg-cyan-500\/20/);
    expect(active.className).toMatch(/border-cyan-500\/50/);
    expect(active.className).toMatch(/text-cyan-900/);
    expect(active.getAttribute("aria-current")).toBe("page");

    // inactive: 无域色 class
    expect(inactive.getAttribute("data-active")).toBe("false");
    expect(inactive.className).not.toMatch(/bg-cyan-500/);
    expect(inactive.getAttribute("aria-current")).toBeNull();
  });

  // ---- Test 2b: 显式 category prop 切换域色 ----
  it("applies work-category styling (blue-500) when category=work is passed", () => {
    render(<SubNav items={items} activeId="list" category="work" />);
    const active = screen.getByTestId("subnav-item-list");
    expect(active.className).toMatch(/bg-blue-500\/20/);
    expect(active.className).toMatch(/border-blue-500\/50/);
    expect(active.className).toMatch(/text-blue-900/);
  });

  // ---- Test 2c: per-item category 覆盖 SubNav-level (per 2026-09-02 17:32 JST) ----
  // 4 view 拆 4 个独立 it, 避免 vitest 多次 render 串扰
  const perItemItems: SubNavItem[] = [
    { id: "kanban", label: "Kanban", href: "/issues?view=kanban", category: "work" },
    { id: "list",   label: "List",   href: "/issues?view=list",   category: "agent" },
    { id: "tree",   label: "Tree",   href: "/issues?view=tree",   category: "integration" },
    { id: "sprint", label: "Sprint", href: "/issues?view=sprint", category: "system" },
  ];

  it("per-item: kanban (category=work) → blue-500 active", () => {
    render(<SubNav items={perItemItems} activeId="kanban" />);
    expect(screen.getByTestId("subnav-item-kanban").className).toMatch(/bg-blue-500\/20/);
  });
  it("per-item: list (category=agent) → emerald-500 active", () => {
    render(<SubNav items={perItemItems} activeId="list" />);
    expect(screen.getByTestId("subnav-item-list").className).toMatch(/bg-emerald-500\/20/);
  });
  it("per-item: tree (category=integration) → violet-500 active", () => {
    render(<SubNav items={perItemItems} activeId="tree" />);
    expect(screen.getByTestId("subnav-item-tree").className).toMatch(/bg-violet-500\/20/);
  });
  it("per-item: sprint (category=system) → amber-500 active", () => {
    render(<SubNav items={perItemItems} activeId="sprint" />);
    expect(screen.getByTestId("subnav-item-sprint").className).toMatch(/bg-amber-500\/20/);
  });

  // ---- Test 2d: per-item 不传 category 时 fallback SubNav-level ----
  it("falls back to SubNav-level category when item.category is undefined", () => {
    const itemsNoCategory: SubNavItem[] = [
      { id: "a", label: "A", href: "/x?a=1" },
      { id: "b", label: "B", href: "/x?b=2" },
    ];
    render(<SubNav items={itemsNoCategory} activeId="a" category="system" />);
    const a = screen.getByTestId("subnav-item-a");
    // fallback 到 system 域色: amber
    expect(a.className).toMatch(/bg-amber-500\/20/);
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




