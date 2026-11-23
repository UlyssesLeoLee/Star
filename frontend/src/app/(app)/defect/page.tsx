/*
@cypher schema=1 source_sha256=placeholder
MERGE (self:File {path:"frontend/src/app/(app)/defect/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/defect/page.tsx::DefectPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher*/
// =====================================================================
// /defect — 智能缺陷管理主页 (per SRS-DEFECT-INTELLIGENCE-001 §4.4)
// 布局: 左 筛选 + 缺陷列表 | 右 详情抽屉 (复用 WorkItemDetailDrawer 布局风格)
// 缺陷以 Bug WorkItem 承载，纳入主导航 Project→Lane→Worktree→Branch
// ====================================================================="use client";

import { useState, useEffect, useMemo } from "react";
import { useStore } from "@/lib/store";
import { useWorktreeGroupApi } from "@/lib/group/groupProjection";
import { useNavStore } from "@/lib/nav/navStore";
import { clsx } from "clsx";
import { Bug, FileText, AlertTriangle, CheckCircle2, RefreshCw, Filter, Plus } from "lucide-react";
import { StatusPill } from "@/components/StatusPill";
import type { GroupWorkItemView } from "@/lib/group/groupProjection";

export interface DefectItem {
  id: string;
  title: string;
  phenomenon: string;
  actual_result?: string;
  expected_result?: string;
  reproduction_steps?: string;
  status: string;
  severity: string;
  priority: string;
  assignee_id?: string;
  reporter_id: string;
  lane_id?: string;
  worktree_id?: string;
  branch_id?: string;
  branch_name?: string;
  fix_version?: string;
  lifecycle_version: number;
  created_at: string;
  updated_at: string;
}

const DEFAULT_FILTER = {
  status: [] as string[],
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
  const [defects, setDefects] = useState<DefectItem[]>([]);
  const [filter, setFilter] = useState<typeof DEFAULT_FILTER>(() => ({ ...DEFAULT_FILTER }));
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedDefect, setSelectedDefect] = useState<DefectItem | null>(null);

  const selectedLaneId = navStore.selectedLaneId;
  const selectedWorktreeId = navStore.selectedWorktreeId;
  const selectedBranch = navStore.selectedBranch;

  // 获取 store 中当前项目下的 worktrees，用于批量拉取该 Lane/Project 下的 Bug WorkItem
  const worktrees = storeApi.worktrees;

  const projectId = navStore.selectedProjectId ?? "";

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);

    const loadDefects = async () => {
      if (!worktreeGroupApi || !projectId) {
        setDefects([]);
        setLoading(false);
        return;
      }

      try {
        // 选中特定 worktree 时只拉取该 worktree；否则拉取 store 中当前项目下所有 worktree 的 Bug
        const targetWorktreeIds = selectedWorktreeId
          ? [selectedWorktreeId]
          : worktrees
              .filter((w) => w.project_id === projectId && (!selectedLaneId || w.lane_id === selectedLaneId))
              .map((w) => w.id);

        const allItems: GroupWorkItemView[] = [];
        await Promise.all(
          targetWorktreeIds.map(async (wtId) => {
            try {
              const items = await worktreeGroupApi.listWorkItems<GroupWorkItemView[]>(wtId);
              allItems.push(...items);
            } catch {
              // 单个 worktree 失败不阻塞整体
            }
          })
        );

        if (!cancelled) {
          const bugs = allItems
            .filter((item) => item.kind === "bug")
            .map((item) => ({
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
              reporter_id: item.reporter_id,
              lane_id: undefined,
              worktree_id: item.worktree_id,
              branch_id: item.branch_id ?? undefined,
              branch_name: item.branch_name ?? undefined,
              fix_version: undefined,
              lifecycle_version: item.lifecycle_version,
              created_at: "",
              updated_at: "",
            }));
          setDefects(bugs);
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
  }, [selectedLaneId, selectedWorktreeId, selectedBranch, worktreeGroupApi, worktrees, projectId]);

  const filtered = useMemo(() => {
    return defects.filter((d) => {
      if (filter.status.length && !filter.status.includes(d.status)) return false;
      if (filter.severity.length && !filter.severity.includes(d.severity)) return false;
      if (filter.priority.length && !filter.priority.includes(d.priority)) return false;
      if (filter.assignee_id && d.assignee_id !== filter.assignee_id) return false;
      if (filter.text && !(d.title + " " + d.phenomenon).toLowerCase().includes(filter.text.toLowerCase()))
        return false;
      return true;
    });
  }, [defects, filter]);

  const handleCreate = () => {
    // 触发创建缺陷（复用 WorkItemDetailDrawer new 模式，kind=Bug）
    setSelectedDefect(null);
    // 由父组件或全局状态打开创建抽屉，此处仅标记意图
    window.dispatchEvent(new CustomEvent("defect:create", { detail: { lane_id: selectedLaneId } }));
  };

  const handleSelect = (d: DefectItem) => {
    setSelectedDefect(d);
  };

  return (
    <div className="flex h-full bg-bg">
      {/* 左: 筛选 + 列表 */}
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
              onChange={(e) => setFilter({ ...filter, status: Array.from(e.target.selectedOptions, (o) => o.value) })}
              className="w-full text-xs bg-bg-soft border border-line rounded px-2 py-1.5 text-ink outline-none focus:border-accent"
            >
              <option value="triage">待分诊</option>
              <option value="confirmed">已确认</option>
              <option value="in_progress">处理中</option>
              <option value="pending_verify">待验证</option>
              <option value="closed">已关闭</option>
              <option value="reopened">重新打开</option>
            </select>
            <select
              multiple
              value={filter.severity}
              onChange={(e) => setFilter({ ...filter, severity: Array.from(e.target.selectedOptions, (o) => o.value) })}
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
            <div className="flex items-center justify-center py-8 text-ink-mute text-xs">加载中...</div>
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
                    selectedDefect?.id === d.id && "bg-accent/10 border-accent/50"
                  )}
                >
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-1">
                      <span className="font-mono font-semibold text-ink truncate">{d.title}</span>
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

      {/* 右: 详情 */}
      <main className="flex-1 min-w-0 overflow-y-auto p-4">
        {selectedDefect ? (
          <div className="max-w-2xl mx-auto">
            <div className="mb-4 flex items-center justify-between">
              <h1 className="text-lg font-bold text-ink">{selectedDefect.title}</h1>
              <StatusPill value={selectedDefect.status} />
            </div>
            <div className="grid grid-cols-2 gap-4 mb-4">
              <div className="rounded-lg border border-line bg-bg-soft p-3">
                <div className="text-[10px] font-mono uppercase tracking-wider text-ink-dim mb-1">现象描述</div>
                <div className="text-xs text-ink whitespace-pre-wrap">{selectedDefect.phenomenon}</div>
              </div>
              <div className="rounded-lg border border-line bg-bg-soft p-3">
                <div className="text-[10px] font-mono uppercase tracking-wider text-ink-dim mb-1">实际结果</div>
                <div className="text-xs text-ink whitespace-pre-wrap">{selectedDefect.actual_result || "（未填写）"}</div>
              </div>
            </div>
            <div className="rounded-lg border border-line bg-bg-card p-3 mb-4">
              <div className="flex items-center gap-2 mb-2">
                <FileText size={14} className="text-accent" />
                <span className="text-sm font-bold text-ink">修复与验证</span>
              </div>
              <div className="text-xs text-ink-mute">
                修复版本: {selectedDefect.fix_version || "（未设定）"}<br />
                验证状态: {selectedDefect.status === "pending_verify" ? "待验证" : selectedDefect.status === "closed" ? "已验证" : "（未提交）"}
              </div>
            </div>
            <button
              type="button"
              onClick={() => setSelectedDefect(null)}
              className="text-xs text-accent hover:underline"
            >
              ← 返回列表
            </button>
          </div>
        ) : (
          <div className="flex h-full items-center justify-center text-ink-mute text-xs">
            请从左侧列表选择一条缺陷查看详情
          </div>
        )}
      </main>
    </div>
  );
}
