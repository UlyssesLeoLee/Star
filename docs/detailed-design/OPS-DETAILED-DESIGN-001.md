# OPS-DETAILED-DESIGN-001

> **STAR Ops Console 詳細設計書 v0.1**
>
> - 状态: Detailed Design Baseline (Draft)
> - 上游: `docs/requirements/SRS-STAR-OPS-001.md` v0.1 + `docs/basic-design/OPS-BASIC-DESIGN-001.md` v0.1
> - 下游: 实装代码 (per commit `03d7d43` + hotfix `7934131` MVP-骨架已落档)
> - 核心语言: Rust (后端) + TypeScript (前端)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-09-08 JST

---

## §0 目的 (Purpose)

本詳細設計書は `OPS-BASIC-DESIGN-001.md` v0.1 で定めた基本設計を実装可能なレベルまで展開する。MVP-骨架段階 (commit `03d7d43` + hotfix `7934131` 実証) のコード形状と 100% 一致させ、後の 4 つの [M]/[S] 子項 (F-01..F-04 端到端) 推进時の実装仕様を提供する。

**核心スコープ**:

- **5 維** 設計: モジュール / クラス / 時序 / 状態遷移 / テスト
- **W/T/M 6 表 100% カバー** (守門 #13) — データ永続化 §7 で詳細
- **Hybrid AI 4 級 Ladder** (per ADR-0026 §2.2) — §3 で詳細
- **不做什么** (per SRS-001 §3.2): 跨域编排 / 真实 K8s / 真实 LLM / OAuth 2.0 / 詳設で OpenAPI utoipa 自動生成

---

## §1 モジュール設計 (Module Design)

### 1.1 物理 crate (workspace 48/48, per `crates/star-ops/Cargo.toml`)

```
crates/star-ops/
├── Cargo.toml                  # name=star-ops, 1 [[bin]] + 1 [lib], 9 deps (post 7934131 self-review)
├── src/
│   ├── main.rs                 # binary 入口 (per star-mcp 模式)
│   ├── lib.rs                  # lib 入口, 暴露 4 个子模块
│   ├── error.rs                # 6-field 错误模型 + 5 variant OpsError + IntoResponse
│   ├── ops_api.rs              # 8 REST stub + 2 health, axum 0.8 Router
│   ├── ops_domain/
│   │   ├── mod.rs              # 3 子域聚合
│   │   ├── cluster.rs          # F-01: HelmRelease + 3 stub
│   │   ├── log.rs              # F-02: LogEntry + LogAnalysis
│   │   └── metrics.rs          # F-03: OpsMetric + 5 KPI
│   └── ops_ai/
│       ├── mod.rs              # AiChannel trait + default_ladder() 工厂
│       ├── mock.rs             # L1: MockChannel (subprocess 路径 [M] 启用)
│       ├── openai_stub.rs      # L2: OpenAiStub (NOT_IMPLEMENTED)
│       ├── anthropic_stub.rs   # L3: AnthropicStub (NOT_IMPLEMENTED)
│       └── ladder.rs           # 4 级回退逻辑
```

**总行数**: 14 文件, ~1.2K 行 (含注释 + tests), MVP 阶段 0 dead code (per 7934131 self-review)。

### 1.2 依存関係 (per 7934131, self-review 后最小化)

```toml
# workspace 基础 (8 项)
serde, serde_json, thiserror, anyhow, tokio, uuid, chrono, tracing
# main.rs 启动用
tracing-subscriber = "0.3" (env-filter)
# HTTP server
axum = "0.8"
# async-trait
async-trait = { workspace = true }
# dev-deps (1 项)
tower = "0.5" (util, 仅 oneshot tests)

# 显式不引入 (per SRS-001 §10.2, MVP 阶段):
# kube = "0.95"       # F-01 [M] 启用
# reqwest = "0.12"    # F-02 [M] 启用
# walkdir = "2"        # F-04 [M] 启用
# star-context         # P0-1 联动 [M] 启用 (per 7934131 self-review)
```

**依賴 9 项 + 1 dev-dep = 10 总依赖**, MVP 阶段最小化。

### 1.3 frontend 物理モジュール (per `/ops` 路由 + UserMenu 入口)

```
frontend/src/
├── app/
│   └── ops/
│       └── page.tsx            # 4 tab 骨架 + Hero 头部 + 4 KPI 胶囊
├── components/
│   └── UserMenu.tsx            # line 219-235 加 Wrench 入口
└── lib/
    └── i18n/
        ├── dictionary.ts       # Dictionary interface + userMenu.ops + opsConsole
        ├── zh-CN.ts            # 中文翻译
        ├── en.ts               # 英文翻译
        └── ja.ts               # 日文翻译
```

### 1.4 Python 自動化 (per 守門 #19 v19 + #21 v21 + #23)

```
scripts/automation/
├── ai_log_mock.py              # L1 mock subprocess 入口 (3ms 跑通)
├── registry.md                 # 索引 +1 行 (per 守門 #21 v21)
└── (其他既有基类保持不变)
```

### 1.5 モジュール責務マトリクス (per DDD bounded context)

| モジュール | 責務 | 上流 | 下流 |
|---|---|---|---|
| `error` | 6-field 错误模型 | star-mcp error.rs 模式 | ops_api, ops_domain, ops_ai |
| `ops_api` | HTTP routing | axum 0.8 | ops_domain, ops_ai (state) |
| `ops_domain::cluster` | F-01 数据 + 业务 | serde | ops_api::cluster_* |
| `ops_domain::log` | F-02 数据 + 业务 | ops_ai::AiChannel | ops_api::log_* |
| `ops_domain::metrics` | F-03 数据 + 业务 | chrono | ops_api::metrics_summary |
| `ops_ai` | AI 通道抽象 + Ladder | async-trait | ops_api::log_upload |
| `ops_ai::mock` | L1 mock (subprocess) | tokio::process | ops_ai::Ladder |
| `ops_ai::{openai,anthropic}_stub` | L2/L3 stub | async-trait | ops_ai::Ladder |
| `ops_ai::ladder` | 4 级回退 | ops_ai::* | ops_api::log_upload |
| `frontend/app/ops/page.tsx` | UI 4 tab | Tabs 组件 | 8 REST endpoint |
| `scripts/automation/ai_log_mock.py` | subprocess 模板 | argparse | star-ops::mock |

---

## §2 クラス設計 (Class Design)

### 2.1 错误模型 (per OPS-BASIC-DESIGN §3.5 + 守門 #7 0 unsafe)

```rust
// crates/star-ops/src/error.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorSourceKind {
    Internal,    // 内部逻辑错误 (MVP stub 未实装)
    External,    // 外部系统错误 (K8s/LLM 调用失败, 实装阶段触发)
    Policy,      // 策略层拒绝 (权限/限流, MVP stub)
    Validation,  // 参数 / schema 校验失败
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpsErrorBody {
    pub code: String,               // SCREAMING_SNAKE_CASE
    pub message: String,            // 人类可读
    pub source_module: String,      // e.g. "star_ops::ops_api::cluster"
    pub source_kind: ErrorSourceKind,
    pub retriable: bool,
    pub hint: Option<String>,
}

#[derive(Debug, Error)]
pub enum OpsError {
    #[error("NOT_IMPLEMENTED: {0}")]
    NotImplemented(String),
    #[error("UNAUTHORIZED: {0}")]
    Unauthorized(String),
    #[error("RATE_LIMITED: {0}")]
    RateLimited(String),
    #[error("BAD_REQUEST: {0}")]
    BadRequest(String),
    #[error("INTERNAL: {0}")]
    Internal(String),
}

impl OpsError {
    pub fn to_body(&self, source_module: &str) -> OpsErrorBody { ... }
    pub fn not_implemented(what: &str) -> Self { ... }
    pub fn no_channel_available() -> Self { ... }
}

impl axum::response::IntoResponse for OpsError {
    fn into_response(self) -> axum::response::Response { ... }
}
```

**5 个 5 unit tests** (per error.rs:119-145): 验证每个 variant 的 code/source_kind/retriable 字段。

### 2.2 AI 通道 trait (per OPS-BASIC-DESIGN §5.1 + 守門 #23)

```rust
// crates/star-ops/src/ops_ai/mod.rs
#[async_trait]
pub trait AiChannel: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_enabled(&self) -> bool;
    async fn analyze_log(&self, log: &LogEntry) -> Result<LogAnalysis, OpsError>;
}

pub fn default_ladder() -> ladder::Ladder {
    ladder::Ladder::new(vec![
        Box::new(mock::MockChannel),
        Box::new(openai_stub::OpenAiStub::default()),
        Box::new(anthropic_stub::AnthropicStub::default()),
    ])
}
```

### 2.3 4 级 Ladder (per ADR-0026 §2.2)

```rust
// crates/star-ops/src/ops_ai/ladder.rs
pub struct Ladder {
    channels: Vec<Box<dyn AiChannel>>,  // 顺序 = L1→L2→L3→L4
}

impl Ladder {
    pub fn new(channels: Vec<Box<dyn AiChannel>>) -> Self { ... }

    pub async fn analyze_log(&self, log: &LogEntry) -> Result<LogAnalysis, OpsError> {
        let mut last_err = None;
        for ch in &self.channels {
            if !ch.is_enabled() { continue; }
            match ch.analyze_log(log).await {
                Ok(analysis) => {
                    // 守門 #23: confidence < 0.5 必标 "需人工 review"
                    if analysis.confidence < 0.5 {
                        tracing::warn!(...);
                    }
                    return Ok(analysis);
                }
                Err(OpsError::Internal(_)) => {
                    // retriable, 继续下一通道
                    last_err = Some(...);
                    continue;
                }
                Err(e) => return Err(e),  // NOT_IMPLEMENTED / Validation 直接返
            }
        }
        Err(last_err.unwrap_or_else(OpsError::no_channel_available))
    }
}
```

**回退规则**:
- L1 mock: 永远启用 (兜底, 守門 #23 mock 永远 < 0.5)
- L2 OpenAI: `api_key.is_some()` 时启用, 未配置自动跳到 L3
- L3 Anthropic: 同 L2
- L4 兜底: 永远 L1 (兜底本身)

### 2.4 Mock 通道 (per 守門 #23 + 守門 #24 v2)

```rust
// crates/star-ops/src/ops_ai/mock.rs
pub struct MockChannel;

#[async_trait]
impl AiChannel for MockChannel {
    fn name(&self) -> &'static str { "mock" }
    fn is_enabled(&self) -> bool { true }

    async fn analyze_log(&self, log: &LogEntry) -> Result<LogAnalysis, OpsError> {
        // MVP: 直接返 mock_analyze(log) (subprocess 路径 [M] 启用)
        // 实装路径: tokio::process::Command::new("python").arg("ai_log_mock.py")...
        Ok(mock_analyze(log))
    }
}

fn mock_analyze(log: &LogEntry) -> LogAnalysis {
    let (anomalies, suggestions) = if matches!(log.level, LogLevel::Error) {
        // 1 个 spike + 1 个 s-healthcheck suggestion, confidence 0.42
    } else {
        (vec![], vec![])
    };
    LogAnalysis { ..., confidence: 0.42, generated_by: "mock".to_string() }
}
```

**3 unit tests** (per mock.rs:140-180): mock_channel_always_enabled / mock_analyze_error_log / mock_analyze_info_log。

### 2.5 REST 路由 (per OPS-BASIC-DESIGN §3)

```rust
// crates/star-ops/src/ops_api.rs
#[derive(Clone)]
pub struct AppState {
    pub ladder: std::sync::Arc<ladder::Ladder>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        // F-01 (4)
        .route("/api/ops/cluster/releases", get(cluster_list))
        .route("/api/ops/cluster/canary", post(cluster_canary))
        .route("/api/ops/cluster/rollback", post(cluster_rollback))
        .route("/api/ops/cluster/status", get(cluster_status))
        // F-02 (2)
        .route("/api/ops/log/upload", post(log_upload))
        .route("/api/ops/log/analysis/{id}", get(log_analysis))  // axum 0.8: {id} 不是 :id
        // F-03 (1)
        .route("/api/ops/metrics/summary", get(metrics_summary))
        // F-04 (1)
        .route("/api/ops/docs", get(docs_list))
        // Health (2)
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .with_state(state)
}
```

**4 e2e tests** (per ops_api.rs:295-360): cluster_list / metrics_summary / healthz / log_analysis, 全部走 `tower::ServiceExt::oneshot`。

### 2.6 数据结构 (per OPS-BASIC-DESIGN §4)

| Struct | 字段 (MVP 关键) | 派生 / 注解 |
|---|---|---|
| `HelmRelease` | name, namespace, chart, revision, status, last_deployed_at, canary_weight | 2 unit tests |
| `CanaryRequest` | release_name, canary_weight (0-100), target_revision | 1 unit test |
| `RollbackRequest` | release_name, target_revision | 跟 canary 共用 ack |
| `LogEntry` | id (Uuid), source, level, message, timestamp, trace_id | 1 unit test |
| `LogAnalysis` | log_id, summary, anomalies, suggestions, confidence, generated_by | 1 unit test (守門 #23) |
| `Anomaly` | type, timestamp, level, message_excerpt | — |
| `Suggestion` | id, text, confidence | — |
| `OpsMetric` | name, value, unit, trend, last_updated | 1 unit test |
| `OpsResponse<T>` | data, meta (含 stub/hint/ai_channel/needs_review) | 通用 wrapper |

---

## §3 Hybrid AI 詳細 (Hybrid AI Detailed Design)

### 3.1 通道抽象 (per §2.2)

`AiChannel` trait 4 個方法:
1. `name()` — 静态字符串, 用於 `meta.generated_by` (mock / openai / anthropic)
2. `is_enabled()` — bool, false 时被 ladder 跳过
3. `analyze_log()` — async, 返 `Result<LogAnalysis, OpsError>`

### 3.2 4 級 Ladder 状態 (per ADR-0026 §2.2 + §2.3)

```
┌─────────────────────────────────────────┐
│ L1: MockChannel (is_enabled=true)       │  ← 永远兜底
│  - 模板生成, confidence=0.42             │
│  - subprocess 路径 [M] 子項启用           │
└─────────────────────────────────────────┘
                ↓ 失敗/未配置
┌─────────────────────────────────────────┐
│ L2: OpenAiStub (is_enabled=api_key?)    │  ← 拍板 [M] 时启用 reqwest
│  - real API: 401/429/5xx → retriable    │
│  - MVP: NOT_IMPLEMENTED                  │
└─────────────────────────────────────────┘
                ↓ 失敗
┌─────────────────────────────────────────┐
│ L3: AnthropicStub (is_enabled=api_key?) │  ← 拍板 [M] 时启用 reqwest
│  - 同 L2                                 │
│  - MVP: NOT_IMPLEMENTED                  │
└─────────────────────────────────────────┘
                ↓ 失敗
        L4: 兜底 = L1 (回 L1)
```

### 3.3 拍板决策树 (per Q2 Hybrid 拍板)

```
is L1 enabled? (永远 yes)
  → YES: try mock
      ├─ success: 返 mock analysis
      └─ fail (Internal/retriable): 继续
is L2 enabled? (api_key 配置?)
  → YES: try openai
      ├─ success: 返 openai analysis
      ├─ fail retriable: 继续
      └─ fail non-retriable (NOT_IMPLEMENTED/Validation): 直接返
is L3 enabled?
  → YES: 同 L2
所有 L1-L3 失敗: 返 OpsError::no_channel_available()
```

### 3.4 守門 #23 派生 (per 2026-09-02 09:01 JST 拍板)

- mock 通道 confidence 永远 < 0.5 (强制 0.42)
- mock 通道永远 needs_review = true
- 不开 OpenAI/Anthropic 第三方 API
- API key 不走 UI 输入, 走 star-credential 加密存储 ([M] 子项)

### 3.5 subprocess 路径 (实装阶段 [M] 启用)

```rust
// crates/star-ops/src/ops_ai/mock.rs (预留 #[allow(dead_code)] call_subprocess_stub)
pub async fn call_subprocess_stub(log_message: &str) -> Result<MockSubprocessOutput, OpsError> {
    let output = tokio::process::Command::new("python")
        .arg("scripts/automation/ai_log_mock.py")
        .arg(log_message)
        .output()
        .await
        .map_err(|e| OpsError::Internal(format!("subprocess 调起失败: {}", e)))?;
    // ... 解析 JSON 返 MockSubprocessOutput
}
```

`scripts/automation/ai_log_mock.py` 跑通 3ms 实证, JSON 形状:
```json
{
  "summary": "...",
  "anomalies": [{"type": "spike", "timestamp": "...", "level": "ERROR", "message_excerpt": "..."}],
  "suggestions": [{"id": "s-healthcheck", "text": "...", "confidence": 0.42}],
  "confidence": 0.42,
  "generated_by": "mock",
  "elapsed_ms": 3,
  "needs_review": true
}
```

### 3.6 OpenAI / Anthropic stub 接口 (实装 [M] 启用)

```rust
// crates/star-ops/src/ops_ai/openai_stub.rs
#[derive(Default)]
pub struct OpenAiStub {
    pub api_key: Option<String>,  // [M] 阶段从 star-credential 读
}

#[async_trait]
impl AiChannel for OpenAiStub {
    fn name(&self) -> &'static str { "openai" }
    fn is_enabled(&self) -> bool { self.api_key.is_some() }
    async fn analyze_log(&self, _log: &LogEntry) -> Result<LogAnalysis, OpsError> {
        // [M] 实装:
        // 1. reqwest::Client::new() 建 HTTP client
        // 2. POST https://api.openai.com/v1/chat/completions
        // 3. Bearer ${api_key}
        // 4. 解析 response, 转 LogAnalysis
        // 5. 错误映射: 401/429/5xx → OpsError::Internal(retriable)
        Err(OpsError::not_implemented("OpenAI 真实 API 调用"))
    }
}
```

Anthropic stub 同 pattern。

---

## §4 シーケンス図 (Sequence Diagrams)

### 4.1 UC-01 User 點击 UserMenu → 跳 /ops

```
┌────────┐  ┌──────────────┐  ┌─────────────┐  ┌────────┐
│ User   │  │ UserMenu.tsx │  │ Next.js     │  │ /ops   │
│        │  │ (React)      │  │ Router      │  │ page   │
└───┬────┘  └──────┬───────┘  └──────┬──────┘  └───┬────┘
    │ click avatar │                  │             │
    │──────────────>                  │             │
    │              open menu           │             │
    │              (Wrench 入口可见)    │             │
    │ click Wrench  │                  │             │
    │──────────────>                  │             │
    │              router.push("/ops")  │             │
    │─────────────────────────────────>│             │
    │                                  │ render     │
    │                                  │ <OpsPage>  │
    │                                  │────────────>│
    │                                  │             │
    │                                  │ Hero 头部   │
    │                                  │ 4 KPI 胶囊  │
    │                                  │ <Tabs>      │
    │                                  │ defaultValue│
    │                                  │ ="cluster"  │
    │                                  │ 4 <TabsContent> (cluster/logai/metrics/docs) │
    │                                  │             │
    │                              render 占位卡片   │
    │<─────────────────────────────────────────────│
    │              显示 /ops 页 (4 tab 骨架)          │
```

### 4.2 UC-02 /ops cluster tab → curl 验证 REST stub

```
┌────────┐  ┌────────────┐  ┌─────────────┐  ┌──────────────┐
│ curl   │  │ axum 0.8   │  │ ops_api     │  │ ops_domain   │
│ client │  │ Router     │  │ (handler)   │  │ cluster.rs   │
└───┬────┘  └──────┬─────┘  └──────┬──────┘  └──────┬───────┘
    │ GET          │                │                │
    │ /api/ops/    │                │                │
    │ cluster/     │                │                │
    │ releases     │                │                │
    │─────────────>│                │                │
    │              │ match route    │                │
    │              │───────────────>│                │
    │              │                │ HelmRelease    │
    │              │                │ ::list_stub()   │
    │              │                │───────────────>│
    │              │                │                │ hardcoded 1 条
    │              │                │                │ (star-mcp v3 Healthy)
    │              │                │<───────────────│
    │              │                │ wrap in        │
    │              │                │ OpsResponse    │
    │              │                │ {data, meta:   │
    │              │                │  stub=true,    │
    │              │                │  total=1,      │
    │              │                │  hint="F-01    │
    │              │                │  实装阶段接入  │
    │              │                │  kube-rs"}     │
    │              │<───────────────│                │
    │<─────────────│ JSON 200       │                │
    │ {data: [...], meta: {stub:true, total:1, ...}}  │
```

### 4.3 UC-03 /ops logai tab upload → mock AI 分析

```
┌────────┐  ┌────────────┐  ┌─────────────┐  ┌────────────┐  ┌────────────┐
│ curl   │  │ axum       │  │ log_upload  │  │ Ladder     │  │ MockChannel│
└───┬────┘  └──────┬─────┘  └──────┬──────┘  └──────┬─────┘  └──────┬─────┘
    │ POST         │                │                │               │
    │ /api/ops/    │                │                │               │
    │ log/upload   │                │                │               │
    │ {source:..., │                │                │               │
    │  content:    │                │                │               │
    │  "ERROR..."} │                │                │               │
    │─────────────>│                │                │               │
    │              │ match route    │                │               │
    │              │───────────────>│                │               │
    │              │                │ build LogEntry │               │
    │              │                │ (Uuid, Error)  │               │
    │              │                │                │               │
    │              │                │ ladder         │               │
    │              │                │ .analyze_log() │               │
    │              │                │──────────────>│               │
    │              │                │                │ check L1      │
    │              │                │                │ is_enabled    │
    │              │                │                │ = true        │
    │              │                │                │               │
    │              │                │                │ analyze_log() │
    │              │                │                │──────────────>│
    │              │                │                │               │ mock_analyze(log)
    │              │                │                │               │ → LogAnalysis
    │              │                │                │               │   confidence=0.42
    │              │                │                │               │   generated_by="mock"
    │              │                │                │<──────────────│
    │              │                │                │               │
    │              │                │                │ 守門 #23 check│
    │              │                │                │ confidence<0.5│
    │              │                │                │ → warn log    │
    │              │                │<──────────────│               │
    │              │                │ 返 Ok(analysis)│               │
    │              │                │                │               │
    │              │                │ wrap in        │               │
    │              │                │ UploadAck      │               │
    │              │                │ + OpsMeta      │               │
    │              │                │   ai_channel=  │               │
    │              │                │   "mock"       │               │
    │              │                │   analysis_    │               │
    │              │                │   triggered=   │               │
    │              │                │   true         │               │
    │              │<───────────────│                │               │
    │<─────────────│ JSON 200       │                │               │
    │ {data: {log_id, entry_count:1, analysis_triggered:true}, meta: {stub:true, ai_channel:"mock", ...}} │
```

### 4.4 UC-04 /ops logai analysis/{id} → 查 mock 结果

类似 UC-03, 但 GET 路径. `log_analysis(Path(id): Path<Uuid>)` 返 `LogAnalysis::stub_for(id)`, `meta.needs_review = confidence < 0.5` (守門 #23)。

### 4.5 UC-05 [M] 阶段: subprocess 路径

```
┌────────┐  ┌────────────┐  ┌─────────────┐  ┌────────────┐  ┌────────────┐
│ curl   │  │ MockChannel│  │ tokio::     │  │ ai_log_    │  │ JSON 解析  │
│ upload │  │            │  │ process     │  │ mock.py    │  │            │
└───┬────┘  └──────┬─────┘  └──────┬──────┘  └──────┬─────┘  └──────┬─────┘
    │ analyze_log  │                │                │               │
    │─────────────>│                │                │               │
    │              │ Command::new   │                │               │
    │              │ ("python")     │                │               │
    │              │ .arg("scripts/ │                │               │
    │              │  automation/   │                │               │
    │              │  ai_log_       │                │               │
    │              │  mock.py")     │                │               │
    │              │ .arg(log_msg)  │                │               │
    │              │───────────────>│                │               │
    │              │                │ spawn python   │               │
    │              │                │ subprocess     │               │
    │              │                │───────────────>│               │
    │              │                │                │ regex 抽取    │
    │              │                │                │ 3 anomalies  │
    │              │                │                │ 1 suggestion │
    │              │                │                │ JSON print   │
    │              │                │                │ (3ms)        │
    │              │                │<───────────────│               │
    │              │                │ stdout bytes   │               │
    │              │                │───────────────>│               │
    │              │                │                │               │ serde_json::from_slice
    │              │                │                │               │ → MockSubprocessOutput
    │              │<───────────────│                │               │
    │              │ → LogAnalysis   │                │               │
    │<─────────────│                │                │               │
```

**注意**: subprocess 路径在 MVP 阶段 `#[allow(dead_code)]` 预留, 实际未调 (per §2.4 mock_analyze 直接返)。[M] 阶段改为 `call_subprocess_stub(&log.message).await?`。

---

## §5 状態遷移図 (State Machine Diagrams)

### 5.1 HelmRelease.status (F-01)

```
              deploy
   ┌─────────────────────>┐
   │                       │
[Pending]              [Deploying]
   │                       │
   │ timeout/health        │ ok
   │ check pass            │
   │<─────────────[Healthy]│
   │                  ▲    │
   │                  │    │
   │       health     │    │ rollback / canary
   │       check fail │    │
   │                  │    │
   │              [Degraded]
   │                  │    │
   │                  │    │ all replicas down
   │                  └────[Failed]
   │                       │
   │                       │ manual recovery
   └───────────────────────┘
```

**当前 MVP 状态**: enum 5 variant, `list_stub()` 仅返 1 条 Healthy (per §2.6)。

### 5.2 LogAnalysis.confidence (F-02, 守門 #23)

```
[AI Channel analyze]
        │
        ▼
┌──────────────┐
│ confidence?  │
└──────┬───────┘
       │ < 0.5
       ▼
[mock: needs_review=true, UI 标"需人工"]
       │
       │ ≥ 0.5
       ▼
[正常返回, UI 隐藏提示]
```

**守門 #23 强制**: mock 通道 confidence 永远 0.42, 走 needs_review 分支。

### 5.3 Ladder 状态 (F-02, per §2.3)

```
[init: L1=L2=L3 disabled check]
    │
    ▼
[L1 is_enabled?]
    ├─ yes → try mock
    │       ├─ success → return analysis
    │       └─ fail (Internal/retriable) → continue
    └─ no  → skip
    │
    ▼
[L2 is_enabled? (api_key?)]
    ├─ yes → try openai
    │       ├─ success → return analysis
    │       ├─ fail retriable → continue
    │       └─ fail non-retriable → return Err
    └─ no  → skip
    │
    ▼
[L3 is_enabled?]
    ├─ yes → try anthropic
    │       ├─ success / fail retriable / fail non-retriable
    │       └─ (同 L2)
    └─ no  → skip
    │
    ▼
[all L1-L3 失败]
    │
    ▼
[return Err(OpsError::no_channel_available())]
```

### 5.4 Tabs 状态 (frontend, 守門 #6 v2 advisory)

```
[page load]
    │
    ▼
[useState activeTab = "cluster"]
    │
    ▼
[<Tabs value={activeTab} defaultValue="cluster">]
    │
    │ user click
    ▼
[onValueChange → setActiveTab(new)]
    │
    ▼
[re-render <TabsContent value=...>]
```

---

## §6 エラー処理設計 (Error Handling)

### 6.1 OpsError 5 variant × HTTP status code

| variant | HTTP status | 守門 | 触发 |
|---|---|---|---|
| `NotImplemented` | 501 | 守門 #1 (R-05 工具/数据接口) | MVP 8 stub 端点 + 真实 K8s/LLM 调用 (实装阶段) |
| `Unauthorized` | 401 | 守門 #1 v25 (ActorContext auth) | MVP 简化为 stub, 实装 [M] 接 star-context::ActorContext |
| `RateLimited` | 429 | 守門 #6 (限流 60 req/min per key) | MVP 不启用, 实装 [M] 接 tower-governor |
| `BadRequest` | 400 | 守門 #6 v2 (frontend typecheck advisory) | JSON 解析失败 / 字段缺 |
| `Internal` | 500 | 守門 #7 0 unsafe | subprocess 失败 / DB 错误 / 解析错 |

### 6.2 Ladder retriable 规则 (per §2.3)

- `OpsError::Internal(_)` → retriable, 继续下一通道
- `OpsError::NotImplemented(_)` → **非** retriable, 直接返 (避免 L2/L3 永远 NOT_IMPLEMENTED 卡死)
- `OpsError::BadRequest(_)` → **非** retriable, 直接返
- `OpsError::Unauthorized(_)` → **非** retriable, 直接返
- `OpsError::RateLimited(_)` → retriable, 继续下一通道

### 6.3 6-field 错误响应 (per §2.1)

```json
{
  "error": {
    "code": "NOT_IMPLEMENTED",
    "message": "F-01 集群更新端到端实装待 [M] 子项拍板",
    "source_module": "star_ops::ops_api::cluster",
    "source_kind": "internal",
    "retriable": false,
    "hint": "见 docs/requirements/SRS-STAR-OPS-001.md §3.1 落档清单 + 拍板 [M] 子项"
  }
}
```

### 6.4 前端错误处理 (per frontend advisory)

MVP 阶段前端 4 tab 占位, 无 error boundary (per 拍板 Q1)。`/ops/page.tsx` 仅显示静态 placeholder cards, 不调 API。[M] 阶段加 `useQuery` + retry + toast。

### 6.5 守門 #5 v2 (API key 安全, per 2026-09-02 09:01 JST)

- 真实 LLM 通道 API key 走 `star-credential` 加密存储 ([M] 阶段)
- 不进环境变量 (守門 #5 hard ban: 禁 `Get-ChildItem env:` / `echo $VAR`)
- 不走 UI 输入 (OpenAI/Anthropic 第三方 API 不暴露)

---

## §7 データ永続化設計 (Data Persistence, W/T/M 100% per 守門 #13)

### 7.1 6 张表 W/T/M 严格分类 (per OPS-BASIC-DESIGN §4 + SRS-001 §8)

| # | 表名 | 分类 | 派生规 | MVP 状态 | 实装阶段 |
|---|---|---|---|---|---|
| 1 | `ops_helm_release_state` | T | 物理删除禁止 + 監査必須 + RLS 13 類必携 | schema 未落 (MVP 内存) | [M] 子项 F-01 |
| 2 | `ops_cluster_action_log` | T | 同上 + WORM | 同上 | [M] 子项 F-01 |
| 3 | `ops_log_query_log` | T | 同上 | 同上 | [M] 子项 F-02 |
| 4 | `ops_log_entry` | W | 物理删除 + 短 TTL (7d) | 同上 | [M] 子项 F-02 |
| 5 | `ops_log_analysis` | W | 短 TTL (30d) | 同上 | [M] 子项 F-02 |
| 6 | `ops_metrics_config` | M | 物理删除禁止 + SCD Type 2 + RLS 13 類必携 | 同上 | [M] 子项 F-03 |

**覆盖率**: 6/6 = **100%** (per 守門 #13 100% 表覆盖硬约束)
**混合分类**: 0 张 (100% 干净)

### 7.2 RLS 13 類 (T/M 必携 per 守門 #13 派生规 (b)+(c))

T 类 (3 张) + M 类 (1 张) = 4 张必携 RLS 13 類:
- `tenant_id`, `user_id`, `role_id`, `permission_id`, `policy_id`
- `workspace_id`, `project_id`, `work_item_id`, `agent_id`
- `session_id`, `trace_id`, `source_module`, `source_kind`

W 类 (2 张) 仅 `retention_period` (per 守門 #13 派生规 (d))。

### 7.3 引用基线 (per 守門 #13)

- `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` v0.1
- `docs/data-design/ipa-detail/00-CLASSIFICATION-RULES.md` v0.1

### 7.4 物理 DB 落位 (per HANDOFF-ST-001 v0.8 §10)

实装阶段: 接 `star-dto` (per 7934131 commit 前已存在, 跨 multi-sub-session 桥接) + `infrastructure` crate (sqlx 0.8 + PostgreSQL)。MVP 阶段 schema 不落, 内存 + stub data。

### 7.5 retention 策略

| W 类表 | retention | 物理删除 | タイマー失効 |
|---|---|---|---|
| `ops_log_entry` | 7 天 | ✓ | ✓ (每天 cron) |
| `ops_log_analysis` | 30 天 | ✓ | ✓ |

T/M 类: 物理删除禁止, SCD Type 2 (`valid_from` + `valid_to` + `is_current` 字段)。

---

## §8 テスト設計 (Test Design)

### 8.1 4 层测试覆盖 (per 守門 #1 累积规 + LangGraph v0.2 §8 模式)

| 层 | 数量 | 文件 | 状态 |
|---|---|---|---|
| **UT (Unit Test)** | 9 | error.rs (5) + cluster.rs (2) + log.rs (2) + metrics.rs (1) + mock.rs (3, 含 1 重复字段 test) | ✅ 15/15 pass |
| **IT (Integration Test)** | 0 | (MVP 阶段无, [M] 阶段加 DB/跨 crate IT) | ⏳ 待 [M] |
| **E2E (End-to-End Test)** | 4 | ops_api.rs: cluster_list / metrics_summary / healthz / log_analysis (走 tower::oneshot) | ✅ 4/4 pass |
| **PT (Performance Test)** | 0 | (MVP 阶段无, [M] 阶段加 P95 < 100ms 守门) | ⏳ 待 [M] |

**总测试数**: 15, **通过率 100%** (per cargo test 0 fail 实证)。

### 8.2 UT 详细 (per §2.6)

| 测试名 | 位置 | 验证内容 |
|---|---|---|
| `not_implemented_returns_501` | error.rs:121 | OpsError::not_implemented → 6-field body (code, source_kind, hint) |
| `unauthorized_returns_401_with_policy_source` | error.rs:131 | OpsError::Unauthorized → source_kind=Policy |
| `rate_limited_is_retriable` | error.rs:139 | OpsError::RateLimited → retriable=true |
| `list_stub_returns_one_helm_release` | cluster.rs:85 | 1 条 stub, status=Healthy |
| `canary_request_validates_weight_range` | cluster.rs:92 | canary_weight 0-100 验证 (MVP 简化为 struct) |
| `log_entry_stub_has_error_level` | log.rs:96 | stub 1 条 ERROR log + trace_id |
| `log_analysis_stub_confidence_below_threshold` | log.rs:104 | **守門 #23**: mock confidence < 0.5 |
| `summary_stub_returns_five_kpis` | metrics.rs:73 | 5 KPI hardcoded |
| `mock_channel_always_enabled` | mock.rs:147 | MockChannel 永远 is_enabled=true |
| `mock_analyze_error_log_produces_anomaly` | mock.rs:155 | ERROR → 1 anomaly + 1 suggestion + 0.42 confidence |
| `mock_analyze_info_log_produces_no_anomaly` | mock.rs:170 | INFO → 0 anomaly |

### 8.3 E2E 详细 (per §2.5)

| 测试名 | 验证内容 |
|---|---|
| `cluster_list_returns_one_release` | GET /api/ops/cluster/releases → 200 |
| `metrics_summary_returns_five_kpis` | GET /api/ops/metrics/summary → 200 |
| `healthz_returns_200` | GET /healthz → 200 OK |
| `log_analysis_returns_stub` | GET /api/ops/log/analysis/{Uuid::nil()} → 200 |

**E2E 模式**: 走 `axum::Router::oneshot()` 模拟 HTTP 请求, 验证 status code + (后续 [M] 加 body parse)。

### 8.4 IT/PT 缺口 ([M] 阶段补)

- **IT**: DB 跨 crate IT (接 star-context + infrastructure + sqlx)
- **PT**: API P95 < 200ms (stub) → < 100ms (实装), cargo bench + criterion

### 8.5 subprocess 测试 (守門 #23 + 守門 #24 v2)

`python scripts/automation/ai_log_mock.py` 跑通 3ms 实证 (per commit `03d7d43` 验证日志):
- stdin 注入测试 log → 3 anomalies 正确抽取
- confidence 永远 0.42 < 0.5 (守門 #23 派生规)
- needs_review = true (mock 永远)

**注意**: subprocess 调用路径在 Rust 端 `#[allow(dead_code)]` 预留, 未实装 (per §3.5)。[M] 阶段启用, IT 验证 subprocess 3ms 实证 + JSON 解析 0 错。

### 8.6 守门回归 (per 守門 #1 累积规 v1-v25)

每 [M]/[S] 子项推进时必跑:
1. `cargo check -p star-ops --all-targets -j 4` 0 err
2. `cargo test -p star-ops --lib -j 4` 100% pass
3. `cargo fmt -p star-ops --check` 0 err
4. `cargo clippy -p star-ops --all-targets -j 4` 0 err (advisory)
5. `cargo check --workspace --all-targets -j 4` 0 err (per 守門 #1 v1)
6. `npx tsc --noEmit` 我改 4 文件 0 错 (per 守門 #6 v2 advisory)
7. `python scripts/automation/ai_log_mock.py` exit 0 (per 守門 #23)

---

## §9 既知の課題 (Known Issues) — 初版 v0.1

### 9.1 真实限制 (MVP 阶段, 守門 #11 缺标比错标)

| # | 課題 | 等级 | 触发条件 | 缓解 |
|---|---|---|---|---|
| 1 | 4 类功能 (F-01..F-04) 仅 stub | P1 | MVP 阶段默认 | 拍板 4 子项后逐个推进 (估 2.0M token) |
| 2 | Hybrid AI 通道 OpenAI/Anthropic stub 返 NOT_IMPLEMENTED | P1 | api_key 未配置 | [M] 子项 F-02 实装 reqwest + 真实 LLM 通道 |
| 3 | 真实 LLM API key 未配置, mock 兜底 | P2 | MVP 简化为 None | [M] 接 star-credential 加密存储 |
| 4 | K8s/Helm client 未引入 (kube-rs) | P1 | F-01 端到端需求 | [M] 子项 F-01 评估 + 引入 kube = "0.95" |
| 5 | PostgreSQL 持久化未实装 | P2 | MVP 内存 + stub | [M] 接 star-dto + sqlx (per HANDOFF-ST-001 §10) |
| 6 | 跨域编排 / 任务卡集成未实装 | P1 | 守門 #3 5 域 Lead 真人到位前 | 真人到位后追溯 |
| 7 | frontend `/ops/page.tsx` 4 tab 占位卡片无 API 调用 | P2 | MVP 简化为静态 | [M] 阶段加 useQuery + retry |
| 8 | 守門 #1 v20 (调试控制台不污染 main) — `ai_log_mock.py` 走 subprocess 但**不**通过 console_server.py (per 守門 #9 v3 实证) | P3 | MVP 简化 | [M] 阶段可选接 console_server.py 统一入口 |
| 9 | 5 域 Lead 真人未到位, Mavis 长期代签 | 中 | 持续 | 真人到位后追溯签字 (per 守門 #14 + 9/3 19:35 JST 拍板 D) |
| 10 | i18n 3 语言 MVP 简化版, 部分 key 缺 token | P3 | 拍板 Q1 范围 | [S] 子项校对 |

### 9.2 设计决策 (待 [M]/[S] 子项拍板重审)

| # | 决策 | 拍板状态 | 触发重审 |
|---|---|---|---|
| D-1 | Hybrid AI 通道顺序 L1=mock, L2=openai, L3=anthropic | 已拍板 (per Q2 Hybrid) | 5 域 Lead 真人到位后可调整 |
| D-2 | 端口 8090 (per star-mcp 8080/8081 顺延) | MVP 阶段拍板 | K8s deployment 阶段确认 |
| D-3 | 单 crate star-ops (47 → 48 package) | 已拍板 (per Q3) | 拆 star-ops-api + star-ops-domain + star-ops-ai [L] 子项评估 |
| D-4 | 跳详设 → MVP 后补 → 续实装迭代 | 已拍板 (per Q4) | 4 个 [M] 子项推进时逐子项补详设 |
| D-5 | Mavis 临时代签 5 域 Lead | 拍板 (per 9/3 11:35 JST 反转) | 真人到位后追溯签字覆盖 |

### 9.3 跨 session 续 (per AGENTS.md §4 #9 主体规则)

- 子代理 dispatch 必先 `automation/dispatcher.py brief(...)` 落 `docs/briefs/<task_id>.md` (per 守門 #9 v20)
- 必 `git log -p --follow <wt-branch>` 实证 worktree commit 在 main 链上
- 10 background task `net::ERR_CONNECTION_CLOSED` 但 status=succeeded 实证 (per AGENTS.md §4 #9), Mavis 接手 root session 一次性落地更安全

### 9.4 父 phase (`PHASE-OPS-INTRY-REPORT.md` v0.1) 链接

本详设是 `PHASE-OPS-INTRY-REPORT.md` §3 已知缺口 #1 / #2 / #4 / #5 / #6 / #7 / #10 的实现仕様来源, 子项推进时必先读本文件 §3 / §4 / §7 / §8 4 维。

---

## §10 签字栏 (per AGENTS.md §3 模板)

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 JST |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |

> 5 域 Lead 真人到位后追溯签字覆盖 (per 守門 #14 + 9/3 19:35 JST 拍板 D), 不沿用代签决策 (per 守門 #1 禁回溯)

---

## §11 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版詳細設計 (5 维 + Hybrid AI + W/T/M 6 表 + 4 层测试) | 2026-09-08 08:14 JST 用户发令 "制作详细设计" + 拍板 Q4 跳详设后续补 (per `PHASE-OPS-INTRY-REPORT.md` §10 拍板矩阵) |

---

## §12 引用文档 (References)

| 文档 | 用途 | 引用章节 |
|---|---|---|
| `docs/requirements/SRS-STAR-OPS-001.md` v0.1 | 需求定義書 (上游) | §4 F-01..F-04, §5 Hybrid AI, §8 W/T/M |
| `docs/basic-design/OPS-BASIC-DESIGN-001.md` v0.1 | 基本設計書 (上游) | §3 REST 契约, §5 Hybrid AI 4 級 Ladder, §6 部署 |
| `docs/reports/PHASE-OPS-INTRY-REPORT.md` v0.1 | MVP-骨架 落档报告 (父) | §3 已知缺口, §5 守门规则 |
| `docs/automation-design.md` v0.2 §4.16 | 任务卡表 (per 守門 #21 v21) | §9.1 真实限制 |
| `docs/architecture/2026-08-26-upgrade/adr/0026-star-ai-compat.md` | ADR-0026 STAR AI 兼容 + Fallback Ladder 4 級 | §3 Hybrid AI 详细 |
| `docs/architecture/2026-09-03-langgraph/03-detailed-design.md` v0.2 | LangGraph 详设模板 (12 节) | §1-§8 章节结构 |
| `docs/architecture/2026-09-03-agent-runtime/03-detailed-design.md` v0.1 | Agent Runtime 详设 | §7 5 表 schema W/T/M 模式 |
| `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` v0.1 | W/T/M 100 表索引 | §7 引用基线 |
| `docs/data-design/ipa-detail/00-CLASSIFICATION-RULES.md` v0.1 | 跨项目 ルール手册 | §7 引用基线 |
| `AGENTS.md` §4 + §4.1 累积规 v1-v26 | 守門 15-17 项 + 派生规 26 条 | §6 错误处理, §8 测试 |
| `scripts/automation/ai_log_mock.py` v0.1 | subprocess mock (3ms 跑通) | §3.5, §4.5, §8.5 |
| `crates/star-ops/src/ops_api.rs` (commit `03d7d43` + `7934131`) | 8 REST stub 实际代码 | §2.5, §2.6, §4 时序图 |
| `crates/star-ops/src/ops_ai/{mod,mock,openai_stub,anthropic_stub,ladder}.rs` | Hybrid AI 4 通道实装 | §3 详细 |
| `frontend/src/app/ops/page.tsx` (commit `03d7d43`) | 4 tab 骨架 | §1.3, §4.1, §5.4 |
| `frontend/src/lib/i18n/{dictionary,zh-CN,en,ja}.ts` | 3 语言 i18n | §1.3, §6.4 |
| `docs/reports/HANDOFF-ST-001.md` v0.8 §10 | 跨 multi-sub-session 桥接 + star-dto 引入 | §7.4 物理 DB 落位 |
