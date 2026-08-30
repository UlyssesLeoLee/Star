# 40. Webhook Adapter

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/services/01 Service Adapter](01-service-adapter-spec.md) · [spec/services/02 SSE Server Push](02-sse-streaming-spec.md) · [spec/flows/08 Event Model](../flows/08-event-model.md) · [arch/03 §8 Event 命名空间](../../arch/03-star-ai-compat-arch.md) · [arch/06 §3 威胁模型](../../arch/06-threat-model-nfr.md) · [ADR-0021 Zero Vendor Cooperation](../../adr/0021-zero-vendor-cooperation.md)

## §0 目的（SA 层的入站接收）

[01 Service Adapter](01-service-adapter-spec.md) 描述了 STAR **出站**调用 vendor 的抽象（SA → vendor REST）。本 spec 描述**入站**接收 —— vendor 主动 POST 到 STAR 的反向路径（vendor → STAR Webhook）。两者共同构成 SA 层的"双向翻译"边界。

**入站 vs 出站边界**：

| 维度 | SA 出站（[01](01-service-adapter-spec.md)）| Webhook Adapter 入站（本 spec）|
|---|---|---|
| 方向 | STAR → vendor | vendor → STAR |
| 触发 | STAR 业务逻辑 | vendor 事件 |
| 鉴权 | STAR 持 vendor 凭据 | vendor 验证 STAR 端点（HMAC 签名）|
| 频率 | 低（按需）| 中-高（每个 push / PR / pipeline）|
| 失败处理 | [01 §4 重试](01-service-adapter-spec.md) 矩阵 | 本 spec §5 失败重试 + 死信 |

**为何独立 spec**（不并入 [01](01-service-adapter-spec.md)）：
1. 入站 / 出站 trait 形态完全不同（出站是 `request()`，入站是 `verify_signature()` + `parse()` + `dispatch()`）
2. 入站侧重**安全**（签名验证 / 重放攻击 / 幂等去重），出站侧重**协议转换**（per [01 §2 4 provider 矩阵](01-service-adapter-spec.md)）
3. 入站路径走 **HTTP server**（STAR 暴露端点），出站走 **HTTP client**（STAR 调用 vendor），CRUD 角色相反

## §1 Webhook 端点

### 1.1 端点模式：`POST /webhook/{provider}`

```
POST /webhook/github       ← GitHub / Gitea
POST /webhook/gitlab       ← GitLab
POST /webhook/bitbucket    ← Bitbucket Cloud
POST /webhook/jira         ← Jira（per §1.3 扩展）
```

- **路径参数** `{provider}`：vendor 标识（per [01 §1 `provider_id`](01-service-adapter-spec.md)）
- **方法**：`POST`（vendor 主动推送），其他 method → `405 Method Not Allowed`
- **Content-Type**：
  - GitHub / Gitea / Bitbucket: `application/json`
  - GitLab: `application/json`（per GitLab webhook spec）
  - Jira: `application/json`（per Jira webhook 2.0）
- **响应时序**（per vendor 最佳实践）：
  - STAR 收到请求 → 立即返回 `202 Accepted`（**不**做完整业务处理）
  - 业务处理走**异步任务队列**（`crates/star-event/` 落地 + 触发 [flows/08 13 个 STAR Domain Event](../flows/08-event-model.md)）
  - vendor 期望 < 5s 响应，超时会被 vendor 标记"投递失败" → 触发 vendor 重发 → 走本 spec §3 幂等去重

### 1.2 端点注册流程

```
[STAR admin]                                              [Vendor (GitHub)]
   | -- star webhook add github --repo owner/repo \        |
   |      --url https://star.acme.com/webhook/github \    |
   |      --events "pull_request,push,check_run" \        |
   |      --secret <env:GITHUB_WEBHOOK_SECRET>            |
   | --→ SA 出站调用 vendor API 创建 webhook →       | -- POST /repos/{owner}/{repo}/hooks -->
   | <-- WebhookId (e.g. "hook_abc123") ---------    | <-- 201 Created, id: hook_abc123 --
   | --→ 持久化 WebhookId + secret_ref 到 DB →      |
   | <-- 完成 -------------------------------        |
```

### 1.3 扩展 provider

MVP 范围 = GitHub / GitLab / Bitbucket / Gitea（per [01 §2 4 provider 矩阵](01-service-adapter-spec.md)）。Jira / Azure DevOps / Slack 等**非 Git** provider 是 Phase 2+ 评估，**不**进 MVP 退出条件（per [arch/03 §2.3 Level 1-2 MVP 范围](../../arch/03-star-ai-compat-arch.md)）。

## §2 签名验证

### 2.1 通用原则

- **不**信任 vendor 请求 body，**必须**先验签再做业务处理
- 签名验证失败 → 立即 `401 Unauthorized`，**不**入审计（避免日志污染，per [arch/06 §3.2](../../arch/06-threat-model-nfr.md) 噪声攻击）
- 验签中间件**不**打印任何 secret 内容（per 8/27 11:06 JST hard ban）

### 2.2 Per-provider 签名方案

| Provider | Header | 算法 | 计算方式 | secret 来源 |
|---|---|---|---|---|
| **GitHub** | `X-Hub-Signature-256` | HMAC-SHA256 | `hmac_sha256(secret, body)` → hex | 配置 `secret_ref: env:GITHUB_WEBHOOK_SECRET` |
| **Gitea** | `X-Gitea-Signature` | HMAC-SHA256 | 同 GitHub | 同 |
| **Bitbucket Cloud** | `X-Hub-Signature` | HMAC-SHA1（**已 deprecated**）→ 计划 HMAC-SHA256 | 同 GitHub | 同 |
| **Bitbucket Server** | `X-Hub-Signature` | HMAC-SHA256 | 同 GitHub | 同 |
| **GitLab** | `X-Gitlab-Token` | 字符串相等 | `== configured_token` | `secret_ref: env:GITLAB_WEBHOOK_TOKEN` |
| **Jira**（Phase 2+）| `X-Atlassian-Webhook-Identifier` | 字符串相等 | `== configured_token` | 同 |

### 2.3 GitHub 签名验证示例（**伪代码**，本 spec 不实装 Rust）

```
// 伪代码（per §2.2 GitHub HMAC-SHA256）
expected_sig = "sha256=" + hmac_sha256_hex(secret, raw_body)
received_sig = headers["X-Hub-Signature-256"]
if !constant_time_eq(expected_sig, received_sig):
    return 401 Unauthorized
// 验签通过 → 解析 body → 走 §3 幂等去重
```

**安全约束**（per [arch/06 §3.1](../../arch/06-threat-model-nfr.md)）：
- **必须**用 `constant_time_eq` 比较（防 timing attack），**不**用 `==`
- secret 长度 ≥ 32 字节（GitHub 文档最低要求），配置加载时校验
- secret **不**进 git，**不**进日志，**不**进审计（per 8/27 11:06 JST hard ban）

### 2.4 签名头验证的元数据

除 body 签名外，**还**校验：

- `User-Agent`：必须是 vendor 官方 UA（如 `GitHub-Hookshot/abc123`），防止任意 client 假冒
- `Content-Type`：必须是 `application/json`（GitLab / Jira 支持 form，但 MVP 仅接 JSON）
- 源 IP（per [arch/06 §3.3](../../arch/06-threat-model-nfr.md)）：白名单 vendor 出口 IP 段（GitHub / GitLab 均发布官方 IP 列表，定期拉取更新）

## §3 事件去重

**目的**：vendor 重发机制（GitHub 默认 24h 重试 3 次）下保证 STAR 端**幂等**。

### 3.1 幂等键 = `(provider, delivery_id)`

| Provider | 幂等键字段 | Header / Payload 位置 |
|---|---|---|
| **GitHub** | `X-GitHub-Delivery` (UUID) | Header |
| **Gitea** | `X-Gitea-Delivery` (UUID) | Header |
| **Bitbucket Cloud** | `X-Request-UUID` | Header |
| **GitLab** | 缺 delivery_id → **用 `object_kind + id + updated_at` 组合** | Payload 内 |
| **Jira**（Phase 2+）| `X-Atlassian-Webhook-Identifier` | Header |

### 3.2 去重实现

```
// 伪代码
async fn handle_webhook(provider, delivery_id, body):
    if redis.set(f"webhook:dedup:{provider}:{delivery_id}", "1", nx=True, ex=86400) == 0:
        log("duplicate delivery ignored", provider, delivery_id)
        return 200 OK  // 静默忽略，不报错
    // 首次投递 → 走 §4 业务处理
    dispatch_event(provider, body)
    return 202 Accepted
```

**存储**：Redis（推荐）/ PostgreSQL `webhook_deliveries` 表（fallback），TTL = 24h（覆盖 GitHub 重试窗口）

**GitLab 特殊情况**：GitLab 无 `delivery_id`，需用 `(object_kind, object_attributes.id, object_attributes.updated_at)` 三元组作为幂等键（per GitLab webhook 文档）。`updated_at` 保证 vendor 编辑后重发不会丢。

## §4 事件类型 → 内部事件映射

**核心约束**（per [arch/03 §8](../../arch/03-star-ai-compat-arch.md) B-17 修复 + [flows/08 §1.1](../flows/08-event-model.md)）：
- vendor 事件 → 内部事件转译在 **Application Service** 内完成
- 内部事件名严格用 [flows/08 §1.1 13 个 STAR Domain Event](../flows/08-event-model.md) **加 `.star` 后缀**
- **不**直接转发 vendor 原始事件（per [arch/05 §3 bridge 不透明原则](../../arch/05-gitgit-compat-arch.md)）

### 4.1 完整映射表

| GitHub `X-GitHub-Event` | GitLab `X-Gitlab-Event` | Bitbucket `X-Event-Key` | Gitea | STAR 内部事件（`.star` 后缀）| payload 关键字段 |
|---|---|---|---|---|---|
| `pull_request` (`action=opened`) | `Merge Request Hook` | `pullrequest:created` | `pull_request` (action=opened) | `MergeRequestCreated.star` | `mr_id, title, author, base, head, url` |
| `pull_request` (`action=closed, merged=true`) | `Merge Request Hook` (state=merged) | `pullrequest:fulfilled` | `pull_request` (action=closed, merged=true) | `MergeRequestMerged.star` | `mr_id, merged_by, merged_at, sha` |
| `pull_request_review` | (无原生, 走 MR Hook) | `pullrequest:approved` | `pull_request_review` | `HumanReviewRequested.star` | `mr_id, reviewer, review_state` |
| `push` | `Push Hook` | `repo:push` | `push` | `CodeModified.star` | `repo, ref, commits[], pusher` |
| `check_run` (`status=completed`) | `Pipeline Hook` (object_attributes.status=success/failed) | `pipeline:completed` | (Gitea 无 pipeline) | `PipelineStatusChanged.star` | `pipeline_run_id, status, conclusion, duration_ms` |
| `issues` (`action=opened`) | `Issue Hook` | `issue:created` | `issues` | (不转译, 落 Issue Tracker) | — |
| `release` | `Release Hook` | `repo:release` | `release` | (不转译, Phase 2+) | — |
| `*` (其他) | `*` (其他) | `*` (其他) | `*` (其他) | **不转译**，仅落 `webhook_audit` 表 | `raw_event, source` |

### 4.2 映射实施位置

- **`crates/star-webhook/src/mapping.rs`**（计划位置）：每个 provider 一个 `pub fn map_<provider>_event(...) -> Option<DomainEvent>`
- **`crates/star-webhook/src/dispatcher.rs`**（计划位置）：调 mapping → emit 到 [flows/08 §1.1 13 个 STAR Domain Event](../flows/08-event-model.md) → SSE 推（per [02 SSE §0](02-sse-streaming-spec.md)）
- **`crates/star-webhook/src/handlers/`**（计划位置）：每个 provider 一个 `mod.rs`，含签名验证（per §2）+ 去重（per §3）

## §5 失败重试 + 死信

### 5.1 重试策略

- **vendor 端重发**（GitHub 默认 24h / 3 次）：**不**主动重试，依赖 §3 幂等去重
- **STAR 端业务处理失败**（如映射 panic / DB 写失败）：
  - 短暂错误（DB 瞬断 / 网络超时）：in-memory retry 3 次，exponential backoff 1s/2s/4s
  - 持续错误：写入 `webhook_dead_letter` 表（per [flows/07 §6 AuditEntry](../flows/07-audit-model.md)），告警
  - 死信条目 ≥ 100 → 触发 P1 告警（[arch/06 §3 NFR](../../arch/06-threat-model-nfr.md)）

### 5.2 死信表 schema（计划）

```sql
-- 伪 schema
CREATE TABLE webhook_dead_letter (
    id              BIGSERIAL PRIMARY KEY,
    provider        VARCHAR(32) NOT NULL,    -- 'github' / 'gitlab' / ...
    delivery_id     VARCHAR(128) NOT NULL,
    event_type      VARCHAR(64) NOT NULL,    -- vendor 原始 event type
    raw_body        JSONB NOT NULL,
    error_message   TEXT NOT NULL,
    failed_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    retry_count     INT NOT NULL DEFAULT 0,
    resolved        BOOLEAN NOT NULL DEFAULT FALSE,
    resolved_at     TIMESTAMPTZ,
    resolved_by     VARCHAR(64)              -- 'auto' / operator_id
);
CREATE INDEX idx_dl_provider_unresolved ON webhook_dead_letter (provider) WHERE resolved = FALSE;
```

### 5.3 告警 + 人工介入

- 死信表新增条目 → 触发 `STAR-WEBHOOK-DLQ` 告警 → 通知 SRE Lead（per AGENTS.md §4 #3 5 域独立 Lead）
- 人工介入：调 `star webhook dlq replay --id <dlq_id>` 重放单条，**不**自动批量重放（防雪崩）

## §6 已知缺口

| # | 缺口 | 状态 | 触发 |
|---|---|---|---|
| G-01 | vendor IP 白名单**定期拉取**机制未明（GitHub / GitLab 公开 IP meta API 拉取频率 / 缓存策略 / 拉取失败的兜底）| 🟡 待 v0.2 评估 | [arch/06 §3.3](../../arch/06-threat-model-nfr.md) 要求 |
| G-02 | Bitbucket Cloud HMAC-SHA1 兼容性保留 vs 强制 SHA256 切换**未决** | 🟡 待 v0.2 评估 | Bitbucket 仍允许 SHA1 但已 deprecated |
| G-03 | GitLab 无 `delivery_id` 用 `updated_at` 做幂等键，**`updated_at` 格式**（字符串 vs 时间戳）需在 v0.2 钉死 | 🟡 待 v0.2 | GitLab payload 不规范 |
| G-04 | Webhook 端点 **TLS 终止**位置未明（STAR 自己终止 vs 前置 LB / Ingress）| 🟡 待部署架构定 | 本 spec v0.1 仅写"暴露端点"未写"如何暴露" |
| G-05 | 死信表 `resolved` 字段触发**审计事件**（per [flows/07 §6 AuditEntry](../flows/07-audit-model.md)）未明（是 operator 操作 = `ActorType.Human` 还是 `ActorType.Automation`）| 🟡 待 v0.2 | 审计 actor 类型需与 flows/07 对齐 |
| G-06 | 与 [01 Service Adapter §1 `subscribe_webhook()`](01-service-adapter-spec.md) 出站订阅方法是**同一 trait 方法**还是**两个方法**（一个 outbound + 一个 inbound）| 🟡 待 v0.2 | 本 spec 与 01 同步设计，未明确分工 |
| G-07 | `webhook_audit` 表（§4.1 "不转译" 兜底）落所有 vendor 事件，**数据保留期**未明 | 🟡 待 v0.2 | GDPR / 数据保留策略 |
| G-08 | 限流（防 vendor 突发风暴）未在本 spec 单独设计 —— 依赖 §3 幂等 + §5 重试，**入口限流**（如 IP 维度 100 req/s）未加 | 🟡 待 v0.2 + [arch/06 §3.2](../../arch/06-threat-model-nfr.md) 评估 | 突发 vendor 故障场景 |
| G-09 | Phase E 当前仅为 spec 草案，**不**进 MVP 退出条件（per [arch/03 §2.3](../../arch/03-star-ai-compat-arch.md) Level 1-2 范围 = tools + submit）| 🟢 显式不实现 | per F-28 修复措辞统一 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 agent（per 19:39 JST 授权升级）| 初版：入站/出站边界 + 端点模式 + 6 provider 签名方案 + 幂等去重 + 8 类事件映射表（GitHub/GitLab/Bitbucket/Gitea → 13 STAR Domain Event） + 失败重试 + 死信 + 9 项已知缺口 | Phase E spec 起草（3 份：01-sa / 02-sse / 03-webhook）|
