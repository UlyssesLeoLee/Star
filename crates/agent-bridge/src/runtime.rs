//! `runtime.rs` — `AgentRuntime` Trait + InMemoryAgentRuntime (per DD §15)
//!
//! 8 方法:
//! 1. `get_session`        — 单 session
//! 2. `list_sessions`      — 列出
//! 3. `subscribe_events`   — 订阅事件流
//! 4. `get_session_metrics`— session 指标
//! 5. `start_session`      — 启动
//! 6. `stop_session`       — 停止
//! 7. `pause_session`      — 暂停
//! 8. `resume_session`     — 恢复

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures_util::stream::{self, BoxStream};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use graph_core::state::{AgentResultState, AgentStatus};
use graph_core::types::{AgentId, TaskId, WorktreeId};

use crate::error::AgentError;

/// Terminal pane UUID (per FR-ORCA-012 Agent Session 三元组)
///
/// ULIS-197 新增: 1 个 CLI agent × 1 个 terminal × 1 个 worktree 三元组中的
/// terminal 维度独立强类型 ID。
///
/// 复用 `define_uuid_id!` 宏生成的 22 domain 共享结构 (per `domain-local-runtime`
/// §lib.rs 注释), 不引新依赖。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TerminalId(pub Uuid);

impl TerminalId {
    /// 生成新的随机 TerminalId (UUID v4)
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    /// 返回内部原始 Uuid
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for TerminalId {
    /// Default = new random UUID (与其他 define_uuid_id! 22 个 domain 同款)
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for TerminalId {
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

impl std::fmt::Display for TerminalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Token usage (per DD §15)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenUsage {
    /// Input tokens
    pub input: u64,
    /// Output tokens
    pub output: u64,
    /// Total
    pub total: u64,
}

impl TokenUsage {
    /// 构造并累计
    pub fn add(&mut self, input: u64, output: u64) {
        self.input += input;
        self.output += output;
        self.total += input + output;
    }
}

/// AgentSession (per DD §15 + ULYS-197 v1 → v2 字段增量)
///
/// 字段数 v1 → v2: 11 → 13 字段
/// - `terminal_id: Option<TerminalId>` — per FR-ORCA-012 Agent Session 单一概念模型
///   "1 个 CLI agent × 1 个 terminal × 1 个 worktree" 三元组中的 terminal 维度
///   显式记录, 让 UI 在三个层级(worktree card / tab / terminal pane)
///   显示同一 session 的 state
/// - `restart_available: bool` — per FR-ORCA-014 Restart Chip "退而不死",
///   agent 退出(clean or crash)后 UI 显示 Restart chip; 点击 rehydrate
///   同一 agent 同 working dir(Codex 还保留当前 account)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    /// Agent UUID
    pub id: AgentId,
    /// Agent type (e.g. "claude-code", "codex", "multica")
    pub agent_type: String,
    /// Model (e.g. "claude-sonnet-4", "gpt-5")
    pub model: String,
    /// Agent status (14 态 per DD §15)
    pub status: AgentStatus,
    /// Current task
    pub task_id: Option<TaskId>,
    /// Current worktree (WORKS_ON edge)
    pub worktree_id: Option<WorktreeId>,
    /// Terminal pane UUID (per FR-ORCA-012 Agent Session 三元组 = CLI agent × terminal × worktree)
    pub terminal_id: Option<TerminalId>,
    /// Started at
    pub started_at: DateTime<Utc>,
    /// Last activity
    pub last_activity: DateTime<Utc>,
    /// Token usage
    pub token_usage: TokenUsage,
    /// Tool call count
    pub tool_calls: u32,
    /// Result state
    pub result_state: AgentResultState,
    /// Restart chip available (per FR-ORCA-014)
    ///
    /// 设为 `true` 后 UI 显示 "Restart" chip;
    /// 用户点击 Restart 后由调用方重置为 `false` 并 emit `AgentEvent::Restarted`。
    /// 默认 `false`(active session 不显示 chip)。
    pub restart_available: bool,
}

/// Agent filter (per DD §15)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentFilter {
    /// 限定 agent_type
    pub agent_type: Option<String>,
    /// 限定 status 集合
    pub status: Option<Vec<AgentStatus>>,
    /// 限定 worktree_id
    pub worktree_id: Option<WorktreeId>,
    /// 限定 task_id
    pub task_id: Option<TaskId>,
    /// Limit
    pub limit: Option<u32>,
}

/// Agent metrics (per DD §15 get_session_metrics)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    /// Agent ID
    pub agent_id: AgentId,
    /// Total tool calls
    pub tool_calls: u32,
    /// Total tokens
    pub total_tokens: u64,
    /// Duration seconds
    pub duration_secs: i64,
    /// Last activity
    pub last_activity: DateTime<Utc>,
}

/// Agent event (per DD §15 — 5 类)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AgentEvent {
    /// Started
    Started {
        /// Session
        session: AgentSession,
    },
    /// Stopped
    Stopped {
        /// Agent ID
        id: AgentId,
        /// Final result
        result: AgentResultState,
    },
    /// Tool call
    ToolCall {
        /// Agent ID
        id: AgentId,
        /// Tool name
        tool: String,
        /// Tool args (JSON)
        args: serde_json::Value,
    },
    /// Status changed
    StatusChanged {
        /// Agent ID
        id: AgentId,
        /// From
        from: AgentStatus,
        /// To
        to: AgentStatus,
    },
    /// Activity tick
    ActivityTick {
        /// Agent ID
        id: AgentId,
        /// Last activity time
        last_activity: DateTime<Utc>,
    },
    /// **ULYS-197 v1 → v2 新增**: Restart chip available (per FR-ORCA-014)
    ///
    /// Agent 退出(clean or crash)后 emit, 通知 UI 展示 Restart chip。
    /// `restart_available` 字段此时同步置为 `true`。
    RestartAvailable {
        /// Agent ID
        id: AgentId,
        /// 之前 session 的 terminal_id (供 UI 决策重启哪个 pane)
        terminal_id: Option<TerminalId>,
    },
    /// **ULYS-197 v1 → v2 新增**: Restart consumed (per FR-ORCA-014)
    ///
    /// 用户点击 Restart chip 后, 调用方重置 `restart_available = false`
    /// 并 emit 此事件。供下游审计/event-bus 消费。
    RestartConsumed {
        /// Agent ID
        id: AgentId,
        /// 之前 session 的 terminal_id
        terminal_id: Option<TerminalId>,
    },
}

/// AgentRuntime trait (per DD §15 + BD D-AGENT-001)
#[async_trait]
pub trait AgentRuntime: Send + Sync {
    /// 取单 session
    async fn get_session(&self, id: AgentId) -> Result<AgentSession, AgentError>;

    /// 列出 sessions
    async fn list_sessions(&self, filter: AgentFilter) -> Result<Vec<AgentSession>, AgentError>;

    /// 订阅事件
    async fn subscribe_events(&self) -> Result<BoxStream<'static, AgentEvent>, AgentError>;

    /// 取 session metrics
    async fn get_session_metrics(&self, id: AgentId) -> Result<AgentMetrics, AgentError>;

    /// 启动 session
    async fn start_session(
        &self,
        agent_type: &str,
        model: &str,
        task_id: Option<TaskId>,
        worktree_id: Option<WorktreeId>,
    ) -> Result<AgentSession, AgentError>;

    /// 关联 session 到 terminal pane (per FR-ORCA-012 Agent Session 三元组)
    ///
    /// **ULYS-197 v1 → v2 新增**: 用于在 terminal pane 创建后, 把
    /// `AgentSession.terminal_id` 字段填上. 调用方负责保证
    /// `terminal_id` 与 `worktree_id` 一致(都是同一 worktree 视图).
    ///
    /// 默认实现: 找到 session 并就地把 `terminal_id` 字段写上,
    /// 不发事件(emit 由 InMemoryAgentRuntime 重写以带 `RestartAvailable`
    /// 区分逻辑, 详见 trait 默认方法的 `Default impl` 区块)。
    async fn set_terminal_id(
        &self,
        id: AgentId,
        terminal_id: TerminalId,
    ) -> Result<AgentSession, AgentError>;

    /// 停止 session
    async fn stop_session(&self, id: AgentId) -> Result<(), AgentError>;

    /// 暂停 session
    async fn pause_session(&self, id: AgentId) -> Result<(), AgentError>;

    /// 恢复 session
    async fn resume_session(&self, id: AgentId) -> Result<(), AgentError>;
}

/// In-memory AgentRuntime
#[derive(Clone)]
pub struct InMemoryAgentRuntime {
    inner: Arc<RwLock<Inner>>,
    event_tx: broadcast::Sender<AgentEvent>,
}

// (ULIS-197 Clone derive 自动生成, 不再写手动 impl —
//  两字段均为 Arc-派生 Clone, derive 已足够)

struct Inner {
    sessions: HashMap<AgentId, AgentSession>,
}

impl InMemoryAgentRuntime {
    /// 创建新 runtime
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1024);
        Self {
            inner: Arc::new(RwLock::new(Inner {
                sessions: HashMap::new(),
            })),
            event_tx,
        }
    }
}

impl Default for InMemoryAgentRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentRuntime for InMemoryAgentRuntime {
    async fn get_session(&self, id: AgentId) -> Result<AgentSession, AgentError> {
        let guard = self.inner.read().await;
        guard
            .sessions
            .get(&id)
            .cloned()
            .ok_or_else(|| AgentError::not_found(id, "trace"))
    }

    async fn list_sessions(&self, filter: AgentFilter) -> Result<Vec<AgentSession>, AgentError> {
        let guard = self.inner.read().await;
        let mut out: Vec<AgentSession> = guard.sessions.values().cloned().collect();

        if let Some(at) = filter.agent_type {
            out.retain(|s| s.agent_type == at);
        }
        if let Some(statuses) = filter.status {
            out.retain(|s| statuses.contains(&s.status));
        }
        if let Some(wid) = filter.worktree_id {
            out.retain(|s| s.worktree_id == Some(wid));
        }
        if let Some(tid) = filter.task_id {
            out.retain(|s| s.task_id == Some(tid));
        }
        if let Some(limit) = filter.limit {
            out.truncate(limit as usize);
        }
        Ok(out)
    }

    async fn subscribe_events(&self) -> Result<BoxStream<'static, AgentEvent>, AgentError> {
        let rx = self.event_tx.subscribe();
        Ok(Box::pin(stream::unfold(rx, |mut rx| async move {
            rx.recv().await.ok().map(|e| (e, rx))
        })))
    }

    async fn get_session_metrics(&self, id: AgentId) -> Result<AgentMetrics, AgentError> {
        let guard = self.inner.read().await;
        let s = guard
            .sessions
            .get(&id)
            .ok_or_else(|| AgentError::not_found(id, "trace"))?;
        let duration_secs = (Utc::now() - s.started_at).num_seconds();
        Ok(AgentMetrics {
            agent_id: id,
            tool_calls: s.tool_calls,
            total_tokens: s.token_usage.total,
            duration_secs,
            last_activity: s.last_activity,
        })
    }

    async fn start_session(
        &self,
        agent_type: &str,
        model: &str,
        task_id: Option<TaskId>,
        worktree_id: Option<WorktreeId>,
    ) -> Result<AgentSession, AgentError> {
        let id = AgentId::new_v4();
        let now = Utc::now();
        let session = AgentSession {
            id,
            agent_type: agent_type.to_string(),
            model: model.to_string(),
            status: AgentStatus::Starting,
            task_id,
            worktree_id,
            terminal_id: None, // ULIS-197 v2: 调用方后续通过 `set_terminal_id()` 关联
            started_at: now,
            last_activity: now,
            token_usage: TokenUsage::default(),
            tool_calls: 0,
            result_state: AgentResultState::Pending,
            restart_available: false, // ULIS-197 v2: 默认 false, active session 不显示 chip
        };
        let mut guard = self.inner.write().await;
        guard.sessions.insert(id, session.clone());
        drop(guard);

        // INV-WC-10: LLM 不修改 Source of Truth, 只消费数据
        // 这里只 emit Started 事件, 不修改 graph/git state
        let _ = self.event_tx.send(AgentEvent::Started {
            session: session.clone(),
        });

        Ok(session)
    }

    async fn set_terminal_id(
        &self,
        id: AgentId,
        terminal_id: TerminalId,
    ) -> Result<AgentSession, AgentError> {
        let mut guard = self.inner.write().await;
        let s = guard
            .sessions
            .get_mut(&id)
            .ok_or_else(|| AgentError::not_found(id, "trace"))?;
        s.terminal_id = Some(terminal_id);
        let out = s.clone();
        drop(guard);
        Ok(out)
    }

    async fn stop_session(&self, id: AgentId) -> Result<(), AgentError> {
        let mut guard = self.inner.write().await;
        let s = guard
            .sessions
            .get_mut(&id)
            .ok_or_else(|| AgentError::not_found(id, "trace"))?;
        let prev = s.status;
        s.status = AgentStatus::Stopped;
        s.last_activity = Utc::now();
        let result = s.result_state;
        // ULIS-197 v2: agent 退出后置 restart_available = true (per FR-ORCA-014 Restart Chip)
        s.restart_available = true;
        let terminal_id = s.terminal_id;
        drop(guard);

        let _ = self.event_tx.send(AgentEvent::StatusChanged {
            id,
            from: prev,
            to: AgentStatus::Stopped,
        });
        let _ = self.event_tx.send(AgentEvent::Stopped { id, result });
        // ULIS-197 v2: emit RestartAvailable (通知 UI 展示 chip)
        let _ = self
            .event_tx
            .send(AgentEvent::RestartAvailable { id, terminal_id });
        Ok(())
    }

    async fn pause_session(&self, id: AgentId) -> Result<(), AgentError> {
        let mut guard = self.inner.write().await;
        let s = guard
            .sessions
            .get_mut(&id)
            .ok_or_else(|| AgentError::not_found(id, "trace"))?;
        let prev = s.status;
        s.status = AgentStatus::Paused;
        s.last_activity = Utc::now();
        drop(guard);

        let _ = self.event_tx.send(AgentEvent::StatusChanged {
            id,
            from: prev,
            to: AgentStatus::Paused,
        });
        Ok(())
    }

    async fn resume_session(&self, id: AgentId) -> Result<(), AgentError> {
        let mut guard = self.inner.write().await;
        let s = guard
            .sessions
            .get_mut(&id)
            .ok_or_else(|| AgentError::not_found(id, "trace"))?;
        let prev = s.status;
        s.status = AgentStatus::Thinking;
        s.last_activity = Utc::now();
        drop(guard);

        let _ = self.event_tx.send(AgentEvent::StatusChanged {
            id,
            from: prev,
            to: AgentStatus::Thinking,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph_core::types::TaskId;

    #[tokio::test]
    async fn agent_runtime_trait_8_methods() {
        // Per TEST-DESIGN §2.2: agent_runtime_trait_8_methods
        let runtime = InMemoryAgentRuntime::new();

        // 1. start_session
        let s1 = runtime
            .start_session("claude-code", "claude-sonnet-4", None, None)
            .await
            .unwrap();
        assert_eq!(s1.status, AgentStatus::Starting);
        assert_eq!(s1.agent_type, "claude-code");

        // 2. get_session
        let got = runtime.get_session(s1.id).await.unwrap();
        assert_eq!(got.id, s1.id);

        // 3. list_sessions
        let all = runtime.list_sessions(AgentFilter::default()).await.unwrap();
        assert_eq!(all.len(), 1);

        // 4. subscribe_events (validate via 8. stop)
        let _stream = runtime.subscribe_events().await.unwrap();

        // 5. get_session_metrics
        let metrics = runtime.get_session_metrics(s1.id).await.unwrap();
        assert_eq!(metrics.agent_id, s1.id);

        // 6. pause
        runtime.pause_session(s1.id).await.unwrap();
        let paused = runtime.get_session(s1.id).await.unwrap();
        assert_eq!(paused.status, AgentStatus::Paused);

        // 7. resume
        runtime.resume_session(s1.id).await.unwrap();
        let resumed = runtime.get_session(s1.id).await.unwrap();
        assert_eq!(resumed.status, AgentStatus::Thinking);

        // 8. stop
        runtime.stop_session(s1.id).await.unwrap();
        let stopped = runtime.get_session(s1.id).await.unwrap();
        assert_eq!(stopped.status, AgentStatus::Stopped);
    }

    #[tokio::test]
    async fn agent_session_sync_14_states() {
        // Per TEST-DESIGN §2.2: agent_session_sync_14_states
        // graph-core::AgentStatus 共 14 态, 验证 sync 逻辑
        use graph_core::state::AgentStatus::*;
        let statuses = [
            Registered,
            Starting,
            Idle,
            Claimed,
            Thinking,
            ToolCalling,
            AwaitingTool,
            Streaming,
            AwaitingFeedback,
            Completed,
            Failed,
            Paused,
            Stopped,
            Crashed,
        ];
        assert_eq!(statuses.len(), 14, "AgentStatus 必须 14 态");

        // 验证每次状态变更都能 emit StatusChanged 事件
        let runtime = InMemoryAgentRuntime::new();
        let s = runtime
            .start_session(
                "claude-code",
                "claude-sonnet-4",
                Some(TaskId::new_v4()),
                None,
            )
            .await
            .unwrap();
        let mut events = runtime.subscribe_events().await.unwrap();
        // 触发 pause → resume → stop 状态变化
        let runtime_clone_start = s.id;
        tokio::spawn(async move {
            let r2 = runtime;
            r2.pause_session(runtime_clone_start).await.ok();
            r2.resume_session(runtime_clone_start).await.ok();
            r2.stop_session(runtime_clone_start).await.ok();
        });

        use futures_util::StreamExt;
        let mut count = 0;
        let timeout = tokio::time::sleep(std::time::Duration::from_millis(100));
        tokio::pin!(timeout);
        loop {
            tokio::select! {
                _ = timeout.as_mut() => break,
                evt = events.next() => {
                    if let Some(AgentEvent::StatusChanged { .. }) = evt {
                        count += 1;
                    }
                    if evt.is_none() { break; }
                }
            }
        }
        assert!(
            count >= 3,
            "至少 3 个 StatusChanged 事件 (pause/resume/stop)"
        );
    }

    #[test]
    fn agent_status_14_variants_complete() {
        // Per DD §15: AgentStatus 必须 14 态, 验证 enum 完整
        use graph_core::state::AgentStatus::*;
        let all = [
            Registered,
            Starting,
            Idle,
            Claimed,
            Thinking,
            ToolCalling,
            AwaitingTool,
            Streaming,
            AwaitingFeedback,
            Completed,
            Failed,
            Paused,
            Stopped,
            Crashed,
        ];
        assert_eq!(all.len(), 14);
    }

    #[test]
    fn token_usage_accumulates() {
        let mut t = TokenUsage::default();
        t.add(100, 50);
        assert_eq!(t.input, 100);
        assert_eq!(t.output, 50);
        assert_eq!(t.total, 150);
        t.add(200, 100);
        assert_eq!(t.input, 300);
        assert_eq!(t.output, 150);
        assert_eq!(t.total, 450);
    }

    // =================================================================
    // ULYS-197 新增测试 (per FR-ORCA-012 + FR-ORCA-014)
    // =================================================================

    #[test]
    fn terminal_id_serde_roundtrip() {
        // Per FR-ORCA-012: TerminalId 是 session 必备字段
        // 验证 serde(transparent) round-trip + Display 实现
        use std::str::FromStr;
        let raw = Uuid::new_v4();
        let tid = TerminalId::from(raw);
        assert_eq!(tid.as_uuid(), raw);

        let json = serde_json::to_string(&tid).expect("serialize");
        let back: TerminalId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(tid, back);

        // Display 实现
        assert_eq!(tid.to_string(), raw.to_string());

        // Uuid::from_str round-trip
        let parsed = Uuid::from_str(&tid.to_string()).expect("uuid from str");
        assert_eq!(parsed, raw);
    }

    #[tokio::test]
    async fn terminal_id_default_none_on_new_session() {
        // Per FR-ORCA-012: 新 session 默认 terminal_id = None,
        // 调用方后续通过 set_terminal_id() 关联 terminal pane.
        let runtime = InMemoryAgentRuntime::new();
        let s = runtime
            .start_session("claude-code", "claude-sonnet-4", None, None)
            .await
            .unwrap();
        assert!(s.terminal_id.is_none(), "默认 terminal_id = None");
        assert!(!s.restart_available, "默认 restart_available = false");
    }

    #[tokio::test]
    async fn set_terminal_id_writes_field_and_persists() {
        // set_terminal_id() 后字段持久(后续 get_session 读回)
        let runtime = InMemoryAgentRuntime::new();
        let s = runtime
            .start_session("codex", "gpt-5", None, None)
            .await
            .unwrap();

        let tid = TerminalId::new();
        let updated = runtime.set_terminal_id(s.id, tid).await.unwrap();
        assert_eq!(updated.terminal_id, Some(tid));

        // get_session 应该返回相同 terminal_id
        let got = runtime.get_session(s.id).await.unwrap();
        assert_eq!(got.terminal_id, Some(tid), "set 后 get_session 必须读回");
    }

    #[tokio::test]
    async fn stop_session_sets_restart_available_and_emits_event() {
        // Per FR-ORCA-014: agent 退出后 restart_available = true +
        // emit RestartAvailable event
        use futures_util::StreamExt;
        let runtime = InMemoryAgentRuntime::new();
        let s = runtime
            .start_session("claude-code", "claude-sonnet-4", None, None)
            .await
            .unwrap();
        let tid = TerminalId::new();
        runtime.set_terminal_id(s.id, tid).await.unwrap();

        let mut events = runtime.subscribe_events().await.unwrap();

        // 触发 stop
        let runtime_for_stop = runtime.clone();
        let id_for_stop = s.id;
        tokio::spawn(async move {
            runtime_for_stop.stop_session(id_for_stop).await.ok();
        });

        // 等待 RestartAvailable event 出现(最多 200ms)
        let mut saw_restart_available = false;
        let timeout = tokio::time::sleep(std::time::Duration::from_millis(200));
        tokio::pin!(timeout);
        loop {
            tokio::select! {
                _ = timeout.as_mut() => break,
                evt = events.next() => {
                    if let Some(AgentEvent::RestartAvailable { id, terminal_id }) = evt {
                        assert_eq!(id, s.id);
                        assert_eq!(terminal_id, Some(tid));
                        saw_restart_available = true;
                        break;
                    }
                    if evt.is_none() { break; }
                }
            }
        }
        assert!(
            saw_restart_available,
            "stop_session 必须 emit RestartAvailable event"
        );

        // get_session 必须看到 restart_available = true
        let got = runtime.get_session(s.id).await.unwrap();
        assert!(
            got.restart_available,
            "stop 后 restart_available 必须为 true"
        );
        assert_eq!(got.terminal_id, Some(tid), "terminal_id 字段保留");
    }
}
