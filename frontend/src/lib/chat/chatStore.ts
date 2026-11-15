/*
@cypher schema=1 source_sha256=3330be574671ab20f90e68e6b60f6dc6cc51f28ce1a6a48cd7a77c9ad87580b5
MERGE (self:File {path:"frontend/src/lib/chat/chatStore.ts"})
MERGE (store:Symbol {id:"frontend/src/lib/chat/chatStore.ts::useChatStore",kind:"hook"})
MERGE (self)-[:DEFINES]->(store)
@endcypher
*/
"use client";

// =====================================================================
// chatStore — 聊天模型 (per 2026-11-14 用户发令: 参考 Multica 做法)
//   1. 任务卡聊天: 点击任务卡打开与该任务卡相关的聊天页
//      - 每个话题可单独回复; 底部聊天栏输入可回复当前话题或开新话题
//   2. 全局聊天: Agent 常见会话管理 — 会话/子会话两级结构
//      - session (顶层会话) → sub-session (子会话, parentId 指向)
// =====================================================================

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

/** IM1.0 对齐 (per 2026-11-15 用户发令: chat 组件用 IM1.0 改造)
 *  MessageState: sent/delivered/read (IM1.0 im-core message::MessageState 子集)
 */
export type ChatMessageState = "sent" | "delivered" | "read";

export interface ChatMessage {
  id: string;
  author: string;        // "Ulysses" | "Agent A1" | ...
  authorKind: "human" | "agent" | "system";
  text: string;
  at: string;            // ISO time
  /** IM1.0 对齐: 会话内序号 (sequence, 单调递增, 稳定排序) */
  sequence: number;
  /** IM1.0 对齐: 引用回复 (reply_to → 被引用消息 id) */
  replyTo?: string;
  /** IM1.0 对齐: 消息状态 (sent→delivered→read) */
  state: ChatMessageState;
  editedAt?: string;     // IM1.0 edited_at
}

/** IM1.0 对齐: ConversationKind 子集 — task=Channel, global=Group, 系统=System */
export type ChatTopicKind = "channel" | "group" | "system";

export interface ChatTopicMember {
  userId: string;
  role: "owner" | "admin" | "member";  // IM1.0 MemberRole
  joinedAt: string;
}

export interface ChatTopic {
  id: string;
  scope: "task" | "global";
  /** task scope: workItemId (e.g. T-142); global scope: sessionId */
  refId: string;
  title: string;
  /** IM1.0 对齐: 会话类型 (task=channel, global=group) */
  kind: ChatTopicKind;
  /** IM1.0 对齐: 会话成员与角色 */
  members: ChatTopicMember[];
  messages: ChatMessage[];
  createdAt: string;
}

export interface GlobalSession {
  id: string;
  title: string;
  parentId: string | null;   // null = 顶层会话; 非 null = 子会话
  createdAt: string;
}

/**
 * 底栏聊天范围 (per 2026-11-15 任务: 底栏聊天保留范围选择)
 *   - WORKTREE: 当前 Worktree 对话
 *   - GLOBAL:   全局范围 — 必须先选择授权目标, 提交时重新校验权限
 */
export type ChatScope = "WORKTREE" | "GLOBAL";

/** 聊天 agent 设置 (per 2026-11-15: 主流 agent 功能 — 模型/effort/思考模式/自动模式) */
export type ChatEffort = "low" | "medium" | "high";
export interface ChatAgentSettings {
  model: string;
  effort: ChatEffort;
  thinkingMode: boolean;
  autoMode: boolean;
}
export const DEFAULT_AGENT_SETTINGS: ChatAgentSettings = {
  model: "deepseek-v4-pro",
  effort: "medium",
  thinkingMode: false,
  autoMode: true,
};
export const CHAT_MODELS = ["deepseek-v4-pro", "MiniMax-M3", "gpt-4o", "claude-sonnet-4"] as const;

interface ChatState {
  topics: ChatTopic[];
  globalSessions: GlobalSession[];
  activeTopicId: string | null;
  /** 底栏聊天当前范围 (persist) */
  chatScope: ChatScope;
  /** GLOBAL 范围的授权目标 worktree id; 未选 = null (persist, 提交时重新校验) */
  globalAuthTargetId: string | null;
  /** 聊天 agent 设置 (persist) */
  agentSettings: ChatAgentSettings;

  /** 点击任务卡 → 打开/创建该任务卡的聊天话题 (幂等) */
  openTopicForTask: (workItemId: string, taskTitle: string) => string;
  /** 全局聊天: 打开/创建会话话题 */
  openTopicForSession: (sessionId: string) => string;
  /** 创建全局会话 (顶层或子会话) */
  createSession: (title: string, parentId?: string | null) => string;
  /** 回复当前话题; 或开新话题 */
  reply: (topicId: string, author: string, authorKind: ChatMessage["authorKind"], text: string, replyTo?: string) => void;
  /** IM1.0 对齐: 标记话题内消息已读 (delivered→read) */
  markTopicRead: (topicId: string) => void;
  /** 在同一个聊天页(同一 ref)开新话题 */
  createTopic: (scope: "task" | "global", refId: string, title: string) => string;
  setActiveTopic: (topicId: string | null) => void;
  /** 切换底栏聊天范围 (WORKTREE / GLOBAL) */
  setChatScope: (scope: ChatScope) => void;
  /** 设置 GLOBAL 范围授权目标 worktree (null = 清除) */
  setGlobalAuthTarget: (worktreeId: string | null) => void;
  setAgentSettings: (patch: Partial<ChatAgentSettings>) => void;
}

const nowIso = () => new Date().toISOString();

let seq = 0;
const uid = (prefix: string) => `${prefix}-${Date.now().toString(36)}-${(seq++).toString(36)}`;

function seedGlobalSessions(): GlobalSession[] {
  const t = nowIso();
  return [
    { id: "gs-001", title: "关系图环检测上线", parentId: null, createdAt: t },
    { id: "gs-002", title: "BFS 遍历优化讨论", parentId: "gs-001", createdAt: t },
    { id: "gs-003", title: "接口契约变更", parentId: "gs-001", createdAt: t },
    { id: "gs-004", title: "周会 · 团队同步", parentId: null, createdAt: t },
  ];
}

export const useChatStore = create<ChatState>()(
  persist(
    (set, get) => ({
      topics: [],
      globalSessions: seedGlobalSessions(),
      activeTopicId: null,
      chatScope: "WORKTREE",
      globalAuthTargetId: null,
      agentSettings: DEFAULT_AGENT_SETTINGS,

      openTopicForTask: (workItemId, taskTitle) => {
        const existing = get().topics.find(
          (t) => t.scope === "task" && t.refId === workItemId && t.messages.length >= 0,
        );
        if (existing) {
          set({ activeTopicId: existing.id });
          return existing.id;
        }
        const id = uid("topic");
        const topic: ChatTopic = {
          id,
          scope: "task",
          refId: workItemId,
          title: taskTitle || workItemId,
          kind: "channel",
          members: [
            { userId: "usr-arch", role: "owner", joinedAt: nowIso() },
            { userId: "usr-002", role: "member", joinedAt: nowIso() },
          ],
          messages: [
            {
              id: uid("msg"),
              author: "system",
              authorKind: "system",
              text: `已打开任务卡 ${workItemId} 的聊天。话题按条回复，也可在底部开新话题。`,
              at: nowIso(),
              sequence: 1,
              state: "read",
            },
          ],
          createdAt: nowIso(),
        };
        set((s) => ({ topics: [...s.topics, topic], activeTopicId: id }));
        return id;
      },

      openTopicForSession: (sessionId) => {
        const session = get().globalSessions.find((g) => g.id === sessionId);
        const existing = get().topics.find((t) => t.scope === "global" && t.refId === sessionId);
        if (existing) {
          set({ activeTopicId: existing.id });
          return existing.id;
        }
        const id = uid("topic");
        const topic: ChatTopic = {
          id,
          scope: "global",
          refId: sessionId,
          title: session?.title ?? sessionId,
          kind: "group",
          members: [{ userId: "usr-arch", role: "owner", joinedAt: nowIso() }],
          messages: [],
          createdAt: nowIso(),
        };
        set((s) => ({ topics: [...s.topics, topic], activeTopicId: id }));
        return id;
      },

      createSession: (title, parentId = null) => {
        const id = uid("gs");
        const session: GlobalSession = { id, title, parentId, createdAt: nowIso() };
        set((s) => ({ globalSessions: [...s.globalSessions, session] }));
        return id;
      },

      reply: (topicId, author, authorKind, text, replyTo) => {
        set((s) => {
          const topic = s.topics.find((t) => t.id === topicId);
          if (!topic) return s;
          const seq = (topic.messages.reduce((m, x) => Math.max(m, x.sequence), 0) + 1) || 1;
          const msg: ChatMessage = {
            id: uid("msg"),
            author,
            authorKind,
            text,
            at: nowIso(),
            sequence: seq,
            replyTo,
            state: "sent",
          };
          return {
            topics: s.topics.map((t) =>
              t.id === topicId ? { ...t, messages: [...t.messages, msg] } : t,
            ),
          };
        });
      },

      createTopic: (scope, refId, title) => {
        const id = uid("topic");
        const topic: ChatTopic = {
          id,
          scope,
          refId,
          title: title || (scope === "task" ? refId : "新话题"),
          kind: scope === "task" ? "channel" : "group",
          members: [{ userId: "usr-arch", role: "owner", joinedAt: nowIso() }],
          messages: [],
          createdAt: nowIso(),
        };
        set((s) => ({ topics: [...s.topics, topic], activeTopicId: id }));
        return id;
      },

      setActiveTopic: (topicId) =>
        set((s) => ({
          activeTopicId: topicId,
          topics: s.topics.map((t) =>
            t.id === topicId
              ? {
                  ...t,
                  messages: t.messages.map((m) =>
                    m.authorKind !== "human" && m.state !== "read" ? { ...m, state: "read" as const } : m,
                  ),
                }
              : t,
          ),
        })),

      markTopicRead: (topicId) =>
        set((s) => ({
          topics: s.topics.map((t) =>
            t.id === topicId
              ? {
                  ...t,
                  messages: t.messages.map((m) =>
                    m.authorKind !== "human" && m.state !== "read" ? { ...m, state: "read" as const } : m,
                  ),
                }
              : t,
          ),
        })),

      setChatScope: (scope) => set({ chatScope: scope }),

      setGlobalAuthTarget: (worktreeId) => set({ globalAuthTargetId: worktreeId }),

      setAgentSettings: (patch) =>
        set((s) => ({ agentSettings: { ...s.agentSettings, ...patch } })),
    }),
    {
      name: "star-chat-store:v1",
      version: 1,
      storage: createJSONStorage(() => (typeof window !== "undefined" ? localStorage : {
        getItem: () => null,
        setItem: () => {},
        removeItem: () => {},
      })),
      partialize: (s) => ({
        topics: s.topics,
        globalSessions: s.globalSessions,
        activeTopicId: s.activeTopicId,
        chatScope: s.chatScope,
        globalAuthTargetId: s.globalAuthTargetId,
        agentSettings: s.agentSettings,
      }),
    },
  ),
);
