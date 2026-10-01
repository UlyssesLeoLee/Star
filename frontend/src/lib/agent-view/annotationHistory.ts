// =====================================================================
// /agent-view annotation undo/redo history (per 2026-10-01 OOB 任务 #5)
// =====================================================================
// 设计: 每条历史 = annotations + freeConnectors snapshot.
// undo: pop tail, push to redo
// redo: pop redo, push to current
// 新操作 (create/update/delete/move) → 清空 redo stack
// =====================================================================

import type {
  AgentCanvasAnnotation, AgentCanvasFreeConnector,
} from "@/lib/agent-view/types";

export interface AnnotationSnapshot {
  annotations: AgentCanvasAnnotation[];
  freeConnectors: AgentCanvasFreeConnector[];
}

export interface AnnotationHistoryState {
  past: AnnotationSnapshot[];
  current: AnnotationSnapshot;
  future: AnnotationSnapshot[];
}

/**
 * 创建初始状态 (空)
 */
export function emptyHistory(): AnnotationHistoryState {
  return { past: [], current: { annotations: [], freeConnectors: [] }, future: [] };
}

/**
 * 初始化 — 用现成数据作当前状态, history 为空
 */
export function initHistory(
  annotations: AgentCanvasAnnotation[],
  freeConnectors: AgentCanvasFreeConnector[],
): AnnotationHistoryState {
  return { past: [], current: { annotations, freeConnectors }, future: [] };
}

const MAX_HISTORY = 50;

/**
 *  push 新状态 (commit operation). 清空 redo stack.
 */
export function commit(
  state: AnnotationHistoryState,
  next: AnnotationSnapshot,
): AnnotationHistoryState {
  if (next.annotations.length === state.current.annotations.length &&
      next.freeConnectors.length === state.current.freeConnectors.length) {
    // Same length — still need diff check on positions. Skip if identical refs.
    let identical = true;
    for (let i = 0; i < next.annotations.length; i += 1) {
      if (next.annotations[i] !== state.current.annotations[i]) {
        identical = false;
        break;
      }
    }
    if (identical) {
      for (let i = 0; i < next.freeConnectors.length; i += 1) {
        if (next.freeConnectors[i] !== state.current.freeConnectors[i]) {
          identical = false;
          break;
        }
      }
      if (identical) return state;
    }
  }
  const newPast = [...state.past, state.current];
  if (newPast.length > MAX_HISTORY) newPast.shift();
  return { past: newPast, current: next, future: [] };
}

/** undo — 返回上一 snapshot, 同时把它加到 future stack */
export function undo(state: AnnotationHistoryState): AnnotationHistoryState {
  if (state.past.length === 0) return state;
  const previous = state.past[state.past.length - 1];
  return {
    past: state.past.slice(0, -1),
    current: previous,
    future: [...state.future, state.current],
  };
}

/** redo — 取 future stack 顶, 加到 past */
export function redo(state: AnnotationHistoryState): AnnotationHistoryState {
  if (state.future.length === 0) return state;
  const next = state.future[state.future.length - 1];
  return {
    past: [...state.past, state.current],
    current: next,
    future: state.future.slice(0, -1),
  };
}

/** canUndo / canRedo 简单 helper */
export function canUndo(state: AnnotationHistoryState): boolean {
  return state.past.length > 0;
}
export function canRedo(state: AnnotationHistoryState): boolean {
  return state.future.length > 0;
}
