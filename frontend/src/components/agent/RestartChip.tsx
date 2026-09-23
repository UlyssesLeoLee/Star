// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/agent/RestartChip.tsx
//
// RestartChip — agent 退出(clean or crash)后, 当后端字段
// `AgentSession::restart_available = true` 或推 `AgentEvent::RestartAvailable`
// 时, UI 显示 "Restart" chip; 点击触发 `useRestart().restart()` (per FR-ORCA-014,
// ULYS-197 v1 → v2 + ULYS-214 A4 完善)。
//
// 设计要点 (per project conventions 守门 #5 secret safety + 现有
// `worktree-canvas/HealthBadge.tsx` 的 chip 风格):
//   - **条件渲染**: `available !== true` 时返回 null(不渲染), 避免无意义
//     chip 残留, 与 `WorktreeCard` 的 status badge 不同的"有则示之"模型。
//   - **loading / error 视觉**: 点 Restart 后 isRestarting 显示 spinner; 失败
//     沿用 chip 样式 + aria-live, 不弹 modal (低干扰符合"恢复"语义)。
//   - **可注入**: `useRestart` 默认走轮询 + 404 fetch, 真实通道到位时
//     SessionCard 可注入替代 fetcher, 本组件无需改。
//   - **a11y**: button 节点 + aria-disabled 而不是真 disabled, 这样 screen
//     reader 仍能听到 "Restart agent session" label 在 isRestarting 时。
"use client";

import { memo } from "react";
import { useRestart, type UseRestartResult } from "@/lib/hooks/use-restart";
import type { Uuid } from "@/types/ids";

interface Props {
  /** Agent session UUID. */
  sessionId: Uuid;
  /** 自定义 label (默认 i18n-friendly 英文文案, 等 i18n 上游拉齐后建议提出 key). */
  label?: string;
  /**
   * 显式 override "是否可重启" — 允许 SessionCard 在没有任何 fetcher 数据时,
   * 仅凭 `AgentSession.restart_available` 也渲染 chip。
   *
   * 如果提供 (boolean), 则本组件不再调用 useRestart() — 由父组件负责
   * 拉数据 + 触发 restart (走 `useRestartResult` 注入式)。
   */
  available?: boolean;
  /**
   * 同上: 注入式使用, 由父组件传入完整的 useRestart 结果。
   * 与 `available` / `sessionId` 互斥:
   *   - 传 `result` → 完全用注入, 不调 hook
   *   - 传 `available=false` → 显式不渲染
   *   - 否则 → 调 hook 取数据
   */
  result?: UseRestartResult;
  /**
   * 自定义 class, 给 SessionCard 内联布局调整 (默认: chip 风格)。
   */
  className?: string;
}

function Inner({
  sessionId,
  label = "Restart agent session",
  available: availableOverride,
  result,
  className,
}: Props) {
  // (1) 显式 override: 父组件说"不渲染"
  if (availableOverride === false) return null;

  // (2) 注入式: 父组件完全控制
  if (result) {
    if (!result.restartAvailable) return null;
    return <ChipButton {...chipButtonProps(result, label, className)} />;
  }

  // (3) 自管: 调 hook
  const live = useRestart(sessionId);
  if (availableOverride !== true && !live.restartAvailable) return null;

  return <ChipButton {...chipButtonProps(live, label, className)} />;
}

function chipButtonProps(
  r: UseRestartResult,
  label: string,
  className?: string,
) {
  return {
    label,
    isRestarting: r.isRestarting,
    error: r.error,
    onClick: () => {
      if (!r.isRestarting) r.restart();
    },
    className,
  };
}

function ChipButton({
  label,
  isRestarting,
  error,
  onClick,
  className,
}: {
  label: string;
  isRestarting: boolean;
  error: Error | null;
  onClick: () => void;
  className?: string;
}) {
  const baseClass =
    "inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-[11px] font-medium transition focus:outline-none focus:ring-2 focus:ring-blue-300";
  const toneClass = error
    ? "border-red-300 bg-red-50 text-red-700 hover:bg-red-100"
    : "border-amber-300 bg-amber-50 text-amber-800 hover:bg-amber-100";

  return (
    <button
      type="button"
      data-testid="restart-chip"
      data-state={isRestarting ? "restarting" : error ? "error" : "ready"}
      aria-busy={isRestarting}
      aria-disabled={isRestarting}
      aria-label={error ? `Restart failed: ${error.message}. ${label}` : label}
      title={error ? error.message : label}
      onClick={onClick}
      className={[baseClass, toneClass, className].filter(Boolean).join(" ")}
    >
      {isRestarting ? (
        <Spinner />
      ) : (
        <span aria-hidden="true" className="text-[10px]">
          ↻
        </span>
      )}
      <span>{error ? "Retry restart" : "Restart"}</span>
    </button>
  );
}

function Spinner() {
  // CSS-only 8px spinner, 不依赖额外包; aria-hidden 让 screen reader 忽略
  return (
    <span
      aria-hidden="true"
      className="inline-block h-2.5 w-2.5 animate-spin rounded-full border-2 border-amber-300 border-t-amber-700"
    />
  );
}

export const RestartChip = memo(Inner);
export default RestartChip;
