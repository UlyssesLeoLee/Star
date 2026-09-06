// =====================================================================
// GanttBar unit tests (per W2 任务 §4 + design §11.3)
//
// 验收:
//   - 拖动时 style.left 实时变化
//   - 颜色按 status (todo 灰 / in_progress 蓝 / done 绿 / blocked 红)
// =====================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, fireEvent, screen } from "@testing-library/react";
import { GanttBar } from "./GanttBar";

const CHART_START = "2026-01-01"; // chart origin
const SPRINT_START = "2026-02-01"; // sprint day 0 from chart
const SPRINT_END = "2026-02-15";   // 14 day sprint (day 31-45 from chart)

describe("GanttBar", () => {
  beforeEach(() => {
    // ensure no leftover document listeners
    document.body.innerHTML = "";
  });

  it("renders with the right background color for each status", () => {
    const statuses: Array<{
      status: Parameters<typeof GanttBar>[0]["item"]["status"];
      expected: string;
    }> = [
      { status: "todo", expected: "var(--ink-mute, #475569)" },
      { status: "in_progress", expected: "var(--info-DEFAULT, #58a6ff)" },
      { status: "done", expected: "var(--ok-DEFAULT, #10b981)" },
      { status: "blocked", expected: "var(--err-DEFAULT, #ff3366)" },
      { status: "review", expected: "var(--warn-DEFAULT, #f59e0b)" },
      // per DRIFT-α-017: active/planned 原硬编码与 StatusPill (sprint/page.tsx:628
      // 渲染同一个 SprintStatus 字段) 不一致, 已修正 — 这两条锁定修正后的值
      { status: "active", expected: "var(--ok-DEFAULT, #10b981)" },
      { status: "planned", expected: "var(--info-DEFAULT, #58a6ff)" },
      { status: "cancelled", expected: "var(--ink-mute, #475569)" },
    ];

    for (const { status, expected } of statuses) {
      const { container, unmount } = render(
        <GanttBar
          item={{ id: `bar-${status}`, label: `bar ${status}`, status }}
          startDate={SPRINT_START}
          endDate={SPRINT_END}
          dateRangeStart={CHART_START}
          pxPerDay={10}
        />,
      );
      const el = container.querySelector('[data-testid="gantt-bar"]') as HTMLElement;
      expect(el, `bar for status=${status} should render`).toBeTruthy();
      expect(el.dataset.barStatus).toBe(status);
      // per DRIFT-α-017: 引用 theme.css 语义色变量而非硬编码 hex, jsdom 不解析 var()
      expect(el.style.backgroundColor).toBe(expected);
      unmount();
    }
  });

  it("critical path overrides status color with red", () => {
    const { container } = render(
      <GanttBar
        item={{ id: "ms-crit", label: "Critical", status: "done" }}
        startDate={SPRINT_START}
        endDate={SPRINT_END}
        dateRangeStart={CHART_START}
        pxPerDay={10}
        isCritical
      />,
    );
    const el = container.querySelector('[data-testid="gantt-bar"]') as HTMLElement;
    expect(el.style.backgroundColor).toBe("var(--err-DEFAULT, #ff3366)");
    expect(el.dataset.barCritical).toBe("true");
  });

  it("drag updates style.left in real-time during mousemove", () => {
    const onDragEnd = vi.fn();
    const { container } = render(
      <GanttBar
        item={{ id: "bar-drag", label: "Drag me", status: "in_progress" }}
        startDate={SPRINT_START}
        endDate={SPRINT_END}
        dateRangeStart={CHART_START}
        pxPerDay={10}
        onDragEnd={onDragEnd}
      />,
    );
    const el = container.querySelector('[data-testid="gantt-bar"]') as HTMLElement;
    const initialLeft = el.style.left;
    expect(initialLeft).toBeTruthy();

    // Simulate mousedown at x=100
    fireEvent.mouseDown(el, { clientX: 100, preventDefault: () => {}, stopPropagation: () => {} });

    // mousemove (document-level) — drag delta +50px
    fireEvent.mouseMove(document, { clientX: 150 });
    // After mousemove, style.left should reflect the 50px drag offset
    // initial baseLeft (Feb 1 = day 31 from Jan 1) = 31 * 10 = 310px; + 50 delta = 360px
    // jsdom converts "360px" string to set on style.left
    const afterMoveLeft = el.style.left;
    expect(afterMoveLeft).not.toBe(initialLeft);
    // Should equal baseLeft + 50
    const baseLeft = 31 * 10;
    const expected = `${baseLeft + 50}px`;
    expect(afterMoveLeft).toBe(expected);

    // mouseup -> onDragEnd fires with new dates
    fireEvent.mouseUp(document);
    expect(onDragEnd).toHaveBeenCalledTimes(1);
    const [newStart, newEnd] = onDragEnd.mock.calls[0];
    // 50px / 10px/day = +5 days
    expect(newStart).toBe("2026-02-06");
    expect(newEnd).toBe("2026-02-20");
  });

  it("does not call onDragEnd if delta is 0 (no-op drag)", () => {
    const onDragEnd = vi.fn();
    const { container } = render(
      <GanttBar
        item={{ id: "bar-noop", label: "Noop", status: "in_progress" }}
        startDate={SPRINT_START}
        endDate={SPRINT_END}
        dateRangeStart={CHART_START}
        pxPerDay={10}
        onDragEnd={onDragEnd}
      />,
    );
    const el = container.querySelector('[data-testid="gantt-bar"]') as HTMLElement;
    fireEvent.mouseDown(el, { clientX: 100, preventDefault: () => {}, stopPropagation: () => {} });
    fireEvent.mouseMove(document, { clientX: 100 }); // same x
    fireEvent.mouseUp(document);
    expect(onDragEnd).not.toHaveBeenCalled();
  });

  it("onClick is called when bar is not draggable (no onDragEnd)", () => {
    const onClick = vi.fn();
    const { container } = render(
      <GanttBar
        item={{ id: "bar-click", label: "Click me", status: "done" }}
        startDate={SPRINT_START}
        endDate={SPRINT_END}
        dateRangeStart={CHART_START}
        pxPerDay={10}
        onClick={onClick}
      />,
    );
    const el = container.querySelector('[data-testid="gantt-bar"]') as HTMLElement;
    fireEvent.click(el);
    expect(onClick).toHaveBeenCalledTimes(1);
  });
});
