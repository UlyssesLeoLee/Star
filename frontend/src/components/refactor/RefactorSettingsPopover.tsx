"use client";

// =====================================================================
// RefactorSettingsPopover — 顶部设置弹窗 (per 2026-09-02 10:41 JST 拍板)
// =====================================================================
// 弹窗内容:
//   1. 改 batch_size (1-50, 默认 5)
//   2. 重置列 (clear custom cols, 回默认 5)
//   3. 关闭 / 拉下一批 / 加任务 等快捷入口 (供主页面嵌入)
// =====================================================================

import { useEffect, useRef, useState } from "react";
import { clsx } from "clsx";
import { Settings, RotateCcw, X } from "lucide-react";
import { useTranslation } from "@/lib/i18n";

export interface RefactorSettingsPopoverProps {
  batchSize: number;
  onChangeBatchSize: (size: number) => void;
  onResetColumns: () => void;
}

export function RefactorSettingsPopover({
  batchSize, onChangeBatchSize, onResetColumns,
}: RefactorSettingsPopoverProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [localSize, setLocalSize] = useState(batchSize);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => setLocalSize(batchSize), [batchSize]);

  // 外部点击关闭
  useEffect(() => {
    if (!open) return;
    const onClick = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", onClick);
    return () => document.removeEventListener("mousedown", onClick);
  }, [open]);

  const commitSize = () => {
    const n = Math.max(1, Math.min(50, Math.floor(Number(localSize) || 5)));
    onChangeBatchSize(n);
  };

  const handleReset = () => {
    if (typeof window !== "undefined") {
      if (!window.confirm(t.refactor.resetColumnsConfirm)) return;
    }
    onResetColumns();
    setOpen(false);
  };

  return (
    <div className="relative" ref={ref}>
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        data-testid="refactor-settings-toggle"
        className={clsx(
          "flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium border-2 transition-colors cel-btn-3d shadow-[2px_2px_0px_0px_var(--cel-ink)]",
          "border-[var(--cel-ink)] bg-bg-soft hover:text-accent",
          open && "text-accent bg-accent/10",
        )}
      >
        <Settings size={12} />
        <span className="font-mono">Settings</span>
      </button>
      {open && (
        <div
          data-testid="refactor-settings-popover"
          className="absolute right-0 top-full mt-2 z-30 w-72 card border-accent/40 shadow-[0_4px_24px_rgba(0,0,0,0.4)] space-y-3"
        >
          <div className="flex items-center justify-between">
            <div className="text-[10px] font-mono uppercase tracking-wider text-ink-dim font-bold">
              Refactor Settings
            </div>
            <button
              type="button"
              onClick={() => setOpen(false)}
              className="p-1 rounded text-ink-mute hover:text-ink hover:bg-bg-soft"
            >
              <X size={11} />
            </button>
          </div>

          {/* batch size */}
          <div>
            <label className="block text-[10px] font-mono text-ink-dim mb-1">
              {t.refactor.batchSizeLabel}
            </label>
            <div className="flex items-center gap-2">
              <input
                type="number"
                min={1}
                max={50}
                value={localSize}
                onChange={(e) => setLocalSize(Number(e.target.value))}
                onBlur={commitSize}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    commitSize();
                    (e.target as HTMLInputElement).blur();
                  }
                }}
                data-testid="refactor-batch-size-input"
                className="flex-1 px-2 py-1.5 border-2 border-[var(--cel-ink)] bg-bg-card text-xs font-mono focus:outline-none focus:border-accent"
              />
              <button
                type="button"
                onClick={commitSize}
                className="px-2 py-1.5 border-2 border-[var(--cel-ink)] bg-accent/10 text-accent text-[10px] font-mono font-bold hover:bg-accent/20 cel-btn-3d shadow-[2px_2px_0px_0px_var(--cel-ink)]"
              >
                Apply
              </button>
            </div>
            <div className="text-[9px] font-mono text-ink-mute mt-1">
              {t.refactor.batchSizeHint}
            </div>
          </div>

          {/* reset columns */}
          <div className="pt-2 border-t border-line">
            <button
              type="button"
              onClick={handleReset}
              data-testid="refactor-reset-columns"
              className="w-full flex items-center justify-center gap-2 px-3 py-1.5 border-2 border-err bg-err/5 text-err text-[10px] font-mono font-bold hover:bg-err/15 cel-btn-3d shadow-[2px_2px_0px_0px_var(--err-DEFAULT)]"
              title={t.refactor.resetColumnsTitle}
            >
              <RotateCcw size={11} />
              {t.refactor.resetColumns}
            </button>
            <div className="text-[9px] font-mono text-ink-mute mt-1 text-center">
              {t.refactor.resetColumnsTitle}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
