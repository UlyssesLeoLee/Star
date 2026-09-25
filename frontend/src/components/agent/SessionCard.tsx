// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/agent/SessionCard.tsx
//
// SessionCard — Agent Session 卡片 (per ULYS-214 A4 完善)。
//
// 设计:
//   - 全新组件: 仓库里此前没有任何同名/同职责组件(最接近的
//     `worktree-canvas/WorktreeCard.tsx` 是 worktree 健康度卡片, 不含 agent
//     session 状态字段)。参考了它「卡片 div + 内部小 badge / chip 子组件」结构。
//   - 集成 `RestartChip`: 当 `session.restart_available === true` 或后端推
//     `AgentEvent::RestartAvailable` 时显示。
//   - **不**复用 `useBoardSync` 这类 SSE 通道: 本期轮询路径自己随 hook 管理,
//     与项目其它 store 解耦 (per ULYS-214 父 issue 约束 "纯前端, 不动 crate")。
//
// 不变量 (守门):
//   - 输入 `AgentSession | undefined` 容错 — 父组件没拿到数据时显示 skeleton
//     而非崩。
//   - `terminal_id` 仅为字段镜像 (本期不渲染到 UI 但保留在下一次返回的 DTO 上
//     以便下游切到 SSE 后直接消费)。
//   - i18n 不强依赖 — 与已有 `WorktreeCard` 一样使用中文短语 + 英文混合, 待
//     i18n 组件提取工具复用一轮 (scope 外) 后再提案 keys。
"use client";

import { memo } from "react";
import { RestartChip } from "./RestartChip";
import type { AgentSession } from "@/types/ids";

export interface SessionCardProps {
  /**
   * Session 数据. undefined = 还在加载 / 拉取失败 → 显示 skeleton 占位。
   */
  session?: AgentSession;
  /** 是否选中(影响边框色), 默认 false。 */
  selected?: boolean;
  /** 卡片整体 onClick, 父组件用于打开 drawer。 */
  onClick?: () => void;
}

/**
 * 把 AgentStatus 14 状态机(\`AgentStatus\`)映射到 chip-friendly 的短标签 + 色系.
 *
 * 13/14 状态用前景 + 背景色两个 token 区分; 用 color 越多越难看清, 这里用
 * 6 个"色族"分类即可(active/feedback/tool/validation/done/error)。
 */
function statusToneClass(status: AgentSession["status"]): string {
  // 仅前台 tab 上 chip 提示色(轻量)
  switch (status) {
    case "executing":
      return "border-blue-300 bg-blue-50 text-blue-700";
    case "compiling_context":
    case "planning":
    case "spawning":
    case "queued":
    case "initializing":
      return "border-slate-300 bg-slate-50 text-slate-700";
    case "awaiting_feedback":
    case "awaiting_human":
      return "border-amber-300 bg-amber-50 text-amber-800";
    case "awaiting_tool":
    case "validating":
      return "border-violet-300 bg-violet-50 text-violet-700";
    case "paused":
      return "border-yellow-300 bg-yellow-50 text-yellow-800";
    case "completed":
      return "border-green-300 bg-green-50 text-green-700";
    case "failed":
      return "border-red-300 bg-red-50 text-red-700";
    case "cancelled":
      return "border-slate-300 bg-slate-100 text-slate-500";
  }
}

function formatDuration(started_at: string, ended_at?: string): string {
  const start = Date.parse(started_at);
  const end = ended_at ? Date.parse(ended_at) : Date.now();
  if (Number.isNaN(start) || Number.isNaN(end)) return "-";
  const sec = Math.max(0, Math.round((end - start) / 1000));
  if (sec < 60) return `${sec}s`;
  if (sec < 3600) return `${Math.round(sec / 60)}m`;
  return `${Math.round(sec / 3600)}h`;
}

function Skeleton() {
  return (
    <div
      data-testid="session-card-skeleton"
      className="rounded border border-slate-200 bg-white p-2 text-xs shadow-sm dark:border-slate-700 dark:bg-slate-900"
    >
      <div className="mb-1 flex items-center justify-between">
        <span className="block h-3 w-24 rounded bg-slate-200 dark:bg-slate-700" />
        <span className="block h-3 w-12 rounded bg-slate-200 dark:bg-slate-700" />
      </div>
      <div className="mb-1 h-3 w-40 rounded bg-slate-200 dark:bg-slate-700" />
      <div className="flex items-center justify-between">
        <span className="block h-3 w-10 rounded bg-slate-200 dark:bg-slate-700" />
        <span className="block h-4 w-12 rounded-full bg-slate-200 dark:bg-slate-700" />
      </div>
    </div>
  );
}

function Inner({ session, selected = false, onClick }: SessionCardProps) {
  if (!session) return <Skeleton />;

  const cardClass = `rounded border bg-white p-2 text-xs shadow-sm transition dark:bg-slate-900 ${
    selected
      ? "border-blue-500 ring-2 ring-blue-200"
      : "border-slate-200 hover:border-slate-300 dark:border-slate-700"
  }`;

  const statusClass =
    "inline-flex items-center rounded-full border px-2 py-0.5 text-[10px] font-medium " +
    statusToneClass(session.status);

  // chip 区: 显示 "Restart chip" 仅当字段为 true (RestartChip 自己也门控,
  // 这里 explicit true 走"显式 override"分支, 不重复内部 hook; 默认
  // session.restart_available === undefined 时由 RestartChip 自己拉)
  const showRestart = session.restart_available === true;

  return (
    <div
      data-testid={`session-card-${session.id}`}
      data-selected={selected ? "true" : "false"}
      onClick={onClick}
      className={cardClass}
    >
      <div className="mb-1 flex items-center justify-between gap-2">
        <span
          className="truncate font-medium"
          title={session.name}
          data-testid="session-name"
        >
          {session.name}
        </span>
        <span data-testid="session-status" className={statusClass}>
          {session.status.replace(/_/g, " ")}
        </span>
      </div>

      <div
        className="mb-1 truncate font-mono text-slate-500"
        title={`worktree ${session.worktree_id}`}
      >
        {session.worktree_id}
      </div>

      <div className="flex items-center justify-between gap-2 text-slate-400">
        <span data-testid="session-duration">
          {formatDuration(session.started_at, session.ended_at)}
        </span>
        <span className="font-mono text-[10px]" data-testid="session-kind">
          {session.agent_kind}
        </span>
        {/* chip: 注入"显式 override"分支, 不调用内部 hook;
            session.restart_available === false 走显式 false 不渲染分支,
            session.restart_available === undefined 时 RestartChip 会自己 hook 一下 */}
        <RestartChip
          sessionId={session.id}
          available={showRestart ? true : false}
        />
      </div>
    </div>
  );
}

export const SessionCard = memo(Inner);
export default SessionCard;
