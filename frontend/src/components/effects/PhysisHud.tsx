/*
@cypher schema=1 source_sha256=b6fe5e397a371a1731e05c8d8c66439187bf7837c1b1098211ea4e1329795fc5
MERGE (self:File {path:"frontend/src/components/effects/PhysisHud.tsx"})
MERGE (hud:Symbol {id:"frontend/src/components/effects/PhysisHud.tsx::PhysisHud",kind:"component"})
MERGE (self)-[:DEFINES]->(hud)
@endcypher
*/
"use client";

// =====================================================================
// PhysisHud — Physis 3D 主题游戏化 HUD (per 2026-11-18 全息3D化升级)
//   仅 physis 主题挂载: 左下角全息战术面板
//   - CSS 3D 透视倾斜 + 全息边缘光 + 扫描线动画
//   - NODES/COMBO/BURST 三仪表: 数字翻动 + 能量条
//   - 点击任意处 +1 连击 (与背景层冲击波联动)
//   - prefers-reduced-motion: 关闭扫描线/脉冲动画
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
  const [comboFlash, setComboFlash] = useState(false);

  useEffect(() => {
    if (resolvedTheme !== "physis") return;
    const onDown = () => {
      setCombo((c) => c + 1);
      setPulse(true);
      setComboFlash(true);
      setTimeout(() => setPulse(false), 300);
      setTimeout(() => setComboFlash(false), 200);
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

  const nodeCount = Math.min(8, worktrees.length) + Math.min(18, workItems.length) + Math.min(6, agentSessions.length);
  const nodeMax = 32;
  const comboEnergy = Math.min(100, (combo % 50) * 2);

  return (
    <div
      data-testid="physis-hud"
      className="fixed left-4 bottom-4 z-40 pointer-events-none select-none"
      style={{ perspective: "600px" }}
    >
      <div
        className="physis-hud-panel relative rounded-xl border border-accent/50 bg-bg-card/80 backdrop-blur-md px-4 py-3 font-mono shadow-soft"
        style={{
          transform: "rotateX(4deg) rotateY(-3deg)",
          transformStyle: "preserve-3d",
          boxShadow: "0 0 18px 2px color-mix(in srgb, var(--color-primary, #5ad2ff) 25%, transparent), inset 0 0 12px 1px color-mix(in srgb, var(--color-primary, #5ad2ff) 12%, transparent)",
        }}
      >
        {/* 扫描线 */}
        <div className="physis-hud-scanline absolute inset-0 rounded-xl overflow-hidden pointer-events-none" aria-hidden>
          <div className="physis-hud-scanbar absolute left-0 right-0 h-[2px] bg-accent/40" />
        </div>

        {/* 四角战术标记 */}
        <span className="absolute -top-px -left-px size-2.5 border-t-2 border-l-2 border-accent" aria-hidden />
        <span className="absolute -top-px -right-px size-2.5 border-t-2 border-r-2 border-accent" aria-hidden />
        <span className="absolute -bottom-px -left-px size-2.5 border-b-2 border-l-2 border-accent" aria-hidden />
        <span className="absolute -bottom-px -right-px size-2.5 border-b-2 border-r-2 border-accent" aria-hidden />

        <div className="flex items-center gap-3 text-[10px]" style={{ transform: "translateZ(8px)" }}>
          <span className="flex items-center gap-1.5">
            <span className={`size-1.5 rounded-full bg-ok ${pulse ? "animate-ping" : ""}`} />
            <span className="text-ink-dim">NODES</span>
            <span className="text-accent font-bold tabular-nums">{nodeCount}</span>
          </span>
          <span className="text-line">|</span>
          <span className="flex items-center gap-1.5">
            <span className="text-ink-dim">COMBO</span>
            <span
              className={`text-warn font-bold tabular-nums transition-transform duration-150 ${comboFlash ? "scale-150" : pulse ? "scale-125" : ""}`}
              style={comboFlash ? { textShadow: "0 0 8px var(--warn-DEFAULT, #d29922)" } : undefined}
            >
              {combo}
            </span>
          </span>
          <span className="text-line">|</span>
          <span className="flex items-center gap-1.5">
            <span className="text-ink-dim">BURST</span>
            <span className="text-err font-bold tabular-nums">{bursts}</span>
          </span>
        </div>

        {/* 能量条: 节点密度 + 连击能量 */}
        <div className="mt-2 flex flex-col gap-1" style={{ transform: "translateZ(4px)" }}>
          <div className="flex items-center gap-1.5">
            <span className="text-[8px] text-ink-mute w-8">DENS</span>
            <div className="flex-1 h-1 rounded-full bg-line/40 overflow-hidden">
              <div
                className="h-full rounded-full bg-ok transition-all duration-500"
                style={{ width: `${Math.min(100, (nodeCount / nodeMax) * 100)}%` }}
              />
            </div>
          </div>
          <div className="flex items-center gap-1.5">
            <span className="text-[8px] text-ink-mute w-8">PWR</span>
            <div className="flex-1 h-1 rounded-full bg-line/40 overflow-hidden">
              <div
                className="h-full rounded-full bg-warn transition-all duration-200"
                style={{ width: `${comboEnergy}%`, boxShadow: comboEnergy > 60 ? "0 0 6px var(--warn-DEFAULT, #d29922)" : undefined }}
              />
            </div>
          </div>
        </div>

        <div className="mt-1.5 text-[9px] text-ink-mute" style={{ transform: "translateZ(2px)" }}>
          GVPE · WASM · node-edge 图 — worktree/task/agent 实体节点 · 点击触发冲击波
        </div>
      </div>

      <style jsx>{`
        @keyframes physis-hud-scan {
          0% { top: -2px; opacity: 0; }
          10% { opacity: 1; }
          90% { opacity: 1; }
          100% { top: 100%; opacity: 0; }
        }
        .physis-hud-scanbar {
          animation: physis-hud-scan 4s linear infinite;
        }
        @keyframes physis-hud-breathe {
          0%, 100% { box-shadow: 0 0 18px 2px color-mix(in srgb, var(--color-primary, #5ad2ff) 25%, transparent), inset 0 0 12px 1px color-mix(in srgb, var(--color-primary, #5ad2ff) 12%, transparent); }
          50% { box-shadow: 0 0 26px 4px color-mix(in srgb, var(--color-primary, #5ad2ff) 40%, transparent), inset 0 0 16px 2px color-mix(in srgb, var(--color-primary, #5ad2ff) 20%, transparent); }
        }
        .physis-hud-panel {
          animation: physis-hud-breathe 3.5s ease-in-out infinite;
        }
        @media (prefers-reduced-motion: reduce) {
          .physis-hud-scanbar, .physis-hud-panel {
            animation: none;
          }
        }
      `}</style>
    </div>
  );
}
