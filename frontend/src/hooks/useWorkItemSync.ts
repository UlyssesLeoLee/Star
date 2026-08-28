// =====================================================================
// useWorkItemSync — 2s 轮询 workItemSync (per §8.1)
// =====================================================================
// - 同样模式: TanStack Query + 写回 store
// - 走 productionApi.workItemSync (mock)
// =====================================================================
"use client";

import { useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { productionApi } from "@/lib/api";
import { useStore } from "@/lib/store";

export interface UseWorkItemSyncOptions {
  projectId?: string;
  enabled?: boolean;
  intervalMs?: number;
}

export const useWorkItemSync = (opts: UseWorkItemSyncOptions = {}) => {
  const {
    projectId = "prj-physis",
    enabled = true,
    intervalMs = 2_000,
  } = opts;

  const applyRemoteChange = useStore((s) => s.applyRemoteChange);

  const query = useQuery({
    queryKey: ["work-item-sync", projectId],
    queryFn: () => productionApi.workItemSync(projectId),
    enabled,
    refetchInterval: intervalMs,
    refetchIntervalInBackground: true,
    staleTime: 1_000,
    refetchOnWindowFocus: true,
  });

  useEffect(() => {
    if (query.data) {
      applyRemoteChange({ workItems: query.data.items });
    }
  }, [query.data, applyRemoteChange]);

  return query;
};
