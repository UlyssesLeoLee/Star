// frontend/src/mocks/handlers/index.ts
// MSW handler re-export (per mock-msw-handlers.md §2.1)
//
// 设计: handlers 数组聚合 — MSW server 一次性注册所有 endpoint

export { agentsHandlers } from "./agents";
export { analyticsHandlers } from "./analytics";
export { inboxHandlers } from "./inbox";
export { cliHandlers } from "./cli";
export { incidentHandlers } from "./incidents";

import { agentsHandlers } from "./agents";
import { analyticsHandlers } from "./analytics";
import { inboxHandlers } from "./inbox";
import { cliHandlers } from "./cli";
import { incidentHandlers } from "./incidents";

export const handlers = [
  ...agentsHandlers,
  ...analyticsHandlers,
  ...inboxHandlers,
  ...cliHandlers,
  ...incidentHandlers,
];
