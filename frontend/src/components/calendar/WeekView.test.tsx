// =====================================================================
// WeekView 单元测试 (per dynamic-interaction-design.md §5.2 + §11.3)
// 覆盖:
//   1. 渲染 7 天 (Sun-Sat)
//   2. 时区显示 (UTC + user TZ)
//   3. 模拟 work-item 拖动改日期
// =====================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { WeekView } from "./WeekView";
import type { CalendarEvent } from "./types";

const today = new Date(2026, 7, 28); // Friday

const sampleEvents: CalendarEvent[] = [
  { id: "wi-007", kind: "work_item", title: "PHYSIS-7 · Auto Rule", start_date: "2026-08-28T00:00:00.000Z", color: "err", badge: "P0" },
  { id: "wi-013", kind: "work_item", title: "PHYSIS-13 · Validation", start_date: "2026-08-30T00:00:00.000Z", color: "warn", badge: "P2" },
];

describe("WeekView", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(today);
  });

  it("renders 7 days", () => {
    render(<WeekView startDate={today} events={[]} onEventMove={() => {}} />);
    const days = screen.getAllByTestId("week-day");
    expect(days).toHaveLength(7);
  });

  it("shows timezone banner (UTC + user TZ)", () => {
    render(<WeekView startDate={today} events={[]} onEventMove={() => {}} userTimezone="Asia/Tokyo" />);
    const tz = screen.getByTestId("week-tz");
    expect(tz.textContent).toContain("UTC");
    expect(tz.textContent).toContain("Asia/Tokyo");
  });

  it("calls onEventMove on drag-drop of work-item to a different day", () => {
    const handleMove = vi.fn();
    render(<WeekView startDate={today} events={sampleEvents} onEventMove={handleMove} />);
    // 找 2026-08-30 (Sunday of next week 实际是 8/30)
    const days = screen.getAllByTestId("week-day");
    const day830 = days.find((d) => d.getAttribute("data-date") === "2026-08-30");
    expect(day830).toBeDefined();

    const dataTransfer = {
      getData: (type: string) => (type === "text/plain" ? "wi-007" : ""),
      types: ["text/plain"],
      dropEffect: "move",
    } as unknown as DataTransfer;

    fireEvent.drop(day830!, { dataTransfer });
    expect(handleMove).toHaveBeenCalledWith("wi-007", "2026-08-30");
  });

  it("renders work-item events in their respective day", () => {
    render(<WeekView startDate={today} events={sampleEvents} onEventMove={() => {}} />);
    const events = screen.getAllByTestId("week-event");
    // 至少这 2 个 (可能还有 sprint 但 sample 里没)
    const ids = events.map((e) => e.getAttribute("data-event-id"));
    expect(ids).toContain("wi-007");
    expect(ids).toContain("wi-013");
  });
});
