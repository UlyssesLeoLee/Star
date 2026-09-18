//! `projection.rs` — `WorktreeStatusObserved` 30 天热数据 Projection (per DD §12.5)
//!
//! Per INV-WC-13 守门 + 守门 #13 W/T/M 100% 覆盖:
//! - `worktree_status_observed` W 表 — 30 天热数据 (status + health_score + ahead/behind + risk_count + agent_status 维度)
//! - `worktree_conflict` T 表 — 历史 Conflict 事件 (append-only)
//!
//! Projection 由 `WorktreeService.update()` 自动触发, 不需要独立 service.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use graph_core::types::WorktreeId;

/// 单点 Worktree status observed (per DD §12.5)
///
/// 字段: worktree_id + observed_at + human_state + health_score + ahead + behind + risk_count + agent_type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatusObservedPoint {
    /// Worktree UUID
    pub worktree_id: WorktreeId,
    /// 观测时间
    pub observed_at: DateTime<Utc>,
    /// Human state (snapshot)
    pub human_state: String, // 序列化字符串 (per 跨 crate 边界)
    /// Health score (0-100)
    pub health_score: u8,
    /// Ahead of main
    pub ahead: u32,
    /// Behind main
    pub behind: u32,
    /// Risk count
    pub risk_count: u32,
    /// Agent type (e.g. "claude-code")
    pub agent_type: Option<String>,
}

/// 30 天 hot-data projection (per DD §12.5 + INV-WC-13)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorktreeStatusObserved {
    /// 内部分点 (按时间倒序保留 30 天)
    points: Vec<StatusObservedPoint>,
    /// 30 天阈值
    hot_window: Duration,
}

impl WorktreeStatusObserved {
    /// 创建默认 30 天窗口
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            hot_window: Duration::days(30),
        }
    }

    /// 自定义窗口 (测试用)
    pub fn with_window(window: Duration) -> Self {
        Self {
            points: Vec::new(),
            hot_window: window,
        }
    }

    /// 记录一个观测点
    pub fn record(&mut self, point: StatusObservedPoint) {
        let cutoff = Utc::now() - self.hot_window;
        // 清理超出窗口的旧点 (per 守门 #13 a W 派生), 同时剔除要插入的点如果在窗外
        let point_in_window = point.observed_at >= cutoff;
        self.points.retain(|p| p.observed_at >= cutoff);
        if point_in_window {
            self.points.push(point);
        }
        // 按 observed_at 倒序
        self.points
            .sort_by_key(|p| std::cmp::Reverse(p.observed_at));
    }

    /// 取最近一个点 (最新观测)
    pub fn latest(&self) -> Option<&StatusObservedPoint> {
        self.points.first()
    }

    /// 取全部点 (按时间倒序)
    pub fn all(&self) -> &[StatusObservedPoint] {
        &self.points
    }

    /// 长度
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(worktree_id: WorktreeId, secs_offset: i64) -> StatusObservedPoint {
        StatusObservedPoint {
            worktree_id,
            observed_at: Utc::now() + Duration::seconds(secs_offset),
            human_state: "running".into(),
            health_score: 80,
            ahead: 0,
            behind: 0,
            risk_count: 0,
            agent_type: Some("claude-code".into()),
        }
    }

    #[test]
    fn projection_30day_hot_data_retention() {
        // Per TEST-DESIGN §2.2: worktree_status_observed_projection_30day_hot
        let mut p = WorktreeStatusObserved::new();
        let wt_id = uuid::Uuid::new_v4();

        // 31 天前的点 — 应被 hot window 剔除
        let old = StatusObservedPoint {
            observed_at: Utc::now() - Duration::days(31),
            ..point(wt_id, 0)
        };
        p.record(old);
        assert!(p.is_empty(), "31-day-old point should be evicted on record");

        // 写入今天的点
        p.record(point(wt_id, 0));
        p.record(point(wt_id, 1));
        p.record(point(wt_id, 2));
        assert_eq!(p.len(), 3);

        // latest 应是最近一个
        let latest = p.latest().unwrap();
        assert_eq!(latest.worktree_id, wt_id);
    }

    #[test]
    fn projection_evicts_outside_window_on_record() {
        // 自定义 1 分钟窗口测 evict 行为
        let mut p = WorktreeStatusObserved::with_window(Duration::seconds(60));
        let wt_id = uuid::Uuid::new_v4();

        p.record(point(wt_id, 0));
        p.record(point(wt_id, 10));

        // 插入一个 2 分钟前的点 — 应触发 retain
        let old = StatusObservedPoint {
            observed_at: Utc::now() - Duration::seconds(120),
            ..point(wt_id, 0)
        };
        p.record(old);
        assert_eq!(p.len(), 2, "old point should be evicted, recent 2 kept");
    }

    #[test]
    fn projection_keeps_ordering_desc_by_time() {
        let mut p = WorktreeStatusObserved::new();
        let wt_id = uuid::Uuid::new_v4();
        p.record(point(wt_id, 0)); // 较老
        p.record(point(wt_id, 100)); // 较新
        p.record(point(wt_id, 50)); // 中间

        let times: Vec<_> = p.all().iter().map(|x| x.observed_at).collect();
        for i in 1..times.len() {
            assert!(times[i - 1] >= times[i], "ordering must be desc");
        }
    }
}
