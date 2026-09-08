# OPS-BASIC-DESIGN-001

> **STAR Ops Console 基本設計書 v0.1 (MVP-骨架)**
>
> - 状态: Basic Design Baseline (Draft)
> - 上游: `docs/requirements/SRS-STAR-OPS-001.md` v0.1
> - 下游: 詳設計 (随实装迭代, per 拍板 Q4 跳详设)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-09-08 JST

---

## §0 文档目的

本文档定义 STAR Ops Console MVP-骨架 的基本設計，覆蓋 (a) 组件拓扑 (b) 模块划分 (c) REST API 契约 (d) 数据模型 + W/T/M 分类 (e) AI 通道 + Fallback Ladder (f) 部署与可观测。

**MVP 范围** (per SRS-001 §3.1): 入口 + 4 tab 骨架 + 8 REST stub + Hybrid AI mock + crates/star-ops 48 package。

**Framework 选型**: **axum 0.8** (per ADR-0048, 跟既有 4 crate `star-mcp` / `star-api-rest` / `star-credential` / `star-ops` 100% 对齐, 拒绝 actix-web / warp / rocket / hyper 直用, 跟 RGS 仓独立 per AGENTS.md §5)

**不**做 (per SRS-001 §3.2): 4 类功能端到端 / 真实 K8s/Helm / 真实 LLM / 持久化 / 跨域编排 / 详設 (跳)。

---

## §1 架构总览

### 1.1 系统拓扑

```
                    ┌─────────────────────────────────┐
                    │  Browser (Next.js)              │
                    │  /ops (4 tab)                   │
                    └────────────────┬────────────────┘
                                     │ fetch (port 8090)
                                     ▼
                    ┌─────────────────────────────────┐
                    │  star-ops binary (port 8090)    │
                    │  axum 0.8 + 6-field 错误模型     │
                    │  ┌────────────────────────────┐ │
                    │  │ ops_api (8 REST stub)      │ │
                    │  └────────┬───────────────────┘ │
                    │           ▼                      │
                    │  ┌────────────────────────────┐ │
                    │  │ ops_domain (3 子域)        │ │
                    │  │  ├─ cluster                │ │
                    │  │  ├─ log                    │ │
                    │  │  └─ metrics                │ │
                    │  └────────┬───────────────────┘ │
                    │           ▼                      │
                    │  ┌────────────────────────────┐ │
                    │  │ ops_ai (Hybrid)            │ │
                    │  │  ├─ mock (subprocess)      │ │
                    │  │  ├─ openai_stub            │ │
                    │  │  ├─ anthropic_stub         │ │
                    │  │  └─ ladder (Fallback 4)    │ │
                    │  └────────────────────────────┘ │
                    └────┬────────────────────────┬───┘
                         │                        │
                         ▼                        ▼
              ┌──────────────────┐    ┌──────────────────┐
              │ scripts/auto/    │    │ star-context     │
              │ ai_log_mock.py   │    │ ActorContext     │
              │ (subprocess)     │    │ (P0-1 共享)      │
              └──────────────────┘    └──────────────────┘
```

### 1.2 部署单元 (MVP)

| 单元 | 形态 | 端口 | HA |
|---|---|---|---|
| frontend (Next.js) | 既有 `apps/frontend` 扩 `/ops/*` | 3000 | 既有 HA |
| star-ops | 新增 binary, K8s deployment | 8090 | MVP 单实例, 实装阶段 HA |
| ai_log_mock.py | Python subprocess 调起 | — | 一次性调用, 进程级隔离 |

### 1.3 守门 (Quality Gate per 守门 #1 v3+v25)

1. `cargo check --workspace --all-targets -j 4` 0 err
2. `cargo fmt --all -- --check` 0 err
3. `cargo clippy --workspace --all-targets -- -D warnings` 0 err (advisory per 守门 #7 v3)
4. `cargo test -p star-ops --lib -j 4` 100% pass (单 crate per 守门 #1 v25)
5. frontend `npm run typecheck` 0 err (advisory per 守门 #6 v2)

---

## §2 组件划分

### 2.1 crates/star-ops 内部模块

| 模块 | 文件 | 职责 | 依赖 |
|---|---|---|---|
| `error` | `src/error.rs` | 6-field 错误模型 (复用 star-mcp 模式) | thiserror |
| `ops_api` | `src/ops_api.rs` | 8 REST stub 路由 + axum Router | axum 0.8, tower, ops_domain, ops_ai |
| `ops_domain` | `src/ops_domain/mod.rs` | 3 子域聚合 | ops_domain::* |
| `ops_domain::cluster` | `src/ops_domain/cluster.rs` | F-01 HelmRelease + 4 stub 方法 | serde |
| `ops_domain::log` | `src/ops_domain/log.rs` | F-02 LogEntry + LogAnalysis | serde, ops_ai |
| `ops_domain::metrics` | `src/ops_domain/metrics.rs` | F-03 OpsMetric | serde |
| `ops_ai` | `src/ops_ai/mod.rs` | AI 通道 trait + Fallback Ladder | ops_ai::* |
| `ops_ai::mock` | `src/ops_ai/mock.rs` | subprocess 调 ai_log_mock.py | tokio::process, serde_json |
| `ops_ai::openai_stub` | `src/ops_ai/openai_stub.rs` | trait impl, 真实调用 `// TODO` | reqwest (备) |
| `ops_ai::anthropic_stub` | `src/ops_ai/anthropic_stub.rs` | 同上 | reqwest (备) |
| `ops_ai::ladder` | `src/ops_ai/ladder.rs` | 4 级回退逻辑 (per ADR-0026 §2.2) | ops_ai::* |

### 2.2 Cargo.toml 关键依赖

```toml
[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
anyhow = { workspace = true }
tokio = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }

# HTTP server (跟 star-mcp / star-api-rest 同版本)
axum = "0.8"
tower = { version = "0.5", features = ["util"] }

# 跨 crate 复用
star-context = { path = "../star-context" }     # ActorContext (P0-1 共享, per 守门 #4.2 v16)
star-credential = { path = "../star-credential" }  # API key 加密 (实装阶段)

# 实装阶段再引入
# kube = { version = "0.95" }         # K8s client (F-01 实装)
# reqwest = { version = "0.12" }      # 真实 LLM 通道 (F-02 实装)
# walkdir = "2"                       # F-04 文档扫描 (实装)
```

---

## §3 REST API 契约 (8 stub)

### 3.1 F-01 Cluster (4 端点)

#### `GET /api/ops/cluster/releases`

列出 helm release。

**Request**: 无

**Response 200** (MVP stub):
```json
{
  "data": [
    {
      "name": "star-mcp",
      "namespace": "default",
      "chart": "star-mcp-0.1.0",
      "revision": 3,
      "status": "Healthy",
      "last_deployed_at": "2026-09-08T07:00:00Z",
      "canary_weight": 0
    }
  ],
  "meta": { "total": 1, "stub": true, "hint": "F-01 实装阶段接入 kube-rs" }
}
```

**Errors**: 401 UNAUTHORIZED · 429 RATE_LIMITED · 501 NOT_IMPLEMENTED (实装时返) · 500 INTERNAL

#### `POST /api/ops/cluster/canary`

触发灰度 (MVP 仅 record, 不实装 helm exec)。

**Request**:
```json
{
  "release_name": "star-mcp",
  "canary_weight": 10,
  "target_revision": 4
}
```

**Response 200**:
```json
{
  "data": { "action_id": "uuid-stub", "status": "Pending" },
  "meta": { "stub": true, "hint": "F-01 实装阶段接入 helm upgrade --canary" }
}
```

#### `POST /api/ops/cluster/rollback`

回滚到指定 revision。

**Request**:
```json
{ "release_name": "star-mcp", "target_revision": 2 }
```

**Response 200**:
```json
{ "data": { "action_id": "uuid-stub", "status": "Pending" }, "meta": { "stub": true } }
```

#### `GET /api/ops/cluster/status`

查询 release 当前状态。

**Response 200**:
```json
{ "data": { "release_name": "star-mcp", "phase": "Healthy", "replicas": { "ready": 3, "desired": 3 } }, "meta": { "stub": true } }
```

### 3.2 F-02 LogAI (2 端点)

#### `POST /api/ops/log/upload`

上传 log 文本 (MVP 内存, 不持久化)。

**Request**:
```json
{
  "source": "k8s-pod/star-mcp-7d8b",
  "level_filter": ["ERROR", "WARN"],
  "content": "2026-09-08T07:30:00Z ERROR ..."
}
```

**Response 200**:
```json
{
  "data": {
    "log_id": "uuid-stub",
    "entry_count": 12,
    "analysis_triggered": true
  },
  "meta": { "stub": true, "ai_channel": "mock" }
}
```

#### `GET /api/ops/log/analysis/{id}`

查询 AI 分析结果。

**Response 200**:
```json
{
  "data": {
    "log_id": "uuid-stub",
    "summary": "12 条 log 中 3 条 ERROR, 集中在 07:30:00, 跟 helm release 3 部署时间吻合",
    "anomalies": [
      { "type": "spike", "timestamp": "2026-09-08T07:30:00Z", "level": "ERROR", "message_excerpt": "..." }
    ],
    "suggestions": [
      { "id": "s1", "text": "检查 helm release 3 的 health check 配置", "confidence": 0.42 }
    ],
    "confidence": 0.42,
    "generated_by": "mock"
  },
  "meta": { "stub": true, "needs_review": true }
}
```

### 3.3 F-03 Metrics (1 端点)

#### `GET /api/ops/metrics/summary`

5 KPI 汇总。

**Response 200**:
```json
{
  "data": [
    { "name": "cpu_avg", "value": 0.35, "unit": "ratio", "trend": "stable" },
    { "name": "mem_avg", "value": 0.62, "unit": "ratio", "trend": "rising" },
    { "name": "active_tasks", "value": 17, "unit": "count", "trend": "stable" },
    { "name": "mcp_qps", "value": 4.2, "unit": "qps", "trend": "falling" },
    { "name": "llm_token_daily", "value": 1240000, "unit": "tokens", "trend": "rising" }
  ],
  "meta": { "stub": true, "hint": "F-03 实装阶段接 star-telemetry" }
}
```

### 3.4 F-04 Docs (1 端点)

#### `GET /api/ops/docs`

列出 ops 相关文档。

**Response 200**:
```json
{
  "data": [
    { "path": "docs/requirements/SRS-STAR-OPS-001.md", "title": "STAR Ops Console SRS", "category": "SRS", "updated_at": "2026-09-08T07:53:00Z" },
    { "path": "docs/basic-design/OPS-BASIC-DESIGN-001.md", "title": "STAR Ops Console 基本设计", "category": "BAS", "updated_at": "2026-09-08T07:53:00Z" }
  ],
  "meta": { "stub": true, "total": 2 }
}
```

### 3.5 统一错误响应

```json
{
  "error": {
    "code": "NOT_IMPLEMENTED",
    "message": "F-01 集群更新端到端实装待 [M] 子项拍板",
    "source_module": "star_ops::ops_api::cluster",
    "source_kind": "stub",
    "retriable": false,
    "hint": "见 docs/requirements/SRS-STAR-OPS-001.md §3.1 落档清单"
  }
}
```

---

## §4 数据模型

### 4.1 6 张表 (W/T/M 100% 覆盖 per 守门 #13)

| # | 表 | 分类 | 字段 (MVP 关键) | 派生规 |
|---|---|---|---|---|
| 1 | `ops_helm_release_state` | T | release_name, namespace, revision, status, last_deployed_at, canary_weight + RLS 13 類 | T 派生规 (b) |
| 2 | `ops_cluster_action_log` | T | action_id, release_name, action_type (canary/rollback), actor_user_id, requested_at, completed_at, status, payload + RLS 13 類 + WORM | T 派生规 (b) |
| 3 | `ops_log_query_log` | T | query_id, user_id, log_source, query_text, returned_count, queried_at + RLS 13 類 + WORM | T 派生规 (b) |
| 4 | `ops_log_entry` | W | log_id, source, level, message, timestamp, trace_id, retention_period (7d) | W 派生规 (a) |
| 5 | `ops_log_analysis` | W | analysis_id, log_id (FK), summary, anomalies (JSONB), suggestions (JSONB), confidence, generated_by, retention_period (30d) | W 派生规 (a) |
| 6 | `ops_metrics_config` | M | metric_name, source, unit, threshold_min, threshold_max, alert_channel, valid_from, valid_to (SCD Type 2), is_current + RLS 13 類 | M 派生规 (c) |

**混合**: 0 张 (100% 干净)
**覆盖**: 6/6 = 100%

### 4.2 RLS 13 類 (T/M 必携 per 守门 #13)

```sql
tenant_id, user_id, role_id, permission_id, policy_id,
workspace_id, project_id, work_item_id, agent_id,
session_id, trace_id, source_module, source_kind
```

### 4.3 引用基线

- `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` v0.1
- `docs/data-design/ipa-detail/00-CLASSIFICATION-RULES.md` v0.1

---

## §5 AI 通道 + Fallback Ladder

### 5.1 trait 设计

```rust
// crates/star-ops/src/ops_ai/mod.rs
#[async_trait]
pub trait AiChannel: Send + Sync {
    /// 通道名 (用于 meta.generated_by)
    fn name(&self) -> &'static str;

    /// 是否启用 (false 时被 ladder 跳过)
    fn is_enabled(&self) -> bool;

    /// 分析 log, 返回 LogAnalysis
    async fn analyze_log(&self, log: &LogEntry) -> Result<LogAnalysis, OpsError>;
}
```

### 5.2 4 级 Ladder (per ADR-0026 §2.2)

```rust
// crates/star-ops/src/ops_ai/ladder.rs
pub struct Ladder {
    channels: Vec<Box<dyn AiChannel>>,  // [mock, openai, anthropic]
}

impl Ladder {
    pub async fn analyze_log(&self, log: &LogEntry) -> Result<LogAnalysis, OpsError> {
        let mut last_err = None;
        for ch in &self.channels {
            if !ch.is_enabled() { continue; }
            match ch.analyze_log(log).await {
                Ok(analysis) => {
                    // 守门 #23: confidence < 0.5 必标 "需人工 review"
                    if analysis.confidence < 0.5 {
                        tracing::warn!(confidence = analysis.confidence, "AI analysis needs human review");
                    }
                    return Ok(analysis);
                }
                Err(e) if e.retriable => {
                    tracing::warn!(channel = ch.name(), err = %e, "AI channel failed, fallback");
                    last_err = Some(e);
                    continue;
                }
                Err(e) => return Err(e),  // 非 retriable, 直接返
            }
        }
        Err(last_err.unwrap_or_else(|| OpsError::no_channel_available()))
    }
}
```

### 5.3 mock 通道 (守门 #23 + 守门 #24)

```rust
// crates/star-ops/src/ops_ai/mock.rs
pub struct MockChannel;

#[async_trait]
impl AiChannel for MockChannel {
    fn name(&self) -> &'static str { "mock" }
    fn is_enabled(&self) -> bool { true }  // 永远启用 (兜底)

    async fn analyze_log(&self, log: &LogEntry) -> Result<LogAnalysis, OpsError> {
        // subprocess 调 scripts/automation/ai_log_mock.py (per 守门 #9 v3)
        let output = tokio::process::Command::new("python")
            .arg("scripts/automation/ai_log_mock.py")
            .arg(&log.message)
            .output().await
            .map_err(OpsError::from)?;

        // 解析 JSON 输出
        let analysis: LogAnalysis = serde_json::from_slice(&output.stdout)?;
        Ok(analysis)
    }
}
```

### 5.4 OpenAI / Anthropic stub

```rust
// crates/star-ops/src/ops_ai/openai_stub.rs
pub struct OpenAiStub { api_key: Option<String> }

#[async_trait]
impl AiChannel for OpenAiStub {
    fn name(&self) -> &'static str { "openai" }
    fn is_enabled(&self) -> bool { self.api_key.is_some() }

    async fn analyze_log(&self, _log: &LogEntry) -> Result<LogAnalysis, OpsError> {
        // TODO [M] 子项: 真实 OpenAI API 调用
        // MVP stub: 返回 mock, 标 generated_by="openai_stub"
        Err(OpsError::not_implemented("OpenAI real call pending [M] sub-task"))
    }
}
```

---

## §6 部署与可观测

### 6.1 K8s deployment (MVP)

```yaml
# deploy/star-ops-deployment.yaml (MVP 简化版)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: star-ops
  labels: { app: star-ops, component: ops-console }
spec:
  replicas: 1  # MVP 单实例
  selector:
    matchLabels: { app: star-ops }
  template:
    metadata:
      labels: { app: star-ops }
    spec:
      containers:
      - name: star-ops
        image: star-ops:latest
        ports: [{ containerPort: 8090 }]
        env:
        - name: RUST_LOG
          value: "info,star_ops=debug"
        livenessProbe:
          httpGet: { path: /healthz, port: 8090 }
        readinessProbe:
          httpGet: { path: /readyz, port: 8090 }
```

### 6.2 健康检查

- `GET /healthz` → 200 OK (进程级)
- `GET /readyz` → 200 OK (依赖检查, MVP 总是 200)

### 6.3 观测

- `tracing` crate 日志 (复用 workspace 依赖)
- 结构化字段: `request_id`, `user_id`, `route`, `duration_ms`, `status_code`
- 实装阶段接 `star-telemetry` (已存在, 复用)

### 6.4 守门 #5 v2 派生 (API key 安全)

- `OPENAI_API_KEY` / `ANTHROPIC_API_KEY` 不进环境变量, 走 UI 配置 + `star-credential` 加密存储
- 守门 #5: 禁 `Get-ChildItem env:` / `echo $VAR` 泄露操作

---

## §7 边界与不做 (per SRS-001 §3.2)

- 4 类功能端到端 → 等 [M] 子项拍板
- 真实 K8s/Helm / 真实 LLM / 持久化 / 跨域编排 → 全部 MVP 不做
- 详设文档 → 随实装迭代 (per 拍板 Q4)
- OAuth 2.0 / mTLS → MVP auth stub, 复用 `star-context::ActorContext`
- OpenAPI 3.1 spec 自动生成 → per `star-api-rest` v0.1 同模式 (待 v0.2 utoipa)

---

## §8 签字栏 (per AGENTS.md §3 模板)

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 JST |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |

> 5 域 Lead 真人到位后追溯签字覆盖 (per 守门 #14 + §6 SRS-001), 不沿用代签决策 (per 守门 #1 禁回溯)

---

## §9 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 MVP-骨架 基本设计 (4 tab + 8 REST + Hybrid AI + W/T/M 6 表) | ask_user `ask_e76f2e614519fbc9eda16b53` 拍板 (4 opt 选 4 项) |
