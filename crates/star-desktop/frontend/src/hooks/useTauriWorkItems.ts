// crates/star-desktop/frontend/src/hooks/useTauriWorkItems.ts
// =====================================================================
// Star Desktop — Tauri IPC React hook (PR-250 follow-up of PR #249)
// Per docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P5 阶段 3
//
// 调用 Tauri IPC `list_work_items`，返回 Run-owned WorkItem[] + loading/error state
// 缺少 Tauri/Run provider 时 fail closed，不展示本地 mock Task Card。
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

export function useTauriWorkItems(): UseTauriWorkItemsResult {
  const [workItems, setWorkItems] = useState<WorkItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchWorkItems = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      // Tauri IPC: invoke canonical Run-scoped task projection.
      const items = await invoke<WorkItem[]>("list_work_items");
      setWorkItems(items);
    } catch (e) {
      setWorkItems([]);
      setError(e instanceof Error ? e : new Error(String(e)));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchWorkItems();
  }, [fetchWorkItems]);

  return { workItems, loading, error, refetch: fetchWorkItems };
}
