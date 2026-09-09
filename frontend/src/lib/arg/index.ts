// =====================================================================
// frontend/src/lib/arg/index.ts — public re-exports for the arg module
// =====================================================================
export * from "./types";
export * as argApi from "./api";
export {
  ArgWebSocketClient,
  wsBridgeToArgEvent,
  type ArgWSOptions,
  type WSStatus,
} from "./ws";
export {
  useARGStore,
  selectAgentCount,
  selectEdgeCount,
  selectUnlockedCount,
  selectEdgesByAgent,
  type ARGStoreState,
} from "./store";
