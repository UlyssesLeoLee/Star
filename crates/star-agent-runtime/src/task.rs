/*
@cypher schema=1 source_sha256=e28ad0d0707252aefd6ae6c2a0cfaee1aa93bedfc87978ec14136cbacb0e1068
MERGE (self:File {path:"crates/star-agent-runtime/src/task.rs"})
@endcypher
*/
//! L0 Task (per 03-DD §5.1 task_queue 业务投影 + SRS §52 幂等)。

use std::time::SystemTime;

use crate::agent_type::AgentType;
use crate::components::Priority;
use crate::id::{CorrelationId, TaskId, TenantId, TraceId};

/// Task 队列状态 (per 03-DD §5.1 status 列)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    /// 待调度。
    Pending,
    /// 执行中。
    Running,
    /// 完成。
    Done,
    /// 失败 (retry_count < max_retry)。
    Failed,
    /// 死信 (进 DLQ, per 03-DD §5.2)。
    Dead,
}

/// L0 派发任务 (per 03-DD §5.1 schema)。
#[derive(Debug, Clone)]
pub struct Task {
    /// Task ID。
    pub task_id: TaskId,
    /// brief 内容/路径 (per 守门 #20)。
    pub brief: String,
    /// SA-01..SA-09。
    pub agent_type: AgentType,
    /// Tenant。
    pub tenant_id: TenantId,
    /// 优先级 (0=Critical, 4=Background, per 03-DD §5.1)。
    pub priority: Priority,
    /// 状态。
    pub status: TaskStatus,
    /// 重试计数。
    pub retry_count: u32,
    /// 最大重试 (per SRS §51)。
    pub max_retry: u32,
    /// 附加 payload (JSON)。
    pub payload: Option<String>,
    /// 创建时间。
    pub created_at: SystemTime,
    /// Trace ID (per SRS §62)。
    pub trace_id: TraceId,
    /// Correlation ID (幂等, per SRS §52)。
    pub correlation_id: Option<CorrelationId>,
    /// 操作 ID (幂等 key, per SRS §52)。
    pub operation_id: TraceId,
}

impl Task {
    /// 创建 pending 任务。
    pub fn new(
        brief: String,
        agent_type: AgentType,
        tenant_id: TenantId,
        priority: Priority,
    ) -> Self {
        Self {
            task_id: TaskId::new(),
            brief,
            agent_type,
            tenant_id,
            priority,
            status: TaskStatus::Pending,
            retry_count: 0,
            max_retry: 3,
            payload: None,
            created_at: SystemTime::now(),
            trace_id: TraceId::new(),
            correlation_id: None,
            operation_id: TraceId::new(),
        }
    }

    /// 失败处理 (per 03-DD §2.2 mark_failed): retry=true 且未超限 → Pending。
    pub fn mark_failed(&mut self, retry: bool) {
        self.retry_count += 1;
        if retry && self.retry_count < self.max_retry {
            self.status = TaskStatus::Pending;
        } else {
            self.status = TaskStatus::Dead;
        }
    }
}
