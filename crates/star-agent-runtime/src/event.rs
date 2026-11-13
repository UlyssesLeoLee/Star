/*
@cypher schema=1 source_sha256=bdc5712398e0a139914118504f48ba48cf567b4ddfcb77c57e052f33163def2d
MERGE (self:File {path:"crates/star-agent-runtime/src/event.rs"})
@endcypher
*/
//! Event Driven + Mailbox (per SRS §26-§28 + 03-DD §4.2/§4.3)。
//!
//! 处理链 (per SRS §26): `Event → Event Bus → Event Router → Mailbox → Scheduler
//! → Agent Activation`。Agent 不通过持续轮询保持活跃。
//! 禁止 Agent A 持有 Agent B Object Reference (per SRS §27 REQ-EVT-002)。
//! 大消息走 PayloadRef, 不在 Agent 间 Clone (per SRS §28 REQ-PAYLOAD-001)。

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::time::SystemTime;

use crate::id::{AgentId, CorrelationId, EventId, MsgId, TenantId, TraceId};

/// 9 类事件源 (per SRS §26 REQ-EVT-001)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    /// 用户消息。
    UserMessage,
    /// Agent 消息。
    AgentMessage,
    /// Tool 结果。
    ToolResult,
    /// LLM 结果。
    LlmResult,
    /// MCP 事件。
    McpEvent,
    /// 定时器。
    Timer,
    /// Workflow 事件。
    WorkflowEvent,
    /// 系统事件。
    SystemEvent,
    /// 外部事件。
    ExternalEvent,
}

/// PayloadRef (per SRS §28 REQ-PAYLOAD-001): 大消息存 Payload Store, 只传引用。
#[derive(Debug, Clone)]
pub struct PayloadRef {
    /// Store ID。
    pub store_id: String,
    /// 内部 key。
    pub key: String,
    /// 大小 (quota 计量)。
    pub size_bytes: u64,
    /// TTL。
    pub ttl_seconds: u64,
}

/// 标准事件 (per SRS §27 REQ-EVT-002)。
#[derive(Debug, Clone)]
pub struct AgentEvent {
    /// Event ID。
    pub event_id: EventId,
    /// 源 agent (无 = 外部)。
    pub source_agent_id: Option<AgentId>,
    /// 目标 agent。
    pub target_agent_id: AgentId,
    /// Tenant。
    pub tenant_id: TenantId,
    /// 事件类型。
    pub event_type: EventType,
    /// 大消息引用。
    pub payload_ref: Option<PayloadRef>,
    /// 时间戳。
    pub timestamp: SystemTime,
    /// Trace ID (per SRS §62)。
    pub trace_id: TraceId,
    /// Correlation ID。
    pub correlation_id: Option<CorrelationId>,
}

impl AgentEvent {
    /// 创建事件 (event_id/trace_id 自动生成, timestamp=now)。
    pub fn new(
        source_agent_id: Option<AgentId>,
        target_agent_id: AgentId,
        tenant_id: TenantId,
        event_type: EventType,
    ) -> Self {
        Self {
            event_id: EventId::new(),
            source_agent_id,
            target_agent_id,
            tenant_id,
            event_type,
            payload_ref: None,
            timestamp: SystemTime::now(),
            trace_id: TraceId::new(),
            correlation_id: None,
        }
    }
}

/// Mailbox 消息 (per 03-DD §4.3): 只存信封, 大消息不内联。
#[derive(Debug, Clone)]
pub struct MailboxMessage {
    /// 消息 ID。
    pub msg_id: MsgId,
    /// 来源事件 ID。
    pub event_id: EventId,
    /// 大消息引用。
    pub payload_ref: Option<PayloadRef>,
    /// 时间戳。
    pub timestamp: SystemTime,
    /// 已读。
    pub read: bool,
}

impl MailboxMessage {
    /// 由事件生成信封。
    pub fn from_event(event: &AgentEvent) -> Self {
        Self {
            msg_id: MsgId::new(),
            event_id: event.event_id,
            payload_ref: event.payload_ref.clone(),
            timestamp: event.timestamp,
            read: false,
        }
    }
}

/// EventBus 错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventBusError {
    /// 订阅不存在。
    NoSuchSubscription(AgentId),
}

impl fmt::Display for EventBusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventBusError::NoSuchSubscription(id) => write!(f, "agent {id} 无订阅"),
        }
    }
}

impl std::error::Error for EventBusError {}

/// EventBus 接口 (per 02-BD §5.3)。
pub trait EventBus {
    /// 发布事件 → 目标 mailbox, 返回 event_id。
    fn publish(&mut self, event: AgentEvent) -> EventId;
    /// 订阅 (sub_id 由实现分配)。
    fn subscribe(&mut self, agent_id: AgentId);
    /// 退订。
    fn unsubscribe(&mut self, agent_id: AgentId) -> Result<(), EventBusError>;
}

/// EventRouter (per 02-BD §5.3): 路由到 target agent。
pub trait EventRouter {
    /// route(event) → target agent id。
    fn route(&self, event: &AgentEvent) -> AgentId;
}

/// 默认路由: 按 target_agent_id。
#[derive(Debug, Default, Clone, Copy)]
pub struct TargetRouter;

impl EventRouter for TargetRouter {
    fn route(&self, event: &AgentEvent) -> AgentId {
        event.target_agent_id
    }
}

/// 进程内 EventBus (bounded mailbox, per SRS §44 禁止无界)。
#[derive(Debug)]
pub struct InMemoryEventBus {
    mailboxes: HashMap<AgentId, VecDeque<MailboxMessage>>,
    max_size: usize,
}

impl InMemoryEventBus {
    /// 新建 (mailbox 上限 max_size)。
    pub fn new(max_size: usize) -> Self {
        assert!(
            max_size > 0,
            "max_size 必须 >= 1 (禁止无界 mailbox, per SRS §44)"
        );
        Self {
            mailboxes: HashMap::new(),
            max_size,
        }
    }

    /// 未读数 (per MailboxRef.unread_count)。
    pub fn unread_count(&self, agent_id: AgentId) -> usize {
        self.mailboxes
            .get(&agent_id)
            .map(|m| m.iter().filter(|msg| !msg.read).count())
            .unwrap_or(0)
    }

    /// 取未读并标记已读。
    pub fn drain_unread(&mut self, agent_id: AgentId) -> Vec<MailboxMessage> {
        let Some(mailbox) = self.mailboxes.get_mut(&agent_id) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for msg in mailbox.iter_mut() {
            if !msg.read {
                msg.read = true;
                out.push(msg.clone());
            }
        }
        out
    }
}

impl EventBus for InMemoryEventBus {
    fn publish(&mut self, event: AgentEvent) -> EventId {
        let target = TargetRouter.route(&event);
        let mailbox = self.mailboxes.entry(target).or_default();
        mailbox.push_back(MailboxMessage::from_event(&event));
        // bounded: 满时丢最旧 (per SRS §44)
        while mailbox.len() > self.max_size {
            mailbox.pop_front();
        }
        event.event_id
    }

    fn subscribe(&mut self, agent_id: AgentId) {
        self.mailboxes.entry(agent_id).or_default();
    }

    fn unsubscribe(&mut self, agent_id: AgentId) -> Result<(), EventBusError> {
        if self.mailboxes.remove(&agent_id).is_some() {
            Ok(())
        } else {
            Err(EventBusError::NoSuchSubscription(agent_id))
        }
    }
}
