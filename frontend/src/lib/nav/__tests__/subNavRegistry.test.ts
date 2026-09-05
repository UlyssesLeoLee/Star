// =====================================================================
// subNavRegistry.test.ts — SubNav 数据源注册表测试 (per 2026-09-03 12:36 JST 拍板)
// =====================================================================
// 覆盖:
//   1. findSubNavGroup: pathname 命中逻辑 (严格相等 / 前缀 / 跨子路径 / 跨 query)
//   2. findActiveSubNavItem: query 字符串解析, 决定 active item id
//   3. 边界: 空字符串 / null / 不存在的 pathname
// =====================================================================

import { describe, it, expect } from "vitest";
import {
  SUBNAV_REGISTRY,
  findSubNavGroup,
  findActiveSubNavItem,
} from "../subNavRegistry";

describe("subNavRegistry — findSubNavGroup", () => {
  it("matches /projects strict equality", () => {
    const group = findSubNavGroup("/projects");
    expect(group).not.toBeNull();
    expect(group?.pathnamePrefix).toBe("/projects");
    expect(group?.items.map((i) => i.id)).toEqual([
      "kanban",
      "timeline",
      "backlog",
      "agents",
      "worktrees",
    ]);
  });

  it("matches /projects?tab=kanban (query string)", () => {
    const group = findSubNavGroup("/projects?tab=kanban");
    expect(group?.pathnamePrefix).toBe("/projects");
  });

  it("matches /projects/X (subpath)", () => {
    const group = findSubNavGroup("/projects/abc-123");
    expect(group?.pathnamePrefix).toBe("/projects");
  });

  it("matches /sprint strict equality (per 2026-09-05 19:13 JST 拍板: /issues 重命名 /sprint, Kanban 已删)", () => {
    const group = findSubNavGroup("/sprint");
    expect(group?.pathnamePrefix).toBe("/sprint");
    expect(group?.items.map((i) => i.id)).toEqual([
      "sprint",
      "list",
      "tree",
    ]);
  });

  it("matches /issues?view=list (query string)", () => {
    const group = findSubNavGroup("/sprint?view=list");
    expect(group?.pathnamePrefix).toBe("/sprint");
  });

  it("returns null for non-registered pathnames", () => {
    expect(findSubNavGroup("/inbox")).toBeNull();
    expect(findSubNavGroup("/settings")).toBeNull();
    expect(findSubNavGroup("/agents")).toBeNull();
  });

  it("returns null for null pathname", () => {
    expect(findSubNavGroup(null)).toBeNull();
  });

  it("returns null for empty string", () => {
    expect(findSubNavGroup("")).toBeNull();
  });

  it("first match wins (e.g. /projects wins over /projects/X variants)", () => {
    const group = findSubNavGroup("/projects/sub/whatever");
    expect(group?.pathnamePrefix).toBe("/projects");
  });
});

describe("subNavRegistry — findActiveSubNavItem", () => {
  it("returns 'kanban' for /projects?tab=kanban", () => {
    const group = findSubNavGroup("/projects");
    expect(findActiveSubNavItem(group!, "?tab=kanban")).toBe("kanban");
  });

  it("returns 'timeline' for /projects?tab=timeline", () => {
    const group = findSubNavGroup("/projects");
    expect(findActiveSubNavItem(group!, "?tab=timeline")).toBe("timeline");
  });

  it("returns 'agents' for /projects?tab=agents", () => {
    const group = findSubNavGroup("/projects");
    expect(findActiveSubNavItem(group!, "?tab=agents")).toBe("agents");
  });

  it("returns null when query is empty (default tab state)", () => {
    const group = findSubNavGroup("/projects");
    expect(findActiveSubNavItem(group!, "")).toBeNull();
    expect(findActiveSubNavItem(group!, null)).toBeNull();
  });

  it("returns null when query is set but no item matches", () => {
    const group = findSubNavGroup("/projects");
    expect(findActiveSubNavItem(group!, "?foo=bar")).toBeNull();
  });

  it("works for /issues with view= query param", () => {
    const group = findSubNavGroup("/sprint");
    expect(findActiveSubNavItem(group!, "?view=list")).toBe("list");
    expect(findActiveSubNavItem(group!, "?view=tree")).toBe("tree");
    expect(findActiveSubNavItem(group!, "?view=sprint")).toBe("sprint");
  });

  it("accepts search string without leading ?", () => {
    const group = findSubNavGroup("/projects");
    expect(findActiveSubNavItem(group!, "tab=kanban")).toBe("kanban");
  });
});

describe("subNavRegistry — registry shape", () => {
  it("has at least 2 groups registered (per 2026-09-03 拍板 #1: /projects + /issues)", () => {
    expect(SUBNAV_REGISTRY.length).toBeGreaterThanOrEqual(2);
  });

  it("every group has pathnamePrefix / topLabel / items with at least 1 item", () => {
    SUBNAV_REGISTRY.forEach((g) => {
      expect(g.pathnamePrefix).toMatch(/^\//);
      expect(typeof g.topLabel).toBe("string");
      expect(g.items.length).toBeGreaterThan(0);
      g.items.forEach((item) => {
        expect(item.id).toBeTruthy();
        expect(item.label).toBeTruthy();
        expect(item.code).toBeTruthy();
        expect(item.query).toMatch(/^[a-z]+=[a-z]+$/);
      });
    });
  });
});
