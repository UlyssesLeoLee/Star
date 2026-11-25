// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
@cypher schema=1 source_sha256=placeholder
MERGE (self:File {path:"frontend/src/components/nav/ShortcutButtons.tsx"})
MERGE (component:Symbol {id:"frontend/src/components/nav/ShortcutButtons.tsx::ShortcutButtons",kind:"component"})
MERGE (nav:Symbol {id:"frontend/src/lib/nav/navStore.ts::useNavStore",kind:"hook"})
MERGE (link:Symbol {id:"frontend/src/components/navigation/GuardedLink.tsx::GuardedLink",kind:"component"})
MERGE (self)-[:DEFINES]->(component)
MERGE (component)-[:IMPORTS]->(nav),(component)-[:IMPORTS]->(link)
MERGE (component)-[:READS]->(nav)
MERGE (component)-[:USES]->(link)
@endcypher
*/
"use client";

/**
 * 自定义快捷按钮渲染（per G-6 关闭条件 + 2026-11-22 用户发令）。
 *
 * 此前 `shortcutButtons` 只存在于 navStore，**没有任何渲染处** ——
 * 用户添加的快捷按钮不会出现在任何地方（store-only 特性）。
 * 本组件补上渲染位。
 *
 * **Tab 感知跳转**（G-6 的核心）：快捷按钮带 `targetTab` 时，点击应打开
 * **目标页面的那个选项卡**，而不是该页面的默认选项卡。
 * 实现方式：复用 `GuardedLink` 的 `targetTab` 能力（它把 `targetTab`
 * 拼成 `?tab=<targetTab>` 并 `router.push`），而不是自己拼 URL ——
 * 保证与站内其它 tab 跳转行为一致（含未保存导航守卫）。
 */

import { clsx } from "clsx";
import { usePathname, useSearchParams } from "next/navigation";
import { useMemo } from "react";
import { GuardedLink } from "@/components/navigation/GuardedLink";
import { useNavStore, type ShortcutButton } from "@/lib/nav/navStore";

/** 快捷按钮当前是否激活（路径匹配 + tab 匹配）。 */
function isActive(
  button: ShortcutButton,
  pathname: string,
  currentTab: string | null,
): boolean {
  const [buttonPath, buttonQuery] = button.href.split("?");
  if (pathname !== buttonPath) return false;
  if (!button.targetTab) return true;
  // 带 targetTab 时必须 tab 也一致才算激活
  if (!currentTab) return false;
  if (buttonQuery) {
    const params = new URLSearchParams(buttonQuery);
    if (params.get("tab") !== currentTab) return false;
  }
  return button.targetTab === currentTab;
}

export function ShortcutButtons({ className }: { className?: string }) {
  const buttons = useNavStore((s) => s.shortcutButtons);
  const pathname = usePathname() ?? "/";
  const searchParams = useSearchParams();
  const currentTab = searchParams.get("tab");

  const sorted = useMemo(
    () =>
      [...buttons].sort(
        (a, b) => (a.order ?? 0) - (b.order ?? 0) || a.createdAt.localeCompare(b.createdAt),
      ),
    [buttons],
  );

  if (sorted.length === 0) return null;

  return (
    <nav
      aria-label="自定义快捷入口"
      data-testid="shortcut-buttons"
      className={clsx("flex flex-wrap items-center gap-1", className)}
    >
      {sorted.map((button) => {
        const active = isActive(button, pathname, currentTab);
        return (
          <GuardedLink
            key={button.id}
            href={button.href}
            // tab 感知跳转：交给 GuardedLink 统一处理（拼 ?tab= + router.push）
            targetTab={button.targetTab}
            data-testid={`shortcut-button-${button.id}`}
            data-active={active ? "true" : "false"}
            aria-current={active ? "page" : undefined}
            title={button.label}
            className={clsx(
              "inline-flex items-center gap-1 rounded border border-line px-2 py-1",
              "text-[11px] font-mono transition-colors",
              "hover:border-accent hover:text-accent",
              "focus-visible:outline focus-visible:outline-accent",
              active
                ? "border-accent/60 bg-accent/10 text-accent"
                : "bg-bg-card text-ink-dim",
            )}
          >
            <span className="truncate">{button.label}</span>
            {button.targetTab ? (
              <span
                aria-hidden="true"
                data-testid={`shortcut-button-tab-${button.id}`}
                className="rounded bg-accent/15 px-1 text-[9px] text-accent"
              >
                {button.targetTab}
              </span>
            ) : null}
          </GuardedLink>
        );
      })}
    </nav>
  );
}

export default ShortcutButtons;
