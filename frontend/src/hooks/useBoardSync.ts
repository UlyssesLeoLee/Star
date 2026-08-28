// =====================================================================
// useBoardSync — 2s 轮询 boardSync 拉多人协同增量 (per §8.1)
// =====================================================================
// - 复用 TanStack Query 的 useQuery
// - refetchInterval: 2000ms,后台也跑 (refetchIntervalInBackground: true)
// - staleTime: 1000ms 避免抖动
// - onSuccess: 调 useStore.applyRemoteChange 写入本地 (last-write-wins)
// =====================================================================
"use client";

import { useEffect } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { productionApi } from "@/lib/api";
import { useStore } from "@/lib/store";

export interface UseBoardSyncOptions {
  /** 默认 "prj-physis" (mock 主项目) */
  projectId?: string;
  /** 是否启用轮询,默认 true */
  enabled?: boolean;
  /** 自定义 interval,默认 2000ms */
  intervalMs?: number;
}

export const useBoardSync = (opts: UseBoardSyncOptions = {}) => {
  const {
    projectId = "prj-physis",
    enabled = true,
    intervalMs = 2_000,
  } = opts;

  const applyRemoteChange = useStore((s) => s.applyRemoteChange);

  const query = useQuery({
    queryKey: ["board-sync", projectId],
    queryFn: () => productionApi.boardSync(projectId),
    enabled,
    refetchInterval: intervalMs,
    refetchIntervalInBackground: true,
    staleTime: 1_000,
    refetchOnWindowFocus: true,
  });

  // 每次 query data 变化 → 写回 store (last-write-wins)
  useEffect(() => {
    if (query.data) {
      applyRemoteChange({ board: query.data.snapshot.board });
    }
  }, [query.data, applyRemoteChange]);

  return query;
};
