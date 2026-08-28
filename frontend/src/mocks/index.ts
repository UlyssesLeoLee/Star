// frontend/src/mocks/index.ts
// mock infra entry (per mock-data-isolation.md §2.1)
//
// 当前 Phase E.2 (M1) 不引 MSW (per 设计书 §5 P2 缺口, 留 Phase E.3+)
// M1 范围: data/ + schemas/ + __tests__/ + seed.ts. handlers/ 留空.
//
// 已知缺口 (per 缺标比错标, 8/26 JST):
//   1. MSW handler 完整化 (page 改 fetch 而非直接 import) — P2
//   2. fixtures/ 目录人工对照 (read-only JSON) — P3
//   3. mock data i18n (zh-CN / en-US) — P3
//   4. lib/store.ts (W5) mock 改造 — P3 (W5 scope, 不在 M1)

export * from "./data";
export * from "./schemas/agent";
export * from "./schemas/inbox";
export * from "./schemas/analytics";
export { mulberry32 } from "./seed";
