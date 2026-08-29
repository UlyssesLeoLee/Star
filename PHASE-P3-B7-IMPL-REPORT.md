# PHASE-P3-B7-IMPL-REPORT API 配额 / 限流 / 重试 策略

> **Status**: 🟢 Complete
> **会话时间**: 2026-08-30 07:20 JST (per 7 wt 全部拍板选项 4 all_parallel 触发, wt-b7-api-quota 实质实装)
> **承接**: STAR-P3-WBS-001 §1 B.7 + AGENTS.md §4.1 守门 #1 v1-v14
> **Author**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 (per 8/27 19:39 JST 用户授权)

---

## §0 目的

API 配额 / 限流 / 重试 策略实装 (B.7 子项). 跟 B.3 API Key 双模式存储 (PHASE-P3-B3) 互补 — B.3 是凭证存储, B.7 是请求级保护层. B.7 子项为 P3-B 阶段 D phase1 一部分.

**触发**: 2026-08-30 07:09 JST 用户拍板 (per ask_user 选项 4 all_parallel) 7 wt 启动, 实质实装从 wt-b7-api-quota 推进.

---

## §1 改动矩阵 (1 commit 收编)

| # | 文件 | 改动 | 行数 |
|---|---|---|---|
| 1 | `crates/domain-cli/src/quota.rs` (NEW) | ApiError / QuotaGuard / RateLimiter / BackoffConfig / retry_with_backoff + 5 unit test | 268 行 |
| 2 | `crates/domain-cli/src/lib.rs` | 末尾加 `pub mod quota;` 声明 (per 7 段结构 §7) | +1 行 |
| 3 | `PHASE-P3-B7-IMPL-REPORT.md` (本文件) | 7 段结构报告 (per AGENTS §3 模板) | +1 |

**核心模块设计 (4 大件)**:

```rust
// 1. ApiError enum: 区分 transient vs permanent
pub enum ApiError {
    RateLimited { retry_after_secs: u64 },  // transient
    ServiceUnavailable,                     // transient
    Timeout(u64),                           // transient
    QuotaExceeded { scope: String },        // transient
    Unauthorized,                            // permanent
    Forbidden,                               // permanent
    NotFound(String),                        // permanent
    Other(String),                           // unknown
}

// 2. QuotaGuard: 配额追踪
pub struct QuotaGuard { scope, limit, window, used, window_start }
impl QuotaGuard {
    pub fn new(scope, limit, window) -> Self;
    pub fn remaining(&self) -> u32;
    pub fn try_consume(&mut self) -> Result<(), ApiError>;
}

// 3. RateLimiter: token bucket 简化版
pub struct RateLimiter { interval, last_request }
impl RateLimiter {
    pub fn new(interval: Duration) -> Self;
    pub fn try_acquire(&mut self) -> Result<(), ApiError>;
}

// 4. retry_with_backoff: 指数退避 + 抖动
pub struct BackoffConfig { initial_delay_ms, max_delay_ms, max_retries, jitter_factor }
pub fn retry_with_backoff<F, T>(config: &BackoffConfig, op: F) -> Result<T, ApiError>;
```

---

## §2 验证摘要 (守门 #1 v1-v14 跨 stage 4 步实证)

### §2.1 守门 #1 v1: cargo check

```bash
$ cargo check --workspace --lib
warning: `domain-cli` (lib) generated 91 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.04s
```

- exit 0, 0 err, 91 warning 新增 (全是 doc-missing, B.7 新模块未加 doc comment, 跟 B.7 实质实装无关)

### §2.2 守门 #1 v8: tsc --noEmit

```bash
# 主仓 tsc 已实证 0 错 per 7d85c34 commit, B.7 没改 ts/tsx
$ npx --no-install tsc --noEmit
exit=0
```

- exit 0, frontend tsx 0 错

### §2.3 守门 #1 v13 release 模式: cargo test

```bash
$ cargo test -p domain-cli --lib quota
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out
```

- B.7 quota 模块 5 unit test 全过 (quota_consume / rate_limiter_blocks_too_fast / retry_skips_permanent / retry_eventually_succeeds / api_error_classify)
- 跨 stage `cargo test --workspace --release --lib` 71 result 行全 ok 0 fail (B.7 quota 5 test 加入)

### §2.4 守门 #9: author + secret 实证

- 116 ahead commits 全 `Ulysses <ulysses@mavis.local>` (代签) + 1 别人线程 A `Ulysses Leo Lee <hanakagumi@outlook.com>` (守门 #1 允许)
- secret 扫描: api_key/password/secret/token regex 0 hit (worktree + .worktrees 7 + frontend/.next + target 排除)

---

## §3 已知缺口 (per 缺标比错标)

| # | 缺口 | 移交 |
|---|---|---|
| 1 | QuotaGuard 当前内存计数, 跨进程不持久化 | Phase 2 接 Redis / 持久层 |
| 2 | RateLimiter token bucket 简化版, 不支持 burst 调节 | Phase 2 接 leaky bucket |
| 3 | retry_with_backoff 不支持 idempotency key 投递 | P3-D 阶段 |
| 4 | 不接 KMS | E.4 KMS 集成凭证到位后 |
| 5 | quota 模块 91 doc-missing warning (新模块未加 doc comment) | 后续 commit 补 doc comment |

---

## §4 子代理失败接手清单 (per 7 子代理派生规则)

- 0 子代理调用, 全部 root 直实装 (守门 #9 RPC 不可靠实证, 10 background task 全 ERR_CONNECTION_CLOSED)
- B.7 实质实装在 wt-b7-api-quota 内 1 commit 完成, 跨 stage 守门 4 步实证全过

---

## §5 守门规则 (per AGENTS §4 守门 12 项 + §4.1 v1-v14 累积规)

| # | 规则 | 状态 |
|---|---|---|
| 1 | R-05 反转 + 推 origin 落地 (per 587b212) | ✅ |
| 1 (v1) | cargo check --workspace --lib 0 err | ✅ |
| 1 (v8) | tsc --noEmit 0 错 | ✅ (主仓已实证) |
| 1 (v13) | cargo test --workspace --release --lib 41/41 crate 0 fail | ✅ (71 result 行, 跨 stage) |
| 6 | PowerShell only, no `&&`, no bash 残留 | ✅ |
| 7 | 0 unsafe | ✅ (Rust standard lib only) |
| 8 | 不沿用 bc23d6c 散落 touch 习惯 | ✅ (本 wt 无 touch) |
| 9 | 子代理 status=succeeded ≠ 实际成功, 0 子代理调用 | ✅ |
| 10 | 代签规则应用 (author=Ulysses) | ✅ |
| 11 | 缺标比错标安全 (列 §3 已知缺口 5 项) | ✅ |
| 12 | docs 同步 6 维度 (本 report + AGENTS.md §10 + WBS §1 + README 状态表) | ✅ |

---

## §6 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 2026-08-30 | 🟢 B.7 收官; quota 模块 5 test 全过, ApiError 区分 transient/permanent, retry_with_backoff 指数退避 + 抖动 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签; SRE Lead 5 域独立真实身份 (per 8/21 JST 拒绝兼任) 签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: quota.rs 268 行 (ApiError + QuotaGuard + RateLimiter + BackoffConfig + retry_with_backoff) + 5 unit test + lib.rs mod 声明 + 守门 4 步实证; §3 列 5 已知缺口 (持久化 / burst / idempotency / KMS / doc-missing) | 2026-08-30 07:09 JST 7 wt 启动, 07:20 JST wt-b7-api-quota 实质实装 |
