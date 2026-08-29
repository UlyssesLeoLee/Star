"use client";

import { clsx } from "clsx";

export function PageHeader({
  title, subtitle, description, icon, track, count, action, actions,
}: {
  title: string;
  subtitle?: string;
  description?: string;
  icon?: React.ReactNode;
  track?: string;
  count?: number | string;
  action?: React.ReactNode;
  actions?: React.ReactNode;
}) {
  const desc = subtitle ?? description;
  const act = action ?? actions;
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
        {desc && <p className="text-sm text-ink-dim max-w-3xl">{desc}</p>}
      </div>
      {act && <div>{act}</div>}
    </div>
  );
}

export function Stat({
  label, value, hint, tone, accent, icon: Icon,
}: {
  label: string;
  value: string | number;
  hint?: string;
  tone?: "ok" | "warn" | "err" | "info" | "default";
  accent?: string;
  icon?: React.ElementType;
}) {
  const effectiveTone = tone ?? (accent === "primary" ? "info" : accent === "success" ? "ok" : "default");
  const color = {
    ok: "text-ok drop-shadow-[0_0_8px_rgba(16,185,129,0.35)]",
    warn: "text-warn drop-shadow-[0_0_8px_rgba(245,158,11,0.35)]",
    err: "text-err drop-shadow-[0_0_8px_rgba(255,51,102,0.35)]",
    info: "text-info drop-shadow-[0_0_8px_rgba(0,240,255,0.35)]",
    default: "text-ink",
  }[effectiveTone];
  return (
    <div className="card group hover:border-accent/40 transition-all duration-200">
      <div className="text-[10px] uppercase tracking-wider text-ink-mute flex items-center justify-between">
        <span className="flex items-center gap-1">
          {Icon && <Icon size={11} className="text-accent" />}
          {label}
        </span>
        <span className="opacity-0 group-hover:opacity-100 text-[8px] font-mono text-accent transition-opacity">// STAT</span>
      </div>
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
