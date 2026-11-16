/*
@cypher schema=1 source_sha256=0dceb8fb25413b2073a4e1bc4a98ca10bb2ed7ce5eae6654285ed034c56f4fe7
MERGE (self:File {path:"frontend/src/types/notifications.ts"})
MERGE (notification:Type {id:"frontend/src/types/notifications.ts::Notification",kind:"interface"})
MERGE (comment:Type {id:"frontend/src/types/notifications.ts::Comment",kind:"interface"})
MERGE (isNotification:Symbol {id:"frontend/src/types/notifications.ts::isNotification",kind:"function"})
MERGE (isComment:Symbol {id:"frontend/src/types/notifications.ts::isComment",kind:"function"})
MERGE (self)-[:DEFINES]->(notification)
MERGE (self)-[:DEFINES]->(comment)
MERGE (self)-[:DEFINES]->(isNotification)
MERGE (self)-[:DEFINES]->(isComment)
@endcypher
*/

// =====================================================================
// types/notifications.ts — 真实通知服务 REST 契约类型 (per 2026-11-17)
//
// 契约与后端 lane 逐字段对齐 (crates/star-api-rest/src/routes/notifications.rs):
//   Notification wire: id, tenant_id, kind, subject, body, subject_url?,
//     actor_id?, actor_name?, target_type?, target_id?, read, created_at
//   kind ∈ review_request | comment | mention | system (serde lowercase)
//   Comment wire: id, tenant_id, target_type, target_id, author_id,
//     author_name (缺省回落 author_id), body, created_at
// =====================================================================

/** 通知类别 (后端 NotificationKind, serde lowercase) */
export const NOTIFICATION_KINDS = [
  "review_request",
  "comment",
  "mention",
  "system",
] as const;
export type NotificationKind = (typeof NOTIFICATION_KINDS)[number];

export function isNotificationKind(v: unknown): v is NotificationKind {
  return typeof v === "string" && (NOTIFICATION_KINDS as readonly string[]).includes(v);
}

export interface Notification {
  id: string;
  tenant_id: string;
  /** 宽容为 string (后端枚举只 4 值, 前端不因新增值崩溃) */
  kind: string;
  subject: string;
  body: string;
  /** 跳转目标 URL (可选, 无则详情面板不渲染跳转按钮) */
  subject_url?: string | null;
  actor_id?: string | null;
  actor_name?: string | null;
  /** 后端 TargetType 枚举: work_item | merge_request | agent; 与 target_id 成对 */
  target_type?: string | null;
  target_id?: string | null;
  read: boolean;
  /** RFC 3339 UTC */
  created_at: string;
}

export interface Comment {
  id: string;
  tenant_id: string;
  /** 自由字符串 (评论目标不套 notification 的 TargetType 枚举) */
  target_type: string;
  target_id: string;
  /** 创建时取 X-User-Id 请求头 */
  author_id: string;
  /** 缺省回落为 author_id */
  author_name: string;
  body: string;
  /** RFC 3339 UTC */
  created_at: string;
}

/** 宽容校验: 仅保证渲染所需的最小字段形状, 不拒收后端新增字段 */
export function isNotification(v: unknown): v is Notification {
  if (typeof v !== "object" || v === null) return false;
  const n = v as Record<string, unknown>;
  return (
    typeof n.id === "string" &&
    n.id.length > 0 &&
    typeof n.kind === "string" &&
    typeof n.subject === "string" &&
    typeof n.body === "string" &&
    typeof n.read === "boolean" &&
    typeof n.created_at === "string"
  );
}

export function isComment(v: unknown): v is Comment {
  if (typeof v !== "object" || v === null) return false;
  const c = v as Record<string, unknown>;
  return (
    typeof c.id === "string" &&
    c.id.length > 0 &&
    typeof c.target_type === "string" &&
    typeof c.target_id === "string" &&
    typeof c.body === "string" &&
    typeof c.created_at === "string"
  );
}
