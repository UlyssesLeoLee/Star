// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/lib/hooks/use-annotations.ts
//
// Diff Annotation zustand store + hooks (per FR-ORCA-035 §11 v1.0, ULYS-211).
//
// 为什么用 zustand 而不是 react-query (同目录 use-credentials.ts 用的是后者):
//   DiffAnnotator 的 gutter 每行都要问"这行有没有 annotation", 需要一份
//   跨组件共享、按 file_path 索引的本地投影; 且 feed_to_agent 的结果是
//   一次性 prompt fragment, 不适合放 query cache。store 形态与
//   src/stores/worktreeCanvasStore.ts 一致 (独立 store 文件, 0 改既有 store)。
//
// created_at ASC 排序由**前端**兜底: annotate.rs 的 `list_for_file` 直接
// 遍历 HashMap::values(), 顺序不保证 (只有 `list_for_agent_run` 走
// by_agent_run 索引才是 ASC)。

"use client";

import { useCallback, useEffect } from "react";
import { create } from "zustand";

import {
  annotationsApi,
  subscribeAnnotations,
  type CreateAnnotationRequest,
  type DiffAnnotation,
} from "@/lib/api/annotations";

/** 稳定引用 — 避免 selector 每次 new [] 触发无限 re-render */
const EMPTY: readonly DiffAnnotation[] = Object.freeze([]);

function byCreatedAtAsc(a: DiffAnnotation, b: DiffAnnotation): number {
  const d = a.created_at.localeCompare(b.created_at);
  // created_at 同毫秒时用 id 兜底, 保证顺序稳定
  return d !== 0 ? d : a.id.localeCompare(b.id);
}

function errorMessage(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

export interface AnnotationState {
  /** file_path → annotations (created_at ASC) */
  byFile: Record<string, DiffAnnotation[]>;
  /** file_path → 是否在加载 */
  loadingFile: Record<string, boolean>;
  /** 最近一次操作的错误 (null = 无) */
  error: string | null;
  /** feed_to_agent 的产出; null = 尚未 feed 或该 run 无 annotation */
  promptFragment: string | null;
  feeding: boolean;

  /** `list_for_file` — 拉某文件全部 annotation 到 store */
  loadForFile: (filePath: string) => Promise<void>;
  /** `add` — 成功返回新建的 annotation, 失败返回 null 并置 error */
  addAnnotation: (req: CreateAnnotationRequest) => Promise<DiffAnnotation | null>;
  /** `delete` */
  deleteAnnotation: (id: string, filePath: string) => Promise<void>;
  /** `feed_to_agent` — 返回 prompt fragment (该 run 无 annotation 时为 null) */
  feedToAgent: (agentRunId: string) => Promise<string | null>;

  /** 本地 upsert (WebSocket 推送 / 乐观更新用) */
  upsertLocal: (annotation: DiffAnnotation) => void;
  /** 本地移除 (WebSocket 推送用; 不知道 file_path 时全表扫) */
  removeLocal: (id: string) => void;
  clearPromptFragment: () => void;
  clearError: () => void;
  reset: () => void;
}

export const useAnnotationStore = create<AnnotationState>((set, get) => ({
  byFile: {},
  loadingFile: {},
  error: null,
  promptFragment: null,
  feeding: false,

  loadForFile: async (filePath) => {
    set((s) => ({ loadingFile: { ...s.loadingFile, [filePath]: true }, error: null }));
    try {
      const list = await annotationsApi.listForFile(filePath);
      set((s) => ({
        byFile: { ...s.byFile, [filePath]: [...list].sort(byCreatedAtAsc) },
        loadingFile: { ...s.loadingFile, [filePath]: false },
      }));
    } catch (err) {
      set((s) => ({
        loadingFile: { ...s.loadingFile, [filePath]: false },
        error: errorMessage(err),
      }));
    }
  },

  addAnnotation: async (req) => {
    set({ error: null });
    try {
      const created = await annotationsApi.add(req);
      get().upsertLocal(created);
      return created;
    } catch (err) {
      set({ error: errorMessage(err) });
      return null;
    }
  },

  deleteAnnotation: async (id, filePath) => {
    set({ error: null });
    try {
      await annotationsApi.delete(id);
      set((s) => ({
        byFile: {
          ...s.byFile,
          [filePath]: (s.byFile[filePath] ?? []).filter((a) => a.id !== id),
        },
      }));
    } catch (err) {
      set({ error: errorMessage(err) });
    }
  },

  feedToAgent: async (agentRunId) => {
    set({ feeding: true, error: null });
    try {
      const res = await annotationsApi.feedToAgent(agentRunId);
      set({ promptFragment: res.prompt_fragment, feeding: false });
      return res.prompt_fragment;
    } catch (err) {
      set({ feeding: false, error: errorMessage(err) });
      return null;
    }
  },

  upsertLocal: (annotation) =>
    set((s) => {
      const current = s.byFile[annotation.file_path] ?? [];
      const next = current.some((a) => a.id === annotation.id)
        ? current.map((a) => (a.id === annotation.id ? annotation : a))
        : [...current, annotation];
      return {
        byFile: { ...s.byFile, [annotation.file_path]: next.sort(byCreatedAtAsc) },
      };
    }),

  removeLocal: (id) =>
    set((s) => {
      const byFile: Record<string, DiffAnnotation[]> = {};
      for (const [path, list] of Object.entries(s.byFile)) {
        byFile[path] = list.filter((a) => a.id !== id);
      }
      return { byFile };
    }),

  clearPromptFragment: () => set({ promptFragment: null }),
  clearError: () => set({ error: null }),
  reset: () =>
    set({
      byFile: {},
      loadingFile: {},
      error: null,
      promptFragment: null,
      feeding: false,
    }),
}));

// =====================================================================
// hooks
// =====================================================================

/** 某文件的 annotation 列表 (created_at ASC); 无数据时返回稳定空数组 */
export function useAnnotationsForFile(filePath: string): readonly DiffAnnotation[] {
  return useAnnotationStore((s) => s.byFile[filePath] ?? EMPTY);
}

/** 某文件是否在加载中 */
export function useAnnotationsLoading(filePath: string): boolean {
  return useAnnotationStore((s) => s.loadingFile[filePath] ?? false);
}

/**
 * mount 时 `list_for_file`, 并 (若后端支持) 订阅 WebSocket 增量。
 * 返回该文件当前 annotation 列表。
 */
export function useFileAnnotations(
  filePath: string,
  agentRunId: string,
): readonly DiffAnnotation[] {
  const loadForFile = useAnnotationStore((s) => s.loadForFile);
  const upsertLocal = useAnnotationStore((s) => s.upsertLocal);
  const removeLocal = useAnnotationStore((s) => s.removeLocal);

  useEffect(() => {
    if (!filePath) return;
    void loadForFile(filePath);
  }, [filePath, loadForFile]);

  useEffect(() => {
    if (!agentRunId) return;
    // ws endpoint 缺位时 subscribeAnnotations 返回 no-op unsubscribe (见 api 层注释)
    return subscribeAnnotations(agentRunId, {
      onEvent: (ev) => {
        if (ev.type === "annotation_added") upsertLocal(ev.annotation);
        else removeLocal(ev.id);
      },
      // ws 不可用不是 UI 级错误: REST 仍是真源, 静默即可
      onError: () => undefined,
    });
  }, [agentRunId, upsertLocal, removeLocal]);

  return useAnnotationsForFile(filePath);
}

/** feed_to_agent 动作 + 结果 */
export function useFeedToAgent(agentRunId: string) {
  const feedToAgentAction = useAnnotationStore((s) => s.feedToAgent);
  const promptFragment = useAnnotationStore((s) => s.promptFragment);
  const feeding = useAnnotationStore((s) => s.feeding);
  const clearPromptFragment = useAnnotationStore((s) => s.clearPromptFragment);

  const feed = useCallback(
    () => feedToAgentAction(agentRunId),
    [agentRunId, feedToAgentAction],
  );

  return { feed, promptFragment, feeding, clearPromptFragment };
}
