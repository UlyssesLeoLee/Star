"use client";

// =====================================================================
// WorkItemDetailDrawer — Kanban 卡右侧抽屉详情 (per 2026-08-31 12:07 JST Ulysses 拍板)
// =====================================================================
// 触发 (per 拍板: "参考 Jira 和 multica.ai"):
//   - Kanban 列底部 +Add task 按钮  → mode="new"   (status = 点击的列)
//   - KanbanCard 整张点击            → mode="view"  (status = wi.status)
//
// 模式:
//   - "new"  : title 必填, Status 锁定为触发列, 其他字段可选, 底部 "Create" 按钮
//   - "view" : 所有字段 inline 编辑, 底部 "Save" 持久化 (走 store.updateWorkItemField)
//
// 字段 (per Star 设计 + Multica slide-over + Jira issue detail):
//   1. Title        inline 编辑 (autoFocus in new mode)
//   2. Status       6 选 dropdown (todo/in_progress/review/blocked/done/wontfix)
//   3. Priority     4 选 (p0/p1/p2/p3)
//   4. Kind         5 选 (story/task/bug/spike/epic)
//   5. Assignee     Identity (人) / Agent (AgentSession) / Squad (Phase 2 占位)
//                   per 拍板: "Identity + Agent + Squad 预留"
//   6. Worktree     下拉选 (Phase 2 跳 wt 详情; 本轮只展示 17 状态机 StatusPill)
//   7. Description  textarea (Markdown stub, Phase 2+ 接 MD 编辑器)
//   8. Labels       chip input, Enter 加, 已有 chip 可删
//
// 守门:
//   - 不引新依赖 (复用 StatusPill / useStore / clsx)
//   - 走 portal 挂 body, ESC + 背景 click 关闭
//   - focus trap 简版: open 时 autoFocus title input
//   - drawer 内部所有 update 走 store, 不绕开
//
// 已知缺口 (per 缺标比错标):
//   - Markdown 编辑器是 textarea stub (Phase 2+)
//   - Comments timeline 是 mock list (Phase 2+ 接 store.comments)
//   - Squad 选项 Phase 2 接入; 本轮 0 项
//   - reviewer_id 字段本轮不加 (Phase 2 接 manager 审核流)
//   - SSR-safe: portal 只在 client 端挂载 (typeof window check)
// =====================================================================

import { useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { clsx } from "clsx";
import {
  X, Trash2, Plus, Flag, Hash, Tag, FileText, User, Bot, Users,
  GitBranch, AlertTriangle, Save, ChevronDown,
} from "lucide-react";
import { StatusPill } from "@/components/StatusPill";
import { useStore } from "@/lib/store";
import type {
  WorkItem, WorkItemStatus, WorkItemKind, WorkItemPriority,
  Identity, AgentSession, Worktree,
} from "@/types/ids";

// ---- Mode discriminator ----
export type WorkItemDrawerMode =
  | { kind: "new";  defaultStatus: WorkItemStatus }
  | { kind: "view"; workItemId: string };

export interface WorkItemDetailDrawerProps {
  open: WorkItemDrawerMode | null;
  onClose: () => void;
  /** 父组件传 project 上下文 (new 模式必须) */
  projectId?: string;
  projectKey?: string;
  tenantId?: string;
  reporterId?: string;
}

// ---- Static enums (跟 types/ids.ts 同步) ----
const STATUSES: WorkItemStatus[] = [
  "todo", "in_progress", "review", "blocked", "done", "wontfix",
];
const PRIORITIES: WorkItemPriority[] = ["p0", "p1", "p2", "p3"];
const KINDS: WorkItemKind[] = ["story", "task", "bug", "spike", "epic"];

// ---- Drawer width (per Jira/Multica 一致, 480px 偏窄便于 board 仍可见) ----
const DRAWER_W = 480;

export function WorkItemDetailDrawer({
  open, onClose, projectId, projectKey, tenantId, reporterId,
}: WorkItemDetailDrawerProps) {
  // ---- 读 store ----
  const workItems = useStore((s) => s.workItems);
  const identities = useStore((s) => s.identities);
  const agentSessions = useStore((s) => s.agentSessions);
  const worktrees = useStore((s) => s.worktrees);
  const comments = useStore((s) => s.comments);
  const addWorkItem = useStore((s) => s.addWorkItem);
  const updateWorkItemField = useStore((s) => s.updateWorkItemField);
  const removeWorkItem = useStore((s) => s.removeWorkItem);

  // ---- view 模式: 当前 workItem ----
  const currentWi = useMemo<WorkItem | null>(() => {
    if (!open || open.kind !== "view") return null;
    return workItems.find((w) => w.id === open.workItemId) ?? null;
  }, [open, workItems]);

  // ---- 草稿态 (Drawer 内编辑缓冲) ----
  // new 模式: { title, status (锁定), priority, kind, description, assignee_id, worktree_id, labels }
  // view 模式: 镜像 currentWi, 编辑时 set
  type Draft = {
    title: string;
    status: WorkItemStatus;
    priority: WorkItemPriority;
    kind: WorkItemKind;
    description: string;
    assignee_id: string | undefined;
    worktree_id: string | undefined;
    labels: string[];
    newLabel: string;
  };
  const [draft, setDraft] = useState<Draft | null>(null);
  // 每次 open 变化重置 draft
  useEffect(() => {
    if (!open) { setDraft(null); return; }
    if (open.kind === "new") {
      setDraft({
        title: "",
        status: open.defaultStatus,
        priority: "p2",
        kind: "task",
        description: "",
        assignee_id: undefined,
        worktree_id: undefined,
        labels: [],
        newLabel: "",
      });
    } else if (open.kind === "view" && currentWi) {
      setDraft({
        title: currentWi.title,
        status: currentWi.status,
        priority: currentWi.priority,
        kind: currentWi.kind,
        description: currentWi.description,
        assignee_id: currentWi.assignee_id,
        worktree_id: currentWi.worktree_id,
        labels: [...currentWi.labels],
        newLabel: "",
      });
    }
  }, [open, currentWi]);

  // ---- ESC 关闭 ----
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  // ---- 防止 body 滚动 ----
  useEffect(() => {
    if (!open) return;
    const prev = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => { document.body.style.overflow = prev; };
  }, [open]);

  // ---- 解析 ----
  const identityMap = useMemo(
    () => Object.fromEntries(identities.map((u) => [u.id, u])),
    [identities],
  );
  const agentMap = useMemo(
    () => Object.fromEntries(agentSessions.map((a) => [a.id, a])),
    [agentSessions],
  );
  const wtMap = useMemo(
    () => Object.fromEntries(worktrees.map((w) => [w.id, w])),
    [worktrees],
  );

  // ---- 当前 assignee 分类 (person / agent / squad / none) ----
  // 简化: assignee_id 形如 usr-* 是人, ag-* 是 agent, sq-* 是 squad (Phase 2)
  const currentAssigneeKind: "person" | "agent" | "squad" | "none" = useMemo(() => {
    if (!draft?.assignee_id) return "none";
    const id = draft.assignee_id;
    if (id.startsWith("usr-")) return "person";
    if (id.startsWith("ag-")) return "agent";
    if (id.startsWith("sq-")) return "squad";
    return "none";
  }, [draft?.assignee_id]);
  const currentAssigneeDisplay = useMemo(() => {
    if (!draft?.assignee_id) return "未分配";
    if (currentAssigneeKind === "person") {
      return identityMap[draft.assignee_id]?.display_name ?? draft.assignee_id;
    }
    if (currentAssigneeKind === "agent") {
      const a = agentMap[draft.assignee_id];
      return a ? `${a.agent_kind} · ${a.id}` : draft.assignee_id;
    }
    if (currentAssigneeKind === "squad") {
      return `${draft.assignee_id} (Phase 2)`;
    }
    return draft.assignee_id;
  }, [draft, currentAssigneeKind, identityMap, agentMap]);

  // ---- 关联 wt ----
  const currentWt: Worktree | null = draft?.worktree_id
    ? (wtMap[draft.worktree_id] ?? null)
    : null;
  const currentWtAgent: AgentSession | null = currentWt?.agent_session_id
    ? (agentMap[currentWt.agent_session_id] ?? null)
    : null;

  // ---- Comments stub (per view 模式显示 mock list; new 模式空) ----
  const currentComments = useMemo(() => {
    if (open?.kind !== "view" || !currentWi) return [];
    return comments.filter((c) => c.target_kind === "work_item" && c.target_id === currentWi.id);
  }, [open, currentWi, comments]);

  // ---- Submit handlers ----
  const titleRef = useRef<HTMLInputElement | null>(null);
  const canSave = !!draft && draft.title.trim().length > 0;
  const handleCreate = () => {
    if (!draft || !canSave || !projectId || !tenantId || !reporterId) return;
    const newId = addWorkItem({
      tenant_id: tenantId,
      project_id: projectId,
      title: draft.title.trim(),
      status: draft.status,
      kind: draft.kind,
      priority: draft.priority,
      reporter_id: reporterId,
      description: draft.description,
      assignee_id: draft.assignee_id,
      labels: draft.labels,
      worktree_id: draft.worktree_id,
    });
    // 关闭后让父组件切到 view 模式显示新建的卡 (per Multica 模式)
    onClose();
    // eslint-disable-next-line no-console
    console.info(`[WorkItemDetailDrawer] created work-item ${newId} (key auto-generated)`);
  };
  const handleSave = () => {
    if (!draft || !currentWi) return;
    // 字段分别 update (走 store 单字段更新 + 状态机)
    if (draft.title !== currentWi.title) updateWorkItemField(currentWi.id, "title", draft.title.trim());
    if (draft.status !== currentWi.status) updateWorkItemField(currentWi.id, "status", draft.status);
    if (draft.priority !== currentWi.priority) updateWorkItemField(currentWi.id, "priority", draft.priority);
    if (draft.kind !== currentWi.kind) updateWorkItemField(currentWi.id, "kind", draft.kind);
    if (draft.description !== currentWi.description) updateWorkItemField(currentWi.id, "description", draft.description);
    if (draft.assignee_id !== currentWi.assignee_id) updateWorkItemField(currentWi.id, "assignee_id", draft.assignee_id);
    if (draft.worktree_id !== currentWi.worktree_id) updateWorkItemField(currentWi.id, "worktree_id", draft.worktree_id);
    // labels 走数组比较
    if (JSON.stringify(draft.labels) !== JSON.stringify(currentWi.labels)) {
      updateWorkItemField(currentWi.id, "labels", draft.labels);
    }
  };
  const handleDelete = () => {
    if (!currentWi) return;
    if (typeof window !== "undefined" && !window.confirm(`删除任务卡 "${currentWi.title}"?`)) return;
    removeWorkItem(currentWi.id);
    onClose();
  };
  // autoFocus title in new mode
  useEffect(() => {
    if (open?.kind === "new") {
      requestAnimationFrame(() => titleRef.current?.focus());
    }
  }, [open]);

  // ---- 渲染 ----
  if (!open || !draft) return null;
  if (typeof document === "undefined") return null;

  const isNew = open.kind === "new";
  const modeBadge = isNew ? "NEW" : currentWi?.key ?? "VIEW";

  const drawerContent = (
    <div
      data-testid="work-item-drawer-root"
      className="fixed inset-0 z-50 flex justify-end"
      role="dialog"
      aria-modal="true"
    >
      {/* 背景: click 关闭 (per Jira/Multica 一致) */}
      <div
        className="absolute inset-0 bg-black/40 backdrop-blur-[1px] animate-in fade-in duration-150"
        onClick={onClose}
        data-testid="work-item-drawer-backdrop"
        aria-hidden
      />
      {/* 抽屉本体: 右侧滑入 */}
      <div
        data-testid="work-item-drawer-panel"
        className="relative h-full bg-bg-card border-l border-line shadow-2xl flex flex-col animate-in slide-in-from-right duration-200"
        style={{ width: DRAWER_W, maxWidth: "92vw" }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* === Header === */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-line bg-bg-soft/50 shrink-0">
          <div className="flex items-center gap-2 text-xs font-mono text-ink-mute">
            <span className="text-[10px] text-ink-dim uppercase tracking-wider">
              {isNew ? "新增任务" : "任务详情"}
            </span>
            <span className="text-[10px] text-ink-dim">·</span>
            <span className="text-[10px] text-accent font-semibold">
              {isNew ? "NEW" : (currentWi?.key ?? "—")}
            </span>
            {!isNew && currentWi && (
              <>
                <span className="text-[10px] text-ink-dim">·</span>
                <StatusPill value={currentWi.status} size="xs" />
              </>
            )}
          </div>
          <button
            type="button"
            data-testid="work-item-drawer-close"
            onClick={onClose}
            className="p-1.5 rounded hover:bg-bg-soft text-ink-mute hover:text-ink transition-colors"
            aria-label="关闭抽屉"
          >
            <X size={14} />
          </button>
        </div>

        {/* === Body (scrollable) === */}
        <div className="flex-1 overflow-y-auto px-5 py-4 space-y-4">
          {/* 1. Title */}
          <div>
            <label className="text-[10px] font-mono uppercase tracking-wider text-ink-dim block mb-1">
              标题 {isNew && <span className="text-err">*</span>}
            </label>
            <input
              ref={titleRef}
              data-testid="work-item-drawer-title"
              type="text"
              value={draft.title}
              onChange={(e) => setDraft({ ...draft, title: e.target.value })}
              placeholder="任务标题..."
              className="w-full text-sm font-medium bg-bg-soft border border-line rounded px-2.5 py-2 text-ink placeholder:text-ink-mute outline-none focus:border-accent"
            />
            {isNew && !canSave && (
              <div className="text-[10px] text-err mt-1 flex items-center gap-1">
                <AlertTriangle size={9} /> 标题必填
              </div>
            )}
          </div>

          {/* 2. Status + Priority + Kind 三栏并排 */}
          <div className="grid grid-cols-3 gap-2">
            <FieldSelect
              label="Status"
              icon={<Hash size={10} />}
              value={draft.status}
              options={STATUSES.map((s) => ({ value: s, label: s.replace(/_/g, " ") }))}
              onChange={(v) => setDraft({ ...draft, status: v as WorkItemStatus })}
              testid="work-item-drawer-status"
              disabled={isNew}  // new 模式 status 锁定 (列触发)
            />
            <FieldSelect
              label="Priority"
              icon={<Flag size={10} />}
              value={draft.priority}
              options={PRIORITIES.map((p) => ({ value: p, label: p.toUpperCase() }))}
              onChange={(v) => setDraft({ ...draft, priority: v as WorkItemPriority })}
              testid="work-item-drawer-priority"
            />
            <FieldSelect
              label="Kind"
              icon={<Tag size={10} />}
              value={draft.kind}
              options={KINDS.map((k) => ({ value: k, label: k }))}
              onChange={(v) => setDraft({ ...draft, kind: v as WorkItemKind })}
              testid="work-item-drawer-kind"
            />
          </div>

          {/* 3. Assignee (3 段: Person / Agent / Squad) */}
          <div>
            <label className="text-[10px] font-mono uppercase tracking-wider text-ink-dim block mb-1.5">
              <User size={9} className="inline mr-1" /> Assignee
            </label>
            <div className="flex gap-1 mb-1.5">
              {(["person", "agent", "squad"] as const).map((k) => {
                const active = currentAssigneeKind === k;
                const Icon = k === "person" ? User : k === "agent" ? Bot : Users;
                return (
                  <button
                    key={k}
                    type="button"
                    data-testid={`work-item-drawer-assignee-kind-${k}`}
                    onClick={() => {
                      // 切 kind 时清 assignee_id (避免跨 kind 错位)
                      setDraft({ ...draft, assignee_id: undefined });
                    }}
                    className={clsx(
                      "flex-1 flex items-center justify-center gap-1 px-2 py-1 text-[10px] font-mono rounded border transition-colors",
                      active
                        ? "border-accent text-accent bg-accent/10"
                        : "border-line text-ink-mute hover:border-ink-mute/60 hover:text-ink",
                    )}
                  >
                    <Icon size={10} />
                    {k === "person" ? "人" : k === "agent" ? "Agent" : "Squad"}
                  </button>
                );
              })}
            </div>
            {/* 当前 kind 对应的下拉 */}
            {currentAssigneeKind === "person" && (
              <select
                data-testid="work-item-drawer-assignee-person"
                value={draft.assignee_id ?? ""}
                onChange={(e) => setDraft({ ...draft, assignee_id: e.target.value || undefined })}
                className="w-full text-xs bg-bg-soft border border-line rounded px-2 py-1.5 text-ink outline-none focus:border-accent"
              >
                <option value="">未分配</option>
                {identities.map((u) => (
                  <option key={u.id} value={u.id}>{u.display_name} ({u.email})</option>
                ))}
              </select>
            )}
            {currentAssigneeKind === "agent" && (
              <select
                data-testid="work-item-drawer-assignee-agent"
                value={draft.assignee_id ?? ""}
                onChange={(e) => setDraft({ ...draft, assignee_id: e.target.value || undefined })}
                className="w-full text-xs bg-bg-soft border border-line rounded px-2 py-1.5 text-ink outline-none focus:border-accent"
              >
                <option value="">未分配</option>
                {agentSessions.map((a) => (
                  <option key={a.id} value={a.id}>
                    {a.agent_kind} · {a.id} · {a.status}
                  </option>
                ))}
              </select>
            )}
            {currentAssigneeKind === "squad" && (
              <div className="text-[10px] text-ink-mute italic px-2 py-1.5 border border-dashed border-line rounded">
                Squad 调度 Phase 2 接入, 本轮 UI 占位
              </div>
            )}
            <div className="text-[10px] text-ink-mute font-mono mt-1">
              当前: <span className="text-ink-dim">{currentAssigneeDisplay}</span>
            </div>
          </div>

          {/* 4. Worktree 关联 (Star 特有, AI 创作核心) */}
          <div>
            <label className="text-[10px] font-mono uppercase tracking-wider text-ink-dim block mb-1">
              <GitBranch size={9} className="inline mr-1" /> Worktree 关联
            </label>
            <select
              data-testid="work-item-drawer-worktree"
              value={draft.worktree_id ?? ""}
              onChange={(e) => setDraft({ ...draft, worktree_id: e.target.value || undefined })}
              className="w-full text-xs bg-bg-soft border border-line rounded px-2 py-1.5 text-ink outline-none focus:border-accent"
            >
              <option value="">未关联</option>
              {worktrees.map((w) => (
                <option key={w.id} value={w.id}>{w.name} · {w.status}</option>
              ))}
            </select>
            {currentWt && (
              <div className="mt-2 px-2.5 py-2 rounded border border-line bg-bg-soft/40 space-y-1.5">
                <div className="flex items-center justify-between text-[10px]">
                  <span className="font-mono text-ink-dim">17 状态机</span>
                  <StatusPill value={currentWt.status} size="xs" />
                </div>
                {currentWtAgent && (
                  <div className="flex items-center justify-between text-[10px]">
                    <span className="font-mono text-ink-dim flex items-center gap-1">
                      <Bot size={9} /> 关联 Agent
                    </span>
                    <span className="text-info font-mono">
                      {currentWtAgent.agent_kind} · {currentWtAgent.status}
                    </span>
                  </div>
                )}
                <div className="text-[9px] text-ink-mute font-mono">
                  branch: {currentWt.branch} (base: {currentWt.base_branch})
                </div>
              </div>
            )}
          </div>

          {/* 5. Description */}
          <div>
            <label className="text-[10px] font-mono uppercase tracking-wider text-ink-dim block mb-1">
              <FileText size={9} className="inline mr-1" /> Description (Markdown stub)
            </label>
            <textarea
              data-testid="work-item-drawer-description"
              value={draft.description}
              onChange={(e) => setDraft({ ...draft, description: e.target.value })}
              placeholder="任务描述...&#10;(Phase 2+ 接 Markdown 编辑器)"
              rows={4}
              className="w-full text-xs bg-bg-soft border border-line rounded px-2.5 py-2 text-ink placeholder:text-ink-mute outline-none focus:border-accent resize-y font-mono"
            />
          </div>

          {/* 6. Labels (chip input) */}
          <div>
            <label className="text-[10px] font-mono uppercase tracking-wider text-ink-dim block mb-1">
              <Tag size={9} className="inline mr-1" /> Labels
            </label>
            <div className="flex flex-wrap gap-1 mb-1.5">
              {draft.labels.map((l) => (
                <span
                  key={l}
                  data-testid={`work-item-drawer-label-${l}`}
                  className="inline-flex items-center gap-1 text-[10px] font-mono px-1.5 py-0.5 rounded bg-bg-soft border border-line/60 text-ink-dim"
                >
                  #{l}
                  <button
                    type="button"
                    onClick={() => setDraft({ ...draft, labels: draft.labels.filter((x) => x !== l) })}
                    className="text-ink-mute hover:text-err"
                    aria-label={`删除 label ${l}`}
                  >
                    <X size={9} />
                  </button>
                </span>
              ))}
            </div>
            <div className="flex gap-1">
              <input
                data-testid="work-item-drawer-label-input"
                type="text"
                value={draft.newLabel}
                onChange={(e) => setDraft({ ...draft, newLabel: e.target.value })}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    const v = draft.newLabel.trim();
                    if (v && !draft.labels.includes(v)) {
                      setDraft({ ...draft, labels: [...draft.labels, v], newLabel: "" });
                    }
                  }
                }}
                placeholder="加 label, Enter 提交"
                className="flex-1 text-xs bg-bg-soft border border-line rounded px-2 py-1 text-ink placeholder:text-ink-mute outline-none focus:border-accent"
              />
              <button
                type="button"
                onClick={() => {
                  const v = draft.newLabel.trim();
                  if (v && !draft.labels.includes(v)) {
                    setDraft({ ...draft, labels: [...draft.labels, v], newLabel: "" });
                  }
                }}
                className="px-2 py-1 text-[10px] rounded border border-line hover:border-accent text-ink-mute hover:text-accent"
                aria-label="加 label"
              >
                <Plus size={10} />
              </button>
            </div>
          </div>

          {/* 7. Comments timeline (view 模式, mock) */}
          {!isNew && currentWi && (
            <div>
              <label className="text-[10px] font-mono uppercase tracking-wider text-ink-dim block mb-1.5">
                Comments ({currentComments.length})
              </label>
              {currentComments.length === 0 ? (
                <div className="text-[10px] text-ink-mute italic px-2 py-2 border border-dashed border-line rounded">
                  暂无评论 (Phase 2+ 接 store.comments 写)
                </div>
              ) : (
                <div className="space-y-1.5">
                  {currentComments.map((c) => (
                    <div key={c.id} className="text-[11px] px-2 py-1.5 border border-line/60 rounded bg-bg-soft/40">
                      <div className="text-[9px] text-ink-mute font-mono mb-0.5">
                        {c.author_id} · {c.created_at.slice(0, 16)}
                      </div>
                      <div className="text-ink-dim">{c.body}</div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>

        {/* === Footer (sticky) === */}
        <div className="shrink-0 flex items-center justify-between gap-2 px-4 py-3 border-t border-line bg-bg-soft/50">
          <div className="text-[10px] text-ink-mute font-mono">
            {isNew ? (
              projectId ? "新卡将加入 board" : "⚠ 缺 project 上下文"
            ) : (
              <>updated_at: {currentWi?.updated_at?.slice(0, 16) ?? "—"}</>
            )}
          </div>
          <div className="flex items-center gap-1.5">
            {!isNew && (
              <button
                type="button"
                data-testid="work-item-drawer-delete"
                onClick={handleDelete}
                className="inline-flex items-center gap-1 px-2.5 py-1.5 text-[11px] rounded border border-err/40 text-err hover:bg-err/10 transition-colors"
              >
                <Trash2 size={11} /> 删除
              </button>
            )}
            <button
              type="button"
              data-testid="work-item-drawer-cancel"
              onClick={onClose}
              className="px-3 py-1.5 text-[11px] rounded border border-line text-ink-mute hover:text-ink hover:border-ink-mute/60 transition-colors"
            >
              取消
            </button>
            {isNew ? (
              <button
                type="button"
                data-testid="work-item-drawer-create"
                onClick={handleCreate}
                disabled={!canSave || !projectId}
                className="inline-flex items-center gap-1 px-3 py-1.5 text-[11px] rounded bg-accent text-bg hover:bg-accent/90 disabled:bg-ink-mute/40 disabled:cursor-not-allowed transition-colors font-semibold"
              >
                <Plus size={11} /> Create
              </button>
            ) : (
              <button
                type="button"
                data-testid="work-item-drawer-save"
                onClick={handleSave}
                className="inline-flex items-center gap-1 px-3 py-1.5 text-[11px] rounded bg-accent text-bg hover:bg-accent/90 transition-colors font-semibold"
              >
                <Save size={11} /> Save
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );

  return createPortal(drawerContent, document.body);
}

// =====================================================================
// FieldSelect — 紧凑下拉 (status / priority / kind 共用)
// =====================================================================
function FieldSelect<T extends string>({
  label, icon, value, options, onChange, testid, disabled,
}: {
  label: string;
  icon: React.ReactNode;
  value: T;
  options: { value: T; label: string }[];
  onChange: (v: T) => void;
  testid: string;
  disabled?: boolean;
}) {
  return (
    <div>
      <label className="text-[9px] font-mono uppercase tracking-wider text-ink-dim block mb-1">
        {icon} {label}
      </label>
      <div className="relative">
        <select
          data-testid={testid}
          value={value}
          onChange={(e) => onChange(e.target.value as T)}
          disabled={disabled}
          className={clsx(
            "w-full text-[11px] bg-bg-soft border border-line rounded pl-2 pr-6 py-1.5 text-ink outline-none focus:border-accent appearance-none",
            disabled && "opacity-60 cursor-not-allowed",
          )}
        >
          {options.map((o) => (
            <option key={o.value} value={o.value}>{o.label}</option>
          ))}
        </select>
        <ChevronDown size={10} className="absolute right-1.5 top-1/2 -translate-y-1/2 text-ink-mute pointer-events-none" />
      </div>
    </div>
  );
}
