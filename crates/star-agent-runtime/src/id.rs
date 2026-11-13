/*
@cypher schema=1 source_sha256=b97d2c6747d15b3368f35855f2afa293ace29d597cb8ad8b5fb78fd7abfe74f2
MERGE (self:File {path:"crates/star-agent-runtime/src/id.rs"})
@endcypher
*/
//! Newtype ID 类型 (per 03-DD §2.1 类型别名)。
//!
//! 03-DD §2.1 定义 `pub type AgentId = Uuid;` 等 14 个别名。本骨架 std-only,
//! 以 `u128` + 32 位 hex `Display` 占位 (非加密随机: 时间戳 + 单调序列),
//! 后续注册进根 workspace 后替换为 `uuid` crate (结构不变, 仅替换 `id.rs`)。

use std::fmt;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// 生成 raw id: (unix nanos << 64) | 单调序列。非加密随机 (骨架占位)。
fn generate_raw() -> u128 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    ((nanos as u128) << 64) | (seq as u128)
}

/// ID 解析错误 (hex 32 位之外 / 非 hex)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseIdError(());

impl fmt::Display for ParseIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "非法 ID: 需 32 位 hex 字符串")
    }
}

impl std::error::Error for ParseIdError {}

macro_rules! newtype_id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub u128);

        impl $name {
            /// 生成新 ID。
            pub fn new() -> Self {
                Self(generate_raw())
            }

            /// 零值 ID (测试/哨兵)。
            pub const fn zero() -> Self {
                Self(0)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:032x}", self.0)
            }
        }

        impl FromStr for $name {
            type Err = ParseIdError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if s.len() != 32 {
                    return Err(ParseIdError(()));
                }
                let v = u128::from_str_radix(s, 16).map_err(|_| ParseIdError(()))?;
                Ok(Self(v))
            }
        }
    };
}

newtype_id!(
    /// Agent ID (per SRS §9 REQ-ECS-001)。
    AgentId
);
newtype_id!(
    /// Tenant ID (per SRS §9)。
    TenantId
);
newtype_id!(
    /// Context ID (per SRS §12 REQ-ECS-004)。
    ContextId
);
newtype_id!(
    /// Memory ID (per SRS §13 REQ-ECS-005)。
    MemoryId
);
newtype_id!(
    /// Policy ID (per SRS §15 REQ-ECS-007)。
    PolicyId
);
newtype_id!(
    /// ACL ID (per SRS §16 REQ-ECS-008)。
    AclId
);
newtype_id!(
    /// Mailbox ID (per SRS §19 REQ-ECS-011)。
    MailboxId
);
newtype_id!(
    /// Workflow ID (per SRS §20 REQ-ECS-012)。
    WorkflowId
);
newtype_id!(
    /// Checkpoint ID (per SRS §42 REQ-CHECKPOINT-001)。
    CheckpointId
);
newtype_id!(
    /// Trace ID (per SRS §62 REQ-TRACE-001)。
    TraceId
);
newtype_id!(
    /// Correlation ID (per SRS §27)。
    CorrelationId
);
newtype_id!(
    /// Event ID (per SRS §27)。
    EventId
);
newtype_id!(
    /// Mailbox 消息 ID (per 03-DD §4.3)。
    MsgId
);
newtype_id!(
    /// L0 Task ID (per 03-DD §5.1 task_queue)。
    TaskId
);
