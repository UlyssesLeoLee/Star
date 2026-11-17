/*
@cypher schema=1 source_sha256=9a3761058f88b25fa65c7aa14cca7bb1d674f144e3c1d9ebe7f5901eaee7b700
MERGE (self:File {path:"frontend/src/lib/nema/nemaCollaboration.ts"})
MERGE (presence:Symbol {id:"frontend/src/lib/nema/nemaCollaboration.ts::nemaEditorPresence",kind:"function"})
MERGE (resolve_actor:Symbol {id:"frontend/src/lib/nema/nemaCollaboration.ts::resolveNemaActor",kind:"function"})
MERGE (realtime_flag:Symbol {id:"frontend/src/lib/nema/nemaCollaboration.ts::NEMA_REALTIME_COLLABORATION_AVAILABLE",kind:"constant"})
MERGE (local_user:Symbol {id:"frontend/src/lib/nema/nemaCollaboration.ts::NEMA_LOCAL_DEFAULT_USER_ID",kind:"constant"})
MERGE (presence_kind:Type {id:"frontend/src/lib/nema/nemaCollaboration.ts::NemaPresenceKind"})
MERGE (presence_type:Type {id:"frontend/src/lib/nema/nemaCollaboration.ts::NemaPresence"})
MERGE (live_provider:Type {id:"frontend/src/lib/nema/nemaCollaboration.ts::NemaLivePresence"})
MERGE (auth:File {path:"frontend/src/lib/auth.ts"})
MERGE (get_session:Symbol {id:"frontend/src/lib/auth.ts::getSession",kind:"function"})
MERGE (nema:File {path:"frontend/src/lib/nema/nema.ts"})
MERGE (trace_type:Type {id:"frontend/src/lib/nema/nema.ts::NemaEditTraceEntry"})
MERGE (identity:Type {id:"frontend/src/types/ids.ts::Identity"})
MERGE (self)-[:DEFINES]->(presence)
MERGE (self)-[:DEFINES]->(resolve_actor)
MERGE (self)-[:DEFINES]->(realtime_flag)
MERGE (self)-[:DEFINES]->(local_user)
MERGE (self)-[:DEFINES]->(presence_kind)
MERGE (self)-[:DEFINES]->(presence_type)
MERGE (self)-[:DEFINES]->(live_provider)
MERGE (self)-[:IMPORTS]->(auth)
MERGE (self)-[:IMPORTS]->(nema)
MERGE (self)-[:USES_TYPE]->(trace_type)
MERGE (self)-[:USES_TYPE]->(identity)
MERGE (resolve_actor)-[:CALLS]->(get_session)
MERGE (presence)-[:USES_TYPE]->(trace_type)
@endcypher
*/
// =====================================================================
// 共同编辑的编辑者在线状态与身份来源 (per 2026-11-17 用户发令)
// 诚实原则 (缺标比错标):
//   - "正在编辑" / "离线" 只能来自真实实时协作连接 (live presence 源);
//     当前没有接入实时协作通道, 这两态永远不会被给出;
//   - 没有真实连接时, 状态退化为文档可证的 "未开始" (无编辑痕迹) 或
//     "未接入 (实时协作未接入)"。
// =====================================================================

import { getSession } from "@/lib/auth";
import type { Identity } from "@/types/ids";
import type { NemaEditTraceEntry } from "./nema";

export type NemaPresenceKind = "editing" | "offline" | "not_started" | "unavailable";

export interface NemaPresence {
  kind: NemaPresenceKind;
  /** UI 显示文案 */
  label: string;
  /** 状态的真实来源 (诚实标注) */
  source: string;
}

/** 实时 presence 源: 返回 editing/offline; 未知返回 null。接入真实协作通道后提供实现。 */
export type NemaLivePresence = (editor: string) => "editing" | "offline" | null;

/**
 * 实时协作通道当前未接入 (无 WS/RTC presence 后端)。
 * 接入后: 这里改为探测真实连接, 并把 nemaEditorPresence 的 live 参数接上。
 */
export const NEMA_REALTIME_COLLABORATION_AVAILABLE = false;

/**
 * 计算编辑者状态。诚实来源优先级:
 *   1. 真实 presence 源给出 editing/offline → 采用 (唯一能给出
 *      "正在编辑"/"离线" 的路径, 缺省不提供 → 永不显示);
 *   2. 没有实时源时, 从 .nema 编辑痕迹推断: 该编辑者无任何痕迹 → "未开始";
 *   3. 其余 (有痕迹但无实时连接) → "未接入 (实时协作未接入)"。
 */
export function nemaEditorPresence(
  editors: readonly string[],
  editTrail: readonly NemaEditTraceEntry[],
  editor: string,
  live?: NemaLivePresence,
): NemaPresence {
  const liveState = live?.(editor) ?? null;
  if (liveState === "editing") {
    return { kind: "editing", label: "正在编辑", source: "实时协作连接" };
  }
  if (liveState === "offline") {
    return { kind: "offline", label: "离线", source: "实时协作连接" };
  }
  if (editors.includes(editor) && !editTrail.some((entry) => entry.editor === editor)) {
    return { kind: "not_started", label: "未开始", source: ".nema 编辑痕迹 (无该编辑者记录)" };
  }
  return { kind: "unavailable", label: "未接入 (实时协作未接入)", source: "实时协作未接入" };
}

/** 与 settings IdentityPanels 一致的本地默认当前用户 id (真实会话由 /api/v1/auth/me 注入)。 */
export const NEMA_LOCAL_DEFAULT_USER_ID = "usr-001";

/**
 * 复用仓库既有身份机制解析当前编辑者 (不新造用户体系):
 *   1. lib/auth getSession() 的 user_id 命中 store.identities → 用该身份显示名;
 *   2. 无会话 → 本地默认当前用户 (usr-001, 同 settings/IdentityPanels 规则);
 *   3. identities 为空 → 退回 user id 字符串。
 */
export function resolveNemaActor(identities: readonly Identity[]): string {
  const session = getSession();
  const sessionUserId = session?.payload.user_id;
  if (sessionUserId) {
    const fromSession = identities.find((identity) => identity.id === sessionUserId);
    if (fromSession) return fromSession.display_name;
    return sessionUserId;
  }
  const current =
    identities.find((identity) => identity.id === NEMA_LOCAL_DEFAULT_USER_ID) ?? identities[0];
  return current?.display_name ?? NEMA_LOCAL_DEFAULT_USER_ID;
}
