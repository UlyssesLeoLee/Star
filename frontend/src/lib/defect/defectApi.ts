// =====================================================================
// defectApi.ts — 缺陷域真实 REST 封装 (per DD-DEFECT-INTELLIGENCE-001 §1.0)
//
// REST 契约 (crates/star-api-rest/src/routes/defects.rs, 2026-11-24):
//   GET  /api/v1/defects?project_id&lane_id&page&page_size
//        → { data: { items, total, page, page_size, degraded, degradation_reason? }, meta }
//   POST /api/v1/defects
//        → { data: Defect, meta }   (CreateDefectCommand 请求体)
//
// 约定 (沿用 notificationsApi 既定纪律):
//   1. 统一走 realFetch (@/mocks/real-mode): base_url 来自 localStorage
//      `real_api_base` 或 NEXT_PUBLIC_API_BASE_URL, Bearer auth 自动注入。
//   2. real-mode 未开启 → 抛 DefectApiError(real_mode_disabled),
//      绝不回落 mock 数据冒充真实。
//   3. 错误统一映射为 DefectApiError (code + status)。缺陷页据此显式降级到
//      worktreeGroupApi 路径并在 UI 标注「来源：降级路径」(per N-01 / G-14)。
// =====================================================================

import { realFetch, getRealModeState } from "@/mocks/real-mode";

const API_ROOT = "/api/v1";

export type DefectApiErrorCode =
  | "real_mode_disabled"
  | "network_error"
  | "http_error"
  | "unauthorized"
  | "parse_error";

export class DefectApiError extends Error {
  readonly code: DefectApiErrorCode;
  readonly status: number | null;

  constructor(message: string, code: DefectApiErrorCode, status: number | null = null) {
    super(message);
    this.name = "DefectApiError";
    this.code = code;
    this.status = status;
  }
}

/** real-mode 未开启时直接失败 (诚实未接入, 不静默降级 mock) */
function assertRealModeEnabled(): void {
  if (!getRealModeState().enabled) {
    throw new DefectApiError(
      "缺陷读模型未接入: real-mode 未开启 (设置 localStorage `use_real_api=true` 或 NEXT_PUBLIC_USE_REAL_API)",
      "real_mode_disabled",
    );
  }
}

function errText(err: unknown): string {
  if (err instanceof Error) return err.message;
  return String(err);
}

/** 后端统一成功响应 envelope (RestResponse<T>: { data, meta }) */
interface RestEnvelope<T> {
  data: T;
  meta?: unknown;
}

async function requestJson<T>(path: string, init: RequestInit = {}): Promise<RestEnvelope<T>> {
  assertRealModeEnabled();
  let res: Response;
  try {
    res = await realFetch(`${API_ROOT}${path}`, init);
  } catch (err) {
    throw new DefectApiError(`缺陷读模型不可达: ${errText(err)}`, "network_error");
  }
  if (res.status === 401 || res.status === 403) {
    throw new DefectApiError(
      `HTTP ${res.status} 未授权: 缺陷读模型拒绝访问 (检查 real_api_key / 会话能力)`,
      "unauthorized",
      res.status,
    );
  }
  if (!res.ok) {
    throw new DefectApiError(`HTTP ${res.status} ${res.statusText || ""}`.trim(), "http_error", res.status);
  }
  try {
    return (await res.json()) as RestEnvelope<T>;
  } catch (err) {
    throw new DefectApiError(`缺陷读模型响应解析失败: ${errText(err)}`, "parse_error", res.status);
  }
}

function buildQuery(params: Record<string, string | number | undefined>): string {
  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value === undefined || value === null || value === "") continue;
    search.set(key, String(value));
  }
  const qs = search.toString();
  return qs ? `?${qs}` : "";
}

// ── 读模型视图 ───────────────────────────────────────────────────────

/**
 * 验证记录视图 (per DD §3.2.2 目标读模型)。
 * 物理列表契约 (DefectListItem) 尚未返回该列 → 前端按「未接入」渲染。
 */
export interface DefectVerificationView {
  verifier_id: string;
  fix_version: string;
  method: string | null;
  result: "passed" | "failed";
  evidence_id: string | null;
  created_at: string;
}

/**
 * 缺陷列表项 (物理契约: domain_defect::DefectListItem)。
 * `status` 为缺陷域状态 serde 名 ("Triage" / "Confirmed" / "InProgress" /
 * "PendingVerify" / "Closed" / "PendingInfo" / "Duplicate" / "NoAction" / "Reopened"),
 * 与看板 6 列 (WorkItemStatus) 是两个域的事实。
 *
 * 目标读模型扩展列 (DD §3.2.2 / D-DI-03): 物理契约尚未返回
 * phenomenon / expected_result / actual_result / reproduction_steps /
 * created_at / latest_verification。一旦后端读模型补齐, 前端按
 * 「列存在 = 已接入」直接消费, 无需改本模块调用方。
 */
export interface DefectListItemView {
  id: string;
  title: string;
  status: string;
  severity: string; // "S1".."S4" | "Unknown"
  priority: string; // "P0".."P3" | "Unscheduled"
  reporter_id: string;
  assignee_id: string | null;
  lane_id: string | null;
  worktree_id: string | null;
  branch_name: string | null;
  fix_version: string | null;
  lifecycle_version: number;
  updated_at: string;
  // —— 目标读模型扩展列 (物理契约尚未返回; 列缺失 = 未接入) ——
  phenomenon?: string;
  expected_result?: string | null;
  actual_result?: string | null;
  reproduction_steps?: string | null;
  created_at?: string;
  latest_verification?: DefectVerificationView | null;
}

export interface ListDefectsParams {
  project_id: string;
  lane_id?: string;
  page?: number;
  page_size?: number;
}

export interface DefectListData {
  items: DefectListItemView[];
  total: number;
  page: number;
  page_size: number;
  /** 服务端标注降级 (InMemory, 数据不持久) — 前端必须如实呈现, per G-16 */
  degraded: boolean;
  degradation_reason?: string | null;
}

/**
 * GET /api/v1/defects — 缺陷列表读模型。
 * 失败抛 DefectApiError; 调用方负责显式降级与来源标注。
 */
export async function listDefects(params: ListDefectsParams): Promise<DefectListData> {
  const res = await requestJson<DefectListData>(
    `/defects${buildQuery({
      project_id: params.project_id,
      lane_id: params.lane_id,
      page: params.page ?? 1,
      page_size: params.page_size ?? 50,
    })}`,
  );
  return res.data;
}

// ── 创建命令 ─────────────────────────────────────────────────────────

/** POST /api/v1/defects 请求体 (与 CreateDefectCommand 逐字段对齐) */
export interface CreateDefectInput {
  idempotency_key?: string;
  tenant_id: string;
  project_id: string;
  lane_id?: string | null;
  worktree_id?: string | null;
  branch_id?: string | null;
  branch_name?: string | null;
  title: string;
  phenomenon: string;
  expected_result?: string | null;
  actual_result?: string | null;
  reproduction_steps?: string | null;
  impact_scope?: string | null;
  discovery_stage?: string | null;
  severity?: string;
  priority?: string;
  reporter_id: string;
  environment?: string | null;
  labels?: string[];
  discovery_version?: string | null;
}

/** POST /api/v1/defects 回执 (domain_defect::Defect) */
export interface DefectDetailView {
  id: string;
  tenant_id: string;
  project_id: string;
  lane_id: string | null;
  worktree_id: string | null;
  branch_id: string | null;
  branch_name: string | null;
  title: string;
  phenomenon: string;
  expected_result: string | null;
  actual_result: string | null;
  reproduction_steps: string | null;
  impact_scope: string | null;
  discovery_stage: string | null;
  severity: string;
  priority: string;
  status: string;
  reporter_id: string;
  fixer_id: string | null;
  assignee_id: string | null;
  environment: string | null;
  labels: string[];
  discovery_version: string | null;
  fix_version: string | null;
  lifecycle_version: number;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
}

/**
 * POST /api/v1/defects — owner 创建命令 (per BD §3.5 单一写入路径)。
 * 缺陷页「新建缺陷」接线项: 创建提交路径应改走本命令并以回执刷新列表
 * (per DD §3.2.3 / D-DI-04 / G-13); 本地 store.addWorkItem 仅作降级。
 */
export async function createDefect(input: CreateDefectInput): Promise<DefectDetailView> {
  const res = await requestJson<DefectDetailView>("/defects", {
    method: "POST",
    body: JSON.stringify(input),
  });
  return res.data;
}
