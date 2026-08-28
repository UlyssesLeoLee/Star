// =====================================================================
// 全局 in-memory store + 操作 API
// =====================================================================
"use client";

import { create } from "zustand";
import * as seed from "./seed";
import type {
  Worktree, WorktreeStatus,
  AgentSession, AgentStatus,
  Feedback, FeedbackStatus,
  PullRequest, PullRequestStatus,
  WorkItem, WorkItemStatus,
  ChangeSet, ChangeSetStatus,
  Notification, NotificationStatus,
  Canvas, CanvasElement, CanvasConnector,
} from "@/types/ids";

interface StoreState {
  // read accessors (immutable from outside)
  tenants: typeof seed.tenants;
  projects: typeof seed.projects;
  identities: typeof seed.identities;
  workspaces: typeof seed.workspaces;
  workItems: WorkItem[];
  comments: typeof seed.comments;
  permissionSchemes: typeof seed.permissionSchemes;
  permissionRules: typeof seed.permissionRules;
  workflows: typeof seed.workflows;
  changeSets: ChangeSet[];
  worktrees: Worktree[];
  agentSessions: AgentSession[];
  feedbacks: Feedback[];
  contextPackets: typeof seed.contextPackets;
  contextDecisions: typeof seed.contextDecisions;
  validationCases: typeof seed.validationCases;
  localRuntimes: typeof seed.localRuntimes;
  repositories: typeof seed.repositories;
  pullRequests: PullRequest[];
  notifications: Notification[];
  searchHits: typeof seed.searchHits;
  savedSearches: typeof seed.savedSearches;
  integrations: typeof seed.integrations;
  presenceCursors: typeof seed.presenceCursors;
  whiteboards: typeof seed.whiteboards;
  canvases: Canvas[];
  canvasElements: CanvasElement[];
  canvasConnectors: CanvasConnector[];
  sprints: typeof seed.sprints;
  milestones: typeof seed.milestones;
  burndownSeries: typeof seed.burndownSeries;
  board: typeof seed.board;
  relations: typeof seed.relations;
  auditEvents: typeof seed.auditEvents;
  automationRules: typeof seed.automationRules;

  // mutations (5 状态机)
  transitionWorktree: (id: string, to: WorktreeStatus) => void;
  transitionAgent:    (id: string, to: AgentStatus) => void;
  transitionFeedback: (id: string, to: FeedbackStatus) => void;
  transitionPR:       (id: string, to: PullRequestStatus) => void;
  transitionWorkItem: (id: string, to: WorkItemStatus) => void;
  transitionChangeSet:(id: string, to: ChangeSetStatus) => void;
  markNotificationRead: (id: string) => void;

  // W3 Calendar mutations (per dynamic-interaction-design.md §4.3 + §5)
  // W3 minimal data layer contribution: drag-to-reschedule on Calendar / Gantt
  //  - transitionMilestone: 拖 milestone 条左右 = 改 due_date
  //  - updateWorkItemDueDate: 拖 work-item 到不同日期 = 改 due_date
  //  - updateSprintDates: 拖 sprint 条左右 = 改 start/end_date
  // W5 后续会叠 persist + toast (per §11.2 W5 在 W1-W4 之后)
  transitionMilestone: (id: string, newDueDate: string) => void;
  updateWorkItemDueDate: (id: string, newDueDate: string) => void;
  updateSprintDates: (id: string, newStart: string, newEnd: string) => void;

  // Canvas mutations(无限画布,frontend-canvas-design.md §2)
  addCanvasElement: (element: CanvasElement) => void;
  moveCanvasElement: (id: string, x: number, y: number) => void;
  deleteCanvasElement: (id: string) => void;
  addCanvasConnector: (connector: CanvasConnector) => void;
  setCanvasViewport: (canvasId: string, x: number, y: number, zoom: number) => void;
}

export const useStore = create<StoreState>((set) => ({
  tenants: seed.tenants,
  projects: seed.projects,
  identities: seed.identities,
  workspaces: seed.workspaces,
  workItems: seed.workItems,
  comments: seed.comments,
  permissionSchemes: seed.permissionSchemes,
  permissionRules: seed.permissionRules,
  workflows: seed.workflows,
  changeSets: seed.changeSets,
  worktrees: seed.worktrees,
  agentSessions: seed.agentSessions,
  feedbacks: seed.feedbacks,
  contextPackets: seed.contextPackets,
  contextDecisions: seed.contextDecisions,
  validationCases: seed.validationCases,
  localRuntimes: seed.localRuntimes,
  repositories: seed.repositories,
  pullRequests: seed.pullRequests,
  notifications: seed.notifications,
  searchHits: seed.searchHits,
  savedSearches: seed.savedSearches,
  integrations: seed.integrations,
  presenceCursors: seed.presenceCursors,
  whiteboards: seed.whiteboards,
  canvases: seed.canvases,
  canvasElements: seed.canvasElements,
  canvasConnectors: seed.canvasConnectors,
  sprints: seed.sprints,
  milestones: seed.milestones,
  burndownSeries: seed.burndownSeries,
  board: seed.board,
  relations: seed.relations,
  auditEvents: seed.auditEvents,
  automationRules: seed.automationRules,

  transitionWorktree: (id, to) =>
    set((s) => ({
      worktrees: s.worktrees.map((w) => w.id === id ? { ...w, status: to, last_event_at: new Date().toISOString(), lock_version: w.lock_version + 1 } : w),
    })),
  transitionAgent: (id, to) =>
    set((s) => ({
      agentSessions: s.agentSessions.map((a) => a.id === id ? { ...a, status: to, ended_at: ["completed","failed","cancelled"].includes(to) ? new Date().toISOString() : a.ended_at } : a),
    })),
  transitionFeedback: (id, to) =>
    set((s) => ({
      feedbacks: s.feedbacks.map((f) => f.id === id ? { ...f, status: to, answered_at: to === "resolved" || to === "wontfix" ? new Date().toISOString() : f.answered_at } : f),
    })),
  transitionPR: (id, to) =>
    set((s) => ({
      pullRequests: s.pullRequests.map((p) => p.id === id ? { ...p, status: to, merged_at: to === "merged" ? new Date().toISOString() : p.merged_at } : p),
    })),
  transitionWorkItem: (id, to) =>
    set((s) => ({
      workItems: s.workItems.map((w) => w.id === id ? { ...w, status: to, updated_at: new Date().toISOString() } : w),
    })),
  transitionChangeSet: (id, to) =>
    set((s) => ({
      changeSets: s.changeSets.map((c) => c.id === id ? { ...c, status: to } : c),
    })),
  markNotificationRead: (id) =>
    set((s) => ({
      notifications: s.notifications.map((n) => n.id === id ? { ...n, status: "read" as NotificationStatus } : n),
    })),

  // W3 Calendar drag-to-reschedule (per dynamic-interaction-design.md §4.3 + §5)
  transitionMilestone: (id, newDueDate) =>
    set((s) => ({
      milestones: s.milestones.map((m) => m.id === id ? { ...m, due_date: newDueDate } : m),
    })),
  updateWorkItemDueDate: (id, newDueDate) =>
    set((s) => ({
      workItems: s.workItems.map((w) => w.id === id ? { ...w, due_date: newDueDate, updated_at: new Date().toISOString() } : w),
    })),
  updateSprintDates: (id, newStart, newEnd) =>
    set((s) => ({
      sprints: s.sprints.map((sp) => sp.id === id ? { ...sp, start_date: newStart, end_date: newEnd } : sp),
    })),

  // Canvas mutations
  addCanvasElement: (element) =>
    set((s) => ({ canvasElements: [...s.canvasElements, element] })),
  moveCanvasElement: (id, x, y) =>
    set((s) => ({
      canvasElements: s.canvasElements.map((e) => e.id === id ? { ...e, x, y, updated_at: new Date().toISOString() } : e),
    })),
  deleteCanvasElement: (id) =>
    set((s) => ({
      canvasElements: s.canvasElements.filter((e) => e.id !== id),
      canvasConnectors: s.canvasConnectors.filter((c) => c.from_element_id !== id && c.to_element_id !== id),
    })),
  addCanvasConnector: (connector) =>
    set((s) => ({ canvasConnectors: [...s.canvasConnectors, connector] })),
  setCanvasViewport: (canvasId, x, y, zoom) =>
    set((s) => ({
      canvases: s.canvases.map((c) => c.id === canvasId ? { ...c, viewport: { x, y, zoom } } : c),
    })),
}));
