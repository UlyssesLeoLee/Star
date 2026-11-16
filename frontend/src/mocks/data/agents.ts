/*
@cypher schema=1 source_sha256=1dc93ae74825996793c94dfa7c8e8747c7f405611513146ae397d774d74770d8
MERGE (self:File {path:"frontend/src/mocks/data/agents.ts"})
MERGE (agents:Symbol {id:"frontend/src/mocks/data/agents.ts::MOCK_AGENTS",kind:"constant"})
MERGE (row:Type {id:"frontend/src/mocks/schemas/agent.ts::AgentRow"})
MERGE (self)-[:DEFINES]->(agents)
MERGE (self)-[:USES_TYPE]->(row)
@endcypher
*/
// frontend/src/mocks/data/agents.ts
// MOCK_AGENTS — 5 row fixture (per mock-data-isolation.md §2.1)
// Source: previously inline in frontend/src/app/(app)/agents/page.tsx
// Mock data content unchanged (per 缺标比错标, 仅抽位置, 业务逻辑不变).
// 2026-11-17: Agent 中心页面已接真实运行时 (lib/agents/agentsApi.ts),
//   MOCK_AGENTS_FALLBACK 已删除 — 本文件仅剩 MSW handler / 测试 fixture 用途.

import type { AgentRow } from "@/mocks/schemas/agent";

export const MOCK_AGENTS: ReadonlyArray<AgentRow> = [
  { id: "ag-001", name: "Ulysses-CLI",    status: "active",      role: "root / architect", last_active: "2 min ago" },
  { id: "ag-002", name: "Physis-builder", status: "in_progress", role: "rust / physics",   last_active: "5 min ago" },
  { id: "ag-003", name: "Star-frontend",  status: "paused",      role: "react / nextjs",   last_active: "1 h ago"   },
  { id: "ag-004", name: "Doc-scribe",     status: "active",      role: "docs / adr",       last_active: "12 min ago" },
  { id: "ag-005", name: "Review-bot",     status: "failed",      role: "ci / review",      last_active: "23 min ago" },
];
