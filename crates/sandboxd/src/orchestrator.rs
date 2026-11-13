//! `sandboxd` orchestrator 模块 (per FR-1.2 session 池化 + FR-4 session 生命周期, DD §2.1 / §4.1)
//!
//! - session 池: `DashMap<Uuid, Arc<Session>>` 并发安全 (per DD §2.1)
//! - 生命周期: Created → Running → Destroyed / Failed (per DD §4.1 状态机)
//! - 调度: FIFO + 容量拒绝 + 超时回收 (per FR-1.2)
//! - fail-closed: audit 写失败 → BLOCK session 创建 (per FR-7.2)
//! - StreamLogs: 1000 条环形缓冲 + broadcast 实时推送 (per FR-2.3)

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use futures_util::Stream;
use serde_json::json;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::audit::{AuditEvent, AuditSink};
use crate::backend::{
    BackendFactory, BackendHandle, CommandSpec, DefaultBackendFactory, RunResult,
};
use crate::error::SandboxdError;
use crate::metrics::SandboxdMetrics;
use crate::policy::{ActorIds, SessionSpec};
use crate::proto as pb;

/// 日志环形缓冲容量 (per FR-2.3: 1000 条, 满则 drop 老的)
pub const LOG_RING_CAPACITY: usize = 1000;

/// 超时回收扫描间隔 (秒)
pub const TIMEOUT_SWEEP_INTERVAL_SECS: u64 = 30;

/// session 状态 (per DD §4.1 状态机)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    /// 创建完成, 等待首个命令
    Created,
    /// 运行中 (首个 RunCommand 自动迁移, per DD §4.1)
    Running,
    /// 已销毁 (terminal)
    Destroyed,
    /// 失败 (terminal, 4 维策略 apply 失败 / PG INSERT 失败)
    Failed,
}

impl SessionStatus {
    /// 是否终态 (per DD §4.1: Destroyed / Failed 0 后续状态)
    pub fn is_terminal(&self) -> bool {
        matches!(self, SessionStatus::Destroyed | SessionStatus::Failed)
    }
}

/// session 日志环形缓冲 + 广播 (per FR-2.3)
pub struct LogRing {
    /// 环形缓冲 (offset 单调递增, capacity 满 pop_front)
    entries: Mutex<VecDeque<pb::LogEntry>>,
    /// 下一个 offset (全局单调)
    next_offset: AtomicU64,
    /// 缓冲容量
    capacity: usize,
    /// 实时广播 (StreamLogs 订阅)
    tx: broadcast::Sender<pb::LogEntry>,
}

impl LogRing {
    /// 构造环形缓冲 (capacity 默认 [`LOG_RING_CAPACITY`])
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self {
            entries: Mutex::new(VecDeque::with_capacity(capacity)),
            next_offset: AtomicU64::new(0),
            capacity,
            tx,
        }
    }

    /// 推入 1 条日志, 返回 offset (per FR-2.3 满则 drop 老的)
    pub fn push(
        &self,
        session_id: Uuid,
        event_type: pb::log_entry::EventType,
        details: String,
    ) -> u64 {
        let offset = self.next_offset.fetch_add(1, Ordering::SeqCst);
        let entry = pb::LogEntry {
            offset,
            session_id: session_id.to_string(),
            event_type: event_type as i32,
            details,
            timestamp_ms: Utc::now().timestamp_millis(),
        };
        {
            let mut entries = self.entries.lock().expect("log ring mutex poisoned");
            while entries.len() >= self.capacity {
                entries.pop_front();
            }
            entries.push_back(entry.clone());
        }
        let _ = self.tx.send(entry);
        offset
    }

    /// 从 from_offset 起重放已缓冲条目 (per BD §3.2 StreamLogs)
    pub fn replay(&self, from_offset: u64) -> Vec<pb::LogEntry> {
        let entries = self.entries.lock().expect("log ring mutex poisoned");
        entries
            .iter()
            .filter(|e| e.offset >= from_offset)
            .cloned()
            .collect()
    }

    /// 订阅实时日志流
    pub fn subscribe(&self) -> broadcast::Receiver<pb::LogEntry> {
        self.tx.subscribe()
    }
}

/// 1 个 sandbox session (per DD §2.1 Session struct)
pub struct Session {
    /// session 全局唯一 ID
    pub session_id: Uuid,
    /// 调用 client (dispatcher / mavis / 其他)
    pub client_id: String,
    /// 客户端 task 标识
    pub task_id: String,
    /// 父 session (嵌套)
    pub parent_session_id: Option<Uuid>,
    /// 创建时冻结的 4 维策略快照 (per INV-SBX-02)
    pub policy_snapshot: SessionSpec,
    /// session 状态 (per DD §4.1)
    pub status: Mutex<SessionStatus>,
    /// 创建时间 (append-only)
    pub created_at: DateTime<Utc>,
    /// 销毁时间 (mutable 标记)
    pub destroyed_at: Mutex<Option<DateTime<Utc>>>,
    /// 销毁原因 (正常完成 / 超时 / 失败)
    pub destroy_reason: Mutex<Option<String>>,
    /// 平台特定 backend handle
    pub backend_handle: Box<dyn BackendHandle>,
    /// RLS 13 類 (跟 ActorContext 对齐, per 守门 #13c)
    pub tenant_id: Uuid,
    /// RLS 13 類
    pub workspace_id: Uuid,
    /// RLS 13 類 (per D-7 未来注入 ARG Edge)
    pub actor_id: Uuid,
    /// session 日志环 (per FR-2.3)
    pub logs: LogRing,
    /// session 级超时 (秒, 默认 1800 per FR-4.3)
    pub timeout_sec: u32,
    /// 在飞命令数 (优雅停机用)
    pub in_flight: AtomicU64,
}

impl Session {
    /// 当前状态
    pub fn status(&self) -> SessionStatus {
        *self.status.lock().expect("status mutex poisoned")
    }
}

/// session 池化 + 生命周期 orchestrator (per FR-1.2 / FR-4)
pub struct Orchestrator {
    /// 并发安全 session 池 (per DD §2.1)
    sessions: Arc<DashMap<Uuid, Arc<Session>>>,
    /// 最大并发 session 数 (默认 100, per FR-1.2)
    max_concurrent: usize,
    /// session 默认超时 (秒)
    default_timeout_sec: u32,
    /// audit sink (fail-closed 联动, per FR-7.2)
    audit: Arc<dyn AuditSink>,
    /// Prometheus 指标
    metrics: Arc<SandboxdMetrics>,
    /// backend 工厂
    factory: Box<dyn BackendFactory>,
    /// RLS 13 類三元组 (v0.1 服务端注入)
    actor: ActorIds,
    /// daemon 启动时刻
    pub started_at: Instant,
    /// 累计 session 总数 (healthz sessions_total)
    total_sessions: AtomicU64,
    /// 优雅停机标记 (FR-1.3: 期间拒绝新 session, 返回 Unavailable)
    shutting_down: AtomicBool,
    /// 全局在飞命令数 (FR-1.3 宽限期判断)
    running_commands: AtomicU64,
}

/// StreamLogs 返回的 server streaming 流 (per BD §3.2)
pub type LogStream =
    std::pin::Pin<Box<dyn Stream<Item = Result<pb::LogEntry, tonic::Status>> + Send>>;

/// orchestrator 健康状态 (healthz 用)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestratorHealth {
    /// 是否健康
    pub healthy: bool,
    /// 详情
    pub detail: String,
}

impl Orchestrator {
    /// 构造 orchestrator
    pub fn new(
        max_concurrent: usize,
        default_timeout_sec: u32,
        audit: Arc<dyn AuditSink>,
        metrics: Arc<SandboxdMetrics>,
        factory: Box<dyn BackendFactory>,
        actor: ActorIds,
    ) -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            max_concurrent,
            default_timeout_sec,
            audit,
            metrics,
            factory,
            actor,
            started_at: Instant::now(),
            total_sessions: AtomicU64::new(0),
            shutting_down: AtomicBool::new(false),
            running_commands: AtomicU64::new(0),
        }
    }

    /// 默认工厂构造 (测试 / 内嵌用)
    pub fn with_defaults(
        max_concurrent: usize,
        default_timeout_sec: u32,
        audit: Arc<dyn AuditSink>,
        metrics: Arc<SandboxdMetrics>,
    ) -> Self {
        Self::new(
            max_concurrent,
            default_timeout_sec,
            audit,
            metrics,
            Box::new(DefaultBackendFactory),
            ActorIds::default(),
        )
    }

    /// 创建 session (per FR-4.1, DD §3.1 时序)
    ///
    /// 副作用: 校验策略 → audit (fail-closed) → backend apply 4 维隔离 → 入池。
    /// 失败: 资源不足 / 策略非法 / 平台不支持 → Err (server 层映射 FAILED + error)。
    pub async fn create_session(
        &self,
        client_id: String,
        task_id: String,
        parent_session_id: Option<Uuid>,
        policy: SessionSpec,
        timeout_sec: u32,
    ) -> Result<Uuid, SandboxdError> {
        // FR-1.3: 停机期间拒绝新 session (返回 Unavailable)
        if self.shutting_down.load(Ordering::SeqCst) {
            return Err(SandboxdError::Internal(
                "daemon shutting down: reject new sessions (Unavailable)".to_string(),
            ));
        }
        // DD §3.1 时序点 1: 策略合法性 fail-fast
        policy.validate()?;
        // FR-1.2: 容量拒绝
        if self.sessions.len() >= self.max_concurrent {
            self.metrics
                .sessions_total
                .with_label_values(&["rejected"])
                .inc();
            return Err(SandboxdError::CapacityExceeded {
                max: self.max_concurrent,
            });
        }
        let session_id = Uuid::new_v4();
        let timeout = if timeout_sec == 0 {
            self.default_timeout_sec
        } else {
            timeout_sec
        };
        let snapshot = serde_json::to_value(&policy)
            .map_err(|e| SandboxdError::Internal(format!("policy snapshot: {e}")))?;
        // DD §3.1 时序点 2: 持久化创建记录 (FR-7.2 fail-closed: 失败 → BLOCK 创建)
        let created_event = AuditEvent::session_event(
            session_id,
            &client_id,
            &task_id,
            parent_session_id,
            "Created",
            Some(snapshot),
            self.actor.tenant_id,
            self.actor.workspace_id,
            self.actor.actor_id,
        );
        if let Err(e) = self.audit.record(&created_event).await {
            self.metrics
                .sessions_total
                .with_label_values(&["failed"])
                .inc();
            return Err(e);
        }
        // DD §3.1 时序点 3: 应用 4 维策略 (最大开销)
        let backend = self.factory.create()?;
        let handle = match backend.apply(&policy, session_id).await {
            Ok(h) => h,
            Err(e) => {
                let failed_event = AuditEvent::session_event(
                    session_id,
                    &client_id,
                    &task_id,
                    parent_session_id,
                    "Failed",
                    None,
                    self.actor.tenant_id,
                    self.actor.workspace_id,
                    self.actor.actor_id,
                );
                let _ = self.audit.record(&failed_event).await;
                self.metrics
                    .sessions_total
                    .with_label_values(&["failed"])
                    .inc();
                return Err(e);
            }
        };
        let session = Arc::new(Session {
            session_id,
            client_id,
            task_id,
            parent_session_id,
            policy_snapshot: policy,
            status: Mutex::new(SessionStatus::Created),
            created_at: Utc::now(),
            destroyed_at: Mutex::new(None),
            destroy_reason: Mutex::new(None),
            backend_handle: handle,
            tenant_id: self.actor.tenant_id,
            workspace_id: self.actor.workspace_id,
            actor_id: self.actor.actor_id,
            logs: LogRing::new(LOG_RING_CAPACITY),
            timeout_sec: timeout,
            in_flight: AtomicU64::new(0),
        });
        self.sessions.insert(session_id, session);
        self.total_sessions.fetch_add(1, Ordering::SeqCst);
        self.metrics
            .sessions_total
            .with_label_values(&["created"])
            .inc();
        self.metrics.sessions_active.inc();
        tracing::info!(%session_id, "session created");
        Ok(session_id)
    }

    /// session 内跑命令 (per FR-4.2, DD §3.2 时序)
    ///
    /// 命中违规不阻断 (fail-open per FR-4.2): audit log + hit_violations 返回。
    /// capability 维提权拦截为 BLOCK (per FR-3.4 drop 语义, 见 backend 文档)。
    pub async fn run_command(
        &self,
        session_id: Uuid,
        cmd: &CommandSpec,
        timeout_sec: u32,
    ) -> Result<RunResult, SandboxdError> {
        let session = self
            .sessions
            .get(&session_id)
            .ok_or(SandboxdError::SessionNotFound(session_id))?
            .clone();
        // DD §4.1: Created → Running 首个命令自动迁移; 终态拒绝
        {
            let mut st = session.status.lock().expect("status mutex poisoned");
            match *st {
                SessionStatus::Destroyed | SessionStatus::Failed => {
                    return Err(SandboxdError::Internal(format!(
                        "session not runnable (status {st:?})"
                    )));
                }
                SessionStatus::Created => *st = SessionStatus::Running,
                SessionStatus::Running => {}
            }
        }
        // DD §3.2: audit ProcessStart (per 守门 #5: env 仅 KEY 名)
        let start_ev = AuditEvent::audit_event(
            session_id,
            &session.client_id,
            &session.task_id,
            "ProcessStart",
            None,
            Vec::new(),
            session.tenant_id,
            session.workspace_id,
            session.actor_id,
        );
        let _ = self.audit.record(&start_ev).await;
        session.logs.push(
            session_id,
            pb::log_entry::EventType::ProcessStart,
            json!({ "argv": cmd.cmd, "env_keys": cmd.env_keys(), "cwd": cmd.cwd }).to_string(),
        );
        session.in_flight.fetch_add(1, Ordering::SeqCst);
        self.running_commands.fetch_add(1, Ordering::SeqCst);
        let result = session.backend_handle.run(cmd, timeout_sec).await;
        self.running_commands.fetch_sub(1, Ordering::SeqCst);
        session.in_flight.fetch_sub(1, Ordering::SeqCst);
        let run_result = result?;
        // 违规分类 → metrics (per FR-6.1)
        self.classify_violations(&run_result.hit_violations);
        // DD §3.2: audit ProcessExit
        let exit_ev = AuditEvent::audit_event(
            session_id,
            &session.client_id,
            &session.task_id,
            "ProcessExit",
            Some(run_result.latency_ms as i64),
            run_result.hit_violations.clone(),
            session.tenant_id,
            session.workspace_id,
            session.actor_id,
        );
        let _ = self.audit.record(&exit_ev).await;
        session.logs.push(
            session_id,
            pb::log_entry::EventType::ProcessExit,
            json!({ "returncode": run_result.returncode, "latency_ms": run_result.latency_ms })
                .to_string(),
        );
        Ok(run_result)
    }

    /// 销毁 session (per FR-4.3, DD §3.3 时序): 级联 kill + 回收 + audit
    pub async fn destroy_session(
        &self,
        session_id: Uuid,
        reason: &str,
    ) -> Result<(), SandboxdError> {
        let (_, session) = self
            .sessions
            .remove(&session_id)
            .ok_or(SandboxdError::SessionNotFound(session_id))?;
        {
            let mut st = session.status.lock().expect("status mutex poisoned");
            *st = SessionStatus::Destroyed;
            *session.destroyed_at.lock().expect("destroyed_at poisoned") = Some(Utc::now());
            *session
                .destroy_reason
                .lock()
                .expect("destroy_reason poisoned") = Some(reason.to_string());
        }
        // FR-4.3: 级联 kill + 回收所有隔离
        session.backend_handle.destroy().await?;
        // metrics: destroyed counter + duration histogram (per FR-6.1)
        self.metrics
            .sessions_total
            .with_label_values(&["destroyed"])
            .inc();
        self.metrics.sessions_active.dec();
        let duration = Utc::now()
            .signed_duration_since(session.created_at)
            .num_milliseconds() as f64
            / 1000.0;
        self.metrics.session_duration_seconds.observe(duration);
        // audit (per FR-4.3)
        let ev = AuditEvent::session_event(
            session_id,
            &session.client_id,
            &session.task_id,
            session.parent_session_id,
            "Destroyed",
            None,
            session.tenant_id,
            session.workspace_id,
            session.actor_id,
        );
        let _ = self.audit.record(&ev).await;
        tracing::info!(%session_id, reason, "session destroyed");
        Ok(())
    }

    /// 订阅 session 日志流 (per FR-2.3): 环形缓冲重放 + broadcast 实时
    pub fn stream_logs(
        &self,
        session_id: Uuid,
        from_offset: u64,
    ) -> Result<LogStream, SandboxdError> {
        let session = self
            .sessions
            .get(&session_id)
            .ok_or(SandboxdError::SessionNotFound(session_id))?
            .clone();
        let replay: Vec<_> = session
            .logs
            .replay(from_offset)
            .into_iter()
            .map(Ok)
            .collect();
        let rx = session.logs.subscribe();
        let live = BroadcastStream::new(rx)
            .map(|r| r.map_err(|e| tonic::Status::internal(format!("log stream lagged: {e}"))));
        Ok(Box::pin(tokio_stream::iter(replay).chain(live)))
    }

    /// 查询 session 状态 (测试 / 诊断用; 不存在 → None)
    pub fn session_status(&self, session_id: Uuid) -> Option<SessionStatus> {
        self.sessions.get(&session_id).map(|s| s.status())
    }

    /// 活跃 session 数 (healthz sessions_active)
    pub fn active_count(&self) -> usize {
        self.sessions
            .iter()
            .filter(|entry| !entry.status().is_terminal())
            .count()
    }

    /// 累计 session 总数 (healthz sessions_total)
    pub fn total_count(&self) -> u64 {
        self.total_sessions.load(Ordering::SeqCst)
    }

    /// 健康状态 (per FR-6.3)
    pub fn health_status(&self) -> OrchestratorHealth {
        let shutting = self.shutting_down.load(Ordering::SeqCst);
        OrchestratorHealth {
            healthy: !shutting,
            detail: if shutting {
                "daemon shutting down".to_string()
            } else {
                format!("sessions: {}/{}", self.active_count(), self.max_concurrent)
            },
        }
    }

    /// 在飞命令数 (FR-1.3 宽限期判断)
    pub fn running_commands(&self) -> u64 {
        self.running_commands.load(Ordering::SeqCst)
    }

    /// 启动超时回收后台任务 (per FR-4.3: 30 min 默认 timeout 强制 destroy)
    ///
    /// 需 `Arc<Self>` (后台任务持有池引用)。
    pub fn spawn_timeout_sweeper(self: &Arc<Self>) -> tokio::task::JoinHandle<()> {
        let this = Arc::clone(self);
        tokio::spawn(async move {
            let mut ticker =
                tokio::time::interval(Duration::from_secs(TIMEOUT_SWEEP_INTERVAL_SECS));
            ticker.tick().await; // 首个 tick 立即触发
            loop {
                ticker.tick().await;
                if this.shutting_down.load(Ordering::SeqCst) {
                    return;
                }
                let now = Utc::now();
                let expired: Vec<Uuid> = this
                    .sessions
                    .iter()
                    .filter(|entry| {
                        let timeout = entry.timeout_sec as i64;
                        now.signed_duration_since(entry.created_at).num_seconds() > timeout
                    })
                    .map(|entry| *entry.key())
                    .collect();
                for id in expired {
                    tracing::info!(%id, "session timeout, force destroy");
                    if let Err(e) = this.destroy_session(id, "session_timeout").await {
                        tracing::warn!(%id, "timeout destroy failed: {e}");
                    }
                }
            }
        })
    }

    /// 优雅停机 (per FR-1.3): 拒绝新 session → 等在飞命令 (默认 30s grace) → 强制销毁残留
    pub async fn shutdown(self: &Arc<Self>, grace: Duration) {
        self.shutting_down.store(true, Ordering::SeqCst);
        tracing::info!(
            "graceful shutdown: draining in-flight commands (grace {:?})",
            grace
        );
        let deadline = Instant::now() + grace;
        loop {
            if self.running_commands() == 0 || Instant::now() >= deadline {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        let ids: Vec<Uuid> = self.sessions.iter().map(|e| *e.key()).collect();
        for id in ids {
            if let Err(e) = self.destroy_session(id, "daemon_shutdown").await {
                tracing::warn!(%id, "shutdown destroy failed: {e}");
            }
        }
        tracing::info!("graceful shutdown complete");
    }

    /// 违规分类 → metrics counter (per FR-6.1: 按 "dim:" 前缀分类)
    fn classify_violations(&self, violations: &[String]) {
        for v in violations {
            if let Some(rest) = v.strip_prefix("network:") {
                self.metrics
                    .network_violations_total
                    .with_label_values(&["blocked"])
                    .inc();
                tracing::debug!(violation = rest, "network violation");
            } else if let Some(rest) = v.strip_prefix("fs:") {
                self.metrics
                    .fs_violations_total
                    .with_label_values(&["blocked"])
                    .inc();
                tracing::debug!(violation = rest, "fs violation");
            } else if let Some(rest) = v.strip_prefix("capability:") {
                self.metrics
                    .capability_violations_total
                    .with_label_values(&["dropped"])
                    .inc();
                tracing::debug!(violation = rest, "capability violation");
            } else if let Some(rest) = v.strip_prefix("resource:") {
                let limit_type = if rest.contains("memory") {
                    "memory"
                } else if rest.contains("cpu") {
                    "cpu"
                } else if rest.contains("process") {
                    "processes"
                } else {
                    "other"
                };
                self.metrics
                    .resource_limit_hits_total
                    .with_label_values(&[limit_type])
                    .inc();
                tracing::debug!(violation = rest, "resource limit hit");
            }
        }
    }
}
