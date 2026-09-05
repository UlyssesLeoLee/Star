"use client";

// =====================================================================
// useAgentGame — React hook, 集成 game state 到 React 组件
// =====================================================================
// Per 2026-09-05 11:42 JST 拍板
//   - 提供 gameState (从 store 读)
//   - 提供 actions: claimReward / spendCost / reviveAgent / restartAgent / choosePerk
//   - 提供 derived: alive, level, hp, coins, perks
//   - 监听 store 变化, 自动重派生
// =====================================================================

import { useStore } from "@/lib/store";
import { useMemo, useCallback } from "react";
import type { AgentGameState, PerkId } from "@/lib/agent-game/types";

export function useAgentGame(agentId: string | null) {
  const allGameStates = useStore((s) => s.agentGameStates);
  const claimReward = useStore((s) => s.claimReward);
  const spendCost = useStore((s) => s.spendCost);
  const reviveAgent = useStore((s) => s.reviveAgent);
  const restartAgent = useStore((s) => s.restartAgent);
  const choosePerk = useStore((s) => s.choosePerk);
  const initAgentGame = useStore((s) => s.initAgentGame);

  const gameState = useMemo<AgentGameState | null>(() => {
    if (!agentId) return null;
    return allGameStates[agentId] ?? null;
  }, [allGameStates, agentId]);

  const claim = useCallback((workItemId: string) => {
    if (!agentId) return null;
    return claimReward(agentId, workItemId);
  }, [agentId, claimReward]);

  const spend = useCallback((costDelta: number) => {
    if (!agentId) return null;
    return spendCost(agentId, costDelta);
  }, [agentId, spendCost]);

  const revive = useCallback(() => {
    if (!agentId) return null;
    return reviveAgent(agentId);
  }, [agentId, reviveAgent]);

  const restart = useCallback(() => {
    if (!agentId) return;
    restartAgent(agentId);
  }, [agentId, restartAgent]);

  const pickPerk = useCallback((perkId: PerkId) => {
    if (!agentId) return;
    choosePerk(agentId, perkId);
  }, [agentId, choosePerk]);

  const init = useCallback((budgetUsd: number) => {
    if (!agentId) return;
    initAgentGame(agentId, budgetUsd);
  }, [agentId, initAgentGame]);

  return { gameState, claim, spend, revive, restart, pickPerk, init };
}
