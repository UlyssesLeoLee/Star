# PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT — Star 排他与幂等架构 view 实施计划

> **状态**: 🟡 Plan v0.1 (8 子项 0/8 done, 拍板 + IPA 3 文档齐备, 实施待 5 域 Lead 真人到位)
> **生效**: 2026-09-07
> **制定者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签字**: 🟢 Mavis 接手终审 (per 2026-08-27 19:39 + 21:59 JST 用户授权"允许你代签")
> **关联**: [01-requirements.md v1.0](../architecture/2026-09-07-exclusion-idempotency/01-requirements.md) · [02-basic-design.md v1.0](../architecture/2026-09-07-exclusion-idempotency/02-basic-design.md) · [03-detailed-design.md v1.0](../architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md) · [ADR-0048 排他与幂等架构 view 拍板](../architecture/2026-08-26-upgrade/adr/0048-exclusion-idempotency-design.md) · [AGENTS.md §4 守门硬约束](../../AGENTS.md) · [AGENTS.md §4 #13 W/T/M 横展开强约束](../../AGENTS.md) · [STAR-OLU-001.md](../ol/STAR-OLU-001.md) (1 SRE · 周 ≈ 1.2M tokens) · [docs/automation-design.md](../automation-design.md)
> **类比报告**: [PHASE-LANGGRAPH-TMO-IMPL-REPORT.md](PHASE-LANGGRAPH-TMO-IMPL-REPORT.md) v0.3 (TMO 7/7 done, 实施完毕参考) · [PHASE-D2-CLI-IMPL-REPORT.md](PHASE-D2-CLI-IMPL-REPORT.md) · [PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md](PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md)

---

## 0. 目的 (Purpose)

本文档规划 **Star 排他与幂等架构 view (Star-EI)** 8 子项 + 5 张新表 + 4 层责任矩阵 + 6 协议的实施, per [01 §2-§5 14 功能 + 5 NFR + 4 制約 + 8 想定シナリオ](../architecture/2026-09-07-exclusion-idempotency/01-requirements.md) + [02 §1-§9 18 组件 + 4 选型 + 5 表 + 6 协议](../architecture/2026-09-07-exclusion-idempotency/02-basic-design.md) + [03 §1-§8 23 module + 5 状态机 + 5 时序 + 5 表 DDL + 18 UT/8 IT/12 E2E](../architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md) 落地.

**实施目标**: 4 层排他/幂等全栈覆盖, 5 张新表 (3 T + 1 T archive + 1 M) per 守门 #13 W/T/M 严格分类 + RLS 13 类 + audit 100% 必携, 6 协议 (Idempotency-Key / X-Dispatch-Id / X-Task-Lock-Token / X-Trace-Id / advisory lock / lock 释放) 4 层贯穿, 18 组件 (5 Rust + 8 Python + 5 TypeScript) 落地, 8 想定シナリオ E2E 100% pass.

**v0.1 状态 (2026-09-07 落档)**: 拍板 (per `ask_37d138ffb93a12279b35a46e` 4 推荐项) + IPA 3 文档 v1.0 + ADR-0048 v1.0 全部齐备, 8 子项 0/8 done 实施待 5 域 Lead 真人到位 (per 守门 #14 v2 拍板 D 维持). 预估 ~2.3M tokens 总体 (per [STAR-OLU-001.md §2](../ol/STAR-OLU-001.md)).

**触发条件**: 5 域 Lead 真人到位后 (per 守门 #14 v2 拍板 D), 排他/幂等 view 实施进入"装装"阶段 (per AGENTS.md §4 #25 拍板 P0 依赖真人 Lead 拍板启动). 当前阶段 Mavis 临时代签维持, 真人到位后追溯签字覆盖 (per 守门 #14 拍板 D 维持).

**实施路径** (per 守门 #19 + #9 v3 + #24): 5 张新表 DDL + 5 Rust crate (`star-mutex`) 走 `cargo check/clippy/test --workspace` 实证 0 err; 8 Python 脚本走 `scripts/automation/exclusion/<purpose>.py` 落地 (per 守门 #19 v19); 5 TypeScript 走 `frontend/src/lib/exclusion/<purpose>.ts`; 调试控制台走 `console_server.py` 扩展 `/api/exclusion/*` 8 端点 (per 守门 #22 不污染 main 编译).

## 1. 任务完成矩阵 (8 子项 + 5 张新表 + 6 协议)

### 1.1 8 子项 (per 03 §1.1 模块布局)

| # | 子项 | 内容 | 模块 | token 估 | 实施路径 | 守门 | 状态 (v0.1) |
|---|---|---|---|---|---|---|---|
| **EX-01** | 5 张新表 DDL + RLS 13 类 | idempotency_keys (T) / lease_log (T) / advisory_lock_audit (T) / idempotency_keys_archive (T) / exclusion_policy_master (M, SCD Type 2) | M-无 (SQL migration) | ~0.2M | `docs/migrations/2026-09-07-exclusion-rls.sql` + `docs/migrations/2026-09-07-audit-trigger.sql` | #13 W/T/M / #13 c RLS / #13 d audit | 🟡 **plan** |
| **EX-02** | star-mutex 共享 crate (Rust) | `crates/star-mutex/src/lib.rs` 18 module (M-08..M-14) + `domain_mutex` proc-macro | M-08..M-14 (Rust) | ~0.3M | `crates/star-mutex/Cargo.toml` + 7 源文件 + 1 proc-macro | #6 PowerShell / #7 0 unsafe / #13 | 🟡 **plan** |
| **EX-03** | L0 DispatchLockManager (Python) | `scripts/automation/exclusion/dispatch_lock.py` + 派发级排他 + 派发级幂等 | M-16 + M-17 (Py) | ~0.3M | `scripts/automation/exclusion/dispatch_lock.py` + `idempotency_key_store.py` | #19 Python 化 / #9 v3 / #5 env | 🟡 **plan** |
| **EX-04** | L1 SubAgentLock (Python) | `scripts/automation/exclusion/subagent_lock.py` + lease + heartbeat + LockWatcher | M-18 + M-19 (Py) | ~0.3M | `scripts/automation/exclusion/subagent_lock.py` + `lock_watcher.py` | #19 / #9 v3 / 守门 #14 | 🟡 **plan** |
| **EX-05** | UI IdempotencyManager (TypeScript) | `frontend/src/lib/exclusion/idempotency.ts` + 4 component | M-01..M-07 (TS) | ~0.2M | `frontend/src/lib/exclusion/{idempotency,lock_status,trace_propagator,business_hash}.ts` + 3 component | #6 PowerShell / #7 0 unsafe | 🟡 **plan** |
| **EX-06** | star-mcp 16 tool 幂等改造 (Rust) | `crates/star-mcp/src/middleware/idempotency.rs` + 16 tool 加 Idempotency-Key 解析 | M-15 (Rust) + 16 tool 改造 | ~0.4M | `crates/star-mcp/src/middleware/idempotency.rs` + 16 tool 改造 patch | #6 / #7 / 跟 ADR-0032 MCP 一致 | 🟡 **plan** |
| **EX-07** | 4 层统一可观测性 (Python) | `scripts/automation/exclusion/lock_metric_exporter.py` + `lock_leak_alerter.py` + `archive_cron.py` | M-20..M-22 (Py) | ~0.3M | `scripts/automation/exclusion/lock_metric_exporter.py` + `lock_leak_alerter.py` + `archive_cron.py` | #19 / #9 v3 | 🟡 **plan** |
| **EX-08** | 集成测试 + 性能压测 (Py + Rust) | 18 UT + 8 IT + 12 E2E + S-08 1000 并发压测 | 18 UT + 8 IT + 12 E2E | ~0.3M | `tests/e2e/*.spec.ts` + `crates/star-mutex/tests/*.rs` + `scripts/automation/exclusion/tests/*.py` | #1 4 守门 (check/fmt/clippy/test --workspace) | 🟡 **plan** |
| **总** | **8 子项** | **5 表 + 18 组件 + 6 协议 + 38 测试** | **M-01..M-22** | **~2.3M** | — | — | **0/8 plan** |

### 1.2 守门 #13 W/T/M 分类验证 (5 张新表)

| # | 表名 | 分类 | 物理删除 | 审计 trigger | RLS 13 类 | 验证 (v0.1 plan) |
|---|---|---|---|---|---|---|
| 1 | `idempotency_keys` | **T** Transaction | 禁止 (24h 后归档到 archive) | `trg_idem_keys_audit` | 13 类 (tenant_id + workspace_id) | 🟡 plan: 走 `docs/migrations/2026-09-07-exclusion-rls.sql` |
| 2 | `lease_log` | **T** Transaction | 禁止 (append-only 永久) | `trg_lease_log_audit` | 13 类 | 🟡 plan |
| 3 | `advisory_lock_audit` | **T** Transaction | 禁止 (append-only 永久) | `trg_lock_audit_audit` | 13 类 | 🟡 plan |
| 4 | `idempotency_keys_archive` | **T** Transaction | 禁止 (append-only 永久) | 同 #1 | 13 类 | 🟡 plan (24h cron 移动) |
| 5 | `exclusion_policy_master` | **M** Master | 禁止 (SCD Type 2 永久) | `trg_excl_policy_audit` | 13 类 | 🟡 plan (SCD Type 2 trigger) |

**派生规** (per 守门 #13 a + c + d):
- (a) W = 物理删除 / タイマー失効 / 短 TTL 明示 retention
- (b) T = 物理删除禁止 + 監査必須 + RLS 13 類必携
- (c) M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携
- (d) Master 100% RLS / Transaction 100% audit / Work 100% retention_period

**验证 (per 守门 #13 a/c/d)**:
- 5 表 100% 覆盖 (3 T + 1 T archive + 1 M) 禁止混在一括列举 ✓
- 5 表 RLS 13 类 100% 必携 ✓
- 5 表 audit trigger 100% 必携 (per ADR-0043 WORM) ✓
- 1 M 表 SCD Type 2 必携 (valid_from / valid_to) ✓

### 1.3 6 协议落地矩阵

| 协议 | 层 | 落地组件 | 验证 |
|---|---|---|---|
| `Idempotency-Key` header | UI → L0 → L1 | M-01 IdempotencyManager (TS) + M-15 star-mcp-idem-middleware (Rust) + M-17 IdempotencyKeyStore (Py) | E2E-01 + E2E-02 + E2E-10 |
| `X-Dispatch-Id` header | L0 → L1 | M-16 DispatchLockManager (Py) | E2E-03 + E2E-05 |
| `X-Task-Lock-Token` | L1 SubAgent | M-18 SubAgentLock (Py) | E2E-06 |
| `X-Trace-Id` (4 层共享) | UI → L0 → L1 → L3 | M-03 TraceIdPropagator (TS) + M-12 TraceIdPropagator (Rust) + M-16/M-18 (Py) | E2E-09 |
| PG advisory lock | L0 / L1 / L3 | M-14 PgAdvisoryLock (Rust) + M-16/M-18 (Py via asyncpg) | E2E-03 + E2E-04 + E2E-12 |
| lock 释放协议 | L1 / L3 | M-18 SubAgentLock.release + M-08 DomainMutex (Drop guard) | E2E-06 + E2E-11 |

### 1.4 18 组件落地矩阵 (per 02 §5.1)

| # | 组件 | 层 | 语言 | 路径 | 子项 |
|---|---|---|---|---|---|
| C-01 | `IdempotencyManager` | L0 UI | TypeScript | `frontend/src/lib/exclusion/idempotency.ts` | EX-05 |
| C-02 | `LockStatusToast` | L0 UI | TypeScript | `frontend/src/components/exclusion/LockStatusToast.tsx` | EX-05 |
| C-03 | `DispatchLockManager` | L1 TopAgent | Python | `scripts/automation/exclusion/dispatch_lock.py` | EX-03 |
| C-04 | `IdempotencyKeyStore` | L1 TopAgent | Python | `scripts/automation/exclusion/idempotency_key_store.py` | EX-03 |
| C-05 | `SubAgentLock` | L2 SubAgent | Python | `scripts/automation/exclusion/subagent_lock.py` | EX-04 |
| C-06 | `LockWatcher` | L2 SubAgent | Python | `scripts/automation/exclusion/lock_watcher.py` | EX-04 |
| C-07 | `DomainMutex` | L3 Domain | Rust | `crates/star-mutex/src/lib.rs` | EX-02 |
| C-08 | `StarMutexAdapter` | L3 Domain | Rust | `crates/star-mutex/src/adapter.rs` | EX-02 |
| C-09 | `LockAuditLogger` | L4 Cross | Rust | `crates/star-mutex/src/audit.rs` | EX-02 |
| C-10 | `ExclusionPolicyLoader` | L4 Cross | Rust | `crates/star-mutex/src/policy_loader.rs` | EX-02 |
| C-11 | `TraceIdPropagator` | L4 Cross | Rust | `crates/star-mutex/src/trace.rs` | EX-02 |
| C-12 | `star-mcp-idem-middleware` | L1 MCP | Rust | `crates/star-mcp/src/middleware/idempotency.rs` | EX-06 |
| C-13 | `LockMetricExporter` | L4 Obs | Python | `scripts/automation/exclusion/lock_metric_exporter.py` | EX-07 |
| C-14 | `LockLeakAlerter` | L4 Obs | Python | `scripts/automation/exclusion/lock_leak_alerter.py` | EX-07 |
| C-15 | `ArchiveCronJob` | L4 Ops | Python | `scripts/automation/exclusion/archive_cron.py` | EX-07 |
| C-16 | `exclusion_rls_migration` | L4 DB | SQL | `docs/migrations/2026-09-07-exclusion-rls.sql` | EX-01 |
| C-17 | `audit_trigger_setup` | L4 DB | SQL | `docs/migrations/2026-09-07-audit-trigger.sql` | EX-01 |
| C-18 | `exclusion_debug_console` | L4 Debug | Python | `scripts/automation/console_server.py` 扩展 | EX-07 |

**18 组件** = 5 Rust (C-07..C-12 减 C-12) + 8 Python (C-03..C-06, C-13..C-15, C-18) + 5 TypeScript (C-01..C-02, 含 trace_propagator / business_hash / lock_status sub-component) + 2 SQL (C-16..C-17) + 1 Rust (C-12 star-mcp).

## 2. 验证摘要 (per 守门 #1 4 项全 0 错)

实施时验证 (per 守门 #1 + #19 v19 + #9 v3 + #22 + #23):

### 2.1 Rust 部分 (EX-02 + EX-06)

- (1) `cargo check --workspace --all-targets -j 4` 0 err (per 守门 #1 v19 + 守门 #1 v25)
- (2) `cargo fmt + clippy` 0 警告 (per 守门 #6 v2 + 守门 #7 v3 advisory)
- (3) `cargo test -p star-mutex --lib -j 4` 100% pass (per 守门 #1 v25 跳过 workspace)
- (4) `cargo build --release + doc + bench --no-run` 全 0 错

### 2.2 Python 部分 (EX-03 + EX-04 + EX-07)

- (1) `python -m pytest scripts/automation/exclusion/tests/` 100% pass
- (2) `python -m py_compile` 0 err
- (3) `python -m ruff check` 0 警告
- (4) `python -m mypy --strict` 0 err

### 2.3 TypeScript 部分 (EX-05)

- (1) `pnpm tsc --noEmit` 0 err
- (2) `pnpm eslint` 0 警告
- (3) `pnpm test` 100% pass
- (4) `pnpm build` 0 错

### 2.4 SQL 部分 (EX-01)

- (1) `psql -f docs/migrations/2026-09-07-exclusion-rls.sql` 0 err
- (2) `psql -f docs/migrations/2026-09-07-audit-trigger.sql` 0 err
- (3) RLS 13 类 policy 100% 验证 (per 守门 #13 c)
- (4) audit trigger 100% 验证 (per 守门 #13 d)

## 3. 已知缺口 (per 守门 #11 缺标比错标安全)

| # | 缺口 | 影响 | 拍板需求 | 跟踪 |
|---|---|---|---|---|
| **G-EI-01** | 5 域 Lead 真人未到位, 排他/幂等实施进入"装装"阶段触发条件需 DDD Review Lead 拍板 | 中 | DDD Review Lead 真人到位后 | 跟踪 per [docs/recruitment/5-business-domain-lead-referral.md v0.1](../recruitment/5-business-domain-lead-referral.md) |
| **G-EI-02** | PG 跨 cluster 锁不支持, 后续如需扩展需引入 Etcd 备选 | 中 | 跨 cluster 部署需求出现时 | 备选 D-01 v2 拍板 |
| **G-EI-03** | 锁 schema 演进走 DDD Review 拍板, 5 张新表 DDL 需 PG 容量规划 | 中 | 实施阶段 EX-01 时 | 实施 EX-01 时拍板 |
| **G-EI-04** | 双键 dedup 业务主键 hash 算法的版本兼容性 (e.g. work_item 主键从 UUID 改 (tenant, work_id) 复合) 需 DDD Review 拍板 | 中 | 业务 schema 演进时 | 跟踪 per 5 域 Lead 到位 |
| **G-EI-05** | 锁泄漏告警阈值 (24h 1h) 需 SRE Lead 拍板 | 低 | EX-07 实施时 | SRE Lead 真人到位 |
| **G-EI-06** | 锁 metric 5 项 + Grafana dashboard 需 SRE Lead 评审 | 低 | EX-07 实施时 | SRE Lead 真人到位 |
| **G-EI-07** | 16 tool 幂等改造涉及 5 域 Lead RACI 责任分配 (哪个域负责哪个 tool) | 中 | EX-06 实施时 | 5 域 Lead 到位后 |
| **G-EI-08** | 排他/幂等 view 跟现有 LangGraph / Agent Runtime view 集成点 (e.g. 16 tool 幂等跟 Agent Runtime LLM/HTTP/MCP 池调用) 需 DDD Review 拍板 | 中 | EX-06 实施时 | DDD Review Lead 拍板 |

## 4. 子代理失败接手清单 (per 7 子代理派生规则 + 守门 #9 v3)

实施时若子代理 RPC 不可靠 (per 守门 #9 v3 实证), 走 subprocess 替代:

- **优先路径**: `SubagentDispatcher.brief(task_id, content, agent)` → `docs/briefs/<task_id>.md`
- **subprocess 路径**: `python scripts/automation/<purpose>.py` 走 console_server.py 8080 端口 (per 守门 #22)
- **AI mock 路径**: 调试页 AI 修改走 `scripts/automation/ai_edit_mock.py` 模板生成 (per 守门 #23), 不开 OpenAI/Anthropic 第三方 API

**8 子项接手清单**:

| 子项 | 推荐子代理 | 失败接手 (per 守门 #9 v3) |
|---|---|---|
| EX-01 SQL DDL | worker (DDL 强 schema 任务) | 直接 Mavis 接手写 DDL |
| EX-02 Rust crate | worker (大文件 Rust 任务) | Mavis 接手 + 分批 commit |
| EX-03 Python DispatchLock | worker (中等 Python 任务) | console_server.py 走 subprocess 实证 |
| EX-04 Python SubAgentLock | worker (中等 Python 任务) | 同上 |
| EX-05 TypeScript UI | worker (前端任务) | Mavis 接手 + pnpm build 实证 |
| EX-06 star-mcp 16 tool | worker (大改 16 tool 任务) | Mavis 接手 + 走守门 #1 v25 跳过 workspace |
| EX-07 可观测性 | worker (中等 Python 任务) | console_server.py 走 subprocess 实证 |
| EX-08 测试 + 压测 | worker (大规模测试任务) | 走 S-08 1000 并发压测脚本 |

## 5. 守门规则 (15-17 项, per AGENTS.md §4)

实施时必须遵守的守门 (15-17 项):

| # | 守门 | 适用 | 验证 |
|---|---|---|---|
| 1 | 守门 #1 4 守门 (check/clippy/test/build) | EX-02 + EX-06 Rust | cargo 命令实证 |
| 1a | 守门 #1 v25 cargo test 跳过 workspace | EX-02 (单 crate 测) | `cargo test -p star-mutex --lib -j 4` |
| 2 | 守门 #5 环境变量安全 | EX-03 + EX-04 + EX-07 Python | 不打印 DATABASE_URL, 仅引用 |
| 3 | 守门 #6 PowerShell only | 全栈 | PowerShell 命令 |
| 4 | 守门 #7 0 unsafe | EX-02 + EX-06 Rust | cargo clippy 实证 |
| 5 | 守门 #9 v3 子代理 dispatch 走 subprocess | EX-08 测试 | console_server.py 8080 实证 |
| 6 | 守门 #10 author=Ulysses | git commit | 5 commit author=Ulysses |
| 7 | 守门 #11 缺标比错标 | G-EI-01..08 | §3 显式列出 |
| 8 | 守门 #12 AI 协作文档治理 | 全栈文档 | 禁回溯叙事 + BAS git log --follow |
| 9 | 守门 #13 W/T/M 严格分类 | EX-01 5 张新表 | DDL 实证 + 验证脚本 |
| 10 | 守门 #13 a L0 协调 L1↔L1 | EX-02 + EX-04 架构 | 守门 #13 a 派生规 |
| 11 | 守门 #13 c Master 100% RLS | EX-01 exclusion_policy_master | RLS policy DDL 实证 |
| 12 | 守门 #13 d Transaction 100% audit | EX-01 3 张 T 表 | audit trigger DDL 实证 |
| 13 | 守门 #14 v2 5 域 Lead 临时代签 | 全栈 | 报告签字栏 5 角色 Mavis 临时代签 |
| 14 | 守门 #15 守门 #12 死循环饱和 | docs 同步 | 实施时按需触发 |
| 15 | 守门 #19 agent 交互 Python 化 | EX-03/04/07 Python | 走 `scripts/automation/exclusion/<purpose>.py` |
| 16 | 守门 #22 控制台不污染 main | EX-07 console_server | `cargo check --workspace --lib` 0 err 实证 |
| 17 | 守门 #23 AI mock 不开外部 API | EX-08 调试 | ai_edit_mock.py 模板生成 |

**16 守门** (per 守门 #1 v25, 实证 4 守门改为 advisory, 加 守门 #25 实证).

## 6. 签字栏 (5 角色)

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 |
| SRE Lead | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 (per 守门 #14 v2 拍板 D 临时代签) |
| 平台 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 (per 守门 #14 v2 拍板 D 临时代签) |
| 评审主持 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 (per 守门 #14 v2 拍板 D 临时代签) |
| PM | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 (per 守门 #14 v2 拍板 D 临时代签) |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-07 20:55 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初稿, 8 子项 0/8 plan + 5 张新表 W/T/M 严格分类 + 6 协议落地 + 18 组件 + 16 守门 + 8 缺口 | per `ask_37d138ffb93a12279b35a46e` 4 推荐项拍板 + IPA 3 文档 v1.0 落档 |
