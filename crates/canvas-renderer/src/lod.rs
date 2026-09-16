//! `lod.rs` — Semantic Zoom LOD (per DD §29 + INV-WC-05)
//!
//! 3 级 (per ULYS-57.3 acceptance, 简化自 DD §29 6 级):
//! - L0 Project — Repository
//! - L1 Repo+Worktree — Repository + Branch + Worktree
//! - L2 Task+Agent — + Task + AgentSession + Commit + PullRequest + File + Symbol + TestRun + Issue
//!
//! 阈值 (per spec §4.4 INV-WC-05):
//! - zoom < 0.3  → L0
//! - 0.3 ≤ zoom < 0.7 → L1
//! - zoom ≥ 0.7 → L2

use crate::error::CanvasError;
use graph_core::node::NodeKind;
use serde::{Deserialize, Serialize};

/// LOD 级别 (3 级, per ULYS-57.3 acceptance)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LodLevel {
    /// L0: Project 层级 (Repository)
    L0Project,
    /// L1: Repo + Worktree 层级
    L1RepoWorktree,
    /// L2: Task + Agent 层级 (含更细粒度)
    L2TaskAgent,
}

impl LodLevel {
    /// 取得 LOD 级别序号 (0/1/2)
    pub fn level(self) -> u8 {
        match self {
            Self::L0Project => 0,
            Self::L1RepoWorktree => 1,
            Self::L2TaskAgent => 2,
        }
    }

    /// 字符串形式
    pub fn as_str(self) -> &'static str {
        match self {
            Self::L0Project => "L0_Project",
            Self::L1RepoWorktree => "L1_RepoWorktree",
            Self::L2TaskAgent => "L2_TaskAgent",
        }
    }
}

/// LOD 选择器 — 根据 zoom 选择 LOD 级别
#[derive(Debug, Default, Clone)]
pub struct LodSelector;

impl LodSelector {
    /// 构造
    pub fn new() -> Self {
        Self
    }

    /// 根据 zoom 选取 LOD 级别 (per INV-WC-05)
    ///
    /// # Errors
    ///
    /// - `LOD_INVALID` — zoom 为负数或 NaN
    pub fn from_zoom(&self, zoom: f32) -> Result<LodLevel, CanvasError> {
        if zoom.is_nan() || zoom < 0.0 {
            return Err(CanvasError::new(
                "LOD_INVALID",
                format!("zoom must be non-negative finite, got {zoom}"),
            ));
        }
        let level = if zoom < 0.3 {
            LodLevel::L0Project
        } else if zoom < 0.7 {
            LodLevel::L1RepoWorktree
        } else {
            LodLevel::L2TaskAgent
        };
        Ok(level)
    }

    /// 判断给定 NodeKind 在指定 LOD 级别下是否可见 (per DD §29 + INV-WC-05)
    pub fn is_node_kind_in_lod(kind: NodeKind, lod: LodLevel) -> bool {
        use NodeKind::*;
        match lod {
            LodLevel::L0Project => matches!(kind, Repository),
            LodLevel::L1RepoWorktree => matches!(kind, Repository | Branch | Worktree),
            LodLevel::L2TaskAgent => true, // 所有 11 Node 类型
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_renderer_lod_3_levels() {
        // 守门 UT-3 (T12): 3 LOD 级别阈值
        let sel = LodSelector::new();

        // zoom < 0.3 → L0
        assert_eq!(sel.from_zoom(0.0).unwrap(), LodLevel::L0Project);
        assert_eq!(sel.from_zoom(0.2).unwrap(), LodLevel::L0Project);

        // 0.3 ≤ zoom < 0.7 → L1
        assert_eq!(sel.from_zoom(0.3).unwrap(), LodLevel::L1RepoWorktree);
        assert_eq!(sel.from_zoom(0.5).unwrap(), LodLevel::L1RepoWorktree);
        assert_eq!(sel.from_zoom(0.69).unwrap(), LodLevel::L1RepoWorktree);

        // zoom ≥ 0.7 → L2
        assert_eq!(sel.from_zoom(0.7).unwrap(), LodLevel::L2TaskAgent);
        assert_eq!(sel.from_zoom(1.0).unwrap(), LodLevel::L2TaskAgent);
        assert_eq!(sel.from_zoom(5.0).unwrap(), LodLevel::L2TaskAgent);

        // 仅 3 级 (守门用)
        assert_eq!(LodLevel::L0Project.level(), 0);
        assert_eq!(LodLevel::L1RepoWorktree.level(), 1);
        assert_eq!(LodLevel::L2TaskAgent.level(), 2);
    }

    #[test]
    fn lod_node_kind_visibility() {
        // Repository 在所有 LOD 都可见
        assert!(LodSelector::is_node_kind_in_lod(
            NodeKind::Repository,
            LodLevel::L0Project
        ));
        assert!(LodSelector::is_node_kind_in_lod(
            NodeKind::Repository,
            LodLevel::L1RepoWorktree
        ));
        assert!(LodSelector::is_node_kind_in_lod(
            NodeKind::Repository,
            LodLevel::L2TaskAgent
        ));

        // Worktree 在 L0 不可见
        assert!(!LodSelector::is_node_kind_in_lod(
            NodeKind::Worktree,
            LodLevel::L0Project
        ));
        assert!(LodSelector::is_node_kind_in_lod(
            NodeKind::Worktree,
            LodLevel::L1RepoWorktree
        ));

        // Task 只在 L2 可见
        assert!(!LodSelector::is_node_kind_in_lod(
            NodeKind::Task,
            LodLevel::L0Project
        ));
        assert!(!LodSelector::is_node_kind_in_lod(
            NodeKind::Task,
            LodLevel::L1RepoWorktree
        ));
        assert!(LodSelector::is_node_kind_in_lod(
            NodeKind::Task,
            LodLevel::L2TaskAgent
        ));
    }

    #[test]
    fn lod_rejects_invalid_zoom() {
        let sel = LodSelector::new();
        assert!(sel.from_zoom(-0.1).is_err());
        assert!(sel.from_zoom(f32::NAN).is_err());
    }
}