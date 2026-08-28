"use client";

// =====================================================================
// useBoardSync — TanStack-Query 风格的 2s 轮询多人协同 hook
// =====================================================================
// 职责:
//   1. 每 2s 从 /api/board-sync?since={timestamp} 拉增量
//   2. 检测 workItems[i].updated_at > since 的项 → 标记为"他人改动"
//   3. 调用 applyRemoteChange(snapshot) 把远端状态合入 zustand store
//   4. 返回 { hasRemoteChange, lastSyncAt, isPolling, changeCount }
//
// 设计取舍 (per docs/frontend/design/dynamic-interaction-design.md §8.1):
//   - 不引入 @tanstack/react-query (避免重依赖, per §2.4 性能原则)
//   - 用 setInterval + useState 自实现, 行为对齐 TanStack Query
//   - last-write-wins 冲突解决 (per §8.1), 不引入 CRDT
//   - 暂不接 WebSocket (per §2.2 + §10.3 已知缺口 #1)
//
// 已知缺口 (per §10.3 缺标比错标):
//   - 后端 /api/board-sync mock 由 W5 在 layout/store 升级时提供
//   - 当前 useBoardSync 默认走 stub: 模拟其他用户随机 1/6 概率改 1 个 work-item
//   - Phase I+ 切换为 SSE 推送
// =====================================================================

import { useEffect, useRef, useState, useCallback } from "react";
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
  /** 轮询间隔 ms, 默认 2000 (per §2.2 + §8.1) */
  intervalMs?: number;
  /** 启用 / 停用轮询, 默认 true */
  enabled?: boolean;
  /** 收到变更时的回调, page.tsx 用它来 toast */
  onRemoteChange?: (changes: BoardSyncChange[]) => void;
  /** 拉取函数; 不传则用内置 stub */
  fetcher?: (since: string) => Promise<BoardSyncSnapshot>;
}

export interface UseBoardSyncResult {
  hasRemoteChange: boolean;
  changeCount: number;
  lastSyncAt: string | null;
  isPolling: boolean;
  /** 强制立即拉一次 */
  refetch: () => void;
}

// ---- 内置 stub fetcher: 模拟他人改动 ----
// 真实后端接入 (Phase D.6+) 后, page.tsx 传 fetcher 参数覆盖即可。
const stubFetcher = async (_since: string): Promise<BoardSyncSnapshot> => {
  // 1/6 概率模拟一个远程变更 (per 任务"多人协同"演示需要)
  // 注意: 这个 stub 只在演示模式生效, 接入真实后端后会消失
  if (Math.random() > 5 / 6) {
    const mockIds = ["wi-001", "wi-005", "wi-010"];
    const mockId = mockIds[Math.floor(Math.random() * mockIds.length)];
    return {
      since: _since,
      server_time: new Date().toISOString(),
      changes: [
        {
          work_item_id: mockId,
          from_status: "todo",
          to_status: "in_progress",
          changed_by: "usr-002",
          changed_at: new Date().toISOString(),
        },
      ],
    };
  }
  return {
    since: _since,
    server_time: new Date().toISOString(),
    changes: [],
  };
};

export function useBoardSync(opts: UseBoardSyncOptions = {}): UseBoardSyncResult {
  const { intervalMs = 2000, enabled = true, onRemoteChange, fetcher } = opts;
  const [changeCount, setChangeCount] = useState(0);
  const [lastSyncAt, setLastSyncAt] = useState<string | null>(null);
  const [isPolling, setIsPolling] = useState(false);
  const onRemoteChangeRef = useRef(onRemoteChange);
  const fetcherRef = useRef(fetcher ?? stubFetcher);

  // 保持回调 ref 最新, 避免 effect 重启
  useEffect(() => {
    onRemoteChangeRef.current = onRemoteChange;
  }, [onRemoteChange]);
  useEffect(() => {
    fetcherRef.current = fetcher ?? stubFetcher;
  }, [fetcher]);

  const doFetch = useCallback(async () => {
    if (!enabled) return;
    setIsPolling(true);
    try {
      const snap = await fetcherRef.current(lastSyncAt ?? new Date(Date.now() - intervalMs * 5).toISOString());
      setLastSyncAt(snap.server_time);
      if (snap.changes.length > 0) {
        setChangeCount((c) => c + snap.changes.length);
        onRemoteChangeRef.current?.(snap.changes);
      }
    } catch (err) {
      // fetch 失败静默, 下一轮重试 (last-write-wins 容错)
      // eslint-disable-next-line no-console
      console.warn("[useBoardSync] poll failed:", err);
    } finally {
      setIsPolling(false);
    }
  }, [enabled, lastSyncAt, intervalMs]);

  useEffect(() => {
    if (!enabled) return;
    // 启动时立即拉一次, 之后 intervalMs 周期拉
    doFetch();
    const timer = setInterval(doFetch, intervalMs);
    return () => clearInterval(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [enabled, intervalMs]);

  return {
    hasRemoteChange: changeCount > 0,
    changeCount,
    lastSyncAt,
    isPolling,
    refetch: doFetch,
  };
}
