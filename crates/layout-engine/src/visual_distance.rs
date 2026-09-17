//! `visual_distance.rs` — visualDistance log 公式 (per FR-WT-005 + DD §36)
//!
//! `visualDistance(commitDistance) = log(commitDistance + 1) * 30`
//!
//! 设计意图 (per DD §36.1):
//! - 视觉距离非线性映射 commit 数, 避免大数值时拉伸画布
//! - log(commitDistance + 1) 保证 commitDistance=0 → 0 (WT 与 main 重合)
//! - 30 是 base pixel, 可调整 (per §36.1)
//!
//! 守门 UT-2 (T10): `visual_distance_log_n_plus_1_formula`

use serde::{Deserialize, Serialize};

/// visualDistance 系数 (per DD §36.1)
pub const VISUAL_DISTANCE_SCALE: f64 = 30.0;

/// 给定 commitDistance, 返回视觉距离 (per FR-WT-005)
///
/// `visualDistance(commitDistance) = ln(commitDistance + 1) * VISUAL_DISTANCE_SCALE`
///
/// 注: Rust `f64::ln` 用自然对数 e; Math.log 在 JS 也是 e (per MDN — `Math.log(x)` = ln(x)).
///
/// # 示例 (per DD §36.1)
///
/// - visualDistance(0) == 0
/// - visualDistance(1) ≈ 20.79
/// - visualDistance(10) ≈ 69.08
/// - visualDistance(100) ≈ 138.63
/// - visualDistance(1000) ≈ 207.94
pub fn visual_distance(commit_distance: u64) -> f64 {
    ((commit_distance as f64) + 1.0).ln() * VISUAL_DISTANCE_SCALE
}

/// VisualDistance 配置 (per spec §8.8 — 可调系数)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualDistanceConfig {
    /// 系数 (默认 30, per DD §36.1)
    pub scale: f64,
}

impl Default for VisualDistanceConfig {
    fn default() -> Self {
        Self {
            scale: VISUAL_DISTANCE_SCALE,
        }
    }
}

impl VisualDistanceConfig {
    /// 给定 commit_distance 计算视觉距离
    pub fn compute(&self, commit_distance: u64) -> f64 {
        ((commit_distance as f64) + 1.0).ln() * self.scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visual_distance_log_n_plus_1_formula() {
        // 守门 UT-2 (T10): visualDistance log(n+1) 公式
        // 公式 per DD §36.1: `visualDistance(n) = log(n + 1) * 30` (JS Math.log = ln)
        // 容差 0.1 (f64 浮点精度)
        let eps = 0.1;

        // 0 → 0 (因为 ln(1) = 0)
        assert!(visual_distance(0).abs() < eps);

        // 1 → ln(2) * 30 ≈ 20.794
        assert!(
            (visual_distance(1) - 20.794_415).abs() < eps,
            "visual_distance(1) = {}",
            visual_distance(1)
        );

        // 10 → ln(11) * 30 ≈ 71.937
        // 注: DD §36.1 spec 写的 69.08 是 ln(10)*30 的近似, 公式文字是 log(n+1)
        assert!(
            (visual_distance(10) - 71.936_858).abs() < eps,
            "visual_distance(10) = {}",
            visual_distance(10)
        );

        // 100 → ln(101) * 30 ≈ 138.454
        assert!(
            (visual_distance(100) - 138.453_615).abs() < eps,
            "visual_distance(100) = {}",
            visual_distance(100)
        );

        // 1000 → ln(1001) * 30 ≈ 207.263
        assert!(
            (visual_distance(1000) - 207.262_643).abs() < eps,
            "visual_distance(1000) = {}",
            visual_distance(1000)
        );

        // monotonic
        assert!(visual_distance(0) < visual_distance(1));
        assert!(visual_distance(1) < visual_distance(10));
        assert!(visual_distance(10) < visual_distance(100));
        assert!(visual_distance(100) < visual_distance(1000));

        // log(0+1) = 0 → 0
        assert_eq!(visual_distance(0), 0.0);
    }

    #[test]
    fn visual_distance_config_scales() {
        let cfg = VisualDistanceConfig { scale: 60.0 };
        assert!((cfg.compute(1) - 41.58).abs() < 0.01);
    }
}
