# Brief: EX-06 — star-mcp 16 tool 幂等改造 (Rust)

> **状态**: 🟡 Brief v0.1
> **拍板**: per `ask_2e8740e6779ac6a8d854c590` 4 推荐项
> **wt-branch**: `wt-ex-06-mcp-idem`
> **base**: `main` (阶段 2 merge 后)
> **关联**: [03-detailed-design.md §1.1 + §2.4.1 M-15](../architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md) · [PHASE §1.1 EX-06](../../reports/PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md) · [ADR-0032 MCP Transport stdio](../architecture/2026-08-26-upgrade/adr/0032-mcp-transport-stdio.md)

---

## 1. 目标

在 `crates/star-mcp/src/middleware/idempotency.rs` 落档 1 middleware + 16 tool 改造, 实现 `Idempotency-Key` header 解析 + 双键 dedup + 写 `idempotency_keys` 表.

## 2. 范围

### 2.1 In-Scope

- `crates/star-mcp/src/middleware/idempotency.rs` — `IdempotencyMiddleware` (Axum tower middleware)
- `crates/star-mcp/src/middleware/mod.rs` — middleware 模块声明
- 16 tool 改造 patch (在 `crates/star-mcp/src/tools/*.rs` 加 `Idempotency-Key` 解析 + dedup 写入):
  - 1. create_merge_request
  - 2. create_worktree
  - 3. search_issues (read-only, 不幂等, 仅加 header 透传)
  - 4. search_code (read-only, 不幂等)
  - 5. get_symbol (read-only, 不幂等)
  - 6. bulk_operation
  - 7. merge_tasks
  - 8. split_task
  - 9. reorder_dependencies
  - 10. summarize (idempotent by nature, 加 header 解析)
  - 11. reassign_agent
  - 12. update_metadata (双键校验)
  - 13. create_issue (新增, per 9/3 P3-C)
  - 14. update_issue
  - 15. close_issue
  - 16. reopen_issue
- 1 集成测试 (per `02 §7.2 IT-06` 16 tool Idempotency-Key 解析)
- 1 中间件 UT (per `02 §7.1`)

### 2.2 Out-of-Scope

- ❌ 5 表 DDL (EX-01 范围)
- ❌ star-mutex crate (EX-02 范围)
- ❌ DispatchLockManager / SubAgentLock (EX-03/04 范围)
- ❌ UI (EX-05 范围)
- ❌ 完整 16 tool 重写, 仅加 Idempotency-Key 解析 patch

## 3. 已知缺口

- 16 tool RACI 责任分配 (per G-EI-07) 5 域 Lead 拍板, 暂统一走 `IdempotencyMiddleware` 全局, 不分域
- 跨 cluster 锁不支持 (per G-EI-02)
- 双键 hash 算法版本兼容 (per G-EI-04)

## 4. 守门

1. 守门 #1 4 守门 (per 守门 #1 v19 + #1 v25): `cargo check --workspace --all-targets -j 4` 0 err + `cargo fmt + clippy` 0 警告 + `cargo test -p star-mcp --lib -j 4` 100% pass + `cargo build --release` 0 err
2. 守门 #7 0 unsafe
3. 守门 #9 v3: 派 worker 子代理 (16 tool patch 量大)
4. 守门 #10 author=Ulysses
5. 守门 #11 缺标比错标
6. 守门 #13 a L0 协调: middleware 在 L0 派发层, L1 tool 不直接调
7. 守门 #19 v19: 守门 #19 仅适用 Python 子项, Rust 例外
8. 守门 #22 控制台不污染: 不动 console_server.py (本子项是 Rust)

## 5. 依赖

### 5.1 上游

- EX-01 (idempotency_keys 表存在)
- EX-02 (star-mutex LockAuditLogger 可选, middleware 直接写 idempotency_keys 表)
- EX-03 (DispatchLockManager 调用接口)

### 5.2 下游

- 16 tool 客户端集成 (gm-console frontend, EX-05)

## 6. 交付物

| # | 路径 | 描述 |
|---|---|---|
| 1 | `crates/star-mcp/src/middleware/mod.rs` | middleware 模块声明 (~0.5KB) |
| 2 | `crates/star-mcp/src/middleware/idempotency.rs` | IdempotencyMiddleware Axum tower (~5KB) |
| 3 | `crates/star-mcp/src/tools/*.rs` (16 tool patch) | 16 tool 各加 Idempotency-Key 解析 (合计 ~10KB patch) |
| 4 | `crates/star-mcp/tests/it_idempotency_middleware.rs` | IT-06 (~3KB) |
| 5 | `crates/star-mcp/src/middleware/tests/idempotency_test.rs` | middleware UT (~2KB) |
| 6 | `docs/briefs/ex-06-mcp-idem.md` | 本 brief |

**总 20 文件, ~21KB raw**

## 7. 验收

- [ ] `cargo check --workspace --all-targets -j 4` exit 0, 0 err
- [ ] `cargo fmt --all -- --check` 0 diff
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 0 err (本子项严格要求)
- [ ] `cargo test -p star-mcp --lib -j 4` 100% pass (16 tool 测试 0 回归)
- [ ] `cargo test -p star-mcp --tests -j 4` 100% pass (1/1 IT)
- [ ] 16 tool 全部加 `Idempotency-Key` header 解析
- [ ] `git log -p --follow crates/star-mcp/src/middleware/idempotency.rs` 实证中间件完整
- [ ] commit author = Ulysses per 守门 #10

## 8. 实施路径

### 8.1 派 worker 子代理

1. 创建 wt (post 阶段 2 merge): `git worktree add ../.worktrees/wt-ex-06-mcp-idem -b wt-ex-06-mcp-idem main`
2. 派 worker 子代理: prompt 含本 brief + 守门 + 16 tool 路径
3. worker 在 wt 内写 20 文件 + 实证守门 #1 4 步
4. Mavis 端实证: `git log -p --follow crates/star-mcp/src/middleware/idempotency.rs` + 16 tool 各 `git log -p --follow`
5. 切回 main + `git merge --no-ff wt-ex-06-mcp-idem`
6. 阶段 3 跨 2 wt cargo check 实证

### 8.2 worker 失败接手

- worker status="succeeded" 但 git log 没看到 commit → Mavis 直接接手
- worker RPC 失败 → Mavis 直接接手

## 9. 风险

- 16 tool patch 量大 → 走 mechanical refactor template (per 守门 #19 v19 派生, Rust 版)
- cargo test 16 tool 0 回归 → 实证现有 16 tool 测试 (per 9/3 P3-C 100% PASS)
- 守门 #1 v25 cargo test 跳过 workspace, 但本子项需 IT 跨 star-mcp → 走 `cargo test -p star-mcp --tests` 单 crate IT

## 10-11. 签字 / 修订历史

5 角色 Mavis 临时代签 + v0.1 初稿.
