"use client";

import { useStore } from "@/lib/store";
import { PageHeader, SectionTitle, Stat } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { useState } from "react";
import { clsx } from "clsx";

/* ====================================================================
 * 可复用 page builders — 给剩下 24 个 domain 用
 * 每个 builder 接受 seed/数据结构 + 列定义 + (可选)状态机图
 * ==================================================================== */

/** 通用 list 页面 */
export function ListPage<T extends { id: string }>({
  title, subtitle, icon, track, items, columns, searchKeys = [], rowHref,
}: {
  title: string;
  subtitle: string;
  icon: React.ReactNode;
  track: string;
  items: T[];
  columns: Array<{
    key: string;
    label: string;
    render: (item: T) => React.ReactNode;
    width?: string;
  }>;
  searchKeys?: Array<keyof T>;
  rowHref?: (item: T) => string;
}) {
  const [q, setQ] = useState("");
  const filtered = items.filter((item) => {
    if (!q) return true;
    const qLower = q.toLowerCase();
    return searchKeys.some((k) => {
      const v = item[k];
      return typeof v === "string" && v.toLowerCase().includes(qLower);
    });
  });

  return (
    <div className="max-w-7xl">
      <PageHeader title={title} subtitle={subtitle} icon={icon} track={track} count={items.length} />
      <div className="card">
        <div className="flex items-center justify-between mb-3">
          <div className="text-xs text-ink-dim">
            Showing <span className="font-mono text-ink">{filtered.length}</span> / {items.length}
          </div>
          <input
            value={q}
            onChange={(e) => setQ(e.target.value)}
            placeholder="Filter..."
            className="rounded-md border border-line bg-bg-soft px-3 py-1 text-sm placeholder:text-ink-mute focus:outline-none focus:border-accent w-64"
          />
        </div>
        <table className="table">
          <thead>
            <tr>
              {columns.map((c) => (
                <th key={c.key} style={c.width ? { width: c.width } : undefined}>{c.label}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {filtered.map((item) => (
              <tr key={item.id} className={rowHref ? "cursor-pointer" : ""}>
                {columns.map((c) => (
                  <td key={c.key}>{c.render(item)}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

/** Stat grid page */
export function StatsPage({
  title, subtitle, icon, track, stats, children,
}: {
  title: string;
  subtitle: string;
  icon: React.ReactNode;
  track: string;
  stats: Array<{ label: string; value: string | number; hint?: string; tone?: "ok" | "warn" | "err" | "info" | "default" }>;
  children?: React.ReactNode;
}) {
  return (
    <div className="max-w-7xl">
      <PageHeader title={title} subtitle={subtitle} icon={icon} track={track} />
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-5">
        {stats.map((s, i) => <Stat key={i} {...s} />)}
      </div>
      {children}
    </div>
  );
}
