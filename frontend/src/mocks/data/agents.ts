// frontend/src/mocks/data/agents.ts
// MOCK_AGENTS — 5 row fixture (per mock-data-isolation.md §2.1)
// Source: previously inline in frontend/src/app/(app)/agents/page.tsx
// Mock data content unchanged (per 缺标比错标, 仅抽位置, 业务逻辑不变).

import type { AgentRow } from "@/mocks/schemas/agent";

export const MOCK_AGENTS: ReadonlyArray<AgentRow> = [
  { id: "ag-001", name: "Ulysses-CLI",    status: "active",      role: "root / architect", last_active: "2 min ago" },
  { id: "ag-002", name: "Physis-builder", status: "in_progress", role: "rust / physics",   last_active: "5 min ago" },
  { id: "ag-003", name: "Star-frontend",  status: "paused",      role: "react / nextjs",   last_active: "1 h ago"   },
  { id: "ag-004", name: "Doc-scribe",     status: "active",      role: "docs / adr",       last_active: "12 min ago" },
  { id: "ag-005", name: "Review-bot",     status: "failed",      role: "ci / review",      last_active: "23 min ago" },
];

// FALLBACK alias for page SSR — page 改 useEffect+fetch 后, SSR 阶段用 FALLBACK 兜底
// (per mock-msw-handlers.md §2.4 + §4 #1 缺标)
// 业务数据不变, 仅 alias. 真实接入 backend 后删 FALLBACK.
export const MOCK_AGENTS_FALLBACK: ReadonlyArray<AgentRow> = MOCK_AGENTS;
