//! `view_mode.rs` — 5 View Mode 切换 (per FR-UI-007..011 + DD §11)
//!
//! 5 模式 (per ULYS-57.3 acceptance):
//! - TREE       — Repository tree (dagre)
//! - DEPENDENCY — Worktree 依赖图 (d3-force)
//! - RISK       — Risk 优先级视图 (ELK layered)
//! - AGENT      — Agent 分配视图 (cluster)
//! - HISTORY    — 时间线视图 (linear)

use crate::error::CanvasError;
use graph_core::node::NodeKind;
use serde::{Deserialize, Serialize};

/// 5 View Mode (per FR-UI-007..011)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewMode {
    /// TREE — Repository tree
    Tree,
    /// DEPENDENCY — Worktree 依赖图
    Dependency,
    /// RISK — Risk 优先级
    Risk,
    /// AGENT — Agent 分配
    Agent,
    /// HISTORY — 时间线
    History,
}

impl ViewMode {
    /// 字符串形式
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tree => "tree",
            Self::Dependency => "dependency",
            Self::Risk => "risk",
            Self::Agent => "agent",
            Self::History => "history",
        }
    }

    /// 全部 5 个 (守门用)
    pub const ALL: [ViewMode; 5] = [
        Self::Tree,
        Self::Dependency,
        Self::Risk,
        Self::Agent,
        Self::History,
    ];

    /// 解析 (case-insensitive)
    ///
    /// # Errors
    ///
    /// - `VIEW_MODE_INVALID` — 未知模式
    pub fn parse(s: &str) -> Result<Self, CanvasError> {
        match s.to_ascii_lowercase().as_str() {
            "tree" => Ok(Self::Tree),
            "dependency" | "deps" => Ok(Self::Dependency),
            "risk" => Ok(Self::Risk),
            "agent" => Ok(Self::Agent),
            "history" | "timeline" => Ok(Self::History),
            other => Err(CanvasError::new(
                "VIEW_MODE_INVALID",
                format!("unknown view mode: '{other}' (allowed: tree, dependency, risk, agent, history)"),
            )),
        }
    }

    /// 该模式推荐的 layout 算法
    pub fn recommended_layout(&self) -> &'static str {
        match self {
            Self::Tree => "dagre",
            Self::Dependency => "d3-force",
            Self::Risk => "elk",
            Self::Agent => "d3-force",
            Self::History => "timeline",
        }
    }

    /// 该模式下哪些 NodeKind 主显示
    pub fn primary_node_kinds(&self) -> &'static [NodeKind] {
        use NodeKind::*;
        match self {
            Self::Tree => &[Repository, Branch, Worktree],
            Self::Dependency => &[Worktree, Task, AgentSession],
            Self::Risk => &[Worktree, TestRun, Issue],
            Self::Agent => &[AgentSession, Worktree, Task],
            Self::History => &[Worktree, Commit, PullRequest],
        }
    }
}

/// View Mode layout 描述
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewModeLayout {
    /// view mode
    pub mode: ViewMode,
    /// 推荐 layout 算法
    pub layout: String,
    /// 主 NodeKind
    pub primary_kinds: Vec<NodeKind>,
}

impl From<ViewMode> for ViewModeLayout {
    fn from(mode: ViewMode) -> Self {
        Self {
            mode,
            layout: mode.recommended_layout().to_string(),
            primary_kinds: mode.primary_node_kinds().to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_5_view_modes_correct_layout() {
        // 守门 UT-3 (T12): 5 View Mode 各自 layout + 主节点类型
        assert_eq!(ViewMode::ALL.len(), 5);

        // TREE → dagre + Repository/Branch/Worktree
        let layout = ViewModeLayout::from(ViewMode::Tree);
        assert_eq!(layout.mode, ViewMode::Tree);
        assert_eq!(layout.layout, "dagre");
        assert!(layout.primary_kinds.contains(&NodeKind::Repository));
        assert!(layout.primary_kinds.contains(&NodeKind::Worktree));

        // DEPENDENCY → d3-force + Worktree/Task/AgentSession
        let layout = ViewModeLayout::from(ViewMode::Dependency);
        assert_eq!(layout.layout, "d3-force");
        assert!(layout.primary_kinds.contains(&NodeKind::Task));

        // RISK → elk + Worktree/TestRun/Issue
        let layout = ViewModeLayout::from(ViewMode::Risk);
        assert_eq!(layout.layout, "elk");
        assert!(layout.primary_kinds.contains(&NodeKind::Issue));

        // AGENT → d3-force + AgentSession
        let layout = ViewModeLayout::from(ViewMode::Agent);
        assert_eq!(layout.layout, "d3-force");
        assert!(layout.primary_kinds.contains(&NodeKind::AgentSession));

        // HISTORY → timeline + Worktree/Commit/PR
        let layout = ViewModeLayout::from(ViewMode::History);
        assert_eq!(layout.layout, "timeline");
        assert!(layout.primary_kinds.contains(&NodeKind::Commit));
    }

    #[test]
    fn view_mode_parse_round_trip() {
        for mode in ViewMode::ALL {
            assert_eq!(ViewMode::parse(mode.as_str()).unwrap(), mode);
        }
        assert!(ViewMode::parse("invalid").is_err());
    }

    #[test]
    fn view_mode_aliases() {
        assert_eq!(ViewMode::parse("deps").unwrap(), ViewMode::Dependency);
        assert_eq!(ViewMode::parse("timeline").unwrap(), ViewMode::History);
    }
}
