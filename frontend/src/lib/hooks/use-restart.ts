// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/lib/hooks/use-restart.ts
//
// useRestart — per ULYS-214 (A4 完善) 与后端 1e3ac6e8 的两个事件对齐:
//   - `AgentEvent::RestartAvailable { id, terminal_id }` — agent 退出(clean or
//     crash)后, 后端置 `restart_available = true` 并 emit 此事件, UI 据此显示
//     Restart chip (per FR-ORCA-014 "退而不死")。
//   - `AgentEvent::RestartConsumed { id, terminal_id }` — 用户点击 Restart 后,
//     后端重置 `restart_available = false` 并 emit 此事件, 供审计/event-bus 消费。
//
// 当前前端范围(per ULYS-214 父 issue 约束):
//   - 实际 REST/SSE 通道尚未接通(查 `origin/dev` 上 1e3ac6e8 + 13 个 agent 路由
//     只有 start/stop, 没有 restart, 也没有 AgentSession 在 controller.rs 里暴露);
//   - 因此默认走 **TanStack Query 轮询**(3s)作为过渡通道, 注入 `statusFetcher`
//     后可原地换成 SSE/WebSocket;
//   - restart() 走 POST, 契约按 `credentials.ts` `/api/v2` 前缀风格构造;
//     默认实现会 404 (per ULYS-214 §"三处差距" #2 第二项), 同样支持
//     `restartFetcher` 注入换实现。
//
// 主要消费者: `frontend/src/components/agent/SessionCard.tsx`(把 chip 渲染在
// agent session 卡片上, 与 SessionCard prop 配合)。
"use client";

import { useCallback, useEffect, useRef } from "react";
import {
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import type { Uuid } from "@/types/ids";

// ----------------------------------------------------------------------------
// Types
// ----------------------------------------------------------------------------

/**
 * 最小化的 "available" 探测类型, 让 hook 即便在 backend 没接好的时候也能用。
 *
 * 真实后端会推 `AgentEvent::RestartAvailable`, channel 接通后 fetcher 返回
 * `{ restart_available: boolean }` 与 `AgentSession` 字段对齐(本类型只读
 * 本期需要的 chip 开关)。
 */
export interface AgentRestartStatus {
  /** `AgentSession::restart_available`, per FR-ORCA-014. */
  restart_available: boolean;
  /** 后端状态最近变更时间 (ISO 8601); fallback = startup + loop age. */
  updated_at?: string;
}

export interface UseRestartOptions {
  /**
   * 状态 fetcher。默认: `GET /api/v2/agent-sessions/{id}` 返回 `AgentRestartStatus`
   * (本期契约, 后端 404 见 issue §"三处差距" #2 第二项)。
   */
  statusFetcher?: (sessionId: Uuid) => Promise<AgentRestartStatus>;
  /**
   * restart 触发 fetcher。默认: `POST /api/v2/agent-sessions/{id}/restart`,
   * 后端 404 同上。
   */
  restartFetcher?: (sessionId: Uuid) => Promise<AgentRestartStatus>;
  /** 轮询间隔 (ms); 默认 3000。设 0 / 负数 = 关轮询(纯 fetch 一次)。 */
  pollIntervalMs?: number;
  /** 是否启用。默认 true。 */
  enabled?: boolean;
}

export interface UseRestartResult {
  /** 后端现在的 chip 状态 (后端字段映射)。 */
  restartAvailable: boolean;
  /** 最近一次数据拉取时间戳 (Date.now ms), 用于 staleness 展示。 */
  lastCheckedAt: number | null;
  /** 是否初次加载中。 */
  isLoading: boolean;
  /** 启动 restart 请求, 触发 mutation (POST)。 */
  restart: () => void;
  /** mutation 状态。 */
  isRestarting: boolean;
  /** 最近一次 mutation error (如有)。 */
  error: Error | null;
  /** 手动 revalidate (同 invalidateQueries). */
  refetch: () => void;
}

// ----------------------------------------------------------------------------
// Default fetchers
// ----------------------------------------------------------------------------

async function defaultStatusFetcher(sessionId: Uuid): Promise<AgentRestartStatus> {
  const res = await fetch(`/api/v2/agent-sessions/${sessionId}`, {
    method: "GET",
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  if (!res.ok) {
    throw new Error(
      `[useRestart] status ${res.status} ${res.statusText} for session ${sessionId}`,
    );
  }
  const body = (await res.json()) as Partial<AgentRestartStatus>;
  return {
    restart_available: Boolean(body.restart_available),
    updated_at: body.updated_at,
  };
}

async function defaultRestartFetcher(
  sessionId: Uuid,
): Promise<AgentRestartStatus> {
  const res = await fetch(`/api/v2/agent-sessions/${sessionId}/restart`, {
    method: "POST",
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  if (!res.ok) {
    throw new Error(
      `[useRestart] restart ${res.status} ${res.statusText} for session ${sessionId}`,
    );
  }
  const body = (await res.json()) as Partial<AgentRestartStatus>;
  return {
    restart_available: Boolean(body.restart_available),
    updated_at: body.updated_at,
  };
}

// ----------------------------------------------------------------------------
// Hook
// ----------------------------------------------------------------------------

const DEFAULT_POLL_MS = 3000;

/**
 * useRestart(sessionId, opts) — 订阅 `AgentEvent::RestartAvailable` (轮询
 * 过渡实现) 并暴露 `restart()` 触发 `POST .../restart`(后端将重置
 * `restart_available = false` 并 emit `RestartConsumed`)。
 *
 * 注意 (per ULYS-214 design rationale):
 *   - 轮询不是最终通道; SSE / WS 接入后请注入 `statusFetcher` 走 push。
 *   - 默认 fetch 路径是契约预测, 后端 controller.rs(13 路由, 仅 start/stop)
 *     还没有对应路由, 调用会 404, 这个状态被 chip 容忍(渲染仅依赖 boolean;
 *     fetch 失败时 chip 不显示而非抛错)。
 *   - 返回 `restartAvailable: false` 是安全默认(没拉到数据前), 不会误触发
 *     "可重启" 视觉。
 */
export function useRestart(
  sessionId: Uuid,
  opts: UseRestartOptions = {},
): UseRestartResult {
  const {
    statusFetcher = defaultStatusFetcher,
    restartFetcher = defaultRestartFetcher,
    pollIntervalMs = DEFAULT_POLL_MS,
    enabled = true,
  } = opts;

  const queryClient = useQueryClient();
  const queryKey = ["agent-session", sessionId, "restart-status"] as const;

  const query = useQuery<AgentRestartStatus>({
    queryKey,
    queryFn: () => statusFetcher(sessionId),
    enabled: enabled && !!sessionId,
    refetchInterval:
      enabled && pollIntervalMs > 0 ? pollIntervalMs : false,
    // 路径不稳时保持 last data, 不要闪 — Restart chip 闪烁对 UX 不利
    placeholderData: (previousData) => previousData,
    retry: 1,
  });

  const mutation = useMutation<AgentRestartStatus, Error, void>({
    mutationFn: () => restartFetcher(sessionId),
    onSuccess: (data) => {
      // 写入 query cache, 立刻反映新状态(restart_available = false 等)。
      // 注意: 这里不用 `invalidateQueries` 触发 refetch — 因为默认 fetcher
      // (`statusFetcher`) 仍是同一 GET, 而 mutation 的服务端响应已包含最新
      // 状态。invalidate 会发起一轮额外 GET, 而且默认 fetcher 仍会得到旧值
      // (后端写完 reset 的 events 是异步的), 反而冲掉我们刚 setQueryData 进
      // 的新数据, 导致 chip 闪回到 available.
      queryClient.setQueryData(queryKey, data);
    },
  });

  // 缓存最新的 lastCheckedAt, 保证组件每次 render 拿到 monotonic 时间
  const lastCheckedAtRef = useRef<number | null>(null);
  useEffect(() => {
    if (query.dataUpdatedAt > 0) {
      lastCheckedAtRef.current = query.dataUpdatedAt;
    }
  }, [query.dataUpdatedAt]);

  const restart = useCallback(() => {
    if (!sessionId || mutation.isPending) return;
    mutation.mutate();
  }, [sessionId, mutation]);

  const refetch = useCallback(() => {
    queryClient.invalidateQueries({ queryKey });
  }, [queryClient, queryKey]);

  // 渲染: 没拉到数据时 = false (保守), 避免 "可重启" 假阳
  const restartAvailable = Boolean(query.data?.restart_available);

  return {
    restartAvailable,
    lastCheckedAt: lastCheckedAtRef.current,
    isLoading: query.isLoading,
    restart,
    isRestarting: mutation.isPending,
    // mutation error 优先 (用户主动触发的), 没触发就回退到 query error
    // (passive 探测的) — 这是触达用户最多的语义层
    error: (mutation.error as Error | null) ?? (query.error as Error | null) ?? null,
    refetch,
  };
}
