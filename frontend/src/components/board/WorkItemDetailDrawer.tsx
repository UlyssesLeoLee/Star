/*
@cypher schema=1 source_sha256=47d613032f867dc2362ce150d472689780774a8c9b0be7a4919253f19eb34641
MERGE (self:File {path:"frontend/src/components/board/WorkItemDetailDrawer.tsx"})
MERGE (agentTypes:File {path:"frontend/src/lib/agent/types.ts"})
MERGE (agentKindDisplayLabel:Function {name:"agentKindDisplayLabel", language:"typescript", visibility:"public"})
MERGE (workItemDetailDrawer:Function {name:"WorkItemDetailDrawer", language:"typescript", visibility:"public"})
MERGE (self)-[:IMPORTS]->(agentTypes)
MERGE (workItemDetailDrawer)-[:CALLS]->(agentKindDisplayLabel)
MERGE (terminalHookFile:File {path:"frontend/src/hooks/useTerminalStackWs.ts"})
MERGE (useTerminalStackWs:Function {name:"useTerminalStackWs", language:"typescript", visibility:"public"})
MERGE (terminalHookFile)-[:DEFINES]->(useTerminalStackWs)
MERGE (workItemDetailDrawer)-[:CALLS]->(useTerminalStackWs)
@endcypher
*/
"use client";

// =====================================================================
// WorkItemDetailDrawer — Kanban 卡右侧抽屉详情 (per 2026-08-31 12:07 JST Ulysses 拍板)
// =====================================================================
// 触发 (per 拍板: 任务卡可从看板创建或打开):
//   - Kanban 列底部 +Add task 按钮  → mode="new"   (status = 点击的列)
//   - KanbanCard 整张点击            → mode="view"  (status = wi.status)
//
// 模式:
//   - "new"  : title 必填, Status 锁定为触发列, 其他字段可选, 底部 "Create" 按钮
//   - "view" : 所有字段 inline 编辑, 底部 "Save" 持久化 (走 store.updateWorkItemField)
//
// 字段 (per Star 设计):
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
import { TaskCardCommand, type TaskCardSendTarget } from "@/components/board/TaskCardCommand";
import { useNavStore } from "@/lib/nav/navStore";
import { useChatStore } from "@/lib/chat/chatStore";
import { clsx } from "clsx";
import {
  X, Trash2, Plus, Flag, Hash, Tag, FileText, User, Bot, Users,
  GitBranch, AlertTriangle, Save, ChevronDown, ListTodo, Workflow,
  Terminal, ArrowRightLeft, MessageSquare, Clock,
} from "lucide-react";
import { StatusPill } from "@/components/StatusPill";
import { useStore } from "@/lib/store";
import { agentKindDisplayLabel } from "@/lib/agent/types";
import { openWorkItemDocument, DOCUMENT_KIND_META } from "@/lib/documents";
import { useTerminalStackWs } from "@/hooks/useTerminalStackWs";
import { searchIpaSkills } from "@/lib/ipa-skills/search";
import { IPA_SKILL_BY_ID } from "@/lib/ipa-skills/catalog";
import { useWorktreeGroupApi } from "@/lib/group/groupProjection";
import { useRunDirectory } from "@/lib/run/runDirectorySession";
import type { CloudBranch } from "@/lib/run/runDirectoryApi";
import type {
  WorkItem, WorkItemStatus, WorkItemKind, WorkItemPriority,
  Identity, AgentSession, Worktree,
  WorkItemDocument,
} from "@/types/ids";

// ---- 新建命令（供 onCreate 接管，per G-13 单一写入路径） ----
export type WorkItemCreateCommand = {
  tenant_id: string;
  project_id: string;
  title: string;
  status: WorkItemStatus;
  kind: WorkItemKind;
  priority: WorkItemPriority;
  reporter_id: string;
  description: string;
  assignee_id: string | undefined;
  labels: string[];
  worktree_id: string | undefined;
  branch_name: string | null;
};

// ---- Mode discriminator ----
export type WorkItemDrawerMode =
  | { kind: "new"; defaultStatus: WorkItemStatus; defaultKind?: WorkItemKind; lockKind?: boolean }
  | { kind: "view"; workItemId: string };

export interface WorkItemDetailDrawerProps {
  open: WorkItemDrawerMode | null;
  onClose: () => void;
  /** 父组件传 project 上下文 (new 模式必须) */
  projectId?: string;
  projectKey?: string;
  tenantId?: string;
  reporterId?: string;
  /** new 模式下的默认 kind (缺陷创建窗口传 "bug" 锁定为缺陷类型) */
  defaultKind?: WorkItemKind;
  /** new 模式下是否锁定 kind (锁定后只显示只读文本, 不可切换) */
  lockKind?: boolean;
  /** 点击子任务行时打开子任务卡 (per 2026-11-11 子任务层级) */
  onOpenChildTask?: (workItemId: string) => void;
  /**
   * 新建命令**接管回调**（per G-13 单一写入路径）。
   *
   * 提供时：抽屉不再直接写本地 store，而是把创建命令交给调用方
   * （缺陷域走 owner 创建命令 `POST /defects`），由调用方用回执/revision 刷新。
   * 未提供时：回退到本地 store（其它模块的既有行为，不受影响）。
   */
  onCreate?: (command: WorkItemCreateCommand) => Promise<string | null>;
}

// ---- Static enums (跟 types/ids.ts 同步) ----
const STATUSES: WorkItemStatus[] = [
  "todo", "in_progress", "review", "blocked", "done", "wontfix",
];
const PRIORITIES: WorkItemPriority[] = ["p0", "p1", "p2", "p3"];
const KINDS: WorkItemKind[] = ["story", "task", "bug", "spike", "epic"];

// ---- Drawer width (自适应窗口) ----
// per 用户发令: 任务卡整体自适应主窗口区域，展开/收缩随窗口变化
// 不再硬编码固定宽度，由 CSS min(95vw, 1200px) 自动决定

export function WorkItemDetailDrawer({
  open, onClose, projectId, projectKey, tenantId, reporterId, onOpenChildTask,
  defaultKind, lockKind, onCreate,
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
  // per 2026-11-11 用户发令: 任务卡关联多文档 (Drawer 内管理)
  const addWorkItemDocument = useStore((s) => s.addWorkItemDocument);
  const removeWorkItemDocument = useStore((s) => s.removeWorkItemDocument);
  // per 2026-11-11 用户发令: 任务卡开子任务 / 关联 IPA 技能 / 文件即任务
  const addSubTask = useStore((s) => s.addSubTask);
  const addFileSubTask = useStore((s) => s.addFileSubTask);
  const linkIpaSkill = useStore((s) => s.linkIpaSkill);
  const unlinkIpaSkill = useStore((s) => s.unlinkIpaSkill);
  const [newDocTitle, setNewDocTitle] = useState("");
  const [newDocPath, setNewDocPath] = useState("");
  const [newDocKind, setNewDocKind] = useState<WorkItemDocument["kind"]>("document");
  const [newSubTaskTitle, setNewSubTaskTitle] = useState("");
  const [newFileSubTaskTitle, setNewFileSubTaskTitle] = useState("");
  const [newFileSubTaskPath, setNewFileSubTaskPath] = useState("");
  const [ipaQuery, setIpaQuery] = useState("");
    const [activeTab, setActiveTab] = useState<"details" | "cli" | "comments" | "history">("details");

    // ---- 跨 branch 显式互传 (handoff) — per 2026-11-21 用户发令: 每个 branch 独立一套数据,
  // 不传时各自独立; 互传必须用户手动指定目标 branch。服务端走 SCD2 + 审计 + 幂等。 ----
  const groupApi = useWorktreeGroupApi();
  const runDirectory = useRunDirectory();
  const [handoffTarget, setHandoffTarget] = useState("");
  const [handoffReason, setHandoffReason] = useState("");
  const [handoffBusy, setHandoffBusy] = useState(false);
  const [handoffError, setHandoffError] = useState<string | null>(null);
  const [handoffDone, setHandoffDone] = useState<string | null>(null);
  const [handoffBranches, setHandoffBranches] = useState<CloudBranch[]>([]);

  // ---- CLI 面板接真实 terminal REST (per 2026-11-16 用户发令: star-api-rest pane 接线) ----
  // sessionId=null → 由 REST createPane 建 pane; 后端不可达自动回退 preview (mode 标注),
  // 只加调用不重构: 命令文本经 writeLine append 到终端滚动缓冲。
  const { writeLine } = useTerminalStackWs({ sessionId: null });

  // ---- view 模式: 当前 workItem ----
  const currentWi = useMemo<WorkItem | null>(() => {
    if (!open || open.kind !== "view") return null;
    return workItems.find((w) => w.id === open.workItemId) ?? null;
  }, [open, workItems]);

  // 目标 branch 列表来自服务端 SCM 真源 (同 Project 已注册 cloud_branch, 含 UUID);
  // 绝不从本地 git 枚举名推断所有权 — handoff 需要 target_branch_id UUID。
  useEffect(() => {
    if (!currentWi || !projectId || runDirectory.status !== "ready" || !runDirectory.client) {
      setHandoffBranches([]);
      return;
    }
    const controller = new AbortController();
    runDirectory.client.listBranches(projectId, undefined, controller.signal)
      .then((page) => setHandoffBranches(page.rows))
      .catch(() => { if (!controller.signal.aborted) setHandoffBranches([]); });
    return () => controller.abort();
  }, [currentWi, projectId, runDirectory.status, runDirectory.client]);

  // ---- 子任务 (per 2026-11-11 用户发令: parent_id 层级, 与父卡同一套内容模型) ----
  const subTasks = useMemo<WorkItem[]>(() => {
    if (!open || open.kind !== "view") return [];
    return workItems.filter((w) => w.parent_id === open.workItemId);
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
        // per G-13 / DD §3.2.3: prop 级 defaultKind 优先, mode 级兜底
        kind: defaultKind ?? open.defaultKind ?? "task",
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
      return a ? `${agentKindDisplayLabel(a.agent_kind)} · ${a.id}` : draft.assignee_id;
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
  // per 2026-11-21 用户发令: 无法创建 → 屏幕顶端失败原因 + 完善建议链接
  const [createFailure, setCreateFailure] = useState<{ reason: string; hint: string; link?: string } | null>(null);
  // owner 写入路径进行中（per G-13）
  const [createBusy, setCreateBusy] = useState(false);
  const handleCreate = async () => {
    if (!draft || !canSave) {
      setCreateFailure({ reason: "标题必填", hint: "填写任务卡标题后再创建。" });
      return;
    }
    if (!projectId || !tenantId || !reporterId) {
      const missing = [
        !projectId && "项目 (先在左侧目录树选择 Project)",
        !tenantId && "租户上下文",
        !reporterId && "报告人身份",
      ].filter(Boolean).join("、");
      setCreateFailure({
        reason: `缺少创建上下文: ${missing}`,
        hint: "完善工作区上下文后再创建任务卡。",
        link: "/settings",
      });
      return;
    }
    setCreateFailure(null);
    // per 2026-11-21: 创建即绑定当前 branch (branch 级数据隔离)
    // branch_id 保留为服务端 UUID（本地任务无，设 null）；branch_name 记录本地分支名
    const branchName = useNavStore.getState().selectedBranch || null;

    // --- G-13 单一写入路径 ---
    // 调用方提供 onCreate 时：走 owner 创建命令（缺陷域 = POST /defects），
    // 由调用方用回执/revision 刷新；此处**不**直接写本地 store。
    // 未提供 onCreate 时：回退本地 store（其它模块既有行为不变）。
    if (onCreate) {
      setCreateBusy(true);
      try {
        const createdId = await onCreate({
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
          branch_name: branchName,
        });
        if (createdId) {
          onClose();
          // eslint-disable-next-line no-console
          console.info(`[WorkItemDetailDrawer] created work-item ${createdId} via owner`);
        } else {
          setCreateFailure({
            reason: "创建未成功",
            hint: "owner 未返回新记录 id，请重试。",
          });
        }
      } catch (err) {
        setCreateFailure({
          reason: "创建失败",
          hint: err instanceof Error ? err.message : "未知错误",
        });
      } finally {
        setCreateBusy(false);
      }
      return;
    }

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
      branch_id: null,
      branch_name: branchName,
    });
    // 关闭后让父组件切到 view 模式显示新建的卡
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
  // 跨 branch 互传: 仅服务端卡 (有 worktree_id + lifecycle_version) 可用;
  // 本地卡无服务端版本, 明确禁用并给出原因 (不冒充可互传)。
  const handleHandoff = async () => {
    if (!currentWi || !groupApi || !handoffTarget) return;
    const sourceWorktreeId = currentWi.worktree_id;
    const expectedVersion = currentWi.lifecycle_version;
    if (!sourceWorktreeId || !expectedVersion) {
      setHandoffError("该任务卡未绑定服务端 Worktree/版本, 无法跨 branch 互传。请先关联 Worktree 并从服务端刷新。");
      return;
    }
    if (handoffTarget === currentWi.branch_id) {
      setHandoffError("目标 branch 与当前 branch 相同, 无需互传。");
      return;
    }
    setHandoffBusy(true);
    setHandoffError(null);
    setHandoffDone(null);
    try {
      const receipt = await groupApi.handoffWorkItem<{ handed_off: boolean; target_branch_id: string; task_card?: { branch_name?: string | null; lifecycle?: { version?: number } } }>(
        sourceWorktreeId,
        currentWi.id,
        {
          target_branch_id: handoffTarget,
          expected_version: expectedVersion,
          correlation_id: typeof crypto !== "undefined" && crypto.randomUUID ? crypto.randomUUID() : undefined,
          reason: handoffReason.trim() || undefined,
        },
        typeof crypto !== "undefined" && crypto.randomUUID ? crypto.randomUUID() : `handoff-${Date.now()}`,
      );
      // 本地缓存同步新 branch/版本 (服务端真源已转移; 下次刷新会覆盖)
      const targetName = handoffBranches.find((b) => b.branch_id === handoffTarget)?.name ?? receipt.task_card?.branch_name ?? null;
      updateWorkItemField(currentWi.id, "branch_id", handoffTarget);
      updateWorkItemField(currentWi.id, "branch_name", targetName);
      if (receipt.task_card?.lifecycle?.version) {
        updateWorkItemField(currentWi.id, "lifecycle_version", receipt.task_card.lifecycle.version);
      }
      setHandoffDone(`已互传到 branch「${targetName ?? handoffTarget}」`);
      setHandoffTarget("");
      setHandoffReason("");
    } catch (err) {
      const e = err as { status?: number; code?: string; message?: string };
      setHandoffError(
        e?.code === "version_conflict" || e?.status === 409
          ? "版本冲突: 任务卡已被其他操作更新, 请刷新后重试。"
          : e?.status === 403
            ? "越权: 当前会话无目标 branch 的写入权限。"
            : e?.message || "互传失败, 请重试。",
      );
    } finally {
      setHandoffBusy(false);
    }
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
  // per G-13 / DD §3.2.3: kind 锁定 (缺陷创建窗口 lockKind=true) — 只读文本, 不可切换
  const kindLocked = isNew && (lockKind === true || open.lockKind === true);

  const drawerContent = (
      <div
        data-testid="work-item-drawer-root"
        className="fixed inset-0 z-50 flex items-center justify-center p-3"
        role="dialog"
        aria-modal="true"
      >
        {/* 背景: click 关闭 */}
        <div
          className="absolute inset-0 bg-black/5 backdrop-blur-sm animate-in fade-in duration-150"
          onClick={onClose}
          data-testid="work-item-drawer-backdrop"
          aria-hidden
        />
        {/* 子窗口本体: 居中弹出 (per 2026-11-15 用户发令: 不要右侧弹出) */}
        <div
          data-testid="work-item-drawer-panel"
          className="relative h-[92vh] max-h-[min(92vh,900px)] bg-bg-card border-2 border-[var(--cel-ink)] shadow-soft rounded-xl flex flex-col animate-in zoom-in-95 duration-150 overflow-hidden"
                    style={{ width: "min(95vw, 1200px)", maxWidth: "min(95vw, 1200px)" }}
          onClick={(e) => e.stopPropagation()}
        >
          {/* === Header with Tabs === */}
          <div className="flex flex-col border-b border-line bg-bg-soft/50 shrink-0">
            <div className="flex items-center justify-between px-4 py-3">
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
            {/* Tab bar */}
            {!isNew && (
              <div
                className="flex gap-1 px-2 pb-2 border-b border-line/50"
                data-testid="work-item-drawer-tabs"
                role="tablist"
              >
                {[
                  { id: "details", label: "详情", icon: FileText },
                  { id: "cli", label: "CLI", icon: Terminal },
                  { id: "comments", label: "评论", icon: MessageSquare },
                  { id: "history", label: "历史", icon: Clock },
                ].map(({ id, label, icon: Icon }) => (
                  <button
                    key={id}
                    type="button"
                    role="tab"
                    aria-selected={activeTab === id}
                    onClick={() => setActiveTab(id as typeof activeTab)}
                    className={clsx(
                      "flex items-center gap-1 px-3 py-1.5 text-[10px] font-mono rounded-t transition-colors",
                      activeTab === id
                        ? "border-b-2 border-accent text-accent bg-accent/10"
                        : "text-ink-mute hover:text-ink hover:bg-bg-soft/50",
                    )}
                    data-testid={`work-item-drawer-tab-${id}`}
                  >
                    <Icon size={10} /> {label}
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* per 2026-11-21 用户发令: 无法创建 → 弹窗顶端失败原因 + 完善建议链接 */}
          {isNew && createFailure && (
            <div
              data-testid="work-item-drawer-create-failure"
              className="shrink-0 flex items-start gap-2 border-b border-err/40 bg-err/10 px-4 py-2.5 text-xs text-err"
              role="alert"
            >
              <span className="mt-0.5 size-1.5 shrink-0 rounded-full bg-err" />
              <div className="min-w-0 flex-1">
                <div className="font-semibold">无法创建任务卡: {createFailure.reason}</div>
                <div className="mt-0.5 text-err/90">
                  {createFailure.hint}
                  {createFailure.link && (
                    <a href={createFailure.link} className="ml-1.5 font-semibold underline underline-offset-2" data-testid="work-item-drawer-create-failure-link">
                      完善建议 →
                    </a>
                  )}
                </div>
              </div>
              <button type="button" onClick={() => setCreateFailure(null)} className="shrink-0 text-err/70 hover:text-err" aria-label="关闭失败提示">
                <X size={12} />
              </button>
            </div>
          )}

        {/* === Body (scrollable) === */}
        {/* === Body (scrollable) === */}
        {activeTab === "details" && (
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
            {kindLocked ? (
              <FieldReadonly
                label="Kind"
                icon={<Tag size={10} />}
                value={draft.kind}
                testid="work-item-drawer-kind"
              />
            ) : (
              <FieldSelect
                label="Kind"
                icon={<Tag size={10} />}
                value={draft.kind}
                options={KINDS.map((k) => ({ value: k, label: k }))}
                onChange={(v) => setDraft({ ...draft, kind: v as WorkItemKind })}
                testid="work-item-drawer-kind"
              />
            )}
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
                    {agentKindDisplayLabel(a.agent_kind)} · {a.id} · {a.status}
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

          {/* 4. Worktree 关联 */}
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
                      {agentKindDisplayLabel(currentWtAgent.agent_kind)} · {currentWtAgent.status}
                    </span>
                  </div>
                )}
                <div className="text-[9px] text-ink-mute font-mono">
                  branch: {currentWt.branch} (base: {currentWt.base_branch})
                </div>
              </div>
            )}
          </div>

          {/* 4b. 跨 branch 显式互传 (handoff) */}
          {!isNew && currentWi && (
            <div data-testid="work-item-drawer-handoff">
              <label className="text-[10px] font-mono uppercase tracking-wider text-ink-dim block mb-1">
                <ArrowRightLeft size={9} className="inline mr-1" /> 跨 Branch 互传 (Handoff)
              </label>
              <div className="text-[9px] text-ink-mute mb-1.5">
                每个 branch 独立一套数据; 不互传时任务卡留在当前 branch。互传需手动指定目标 branch。
              </div>
              {currentWi.branch_name && (
                <div className="text-[10px] font-mono text-ink-dim mb-1">
                  当前 branch: <span className="text-info">{currentWi.branch_name}</span>
                </div>
              )}
              {(!currentWi.worktree_id || !currentWi.lifecycle_version) ? (
                <div className="text-[10px] text-ink-mute italic px-2 py-1.5 border border-dashed border-line rounded">
                  本地任务卡 (无服务端 Worktree/版本) 不支持跨 branch 互传。请先关联 Worktree 并从服务端刷新。
                </div>
              ) : (
                <div className="space-y-1.5">
                  <select
                    data-testid="work-item-drawer-handoff-target"
                    value={handoffTarget}
                    onChange={(e) => { setHandoffTarget(e.target.value); setHandoffError(null); setHandoffDone(null); }}
                    disabled={handoffBusy || handoffBranches.length === 0}
                    className="w-full text-xs bg-bg-soft border border-line rounded px-2 py-1.5 text-ink outline-none focus:border-accent disabled:opacity-50"
                  >
                    <option value="">{handoffBranches.length === 0 ? "无可用目标 branch (需服务端会话)" : "选择目标 branch…"}</option>
                    {handoffBranches
                      .filter((b) => b.branch_id !== currentWi.branch_id)
                      .map((b) => (
                        <option key={b.branch_id} value={b.branch_id}>{b.name}</option>
                      ))}
                  </select>
                  <input
                    data-testid="work-item-drawer-handoff-reason"
                    value={handoffReason}
                    onChange={(e) => setHandoffReason(e.target.value)}
                    placeholder="互传原因 (可选)"
                    maxLength={4000}
                    disabled={handoffBusy}
                    className="w-full text-xs bg-bg-soft border border-line rounded px-2 py-1.5 text-ink placeholder:text-ink-mute outline-none focus:border-accent disabled:opacity-50"
                  />
                  <button
                    type="button"
                    data-testid="work-item-drawer-handoff-submit"
                    onClick={handleHandoff}
                    disabled={handoffBusy || !handoffTarget || !groupApi}
                    className="w-full text-xs font-mono px-2 py-1.5 rounded border border-accent text-accent hover:bg-accent/10 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
                  >
                    {handoffBusy ? "互传中…" : "确认互传到目标 branch"}
                  </button>
                  {handoffError && (
                    <div data-testid="work-item-drawer-handoff-error" className="text-[10px] text-err flex items-start gap-1">
                      <AlertTriangle size={10} className="mt-0.5 shrink-0" /> {handoffError}
                    </div>
                  )}
                  {handoffDone && (
                    <div data-testid="work-item-drawer-handoff-done" className="text-[10px] text-ok">{handoffDone}</div>
                  )}
                </div>
              )}
            </div>
          )}

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
          </div>
          )}
          {activeTab === "cli" && (
          <div data-testid="work-item-cli-panel" className="h-full flex flex-col">
            <div className="flex items-center gap-2 px-3 py-1.5 border-b border-white/10 bg-[#0b0d10] shrink-0">
              <Terminal size={11} className="text-[#79c0ff]" />
              <span className="text-[9px] font-mono font-bold uppercase tracking-wider text-[#8b949e]">CLI</span>
              <span className="text-[8px] font-mono text-[#8b949e]">
                {currentWi?.worktree_id ? `worktree ${currentWi.worktree_id}` : "—"}
              </span>
            </div>
            <div className="flex-1 overflow-hidden">
              <TaskCardCommand
                taskId={currentWi?.id ?? ""}
                taskKey={currentWi?.key ?? ""}
                onSubmitCommand={(cmd, target: TaskCardSendTarget) => {
                  void writeLine(cmd);
                  if (target === "card") return;
                  const chat = useChatStore.getState();
                  if (target === "worktree") {
                    const branch = useNavStore.getState().selectedBranch;
                    const refId = branch ? `${currentWi?.id ?? ""}@${branch}` : (currentWi?.id ?? "");
                    const topicId = chat.openTopicForTask(refId, currentWi?.title ?? "");
                    chat.setChatScope("WORKTREE");
                    chat.reply(topicId, "Ulysses", "human", cmd);
                  } else {
                    const sessionId = chat.globalSessions[0]?.id ?? chat.createSession(`任务 ${currentWi?.key ?? ""} 讨论`);
                    const topicId = chat.openTopicForSession(sessionId);
                    chat.setChatScope("GLOBAL");
                    chat.reply(topicId, "Ulysses", "human", `[任务卡 ${currentWi?.key ?? ""}] ${cmd}`);
                  }
                }}
                onHandoff={(target) => {
                  const branch = useNavStore.getState().selectedBranch;
                  const refId = branch ? `${currentWi?.id ?? ""}@${branch}` : (currentWi?.id ?? "");
                  const topicId = useChatStore.getState().openTopicForTask(refId, currentWi?.title ?? "");
                  useChatStore.getState().reply(topicId, "Ulysses", "human", `[交接] 任务 ${currentWi?.key} 交接给 ${target}${branch ? ` (branch: ${branch})` : ""}`);
                }}
              />
            </div>
          </div>
          )}
          {activeTab === "comments" && (
          <div className="h-full flex flex-col">
            <div className="flex items-center gap-2 px-3 py-2 border-b border-line shrink-0">
              <MessageSquare size={12} className="text-accent" />
              <span className="text-sm font-semibold">评论</span>
              <span className="text-[10px] text-ink-mute">({currentComments.length})</span>
            </div>
            <div className="flex-1 overflow-y-auto px-3 py-3 space-y-3">
              {currentComments.length === 0 ? (
                <div className="flex items-center justify-center h-full text-ink-mute">
                  暂无评论
                </div>
              ) : (
                currentComments.map((c) => (
                  <div key={c.id} className="flex gap-2 text-sm">
                    <div className="w-6 h-6 rounded-full bg-accent/10 flex items-center justify-center">
                      <User size={10} className="text-accent" />
                    </div>
                    <div className="flex-1">
                      <div className="flex items-center gap-2 text-[10px] text-ink-mute">
                        <span className="font-mono">{c.author_id}</span>
                        <span>{new Date(c.created_at).toLocaleString()}</span>
                      </div>
                      <div className="text-ink whitespace-pre-wrap mt-1">{c.body}</div>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
          )}
          {activeTab === "history" && (
          <div className="h-full flex flex-col">
            <div className="flex items-center gap-2 px-3 py-2 border-b border-line shrink-0">
              <Clock size={12} className="text-accent" />
              <span className="text-sm font-semibold">历史</span>
            </div>
            <div className="flex-1 overflow-y-auto px-3 py-3 space-y-2">
              {!currentWi ? (
                <div className="text-ink-mute text-center py-8">新建任务无历史记录</div>
              ) : (
                <div className="text-[10px] font-mono text-ink-dim space-y-1">
                  <div className="flex gap-2 text-ink-mute">
                    <span>创建:</span>
                    <code>{currentWi.created_at}</code>
                  </div>
                  <div className="flex gap-2 text-ink-mute">
                    <span>更新:</span>
                    <code>{currentWi.updated_at}</code>
                  </div>
                  <div className="flex gap-2 text-ink-mute">
                    <span>报告人:</span>
                    <code>{currentWi.reporter_id}</code>
                  </div>
                  <div className="flex gap-2 text-ink-mute">
                    <span>Kind:</span>
                    <code>{currentWi.kind}</code>
                  </div>
                  <div className="pt-2 border-t border-line/40">
                    <span className="font-semibold text-ink">状态流转:</span>
                    <div className="mt-1 flex gap-1 flex-wrap">
                      {STATUSES.map((s) => (
                        <span
                          key={s}
                          className={clsx(
                            "px-1.5 py-0.5 text-[9px] font-mono rounded border",
                            s === currentWi.status
                              ? "border-accent bg-accent/10 text-accent font-bold"
                              : "border-line text-ink-mute",
                          )}
                        >
                          {s.replace(/_/g, " ")}
                        </span>
                      ))}
                    </div>
                  </div>
                </div>
              )}
            </div>
          </div>
          )}
        
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

        {/* === CLI 面板 (per 2026-11-15 用户发令: 任务卡子窗口带 CLI) === */}
        {!isNew && (
          <div
            data-testid="work-item-cli-panel"
            className="flex-1 border-t-2 border-[var(--cel-ink)] bg-[#0b0d10] flex flex-col"
          >
            <div className="flex items-center gap-2 px-3 py-1.5 border-b border-white/10">
              <Terminal size={11} className="text-[#79c0ff]" />
              <span className="text-[9px] font-mono font-bold uppercase tracking-wider text-[#8b949e]">CLI</span>
              <span className="text-[8px] font-mono text-[#8b949e]">
                {currentWi?.worktree_id ? `worktree ${currentWi.worktree_id}` : "—"}
              </span>
            </div>
            <TaskCardCommand
              taskId={currentWi?.id ?? ""}
              taskKey={currentWi?.key ?? ""}
              onSubmitCommand={(cmd, target: TaskCardSendTarget) => {
                // 命令文本 append 到 CLI 终端滚动缓冲 (star-api-rest writeLine;
                // 失败自动回退 preview, mode='error' 诚实标注 — 只加调用不重构)
                void writeLine(cmd);
                // per 2026-11-21 用户发令: 默认仅保留任务卡; 只有手动指定
                // Worktree/Global 才写入聊天话题 (branch 级隔离: 话题 refId
                // 带 branch 维度, 不同 branch 各自独立, 互传需显式交接)。
                if (target === "card") return;
                const chat = useChatStore.getState();
                if (target === "worktree") {
                  const branch = useNavStore.getState().selectedBranch;
                  const refId = branch ? `${currentWi?.id ?? ""}@${branch}` : (currentWi?.id ?? "");
                  const topicId = chat.openTopicForTask(refId, currentWi?.title ?? "");
                  chat.setChatScope("WORKTREE");
                  chat.reply(topicId, "Ulysses", "human", cmd);
                } else {
                  // Global: 复用全局会话通道; 未选会话时开一个以任务卡命名的会话
                  const sessionId = chat.globalSessions[0]?.id ?? chat.createSession(`任务 ${currentWi?.key ?? ""} 讨论`);
                  const topicId = chat.openTopicForSession(sessionId);
                  chat.setChatScope("GLOBAL");
                  chat.reply(topicId, "Ulysses", "human", `[任务卡 ${currentWi?.key ?? ""}] ${cmd}`);
                }
              }}
              onHandoff={(target) => {
                // 交接 = 显式跨 branch 互传通道 (per 2026-11-21: 不传时各自独立)
                const branch = useNavStore.getState().selectedBranch;
                const refId = branch ? `${currentWi?.id ?? ""}@${branch}` : (currentWi?.id ?? "");
                const topicId = useChatStore.getState().openTopicForTask(refId, currentWi?.title ?? "");
                useChatStore.getState().reply(topicId, "Ulysses", "human", `[交接] 任务 ${currentWi?.key} 交接给 ${target}${branch ? ` (branch: ${branch})` : ""}`);
              }}
            />
          </div>
        )}
      </div>
    </div>
  );

  return createPortal(drawerContent, document.body);
}

// =====================================================================
// FieldReadonly — 锁定字段只读展示 (per G-13 / DD §3.2.3: kind 锁定不可切换)
// =====================================================================
function FieldReadonly({
  label, icon, value, testid,
}: {
  label: string;
  icon: React.ReactNode;
  value: string;
  testid: string;
}) {
  return (
    <div>
      <label className="text-[9px] font-mono uppercase tracking-wider text-ink-dim block mb-1">
        {icon} {label}
      </label>
      <div
        data-testid={`${testid}-locked`}
        data-kind-locked="true"
        className="w-full text-[11px] bg-bg-soft border border-line rounded pl-2 pr-6 py-1.5 text-ink opacity-60 cursor-not-allowed select-none"
      >
        {value}
      </div>
    </div>
  );
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
