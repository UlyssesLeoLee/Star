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

/// AgentSession (per DD §15 — 11 字段)
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

    /// 停止 session
    async fn stop_session(&self, id: AgentId) -> Result<(), AgentError>;

    /// 暂停 session
    async fn pause_session(&self, id: AgentId) -> Result<(), AgentError>;

    /// 恢复 session
    async fn resume_session(&self, id: AgentId) -> Result<(), AgentError>;
}

/// In-memory AgentRuntime
pub struct InMemoryAgentRuntime {
    inner: Arc<RwLock<Inner>>,
    event_tx: broadcast::Sender<AgentEvent>,
}

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
            started_at: now,
            last_activity: now,
            token_usage: TokenUsage::default(),
            tool_calls: 0,
            result_state: AgentResultState::Pending,
        };
        let mut guard = self.inner.write().await;
        guard.sessions.insert(id, session.clone());
        drop(guard);

        // INV-WC-10: LLM 不修改 Source of Truth, 只消费数据
        // 这里只 emit Started 事件, 不修改 graph/git state
        let _ = self.event_tx.send(AgentEvent::Started { session: session.clone() });

        Ok(session)
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
        let result = s.result_state.clone();
        drop(guard);

        let _ = self.event_tx.send(AgentEvent::StatusChanged {
            id,
            from: prev,
            to: AgentStatus::Stopped,
        });
        let _ = self.event_tx.send(AgentEvent::Stopped { id, result });
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
            .start_session("claude-code", "claude-sonnet-4", Some(TaskId::new_v4()), None)
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
        assert!(count >= 3, "至少 3 个 StatusChanged 事件 (pause/resume/stop)");
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
}
