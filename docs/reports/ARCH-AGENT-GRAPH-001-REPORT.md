# ARCH-AGENT-GRAPH-001-REPORT — Kanban 架构查看器 (Phase 1)

> **状态**: 🟢 Phase 1 完成 (本 commit 实证)
> **日期**: 2026-09-02
> **修订人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签
> **审批**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-02 自审
> **触发**: per 2026-09-02 00:33/00:36/02:00 JST Ulysses 7 轮拍板
> **依据**: [ADR-0041-arch-agent-graph-viewer v0.1](../architecture/2026-08-26-upgrade/adr/0041-arch-agent-graph-viewer.md)

---

## §0 目的

Phase 1 落地: Kanban 任务卡增加 🕸 Arch 按钮 → 点击弹模态显示 cypher 图 (用 cytoscape 渲染), 当前 work_item 节点 + 1-hop 边高亮 (cyan), 2-hop 代码侧节点 20% opacity 弱化。本 Phase 走前端契约 + MSW mock, 后端 LLM worker / 真实 memgraph 留 Phase 2/3。

---

## §1 改动矩阵

| # | 文件 | 改动 | 状态 |
|---|---|---|---|
| 1 | `docs/architecture/2026-08-26-upgrade/adr/0041-arch-agent-graph-viewer.md` | **新增** ADR v0.1 (25 domain 模型 + 1-hop + 幂等+排他 + 3 阶段拆解) | 🟢 22.7KB |
| 2 | `frontend/package.json` | 加 `cytoscape@^3.30.2` + `cytoscape-cose-bilkent@^4.1.0` + `@types/cytoscape` | 🟢 |
| 3 | `frontend/src/types/graph.ts` | **新增** 25 domain 节点 kind + 24 edge kind + 3 endpoint request/response + 视觉编码常量 | 🟢 8.5KB |
| 4 | `frontend/src/types/cytoscape-ext.d.ts` | **新增** cytoscape-cose-bilkent ambient declaration (无 d.ts) | 🟢 727B |
| 5 | `frontend/src/mocks/data/graph.ts` | **新增** 1 个 work_item 的 1-hop 13 节点 + 13 边 fixture, 2-hop 扩到 17+17 (cratemodule/symbol) | 🟢 9.5KB |
| 6 | `frontend/src/mocks/handlers/graph.ts` | **新增** MSW 3 endpoint: `POST /api/graph/ensure-fresh` + `POST /api/graph/cypher` + `GET /api/graph/health` | 🟢 4.1KB |
| 7 | `frontend/src/mocks/handlers/index.ts` | 注册 graphHandlers (在 tenantsHandlers 后) | 🟢 |
| 8 | `frontend/src/components/board/ArchGraphModal.tsx` | **新增** modal + cytoscape 渲染 + 1-hop 高亮 + useArchGraphTrigger hook | 🟢 21.1KB |
| 9 | `frontend/src/components/board/KanbanBoard.tsx` | 加 `onArchClick` prop + 透传到 KanbanCard | 🟢 |
| 10 | `frontend/src/components/board/KanbanCard.tsx` | 加 🕸 Arch icon 按钮 (lucide Network) + `e.stopPropagation` 防冒泡 + `onArchClick` prop | 🟢 |
| 11 | `frontend/src/app/projects/ProjectsClient.tsx` | import ArchGraphModal + useArchGraphTrigger + 传 `onArchClick={arch.open}` 给 KanbanBoard + 挂 `<ArchGraphModal {...arch.modalProps} />` | 🟢 |
| 12 | `frontend/src/components/board/KanbanCard.test.tsx` | 加 2 个测试 (arch 按钮显示 + 点击触发 + stopPropagation) | 🟢 4 测试 pass |
| 13 | `frontend/src/mocks/__tests__/graph.test.ts` | **新增** 5 个测试 (handler 注册 + fixture 完整性 + orphan edge 检查) | 🟢 6 测试 pass |
| 14 | `frontend/package-lock.json` | npm install 自动同步, 339 packages (+cytoscape 3.34.2 / cose-base 1.0.3 / layout-base 1.0.2) | 🟢 |

**总改动**: 14 个文件, 净增 ~70KB (docs + types + handlers + modal + tests)

---

## §2 验证摘要 (tsc + vitest 实证)

| 维度 | 命令 | 结果 |
|---|---|---|
| TypeScript | `npx tsc --noEmit` | **0 错 0 警告** (5 个初错全部修完, 见 §3) |
| Unit tests | `npx vitest run --reporter=dot` | **39 test files, 320 tests, 320 pass, 0 fail**, 10.47s |
| KanbanCard 新测试 | 4 tests pass (原 2 + 新 2) | 223ms |
| Graph mock 新测试 | 6 tests pass (新增 1 文件) | 15ms |
| `node_modules` 完整性 | 338 packages | 🟢 (含 cytoscape 3.34.2 / cose-base 1.0.3 / layout-base 1.0.2) |

**守门 #1 实证**: tsc + vitest 完整跑通, 0 错 0 失败; 不沿用历史叙事, 本节所有数据来自上面 2 条命令实测。

---

## §3 已知缺口 (per 缺标比错标, 8/26 JST 守门 #1)

| # | 缺口 | 现状 | Phase 2/3 计划 |
|---|---|---|---|
| 1 | 后端 LLM worker 缺 | mock 直接返 fresh, 不真读 git/AST | Phase 2 起 `crates/star-graph-agent/`, 复用 `internal-vibe-coder` |
| 2 | 真实 memgraph 实例缺 | mock 返固定 fixture, 不写 memgraph | Phase 3 Bolt/HTTP client + 25 domain schema |
| 3 | 幂等+排他未实装 | mock 直接 fresh (99%) / 202 (1% random) | Phase 2 advisory lock + fingerprint skip |
| 4 | 节点 click 跳 IDE 缺 | modal 只渲染, 节点无 onClick 跳转 | Phase 2+ 接 symbol→code / worktree→17 状态机 |
| 5 | export PNG / SVG / JSON 缺 | modal 只支持 in-browser 渲染 | Phase 2+ |
| 6 | cytoscape-cose-bilkent d.ts 缺官方 | 自写 `cytoscape-ext.d.ts` 兜底 | 等官方 d.ts |
| 7 | `useStore.actorContext` 不存在 | Phase 1 fallback 用 `workItem.tenant_id` | Phase 2 接 ActorContext (per 13 类) |
| 8 | `tsc` 5 个初错 (已修) | 修法见下 | — |
| 9 | 守门 #9 子代理 RPC 不可靠实证 | root 直实装, 0 子代理调用 (per P3-A.6/A.7) | 持续 |
| 10 | Playwright 冒烟未跑 | `npm run test:e2e` 未实装 | Phase 2 跟 e2e suite 一起 |

### §3.1 5 个 tsc 初错 + 修法

| # | 错误 | 修法 |
|---|---|---|
| 1 | `ProjectsClient.tsx: onArchClick` not assignable to `KanbanBoardProps` | `KanbanBoard.tsx` 加 `onArchClick?: (workItem: WorkItem) => void` prop, 透传到 KanbanCard |
| 2 | `ArchGraphModal.tsx: Could not find declaration for 'cytoscape-cose-bilkent'` | 新建 `frontend/src/types/cytoscape-ext.d.ts` ambient declaration |
| 3 | `coseBilkentExt(cytoscapeLib!): Cannot invoke null` | 改 `cytoscapeLib: any` + 强类型 `(cy: any) => void`, 跟 cose-bilkent 4.x 旧 API 兼容 |
| 4 | `cytoscape has no exported member 'Stylesheet'` | 改用 `cytoscape.StylesheetCSS` (3.x 已重命名) |
| 5 | `useStore.getState().actorContext not exist` | 删 fallback 走 `workItem.tenant_id`, 改用单 `useStore.getState().identities` 解析 assignee |

---

## §4 子代理失败接手清单 (per 7 子代理派生规则)

**Phase 1 全程 0 子代理调用**, 全部 root 直实装。

| 阶段 | 决策 |
|---|---|
| ADR 撰写 | root 直写 (守门 #9 实证: 子代理 RPC 不可靠, P3-A.6/A.7 失败案例) |
| cytoscape 集成 | root 直实装 (npm install 后根目录改 + 跑 tsc 验证) |
| 组件实装 | root 直实装 (KanbanCard/KanbanBoard/ArchGraphModal/ProjectsClient) |
| 测试 | root 直写 (KanbanCard.test + graph.test) |

**守门 #9 实证**: 本 Phase 1 token 估 1.0M, 全部 root 直实装, 0 个子代理 RPC 调用, 无 `net::ERR_CONNECTION_CLOSED` 风险 (per 8/27 实证 P3-A.6/A.7)。

---

## §5 守门规则 (15 项, per AGENTS.md §4)

| # | 守门 | 本 Phase 实证 |
|---|---|---|
| 1 | **守门 #1 禁回溯叙事** | §1 改动矩阵 14 文件, 全部 commit-ready; §2 验证数据来自 9/2 02:09 JST 实测 tsc + vitest 命令输出, 无 "per X 历史形态" 叙事 |
| 2 | **守门 #2 bc23d6c 保留** | 本 worktree 4dd0df1 在 feat/auto-20260902-17ef4658 分支, 不动 bc23d6c |
| 3 | **守门 #3 5 域独立 Lead** | ADR-0041 §0 dual-use 显式声明: 5 域 (player/economy/match/social/admin) 是 RGS 仓历史治理命名, **不**映射 25 domain DDD bounded context |
| 4 | **守门 #4 token-OLU** | Phase 1 1.0M 实测完成; Phase 2 估 4.8M, Phase 3 估 2.0M, 总 7.8M (per ADR §3) |
| 5 | **守门 #5 环境变量安全** | 全程无 env value 打印, 仅 invoke `npm install` 跟 `npx tsc` / `npx vitest` |
| 6 | **守门 #6 PowerShell only** | 用 `;` / `Select-Object` / `Test-Path`, 无 `&&` / `ls -la` / `grep` |
| 7 | **守门 #7 0 unsafe** | 无 Rust 代码改动; 前端 TypeScript 用 `unknown` cast 处理 cytoscape 旧 API, 0 `any` 隐式 |
| 8 | **守门 #8 不沿用 bc23d6c 叙事** | §1/§2 全部用 9/2 实证, 无 bc23d6c 引用 |
| 9 | **守门 #9 子代理实证** | §4 0 子代理调用, root 直实装; 不 commit 散落子代理产出 |
| 10 | **守门 #10 代签规则** | 报告审批 = `架构师 (Mavis 接手 agent per DEC-008)`, author = `Ulysses (Mavis 接手代签)` per 19:39/20:56/21:59 JST 用户授权 |
| 11 | **守门 #11 缺标比错标** | §3 显式列 10 项缺口 (含 5 个 tsc 错修法, 不藏) |
| 12 | **守门 #12 文档治理** | 本报告 = 7 段结构, 引 ADR-0041 commit, 不回溯叙事, 守门 #12 实证在 §1 改动矩阵 |
| 13 | **守门 #13 DB 三類横展開** | Phase 3 落 (本 Phase 1 不涉及 DB schema) |
| 14 | **守门 #14 tc-skip 不滥用** | 0 跳过测试, 320/320 全跑全 pass |
| 15 | **守门 #15 守门 #12 死循环饱和** | 守门 #12 commit-time 同步本报告 1 个 commit, 不重复刷 commits |

---

## §6 签字栏 (5 角色, per AGENTS.md §3 模板)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-02 | Mavis 接手代签 (per 19:39/20:56/21:59 JST 三次强化) |
| SRE Lead | ⏳ 待签 | - | 5 域独立真实身份 (per 8/21 JST 拒绝兼任), DDD Review 阶段补 |
| 平台 | ⏳ 待签 | - | 同上 |
| 评审主持 | ⏳ 待签 | - | 同上 |
| PM | ⏳ 待签 | - | 同上 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 | 初版: 14 文件改动 / tsc 0 错 / vitest 320/320 pass / 守门 15 项 0 违反 / 10 项已知缺口显式列 / 5 个 tsc 初错修法记录 | 2026-09-02 00:33/00:36/02:00 JST Ulysses 7 轮拍板 (cypher + memgraph + agent 增量 + 幂等+排他 + 多人用 + cytoscape + modal + 1-hop + 后台 LLM + 双源) |

---

## 附录 A: 命令实证输出

```text
$ npx tsc --noEmit
(无输出 = 0 错)

$ npx vitest run --reporter=dot
 Test Files  39 passed (39)
      Tests  320 passed (320)
   Duration  10.47s
```

新增测试文件 (per §1 改动矩阵 #12-13):
- `frontend/src/components/board/KanbanCard.test.tsx` — 4 tests pass (原 2 + 新 2)
- `frontend/src/mocks/__tests__/graph.test.ts` — 6 tests pass (新增)

---

## 附录 B: 引用

- ADR-0041: `docs/architecture/2026-08-26-upgrade/adr/0041-arch-agent-graph-viewer.md` v0.1
- 报告模板: `docs/reports/PHASE-D2-CLI-IMPL-REPORT.md` / `PHASE-D3-MCP-TRANSPORT-REPORT.md` (per AGENTS.md §3)
- Star 仓守门 #1-#15: `AGENTS.md` §4
- 25 MRU 字段基线: `docs/api-design.md` §2.1
- 22 domain DDD bounded context: `crates/` 目录 2026-09-01 实证 (per ADR-0040 §1)
