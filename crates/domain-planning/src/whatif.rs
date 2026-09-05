//! Star Planning — What-if 沙盒 + 信心度 + 基线 (wt-w10-whatif 扩展)
//!
//! - What-if 沙盒: Plan 副本, 不影响原数据, 多版本对比
//! - 信心度: Committed / Planned / Exploratory
//! - 基线 (Baseline): 保存当前排程, 后续可对比差异

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// 1. What-if 沙盒
// =====================================================================

/// What-if 沙盒场景 (Plan 副本, 不影响原数据)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhatIfScenario {
    /// 场景 ID
    pub id: Uuid,
    /// 场景名称
    pub name: String,
    /// 来源 Plan ID (只读)
    pub source_plan_id: Uuid, // 来源 Plan (只读)
    /// 排程调整列表
    pub schedule_adjustments: Vec<ScheduleAdjustment>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 创建者
    pub created_by: Uuid,
}

/// 单项排程调整
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScheduleAdjustment {
    /// 工作项 ID
    pub work_item_id: Uuid,
    /// 原始开始时间
    pub original_start: DateTime<Utc>,
    /// 原始结束时间
    pub original_end: DateTime<Utc>,
    /// 调整后开始时间
    pub new_start: DateTime<Utc>,
    /// 调整后结束时间
    pub new_end: DateTime<Utc>,
    /// 调整原因 ("delay", "accelerate", "reassign")
    pub reason: String, // "delay", "accelerate", "reassign"
}

impl WhatIfScenario {
    /// 创建新的 what-if 场景
    pub fn new(name: impl Into<String>, source_plan_id: Uuid, actor: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            source_plan_id,
            schedule_adjustments: Vec::new(),
            created_at: Utc::now(),
            created_by: actor,
        }
    }

    /// 添加调整 (不影响 source plan)
    pub fn add_adjustment(&mut self, adj: ScheduleAdjustment) {
        self.schedule_adjustments.push(adj);
    }

    /// 计算总延期天数 (new_end - original_end)
    pub fn total_delay_days(&self) -> i64 {
        let mut total_seconds: i64 = 0;
        for adj in &self.schedule_adjustments {
            let delay = (adj.new_end - adj.original_end).num_seconds();
            total_seconds += delay;
        }
        total_seconds / 86400
    }

    /// 计算受影响工作项数
    pub fn affected_count(&self) -> usize {
        self.schedule_adjustments.len()
    }

    /// 比对两个 scenario (返回延期差值)
    pub fn diff(other: &WhatIfScenario) -> i64 {
        other.total_delay_days() // 简化
    }
}

// =====================================================================
// 2. 信心度
// =====================================================================

/// 排程信心度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    /// 高: 在 active sprint 或 next sprint
    Committed,   // 高: 在 active sprint 或 next sprint
    /// 中: backlog-refined, 未 sprint-scheduled
    Planned,     // 中: backlog-refined, 未 sprint-scheduled
    /// 低: 仅方向性承诺
    Exploratory, // 低: 仅方向性承诺
}

impl Confidence {
    /// 返回信心度对应的展示颜色
    pub fn color(&self) -> &'static str {
        match self {
            Self::Committed => "#3D8B5F",   // 绿
            Self::Planned => "#C77B30",     // 橙
            Self::Exploratory => "#94A3B8", // 灰
        }
    }

    /// 返回信心度的中文名称
    pub fn name_zh(&self) -> &'static str {
        match self {
            Self::Committed => "已承诺",
            Self::Planned => "已规划",
            Self::Exploratory => "探索性",
        }
    }
}

// =====================================================================
// 3. 基线
// =====================================================================

/// 排程基线 (保存当前排程, 后续可对比差异)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Baseline {
    /// 基线 ID
    pub id: Uuid,
    /// 基线名称
    pub name: String,
    /// 所属 Plan ID
    pub plan_id: Uuid,
    /// Plan 当前状态的 JSON 快照
    pub snapshot: serde_json::Value, // Plan 当前状态的 JSON 快照
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 创建者
    pub created_by: Uuid,
}

impl Baseline {
    /// 创建新的基线
    pub fn new(
        name: impl Into<String>,
        plan_id: Uuid,
        snapshot: serde_json::Value,
        actor: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            plan_id,
            snapshot,
            created_at: Utc::now(),
            created_by: actor,
        }
    }
}

/// 基线 vs 当前状态的差异
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaselineDiff {
    /// 基线 ID
    pub baseline_id: Uuid,
    /// 当前状态快照
    pub current: serde_json::Value,
    /// 变更列表
    pub changes: Vec<BaselineChange>,
}

/// 单个字段的基线变更
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaselineChange {
    /// 变更字段名
    pub field: String,
    /// 旧值
    pub old_value: serde_json::Value,
    /// 新值
    pub new_value: serde_json::Value,
}

// =====================================================================
// 4. WhatIfService 聚合
// =====================================================================

/// What-if 沙盒 + 信心度 + 基线聚合服务
pub struct WhatIfService;

impl WhatIfService {
    /// 创建新的 WhatIfService
    pub fn new() -> Self {
        Self
    }

    /// 创建 scenario
    pub fn create_scenario(
        &self,
        name: impl Into<String>,
        source_plan_id: Uuid,
        actor: Uuid,
    ) -> WhatIfScenario {
        WhatIfScenario::new(name, source_plan_id, actor)
    }

    /// 保存基线
    pub fn save_baseline(
        &self,
        name: impl Into<String>,
        plan_id: Uuid,
        snapshot: serde_json::Value,
        actor: Uuid,
    ) -> Baseline {
        Baseline::new(name, plan_id, snapshot, actor)
    }

    /// 对比基线 vs 当前 (简化: 字段数对比)
    pub fn diff_baseline(&self, baseline: &Baseline, current: &serde_json::Value) -> BaselineDiff {
        let mut changes = Vec::new();
        if let (Some(b), Some(c)) = (baseline.snapshot.as_object(), current.as_object()) {
            for (k, v_old) in b {
                if let Some(v_new) = c.get(k) {
                    if v_old != v_new {
                        changes.push(BaselineChange {
                            field: k.clone(),
                            old_value: v_old.clone(),
                            new_value: v_new.clone(),
                        });
                    }
                } else {
                    changes.push(BaselineChange {
                        field: k.clone(),
                        old_value: v_old.clone(),
                        new_value: serde_json::Value::Null,
                    });
                }
            }
        }
        BaselineDiff {
            baseline_id: baseline.id,
            current: current.clone(),
            changes,
        }
    }
}

impl Default for WhatIfService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whatif_new() {
        let s = WhatIfScenario::new("Test", Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(s.name, "Test");
        assert_eq!(s.affected_count(), 0);
        assert_eq!(s.total_delay_days(), 0);
    }

    #[test]
    fn test_whatif_add_adjustment() {
        let mut s = WhatIfScenario::new("Delay Feature B", Uuid::new_v4(), Uuid::new_v4());
        let now = Utc::now();
        s.add_adjustment(ScheduleAdjustment {
            work_item_id: Uuid::new_v4(),
            original_start: now,
            original_end: now + chrono::Duration::days(7),
            new_start: now + chrono::Duration::days(7),
            new_end: now + chrono::Duration::days(14),
            reason: "delay".into(),
        });
        assert_eq!(s.affected_count(), 1);
        assert_eq!(s.total_delay_days(), 7);
    }

    #[test]
    fn test_confidence_color() {
        assert_eq!(Confidence::Committed.color(), "#3D8B5F");
        assert_eq!(Confidence::Planned.color(), "#C77B30");
        assert_eq!(Confidence::Exploratory.color(), "#94A3B8");
    }

    #[test]
    fn test_baseline_new() {
        let b = Baseline::new(
            "Initial",
            Uuid::new_v4(),
            serde_json::json!({"x": 1}),
            Uuid::new_v4(),
        );
        assert_eq!(b.name, "Initial");
    }

    #[test]
    fn test_diff_baseline_no_change() {
        let svc = WhatIfService::new();
        let snap = serde_json::json!({"a": 1, "b": 2});
        let b = Baseline::new("v1", Uuid::new_v4(), snap.clone(), Uuid::new_v4());
        let d = svc.diff_baseline(&b, &snap);
        assert_eq!(d.changes.len(), 0);
    }

    #[test]
    fn test_diff_baseline_one_change() {
        let svc = WhatIfService::new();
        let snap = serde_json::json!({"a": 1, "b": 2});
        let current = serde_json::json!({"a": 1, "b": 99});
        let b = Baseline::new("v1", Uuid::new_v4(), snap, Uuid::new_v4());
        let d = svc.diff_baseline(&b, &current);
        assert_eq!(d.changes.len(), 1);
        assert_eq!(d.changes[0].field, "b");
    }
}
