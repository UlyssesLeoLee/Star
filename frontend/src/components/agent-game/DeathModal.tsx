"use client";

// =====================================================================
// DeathModal — 死亡 modal (Revive for 50 🪙 / Restart Lv 1)
// =====================================================================
// Per 2026-09-05 11:42 JST 拍板 #1 + #4:
//   - 死亡触发: cost 预算超支
//   - 复活: 扣 50 金币 + Lv 1 (perks 清零)
//   - 重开: 不扣币 + Lv 1 (perks 清零)
// =====================================================================

import { Coins, Heart, Skull, RotateCcw, RefreshCw, Award } from "lucide-react";
import type { AgentGameState, DeathEvent } from "@/lib/agent-game/types";
import { REVIVE_COST } from "@/lib/agent-game/types";

interface DeathModalProps {
  event: DeathEvent;
  gameState: AgentGameState;
  onRevive: () => void;
  onRestart: () => void;
  onClose: () => void;
}

export function DeathModal({ event, gameState, onRevive, onRestart, onClose }: DeathModalProps) {
  const canRevive = event.canRevive;

  return (
    <div
      data-testid="death-modal"
      className="fixed inset-0 z-50 flex items-center justify-center bg-bg/90 backdrop-blur-sm"
      role="dialog"
      aria-modal="true"
      aria-labelledby="death-modal-title"
    >
      <div className="card max-w-md w-[90%]" data-testid="death-modal-card">
        <div className="text-center">
          <Skull size={48} className="text-err mx-auto mb-3 animate-pulse" />
          <h2 id="death-modal-title" className="text-lg font-semibold text-err mb-1">
            Agent 阵亡!
          </h2>
          <p className="text-xs text-ink-mute mb-4">
            cost 超支 {(event.triggerCostRatio * 100).toFixed(0)}% · HP = 0
          </p>

          <div className="grid grid-cols-3 gap-2 mb-4 text-[10px]">
            <div className="flex flex-col items-center gap-0.5 p-2 bg-bg-soft rounded">
              <Award size={12} className="text-accent" />
              <span className="text-ink-mute">历史最高</span>
              <span className="font-mono font-semibold">Lv {gameState.highestLevel}</span>
            </div>
            <div className="flex flex-col items-center gap-0.5 p-2 bg-bg-soft rounded">
              <Coins size={12} className="text-warn" />
              <span className="text-ink-mute">剩余金币</span>
              <span className="font-mono font-semibold">{event.snapshotCoins}</span>
            </div>
            <div className="flex flex-col items-center gap-0.5 p-2 bg-bg-soft rounded">
              <Heart size={12} className="text-err" />
              <span className="text-ink-mute">死亡次数</span>
              <span className="font-mono font-semibold">{gameState.deaths}</span>
            </div>
          </div>

          <div className="flex flex-col gap-2">
            <button
              data-testid="death-modal-revive"
              onClick={onRevive}
              disabled={!canRevive}
              className={`btn text-xs py-2 ${canRevive ? "border-warn/50 text-warn hover:bg-warn/10" : "border-ink-mute/30 text-ink-mute opacity-50 cursor-not-allowed"}`}
            >
              <RotateCcw size={12} /> 复活 ({REVIVE_COST} 🪙) · 保留 {event.snapshotCoins} → {event.snapshotCoins - REVIVE_COST}
            </button>
            <button
              data-testid="death-modal-restart"
              onClick={onRestart}
              className="btn text-xs py-2 border-ink-mute/40 text-ink-dim hover:bg-bg-soft"
            >
              <RefreshCw size={12} /> 重开 (不扣币) · Lv 1
            </button>
            <button
              data-testid="death-modal-close"
              onClick={onClose}
              className="btn text-[10px] py-1 text-ink-mute"
            >
              关闭 (稍后处理)
            </button>
          </div>

          {!canRevive && (
            <p className="text-[10px] text-err mt-2">
              金币不足 (需 {REVIVE_COST} / 有 {event.snapshotCoins}), 只能重开
            </p>
          )}
        </div>
      </div>
    </div>
  );
}
