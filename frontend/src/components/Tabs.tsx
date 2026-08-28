"use client";

import { clsx } from "clsx";

export type BadgeTone = "default" | "accent" | "warn" | "err" | "ok" | "info";
export type TabVariant = "underline" | "pills" | "cards";
export type TabSize = "sm" | "md" | "lg";

export interface TabItem {
  id: string;
  label: string;
  icon?: React.ReactNode;
  badge?: string | number;
  badgeTone?: BadgeTone;
}

const badgeToneClass: Record<BadgeTone, string> = {
  default: "bg-bg-card text-ink-mute border-line",
  accent: "bg-accent/15 text-accent border-accent/30",
  warn: "bg-warn/15 text-warn border-warn/30",
  err: "bg-err/15 text-err border-err/30",
  ok: "bg-ok/15 text-ok border-ok/30",
  info: "bg-info/15 text-info border-info/30",
};

export function Tabs({
  items,
  active,
  onChange,
  variant = "underline",
  size = "md",
  ariaLabel = "Navigation tabs",
}: {
  items: TabItem[];
  active: string;
  onChange: (id: string) => void;
  variant?: TabVariant;
  size?: TabSize;
  ariaLabel?: string;
}) {
  const sizeClass = size === "sm" ? "px-2.5 py-1.5 text-xs" : size === "lg" ? "px-5 py-3 text-base" : "px-3 py-2 text-sm";

  const handleKey = (e: React.KeyboardEvent<HTMLButtonElement>, idx: number) => {
    if (e.key === "ArrowRight" || e.key === "ArrowLeft") {
      e.preventDefault();
      const next = e.key === "ArrowRight"
        ? (idx + 1) % items.length
        : (idx - 1 + items.length) % items.length;
      const el = document.querySelector<HTMLButtonElement>(`[data-tab-idx="${next}"]`);
      el?.focus();
      onChange(items[next].id);
    }
    if (e.key === "Home") { e.preventDefault(); onChange(items[0].id); }
    if (e.key === "End") { e.preventDefault(); onChange(items[items.length - 1].id); }
  };

  const containerClass =
    variant === "pills"
      ? "flex items-center gap-1 p-1 rounded-lg bg-bg-soft/60 border border-line mb-4 w-fit"
      : variant === "cards"
      ? "flex items-center gap-0 border-b border-line mb-4"
      : "flex items-center gap-0 border-b border-line mb-4";

  return (
    <div
      role="tablist"
      aria-label={ariaLabel}
      data-testid="planning-tabs"
      className={containerClass}
    >
      {items.map((it, idx) => {
        const isActive = it.id === active;
        const tone = it.badgeTone ?? "default";
        return (
          <button
            key={it.id}
            type="button"
            role="tab"
            aria-selected={isActive}
            data-testid={`planning-tab-${it.id}`}
            data-tab-idx={idx}
            onClick={() => onChange(it.id)}
            onKeyDown={(e) => handleKey(e, idx)}
            className={clsx(
              "relative flex items-center gap-1.5 font-medium transition-colors outline-none",
              sizeClass,
              variant === "pills" && [
                "rounded-md",
                isActive
                  ? "bg-bg-card text-ink shadow-sm border border-line"
                  : "text-ink-dim hover:text-ink hover:bg-bg-soft",
              ],
              variant !== "pills" && [
                isActive ? "text-accent" : "text-ink-dim hover:text-ink",
              ],
            )}
          >
            {it.icon}
            <span>{it.label}</span>
            {it.badge !== undefined && (
              <span className={clsx("ml-0.5 text-[9px] font-mono px-1.5 py-0 rounded-full border", badgeToneClass[tone])}>
                {it.badge}
              </span>
            )}
            {isActive && variant !== "pills" && (
              <span aria-hidden className="absolute left-0 right-0 -bottom-px h-0.5 bg-accent" />
            )}
          </button>
        );
      })}
    </div>
  );
}
