// frontend/src/mocks/handlers/index.ts
// MSW handler re-export (per mock-msw-handlers.md §2.1)
//
// 设计: handlers 数组聚合 — MSW server 一次性注册所有 endpoint
//
// 5 域映射 (per test-design.md v0.2 §2.1.2 + 2026-08-31 wt-test-mock-5d):
//   player 域: agents (既有) + workspaces (新加)
//   economy 域: analytics (既有) + billing (新加)
//   match 域:   worktrees (新加)
//   social 域:  inbox (既有) + comments (新加)
//   admin 域:   tenants + rbac (新加, 合并在一个 handler 文件)

export { agentsHandlers } from "./agents";
export { analyticsHandlers } from "./analytics";
export { inboxHandlers } from "./inbox";
export { cliHandlers } from "./cli";
export { validationHandlers } from "./validation";
export { designArtifactHandlers } from "./design-artifacts";
export { incidentHandlers } from "./incidents";
export { workspacesHandlers } from "./workspaces";
export { billingHandlers } from "./billing";
export { worktreesHandlers } from "./worktrees";
export { commentsHandlers } from "./comments";
export { tenantsHandlers } from "./tenants";

import { agentsHandlers } from "./agents";
import { analyticsHandlers } from "./analytics";
import { inboxHandlers } from "./inbox";
import { cliHandlers } from "./cli";
import { validationHandlers } from "./validation";
import { designArtifactHandlers } from "./design-artifacts";
import { incidentHandlers } from "./incidents";
import { workspacesHandlers } from "./workspaces";
import { billingHandlers } from "./billing";
import { worktreesHandlers } from "./worktrees";
import { commentsHandlers } from "./comments";
import { tenantsHandlers } from "./tenants";

export const handlers = [
  ...agentsHandlers,
  ...analyticsHandlers,
  ...inboxHandlers,
  ...cliHandlers,
  ...validationHandlers,
  ...designArtifactHandlers,
  ...incidentHandlers,
  ...workspacesHandlers,
  ...billingHandlers,
  ...worktreesHandlers,
  ...commentsHandlers,
  ...tenantsHandlers,
];
