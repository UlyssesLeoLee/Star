"use client";

// =====================================================================
// RefactorKpiRow — 顶部 5 KPI 卡 (per 2026-09-02 10:41 JST 拍板)
// =====================================================================
// 展示当前 round 内每列卡数 + 总数 + 完成率
// 颜色按 status (跟 workItem StatusPill 颜色映射一致)
// =====================================================================

import { clsx } from "clsx";
import { useTranslation } from "@/lib/i18n";
import type { RefactorColumn, RefactorCard as RefactorCardData } from "@/types/ids";

export interface RefactorKpiRowProps {
  columns: RefactorColumn[];
  cards: RefactorCardData[];
  /** 当前 batch index (1-based) 和总批数 */
  currentBatchIdx?: number;
  totalBatches?: number;
}

const KPI_TONE: Record<string, string> = {
  todo: "border-ink-mute/40 text-ink-dim",
  doing: "border-info/40 text-info",
  testing: "border-warn/40 text-warn",
  review: "border-warn/40 text-warn",
  done: "border-ok/40 text-ok",
};

export function RefactorKpiRow({ columns, cards, currentBatchIdx, totalBatches }: RefactorKpiRowProps) {
  const { t } = useTranslation();
  const sorted = [...columns].sort((a, b) => a.position - b.position);
  const total = cards.length;
  const done = cards.filter((c) => c.refactor_status === "done").length;
  const ratio = total > 0 ? Math.round((done / total) * 100) : 0;

  return (
    <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-5 gap-2">
      {sorted.map((col) => {
        const count = cards.filter((c) => c.refactor_status === col.status).length;
        const tone = KPI_TONE[col.status] ?? "border-line text-ink-dim";
        const label = col.name ?? col.status;
        return (
          <div
            key={col.status}
            data-testid={`refactor-kpi-${col.status}`}
            className={clsx("card group transition-all duration-200 hover:border-accent/50", tone)}
          >
            <div className="text-[9px] uppercase tracking-wider text-ink-mute font-mono font-medium">
              {label}
            </div>
            <div className="text-2xl font-bold mt-0.5 font-mono tracking-tight">
              {count}
            </div>
          </div>
        );
      })}

      {/* 进度指示 */}
      {total > 0 && (
        <div
          data-testid="refactor-kpi-progress"
          className="card col-span-2 sm:col-span-1 border-accent/30 bg-accent/5"
        >
          <div className="text-[9px] uppercase tracking-wider text-accent font-mono font-medium">
            {t.refactor.finishedCards}
          </div>
          <div className="text-2xl font-bold mt-0.5 font-mono tracking-tight text-accent">
            {ratio}%
          </div>
          {currentBatchIdx && totalBatches && (
            <div className="text-[9px] font-mono text-ink-mute mt-0.5">
              {t.refactor.batchLabel} {currentBatchIdx}/{totalBatches}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
