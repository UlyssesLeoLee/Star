"use client";

// =====================================================================
// useRefactorToasts — toast queue (per gap #4 / #6)
// =====================================================================

import { useEffect, useState, useCallback, useRef } from "react";
import { clsx } from "clsx";
import { CheckCircle2, AlertTriangle, AlertCircle, Info, X } from "lucide-react";

export type ToastKind = "ok" | "warn" | "err" | "info";

export interface Toast {
  id: string;
  kind: ToastKind;
  message: string;
  /** auto-dismiss in ms, 0 = no auto-dismiss */
  ttlMs?: number;
}

type IconType = typeof CheckCircle2;

const ICONS: Record<ToastKind, IconType> = {
  ok: CheckCircle2,
  warn: AlertTriangle,
  err: AlertCircle,
  info: Info,
};

const COLORS: Record<ToastKind, string> = {
  ok: "border-ok/50 bg-ok/10 text-ok",
  warn: "border-warn/50 bg-warn/10 text-warn",
  err: "border-err/50 bg-err/10 text-err",
  info: "border-info/50 bg-info/10 text-info",
};

let _counter = 0;
const nextId = (): string => `t-${Date.now()}-${++_counter}`;

export function useRefactorToasts() {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const timersRef = useRef<Map<string, ReturnType<typeof setTimeout>>>(new Map());

  const dismiss = useCallback((id: string) => {
    setToasts((cur) => cur.filter((t) => t.id !== id));
    const timer = timersRef.current.get(id);
    if (timer) {
      clearTimeout(timer);
      timersRef.current.delete(id);
    }
  }, []);

  const push = useCallback(
    (kind: ToastKind, message: string, ttlMs = 3000): string => {
      const id = nextId();
      setToasts((cur) => {
        const next = [...cur, { id, kind, message, ttlMs }];
        return next.slice(-3);
      });
      if (ttlMs > 0) {
        const timer = setTimeout(() => dismiss(id), ttlMs);
        timersRef.current.set(id, timer);
      }
      return id;
    },
    [dismiss],
  );

  useEffect(() => {
    const map = timersRef.current;
    return () => {
      map.forEach((t) => clearTimeout(t));
      map.clear();
    };
  }, []);

  return { toasts, push, dismiss } as const;
}

export function RefactorToaster({ toasts, onDismiss }: { toasts: Toast[]; onDismiss: (id: string) => void }) {
  if (toasts.length === 0) return null;
  return (
    <div data-testid="refactor-toaster" role="status" aria-live="polite" className="fixed bottom-4 right-4 z-50 space-y-2 pointer-events-none">
      {toasts.map((t) => {
        const Icon = ICONS[t.kind];
        return (
          <div key={t.id} data-testid={`refactor-toast-${t.kind}`} className={clsx("pointer-events-auto card flex items-center gap-2 px-3 py-2 min-w-[260px] max-w-[400px] border shadow-[0_4px_16px_rgba(0,0,0,0.4)]", COLORS[t.kind])}>
            <Icon size={14} className="shrink-0" />
            <div className="flex-1 text-xs font-mono leading-snug">{t.message}</div>
            <button type="button" onClick={() => onDismiss(t.id)} aria-label="dismiss" className="shrink-0 p-0.5 rounded text-ink-mute hover:text-ink hover:bg-bg-soft">
              <X size={11} />
            </button>
          </div>
        );
      })}
    </div>
  );
}
