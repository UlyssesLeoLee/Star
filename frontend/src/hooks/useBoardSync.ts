"use client";

// =====================================================================
// useBoardSync — TanStack-Query 风格的 2s 轮询多人协同 hook
// =====================================================================
// 职责:
//   1. 用 TanStack Query 每 2s 从 productionApi.boardSync(projectId) 拉增量
//   2. 检测 workItems[i].updated_at > since 的项 → 标记为"他人改动"
//   3. 调用 onRemoteChange(changes) 回调, page.tsx 用它来 toast
//   4. 暴露 { data, fetchStatus, refetch, hasRemoteChange, changeCount, lastSyncAt, isPolling }
//
// 设计取舍 (per docs/frontend/design/dynamic-interaction-design.md §8.1):
//   - W5 重构: 用 @tanstack/react-query useQuery + refetchInterval (而非手写 setInterval)
//   - last-write-wins 冲突解决 (per §8.1), 不引入 CRDT
//   - 暂不接 WebSocket (per §2.2 + §10.3 已知缺口 #1)
//
// 已知缺口 (per §10.3 缺标比错标):
//   - productionApi.boardSync 走 zustand store 拿全量, since cursor 忽略 (mock 阶段)
//   - Phase I+ 切换为 SSE 推送
// =====================================================================

import { useEffect, useRef, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { productionApi } from "@/lib/api";
import type { WorkItem } from "@/types/ids";

export interface BoardSyncChange {
  work_item_id: string;
  from_status: WorkItem["status"];
  to_status: WorkItem["status"];
  changed_by: string; // user_id of who made the change
  changed_at: string; // ISO8601
}

export interface BoardSyncSnapshot {
  since: string;                    // cursor (ISO8601)
  changes: BoardSyncChange[];       // 增量变更列表
  server_time: string;              // 服务器当前时间, 客户端下次轮询用
}

export interface UseBoardSyncOptions {
  /** 项目 id, queryKey 必含 (per §8.1) */
  projectId: string;
  /** 轮询间隔 ms, 默认 2000 (per §2.2 + §8.1) */
  intervalMs?: number;
  /** 启用 / 停用轮询, 默认 true */
  enabled?: boolean;
  /** 收到变更时的回调, page.tsx 用它来 toast */
  onRemoteChange?: (changes: BoardSyncChange[]) => void;
  /** 拉取函数; 不传则用 productionApi.boardSync */
  fetcher?: (projectId: string, since?: string) => Promise<{ cursor: string; snapshot: { board: unknown; recentActivity: unknown[] } }>;
}

export interface UseBoardSyncResult {
  hasRemoteChange: boolean;
  changeCount: number;
  lastSyncAt: string | null;
  isPolling: boolean;
  /** 最近一次 fetcher 返回的原始 response (测试断言用) */
  data: { cursor: string; snapshot: { board: unknown; recentActivity: unknown[] } } | undefined;
  /** TanStack-Query 风格: "idle" | "fetching" (测试断言用) */
  fetchStatus: "idle" | "fetching";
  /** 强制立即拉一次 */
  refetch: () => void;
}

export function useBoardSync(opts: UseBoardSyncOptions): UseBoardSyncResult {
  const { projectId, intervalMs = 2000, enabled = true, onRemoteChange, fetcher } = opts;
  const [changeCount, setChangeCount] = useState(0);
  const [lastSyncAt, setLastSyncAt] = useState<string | null>(null);
  const onRemoteChangeRef = useRef(onRemoteChange);
  const queryClient = useQueryClient();

  useEffect(() => {
    onRemoteChangeRef.current = onRemoteChange;
  }, [onRemoteChange]);

  const queryKey = ["board-sync", projectId];
  const query = useQuery({
    queryKey,
    queryFn: async () => {
      const fn = fetcher ?? productionApi.boardSync;
      return fn(projectId);
    },
    enabled,
    refetchInterval: enabled ? intervalMs : false,
    refetchOnMount: enabled,
    refetchOnWindowFocus: false,
  });

  // 每次 data 变化: 更新 lastSyncAt + 触发 onRemoteChange
  useEffect(() => {
    if (!query.data) return;
    setLastSyncAt(query.data.cursor);
    const recent = (query.data.snapshot?.recentActivity ?? []) as Array<{ work_item_id: string; to_status: WorkItem["status"]; actor_id: string; at: string }>;
    if (recent.length > 0) {
      const changes: BoardSyncChange[] = recent.map((r) => ({
        work_item_id: r.work_item_id,
        from_status: "todo",
        to_status: r.to_status,
        changed_by: r.actor_id,
        changed_at: r.at,
      }));
      setChangeCount((c) => c + changes.length);
      onRemoteChangeRef.current?.(changes);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [query.data]);

  return {
    hasRemoteChange: changeCount > 0,
    changeCount,
    lastSyncAt,
    isPolling: query.isFetching,
    data: query.data,
    fetchStatus: query.fetchStatus as "idle" | "fetching",
    refetch: () => { void queryClient.invalidateQueries({ queryKey }); },
  };
}
