"use client";

// =====================================================================
// PerkPicker — 5 选 1 Power-up 选择 modal
// =====================================================================
// Per 2026-09-05 11:42 JST 拍板 #2: 升级时 5 选 1, 累计叠加 (除 lucky_star)
// =====================================================================

import type { PerkDefinition, AgentGameState, PerkId } from "@/lib/agent-game/types";
import { Sparkles, X, TrendingUp } from "lucide-react";

interface PerkPickerProps {
  gameState: AgentGameState;
  /** 5 选 1 候选 (per getPerkChoices) */
  choices: ReadonlyArray<PerkDefinition>;
  onPick: (perkId: PerkId) => void;
  onClose: () => void;
}

const PERK_ICONS: Record<PerkId, string> = {
  xp_boost: "📈",
  coin_magnet: "🪙",
  bounty_hunter: "🎯",
  iron_will: "🛡️",
  lucky_star: "🍀",
};

const PERK_COLORS: Record<PerkId, string> = {
  xp_boost: "border-info/40 bg-info/10 text-info",
  coin_magnet: "border-warn/40 bg-warn/10 text-warn",
  bounty_hunter: "border-err/40 bg-err/10 text-err",
  iron_will: "border-ok/40 bg-ok/10 text-ok",
  lucky_star: "border-accent/40 bg-accent/10 text-accent",
};

export function PerkPicker({ gameState, choices, onPick, onClose }: PerkPickerProps) {
  const counts = gameState.perks.reduce<Record<PerkId, number>>((acc, p) => {
    acc[p] = (acc[p] ?? 0) + 1;
    return acc;
  }, { xp_boost: 0, coin_magnet: 0, bounty_hunter: 0, iron_will: 0, lucky_star: 0 });

  return (
    <div
      data-testid="perk-picker-modal"
      className="fixed inset-0 z-50 flex items-center justify-center bg-bg/80 backdrop-blur-sm"
      role="dialog"
      aria-modal="true"
      aria-labelledby="perk-picker-title"
    >
      <div className="card max-w-2xl w-[90%] max-h-[80vh] overflow-y-auto" data-testid="perk-picker-card">
        <div className="flex items-center justify-between mb-3">
          <div>
            <h2 id="perk-picker-title" className="text-base font-semibold flex items-center gap-2">
              <Sparkles size={16} className="text-accent" />
              升级到 Lv {gameState.level}!
            </h2>
            <p className="text-[10px] text-ink-mute mt-0.5">选择 1 个 power-up (累计叠加, 除 lucky_star)</p>
          </div>
          <button
            data-testid="perk-picker-close"
            onClick={onClose}
            className="btn p-1"
            aria-label="close"
          >
            <X size={14} />
          </button>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
          {choices.map((perk) => {
            const count = counts[perk.id] ?? 0;
            const stackable = perk.stackable;
            return (
              <button
                key={perk.id}
                data-testid={`perk-option-${perk.id}`}
                onClick={() => onPick(perk.id)}
                className={`card text-left p-3 hover:border-accent/60 transition-colors ${PERK_COLORS[perk.id]}`}
              >
                <div className="flex items-start gap-2">
                  <div className="text-2xl shrink-0" aria-hidden>
                    {PERK_ICONS[perk.id]}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-1.5">
                      <span className="text-sm font-semibold">{perk.name}</span>
                      {count > 0 && (
                        <span className="text-[10px] px-1 rounded border border-current/40 font-mono">
                          ×{count}
                        </span>
                      )}
                    </div>
                    <div className="text-[10px] text-ink-mute mt-0.5">{perk.description}</div>
                    <div className="text-[9px] uppercase tracking-wider mt-1 opacity-70">
                      {stackable ? "STACKABLE" : "SINGLE"}
                    </div>
                  </div>
                </div>
              </button>
            );
          })}
        </div>

        <div className="mt-3 text-[10px] text-ink-mute text-center flex items-center justify-center gap-1">
          <TrendingUp size={10} />
          已选 {gameState.perks.length} 个 perk · 当前 Lv {gameState.level}
        </div>
      </div>
    </div>
  );
}
