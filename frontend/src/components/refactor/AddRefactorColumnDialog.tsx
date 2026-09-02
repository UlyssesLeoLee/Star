"use client";

// =====================================================================
// AddRefactorColumnDialog — 替代 window.prompt 的列添加弹窗 (per 缺口 #3)
// =====================================================================
// 缺口背景 (per 2026-09-02 10:56 JST 补缺口):
//   - 原 RefactorSweepBoard 用 window.prompt, 移动端 + 触屏体验差
//   - 桌面端也无法做校验 / 提示内置 status
// 改进:
//   - 弹窗含: status 输入 + name 输入 + 提示 (内置 5 态 + 现有列名防重)
//   - 校验: status 必填, 不能跟现有列重复, 自动 trim + 规范 (kebab-case 提示)
//   - 提交后弹窗关闭, 父组件接 addRefactorColumn callback
// =====================================================================

import { useEffect, useRef, useState } from "react";
import { clsx } from "clsx";
import { Plus, X, Info } from "lucide-react";
import { useTranslation } from "@/lib/i18n";
import { REFACTOR_DEFAULT_STATUSES } from "@/types/ids";

export interface AddRefactorColumnDialogProps {
  open: boolean;
  onClose: () => void;
  /** 现有列的 status 集合, 用于防重提示 */
  existingStatuses: string[];
  onAdd: (status: string, name?: string) => void;
}

export function AddRefactorColumnDialog({
  open, onClose, existingStatuses, onAdd,
}: AddRefactorColumnDialogProps) {
  const { t } = useTranslation();
  const [status, setStatus] = useState("");
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const statusRef = useRef<HTMLInputElement>(null);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (open) {
      setStatus("");
      setName("");
      setError(null);
      // 自动聚焦 status 输入
      setTimeout(() => statusRef.current?.focus(), 50);
    }
  }, [open]);

  // 外部点击关闭
  useEffect(() => {
    if (!open) return;
    const onClick = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose();
    };
    document.addEventListener("mousedown", onClick);
    return () => document.removeEventListener("mousedown", onClick);
  }, [open, onClose]);

  // Enter 提交, Esc 取消
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
      else if (e.key === "Enter" && (e.target as HTMLElement).tagName !== "BUTTON") {
        submit();
      }
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, status, name]);

  const submit = () => {
    const s = status.trim().toLowerCase().replace(/\s+/g, "_");
    if (!s) {
      setError("status 必填");
      return;
    }
    if (existingStatuses.includes(s)) {
      setError(`status "${s}" 已存在`);
      return;
    }
    if (REFACTOR_DEFAULT_STATUSES.includes(s as typeof REFACTOR_DEFAULT_STATUSES[number])) {
      // 内置 status 但用户输入 — 允许, 视为沿用
    }
    const n = name.trim() || undefined;
    onAdd(s, n);
    onClose();
  };

  if (!open) return null;
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
      data-testid="add-refactor-column-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="add-col-title"
    >
      <div
        ref={ref}
        className="card w-[min(440px,90vw)] border-accent/40 shadow-[0_8px_40px_rgba(0,0,0,0.5)]"
      >
        <div className="flex items-center justify-between mb-3 pb-2 border-b border-line">
          <div className="flex items-center gap-2">
            <Plus size={14} className="text-accent" />
            <div id="add-col-title" className="text-sm font-bold">
              {t.refactor.addColumnTitle}
            </div>
          </div>
          <button
            type="button"
            onClick={onClose}
            aria-label="close"
            className="p-1 rounded text-ink-mute hover:text-ink hover:bg-bg-soft"
          >
            <X size={14} />
          </button>
        </div>

        {/* 提示 */}
        <div className="flex items-start gap-1.5 mb-3 p-2 rounded border border-info/30 bg-info/5">
          <Info size={11} className="text-info shrink-0 mt-0.5" />
          <div className="text-[10px] text-ink-dim leading-relaxed">
            内置 status: {REFACTOR_DEFAULT_STATUSES.map((s) => (
              <code key={s} className="font-mono mx-0.5 px-1 py-0.5 rounded bg-bg-card border border-line text-info">{s}</code>
            ))}
            。自定义 status 建议用英文 (e.g. <code className="font-mono px-1 rounded bg-bg-card border border-line">spike</code> / <code className="font-mono px-1 rounded bg-bg-card border border-line">blocked</code>)
          </div>
        </div>

        {/* status 输入 */}
        <div className="mb-3">
          <label className="block text-[10px] font-mono text-ink-dim mb-1 uppercase tracking-wider">
            Status (必填, 不可改)
          </label>
          <input
            ref={statusRef}
            value={status}
            onChange={(e) => { setStatus(e.target.value); setError(null); }}
            placeholder="e.g. spike / blocked / on_hold"
            data-testid="add-col-status-input"
            className="w-full px-3 py-1.5 rounded border border-line bg-bg-card text-xs font-mono focus:outline-none focus:border-accent"
            autoComplete="off"
            spellCheck={false}
          />
        </div>

        {/* name 输入 (可选) */}
        <div className="mb-3">
          <label className="block text-[10px] font-mono text-ink-dim mb-1 uppercase tracking-wider">
            Display Name (可选, 留空用 i18n 兜底)
          </label>
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="显示名 (e.g. Spike 调研)"
            data-testid="add-col-name-input"
            className="w-full px-3 py-1.5 rounded border border-line bg-bg-card text-xs focus:outline-none focus:border-accent"
            autoComplete="off"
          />
        </div>

        {/* 错误 */}
        {error && (
          <div
            data-testid="add-col-error"
            className="mb-3 px-2 py-1.5 rounded border border-err/40 bg-err/10 text-err text-[10px] font-mono"
            role="alert"
          >
            ⚠ {error}
          </div>
        )}

        {/* 按钮 */}
        <div className="flex items-center justify-end gap-2 pt-2 border-t border-line">
          <button
            type="button"
            onClick={onClose}
            data-testid="add-col-cancel"
            className="px-3 py-1.5 rounded text-xs font-mono text-ink-dim hover:bg-bg-soft"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={submit}
            data-testid="add-col-submit"
            className={clsx(
              "px-3 py-1.5 rounded text-xs font-mono font-bold flex items-center gap-1.5",
              status.trim()
                ? "bg-accent text-bg hover:bg-accent/90"
                : "bg-bg-soft text-ink-mute cursor-not-allowed",
            )}
          >
            <Plus size={11} />
            Add Column
          </button>
        </div>
      </div>
    </div>
  );
}
