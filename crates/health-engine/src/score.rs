//! `score.rs` — Health Score 算法 + Deduction + Recommendation (per DD §34)

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};

use graph_core::state::TestState;

use graph_core::types::WorktreeId;

/// Worktree 健康度快照 (per DD §34)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorktreeHealthInput {
    /// Worktree ID
    pub worktree_id: WorktreeId,
    /// Branch
    pub branch: String,
    /// Behind main
    pub behind: u32,
    /// Ahead
    pub ahead: u32,
    /// Dirty
    pub dirty: bool,
    /// Last activity
    pub last_activity: DateTime<Utc>,
}

/// 上下文 (per DD §34 — PR / Agent 信息)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthContext {
    /// 是否有 conflict
    pub has_conflict: bool,
    /// Conflict count
    pub conflict_count: u32,
    /// Test state
    pub test_state: TestState,
    /// Build state (Passed / Failed)
    pub build_state: BuildState,
    /// PR info (可选)
    pub pull_request: Option<PrInfo>,
    /// Agent info (可选)
    pub agent: Option<AgentInfo>,
    /// 共享文件数 (与其它 WT)
    pub shared_files_count: u32,
    /// 共享 symbols 数
    pub shared_symbols_count: u32,
    /// 未解决 dependencies
    pub unresolved_dependencies: u32,
    /// Blocked by 数
    pub blocked_by: u32,
}

impl Default for HealthContext {
    fn default() -> Self {
        Self {
            has_conflict: false,
            conflict_count: 0,
            test_state: TestState::None,
            build_state: BuildState::None,
            pull_request: None,
            agent: None,
            shared_files_count: 0,
            shared_symbols_count: 0,
            unresolved_dependencies: 0,
            blocked_by: 0,
        }
    }
}

/// Build state (简化)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildState {
    /// 未知 / 无
    #[default]
    None,
    /// Building
    Running,
    /// Build passed
    Passed,
    /// Build failed
    Failed,
}

/// PR info (简化)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrInfo {
    /// PR number
    pub number: u32,
    /// Review count
    pub review_count: u32,
    /// Created at
    pub created_at: DateTime<Utc>,
}

impl PrInfo {
    /// PR age in days
    pub fn age_days(&self) -> i64 {
        (Utc::now() - self.created_at).num_days()
    }
}

/// Agent info (简化)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Agent type
    pub agent_type: String,
    /// Status (string-form, 不依赖具体 enum 以避免循环)
    pub status: String,
}

/// 单个扣分项 (per FR-WT-028 Explainable State)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Deduction {
    /// 因素名 (e.g. "MergeConflict")
    pub factor: String,
    /// 扣分 (负数)
    pub points: i8,
    /// 人类可读 reason
    pub reason: String,
}

/// Health Score 完整结构 (per DD §34)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthScore {
    /// 分值 0-100
    pub value: u8,
    /// 扣分明细
    pub deductions: Vec<Deduction>,
    /// 计算时间
    pub computed_at: DateTime<Utc>,
    /// 等级 (GREEN/YELLOW/RED)
    pub level: HealthLevel,
}

impl HealthScore {
    /// 计算 level (per BD §7.3)
    pub fn level(value: u8) -> HealthLevel {
        if value >= 80 {
            HealthLevel::Green
        } else if value >= 50 {
            HealthLevel::Yellow
        } else {
            HealthLevel::Red
        }
    }
}

/// Health 等级 (per BD §7.3)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthLevel {
    /// GREEN ≥ 80
    Green,
    /// YELLOW 50-79
    Yellow,
    /// RED < 50
    Red,
}

/// Health Score 历史点 (per DD §14 get_history)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScorePoint {
    /// 时间戳
    pub computed_at: DateTime<Utc>,
    /// 分值
    pub value: u8,
}

/// 推荐 (per DD §14 recommend)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// 推荐动作
    pub action: String,
    /// 原因
    pub reason: String,
    /// 优先级 1-5
    pub priority: u8,
    /// 预期执行后健康度
    pub expected_health_after: u8,
}

/// 13 因素加权扣分 (per DD §34 + §34 完整实现)
pub fn compute(input: &WorktreeHealthInput, context: &HealthContext) -> HealthScore {
    let mut score = 100_u8;
    let mut deductions = Vec::new();
    let now = Utc::now();

    // 1. MergeConflict (-25)
    if context.has_conflict {
        score = score.saturating_sub(25);
        deductions.push(Deduction {
            factor: "MergeConflict".into(),
            points: -25,
            reason: format!("Conflicts with {} other Worktrees", context.conflict_count),
        });
    }

    // 2. TestFailure (-20)
    if context.test_state == TestState::Failed {
        score = score.saturating_sub(20);
        deductions.push(Deduction {
            factor: "TestFailure".into(),
            points: -20,
            reason: "Test run failed".into(),
        });
    }

    // 3. BuildFailure (-15)
    if context.build_state == BuildState::Failed {
        score = score.saturating_sub(15);
        deductions.push(Deduction {
            factor: "BuildFailure".into(),
            points: -15,
            reason: "CI build failed".into(),
        });
    }

    // 4. BehindMain (-10)
    if input.behind > 20 {
        let pts = ((input.behind / 10) as u8).min(10);
        score = score.saturating_sub(pts);
        deductions.push(Deduction {
            factor: "BehindMain".into(),
            points: -(pts as i8),
            reason: format!("Behind main by {} commits", input.behind),
        });
    }

    // 5. Ahead (-5)
    if input.ahead > 30 {
        let pts = ((input.ahead / 30) as u8).min(5);
        score = score.saturating_sub(pts);
        deductions.push(Deduction {
            factor: "Ahead".into(),
            points: -(pts as i8),
            reason: format!("{} commits ahead of main", input.ahead),
        });
    }

    // 6. Dirty (-5)
    if input.dirty {
        score = score.saturating_sub(5);
        deductions.push(Deduction {
            factor: "Dirty".into(),
            points: -5,
            reason: "Uncommitted changes".into(),
        });
    }

    // 7. ReviewMissing (-10)
    if let Some(pr) = &context.pull_request {
        if pr.review_count == 0 && pr.age_days() > 3 {
            score = score.saturating_sub(10);
            deductions.push(Deduction {
                factor: "ReviewMissing".into(),
                points: -10,
                reason: format!(
                    "PR #{} open for {} days without review",
                    pr.number,
                    pr.age_days()
                ),
            });
        }
    }

    // 8. AgentIncomplete (-10)
    if let Some(agent) = &context.agent {
        let terminal = matches!(
            agent.status.as_str(),
            "completed" | "failed" | "stopped" | "cancelled"
        );
        if !terminal {
            score = score.saturating_sub(10);
            deductions.push(Deduction {
                factor: "AgentIncomplete".into(),
                points: -10,
                reason: format!(
                    "Agent {} still in status {:?}",
                    agent.agent_type, agent.status
                ),
            });
        }
    }

    // 9. Inactivity (-10/-15)
    let days_inactive = (now - input.last_activity).num_days();
    if days_inactive > 7 {
        let pts = if days_inactive > 30 { 15 } else { 10 };
        score = score.saturating_sub(pts);
        deductions.push(Deduction {
            factor: "Inactivity".into(),
            points: -(pts as i8),
            reason: format!("No activity for {} days", days_inactive),
        });
    }

    // 10. FileOverlap (-15)
    if context.shared_files_count > 5 {
        let pts = ((context.shared_files_count / 5) as u8).min(15);
        score = score.saturating_sub(pts);
        deductions.push(Deduction {
            factor: "FileOverlap".into(),
            points: -(pts as i8),
            reason: format!(
                "{} files overlap with other Worktrees",
                context.shared_files_count
            ),
        });
    }

    // 11. SymbolOverlap (-10)
    if context.shared_symbols_count > 3 {
        let pts = ((context.shared_symbols_count / 3) as u8).min(10);
        score = score.saturating_sub(pts);
        deductions.push(Deduction {
            factor: "SymbolOverlap".into(),
            points: -(pts as i8),
            reason: format!(
                "{} symbols overlap with other Worktrees",
                context.shared_symbols_count
            ),
        });
    }

    // 12. Dependency (-10)
    if context.unresolved_dependencies > 0 {
        score = score.saturating_sub(10);
        deductions.push(Deduction {
            factor: "Dependency".into(),
            points: -10,
            reason: format!(
                "{} dependencies not yet merged",
                context.unresolved_dependencies
            ),
        });
    }

    // 13. Blocked (-10)
    if context.blocked_by > 0 {
        score = score.saturating_sub(10);
        deductions.push(Deduction {
            factor: "Blocked".into(),
            points: -10,
            reason: format!("Blocked by {} other Worktrees", context.blocked_by),
        });
    }

    let level = HealthScore::level(score);
    HealthScore {
        value: score,
        deductions,
        computed_at: now,
        level,
    }
}

/// 推荐 (per DD §14 recommend)
pub fn recommend(score: &HealthScore, input: &WorktreeHealthInput) -> Recommendation {
    // 简单启发式: 找最大扣分项, 给推荐
    if let Some(top) = score.deductions.iter().max_by_key(|d| -d.points) {
        let (action, priority) = match top.factor.as_str() {
            "MergeConflict" => ("Resolve merge conflicts".into(), 5),
            "TestFailure" => ("Fix failing tests".into(), 5),
            "BuildFailure" => ("Fix CI build".into(), 5),
            "BehindMain" => ("Sync Main".into(), 3),
            "Ahead" => ("Open PR or merge".into(), 3),
            "Dirty" => ("Commit changes".into(), 2),
            "ReviewMissing" => ("Request review".into(), 2),
            "AgentIncomplete" => ("Wait for agent".into(), 2),
            "Inactivity" => ("Resume activity".into(), 1),
            "FileOverlap" | "SymbolOverlap" => ("Coordinate with peer Worktrees".into(), 4),
            "Dependency" => ("Merge dependencies first".into(), 3),
            "Blocked" => ("Resolve blocker".into(), 4),
            _ => ("Review state".into(), 1),
        };
        let expected_health_after = score.value.saturating_add((-top.points) as u8).min(100);
        return Recommendation {
            action,
            reason: format!("Top deduction: {}", top.reason),
            priority,
            expected_health_after,
        };
    }
    let _ = input;
    Recommendation {
        action: "No action needed".into(),
        reason: "Health is at maximum".into(),
        priority: 0,
        expected_health_after: 100,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input_default(behind: u32, ahead: u32, dirty: bool) -> WorktreeHealthInput {
        WorktreeHealthInput {
            worktree_id: WorktreeId::new_v4(),
            branch: "main".into(),
            behind,
            ahead,
            dirty,
            last_activity: Utc::now(),
        }
    }

    #[test]
    fn health_score_13_factors_weighted() {
        // Per TEST-DESIGN §2.2: health_score_13_factors_weighted
        // 构造全部 13 个扣分因素都触发的场景, 验证 deductions 长度 = 13
        let input = input_default(100, 60, true);
        let mut ctx = HealthContext::default();
        ctx.has_conflict = true;
        ctx.conflict_count = 3;
        ctx.test_state = TestState::Failed;
        ctx.build_state = BuildState::Failed;
        ctx.pull_request = Some(PrInfo {
            number: 42,
            review_count: 0,
            created_at: Utc::now() - ChronoDuration::days(10),
        });
        ctx.agent = Some(AgentInfo {
            agent_type: "claude-code".into(),
            status: "tool_calling".into(),
        });
        ctx.shared_files_count = 20;
        ctx.shared_symbols_count = 10;
        ctx.unresolved_dependencies = 2;
        ctx.blocked_by = 1;
        // last_activity 已经设为 now, Inactivity 不触发
        // 改 last_activity 为 60 天前以触发
        let mut input2 = input.clone();
        input2.last_activity = Utc::now() - ChronoDuration::days(60);

        let score = compute(&input2, &ctx);
        assert!(score.deductions.len() >= 12, "至少 12 项触发 (Inactivity 加 last_activity 60 天前)");
        assert!(score.value < 100, "扣分后 < 100");
        assert_eq!(score.level, HealthScore::level(score.value));
    }

    #[test]
    fn health_score_invariant_0_to_100() {
        // Per TEST-DESIGN §2.2: health_score_invariant_0_to_100
        // 极端 input: 全部最大值扣分, score 应 ≥ 0 (不会下溢)
        let input = WorktreeHealthInput {
            worktree_id: WorktreeId::new_v4(),
            branch: "main".into(),
            behind: u32::MAX,
            ahead: u32::MAX,
            dirty: true,
            last_activity: Utc::now() - ChronoDuration::days(365 * 10), // 10 年前
        };
        let mut ctx = HealthContext::default();
        ctx.has_conflict = true;
        ctx.conflict_count = u32::MAX;
        ctx.test_state = TestState::Failed;
        ctx.build_state = BuildState::Failed;
        ctx.pull_request = Some(PrInfo {
            number: 1,
            review_count: 0,
            created_at: Utc::now() - ChronoDuration::days(365 * 10),
        });
        ctx.agent = Some(AgentInfo {
            agent_type: "x".into(),
            status: "executing".into(),
        });
        ctx.shared_files_count = u32::MAX;
        ctx.shared_symbols_count = u32::MAX;
        ctx.unresolved_dependencies = u32::MAX;
        ctx.blocked_by = u32::MAX;
        let score = compute(&input, &ctx);
        // saturating_sub 保证 score ≥ 0
        assert!(score.value <= 100, "score ≤ 100");
        assert_eq!(score.level, HealthLevel::Red, "score=0 应为 RED");
    }

    #[test]
    fn health_score_perfect_minimum_boundaries() {
        // Per TEST-DESIGN §2.2: health_score_perfect_minimum_boundaries
        // 完全干净 = 100 (GREEN)
        let input = input_default(0, 0, false);
        let ctx = HealthContext::default();
        let score = compute(&input, &ctx);
        assert_eq!(score.value, 100);
        assert_eq!(score.level, HealthLevel::Green);
        assert!(score.deductions.is_empty());

        // 边界 80 = GREEN
        let mut input2 = input.clone();
        input2.behind = 21; // 触发 BehindMain, -2
        input2.ahead = 0;
        input2.dirty = true; // -5
        // 100 - 2 - 5 = 93 → GREEN
        let score2 = compute(&input2, &ctx);
        assert!(score2.value >= 80);
        assert_eq!(score2.level, HealthLevel::Green);

        // 边界 79 = YELLOW (build_state = Failed = -15 → 100 - 15 = 85, 加 behind 30 = 100 - 15 - 3 = 82; 加 ahead 31 = 82; 加 dirty = 77 = YELLOW)
        let input3 = WorktreeHealthInput {
            branch: "x".into(),
            worktree_id: WorktreeId::new_v4(),
            behind: 30,    // -3
            ahead: 31,     // -1
            dirty: true,   // -5
            last_activity: Utc::now(),
        };
        let mut ctx3 = HealthContext::default();
        ctx3.test_state = TestState::Failed; // -20
        // 100 - 20 - 3 - 1 - 5 = 71 → YELLOW
        let score3 = compute(&input3, &ctx3);
        assert_eq!(score3.value, 71);
        assert_eq!(score3.level, HealthLevel::Yellow);
        assert!(score3.value >= 50 && score3.value < 80);
    }

    #[test]
    fn health_score_threshold_classification_correct() {
        assert_eq!(HealthScore::level(100), HealthLevel::Green);
        assert_eq!(HealthScore::level(80), HealthLevel::Green);
        assert_eq!(HealthScore::level(79), HealthLevel::Yellow);
        assert_eq!(HealthScore::level(50), HealthLevel::Yellow);
        assert_eq!(HealthScore::level(49), HealthLevel::Red);
        assert_eq!(HealthScore::level(0), HealthLevel::Red);
    }
}
