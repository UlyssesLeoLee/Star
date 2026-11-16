/*
@cypher schema=1 source_sha256=27ea6b5b76ff53832e2a3f0638e53f3794df67e25fdcd60e56d8b1633e53feda
MERGE (self:File {path:"frontend/src/lib/chat/dmStore.ts"})
MERGE (store:Symbol {id:"frontend/src/lib/chat/dmStore.ts::useDmStore",kind:"hook"})
MERGE (gateway:Symbol {id:"frontend/src/lib/chat/imGateway.ts::ImGateway",kind:"interface"})
MERGE (seed:Symbol {id:"frontend/src/lib/seed.ts::identities",kind:"constant"})
MERGE (self)-[:DEFINES]->(store)
MERGE (self)-[:DEFINES]->(gateway)
MERGE (self)-[:CALLS]->(gateway)
MERGE (self)-[:IMPORTS]->(seed)
@endcypher
*/
"use client";

// =====================================================================
// dmStore — 用户间即时通信 (DM/群聊) 本地域模型 (IM1.0 对齐)
//   1. 会话: conversations[{id, kind:'dm'|'group', memberIds[], title?,
//      messages[], unread, lastActivityAt, realtimeConnected}]
//   2. actions: openOrCreateConversation / sendDm / markConversationRead /
//      setRealtimeConnected (+ setActiveConversation / setCurrentUserId /
//      bindImGateway / receiveFromGateway / tryLoadImGateway)
//   3. 实时层: sibling 任务 imGateway.ts 按 ImGateway 接口签名实现;
//      未绑定/不可达 → 回退本地模式 (realtimeConnected=false, 诚实显示「未接入」)
//   4. 持久化: localStorage key "star-dm-conversations" (SSR-safe)
// =====================================================================

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { identities } from "@/lib/seed";

/** IM1.0 对齐: 消息状态子集 (sent→delivered→read) */
export type DmMessageState = "sent" | "delivered" | "read";

export interface DmMessage {
  id: string;
  conversationId: string;
  /** 发送者用户 id */
  authorId: string;
  /** 发送者显示名 */
  authorName: string;
  text: string;
  at: string; // ISO time
  /** IM1.0 对齐: 会话内序号 (单调递增, 稳定排序) */
  sequence: number;
  /** IM1.0 对齐: 消息状态; 本地模式自己的消息停留在 sent (诚实: 无送达确认) */
  state: DmMessageState;
  /** IM1.0 对齐: 已读回执成员 id 列表 */
  readBy: string[];
}

export type DmKind = "dm" | "group";

export interface DmConversation {
  id: string;
  kind: DmKind;
  /** 全部成员 (含当前用户); 存储时有序 */
  memberIds: string[];
  /** 群聊名称; DM 缺省 = 对方显示名 */
  title?: string;
  messages: DmMessage[];
  /** 本端未读数 */
  unread: number;
  lastActivityAt: string;
  /** 该会话是否经实时网关连通 (persist 时强制 false, 重载后需重新接入) */
  realtimeConnected: boolean;
}

/** 发送者参数 (sendDm 第三参): 字符串 = 用户 id */
export type DmAuthor = { id: string; name?: string } | string;

// =====================================================================
// IM 网关契约 (sibling 任务 frontend/src/lib/chat/imGateway.ts 按此签名
// 实现; 类型契约占位见 src/types/imGateway.d.ts)。网关未绑定/不可达时,
// dmStore 自动回退本地模式 — 消息仅本地暂存, 不冒充实时送达。
// =====================================================================
export interface ImGatewayOutboundMessage {
  conversationId: string;
  senderId: string;
  senderName: string;
  text: string;
  at: string; // ISO time
}

export interface ImGatewayInboundMessage {
  conversationId: string;
  senderId: string;
  senderName: string;
  text: string;
  at: string; // ISO time
}

export interface ImGateway {
  /** WebSocket 是否已连接且认证完成 */
  isConnected(): boolean;
  /** 上行发送; 返回 false / reject = 网关不可达 → 调用方回退本地 */
  sendMessage(msg: ImGatewayOutboundMessage): Promise<boolean> | boolean;
  /** 订阅远端下发的消息; 返回退订函数 */
  subscribe(onMessage: (msg: ImGatewayInboundMessage) => void): () => void;
}

/** 当前登录用户 (seed.ts identities usr-001 = Ulysses) */
export const DM_CURRENT_USER_ID = "usr-001";

/** 用户目录 (本地 seed; 真实用户列表由网关接入后提供) */
export const DM_KNOWN_USERS: { id: string; name: string }[] = identities
  .filter((i) => i.status === "active")
  .map((i) => ({ id: i.id, name: i.display_name }));

/** 用户 id → 显示名 (未知 id 回退为 id 本身) */
export const dmUserName = (id: string): string =>
  DM_KNOWN_USERS.find((u) => u.id === id)?.name ?? id;

interface DmState {
  conversations: DmConversation[];
  activeConversationId: string | null;
  /** 本端用户 id (persist 不存; 由登录态注入, 默认 seed 用户) */
  currentUserId: string;
  /** 实时网关连通性 (运行时状态; persist 不存) */
  realtimeConnected: boolean;
  /** 已绑定的 IM 网关 (运行时; persist 不存) */
  gateway: ImGateway | null;
  unsubscribeGateway: (() => void) | null;

  /**
   * 打开或创建会话 (幂等): 成员集合相同且 kind 相同 → 复用既有会话。
   * memberIds = 对方成员 (不含当前用户, 当前用户自动加入);
   * 1 人 → DM, ≥2 人 → 群聊。返回会话 id (成员为空返回 "")。
   */
  openOrCreateConversation: (memberIds: string[], title?: string) => string;
  /** 发送消息; author = 发送者 (id 或 {id, name?})。网关连通时按接口上行, 失败回退本地 */
  sendDm: (conversationId: string, text: string, author: DmAuthor) => void;
  /** 标记会话已读: unread=0, 他人消息 state→read 且本端加入 readBy */
  markConversationRead: (conversationId: string) => void;
  /** 设置实时连通性 (同步到每个会话的 realtimeConnected) */
  setRealtimeConnected: (connected: boolean) => void;
  setActiveConversation: (id: string | null) => void;
  setCurrentUserId: (id: string) => void;
  /** 绑定/解绑 IM 网关 (sibling imGateway.ts 接入点; null = 回退本地模式) */
  bindImGateway: (gateway: ImGateway | null) => void;
  /** 网关下发消息落库 (未知会话忽略 — 不伪造会话) */
  receiveFromGateway: (incoming: ImGatewayInboundMessage) => void;
  /** 懒加载 sibling imGateway.ts (模块未落地/不可达 → null → 本地模式) */
  tryLoadImGateway: () => Promise<ImGateway | null>;
}

const nowIso = () => new Date().toISOString();

let uidSeq = 0;
const uid = (prefix: string) => `${prefix}-${Date.now().toString(36)}-${(uidSeq++).toString(36)}`;

const sameMemberSet = (a: string[], b: string[]): boolean => {
  const x = [...a].sort();
  const y = [...b].sort();
  return x.length === y.length && x.every((v, i) => v === y[i]);
};

/** 网关失败/解绑 → 全局与会话级实时标志一并回落 (诚实标记) */
const markRealtimeDown = (connected: boolean) =>
  (set: (fn: (s: DmState) => Partial<DmState>) => void) => {
    set((s) => ({
      realtimeConnected: connected,
      conversations: s.conversations.map((c) =>
        c.realtimeConnected === connected ? c : { ...c, realtimeConnected: connected },
      ),
    }));
  };

export const useDmStore = create<DmState>()(
  persist(
    (set, get) => ({
      conversations: [],
      activeConversationId: null,
      currentUserId: DM_CURRENT_USER_ID,
      realtimeConnected: false,
      gateway: null,
      unsubscribeGateway: null,

      openOrCreateConversation: (memberIds, title) => {
        const me = get().currentUserId;
        const peers = Array.from(new Set(memberIds.filter((id) => id && id !== me)));
        if (peers.length === 0) {
          // 诚实: 没有对方成员就不创建 (面板端已禁用, 这里兜底)
          return "";
        }
        const full = [me, ...peers].sort();
        const kind: DmKind = full.length === 2 ? "dm" : "group";
        const existing = get().conversations.find(
          (c) => c.kind === kind && sameMemberSet(c.memberIds, full),
        );
        if (existing) {
          set({ activeConversationId: existing.id });
          return existing.id;
        }
        const id = uid("dm");
        const defaultTitle =
          kind === "dm" ? dmUserName(peers[0]) : peers.map(dmUserName).join("、");
        const conv: DmConversation = {
          id,
          kind,
          memberIds: full,
          title: title?.trim() || defaultTitle,
          messages: [],
          unread: 0,
          lastActivityAt: nowIso(),
          realtimeConnected: get().realtimeConnected,
        };
        set((s) => ({ conversations: [...s.conversations, conv], activeConversationId: id }));
        return id;
      },

      sendDm: (conversationId, text, author) => {
        const trimmed = text.trim();
        if (!trimmed) return;
        const authorId = typeof author === "string" ? author : author.id;
        const authorName =
          typeof author === "string" ? dmUserName(author) : (author.name ?? dmUserName(author.id));
        const conv = get().conversations.find((c) => c.id === conversationId);
        if (!conv) return;
        const msg: DmMessage = {
          id: uid("msg"),
          conversationId,
          authorId,
          authorName,
          text: trimmed,
          at: nowIso(),
          sequence: conv.messages.reduce((m, x) => Math.max(m, x.sequence), 0) + 1,
          state: "sent",
          readBy: [authorId],
        };
        set((s) => ({
          conversations: s.conversations.map((c) =>
            c.id === conversationId
              ? {
                  ...c,
                  messages: [...c.messages, msg],
                  unread:
                    c.unread +
                    (authorId !== s.currentUserId && conversationId !== s.activeConversationId
                      ? 1
                      : 0),
                  lastActivityAt: msg.at,
                }
              : c,
          ),
        }));
        // 实时网关: 已绑定且连通 → 按接口签名上行; 失败/不可达 → 回退本地 (诚实)
        const { gateway, realtimeConnected } = get();
        if (gateway && realtimeConnected) {
          try {
            const r = gateway.sendMessage({
              conversationId,
              senderId: authorId,
              senderName: authorName,
              text: trimmed,
              at: msg.at,
            });
            const onFail = () => markRealtimeDown(false)(set);
            if (r instanceof Promise) {
              void r.then((ok) => {
                if (!ok) onFail();
              }).catch(onFail);
            } else if (!r) {
              onFail();
            }
          } catch {
            markRealtimeDown(false)(set);
          }
        }
      },

      receiveFromGateway: (incoming) => {
        const conv = get().conversations.find((c) => c.id === incoming.conversationId);
        if (!conv) return; // 未知会话: 本地模式不伪造会话 (诚实原则)
        const me = get().currentUserId;
        const msg: DmMessage = {
          id: uid("msg"),
          conversationId: conv.id,
          authorId: incoming.senderId,
          authorName: incoming.senderName,
          text: incoming.text,
          at: incoming.at || nowIso(),
          sequence: conv.messages.reduce((m, x) => Math.max(m, x.sequence), 0) + 1,
          state: incoming.senderId === me ? "sent" : "delivered",
          readBy: [incoming.senderId],
        };
        set((s) => ({
          conversations: s.conversations.map((c) =>
            c.id === conv.id
              ? {
                  ...c,
                  messages: [...c.messages, msg],
                  unread:
                    c.unread +
                    (incoming.senderId !== s.currentUserId && conv.id !== s.activeConversationId
                      ? 1
                      : 0),
                  lastActivityAt: msg.at,
                }
              : c,
          ),
        }));
      },

      markConversationRead: (conversationId) => {
        const me = get().currentUserId;
        set((s) => ({
          conversations: s.conversations.map((c) =>
            c.id !== conversationId
              ? c
              : {
                  ...c,
                  unread: 0,
                  messages: c.messages.map((m) =>
                    m.authorId !== me && m.state !== "read"
                      ? {
                          ...m,
                          state: "read" as const,
                          readBy: m.readBy.includes(me) ? m.readBy : [...m.readBy, me],
                        }
                      : m,
                  ),
                },
          ),
        }));
      },

      setRealtimeConnected: (connected) => {
        markRealtimeDown(connected)(set);
      },

      setActiveConversation: (id) => set({ activeConversationId: id }),

      setCurrentUserId: (id) => set({ currentUserId: id }),

      bindImGateway: (gateway) => {
        const prev = get();
        prev.unsubscribeGateway?.();
        if (!gateway) {
          set({
            gateway: null,
            unsubscribeGateway: null,
            realtimeConnected: false,
            conversations: prev.conversations.map((c) =>
              c.realtimeConnected ? { ...c, realtimeConnected: false } : c,
            ),
          });
          return;
        }
        const connected = gateway.isConnected();
        let unsub: (() => void) | null = null;
        try {
          unsub = gateway.subscribe((incoming) => get().receiveFromGateway(incoming));
        } catch {
          unsub = null; // 订阅失败 → 保持本地模式
        }
        set({
          gateway,
          unsubscribeGateway: unsub,
          realtimeConnected: connected,
          conversations: prev.conversations.map((c) =>
            c.realtimeConnected === connected ? c : { ...c, realtimeConnected: connected },
          ),
        });
      },

      tryLoadImGateway: async () => {
        try {
          // sibling 任务 frontend/src/lib/chat/imGateway.ts 提供真实实现
          // (契约见 src/types/imGateway.d.ts); 模块尚未落地/不可达 →
          // 动态 import 失败 → 回退本地模式。
          const mod = (await import("@/lib/chat/imGateway")) as {
            getImGateway?: () => ImGateway | null;
            createImGateway?: () => ImGateway;
            connectImGateway?: (options?: Record<string, unknown>) => ImGateway;
          };
          const raw: unknown =
            mod.getImGateway?.() ??
            mod.createImGateway?.() ??
            mod.connectImGateway?.() ??
            null;
          if (!raw) return null;
          // 真实 ImGateway class (imGateway.ts) 与本 store 的 ImGateway 接口形状不同:
          //   class: getState()/onServerFrame(cb)/sendMessage({conversationId,kind,content,replyTo})
          //   接口: isConnected()/subscribe(onMessage)/sendMessage({conversationId,senderId,...})
          // → 此处做适配 (真实 class 的 ServerFrame → 本 store 的 inbound message)。
          const candidate = raw as {
            isConnected?: () => boolean;
            subscribe?: (onMessage: (msg: ImGatewayInboundMessage) => void) => () => void;
          };
          if (typeof candidate.isConnected === "function" && typeof candidate.subscribe === "function") {
            return candidate as ImGateway;
          }
          const classGateway = raw as {
            getState?: () => string;
            onServerFrame?: (cb: (frame: { type: string; [k: string]: unknown }) => void) => () => void;
            sendMessage?: (input: { conversationId: string; kind: string; content: unknown }) => unknown;
          };
          if (typeof classGateway.getState === "function" && typeof classGateway.onServerFrame === "function") {
            const gateway = classGateway as {
              getState: () => string;
              onServerFrame: (cb: (frame: { type: string; [k: string]: unknown }) => void) => () => void;
              sendMessage: (input: { conversationId: string; kind: string; content: unknown }) => unknown;
            };
            return {
              isConnected: () => gateway.getState() === "connected",
              sendMessage: (msg) => {
                try {
                  gateway.sendMessage({
                    conversationId: msg.conversationId,
                    kind: "text",
                    content: { kind: "text", text: msg.text },
                  });
                  return true;
                } catch {
                  return false;
                }
              },
              subscribe: (onMessage) =>
                gateway.onServerFrame((frame) => {
                  if (frame.type !== "message_new") return;
                  const payload = frame.message as { conversation_id?: string; content?: { text?: string }; sequence?: number } | undefined;
                  if (!payload?.conversation_id) return;
                  onMessage({
                    conversationId: payload.conversation_id,
                    senderId: String(frame.sender_id ?? ""),
                    senderName: String(frame.sender_name ?? ""),
                    text: payload.content?.text ?? "",
                    at: new Date().toISOString(),
                  });
                }),
            };
          }
          return null;
        } catch {
          return null;
        }
      },
    }),
    {
      name: "star-dm-conversations",
      version: 1,
      storage: createJSONStorage(() =>
        typeof window !== "undefined"
          ? localStorage
          : { getItem: () => null, setItem: () => {}, removeItem: () => {} },
      ),
      partialize: (s) => ({
        conversations: s.conversations.map((c) => ({ ...c, realtimeConnected: false })),
        activeConversationId: s.activeConversationId,
      }),
      // 重载后 realtimeConnected 一律回落 false — 网关需重新接入 (诚实)
      merge: (persisted, current) => {
        const p = (persisted ?? {}) as {
          conversations?: DmConversation[];
          activeConversationId?: string | null;
        };
        return {
          ...current,
          activeConversationId: p.activeConversationId ?? null,
          conversations: (p.conversations ?? []).map((c) => ({
            ...c,
            realtimeConnected: false,
          })),
        };
      },
    },
  ),
);
