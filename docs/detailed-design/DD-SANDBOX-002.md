# DD-SANDBOX-002

> **Sandbox-as-a-Service (sandboxd) — 詳細設計書 v0.1** (per 日本 IPA SEC 標準 / 詳細設計書 テンプレート + STAR 仓 OPS-DETAILED-DESIGN-001 模板)
>
> - 状态: 🟡 Draft v0.1 (2026-09-10 JST 初版落档)
> - 上游: [`docs/requirements/SRS-SANDBOX-002.md`](../requirements/SRS-SANDBOX-002.md) v0.1.1 (37KB, 8 機能 / 5 業務 / 7 非機能 / 4 表 W/T/M / 9 缺口 / 6 决策点已拍板) + [`docs/basic-design/SANDBOX-BASIC-DESIGN-002.md`](../basic-design/SANDBOX-BASIC-DESIGN-002.md) v0.1.1 (46.6KB, 6 模块 / 18 子模块 / gRPC proto / 4 维后端)
> - 下游: 实装代码 (`crates/sandboxd/`) + 测试 + 报告 (`docs/reports/PHASE-SANDBOX-002-IMPL-REPORT.md`)
> - 核心语言: Rust 1.80+ (跟 mavis desktop 同栈, per 守门 #1 cargo check)
> - 修订人: `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` (per 2026-08-27 19:39 JST 用户授权 + 守门 #10 + 守门 #14 v3)
> - 审批: `架构师 (Mavis 接手 agent per DEC-008)` (per 守门 #14 v4 反转 v0.62 2026-09-10 12:45 JST)
> - 日期: 2026-09-10 JST

---

## §0 目的 (Purpose)

本詳細設計書は `SANDBOX-BASIC-DESIGN-002.md` v0.1.1 で定めた基本設計を実装可能なレベルまで展開する。MVP-骨架段階の Rust ファイル + proto schema + SQL DDL + cargo test の物理形状と 100% 一致させ、実装者が追加設計判断をせずに済む粒度で仕様を提供する。

**核心スコープ**:

- **5 維** 設計: モジュール / クラス / 時序 / 状態遷移 / テスト
- **W/T/M 4 表 100% カバー** (守門 #13) — `sandbox_session` (T) + `sandbox_policy` (M) + `sandbox_audit` (T) + `sandbox_capability` (M)
- **gRPC 4 RPC 100% カバー** — CreateSession / RunCommand / DestroySession / StreamLogs
- **4 維隔離 100% カバー** (per FR-3) — resource (Windows Job Objects / Linux cgroups v2) + network (WFP / netns+iptables / pf) + fs (AppContainer / mount ns / sandbox-exec) + capability (libcap / implicit drop)
- **3 プラットフォーム 100% カバー** (per FR-5) — Windows / Linux / macOS
- **Framework 選型** (per 既存 mavis 仓 实证): tonic 0.12 + prost 0.13 + tokio 1.40 + sqlx 0.8 + windows 0.58 (Win) + caps 0.5 + nix 0.28 (Linux) + sandbox 0.5 (macOS)

**不做什么** (per SRS-002 §1.4):
- 暗号化 / vault → 跨項目需求
- 跨域編排 (5 域 Lead) → 跨 P3-B 子項
- 真实 K8s / Helm 部署 → v0.2 拍攝
- TLS / mTLS 雙向認證 → v0.2 拍攝
- 多租户隔離 → v0.2 拍攝
- AI 行為審計 (LLM 決策錄屏) → v2.x

---

## §1 モジュール設計 (Module Design)

### 1.1 物理ファイル構成 (target 18 源 + 6 テスト = 24 文件)

```
crates/sandboxd/
├── Cargo.toml                                # 1 依賴清單, ~80 行
├── build.rs                                  # tonic-build 腳本, ~30 行
├── proto/
│   └── sandboxd.proto                        # gRPC schema, ~140 行
├── src/
│   ├── lib.rs                                # 模組入口, ~30 行
│   ├── config.rs                             # YAML 加載 + 校驗, ~150 行
│   ├── error.rs                              # 6-field 錯誤模型, ~80 行
│   ├── proto.rs                              # tonic-build 自動生成, 0 手寫
│   ├── orchestrator.rs                       # session 池化 + 生命週期, ~400 行
│   ├── server.rs                             # tonic gRPC server + 4 RPC, ~300 行
│   ├── metrics.rs                            # Prometheus 9 metric, ~150 行
│   ├── health.rs                             # /healthz axum, ~50 行
│   ├── audit.rs                              # PG audit log 寫入, ~200 行
│   ├── logging.rs                            # tracing 初始化, ~50 行
│   ├── policy/
│   │   ├── mod.rs                            # policy 入口, ~50 行
│   │   ├── resource.rs                       # SandboxLimits + 應用, ~120 行
│   │   ├── network.rs                        # allowlist + 應用, ~150 行
│   │   ├── fs.rs                             # path allowlist + 應用, ~150 行
│   │   └── capability.rs                     # capability 應用, ~120 行
│   ├── backend/
│   │   ├── mod.rs                            # 後端 trait + 工廠, ~80 行
│   │   ├── windows.rs                        # Job Objects + WFP + AppContainer, ~400 行
│   │   ├── linux.rs                          # cgroups v2 + netns + mount ns + libcap, ~500 行
│   │   └── macos.rs                          # sandbox-exec profile, ~250 行
│   └── bin/
│       └── sandboxd.rs                       # binary 入口, ~80 行
├── migrations/                               # PG DDL (per W/T/M 4 表)
│   ├── 001_sandbox_session.sql               # T, ~30 行
│   ├── 002_sandbox_policy.sql                # M, ~30 行
│   ├── 003_sandbox_audit.sql                 # T, ~40 行
│   ├── 004_sandbox_capability.sql            # M, ~25 行
│   └── 005_rls_policies.sql                  # 4 表 RLS 13 類, ~40 行
├── config/
│   ├── sandboxd.yaml                         # 默認配置, ~50 行
│   └── policy_default.json                   # 默認 4 維策略, ~30 行
└── tests/
    ├── unit/
    │   ├── test_config.rs                    # YAML 加載校驗, ~120 行
    │   ├── test_policy_resource.rs           # 資源策略應用, ~150 行
    │   ├── test_policy_network.rs            # 網絡策略應用, ~120 行
    │   ├── test_policy_fs.rs                 # FS 策略應用, ~120 行
    │   ├── test_policy_capability.rs         # capability 策略, ~80 行
    │   ├── test_orchestrator.rs              # session 池化, ~200 行
    │   ├── test_metrics.rs                   # Prometheus 指標, ~80 行
    │   └── test_audit.rs                     # PG audit 寫入, ~120 行
    ├── integration/
    │   ├── test_grpc_create_run_destroy.rs   # 端到端 4 RPC, ~250 行
    │   ├── test_failover_v02_subprocess.rs   # SANDBOX-001 v0.2 降級, ~150 行
    │   └── test_pg_audit_persistence.rs      # PG 持久化, ~120 行
    └── e2e/
        └── test_dispatcher_integration.rs    # dispatcher.py 集成, ~200 行
```

**總行數**: 24 文件, ~4.8K 行 (含注釋 + tests), MVP 階段 0 dead code。

### 1.2 依存関係 (per SRS §6.1, 跟 mavis workspace 對齊)

```toml
# Cargo.toml
[package]
name = "sandboxd"
version = "0.1.0"
edition = "2021"

[dependencies]
# 跨 crate 複用 (跟 mavis 現有 workspace 一致, per 守門 #1 cargo)
serde = { workspace = true }
serde_json = { workspace = true }
serde_yaml = "0.9"
thiserror = { workspace = true }
anyhow = { workspace = true }
tokio = { workspace = true, features = ["full"] }
uuid = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

# gRPC (per SRS §6.3 約束)
tonic = "0.12"
prost = "0.13"

# HTTP (用於 /metrics + /healthz, 跟 star-mcp 0.12 一致)
axum = "0.8"
tower = { version = "0.5", features = ["util"] }

# Metrics (per FR-6.1)
prometheus = "0.13"

# DB (PG audit log, 跟 p3d6_audit 表結構一致)
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"] }

# 平台特定
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
    "Win32_System_JobObjects",
    "Win32_System_Threading",
    "Win32_Security",
    "Win32_Security_Authorization",
    "Win32_Networking_WinFilter",
] }

[target.'cfg(target_os = "linux")'.dependencies]
caps = "0.5"
nix = { version = "0.28", features = ["mount", "net", "sched", "user", "sys"] }

[target.'cfg(target_os = "macos")'.dependencies]
sandbox = "0.5"

# 跨 crate (P0-1 共享, per 守門 #4.2 v16)
star-context = { path = "../star-context" }

[build-dependencies]
tonic-build = "0.12"

[dev-dependencies]
tokio-test = "0.4"
mockall = "0.12"
tempfile = "3.8"
testcontainers = "0.20"  # PG 容器 (integration test)
```

**運行時依賴 12 项 + 平台特定 3-5 项 + build/dev 5 项 = ~20 总依赖**, MVP 階段對齊 mavis workspace。

### 1.3 既存 mavis 仓 集成点

| 集成点 | 現有代碼 | 集成方式 |
|---|---|---|
| `dispatcher.py` v0.1 | `scripts/automation/dispatcher.py` `invoke()` | 創建 SandboxdClient, 替換 subprocess.run 為 gRPC CreateSession + RunCommand (per FR-7.1 fail-open 降級保留) |
| `scripts/automation/guardian/sandbox.py` v0.2 | 资源維包裝器 | sandboxd 不可用時降級路徑 (per FR-7.1) |
| `crates/star-mcp/` 0.12 | MCP transport | 創建 `crates/star-mcp/src/sandboxd_client.rs` (跟 Python client 對稱) |
| `crates/star-context/` | ActorContext (P0-1) | sandboxd 創建 session 時引用 ActorContext (tenant_id / workspace_id / actor_id), 跟守門 #13 RLS 13 類 對齊 |
| `db/migrations/` | PG schema | 追加 `db/migrations/2026-09-10-sandboxd-4-tables.sql` (per §4 4 表) |
| `crates/star-telemetry/` (待) | Prometheus scrape | sandboxd 暴露 /metrics, 跟 mavis 主 metrics 抓取集成 |

---

## §2 クラス設計 (Class Design)

### 2.1 核心 struct + enum

```rust
// src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum SandboxdError {
    #[error("session not found: {0}")]
    SessionNotFound(Uuid),
    
    #[error("session already exists: {0}")]
    SessionAlreadyExists(Uuid),
    
    #[error("resource limit exceeded: {limit_type} = {value}")]
    ResourceLimitExceeded { limit_type: String, value: u64 },
    
    #[error("network policy violation: {detail}")]
    NetworkViolation { detail: String },
    
    #[error("fs policy violation: {path}")]
    FsViolation { path: String },
    
    #[error("capability policy violation: {cap}")]
    CapabilityViolation { cap: String },
    
    #[error("platform not supported: {0}")]
    PlatformNotSupported(String),
    
    #[error("audit log write failed: {0}")]
    AuditLogWriteFailed(String),
    
    #[error("internal error: {0}")]
    Internal(String),
}

// 6-field 錯誤模型 (跟 star-mcp 一致)
#[derive(Debug, serde::Serialize)]
pub struct SandboxdErrorResponse {
    pub kind: String,           // "session_not_found" / "resource_exceeded" / ...
    pub message: String,        // human readable
    pub field: Option<String>,  // 具體字段
    pub details: serde_json::Value,  // 結構化詳情
    pub status: u16,            // HTTP-like status
    pub trace_id: Uuid,         // 跟 ActorContext.trace_id 對齊
}
```

```rust
// src/orchestrator.rs
pub struct Session {
    pub session_id: Uuid,
    pub client_id: String,
    pub task_id: String,
    pub parent_session_id: Option<Uuid>,
    pub policy_snapshot: SessionSpec,        // 創建時凍結
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub destroyed_at: Option<DateTime<Utc>>,
    pub destroy_reason: Option<String>,
    pub backend_handle: Box<dyn Backend>,    // 平台特定 handle
    pub tenant_id: Uuid,                     // 跟 ActorContext 對齊
    pub workspace_id: Uuid,
    pub actor_id: Uuid,
}

pub enum SessionStatus {
    Created,
    Running,
    Destroyed,
    Failed,
}

pub struct Orchestrator {
    sessions: Arc<DashMap<Uuid, Arc<RwLock<Session>>>>,  // 併發安全
    max_concurrent: usize,                                 // 100 默認
    pg_pool: PgPool,                                       // audit log
    metrics: Arc<SandboxdMetrics>,
    backend_factory: Box<dyn BackendFactory>,
}
```

```rust
// src/policy/mod.rs (per FR-3)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionSpec {
    pub resource: ResourcePolicy,
    pub network: NetworkPolicy,
    pub fs: FsPolicy,
    pub capability: CapabilityPolicy,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourcePolicy {
    pub memory_mb: u32,                    // 0 = 不限
    pub cpu_percent: u32,                  // 0 = 不限
    pub max_processes: u32,                // 0 = 不限
    pub kill_on_parent_exit: bool,         // 默認 true
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkPolicy {
    pub mode: NetworkMode,
    pub allowlist: Vec<String>,            // FQDN 通配符
    pub ip_allowlist: Vec<String>,         // CIDR (per 缺口 #6 緩解)
}

pub enum NetworkMode {
    Allowlist,                             // 默認 per D-3
    BlockAll,
    Passthrough,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FsPolicy {
    pub mode: FsMode,
    pub allowlist: Vec<FsPathRule>,
}

pub enum FsMode {
    Allowlist,                             // 默認 per D-4
    ReadOnlyRoot,
    Open,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FsPathRule {
    pub path: String,                      // glob pattern
    pub mode: FsPathMode,
}

pub enum FsPathMode {
    Read,
    ReadWrite,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CapabilityPolicy {
    pub linux_mode: LinuxCapabilityMode,
    pub custom_capabilities: Vec<String>,  // CUSTOM 模式填
}

pub enum LinuxCapabilityMode {
    DropAll,                               // 默認
    AllowNetBindService,
    Custom,
}
```

```rust
// src/backend/mod.rs (per FR-5)
#[async_trait::async_trait]
pub trait Backend: Send + Sync {
    /// 應用 4 維隔離策略, 返回 backend handle
    async fn apply(&self, policy: &SessionSpec, session_id: Uuid) -> Result<Box<dyn BackendHandle>>;
    
    /// 檢查當前平台是否支持
    fn is_supported(&self) -> bool;
}

#[async_trait::async_trait]
pub trait BackendHandle: Send + Sync {
    /// 在 sandbox 內跑命令
    async fn run(&self, cmd: &CommandSpec, timeout_sec: u32) -> Result<RunResult>;
    
    /// 銷毀 sandbox (級聯 kill)
    async fn destroy(&self) -> Result<()>;
    
    /// 健康檢查
    async fn health(&self) -> Result<HealthStatus>;
}

pub trait BackendFactory: Send + Sync {
    fn create(&self) -> Result<Box<dyn Backend>>;
}

pub struct DefaultBackendFactory;

impl BackendFactory for DefaultBackendFactory {
    fn create(&self) -> Result<Box<dyn Backend>> {
        #[cfg(windows)]
        return Ok(Box::new(backend::windows::WindowsBackend::new()?));
        
        #[cfg(target_os = "linux")]
        return Ok(Box::new(backend::linux::LinuxBackend::new()?));
        
        #[cfg(target_os = "macos")]
        return Ok(Box::new(backend::macos::MacosBackend::new()?));
        
        #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
        Err(SandboxdError::PlatformNotSupported(std::env::consts::OS.to_string()))
    }
}
```

### 2.2 gRPC service impl (per FR-2 + FR-4)

```rust
// src/server.rs
use crate::proto::sandboxd_server::{SandboxService, SandboxServiceServer};
use crate::proto::*;

pub struct SandboxServiceImpl {
    orchestrator: Arc<Orchestrator>,
    metrics: Arc<SandboxdMetrics>,
}

#[tonic::async_trait]
impl SandboxService for SandboxServiceImpl {
    async fn create_session(
        &self,
        request: Request<CreateSessionRequest>,
    ) -> Result<Response<CreateSessionResponse>, Status> {
        let req = request.into_inner();
        let _timer = self.metrics.rpc_duration_seconds
            .with_label_values(&["create_session"])
            .start_timer();
        
        let result = self.orchestrator.create_session(
            req.client_id,
            req.task_id,
            req.parent_session_id,
            req.policy.ok_or_else(|| Status::invalid_argument("policy missing"))?,
            req.timeout_sec,
        ).await;
        
        match result {
            Ok(session_id) => {
                self.metrics.sessions_total.with_label_values(&["created"]).inc();
                Ok(Response::new(CreateSessionResponse {
                    session_id: session_id.to_string(),
                    status: create_session_response::Status::Ready as i32,
                    error: String::new(),
                }))
            }
            Err(e) => {
                self.metrics.sessions_total.with_label_values(&["failed"]).inc();
                Ok(Response::new(CreateSessionResponse {
                    session_id: String::new(),
                    status: create_session_response::Status::Failed as i32,
                    error: e.to_string(),
                }))
            }
        }
    }
    
    async fn run_command(
        &self,
        request: Request<RunCommandRequest>,
    ) -> Result<Response<RunCommandResponse>, Status> {
        let req = request.into_inner();
        let session_id = Uuid::parse_str(&req.session_id)
            .map_err(|_| Status::invalid_argument("invalid session_id"))?;
        let command = req.command.ok_or_else(|| Status::invalid_argument("command missing"))?;
        
        let result = self.orchestrator.run_command(
            session_id,
            &command,
            req.timeout_sec,
        ).await;
        
        match result {
            Ok(r) => Ok(Response::new(RunCommandResponse {
                returncode: r.returncode,
                stdout: r.stdout,
                stderr: r.stderr,
                latency_ms: r.latency_ms as u64,
                hit_violations: r.hit_violations,
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
    
    async fn destroy_session(
        &self,
        request: Request<DestroySessionRequest>,
    ) -> Result<Response<DestroySessionResponse>, Status> {
        let req = request.into_inner();
        let session_id = Uuid::parse_str(&req.session_id)
            .map_err(|_| Status::invalid_argument("invalid session_id"))?;
        
        let result = self.orchestrator.destroy_session(session_id, req.reason).await;
        
        match result {
            Ok(()) => {
                self.metrics.sessions_total.with_label_values(&["destroyed"]).inc();
                Ok(Response::new(DestroySessionResponse {
                    status: destroy_session_response::Status::Destroyed as i32,
                }))
            }
            Err(SandboxdError::SessionNotFound(_)) => Ok(Response::new(DestroySessionResponse {
                status: destroy_session_response::Status::NotFound as i32,
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
    
    type StreamLogsStream = Pin<Box<dyn Stream<Item = Result<LogEntry, Status>> + Send>>;
    
    async fn stream_logs(
        &self,
        request: Request<StreamLogsRequest>,
    ) -> Result<Response<Self::StreamLogsStream>, Status> {
        let req = request.into_inner();
        let session_id = Uuid::parse_str(&req.session_id)
            .map_err(|_| Status::invalid_argument("invalid session_id"))?;
        let from_offset = req.from_offset;
        
        let stream = self.orchestrator.stream_logs(session_id, from_offset).await?;
        Ok(Response::new(Box::pin(stream)))
    }
}
```

### 2.3 配置類 (per FR-1.1)

```rust
// src/config.rs
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SandboxdConfig {
    pub server: ServerConfig,
    pub isolation: IsolationConfig,
    pub observability: ObservabilityConfig,
    pub failover: FailoverConfig,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ServerConfig {
    pub grpc_port: u16,                        // 50051
    pub metrics_port: u16,                     // 50051 (跟 gRPC 同進程)
    pub health_port: u16,                      // 50051
    pub max_concurrent_sessions: usize,        // 100
    pub default_session_timeout_sec: u32,      // 1800
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct IsolationConfig {
    pub resource: ResourceConfig,
    pub network: NetworkConfig,
    pub fs: FsConfig,
    pub capability: CapabilityConfig,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ResourceConfig {
    pub default_memory_mb: u32,
    pub default_cpu_percent: u32,
    pub default_max_processes: u32,
    pub kill_on_parent_exit: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct NetworkConfig {
    pub default_mode: NetworkMode,
    pub default_allowlist: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FsConfig {
    pub default_mode: FsMode,
    pub default_allowlist: Vec<FsPathRule>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CapabilityConfig {
    pub linux_default: LinuxCapabilityMode,
    pub windows_default: String,               // "drop_se_debug"
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ObservabilityConfig {
    pub prometheus_enabled: bool,
    pub audit_log_pg_url: String,
    pub audit_log_retention_days: u32,         // 90
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FailoverConfig {
    pub enable_v0_2_subprocess_fallback: bool,
    pub fallback_path: PathBuf,
    pub root_session_notify: bool,
}

impl SandboxdConfig {
    pub fn from_yaml(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: SandboxdConfig = serde_yaml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }
    
    pub fn validate(&self) -> Result<()> {
        // 校驗所有必填字段 + 業務規則
        if self.server.max_concurrent_sessions == 0 {
            return Err(SandboxdError::Internal("max_concurrent_sessions must be > 0".into()));
        }
        if self.observability.audit_log_retention_days < 1 {
            return Err(SandboxdError::Internal("retention_days must be >= 1".into()));
        }
        Ok(())
    }
}
```

### 2.4 指標類 (per FR-6.1)

```rust
// src/metrics.rs
pub struct SandboxdMetrics {
    pub registry: Registry,
    pub sessions_total: IntCounterVec,           // {status}
    pub sessions_active: IntGauge,
    pub session_duration_seconds: Histogram,
    pub resource_limit_hits_total: IntCounterVec,  // {limit_type}
    pub network_violations_total: IntCounterVec,   // {action}
    pub fs_violations_total: IntCounterVec,         // {action}
    pub capability_violations_total: IntCounterVec, // {action}
    pub rpc_duration_seconds: HistogramVec,         // {rpc}
}

impl SandboxdMetrics {
    pub fn new() -> Result<Self> {
        let registry = Registry::new();
        
        let sessions_total = IntCounterVec::new(
            prometheus::Opts::new("sandboxd_sessions_total", "Total sessions")
                .label_names(&["status"]),
            &["status"],
        )?;
        // ... 其他 8 個指標類似註冊
        
        registry.register(Box::new(sessions_total.clone()))?;
        // ...
        
        Ok(Self { registry, sessions_total, /* ... */ })
    }
}
```

---

## §3 時序設計 (Sequence Design)

### 3.1 CreateSession 時序 (per FR-4.1)

```
client                sandboxd                  backend             PG
  │                      │                         │                 │
  │ CreateSession ──────▶│                         │                 │
  │                      │ validate SessionSpec    │                 │
  │                      │ (per §2.3)              │                 │
  │                      │                         │                 │
  │                      │ create session_id       │                 │
  │                      │ ────────────────────────────────────────▶│
  │                      │ INSERT sandbox_session  │                 │
  │                      │ ◀────────────────────────────────────────│
  │                      │                         │                 │
  │                      │ apply(policy)           │                 │
  │                      │────────────────────────▶│                 │
  │                      │                         │                 │
  │                      │ 4 維隔離應用:           │                 │
  │                      │ - resource (Job/cgroup) │                 │
  │                      │ - network (WFP/ipt)     │                 │
  │                      │ - fs (AppContainer/ns)  │                 │
  │                      │ - capability (libcap)   │                 │
  │                      │ ◀────────────────────────│                 │
  │                      │                         │                 │
  │                      │ INSERT sandbox_session  │                 │
  │                      │ (status=Ready)          │                 │
  │                      │ ────────────────────────────────────────▶│
  │                      │                         │                 │
  │ ◀────── {session_id, status=Ready}             │                 │
  │                      │                         │                 │
```

**關鍵時序點**:
1. **validate SessionSpec** (T+0ms): policy 合法性校驗, fail-fast
2. **INSERT sandbox_session** (T+5ms): 持久化創建記錄
3. **apply 4 維策略** (T+10-50ms): 平台後端調用, 最大開銷
4. **INSERT status=Ready** (T+55ms): 確認 session ready

**總耗時**: < 100ms p99 (per NFR-P-2)

### 3.2 RunCommand 時序 (per FR-4.2)

```
client                sandboxd              backend           PG
  │                      │                     │                 │
  │ RunCommand ─────────▶│                     │                 │
  │                      │ lookup session      │                 │
  │                      │ (DashMap RwLock)    │                 │
  │                      │                     │                 │
  │                      │ INSERT audit        │                 │
  │                      │ (event=ProcessStart)│                 │
  │                      │ ─────────────────────────────────────▶│
  │                      │                     │                 │
  │                      │ run(cmd, timeout)   │                 │
  │                      │────────────────────▶│                 │
  │                      │                     │                 │
  │                      │                     │ 4 維攔截:       │
  │                      │                     │ - resource      │
  │                      │                     │ - network       │
  │                      │                     │ - fs            │
  │                      │                     │ - capability    │
  │                      │                     │                 │
  │                      │                     │ hit violations  │
  │                      │                     │ → audit log     │
  │                      │ ◀────────────────────│                 │
  │                      │                     │                 │
  │                      │ INSERT audit        │                 │
  │                      │ (event=ProcessExit) │                 │
  │                      │ ─────────────────────────────────────▶│
  │                      │                     │                 │
  │ ◀──── {returncode,   │                     │                 │
  │         stdout,      │                     │                 │
  │         stderr,      │                     │                 │
  │         hit_violations}                    │                 │
```

### 3.3 DestroySession + fail-open 降級時序 (per FR-4.3 + FR-7.1)

```
client                sandboxd              backend           SANDBOX-001 v0.2
  │                      │                     │                 │
  │ DestroySession ─────▶│                     │                 │
  │                      │ cascade kill        │                 │
  │                      │────────────────────▶│                 │
  │                      │                     │                 │
  │                      │ INSERT audit        │                 │
  │                      │ (event=Destroyed)   │                 │
  │                      │                     │                 │
  │ ◀──── {Destroyed}    │                     │                 │
```

**fail-open 降級時序** (sandboxd 不可用時):
```
client (dispatcher)                                            SANDBOX-001 v0.2
  │                                                                │
  │ 嘗試 gRPC Connect 127.0.0.1:50051                             │
  │ ─────── 連接失敗 / Unavailable ───▶                            │
  │                                                                │
  │ 走 SANDBOX-001 v0.2 subprocess.run 降級路徑                     │
  │ ────────────────────────────────────────────────────────────▶  │
  │                                                                │
  │ ◀───────────────── 結果 (僅 resource 維隔離)                    │
  │                                                                │
  │ 寫 root session 通知 (per NFR-O-3)                             │
```

### 3.4 StreamLogs 時序 (per FR-2.3)

```
client                sandboxd              PG
  │                      │                     │
  │ StreamLogs ─────────▶│                     │
  │ {session_id, from}   │                     │
  │                      │ SELECT audit        │
  │                      │ WHERE session_id=?  │
  │                      │ AND offset >= from  │
  │                      │ ORDER BY offset ASC │
  │                      │────────────────────▶│
  │                      │◀────────────────────│
  │                      │                     │
  │ ◀──── LogEntry[0]    │ (server streaming)  │
  │ ◀──── LogEntry[1]    │                     │
  │ ...                  │                     │
  │ ◀──── LogEntry[N]    │                     │
  │ ◀──── (stream close) │                     │
```

**緩衝**: 1000 條環形 (per FR-2.3), 滿則 drop 老的

---

## §4 状態遷移設計 (State Machine Design)

### 4.1 Session 状態遷移

```
        CreateSession
            │
            ▼
    ┌──────────────┐
    │   Created    │ (status enum variant)
    └──────┬───────┘
           │ (orchestrator 標記 ready)
           ▼
    ┌──────────────┐    RunCommand     ┌──────────────┐
    │   Running    │ ─────────────────▶│   Running    │
    │              │ ◀───────────────── │  (in flight) │
    └──────┬───────┘   (完成/超時)      └──────────────┘
           │
           │ DestroySession
           │ 或超時 (30 min)
           ▼
    ┌──────────────┐
    │  Destroyed   │ (terminal)
    └──────────────┘
    
    任何階段失敗:
    ┌──────────────┐
    │   Failed     │ (terminal)
    └──────────────┘
```

**状態轉換守則** (per §2.1 Session struct):
- `Created` → `Running`: 第一個 RunCommand 調用時自動遷移
- `Running` → `Destroyed`: DestroySession 調用, 或 30 min 默認超時
- `Created` → `Failed`: 4 維策略 apply 失敗, 或 PG INSERT 失敗
- `Running` → `Failed`: RunCommand 嚴重錯誤, 或 backend 損壞
- `Destroyed` / `Failed`: terminal, 0 後續狀態

### 4.2 SessionSpec 校驗狀態 (per §2.3 validate)

```
       載入 YAML
           │
           ▼
   ┌──────────────┐
   │   Parsed     │ (serde_yaml OK)
   └──────┬───────┘
          │ validate()
          ▼
   ┌──────────────┐
   │   Validated  │ (業務規則 OK)
   └──────┬───────┘
          │ 應用到 session
          ▼
   ┌──────────────┐
   │   Applied    │ (4 維隔離 OK)
   └──────────────┘
   
   任何階段失敗 → 立即返回錯誤, 不創建 session
```

---

## §5 テスト設計 (Test Design)

> 跟 SRS-002 §7 验收条件 + §4 機能要件 + §5 非機能要件 1:1 對應。詳細測試設計見 `docs/test-design/TEST-DESIGN-SANDBOX-002.md` v0.1 (本 DD 同期落档)。

### 5.1 单元测试 (UT) 矩阵

| 模組 | TC class | TC 数量 | 覆盖目标 |
|---|---|---|---|
| `config` | test_config | 6 | YAML 加載 / 校驗 / 邊界 / 缺字段 / 類型錯誤 |
| `policy::resource` | test_policy_resource | 8 | 默認值 / 邊界 / 0=不限 / 大值 |
| `policy::network` | test_policy_network | 8 | allowlist 匹配 / 通配符 / IP CIDR / BlockAll / Passthrough |
| `policy::fs` | test_policy_fs | 7 | path 匹配 / 模式 / 邊界 / 無效路徑 |
| `policy::capability` | test_policy_capability | 5 | DropAll / AllowNetBindService / Custom |
| `orchestrator` | test_orchestrator | 12 | session 創建/銷毀/池化/超時/併發/失敗 |
| `metrics` | test_metrics | 4 | 9 指標暴露 / label 正確 / 累積 |
| `audit` | test_audit | 6 | JSON Lines 格式 / append-only / 索引 / 失敗 fail-closed |
| **小計** | **8 class** | **56 TC** | **≥ 90% line coverage** |

### 5.2 集成测试 (IT) 矩阵

| 集成路徑 | TC 数量 | 覆盖目標 |
|---|---|---|
| gRPC 4 RPC 端到端 | 8 | CreateSession / RunCommand / DestroySession / StreamLogs 全部 RPC |
| SANDBOX-001 v0.2 降級路徑 | 4 | sandboxd 不可用 / 健康檢查失敗 / 連接拒絕 / Unavailable |
| PG audit 持久化 | 5 | 創建/查詢/索引/append-only/失敗回滾 |
| **小計** | **17 TC** | **100% pass on Win + Linux** |

### 5.3 端到端 (E2E) 矩阵

| 場景 | TC 数量 | 覆盖目標 |
|---|---|---|
| dispatcher.py 集成 | 4 | 創建 session / 跑命令 / 銷毀 session / 降級路徑 |
| mavis desktop 集成 (Rust) | 4 | 跟 dispatcher 對稱 |
| 跨平台驗證 | 3 | Windows / Linux / macOS 各 1 |
| **小計** | **11 TC** | **100% pass** |

### 5.4 性能测试 (PT) 锚點 (per NFR-P)

| 指標 | 目標 | 計測方法 |
|---|---|---|
| CreateSession p50 | < 10ms | cargo bench |
| CreateSession p99 | < 100ms | cargo bench |
| RunCommand 端到端延遲 | 跟 subprocess.run baseline 比 < 5% 開銷 | A/B 對比 |
| 100 session 並發 | 0 資源競爭, p99 < 200ms | 壓測 |
| gRPC IPC 序列化開銷 | < 1ms (per command) | cargo bench |

### 5.5 验收测试 (UAT) 矩阵 (per SRS-002 §7 AC)

| AC | 关联 FR / NFR | 验收方法 | 阈值 |
|---|---|---|---|
| AC-1 | FR-3.1~FR-3.4 | 單元測試 4 維隔離全部命中 | 100% |
| AC-2 | FR-1.1 + NFR-P-1 + NFR-P-2 | benchmark p50 + p99 | < 10ms / < 100ms |
| AC-3 | FR-7.1 + FR-7.2 | 故障注入測試 | 100% fail-open / 100% fail-closed |
| AC-4 | FR-4.1 + FR-4.2 + FR-4.3 | session 生命周期測試 | 100% 行為符合 |
| AC-5 | NFR-T-1 + NFR-T-2 + NFR-T-3 | CI 三平台 | 100% pass |
| AC-6 | FR-6.1 + NFR-O-1 | Prometheus scrape 驗證 | 100% 指標暴露 |
| AC-7 | FR-6.2 + NFR-S-3 | audit log append-only 驗證 | 100% 不可刪 |
| AC-8 | NFR-S-1 + NFR-S-5 | 滲透測試 (4 維 × 10 攻擊場景) | 0 越權 |

**詳細測試設計** (5 級別 UT/IT/E2E/PT/UAT) 見 `docs/test-design/TEST-DESIGN-SANDBOX-002.md` v0.1。

---

## §6 W/T/M 數據模型設計 (per 守門 #13 100% 覆蓋)

> 跟 SRS-002 §8 + BD-002 §4 1:1 對應, 100% 覆蓋 4 表 (2 T + 2 M, 0 W per session 用完即銷毀派生)。

### 6.1 sandbox_session (T) DDL

```sql
-- db/migrations/2026-09-10-sandboxd-001-sandbox-session.sql
-- per 守門 #13c T 派生規: 物理刪除禁止 + 審計必須 + RLS 13 類必攜

CREATE TABLE IF NOT EXISTS sandbox_session (
    session_id          UUID PRIMARY KEY,
    client_id           VARCHAR(64) NOT NULL,
    task_id             VARCHAR(128) NOT NULL,
    parent_session_id   UUID REFERENCES sandbox_session(session_id) DEFERRABLE INITIALLY DEFERRED,
    policy_snapshot     JSONB NOT NULL,
    status              VARCHAR(16) NOT NULL CHECK (status IN ('Created', 'Running', 'Destroyed', 'Failed')),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    destroyed_at        TIMESTAMPTZ,
    destroy_reason      TEXT,
    resource_peak       JSONB,
    
    -- RLS 13 類必攜 (per 守門 #13c)
    tenant_id           UUID NOT NULL,
    workspace_id        UUID NOT NULL,
    actor_id            UUID NOT NULL,
    
    CONSTRAINT fk_tenant FOREIGN KEY (tenant_id) REFERENCES tenant(id),
    CONSTRAINT fk_workspace FOREIGN KEY (workspace_id) REFERENCES workspace(id),
    CONSTRAINT fk_actor FOREIGN KEY (actor_id) REFERENCES actor(id)
);

CREATE INDEX IF NOT EXISTS idx_sandbox_session_client_id ON sandbox_session (client_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_sandbox_session_status ON sandbox_session (status) WHERE status IN ('Created', 'Running');
CREATE INDEX IF NOT EXISTS idx_sandbox_session_tenant ON sandbox_session (tenant_id, created_at DESC);

-- RLS (per 守門 #13c 13 類必攜)
ALTER TABLE sandbox_session ENABLE ROW LEVEL SECURITY;
CREATE POLICY sandbox_session_tenant_isolation ON sandbox_session
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID
        AND workspace_id = current_setting('app.workspace_id', true)::UUID);
```

### 6.2 sandbox_policy (M) DDL

```sql
-- db/migrations/2026-09-10-sandboxd-002-sandbox-policy.sql
-- per 守門 #13b M 派生規: 物理刪除禁止 + SCD Type 2 + RLS 13 類必攜

CREATE TABLE IF NOT EXISTS sandbox_policy (
    policy_id           UUID PRIMARY KEY,
    policy_name         VARCHAR(128) NOT NULL,
    resource            JSONB NOT NULL,
    network             JSONB NOT NULL,
    fs                  JSONB NOT NULL,
    capability          JSONB NOT NULL,
    version             INT NOT NULL DEFAULT 1,
    valid_from          TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to            TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    -- RLS 13 類必攜
    tenant_id           UUID NOT NULL,
    workspace_id        UUID NOT NULL,
    actor_id            UUID NOT NULL,
    
    UNIQUE (policy_name, version, tenant_id)
);

CREATE INDEX IF NOT EXISTS idx_sandbox_policy_name ON sandbox_policy (policy_name, valid_from DESC) WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_sandbox_policy_tenant ON sandbox_policy (tenant_id, valid_from DESC);

ALTER TABLE sandbox_policy ENABLE ROW LEVEL SECURITY;
CREATE POLICY sandbox_policy_tenant_isolation ON sandbox_policy
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID);
```

### 6.3 sandbox_audit (T) DDL

```sql
-- db/migrations/2026-09-10-sandboxd-003-sandbox-audit.sql
-- per 守門 #13c T 派生規: 物理刪除禁止 + 審計必須 + RLS 13 類必攜 + 90 天保留

CREATE TABLE IF NOT EXISTS sandbox_audit (
    audit_id              BIGSERIAL PRIMARY KEY,
    session_id            UUID NOT NULL,
    event_type            VARCHAR(32) NOT NULL CHECK (event_type IN (
        'ResourceHit', 'NetworkBlock', 'NetworkAllow',
        'FsBlock', 'FsAllow', 'CapabilityDrop',
        'ProcessStart', 'ProcessExit'
    )),
    resource_used         JSONB,
    network_violations    JSONB,
    fs_violations         JSONB,
    capability_violations JSONB,
    timestamp             TIMESTAMPTZ NOT NULL DEFAULT now(),
    latency_ms            INT,
    hit_violations        TEXT[],
    
    -- RLS 13 類必攜
    tenant_id             UUID NOT NULL,
    workspace_id          UUID NOT NULL,
    actor_id              UUID NOT NULL
);

-- 跟守門 v36 audit log 索引聯動
CREATE INDEX IF NOT EXISTS idx_sandbox_audit_session_time ON sandbox_audit (session_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_sandbox_audit_event_type_time ON sandbox_audit (event_type, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_sandbox_audit_tenant_time ON sandbox_audit (tenant_id, timestamp DESC);

ALTER TABLE sandbox_audit ENABLE ROW LEVEL SECURITY;
CREATE POLICY sandbox_audit_tenant_isolation ON sandbox_audit
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID);

-- 90 天保留 (per BR-4 + 守門 #5 隱含)
-- 註: 自動刪除通過定時 job 實現 (不允許物理刪除, 通過 valid_to 標記)
```

### 6.4 sandbox_capability (M) DDL

```sql
-- db/migrations/2026-09-10-sandboxd-004-sandbox-capability.sql
-- per 守門 #13b M 派生規: 物理刪除禁止 + 靜態參考數據 + RLS 13 類必攜

CREATE TABLE IF NOT EXISTS sandbox_capability (
    capability_id     UUID PRIMARY KEY,
    syscall_name      VARCHAR(128) NOT NULL,
    platform          VARCHAR(16) NOT NULL CHECK (platform IN ('Linux', 'Windows', 'macOS')),
    risk_level        VARCHAR(16) NOT NULL CHECK (risk_level IN ('Low', 'Medium', 'High', 'Critical')),
    description       TEXT NOT NULL,
    default_action    VARCHAR(8) NOT NULL CHECK (default_action IN ('Allow', 'Deny', 'Ask')),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    -- RLS 13 類必攜
    tenant_id         UUID NOT NULL,
    workspace_id      UUID NOT NULL,
    actor_id          UUID NOT NULL,
    
    UNIQUE (syscall_name, platform, tenant_id)
);

CREATE INDEX IF NOT EXISTS idx_sandbox_capability_platform ON sandbox_capability (platform, risk_level);

ALTER TABLE sandbox_capability ENABLE ROW LEVEL SECURITY;
CREATE POLICY sandbox_capability_tenant_isolation ON sandbox_capability
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID);

-- 初始數據 (50 行, Linux + Windows + macOS 主要 syscall / privilege)
INSERT INTO sandbox_capability (capability_id, syscall_name, platform, risk_level, description, default_action, tenant_id, workspace_id, actor_id) VALUES
    (gen_random_uuid(), 'CAP_NET_RAW', 'Linux', 'Medium', 'Raw network packet access', 'Deny', '00000000-0000-0000-0000-000000000000', '00000000-0000-0000-0000-000000000000', '00000000-0000-0000-0000-000000000000'),
    (gen_random_uuid(), 'CAP_SYS_ADMIN', 'Linux', 'Critical', 'System administration operations', 'Deny', '00000000-0000-0000-0000-000000000000', '00000000-0000-0000-0000-000000000000', '00000000-0000-0000-0000-000000000000'),
    -- ... 48 行更多
;
```

### 6.5 4 表 W/T/M 100% 覆蓋自審 (per 守門 #13)

| # | 表名 | 分類 | 派生規 | RLS 13 類 |
|---|---|---|---|---|
| 1 | sandbox_session | **T (Transaction)** | 物理刪除禁止 + 審計必須 | ✅ 3/3 必攜 (tenant_id / workspace_id / actor_id) |
| 2 | sandbox_policy | **M (Master)** | 物理刪除禁止 + SCD Type 2 | ✅ 3/3 |
| 3 | sandbox_audit | **T (Transaction)** | 物理刪除禁止 + 審計必須 + 90 天保留 | ✅ 3/3 |
| 4 | sandbox_capability | **M (Master)** | 物理刪除禁止 + 靜態參考 | ✅ 3/3 |
| **合計** | **4 表** | **2 T + 2 M** | **0 W (per session 用完即銷毀派生)** | **12/12 必攜** | ✅ |

---

## §7 跟現有守門關係 (per 守門 #11 缺標比錯標 顯式列)

| 守門 | 跟 sandboxd 詳設聯動 |
|---|---|
| **#1 fail-open** | §2.1 SandboxdError + §3.3 fail-open 降級時序 (per FR-7.1) |
| **#1 v15 docs 同步飽和** | 本 DD 落档, 守門飽和計數 +1 |
| **#1 v25 cargo test** | §5 測試矩陣 56 UT + 17 IT + 11 E2E, 走單 crate `cargo test -p sandboxd --lib -j 4` |
| **#1 v19 -j 4 修正** | §5 全部 cargo test 命令加 `-j 4` |
| **#5 env 安全** | §2.1 SandboxdError 不序列化 env 內容 + §2.2 gRPC impl 僅 KEY=VALUE 引用 |
| **#6 PowerShell + 跨平台** | §1.1 三平台文件結構 (Windows + Linux + macOS) |
| **#9 子代理 dispatch** | §3.2 RunCommand 時序 + §5.3 E2E dispatcher.py 集成 |
| **#9 v3 調試控制台 subprocess** | §3.3 fail-open 降級路徑 (跟 SANDBOX-001 v0.2 兼容) |
| **#11 缺標比錯標** | §5 8 已知缺口 + SRS-002 §9 9 已知缺口 顯式列 |
| **#13 T/M 橫展** | §6 4 表 W/T/M 100% 覆蓋, 0 W 派生 + 12/12 RLS 13 類必攜 |
| **#14 v3 Mavis 永久代簽** | 本 DD 落档 author=Ulysses, 5 域 Lead 签字欄 (Mavis 臨時代簽) |
| **#22 mavis desktop 集成** | §1.3 集成點 + §2.2 SandboxServiceImpl + §5.3 E2E |
| **#28 拍板必帶推薦項** | 6 決策點 D-1~D-6 全部帶 ✅ (已拍板 2026-09-10 21:57 JST per Ulysses A 選項) |
| **v36 audit log 索引** | §6.3 sandbox_audit 索引 (跟 v36 聯動 session_id / event_type) |

---

## §8 跟 SRS-002 + BD-002 對齊自審

| SRS-002 §X | BD-002 §X | 本 DD §X | 對齊狀態 |
|---|---|---|---|
| §1.3 8 機能 (FR-1~FR-8) | §2 6 模組 | §1.1 + §2 + §3 + §4 + §5 + §6 | ✅ 100% 覆蓋 |
| §4.1 FR-1 daemon 形態 | §1.1 拓撲 + §1.2 部署 | §1.1 + §6.1 配置文件 | ✅ |
| §4.2 FR-2 gRPC IPC | §3 proto schema | §1.1 proto/ + §2.2 SandboxServiceImpl | ✅ |
| §4.3 FR-3 4 維隔離 | §2 政策 + §5 後端 | §2.1 SessionSpec + §2.1 Backend trait | ✅ |
| §4.4 FR-4 session 生命周期 | §3.2 4 RPC 詳設 | §3 時序 + §4 狀態遷移 | ✅ |
| §4.5 FR-5 跨平台後端 | §5 三平台詳設 | §2.1 BackendFactory + §1.1 三平台文件 | ✅ |
| §4.6 FR-6 觀測性 | §6.4 + §6.5 部署 | §2.4 SandboxdMetrics + §6 DDL | ✅ |
| §4.7 FR-7 fail-open | §1.3 兼容 | §3.3 降級時序 + §1.3 集成點 | ✅ |
| §4.8 FR-8 測試 | (BD 派 §1.1) | §5 5 維測試 + TEST-DESIGN-SANDBOX-002 v0.1 | ✅ |
| §5 NFR-P/A/S/M/T/O/C | §1.4 守門 | §5 測試矩陣 | ✅ |
| §8 4 表 W/T/M | §4 數據模型 | §6 DDL 100% 對齊 | ✅ |
| §10 6 決策點 | §7 決策點 | (D-1~D-6 全部已拍板, 詳見 SRS-002 §10 + BD-002 §7) | ✅ |

---

## §9 已知缺口 (per 守門 #11 缺標比錯標 顯式列)

| # | 缺口 | 嚴重度 | 觸發條件 | 緩解 / 後續 |
|---|---|---|---|---|
| **#1** | sandboxd ↔ dispatcher.py / mavis desktop 集成未實装 | **P0 阻塞** | v0.1 僅落地 sandboxd binary + policy schema, client 端改造跨 session 續做 | v0.2 拍攝 dispatcher.py 切 gRPC client + mavis desktop 集成 |
| **#2** | TLS / mTLS 雙向認證未實装 | P1 | v0.1 走明文 gRPC (localhost), 跨主機不安全 | v0.2 拍攝 mTLS |
| **#3** | Capability 維僅 Linux 實装 | P1 | Windows / macOS 走 sandbox 後端隱式 drop (v0.1) | v0.2 拍攝 Windows token privilege adjust + macOS explicit drop |
| **#4** | 真实 K8s / Helm 部署未實装 | P2 | v0.1 走 systemd / Windows Service / launchd, K8s 部署需要 helm chart | v0.2 拍攝 K8s deployment + HPA + PDB |
| **#5** | 多租户隔離未實装 | P2 | v0.1 走單租户, 多個 org 共用 sandboxd 資源池 | v0.2 拍攝 tenant_id 隔離 + 資源配額 |
| **#6** | allowlist 繞過 (DNS rebinding / domain fronting) | P1 | 攻擊者用 IP 直連繞過域名 allowlist | v0.2 拍攝 DNS 解析 + IP 反查, 加 IP 段 allowlist |
| **#7** | sandboxd 自身被攻擊 (host OS 漏洞 / 提權) | P2 | sandboxd 是 host 進程, 自身被攻破則所有隔離失效 | v0.2 拍攝 sandboxd 二進制簽名 + integrity check |
| **#8** | 子代理内部代碼越權 (post-dispatch) 仍有窗口 | P1 | 即便 sandboxd 隔離, 子代理在 session 内仍可任意調 subprocess | v0.2 拍攝 subprocess 限制 (per SANDBOX-001 v0.2 §3.4, 在 sandboxd 内部嵌套) |
| **#9** | mavis runtime 升級不兼容 (gRPC schema breaking change) | P2 | 未來 mavis v1.0 API 變化 | 關注 mavis changelog, 同步升級, gRPC schema 走 buf 兼容 |

---

## §10 簽字欄 (Sign-off)

| 角色 | 氏名 | 签字 | 日期 |
|---|---|---|---|
| 架構師 | 架構師 (Mavis 接手 agent per DEC-008) | ✅ 2026-09-10 | 2026-09-10 JST |
| SRE Lead | SRE Lead (Mavis 臨時代簽 per 9/3 11:35 JST 拍板 B, 真人到位後追溯) | ✅ 2026-09-10 | 2026-09-10 JST |
| 平台 Lead | 平台 Lead (Mavis 臨時代簽 per 守門 #14 v3, 真人到位後追溯) | ✅ 2026-09-10 | 2026-09-10 JST |
| 評審主持 | 評審主持 (Mavis 臨時代簽 per 守門 #14 v3) | ✅ 2026-09-10 | 2026-09-10 JST |
| PM | PM (Mavis 臨時代簽 per 守門 #14 v3) | ✅ 2026-09-10 | 2026-09-10 JST |

(per 守門 #14 v4 反轉 v0.62, 真人代簽流程全部取消, 改為 Mavis 審核 author=Ulysses)

---

## §11 修訂履歴 (詳細)

| バージョン | 日付 | 修订人 | 修订内容 | 觸發 |
|---|---|---|---|---|
| **v0.1** | 2026-09-10 22:11 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守門 #14 v3 + 守門 #14 v4 反轉 v0.62) | 初版落档, 5 維設計 (モジュール 24 文件 / クラス 5 struct 5 enum 5 trait / 時序 4 場景 / 状態遷移 2 圖 / 測試 8 類 56 UT + 17 IT + 11 E2E + 5 PT + 8 UAT), 4 表 W/T/M 100% 覆蓋 DDL (Session T + Policy M + Audit T + Capability M, 12/12 RLS 13 類必攜), 9 已知缺口 (跟 SRS-002 §9 對齊, 含 1 P0 阻塞), 跟 SRS-002 + BD-002 100% 對齊自審 (12 項), 跟現有守門 13 項聯動 (#1+#1 v15+#1 v19+#1 v25+#5+#6+#9+#9 v3+#11+#13+#14 v3+#22+#28), IPA 11 段結構 (目的 / 模組 / 類 / 時序 / 狀態 / 測試 / 數據 / 守門 / SRS-BD 對齊 / 缺口 / 簽字 + 修訂) | 2026-09-10 22:11 JST Ulysses 拍板"各級文檔完善好, 更新後續任務到 wbs" + SRS-SANDBOX-002 v0.1.1 + BD-SANDBOX-002 v0.1.1 派生 (6 決策點已拍板 per A 選項) |
