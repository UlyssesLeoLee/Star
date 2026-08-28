// frontend/src/mocks/__tests__/kanban.test.ts

import { describe, it, expect } from "vitest";
import { KANBAN_COLUMNS } from "@/mocks/data";

describe("KANBAN_COLUMNS", () => {
  it("has 4 columns (per W1 dynamic-interaction-design §3.4)", () => {
    expect(KANBAN_COLUMNS).toHaveLength(4);
  });

  it("order: todo → in_progress → review → done", () => {
    expect(KANBAN_COLUMNS).toEqual([
      "todo",
      "in_progress",
      "review",
      "done",
    ]);
  });

  it("all values are unique (no duplicate columns)", () => {
    expect(new Set(KANBAN_COLUMNS).size).toBe(KANBAN_COLUMNS.length);
  });
});
