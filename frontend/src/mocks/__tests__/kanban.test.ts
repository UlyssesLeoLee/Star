/*
@cypher schema=1 source_sha256=2902330ca04957732153de3085ea03c666db0cd75e2080e8f586fe69f76abfbb
MERGE (self:File {path:"frontend/src/mocks/__tests__/kanban.test.ts"})
MERGE (columns:Symbol {id:"frontend/src/mocks/data/kanban.ts::KANBAN_COLUMNS",kind:"constant"})
MERGE (self)-[:TESTS]->(columns)
@endcypher
*/

// frontend/src/mocks/__tests__/kanban.test.ts

import { describe, it, expect } from "vitest";
import { KANBAN_COLUMNS } from "@/mocks/data";

describe("KANBAN_COLUMNS", () => {
  it("has 4 Jira-consistent columns (blocked is a status, not a column)", () => {
    expect(KANBAN_COLUMNS).toHaveLength(4);
  });

  it("order: todo → in_progress → review → done (Jira consistent)", () => {
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
