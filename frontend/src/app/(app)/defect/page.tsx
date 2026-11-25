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
//
// 数据来源 (per N-01 / G-14 / G-17):
//   1. 优先 REST 读模型 GET /api/v1/defects (frontend/src/lib/defect/defectApi.ts)
//   2. 失败 → 显式降级到 worktreeGroupApi 多 worktree 路径, UI 标注「来源：降级路径」
// 扩展字段三态渲染 (per DD §3.2.2 / D-DI-06): 有值 / （未填写）/ （未接入）;
// 验证状态只来自真实字段 latest_verification, 任何情况下不得由 status==="done" 推断。
// =====================================================================
"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { useStore } from "@/lib/store";
import { getSession } from "@/lib/auth";
import {
  useWorktreeGroupApi,
  type GroupWorkItemView,
  type WorkItemLifecycleStatus,
} from "@/lib/group/groupProjection";
import {
  createDefect,
  listDefects,
  type DefectListItemView,
} from "@/lib/defect/defectApi";
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
  type WorkItemCreateCommand,
  type WorkItemDrawerMode,
} from "@/components/board/WorkItemDetailDrawer";
import type { WorkItemKind, WorkItemPriority, WorkItemStatus } from "@/types/ids";

// 看板 6 列状态 (与 Lane/KanbanBoard 对齐，缺陷整合进 Lane)
const KANBAN_STATUSES: WorkItemStatus[] = [
  "todo",
  "in_progress",
  "review",
  "blocked",
  "done",
  "wontfix",
];

// ---------------------------------------------------------------------
// 缺陷域状态 → 看板 6 列投影 (per SRS §4.4: 缺陷整合进 Lane 的看板列)
// 这只是视图投影, 不是业务状态推断: DefectStatus (缺陷域) 与 WorkItemStatus
// (看板列) 是两个域的事实, 映射仅用于列摆放。验证结论一律看
// latest_verification, 与看板列无关 (per DD §3.2.2)。
// ---------------------------------------------------------------------
const DEFECT_STATUS_TO_KANBAN: Record<string, WorkItemStatus> = {
  Triage: "todo",
  Confirmed: "todo",
  PendingInfo: "blocked",
  InProgress: "in_progress",
  PendingVerify: "review",
  Closed: "done",
  Reopened: "blocked",
  Duplicate: "wontfix",
  NoAction: "wontfix",
};

const DEFECT_PRIORITY_TO_WORKITEM: Record<string, WorkItemPriority> = {
  P0: "p0",
  P1: "p1",
  P2: "p2",
  P3: "p3",
  Unscheduled: "p3",
  // 降级路径 (GroupWorkItemView) 直接给 p0..p3
  p0: "p0",
  p1: "p1",
  p2: "p2",
  p3: "p3",
};

export function defectStatusToKanbanStatus(raw: string): WorkItemStatus {
  return (
    DEFECT_STATUS_TO_KANBAN[raw] ??
    DEFECT_STATUS_TO_KANBAN[raw.toLowerCase()] ??
    "todo"
  );
}

export function defectPriorityToWorkItemPriority(priority: string): WorkItemPriority {
  return DEFECT_PRIORITY_TO_WORKITEM[priority] ?? "p3";
}

/**
 * WorkItem 优先级 → 缺陷域优先级（per `CreateDefectCommand.priority`）。
 *
 * 缺陷域枚举：P0 / P1 / P2 / P3 / unscheduled
 * （与 `domain_defect::DefectPriority::as_str()` 逐值一致）。
 */
export function workItemPriorityToDefectPriority(
  priority: WorkItemPriority,
): string {
  switch (priority) {
    case "p0":
      return "P0";
    case "p1":
      return "P1";
    case "p2":
      return "P2";
    case "p3":
      return "P3";
    default:
      return "unscheduled";
  }
}

// ---------------------------------------------------------------------
// 扩展字段三态渲染 (per DD §3.2.2 / D-DI-06 / CON-DI-05):
//   value       — 读模型返回该列且有值
//   empty       — 读模型返回该列但为空 (null/"") = 字段已接入但未填写
//   unavailable — 读模型未返回该列 (未接入), 不得回退到占位常量或界面推断
// ---------------------------------------------------------------------
export type ExtensionFieldState = "value" | "empty" | "unavailable";

export function extensionFieldState(
  value: string | null | undefined,
  columnPresent: boolean,
): ExtensionFieldState {
  if (!columnPresent) return "unavailable";
  if (value == null || value === "") return "empty";
  return "value";
}

export function renderExtensionFieldText(
  value: string | null | undefined,
  columnPresent: boolean,
): string {
  const state = extensionFieldState(value, columnPresent);
  if (state === "value") return value as string;
  if (state === "empty") return "（未填写）";
  return "（未接入）";
}

// 验证状态专条 (per DD §3.2.2):
//   latest_verification 列缺失 (undefined) → （未接入）
//   null                                    → （未提交）
//   passed / failed                         → 真实验证结论
// 任何情况下不得由看板 status === "done" 推断验证结论。
export type VerificationState = "passed" | "failed" | "unsubmitted" | "unavailable";

export function verificationState(
  verification: DefectItem["latest_verification"],
): VerificationState {
  if (verification === undefined) return "unavailable";
  if (verification === null) return "unsubmitted";
  return verification.result === "passed" ? "passed" : "failed";
}

export function renderVerificationText(
  verification: DefectItem["latest_verification"],
): string {
  if (verification === undefined) return "（未接入）";
  if (verification === null) return "（未提交）";
  return verification.result === "passed"
    ? `已验证（${verification.fix_version}）`
    : `验证失败（${verification.fix_version}）`;
}

export interface DefectItem {
  id: string;
  title: string;
  phenomenon: string;
  /** 缺陷域原始状态 (REST 读模型 DefectListItem.status; 降级路径无) */
  defectStatus?: string;
  /** 看板 6 列投影状态 */
  status: WorkItemStatus;
  /**
   * 以下扩展字段: undefined = 读模型未返回该列 (未接入);
   * null/"" = 已接入但未填写。列存在性由 extensionColumns 判定。
   */
  actual_result?: string | null;
  expected_result?: string | null;
  reproduction_steps?: string | null;
  severity?: string;
  priority: string;
  assignee_id?: string | null;
  reporter_id: string;
  lane_id?: string | null;
  worktree_id?: string | null;
  branch_id?: string | null;
  branch_name?: string | null;
  fix_version?: string | null;
  lifecycle_version: number;
  created_at?: string;
  updated_at?: string;
  latest_verification?: {
    verifier_id: string;
    fix_version: string;
    method: string | null;
    result: "passed" | "failed";
    evidence_id: string | null;
    created_at: string;
  } | null;
  /** 读模型实际返回的扩展列名集合; 不在集合内 = 未接入 (per 三态规则) */
  extensionColumns?: string[];
}

export function hasExtensionColumn(item: DefectItem, column: string): boolean {
  return item.extensionColumns?.includes(column) ?? false;
}

// REST 读模型物理契约必含的扩展列 (DefectListItem)
const REST_BASE_COLUMNS = [
  "severity",
  "priority",
  "fix_version",
  "assignee_id",
  "lane_id",
  "worktree_id",
  "branch_name",
  "updated_at",
];

// 目标读模型列 (DD §3.2.2): 物理契约尚未返回, 列一旦出现即视为已接入
const REST_OPTIONAL_COLUMNS = [
  "phenomenon",
  "expected_result",
  "actual_result",
  "reproduction_steps",
  "created_at",
  "latest_verification",
] as const;

function restRowToDefectItem(row: DefectListItemView): DefectItem {
  const columns = new Set<string>(REST_BASE_COLUMNS);
  for (const key of REST_OPTIONAL_COLUMNS) {
    if (key in row) columns.add(key);
  }
  return {
    id: row.id,
    title: row.title,
    phenomenon: row.phenomenon ?? "",
    defectStatus: row.status,
    status: defectStatusToKanbanStatus(row.status),
    severity: row.severity,
    priority: row.priority,
    assignee_id: row.assignee_id,
    reporter_id: row.reporter_id,
    lane_id: row.lane_id,
    worktree_id: row.worktree_id,
    branch_id: null,
    branch_name: row.branch_name,
    fix_version: row.fix_version,
    lifecycle_version: row.lifecycle_version,
    created_at: row.created_at,
    updated_at: row.updated_at,
    actual_result: row.actual_result,
    expected_result: row.expected_result,
    reproduction_steps: row.reproduction_steps,
    latest_verification: row.latest_verification,
    extensionColumns: [...columns],
  };
}

// 降级路径 (GroupWorkItemView): 不含缺陷扩展列。
// 现象以 WorkItem.description 近似展示 (per DD §3.2.2 判定为「部分」来源);
// 其余扩展列一律按未接入渲染, 不硬编码占位。
export function groupItemToDefectItem(item: GroupWorkItemView): DefectItem {
  return {
    id: item.id,
    title: item.title,
    phenomenon: item.description,
    status: item.status,
    priority: item.priority,
    reporter_id: item.reporter_id,
    worktree_id: item.worktree_id,
    branch_id: item.branch_id,
    branch_name: item.branch_name,
    lifecycle_version: item.lifecycle_version,
    extensionColumns: ["phenomenon"],
  };
}

// ---------------------------------------------------------------------
// 降级路径 envelope 映射: worktreeGroupApi.listWorkItems 返回的是
// { worktree_id, project_id, work_items: [...] } 原始 envelope, 不是数组
// (旧代码把 envelope 当数组 spread, 恒抛错且被 catch 静默吞掉 — G-17 成因之一)。
// 映射逻辑与 groupProjection.mapWorkItems 对齐 (该函数未导出, 此处按契约本地映射)。
// ---------------------------------------------------------------------
interface DegradedWorkItemEnvelope {
  worktree_id: string;
  project_id: string;
  work_items: Array<Record<string, unknown>>;
}

const DEGRADED_STATUS_MAP: Record<string, WorkItemStatus> = {
  pending: "todo",
  claimed: "in_progress",
  in_progress: "in_progress",
  completed: "done",
  failed: "blocked",
  cancelled: "wontfix",
};

const DEGRADED_PRIORITY_MAP: Record<string, WorkItemPriority> = {
  low: "p3",
  medium: "p2",
  high: "p1",
  urgent: "p0",
};

function mapDegradedEnvelopeWorkItems(
  envelope: DegradedWorkItemEnvelope,
  worktreeId: string,
): GroupWorkItemView[] {
  return envelope.work_items.map((item) => {
    const id = requiredString(item.work_item_id, "work_item.work_item_id");
    const lifecycle = asRecord(item.lifecycle, "work_item.lifecycle");
    const itemType = requiredString(item.item_type, "work_item.item_type");
    const kind: WorkItemKind = ["story", "task", "bug", "spike", "epic"].includes(itemType)
      ? (itemType as WorkItemKind)
      : "task";
    const status = DEGRADED_STATUS_MAP[String(lifecycle.status)];
    const priority = DEGRADED_PRIORITY_MAP[String(item.priority)];
    const lifecycleVersion = Number(lifecycle.version);
    if (!status || !priority || !Number.isSafeInteger(lifecycleVersion) || lifecycleVersion < 1) {
      throw new Error("Group API returned an unsupported WorkItem state.");
    }
    return {
      id,
      project_id: envelope.project_id,
      worktree_id: worktreeId,
      key: id,
      title: requiredString(item.title, "work_item.title"),
      description: typeof item.description === "string" ? item.description : "",
      kind,
      status,
      priority,
      lifecycle_status: String(lifecycle.status) as WorkItemLifecycleStatus,
      lifecycle_version: lifecycleVersion,
      review_state: typeof lifecycle.review_state === "string" ? lifecycle.review_state : "none",
      active_worktree_id:
        typeof lifecycle.active_worktree_id === "string" ? lifecycle.active_worktree_id : null,
      labels: Array.isArray(item.labels)
        ? item.labels.filter((label): label is string => typeof label === "string")
        : [],
      reporter_id: requiredString(item.reporter_user_id, "work_item.reporter_user_id"),
      branch_id: typeof item.branch_id === "string" ? item.branch_id : null,
      branch_name: typeof item.branch_name === "string" ? item.branch_name : null,
    };
  });
}

function asRecord(value: unknown, label: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`Invalid Group API response: ${label}.`);
  }
  return value as Record<string, unknown>;
}

function requiredString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`Invalid Group API response: ${label}.`);
  }
  return value;
}

// ---------------------------------------------------------------------
// 数据来源 (N-01): REST 优先; 失败显式降级; 降级必须在 UI 可见
// ---------------------------------------------------------------------
export type DefectDataSource =
  | { kind: "rest"; serviceDegraded: boolean; degradationReason?: string | null }
  | { kind: "degraded-fallback"; restError: string }
  | { kind: "none" };

// 逐 worktree 加载结果 (per G-17: 单 worktree 失败不得静默吞掉)
export interface WorktreeLoadResult {
  worktreeId: string;
  ok: boolean;
  error?: string;
  items: GroupWorkItemView[];
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
  // 数据来源 + 降级路径逐 worktree 结果 (per N-01 / G-17)
  const [dataSource, setDataSource] = useState<DefectDataSource>({ kind: "none" });
  const [loadResults, setLoadResults] = useState<WorktreeLoadResult[] | null>(null);

  const selectedLaneId = navStore.selectedLaneId;
  const selectedWorktreeId = navStore.selectedWorktreeId;
  const projectId = navStore.selectedProjectId ?? "";
  const worktrees = storeApi.worktrees;
  const addWorkItem = storeApi.addWorkItem;

  // 加载缺陷: 优先 REST 读模型, 失败显式降级到多 worktree 路径 (per N-01 / G-14 / G-17)
  /**
   * 加载缺陷列表（per N-01 / G-14 / G-17）。
   *
   * - 优先 REST 读模型 `GET /api/v1/defects`
   * - 失败显式降级到多 worktree 路径，逐 worktree 结果可见
   * - 降级状态与原因都进 `dataSource`，UI 如实呈现
   *
   * 抽成 useCallback 是为了让 `handleOwnerCreate`（创建后刷新）复用同一条路径。
   */
  const reloadDefects = useCallback(async (): Promise<void> => {
  const hydrateStoreItems = (items: DefectItem[]) => {
    // waterfall 到 store: 使 WorkItemDetailDrawer view 模式能读到 currentWi
    for (const item of items) {
      try {
        addWorkItem({
          id: item.id,
          tenant_id: tenantId,
          project_id: projectId,
          title: item.title,
          description: item.phenomenon || "",
          kind: "bug" as const,
          status: item.status,
          priority: defectPriorityToWorkItemPriority(item.priority),
          reporter_id: item.reporter_id,
          labels: [],
          worktree_id: item.worktree_id ?? undefined,
          branch_id: item.branch_id ?? undefined,
          branch_name: item.branch_name ?? undefined,
          lifecycle_version: item.lifecycle_version,
        });
      } catch {
        // 水合单条失败不阻塞整体
      }
    }
  };

  const loadDefects = async () => {
    if (!projectId || !tenantId || !reporterId) {
      setDefects([]);
      setLoadResults(null);
      setDataSource({ kind: "none" });
      return;
    }

    // ---- 1. 优先: REST 读模型 GET /api/v1/defects (N-01) ----
    let restErrorText = "";
    try {
      const res = await listDefects({
        project_id: projectId,
        lane_id: selectedLaneId || undefined,
        page: 1,
        page_size: 200,
      });
      const defectItems = res.items.map(restRowToDefectItem);
      setDefects(defectItems);
      setLoadResults(null);
      setDataSource({
        kind: "rest",
        serviceDegraded: res.degraded,
        degradationReason: res.degradation_reason ?? null,
      });
      hydrateStoreItems(defectItems);
      return;
    } catch (restErr) {
      // 显式降级, 不静默 (per G-14 / N-01)
      restErrorText = restErr instanceof Error ? restErr.message : String(restErr);
    }

    // ---- 2. 降级路径: worktreeGroupApi 多 worktree (G-17 逐 worktree 结果) ----
    if (!worktreeGroupApi) {
      setDefects([]);
      setLoadResults(null);
      setDataSource({ kind: "degraded-fallback", restError: restErrorText });
      setError(
        `缺陷读模型不可用 (${restErrorText})，且 worktreeGroupApi 未接入。`,
      );
      return;
    }

    const targetWorktreeIds = selectedWorktreeId
      ? [selectedWorktreeId]
      : worktrees
          .filter(
            (w) =>
              w.project_id === projectId &&
              (!selectedLaneId || w.lane_id === selectedLaneId),
          )
          .map((w) => w.id);

    // 逐 worktree 收集结果: 单个失败不阻塞整体, 但失败范围必须可见 (G-17)
    const results: WorktreeLoadResult[] = await Promise.all(
      targetWorktreeIds.map(
        async (wtId): Promise<WorktreeLoadResult> => {
          try {
            const envelope =
              await worktreeGroupApi.listWorkItems<DegradedWorkItemEnvelope>(wtId);
            const items = mapDegradedEnvelopeWorkItems(envelope, wtId);
            return { worktreeId: wtId, ok: true, items };
          } catch (err) {
            return {
              worktreeId: wtId,
              ok: false,
              error: err instanceof Error ? err.message : String(err),
              items: [],
            };
          }
        },
      ),
    );

    const allItems = results.flatMap((r) => r.items);
    const bugs = allItems.filter((item) => item.kind === "bug");
    const defectItems = bugs.map(groupItemToDefectItem);
    hydrateStoreItems(defectItems);
    setDefects(defectItems);
    setLoadResults(results);
    setDataSource({ kind: "degraded-fallback", restError: restErrorText });
  };

  // 真正执行加载（内层函数定义完成后调用）
  await loadDefects();
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

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);

    reloadDefects()
      .catch((e: unknown) => {
        if (!cancelled) setError(e instanceof Error ? e.message : String(e));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [reloadDefects]);


  const filtered = useMemo(() => {
    return defects.filter((d) => {
      if (filter.status.length && !filter.status.includes(d.status))
        return false;
      if (
        filter.severity.length &&
        (!d.severity || !filter.severity.includes(d.severity))
      )
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
  /**
   * 新建缺陷 → owner 创建命令（per G-13 单一写入路径 / D-DI-04）。
   *
   * 走 `POST /api/v1/defects`（真实 owner，含幂等键 + 版本回执），
   * 成功后用回执刷新列表；失败时**不**静默落到本地 store，
   * 而是抛出让抽屉显示阻断原因。
   *
   * `idempotency_key` 用 `crypto.randomUUID()`：同一命令重试复用同一键，
   * 服务端按 key 去重（per FR-DI-020）。
   */
  const handleOwnerCreate = useCallback(
    async (command: WorkItemCreateCommand): Promise<string | null> => {
      const created = await createDefect({
        idempotency_key:
          typeof crypto !== "undefined" && "randomUUID" in crypto
            ? crypto.randomUUID()
            : `${command.project_id}:${command.title}:${Date.now()}`,
        tenant_id: command.tenant_id,
        project_id: command.project_id,
        lane_id: selectedLaneId ?? null,
        worktree_id: command.worktree_id ?? null,
        branch_name: command.branch_name,
        title: command.title,
        phenomenon: command.description,
        reporter_id: command.reporter_id,
        severity: "unknown",
        priority: workItemPriorityToDefectPriority(command.priority),
        labels: command.labels,
      });
      // 用回执刷新列表（权威事实来自 owner，不本地臆造）
      await reloadDefects();
      return created.id;
    },
    [selectedLaneId, reloadDefects],
  );

  const handleCreate = useCallback(() => {
    setSelectedDefect(null);
    // mode 级 + prop 级双通道锁定 (per DD §3.2.3 方案 A/B 兼容)
    setDrawerOpen({
      kind: "new",
      defaultStatus: "todo",
      defaultKind: "bug",
      lockKind: true,
    });
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

  const partialFailures = useMemo(
    () => (loadResults ?? []).filter((r) => !r.ok),
    [loadResults],
  );

  return (
    <div className="flex h-full bg-bg">
      {/* 左: 筛选 + 缺陷列表 */}
      <aside className="w-full lg:w-3/5 shrink-0 border-r border-line bg-bg-card overflow-y-auto p-4">
        <div className="flex items-center justify-between px-3 py-3 border-b border-line">
          <div className="flex items-center gap-2">
            <h2 className="text-sm font-bold text-ink">缺陷管理</h2>
            {/* 数据来源标注 (N-01: 降级必须可见; G-16: 服务端降级如实呈现) */}
            <span
              data-testid="defect-data-source"
              className="text-[10px] font-mono text-ink-mute"
            >
              {dataSource.kind === "rest"
                ? `来源：REST 读模型${dataSource.serviceDegraded ? " · 服务端标注降级 (InMemory)" : ""}`
                : dataSource.kind === "degraded-fallback"
                  ? "来源：降级路径"
                  : ""}
            </span>
          </div>
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

        {/* 扩展字段来源不可用顶部横幅 (per D-DI-06: 降级时标注来源不可用) */}
        {dataSource.kind === "degraded-fallback" && (
          <div
            data-testid="defect-source-unavailable-banner"
            role="alert"
            className="flex items-start gap-2 border-b border-warn/40 bg-warn/10 px-3 py-2 text-xs text-warn"
          >
            <AlertTriangle size={12} className="mt-0.5 shrink-0" />
            <div>
              缺陷读模型 (/api/v1/defects) 请求失败: {dataSource.restError}。
              已降级到 worktreeGroupApi 路径，缺陷扩展字段（实际结果/预期结果/修复版本/验证状态）来源不可用。
            </div>
          </div>
        )}
        {dataSource.kind === "rest" && dataSource.serviceDegraded && (
          <div
            data-testid="defect-source-unavailable-banner"
            role="alert"
            className="flex items-start gap-2 border-b border-warn/40 bg-warn/10 px-3 py-2 text-xs text-warn"
          >
            <AlertTriangle size={12} className="mt-0.5 shrink-0" />
            <div>
              服务端标注缺陷读模型降级 (InMemory):{" "}
              {dataSource.degradationReason ?? "数据不持久、无真实跨租户隔离"}
            </div>
          </div>
        )}

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
          {/* 部分失败提示 (G-17: 局部结果绝不冒充完整集合) */}
          {partialFailures.length > 0 && loadResults && (
            <div
              data-testid="defect-partial-failure"
              role="alert"
              className="rounded border border-warn/40 bg-warn/10 px-3 py-2 text-xs text-warn mb-2"
            >
              <div className="flex items-center gap-1 font-semibold">
                <AlertTriangle size={12} className="shrink-0" />
                <span>
                  部分结果: {partialFailures.length}/{loadResults.length} 个
                  worktree 加载失败, 当前列表不完整。
                </span>
              </div>
              <ul className="mt-1 space-y-0.5 pl-4 list-disc">
                {partialFailures.map((r) => (
                  <li
                    key={r.worktreeId}
                    data-testid={`defect-worktree-failure-${r.worktreeId}`}
                  >
                    <span className="font-mono">{r.worktreeId}</span>:{" "}
                    {r.error ?? "未知错误"}
                  </li>
                ))}
              </ul>
            </div>
          )}
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
                      <span>{d.severity ?? "（未接入）"}</span>
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
          onCreate={handleOwnerCreate}
        />

        {/* 缺陷扩展字段 (现象/实际结果/修复版本/验证状态)，仅在选中缺陷时展示 */}
        {selectedDefect && (
          <div className="max-w-2xl mx-auto mt-6 space-y-4">
            <div className="rounded-lg border border-line bg-bg-card p-3">
              <div className="flex items-center gap-2 mb-2">
                <FileText size={14} className="text-accent" />
                <span className="text-sm font-bold text-ink">缺陷扩展详情</span>
              </div>

              {/* 扩展字段来源未接入标注 (per D-DI-06: 在该区块顶部标注来源不可用) */}
              {(!hasExtensionColumn(selectedDefect, "latest_verification") ||
                !hasExtensionColumn(selectedDefect, "expected_result")) && (
                <div
                  data-testid="defect-extension-source-unavailable"
                  className="flex items-start gap-2 mb-3 rounded border border-warn/40 bg-warn/10 px-3 py-2 text-xs text-warn"
                >
                  <AlertTriangle size={12} className="mt-0.5 shrink-0" />
                  <span>
                    部分扩展字段来源未接入: 当前读模型未返回
                    {!hasExtensionColumn(selectedDefect, "latest_verification")
                      ? " 验证状态"
                      : ""}
                    {!hasExtensionColumn(selectedDefect, "expected_result")
                      ? " 预期结果/实际结果"
                      : ""}
                    等列，按「未接入」展示，不得由看板状态推断。
                  </span>
                </div>
              )}

              <div className="grid grid-cols-2 gap-4 mb-4">
                <div className="rounded-lg border border-line bg-bg-soft p-3">
                  <div className="text-[10px] font-mono uppercase tracking-wider text-ink-dim mb-1">
                    现象描述
                  </div>
                  <ExtensionFieldValue
                    value={selectedDefect.phenomenon || null}
                    columnPresent={hasExtensionColumn(selectedDefect, "phenomenon")}
                    testid="defect-field-phenomenon"
                  />
                </div>
                <div className="rounded-lg border border-line bg-bg-soft p-3">
                  <div className="text-[10px] font-mono uppercase tracking-wider text-ink-dim mb-1">
                    实际结果
                  </div>
                  <ExtensionFieldValue
                    value={selectedDefect.actual_result ?? null}
                    columnPresent={hasExtensionColumn(selectedDefect, "actual_result")}
                    testid="defect-field-actual-result"
                  />
                </div>
              </div>
              <div className="rounded-lg border border-line bg-bg-soft p-3">
                <div className="flex items-center gap-2 mb-2">
                  <CheckCircle2 size={14} className="text-accent" />
                  <span className="text-sm font-bold text-ink">修复与验证</span>
                </div>
                <div className="text-xs text-ink-mute space-y-1">
                  <div className="flex items-baseline gap-1">
                    <span>修复版本:</span>
                    <ExtensionFieldValue
                      value={selectedDefect.fix_version ?? null}
                      columnPresent={hasExtensionColumn(selectedDefect, "fix_version")}
                      testid="defect-field-fix-version"
                    />
                  </div>
                  <div className="flex items-baseline gap-1">
                    <span>验证状态:</span>
                    <VerificationFieldValue
                      verification={selectedDefect.latest_verification}
                      columnPresent={hasExtensionColumn(
                        selectedDefect,
                        "latest_verification",
                      )}
                      testid="defect-field-verification"
                    />
                  </div>
                  <div className="flex items-center gap-1">
                    <span>看板状态:</span>
                    <StatusPill value={selectedDefect.status} size="xs" />
                    {selectedDefect.defectStatus && (
                      <span className="text-[10px] text-ink-mute font-mono">
                        (缺陷域: {selectedDefect.defectStatus})
                      </span>
                    )}
                  </div>
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

// ---------------------------------------------------------------------
// 扩展字段三态渲染组件 (per DD §3.2.2 / D-DI-06)
// ---------------------------------------------------------------------
function ExtensionFieldValue({
  value,
  columnPresent,
  testid,
}: {
  value: string | null | undefined;
  columnPresent: boolean;
  testid: string;
}) {
  const state = extensionFieldState(value, columnPresent);
  return (
    <div
      data-testid={testid}
      data-field-state={state}
      className="text-xs text-ink whitespace-pre-wrap"
    >
      {renderExtensionFieldText(value, columnPresent)}
    </div>
  );
}

// 验证状态三态渲染 (专条: 列缺失 = 未接入; null = 未提交; 绝不从看板状态推断)
function VerificationFieldValue({
  verification,
  columnPresent,
  testid,
}: {
  verification: DefectItem["latest_verification"];
  columnPresent: boolean;
  testid: string;
}) {
  const state: VerificationState = columnPresent
    ? verificationState(verification)
    : "unavailable";
  return (
    <span
      data-testid={testid}
      data-field-state={state}
      className="text-xs text-ink"
    >
      {columnPresent ? renderVerificationText(verification) : "（未接入）"}
    </span>
  );
}
