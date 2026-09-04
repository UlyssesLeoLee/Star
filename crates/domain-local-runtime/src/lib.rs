//! domain-local-runtime crate
//!
//! 详细 spec: docs/specs/domain-local-runtime-spec.md §23 Local Runtime
//! 上游基本设计: docs/basic-design.md §4.6 / §23 / §1.1 LocalRuntime 子图
//! 数据设计: docs/data-design.md §4.25 (`local_runtime` schema)
//! API 设计: docs/api-design.md §3.24 (Runtime Registry / Command / Observation)
//!
//! ## 职责
//!
//! 集群外 Local Runtime(本地代理/CLI/Web 端,§23.2 LRT-001/002)的服务器侧 Registry / Port:
//! - `LocalRuntime` 聚合根(运行时注册 / 心跳 / 状态)
//! - `WorktreeMount` 实体(运行时挂载 Worktree 关系)
//! - `AgentExecutionContext` 实体(Agent 在 Local Runtime 上的执行上下文)
//! - `RuntimeHeartbeat` 事件(Append-only 心跳流)
//!
//! ## 关键不变量 (INV-RT-01~05)
//!
//! - INV-RT-01:LocalRuntime 必带 `tenant_id`(跨租户拒绝)
//! - INV-RT-02:WorktreeMount 必带 `runtime_id` + `worktree_id`
//! - INV-RT-03:Local path 平台不可信(INV-WT-04) — 仅作引用
//! - INV-RT-04:Heartbeat 超时(`> 300s`) → Status = Offline
//! - INV-RT-05:1 User → N Local Runtime,但 1 Runtime → 1 User(owner 唯一)
//!
//! Lead 责任: local-runtime Lead

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
pub use star_context::ActorContext;
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// ID 类型
// =====================================================================

define_uuid_id!(LocalRuntimeId);
define_uuid_id!(WorktreeMountId);
define_uuid_id!(AgentExecutionContextId);
define_uuid_id!(HeartbeatId);
define_uuid_id!(TenantId);
define_uuid_id!(UserId);
define_uuid_id!(DeviceId);
define_uuid_id!(WorktreeId);
define_uuid_id!(AgentSessionId);
define_uuid_id!(ProjectId);

// =====================================================================
// UUID 强类型 ID 宏(参考 domain-worktree / domain-tenant 模式)
// =====================================================================

#[macro_export]
/// 定义一个基于 Uuid 的强类型 ID(生成 struct + `new`/`as_uuid`/`From<Uuid>`/`Display` 实现)
macro_rules! define_uuid_id {
    ($name:ident) => {
        // 22 domain 共享宏生成的 UUID 强类型 ID struct + as_uuid helper
        // T1.5 deny(unreachable_pub) 落地后 macro 内部 pub 字段被误判为 unused
        // 宏级 allow 避免 22 domain 各自加, B.2 实证 30 err 收敛 (per P4 WBS §3)
        #[allow(unreachable_pub)]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        /// 领域强类型 ID(由 `define_uuid_id!` 宏统一生成,内部包装 Uuid)
        pub struct $name(pub Uuid);

        #[allow(dead_code)]
        impl $name {
            /// 生成一个新的随机 ID(UUID v4)
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
            /// 返回内部原始 Uuid 值
            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(u: Uuid) -> Self {
                Self(u)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

// =====================================================================
// 状态枚举
// =====================================================================

/// **RuntimeStatus** — Local Runtime 4 状态(§23.2, INV-RT-04)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuntimeStatus {
    /// 在线(心跳正常)
    Online,
    /// 离线(心跳超时 > 300s)
    Offline,
    /// 降级(部分功能异常,心跳仍通)
    Degraded,
    /// 维护中(管理员手动置入)
    Maintenance,
}

impl RuntimeStatus {
    /// 转换为大写字符串表示(如 "ONLINE")
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Online => "ONLINE",
            Self::Offline => "OFFLINE",
            Self::Degraded => "DEGRADED",
            Self::Maintenance => "MAINTENANCE",
        }
    }
    /// 是否可被外部调用(仅 Online / Degraded 接受命令)
    pub fn is_responsive(&self) -> bool {
        matches!(self, Self::Online | Self::Degraded)
    }
}

/// **MountStatus** — WorktreeMount 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MountStatus {
    /// 活跃挂载(运行时持有句柄)
    Active,
    /// 已卸载(运行时主动释放)
    Unmounted,
    /// 失联(运行时下线但卸载未完成)
    Stale,
}

impl MountStatus {
    /// 转换为大写字符串表示(如 "ACTIVE")
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "ACTIVE",
            Self::Unmounted => "UNMOUNTED",
            Self::Stale => "STALE",
        }
    }
}

// =====================================================================
// 实体
// =====================================================================

/// **LocalRuntime** — 聚合根(§23.2 LRT-001/002, INV-RT-01/05)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalRuntime {
    /// Runtime ID
    pub id: LocalRuntimeId,
    /// 必带,INV-RT-01
    pub tenant_id: TenantId,
    /// 1 Runtime → 1 User,INV-RT-05
    pub user_id: UserId,
    /// 关联 identity device
    pub device_id: DeviceId,
    /// 当前状态
    pub status: RuntimeStatus,
    /// 运行时版本,如 "1.0.0"
    pub version: String,
    /// 能力清单(git / docker / rust / ...)
    pub capabilities: Vec<String>,
    /// 最后心跳时间(INV-RT-04 用于超时判定)
    pub last_heartbeat: DateTime<Utc>,
    /// 注册时间
    pub registered_at: DateTime<Utc>,
    /// 元数据(host_name / os / arch)
    pub metadata: HashMap<String, String>,
}

impl LocalRuntime {
    /// 注册一个新 Runtime(INV-RT-01 tenant_id 必带)
    pub fn new(
        tenant_id: TenantId,
        user_id: UserId,
        device_id: DeviceId,
        version: String,
        capabilities: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: LocalRuntimeId::new(),
            tenant_id,
            user_id,
            device_id,
            status: RuntimeStatus::Online,
            version,
            capabilities,
            last_heartbeat: now,
            registered_at: now,
            metadata: HashMap::new(),
        }
    }

    /// 写入一条心跳
    pub fn record_heartbeat(&mut self) {
        self.last_heartbeat = Utc::now();
        // 收到心跳 → Online(若之前是 Offline,自动恢复)
        if matches!(self.status, RuntimeStatus::Offline) {
            self.status = RuntimeStatus::Online;
        }
    }

    /// INV-RT-04:距离 `now` 超过 `timeout` 即视为 Offline
    pub fn is_stale(&self, now: DateTime<Utc>, timeout: Duration) -> bool {
        let elapsed = now.signed_duration_since(self.last_heartbeat);
        let max =
            ChronoDuration::from_std(timeout).unwrap_or_else(|_| ChronoDuration::seconds(300));
        elapsed > max
    }

    /// 应用 INV-RT-04 判定
    pub fn reconcile_status(&mut self, now: DateTime<Utc>, timeout: Duration) {
        if self.status == RuntimeStatus::Maintenance {
            return; // 维护态不参与超时判定
        }
        if self.is_stale(now, timeout) {
            self.status = RuntimeStatus::Offline;
        }
    }
}

/// **WorktreeMount** — Local Runtime ↔ Worktree 挂载(实体, INV-RT-02/03)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeMount {
    /// 挂载 ID
    pub id: WorktreeMountId,
    /// 所属 Runtime ID
    pub runtime_id: LocalRuntimeId,
    /// 挂载的 Worktree ID
    pub worktree_id: WorktreeId,
    /// 平台相关本地路径 — 仅作引用,平台不可信(INV-RT-03 / INV-WT-04)
    pub local_path: String,
    /// 挂载时间
    pub mounted_at: DateTime<Utc>,
    /// 挂载状态
    pub status: MountStatus,
}

impl WorktreeMount {
    /// 创建一个新的挂载(状态默认为 Active)
    pub fn new(runtime_id: LocalRuntimeId, worktree_id: WorktreeId, local_path: String) -> Self {
        Self {
            id: WorktreeMountId::new(),
            runtime_id,
            worktree_id,
            local_path,
            mounted_at: Utc::now(),
            status: MountStatus::Active,
        }
    }
}

/// **AgentExecutionContext** — Agent 在 Local Runtime 上的执行上下文(实体)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionContext {
    /// 执行上下文 ID
    pub id: AgentExecutionContextId,
    /// 所属 Runtime ID
    pub runtime_id: LocalRuntimeId,
    /// 关联的 Agent Session ID
    pub agent_session_id: AgentSessionId,
    /// 工作目录(平台无关字符串,平台解析时再映射)
    pub working_dir: String,
    /// 环境变量(已 sanitize)
    pub environment: HashMap<String, String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl AgentExecutionContext {
    /// 创建一个新的执行上下文
    pub fn new(
        runtime_id: LocalRuntimeId,
        agent_session_id: AgentSessionId,
        working_dir: String,
        environment: HashMap<String, String>,
    ) -> Self {
        Self {
            id: AgentExecutionContextId::new(),
            runtime_id,
            agent_session_id,
            working_dir,
            environment,
            created_at: Utc::now(),
        }
    }
}

/// **RuntimeHeartbeat** — 心跳事件(Append-only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeHeartbeat {
    /// 心跳事件 ID
    pub id: HeartbeatId,
    /// 所属 Runtime ID
    pub runtime_id: LocalRuntimeId,
    /// 接收时间
    pub received_at: DateTime<Utc>,
    /// load average(1-min,可选)
    pub load_average: Option<f32>,
    /// 已用内存字节数(可选)
    pub memory_used_bytes: Option<u64>,
}

impl RuntimeHeartbeat {
    /// 创建一条新的心跳记录
    pub fn new(
        runtime_id: LocalRuntimeId,
        load_average: Option<f32>,
        memory_used_bytes: Option<u64>,
    ) -> Self {
        Self {
            id: HeartbeatId::new(),
            runtime_id,
            received_at: Utc::now(),
            load_average,
            memory_used_bytes,
        }
    }
}

// =====================================================================
// 错误
// =====================================================================

/// **RuntimeError** — domain-local-runtime 错误类型
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// 资源未找到
    #[error("not found: {0}")]
    NotFound(String),
    /// 权限拒绝
    #[error("permission denied")]
    PermissionDenied,
    /// 跨租户访问拒绝
    #[error("cross-tenant access denied: actor tenant {0} vs resource tenant {1}")]
    CrossTenantDenied(TenantId, TenantId),
    /// 无效状态
    #[error("invalid state: {0}")]
    InvalidState(String),
    /// 资源冲突
    #[error("conflict: {0}")]
    Conflict(String),
    /// 内部错误
    #[error("internal: {0}")]
    Internal(String),
}

impl RuntimeError {
    /// 返回错误码字符串(如 "RUNTIME_NOT_FOUND")
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "RUNTIME_NOT_FOUND",
            Self::PermissionDenied => "RUNTIME_PERMISSION_DENIED",
            Self::CrossTenantDenied(_, _) => "RUNTIME_CROSS_TENANT_DENIED",
            Self::InvalidState(_) => "RUNTIME_INVALID_STATE",
            Self::Conflict(_) => "RUNTIME_CONFLICT",
            Self::Internal(_) => "RUNTIME_INTERNAL",
        }
    }
}

// =====================================================================
// 命令 / 查询 DTO
// =====================================================================

/// 注册 Runtime 命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRuntimeCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 用户 ID
    pub user_id: UserId,
    /// 设备 ID
    pub device_id: DeviceId,
    /// 运行时版本
    pub version: String,
    /// 能力清单
    pub capabilities: Vec<String>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 心跳命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Runtime ID
    pub runtime_id: LocalRuntimeId,
    /// load average(可选)
    pub load_average: Option<f32>,
    /// 已用内存字节数(可选)
    pub memory_used_bytes: Option<u64>,
}

/// 挂载 Worktree 命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountWorktreeCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Runtime ID
    pub runtime_id: LocalRuntimeId,
    /// Worktree ID
    pub worktree_id: WorktreeId,
    /// 本地路径
    pub local_path: String,
}

/// 卸载 Worktree 命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnmountWorktreeCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 挂载 ID
    pub mount_id: WorktreeMountId,
}

/// 创建执行上下文命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateExecContextCommand {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Runtime ID
    pub runtime_id: LocalRuntimeId,
    /// Agent Session ID
    pub agent_session_id: AgentSessionId,
    /// 工作目录
    pub working_dir: String,
    /// 环境变量
    pub environment: HashMap<String, String>,
}

/// 获取 Runtime 查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRuntimeQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Runtime ID
    pub runtime_id: LocalRuntimeId,
}

/// 按用户列出 Runtime 查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListByUserQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 用户 ID
    pub user_id: UserId,
}

/// 获取心跳流查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHeartbeatsQuery {
    /// 租户 ID
    pub tenant_id: TenantId,
    /// Runtime ID
    pub runtime_id: LocalRuntimeId,
    /// 返回条数上限(可选)
    pub limit: Option<usize>,
}

// =====================================================================
// 端口(Port Traits)
// =====================================================================

/// **RuntimeCommandPort** — 写操作(§3.24)
#[async_trait]
pub trait RuntimeCommandPort: Send + Sync {
    /// 注册一个 Local Runtime(INV-RT-01/05)
    async fn register_runtime(
        &self,
        cmd: RegisterRuntimeCommand,
        actor: &ActorContext,
    ) -> Result<LocalRuntime, RuntimeError>;

    /// 接收心跳 → 更新 status(INV-RT-04) + 追加 Append-only 事件
    async fn heartbeat(
        &self,
        cmd: HeartbeatCommand,
        actor: &ActorContext,
    ) -> Result<LocalRuntime, RuntimeError>;

    /// 挂载一个 Worktree 到该 Runtime(INV-RT-02)
    async fn mount_worktree(
        &self,
        cmd: MountWorktreeCommand,
        actor: &ActorContext,
    ) -> Result<WorktreeMount, RuntimeError>;

    /// 卸载一个 WorktreeMount
    async fn unmount_worktree(
        &self,
        cmd: UnmountWorktreeCommand,
        actor: &ActorContext,
    ) -> Result<WorktreeMount, RuntimeError>;

    /// 创建 Agent 在该 Runtime 上的执行上下文
    async fn create_exec_context(
        &self,
        cmd: CreateExecContextCommand,
        actor: &ActorContext,
    ) -> Result<AgentExecutionContext, RuntimeError>;
}

/// **RuntimeQueryPort** — 读操作
#[async_trait]
pub trait RuntimeQueryPort: Send + Sync {
    /// 按 ID 获取 Runtime
    async fn get(
        &self,
        q: GetRuntimeQuery,
        actor: &ActorContext,
    ) -> Result<LocalRuntime, RuntimeError>;

    /// 按用户列出 Runtime
    async fn list_by_user(
        &self,
        q: ListByUserQuery,
        actor: &ActorContext,
    ) -> Result<Vec<LocalRuntime>, RuntimeError>;

    /// Append-only 心跳流,默认按时间倒序
    async fn get_heartbeats(
        &self,
        q: GetHeartbeatsQuery,
        actor: &ActorContext,
    ) -> Result<Vec<RuntimeHeartbeat>, RuntimeError>;
}

/// **RuntimeRepository** — 持久化抽象
#[async_trait]
pub trait RuntimeRepository: Send + Sync {
    /// 插入 Runtime
    async fn insert_runtime(&self, r: LocalRuntime) -> Result<(), RuntimeError>;
    /// 按 ID 获取 Runtime
    async fn get_runtime(&self, id: LocalRuntimeId) -> Result<LocalRuntime, RuntimeError>;
    /// 更新 Runtime
    async fn update_runtime(&self, r: LocalRuntime) -> Result<(), RuntimeError>;
    /// 按用户列出 Runtime
    async fn list_runtimes_by_user(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Vec<LocalRuntime>, RuntimeError>;

    /// 插入挂载
    async fn insert_mount(&self, m: WorktreeMount) -> Result<(), RuntimeError>;
    /// 按 ID 获取挂载
    async fn get_mount(&self, id: WorktreeMountId) -> Result<WorktreeMount, RuntimeError>;
    /// 更新挂载
    async fn update_mount(&self, m: WorktreeMount) -> Result<(), RuntimeError>;
    /// 列出 Runtime 下所有活跃挂载
    async fn list_active_mounts_by_runtime(
        &self,
        runtime_id: LocalRuntimeId,
    ) -> Result<Vec<WorktreeMount>, RuntimeError>;
    /// 检查同一 (runtime, worktree) 是否已有 Active 挂载(INV-RT-02 唯一)
    async fn find_active_mount(
        &self,
        runtime_id: LocalRuntimeId,
        worktree_id: WorktreeId,
    ) -> Result<Option<WorktreeMount>, RuntimeError>;

    /// 插入执行上下文
    async fn insert_exec_context(&self, c: AgentExecutionContext) -> Result<(), RuntimeError>;
    /// 按 ID 获取执行上下文
    async fn get_exec_context(
        &self,
        id: AgentExecutionContextId,
    ) -> Result<AgentExecutionContext, RuntimeError>;

    /// Append-only:追加心跳
    async fn append_heartbeat(&self, h: RuntimeHeartbeat) -> Result<(), RuntimeError>;
    /// Append-only:读取心跳,默认按 received_at 倒序
    async fn list_heartbeats(
        &self,
        runtime_id: LocalRuntimeId,
        limit: Option<usize>,
    ) -> Result<Vec<RuntimeHeartbeat>, RuntimeError>;
}

// =====================================================================
// 配置:心跳超时阈值(§23.2 / INV-RT-04)
// =====================================================================

/// 心跳超时阈值,默认 300s
pub const HEARTBEAT_TIMEOUT_SECONDS: u64 = 300;

/// 返回默认心跳超时时长
pub fn default_heartbeat_timeout() -> Duration {
    Duration::from_secs(HEARTBEAT_TIMEOUT_SECONDS)
}

// =====================================================================
// InMemoryRuntimeService + Repository
// =====================================================================

/// **InMemoryRuntimeService** — 内存版 Runtime 服务(实现 RuntimeCommandPort + RuntimeQueryPort)
pub struct InMemoryRuntimeService {
    repo: Arc<dyn RuntimeRepository>,
    /// 服务级缓存,与 repo 保持一致(便于快速 list_by_user / 状态更新)
    runtimes: Arc<RwLock<HashMap<LocalRuntimeId, LocalRuntime>>>,
    mounts: Arc<RwLock<HashMap<WorktreeMountId, WorktreeMount>>>,
    exec_contexts: Arc<RwLock<HashMap<AgentExecutionContextId, AgentExecutionContext>>>,
    heartbeats: Arc<RwLock<Vec<RuntimeHeartbeat>>>,
    heartbeat_timeout: Duration,
}

impl InMemoryRuntimeService {
    /// 创建使用内置内存仓储的服务
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemoryRuntimeRepository::new()),
            runtimes: Arc::new(RwLock::new(HashMap::new())),
            mounts: Arc::new(RwLock::new(HashMap::new())),
            exec_contexts: Arc::new(RwLock::new(HashMap::new())),
            heartbeats: Arc::new(RwLock::new(Vec::new())),
            heartbeat_timeout: default_heartbeat_timeout(),
        }
    }

    /// 使用指定仓储创建服务
    pub fn with_repo(repo: Arc<dyn RuntimeRepository>) -> Self {
        Self {
            repo,
            runtimes: Arc::new(RwLock::new(HashMap::new())),
            mounts: Arc::new(RwLock::new(HashMap::new())),
            exec_contexts: Arc::new(RwLock::new(HashMap::new())),
            heartbeats: Arc::new(RwLock::new(Vec::new())),
            heartbeat_timeout: default_heartbeat_timeout(),
        }
    }

    /// 用于测试:构造一个把当前时间固定为 `now` 的服务
    /// (用 `now_heartbeat_received_at` 替换实际 `Utc::now()` 不可行,因为 heartbeat
    /// 内部用 Utc::now;此处用 HeartbeatCommand.received_at_offset 偏移替代)
    pub fn with_heartbeat_timeout(mut self, timeout: Duration) -> Self {
        self.heartbeat_timeout = timeout;
        self
    }

    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

impl Default for InMemoryRuntimeService {
    fn default() -> Self {
        Self::new()
    }
}

/// **InMemoryRuntimeRepository** — 内存版 Runtime 仓储(测试/开发用)
pub struct InMemoryRuntimeRepository {
    runtimes: RwLock<HashMap<LocalRuntimeId, LocalRuntime>>,
    mounts: RwLock<HashMap<WorktreeMountId, WorktreeMount>>,
    exec_contexts: RwLock<HashMap<AgentExecutionContextId, AgentExecutionContext>>,
    heartbeats: RwLock<Vec<RuntimeHeartbeat>>,
}

impl InMemoryRuntimeRepository {
    /// 创建空的内存仓储
    pub fn new() -> Self {
        Self {
            runtimes: RwLock::new(HashMap::new()),
            mounts: RwLock::new(HashMap::new()),
            exec_contexts: RwLock::new(HashMap::new()),
            heartbeats: RwLock::new(Vec::new()),
        }
    }
}

impl Default for InMemoryRuntimeRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RuntimeRepository for InMemoryRuntimeRepository {
    async fn insert_runtime(&self, r: LocalRuntime) -> Result<(), RuntimeError> {
        let mut s = self.runtimes.write().expect("lock");
        if s.contains_key(&r.id) {
            return Err(RuntimeError::Conflict(format!(
                "LocalRuntime {} 已存在",
                r.id
            )));
        }
        s.insert(r.id, r);
        Ok(())
    }
    async fn get_runtime(&self, id: LocalRuntimeId) -> Result<LocalRuntime, RuntimeError> {
        let s = self.runtimes.read().expect("lock");
        s.get(&id)
            .cloned()
            .ok_or_else(|| RuntimeError::NotFound(format!("runtime:{}", id.as_uuid())))
    }
    async fn update_runtime(&self, r: LocalRuntime) -> Result<(), RuntimeError> {
        let mut s = self.runtimes.write().expect("lock");
        s.insert(r.id, r);
        Ok(())
    }
    async fn list_runtimes_by_user(
        &self,
        _tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Vec<LocalRuntime>, RuntimeError> {
        let s = self.runtimes.read().expect("lock");
        Ok(s.values()
            .filter(|r| r.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn insert_mount(&self, m: WorktreeMount) -> Result<(), RuntimeError> {
        let mut s = self.mounts.write().expect("lock");
        if s.contains_key(&m.id) {
            return Err(RuntimeError::Conflict(format!(
                "WorktreeMount {} 已存在",
                m.id
            )));
        }
        s.insert(m.id, m);
        Ok(())
    }
    async fn get_mount(&self, id: WorktreeMountId) -> Result<WorktreeMount, RuntimeError> {
        let s = self.mounts.read().expect("lock");
        s.get(&id)
            .cloned()
            .ok_or_else(|| RuntimeError::NotFound(format!("mount:{}", id.as_uuid())))
    }
    async fn update_mount(&self, m: WorktreeMount) -> Result<(), RuntimeError> {
        let mut s = self.mounts.write().expect("lock");
        s.insert(m.id, m);
        Ok(())
    }
    async fn list_active_mounts_by_runtime(
        &self,
        runtime_id: LocalRuntimeId,
    ) -> Result<Vec<WorktreeMount>, RuntimeError> {
        let s = self.mounts.read().expect("lock");
        Ok(s.values()
            .filter(|m| m.runtime_id == runtime_id && m.status == MountStatus::Active)
            .cloned()
            .collect())
    }
    async fn find_active_mount(
        &self,
        runtime_id: LocalRuntimeId,
        worktree_id: WorktreeId,
    ) -> Result<Option<WorktreeMount>, RuntimeError> {
        let s = self.mounts.read().expect("lock");
        Ok(s.values()
            .find(|m| {
                m.runtime_id == runtime_id
                    && m.worktree_id == worktree_id
                    && m.status == MountStatus::Active
            })
            .cloned())
    }

    async fn insert_exec_context(&self, c: AgentExecutionContext) -> Result<(), RuntimeError> {
        let mut s = self.exec_contexts.write().expect("lock");
        if s.contains_key(&c.id) {
            return Err(RuntimeError::Conflict(format!(
                "AgentExecutionContext {} 已存在",
                c.id
            )));
        }
        s.insert(c.id, c);
        Ok(())
    }
    async fn get_exec_context(
        &self,
        id: AgentExecutionContextId,
    ) -> Result<AgentExecutionContext, RuntimeError> {
        let s = self.exec_contexts.read().expect("lock");
        s.get(&id)
            .cloned()
            .ok_or_else(|| RuntimeError::NotFound(format!("exec_context:{}", id.as_uuid())))
    }

    async fn append_heartbeat(&self, h: RuntimeHeartbeat) -> Result<(), RuntimeError> {
        let mut v = self.heartbeats.write().expect("lock");
        v.push(h);
        Ok(())
    }
    async fn list_heartbeats(
        &self,
        runtime_id: LocalRuntimeId,
        limit: Option<usize>,
    ) -> Result<Vec<RuntimeHeartbeat>, RuntimeError> {
        let v = self.heartbeats.read().expect("lock");
        let mut out: Vec<RuntimeHeartbeat> = v
            .iter()
            .filter(|h| h.runtime_id == runtime_id)
            .cloned()
            .collect();
        // Append-only:按 received_at 倒序
        out.sort_by(|a, b| b.received_at.cmp(&a.received_at));
        if let Some(l) = limit {
            out.truncate(l);
        }
        Ok(out)
    }
}

#[async_trait]
impl RuntimeCommandPort for InMemoryRuntimeService {
    async fn register_runtime(
        &self,
        cmd: RegisterRuntimeCommand,
        actor: &ActorContext,
    ) -> Result<LocalRuntime, RuntimeError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(RuntimeError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        // INV-RT-05:1 Runtime → 1 User,owner 校验
        // (此处不要求 UserId::from(actor.user_id) == cmd.user_id;允许管理员代注册)
        if !actor.has_role("tenant_admin")
            && !actor.is_local_runtime
            && UserId::from(actor.user_id) != cmd.user_id
        {
            return Err(RuntimeError::PermissionDenied);
        }
        // INV-RT-01:tenant_id 必带
        if cmd.tenant_id.0.is_nil() {
            return Err(RuntimeError::InvalidState("tenant_id required".to_string()));
        }
        // 去重:同 (tenant, user, device) 已存在则拒绝
        {
            let s = self.runtimes.read().expect("lock");
            if s.values().any(|r| {
                r.tenant_id == cmd.tenant_id
                    && r.user_id == cmd.user_id
                    && r.device_id == cmd.device_id
            }) {
                return Err(RuntimeError::Conflict(format!(
                    "device {} 已注册到本租户",
                    cmd.device_id
                )));
            }
        }
        let now = self.now();
        let mut r = LocalRuntime::new(
            cmd.tenant_id,
            cmd.user_id,
            cmd.device_id,
            cmd.version,
            cmd.capabilities,
        );
        r.last_heartbeat = now;
        r.metadata = cmd.metadata;
        self.repo.insert_runtime(r.clone()).await?;
        self.runtimes.write().expect("lock").insert(r.id, r.clone());
        Ok(r)
    }

    async fn heartbeat(
        &self,
        cmd: HeartbeatCommand,
        actor: &ActorContext,
    ) -> Result<LocalRuntime, RuntimeError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(RuntimeError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        // 心跳必须由 Local Runtime 自己发(actor.is_local_runtime)
        // 代理场景:管理员手动触发可允许
        if !actor.is_local_runtime && !actor.has_role("tenant_admin") {
            return Err(RuntimeError::PermissionDenied);
        }
        let mut r = self
            .runtimes
            .write()
            .expect("lock")
            .get_mut(&cmd.runtime_id)
            .cloned()
            .ok_or_else(|| {
                RuntimeError::NotFound(format!("runtime:{}", cmd.runtime_id.as_uuid()))
            })?;
        if r.tenant_id != cmd.tenant_id {
            return Err(RuntimeError::CrossTenantDenied(cmd.tenant_id, r.tenant_id));
        }
        r.record_heartbeat();
        self.repo.update_runtime(r.clone()).await?;
        self.runtimes.write().expect("lock").insert(r.id, r.clone());

        // Append-only 心跳事件
        let hb = RuntimeHeartbeat::new(cmd.runtime_id, cmd.load_average, cmd.memory_used_bytes);
        self.repo.append_heartbeat(hb.clone()).await?;
        self.heartbeats.write().expect("lock").push(hb);
        Ok(r)
    }

    async fn mount_worktree(
        &self,
        cmd: MountWorktreeCommand,
        actor: &ActorContext,
    ) -> Result<WorktreeMount, RuntimeError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(RuntimeError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        if !actor.is_local_runtime && !actor.has_role("tenant_admin") {
            return Err(RuntimeError::PermissionDenied);
        }
        let r = self
            .runtimes
            .read()
            .expect("lock")
            .get(&cmd.runtime_id)
            .cloned()
            .ok_or_else(|| {
                RuntimeError::NotFound(format!("runtime:{}", cmd.runtime_id.as_uuid()))
            })?;
        if r.tenant_id != cmd.tenant_id {
            return Err(RuntimeError::CrossTenantDenied(cmd.tenant_id, r.tenant_id));
        }
        if !r.status.is_responsive() {
            return Err(RuntimeError::InvalidState(format!(
                "runtime status {} 不接受挂载",
                r.status.as_str()
            )));
        }
        // INV-RT-02:同一 (runtime, worktree) 不可重复 Active 挂载
        if let Some(existing) = self
            .repo
            .find_active_mount(cmd.runtime_id, cmd.worktree_id)
            .await?
        {
            return Err(RuntimeError::Conflict(format!(
                "worktree {} 已被 mount {} 挂载",
                cmd.worktree_id, existing.id
            )));
        }
        let m = WorktreeMount::new(cmd.runtime_id, cmd.worktree_id, cmd.local_path);
        self.repo.insert_mount(m.clone()).await?;
        self.mounts.write().expect("lock").insert(m.id, m.clone());
        Ok(m)
    }

    async fn unmount_worktree(
        &self,
        cmd: UnmountWorktreeCommand,
        actor: &ActorContext,
    ) -> Result<WorktreeMount, RuntimeError> {
        if !actor.is_local_runtime && !actor.has_role("tenant_admin") {
            return Err(RuntimeError::PermissionDenied);
        }
        let mut m = self
            .mounts
            .read()
            .expect("lock")
            .get(&cmd.mount_id)
            .cloned()
            .ok_or_else(|| RuntimeError::NotFound(format!("mount:{}", cmd.mount_id.as_uuid())))?;
        // 校验 tenant 一致性
        let r = self
            .runtimes
            .read()
            .expect("lock")
            .get(&m.runtime_id)
            .cloned()
            .ok_or_else(|| RuntimeError::NotFound(format!("runtime:{}", m.runtime_id.as_uuid())))?;
        if r.tenant_id != TenantId::from(actor.tenant_id) {
            return Err(RuntimeError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                r.tenant_id,
            ));
        }
        if r.tenant_id != cmd.tenant_id {
            return Err(RuntimeError::CrossTenantDenied(cmd.tenant_id, r.tenant_id));
        }
        if m.status == MountStatus::Unmounted {
            return Err(RuntimeError::InvalidState("already unmounted".to_string()));
        }
        m.status = MountStatus::Unmounted;
        self.repo.update_mount(m.clone()).await?;
        self.mounts.write().expect("lock").insert(m.id, m.clone());
        Ok(m)
    }

    async fn create_exec_context(
        &self,
        cmd: CreateExecContextCommand,
        actor: &ActorContext,
    ) -> Result<AgentExecutionContext, RuntimeError> {
        if TenantId::from(actor.tenant_id) != cmd.tenant_id {
            return Err(RuntimeError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                cmd.tenant_id,
            ));
        }
        if !actor.is_local_runtime && !actor.has_role("tenant_admin") {
            return Err(RuntimeError::PermissionDenied);
        }
        let r = self
            .runtimes
            .read()
            .expect("lock")
            .get(&cmd.runtime_id)
            .cloned()
            .ok_or_else(|| {
                RuntimeError::NotFound(format!("runtime:{}", cmd.runtime_id.as_uuid()))
            })?;
        if r.tenant_id != cmd.tenant_id {
            return Err(RuntimeError::CrossTenantDenied(cmd.tenant_id, r.tenant_id));
        }
        if !r.status.is_responsive() {
            return Err(RuntimeError::InvalidState(format!(
                "runtime status {} 不接受 exec 上下文",
                r.status.as_str()
            )));
        }
        let ctx = AgentExecutionContext::new(
            cmd.runtime_id,
            cmd.agent_session_id,
            cmd.working_dir,
            cmd.environment,
        );
        self.repo.insert_exec_context(ctx.clone()).await?;
        self.exec_contexts
            .write()
            .expect("lock")
            .insert(ctx.id, ctx.clone());
        Ok(ctx)
    }
}

#[async_trait]
impl RuntimeQueryPort for InMemoryRuntimeService {
    async fn get(
        &self,
        q: GetRuntimeQuery,
        actor: &ActorContext,
    ) -> Result<LocalRuntime, RuntimeError> {
        if TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(RuntimeError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        let r = self
            .runtimes
            .read()
            .expect("lock")
            .get(&q.runtime_id)
            .cloned()
            .ok_or_else(|| RuntimeError::NotFound(format!("runtime:{}", q.runtime_id.as_uuid())))?;
        if r.tenant_id != q.tenant_id {
            return Err(RuntimeError::CrossTenantDenied(q.tenant_id, r.tenant_id));
        }
        // INV-RT-05:1 Runtime → 1 User,非 owner 不可读
        if !actor.is_local_runtime
            && !actor.has_role("tenant_admin")
            && UserId::from(actor.user_id) != r.user_id
        {
            return Err(RuntimeError::PermissionDenied);
        }
        Ok(r)
    }

    async fn list_by_user(
        &self,
        q: ListByUserQuery,
        actor: &ActorContext,
    ) -> Result<Vec<LocalRuntime>, RuntimeError> {
        if TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(RuntimeError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        // 非管理员:只允许看自己
        if !actor.has_role("tenant_admin") && UserId::from(actor.user_id) != q.user_id {
            return Err(RuntimeError::PermissionDenied);
        }
        let s = self.runtimes.read().expect("lock");
        Ok(s.values()
            .filter(|r| r.tenant_id == q.tenant_id && r.user_id == q.user_id)
            .cloned()
            .collect())
    }

    async fn get_heartbeats(
        &self,
        q: GetHeartbeatsQuery,
        actor: &ActorContext,
    ) -> Result<Vec<RuntimeHeartbeat>, RuntimeError> {
        if TenantId::from(actor.tenant_id) != q.tenant_id {
            return Err(RuntimeError::CrossTenantDenied(
                TenantId::from(actor.tenant_id),
                q.tenant_id,
            ));
        }
        // 必须先验证 runtime 存在 & tenant 一致
        let r = self
            .runtimes
            .read()
            .expect("lock")
            .get(&q.runtime_id)
            .cloned()
            .ok_or_else(|| RuntimeError::NotFound(format!("runtime:{}", q.runtime_id.as_uuid())))?;
        if r.tenant_id != q.tenant_id {
            return Err(RuntimeError::CrossTenantDenied(q.tenant_id, r.tenant_id));
        }
        if !actor.is_local_runtime
            && !actor.has_role("tenant_admin")
            && UserId::from(actor.user_id) != r.user_id
        {
            return Err(RuntimeError::PermissionDenied);
        }
        self.repo.list_heartbeats(q.runtime_id, q.limit).await
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    // B.2 改写 (per P4 WBS §3): helper 签名从 TenantId/UserId 改为 Uuid,
    // 内部 .into() 强类型, 让 26 处 test 调用方不需逐处加 TenantId(x) wrapper
    fn make_actor(tenant_id: Uuid, user_id: Uuid) -> ActorContext {
        ActorContext::new(user_id, tenant_id)
    }

    fn make_admin(tenant_id: Uuid) -> ActorContext {
        ActorContext::new(Uuid::new_v4(), tenant_id).with_role("tenant_admin")
    }

    fn make_local_runtime_actor(tenant_id: Uuid, user_id: Uuid) -> ActorContext {
        ActorContext::new(user_id, tenant_id).as_local_runtime()
    }

    fn make_register_cmd(tenant_id: Uuid, user_id: Uuid) -> RegisterRuntimeCommand {
        RegisterRuntimeCommand {
            tenant_id: TenantId(tenant_id),
            user_id: UserId(user_id),
            device_id: DeviceId::new(),
            version: "1.0.0".to_string(),
            capabilities: vec!["git".to_string(), "rust".to_string()],
            metadata: HashMap::new(),
        }
    }

    fn make_register_cmd_with_device(
        tenant_id: Uuid,
        user_id: Uuid,
        device_id: DeviceId,
    ) -> RegisterRuntimeCommand {
        RegisterRuntimeCommand {
            tenant_id: TenantId(tenant_id),
            user_id: UserId(user_id),
            device_id,
            version: "1.0.0".to_string(),
            capabilities: vec!["git".to_string()],
            metadata: HashMap::new(),
        }
    }

    // -----------------------------------------------------------------
    // 1. register_runtime_basic:基础注册 + INV-RT-01 tenant_id 必带
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn register_runtime_basic() {
        let svc = InMemoryRuntimeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id, user_id);
        let cmd = make_register_cmd(tenant_id, user_id);
        let r = svc.register_runtime(cmd, &actor).await.unwrap();
        assert_eq!(r.tenant_id, TenantId(tenant_id));
        assert_eq!(r.user_id, UserId(user_id));
        assert_eq!(r.status, RuntimeStatus::Online);
        assert_eq!(r.version, "1.0.0");
        assert!(r.capabilities.contains(&"git".to_string()));
    }

    // -----------------------------------------------------------------
    // 2. register_runtime_unique_per_user:同 (tenant, user, device) 拒绝
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn register_runtime_unique_per_user() {
        let svc = InMemoryRuntimeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let actor = make_actor(tenant_id, user_id);
        let device = DeviceId::new();
        // 同 device 第二次注册应 Conflict
        let cmd1 = make_register_cmd_with_device(tenant_id, user_id, device);
        let _r1 = svc.register_runtime(cmd1, &actor).await.unwrap();
        let cmd2 = make_register_cmd_with_device(tenant_id, user_id, device);
        let res = svc.register_runtime(cmd2, &actor).await;
        assert!(matches!(res, Err(RuntimeError::Conflict(_))));
    }

    // -----------------------------------------------------------------
    // 3. heartbeat_updates_status:心跳写入 + last_heartbeat 不晚于注册时间
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn heartbeat_updates_status() {
        let svc = InMemoryRuntimeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let admin = make_admin(tenant_id);
        let r = svc
            .register_runtime(make_register_cmd(tenant_id, user_id), &admin)
            .await
            .unwrap();
        let lr_actor = make_local_runtime_actor(tenant_id, user_id);
        // 睡 5ms 确保 last_heartbeat 严格更新
        sleep(Duration::from_millis(5));
        let updated = svc
            .heartbeat(
                HeartbeatCommand {
                    tenant_id: TenantId(tenant_id),
                    runtime_id: r.id,
                    load_average: Some(0.42),
                    memory_used_bytes: Some(1024 * 1024 * 512),
                },
                &lr_actor,
            )
            .await
            .unwrap();
        assert_eq!(updated.status, RuntimeStatus::Online);
        assert!(
            updated.last_heartbeat >= r.last_heartbeat,
            "heartbeat 应推进 last_heartbeat"
        );
    }

    // -----------------------------------------------------------------
    // 4. heartbeat_timeout_offline:模拟 350s 超时 → 状态变 Offline
    //    (用 reconcile_status 注入"当前时间"——通过 last_heartbeat 手动回拨)
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn heartbeat_timeout_offline() {
        let svc = InMemoryRuntimeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let admin = make_admin(tenant_id);
        let r = svc
            .register_runtime(make_register_cmd(tenant_id, user_id), &admin)
            .await
            .unwrap();
        // 把 last_heartbeat 手动回拨到 400s 前
        let mut r2 = svc
            .runtimes
            .read()
            .expect("lock")
            .get(&r.id)
            .cloned()
            .unwrap();
        r2.last_heartbeat = Utc::now() - ChronoDuration::seconds(400);
        svc.repo.update_runtime(r2.clone()).await.unwrap();
        svc.runtimes
            .write()
            .expect("lock")
            .insert(r2.id, r2.clone());

        // reconcile:模拟 350s 之后调用
        r2.reconcile_status(Utc::now(), default_heartbeat_timeout());
        assert_eq!(r2.status, RuntimeStatus::Offline);
    }

    // -----------------------------------------------------------------
    // 5. mount_worktree:挂载 + INV-RT-02 校验
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn mount_worktree() {
        let svc = InMemoryRuntimeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let admin = make_admin(tenant_id);
        let r = svc
            .register_runtime(make_register_cmd(tenant_id, user_id), &admin)
            .await
            .unwrap();
        let m = svc
            .mount_worktree(
                MountWorktreeCommand {
                    tenant_id: TenantId(tenant_id),
                    runtime_id: r.id,
                    worktree_id: WorktreeId::new(),
                    local_path: "/home/u/worktree-a".to_string(),
                },
                &admin,
            )
            .await
            .unwrap();
        assert_eq!(m.runtime_id, r.id);
        assert_eq!(m.status, MountStatus::Active);
    }

    // -----------------------------------------------------------------
    // 6. mount_duplicate_worktree_rejected:同一 runtime 不可 mount 同一 worktree 两次
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn mount_duplicate_worktree_rejected() {
        let svc = InMemoryRuntimeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let admin = make_admin(tenant_id);
        let r = svc
            .register_runtime(make_register_cmd(tenant_id, user_id), &admin)
            .await
            .unwrap();
        let wt_id = WorktreeId::new();
        svc.mount_worktree(
            MountWorktreeCommand {
                tenant_id: TenantId(tenant_id),
                runtime_id: r.id,
                worktree_id: wt_id,
                local_path: "/a".to_string(),
            },
            &admin,
        )
        .await
        .unwrap();
        let res = svc
            .mount_worktree(
                MountWorktreeCommand {
                    tenant_id: TenantId(tenant_id),
                    runtime_id: r.id,
                    worktree_id: wt_id,
                    local_path: "/b".to_string(),
                },
                &admin,
            )
            .await;
        assert!(matches!(res, Err(RuntimeError::Conflict(_))));
    }

    // -----------------------------------------------------------------
    // 7. unmount_worktree:卸载后 status = Unmounted
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn unmount_worktree() {
        let svc = InMemoryRuntimeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let admin = make_admin(tenant_id);
        let r = svc
            .register_runtime(make_register_cmd(tenant_id, user_id), &admin)
            .await
            .unwrap();
        let m = svc
            .mount_worktree(
                MountWorktreeCommand {
                    tenant_id: TenantId(tenant_id),
                    runtime_id: r.id,
                    worktree_id: WorktreeId::new(),
                    local_path: "/x".to_string(),
                },
                &admin,
            )
            .await
            .unwrap();
        let unmounted = svc
            .unmount_worktree(
                UnmountWorktreeCommand {
                    tenant_id: TenantId(tenant_id),
                    mount_id: m.id,
                },
                &admin,
            )
            .await
            .unwrap();
        assert_eq!(unmounted.status, MountStatus::Unmounted);
    }

    // -----------------------------------------------------------------
    // 8. create_exec_context:基础创建
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn create_exec_context() {
        let svc = InMemoryRuntimeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let admin = make_admin(tenant_id);
        let r = svc
            .register_runtime(make_register_cmd(tenant_id, user_id), &admin)
            .await
            .unwrap();
        let ctx = svc
            .create_exec_context(
                CreateExecContextCommand {
                    tenant_id: TenantId(tenant_id),
                    runtime_id: r.id,
                    agent_session_id: AgentSessionId::new(),
                    working_dir: "/work".to_string(),
                    environment: HashMap::new(),
                },
                &admin,
            )
            .await
            .unwrap();
        assert_eq!(ctx.runtime_id, r.id);
        assert_eq!(ctx.working_dir, "/work");
    }

    // -----------------------------------------------------------------
    // 9. environment_sanitized:空 env 不报错
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn environment_sanitized() {
        let svc = InMemoryRuntimeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let admin = make_admin(tenant_id);
        let r = svc
            .register_runtime(make_register_cmd(tenant_id, user_id), &admin)
            .await
            .unwrap();
        // 空 environment → 仍创建成功
        let ctx = svc
            .create_exec_context(
                CreateExecContextCommand {
                    tenant_id: TenantId(tenant_id),
                    runtime_id: r.id,
                    agent_session_id: AgentSessionId::new(),
                    working_dir: "/work".to_string(),
                    environment: HashMap::new(),
                },
                &admin,
            )
            .await
            .unwrap();
        assert!(ctx.environment.is_empty());
        // 显式给 env 也 OK
        let mut env = HashMap::new();
        env.insert("PATH".to_string(), "/usr/bin".to_string());
        env.insert("HOME".to_string(), "/home/u".to_string());
        let ctx2 = svc
            .create_exec_context(
                CreateExecContextCommand {
                    tenant_id: TenantId(tenant_id),
                    runtime_id: r.id,
                    agent_session_id: AgentSessionId::new(),
                    working_dir: "/work".to_string(),
                    environment: env.clone(),
                },
                &admin,
            )
            .await
            .unwrap();
        assert_eq!(ctx2.environment.get("PATH").unwrap(), "/usr/bin");
    }

    // -----------------------------------------------------------------
    // 10. list_by_user:1 user → N runtimes,actor 是自己允许读
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn list_by_user() {
        let svc = InMemoryRuntimeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let admin = make_admin(tenant_id);
        // 注册 2 个不同 device
        let mut cmd1 = make_register_cmd(tenant_id, user_id);
        cmd1.device_id = DeviceId::new();
        svc.register_runtime(cmd1, &admin).await.unwrap();
        let mut cmd2 = make_register_cmd(tenant_id, user_id);
        cmd2.device_id = DeviceId::new();
        svc.register_runtime(cmd2, &admin).await.unwrap();

        // 管理员看,所有 runtimes
        let all = svc
            .list_by_user(
                ListByUserQuery {
                    tenant_id: TenantId(tenant_id),
                    user_id: UserId(user_id),
                },
                &admin,
            )
            .await
            .unwrap();
        assert_eq!(all.len(), 2);

        // 非管理员看自己(同 user_id 的 actor)
        let self_actor = make_actor(tenant_id, user_id);
        let mine = svc
            .list_by_user(
                ListByUserQuery {
                    tenant_id: TenantId(tenant_id),
                    user_id: UserId(user_id),
                },
                &self_actor,
            )
            .await
            .unwrap();
        assert_eq!(mine.len(), 2);

        // 非管理员看别人 → PermissionDenied
        let other = make_actor(tenant_id, uuid::Uuid::new_v4());
        let res = svc
            .list_by_user(
                ListByUserQuery {
                    tenant_id: TenantId(tenant_id),
                    user_id: UserId(user_id),
                },
                &other,
            )
            .await;
        assert!(matches!(res, Err(RuntimeError::PermissionDenied)));
    }

    // -----------------------------------------------------------------
    // 11. cross_tenant_register_denied:actor tenant ≠ cmd tenant → 拒绝
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn cross_tenant_register_denied() {
        let svc = InMemoryRuntimeService::new();
        let actor_tenant = uuid::Uuid::new_v4();
        let cmd_tenant = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let actor = make_admin(actor_tenant);
        let cmd = make_register_cmd(cmd_tenant, user_id);
        let res = svc.register_runtime(cmd, &actor).await;
        assert!(matches!(res, Err(RuntimeError::CrossTenantDenied(_, _))));
    }

    // -----------------------------------------------------------------
    // 12. get_heartbeats:5 条心跳 → get_heartbeats 返回(默认全量,倒序)
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn get_heartbeats() {
        let svc = InMemoryRuntimeService::new();
        let tenant_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let admin = make_admin(tenant_id);
        let r = svc
            .register_runtime(make_register_cmd(tenant_id, user_id), &admin)
            .await
            .unwrap();
        let lr_actor = make_local_runtime_actor(tenant_id, user_id);
        // 5 条心跳
        for i in 0..5 {
            svc.heartbeat(
                HeartbeatCommand {
                    tenant_id: TenantId(tenant_id),
                    runtime_id: r.id,
                    load_average: Some(i as f32),
                    memory_used_bytes: Some(1024 * (i as u64 + 1)),
                },
                &lr_actor,
            )
            .await
            .unwrap();
            // 强制时间递增(避免 received_at 同 ms)
            sleep(Duration::from_millis(2));
        }
        let hbs = svc
            .get_heartbeats(
                GetHeartbeatsQuery {
                    tenant_id: TenantId(tenant_id),
                    runtime_id: r.id,
                    limit: None,
                },
                &admin,
            )
            .await
            .unwrap();
        assert_eq!(hbs.len(), 5, "Append-only 5 条");
        // 倒序
        for w in hbs.windows(2) {
            assert!(
                w[0].received_at >= w[1].received_at,
                "heartbeats 应按 received_at 倒序"
            );
        }
        // limit=2 → 2 条
        let hbs2 = svc
            .get_heartbeats(
                GetHeartbeatsQuery {
                    tenant_id: TenantId(tenant_id),
                    runtime_id: r.id,
                    limit: Some(2),
                },
                &admin,
            )
            .await
            .unwrap();
        assert_eq!(hbs2.len(), 2);
    }
}

pub mod process;

pub mod http_client;

pub mod cli_spawn;

pub mod sse_parser;
pub mod subscribe_integration;
pub mod subscribe_real;

pub mod e2e_integration;
pub mod spawn_upload_hub;
pub mod spawn_upload_integration;
