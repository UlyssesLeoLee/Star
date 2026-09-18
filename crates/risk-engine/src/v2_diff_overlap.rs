//! `v2_diff_overlap.rs` — V2 Diff Overlap 层 (per DD §13 + §33 + §35)
//!
//! 检测:
//! - `FileOverlap` — 跨 WT 共享文件
//! - `MergeConflict` — 同一文件 line overlap

use chrono::Utc;
use git_adapter::Diff;

use graph_core::types::{RiskScore, WorktreeId};

use crate::config::RiskEngineConfig;
use crate::risk::{Risk, RiskType};

/// 输入 (per DD §33)
#[derive(Debug, Clone)]
pub struct V2Input {
    /// Worktree A
    pub worktree_a: WorktreeId,
    /// Worktree B
    pub worktree_b: WorktreeId,
    /// A diff
    pub diff_a: Diff,
    /// B diff
    pub diff_b: Diff,
}

/// V2 风险检测 (per DD §33 conflict_prediction)
pub fn detect(input: &V2Input, config: &RiskEngineConfig) -> Vec<Risk> {
    if !config.v2_enabled {
        return Vec::new();
    }

    let mut out = Vec::new();
    let now = Utc::now();

    // 跨文件 path 集合
    let paths_a: std::collections::HashSet<_> =
        input.diff_a.files.iter().map(|f| f.path.clone()).collect();
    let paths_b: std::collections::HashSet<_> =
        input.diff_b.files.iter().map(|f| f.path.clone()).collect();

    let shared: Vec<_> = paths_a.intersection(&paths_b).collect();

    if !shared.is_empty() {
        // line overlap: 计算共享文件上的 added/removed line 重叠
        let mut total_line_overlap = 0u32;
        for path in &shared {
            let lines_a: std::collections::HashSet<u32> = input
                .diff_a
                .files
                .iter()
                .find(|f| &f.path == *path)
                .map(|f| f.added_lines.iter().copied().collect())
                .unwrap_or_default();
            let lines_b: std::collections::HashSet<u32> = input
                .diff_b
                .files
                .iter()
                .find(|f| &f.path == *path)
                .map(|f| f.added_lines.iter().copied().collect())
                .unwrap_or_default();
            total_line_overlap += lines_a.intersection(&lines_b).count() as u32;
        }

        // FileOverlap risk
        let score = (shared.len() as f32 / 10.0).min(1.0);
        out.push(Risk {
            id: uuid::Uuid::new_v4(),
            worktree_id: input.worktree_a,
            risk_type: RiskType::FileOverlap,
            risk_score: RiskScore::new(score),
            reason: format!("{} shared files with another WT", shared.len()),
            detected_at: now,
            resolved_at: None,
            related_worktrees: vec![input.worktree_b],
        });

        // MergeConflict risk (if line overlap)
        if total_line_overlap > 0 {
            let score = (total_line_overlap as f32 / 100.0).min(1.0);
            out.push(Risk {
                id: uuid::Uuid::new_v4(),
                worktree_id: input.worktree_a,
                risk_type: RiskType::MergeConflict,
                risk_score: RiskScore::new(score),
                reason: format!("{} overlapping lines", total_line_overlap),
                detected_at: now,
                resolved_at: None,
                related_worktrees: vec![input.worktree_b],
            });
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use git_adapter::DiffFile;
    use std::path::PathBuf;

    #[test]
    fn v2_detects_file_overlap() {
        let config = RiskEngineConfig::default();
        let input = V2Input {
            worktree_a: WorktreeId::new_v4(),
            worktree_b: WorktreeId::new_v4(),
            diff_a: Diff {
                files: vec![DiffFile {
                    path: PathBuf::from("shared.rs"),
                    added_lines: vec![1, 2, 3],
                    removed_lines: vec![],
                    hunks: vec![],
                }],
                total_added: 3,
                total_removed: 0,
            },
            diff_b: Diff {
                files: vec![DiffFile {
                    path: PathBuf::from("shared.rs"),
                    added_lines: vec![4, 5, 6],
                    removed_lines: vec![],
                    hunks: vec![],
                }],
                total_added: 3,
                total_removed: 0,
            },
        };
        let risks = detect(&input, &config);
        assert!(risks.iter().any(|r| r.risk_type == RiskType::FileOverlap));
    }
}
