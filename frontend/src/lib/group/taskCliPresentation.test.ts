/*
@cypher schema=1 source_sha256=1076dc23aa386d362fd139acccb210c723bef522368d39f2dba9d044cc98de82
MERGE (self:File {path:"frontend/src/lib/group/taskCliPresentation.test.ts"})
MERGE (resolve:Function {name:"resolveTaskCliUiState",language:"typescript"})
MERGE (self)-[:TESTS]->(resolve)
@endcypher
*/

import { describe, expect, it } from "vitest";
import { resolveTaskCliUiState } from "./taskCliPresentation";

describe("resolveTaskCliUiState", () => {
  const authorized = {
    hasAuthorizedSession: true,
    uncertain: false,
    needsReattach: false,
    sessionState: null,
  } as const;

  it("keeps the terminal unavailable until a validated session receipt exists", () => {
    expect(resolveTaskCliUiState({ ...authorized, hasAuthorizedSession: false })).toBe("unavailable");
  });

  it("allows the terminal only for a confirmed running authorized session", () => {
    expect(resolveTaskCliUiState(authorized)).toBe("ready");
    expect(resolveTaskCliUiState({ ...authorized, sessionState: "running" })).toBe("ready");
  });

  it("hides the terminal while session status is uncertain or reconnect is required", () => {
    expect(resolveTaskCliUiState({ ...authorized, uncertain: true })).toBe("uncertain");
    expect(resolveTaskCliUiState({ ...authorized, needsReattach: true })).toBe("reattach");
    expect(resolveTaskCliUiState({ ...authorized, sessionState: "disconnected" })).toBe("reattach");
  });

  it("does not present starting or cancelling sessions as usable terminals", () => {
    expect(resolveTaskCliUiState({ ...authorized, sessionState: "starting" })).toBe("not-running");
    expect(resolveTaskCliUiState({ ...authorized, sessionState: "cancelling" })).toBe("not-running");
  });

  it("keeps every terminal session outcome distinct from task completion", () => {
    for (const sessionState of ["completed", "failed", "cancelled", "timed_out", "lost"] as const) {
      expect(resolveTaskCliUiState({ ...authorized, sessionState })).toBe("ended");
    }
  });
});
