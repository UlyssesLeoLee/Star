// =====================================================================
// MonthView 单元测试 (per dynamic-interaction-design.md §5.2 + §11.3)
// 覆盖:
//   1. 渲染 7x6 网格 (42 cells)
//   2. 跨月日期灰显 (data-in-month="0")
//   3. 模拟 drop → 调 onEventMove
// =====================================================================
//
// 用 vitest + @testing-library/react. 不强求 CI 跑 (build 验证只看 TS 编译),
// 但本地能 npm test 跑通.
//
// W3 worker 2026-08-28
// =====================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { MonthView } from "./MonthView";
import type { CalendarEvent } from "./types";

const today = new Date(2026, 7, 28); // 2026-08-28 (local)

const sampleEvents: CalendarEvent[] = [
  { id: "ms-001", kind: "milestone", title: "MVP 0.5", start_date: "2026-08-15T00:00:00.000Z", color: "info", badge: "85%" },
  { id: "wi-007", kind: "work_item", title: "PHYSIS-7 · Auto Rule", start_date: "2026-08-28T00:00:00.000Z", color: "err", badge: "P0" },
];

describe("MonthView", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(today);
  });

  it("renders a 7x6 grid (42 cells)", () => {
    render(<MonthView year={2026} month={7} events={[]} onEventMove={() => {}} />);
    const cells = screen.getAllByTestId("month-cell");
    expect(cells).toHaveLength(42);
  });

  it("greys out cross-month dates (data-in-month=0)", () => {
    render(<MonthView year={2026} month={7} events={[]} onEventMove={() => {}} />);
    const cells = screen.getAllByTestId("month-cell");
    // 2026-08-01 是 Saturday, 8 月只有 31 天, 所以 9/1-9/5 跨月 (周日 - 周四)
    const outOfMonth = cells.filter((c) => c.getAttribute("data-in-month") === "0");
    expect(outOfMonth.length).toBeGreaterThan(0);
    // 至少 1 个 in-month cell
    const inMonth = cells.filter((c) => c.getAttribute("data-in-month") === "1");
    expect(inMonth.length).toBeGreaterThanOrEqual(28);
    expect(inMonth.length).toBeLessThanOrEqual(31);
  });

  it("calls onEventMove(eventId, newDate) on drop", () => {
    const handleMove = vi.fn();
    render(<MonthView year={2026} month={7} events={sampleEvents} onEventMove={handleMove} />);
    // 找 today 所在的 cell (8/28)
    const cells = screen.getAllByTestId("month-cell");
    const cell2026_08_30 = cells.find((c) => c.getAttribute("data-date") === "2026-08-30");
    expect(cell2026_08_30).toBeDefined();

    // 模拟 drop wi-007 到 8/30
    const dataTransfer = {
      getData: (type: string) => (type === "text/plain" ? "wi-007" : ""),
      types: ["text/plain"],
      dropEffect: "move",
    } as unknown as DataTransfer;

    fireEvent.drop(cell2026_08_30!, { dataTransfer });
    expect(handleMove).toHaveBeenCalledWith("wi-007", "2026-08-30");
  });

  it("renders event badges in cells with matching date", () => {
    render(<MonthView year={2026} month={7} events={sampleEvents} onEventMove={() => {}} />);
    const wi7 = screen.getAllByTestId("day-event").find((b) => b.getAttribute("data-event-id") === "wi-007");
    expect(wi7).toBeDefined();
    expect(wi7!.getAttribute("data-event-kind")).toBe("work_item");
  });
});
