// =====================================================================
// subNavRegistry — SubNav 数据源 (per 2026-09-03 12:36 JST 拍板)
// =====================================================================
// 设计目标:
//   1. SubNav items 抽象成共享 registry, 不再是 page-local useState
//   2. Sidebar 的 "project scope" 从这里读 (复用 SubNav 数据源, 不新建)
//   3. 保留原 page-level SubNav 渲染 — 不破坏现有 SubNav.test.tsx
//   4. count 等动态数据由 caller 通过 getItems(pathname, context) 传入
//
// 路径匹配策略:
//   - exactMatch: pathname 严格相等 (e.g. /projects 命中 kanban tab?tab=kanban)
//   - prefixMatch: pathname 前缀匹配 (e.g. /projects?tab=kanban 命中 kanban)
//   - 用 query 字符串简化匹配, 不解析 URLSearchParams (跟 page-local SubNav 风格一致)
//
// 未来扩展 (per 守门 #11 缺标比错标):
//   - 每个 view 的 count 应该走 /api 实时拉, 当前用 mock placeholder
//   - per-project count 应该跟 selectedProjectId 关联
//   - 用户自定义 view (隐藏某些 view, 排顺序) 走 navStore 持久化
// =====================================================================

import {
  Trello,
  SquareChartGantt,
  LayoutDashboard,
  Users,
  GitBranch,
  ListTodo,
  Workflow,
  Calendar,
  type LucideIcon,
} from "lucide-react";
import type { ModuleCategory } from "./registry";

/**
 * SubNav 单项 (跟 components/SubNav.tsx 兼容)
 * 注意: 这里的 href 在生成时拼上 query string, 不存静态 href
 */
export interface SubNavEntry {
  id: string;
  label: string;
  code: string;
  icon: LucideIcon;
  category: ModuleCategory;
  /**
   * query 字符串片段 (e.g. "tab=kanban" / "view=list"), 拼到 pathname 后形成最终 href.
   * 不含前导 "?"。
   */
  query: string;
}

/**
 * 一组 SubNav items, 挂在某个 pathname 前缀下
 */
export interface SubNavGroup {
  /** 路径前缀, pathname 命中时这组 items 全部展开 (e.g. "/projects") */
  pathnamePrefix: string;
  /** 顶部 label, 跟 SubNav topLabel 一致 (per 2026-09-02 Jira 风格) */
  topLabel: string;
  /** 该组的主 category, 决定 left border / bg / text 默认色 (per-item category 优先) */
  category: ModuleCategory;
  items: SubNavEntry[];
}

/**
 * SubNav 共享注册表 (per 2026-09-03 拍板)
 *
 * 注册的 group:
 *   - /projects  → Projects 5 tab (Kanban / Timeline / Backlog / Agents / Worktrees)
 *   - /issues    → Issues 4 view (Kanban / List / Tree / Sprint, 跟现有 SubNav 数据保持一致)
 *
 * 后续 page 接入 SubNav 只需在此注册, Sidebar "project scope" 自动可用.
 */
export const SUBNAV_REGISTRY: SubNavGroup[] = [
  {
    pathnamePrefix: "/projects",
    topLabel: "Project",
    category: "work",
    items: [
      {
        id: "kanban",
        label: "Kanban",
        code: "KB",
        icon: Trello,
        category: "work",
        query: "tab=kanban",
      },
      {
        id: "timeline",
        label: "Timeline",
        code: "TL",
        icon: SquareChartGantt,
        category: "work",
        query: "tab=timeline",
      },
      {
        id: "backlog",
        label: "Backlog",
        code: "BL",
        icon: LayoutDashboard,
        category: "work",
        query: "tab=backlog",
      },
      {
        id: "agents",
        label: "Agents",
        code: "AGT",
        icon: Users,
        category: "agent",
        query: "tab=agents",
      },
      {
        id: "worktrees",
        label: "Worktrees",
        code: "WT",
        icon: GitBranch,
        category: "agent",
        query: "tab=worktrees",
      },
    ],
  },
  {
    pathnamePrefix: "/sprint",
    topLabel: "Sprint",
    category: "work",
    items: [
      // Per 2026-09-05 19:13 JST 拍板: 删 Kanban view (用户明确不需要看板)
      {
        id: "sprint",
        label: "Sprint",
        code: "SPR",
        icon: Calendar,
        category: "system",
        query: "view=sprint",
      },
      {
        id: "list",
        label: "List",
        code: "LST",
        icon: ListTodo,
        category: "agent",
        query: "view=list",
      },
      {
        id: "tree",
        label: "Tree",
        code: "TRE",
        icon: Workflow,
        category: "integration",
        query: "view=tree",
      },
    ],
  },
];

/**
 * 根据 pathname 找到第一个匹配的 SubNav group
 * 命中规则: pathname === prefix 或 pathname.startsWith(prefix + "/") 或 pathname.startsWith(prefix + "?")
 */
export function findSubNavGroup(pathname: string | null): SubNavGroup | null {
  if (!pathname) return null;
  for (const group of SUBNAV_REGISTRY) {
    if (
      pathname === group.pathnamePrefix ||
      pathname.startsWith(group.pathnamePrefix + "/") ||
      pathname.startsWith(group.pathnamePrefix + "?")
    ) {
      return group;
    }
  }
  return null;
}

/**
 * 根据 pathname + 当前 query 字符串, 决定 active item id
 * 例如 pathname=/projects, search="?tab=kanban" → "kanban"
 */
export function findActiveSubNavItem(
  group: SubNavGroup,
  searchParams: string | null
): string | null {
  if (!searchParams) return null;
  // search 形如 "?tab=kanban" 或 "" — 先 strip "?"
  const cleaned = searchParams.startsWith("?") ? searchParams.slice(1) : searchParams;
  if (!cleaned) return null;
  for (const item of group.items) {
    // item.query 形如 "tab=kanban", 检测 cleaned 里是否包含
    const [k, v] = item.query.split("=");
    if (cleaned.includes(`${k}=${v}`)) {
      return item.id;
    }
  }
  return null;
}
