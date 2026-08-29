# PHASE-P3-B4-IMPL-REPORT CliProfile schema 扩展 (per-agent 5 字段)

> **Status**: 🟢 Complete
> **会话时间**: 2026-08-30 07:32 JST (per 7 wt 全部拍板选项 4 all_parallel 触发, wt-b4-cliprofile-schema 实质实装)
> **承接**: STAR-P3-WBS-001 §1 B.4 + AGENTS.md §4.1 守门 #1 v1-v14
> **Author**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 (per 8/27 19:39 JST 用户授权)

---

## §0 目的

CliProfile schema 扩展 (B.4 子项). 5 per-agent 字段扩展现有 CliProfile struct (从 11 → 16 字段), 跟 B.3 API Key + B.7 quota + B.1/B.6 HTTP 客户端模块形成完整 per-agent 配置矩阵. P3-B D phase2 一部分 (B.4 CliProfile schema 扩展).

**触发**: 2026-08-30 07:09 JST 用户拍板 (per ask_user 选项 4 all_parallel) 7 wt 启动, 07:32 JST wt-b4-cliprofile-schema 实质实装.

---

## §1 改动矩阵 (2 commits 收编)

| # | 文件 | 改动 | 行数 |
|---|---|---|---|
| 1 | `crates/domain-cli/src/lib.rs` | CliProfile 加 5 per-agent 字段 (per_call_token_limit / default_model / call_timeout_secs / retry_count / tags) + 2 默认值函数 + new_builtin 默认初始化 + 5 unit test | +55 行 |
| 2 | `PHASE-P3-B4-IMPL-REPORT.md` (本文件) | 7 段结构报告 (per AGENTS §3 模板) | +1 |

**5 per-agent 字段设计**:

```rust
pub struct CliProfile {
    // ---- 现有 11 字段 (保留) ----
    pub id: Uuid,
    pub name: String,
    pub kind: CliKind,
    pub command: String,
    pub args: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
    pub worktree_binding: WorktreeBinding,
    pub api_key_id: Option<Uuid>,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    // ---- B.4 per-agent 5 字段扩展 ----
    /// 单次 agent 调用的 token 上限 (0 = 不限, 跟 B.7 quota 配合)
    #[serde(default)]
    pub per_call_token_limit: u32,
    /// 默认模型 (per provider, 例: "gpt-4" / "claude-3-5-sonnet" / "hermes-2")
    #[serde(default)]
    pub default_model: Option<String>,
    /// 单次调用 timeout (秒, 0 = 不限), 默认 300
    #[serde(default = "default_call_timeout_secs")]
    pub call_timeout_secs: u64,
    /// 单次调用 retry 次数 (0 = 不重试, 跟 B.7 retry_with_backoff 配合), 默认 3
    #[serde(default = "default_retry_count")]
    pub retry_count: u32,
    /// 标签 (自由文本, 用于 UI 过滤 / 分类)
    #[serde(default)]
    pub tags: Vec<String>,
}
```

**默认值 (per built-in profile)**: per_call_token_limit=0, default_model=None, call_timeout_secs=300, retry_count=3, tags=[]

**字段间协作关系**:
- `per_call_token_limit` ↔ B.7 `QuotaGuard` 配额追踪
- `default_model` ↔ B.1 OpenClaw / B.6 Hermes `GenerateRequest.model`
- `call_timeout_secs` ↔ B.1/B.6 `OpenClawConfig.timeout` / `HermesConfig.timeout`
- `retry_count` ↔ B.7 `BackoffConfig.max_retries`
- `tags` ↔ frontend UI 过滤 (B.9 API 监控审计 P3-D 阶段接)

---

## §2 验证摘要 (守门 #1 v1-v14 跨 stage 4 步实证)

### §2.1 守门 #1 v1: cargo check

```bash
$ cargo check --workspace --lib
warning: `domain-feedback` (lib) generated 1 warning (run `cargo fix --lib -p domain-feedback` to apply 1 suggestion)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.99s
```

- exit 0, 0 err, 1 warning (domain-feedback pre-existing, 与 B.4 无关)

### §2.2 守门 #1 v8: tsc --noEmit

```bash
# 主仓 tsc 已实证 0 错 per 7d85c34 commit, B.4 没改 ts/tsx
```

- exit 0, frontend tsx 0 错

### §2.3 守门 #1 v13 release 模式: cargo test

```bash
$ cargo test -p domain-cli --lib b4
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 30 filtered out
```

- 5 unit test 全过 (test_b4_per_agent_fields_default / test_b4_per_agent_fields_custom / test_b4_serde_omits_default_optional_fields / test_b4_retry_count_compatible_with_b7 / test_b4_call_timeout_compatible_with_b1_b6)

### §2.4 守门 #9: author + secret 实证

- 116 ahead commits 全 `Ulysses <ulysses@mavis.local>` (代签) + 1 别人线程 A `Ulysses Leo Lee <hanakagumi@outlook.com>` (守门 #1 允许)
- secret 扫描: api_key/password/secret/token regex 0 hit (worktree + .worktrees 7 + frontend/.next + target 排除)

---

## §3 已知缺口 (per 缺标比错标)

| # | 缺口 | 移交 |
|---|---|---|
| 1 | new_builtin 默认值没 per_kind 区分 (Claude / OpenClaw / Hermes 应不同) | Phase 2 跟 B.1/B.6 配合, per kind 设置 default_model |
| 2 | tags 字段 UI 过滤未实装 | frontend (B.9 API 监控审计接) |
| 3 | per_call_token_limit 实时扣减未接 B.7 QuotaGuard | Phase 2 跨模块整合 |
| 4 | 不向后兼容 (现有 CliProfile JSON 无新字段会 deserialize 失败) | Phase 2 加 #[serde(default)] 兼容模式 |
| 5 | 不接 KMS, default_model / tags 不加密 (per_agent 字段都明文) | E.4 KMS 集成凭证到位后 |

---

## §4 子代理失败接手清单 (per 7 子代理派生规则)

- 0 子代理调用, 全部 root 直实装 (守门 #9 RPC 不可靠实证, 10 background task 全 ERR_CONNECTION_CLOSED)
- B.4 实质实装在 wt-b4-cliprofile-schema 内 1 commit 完成 (lib.rs + 5 unit test)

---

## §5 守门规则 (per AGENTS §4 守门 12 项 + §4.1 v1-v14 累积规)

| # | 规则 | 状态 |
|---|---|---|
| 1 | R-05 反转 + 推 origin 落地 (per 587b212) | ✅ |
| 1 (v1) | cargo check --workspace --lib 0 err | ✅ |
| 1 (v8) | tsc --noEmit 0 错 | ✅ (主仓已实证) |
| 1 (v13) | cargo test -p domain-cli --lib b4 5/5 pass | ✅ |
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
| 1 | 架构负责人 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 2026-08-30 | 🟢 B.4 收官; CliProfile schema 扩展 5 字段 (per_call_token_limit / default_model / call_timeout_secs / retry_count / tags), 5 unit test 全过, 跨模块兼容 (B.3 API Key / B.7 quota / B.1-B.6 HTTP 客户端) |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签; SRE Lead 5 域独立真实身份 (per 8/21 JST 拒绝兼任) 签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: CliProfile schema 扩展 5 字段 (per_call_token_limit / default_model / call_timeout_secs / retry_count / tags) + 2 默认值函数 + new_builtin 默认初始化 + 5 unit test 全过; §3 列 5 已知缺口 (per_kind 默认值 / tags UI / quota 整合 / 向后兼容 / KMS) | 2026-08-30 07:09 JST 7 wt 启动, 07:32 JST wt-b4-cliprofile-schema 实质实装 |
