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
}));
