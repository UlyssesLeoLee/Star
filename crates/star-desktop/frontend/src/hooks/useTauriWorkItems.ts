// crates/star-desktop/frontend/src/hooks/useTauriWorkItems.ts
// =====================================================================
// Star Desktop — Tauri IPC React hook (PR-250 follow-up of PR #249)
// Per docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P5 阶段 3
//
// 调用 Tauri IPC `list_work_items` (per PR #245 MockDb), 返回 WorkItem[] + loading/error state
// Fallback: 当 Tauri 不存在 (browser dev mode), 用 mock data
// =====================================================================

import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { WorkItem } from "../types/ids";

interface UseTauriWorkItemsResult {
  workItems: WorkItem[];
  loading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

const MOCK_WORK_ITEMS: WorkItem[] = [
  {
    id: "wi-001", key: "STAR-001", title: "feat(canvas): W/T/M swimlane",
    status: "in_progress", w_t_m: "W", kind: "story", priority: "P1", story_points: 3,
  },
  {
    id: "wi-002", key: "STAR-002", title: "fix(canvas): bug #71",
    status: "review", w_t_m: "T", kind: "bug", priority: "P0", story_points: 5,
  },
  {
    id: "wi-003", key: "STAR-003", title: "spike(wasm): layout PoC",
    status: "done", w_t_m: "M", kind: "spike", priority: "P3", story_points: 1,
  },
  {
    id: "wi-004", key: "STAR-004", title: "doc(arch): Tauri PoC",
    status: "todo", w_t_m: "M", kind: "task", priority: "P2", story_points: 2,
  },
];

export function useTauriWorkItems(): UseTauriWorkItemsResult {
  const [workItems, setWorkItems] = useState<WorkItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchWorkItems = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      // Tauri IPC: invoke `list_work_items` (per PR #245)
      const items = await invoke<WorkItem[]>("list_work_items");
      setWorkItems(items);
    } catch (e) {
      // Fallback to mock when not in Tauri runtime
      // (e.g., running in Vite dev server outside Tauri shell)
      if (e instanceof Error && e.message.includes("not implemented")) {
        setWorkItems(MOCK_WORK_ITEMS);
      } else if (
        typeof window !== "undefined" &&
        // @ts-ignore — __TAURI_INTERNALS__ injected by Tauri runtime
        !window.__TAURI_INTERNALS__
      ) {
        // Browser dev mode: use mock
        setWorkItems(MOCK_WORK_ITEMS);
      } else {
        setError(e instanceof Error ? e : new Error(String(e)));
      }
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchWorkItems();
  }, [fetchWorkItems]);

  return { workItems, loading, error, refetch: fetchWorkItems };
}
