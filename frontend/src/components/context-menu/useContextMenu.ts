/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/components/context-menu/useContextMenu.ts"})
MERGE (useContextMenu:Symbol {id:"frontend/src/components/context-menu/useContextMenu.ts::useContextMenu",kind:"function"})
MERGE (self)-[:DEFINES]->(useContextMenu)
@endcypher
*/
"use client";

import { useCallback, useState } from "react";
import type { ContextMenuItem, ContextMenuState } from "./ContextMenu";

/** openMenu 接受的坐标来源 (React.MouseEvent / DOM MouseEvent 均满足)。 */
export interface OpenMenuEventLike {
  clientX: number;
  clientY: number;
  preventDefault?: () => void;
  stopPropagation?: () => void;
}

/**
 * 共享右键菜单 hook。
 *
 * 用法:
 *   const { menu, openMenu, closeMenu } = useContextMenu();
 *   <div onContextMenu={(e) => openMenu(e, items)}>…</div>
 *   <ContextMenu state={menu} onClose={closeMenu} />
 *
 * openMenu 内部已 preventDefault + stopPropagation (抑制浏览器原生菜单与
 * 父容器 onContextMenu 冒泡)。情景分派 (空白/单物件/多选/连接线/行/列头)
 * 由调用方在构造 items 时决定。
 */
export function useContextMenu() {
  const [menu, setMenu] = useState<ContextMenuState | null>(null);

  const closeMenu = useCallback(() => {
    setMenu(null);
  }, []);

  const openMenu = useCallback((e: OpenMenuEventLike, items: ContextMenuItem[]) => {
    e.preventDefault?.();
    e.stopPropagation?.();
    setMenu({ open: true, x: e.clientX, y: e.clientY, items });
  }, []);

  return { menu, openMenu, closeMenu };
}
