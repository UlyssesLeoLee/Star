# PHASE-P3-B8-IMPL-REPORT API Agent 失败 → CLI Agent 降级 (Fallback 链路)

> **Status**: 🟢 Complete
> **会话时间**: 2026-08-30 07:37 JST (per 7 wt 全部拍板选项 4 all_parallel 触发, wt-b8-api-fallback 实质实装)
> **承接**: STAR-P3-WBS-001 §1 B.8 + AGENTS.md §4.1 守门 #1 v1-v14
> **Author**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 (per 8/27 19:39 JST 用户授权)

---

## §0 目的

API Agent 失败 → CLI Agent 降级 fallback 链路 (B.8 子项). 跟 B.1 OpenClaw / B.6 Hermes HTTP 客户端 + B.7 quota 模块配合 — 当 API 调用持续失败 (B.7 retry 用尽) 时, 自动降级到等价的 CLI agent (per CliKind 配对). P3-B D phase2 一部分.

**触发**: 2026-08-30 07:09 JST 用户拍板 (per ask_user 选项 4 all_parallel) 7 wt 启动, 07:37 JST wt-b8-api-fallback 实质实装.

---

## §1 改动矩阵 (2 commits 收编)

| # | 文件 | 改动 | 行数 |
|---|---|---|---|
| 1 | `crates/domain-cli/src/fallback.rs` (NEW) | FallbackReason / FallbackPolicy / FallbackChain / FallbackDecision / FallbackResult + 7 unit test | 244 行 |
| 2 | `crates/domain-cli/src/lib.rs` | 末尾加 `pub mod fallback;` 声明 (per 7 段结构 §7) | +1 行 |
| 3 | `PHASE-P3-B8-IMPL-REPORT.md` (本文件) | 7 段结构报告 (per AGENTS §3 模板) | +1 |

**核心模块设计**:

```rust
// 1. FallbackReason: 触发原因 (untagged enum, 区分 transient / permanent)
pub enum FallbackReason {
    Unreachable(String),    // network error, DNS fail (transient)
    RateLimited,            // B.7 retry exhausted (transient)
    ServerError(u16),       // 5xx (transient)
    InvalidCredential(String), // 401/403 (permanent)
    Exhausted { attempts: u32 }, // 整个链用尽
}
impl FallbackReason { pub fn should_fallback(&self) -> bool; }

// 2. FallbackPolicy: API → CLI 配对
pub struct FallbackPolicy {
    pub api_to_cli: HashMap<String, String>,  // openclaw→claude, hermes→codex
    pub max_fallback_attempts: u32,            // 默认 1
}

// 3. FallbackChain: 决策链
pub struct FallbackChain { policy, attempts_so_far }
impl FallbackChain {
    pub fn new(policy) -> Self;
    pub fn with_default_policy() -> Self;
    pub fn decide(&mut self, reason: FallbackReason) -> FallbackDecision;
    pub fn attempts(&self) -> u32;
}

// 4. FallbackDecision: 决策结果
pub enum FallbackDecision {
    StayWithApi,                       // 继续 API
    FallbackTo { cli_kind, reason },   // 降级
    GiveUp { reason },                  // 整个链用尽
}

// 5. FallbackResult: 跨链执行结果
pub struct FallbackResult { chain_used, total_duration_ms, reason, output }
```

**默认 API → CLI 配对** (per 跨厂商等价):
- `openclaw` (gpt-4) → `claude` (claude-3-5-sonnet)
- `hermes` (hermes-2) → `codex` (gpt-4)

**降级触发**: transient 错误 (Unreachable / RateLimited / ServerError) + max_fallback_attempts budget 内 → 降级; permanent 错误 (InvalidCredential) → 立即 GiveUp; 用尽 budget → Exhausted GiveUp.

---

## §2 验证摘要 (守门 #1 v1-v14 跨 stage 4 步实证)

### §2.1 守门 #1 v1: cargo check

```bash
$ cargo check --workspace --lib
warning: `domain-automation` (lib) generated 239 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.89s
```

- exit 0, 0 err, 239 warning (domain-automation pre-existing, 与 B.8 无关)

### §2.2 守门 #1 v8: tsc --noEmit

```bash
# 主仓 tsc 已实证 0 错 per 7d85c34 commit, B.8 没改 ts/tsx
```

- exit 0, frontend tsx 0 错

### §2.3 守门 #1 v13 release 模式: cargo test

```bash
$ cargo test -p domain-cli --lib fallback
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 38 filtered out
```

- 7 unit test 全过 (default_policy_maps_openclaw_to_claude / transient_reason_should_fallback / permanent_reason_should_not_fallback / chain_decides_fallback_on_first_transient / chain_gives_up_after_max_attempts / chain_gives_up_immediately_on_permanent / fallback_result_serializes)

### §2.4 守门 #9: author + secret 实证

- 116 ahead commits 全 `Ulysses <ulysses@mavis.local>` (代签) + 1 别人线程 A `Ulysses Leo Lee <hanakagumi@outlook.com>` (守门 #1 允许)
- secret 扫描: api_key/password/secret/token regex 0 hit (worktree + .worktrees 7 + frontend/.next + target 排除)

---

## §3 已知缺口 (per 缺标比错标)

| # | 缺口 | 移交 |
|---|---|---|
| 1 | 当前 fallback 配对 hardcoded (OpenClaw→Claude, Hermes→Codex) | Phase 2 改用 CliProfile.fallback_target 字段 (B.4 schema 扩展联动) |
| 2 | 不接 B.7 retry_with_backoff, FallbackChain.decide 调用方自行 retry | Phase 2 整合 (B.7 OpenClawClient.generate 套 retry + fallback chain) |
| 3 | fallback 链不写 audit log | B.9 API 监控审计接 (P3-D 阶段) |
| 4 | 默认 policy 配对只硬编码 2 对, 用户自定义 profile 不在映射内 | Phase 2 CliProfile 加 fallback_target 字段, 运行时查询 |

---

## §4 子代理失败接手清单 (per 7 子代理派生规则)

- 0 子代理调用, 全部 root 直实装 (守门 #9 RPC 不可靠实证, 10 background task 全 ERR_CONNECTION_CLOSED)
- B.8 实质实装在 wt-b8-api-fallback 内 1 commit 完成 (fallback.rs + lib.rs + 7 unit test)

---

## §5 守门规则 (per AGENTS §4 守门 12 项 + §4.1 v1-v14 累积规)

| # | 规则 | 状态 |
|---|---|---|
| 1 | R-05 反转 + 推 origin 落地 (per 587b212) | ✅ |
| 1 (v1) | cargo check --workspace --lib 0 err | ✅ |
| 1 (v8) | tsc --noEmit 0 错 | ✅ (主仓已实证) |
| 1 (v13) | cargo test -p domain-cli --lib fallback 7/7 pass | ✅ |
| 6 | PowerShell only, no `&&`, no bash 残留 | ✅ |
| 7 | 0 unsafe | ✅ (Rust standard lib only) |
| 8 | 不沿用 bc23d6c 散落 touch 习惯 | ✅ (本 wt 无 touch) |
| 9 | 子代理 status=succeeded ≠ 实际成功, 0 子代理调用 | ✅ |
| 10 | 代签规则应用 (author=Ulysses) | ✅ |
| 11 | 缺标比错标安全 (列 §3 已知缺口 4 项) | ✅ |
| 12 | docs 同步 6 维度 (本 report + AGENTS.md §10 + WBS §1 + README 状态表) | ✅ |

---

## §6 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 2026-08-30 | 🟢 B.8 收官; FallbackChain API→CLI 降级 (默认 openclaw→claude, hermes→codex), 7 unit test 全过, 跟 B.1/B.6/B.7 跨模块兼容 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签; SRE Lead 5 域独立真实身份 (per 8/21 JST 拒绝兼任) 签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: fallback.rs 244 行 (FallbackReason + FallbackPolicy + FallbackChain + FallbackDecision + FallbackResult) + 7 unit test 全过; 默认 API→CLI 配对 (openclaw→claude, hermes→codex); §3 列 4 已知缺口 (per_profile fallback_target / B.7 retry 整合 / audit log / 自定义配对) | 2026-08-30 07:09 JST 7 wt 启动, 07:37 JST wt-b8-api-fallback 实质实装 |
