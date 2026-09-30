// crates/star-desktop/frontend/src/hooks/useTauriBoardInfo.ts
// =====================================================================
// Star Desktop — Tauri IPC React hook (PR-250)
// 调用 Tauri IPC `get_board_info` (per PR #241 + PR #245)
// 返回 BoardInfo { board_kind_count, swimlane_group_by_count, default_column_count }
// =====================================================================

import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface BoardInfo {
  board_kind_count: number;
  swimlane_group_by_count: number;
  default_column_count: number;
}

interface UseTauriBoardInfoResult {
  boardInfo: BoardInfo | null;
  loading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

const MOCK_BOARD_INFO: BoardInfo = {
  board_kind_count: 3,        // Kanban / Scrum / BugTracking
  swimlane_group_by_count: 4, // Assignee / Priority / Epic / None
  default_column_count: 6,    // todo / in_progress / review / blocked / done / wontfix
};

export function useTauriBoardInfo(): UseTauriBoardInfoResult {
  const [boardInfo, setBoardInfo] = useState<BoardInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchBoardInfo = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const info = await invoke<BoardInfo>("get_board_info");
      setBoardInfo(info);
    } catch (e) {
      if (
        typeof window !== "undefined" &&
        // @ts-ignore
        !window.__TAURI_INTERNALS__
      ) {
        setBoardInfo(MOCK_BOARD_INFO);
      } else {
        setError(e instanceof Error ? e : new Error(String(e)));
      }
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchBoardInfo();
  }, [fetchBoardInfo]);

  return { boardInfo, loading, error, refetch: fetchBoardInfo };
}
