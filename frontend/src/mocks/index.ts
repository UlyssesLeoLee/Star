// frontend/src/mocks/index.ts
// mock infra entry (per mock-data-isolation.md §2.1)
//
// 兼容 d4b3193 commit (mock data layer) + M1 hotfix (zod → TS type guards).
// 当前 Phase E.2 (M1) 不引 MSW (per 设计书 §5 P2 缺口, 留 Phase E.3+)
// M1 范围: data/ + schemas/ + __tests__/ + seed.ts. handlers/ 留空.
//
// 已知缺口 (per 缺标比错标, 8/26 JST):
//   1. MSW handler 完整化 (page 改 fetch 而非直接 import) — P2
//   2. fixtures/ 目录人工对照 (read-only JSON) — P3
//   3. mock data i18n (zh-CN / en-US) — P3
//   4. lib/store.ts (W5) mock 改造 — P3 (W5 scope, 不在 M1)

export * from "./data";
export {
  AGENT_STATUSES,
  isAgentStatus,
  isAgentRow,
  type AgentStatus,
  type AgentRow,
} from "./schemas/agent";
export {
  NOTIF_KINDS,
  isNotifKind,
  isMockNotif,
  type NotifKind,
  type MockNotif,
} from "./schemas/inbox";
export {
  KPI_TONES,
  isKpiTone,
  isKpiCard,
  isCostPoint,
  type KpiTone,
  type KpiCard,
  type CostPoint,
} from "./schemas/analytics";
export { mulberry32 } from "./seed";
