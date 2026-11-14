/*
@cypher schema=1 source_sha256=c433e39b2df790593e589318c350d7d12575bc300514335086c533802c2a2221
MERGE (self:File {path:"frontend/src/components/magos/MagosLanePanel.tsx"})
MERGE (panel:Symbol {id:"frontend/src/components/magos/MagosLanePanel.tsx::MagosLanePanel",kind:"component"})
MERGE (self)-[:DEFINES]->(panel)
@endcypher
*/
"use client";

// =====================================================================
// MagosLanePanel — MAGOS lane/worktree 管理面板
// per INTEGRATION-MAP-GITHUB-REPOS-001 §2.2 (2026-11-14 用户授权深度集成)
// 数据 = /api/magos 只读代理 (magos_guard.py list/plan/doctor)
// 显示: lane 登记 (branch + 锁定文件域 + 状态 + 登记人) + 空态提示
// =====================================================================

import { useEffect, useState } from "react";
import { Shield, GitBranch, FileText, RefreshCw } from "lucide-react";
import { clsx } from "clsx";

interface MagosClaim {
  branch: string;
  state: string;
  entries: string[];
  created: string;
  by: string;
  [key: string]: unknown;
}

interface MagosApiResponse {
  ok: boolean;
  data?: MagosClaim[];
  error?: string;
}

const STATE_TONE: Record<string, string> = {
  planned: "bg-info/10 text-info border-info/40",
  active: "bg-ok/10 text-ok border-ok/40",
  released: "bg-bg-soft text-ink-mute border-line",
  merged: "bg-warn/10 text-warn border-warn/40",
};

export function MagosLanePanel({ repo = "D:\\Star" }: { repo?: string }) {
  const [claims, setClaims] = useState<MagosClaim[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const load = () => {
    setLoading(true);
    fetch(`/api/magos?cmd=list&repo=${encodeURIComponent(repo)}`)
      .then((r) => r.json())
      .then((d: MagosApiResponse) => {
        if (d.ok) setClaims(d.data ?? []);
        else setError(d.error ?? "MAGOS 读取失败");
      })
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    void load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [repo]);

  return (
    <section data-testid="magos-lane-panel" className="card space-y-2">
      <div className="flex items-center justify-between">
        <h2 className="flex items-center gap-2 text-sm font-bold text-ink">
          <Shield size={14} className="text-accent" /> MAGOS 车道 (Lane)
        </h2>
        <button
          type="button"
          onClick={() => void load()}
          data-testid="magos-refresh"
          className="p-1.5 rounded-md text-ink-mute hover:text-accent hover:bg-bg-soft"
          aria-label="刷新 MAGOS 车道"
        >
          <RefreshCw size={13} className={clsx(loading && "animate-spin")} />
        </button>
      </div>

      {loading ? (
        <p className="text-xs text-ink-mute">读取车道登记中…</p>
      ) : error ? (
        <p data-testid="magos-error" className="text-xs text-err">{error}</p>
      ) : !claims || claims.length === 0 ? (
        <p data-testid="magos-empty" className="text-xs text-ink-dim">
          暂无车道登记 — 由 MAGOS 守卫在 Agent 开工时自动登记 (worktree ↔ 锁定文件域)。
        </p>
      ) : (
        <ul className="space-y-1.5">
          {claims.slice(0, 8).map((c, i) => (
            <li
              key={c.branch + i}
              data-testid={`magos-claim-${c.branch}`}
              className="rounded-lg border border-line bg-bg-soft/50 p-2.5 text-xs"
            >
              <div className="flex items-center gap-2">
                <GitBranch size={12} className="text-ink-mute shrink-0" />
                <span className="font-mono font-semibold text-ink truncate">{c.branch}</span>
                <span className={clsx("text-[9px] px-1.5 py-0.5 rounded border shrink-0", STATE_TONE[c.state] ?? "bg-bg-soft text-ink-mute border-line")}>
                  {c.state}
                </span>
                <span className="text-[10px] text-ink-mute shrink-0">{c.created?.slice(0, 10)}</span>
              </div>
              {Array.isArray(c.entries) && c.entries.length > 0 && (
                <div className="mt-1.5 flex items-start gap-1.5 text-[10px] text-ink-dim">
                  <FileText size={11} className="mt-0.5 shrink-0" />
                  <span className="font-mono truncate">
                    {c.entries.length} 文件域: {c.entries.slice(0, 2).join(", ")}{c.entries.length > 2 ? " …" : ""}
                  </span>
                </div>
              )}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
