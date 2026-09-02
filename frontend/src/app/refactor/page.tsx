"use client";

// =====================================================================
// /refactor/page.tsx — Refactor Sweep 重构专项 (per 2026-09-02 10:41 JST 拍板)
// =====================================================================
// 核心职责:
//   1. 顶部项目切换 (per project_id 过滤, 镜像 ProjectsClient 模式)
//   2. 自动 lazy-init: 项目首次访问 -> 初始化 RefactorBoardConfig (5 默认列 + batch_size=5)
//   3. 自动开 Round #1: 项目内 status=done 的 workItems 入 round
//   4. 5 KPI 卡 (per refactor_status 计数)
//   5. 4 操作按钮: Open Next Round / Add Cards / Settings (popover) / History
//   6. 5 列看板 (todo/doing/testing/review/done) + 列自定义
//   7. 底部历史轮次列表
//
// 拍板要点 (per 2026-09-02 10:41 JST):
//   - doing 和 review 中间加 testing (5 态 default, 不可删, 可改可重排)
//   - 列 CRUD 跟 KanbanBoard 1:1 (add/remove/rename/reorder)
//   - 兜底列 todo 不可删, 删其他列时卡归 fallback
//   - 轮次 (round) 走完 -> 自动开下一轮, round_number 累计
//
// 不做 (per 守门 缺标比错标):
//   - 后端 PATCH /refactor-rounds (UI store only, 持久化走 zustand persist)
//   - 多人协作 (applyRemoteChange) — 留 Phase 2
//   - 复杂 WIP / batch 流 (v1: 全卡可见, KPI 计数, 用户自管理节奏)
// =====================================================================

import { useEffect, useMemo, useState, useCallback } from "react";
import { RefreshCw, Plus, ChevronRight, CheckCircle2 } from "lucide-react";
import { clsx } from "clsx";
import { useStore } from "@/lib/store";
import { useTranslation, interpolate } from "@/lib/i18n";
import { PageHeader, SectionTitle } from "@/components/PageHeader";
import { RefactorSweepBoard } from "@/components/refactor/RefactorSweepBoard";
import { RefactorKpiRow } from "@/components/refactor/RefactorKpiRow";
import { RefactorRoundHistory } from "@/components/refactor/RefactorRoundHistory";
import { RefactorSettingsPopover } from "@/components/refactor/RefactorSettingsPopover";
import { AddRefactorCardsDialog } from "@/components/refactor/AddRefactorCardsDialog";
import type {
  Project, WorkItem, RefactorRound, RefactorBoardConfig, RefactorStatus, Uuid,
} from "@/types/ids";

export default function RefactorPage() {
  const { t } = useTranslation();

  // ---- store 订阅 ----
  const projects = useStore((s) => s.projects);
  const workItems = useStore((s) => s.workItems);
  const refactorRounds = useStore((s) => s.refactorRounds);
  const refactorBoardConfigs = useStore((s) => s.refactorBoardConfigs);
  const ensureRefactorBoardConfig = useStore((s) => s.ensureRefactorBoardConfig);
  const addRefactorColumn = useStore((s) => s.addRefactorColumn);
  const removeRefactorColumn = useStore((s) => s.removeRefactorColumn);
  const renameRefactorColumn = useStore((s) => s.renameRefactorColumn);
  const reorderRefactorColumns = useStore((s) => s.reorderRefactorColumns);
  const resetRefactorColumns = useStore((s) => s.resetRefactorColumns);
  const setRefactorBatchSize = useStore((s) => s.setRefactorBatchSize);
  const openRefactorRound = useStore((s) => s.openRefactorRound);
  const startNextRefactorRound = useStore((s) => s.startNextRefactorRound);
  const moveRefactorCard = useStore((s) => s.moveRefactorCard);
  const addRefactorCard = useStore((s) => s.addRefactorCard);
  const mergeRefactorCard = useStore((s) => s.mergeRefactorCard);
  // 给 RefactorSweepBoard 查 workItem.worktree_id 用 (per 2026-09-02 10:50 JST 拍板)
  const worktrees = useStore((s) => s.worktrees);

  // ---- local state ----
  const [selectedProjectId, setSelectedProjectId] = useState<string>(() => projects[0]?.id ?? "");
  const [showAddDialog, setShowAddDialog] = useState(false);

  // ---- 选中项目 + 项目级派生数据 ----
  const selectedProject = useMemo<Project | null>(
    () => projects.find((p) => p.id === selectedProjectId) ?? projects[0] ?? null,
    [projects, selectedProjectId],
  );

  const projectRounds = useMemo(
    () => refactorRounds.filter((r) => r.project_id === selectedProjectId),
    [refactorRounds, selectedProjectId],
  );
  const activeRound = useMemo<RefactorRound | null>(
    () => projectRounds.find((r) => !r.closed_at) ?? null,
    [projectRounds],
  );
  const closedRounds = useMemo(
    () => projectRounds.filter((r) => r.closed_at),
    [projectRounds],
  );

  const projectDoneWIs = useMemo<WorkItem[]>(
    () => workItems.filter((w) => w.project_id === selectedProjectId && w.status === "done"),
    [workItems, selectedProjectId],
  );

  // ---- lazy init: 项目首次访问 -> 初始化 board config ----
  useEffect(() => {
    if (!selectedProjectId) return;
    const cfg = refactorBoardConfigs[selectedProjectId];
    if (!cfg) {
      ensureRefactorBoardConfig(selectedProjectId);
    }
  }, [selectedProjectId, refactorBoardConfigs, ensureRefactorBoardConfig]);

  // ---- lazy init: 项目无 active round 且有 done 任务 -> 自动开 Round #1 ----
  useEffect(() => {
    if (!selectedProjectId) return;
    if (activeRound) return; // 已有 active 不开新
    if (projectDoneWIs.length === 0) return; // 无 done 任务不开
    // 等 config 初始化完成
    if (!refactorBoardConfigs[selectedProjectId]) return;
    // 开新 round
    openRefactorRound(selectedProjectId);
  }, [
    selectedProjectId, activeRound, projectDoneWIs,
    refactorBoardConfigs, openRefactorRound,
  ]);

  const currentConfig: RefactorBoardConfig | null =
    refactorBoardConfigs[selectedProjectId] ?? null;

  // ---- 操作 handlers ----
  const handleOpenNextRound = useCallback(() => {
    if (!selectedProjectId) return;
    const newRoundId = startNextRefactorRound(selectedProjectId);
    if (!newRoundId) {
      // eslint-disable-next-line no-console
      console.warn("[refactor] startNextRefactorRound returned null — current round not all done");
    }
  }, [selectedProjectId, startNextRefactorRound]);

  const handleMoveCard = useCallback(
    (workItemId: string, toStatus: RefactorStatus) => {
      if (!activeRound) return;
      moveRefactorCard(activeRound.id, workItemId, toStatus);
    },
    [activeRound, moveRefactorCard],
  );

  const handleAddCards = useCallback(
    (workItemIds: string[]) => {
      if (!activeRound) return;
      workItemIds.forEach((id) => addRefactorCard(activeRound.id, id));
    },
    [activeRound, addRefactorCard],
  );

  // Merge 单卡 (per 2026-09-02 10:50 JST 拍板)
  //   调 store.mergeRefactorCard, 返回状态码给 console 提示 (后续可接 toast)
  const handleMergeCard = useCallback(
    (workItemId: string) => {
      if (!activeRound) return;
      const result = mergeRefactorCard(activeRound.id, workItemId);
      if (result !== "ok") {
        // eslint-disable-next-line no-console
        console.warn(`[refactor] merge ${workItemId} -> ${result}`);
      }
    },
    [activeRound, mergeRefactorCard],
  );

  // 判断 work_item 是否有关联 worktree (用于 Merge 按钮 title hint 区分)
  const hasWorktree = useCallback(
    (workItemId: string): boolean => {
      const wi = workItems.find((w) => w.id === workItemId);
      if (!wi?.worktree_id) return false;
      return worktrees.some((wt) => wt.id === wi.worktree_id);
    },
    [workItems, worktrees],
  );

  // ---- 派生 UI 状态 ----
  const allDone = activeRound
    ? activeRound.cards.length === 0 || activeRound.cards.every((c) => c.refactor_status === "done")
    : false;
  const alreadyInRound = useMemo(
    () => new Set(activeRound?.cards.map((c) => c.work_item_id) ?? []),
    [activeRound],
  );

  // ---- 渲染 ----
  return (
    <div className="max-w-7xl" data-testid="refactor-page">
      <PageHeader
        title={t.refactor.title}
        subtitle={t.refactor.subtitle}
        icon={<RefreshCw className="text-accent" size={20} />}
        track="F"
        count={
          activeRound
            ? interpolate(t.refactor.roundLabel, { n: activeRound.round_number }) +
              ` · ${activeRound.cards.length} ${t.refactor.totalCards}`
            : projectRounds.length > 0
              ? `${projectRounds.length} rounds closed`
              : undefined
        }
      />

      {/* 项目切换 (multica 风格) */}
      <ProjectSwitcher
        projects={projects}
        selectedId={selectedProjectId}
        onSelect={setSelectedProjectId}
      />

      {/* 主体: 仅当有 active round 时显示看板 */}
      {activeRound && currentConfig ? (
        <>
          {/* KPI 行 */}
          <div className="mb-4">
            <RefactorKpiRow
              columns={currentConfig.columns}
              cards={activeRound.cards}
            />
          </div>

          {/* 操作栏 */}
          <div className="flex flex-wrap items-center justify-between gap-2 mb-4">
            <div className="flex flex-wrap items-center gap-2">
              {allDone ? (
                <button
                  type="button"
                  onClick={handleOpenNextRound}
                  data-testid="refactor-open-next-round"
                  className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-mono font-bold border border-ok/40 bg-ok/10 text-ok hover:bg-ok/20 shadow-[0_0_8px_rgba(16,185,129,0.3)]"
                >
                  <ChevronRight size={12} />
                  {interpolate(t.refactor.openNextRound, { n: (activeRound?.round_number ?? 0) + 1 })}
                </button>
              ) : (
                <span
                  data-testid="refactor-round-progress"
                  className="text-[10px] font-mono text-ink-mute flex items-center gap-1.5"
                >
                  <CheckCircle2 size={11} className="text-ok" />
                  {activeRound.cards.filter((c) => c.refactor_status === "done").length} / {activeRound.cards.length} {t.refactor.finishedCards}
                </span>
              )}
              <button
                type="button"
                onClick={() => setShowAddDialog(true)}
                data-testid="refactor-add-cards"
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-mono font-bold border border-accent/40 bg-accent/10 text-accent hover:bg-accent/20"
              >
                <Plus size={12} />
                {t.refactor.addCards}
              </button>
            </div>
            <div className="flex items-center gap-2">
              <div className="text-[9px] font-mono text-ink-mute">
                {t.refactor.columnsCustomizeHint}
              </div>
              <RefactorSettingsPopover
                batchSize={currentConfig.batch_size}
                onChangeBatchSize={(n) => setRefactorBatchSize(selectedProjectId, n)}
                onResetColumns={() => resetRefactorColumns(selectedProjectId)}
              />
            </div>
          </div>

          {/* 看板主体 */}
          <RefactorSweepBoard
            round={activeRound}
            config={currentConfig}
            onMoveCard={handleMoveCard}
            onAddColumn={(status, name) => addRefactorColumn(selectedProjectId, status, name)}
            onRemoveColumn={(status) => removeRefactorColumn(selectedProjectId, status)}
            onRenameColumn={(status, name) => renameRefactorColumn(selectedProjectId, status, name)}
            onReorderColumns={(from, to) => reorderRefactorColumns(selectedProjectId, from, to)}
            onMergeCard={handleMergeCard}
            hasWorktree={hasWorktree}
          />

          {/* 加卡对话框 */}
          <AddRefactorCardsDialog
            open={showAddDialog}
            onClose={() => setShowAddDialog(false)}
            doneWorkItems={projectDoneWIs}
            alreadyInRound={alreadyInRound}
            onAdd={handleAddCards}
          />
        </>
      ) : (
        // 无 active round 时: 空状态 + 提示
        <div data-testid="refactor-empty" className="card text-center py-12 border-dashed border-line">
          <RefreshCw size={32} className="mx-auto text-ink-mute/40 mb-3" />
          {projectDoneWIs.length === 0 ? (
            <>
              <div className="text-sm font-medium text-ink mb-1">No done work-items yet</div>
              <div className="text-xs text-ink-dim font-mono">{t.refactor.noDoneWorkItems}</div>
            </>
          ) : (
            <>
              <div className="text-sm font-medium text-ink mb-1">
                {projectDoneWIs.length} done work-items ready to refactor
              </div>
              <button
                type="button"
                onClick={() => openRefactorRound(selectedProjectId)}
                data-testid="refactor-start-round"
                className="mt-3 inline-flex items-center gap-1.5 px-4 py-2 rounded-lg text-xs font-mono font-bold border border-accent/40 bg-accent/10 text-accent hover:bg-accent/20"
              >
                <Plus size={12} />
                Start Refactor Round #1
              </button>
            </>
          )}
        </div>
      )}

      {/* 历史轮次 (底部) */}
      {closedRounds.length > 0 && (
        <div className="mt-6">
          <SectionTitle>{t.refactor.historyTitle}</SectionTitle>
          <RefactorRoundHistory rounds={closedRounds} />
        </div>
      )}
    </div>
  );
}

// =====================================================================
// ProjectSwitcher — 顶部项目切换 (multica 风格 chip row, 跟 ProjectsClient 形态一致)
// =====================================================================
function ProjectSwitcher({
  projects, selectedId, onSelect,
}: {
  projects: Project[];
  selectedId: string;
  onSelect: (id: string) => void;
}) {
  if (projects.length === 0) return null;
  return (
    <div
      data-testid="refactor-project-switcher"
      className="flex flex-wrap items-center gap-1.5 mb-5"
    >
      {projects.map((p) => {
        const active = p.id === selectedId;
        return (
          <button
            key={p.id}
            type="button"
            onClick={() => onSelect(p.id)}
            data-testid={`refactor-project-${p.id}`}
            className={clsx(
              "px-3 py-1.5 rounded-lg text-[11px] font-mono font-medium border transition-all",
              active
                ? "border-accent/60 bg-accent/15 text-accent shadow-[0_0_8px_rgba(0,240,255,0.25)]"
                : "border-line bg-bg-soft/40 text-ink-dim hover:border-accent/40 hover:text-ink",
            )}
          >
            <span className="opacity-60 mr-1.5">{p.key}</span>
            {p.name}
          </button>
        );
      })}
    </div>
  );
}
