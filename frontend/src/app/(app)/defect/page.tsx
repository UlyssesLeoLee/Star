/*
@cypher schema=1 source_sha256=placeholder
MERGE (self:File {path:"frontend/src/app/(app)/defect/page.tsx"})
MERGE (page:Function {id:"frontend/src/app/(app)/defect/page.tsx::DefectPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
MERGE (self)-[:USES]->(:Function {id:"frontend/src/components/board/WorkItemDetailDrawer.tsx::WorkItemDetailDrawer"})
@endcypher
*/
// =====================================================================
// /defect — 智能缺陷管理主页 (per SRS-DEFECT-INTELLIGENCE-001 §4.4)
// 布局: 左 筛选 + 缺陷列表 | 右 统一任务详情抽屉 (WorkItemDetailDrawer) + 缺陷扩展字段
// 缺陷以 kind="bug" 的 WorkItem 承载，整合进 Lane 的 Backlog/Sprint，状态用看板 6 列
// 缺陷专用扩展字段 (现象/实际结果/修复版本/验证状态) 作为扩展区块在抽屉下方展示
// =====================================================================
"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { useStore } from "@/lib/store";
import { getSession } from "@/lib/auth";
import { useWorktreeGroupApi } from "@/lib/group/groupProjection";
import { useNavStore } from "@/lib/nav/navStore";
import { clsx } from "clsx";
import {
  Bug,
  FileText,
  AlertTriangle,
  CheckCircle2,
  RefreshCw,
  Filter,
  Plus,
} from "lucide-react";
import { StatusPill } from "@/components/StatusPill";
import {
  WorkItemDetailDrawer,
  type WorkItemDrawerMode,
} from "@/components/board/WorkItemDetailDrawer";
import type { GroupWorkItemView } from "@/lib/group/groupProjection";
import type { WorkItemStatus } from "@/types/ids";

// 看板 6 列状态 (与 Lane/KanbanBoard 对齐，缺陷整合进 Lane)
const KANBAN_STATUSES: WorkItemStatus[] = [
  "todo",
  "in_progress",
  "review",
  "blocked",
  "done",
  "wontfix",
];

export interface DefectItem {
  id: string;
  title: string;
  phenomenon: string; // 缺陷现象 (缺陷扩展字段)
  actual_result?: string; // 实际结果 (缺陷扩展字段)
  expected_result?: string; // 预期结果 (缺陷扩展字段)
  reproduction_steps?: string; // 复现步骤 (缺陷扩展字段)
  status: WorkItemStatus; // 看板 6 列状态 (与 WorkItem 对齐)
  severity: string; // 缺陷严重度 (缺陷扩展字段)
  priority: string;
  assignee_id?: string;
  reporter_id: string;
  lane_id?: string;
  worktree_id?: string;
  branch_id?: string;
  branch_name?: string;
  fix_version?: string; // 修复版本 (缺陷扩展字段)
  lifecycle_version: number;
  created_at: string;
  updated_at: string;
}

const DEFAULT_FILTER = {
  status: [] as WorkItemStatus[],
  severity: [] as string[],
  priority: [] as string[],
  assignee_id: undefined as string | undefined,
  lane_id: undefined as string | undefined,
  worktree_id: undefined as string | undefined,
  text: "" as string,
};

export default function DefectPage() {
  const storeApi = useStore();
  const worktreeGroupApi = useWorktreeGroupApi();
  const navStore = useNavStore();
  const session = getSession();
  const tenantId = session?.payload.tenant_id ?? "";
  const reporterId = session?.payload.user_id ?? "";

  const [defects, setDefects] = useState<DefectItem[]>([]);
  const [filter, setFilter] = useState<typeof DEFAULT_FILTER>(() => ({
    ...DEFAULT_FILTER,
  }));
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedDefect, setSelectedDefect] = useState<DefectItem | null>(null);
  // 抽屉打开模式 (null=关闭; new=新建缺陷; view=查看缺陷详情)
  const [drawerOpen, setDrawerOpen] = useState<WorkItemDrawerMode | null>(null);

  const selectedLaneId = navStore.selectedLaneId;
  const selectedWorktreeId = navStore.selectedWorktreeId;
  const projectId = navStore.selectedProjectId ?? "";
  const worktrees = storeApi.worktrees;
  const addWorkItem = storeApi.addWorkItem;

  // 加载该项目/lane 下的 Bug WorkItem，并 waterfall 到 store (供 WorkItemDetailDrawer view 模式读取 currentWi)
  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);

    const loadDefects = async () => {
      if (!worktreeGroupApi || !projectId || !tenantId || !reporterId) {
        setDefects([]);
        setLoading(false);
        return;
      }

      try {
        const targetWorktreeIds = selectedWorktreeId
          ? [selectedWorktreeId]
          : worktrees
              .filter(
                (w) =>
                  w.project_id === projectId &&
                  (!selectedLaneId || w.lane_id === selectedLaneId),
              )
              .map((w) => w.id);

        const allItems: GroupWorkItemView[] = [];
        await Promise.all(
          targetWorktreeIds.map(async (wtId) => {
            try {
              const items = await worktreeGroupApi.listWorkItems<GroupWorkItemView[]>(
                wtId,
              );
              allItems.push(...items);
            } catch {
              // 单个 worktree 失败不阻塞整体
            }
          }),
        );

        if (!cancelled) {
          const bugs = allItems.filter((item) => item.kind === "bug");

          // waterfall 到 store: 使 WorkItemDetailDrawer view 模式能读到 currentWi
          for (const item of bugs) {
            try {
              addWorkItem({
                id: item.id,
                tenant_id: tenantId,
                project_id: projectId,
                title: item.title,
                description: item.description,
                kind: "bug" as const,
                status: item.status,
                priority: item.priority,
                reporter_id: item.reporter_id ?? reporterId,
                labels: item.labels,
                worktree_id: item.worktree_id,
                branch_id: item.branch_id ?? undefined,
                branch_name: item.branch_name ?? undefined,
                lifecycle_version: item.lifecycle_version,
              });
            } catch {
              // 水合单条失败不阻塞整体
            }
          }

          const defectItems: DefectItem[] = bugs.map((item) => ({
            id: item.id,
            title: item.title,
            phenomenon: item.description,
            actual_result: undefined,
            expected_result: undefined,
            reproduction_steps: undefined,
            status: item.status,
            severity: "unknown",
            priority: item.priority,
            assignee_id: undefined,
            reporter_id: item.reporter_id ?? reporterId,
            lane_id: undefined,
            worktree_id: item.worktree_id,
            branch_id: item.branch_id ?? undefined,
            branch_name: item.branch_name ?? undefined,
            fix_version: undefined,
            lifecycle_version: item.lifecycle_version,
            created_at: "",
            updated_at: "",
          }));
          setDefects(defectItems);
        }
      } catch (e: unknown) {
        if (!cancelled) setError(e instanceof Error ? e.message : String(e));
      } finally {
        if (!cancelled) setLoading(false);
      }
    };

    loadDefects();
    return () => {
      cancelled = true;
    };
  }, [
    selectedLaneId,
    selectedWorktreeId,
    worktreeGroupApi,
    worktrees,
    projectId,
    tenantId,
    reporterId,
    addWorkItem,
  ]);

  const filtered = useMemo(() => {
    return defects.filter((d) => {
      if (filter.status.length && !filter.status.includes(d.status))
        return false;
      if (filter.severity.length && !filter.severity.includes(d.severity))
        return false;
      if (filter.priority.length && !filter.priority.includes(d.priority))
        return false;
      if (filter.assignee_id && d.assignee_id !== filter.assignee_id)
        return false;
      if (
        filter.text &&
        !(d.title + " " + d.phenomenon)
          .toLowerCase()
          .includes(filter.text.toLowerCase())
      )
        return false;
      return true;
    });
  }, [defects, filter]);

  // ---- 新建缺陷：打开 drawer new 模式，kind 锁定为 bug ----
  const handleCreate = useCallback(() => {
    setSelectedDefect(null);
    setDrawerOpen({ kind: "new", defaultStatus: "todo" });
  }, []);

  // ---- 选中缺陷：打开 drawer view 模式 ----
  const handleSelect = useCallback((d: DefectItem) => {
    setSelectedDefect(d);
    setDrawerOpen({ kind: "view", workItemId: d.id });
  }, []);

  // ---- 抽屉关闭后清空选中 ----
  const handleDrawerClose = useCallback(() => {
    setDrawerOpen(null);
    setSelectedDefect(null);
  }, []);

  return (
    <div className="flex h-full bg-bg">
      {/* 左: 筛选 + 缺陷列表 */}
      <aside className="w-full lg:w-3/5 shrink-0 border-r border-line bg-bg-card overflow-y-auto p-4">
        <div className="flex items-center justify-between px-3 py-3 border-b border-line">
          <h2 className="text-sm font-bold text-ink">缺陷管理</h2>
          <button
            type="button"
            onClick={handleCreate}
            disabled={loading}
            data-testid="defect-create-btn"
            className="flex items-center gap-1 px-3 py-1.5 rounded-lg bg-accent text-white text-xs hover:bg-accent/90 disabled:opacity-50"
          >
            <Plus size={12} /> 新建缺陷
          </button>
        </div>

        {/* 筛选条 */}
        <div className="px-3 py-2 space-y-2 border-b border-line">
          <div className="flex items-center gap-2">
            <Filter size={14} className="text-ink-dim" />
            <span className="text-[10px] font-mono text-ink-mute">筛选</span>
          </div>
          <div className="grid grid-cols-2 gap-2">
            <select
              multiple
              value={filter.status}
              onChange={(e) =>
                setFilter({
                  ...filter,
                  status: Array.from(
                    e.target.selectedOptions,
                    (o) => o.value,
                  ) as WorkItemStatus[],
                })
              }
              className="w-full text-xs bg-bg-soft border border-line rounded px-2 py-1.5 text-ink outline-none focus:border-accent"
            >
              <option value="todo">待办</option>
              <option value="in_progress">处理中</option>
              <option value="review">评审</option>
              <option value="blocked">阻塞</option>
              <option value="done">已完成</option>
              <option value="wontfix">不处理</option>
            </select>
            <select
              multiple
              value={filter.severity}
              onChange={(e) =>
                setFilter({
                  ...filter,
                  severity: Array.from(
                    e.target.selectedOptions,
                    (o) => o.value,
                  ),
                })
              }
              className="w-full text-xs bg-bg-soft border border-line rounded px-2 py-1.5 text-ink outline-none focus:border-accent"
            >
              <option value="S1">S1 严重</option>
              <option value="S2">S2 高</option>
              <option value="S3">S3 中</option>
              <option value="S4">S4 低</option>
              <option value="unknown">未确认</option>
            </select>
          </div>
          <div className="flex items-center gap-2">
            <input
              type="text"
              value={filter.text}
              onChange={(e) => setFilter({ ...filter, text: e.target.value })}
              placeholder="搜索标题/现象..."
              className="w-full text-xs bg-bg-soft border border-line rounded px-2 py-1.5 text-ink outline-none focus:border-accent"
            />
          </div>
        </div>

        {/* 缺陷列表 */}
        <div className="px-3 py-2">
          {loading ? (
            <div className="flex items-center justify-center py-8 text-ink-mute text-xs">
              加载中...
            </div>
          ) : error ? (
            <div className="rounded border border-err/40 bg-err/10 px-3 py-2 text-xs text-err">
              加载失败: {error}
            </div>
          ) : filtered.length === 0 ? (
            <div className="flex items-center justify-center py-8 text-ink-mute text-xs">
              暂无缺陷
            </div>
          ) : (
            <ul className="space-y-1">
              {filtered.map((d) => (
                <li
                  key={d.id}
                  onClick={() => handleSelect(d)}
                  className={clsx(
                    "flex items-center gap-2 px-3 py-2 rounded-lg border border-line bg-bg-card text-xs cursor-pointer hover:bg-accent/5",
                    selectedDefect?.id === d.id && "bg-accent/10 border-accent/50",
                  )}
                >
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-1">
                      <span className="font-mono font-semibold text-ink truncate">
                        {d.title}
                      </span>
                      <StatusPill value={d.status} size="xs" />
                    </div>
                    <div className="flex items-center gap-1.5 text-[10px] text-ink-mute mt-0.5">
                      <span>{d.severity}</span>
                      <span>·</span>
                      <span>{d.priority}</span>
                      <span>·</span>
                      <span>{d.branch_name || "未绑定"}</span>
                    </div>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>
      </aside>

      {/* 右: 统一任务详情抽屉 + 缺陷扩展字段 */}
      <main className="flex-1 min-w-0 overflow-y-auto p-4">
        {/* 统一任务卡抽屉 (缺陷详情通用字段: 标题/状态/优先级/类型/指派/工作tree/标签) */}
        <WorkItemDetailDrawer
          open={drawerOpen}
          onClose={handleDrawerClose}
          projectId={projectId || undefined}
          tenantId={tenantId || undefined}
          reporterId={reporterId || undefined}
          defaultKind="bug"
          lockKind={true}
        />

        {/* 缺陷扩展字段 (现象/实际结果/修复版本/验证状态)，仅在选中缺陷时展示 */}
        {selectedDefect && (
          <div className="max-w-2xl mx-auto mt-6 space-y-4">
            <div className="rounded-lg border border-line bg-bg-card p-3">
              <div className="flex items-center gap-2 mb-2">
                <FileText size={14} className="text-accent" />
                <span className="text-sm font-bold text-ink">缺陷扩展详情</span>
              </div>
              <div className="grid grid-cols-2 gap-4 mb-4">
                <div className="rounded-lg border border-line bg-bg-soft p-3">
                  <div className="text-[10px] font-mono uppercase tracking-wider text-ink-dim mb-1">
                    现象描述
                  </div>
                  <div className="text-xs text-ink whitespace-pre-wrap">
                    {selectedDefect.phenomenon}
                  </div>
                </div>
                <div className="rounded-lg border border-line bg-bg-soft p-3">
                  <div className="text-[10px] font-mono uppercase tracking-wider text-ink-dim mb-1">
                    实际结果
                  </div>
                  <div className="text-xs text-ink whitespace-pre-wrap">
                    {selectedDefect.actual_result || "（未填写）"}
                  </div>
                </div>
              </div>
              <div className="rounded-lg border border-line bg-bg-soft p-3">
                <div className="flex items-center gap-2 mb-2">
                  <CheckCircle2 size={14} className="text-accent" />
                  <span className="text-sm font-bold text-ink">修复与验证</span>
                </div>
                <div className="text-xs text-ink-mute">
                  修复版本: {selectedDefect.fix_version || "（未设定）"}<br />
                  验证状态:{selectedDefect.status === "done" ? "已验证" : "（未提交）"}{" "}
                  · 看板状态:
                  <StatusPill value={selectedDefect.status} size="xs" />
                </div>
              </div>
              <button
                type="button"
                onClick={() => {
                  setSelectedDefect(null);
                  setDrawerOpen(null);
                }}
                className="text-xs text-accent hover:underline mt-3"
              >
                ← 返回列表
              </button>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}
