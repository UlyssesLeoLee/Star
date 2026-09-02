// =====================================================================
// Tooltip.test.tsx — 漫画气泡 Tooltip 组件单测
// =====================================================================
// 覆盖: 渲染 / hover 触发 / leave 关闭 / focus 键盘可达 / Esc 关闭 /
//       视口边界翻转方向 / 受控开关 / disabled
// =====================================================================

import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { render, screen, cleanup, fireEvent, act } from "@testing-library/react";
import * as React from "react";
import { Tooltip } from "./tooltip";

// ── 几何 mock: 让 trigger 跟 tooltip 都有可预测的 rect ──────────────
const TRIGGER_RECT = {
  left: 100, top: 100, right: 200, bottom: 150,
  width: 100, height: 50, x: 100, y: 100,
  toJSON() { return this; },
} as DOMRect;

const TIP_RECT_TOP = {
  // top side: tip 在 trigger 上方, 期望落在 (50, 100 - 30 - 8) = (50, 62)
  left: 0, top: 0, right: 100, bottom: 30,
  width: 100, height: 30, x: 0, y: 0,
  toJSON() { return this; },
} as DOMRect;

const TIP_RECT_BOTTOM = {
  // bottom side: tip 在 trigger 下方, 期望落在 (50, 158)
  left: 0, top: 0, right: 100, bottom: 30,
  width: 100, height: 30, x: 0, y: 0,
  toJSON() { return this; },
} as DOMRect;

let currentTipRect: DOMRect = TIP_RECT_TOP;

beforeEach(() => {
  currentTipRect = TIP_RECT_TOP;
  // 默认视口 1024x768 (jsdom 默认) — 偏好 top 一定能放下
  Object.defineProperty(window, "innerWidth", { configurable: true, value: 1024 });
  Object.defineProperty(window, "innerHeight", { configurable: true, value: 768 });

  Element.prototype.getBoundingClientRect = function () {
    const el = this as HTMLElement;
    if (el.getAttribute("data-tooltip-trigger") !== null) return TRIGGER_RECT;
    if (el.getAttribute("role") === "tooltip") return currentTipRect;
    return {
      left: 0, top: 0, right: 0, bottom: 0, width: 0, height: 0, x: 0, y: 0,
      toJSON() { return this; },
    } as DOMRect;
  };

  // 替换 requestAnimationFrame → 同步执行, 避免 useLayoutEffect 排队等待
  // jsdom 没有 RAF, 但 useLayoutEffect 在测试环境是同步的, 这里只是双保险
  if (typeof window !== "undefined" && !window.requestAnimationFrame) {
    window.requestAnimationFrame = ((cb: FrameRequestCallback) => {
      cb(0);
      return 0;
    }) as typeof window.requestAnimationFrame;
  }
});

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

function flush() {
  act(() => {
    // 让 React 把状态更新 + useLayoutEffect 跑完
  });
}

describe("Tooltip", () => {
  it("渲染 trigger, 初始不显示气泡", () => {
    render(
      <Tooltip content="hello">
        <button data-testid="trigger">hover me</button>
      </Tooltip>
    );
    expect(screen.getByTestId("trigger")).toBeInTheDocument();
    expect(screen.queryByRole("tooltip")).toBeNull();
  });

  it("hover 显示气泡 (含 role=tooltip + 内容)", () => {
    vi.useFakeTimers();
    render(
      <Tooltip content="这是一段说明" delayShow={0}>
        <button data-testid="trigger">hover me</button>
      </Tooltip>
    );
    fireEvent.mouseEnter(screen.getByTestId("trigger"));
    act(() => {
      vi.runAllTimers();
    });
    const tip = screen.getByRole("tooltip");
    expect(tip).toBeInTheDocument();
    expect(tip).toHaveTextContent("这是一段说明");
    expect(tip).toHaveAttribute("data-side", "top");
  });

  it("mouseleave 关闭气泡", () => {
    vi.useFakeTimers();
    render(
      <Tooltip content="x" delayShow={0} delayHide={0}>
        <button data-testid="trigger">x</button>
      </Tooltip>
    );
    const trigger = screen.getByTestId("trigger");
    fireEvent.mouseEnter(trigger);
    act(() => vi.runAllTimers());
    expect(screen.queryByRole("tooltip")).not.toBeNull();
    fireEvent.mouseLeave(trigger);
    act(() => vi.runAllTimers());
    expect(screen.queryByRole("tooltip")).toBeNull();
  });

  it("focus 立即显示, blur 关闭 (键盘可达)", () => {
    render(
      <Tooltip content="kb">
        <button data-testid="trigger">x</button>
      </Tooltip>
    );
    const trigger = screen.getByTestId("trigger");
    fireEvent.focus(trigger);
    expect(screen.queryByRole("tooltip")).not.toBeNull();
    fireEvent.blur(trigger);
    expect(screen.queryByRole("tooltip")).toBeNull();
  });

  it("Esc 关闭气泡", () => {
    render(
      <Tooltip content="esc">
        <button data-testid="trigger">x</button>
      </Tooltip>
    );
    fireEvent.focus(screen.getByTestId("trigger"));
    expect(screen.queryByRole("tooltip")).not.toBeNull();
    fireEvent.keyDown(document, { key: "Escape" });
    expect(screen.queryByRole("tooltip")).toBeNull();
  });

  it("偏好 top 但视口上方空间不足 → 自动翻转到 bottom", () => {
    // 构造一个 trigger 紧贴视口顶部, 上面只剩 50px, tip 高 30 + gap 8 = 38, 放得下
    // 所以要 trigger 顶部 - 38 < 0 才能翻. 改 top = 20, 上面只剩 20, 放不下
    const tightTop: DOMRect = {
      left: 100, top: 20, right: 200, bottom: 70,
      width: 100, height: 50, x: 100, y: 20,
      toJSON() { return this; },
    } as DOMRect;
    Element.prototype.getBoundingClientRect = function () {
      const el = this as HTMLElement;
      if (el.getAttribute("data-tooltip-trigger") !== null) return tightTop;
      if (el.getAttribute("role") === "tooltip") return TIP_RECT_BOTTOM;
      return {
        left: 0, top: 0, right: 0, bottom: 0, width: 0, height: 0, x: 0, y: 0,
        toJSON() { return this; },
      } as DOMRect;
    };
    render(
      <Tooltip content="flip" side="top">
        <button data-testid="trigger">x</button>
      </Tooltip>
    );
    fireEvent.focus(screen.getByTestId("trigger"));
    expect(screen.getByRole("tooltip")).toHaveAttribute("data-side", "bottom");
  });

  it("受控 open: open=true 强制显示, open=false 强制隐藏", () => {
    render(
      <Tooltip content="ctrl" open onOpenChange={() => {}}>
        <button data-testid="trigger">x</button>
      </Tooltip>
    );
    expect(screen.queryByRole("tooltip")).not.toBeNull();
    cleanup();
    render(
      <Tooltip content="ctrl" open={false} onOpenChange={() => {}}>
        <button data-testid="trigger">x</button>
      </Tooltip>
    );
    expect(screen.queryByRole("tooltip")).toBeNull();
  });

  it("disabled 时 hover 不显示气泡", () => {
    vi.useFakeTimers();
    render(
      <Tooltip content="no" delayShow={0} disabled>
        <button data-testid="trigger">x</button>
      </Tooltip>
    );
    fireEvent.mouseEnter(screen.getByTestId("trigger"));
    act(() => vi.runAllTimers());
    expect(screen.queryByRole("tooltip")).toBeNull();
  });
});
