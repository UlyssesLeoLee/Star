// crates/star-desktop/frontend/src/hooks/useTauriKeyboardLayout.ts
// =====================================================================
// Star Desktop — Tauri IPC React hook (PR-250)
// 调用 Tauri IPC `get_keyboard_layout` (per PR #240 + PR #245)
// 返回 KeyboardLayout { swimlanes: W/T/M, statuses: 6 }
// =====================================================================

import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Wtm, WorkItemStatus } from "../types/ids";
import { DEFAULT_SWIMLANES, ALL_WORK_ITEM_STATUSES } from "../constants";

export interface KeyboardLayout {
  swimlanes: Wtm[];
  statuses: WorkItemStatus[];
}

interface UseTauriKeyboardLayoutResult {
  layout: KeyboardLayout | null;
  loading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

const MOCK_LAYOUT: KeyboardLayout = {
  swimlanes: DEFAULT_SWIMLANES,
  statuses: ALL_WORK_ITEM_STATUSES,
};

export function useTauriKeyboardLayout(): UseTauriKeyboardLayoutResult {
  const [layout, setLayout] = useState<KeyboardLayout | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchLayout = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const l = await invoke<KeyboardLayout>("get_keyboard_layout");
      setLayout(l);
    } catch (e) {
      if (
        typeof window !== "undefined" &&
        // @ts-ignore
        !window.__TAURI_INTERNALS__
      ) {
        setLayout(MOCK_LAYOUT);
      } else {
        setError(e instanceof Error ? e : new Error(String(e)));
      }
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchLayout();
  }, [fetchLayout]);

  return { layout, loading, error, refetch: fetchLayout };
}
