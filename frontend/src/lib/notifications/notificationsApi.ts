/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/lib/notifications/notificationsApi.ts"})
MERGE (realMode:File {path:"frontend/src/mocks/real-mode.ts"})
MERGE (notifTypes:File {path:"frontend/src/types/notifications.ts"})
MERGE (listNotifications:Symbol {id:"frontend/src/lib/notifications/notificationsApi.ts::listNotifications",kind:"function"})
MERGE (markRead:Symbol {id:"frontend/src/lib/notifications/notificationsApi.ts::markRead",kind:"function"})
MERGE (markUnread:Symbol {id:"frontend/src/lib/notifications/notificationsApi.ts::markUnread",kind:"function"})
MERGE (fetchUnreadCount:Symbol {id:"frontend/src/lib/notifications/notificationsApi.ts::fetchUnreadCount",kind:"function"})
MERGE (listComments:Symbol {id:"frontend/src/lib/notifications/notificationsApi.ts::listComments",kind:"function"})
MERGE (addComment:Symbol {id:"frontend/src/lib/notifications/notificationsApi.ts::addComment",kind:"function"})
MERGE (deleteComment:Symbol {id:"frontend/src/lib/notifications/notificationsApi.ts::deleteComment",kind:"function"})
MERGE (self)-[:IMPORTS]->(realMode)
MERGE (self)-[:IMPORTS]->(notifTypes)
MERGE (self)-[:DEFINES]->(listNotifications)
MERGE (self)-[:DEFINES]->(markRead)
MERGE (self)-[:DEFINES]->(markUnread)
MERGE (self)-[:DEFINES]->(fetchUnreadCount)
MERGE (self)-[:DEFINES]->(listComments)
MERGE (self)-[:DEFINES]->(addComment)
MERGE (self)-[:DEFINES]->(deleteComment)
@endcypher
*/

// =====================================================================
// notificationsApi.ts — 真实通知服务 /api/v1 fetch 封装 (per 2026-11-17)
//
// 约定 (沿用仓库 real-mode 既有纪律):
//   1. 统一走 realFetch (src/mocks/real-mode.ts): base_url 来自
//      localStorage `real_api_base` 或 NEXT_PUBLIC_API_BASE_URL,
//      Bearer auth 自动注入。
//   2. real-mode 未开启 → 抛 NotificationsApiError(real_mode_disabled),
//      绝不回落 mock 数据冒充真实。
//   3. 错误统一映射为 NotificationsApiError (code + status),
//      页面层用 StatusNotice.inferNoticeState 映射三态。
//
// REST 契约 (与后端 lane 逐字段对齐,
//   crates/star-api-rest/src/routes/notifications.rs, 2026-11-17):
//   GET    /api/v1/notifications?tenant_id&kind&unread_only&limit&offset
//          → { data: { notifications, total, limit, offset }, meta }
//   PATCH  /api/v1/notifications/{id}  {read:bool}
//          → { data: { notification }, meta }
//   POST   /api/v1/notifications/{id}/read → { data: { notification }, meta }
//   GET    /api/v1/notifications/unread-count?tenant_id → { data: { count }, meta }
//   GET    /api/v1/comments?target_type&target_id → { data: { comments, total }, meta }
//   POST   /api/v1/comments  {tenant_id,target_type,target_id,body,author_name?}
//          → { data: { comment }, meta } (作者取 X-User-Id 头)
//   DELETE /api/v1/comments/{id} → { data: { deleted, comment_id }, meta }
// =====================================================================

import { realFetch, getRealModeState } from "@/mocks/real-mode";
import type { Notification, Comment } from "@/types/notifications";

const API_ROOT = "/api/v1";

export type NotificationsApiErrorCode =
  | "real_mode_disabled"
  | "network_error"
  | "http_error"
  | "unauthorized"
  | "parse_error";

export class NotificationsApiError extends Error {
  readonly code: NotificationsApiErrorCode;
  readonly status: number | null;

  constructor(message: string, code: NotificationsApiErrorCode, status: number | null = null) {
    super(message);
    this.name = "NotificationsApiError";
    this.code = code;
    this.status = status;
  }
}

/** real-mode 未开启时直接失败 (诚实未接入, 不静默降级 mock) */
function assertRealModeEnabled(): void {
  if (!getRealModeState().enabled) {
    throw new NotificationsApiError(
      "真实通知服务未接入: real-mode 未开启 (设置 localStorage `use_real_api=true` 或 NEXT_PUBLIC_USE_REAL_API)",
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
    throw new NotificationsApiError(
      `真实通知服务不可达: ${errText(err)}`,
      "network_error",
    );
  }
  if (res.status === 401 || res.status === 403) {
    throw new NotificationsApiError(
      `HTTP ${res.status} 未授权: 通知服务拒绝访问 (检查 real_api_key)`,
      "unauthorized",
      res.status,
    );
  }
  if (!res.ok) {
    throw new NotificationsApiError(
      `HTTP ${res.status} ${res.statusText || ""}`.trim(),
      "http_error",
      res.status,
    );
  }
  try {
    return (await res.json()) as RestEnvelope<T>;
  } catch (err) {
    throw new NotificationsApiError(
      `通知服务响应解析失败: ${errText(err)}`,
      "parse_error",
      res.status,
    );
  }
}

function buildQuery(params: Record<string, string | number | boolean | undefined>): string {
  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value === undefined || value === null || value === "") continue;
    search.set(key, String(value));
  }
  const qs = search.toString();
  return qs ? `?${qs}` : "";
}

// ── 通知 ─────────────────────────────────────────────────────────────

export interface ListNotificationsParams {
  tenant_id?: string;
  kind?: string;
  unread_only?: boolean;
  limit?: number;
  offset?: number;
}

interface NotificationListData {
  notifications: Notification[];
  total: number;
  limit: number;
  offset: number;
}

export async function listNotifications(
  params: ListNotificationsParams = {},
): Promise<Notification[]> {
  const res = await requestJson<NotificationListData>(
    `/notifications${buildQuery(params as Record<string, string | number | boolean | undefined>)}`,
  );
  return res.data.notifications;
}

export async function markRead(id: string): Promise<Notification> {
  const res = await requestJson<{ notification: Notification }>(
    `/notifications/${encodeURIComponent(id)}`,
    { method: "PATCH", body: JSON.stringify({ read: true }) },
  );
  return res.data.notification;
}

export async function markUnread(id: string): Promise<Notification> {
  const res = await requestJson<{ notification: Notification }>(
    `/notifications/${encodeURIComponent(id)}`,
    { method: "PATCH", body: JSON.stringify({ read: false }) },
  );
  return res.data.notification;
}

export async function fetchUnreadCount(params: { tenant_id?: string } = {}): Promise<number> {
  const res = await requestJson<{ count: number }>(
    `/notifications/unread-count${buildQuery(params as Record<string, string | number | boolean | undefined>)}`,
  );
  return res.data.count;
}

// ── 评论 ─────────────────────────────────────────────────────────────

interface CommentListData {
  comments: Comment[];
  total: number;
}

export async function listComments(targetType: string, targetId: string): Promise<Comment[]> {
  const res = await requestJson<CommentListData>(
    `/comments${buildQuery({ target_type: targetType, target_id: targetId })}`,
  );
  return res.data.comments;
}

export interface AddCommentInput {
  tenant_id: string;
  target_type: string;
  target_id: string;
  body: string;
  author_name?: string;
}

export async function addComment(input: AddCommentInput): Promise<Comment> {
  const res = await requestJson<{ comment: Comment }>("/comments", {
    method: "POST",
    body: JSON.stringify(input),
  });
  return res.data.comment;
}

export async function deleteComment(id: string): Promise<void> {
  await requestJson<{ deleted: boolean; comment_id: string }>(
    `/comments/${encodeURIComponent(id)}`,
    { method: "DELETE" },
  );
}
