"use client";

// =====================================================================
// RefactorRoundHistory — 历史轮次列表 (per 2026-09-02 10:41 JST 拍板)
// =====================================================================
// 列出所有 closed_at 已填的 round (历史), 进度条 + 起止时间
// active round 在主看板显示, 不在这里列
// =====================================================================

import { useTranslation, interpolate } from "@/lib/i18n";
import { History as HistoryIcon, CheckCircle2, Circle } from "lucide-react";
import { clsx } from "clsx";
import type { RefactorRound } from "@/types/ids";

export interface RefactorRoundHistoryProps {
  rounds: RefactorRound[];
}

export function RefactorRoundHistory({ rounds }: RefactorRoundHistoryProps) {
  const { t } = useTranslation();
  // 按 round_number 降序
  const sorted = [...rounds].sort((a, b) => b.round_number - a.round_number);
  if (sorted.length === 0) {
    return (
      <div data-testid="refactor-history-empty" className="card text-center py-8">
        <HistoryIcon size={24} className="mx-auto text-ink-mute/40 mb-2" />
        <div className="text-xs font-mono text-ink-mute">{t.refactor.historyEmpty}</div>
      </div>
    );
  }
  return (
    <div data-testid="refactor-history" className="space-y-2">
      {sorted.map((r) => {
        const total = r.cards.length;
        const done = r.cards.filter((c) => c.refactor_status === "done").length;
        const pct = total > 0 ? Math.round((done / total) * 100) : 0;
        const isActive = !r.closed_at;
        const startedDate = r.started_at.slice(0, 10);
        const closedDate = r.closed_at?.slice(0, 10) ?? "—";
        return (
          <div
            key={r.id}
            data-testid={`refactor-history-row-${r.round_number}`}
            className={clsx(
              "card hover:border-accent/40 transition-colors",
              isActive && "border-warn/40 bg-warn/5",
            )}
          >
            <div className="flex items-center justify-between mb-1.5">
              <div className="flex items-center gap-2">
                {isActive
                  ? <Circle size={12} className="text-warn animate-pulse" />
                  : <CheckCircle2 size={12} className="text-ok" />
                }
                <span className="text-xs font-mono font-bold">
                  {t.refactor.historyRound}{r.round_number}
                </span>
                {isActive && (
                  <span className="text-[9px] font-mono px-1.5 py-0 rounded border border-warn/40 bg-warn/10 text-warn">
                    {t.refactor.historyActive}
                  </span>
                )}
              </div>
              <span className="text-[10px] font-mono text-ink-dim">
                {interpolate(t.refactor.historyProgress, { done, total })} ({pct}%)
              </span>
            </div>
            {/* 进度条 */}
            <div className="h-1.5 rounded bg-bg-soft overflow-hidden mb-1.5">
              <div
                className={clsx(
                  "h-full transition-all",
                  pct === 100 ? "bg-ok" : isActive ? "bg-warn" : "bg-info",
                )}
                style={{ width: `${pct}%` }}
              />
            </div>
            <div className="flex items-center justify-between text-[9px] font-mono text-ink-mute">
              <span>{t.refactor.historyStarted}: {startedDate}</span>
              <span>{t.refactor.historyClosed}: {closedDate}</span>
            </div>
            {r.notes && (
              <div className="mt-1.5 text-[10px] text-ink-dim line-clamp-2 italic">
                “{r.notes}”
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
