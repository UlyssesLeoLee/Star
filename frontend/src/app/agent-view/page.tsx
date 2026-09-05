"use client";

// =====================================================================
// /agent-view — Agent 视图 (无限画布, 当前工作 agent 筛选, 拟人化游戏化)
// =====================================================================
// Per 2026-09-05 11:25 JST 用户发令 + 拍板 #1/#2/#3:
//   1. 形式 = 无限画布 (Miro 风格, 自由散开)
//   2. 筛选 = 当前工作 agent (auto 选最近 active, 用户可手动覆盖)
//   3. 数据 = 跟 kanban 等界面共享 store (workItems / worktrees / agentSessions)
//   4. 路由 = /agent-view
//   5. 界面名 = "Agent"
//
// Per 2026-09-05 11:42 JST 拍板 (游戏化):
//   - Lv 1..10 升级 (完成 work-item → xp + coins)
//   - 死亡 (cost 超支) → 回 Lv 1, 保留 50% 金币, 可花 50 复活
//   - 重开 (不扣币) 作为没钱的备选
//   - 5 选 1 Power-up (per-life, 升级时弹 PerkPicker)
//   - Lv 1..10 视觉渐进 (色/大小/光环/装饰 emoji)
// =====================================================================

import { useMemo, useCallback, useEffect, useState } from "react";
import { useSearchParams, useRouter, usePathname } from "next/navigation";
import { useStore } from "@/lib/store";
import {
  resolveCurrentAgent,
  pickAgentWorktree,
  pickAgentWorkItems,
} from "@/lib/agent-view/selectors";
import { layoutAgentCanvas, fitToContentViewport } from "@/lib/agent-view/layout";
import type { AgentCanvas } from "@/lib/agent-view/types";
import { AgentCanvasView } from "@/components/agent-view/AgentCanvasView";
import { AgentFilter } from "@/components/agent-view/AgentFilter";
import { GameHUD } from "@/components/agent-game/GameHUD";
import { PerkPicker } from "@/components/agent-game/PerkPicker";
import { DeathModal } from "@/components/agent-game/DeathModal";
import { useAgentGame } from "@/components/agent-game/useAgentGame";
import { getPerkChoices } from "@/lib/agent-game/perks";
import { PageHeader } from "@/components/PageHeader";
import { Bot, AlertTriangle, Maximize2, Zap, Sparkles } from "lucide-react";
import type { PerkId } from "@/lib/agent-game/types";

export default function AgentViewPage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  // store (顶层订阅, 避免 hooks 违规)
  const agents = useStore((s) => s.agentSessions);
  const worktrees = useStore((s) => s.worktrees);
  const workItems = useStore((s) => s.workItems);
  const initAgentGame = useStore((s) => s.initAgentGame);

  // URL 参数: ?agent=ag-XXX
  const urlAgentId = searchParams.get("agent");

  // 解析 current agent
  const resolution = useMemo(
    () => resolveCurrentAgent(agents, urlAgentId),
    [agents, urlAgentId],
  );

  // 取 worktree + work-items
  const worktree = useMemo(
    () => (resolution ? pickAgentWorktree(worktrees, resolution.agent) : null),
    [worktrees, resolution],
  );
  const relatedWorkItems = useMemo(
    () => (resolution ? pickAgentWorkItems(workItems, resolution.agent, worktree) : []),
    [workItems, resolution, worktree],
  );

  // 拟人化游戏化 (per 2026-09-05 11:42 JST 拍板)
  const { gameState, claim, spend, revive, restart, pickPerk, init } = useAgentGame(resolution?.agentId ?? null);

  // Modal 状态
  const [pendingPerkLevel, setPendingPerkLevel] = useState<number | null>(null);
  const [deathEvent, setDeathEvent] = useState<{
    agentId: string; triggerCostRatio: number; snapshotCoins: number; snapshotLevel: number; snapshotHp: number; canRevive: boolean; timestamp: string;
  } | null>(null);

  // 首次访问某 agent: 自动 lazy init
  useEffect(() => {
    if (resolution && !gameState) {
      init(resolution.agent.cost_summary.budget_usd);
    }
  }, [resolution, gameState, init]);

  // 派生 canvas (layout + fit-to-content viewport)
  const canvas: AgentCanvas | null = useMemo(() => {
    if (!resolution) return null;
    const layout = layoutAgentCanvas({
      agent: resolution.agent,
      worktree,
      workItems: relatedWorkItems,
    });
    return {
      agentId: resolution.agent.id,
      nodes: layout.nodes,
      connectors: layout.connectors,
      viewport: fitToContentViewport(layout.bbox, 1200, 800, 60),
      derivedAt: new Date().toISOString(),
    };
  }, [resolution, worktree, relatedWorkItems]);

  // dropdown 切换: 更新 URL (per 拍板 #2)
  const handleAgentChange = useCallback(
    (agentId: string) => {
      const params = new URLSearchParams(searchParams.toString());
      params.set("agent", agentId);
      router.replace(`${pathname}?${params.toString()}`, { scroll: false });
    },
    [router, pathname, searchParams],
  );

  // Claim 回调
  const handleClaim = useCallback((workItemId: string) => {
    const r = claim(workItemId);
    if (!r || !r.ok) return;
    if (r.leveledUp) {
      // 升级 → 弹 PerkPicker (per 拍板 #2, 5 选 1)
      setPendingPerkLevel(r.levelsGained);
    }
  }, [claim]);

  // Spend cost 回调 (消耗资源模拟)
  const handleSpend = useCallback(() => {
    if (!resolution) return;
    const r = spend(0.1);  // 模拟每次执行 step +0.1 usd
    if (!r || !r.ok) return;
    if (r.died) {
      const gs = useStore.getState().agentGameStates[resolution.agentId];
      setDeathEvent({
        agentId: resolution.agentId,
        triggerCostRatio: r.triggerCostRatio,
        snapshotCoins: gs?.coins ?? 0,
        snapshotLevel: gs?.highestLevel ?? 1,
        snapshotHp: 0,
        canRevive: (gs?.coins ?? 0) >= 50,
        timestamp: new Date().toISOString(),
      });
    }
  }, [resolution, spend]);

  // Pick perk 回调
  const handlePickPerk = useCallback((perkId: PerkId) => {
    pickPerk(perkId);
    setPendingPerkLevel(null);
  }, [pickPerk]);

  // Revive 回调
  const handleRevive = useCallback(() => {
    revive();
    setDeathEvent(null);
  }, [revive]);

  // Restart 回调
  const handleRestart = useCallback(() => {
    restart();
    setDeathEvent(null);
  }, [restart]);

  // ---- 空状态 ----
  if (agents.length === 0) {
    return (
      <div className="max-w-3xl">
        <PageHeader
          title="Agent"
          subtitle="无限画布 + 当前工作 agent 筛选 + 拟人化游戏化 · 数据对应 kanban / worktree 视图"
          icon={<Bot className="text-accent" size={20} />}
          track="F"
        />
        <div className="card text-center py-12" data-testid="agent-view-empty">
          <AlertTriangle size={32} className="text-warn mx-auto mb-3" />
          <div className="text-base font-semibold mb-1">No agent sessions</div>
          <div className="text-xs text-ink-dim">
            请先在 <a href="/agents" className="text-info underline">Agents</a> 启动一个 agent session, 再回到此视图.
          </div>
        </div>
      </div>
    );
  }

  if (!resolution) {
    return (
      <div className="max-w-3xl">
        <PageHeader
          title="Agent"
          subtitle="无限画布 + 当前工作 agent 筛选 + 拟人化游戏化 · 数据对应 kanban / worktree 视图"
          icon={<Bot className="text-accent" size={20} />}
          track="F"
        />
        <div className="card text-center py-12" data-testid="agent-view-empty">
          <AlertTriangle size={32} className="text-warn mx-auto mb-3" />
          <div className="text-base font-semibold mb-1">No resolvable agent</div>
        </div>
      </div>
    );
  }

  const { agent } = resolution;
  return (
    <div className="-mx-6 -mt-5 h-[calc(100vh-3.5rem)] flex flex-col">
      {/* Header */}
      <div className="border-b border-line bg-bg-soft/40 px-6 py-3 flex items-center justify-between gap-4">
        <div className="flex items-center gap-3 min-w-0">
          <div className="text-sm font-semibold shrink-0" data-testid="agent-view-title">Agent</div>
          <div className="text-[10px] text-ink-mute font-mono truncate hidden lg:block">
            {agent.id} · {agent.agent_kind} · {agent.current_step} · {relatedWorkItems.length} task{relatedWorkItems.length === 1 ? "" : "s"}
            {worktree && <> · worktree <span className="text-info">{worktree.branch}</span></>}
          </div>
        </div>
        <div className="flex items-center gap-2 flex-wrap">
          {/* Game HUD (per 拍板, 拟人化游戏化) */}
          <GameHUD
            gameState={gameState}
            onRevive={handleRevive}
            onRestart={handleRestart}
            onPickPerk={() => setPendingPerkLevel(0)}
            pendingPerkChoice={pendingPerkLevel !== null}
          />
          <AgentFilter
            agents={agents}
            selectedId={resolution.agentId}
            auto={resolution.auto}
            onChange={handleAgentChange}
          />
          {/* Spend cost 模拟 (per 拍板, 触发死亡检测) */}
          <button
            data-testid="agent-view-spend-cost"
            onClick={handleSpend}
            disabled={!gameState?.alive}
            className="btn text-[10px] py-1 px-2 disabled:opacity-50"
            title="模拟执行 1 step (cost +$0.1)"
          >
            <Zap size={10} /> Step
          </button>
          <a
            href={`/board?assignee_id=&worktree_id=${agent.worktree_id}`}
            className="btn text-[10px] py-1 px-2 hidden md:inline-flex"
            data-testid="agent-view-jump-board"
            title="在 Kanban Board 视图查看关联 work-items"
          >
            <Maximize2 size={10} /> Kanban
          </a>
        </div>
      </div>

      {/* Canvas */}
      {canvas && (
        <div className="flex-1 relative">
          <AgentCanvasView
            canvas={canvas}
            agent={agent}
            worktree={worktree}
            gameState={gameState}
            onClaim={handleClaim}
          />
        </div>
      )}

      {/* 说明 footer */}
      <div className="border-t border-line bg-bg-soft/40 px-6 py-1.5 text-[10px] text-ink-mute font-mono flex items-center justify-between">
        <span>
          V/H 切换 select/pan · +/- 缩放 · 1 适配 · 双击节点跳详情 · 完成 work-item 点 💰 Claim 升级 · 点 Step 消耗 cost
        </span>
        <span>
          nodes {canvas?.nodes.length ?? 0} · connectors {canvas?.connectors.length ?? 0} · derived {canvas?.derivedAt.slice(11, 19) ?? "—"}
        </span>
      </div>

      {/* Modals (per 拍板) */}
      {pendingPerkLevel !== null && gameState && (
        <PerkPicker
          gameState={gameState}
          choices={getPerkChoices()}
          onPick={handlePickPerk}
          onClose={() => setPendingPerkLevel(null)}
        />
      )}

      {deathEvent && gameState && (
        <DeathModal
          event={deathEvent}
          gameState={gameState}
          onRevive={handleRevive}
          onRestart={handleRestart}
          onClose={() => setDeathEvent(null)}
        />
      )}
    </div>
  );
}
