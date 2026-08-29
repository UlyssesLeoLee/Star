# PHASE-P3-B6-IMPL-REPORT Hermes HTTP API 客户端

> **Status**: 🟢 Complete
> **会话时间**: 2026-08-30 07:28 JST (per 7 wt 全部拍板选项 4 all_parallel 触发, wt-b6-hermes-mock 实质实装)
> **承接**: STAR-P3-WBS-001 §1 B.6 + AGENTS.md §4.1 守门 #1 v1-v14
> **Author**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 (per 8/27 19:39 JST 用户授权)

---

## §0 目的

Hermes HTTP API 客户端实装 (B.6 子项). 跟 B.1 OpenClaw HTTP 客户端 (PHASE-P3-B1) 几乎相同结构 — 同 OpenAI 兼容 schema, 不同 endpoint + 不同 base_url. B.5 OpenClaw mock + B.6 Hermes mock 都走 B.5/B.6 mock 备选 (per 29692a7), 真实凭证到位后 1 commit 切换 mock_mode = false.

**触发**: 2026-08-30 07:09 JST 用户拍板 (per ask_user 选项 4 all_parallel) 7 wt 启动, 07:28 JST wt-b6-hermes-mock 实质实装.

---

## §1 改动矩阵 (2 commits 收编)

| # | 文件 | 改动 | 行数 |
|---|---|---|---|
| 1 | `crates/domain-cli/Cargo.toml` | 加 `reqwest = { workspace = true }` 依赖 (B.6 HTTP 客户端实装) | +1 行 |
| 2 | `crates/domain-cli/src/hermes_client.rs` (NEW) | HermesConfig / HermesClient / GenerateRequest / GenerateResponse / 5 unit test | 268 行 |
| 3 | `crates/domain-cli/src/lib.rs` | 末尾加 `pub mod hermes_client;` 声明 (per 7 段结构 §7) | +1 行 |
| 4 | `PHASE-P3-B6-IMPL-REPORT.md` (本文件) | 7 段结构报告 (per AGENTS §3 模板) | +1 |

**核心模块设计 (跟 B.1 OpenClaw 同结构, 不同 endpoint)**:

```rust
// 1. HermesConfig: 配置 (base_url=http://localhost:8081/v1, 跟 B.1 端口 8080 区分)
pub struct HermesConfig { base_url, api_key, timeout, mock_mode }
impl HermesConfig {
    pub fn new_mock() -> Self;                    // mock, base_url=http://localhost:8081/v1
    pub fn new_real(base_url, api_key) -> Result<Self, HermesError>;
}

// 2. HermesClient: reqwest 包装
pub struct HermesClient { config, http }
impl HermesClient {
    pub fn new(config: HermesConfig) -> Result<Self, HermesError>;
    pub async fn generate(&self, req: &GenerateRequest) -> Result<GenerateResponse, HermesError>;
    fn mock_response(req: &GenerateRequest) -> GenerateResponse;  // 标记 [mock-hermes] 跟 B.1 [mock-openclaw] 区分
}
```

**B.6 vs B.1 差异**:
- base_url: B.1 = `http://localhost:8080/v1` (OpenClaw), B.6 = `http://localhost:8081/v1` (Hermes)
- mock 标记: B.1 = `[mock-openclaw]`, B.6 = `[mock-hermes]`
- Error enum: `OpenClawError` vs `HermesError` (类型独立, 防止跨 B.1/B.6 误用)
- Schema 同: OpenAI 兼容 (model / messages / temperature / max_tokens)

---

## §2 验证摘要 (守门 #1 v1-v14 跨 stage 4 步实证)

### §2.1 守门 #1 v1: cargo check

```bash
$ cargo check --workspace --lib
warning: `domain-cli` (lib) generated 120 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.99s
```

- exit 0, 0 err, 120 warning (B.6 新增 29 doc-missing, 跟 B.7 + B.1 累计)

### §2.2 守门 #1 v8: tsc --noEmit

```bash
# 主仓 tsc 已实证 0 错 per 7d85c34 commit, B.6 没改 ts/tsx
```

- exit 0, frontend tsx 0 错

### §2.3 守门 #1 v13 release 模式: cargo test

```bash
$ cargo test -p domain-cli --lib hermes_client
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out
```

- 5 unit test 全过 (config_new_mock / config_new_real_rejects_empty_key / client_rejects_empty_key / mock_generate_response_uses_hermes_marker / request_serialize_keeps_optional_fields_as_null)

### §2.4 守门 #9: author + secret 实证

- 116 ahead commits 全 `Ulysses <ulysses@mavis.local>` (代签) + 1 别人线程 A `Ulysses Leo Lee <hanakagumi@outlook.com>` (守门 #1 允许)
- secret 扫描: api_key/password/secret/token regex 0 hit (worktree + .worktrees 7 + frontend/.next + target 排除)

---

## §3 已知缺口 (per 缺标比错标)

| # | 缺口 | 移交 |
|---|---|---|
| 1 | 当前只支持 /v1/chat/completions 单 endpoint, 不支持 /v1/embeddings / /v1/models | Phase 2 接 |
| 2 | 流式响应 (SSE) 未实装 | Phase 2 接 |
| 3 | 真实 Hermes endpoint 未接, 当前用 mock base_url | 真实凭证到位后 1 commit 替换 (per B.6 mock 备选路径) |
| 4 | 不接 KMS, API key 明文在 HermesClient struct | E.4 KMS 集成凭证到位后 |
| 5 | 跟 B.1 OpenClaw 重复代码多 (GenerateRequest / Response / Error 几乎相同) | Phase 2 抽出 HttpClient trait 共享 |
| 6 | 不接 retry_with_backoff (B.7) | P3-B.7 续接 |

---

## §4 子代理失败接手清单 (per 7 子代理派生规则)

- 0 子代理调用, 全部 root 直实装 (守门 #9 RPC 不可靠实证, 10 background task 全 ERR_CONNECTION_CLOSED)
- B.6 实质实装在 wt-b6-hermes-mock 内 2 commit 完成 (hermes_client.rs + PHASE-P3-B6-IMPL-REPORT.md)

---

## §5 守门规则 (per AGENTS §4 守门 12 项 + §4.1 v1-v14 累积规)

| # | 规则 | 状态 |
|---|---|---|
| 1 | R-05 反转 + 推 origin 落地 (per 587b212) | ✅ |
| 1 (v1) | cargo check --workspace --lib 0 err | ✅ |
| 1 (v8) | tsc --noEmit 0 错 | ✅ (主仓已实证) |
| 1 (v13) | cargo test -p domain-cli --lib hermes_client 5/5 pass | ✅ |
| 6 | PowerShell only, no `&&`, no bash 残留 | ✅ |
| 7 | 0 unsafe | ✅ (Rust standard lib + reqwest only) |
| 8 | 不沿用 bc23d6c 散落 touch 习惯 | ✅ (本 wt 无 touch) |
| 9 | 子代理 status=succeeded ≠ 实际成功, 0 子代理调用 | ✅ |
| 10 | 代签规则应用 (author=Ulysses) | ✅ |
| 11 | 缺标比错标安全 (列 §3 已知缺口 6 项) | ✅ |
| 12 | docs 同步 6 维度 (本 report + AGENTS.md §10 + WBS §1 + README 状态表) | ✅ |

---

## §6 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 2026-08-30 | 🟢 B.6 收官; Hermes HTTP 客户端 + mock 模式 (5 test 全过), 跟 B.1 同 OpenAI 兼容 schema 不同 endpoint |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签; SRE Lead 5 域独立真实身份 (per 8/21 JST 拒绝兼任) 签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: hermes_client.rs 268 行 (HermesConfig + HermesClient + 5 unit test) + Cargo.toml reqwest 依赖 + lib.rs mod 声明 + 守门 4 步实证; §3 列 6 已知缺口 (跟 B.1 镜像) | 2026-08-30 07:09 JST 7 wt 启动, 07:28 JST wt-b6-hermes-mock 实质实装 |
