# 38.1 Real Git Provider 接入规范（Phase F D6）

> **状态**：🟡 Draft v0.1
> **日期**：2026-08-27
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手代签
> **触发**：per ADR-0035 D6 / 2026-08-27 21:59 JST 用户授权
> **依赖**：[ADR-0023 Version Control Provider](../../adr/0023-version-control-provider.md) · [ADR-0035 Phase F 整体架构](../../adr/0035-phase-f-architecture.md) · [spec/services/01 Service Adapter](../services/01-service-adapter-spec.md) · [spec/services/03 Webhook Adapter](../services/03-webhook-adapter-spec.md) · [spec/mcp/03 Error Model](../mcp/03-error-model-spec.md) · [spec/vcs/01 Version Control Provider Abstraction](01-version-control-provider.md) · [spec/vcs/04 Fallback Strategy](04-fallback-strategy.md)

---

## §1 目的

本规范定义 **Star 接入 4 大真实 Git Provider** 的完整契约，**替换 Phase E 时的 mock 数据**。Provider 抽象层由 `crates/star-sa` 实现（per [spec/services/01 §0](../services/01-service-adapter-spec.md) SA 抽象层为何必要 4 条理由）。

**为什么必须接入真实 provider（per ADR-0035 D6 任务）**：
1. Phase E mock 数据掩盖了 4 大 provider 真实 API 差异（路径 / 头 / 错误码 / 限流策略），导致 `crates/star-cli` / `crates/star-mcp` / `crates/star-rest` 三层在联调时无法 end-to-end 验证
2. ADR-0021 零厂商合作约束要求 STAR 自有"中立翻译层"，但翻译层没有真实数据喂入就无法证明中立 —— 必须先有 4 大 provider 实现，"中立"才有可对照的 ground truth
3. Phase G 起步依赖真实 commit / PR / branch 元数据做 context graph 节点入图（per ADR-0031 Context Graph §2 MVP 4 节点）
4. conformance test（per [spec/acceptance/01](../acceptance/01-unknown-agent-test.md)）需要真实 provider 跑 fallback ladder 4 级，mock 不算"通过"

**与现有 spec 的关系**：
- 上位抽象：[spec/vcs/01](01-version-control-provider.md) §2 核心 trait（11 个 method）
- 协议矩阵：[spec/services/01](../services/01-service-adapter-spec.md) §2 L78-93（8 行操作 × 4 provider + L91 起 4 行中性命名映射）
- Webhook 镜像：[spec/services/03](../services/03-webhook-adapter-spec.md)（入站签名验证 / 幂等）
- 错误模型：[spec/mcp/03](../mcp/03-error-model-spec.md) §2（6 字段：code / message / source_module / source_kind / retriable / hint）

本 spec 是"真实 provider 实现"层（SA 子集），**不**重复描述 SA 抽象（per spec/services/01 §0 SA 与 VCS Provider 关系）。

---

## §2 4 Provider 能力矩阵

per [spec/services/01 §2 L78-93](../services/01-service-adapter-spec.md) 表格，4 Provider = **GitHub Cloud / GitLab SaaS / Bitbucket Cloud / Gitea**（自建 Git 路径含 GitGit + Gitea 两类，但 GitGit 由 [spec/vcs/02](02-gitgit-provider.md) 独立规范，本 spec 只覆盖 Gitea）：

| Provider | list_repos | get_repo | list_branches | get_branch | list_commits | get_commit | create_pr | get_pr |
|----------|------------|----------|---------------|------------|--------------|------------|-----------|--------|
| **GitHub Cloud** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **GitLab SaaS** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Bitbucket Cloud** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Gitea** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ | ✅ |

**8 个 method 编号**（per §6 能力探测，capability 数组 16 项中前 8 项 = 1~8）：

| # | method | 用途 |
|---|--------|------|
| 1 | list_repos | 列出 owner 名下所有 repo |
| 2 | get_repo | 拉单个 repo 元数据（默认分支 / visibility / star 数）|
| 3 | list_branches | 列分支（带 last commit SHA + protected 标记）|
| 4 | get_branch | 拉单个分支 tip + protected status |
| 5 | list_commits | 列 commit 列表（支持 sha 起点 + limit）|
| 6 | get_commit | 拉单个 commit（author / committer / tree / parents / message）|
| 7 | create_pull_request | 发起 PR/MR（参数见 §3 trait）|
| 8 | get_pull_request | 拉单个 PR（state / mergeable / head/base ref）|

**注**：Gitea create_pr 行为差异待 §7 已知缺口 #1 说明（Gitea API 路径 `/api/v1/repos/{owner}/{repo}/pulls` 与 GitHub 路径形式相同，但 head/base ref 必须传 branch 名**不含** `refs/heads/` 前缀，与 GitHub 接收 `refs/heads/feat-x` 形式相反；状态字段 Gitea 仅 `open/closed`，不支持 `merged` 显式态 —— 需在 trait 抽象层做归一化）。

**为什么 Bitbucket 也覆盖**（per ADR-0035 D6 任务范围）：Bitbucket Cloud 与 Bitbucket Server (Data Center) **API 完全不同**（v2.0 vs v1.0），本 spec 仅覆盖 Cloud；Server 由 §7 已知缺口 #2 列为 P3 待办（per Bitbucket Server 团队方案差异 + 市场规模占比 < Cloud 5%）。

---

## §3 Provider trait

per [spec/services/01 §1 L17-75](../services/01-service-adapter-spec.md) SA trait + [ADR-0023](../../adr/0023-version-control-provider.md) VCS Provider Abstraction：

```rust
// crates/star-sa/src/provider/git.rs（计划位置，本 spec 不实装 Rust 代码）
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[async_trait]
pub trait Provider: Send + Sync {
    /// 1. 列出 owner 名下所有 repo
    async fn list_repos(&self, owner: &str) -> Result<Vec<Repo>, ProviderError>;

    /// 2. 拉单个 repo 元数据
    async fn get_repo(&self, owner: &str, name: &str) -> Result<Repo, ProviderError>;

    /// 3. 列出分支
    async fn list_branches(&self, owner: &str, repo: &str) -> Result<Vec<Branch>, ProviderError>;

    /// 4. 拉单个分支 tip
    async fn get_branch(&self, owner: &str, repo: &str, branch: &str) -> Result<Branch, ProviderError>;

    /// 5. 列 commit 列表（sha = 起点 SHA，limit = 最多返回条数）
    async fn list_commits(
        &self,
        owner: &str,
        repo: &str,
        sha: Option<&str>,
        limit: u32,
    ) -> Result<Vec<Commit>, ProviderError>;

    /// 6. 拉单个 commit 详情
    async fn get_commit(&self, owner: &str, repo: &str, sha: &str) -> Result<Commit, ProviderError>;

    /// 7. 创建 PR/MR
    /// - head / base: 完整 ref（如 "refs/heads/feat-x"），由 trait 实现层做 vendor 归一化
    /// - title / body: 必填
    /// - draft: 可选，GitHub/GitLab/Gitea 支持，Bitbucket 不支持（trait 实现层做 vendor 兼容）
    async fn create_pull_request(
        &self,
        owner: &str,
        repo: &str,
        args: CreatePrArgs,
    ) -> Result<PullRequest, ProviderError>;

    /// 8. 拉单个 PR
    async fn get_pull_request(
        &self,
        owner: &str,
        repo: &str,
        number: u32,
    ) -> Result<PullRequest, ProviderError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repo {
    pub id: String,                  // provider 内部 ID（GitHub 数字，Bitbucket UUID）
    pub name: String,
    pub full_name: String,           // "owner/repo"
    pub default_branch: String,
    pub visibility: Visibility,      // Public / Internal / Private（GitLab 三态，其他二态）
    pub clone_url: String,           // https://...
    pub ssh_url: String,             // git@...
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub sha: String,                 // tip commit SHA
    pub protected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub author: CommitAuthor,
    pub committer: CommitAuthor,
    pub message: String,
    pub tree_sha: String,
    pub parents: Vec<String>,        // 父 commit SHA 列表（merge commit 通常 2 个）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAuthor {
    pub name: String,
    pub email: String,
    pub date: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePrArgs {
    pub title: String,
    pub body: String,
    pub head: String,                // 源分支 ref（"refs/heads/feat-x"）
    pub base: String,                // 目标分支 ref（"refs/heads/main"）
    pub draft: bool,                 // Bitbucket 忽略此字段
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub number: u32,
    pub state: PrState,              // Open / Merged / Closed（归一化后）
    pub title: String,
    pub body: String,
    pub head_ref: String,            // 源 ref（归一化为 "refs/heads/<name>"）
    pub base_ref: String,            // 目标 ref（同上）
    pub author: String,              // 用户登录名
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub mergeable: Option<bool>,     // None = 未知 / Some(true|false) = provider 报告
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrState {
    Open,
    Merged,
    Closed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Internal,                        // 仅 GitLab
    Private,
}
```

**关键设计决策**：
- `Provider` trait 是 [spec/vcs/01 §2](01-version-control-provider.md) `VersionControlProvider` trait 的**真实 provider 子集**（11 method → 本 spec 8 method，省略 clone/fetch/push/get_file/get_diff/add_webhook 6 个，clone/fetch/push 走 libgit2，[spec/vcs/01 §5](01-version-control-provider.md) 已规定；get_file/get_diff/add_webhook 归 [spec/services/03 Webhook Adapter](../services/03-webhook-adapter-spec.md)）
- `head_ref` / `base_ref` 归一化为 `refs/heads/<name>` 形式（per [spec/services/01 §2 L96](../services/01-service-adapter-spec.md) 中性命名映射：GitHub `ref` 含前缀，GitLab/Gitea 不含，Bitbucket 不含 —— trait 实现层做归一化）
- `Visibility::Internal` 仅 GitLab 实际返回，其他 3 provider 实现层映射为 `Private`（per §7 已知缺口 #1 类似，行为差异是 trait 设计难点）
- `draft` 字段 Bitbucket 忽略（per §7 已知缺口 #1 模式，所有"某 provider 不支持"字段在 trait 实现层静默吞掉，**不**抛错）
- **不**重复定义错误模型，引用 [spec/mcp/03 §2](../mcp/03-error-model-spec.md) 6 字段 `ProviderError`（per §3 错误模型节）

### §3 错误模型（6 字段）

per [spec/mcp/03 §2](../mcp/03-error-model-spec.md) 6 字段 `ProviderError`：

```rust
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[error("[{source_module}/{source_kind}] {code}: {message} (retriable={retriable}, hint={hint})")]
pub struct ProviderError {
    /// 错误代码（如 "RATE_LIMITED" / "AUTH_FAILED" / "NOT_FOUND" / "VENDOR_DIVERGENCE"）
    pub code: String,

    /// 人类可读消息（**不**包含 token / secret / PII，per §4 认证）
    pub message: String,

    /// 出错模块（如 "star-sa::provider::github"）
    pub source_module: String,

    /// 错误种类（如 "http_429" / "http_401" / "json_parse" / "vendor_divergence"）
    pub source_kind: String,

    /// 是否可重试（限流 429 → true；认证 401 → false；404 → false；vendor divergence → false）
    pub retriable: bool,

    /// 修复建议（如 "检查 GITHUB_TOKEN 是否过期" / "Gitea head ref 不应含 refs/heads/ 前缀"）
    pub hint: String,
}
```

**4 provider 错误码枚举**（per [spec/services/01 §0](../services/01-service-adapter-spec.md) SA 抽象层理由 1 vendor 语义污染防御）：

| code | GitHub HTTP | GitLab HTTP | Bitbucket HTTP | Gitea HTTP | retriable |
|------|-------------|-------------|----------------|------------|-----------|
| `RATE_LIMITED` | 429 / 403 | 429 | 429 | 429（无标准头，per [spec/services/01 §2 L85](../services/01-service-adapter-spec.md)）| true |
| `AUTH_FAILED` | 401 | 401 | 401 | 401 | false |
| `NOT_FOUND` | 404 | 404 | 404 | 404 | false |
| `FORBIDDEN` | 403 | 403 | 403 | 403 | false |
| `SERVER_ERROR` | 5xx | 5xx | 5xx | 5xx | true |
| `VENDOR_DIVERGENCE` | - | - | - | 422（head ref 前缀错）| false |

---

## §4 认证机制

per 2026-08-27 11:06 JST Ulysses hard ban：secret 安全规则，**不打印任何 token**（禁 `Get-ChildItem env:` / `echo $VAR` / `cat .env` / commit message 引用 `$env:XXX` 等所有可能泄露 secret 的操作，per worker memory 2026-08-27 21:51 JST 升级规则）。

| Provider | 认证方式 | secret_ref | 注入位置 |
|----------|----------|------------|----------|
| **GitHub** Cloud | Personal Access Token（PAT, classic 或 fine-grained）| `env:GITHUB_TOKEN` | `Authorization: token <PAT>` 头（classic）/ `Authorization: Bearer <PAT>` 头（fine-grained）|
| **GitLab** SaaS | Personal Access Token | `env:GITLAB_TOKEN` | `PRIVATE-TOKEN: <PAT>` 头（per [spec/services/01 §2 L86](../services/01-service-adapter-spec.md)）|
| **Bitbucket** Cloud | App Password（user + app_pwd）| `env:BITBUCKET_APP_PASSWORD` + `env:BITBUCKET_USERNAME` | `Basic base64(user:app_pwd)` 头（per [spec/services/01 §2 L86](../services/01-service-adapter-spec.md)）|
| **Gitea**（自建）| Personal Access Token | `env:GITEA_TOKEN` | `Authorization: token <PAT>` 头（per [spec/services/01 §2 L86](../services/01-service-adapter-spec.md)）|

**4 铁律**（per 2026-08-27 11:06 JST hard ban + worker memory 21:42/21:51 升级）：
1. **不**保存到配置文件（`config.toml` / `settings.json` 等只存 `secret_ref` 引用字符串 `env:GITHUB_TOKEN`，不存 token 值）
2. **不**打印到 log（`tracing::info!` 模板禁止插入 token 字段，per 21:51 升级）
3. **不**在 commit message / PR description / MR comment 引用 `$env:XXX`（per 21:51 升级：PowerShell `git commit -m "..."` 会展开 env 变量导致明文泄露，**必须**用 `git commit -F file.txt`）
4. **不**在测试 fixture 写真实 token（conformance test 用 mock provider，per [spec/services/01 §0 理由 3](../services/01-service-adapter-spec.md) 测试可替换）

**secret_ref 解析顺序**（per [spec/services/01 §0 理由 2](../services/01-service-adapter-spec.md) 凭据注入由 SA 内部完成）：
1. 读 `env:<NAME>` 字符串（如 `env:GITHUB_TOKEN`）
2. SA 启动时一次性解析（`std::env::var`）
3. 解析失败 → `ProviderError { code: "AUTH_FAILED", ... }`（per §3 错误模型），**不**抛 panic

**OAuth App 流程**（vs PAT）：**未涉及**，待 §7 已知缺口 #3（P2）

---

## §5 Rate Limit 处理

per Provider 头解析（per [spec/services/01 §2 L85](../services/01-service-adapter-spec.md) 限流头行）：

| Provider | Remaining 头 | Reset 头 | Retry-After 头 | 标准 429 响应 |
|----------|--------------|----------|----------------|---------------|
| **GitHub** | `X-RateLimit-Remaining` | `X-RateLimit-Reset`（Unix timestamp）| `Retry-After`（秒）| ✅ |
| **GitLab** | `RateLimit-Remaining` | `RateLimit-Reset`（Unix timestamp）| 无 | ✅ |
| **Bitbucket** | `X-RateLimit-Remaining` | `X-RateLimit-Reset`（Unix timestamp）| `Retry-After`（秒）| ✅ |
| **Gitea** | 无标准头 | 无标准头 | 无 | ✅（按 HTTP 429 通用规则）|

**退避策略**（4 provider 统一）：
```
backoff = min(base * 2^attempt + jitter, max)
- base = 1s
- max = 60s
- jitter = uniform(0, 500ms)  // 0~500ms 随机抖动，防雷鸣
- attempt 起点 = 0
```

**死信队列**（per [spec/services/03 §3](../services/03-webhook-adapter-spec.md) Webhook 死信模式）：
- 连续 **5 次**重试失败 → 写入 `dead_letter` 队列（per spec/services/03 §3 dead_letter 模式）
- 死信消息 schema：`{ provider_id, method, path, last_error, attempt_count, first_attempt_at, last_attempt_at }`
- 死信处理由独立 worker（不阻塞主请求路径），per spec/services/03 §3 异步消费模式

**429 主动防御**：
- 每次请求前检查 `RateLimit-Remaining` 头
- 若 `Remaining < 10% * Limit` → 提前 sleep 至 `Reset` 时刻（不等 429 重试）
- 适用 GitHub / GitLab / Bitbucket（per [spec/services/01 §2 L85](../services/01-service-adapter-spec.md) Gitea 无标准头，**不**适用主动防御）

---

## §6 能力探测

启动时调用 `GET /repos/{owner}/{repo}`（per provider）探测 capability 数组 **16 项**（8 个读 method + 8 个写 method 待 Phase F 后续补，本 spec 只定义读 8 项，写 8 项 P2 待办）：

```rust
pub struct ProviderCapabilities {
    /// 16 项 capability 位掩码
    /// bit 0 (1)  = list_repos        ✅ 4 provider 全支持
    /// bit 1 (2)  = get_repo          ✅ 4 provider 全支持
    /// bit 2 (4)  = list_branches     ✅ 4 provider 全支持
    /// bit 3 (8)  = get_branch        ✅ 4 provider 全支持
    /// bit 4 (16) = list_commits      ✅ 4 provider 全支持
    /// bit 5 (32) = get_commit        ✅ 4 provider 全支持
    /// bit 6 (64) = create_pull_request ⚠️ Gitea 待 §7 #1
    /// bit 7 (128)= get_pull_request  ✅ 4 provider 全支持
    /// bit 8-15   = 写操作（待 Phase F 后续 P2 实装）
    pub bits: u32,
}
```

**探测流程**（per [spec/services/01 §5](../services/01-service-adapter-spec.md) 健康检查）：

```
启动 Provider 实例
  ↓
health() → reachable? auth_valid? latency_p50?
  ↓ 否
capability bits = 0 + 写 error log
  ↓ 是
GET /repos/{owner}/{repo}  // 已知存在的测试 repo（如 "octocat/Hello-World" for GitHub）
  ↓ 200 OK
按 §2 矩阵 + 实际响应填充 bits
  ↓ 401/403/404
bits 置 0（= 不支持任何 method）+ 写 error log
```

**探测失败处理**：
- capability 数组置 0（per spec/services/01 §5 探测失败模式）
- 写 error log（含 `provider_id` + `error_code` + `hint`，**不**含 token，per §4 铁律 2）
- 进程**不** panic —— 启动时探测失败仅降级该 provider，CLI/MCP 仍可工作（只是 `provider="github"` 路由不可用）

**探测频率**：
- 启动时 1 次（per Provider 实例生命周期）
- 切换 provider 时（per [spec/vcs/04 §3 Level 2-3](../vcs/04-fallback-strategy.md) Fallback Ladder）不重复探测，复用启动探测结果
- 写操作（create_pr 等）前**不**重复探测（避免 1000 次 PR 创建 = 1000 次探测，性能浪费）

---

## §7 已知缺口

per **缺标比错标安全**硬规则（DDD Review 必查，per 2026-08-26 JST Ulysses 偏好）：

1. **Gitea create_pr 行为差异**（per §2 注释）待 PM 拍板：
   - head ref 前缀：Gitea 不接受 `refs/heads/feat-x` 形式，必须传 `feat-x`；其他 3 provider 接受两种形式
   - draft 字段：Gitea 1.21+ 支持 `draft: true`，旧版忽略（**不**抛错）
   - state 字段：Gitea 仅 `open/closed`，不支持 `merged` 显式态（需 polling `merged_at` 推断）

2. **Bitbucket Cloud vs Server API 未在 §2 矩阵区分**：
   - Cloud = `https://api.bitbucket.org/2.0/`（本 spec 覆盖）
   - Server (Data Center) = 自建 `https://<host>/rest/api/1.0/`（v1.0 路径，**未**覆盖）
   - Server 团队方案差异大，市场规模 < Cloud 5%，P3 待办

3. **OAuth App 流程（vs PAT）未涉及**（per §4 末）：
   - GitHub GitHub App（vs OAuth App / PAT）
   - GitLab OAuth2
   - Bitbucket OAuth2
   - Gitea OAuth2
   - P2 待办

4. **SSH key 认证未涉及**（仅 token，per §4）：
   - `clone_url` / `ssh_url` 已在 §3 Repo 结构体定义，但 trait method 不用 ssh
   - `git push` / `git fetch` 走 libgit2（per [spec/vcs/01 §5](01-version-control-provider.md)），不归本 spec
   - P3 待办

5. **Webhook 接收 vs Provider push 通知未对齐 [spec/services/03](../services/03-webhook-adapter-spec.md)**：
   - spec/services/03 定义入站签名验证 / 幂等
   - 本 spec 定义出站 API 调用
   - **两者交集** = `add_webhook`（订阅）/ `webhook_events`（解析）— 尚未明确定义
   - P1 待 DDD Review 协同

6. **自建 Git (Gitea) 多租户隔离未明**：
   - 单 Gitea 实例多 org 路由（per [spec/services/01 §2 L78](../services/01-service-adapter-spec.md) `owner` 字段语义）
   - 实例 HA / 跨实例负载均衡未涉及
   - P2 待办

7. **Provider 限流配额实际值未量化**（per §5）：
   - GitHub PAT: 5000 req/h (authenticated)
   - GitLab PAT: 300 req/min (per [spec/services/01 §2 L85](../services/01-service-adapter-spec.md) 限流头解析)
   - Bitbucket Cloud: 1000 req/h (per Atlassian doc)
   - Gitea: 实例自定义（无标准）
   - P3 待 SRE Lead 拍板实际值（per [ADR-0035 §10 token-OLU 35-55M](../../adr/0035-phase-f-architecture.md) 35-55M tokens 范围是否需要 per-provider 限流配额对齐）

8. **Provider-specific commit signing (GPG/SSH) 未涉及**：
   - GitHub GPG verify / GitLab GPG verify
   - Bitbucket SSH signing
   - Gitea GPG verify
   - 仅依赖 [spec/vcs/01 §5](01-version-control-provider.md) libgit2 默认不 verify
   - P2 待 Security Lead 拍板

---

## §8 引用文档

- [ADR-0023 Version Control Provider](../../adr/0023-version-control-provider.md) — VCS Provider Abstraction 上位抽象
- [ADR-0035 Phase F 整体架构](../../adr/0035-phase-f-architecture.md) — 本 spec 触发的 Phase F D6 任务
- [spec/services/01 Service Adapter](../services/01-service-adapter-spec.md) — SA 抽象层（trait 上位 / 协议转换矩阵 / 错误模型）
- [spec/services/03 Webhook Adapter](../services/03-webhook-adapter-spec.md) — Webhook 入站（与本 spec 出站镜像）
- [spec/mcp/03 Error Model](../mcp/03-error-model-spec.md) — 6 字段错误模型
- [spec/vcs/01 Version Control Provider Abstraction](01-version-control-provider.md) — 11 method trait（11 → 本 spec 8 子集）
- [spec/vcs/02 GitGit Provider](02-gitgit-provider.md) — GitGit（自建 Git 第一类，本 spec 不覆盖）
- [spec/vcs/04 Fallback Strategy](04-fallback-strategy.md) — Fallback Ladder 4 级（本 spec 启动探测受其约束）

---

## §9 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|------|------|--------|----------|------|
| v0.1 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：4 Provider 接入规范 + 8 trait method + 6 字段 Error + 4 认证（PAT/App Password）+ Rate Limit 退避（base 1s max 60s jitter 0-500ms）+ 能力探测 16 项 bit 0-7 + 8 已知缺口 | ADR-0035 D6 / 2026-08-27 21:59 JST 用户授权"继续, 你可以代签" |
