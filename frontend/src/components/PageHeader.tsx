"use client";

import { clsx } from "clsx";
import { useTranslation, interpolate } from "@/lib/i18n";

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
  const { t } = useTranslation();
  const desc = subtitle ?? description;
  const act = action ?? actions;
  return (
    <div className="mb-6 flex items-end justify-between gap-4 border-b-2 border-black pb-4">
      <div>
        <div className="flex items-center gap-2.5 mb-1.5">
          {icon}
          <h1 className="text-2xl font-black text-ink tracking-tight" style={{textShadow: '3px 3px 0 var(--cel-shadow-color, #000)'}}>{title}</h1>
          {track && (
            <span className="pill border-line text-ink-dim font-mono text-[10px] font-semibold">
              {interpolate(t.pageHeader.trackPill, { track })}
            </span>
          )}
          {count !== undefined && (
            <span className="anime-hud-tag">
              {count}
            </span>
          )}
        </div>
        {desc && <p className="text-xs text-ink-dim max-w-3xl leading-relaxed font-normal">{desc}</p>}
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
  const { t } = useTranslation();
  const effectiveTone = tone ?? (accent === "primary" ? "info" : accent === "success" ? "ok" : "default");
  const color = {
    ok: "text-ok drop-shadow-[0_0_8px_rgba(16,185,129,0.4)]",
    warn: "text-warn drop-shadow-[0_0_8px_rgba(245,158,11,0.4)]",
    err: "text-err drop-shadow-[0_0_8px_rgba(255,51,102,0.4)]",
    info: "text-info drop-shadow-[0_0_8px_rgba(0,240,255,0.4)]",
    default: "text-ink",
  }[effectiveTone];
  return (
    <div className="card group hover:border-accent/50 hover:shadow-[0_4px_16px_rgba(0,0,0,0.25)] transition-all duration-200">
      <div className="text-[10px] uppercase tracking-wider text-ink-mute flex items-center justify-between font-mono font-medium">
        <span className="flex items-center gap-1.5">
          {Icon && <Icon size={12} className="text-accent" />}
          {label}
        </span>
        <span className="opacity-0 group-hover:opacity-100 text-[8px] font-mono text-accent transition-opacity">
          {t.pageHeader.telemetryTag}
        </span>
      </div>
      <div className={clsx("text-2xl font-bold mt-1 font-mono tracking-tight", color)}>{value}</div>
      {hint && <div className="text-[11px] text-ink-mute mt-0.5 leading-snug">{hint}</div>}
    </div>
  );
}

export function SectionTitle({ children, action }: { children: React.ReactNode; action?: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between mb-3 pb-2 border-b-2 border-black">
      <h2 className="text-xs uppercase tracking-wider text-ink-dim font-bold font-mono">{children}</h2>
      {action}
    </div>
  );
}
