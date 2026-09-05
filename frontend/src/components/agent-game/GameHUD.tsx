"use client";

// =====================================================================
// GameHUD — 顶部 HUD (Lv / Coins / HP / Perk 摘要)
// =====================================================================
// Per 2026-09-05 11:42 JST 拍板, 拟人化游戏化 v0.1
//   - 始终显示当前 agent 的 level / xp / coins / hp
//   - 死亡时整体变灰 + 💀 skull
//   - perk 摘要 (5 选 1 累计)
// =====================================================================

import type { AgentGameState } from "@/lib/agent-game/types";
import { MAX_HP, MAX_LEVEL, XP_TO_NEXT_LEVEL, REVIVE_COST } from "@/lib/agent-game/types";
import { Coins, Heart, Skull, Sparkles, TrendingUp, Zap } from "lucide-react";

interface GameHUDProps {
  gameState: AgentGameState | null;
  onRevive?: () => void;
  onRestart?: () => void;
  onPickPerk?: () => void;
  /** 当前是否处于 "升级待选 perk" 状态 (Lv 升了但 perks 没选) */
  pendingPerkChoice?: boolean;
}

export function GameHUD({ gameState, onRevive, onRestart, onPickPerk, pendingPerkChoice }: GameHUDProps) {
  if (!gameState) {
    return (
      <div data-testid="agent-game-hud" className="flex items-center gap-3 text-[10px] text-ink-mute font-mono">
        <span className="opacity-50">no game state</span>
      </div>
    );
  }

  if (!gameState.alive) {
    return (
      <div data-testid="agent-game-hud" className="flex items-center gap-2 text-xs font-mono">
        <Skull size={14} className="text-err shrink-0" />
        <span className="text-err font-semibold">DEAD</span>
        <span className="text-ink-mute">Lv {gameState.highestLevel} → 1</span>
        <span className="text-warn flex items-center gap-0.5">
          <Coins size={10} /> {gameState.coins}
        </span>
        {gameState.coins >= REVIVE_COST ? (
          <button
            data-testid="hud-revive-btn"
            onClick={onRevive}
            className="btn text-[10px] py-0.5 px-1.5 border-warn/50 text-warn hover:bg-warn/10"
          >
            Revive ({REVIVE_COST} 🪙)
          </button>
        ) : (
          <button
            data-testid="hud-restart-btn"
            onClick={onRestart}
            className="btn text-[10px] py-0.5 px-1.5 border-ink-mute/40 text-ink-dim hover:bg-bg-soft"
          >
            Restart Lv 1
          </button>
        )}
        <span className="text-ink-mute text-[10px]">deaths {gameState.deaths}</span>
      </div>
    );
  }

  const xpPct = gameState.level >= MAX_LEVEL ? 100 : Math.round((gameState.xp / XP_TO_NEXT_LEVEL[gameState.level - 1]) * 100);
  const hpPct = Math.round((gameState.hp / MAX_HP) * 100);
  const tierEmoji = gameState.level >= 5 ? "🌟" : gameState.level >= 3 ? "✨" : "";

  return (
    <div data-testid="agent-game-hud" className="flex items-center gap-3 text-[11px] font-mono">
      {/* Level */}
      <div data-testid="hud-level" className="flex items-center gap-1 text-info">
        <TrendingUp size={11} />
        <span className="font-semibold">Lv {gameState.level}{tierEmoji}</span>
      </div>

      {/* XP bar */}
      <div className="flex items-center gap-1">
        <Zap size={10} className="text-warn" />
        <div className="w-16 h-1.5 bg-bg-soft rounded overflow-hidden">
          <div className="h-full bg-warn" style={{ width: `${xpPct}%` }} />
        </div>
        <span className="text-ink-mute text-[10px] w-12">
          {gameState.level >= MAX_LEVEL ? "MAX" : `${gameState.xp}/${XP_TO_NEXT_LEVEL[gameState.level - 1]}`}
        </span>
      </div>

      {/* Coins */}
      <div data-testid="hud-coins" className="flex items-center gap-0.5 text-warn">
        <Coins size={11} />
        <span className="font-semibold">{gameState.coins}</span>
      </div>

      {/* HP bar */}
      <div className="flex items-center gap-1">
        <Heart size={10} className={hpPct <= 30 ? "text-err animate-pulse" : "text-err"} />
        <div className="w-16 h-1.5 bg-bg-soft rounded overflow-hidden">
          <div className={`h-full ${hpPct <= 30 ? "bg-err animate-pulse" : "bg-err"}`} style={{ width: `${hpPct}%` }} />
        </div>
        <span className="text-ink-mute text-[10px] w-12">
          {gameState.hp}/{MAX_HP}
        </span>
      </div>

      {/* Perks (5 选 1 累计) */}
      {gameState.perks.length > 0 && (
        <div data-testid="hud-perks" className="flex items-center gap-0.5 text-[10px] text-ink-mute">
          <Sparkles size={10} className="text-accent" />
          <span>{gameState.perks.length}</span>
        </div>
      )}

      {/* Stats */}
      <span className="text-ink-mute text-[10px] hidden md:inline">
        missions {gameState.completedMissions} · deaths {gameState.deaths}
      </span>

      {/* Pending perk choice indicator */}
      {pendingPerkChoice && (
        <button
          data-testid="hud-pending-perk"
          onClick={onPickPerk}
          className="btn text-[10px] py-0.5 px-1.5 border-accent text-accent animate-pulse"
        >
          ⚡ Pick Perk
        </button>
      )}
    </div>
  );
}
