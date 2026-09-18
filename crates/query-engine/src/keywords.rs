//! `keywords.rs` — 6 关键字定义 (per ULYS-57.3 T11 + spec §8.7)
//!
//! 6 关键字 (per ULYS-57.3 acceptance + DD §17.1):
//! - `show` (HumanState: conflict / unmerged / ready / stale / running / waiting / merged)
//! - `agent` (agent type)
//! - `behind` (op + num)
//! - `ahead` (op + num)
//! - `health` (op + num)
//! - `modified` (file path)

use graph_core::state::HumanState;
use serde::{Deserialize, Serialize};

/// 6 关键字 (守门用)
pub const KEYWORD_COUNT: usize = 6;

/// Search DSL 关键字 (per ULYS-57.3 T11 6 关键字守门)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Keyword {
    /// show: HumanState 过滤
    Show,
    /// agent: agent_type 过滤
    Agent,
    /// behind: 落后 commit 数 (op + num)
    Behind,
    /// ahead: 领先 commit 数 (op + num)
    Ahead,
    /// health: Health Score (op + num)
    Health,
    /// modified: 修改的文件
    Modified,
}

impl Keyword {
    /// 关键字字符串 (DSL 用)
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Show => "show",
            Self::Agent => "agent",
            Self::Behind => "behind",
            Self::Ahead => "ahead",
            Self::Health => "health",
            Self::Modified => "modified",
        }
    }

    /// 全部 6 个 (按 enum 顺序, 守门用)
    pub const ALL: [Keyword; KEYWORD_COUNT] = [
        Self::Show,
        Self::Agent,
        Self::Behind,
        Self::Ahead,
        Self::Health,
        Self::Modified,
    ];
}

/// show 关键字的可选值 (per spec §8.7)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShowValue {
    /// conflict
    Conflict,
    /// unmerged (人类语言, 对应 HumanState != Merged)
    Unmerged,
    /// ready
    Ready,
    /// stale
    Stale,
    /// running
    Running,
    /// waiting
    Waiting,
    /// merged
    Merged,
    /// diverged
    Diverged,
}

impl ShowValue {
    /// 解析 show value 字符串 (case-insensitive)
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "conflict" => Some(Self::Conflict),
            "unmerged" => Some(Self::Unmerged),
            "ready" => Some(Self::Ready),
            "stale" => Some(Self::Stale),
            "running" => Some(Self::Running),
            "waiting" => Some(Self::Waiting),
            "merged" => Some(Self::Merged),
            "diverged" => Some(Self::Diverged),
            _ => None,
        }
    }

    /// 转换为 graph-core 的 HumanState (per DD §17.1)
    pub fn to_human_state(&self) -> Option<HumanState> {
        match self {
            Self::Conflict => Some(HumanState::Conflict),
            Self::Ready => Some(HumanState::Ready),
            Self::Stale => Some(HumanState::Stale),
            Self::Running => Some(HumanState::Running),
            Self::Waiting => Some(HumanState::Waiting),
            Self::Merged => Some(HumanState::Merged),
            Self::Diverged => Some(HumanState::Diverged),
            // Unmerged 是 negation filter (HumanState != Merged), 不是单一状态
            Self::Unmerged => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_6_keywords_show_agent_behind_ahead_health_modified() {
        // 守门 UT-3 (T11): 6 关键字完整定义
        assert_eq!(KEYWORD_COUNT, 6);
        assert_eq!(Keyword::ALL.len(), 6);

        let names: Vec<&str> = Keyword::ALL.iter().map(|k| k.as_str()).collect();
        assert!(names.contains(&"show"));
        assert!(names.contains(&"agent"));
        assert!(names.contains(&"behind"));
        assert!(names.contains(&"ahead"));
        assert!(names.contains(&"health"));
        assert!(names.contains(&"modified"));
    }

    #[test]
    fn show_value_parse_and_map_to_human_state() {
        // ShowValue 解析 + 映射
        assert_eq!(ShowValue::parse("conflict"), Some(ShowValue::Conflict));
        assert_eq!(ShowValue::parse("CONFLICT"), Some(ShowValue::Conflict));
        assert_eq!(ShowValue::parse("unknown"), None);

        assert_eq!(
            ShowValue::Conflict.to_human_state(),
            Some(HumanState::Conflict)
        );
        assert_eq!(
            ShowValue::Unmerged.to_human_state(),
            None
        );
    }
}
