/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/components/context-menu/ContextMenu.tsx"})
MERGE (ContextMenuItem:Symbol {id:"frontend/src/components/context-menu/ContextMenu.tsx::ContextMenuItem",kind:"interface"})
MERGE (ContextMenuState:Symbol {id:"frontend/src/components/context-menu/ContextMenu.tsx::ContextMenuState",kind:"interface"})
MERGE (ContextMenu:Symbol {id:"frontend/src/components/context-menu/ContextMenu.tsx::ContextMenu",kind:"component"})
MERGE (self)-[:DEFINES]->(ContextMenuItem)
MERGE (self)-[:DEFINES]->(ContextMenuState)
MERGE (self)-[:DEFINES]->(ContextMenu)
@endcypher
*/
"use client";

import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { createPortal } from "react-dom";

/**
 * 共享右键菜单基建 (per canvas-ux 任务 §C)。
 *
 * - createPortal 到 document.body (避开画布 transform / z-index 层叠)
 * - clientX/Y 定位 + 视口边界收敛 (靠右/靠下翻转)
 * - Esc / 点击外部关闭; ArrowUp/Down + Enter 键盘可达; role="menu"/menuitem
 * - 稳定 testid: context-menu / context-menu-item-<id>
 * - 情景分派由调用方决定 (空白/单物件/多选/连接线/行/列头), 组件只负责渲染与交互
 */

export interface ContextMenuItem {
  /** 稳定 action id — 决定 testid `context-menu-item-<id>` */
  id: string;
  label: string;
  icon?: ReactNode;
  danger?: boolean;
  disabled?: boolean;
  /** disabled 时的 tooltip (诚实原因, e.g. 待后端接入) */
  disabledReason?: string;
  /** true 时渲染为分隔线, 无按钮 */
  separator?: boolean;
  onSelect?: () => void;
}

export interface ContextMenuState {
  open: boolean;
  x: number;
  y: number;
  items: ContextMenuItem[];
}

const MARGIN = 4;

/**
 * 渲染层: 接收 useContextMenu() 的 state, 关闭回调由 hook 提供。
 * state === null 或 open === false 时不渲染任何东西。
 */
export function ContextMenu({
  state,
  onClose,
}: {
  state: ContextMenuState | null;
  onClose: () => void;
}) {
  const [pos, setPos] = useState<{ x: number; y: number } | null>(null);
  const [focusIdx, setFocusIdx] = useState(0);
  const focusIdxRef = useRef(0);
  const menuRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<Array<HTMLButtonElement | null>>([]);

  // 可交互项的下标映射 (排除分隔线与 disabled), 供键盘导航定位
  const enabledIndex = useMemo(() => {
    const map = new Map<number, number>();
    let posIdx = 0;
    (state?.items ?? []).forEach((item, i) => {
      if (!item.separator && !item.disabled) {
        map.set(i, posIdx);
        posIdx += 1;
      }
    });
    return map;
  }, [state]);

  // 测量尺寸 → 视口边界收敛 (靠右/靠下翻转)
  useLayoutEffect(() => {
    if (!state?.open) {
      setPos(null);
      setFocusIdx(0);
      focusIdxRef.current = 0;
      return;
    }
    setFocusIdx(0);
    focusIdxRef.current = 0;
    const el = menuRef.current;
    if (!el) return;
    const w = el.offsetWidth;
    const h = el.offsetHeight;
    let x = state.x;
    let y = state.y;
    const vw = typeof window !== "undefined" ? window.innerWidth : 0;
    const vh = typeof window !== "undefined" ? window.innerHeight : 0;
    if (vw > 0 && x + w > vw - MARGIN) x = Math.max(MARGIN, vw - w - MARGIN);
    if (vh > 0 && y + h > vh - MARGIN) y = Math.max(MARGIN, vh - h - MARGIN);
    setPos({ x, y });
  }, [state]);

  const close = useCallback(() => {
    onClose();
  }, [onClose]);

  // Esc / 点击外部 / 键盘导航 (capture 阶段拦截, 阻止画布等全局快捷键误触发)
  useEffect(() => {
    if (!state?.open) return;
    const onPointerDown = (ev: PointerEvent) => {
      if (menuRef.current && !menuRef.current.contains(ev.target as Node)) {
        close();
      }
    };
    const onKeyDown = (ev: KeyboardEvent) => {
      if (ev.key === "Escape") {
        ev.preventDefault();
        ev.stopPropagation();
        close();
        return;
      }
      if (
        ev.key !== "ArrowDown" &&
        ev.key !== "ArrowUp" &&
        ev.key !== "Enter" &&
        ev.key !== "Home" &&
        ev.key !== "End"
      ) {
        return;
      }
      ev.preventDefault();
      ev.stopPropagation();
      const items = state.items.filter((i) => !i.separator && !i.disabled);
      if (items.length === 0) return;
      const idx = focusIdxRef.current;
      let next = idx;
      if (ev.key === "ArrowDown") next = Math.min(items.length - 1, idx + 1);
      else if (ev.key === "ArrowUp") next = Math.max(0, idx - 1);
      else if (ev.key === "Home") next = 0;
      else if (ev.key === "End") next = items.length - 1;
      else if (ev.key === "Enter") {
        items[idx]?.onSelect?.();
        close();
        return;
      }
      focusIdxRef.current = next;
      setFocusIdx(next);
    };
    window.addEventListener("pointerdown", onPointerDown, true);
    window.addEventListener("keydown", onKeyDown, true);
    return () => {
      window.removeEventListener("pointerdown", onPointerDown, true);
      window.removeEventListener("keydown", onKeyDown, true);
    };
  }, [state, close]);

  // 键盘导航后把焦点移到目标项
  useEffect(() => {
    if (!state?.open) return;
    itemRefs.current[focusIdx]?.focus();
  }, [focusIdx, state]);

  if (!state?.open || typeof document === "undefined") return null;

  return createPortal(
    <div
      ref={menuRef}
      role="menu"
      aria-label="context menu"
      data-testid="context-menu"
      className="fixed z-[10000] min-w-[180px] max-w-[280px] rounded-md border border-line bg-bg-card py-1 shadow-soft select-none"
      style={{
        left: pos ? pos.x : state.x,
        top: pos ? pos.y : state.y,
        visibility: pos ? "visible" : "hidden",
      }}
      onContextMenu={(e) => e.preventDefault()}
    >
      {state.items.map((item, i) => {
        if (item.separator) {
          return <div key={item.id || `sep-${i}`} role="separator" className="my-1 h-px bg-line" />;
        }
        const enIdx = enabledIndex.get(i);
        return (
          <button
            key={item.id}
            type="button"
            role="menuitem"
            tabIndex={enIdx === focusIdx ? 0 : -1}
            ref={(el) => {
              if (enIdx !== undefined) itemRefs.current[enIdx] = el;
            }}
            data-testid={`context-menu-item-${item.id}`}
            disabled={item.disabled}
            title={item.disabled ? item.disabledReason : undefined}
            onClick={() => {
              if (item.disabled) return;
              item.onSelect?.();
              close();
            }}
            className={`flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-xs ${
              item.danger ? "text-err" : "text-ink"
            } hover:bg-bg-soft focus:bg-bg-soft focus:outline-none disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:bg-transparent`}
          >
            {item.icon ? <span className="shrink-0">{item.icon}</span> : null}
            <span className="flex-1 truncate">{item.label}</span>
          </button>
        );
      })}
    </div>,
    document.body,
  );
}
