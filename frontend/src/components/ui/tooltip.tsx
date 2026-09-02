"use client";

// =====================================================================
// Tooltip — 漫画气泡 (manga-bubble) 悬停说明
// =====================================================================
// 设计要点 (per 2026-09-02 11:06 JST 用户发令):
//   · 外观: 黑边 + 白底 + 黑字, 漫画分镜框风格
//   · 箭头: 黑色描边三角 + 白色填充三角错位 1px, 形成漫画尖角
//   · 方向: 4 向 (top/right/bottom/left) 偏好 + 视口边界自动翻转
//   · 箭头位置: 跟随 trigger 中线, clamp 到气泡内 8..tipSize-8 范围
//   · 触发: hover (默认 200ms 延时) + focus (即时, 键盘可达)
//   · 关闭: mouseleave / blur / Esc 键 / 受控
//   · 容器: createPortal 到 body, 避免被父级 overflow / stacking 裁剪
//   · i18n: 接受 ReactNode, 文案由调用方传入已翻译字符串 (e.g. `t.board.fallbackColumnProtected`)
//           组件本身不翻译内容; 仅暴露可复用的 `tooltip.*` 文案给默认场景
// =====================================================================

import * as React from "react";
import { createPortal } from "react-dom";

export type TooltipSide = "top" | "right" | "bottom" | "left";

export interface TooltipProps {
  /** 触发器 — 任意 React 元素 (cloneElement 注入 ref + 事件). */
  children: React.ReactElement;
  /**
   * 提示内容 — 任意 ReactNode.
   * **i18n 约定**: 文案由调用方负责翻译, 组件不二次翻译.
   * 用法: `<Tooltip content={t.board.fallbackColumnProtected}>` ...
   */
  content: React.ReactNode;
  /** 偏好方向, 默认 "top"; 实际位置会随 viewport 边界自动翻转. */
  side?: TooltipSide;
  /** 气泡与 trigger 的间距 (px), 默认 8. */
  sideOffset?: number;
  /** 悬停多少 ms 后显示, 默认 200; 传 0 即时. */
  delayShow?: number;
  /** 离开多少 ms 后隐藏, 默认 100. */
  delayHide?: number;
  /** 受控开关 (不传则组件自管). */
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  /** 气泡本体追加 className (e.g. max-w). */
  className?: string;
  /** 强制不显示 (e.g. 父级 disable 状态). */
  disabled?: boolean;
  /** tooltip 本体 data-testid. */
  "data-testid"?: string;
}

// ── 几何常量 ─────────────────────────────────────────────────────────
const VIEWPORT_PAD = 4;       // 距视口边缘最小保留 px
const ARROW_OUTER = 7;        // 箭头外边 (黑色描边三角)
const ARROW_INNER = 6;        // 箭头内边 (白色填充三角, 1px 错位形成黑边)
const ARROW_INSET_MIN = 10;   // 箭头距气泡边的最小值 (避免贴角)
const Z_INDEX = 9999;

// ── 位置算法 ─────────────────────────────────────────────────────────
interface TipSize { width: number; height: number; }

function fitOnSide(
  side: TooltipSide,
  trigger: DOMRect,
  tip: TipSize,
  gap: number,
  vw: number,
  vh: number,
): { x: number; y: number; side: TooltipSide; fits: boolean } {
  const tCenterX = trigger.left + trigger.width / 2;
  const tCenterY = trigger.top + trigger.height / 2;
  let x = 0;
  let y = 0;
  switch (side) {
    case "top":
      x = tCenterX - tip.width / 2;
      y = trigger.top - tip.height - gap;
      break;
    case "bottom":
      x = tCenterX - tip.width / 2;
      y = trigger.bottom + gap;
      break;
    case "left":
      x = trigger.left - tip.width - gap;
      y = tCenterY - tip.height / 2;
      break;
    case "right":
      x = trigger.right + gap;
      y = tCenterY - tip.height / 2;
      break;
  }
  const clampedX = Math.max(VIEWPORT_PAD, Math.min(x, vw - tip.width - VIEWPORT_PAD));
  const clampedY = Math.max(VIEWPORT_PAD, Math.min(y, vh - tip.height - VIEWPORT_PAD));
  return { x: clampedX, y: clampedY, side, fits: clampedX === x && clampedY === y };
}

function computePosition(
  trigger: DOMRect,
  tip: TipSize,
  preferred: TooltipSide,
  gap: number,
): { x: number; y: number; side: TooltipSide } {
  const vw = typeof window !== "undefined" ? window.innerWidth : 1024;
  const vh = typeof window !== "undefined" ? window.innerHeight : 768;
  const opposite: Record<TooltipSide, TooltipSide> = {
    top: "bottom", bottom: "top", left: "right", right: "left",
  };
  const others: TooltipSide[] = (["top", "right", "bottom", "left"] as TooltipSide[])
    .filter((s) => s !== preferred && s !== opposite[preferred]);
  const order: TooltipSide[] = [preferred, opposite[preferred], ...others];
  for (const s of order) {
    const r = fitOnSide(s, trigger, tip, gap, vw, vh);
    if (r.fits) return { x: r.x, y: r.y, side: r.side };
  }
  // 全部超出 → 选偏好侧, 接受 clamp
  const r = fitOnSide(preferred, trigger, tip, gap, vw, vh);
  return { x: r.x, y: r.y, side: r.side };
}

function arrowOffset(
  side: TooltipSide,
  trigger: DOMRect,
  tip: TipSize,
  finalX: number,
  finalY: number,
): { x: number; y: number } {
  const tCenterX = trigger.left + trigger.width / 2;
  const tCenterY = trigger.top + trigger.height / 2;
  let ax = 0;
  let ay = 0;
  if (side === "top" || side === "bottom") {
    ax = tCenterX - finalX;
    ax = Math.max(ARROW_INSET_MIN, Math.min(ax, tip.width - ARROW_INSET_MIN));
  } else {
    ay = tCenterY - finalY;
    ay = Math.max(ARROW_INSET_MIN, Math.min(ay, tip.height - ARROW_INSET_MIN));
  }
  return { x: ax, y: ay };
}

// ── 箭头 SVG-style 渲染 (双三角错位 1px = 漫画黑边) ────────────────
interface ArrowProps { side: TooltipSide; x: number; y: number; }
function Arrow({ side, x, y }: ArrowProps) {
  // outer = 黑边, inner = 白填充, 错位 1px 形成 1px 黑边描边
  const common: React.CSSProperties = {
    position: "absolute",
    width: 0,
    height: 0,
    pointerEvents: "none",
  };
  const outer: React.CSSProperties = { ...common };
  const inner: React.CSSProperties = { ...common };
  // 三角指向 trigger 一侧, 自身在气泡内
  if (side === "top") {
    // 箭头朝上, 贴在气泡下边 (因为 tooltip 在 trigger 上方)
    Object.assign(outer, {
      bottom: -ARROW_OUTER,
      left: x - ARROW_OUTER,
      borderLeft: `${ARROW_OUTER}px solid transparent`,
      borderRight: `${ARROW_OUTER}px solid transparent`,
      borderBottom: `${ARROW_OUTER}px solid #0d1117`,
    });
    Object.assign(inner, {
      bottom: -(ARROW_OUTER - 1),
      left: x - ARROW_INNER,
      borderLeft: `${ARROW_INNER}px solid transparent`,
      borderRight: `${ARROW_INNER}px solid transparent`,
      borderBottom: `${ARROW_INNER}px solid #ffffff`,
    });
  } else if (side === "bottom") {
    Object.assign(outer, {
      top: -ARROW_OUTER,
      left: x - ARROW_OUTER,
      borderLeft: `${ARROW_OUTER}px solid transparent`,
      borderRight: `${ARROW_OUTER}px solid transparent`,
      borderTop: `${ARROW_OUTER}px solid #0d1117`,
    });
    Object.assign(inner, {
      top: -(ARROW_OUTER - 1),
      left: x - ARROW_INNER,
      borderLeft: `${ARROW_INNER}px solid transparent`,
      borderRight: `${ARROW_INNER}px solid transparent`,
      borderTop: `${ARROW_INNER}px solid #ffffff`,
    });
  } else if (side === "left") {
    Object.assign(outer, {
      right: -ARROW_OUTER,
      top: y - ARROW_OUTER,
      borderTop: `${ARROW_OUTER}px solid transparent`,
      borderBottom: `${ARROW_OUTER}px solid transparent`,
      borderLeft: `${ARROW_OUTER}px solid #0d1117`,
    });
    Object.assign(inner, {
      right: -(ARROW_OUTER - 1),
      top: y - ARROW_INNER,
      borderTop: `${ARROW_INNER}px solid transparent`,
      borderBottom: `${ARROW_INNER}px solid transparent`,
      borderLeft: `${ARROW_INNER}px solid #ffffff`,
    });
  } else {
    // right
    Object.assign(outer, {
      left: -ARROW_OUTER,
      top: y - ARROW_OUTER,
      borderTop: `${ARROW_OUTER}px solid transparent`,
      borderBottom: `${ARROW_OUTER}px solid transparent`,
      borderRight: `${ARROW_OUTER}px solid #0d1117`,
    });
    Object.assign(inner, {
      left: -(ARROW_OUTER - 1),
      top: y - ARROW_INNER,
      borderTop: `${ARROW_INNER}px solid transparent`,
      borderBottom: `${ARROW_INNER}px solid transparent`,
      borderRight: `${ARROW_INNER}px solid #ffffff`,
    });
  }
  return (
    <>
      <div data-testid="tooltip-arrow-outer" style={outer} aria-hidden />
      <div data-testid="tooltip-arrow-inner" style={inner} aria-hidden />
    </>
  );
}

// ── 合并 ref 工具 ────────────────────────────────────────────────────
function mergeRefs<T>(...refs: Array<React.Ref<T> | undefined>) {
  return (node: T | null) => {
    for (const r of refs) {
      if (!r) continue;
      if (typeof r === "function") r(node);
      else (r as React.MutableRefObject<T | null>).current = node;
    }
  };
}

// ── 主组件 ───────────────────────────────────────────────────────────
export function Tooltip({
  children,
  content,
  side: preferredSide = "top",
  sideOffset = 8,
  delayShow = 200,
  delayHide = 100,
  open: controlledOpen,
  onOpenChange,
  className,
  disabled = false,
  "data-testid": testId,
}: TooltipProps) {
  const isControlled = controlledOpen !== undefined;
  const [internalOpen, setInternalOpen] = React.useState(false);
  const open = isControlled ? !!controlledOpen : internalOpen;

  const [mounted, setMounted] = React.useState(false);
  React.useEffect(() => setMounted(true), []);

  const triggerRef = React.useRef<HTMLElement | null>(null);
  const tipRef = React.useRef<HTMLDivElement | null>(null);
  const [pos, setPos] = React.useState<
    | { x: number; y: number; side: TooltipSide; ax: number; ay: number }
    | null
  >(null);

  const showTimer = React.useRef<number | null>(null);
  const hideTimer = React.useRef<number | null>(null);
  const tipIdRef = React.useRef(`tooltip-${Math.random().toString(36).slice(2, 9)}`);

  const clearTimers = () => {
    if (showTimer.current !== null) {
      window.clearTimeout(showTimer.current);
      showTimer.current = null;
    }
    if (hideTimer.current !== null) {
      window.clearTimeout(hideTimer.current);
      hideTimer.current = null;
    }
  };

  const setOpen = React.useCallback(
    (next: boolean) => {
      if (disabled) return;
      if (!isControlled) setInternalOpen(next);
      onOpenChange?.(next);
    },
    [disabled, isControlled, onOpenChange],
  );

  const measure = React.useCallback(() => {
    const t = triggerRef.current;
    const tip = tipRef.current;
    if (!t || !tip) return;
    const rect = t.getBoundingClientRect();
    const tipRect = tip.getBoundingClientRect();
    const placed = computePosition(rect, { width: tipRect.width, height: tipRect.height }, preferredSide, sideOffset);
    const arr = arrowOffset(placed.side, rect, tipRect, placed.x, placed.y);
    setPos({ x: placed.x, y: placed.y, side: placed.side, ax: arr.x, ay: arr.y });
  }, [preferredSide, sideOffset]);

  // open → 立即渲染气泡 (visibility:hidden 占位) → useLayoutEffect 量尺寸 → 二次渲染显示
  // 解决"pos 依赖 tipRect, tipRect 依赖元素挂载, 元素挂载依赖 pos"的死锁
  // mounted 入 deps: 首次渲染 useEffect 设 mounted=true 后, effect 会再跑一次量尺寸
  React.useLayoutEffect(() => {
    if (!open || !mounted) return;
    measure();
    const onReflow = () => measure();
    window.addEventListener("resize", onReflow);
    window.addEventListener("scroll", onReflow, true);
    return () => {
      window.removeEventListener("resize", onReflow);
      window.removeEventListener("scroll", onReflow, true);
    };
  }, [open, mounted, measure]);

  // 卸载清理
  React.useEffect(() => {
    return () => clearTimers();
  }, []);

  // Esc 关闭
  React.useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        clearTimers();
        setOpen(false);
      }
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [open, setOpen]);

  // 触发器 props 注入
  const childProps = children.props as Record<string, unknown>;
  const origRef = (childProps as { ref?: React.Ref<HTMLElement> }).ref;
  const triggerHandlers = {
    onMouseEnter: (e: React.MouseEvent<HTMLElement>) => {
      (childProps.onMouseEnter as ((ev: React.MouseEvent<HTMLElement>) => void) | undefined)?.(e);
      if (disabled) return;
      clearTimers();
      if (delayShow <= 0) setOpen(true);
      else showTimer.current = window.setTimeout(() => setOpen(true), delayShow);
    },
    onMouseLeave: (e: React.MouseEvent<HTMLElement>) => {
      (childProps.onMouseLeave as ((ev: React.MouseEvent<HTMLElement>) => void) | undefined)?.(e);
      if (disabled) return;
      clearTimers();
      if (delayHide <= 0) setOpen(false);
      else hideTimer.current = window.setTimeout(() => setOpen(false), delayHide);
    },
    onFocus: (e: React.FocusEvent<HTMLElement>) => {
      (childProps.onFocus as ((ev: React.FocusEvent<HTMLElement>) => void) | undefined)?.(e);
      if (disabled) return;
      clearTimers();
      setOpen(true);
    },
    onBlur: (e: React.FocusEvent<HTMLElement>) => {
      (childProps.onBlur as ((ev: React.FocusEvent<HTMLElement>) => void) | undefined)?.(e);
      if (disabled) return;
      clearTimers();
      setOpen(false);
    },
  } as const;

  const trigger = React.cloneElement(children, {
    ref: mergeRefs<HTMLElement>(triggerRef, origRef as React.Ref<HTMLElement> | undefined),
    "aria-describedby": open ? tipIdRef.current : (childProps["aria-describedby"] as string | undefined),
    "data-tooltip-trigger": "",
    ...triggerHandlers,
  });

  const tooltipNode =
    mounted && open ? (
      <div
        ref={tipRef}
        id={tipIdRef.current}
        role="tooltip"
        data-testid={testId ?? "tooltip"}
        data-side={pos?.side ?? preferredSide}
        className={className}
        style={{
          position: "fixed",
          left: pos?.x ?? 0,
          top: pos?.y ?? 0,
          zIndex: Z_INDEX,
          maxWidth: 280,
          padding: "6px 10px",
          background: "#ffffff",
          color: "#0d1117",
          border: "1.5px solid #0d1117",
          borderRadius: 4,
          fontSize: 12,
          lineHeight: 1.4,
          fontFamily:
            "ui-sans-serif, system-ui, -apple-system, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, 'PingFang SC', 'Hiragino Sans', 'Microsoft YaHei', sans-serif",
          fontWeight: 500,
          boxShadow: "2px 2px 0 rgba(13, 17, 23, 0.18)",
          pointerEvents: "none",
          whiteSpace: "normal",
          wordBreak: "break-word",
          // 量尺寸时先隐藏, 算完位置再显示, 避免视觉跳动
          visibility: pos ? "visible" : "hidden",
        }}
      >
        {content}
        {pos ? <Arrow side={pos.side} x={pos.ax} y={pos.ay} /> : null}
      </div>
    ) : null;

  return (
    <>
      {trigger}
      {mounted && tooltipNode ? createPortal(tooltipNode, document.body) : null}
    </>
  );
}

export default Tooltip;
