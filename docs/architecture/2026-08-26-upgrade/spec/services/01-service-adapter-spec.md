# 38. Service Adapter（SA）Protocol

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/vcs/01 Version Control Provider](../vcs/01-version-control-provider.md) · [ADR-0023 Version Control Provider](../../adr/0023-version-control-provider.md) · [ADR-0025 Vendor Adapter Anti-Contamination](../../adr/0025-vendor-adapter-anti-contamination.md) · [spec/services/03 Webhook Adapter](03-webhook-adapter-spec.md)

## §0 目的（SA 抽象层为何必要）

STAR 上层（CLI / MCP / REST / IDE Gateway）**不**直接调用任何 VCS / CI / Issue Tracker vendor SDK。原因（per ADR-0025 反污染原则 + [arch/03 §3 Fallback Ladder](../../arch/03-star-ai-compat-arch.md)）：

1. **vendor 语义污染**：GitHub `pull_request` ≠ GitLab `merge_request` ≠ Gitea `pull_request`，上层如果写 if-else 分发，逻辑会被 vendor 概念反向侵蚀（per DTL-036 v1.4 hotfix 教训，Ulysses 一审即发现 P1/P2/P3 三项违规）
2. **零厂商合作约束**（per [ADR-0021](../../adr/0021-zero-vendor-cooperation.md)）：不允许把 vendor SDK 当一等公民；STAR 必须拥有"中立翻译层"
3. **测试可替换**：SA trait 让 conformance test 用 mock provider 跑通（per [spec/acceptance/01 Unknown Agent Test](../acceptance/01-unknown-agent-test.md)）
4. **入站 + 出站对称**：Webhook Adapter（inbound, [03 §1](03-webhook-adapter-spec.md)）与 SA（outbound, 本 spec）共享同一 trait 抽象边界，但分两 spec 描述 —— 入站侧重签名验证 / 幂等，出站侧重协议转换 / 限流

**SA 与 VCS Provider 关系**：[spec/vcs/01 §2](../vcs/01-version-control-provider.md) `VersionControlProvider` trait 是 SA 的**子集** —— SA 是更广义的"对外服务适配层"，VCS Provider 仅覆盖 Git 协议（clone / fetch / push / PR/MR）。SA 还覆盖 CI / Issue Tracker / Notification（如 GitHub Actions / GitLab CI / Jira）三类**非 Git** 服务。

## §1 SA 接口定义（trait method 列表）

```rust
// crates/star-sa/src/lib.rs（计划位置，本 spec 不实装 Rust 代码）
#[async_trait]
pub trait ServiceAdapter: Send + Sync {
    /// provider 标识（"github" / "gitlab" / "bitbucket" / "gitea" / "jira" / "self_hosted_git"）
    fn provider_id(&self) -> &'static str;

    /// 能力声明（per [spec/vcs/01 §3](../vcs/01-version-control-provider.md) 能力矩阵扩展）
    fn capabilities(&self) -> ServiceCapabilities;

    /// 健康检查（per §5）
    async fn health(&self) -> Result<HealthReport>;

    /// 出站 REST 调用（核心 method）
    /// - method: HTTP method（GET/POST/PUT/PATCH/DELETE）
    /// - path: provider 路径（如 "/repos/{owner}/{repo}/pulls"）
    /// - body: serde_json::Value（vendor 原始 payload）
    /// - auth: 已注入凭据
    /// 返回 vendor 原始响应（per ADR-0025 不做语义转译，转译在上层 Application Service）
    async fn request(&self, method: HttpMethod, path: &str, body: Option<Value>) -> Result<RawResponse>;

    /// webhook 出站订阅（与 [03 Webhook Adapter §1](03-webhook-adapter-spec.md) 端点镜像）
    async fn subscribe_webhook(&self, repo: &Repository, url: &str, events: &[&str]) -> Result<WebhookId>;

    /// 限流状态查询（per §4 限流策略）
    async fn rate_limit_status(&self) -> Result<RateLimitInfo>;
}

pub struct ServiceCapabilities {
    pub supports_webhook: bool,
    pub supports_graphql: bool,        // GitHub v4 / GitLab
    pub supports_lfs: bool,
    pub supports_self_hosted: bool,
    pub max_payload_size: usize,       // 字节
    pub rate_limit_window: Duration,
}

pub struct HealthReport {
    pub provider_id: String,
    pub reachable: bool,
    pub auth_valid: bool,
    pub latency_p50_ms: u64,
    pub last_checked_at: DateTime<Utc>,
}

pub struct RateLimitInfo {
    pub remaining: u32,
    pub limit: u32,
    pub reset_at: DateTime<Utc>,
    pub scope: RateLimitScope,         // Provider / Repo / User
}
```

**关键设计决策**：
- `request()` 返回 `RawResponse`（vendor 原始 JSON），**不**做语义转译。转译是 Application Service 职责（per [arch/01 §4](../../arch/01-current-architecture-analysis.md) 职责分层）。这与 [spec/vcs/01 §2](../vcs/01-version-control-provider.md) `create_pr` 返回 `PullRequest` 强类型**不同** —— SA 故意暴露"低阶原语"，让上层按需封装。
- 凭据注入由 SA 内部完成（`auth` 参数不暴露给 trait 调用方），符合 ADR-0025 反污染原则

## §2 协议转换矩阵（GitHub / GitLab / Bitbucket / 自建 Git）

| 操作 | GitHub | GitLab | Bitbucket | 自建 Git（GitGit / Gitea）|
|---|---|---|---|---|
| 列出 repos | `GET /user/repos` | `GET /api/v4/projects` | `GET /2.0/repositories/{workspace}` | `GET /api/v1/repos/search`（Gitea）|
| 创建 MR/PR | `POST /repos/{owner}/{repo}/pulls` | `POST /api/v4/projects/{id}/merge_requests` | `POST /2.0/repositories/{workspace}/{repo_slug}/pullrequests` | `POST /api/v1/repos/{owner}/{repo}/pulls`（Gitea）|
| MR/PR 状态字段 | `state: open/closed` | `state: opened/closed/merged` | `state: OPEN/MERGED/DECLINED/SUPERSEDED` | `state: open/closed`（Gitea）|
| Webhook 头 | `X-Hub-Signature-256` (HMAC-SHA256) | `X-Gitlab-Token` (custom token) | `X-Hub-Signature` (HMAC-SHA1, deprecated → SHA256) | `X-Gitea-Signature` (HMAC-SHA256) |
| Webhook 事件头 | `X-GitHub-Event` | `X-Gitlab-Event` | `X-Event-Key` | `X-Gitea-Event` |
| 限流头 | `X-RateLimit-Remaining` + `X-RateLimit-Reset` | `RateLimit-Remaining` + `RateLimit-Reset` | `X-RateLimit-Limit` + `X-RateLimit-Remaining`（无 reset）| Gitea 无标准头（per provider）|
| 认证方式 | `Authorization: token <PAT>` / `Bearer <GH Apps>` | `PRIVATE-TOKEN: <PAT>` / `Bearer <OAuth>` | `Bearer <App Password>` / `Basic <user:app_pwd>` | `token <PAT>` (Gitea) / SSH key |
| GraphQL | ✅ `POST /graphql` | ✅ `POST /api/graphql` | ❌ REST only | ❌ Gitea REST only |

**中性命名映射**（per [arch/03 §8](../../arch/03-star-ai-compat-arch.md) Event 命名空间 + ADR-0025 反污染）：

| STAR 中性术语 | GitHub | GitLab | Bitbucket | Gitea |
|---|---|---|---|---|
| MergeRequest | `pull_request` | `merge_request` | `pullrequest` | `pull_request` |
| Pipeline | `check_run` / `actions/workflow_run` | `pipeline` | `pipeline` | 不支持 |
| Issue | `issue` | `issue` | `issue` | `issue` |
| Branch | `ref`（含 `refs/heads/` 前缀）| `branch`（无前缀）| `branch`（无前缀）| `branch`（无前缀）|

**中性命名约束**（per [arch/03 §8](../../arch/03-star-ai-compat-arch.md) 命名空间后缀）：
- 上层代码**只**用 STAR 中性术语（MergeRequest / Pipeline / Issue）
- SA 内部 trait 仍暴露 vendor 原始字段名（`request()` 返回 `RawResponse`），但**不**在 trait surface area 暴露 vendor 概念
- vendor 概念外泄到上层的代码 = 反污染违规（per ADR-0025），code review 🔴 阻断

## §3 配置 schema（provider / endpoint / auth / rate_limit）

```yaml
# ~/.config/star/sa.yaml（计划位置，本 spec 不实装）
providers:
  - provider_id: "github-primary"
    type: "github"
    endpoint: "https://api.github.com"  # GitHub Enterprise: https://github.acme.com/api/v3
    auth:
      method: "token"                    # token | bearer | basic | ssh_key
      secret_ref: "env:GITHUB_TOKEN"     # 仅引用 env var 名，不存明文（per 8/27 11:06 JST 硬约束）
    rate_limit:
      requests_per_hour: 5000
      burst: 100
      backoff_strategy: "exponential_jitter"  # exponential | linear | none
      max_retries: 3
    webhook:
      inbound_url: "https://star.acme.com/webhook/github"  # per [03 §1](03-webhook-adapter-spec.md)
      events: ["pull_request", "push", "check_run"]
      secret_ref: "env:GITHUB_WEBHOOK_SECRET"

  - provider_id: "gitlab-self-hosted"
    type: "gitlab"
    endpoint: "https://gitlab.acme.com"
    auth:
      method: "private_token"
      secret_ref: "env:GITLAB_TOKEN"
    rate_limit:
      requests_per_hour: 2000
      burst: 50
      backoff_strategy: "exponential_jitter"
      max_retries: 5
```

**字段约束**（per [arch/06 §3 威胁模型](../../arch/06-threat-model-nfr.md) + 8/27 11:06 JST 环境变量安全硬约束）：

- `secret_ref` **只**存 env var 名，**绝不**存明文 token
- 配置加载时由 `star-sa` 解析 `env:VAR_NAME` → 调 `std::env::var()`，**不**打印到日志 / 错误消息 / 审计（per 8/27 11:06 JST hard ban）
- 配置文件**不**进 git（per `.gitignore` 标准模式 `*.sa.local.yaml`）

## §4 重试 + 限流策略

### 4.1 重试矩阵

| 错误类型 | HTTP 状态 | 重试 | 退避 |
|---|---|---|---|
| 限流 | 429 | ✅ | 读 `Retry-After` 头（如有），否则指数退避 + jitter |
| 服务器临时 | 500/502/503/504 | ✅ | 指数退避 + jitter（base 1s, max 60s, jitter ±20%）|
| 认证失败 | 401 | ❌ | 立即 fail（per [agent-api/v1 §3.15 Error](../agent-api/01-schema.md) `AUTH_INVALID`）|
| 权限不足 | 403 | ❌ | 立即 fail（`PERMISSION_DENIED`）|
| 资源不存在 | 404 | ❌ | 立即 fail（`NOT_FOUND`）|
| 客户端错误 | 400/422 | ❌ | 立即 fail（`INVALID_REQUEST`，触发 Universal Submit 12 步 [§6 错误处理](../flows/05-universal-submit.md)）|
| 网络超时 | — | ✅ | 同 5xx |

### 4.2 限流策略

- **客户端预限流**：SA 内部 token bucket（`requests_per_hour` + `burst`），避免触发 vendor 限流
- **vendor 限流响应处理**：当收到 429，读 `Retry-After` 头，等待后再重试；超过 `max_retries` 失败 → 返回 `RATE_LIMITED` 错误（per [agent-api/v1 §3.15](../agent-api/01-schema.md) 6 字段 Error）
- **并发控制**：单 provider 实例 max in-flight = 10（默认），可配置

## §5 健康检查

**目的**：避免在 vendor 不可用时把请求堆到 SA 入口（per [arch/06 §3.2](../../arch/06-threat-model-nfr.md) 降级策略）

**触发时机**：
- SA 初始化时**同步**做一次（阻塞 < 2s 超时）
- 后台每 60s 跑一次（per provider 独立 goroutine / tokio task）
- 失败时进入 `degraded` 状态，3 次连续失败 → `unhealthy`

**探测方式**：
- GitHub: `GET /rate_limit`（无需 auth 也能跑，返回 200 即 healthy）
- GitLab: `GET /api/v4/version`（无需 auth）
- Bitbucket: `GET /2.0/user`（需 auth）
- Gitea: `GET /api/v1/version`

**报告字段**（per §1 `HealthReport`）：
- `reachable`: 网络可达
- `auth_valid`: 凭据有效（不打印 secret 内容，仅 boolean）
- `latency_p50_ms`: 滚动窗口 100 个请求
- `last_checked_at`: ISO 8601

**MCP 暴露**：健康状态通过 [mcp/01 §2.3 `get_pipeline_status`](../mcp/01-mcp-spec.md) 旁的扩展 tool `get_provider_health` 暴露（Phase 2+，MVP 不实现）

## §6 已知缺口（per 缺标比错标安全）

| # | 缺口 | 状态 | 触发 |
|---|---|---|---|
| G-01 | Bitbucket Cloud 与 Server 两套 API（`/2.0/` vs `/rest/api/1.0/`）未在 §2 矩阵区分 | 🟡 待 v0.2 补 | 本 spec v0.1 初版未覆盖 |
| G-02 | Azure DevOps Repos 未列入（5 域 [vcs/01 §1](../vcs/01-version-control-provider.md) 4 个 provider 是 baseline，Azure DevOps 是 Phase 2 候选）| 🟡 待 Phase 2 评估 | ADR-0021 零厂商合作约束 |
| G-03 | `ServiceCapabilities.supports_graphql` 字段已定义但 GitHub GraphQL 限流（per node 复杂度）未建模 | 🟡 待 v0.2 补 | v0.1 初版未展开 |
| G-04 | `request()` 返回 `RawResponse` 但 vendor 错误格式（GitHub `{message, errors}` vs GitLab `{message, error_description}`）未定义统一 schema | 🟡 待 v0.2 补 | vendor 错误格式差异大 |
| G-05 | 凭据轮转（PAT 过期 / OAuth refresh）未在 SA 层自动处理 | 🟡 Phase 2+ 评估 | 安全要求但本 spec 范围外 |
| G-06 | 与 [mcp/01 §2.3 `submit` tool](../mcp/01-mcp-spec.md) 的 Universal Submit 12 步流程（[flows/05 §2](../flows/05-universal-submit.md)）集成点未明 —— 是 SA 直接做，还是 Application Service 编排？ | 🟡 待 v0.2 + W4 子代理定 | 本 spec v0.1 不展开 |
| G-07 | Phase E 当前仅为 spec 草案，**不**进 MVP 退出条件（per [arch/03 §2.3](../../arch/03-star-ai-compat-arch.md) Level 1-2 范围 = tools + submit）| 🟢 显式不实现 | per F-28 修复措辞统一 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 agent（per 19:39 JST 授权升级） | 初版：SA 抽象层目的 + trait 11 method + 4 provider 协议转换矩阵 + 配置 schema + 重试/限流 + 健康检查 + 7 项已知缺口 | Phase E spec 起草（3 份：01-sa / 02-sse / 03-webhook）|
