// frontend/src/lib/ops-api.ts
//
// F-02 端到端: Ops Console 8 REST endpoint fetch wrapper
// per docs/briefs/ops-f02-log-ai-impl.md
// per docs/basic-design/OPS-BASIC-DESIGN-001.md §3
//
// 8 端点:
// - F-01 Cluster (4): listReleases / canary / rollback / status
// - F-02 LogAI (2): upload / getAnalysis
// - F-03 Metrics (1): summary
// - F-04 Docs (1): list
//
// 守门 #6 v2: error handling 跟 error.rs 6-field 对齐 (retriable 标记)
// 守门 #5 v2: API key 不入 log / 不 print

const OPS_API_BASE = "/api/ops";

/** log 级别 (per OPS-BASIC-DESIGN §4.1) */
export type LogLevel = "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR";

/** log 上传 body (per ops_api.rs::LogUploadBody) */
export interface LogUploadRequest {
  source: string;
  level_filter?: LogLevel[];
  content: string;
  /** 跨调用链追踪 ID (per 守门 #5 v2) */
  trace_id?: string;
}

/** log 上传 ack (per ops_api.rs::UploadAck) */
export interface LogUploadAck {
  log_id: string;
  entry_count: number;
  analysis_triggered: boolean;
  trace_id: string | null;
}

/** AI 异常 (per OPS-BASIC-DESIGN §3.2) */
export interface LogAnomaly {
  type: string;
  timestamp: string;
  level: LogLevel;
  message_excerpt: string;
}

/** AI 建议 (per OPS-BASIC-DESIGN §3.2) */
export interface LogSuggestion {
  id: string;
  text: string;
  confidence: number;
}

/** AI 分析结果 (per OPS-BASIC-DESIGN §3.2) */
export interface LogAnalysis {
  log_id: string;
  summary: string;
  anomalies: LogAnomaly[];
  suggestions: LogSuggestion[];
  /** 0.0-1.0, 守门 #23: < 0.5 必标 needs_review */
  confidence: number;
  /** AI 通道名: mock | openai | anthropic | openai_stub | anthropic_stub */
  generated_by: string;
}

/** Ops meta (per ops_api.rs::OpsMeta) */
export interface OpsMeta {
  stub: boolean;
  total: number | null;
  hint: string | null;
  ai_channel: string | null;
  analysis_triggered: boolean | null;
  needs_review: boolean | null;
  entry_count: number | null;
  phase: string | null;
}

/** 通用 Ops 响应包装 (per ops_api.rs::OpsResponse) */
export interface OpsResponse<T> {
  data: T;
  meta: OpsMeta;
}

/** Ops 错误响应 (per ops_api.rs::OpsErrorBody 6-field) */
export interface OpsError {
  code: string;
  message: string;
  source_module: string;
  source_kind: "internal" | "external" | "policy" | "validation";
  /** 守门 #6 v2: retriable 必返 (前端可决定 retry 策略) */
  retriable: boolean;
  hint: string | null;
}

/** Ops 错误响应包装 (per ops_api.rs::OpsErrorResponse) */
export interface OpsErrorResponse {
  error: OpsError;
}

/** 内部: 带 trace_id 跟 retriable 标记的 fetch 错误 */
export class OpsApiError extends Error {
  readonly code: string;
  readonly retriable: boolean;
  readonly trace_id: string | null;
  readonly status: number;

  constructor(opts: {
    code: string;
    message: string;
    retriable: boolean;
    trace_id: string | null;
    status: number;
  }) {
    super(opts.message);
    this.name = "OpsApiError";
    this.code = opts.code;
    this.retriable = opts.retriable;
    this.trace_id = opts.trace_id;
    this.status = opts.status;
  }
}

/** 内部: 生成 trace_id (per 守门 #5 v2 跨调用链追踪) */
function generateTraceId(): string {
  // 简单 UUIDv4-ish (前端不引 uuid 库, 16 字符 hex 够用)
  return (
    "trace-" +
    Date.now().toString(36) +
    "-" +
    Math.random().toString(36).slice(2, 10)
  );
}

/** 内部: 统一 fetch 包装 (加 trace_id + 解析 error 6-field) */
async function opsFetch<T>(
  path: string,
  init: RequestInit = {},
  traceId?: string,
): Promise<T> {
  const tid = traceId ?? generateTraceId();
  const headers = new Headers(init.headers);
  headers.set("content-type", "application/json");
  headers.set("x-trace-id", tid);
  const res = await fetch(`${OPS_API_BASE}${path}`, { ...init, headers });
  if (!res.ok) {
    // 守门 #5 v2: 不打印 trace_id 到 console.error (避免 secret 反射)
    let body: OpsErrorResponse | null = null;
    try {
      body = (await res.json()) as OpsErrorResponse;
    } catch {
      // 响应不是 JSON, 走 generic err
    }
    if (body?.error) {
      throw new OpsApiError({
        code: body.error.code,
        message: body.error.message,
        retriable: body.error.retriable,
        trace_id: tid,
        status: res.status,
      });
    }
    throw new OpsApiError({
      code: `HTTP_${res.status}`,
      message: `${path} ${res.statusText || res.status}`,
      retriable: res.status >= 500,
      trace_id: tid,
      status: res.status,
    });
  }
  return (await res.json()) as T;
}

// ============ F-01 Cluster (4 端点) ============

/** Helm Release 列表 (per OPS-BASIC-DESIGN §3.1) */
export interface HelmRelease {
  name: string;
  namespace: string;
  revision: number;
  chart: string;
  app_version: string;
  status: string;
  updated_at: string;
}

export async function listReleases(): Promise<OpsResponse<HelmRelease[]>> {
  return opsFetch<OpsResponse<HelmRelease[]>>("/cluster/releases");
}

export interface CanaryRequest {
  release_name: string;
  target_revision: number;
  canary_weight: number;
}

export interface CanaryAck {
  action_id: string;
  status: string;
}

export async function canary(
  req: CanaryRequest,
  traceId?: string,
): Promise<OpsResponse<CanaryAck>> {
  return opsFetch<OpsResponse<CanaryAck>>(
    "/cluster/canary",
    { method: "POST", body: JSON.stringify(req) },
    traceId,
  );
}

export interface RollbackRequest {
  release_name: string;
  target_revision: number;
}

export async function rollback(
  req: RollbackRequest,
  traceId?: string,
): Promise<OpsResponse<CanaryAck>> {
  return opsFetch<OpsResponse<CanaryAck>>(
    "/cluster/rollback",
    { method: "POST", body: JSON.stringify(req) },
    traceId,
  );
}

export interface ClusterStatus {
  release_name: string;
  phase: string;
  replicas: { ready: number; desired: number };
}

export async function clusterStatus(): Promise<OpsResponse<ClusterStatus>> {
  return opsFetch<OpsResponse<ClusterStatus>>("/cluster/status");
}

// ============ F-02 LogAI (2 端点) ============

/** 上传 log, 触发 AI 分析 (per ops_api.rs::log_upload) */
export async function uploadLog(
  req: LogUploadRequest,
  traceId?: string,
): Promise<OpsResponse<LogUploadAck>> {
  return opsFetch<OpsResponse<LogUploadAck>>(
    "/log/upload",
    { method: "POST", body: JSON.stringify(req) },
    traceId,
  );
}

/** 查 AI 分析结果 (per ops_api.rs::log_analysis) */
export async function getAnalysis(
  logId: string,
  traceId?: string,
): Promise<OpsResponse<LogAnalysis>> {
  return opsFetch<OpsResponse<LogAnalysis>>(
    `/log/analysis/${encodeURIComponent(logId)}`,
    {},
    traceId,
  );
}

// ============ F-03 Metrics (1 端点) ============

/** 趋势方向 (3 态, per ops_api.rs::metrics::TrendDirection) */
export type TrendDirection = "rising" | "stable" | "falling";

/** 运维指标 (per OPS-BASIC-DESIGN §3.3 + star-telemetry 5 KPI) */
export interface OpsMetric {
  name: string;
  value: number;
  unit: string;
  /** 趋势方向: rising / stable / falling */
  trend: TrendDirection;
  /** 最后更新时间 (ISO 8601 UTC) */
  last_updated: string;
}

export async function metricsSummary(): Promise<OpsResponse<OpsMetric[]>> {
  return opsFetch<OpsResponse<OpsMetric[]>>("/metrics/summary");
}

// ============ F-04 Docs (1 端点) ============

export interface DocRef {
  path: string;
  title: string;
  category: string;
  updated_at: string;
}

export interface DocsList {
  data: DocRef[];
  meta: { stub: boolean; total: number };
}

export async function listDocs(): Promise<OpsResponse<DocsList>> {
  return opsFetch<OpsResponse<DocsList>>("/docs");
}
