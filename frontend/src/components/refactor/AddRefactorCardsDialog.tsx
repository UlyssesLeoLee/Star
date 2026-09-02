"use client";

// =====================================================================
// AddRefactorCardsDialog — 从 project status=done 任务中选加入 round
// =====================================================================
// 弹窗: 列出本项目 done 任务, 多选, 提交调 addRefactorCard
// 排除已在 round 内的 (per store.addRefactorCard 防重)
// =====================================================================

import { useEffect, useRef, useState, useMemo } from "react";
import { clsx } from "clsx";
import { Plus, X, Check } from "lucide-react";
import { useTranslation, useStatusLabel } from "@/lib/i18n";
import type { WorkItem } from "@/types/ids";

export interface AddRefactorCardsDialogProps {
  open: boolean;
  onClose: () => void;
  /** 项目 done 任务全集 */
  doneWorkItems: WorkItem[];
  /** round 内已存在的 work_item_id 集合 */
  alreadyInRound: Set<string>;
  onAdd: (workItemIds: string[]) => void;
}

export function AddRefactorCardsDialog({
  open, onClose, doneWorkItems, alreadyInRound, onAdd,
}: AddRefactorCardsDialogProps) {
  const { t } = useTranslation();
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [query, setQuery] = useState("");
  const ref = useRef<HTMLDivElement>(null);
  const statusLabel = useStatusLabel("workItem", "done");

  // 排除已在 round
  const available = useMemo(
    () => doneWorkItems.filter((w) => !alreadyInRound.has(w.id)),
    [doneWorkItems, alreadyInRound],
  );
  const filtered = useMemo(
    () => available.filter((w) => {
      if (!query.trim()) return true;
      const q = query.toLowerCase();
      return w.title.toLowerCase().includes(q)
        || w.key.toLowerCase().includes(q)
        || (w.labels ?? []).some((l) => l.toLowerCase().includes(q));
    }),
    [available, query],
  );

  useEffect(() => {
    if (!open) return;
    const onClick = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        onClose();
      }
    };
    document.addEventListener("mousedown", onClick);
    return () => document.removeEventListener("mousedown", onClick);
  }, [open, onClose]);

  // 重置状态 when open
  useEffect(() => {
    if (open) {
      setSelected(new Set());
      setQuery("");
    }
  }, [open]);

  const toggle = (id: string) => {
    setSelected((cur) => {
      const next = new Set(cur);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const submit = () => {
    if (selected.size === 0) return;
    onAdd(Array.from(selected));
    onClose();
  };

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
      data-testid="refactor-add-dialog"
    >
      <div
        ref={ref}
        className="card w-[min(640px,90vw)] max-h-[80vh] flex flex-col border-accent/40 shadow-[0_8px_40px_rgba(0,0,0,0.5)]"
      >
        <div className="flex items-center justify-between mb-3 pb-2 border-b border-line">
          <div>
            <div className="text-sm font-bold">{t.refactor.addCardsTitle}</div>
            <div className="text-[10px] font-mono text-ink-mute mt-0.5">
              {available.length} available · {selected.size} selected
            </div>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="p-1 rounded text-ink-mute hover:text-ink hover:bg-bg-soft"
          >
            <X size={14} />
          </button>
        </div>

        {/* 搜索 */}
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="filter by title / key / label..."
          data-testid="refactor-add-search"
          className="w-full mb-2 px-3 py-1.5 rounded border border-line bg-bg-soft text-xs focus:outline-none focus:border-accent"
        />

        {/* 列表 */}
        <div className="flex-1 overflow-y-auto space-y-1.5 pr-1 min-h-[200px] max-h-[50vh]">
          {filtered.length === 0 ? (
            <div className="text-center py-12">
              <div className="text-xs font-mono text-ink-mute">
                {available.length === 0 ? t.refactor.noDoneWorkItems : "0 hit"}
              </div>
            </div>
          ) : (
            filtered.map((w) => {
              const isSel = selected.has(w.id);
              return (
                <button
                  key={w.id}
                  type="button"
                  onClick={() => toggle(w.id)}
                  data-testid={`refactor-add-item-${w.id}`}
                  className={clsx(
                    "w-full text-left p-2.5 rounded border transition-colors",
                    isSel
                      ? "border-accent/60 bg-accent/10"
                      : "border-line bg-bg-soft/40 hover:border-accent/40 hover:bg-bg-soft",
                  )}
                >
                  <div className="flex items-start gap-2">
                    <div className={clsx(
                      "shrink-0 size-4 rounded border flex items-center justify-center mt-0.5",
                      isSel ? "border-accent bg-accent text-white" : "border-line bg-bg-card",
                    )}>
                      {isSel && <Check size={10} />}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-1.5 mb-0.5">
                        <span className="font-mono text-[10px] text-info font-medium">
                          {w.key}
                        </span>
                        <span className="text-[9px] font-mono px-1 py-0 rounded border border-ok/40 bg-ok/10 text-ok">
                          {statusLabel}
                        </span>
                        <span className="text-[9px] font-mono px-1 py-0 rounded border border-line bg-bg-card text-ink-mute uppercase">
                          {w.priority}
                        </span>
                      </div>
                      <div className="text-xs font-medium text-ink line-clamp-1">{w.title}</div>
                    </div>
                  </div>
                </button>
              );
            })
          )}
        </div>

        {/* 底部按钮 */}
        <div className="flex items-center justify-end gap-2 mt-3 pt-2 border-t border-line">
          <button
            type="button"
            onClick={onClose}
            className="px-3 py-1.5 rounded text-xs font-mono text-ink-dim hover:bg-bg-soft"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={submit}
            disabled={selected.size === 0}
            data-testid="refactor-add-submit"
            className={clsx(
              "px-3 py-1.5 rounded text-xs font-mono font-bold flex items-center gap-1.5",
              selected.size > 0
                ? "bg-accent text-bg hover:bg-accent/90"
                : "bg-bg-soft text-ink-mute cursor-not-allowed",
            )}
          >
            <Plus size={11} />
            Add {selected.size > 0 ? `(${selected.size})` : ""}
          </button>
        </div>
      </div>
    </div>
  );
}
