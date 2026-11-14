/*
@cypher schema=1 source_sha256=27d91d3d76aa805e5319ebf42ee05767a7a578872849e26b8570e58d37239a4e
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

export interface ChatMessage {
  id: string;
  author: string;        // "Ulysses" | "Agent A1" | ...
  authorKind: "human" | "agent" | "system";
  text: string;
  at: string;            // ISO time
}

export interface ChatTopic {
  id: string;
  scope: "task" | "global";
  /** task scope: workItemId (e.g. T-142); global scope: sessionId */
  refId: string;
  title: string;
  messages: ChatMessage[];
  createdAt: string;
}

export interface GlobalSession {
  id: string;
  title: string;
  parentId: string | null;   // null = 顶层会话; 非 null = 子会话
  createdAt: string;
}

interface ChatState {
  topics: ChatTopic[];
  globalSessions: GlobalSession[];
  activeTopicId: string | null;

  /** 点击任务卡 → 打开/创建该任务卡的聊天话题 (幂等) */
  openTopicForTask: (workItemId: string, taskTitle: string) => string;
  /** 全局聊天: 打开/创建会话话题 */
  openTopicForSession: (sessionId: string) => string;
  /** 创建全局会话 (顶层或子会话) */
  createSession: (title: string, parentId?: string | null) => string;
  /** 回复当前话题; 或开新话题 */
  reply: (topicId: string, author: string, authorKind: ChatMessage["authorKind"], text: string) => void;
  /** 在同一个聊天页(同一 ref)开新话题 */
  createTopic: (scope: "task" | "global", refId: string, title: string) => string;
  setActiveTopic: (topicId: string | null) => void;
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
          messages: [
            {
              id: uid("msg"),
              author: "system",
              authorKind: "system",
              text: `已打开任务卡 ${workItemId} 的聊天。话题按条回复，也可在底部开新话题。`,
              at: nowIso(),
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

      reply: (topicId, author, authorKind, text) => {
        const msg: ChatMessage = { id: uid("msg"), author, authorKind, text, at: nowIso() };
        set((s) => ({
          topics: s.topics.map((t) =>
            t.id === topicId ? { ...t, messages: [...t.messages, msg] } : t,
          ),
        }));
      },

      createTopic: (scope, refId, title) => {
        const id = uid("topic");
        const topic: ChatTopic = {
          id,
          scope,
          refId,
          title: title || (scope === "task" ? refId : "新话题"),
          messages: [],
          createdAt: nowIso(),
        };
        set((s) => ({ topics: [...s.topics, topic], activeTopicId: id }));
        return id;
      },

      setActiveTopic: (topicId) => set({ activeTopicId: topicId }),
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
      }),
    },
  ),
);
