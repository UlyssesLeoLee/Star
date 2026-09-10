//! `agent-domain`: A1-A10 agent 管理 域 (per P3-D.6 实施计划 §4.1)
//!
//! 本 crate 是 P3-D.6 启动实装 阶段 1 基础 任务 1.1, 仅骨架 (0 业务逻辑),
//! 阶段 2 业务 实装阶段 才落地 A1-A10 28 项业务方法 (move/resize/rotate/handoff/...).
//!
//! 派生自 `docs/design/DD-CANVAS-AGENT-001.md` v0.1 §3.1 module 布局 + §4.1 `AgentNode` struct 14 字段.
//! 5 域 Lead 真人未到位, Mavis 临时代签 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D).
//! 真人到位后追溯签字覆盖修订历史 (per 守门 #1 禁回溯叙事).
//!
//! V0.2 阶段 2 任务 2.6 强类型层 (per P3-D.6 实施计划 §3 阶段 2 业务 任务 2.6):
//! - 5 enum 强类型 (AgentRole / AgentKind / Domain / AgentState / TrustScoreTier)
//! - `Agent` struct 14 字段 强类型 (per DD-CANVAS-AGENT-001 §4.14.1)
//! - helper: `AgentState::can_transition_to` (14 状态转移) +
//!   `TrustScoreTier::from_score` / `to_score_range` (5 档转换) +
//!   `update_trust_score` (success +0.01 / failure -0.05, clamp 0.0-1.0)
//!
//! V0.1 (AgentDomainError + AgentNode 14 字段 弱类型) 100% 保留 compat (per 守门 #1 禁回溯叙事
//! + 守门 #19 v19 累积规不破坏 V0.1). V0.2 仅**追加** 5 enum + Agent struct 强类型 + 12 tests.

#![warn(missing_docs)]

pub mod models;

pub use models::agent::{
    update_trust_score, Agent, AgentDomainError, AgentKind, AgentNode, AgentRole, AgentState,
    Domain, TrustScoreTier,
};
