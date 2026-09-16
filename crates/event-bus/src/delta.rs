//! `delta.rs` — Graph Delta Update (per DD §32)
//!
//! 增量更新: 不重算整个 Graph, 仅 patch 变化字段.
//!
//! 三类操作 (per DD §32):
//! - `UpsertNode { id, payload }`: 节点变更 (e.g. WorktreeChanged 触发 WorktreeNode upsert)
//! - `UpsertEdge { id, from, to, payload }`: 边变更 (e.g. RiskDetected 触发新 Edge)
//! - `InvalidateCache { key }`: 缓存失效 (e.g. WorktreeChanged → cache.invalidate("worktree:{id}"))

use graph_core::types::WorktreeId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Delta 操作 (per DD §32)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum DeltaOp {
    /// Upsert Node
    UpsertNode {
        /// Node ID
        id: String,
        /// Node payload (JSON)
        payload: serde_json::Value,
    },
    /// Upsert Edge
    UpsertEdge {
        /// Edge ID
        id: String,
        /// 起点 Node ID
        from: String,
        /// 终点 Node ID
        to: String,
        /// Edge payload (JSON)
        payload: serde_json::Value,
    },
    /// Invalidate Cache
    InvalidateCache {
        /// Cache key (e.g. "worktree:{id}", "health:{id}")
        key: String,
    },
}

/// 应用 Delta 到 Graph (per DD §32 step 1-4)
///
/// 实际生产环境调用 `GraphRepository::upsert_node` / `upsert_edge`,
/// 本期 T9 用 pure 数据结构模拟.
///
/// 返回: 受影响的 op 数量 (测试用)
pub fn apply_delta(ops: &[DeltaOp]) -> usize {
    ops.len()
}

/// 给定 WorktreeChanged 事件, 派生 DeltaOp 列表 (per DD §32 step 1-4)
///
/// - 1. cache.invalidate("worktree:{id}") + cache.invalidate("health:{id}") + cache.invalidate("risk:{id}")
/// - 2. GraphRepository.upsert_node(WorktreeNode)
/// - 3. 若 changed_fields 包含 human_state / behind, 触发 Risk 检测 (返回 UpsertEdge delta)
/// - 4. 若 changed_fields 包含 behind / ahead / test_state / dirty, 触发 Health 重算
/// - 5. SSE publish
pub fn delta_for_worktree_changed(
    worktree_id: WorktreeId,
    changed_fields: &[String],
    node_payload: serde_json::Value,
) -> Vec<DeltaOp> {
    let mut ops = vec![
        // 1. 缓存失效
        DeltaOp::InvalidateCache {
            key: format!("worktree:{worktree_id}"),
        },
        DeltaOp::InvalidateCache {
            key: format!("health:{worktree_id}"),
        },
        DeltaOp::InvalidateCache {
            key: format!("risk:{worktree_id}"),
        },
        // 2. Graph node upsert
        DeltaOp::UpsertNode {
            id: worktree_id.to_string(),
            payload: node_payload,
        },
    ];

    // 3. Risk 增量触发 (per DD §32 step 3)
    let risk_relevant: &[&str] = &["human_state", "behind", "ahead"];
    if changed_fields
        .iter()
        .any(|f| risk_relevant.contains(&f.as_str()))
    {
        // 派生 Risk Edge (UpsertEdge delta) — 仅占位, 实装在 P2 risk-engine
        ops.push(DeltaOp::UpsertEdge {
            id: format!("risk:{}:{}", worktree_id, Uuid::new_v4()),
            from: worktree_id.to_string(),
            to: "risk-target".to_string(),
            payload: serde_json::json!({"trigger": "worktree_changed"}),
        });
    }

    ops
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_apply_returns_count() {
        let ops = vec![
            DeltaOp::InvalidateCache {
                key: "worktree:wt-1".to_string(),
            },
            DeltaOp::UpsertNode {
                id: "wt-1".to_string(),
                payload: serde_json::json!({}),
            },
        ];
        assert_eq!(apply_delta(&ops), 2);
    }

    #[test]
    fn delta_for_worktree_changed_includes_invalidations_and_upsert() {
        let ops = delta_for_worktree_changed(
            Uuid::new_v4(),
            &["behind".to_string()],
            serde_json::json!({"name": "wt-1"}),
        );
        // 3 invalidations + 1 upsert + 1 risk edge = 5
        assert_eq!(ops.len(), 5);
    }

    #[test]
    fn delta_no_risk_edge_if_no_relevant_field_changed() {
        let ops = delta_for_worktree_changed(
            Uuid::new_v4(),
            &["description".to_string()],
            serde_json::json!({}),
        );
        // 3 invalidations + 1 upsert = 4 (no risk edge)
        assert_eq!(ops.len(), 4);
    }
}
