/*
@cypher schema=1 source_sha256=6854142f898225363b746201a3d2bc5eac10ac14c42cf7ffffa6d1a364886af7
MERGE (self:File {path:"frontend/src/components/effects/PhysisHud.tsx"})
MERGE (hud:Symbol {id:"frontend/src/components/effects/PhysisHud.tsx::PhysisHud",kind:"component"})
MERGE (self)-[:DEFINES]->(hud)
@endcypher
*/
"use client";

// =====================================================================
// PhysisHud — Physis 3D 主题游戏化 HUD (per 2026-11-14 用户发令:
//   全 3D 主题应有游戏化动态化整体设计)
//   仅 physis 主题挂载: 左上角粒子状态面板 (粒子数/连击/冲击波)
//   点击任意处 +1 连击 (与背景层冲击波联动)
// =====================================================================

import { useEffect, useState } from "react";
import { useTheme } from "next-themes";
import { useStore } from "@/lib/store";

export function PhysisHud() {
  const { resolvedTheme } = useTheme();
  const worktrees = useStore((s) => s.worktrees);
  const workItems = useStore((s) => s.workItems);
  const agentSessions = useStore((s) => s.agentSessions);
  const [combo, setCombo] = useState(0);
  const [bursts, setBursts] = useState(0);
  const [pulse, setPulse] = useState(false);

  useEffect(() => {
    if (resolvedTheme !== "physis") return;
    const onDown = () => {
      setCombo((c) => c + 1);
      setPulse(true);
      setTimeout(() => setPulse(false), 300);
    };
    const onBurst = () => setBursts((b) => b + 1);
    window.addEventListener("pointerdown", onDown);
    window.addEventListener("physis-burst", onBurst);
    return () => {
      window.removeEventListener("pointerdown", onDown);
      window.removeEventListener("physis-burst", onBurst);
    };
  }, [resolvedTheme]);

  if (resolvedTheme !== "physis") return null;

  return (
    <div
      data-testid="physis-hud"
      className="fixed left-4 bottom-4 z-40 pointer-events-none select-none"
    >
      <div className="rounded-xl border border-accent/40 bg-bg-card/80 backdrop-blur-md px-3.5 py-2.5 font-mono shadow-soft">
        <div className="flex items-center gap-3 text-[10px]">
          <span className="flex items-center gap-1.5">
            <span className={`size-1.5 rounded-full bg-ok ${pulse ? "animate-ping" : ""}`} />
            <span className="text-ink-dim">NODES</span>
            <span className="text-accent font-bold">{Math.min(8, worktrees.length) + Math.min(18, workItems.length) + Math.min(6, agentSessions.length)}</span>
          </span>
          <span className="text-line">|</span>
          <span className="flex items-center gap-1.5">
            <span className="text-ink-dim">COMBO</span>
            <span className={`text-warn font-bold ${pulse ? "scale-125" : ""} transition-transform`}>{combo}</span>
          </span>
          <span className="text-line">|</span>
          <span className="flex items-center gap-1.5">
            <span className="text-ink-dim">BURST</span>
            <span className="text-err font-bold">{bursts}</span>
          </span>
        </div>
        <div className="mt-1.5 text-[9px] text-ink-mute">
          GVPE · WASM · node-edge 图 — worktree/task/agent 实体节点 · 点击触发冲击波
        </div>
      </div>
    </div>
  );
}
