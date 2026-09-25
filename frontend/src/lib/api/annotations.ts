// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/lib/api/annotations.ts
//
// Diff Annotation API client (per FR-ORCA-035 §11 v1.0, ULYS-211 / 父 ULYS-201).
//
// 对端: crates/agent-bridge/src/annotate.rs (commit 29bff3fc, 已在 origin/dev).
// 本文件的 6 个方法与 `AnnotationRegistry` 的 6 个 API 一一对应:
//   add / get / listForAgentRun / listForFile / delete / feedToAgent
//
// ⚠️ 缺标 (per 守门 #11 缺标比错标, 2026-09-23 实测):
//   `grep -ril annotat --include=*.rs crates bff` 只命中 annotate.rs + lib.rs —
//   agent-bridge 的 registry 是 in-memory, **尚无 HTTP/WebSocket 路由**
//   (annotate.rs 模块 doc 亦把 "前端 diff 标注 UI" 列为 P1 followup)。
//   故本文件是 wire contract 的**单一真源**: 类型逐字段镜像 annotate.rs 的
//   serde 形态, 由 src/mocks/handlers/annotations.ts (MSW) 承载。
//   后端补 axum 路由时按本文件的 path / status code 对齐即可, 无需改 UI。
//
// serde 形态对齐要点 (照 annotate.rs 逐条核对, 不是推测):
//   - AnnotationAuthor 带 #[serde(tag = "kind", rename_all = "snake_case")]
//     → internally tagged, 线上是 {"kind":"user"} 对象, **不是** 裸字符串 "user"
//   - line_range: Range<u32> → {"start":N,"end":M} 对象, **不是** 二元数组
//   - line_range 为 1-based 半开区间 [start, end): 标注单行 N 即 {start:N,end:N+1}
//     (annotate.rs 只拒 start==0 与 start>end, 故 start==end 的空区间合法)

const API_BASE = "/api/v2";

// =====================================================================
// 1. wire types (镜像 crates/agent-bridge/src/annotate.rs §1 entity)
// =====================================================================

/** 行号范围 — 1-based 半开区间 [start, end) (镜像 Rust `Range<u32>`) */
export interface LineRange {
  start: number;
  end: number;
}

/** 作者 — internally tagged enum (镜像 `AnnotationAuthor`) */
export type AnnotationAuthor = { kind: "user" } | { kind: "agent" };

export const AUTHOR_USER: AnnotationAuthor = { kind: "user" };
export const AUTHOR_AGENT: AnnotationAuthor = { kind: "agent" };

/** 单条 diff annotation (镜像 `DiffAnnotation`) */
export interface DiffAnnotation {
  /** annotation UUID */
  id: string;
  /** 相对 repo 根的文件路径 */
  file_path: string;
  line_range: LineRange;
  /** annotation 文本 (refinement signal, 进下一轮 prompt) */
  body: string;
  agent_run_id: string;
  /** RFC3339 */
  created_at: string;
  author: AnnotationAuthor;
}

/** POST /annotations 请求体 (镜像 `AnnotationRegistry::add` 入参) */
export interface CreateAnnotationRequest {
  file_path: string;
  line_range: LineRange;
  body: string;
  agent_run_id: string;
  author: AnnotationAuthor;
}

/** POST /annotations/feed 响应 (镜像 `feed_to_agent` 的 `Option<String>`) */
export interface FeedToAgentResponse {
  agent_run_id: string;
  /** annotations 为空时后端返回 None → 此处 null */
  prompt_fragment: string | null;
}

/**
 * 后端错误码 — 镜像 `AnnotationRegistryError` 3 variant
 * (per 守门 #6 v2 6-field schema 的 code 字段)
 */
export type AnnotationErrorCode =
  | "not_found"
  | "invalid_field"
  | "agent_run_not_found";

export interface AnnotationErrorBody {
  code: AnnotationErrorCode;
  message: string;
}

/** fetch 失败时抛出; 保留 HTTP status + 后端 code 供 UI 分支 */
export class AnnotationApiError extends Error {
  readonly status: number;
  readonly code: AnnotationErrorCode | null;

  constructor(status: number, message: string, code: AnnotationErrorCode | null) {
    super(message);
    this.name = "AnnotationApiError";
    this.status = status;
    this.code = code;
  }
}

// =====================================================================
// 2. line_range 工具 (半开区间 ↔ UI 行号, off-by-one 的唯一落点)
// =====================================================================

/** 单行 N → [N, N+1) */
export function singleLineRange(line: number): LineRange {
  return { start: line, end: line + 1 };
}

/** UI 选区 [a, b] (闭区间, 顺序任意) → [min, max+1) */
export function lineSpanRange(anchor: number, head: number): LineRange {
  const start = Math.min(anchor, head);
  const end = Math.max(anchor, head);
  return { start, end: end + 1 };
}

/** 该 range 是否覆盖行号 line (半开区间: end 不含) */
export function rangeCoversLine(range: LineRange, line: number): boolean {
  return line >= range.start && line < range.end;
}

/** 人读形态: [10,11) → "10"; [10,15) → "10-14"; 空区间 [10,10) → "10" */
export function formatLineRange(range: LineRange): string {
  const lastLine = range.end - 1;
  return lastLine <= range.start
    ? `${range.start}`
    : `${range.start}-${lastLine}`;
}

/** 与 annotate.rs `validate_line_range` 同规则的前置校验 (省一次往返) */
export function validateLineRange(range: LineRange): string | null {
  if (!Number.isInteger(range.start) || !Number.isInteger(range.end)) {
    return "line_range must be integers";
  }
  if (range.start === 0) return "line_range.start must be >= 1 (1-based)";
  if (range.start > range.end) {
    return `line_range.start (${range.start}) > end (${range.end})`;
  }
  return null;
}

/**
 * 本地复刻 `DiffAnnotation::to_prompt_fragment`
 * (per annotate.rs 的 6 行格式; 用于提交前预览, 真值仍以后端 feed_to_agent 为准)
 */
export function toPromptFragment(a: DiffAnnotation): string {
  return (
    `[Diff Annotation]\n` +
    `File: ${a.file_path}\n` +
    `Lines: ${a.line_range.start}-${a.line_range.end}\n` +
    `By: ${a.author.kind}\n` +
    `At: ${a.created_at}\n` +
    `Body: ${a.body}\n`
  );
}

// =====================================================================
// 3. transport
// =====================================================================

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    headers: { "Content-Type": "application/json", ...(init?.headers || {}) },
    ...init,
  });
  if (!res.ok) {
    let code: AnnotationErrorCode | null = null;
    let message = res.statusText;
    const text = await res.text();
    if (text) {
      try {
        const body = JSON.parse(text) as Partial<AnnotationErrorBody>;
        code = body.code ?? null;
        message = body.message ?? text;
      } catch {
        message = text;
      }
    }
    throw new AnnotationApiError(res.status, message, code);
  }
  if (res.status === 204) return undefined as T;
  return (await res.json()) as T;
}

export const annotationsApi = {
  /** `AnnotationRegistry::add` — 在 diff 上 drop comment */
  add: (req: CreateAnnotationRequest) => {
    const invalid = validateLineRange(req.line_range);
    if (invalid) {
      return Promise.reject(new AnnotationApiError(400, invalid, "invalid_field"));
    }
    return request<DiffAnnotation>(`/annotations`, {
      method: "POST",
      body: JSON.stringify(req),
    });
  },

  /** `AnnotationRegistry::get` */
  get: (id: string) => request<DiffAnnotation>(`/annotations/${id}`),

  /** `AnnotationRegistry::list_for_agent_run` — created_at ASC */
  listForAgentRun: (agentRunId: string) =>
    request<DiffAnnotation[]>(
      `/annotations?agent_run_id=${encodeURIComponent(agentRunId)}`,
    ),

  /** `AnnotationRegistry::list_for_file` — 跨 agent_run */
  listForFile: (filePath: string) =>
    request<DiffAnnotation[]>(
      `/annotations?file_path=${encodeURIComponent(filePath)}`,
    ),

  /** `AnnotationRegistry::delete` — 204 */
  delete: (id: string) =>
    request<void>(`/annotations/${id}`, { method: "DELETE" }),

  /**
   * `AnnotationRegistry::feed_to_agent` — 把 agent_run 下全部 annotation
   * 合成下一轮 prompt fragment (per FR-ORCA-035 "structured feedback 直接回到
   * agent session 的下一轮 prompt")
   */
  feedToAgent: (agentRunId: string) =>
    request<FeedToAgentResponse>(
      `/annotations/feed?agent_run_id=${encodeURIComponent(agentRunId)}`,
      { method: "POST" },
    ),
};

// =====================================================================
// 4. WebSocket 实时推送 (可选增强)
// =====================================================================

/** ws 事件 — 另一端 (agent refinement loop) 增删 annotation 时推送 */
export type AnnotationEvent =
  | { type: "annotation_added"; annotation: DiffAnnotation }
  | { type: "annotation_deleted"; id: string };

export interface SubscribeOptions {
  onEvent: (ev: AnnotationEvent) => void;
  onError?: (err: unknown) => void;
}

/**
 * 订阅某 agent_run 的 annotation 变更。
 *
 * ⚠️ 缺标: agent-bridge 目前无 ws endpoint (见文件头)。本函数在 WebSocket
 * 不可用 / 连接失败时**静默降级**为 no-op —— UI 仍靠 REST 轮询式刷新工作,
 * 不因缺后端而崩。返回 unsubscribe。
 */
export function subscribeAnnotations(
  agentRunId: string,
  opts: SubscribeOptions,
): () => void {
  if (typeof window === "undefined" || typeof WebSocket === "undefined") {
    return () => undefined;
  }
  let socket: WebSocket;
  try {
    const proto = window.location.protocol === "https:" ? "wss:" : "ws:";
    socket = new WebSocket(
      `${proto}//${window.location.host}${API_BASE}/annotations/ws` +
        `?agent_run_id=${encodeURIComponent(agentRunId)}`,
    );
  } catch (err) {
    opts.onError?.(err);
    return () => undefined;
  }

  socket.onmessage = (ev) => {
    try {
      opts.onEvent(JSON.parse(String(ev.data)) as AnnotationEvent);
    } catch (err) {
      opts.onError?.(err);
    }
  };
  socket.onerror = (err) => opts.onError?.(err);

  return () => {
    // CONNECTING(0) 下 close() 合法, 浏览器会在握手完成后立即关
    if (socket.readyState !== WebSocket.CLOSED) socket.close();
  };
}
