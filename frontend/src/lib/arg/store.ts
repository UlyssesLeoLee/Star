// =====================================================================
// frontend/src/lib/arg/store.ts — zustand useARGStore 5 channel (per BD §4.3.1)
// =====================================================================
// Per docs/design/BD-AGENT-RELATIONSHIP-001.md v0.1 §4.3.1 (useARGStore 5 channel)
// + docs/architecture/2026-09-03-arg/09-arg-05-frontend-e2e.md v0.50 §2
// + docs/architecture/2026-09-03-arg/14-arg-10-arg-frontend.md §2.2
//
// 5 channel:
//   1. agents         — Map<id, Agent>         + loading/error
//   2. edges          — Map<id, Edge>          + loading/error
//   3. templates      — TeamTemplate[]         + loading
//   4. achievements   — Achievement[] + unlockedAchievements Set
//   5. argEvents      — ARGEvent[] (WebSocket log, 保留最近 100)
//
// 9 actions (per brief v0.50 §2.1 B):
//   loadAgents / loadEdges / createEdge / updateEdge / archiveEdge /
//   instantiateTemplate / loadAchievements / unlockAchievement / subscribeEvents
//
// 守门合规:
//   - 守门 #12 v21: [P] 子项 docs 同步 (per automation-design.md §4)
//   - 守门 #14 v2: tenant_id RLS 13 類通过 options.tenantId 传 api
//   - 守门 #9: 失败 fallback to 内存 cache, 不抛 panic
// =====================================================================

"use client";

import { create } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";

import * as api from "./api";
import { ArgWebSocketClient, wsBridgeToArgEvent } from "./ws";
import type {
  Achievement,
  AchievementUnlock,
  Agent,
  ARGEvent,
  CreateEdgeInput,
  Edge,
  EdgeFilter,
  InstantiateTemplateInput,
  TemplateId,
  TemplateInstance,
  TeamTemplate,
  UpdateEdgePatch,
  Uuid,
  WSBridgeEvent,
} from "./types";

// =====================================================================
// Store shape
// =====================================================================

export interface ARGStoreState {
  // ----- channel 1: agents -----
  agents: Map<Uuid, Agent>;
  agentsLoading: boolean;
  agentsError: string | null;

  // ----- channel 2: edges -----
  edges: Map<Uuid, Edge>;
  edgesLoading: boolean;
  edgesError: string | null;

  // ----- channel 3: templates -----
  templates: TeamTemplate[];
  templatesLoading: boolean;
  templatesError: string | null;

  // ----- channel 4: achievements -----
  achievements: Achievement[];
  unlockedAchievements: Set<string>; // achievement code
  achievementsLoading: boolean;
  achievementsError: string | null;
  myUnlocks: AchievementUnlock[];

  // ----- channel 5: events (WebSocket) -----
  argEvents: ARGEvent[];
  wsConnected: boolean;
  wsStatus: "disconnected" | "connecting" | "connected" | "mock";

  // ----- actions (per brief v0.50 §2.1 B) -----
  loadAgents: (tenantId?: Uuid) => Promise<void>;
  loadEdges: (filter?: EdgeFilter, tenantId?: Uuid) => Promise<void>;
  loadTemplates: (tenantId?: Uuid) => Promise<void>;
  createEdge: (input: CreateEdgeInput, tenantId?: Uuid) => Promise<Edge>;
  updateEdge: (id: Uuid, patch: UpdateEdgePatch, tenantId?: Uuid) => Promise<Edge>;
  archiveEdge: (id: Uuid, tenantId?: Uuid) => Promise<void>;
  instantiateTemplate: (
    input: InstantiateTemplateInput,
    tenantId?: Uuid,
  ) => Promise<TemplateInstance>;
  loadAchievements: (tenantId?: Uuid) => Promise<void>;
  loadMyUnlocks: (tenantId?: Uuid) => Promise<void>;
  unlockAchievement: (code: string) => void;
  subscribeEvents: (tenantId?: Uuid) => void;
  unsubscribeEvents: () => void;

  // ----- internal -----
  _applyWsEvent: (ev: WSBridgeEvent) => void;
  _reset: () => void;
}

const MAX_EVENT_LOG = 100;

const initialState = {
  agents: new Map<Uuid, Agent>(),
  agentsLoading: false,
  agentsError: null,
  edges: new Map<Uuid, Edge>(),
  edgesLoading: false,
  edgesError: null,
  templates: [],
  templatesLoading: false,
  templatesError: null,
  achievements: [],
  unlockedAchievements: new Set<string>(),
  achievementsLoading: false,
  achievementsError: null,
  myUnlocks: [],
  argEvents: [],
  wsConnected: false,
  wsStatus: "disconnected" as const,
};

// =====================================================================
// Store factory (per BD §4.3.1)
// =====================================================================

let _wsClient: ArgWebSocketClient | null = null;

export const useARGStore = create<ARGStoreState>()(
  subscribeWithSelector((set, get) => ({
    ...initialState,

    // ----- loadAgents -----
    loadAgents: async (tenantId?: Uuid) => {
      set({ agentsLoading: true, agentsError: null });
      try {
        const list = await api.listAgents({}, {}, { tenantId, retries: 0 });
        const map = new Map<Uuid, Agent>();
        for (const a of list) map.set(a.id, a);
        set({ agents: map, agentsLoading: false });
      } catch (e) {
        set({
          agentsLoading: false,
          agentsError: e instanceof Error ? e.message : String(e),
        });
      }
    },

    // ----- loadEdges -----
    loadEdges: async (filter: EdgeFilter = {}, tenantId?: Uuid) => {
      set({ edgesLoading: true, edgesError: null });
      try {
        const list = await api.listEdges(filter, { tenantId, retries: 0 });
        const map = new Map<Uuid, Edge>();
        for (const e of list) map.set(e.id, e);
        set({ edges: map, edgesLoading: false });
      } catch (e) {
        set({
          edgesLoading: false,
          edgesError: e instanceof Error ? e.message : String(e),
        });
      }
    },

    // ----- loadTemplates -----
    loadTemplates: async (tenantId?: Uuid) => {
      set({ templatesLoading: true, templatesError: null });
      try {
        // Backend doesn't have a list endpoint yet (per ARG.4 §4.12, only
        // `instantiate` is exposed). Use the 5 canonical templates as
        // static seed so the gallery is functional.
        const staticTemplates: TeamTemplate[] = [
          {
            id: "hub-and-spoke",
            name: "Hub-and-Spoke",
            description: "1 Lead + 4 Worker 团队",
            min_agents: 5,
            max_agents: 5,
            edges: defaultTemplateEdges("hub-and-spoke"),
            category: "standard",
          },
          {
            id: "mesh",
            name: "Mesh",
            description: "N 节点全连接 (C(N,2) 边)",
            min_agents: 3,
            max_agents: 10,
            edges: [],
            category: "high_density",
          },
          {
            id: "chain",
            name: "Chain",
            description: "A → B → C → D 链式",
            min_agents: 4,
            max_agents: 4,
            edges: defaultTemplateEdges("chain"),
            category: "pipeline",
          },
          {
            id: "hierarchical",
            name: "Hierarchical",
            description: "1 Lead + 2 Sub-Lead + 6 Worker (3 层)",
            min_agents: 9,
            max_agents: 9,
            edges: defaultTemplateEdges("hierarchical"),
            category: "management",
          },
          {
            id: "review-council",
            name: "ReviewCouncil",
            description: "1 Lead + 3 Reviewer 评审团",
            min_agents: 4,
            max_agents: 4,
            edges: defaultTemplateEdges("review-council"),
            category: "decision",
          },
        ];
        // Tenant id is read for consistency even though no list endpoint exists
        void tenantId;
        set({ templates: staticTemplates, templatesLoading: false });
      } catch (e) {
        set({
          templatesLoading: false,
          templatesError: e instanceof Error ? e.message : String(e),
        });
      }
    },

    // ----- createEdge -----
    createEdge: async (input: CreateEdgeInput, tenantId?: Uuid) => {
      const created = await api.createEdge(input, { tenantId, retries: 0 });
      const edges = new Map(get().edges);
      edges.set(created.id, created);
      set({ edges });
      // Append to event log
      appendEvent(set, get, { type: "edge_created", payload: created });
      return created;
    },

    // ----- updateEdge -----
    updateEdge: async (id: Uuid, patch: UpdateEdgePatch, tenantId?: Uuid) => {
      const updated = await api.updateEdge(id, patch, { tenantId, retries: 0 });
      const edges = new Map(get().edges);
      const before = edges.get(id);
      edges.set(id, updated);
      set({ edges });
      if (before) {
        appendEvent(set, get, {
          type: "edge_updated",
          id,
          before,
          after: updated,
        });
      }
      return updated;
    },

    // ----- archiveEdge -----
    archiveEdge: async (id: Uuid, tenantId?: Uuid) => {
      await api.archiveEdge(id, { tenantId, retries: 0 });
      const edges = new Map(get().edges);
      const before = edges.get(id);
      edges.delete(id);
      set({ edges });
      if (before) {
        appendEvent(set, get, { type: "edge_archived", id });
      }
    },

    // ----- instantiateTemplate -----
    instantiateTemplate: async (
      input: InstantiateTemplateInput,
      tenantId?: Uuid,
    ) => {
      const instance = await api.instantiateTemplate(input, { tenantId, retries: 0 });
      appendEvent(set, get, { type: "template_instantiated", payload: instance });
      return instance;
    },

    // ----- loadAchievements -----
    loadAchievements: async (tenantId?: Uuid) => {
      set({ achievementsLoading: true, achievementsError: null });
      try {
        const list = await api.listAchievements({}, { tenantId, retries: 0 });
        set({ achievements: list, achievementsLoading: false });
      } catch (e) {
        set({
          achievementsLoading: false,
          achievementsError: e instanceof Error ? e.message : String(e),
        });
      }
    },

    // ----- loadMyUnlocks -----
    loadMyUnlocks: async (tenantId?: Uuid) => {
      try {
        const unlocks = await api.myUnlocks({ tenantId }, { tenantId, retries: 0 });
        const codes = new Set<string>(get().unlockedAchievements);
        for (const u of unlocks) codes.add(u.achievement_code);
        set({ myUnlocks: unlocks, unlockedAchievements: codes });
      } catch {
        // Silent — unlocks is a soft signal
      }
    },

    // ----- unlockAchievement (optimistic) -----
    unlockAchievement: (code: string) => {
      const set_ = new Set(get().unlockedAchievements);
      set_.add(code);
      set({ unlockedAchievements: set_ });
    },

    // ----- subscribeEvents (WebSocket) -----
    subscribeEvents: (tenantId?: Uuid) => {
      // Idempotent — disconnect prior client first
      if (_wsClient) {
        _wsClient.disconnect();
        _wsClient = null;
      }
      const client = new ArgWebSocketClient({
        tenantId,
        onEvent: (ev) => get()._applyWsEvent(ev),
        onStatusChange: (status) => {
          set({
            wsStatus: status,
            wsConnected: status === "connected" || status === "mock",
          });
        },
      });
      _wsClient = client;
      client.connect();
    },

    // ----- unsubscribeEvents -----
    unsubscribeEvents: () => {
      if (_wsClient) {
        _wsClient.disconnect();
        _wsClient = null;
        set({ wsConnected: false, wsStatus: "disconnected" });
      }
    },

    // ----- _applyWsEvent (internal) -----
    _applyWsEvent: (ev: WSBridgeEvent) => {
      // 1) log every event (保留最近 100 条)
      const argEv = wsBridgeToArgEvent(ev);
      const events = [...get().argEvents];
      if (argEv) events.push(argEv);
      else
        events.push({
          type: "trust_score_changed",
          agent_id: "ws-bridge",
          before: 0,
          after: 0,
          delta: 0,
        });
      const trimmed = events.length > MAX_EVENT_LOG ? events.slice(-MAX_EVENT_LOG) : events;
      set({ argEvents: trimmed });

      // 2) per-kind refresh (per doc 14 §2.1)
      switch (ev.kind) {
        case "arg_edge_changed":
          // refresh edges (deferred, not blocking)
          void get().loadEdges();
          break;
        case "arg_trust_score_update":
          void get().loadAgents();
          break;
        case "arg_achievement_unlocked":
          void get().loadMyUnlocks();
          break;
        case "arg_dispatch_route":
        case "arg_context_inject":
          // 仅 log, 不刷数据
          break;
      }
    },

    // ----- _reset (test helper) -----
    _reset: () => {
      if (_wsClient) {
        _wsClient.disconnect();
        _wsClient = null;
      }
      set({ ...initialState });
    },
  })),
);

// =====================================================================
// Helpers
// =====================================================================

function appendEvent(
  set: (
    partial:
      | Partial<ARGStoreState>
      | ((state: ARGStoreState) => Partial<ARGStoreState>),
  ) => void,
  get: () => ARGStoreState,
  ev: ARGEvent,
): void {
  const events = [...get().argEvents, ev];
  const trimmed = events.length > MAX_EVENT_LOG ? events.slice(-MAX_EVENT_LOG) : events;
  set({ argEvents: trimmed });
}

function defaultTemplateEdges(
  id: TemplateId,
): import("./types").TemplateEdge[] {
  switch (id) {
    case "hub-and-spoke":
      return [0, 1, 2, 3].map((i) => ({
        from_index: 0,
        to_index: i + 1,
        edge_type: "DELEGATES_TO",
        default_weight: 0.7,
      }));
    case "chain":
      return [0, 1, 2].map((i) => ({
        from_index: i,
        to_index: i + 1,
        edge_type: "DELEGATES_TO",
        default_weight: 0.6,
      }));
    case "hierarchical":
      return [
        { from_index: 0, to_index: 1, edge_type: "DELEGATES_TO", default_weight: 0.7 },
        { from_index: 0, to_index: 2, edge_type: "DELEGATES_TO", default_weight: 0.7 },
        ...[1, 2].flatMap((sub) =>
          [0, 1, 2].map((w) => ({
            from_index: sub,
            to_index: 3 + (sub - 1) * 3 + w,
            edge_type: "DELEGATES_TO" as const,
            default_weight: 0.6,
          })),
        ),
      ];
    case "review-council":
      return [1, 2, 3].map((i) => ({
        from_index: i,
        to_index: 0,
        edge_type: "CONSULTS",
        default_weight: 0.8,
      }));
    case "mesh":
    default:
      return [];
  }
}

// =====================================================================
// Convenience selectors (per brief v0.50 §2.1 B 9 actions)
// =====================================================================

/** Select the connected agent count. */
export const selectAgentCount = (s: ARGStoreState): number => s.agents.size;

/** Select the connected edge count. */
export const selectEdgeCount = (s: ARGStoreState): number => s.edges.size;

/** Select unlocked achievement count. */
export const selectUnlockedCount = (s: ARGStoreState): number =>
  s.unlockedAchievements.size;

/** Select edges adjacent to a given agent id. */
export const selectEdgesByAgent = (agentId: Uuid) => (s: ARGStoreState): Edge[] =>
  Array.from(s.edges.values()).filter(
    (e) => e.from_agent === agentId || e.to_agent === agentId,
  );
