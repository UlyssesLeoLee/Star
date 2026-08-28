"use client";

// =====================================================================
// SubNav — 嵌套子页面左导航 (per docs/frontend/design/ui-redesign-multica-style.md §4)
// =====================================================================
// 职责:
//   1. 180px 宽 sticky 侧栏 (top-16, 顶栏下方)
//   2. 渲染 items 列表 (id / label / href / 可选 count)
//   3. active 状态: bg-accent/12 + 左侧 2px accent border
//   4. hover 状态: bg-bg-soft/40
//   5. text-label uppercase (per multica 风格 §7 fontSize.label = 12px)
//   6. 自动用 usePathname() 决定 active — 也可由父组件传 activeId 覆盖
//   7. 提供 matchActive prop 让父组件自定义匹配逻辑 (e.g. /projects?tab=board)
//
// 设计原则 (per 守门):
//   - 不引外部 icon 库 (icons 用文本标签 — multica 风格以小写 label 为主)
//   - 不持久化 (per W5 store persist 模式, SubNav 状态在 URL 上)
//   - 不动 Sidebar.tsx (向后兼容 W1-W4 旧 page)
//
// 已知缺口 (per 缺标比错标):
//   - 暂未实现折叠/展开 (per §4 sticky 始终显示 — Phase 2+ 视用户反馈)
//   - 暂未实现拖拽排序 (per Phase 2+ 配合 Worktree 编排)
//   - 暂未实现嵌套子菜单 (e.g. /projects/board 嵌套 project id 路径) — Phase 2+
// =====================================================================

import Link from "next/link";
import { usePathname } from "next/navigation";
import { clsx } from "clsx";

export interface SubNavItem {
  id: string;
  label: string;
  href: string;
  count?: number;
  /** 标记为 group separator (上方加 1px divider) */
  divider?: boolean;
}

export interface SubNavProps {
  items: SubNavItem[];
  /** 自定义 active 匹配:默认用 pathname === item.href 或 startsWith(item.href + '/') */
  matchActive?: (item: SubNavItem, pathname: string | null) => boolean;
  /** 测试用, 强制 active id (覆盖 matchActive 和 pathname 判断) */
  activeId?: string;
  /** 顶部可选 label (per multica 风格) */
  topLabel?: string;
}

/** 默认 active 匹配: href 严格相等或子路径 */
function defaultMatch(item: SubNavItem, pathname: string | null): boolean {
  if (!pathname) return false;
  if (pathname === item.href) return true;
  // 子路径: /projects/board-x → /projects active
  if (item.href !== "/" && pathname.startsWith(item.href + "/")) return true;
  return false;
}

export function SubNav({
  items,
  matchActive,
  activeId,
  topLabel,
}: SubNavProps) {
  const pathname = usePathname();

  return (
    <aside
      data-testid="subnav"
      aria-label="Section navigation"
      className="w-[180px] shrink-0 border-r border-line bg-bg-soft/30 h-full sticky top-16"
    >
      {topLabel && (
        <div
          data-testid="subnav-top-label"
          className="px-4 py-3 text-[10px] uppercase tracking-wider text-ink-mute border-b border-line"
        >
          {topLabel}
        </div>
      )}
      <nav className="py-2" aria-label="Subnav items">
        {items.map((item) => {
          const isActive = activeId !== undefined
            ? activeId === item.id
            : matchActive
              ? matchActive(item, pathname)
              : defaultMatch(item, pathname);

          return (
            <div key={item.id} data-divider={item.divider ? "true" : undefined}>
              {item.divider && (
                <div className="my-2 mx-3 border-t border-line" aria-hidden />
              )}
              <Link
                href={item.href}
                data-testid={`subnav-item-${item.id}`}
                data-active={isActive ? "true" : "false"}
                aria-current={isActive ? "page" : undefined}
                className={clsx(
                  // 高度 36px = h-9, padding-left 16px = pl-4
                  "relative flex items-center justify-between h-9 pl-4 pr-3",
                  "text-label uppercase tracking-wider transition-colors",
                  "border-l-2",
                  isActive
                    ? "bg-accent/[0.12] text-accent border-l-accent"
                    : "text-ink-dim hover:bg-bg-soft/40 hover:text-ink border-l-transparent",
                )}
              >
                <span className="truncate">{item.label}</span>
                {item.count !== undefined && (
                  <span
                    data-testid={`subnav-count-${item.id}`}
                    className="ml-2 text-[10px] font-mono text-ink-mute"
                  >
                    {item.count}
                  </span>
                )}
              </Link>
            </div>
          );
        })}
      </nav>
    </aside>
  );
}

export default SubNav;
