"use client";

import { clsx } from "clsx";

export function PageHeader({
  title, subtitle, icon, track, count,
}: {
  title: string;
  subtitle?: string;
  icon?: React.ReactNode;
  track?: string;
  count?: number | string;
}) {
  return (
    <div className="mb-5 flex items-end justify-between gap-4">
      <div>
        <div className="flex items-center gap-2 mb-1">
          {icon}
          <h1 className="text-xl font-semibold text-ink">{title}</h1>
          {track && (
            <span className="pill border-line text-ink-dim font-mono text-[10px]">Track {track}</span>
          )}
          {count !== undefined && (
            <span className="pill border-accent/40 text-accent bg-accent/10 font-mono text-[10px]">
              {count}
            </span>
          )}
        </div>
        {subtitle && <p className="text-sm text-ink-dim max-w-3xl">{subtitle}</p>}
      </div>
    </div>
  );
}

export function Stat({
  label, value, hint, tone,
}: {
  label: string;
  value: string | number;
  hint?: string;
  tone?: "ok" | "warn" | "err" | "info" | "default";
}) {
  const color = {
    ok: "text-ok",
    warn: "text-warn",
    err: "text-err",
    info: "text-info",
    default: "text-ink",
  }[tone ?? "default"];
  return (
    <div className="card">
      <div className="text-[10px] uppercase tracking-wider text-ink-mute">{label}</div>
      <div className={clsx("text-2xl font-semibold mt-0.5 font-mono", color)}>{value}</div>
      {hint && <div className="text-[11px] text-ink-mute mt-0.5">{hint}</div>}
    </div>
  );
}

export function SectionTitle({ children, action }: { children: React.ReactNode; action?: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between mb-2">
      <h2 className="text-xs uppercase tracking-wider text-ink-dim font-medium">{children}</h2>
      {action}
    </div>
  );
}
