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

import { useMemo, useCallback, useEffect, useState, Suspense } from "react";
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
import { RoguelikeCanvas } from "@/components/agent-game/RoguelikeCanvas";
import { AgentSettingsTab } from "@/components/agent-game/AgentSettingsTab";
import { useAgentGame } from "@/components/agent-game/useAgentGame";
import { getPerkChoices } from "@/lib/agent-game/perks";
import { PageHeader, SectionTitle } from "@/components/PageHeader";
import { GasParticlesHint } from "@/components/effects/GasParticlesHint";
import { Bot, AlertTriangle, Maximize2, Zap, Sparkles, Map, RefreshCw, Settings } from "lucide-react";
import type { PerkId } from "@/lib/agent-game/types";
import { useTranslation } from "@/lib/i18n";
import { AnimeCelShaderCanvas, CelPalette } from "@/components/effects/AnimeCelShaderCanvas";
import {
  CelButton3D,
  CelToggle3D,
  CelBeacon3D,
} from "@/components/effects/Cel3DUI";

type ViewMode = "canvas" | "roguelike" | "settings" | "core3d";

function AgentViewContent() {
  const { t } = useTranslation();
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  // store (顶层订阅, 避免 hooks 违规)
  const agents = useStore((s) => s.agentSessions);
  const worktrees = useStore((s) => s.worktrees);
  const workItems = useStore((s) => s.workItems);
  const initAgentGame = useStore((s) => s.initAgentGame);
  const generateAgentMap = useStore((s) => s.generateAgentMap);
  const moveAgentOnMap = useStore((s) => s.moveAgentOnMap);
  const resetAgentMap = useStore((s) => s.resetAgentMap);
  const agentMaps = useStore((s) => s.agentMaps);
  const agentPositions = useStore((s) => s.agentPositions);

  // URL 参数: ?agent=ag-XXX
  const urlAgentId = searchParams.get("agent");
  const urlView = searchParams.get("view") as ViewMode | null;

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

  // View 模式 (URL 持久化, 默认 Canvas v1)
  const [viewMode, setViewMode] = useState<ViewMode>(
    urlView === "roguelike" ? "roguelike" : urlView === "settings" ? "settings" : "canvas",
  );

  // 3渲2 Cel Shader Live Parameters
  const [celPalette, setCelPalette] = useState<CelPalette>("crimson");
  const [celBands, setCelBands] = useState<number>(3);
  const [autoSagaGuard, setAutoSagaGuard] = useState(true);

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

  // 首次访问某 agent (roguelike 模式): 自动生成 map
  useEffect(() => {
    if (resolution && viewMode === "roguelike" && !agentMaps[resolution.agentId]) {
      const seed = (Date.now() ^ resolution.agentId.charCodeAt(0) * 31) % 0x7fffffff;
      generateAgentMap(resolution.agentId, 8, 6, Math.abs(seed));
    }
  }, [resolution, viewMode, agentMaps, generateAgentMap]);

  // 切换 agent 时, 清 pendingPerk + death (避免 stale)
  useEffect(() => {
    setPendingPerkLevel(null);
    setDeathEvent(null);
  }, [resolution?.agentId]);

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

  // View 模式切换
  const handleViewModeChange = useCallback(
    (mode: ViewMode) => {
      setViewMode(mode);
      const params = new URLSearchParams(searchParams.toString());
      if (mode === "canvas") {
        params.delete("view");
      } else {
        params.set("view", mode);
      }
      router.replace(`${pathname}?${params.toString()}`, { scroll: false });
    },
    [router, pathname, searchParams],
  );

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

  // Restart 回调 (死亡 = all agents freeze, 玩家主动重开)
  const handleRestart = useCallback(() => {
    restart();
    if (resolution) {
      resetAgentMap(resolution.agentId);  // 重生 map, agent 回到起点
    }
    setDeathEvent(null);
  }, [restart, resetAgentMap, resolution]);

  // Roguelike 移动
  const handleRoguelikeMove = useCallback((target: { x: number; y: number }) => {
    if (!resolution) return;
    const r = moveAgentOnMap(resolution.agentId, target);
    if (!r.ok) return;
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
    // 走到 enemy cell → 触发 claim (合并 Roguelike + 拟人化游戏化)
    if (r.effect.kind === "enemy" && r.effect.workItemId) {
      const claimR = claim(r.effect.workItemId);
      if (claimR && claimR.ok && claimR.leveledUp) {
        setPendingPerkLevel(claimR.levelsGained);
      }
    }
  }, [resolution, moveAgentOnMap, claim]);

  // Roguelike reset map
  const handleRoguelikeReset = useCallback(() => {
    if (resolution) {
      resetAgentMap(resolution.agentId);
    }
  }, [resolution, resetAgentMap]);

  // ---- 空状态 ----
  if (agents.length === 0) {
    return (
      <div className="max-w-3xl">
        <PageHeader
          title={t.pageTitles['/agent-view'].title}
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
          title={t.pageTitles['/agent-view'].title}
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
    <div className="w-full h-[calc(100vh-5rem)] flex flex-col">
      {/* Header */}
      <div className="border-b border-line bg-bg-soft/40 px-6 py-3 flex items-center justify-between gap-4">
        <div className="flex items-center gap-3 min-w-0">
          <div className="text-sm font-semibold shrink-0" data-testid="agent-view-title">Agent</div>
          <div className="text-xs text-ink-mute font-mono truncate hidden lg:block">
            {agent.id} · {agent.agent_kind} · {agent.current_step} · {relatedWorkItems.length} task{relatedWorkItems.length === 1 ? "" : "s"}
            {worktree && <> · worktree <span className="text-info font-bold">{worktree.branch}</span></>}
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
          <span className="relative inline-block">
            <button
              data-testid="agent-view-spend-cost"
              onClick={handleSpend}
              disabled={!gameState?.alive}
              className="btn text-xs py-1.5 px-3 min-h-[34px] disabled:opacity-50 relative"
              title="模拟执行 1 step (cost +$0.1)"
            >
              <Zap size={12} /> Step
            </button>
            {/* 气态粒子提示 (per 2026-09-05 拍板, 场景 4: agent 下一步 step 按钮) */}
            <GasParticlesHint
              variant="rise"
              color="ok"
              density={0.45}
              width={80}
              height={60}
              offsetX={-4}
              offsetY={-22}
            />
          </span>
          <a
            href={`/board?assignee_id=&worktree_id=${agent.worktree_id}`}
            className="btn text-xs py-1.5 px-3 min-h-[34px] hidden md:inline-flex"
            data-testid="agent-view-jump-board"
            title="在 Kanban Board 视图查看关联 work-items"
          >
            <Maximize2 size={12} /> Kanban
          </a>
        </div>
      </div>

      {/* View Mode Tab (per 2026-09-05 23:00 JST 拍板: Canvas v1 / Roguelike v2 / 3D 战术核心 v3 / Agent 设置) */}
      <div className="border-b-2 border-black bg-[var(--cel-surface-card,#0f1422)] px-6 py-2.5 flex items-center gap-2.5 cel-shadow" data-testid="view-mode-tabs">
        <button
          data-testid="view-mode-core3d"
          onClick={() => handleViewModeChange("core3d")}
          className={`text-sm px-4 py-1.5 font-mono font-bold border-2 border-black transition-all flex items-center gap-1.5 cel-shadow ${viewMode === "core3d" ? "bg-[var(--cel-crimson,#ff184c)] text-black" : "bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-text-secondary,#94a3b8)] hover:text-white"}`}
        >
          <Zap size={13} className="text-[var(--cel-cyan,#00f0ff)]" /> 3D 战术核心 <span className="text-[10px] px-1.5 py-0.5 bg-black text-[var(--cel-cyan,#00f0ff)] border border-black font-mono">v3 CEL</span>
        </button>
        <button
          data-testid="view-mode-canvas"
          onClick={() => handleViewModeChange("canvas")}
          className={`text-sm px-4 py-1.5 font-mono font-bold border-2 border-black transition-all flex items-center gap-1.5 cel-shadow ${viewMode === "canvas" ? "bg-[var(--cel-cyan,#00f0ff)] text-black" : "bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-text-secondary,#94a3b8)] hover:text-white"}`}
        >
          <Sparkles size={13} /> Canvas <span className="text-[10px] px-1.5 py-0.5 bg-black text-[var(--cel-gold,#ffc400)] border border-black font-mono">v1</span>
        </button>
        <button
          data-testid="view-mode-roguelike"
          onClick={() => handleViewModeChange("roguelike")}
          className={`text-sm px-4 py-1.5 font-mono font-bold border-2 border-black transition-all flex items-center gap-1.5 cel-shadow ${viewMode === "roguelike" ? "bg-[var(--cel-gold,#ffc400)] text-black" : "bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-text-secondary,#94a3b8)] hover:text-white"}`}
        >
          <Map size={13} /> Roguelike <span className="text-[10px] px-1.5 py-0.5 bg-black text-[var(--cel-gold,#ffc400)] border border-black font-mono">v2</span>
        </button>
        <button
          data-testid="view-mode-settings"
          onClick={() => handleViewModeChange("settings")}
          className={`text-sm px-4 py-1.5 font-mono font-bold border-2 border-black transition-all flex items-center gap-1.5 cel-shadow ${viewMode === "settings" ? "bg-[var(--cel-text-primary,#ffffff)] text-black" : "bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-text-secondary,#94a3b8)] hover:text-white"}`}
        >
          <Settings size={13} /> Agent 设置
        </button>
      </div>

      {/* Content (按 viewMode 切换) */}
      {viewMode === "core3d" ? (
        <div className="flex-1 overflow-y-auto p-6 max-w-7xl mx-auto w-full grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
          {/* Left Column: S-Class 3D NPR Cel-Shaded Terminal (7 cols) */}
          <div className="lg:col-span-7 space-y-4">
            <div className="card relative overflow-hidden">
              <div className="flex items-center justify-between border-b-2 border-black pb-2.5 mb-3">
                <div>
                  <div className="text-[11px] font-black text-[var(--cel-gold,#ffc400)] uppercase tracking-widest font-mono">
                    LIVE NPR SHADER // S-CLASS
                  </div>
                  <h3 className="text-base font-black uppercase italic tracking-wider text-[var(--cel-text-primary,#ffffff)] flex items-center gap-2">
                    {agent.name} 3D AVATAR
                    <span className="text-xs font-black not-italic px-2 py-0.5 bg-[var(--cel-crimson,#ff184c)] text-black border border-black">
                      神格
                    </span>
                  </h3>
                </div>
                <span className="text-xs font-mono font-bold text-[var(--cel-text-secondary,#94a3b8)]">
                  〔戦術司令機〕
                </span>
              </div>

              {/* Real 3D WebGL Canvas */}
              <div className="relative w-full h-80 sm:h-96 bg-[var(--cel-surface-stage,#090d16)] border-2 border-black overflow-hidden flex items-center justify-center group cel-shadow">
                <div className="absolute inset-0 bg-screentone-dense opacity-20 pointer-events-none" />
                <AnimeCelShaderCanvas
                  palette={celPalette}
                  bands={celBands}
                  outlineThickness={0.05}
                  enableHalftone={true}
                  enableRim={true}
                  speed={1.0}
                  className="w-full h-full"
                />
                <div className="absolute top-2.5 left-2.5 flex items-center gap-1.5 z-20 pointer-events-none">
                  <span className="size-2 rounded-full bg-[var(--cel-cyan,#00f0ff)] animate-ping" />
                  <span className="bg-black/85 border border-[var(--cel-cyan,#00f0ff)]/50 text-[var(--cel-cyan,#00f0ff)] px-2.5 py-1 text-[11px] font-mono font-bold">
                    GLSL_NPR_3D
                  </span>
                </div>
                <div className="absolute top-2.5 right-2.5 z-20 pointer-events-none">
                  <span className="bg-black/85 border border-[var(--cel-gold,#ffc400)]/50 text-[var(--cel-gold,#ffc400)] px-2.5 py-1 text-[11px] font-mono font-bold">
                    {celBands}-BAND CEL
                  </span>
                </div>
              </div>

              {/* Palette & Bands */}
              <div className="mt-3 bg-[var(--cel-surface-stage,#090d16)] border-2 border-black p-3 flex items-center justify-between text-xs font-mono">
                <div className="flex items-center gap-2">
                  <span className="text-[11px] font-bold text-[var(--cel-text-secondary,#94a3b8)] uppercase">PALETTE:</span>
                  <div className="flex gap-1.5">
                    {(["crimson", "cyan", "gold", "stealth"] as CelPalette[]).map((p) => (
                      <button
                        key={p}
                        onClick={() => setCelPalette(p)}
                        className={`px-2.5 py-1 text-[11px] font-bold uppercase border border-black ${celPalette === p ? "bg-[var(--cel-crimson,#ff184c)] text-black border-white" : "bg-[var(--cel-surface-sub,#151c2c)] text-slate-300"}`}
                      >
                        {p}
                      </button>
                    ))}
                  </div>
                </div>
                <div className="flex items-center gap-1.5">
                  <span className="text-[11px] text-[var(--cel-text-secondary,#94a3b8)]">BANDS:</span>
                  {[2, 3, 4].map((b) => (
                    <button
                      key={b}
                      onClick={() => setCelBands(b)}
                      className={`w-6 h-6 text-[11px] font-bold border border-black ${celBands === b ? "bg-[var(--cel-cyan,#00f0ff)] text-black" : "bg-[var(--cel-surface-sub,#151c2c)] text-slate-400"}`}
                    >
                      {b}
                    </button>
                  ))}
                </div>
              </div>
            </div>
          </div>

          {/* Right Column: Protocols & Actions (5 cols) */}
          <div className="lg:col-span-5 space-y-4">
            <div className="card">
              <SectionTitle>Tactical Protocols 〔戦術プロトコル〕</SectionTitle>
              <div className="space-y-3 mt-3">
                <div className="border-2 border-black p-3 bg-[var(--cel-surface-sub,#151c2c)] cel-shadow">
                  <CelToggle3D
                    label="AUTONOMOUS SAGA GUARD"
                    sublabel="IDEMPOTENT RECOVERY"
                    checked={autoSagaGuard}
                    onChange={setAutoSagaGuard}
                  />
                </div>
                <div className="pt-2 border-t-2 border-black flex flex-col gap-2 items-center">
                  <CelButton3D
                    label="01 // SYNC WORKTREE"
                    sublabel="IDEMPOTENT MERGE"
                    variant="cyan"
                    onClick={() => alert("【3D 战术派发】工作树同步成功：0 冲突。")}
                  />
                  <CelButton3D
                    label="02 // EXECUTE STEP"
                    sublabel="DISPATCH RUNTIME"
                    variant="gold"
                    onClick={handleSpend}
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
      ) : viewMode === "canvas" ? (
        <>
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
        </>
      ) : viewMode === "roguelike" ? (
        <>
          {(() => {
            const map = agentMaps[resolution.agentId];
            const pos = agentPositions[resolution.agentId];
            if (!map || !pos) {
              return (
                <div className="flex-1 flex items-center justify-center text-ink-mute text-xs">
                  <RefreshCw size={12} className="animate-spin mr-1" /> 生成 Roguelike map 中...
                </div>
              );
            }
            return (
              <RoguelikeCanvas
                map={map}
                position={pos}
                agent={agent}
                workItems={workItems}
                onMove={handleRoguelikeMove}
                onReset={handleRoguelikeReset}
                canMove={gameState?.alive ?? false}
                agentLevel={gameState?.level ?? 1}
              />
            );
          })()}
        </>
      ) : (
        <div className="flex-1 min-h-0">
          <AgentSettingsTab initialAgentId={resolution.agentId} />
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

export default function AgentViewPage() {
  return (
    <Suspense fallback={<div className="p-8 text-center text-sm font-mono text-ink-dim">Loading Tactical Agent View...</div>}>
      <AgentViewContent />
    </Suspense>
  );
}
