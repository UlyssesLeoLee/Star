// frontend/src/mocks/data/kanban.ts
// KANBAN_COLUMNS (per mock-data-isolation.md §2.1)
// Source: previously hard-coded in frontend/src/components/board/KanbanBoard.tsx
//
// Single source of truth for Kanban column order (per W1 dynamic-interaction-design.md §3.4)
// Jira-consistent default: Backlog is a view, columns are To Do → In Progress → In Review → Done
// 多人共用的本地模式使用此列定义 (含 review 列)

import type { WorkItemStatus } from "@/types/ids";

export const KANBAN_COLUMNS: ReadonlyArray<WorkItemStatus> = [
  "todo",
  "in_progress",
  "review",
  "done",
];
