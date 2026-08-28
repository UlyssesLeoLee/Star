"use client";

import { clsx } from "clsx";

export interface TabItem {
  id: string;
  label: string;
  icon?: React.ReactNode;
  badge?: string | number;
}

export function Tabs({
  items,
  active,
  onChange,
}: {
  items: TabItem[];
  active: string;
  onChange: (id: string) => void;
}) {
  return (
    <div
      role="tablist"
      aria-label="Planning tabs"
      data-testid="planning-tabs"
      className="flex items-center gap-0 border-b border-line mb-4"
    >
      {items.map((it) => {
        const isActive = it.id === active;
        return (
          <button
            key={it.id}
            type="button"
            role="tab"
            aria-selected={isActive}
            data-testid={`planning-tab-${it.id}`}
            onClick={() => onChange(it.id)}
            className={clsx(
              "relative px-3 py-2 text-sm font-medium flex items-center gap-1.5 transition-colors",
              isActive
                ? "text-accent"
                : "text-ink-dim hover:text-ink",
            )}
          >
            {it.icon}
            {it.label}
            {it.badge !== undefined && (
              <span className="ml-1 text-[9px] font-mono text-ink-mute px-1.5 py-0 rounded-full border border-line">
                {it.badge}
              </span>
            )}
            {isActive && (
              <span
                aria-hidden
                className="absolute left-0 right-0 -bottom-px h-0.5 bg-accent"
              />
            )}
          </button>
        );
      })}
    </div>
  );
}
