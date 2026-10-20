/*
@cypher schema=1 source_sha256=85ca29135574d2f307921caff38137285493a4599b2984d67bdb08c3aed387db
MERGE (self:File {path:"frontend/src/lib/group/taskCliPresentation.ts"})
MERGE (resolve:Function {name:"resolveTaskCliUiState",language:"typescript",visibility:"public"})
MERGE (sessionState:Type {name:"TaskCliSessionState",language:"typescript"})
MERGE (self)-[:DEFINES]->(resolve)
MERGE (self)-[:IMPORTS]->(sessionState)
MERGE (resolve)-[:USES_TYPE]->(sessionState)
@endcypher
*/

import type { TaskCliSessionState } from "@/lib/group/worktreeGroupApi";

export type TaskCliUiState = "unavailable" | "ready" | "reattach" | "not-running" | "ended" | "uncertain";

export interface TaskCliUiStateInput {
  hasAuthorizedSession: boolean;
  uncertain: boolean;
  needsReattach: boolean;
  sessionState: TaskCliSessionState | null;
}

/** Only a validated server session receipt can make the terminal surface available. */
export function resolveTaskCliUiState(input: TaskCliUiStateInput): TaskCliUiState {
  if (!input.hasAuthorizedSession) return "unavailable";
  if (input.uncertain) return "uncertain";
  if (input.needsReattach || input.sessionState === "disconnected") return "reattach";
  if (input.sessionState === "starting" || input.sessionState === "cancelling") return "not-running";
  if (input.sessionState && input.sessionState !== "running") return "ended";
  return "ready";
}
