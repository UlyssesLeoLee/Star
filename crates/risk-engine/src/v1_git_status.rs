//! `v1_git_status.rs` — V1 Git Status 层 (per DD §13 + §35)
//!
//! 检测:
//! - `MergeConflict` — porcelain status 含 UU/AA/AU (conflict markers)
//! - `FileOverlap` — porcelain status 含 shared 文件数 (与已检测 WT 比)
//! - `Stale` — last_commit_at > stale_threshold_days
//! - `Divergence` — ahead/behind > 阈值

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use git_adapter::{StatusEntry, StatusKind};
use std::path::PathBuf;

use graph_core::types::{RiskScore, WorktreeId};

use crate::config::RiskEngineConfig;
use crate::risk::{Risk, RiskType};

/// 输入 (per DD §13 + §35)
#[derive(Debug, Clone)]
pub struct V1Input {
    /// Worktree ID
    pub worktree_id: WorktreeId,
    /// Porcelain status (from git-adapter::GitProvider::status)
    pub status: Vec<StatusEntry>,
    /// Ahead of main
    pub ahead: u32,
    /// Behind main
    pub behind: u32,
    /// Last commit at
    pub last_commit_at: Option<DateTime<Utc>>,
}

/// V1 风险检测
pub fn detect(input: &V1Input, config: &RiskEngineConfig) -> Vec<Risk> {
    if !config.v1_enabled {
        return Vec::new();
    }

    let mut out = Vec::new();
    let now = Utc::now();

    // 1. MergeConflict — porcelain 含 conflict 标记
    let conflict_count = input
        .status
        .iter()
        .filter(|s| matches!(s.kind, StatusKind::Conflicting))
        .count();
    if conflict_count > 0 {
        let score = (conflict_count as f32 / 5.0).min(1.0);
        out.push(Risk {
            id: uuid::Uuid::new_v4(),
            worktree_id: input.worktree_id,
            risk_type: RiskType::MergeConflict,
            risk_score: RiskScore::new(score),
            reason: format!("{} conflicted files in porcelain status", conflict_count),
            detected_at: now,
            resolved_at: None,
            related_worktrees: Vec::new(),
        });
    }

    // 2. FileOverlap — modified 文件数
    let modified_count = input
        .status
        .iter()
        .filter(|s| matches!(s.kind, StatusKind::Modified))
        .count();
    if modified_count >= 5 {
        let score = (modified_count as f32 / 50.0).min(1.0);
        out.push(Risk {
            id: uuid::Uuid::new_v4(),
            worktree_id: input.worktree_id,
            risk_type: RiskType::FileOverlap,
            risk_score: RiskScore::new(score),
            reason: format!("{} modified files", modified_count),
            detected_at: now,
            resolved_at: None,
            related_worktrees: Vec::new(),
        });
    }

    // 3. Stale — last_commit_at > stale_threshold_days
    if let Some(last) = input.last_commit_at {
        let days = (now - last).num_days();
        if days > config.stale_threshold_days as i64 {
            let score = (days as f32 / 30.0).min(1.0);
            out.push(Risk {
                id: uuid::Uuid::new_v4(),
                worktree_id: input.worktree_id,
                risk_type: RiskType::Stale,
                risk_score: RiskScore::new(score),
                reason: format!("No commit for {} days", days),
                detected_at: now,
                resolved_at: None,
                related_worktrees: Vec::new(),
            });
        }
    }

    // 4. Divergence — ahead/behind 阈值
    if input.ahead > config.divergence_ahead_threshold
        && input.behind > config.divergence_behind_threshold
    {
        let score = ((input.ahead + input.behind) as f32 / 50.0).min(1.0);
        out.push(Risk {
            id: uuid::Uuid::new_v4(),
            worktree_id: input.worktree_id,
            risk_type: RiskType::Divergence,
            risk_score: RiskScore::new(score),
            reason: format!(
                "Ahead by {}, behind by {}",
                input.ahead, input.behind
            ),
            detected_at: now,
            resolved_at: None,
            related_worktrees: Vec::new(),
        });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_detects_merge_conflict() {
        let config = RiskEngineConfig::default();
        let input = V1Input {
            worktree_id: WorktreeId::new_v4(),
            status: vec![StatusEntry {
                path: PathBuf::from("file.rs"),
                kind: StatusKind::Conflicting,
            }],
            ahead: 0,
            behind: 0,
            last_commit_at: Some(Utc::now()),
        };
        let risks = detect(&input, &config);
        assert!(risks.iter().any(|r| r.risk_type == RiskType::MergeConflict));
    }

    #[test]
    fn v1_detects_stale() {
        let config = RiskEngineConfig::default();
        let old = Utc::now() - ChronoDuration::days(15);
        let input = V1Input {
            worktree_id: WorktreeId::new_v4(),
            status: vec![],
            ahead: 0,
            behind: 0,
            last_commit_at: Some(old),
        };
        let risks = detect(&input, &config);
        assert!(risks.iter().any(|r| r.risk_type == RiskType::Stale));
    }
}
