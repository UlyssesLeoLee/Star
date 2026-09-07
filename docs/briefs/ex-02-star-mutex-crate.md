# Brief: EX-02 — star-mutex 共享 crate (Rust)

> **状态**: 🟡 Brief v0.1
> **拍板**: per `ask_2e8740e6779ac6a8d854c590` 4 推荐项
> **wt-branch**: `wt-ex-02-star-mutex`
> **base**: `main` (阶段 1 merge 后)
> **触发**: 2026-09-07 21:43 JST 用户发令
> **关联**: [03-detailed-design.md §1.1 + §2.3 + §2.3.1 + §2.3.2](../architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md) · [PHASE §1.1 EX-02](../../reports/PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md)

---

## 1. 目标

新建 `crates/star-mutex/` Rust crate, 提供 22 domain-* crate 共享的 `DomainMutex` trait + `StarMutexAdapter` proc-macro + 7 module (M-08..M-14).

## 2. 范围

### 2.1 In-Scope

- `Cargo.toml` workspace member 声明
- 7 源文件 (M-08..M-14):
  - `lib.rs` — `DomainMutex` trait + `MutexGuard` + `DomainMutexError`
  - `adapter.rs` — `StarMutexAdapter` proc-macro
  - `audit.rs` — `LockAuditLogger`
  - `policy_loader.rs` — `ExclusionPolicyLoader`
  - `trace.rs` — `TraceIdPropagator`
  - `version_cas.rs` — `VersionCAS` (乐观锁)
  - `advisory_lock.rs` — `PgAdvisoryLock` (pg_try_advisory_xact_lock 包装)
- 1 macro (`#[domain_mutex(resource_type = "...")]`)
- 1 集成测试 (per `02 §7.2 IT-01` + `IT-04` + `IT-05`)
- 6 UT (per `02 §7.1 UT-05..UT-10` + `UT-11` + `UT-12`)
- 1 demo (在 `domain-work-item` 通过 macro 接入)

### 2.2 Out-of-Scope

- ❌ 实际改 22 domain crate 接入 (后续 H2 阶段)
- ❌ 5 张新表 DDL (EX-01 范围)
- ❌ DispatchLockManager / SubAgentLock (EX-03/04 范围)
- ❌ UI / MCP (EX-05/06 范围)
- ❌ 真实 PG 集成测试 (需 PG 实例, 后续跨 session 续)

## 3. 已知缺口

- 无真实 PG 实例, IT 用 mock pool (per 守门 #13 实证 5/5 subagent RPC 不可靠, 走 Mavis 直接 mock)
- proc-macro 用 `syn` + `quote` 完整实现, 简化版 demo 接入 1 个 domain 验证
- 守门 #1 v19/v25 实证: cargo check + cargo test -p star-mutex --lib -j 4

## 4. 守门

1. **守门 #1 4 守门**: `cargo check --workspace --all-targets -j 4` 0 err + `cargo fmt + clippy` 0 警告 + `cargo test -p star-mutex --lib -j 4` 100% pass + `cargo build --release` 0 err
2. **守门 #7 0 unsafe**: 全 crate 不允许 `unsafe` 块
3. **守门 #10 author=Ulysses**: 1-3 commit, author=`Ulysses <ulysses@mavis.local>`
4. **守门 #13 a L0 协调**: trait 设计上 L2 SubAgent 不能直接调, 必须经 L1 派发 (注释明示)
5. **守门 #19 v19**: 不需要 Python 化 (纯 Rust)
6. **守门 #22 控制台不污染**: 不动 console_server.py

## 5. 依赖

### 5.1 上游

- EX-01 (5 张新表 DDL 已 merge, IT-04 advisory_lock_audit 表存在)

### 5.2 下游

- EX-06 (star-mcp 16 tool 幂等改造) — 依赖 `LockAuditLogger` 写 audit
- EX-07 (可观测性) — 依赖 `TraceIdPropagator` 4 层 trace_id 传递
- 22 domain crate 接入 (后续 H2 阶段)

## 6. 交付物

| # | 路径 | 描述 |
|---|---|---|
| 1 | `crates/star-mutex/Cargo.toml` | workspace member 声明 + 依赖 (tokio / sqlx / async-trait / thiserror / sha2 / chrono / uuid / tracing) |
| 2 | `crates/star-mutex/src/lib.rs` | DomainMutex trait + MutexGuard + DomainMutexError (~3KB) |
| 3 | `crates/star-mutex/src/adapter.rs` | StarMutexAdapter proc-macro (~2KB) |
| 4 | `crates/star-mutex/src/audit.rs` | LockAuditLogger (~2KB) |
| 5 | `crates/star-mutex/src/policy_loader.rs` | ExclusionPolicyLoader (~1.5KB) |
| 6 | `crates/star-mutex/src/trace.rs` | TraceIdPropagator (~1.5KB) |
| 7 | `crates/star-mutex/src/version_cas.rs` | VersionCAS (~2KB) |
| 8 | `crates/star-mutex/src/advisory_lock.rs` | PgAdvisoryLock (~2KB) |
| 9 | `crates/star-mutex/tests/it_advisory_lock.rs` | IT-01 + IT-04 + IT-05 (mock pool) (~3KB) |
| 10 | `crates/star-mutex/tests/lib_test.rs` | UT-05..UT-10 + UT-11 + UT-12 (~4KB) |
| 11 | `docs/briefs/ex-02-star-mutex-crate.md` | 本 brief |

**总 11 文件, ~24KB raw**

## 7. 验收

### 7.1 守门实证 (per 守门 #1 4 守门 + #1 v19/v25)

- [ ] `cargo check --workspace --all-targets -j 4` exit 0, 0 err
- [ ] `cargo fmt --all -- --check` 0 diff
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 0 err (走守门 #1 v25 实证 0.57 警告可接受, 但本子项要求 0)
- [ ] `cargo test -p star-mutex --lib -j 4` 100% pass (6/6 UT, ~5s)
- [ ] `cargo test -p star-mutex --tests -j 4` 100% pass (3/3 IT, mock pool)
- [ ] `cargo build --release -p star-mutex` exit 0

### 7.2 git 实证

- [ ] `git log -p --follow crates/star-mutex/src/lib.rs` 实证 trait 完整
- [ ] `git log -p --follow crates/star-mutex/Cargo.toml` 实证依赖完整
- [ ] commit author = Ulysses per 守门 #10
- [ ] 1-3 commit 含全部 11 文件, 不散落

## 8. 实施路径

### 8.1 派 worker 子代理 (per 守门 #9 v3)

1. 创建 wt: `git worktree add ../.worktrees/wt-ex-02-star-mutex -b wt-ex-02-star-mutex main` (post EX-01 merge)
2. 派 worker 子代理: prompt 含本 brief 全文 + 守门 + 路径
3. worker 在 wt 内写 11 文件 + 实证守门 #1 4 步
4. Mavis 端实证: `git log -p --follow crates/star-mutex/src/lib.rs` 实证 trait 完整
5. 切回 main + `git merge --no-ff wt-ex-02-star-mutex -m "merge: EX-02 star-mutex 共享 crate"`
6. 阶段 2 (本子项) cargo check 0 err 实证

### 8.2 worker 失败接手 (per 守门 #9 v3 fallback)

- 如果 worker status="succeeded" 但 git log 没看到 commit → Mavis 直接接手在 wt 内写代码
- 如果 worker RPC 失败 → Mavis 直接接手

## 9. 风险

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| proc-macro 复杂度超 1M token | 中 | 中 | 简化版 macro, 仅 1 个 demo 接入 |
| cargo check 22 domain crate 接入 macro 报错 | 中 | 高 | demo 仅 1 domain (`domain-work-item`) 验证, 不全量接入 |
| 守门 #1 v25 cargo test 跳过 workspace, 但本子项需 IT 跨模块 | 低 | 低 | 走 `cargo test -p star-mutex --tests` 单 crate IT |

## 10. 签字

5 角色 Mavis 临时代签 (per 守门 #14 v2 拍板 D), author=Ulysses.

## 11. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-07 21:43 JST | Ulysses — Mavis 接手 | 初稿 | per `ask_2e8740e6779ac6a8d854c590` |
