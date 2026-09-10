//! `agent-domain`: A1-A10 agent 管理 域 (per P3-D.6 实施计划 §4.1)
//!
//! 本 crate 是 P3-D.6 启动实装 阶段 1 基础 任务 1.1, 阶段 1 基础 任务 1.1 骨架落地,
//! 阶段 2 业务 实装阶段 落地 业务方法 (跨 batch 1 + batch 2 + 跨 session 续做).
//!
//! 派生自 `docs/design/DD-CANVAS-AGENT-001.md` v0.1 §3.1 module 布局 + §4.1 `AgentNode` struct 14 字段.
//! 5 域 Lead 真人未到位, Mavis 临时代签 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D +
//! 9/10 12:45 JST 守门 #14 v4 反转 v0.62 Mavis 审核 author=Ulysses 永久代签).
//!
//! V0.4 batch 2 阶段 2 任务 2.1 (per P3-D.6 实施计划 §3 阶段 2 业务 任务 2.1):
//! - 3 module 业务实装: handoff.rs (A2.1) + topology.rs (A2.2) + status_sync.rs (A3.1 + A3.3)
//! - 0 业务方法 (留 batch 3+ 续做 move/resize/rotate/...)
//!
//! V0.3 batch 1 阶段 2 任务 2.1 (5 Agent 业务方法 + 2 新类型 + 18 tests)
//! V0.2 阶段 2 任务 2.6 强类型层 (5 enum + Agent struct 14 字段 + 12 tests)
//! V0.1 阶段 1 基础 任务 1.1 骨架 (AgentDomainError 1 变体 + AgentNode 14 字段弱类型 + 2 tests)

#![warn(missing_docs)]

pub mod handoff;
pub mod models;
pub mod parent_child;
pub mod pipeline;
pub mod status_sync;
pub mod topology;
pub mod workitem_assoc;
pub mod worktree_assoc;

pub use handoff::{HandoffConnector, HandoffError};
pub use models::agent::{
    update_trust_score, Agent, AgentDomainError, AgentKind, AgentNode, AgentRole, AgentState,
    Domain, TrustScoreTier,
};
pub use parent_child::{ParentChildConnector, ParentChildError};
pub use pipeline::{PipelineConnector, PipelineError};
pub use status_sync::{StatusSync, StatusSyncError};
pub use topology::{DomainFrame, FrameError};
pub use workitem_assoc::{WorkItemAssocError, WorkItemDragIn};
pub use worktree_assoc::{WorktreeAssocError, WorktreeRing};
