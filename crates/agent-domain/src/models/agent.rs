//! `AgentNode` 域模型 (per DD-AGENT §4.1, 14 字段).
//!
//! 仅 struct 字段定义 + Default impl + derive, 0 业务方法 (move/resize/rotate 等留阶段 2 任务 2.1).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// `agent-domain` 域错误占位 (per DD-AGENT §4.7).
///
/// 阶段 1 基础 仅占位 1 变体, 阶段 2 业务 实装阶段 才完整扩展 (A1-A10 28 项).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentDomainError {
    /// 占位: 阶段 2 业务未实装.
    NotImplemented,
}

/// A1-A10 agent 节点 域模型 (per DD-AGENT §4.1, 14 字段).
///
/// 阶段 1 基础 仅字段定义, 阶段 2 任务 2.1 落地业务方法.
///
/// 注: 不用 `Eq` 因 `f32` 不实现 `Eq` (NaN != NaN).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentNode {
    /// 全局唯一 ID (v4 UUID).
    pub id: Uuid,
    /// 人类可读名称.
    pub name: String,
    /// 原型标签 (e.g. "PM" / "SRE" / "5-Domain-Lead").
    pub archetype: String,
    /// 所属域 (player/economy/match/social/admin, per 5 域 Lead 历史治理命名).
    pub domain: String,
    /// 状态机 14 状态之一 (per SRS-CANVAS-AGENT-001 §3.3 + DD-AGENT §5.2).
    pub status: String,
    /// 信任分 0.0-1.0, 5 档 (per ARG 源 Untrusted/Low/Medium/High/VeryHigh).
    pub trust_score: f32,
    /// 任意 JSON 元数据.
    pub metadata: serde_json::Value,
    /// 创建时间 (UTC).
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间 (UTC).
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// 乐观锁版本号 (per DD-AGENT §5.2 14 状态机 + §5.3 14 状态转移).
    pub version: i32,
    /// 画布 X 坐标.
    pub x: f32,
    /// 画布 Y 坐标.
    pub y: f32,
    /// 节点宽度.
    pub width: f32,
    /// 节点高度.
    pub height: f32,
    /// 节点旋转角度 (弧度).
    pub rotation: f32,
}

impl Default for AgentNode {
    fn default() -> Self {
        // 0 业务逻辑, 仅字段预填
        Self {
            id: Uuid::nil(),
            name: String::new(),
            archetype: String::new(),
            domain: String::new(),
            status: "queued".to_string(), // 14 状态机默认 1 状态 (per DD-AGENT §5.2)
            trust_score: 0.5,             // 5 档默认中位
            metadata: serde_json::Value::Null,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 1,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            rotation: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_node_default() {
        let node = AgentNode::default();
        assert_eq!(node.id, Uuid::nil());
        assert_eq!(node.status, "queued");
        assert_eq!(node.trust_score, 0.5);
        assert_eq!(node.version, 1);
        assert_eq!(node.x, 0.0);
    }

    #[test]
    fn test_agent_node_clone() {
        let node1 = AgentNode::default();
        let node2 = node1.clone();
        assert_eq!(node1, node2);
    }
}
