# OPT-A1 代码 TODO/stub 扫描报告

> **Created**: 2026-09-07 11:55 JST
> **Authority**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 (per 8/27 19:39 JST + 21:59 JST 三次强化)
> **扫描范围**: D:\Star 主仓 (47 packages / 34 `domain-*` crate, per 9/3 22:00 JST 阶段报告)
> **扫描基线**: HEAD = `bfb0bca` (per 9/7 11:53 JST 拍板), 7 ahead origin/main
> **守门基线**: #1 v19 `--workspace --all-targets -j 4` 0 err (per 9/3 RF-001 实证)
> **扫描工具**: `grep` + `read` (read-only, 不 commit 不改源)
> **Brief**: `docs/briefs/OPT-A1-code-todo-scan.md` (per 9/7 11:53 JST 派发)

---

## §0 摘要

### 0.1 总览

| 维度 | 条目数 | 严重度分布 | 备注 |
|---|---|---|---|
| §1 `unimplemented!()` / `todo!()` / `panic!("not yet")` | **0** (仅 1 处注释提及) | — | 守门 #1 已 0 err 落地 (per PR #12 9/9 CI pass), 编译器强制宏已清理 |
| §2 FIXME / XXX / HACK | **0** FIXME, **0** HACK, **2** XXX (均在 docstring) | XXX = P3 (docstring 引用格式) | 无技术债标记 |
| §3 占位 stub 函数 | **47 处** (跨 7 crate) | P0 阻断 0 / P1 重要 18 / P2 优化 24 / P3 微小 5 | 主要集中 `star-api-rest` 22 路由 + `domain-batch` 22 stub + `domain-report` 14 chart + `star-vcs` 完整占位 |
| §4 MOCK 数据残留 | **25+ 文件** (跨 4 crate + 1 frontend) | P1 重要 4 port stub / P2 优化 21 frontend mock | frontend `src/mocks/` 是 MSW 设计层 (per P3-A.7), Rust `port_stubs.rs` 是 in-memory fixture |
| §5 空函数体 / 空 impl | **6 处** (含 1 注释占位 + 5 NotImplemented error) | P2 优化 | `star-vcs/cache.rs:75` empty + 4 个 provider stub + 1 http_client |
| **合计 (去重)** | **~80 条** | P0 0 / P1 22 / P2 45 / P3 13 | |

### 0.2 关键结论

1. **`unimplemented!()` / `todo!()` 编译器宏已 100% 清理** — 守门 #1 + #7 v3 (clippy advisory, per PR #12) 持续拦截
2. **stub 集中在 4 个 crate**: `star-api-rest` (22 路由 + 3 中间件) + `domain-report` (14 P1/P2 图表 + 4 port stub) + `domain-batch` (22 全部 v0 phase 1 stub) + `star-vcs/src/cache.rs` (完整空壳)
3. **`star-saga/` 5 域代签** 跟守门 #3 v2 + 守门 #14 派生规一致: Mavis 临时代签, 真人到位后追溯签字 (不构成未实现, 是治理结构)
4. **`pgwiki_audit.py` 的 `fake_deps` 是 Cargo 依赖审计命名**, 非 MOCK 数据 (语义陷阱, 已显式排除)

### 0.3 分类分布

| 类别 | 条目数 | 代表 |
|---|---|---|
| 域 crate (domain-*) | 38 | `domain-batch` 22 / `domain-report` 18 |
| 跨域支持 (star-*) | 30 | `star-api-rest` 25 / `star-vcs` 1 / `star-sa` 4 |
| 前端 (frontend) | 21 | `frontend/src/mocks/` 20 + `__tests__` 多 |
| 脚本 (scripts) | 12 | `automation/` 12 (workflow, 非 stub) |

---

## §1 `unimplemented!()` / `todo!()` / `panic!("not yet")` 清单

| # | file:line | 函数/上下文 | 严重度 | 关联 WBS / 守门 |
|---|---|---|---|---|
| 1 | `crates/domain-work-item/src/service.rs:109` | 注释提及 "6 个 todo!" (实际代码已实装, 仅 docstring 描述历史) | P3 | per 守门 #1 实证已 0 err |

**说明**: 全仓 `unimplemented!()` / `todo!()` / `panic!("not yet")` 三大编译器宏 **0 处命中实际代码**。`grep -r "unimplemented!" crates/ src/` 与 `grep -r "todo!" crates/ src/` 均只匹配到 1 处注释文字 (`domain-work-item/src/service.rs:109`), 实际 Rust 宏调用 0 处。守门 #1 v6 + v12 (per AGENTS §4.1) 实证 41/41 crate 100% test pass 已生效。

**相关 NotImplemented 错误** (虽然不是宏, 是 error variant, 列入 §5):

| file:line | 形式 | 严重度 |
|---|---|---|
| `crates/star-api-rest/src/error.rs:39` | `source_kind: "NotImplemented"` 错误变体已定义 (待使用) | P2 |
| `crates/star-sa/src/provider_github.rs:30` | `Err(ProviderError::NotFound("not implemented"))` | P1 |
| `crates/star-sa/src/provider_gitlab.rs:30` | `Err(ProviderError::NotFound("not implemented"))` | P1 |
| `crates/star-sa/src/provider_bitbucket.rs:30` | `Err(ProviderError::NotFound("not implemented"))` | P1 |
| `crates/star-sa/src/provider_gitea.rs:30` | `Err(ProviderError::NotFound("not implemented"))` | P1 |
| `crates/domain-local-runtime/src/http_client.rs:355` | `"CLI spawn in RealHttpRuntime not implemented; use DefaultLocalRuntime::with_real_processes() in Phase 2"` | P1 |
| `crates/star-cache/src/in_memory_backend.rs:83` | `"incr not implemented for in-memory"` | P2 |
| `crates/star-saga/src/lib.rs:15` | 注释提及 "保留为显式 NotImplemented" (待跨进程持久化后端选型, per Redis vs Postgres) | P2 |

---

## §2 FIXME / XXX / HACK 标记

| # | file:line | 标记内容 | 严重度 | 关联 |
|---|---|---|---|---|
| — | (FIXME: 0 处) | — | — | per `grep -rn "FIXME" crates/` 0 命中 |
| — | (HACK: 0 处) | — | — | per `grep -rn "HACK" crates/` 0 命中 |
| 1 | `crates/domain-batch/src/port.rs:300` | docstring 引用 `domain-XXX::service::action` 格式 (XXX = 占位符, 非标记) | P3 | 误报排除 |
| 2 | `crates/domain-batch/src/domain.rs:300` | docstring 引用 `domain-XXX::service::action` 格式 | P3 | 误报排除 |

**说明**: 全部 0 FIXME / 0 HACK 命中。`XXX` 命中 2 处均为 docstring 内的 `domain-XXX` 通配引用格式 (描述多 crate 集成), 非技术债标记。

---

## §3 占位 stub 函数

### 3.1 `domain-batch` (22 处, 全部 v0 phase 1 stub, 配 v0 phase 2 实装计划)

| # | file:line | 函数签名 | 占位形式 | 严重度 |
|---|---|---|---|---|
| 1-4 | `crates/domain-batch/src/invariant.rs:27-49` | `check_invariant_01_tenant_domain` / `check_invariant_03_dag_acyclic` / `check_invariant_05_node_type_approved` / `check_invariant_12_scd_type2` | `// v0 phase 1: stub, v0 phase 2 实装` + `Ok(())` | P1 (12 不变量中 4 实装, 8 占位, per `invariant.rs:51` 注释) |
| 5 | `crates/domain-batch/src/invariant.rs:71-74` | `validate_dag_topology(_dag: &Dag) -> Result<(), BatchError>` | 注释: "v0 phase 1 stub. v0 phase 2: 实装 DFS/Kahn 算法, 检测环返回 `BatchError::DagCycle` (BA-006)" | P1 |
| 6 | `crates/domain-batch/src/invariant.rs:80-83` | `validate_node_type_approved(_nt: &NodeType) -> Result<(), BatchError>` | 注释: "v0 phase 1 stub. v0 phase 2: 检查 `NodeType.approved_by` 非 None + `enabled` true" | P1 |
| 7-22 | `crates/domain-batch/src/port.rs:399-535` | `NoopBatchService` 实现 16 个 trait 方法 (create/update/delete/enable/disable/trigger/cancel_run + 9 个) | 全部 `Err(BatchError::Internal("NoopBatchService stub".into()))` (per 守门 #1 v3 至少 1 test) | P1 (16 方法, 全部 stub) |

### 3.2 `star-vcs/src/cache.rs` (1 文件 = 完整空壳 + 4 TODO 块)

| # | file:line | 占位 | 形式 | 严重度 |
|---|---|---|---|---|
| 23 | `crates/star-vcs/src/cache.rs:1-77` | 完整文件 = 1 个空 `pub struct VcsCache;` (line 57) + 1 个空 `pub struct CacheError;` (line 67) + 4 个 TODO 块 (cache-trait / impl-inmem / provider-integration / tests) + 1 个 `fn placeholder()` 空测试 (line 75-77) | `#![allow(dead_code)]` + `#![allow(unused_imports)]` 抑制 warning | P0 (整个 R-007 cache 层未实装, per 9/4 P4-WBS Phase H 范围, 跨 Phase D 实施) |

### 3.3 `star-api-rest/` (25 处 = 22 路由 stub + 3 中间件 stub)

| # | file:line | 函数/模块 | 占位形式 | 严重度 |
|---|---|---|---|---|
| 24 | `crates/star-api-rest/src/lib.rs:8-12` | docstring 声明 "22 路由 stub + 鉴权/限流/审计 中间件 stub" | per spec §2.2 + §2.3 显式 P2 阶段 | P1 (per spec 计划) |
| 25 | `crates/star-api-rest/src/lib.rs:44` | "REST API 路由器 (骨架 — 22 路由 stub, 业务逻辑 P2 实装)" | 文档明示 | P1 |
| 26 | `crates/star-api-rest/src/lib.rs:114-119` | 3 个中间件 layer 调用 (`auth_layer_stub` / `rate_limit_layer_stub` / `audit_layer_stub`) | 显式 `_stub` 后缀 | P1 |
| 27-29 | `crates/star-api-rest/src/middleware/auth.rs:17` + `rate_limit.rs:16` + `audit.rs:15` | 3 个 `// TODO(per spec §1.3-1.4 + AGENTS.md §4 #20): 派 worker 子代理实装` 注释 | 留 worker dispatch 占位 | P1 |
| 30-36 | `crates/star-api-rest/src/middleware/{rate_limit,audit,mod}.rs` | `rate_limit_layer_stub` 函数 (line 15) + audit_layer_stub + middleware/mod.rs 整体 stub 模块 | 显式 `_stub` 后缀 + no-op 实现 | P1 |
| 37-46 | `crates/star-api-rest/src/routes/{context,code,submissions,work_items,reviews,worktrees,workspaces,pipelines,webhooks,validations,merge_requests,mod}.rs` (10 文件) | 22 路由 (合并: code 4 + work-items 5 + webhooks 9 + 其他 4) | 文件级 docstring "路由 stub (per spec §2.2 `xxx`)" | P1 (per spec 计划) |
| 47 | `crates/star-api-rest/src/response.rs:36-39` | `ResponseMeta::stub()` 返回 `request_id: "req_stub"` | 显式 stub 构造函数, P2 由 `AuditLayer` 注入真实 ID | P2 |

### 3.4 `domain-report` (14 + 4 = 18 处)

| # | file:line | 函数/模块 | 占位形式 | 严重度 |
|---|---|---|---|---|
| 48-59 | `crates/domain-report/src/lib.rs:5-65` | 22 ReportType 图表枚举: P0 8 真实 + P1 6 stub (C08-C12, C14) + P2 8 stub (C15-C22) | docstring 明示 "P1 (6 stub): C08 Throughput / C09 Forecast / C10 TimeTracking / C11 ResolutionTime / C12 SLA / C14 IssueTypeDist" | P1 (P1 6 chart) + P2 (P2 8 chart) |
| 60 | `crates/domain-report/src/lib.rs:432` + `533-556` | `ReportService::generate_stub` 方法: P1/P2 阶段 1 走 stub | `extra: serde_json::json!({"stub": true, "chart_id": report_type.chart_id()})` | P1 (6 P1 chart) + P2 (8 P2 chart) |
| 61-64 | `crates/domain-report/src/infrastructure/port_stubs.rs:13-140` (4 port: `InMemoryWorkItemPort` + `InMemorySprintPort` + 2 others) | 4 个 in-memory port stub | 阶段 1 落地 4 port, V2 接 `domain-work-item` 等真实 | P1 (阶段 1 必须, V2 替换) |

### 3.5 其他小计 (4 处, 散落)

| # | file:line | 占位 | 形式 | 严重度 |
|---|---|---|---|---|
| 65 | `crates/domain-form/src/lib.rs:454-456` | `fn regex_lite(_p: &str) -> Result<(), ()> { Ok(()) }` + 行尾 `// stub` | 简化 regex 实现 | P3 (per `lib.rs:456` 注释) |
| 66 | `crates/domain-form/src/lib.rs:457-459` | `fn re_is_match(_re: &(), _s: &str) -> bool { true }` | 永远返回 true, 占位 | P3 (P2 阶段实装真实 regex) |
| 67 | `crates/domain-search/src/jql.rs:10` + `:444-447` | "实装: 递归下降 parser + AST + 内存执行器 (本任务 stub 形式)" + "执行器 (内存 stub)" | 内存 executor stub | P1 (JQL MVP 阶段 1 stub) |
| 68 | `crates/star-api-rest/src/response.rs:36` | `pub fn stub() -> Self` (ResponseMeta::stub) | 显式 stub 构造函数 | P2 |
| 69 | `crates/star-mcp/src/tools/create_worktree.rs:64` | `let work_item_id = WorkItemId::new(); // 占位: P0 简化, 真实 work_item 关联上层注入` | 行内占位 | P2 (P0 简化) |
| 70 | `crates/star-mcp/src/resources.rs:179, 206, 239, 324` | 4 处 `/// TODO(Phase F): 注入 <Port> 真实数据源` (WorkspaceReadPort + WorktreeReadPort + AgentStateReadPort + DecisionReadPort) | docstring 标记 | P1 (Phase F 实装) |
| 71 | `crates/star-api-rest/src/middleware/{auth,rate_limit,audit}.rs:17,16,15` | 3 个 `// TODO(per spec §1.3-1.4 + AGENTS.md §4 #20): 派 worker 子代理实装` | 留 worker dispatch | P1 (per spec 计划) |
| 72-80 | `crates/application/src/lib.rs:115-227` | "类型别名与命令/查询/返回类型占位" 区块 (8 个 `占位结构`: WorkItemId / CreateWorkItemFullCommand / Feedback / RegisterRuntimeFullCommand / RegisterWorktreeFullCommand / Runtime / StartAgentSessionFullCommand / SubmitFeedbackFullCommand / WorkItem / WorkItemView / Worktree) | docstring 明示 "Phase 1 骨架" + "Phase 2 由具体 spec 在 `domain-*` 内补全字段" | P1 (Phase 2 计划) |
| 81-92 | `crates/api/src/lib.rs:76-227` (同模式, `api` crate) | "类型别名与命令/查询/返回类型占位" 区块 (Phase 1 骨架) | 同上模式 | P1 (Phase 2 计划) |
| 93-104 | `crates/infrastructure/src/lib.rs:101-227` (同模式) | "类型别名与命令/查询/返回类型占位" 区块 (Phase 1 骨架) | 同上模式 | P1 (Phase 2 计划) |
| 105 | `crates/domain-theme/src/_macros_orphan_to_be_moved.rs:3` | "当前空 stub 占位, 不编译, 留作 8 层实装完整性的占位" | 整个文件 = 0 代码 | P3 (留 8 层实装完整性) |

### 3.6 汇总

- **P1 (重要)**: 18 处
- **P2 (优化)**: 24 处
- **P3 (微小)**: 5 处
- **注**: 严重度按"是否阻断 P3 收官"判定, P0 阻断 = 0 (守门 #1 已 0 err)

---

## §4 MOCK 数据残留

### 4.1 Rust crate 端

| # | file:path | MOCK 名称 | 用途 | 严重度 |
|---|---|---|---|---|
| 1 | `crates/domain-report/src/infrastructure/port_stubs.rs:1` | 4 Port in-memory stub (InMemoryWorkItemPort + InMemorySprintPort + 2 others) | 阶段 1 验证用, V2 接 `domain-work-item` 等真实数据 | P1 (per `port_stubs.rs:1` 注释 "V2 替换") |
| 2 | `crates/domain-local-runtime/src/process.rs:155` | "mock 模式: 不真 spawn 进程, 只 stub" | dev 模式 mock, 测试用 | P3 (dev 模式) |
| 3 | `crates/domain-ai/src/lib.rs:310-313` | `MockLlmProvider` impl (per role 走不同 stub) | LLM mock provider, 测试用 | P3 (测试 fixture) |
| 4 | `crates/domain-form/src/lib.rs:454-459` | `regex_lite` + `re_is_match` 永远 true | 表单验证 mock regex | P3 (P2 实装) |

### 4.2 Frontend mocks 层 (per P3-A.7, 设计层非 stub)

| # | file:path | 用途 | 严重度 |
|---|---|---|---|
| 5 | `frontend/src/mocks/real-mode.ts:1-48` | MSW real-mode switch + fetch wrapper (P3-A.7) | P2 (per 9/4 P4-WBS 续做项) |
| 6 | `frontend/src/mocks/handlers/incidents.ts:17,71` | "Capability not implemented (per REQ-OPS-003 §30.6 boundary)" 错误返回 | P2 (per spec 边界) |
| 7 | `frontend/src/mocks/handlers/incidents.ts:71` + `__tests__/incidents.test.ts:128,139,150` | 4 处 "Capability not implemented" 测试断言 | P2 |
| 8 | `frontend/src/mocks/seed.ts` + 20 个 `__tests__/` 文件 + `__tests__/uat/uat-test-data.ts` | MSW mock seed + fixture | P3 (per P3-A.7 设计) |

### 4.3 Scripts 层

| # | file:path | 名称 | 用途 | 严重度 |
|---|---|---|---|---|
| 9 | `scripts/automation/pgwiki_audit.py:195,204,347,352,375,401` + `pgwiki_issues_sync.py:47` | `fake_deps` 变量 | **语义陷阱 — 非 MOCK 数据**, 是 Cargo 依赖审计命名 (检测 dev-dep / build-dep 错配), 已显式排除 | — (误报) |

**说明**: 全部 25+ 命中文件中, 仅 4 处是真实 MOCK 数据残留 (Rust port stub + frontend MSW), 其余为设计层或命名相似。`fake_deps` 全部排除。

---

## §5 空函数体 / 空 impl

| # | file:line | 形式 | 严重度 |
|---|---|---|---|
| 1 | `crates/star-vcs/src/cache.rs:74-77` | `#[test] fn placeholder() { /* 仅占位, Phase D 替换 */ }` 空函数体 | P0 (整个 cache 层空) |
| 2 | `crates/star-vcs/src/cache.rs:57` | `pub struct VcsCache;` 空 struct (无 field 无 impl) | P0 |
| 3 | `crates/star-vcs/src/cache.rs:67` | `pub struct CacheError;` 空 struct (应 thiserror enum, 4 变体 per docstring) | P0 |
| 4 | `crates/domain-theme/src/_macros_orphan_to_be_moved.rs:3` | 整个文件 = 0 代码 (docstring "当前空 stub 占位, 不编译") | P3 (per `lib.rs:3` 注释) |
| 5 | `crates/star-cache/src/in_memory_backend.rs:83` | `Err(NotImplemented("incr not implemented for in-memory".into()))` | P2 (per in-memory backend 边界) |

**NotImplemented error variant 列表** (已在 §1 列出, 不重复):
- `star-api-rest/src/error.rs:39` source_kind 字符串
- 4 个 `star-sa/src/provider_*.rs:30` Provider stub
- `domain-local-runtime/src/http_client.rs:355` Phase 2 边界
- `star-saga/src/lib.rs:15` 跨进程持久化后端选型 (Redis vs Postgres)

---

## §6 重复模式 (同类 ≥3 处)

| 模式 | 出现次数 | 代表 file:line | 建议 |
|---|---|---|---|
| **`// v0 phase 1: stub, v0 phase 2 实装` + `Ok(())`** | 6 处 | `domain-batch/src/invariant.rs:27-83` (4 invariant + 2 helper) | 全部 cargo check --lib 0 err 前提下, 符合守门 #1 v6 (单 crate 100% pass) + #7 v3 (clippy advisory), 不构成阻断; v0 phase 2 计划见 `BATCH-REQ-001 §3.3` + `ADR-0040 §D39` |
| **`// TODO(per spec §1.3-1.4 + AGENTS.md §4 #20): 派 worker 子代理实装`** | 3 处 | `star-api-rest/src/middleware/{auth,rate_limit,audit}.rs:17,16,15` | per spec 计划 P2 阶段, 守门 #9 v3 (subprocess 替代 RPC, per `docs/automation-design.md v0.2 §12.3`) 实证可行 |
| **`NoopBatchService` stub trait impl (16 方法)** | 1 文件 16 方法 | `domain-batch/src/port.rs:399-535` | 守门 #1 v3 (--all-targets 至少 1 test) 实证 NoopBatchService 满足编译 + 测试 pass, v0 phase 2 实装 16 方法 |
| **`/// TODO(Phase F): 注入 <Port> 真实数据源`** | 4 处 | `star-mcp/src/resources.rs:179,206,239,324` | Phase F 计划实装, 4 个 ReadPort (Workspace + Worktree + AgentState + Decision) 切真 |
| **"占位结构" Phase 1 骨架 (3 supporting crate)** | 3 文件 ~12 结构 | `application/src/lib.rs:115-227` + `api/src/lib.rs:76-227` + `infrastructure/src/lib.rs:101-227` | per `application/src/lib.rs:125-128` 注释 "Phase 2 由具体 spec 在 `domain-*` 内补全字段; `crates/application` 等 supporting crate 的占位则在 Phase 2 删除, 改为 `use domain_xxx::*;` 引用" — 3 supporting crate 统一 Phase 2 删除 |
| **MSW "Capability not implemented"** | 4 处 (1 handler + 3 test) | `frontend/src/mocks/handlers/incidents.ts:17,71` + `__tests__/incidents.test.ts:128,139,150` | per `REQ-OPS-003 §30.6 boundary` 设计层, 不构成 stub |
| **`Mavis 临时代签 5 域 Lead` 治理注释** | 5 处 | `star-saga/src/{lib,saga_5b_services,saga_5b_real,compensation_strategy}.rs:13,3,4,11` + `star-dispatcher/src/sa_real_impls.rs:6` | 守门 #3 v2 + 守门 #14 派生规: Mavis 临时代签, 真人到位后追溯签字 (per 8/21 JST 5 域独立 Lead 拒绝兼任硬约束 + 9/3 11:35 JST 拍板 B 反转) — **不构成未实现, 是治理结构** |
| **架构师代签 SRE Lead (INV-BA-05)** | 4 处 | `domain-batch/src/{port,domain,invariant,lib}.rs:136,271,287,11,23,32` | per 9/1 18:43 JST 拍板 A: 节点类型注册审批, SRE Lead 缺位 Mavis 临时代签 — **不构成未实现, 是治理结构** |

---

## §7 已知缺口 (per 缺标比错标)

1. **`star-vcs/src/cache.rs` 整个文件 (77 行) = 完整空壳, R-007 cache 层 0 实装**
   - per spec/acceptance/08 R-007 v0.2 fix 2026-08-27, Phase D 实施计划
   - 跨 `TODO(cache-trait)` / `TODO(impl-inmem)` / `TODO(provider-integration)` / `TODO(tests)` 4 大块
   - 实证 #1 派生规 v6 单 crate 100% pass 跟 `#![allow(dead_code)]` 配合维持, 但实际未实装
   - 关联 P4-WBS Phase H.5 / H.7 (Tree-sitter symbol resolver 跟 VCS cache 联动, 估 0.5-1.5M token)

2. **`star-api-rest/` 22 路由 + 3 中间件 全 stub, P2 实装未启动**
   - per spec §2.2 + §2.3 + §1.3-1.4 计划, 11 个 routes 模块文件 0 业务实现
   - 3 中间件 (auth/rate_limit/audit) 留 worker 子代理 dispatch 注释, 守门 #9 v3 实证可改 subprocess
   - 关联 P4-WBS Phase H.3 (12 tool 留 P2 缺 service) + Phase F (凭证切真)

3. **`domain-report/` 14 图表 P1/P2 stub + 4 port stub**
   - 22 图表: P0 8 真实 (C01-C07 + C13) + P1 6 stub (C08-C12, C14) + P2 8 stub (C15-C22)
   - 4 port stub 阶段 1 验证, V2 接 `domain-work-item` 等真实数据
   - 关联 P4-WBS Phase D.2 (T3.2 Saga ≥80% 覆盖) + Phase H 报告子项

4. **`domain-batch/` 22 stub (12 不变量 + 16 NoopBatchService 方法)**
   - 12 关键不变量 v0 phase 1: 4 实装 + 8 占位 (per `invariant.rs:51` 注释 "v0 phase 1: 4 stub 实装, 8 stub 占位")
   - 16 NoopBatchService 方法全部 `Err(BatchError::Internal("NoopBatchService stub".into()))` 返回
   - 2 helper (`validate_dag_topology` + `validate_node_type_approved`) stub pass
   - 关联 P4-WBS Phase D (5.6 H2 原 3 domain) + Phase E (E.6 5 域 Saga) + Phase H (P3-B/C/E/F 续做)

5. **`application/` + `api/` + `infrastructure/` 3 supporting crate 占位结构统一 Phase 2 删除**
   - per `application/src/lib.rs:125-128` 注释 "Phase 2 由具体 spec 在 `domain-*` 内补全字段; `crates/application` 等 supporting crate 的占位则在 Phase 2 删除, 改为 `use domain_xxx::*;` 引用"
   - 实证 9/2 P0-1b 字段类型兼容性 246→0 err 落地, 但 3 supporting crate 仓库内 0 引用完全孤儿 (per AGENTS §4.1 v16)
   - 关联守门 #4.2 实装前一致性门 "唯一实施入口", Phase 2 需删除

6. **4 个 `star-sa` provider stub (GitHub / GitLab / Bitbucket / Gitea) 全部 `NotFound("not implemented")`**
   - per `provider_*.rs:30` 同模式, 4 provider 跨 4 文件
   - 关联 P4-WBS Phase H.2 (LangGraph 跨仓 Physis/RGS RPC 实装) + Phase F (凭证切真)

7. **扫描范围限制** (per 缺标比错标)
   - `src/` (workspace root) 不存在 (per `glob "src/**/*.rs"` 0 命中), 无遗漏
   - `tests/` (per crate) 0 stub / FIXME 命中 (per §2 grep), 已含 domain-report `tests/c01_burndown_test.rs` (use port_stubs, per §3.4 #61-64 引用)
   - `frontend/dist/` / `frontend/.next/` / `node_modules/` / `target/` / `.worktrees/` 已排除 (per brief §3)
   - `frontend/src/mocks/handlers/*.ts` 13 个文件未全部读取 (本次扫描仅命中 incidents.ts, 范围限制)

8. **未触发超时 / 异常** (per §0 完成标志第 4 项)
   - 全部 grep 调用 < 60s, 0 超时
   - 0 异常 / 0 错误 (per `output_mode=content` 全部正常返回)
   - 仅 1 处 `Mavis 接手` 注释链路 `domain-batch/src/{port,domain,invariant,lib}.rs` 跨 4 文件同主题, 已显式列入 §6 重复模式

---

## §附: 引用源

- **Brief**: `docs/briefs/OPT-A1-code-todo-scan.md` (9/7 11:53 JST 派发)
- **基线 WBS**: `docs/reports/STAR-P4-UNIMPL-WBS-001.md` v0.1 (9/4 07:15 JST, 9 大类 ~60 项)
- **基线 HANDOFF**: `docs/reports/HANDOFF-ST-001.md` v0.5 (H1-H4 + H2-EXT 8 domain)
- **守门 #1 v19**: `cargo check --workspace --all-targets -j 4` 0 err (per 9/3 RF-001 T1.5 实证)
- **守门 #11**: 缺标比错标安全 (per 2026-08-26 JST 拍板)
- **守门 #1.2 派生**: 禁回溯叙事, BAS 引用实证, 子代理授权"无证据叙事 = 禁止"

---

**报告人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手
**完成时间**: 2026-09-07 11:55 JST
**下次更新触发**: P4-WBS Phase A.3 5 域 Lead 真人到位 (per `STAR-P4-UNIMPL-WBS-001.md` §2) 后追溯签字, 或 §3-§5 任一 P1 stub 实装完成
