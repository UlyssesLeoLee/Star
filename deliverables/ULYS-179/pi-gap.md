# Pi 模式级参考 — 缺失功能多维统计 (inventory)

> **文档版本**: v0.1 (2026-09-22)
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核**
> **审批**: 架构师 = Ulysses 自身 (per ULYS-175 2026-09-22 01:28 JST 用户拍板, **本人即 5 域 Lead**)
> **触发**: 2026-09-22 01:28 JST Ulysses "5 域 Lead就是我, 我允许你推进到完成"
> **依赖**: [ADR-0026 Managed Agents Runtime 模式参考 — 借鉴 Multica / Pi](../adr/0026-multica-patterns-borrow.md) v0.2 §2.1 模式 4
> **关联 SRS**: [`docs/requirements/SRS-PI-BORROW-001.md`](../requirements/SRS-PI-BORROW-001.md) v0.1 (393 行, 33 KB, 26 FR + 8 NFR + 10 AC + 13 已知缺口 + 6 不抄清单 + 8 角色签字栏)
> **关联 BD**: [`docs/design/BD-PI-BORROW-001.md`](../design/BD-PI-BORROW-001.md) v0.1 (8 段, 5 view 详细 + 数据模型 + 接口 + 模块 + NFR + 已知缺口)
> **关联调研**: [`deliverables/ULYS-175/analysis-pi-borrow-build.md`](../../deliverables/ULYS-175/analysis-pi-borrow-build.md) (353 行, 28 KB)
> **平行 inventory**: [`docs/inventory/multica-gap.md`](./multica-gap.md) v0.1 (Multica 系列平行)
> **受众**: 5 域 Lead 真人 (本次 = Ulysses 兼) / DDD Review / SRE Lead / 详细設計エンジニア

---

## 0. 文档目的

把 Pi (earendil-works/pi v0.87.0, 8 包 TS monorepo) 在 STAR / Mavis 当前现状下的 **借鉴度 / 落地候选 / 优先级 / 工作量估** 落到一张 4 象限表 + 2 张附表，让 Sprint 2-3 / Sprint 3-5 / Sprint 6+ 候选的拍板有实证基础. 跟 `multica-gap.md` Q1-Q4 结构对齐 (但本次是 Pi 而非 SDL = "Single Decision Layer" / Multica).

**Pi 实证规模** (per ULYS-175 调研):
- 仓库: 8 包 monorepo (agent / ai / coding-agent / durable / chord / protocol / server / tui)
- agent 4284 KB / ai 2482 KB / coding-agent 6176 KB / durable 187 KB / chord 506 KB
- 91 个 provider 文件 (`pi-ai/src/providers/`) + `models.generated.ts` 114 KB 自动 catalog
- TypeBox schema 校验 + CBOR framing + 单进程 facet (chord)
- 156 个 Extension hook event (per `pi-coding-agent/src/core/events/`)

---

## 1. 4 象限总表 (per 守门 #11 缺标比错标)

| 象限 | 含义 | Pi 模式 | STAR 现状 | 落地候选 | 优先级 | 工作量估 |
|---|---|---|---|---|---|---|
| **Q1 必做** | 14 状态机 + 静态迁移表跟 Pi 单进程事件流模型直接对齐 | AssistantMessageEvent 12 变体 + StreamFn no-throw | ChatChunk 流式断流即整链 fail, `crates/domain-llm/src/chat.rs:200-218` 无 error 字段 | **PI-1** (本 turn 落档) + **PI-3** Stage 2 | **P0** | 2 + 3 周 |
| **Q1 必做** | TokenUsage 单值, 丢 cache hit | Usage 五元组 (input/output/cacheRead/cacheWrite/cost) | `TokenUsage` 仅 3 字段 (input/output/total), 无 cache 无 cost | **PI-2** (本 turn 落档) | **P0** | 1 周 |
| **Q1 必做** | domain-tool 当前完全空白 (PI-4 起手) | Tool trait 5 方法 (schema/prepareArgs/execute/executionMode/replay) | 只有 init/shutdown/health_check stub, `crates/domain-tool/src/lib.rs:27-43` | **PI-4** Stage 3 (最长路径, 5-7 周) | **P0** | 5-7 周 |
| **Q1 必做** | 12 强制点形同虚设 (PI-5) | beforeToolCall/afterToolCall 钩子 + AgentPolicy::policy_hooks | `crates/domain-agent/src/lib.rs:401-409` AgentPolicy::enforce 已写但 hooks 未接入 | **PI-5** Stage 2 | **P0** | 2 周 |
| **Q2 后续做** | star-taskgraph react-flow 前端增量同步 | JSON Delta 协议 7 ops (retain/insert/delete/annotation/text/path/meta) | polling + console 替代 | **PI-7** Stage 4 | **P1** | 2 周 + 1k UT |
| **Q2 后续做** | star-taskqueue SQLite WAL + 7 态状态机对齐 pi-durable | Compaction 算法 (read_files/modified_files tracking) | star-task 7 态已有, compaction 缺 | **PI-8** Stage 4 | **P1** | 2 周 |
| **Q2 后续做** | domain-worktree 1:N worktree binding 跟 Pi Fork | ContextEdit (omit/replace) + Fork with parent.at | 当前 worktree 14 状态机无 fork 概念 | **PI-9** Stage 4 | **P1** | 2 周 |
| **Q2 后续做** | domain-agent 协作评论机制 | Steering / Follow-up / QueueMode 钩子 | 缺 (per SRS-MULTICA-COLLABORATION 未拍板) | **PI-9** Stage 4 (跟 PI-9 SHOULD 共用 queue.rs) | **P1** | 3 周 |
| **Q3 评估中** | Extension hook event surface | 156 个 event 列表 + EventBus | 缺统一抽象 | **PI-10** ADR-0028 (1 周 ADR-only) | **P2** | 1 周 ADR |
| **Q3 评估中** | star-task 7 态状态机重启恢复 | Checkpoint-driven task resumption | 缺 | **PI-11** P2 | **P2** | (P2) |
| **Q3 评估中** | domain-local-runtime Windows 化 | BashSpawnHook per-tool (spawn 前注入 env/审计) | 缺 | **PI-12** P2 | **P2** | (P2) |
| **Q3 评估中** | pi-durable 2031 行规范通读 | pico-v5.md EntryRecord model/data/edits | 缺规范摘要 | **PI-14** P2 ADR 摘要 | **P2** | (P2) |
| **Q4 不做** | chord 整套 runtime | replicated state + facets + unix socket 单进程 | 多 worker + actix-web + DB, 栈冲突 | ❌ 不抄 (per ULYS-175 §A2) | — | — |
| **Q4 不做** | pi-tui 终端 UI | differential rendering | react-flow + 前端, 栈冲突 | ❌ 不抄 (per §A4) | — | — |
| **Q4 不做** | pi-protocol CBOR | CBOR framing | actix-web JSON + tonic gRPC, 栈冲突 | ❌ 不抄 (per §A3) | — | — |
| **Q4 不做** | 91 provider | TS 生态廉价试错 | 5-7 provider 够, 内部署优先 | ❌ 不抄 (per §A1 + 守门 #11) | — | — |
| **Q4 不做** | OAuth 设备流 provider | pi-ai OAuth flow | 内部署优先 SSO/LDAP, 优先级更高 | ❌ 不抄 (per §D5) | — | — |
| **Q4 不做** | Containerization (Gondolin/OpenShell) | wasm microVM / 另一种 | Windows 无原生 wasm microVM, 走进程隔离 + Docker | ❌ 不抄 (per §A5) | — | — |

---

## 2. 附表 A: PI-1~9 子 issue 拍板时间表

| PI 子 issue | 域 | 估时 | 拍板责任人 | Stage | 启动条件 |
|---|---|---|---|---|---|
| **PI-1** AgentStreamEvent 12 事件协议 | domain-llm | 2 周 | Ulysses (5 域 Lead) | **Stage 1** | 本 turn 已落档代码 + 测试, Stage 1 即时启动 |
| **PI-2** StopReason + Usage 五元组 + ThinkingLevel | domain-llm | 1 周 | Ulysses (5 域 Lead) | **Stage 1** | 本 turn 已落档代码 + 测试, Stage 1 即时启动 |
| **PI-3** AgentLoopBoundary trait + StreamFn no-throw | domain-agent | 3 周 | Ulysses (兼 domain-agent Lead) | Stage 2 | Stage 1 子 issue 全部 `done` 后启动 |
| **PI-4** Tool trait 重写 5 方法 | domain-tool | 5-7 周 (per ULYS-175 估算) | Ulysses (兼 domain-tool Lead) | Stage 3 | Stage 2 子 issue 全部 `done` 后启动 |
| **PI-5** AgentPolicy::policy_hooks 接入 | domain-agent | 2 周 | Ulysses (兼) | Stage 2 | 跟 PI-3 并行启动 |
| **PI-6** JSON Delta 协议 | star-dto | 2 周 | Ulysses (兼 star-dto Lead) | Stage 4 | Stage 3 子 issue `done` 后启动 |
| **PI-7** Compaction 算法 | star-taskqueue | 2 周 | Ulysses (兼 star-taskqueue Lead) | Stage 4 | 跟 PI-6 并行 |
| **PI-8** ContextEdit + Fork with parent.at | domain-worktree | 2 周 | Ulysses (兼 domain-worktree Lead) | Stage 4 | 跟 PI-6 并行 |
| **PI-9** Steering / Follow-up / QueueMode | domain-agent | 3 周 | Ulysses (兼) | Stage 4 | 跟 PI-6 并行 |
| **PI-10** Extension hook event surface (ADR) | docs/architecture | 1 周 | Ulysses (兼架构师) | Stage 1 ADR | Stage 1 子 issue 全部 `done` 后启动 |
| **PI-11** Checkpoint-driven resumption (P2) | star-taskqueue | (P2) | Ulysses (兼) | P2 | Sprint 6+ 拍板 |
| **PI-12** BashSpawnHook per-tool (P2) | domain-local-runtime | (P2) | Ulysses (兼) | P2 | Sprint 6+ 拍板 |
| **PI-13** OAuth 设备流 | (跳过) | — | — | (skip) | 内部署优先 SSO/LDAP, P2 推迟有充分理由 |
| **PI-14** pico-v5.md 通读 → ADR 摘要 (P2) | docs/architecture | (P2) | Ulysses (兼) | P2 | Sprint 6+ 拍板 |

---

## 3. 附表 B: 已落档 (本 turn + prior turns)

| 项目 | 路径 | 行数 | 落档 turn | 状态 |
|---|---|---|---|---|
| **调研报告** | `deliverables/ULYS-175/analysis-pi-borrow-build.md` | 353 行, 28 KB | ULYS-175 turn 1 (2026-09-22 00:27 JST) | ✅ 已落档 |
| **不适配分析** | ULYS-175 评论区 (2026-09-22 00:41 JST) | ~3 KB 评论 | ULYS-175 turn 2 | ✅ 已落档 |
| **SRS-PI-BORROW-001** | `docs/requirements/SRS-PI-BORROW-001.md` | 393 行, 33 KB | ULYS-175 turn 3 (2026-09-22 01:06 JST) | ✅ v0.1 已落档 |
| **SRS-PI-BORROW-001 v0.1.1** (本 turn) | 同上, §10 签字栏更新 | 393 行 | ULYS-175 turn 4 (2026-09-22 01:28 JST 用户拍板) | ✅ v0.1.1 已落档 (本次) |
| **BD-PI-BORROW-001** | `docs/design/BD-PI-BORROW-001.md` | ~280 行, ~25 KB | 本 turn (per ULYS-175 §1.3 承诺 "下个 turn 落档") | ✅ v0.1 已落档 (本次) |
| **inventory/pi-gap.md** | `docs/inventory/pi-gap.md` | 本文件 | 本 turn | ✅ v0.1 已落档 (本次) |
| **PI-1 + PI-2 代码 + 测试** | `crates/domain-llm/src/events.rs` + `chat.rs` + `lib.rs` | 410+ 行新文件 + 14 tests + chat 3 字段 | 本 turn | ✅ 已实装落档 (本次) |

---

## 4. 总结

**本 turn 已交付 (per ULYS-175 2026-09-22 01:28 JST "推进到完成")**:
1. SRS-PI-BORROW-001 v0.1.1 (签字栏更新, 5 域 Lead = Ulysses 兼)
2. BD-PI-BORROW-001 v0.1 (基本設計書)
3. inventory/pi-gap.md v0.1 (本文件)
4. PI-1 + PI-2 代码 + 测试实装 (domain-llm: events.rs + chat.rs + lib.rs)
5. ULYS-175 status 推进 → done (per 守门 #1 调研 issue 完成)
6. PI-3~9 子 issue 池子建立 (per SRS §12 + Stage 1/2/3/4 分组)

**下一步 (next turn)**:
- Stage 1 子 issue (PI-1 + PI-2 + ADR-0028/PI-10) 启动, Sprint 2 开工
- Stage 2 (PI-3 + PI-5) 等 Stage 1 完成后启动

**长期 (Sprint 2-3-5-6)**:
- Stage 1 → Stage 2 → Stage 3 (PI-4 最长路径, 5-7 周) → Stage 4
- 14 项已知缺口逐步关闭
- 6 项不抄清单 hard reject CI lint rule 上线
- 守门 #13 W-T-M 100% 覆盖持续维护

---

## 5. 守门合规 (per AGENTS.md §4)

- ✅ 守门 #1 v15 (新事件触发 — SRS-PI-BORROW-001 立 PI 子 issue 时按 v15 流程走)
- ✅ 守门 #5 (env 安全 — Usage.cost_usd 落档不打印 secret / billing token)
- ✅ 守门 #6 (PowerShell only)
- ✅ 守门 #9 v27 (RPC fallback — StreamFn no-throw 对齐 v27 fallback)
- ✅ 守门 #11 (缺标比错标 — Q4 不做 6 项 + 13 项已知缺口)
- ✅ 守门 #12 v21 ([P] docs 同步)
- ✅ 守门 #13 W-T-M (本表 §1 + BD-PI-BORROW-001 §4)
- ✅ 守门 #14 v4 (Mavis 审核 — author=Ulysses, 8 域签字栏 §10 一次性填齐, 因 1 人 12 角色 + 本人即 5 域 Lead)