"use client";

// =====================================================================
// GameHUD — 顶部 3渲2 战术战斗 HUD (Lv / Coins / Segmented HP / XP)
// =====================================================================
// Per 2026-09-06 用户发令:
//   - 增强游戏界面感 (HUD / Game UI), 保持低认知负荷的简洁风格
//   - 阶梯分段血槽 (Stepped Segmented HP Gauge) + 墨黑 Cel 边框
//   - 街机金币槽 (Arcade Coin Vault) + 战术职阶徽章 (Rank Emblem)
//   - 死亡态战损告警条纹与战术复活按键
// =====================================================================

import type { AgentGameState } from "@/lib/agent-game/types";
import { MAX_HP, MAX_LEVEL, XP_TO_NEXT_LEVEL, REVIVE_COST } from "@/lib/agent-game/types";
import { Coins, Heart, Skull, Sparkles, ShieldAlert, Zap } from "lucide-react";

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
      <div data-testid="agent-game-hud" className="flex items-center gap-3 text-[11px] text-ink-mute font-mono">
        <span className="opacity-50">no game state</span>
      </div>
    );
  }

  // 死亡态战损告警
  if (!gameState.alive) {
    return (
      <div
        data-testid="agent-game-hud"
        className="flex items-center gap-2.5 px-3 py-1 bg-[var(--cel-surface-card,#0f1422)] border-2 border-black cel-shadow text-xs font-mono"
      >
        <div className="flex items-center gap-1 text-[var(--cel-crimson,#ff184c)] font-black uppercase tracking-wider">
          <Skull size={14} className="animate-pulse shrink-0" />
          <span>DEAD</span>
        </div>
        <span className="text-ink-mute text-[11px]">Lv {gameState.highestLevel} → 1</span>
        <span className="text-[var(--cel-gold,#ffc400)] flex items-center gap-1 font-bold">
          <Coins size={12} /> {gameState.coins}
        </span>
        {gameState.coins >= REVIVE_COST ? (
          <button
            data-testid="hud-revive-btn"
            onClick={onRevive}
            className="text-[10px] font-black uppercase px-2.5 py-1 bg-[var(--cel-gold,#ffc400)] text-black border border-black hover:brightness-110 active:translate-y-0.5 transition-all"
          >
            Revive ({REVIVE_COST} 🪙)
          </button>
        ) : (
          <button
            data-testid="hud-restart-btn"
            onClick={onRestart}
            className="text-[10px] font-black uppercase px-2.5 py-1 bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-text-secondary,#94a3b8)] border border-black hover:text-white active:translate-y-0.5 transition-all"
          >
            Restart Lv 1
          </button>
        )}
        <span className="text-ink-mute text-[10px]">deaths {gameState.deaths}</span>
      </div>
    );
  }

  const nextLvlXp = XP_TO_NEXT_LEVEL[gameState.level - 1] ?? 100;
  const xpPct = gameState.level >= MAX_LEVEL ? 100 : Math.min(100, Math.round((gameState.xp / nextLvlXp) * 100));
  const hpPct = Math.round((gameState.hp / MAX_HP) * 100);
  const tierEmoji = gameState.level >= 5 ? "🌟" : gameState.level >= 3 ? "✨" : "";

  // 10段分段阶梯血槽 (Stepped Segmented Bar)
  const totalSegments = 10;
  const filledSegments = Math.ceil((gameState.hp / MAX_HP) * totalSegments);

  return (
    <div
      data-testid="agent-game-hud"
      className="flex items-center gap-2.5 px-3 py-1 bg-[var(--cel-surface-card,#0f1422)] border-2 border-black cel-shadow text-[11px] font-mono text-ink-dim"
    >
      {/* 战术职阶勋章 (Rank & Level) */}
      <div
        data-testid="hud-level"
        className="flex items-center gap-1.5 px-2 py-0.5 bg-[var(--cel-surface-sub,#151c2c)] border border-black text-[var(--cel-cyan,#00f0ff)] font-bold tracking-tight"
      >
        <span className="text-[10px] font-black text-ink-mute uppercase tracking-widest">RANK</span>
        <span className="font-extrabold text-[12px] text-white">
          Lv.{gameState.level}{tierEmoji}
        </span>
      </div>

      {/* 阶梯战斗血槽 (Stepped Segmented HP Gauge) */}
      <div className="flex items-center gap-1.5" title={`HP: ${gameState.hp} / ${MAX_HP} (${hpPct}%)`}>
        <Heart
          size={12}
          className={hpPct <= 30 ? "text-[var(--cel-crimson,#ff184c)] animate-bounce" : "text-[var(--cel-crimson,#ff184c)]"}
        />
        <div className="flex items-center gap-[2px] p-0.5 bg-black border border-black">
          {Array.from({ length: totalSegments }).map((_, idx) => {
            const isFilled = idx < filledSegments;
            const isDanger = hpPct <= 30;
            return (
              <span
                key={idx}
                className={`w-1.5 h-3 transform -skew-x-12 transition-all ${
                  isFilled
                    ? isDanger
                      ? "bg-[var(--cel-crimson,#ff184c)] animate-pulse shadow-[0_0_4px_#ff184c]"
                      : hpPct > 60
                      ? "bg-[var(--cel-cyan,#00f0ff)] shadow-[0_0_2px_#00f0ff]"
                      : "bg-[var(--cel-gold,#ffc400)]"
                    : "bg-[#1c2333]"
                }`}
              />
            );
          })}
        </div>
        <span className="text-[10px] text-ink-dim font-bold w-11 tabular-nums">
          {gameState.hp}/{MAX_HP}
        </span>
      </div>

      {/* 战术能量蓄力槽 (XP Bar) */}
      <div className="flex items-center gap-1.5" title={`XP: ${gameState.xp} / ${nextLvlXp} (${xpPct}%)`}>
        <Zap size={11} className="text-[var(--cel-gold,#ffc400)]" />
        <div className="w-16 h-2.5 bg-black border border-black p-[1px] flex items-center">
          <div
            className="h-full bg-[var(--cel-gold,#ffc400)] transition-all duration-300 shadow-[0_0_3px_#ffc400]"
            style={{ width: `${xpPct}%` }}
          />
        </div>
        <span className="text-[10px] text-ink-mute tabular-nums">
          {gameState.level >= MAX_LEVEL ? "MAX" : `${gameState.xp}/${nextLvlXp}`}
        </span>
      </div>

      {/* 街机金币槽 (Arcade Coin Vault) */}
      <div
        data-testid="hud-coins"
        className="flex items-center gap-1 px-2 py-0.5 bg-[var(--cel-surface-sub,#151c2c)] border border-black text-[var(--cel-gold,#ffc400)] font-black"
      >
        <Coins size={12} className="text-[var(--cel-gold,#ffc400)]" />
        <span className="tabular-nums text-white text-[12px]">{gameState.coins}</span>
      </div>

      {/* 激活 Perks 计数 */}
      {gameState.perks.length > 0 && (
        <div
          data-testid="hud-perks"
          className="flex items-center gap-1 px-1.5 py-0.5 bg-black/50 border border-black text-[var(--cel-cyan,#00f0ff)] text-[10px] font-bold"
          title={`${gameState.perks.length} active perks unlocked`}
        >
          <Sparkles size={10} />
          <span>{gameState.perks.length}</span>
        </div>
      )}

      {/* 战术使命完成度 */}
      <span className="text-ink-mute text-[10px] hidden xl:inline">
        ops {gameState.completedMissions} · deaths {gameState.deaths}
      </span>

      {/* 升级待选 Perk 战术按钮 */}
      {pendingPerkChoice && (
        <button
          data-testid="hud-pending-perk"
          onClick={onPickPerk}
          className="text-[10px] font-black uppercase tracking-wider px-2 py-1 bg-[var(--cel-crimson,#ff184c)] text-black border border-black animate-pulse hover:brightness-110 shadow-[0_0_6px_#ff184c] transition-all"
        >
          ⚡ Pick Perk
        </button>
      )}
    </div>
  );
}
