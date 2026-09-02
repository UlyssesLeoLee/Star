"use client";

// =====================================================================
// RefactorCard — 单张重构卡 (per docs/frontend/design/refactor-sweep-design.md)
// =====================================================================
// 跟 KanbanCard 形态相近, 但:
//   - 渲染 RefactorCard (含 work_item_key/title snapshot, 不依赖 workItems 实时)
//   - 右上角 "Round N" 徽章 — 第 N 次重构计数
//   - history 末次状态时间, 拖动中 opacity 0.5
//   - 颜色按 refactor_status 走 (跟 workItem StatusPill 颜色对齐)
//   - **done 列时显示 [Merge] 按钮** (per 2026-09-02 10:50 JST 拍板)
//   - 已 merged 显示绿色 [Merged · HH:MM] 徽章
// =====================================================================

import { useState } from "react";
import { clsx } from "clsx";
import { useRouter } from "next/navigation";
import { useTranslation, useStatusLabel, interpolate } from "@/lib/i18n";
import type { RefactorCard as RefactorCardData } from "@/types/ids";
import { Hash, History as HistoryIcon, GripVertical, GitMerge, Check, Loader2 } from "lucide-react";

export interface RefactorCardProps {
  card: RefactorCardData;
  isDragging?: boolean;
  onDragStart?: (e: React.DragEvent<HTMLDivElement>, card: RefactorCardData) => void;
  onDragEnd?: (e: React.DragEvent<HTMLDivElement>) => void;
  onClick?: (card: RefactorCardData) => void;
  /** done 列 + 未 merged 时显示 Merge 按钮 (per 2026-09-02 10:50 JST 拍板) */
  onMerge?: (card: RefactorCardData) => void | Promise<void>;
  /** 当前 workItem 是否有关联 worktree (用于按钮 title hint) */
  hasWorktree?: boolean;
  /** round 是否已关闭 (closed round 不允许 merge) */
  readOnly?: boolean;
}

export function RefactorCard({
  card, isDragging, onDragStart, onDragEnd, onClick, onMerge, hasWorktree, readOnly,
}: RefactorCardProps) {
  const router = useRouter();
  const { t } = useTranslation();
  const statusLabel = useStatusLabel("refactor", card.refactor_status);
  const [merging, setMerging] = useState(false);

  const isDone = card.refactor_status === "done";
  const isMerged = Boolean(card.merged_at);

  const handleDragStart = (e: React.DragEvent<HTMLDivElement>) => {
    e.dataTransfer.setData("text/refactor-card-id", card.work_item_id);
    e.dataTransfer.effectAllowed = "move";
    onDragStart?.(e, card);
  };
  const handleClick = () => {
    if (onClick) onClick(card);
    else router.push(`/work-item/${card.work_item_id}`);
  };
  // moved_at 取 ISO 时间短码 (HH:MM) 即可, UI 紧凑
  const movedShort = (() => {
    try {
      const d = new Date(card.moved_at);
      return `${d.getHours().toString().padStart(2, "0")}:${d.getMinutes().toString().padStart(2, "0")}`;
    } catch { return ""; }
  })();
  const mergedShort = (() => {
    if (!card.merged_at) return "";
    try {
      const d = new Date(card.merged_at);
      return `${d.getHours().toString().padStart(2, "0")}:${d.getMinutes().toString().padStart(2, "0")}`;
    } catch { return ""; }
  })();

  const handleMerge = async (e: React.MouseEvent) => {
    e.stopPropagation();
    e.preventDefault();
    if (!onMerge || merging || isMerged || readOnly) return;
    if (typeof window !== "undefined" && hasWorktree) {
      if (!window.confirm(t.refactor.mergeConfirm)) return;
    }
    setMerging(true);
    try {
      await Promise.resolve(onMerge(card));
    } finally {
      // 即使父组件同步完成也保留短暂 loading 闪烁, 给视觉反馈
      setTimeout(() => setMerging(false), 250);
    }
  };

  return (
    <div
      role="article"
      data-testid={`refactor-card-${card.work_item_id}`}
      data-card-id={card.work_item_id}
      data-merged={isMerged ? "true" : "false"}
      draggable
      onDragStart={handleDragStart}
      onDragEnd={onDragEnd}
      onClick={handleClick}
      className={clsx(
        "relative p-2.5 rounded border bg-bg-soft/70 cursor-pointer select-none",
        "hover:bg-bg-soft hover:border-accent/40 hover:-translate-y-0.5 transition-all duration-150",
        "shadow-sm hover:shadow-[0_4px_12px_rgba(0,0,0,0.3)]",
        isDone && !isMerged && "border-ok/30",
        isMerged && "border-ok/60 bg-ok/5 shadow-[0_0_10px_rgba(16,185,129,0.18)]",
        isDragging && "opacity-50 ring-2 ring-accent shadow-[0_0_15px_rgba(0,240,255,0.4)]",
      )}
    >
      {/* 顶部: round 徽章 + 拖动手柄 */}
      <div className="flex items-center justify-between mb-1.5">
        <span
          className="font-mono text-[9px] px-1.5 py-0.5 rounded border border-warn/40 bg-warn/10 text-warn font-bold"
          title={interpolate(t.refactor.refactorRoundBadge, { n: card.round_number })}
        >
          <Hash size={8} className="inline -mt-0.5 mr-0.5" />
          {card.round_number}
        </span>
        <GripVertical size={11} className="text-ink-mute opacity-40" />
      </div>

      {/* key */}
      <div className="font-mono text-[10px] text-info font-medium tracking-tight mb-1">
        <span className="text-[9px] text-ink-mute">// </span>
        {card.work_item_key}
      </div>

      {/* title */}
      <div className="text-xs font-medium text-ink line-clamp-2 leading-snug mb-2">
        {card.work_item_title}
      </div>

      {/* 底部: 当前 refactor 状态 + moved_at */}
      <div className="flex items-center justify-between text-[9px] font-mono text-ink-mute pt-1.5 border-t border-line/40">
        <span className={clsx(
          "px-1.5 py-0.5 rounded border",
          isMerged
            ? "border-ok/40 bg-ok/15 text-ok"
            : "border-accent/40 bg-accent/10 text-accent",
        )}>
          {isMerged ? t.refactor.merged : statusLabel}
        </span>
        {movedShort && !isMerged && (
          <span className="flex items-center gap-1" title={interpolate(t.refactor.refactorMovedAt, { time: card.moved_at })}>
            <HistoryIcon size={8} />
            {movedShort}
          </span>
        )}
        {isMerged && mergedShort && (
          <span className="flex items-center gap-1 text-ok" title={interpolate(t.refactor.mergedAt, { time: card.merged_at ?? "" })}>
            <Check size={8} />
            {mergedShort}
          </span>
        )}
      </div>

      {/* Merge 按钮 (per 2026-09-02 10:50 JST 拍板)
          - 仅 done 列 + 未 merged 时显示
          - round closed (readOnly) 时不显示
          - 合并中显示 spinner
       */}
      {isDone && !isMerged && !readOnly && onMerge && (
        <button
          type="button"
          onClick={handleMerge}
          disabled={merging}
          data-testid={`refactor-merge-${card.work_item_id}`}
          title={hasWorktree ? t.refactor.mergeTitle : t.refactor.mergeNoWorktree}
          className={clsx(
            "mt-2 w-full flex items-center justify-center gap-1.5 py-1 rounded text-[10px] font-mono font-bold",
            "border transition-colors",
            hasWorktree
              ? "border-ok/50 bg-ok/10 text-ok hover:bg-ok/20 hover:border-ok shadow-[0_0_6px_rgba(16,185,129,0.2)]"
              : "border-info/40 bg-info/5 text-info hover:bg-info/10",
            merging && "opacity-60 cursor-wait",
          )}
        >
          {merging
            ? <><Loader2 size={10} className="animate-spin" /> {t.refactor.mergeInProgress}</>
            : <><GitMerge size={10} /> {t.refactor.merge}</>
          }
        </button>
      )}
    </div>
  );
}
