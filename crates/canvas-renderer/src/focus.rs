//! `focus.rs` — Focus Mode N-hop (per DD §37 + INV-WC-07)
//!
//! 1/2/3-hop 透明度:
//! - focus node: 1.0
//! - hop 1: 0.8
//! - hop 2: 0.6
//! - hop 3: 0.4
//! - 其他: 0.1 (背景)
//!
//! MVP 实现: BFS on adjacency list. 真实图数据由前端传入 (per BFF).

use crate::error::CanvasError;
use graph_core::types::WorktreeId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Hop 距离 (per INV-WC-07 + DD §37)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Hop {
    /// 1-hop
    Hop1 = 1,
    /// 2-hop
    Hop2 = 2,
    /// 3-hop
    Hop3 = 3,
}

impl Hop {
    /// 数值
    pub fn value(self) -> u8 {
        self as u8
    }

    /// Hop → 透明度
    pub fn transparency(self) -> f32 {
        match self {
            Self::Hop1 => 0.8,
            Self::Hop2 => 0.6,
            Self::Hop3 => 0.4,
        }
    }

    /// 解析 1/2/3
    ///
    /// # Errors
    ///
    /// - `FOCUS_HOP_OUT_OF_RANGE` — 不是 1/2/3
    pub fn parse(n: u8) -> Result<Self, CanvasError> {
        match n {
            1 => Ok(Self::Hop1),
            2 => Ok(Self::Hop2),
            3 => Ok(Self::Hop3),
            other => Err(CanvasError::new(
                "FOCUS_HOP_OUT_OF_RANGE",
                format!("hop must be 1, 2, or 3, got {other}"),
            )),
        }
    }
}

/// Focus 透明度 (per INV-WC-07)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FocusTransparency(f32);

impl FocusTransparency {
    /// 构造
    pub fn new(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// focus node 透明度
    pub fn focus() -> Self {
        Self::new(1.0)
    }

    /// 其他节点 (背景) 透明度
    pub fn background() -> Self {
        Self::new(0.1)
    }

    /// 取得透明度值
    pub fn value(self) -> f32 {
        self.0
    }
}

/// Focus 计算结果
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FocusResult {
    /// focus 节点 ID
    pub focus_id: WorktreeId,
    /// hop 范围
    pub hop: Hop,
    /// 节点 ID → 透明度映射
    pub opacity_map: HashMap<WorktreeId, FocusTransparency>,
}

/// Focus 计算器
#[derive(Debug, Default, Clone)]
pub struct FocusCalculator;

impl FocusCalculator {
    /// 构造
    pub fn new() -> Self {
        Self
    }

    /// 计算 focus 透明度映射 (BFS)
    ///
    /// `adjacency` 是无向邻接表 (per WorktreeId → 相邻 WorktreeId 列表).
    /// 不在 adjacency 中的节点被当作孤立节点处理.
    pub fn calculate(
        &self,
        focus_id: WorktreeId,
        hop: Hop,
        adjacency: &HashMap<WorktreeId, Vec<WorktreeId>>,
    ) -> FocusResult {
        let mut opacity_map: HashMap<WorktreeId, FocusTransparency> = HashMap::new();

        // focus node: 1.0
        opacity_map.insert(focus_id, FocusTransparency::focus());

        // BFS 找 1..=hop 范围内的节点
        let max_depth = hop.value() as usize;
        let mut visited: HashSet<WorktreeId> = HashSet::new();
        visited.insert(focus_id);
        let mut frontier: VecDeque<(WorktreeId, u8)> = VecDeque::new();
        frontier.push_back((focus_id, 0));

        while let Some((current, depth)) = frontier.pop_front() {
            if usize::from(depth) >= max_depth {
                continue;
            }
            if let Some(neighbors) = adjacency.get(&current) {
                for &next in neighbors {
                    if visited.contains(&next) {
                        continue;
                    }
                    visited.insert(next);
                    let next_depth = depth + 1;
                    let next_hop = Hop::parse(next_depth).unwrap_or(hop);
                    let transparency = FocusTransparency::new(next_hop.transparency());
                    opacity_map.insert(next, transparency);
                    frontier.push_back((next, next_depth));
                }
            }
        }

        FocusResult {
            focus_id,
            hop,
            opacity_map,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph_core::types::WorktreeId;

    #[test]
    fn focus_mode_n_hop_1_2_3_transparency() {
        // 守门 UT-3 (T12): focus mode 1/2/3-hop 透明度 (per INV-WC-07)
        //
        // 构造链: A — B — C — D — E
        // A = focus
        // hop 1: {B: 0.8}
        // hop 2: {B: 0.8, C: 0.6}
        // hop 3: {B: 0.8, C: 0.6, D: 0.4}
        let a = WorktreeId::from_u128(1);
        let b = WorktreeId::from_u128(2);
        let c = WorktreeId::from_u128(3);
        let d = WorktreeId::from_u128(4);
        let e = WorktreeId::from_u128(5);

        let mut adjacency: HashMap<WorktreeId, Vec<WorktreeId>> = HashMap::new();
        adjacency.insert(a, vec![b]);
        adjacency.insert(b, vec![a, c]);
        adjacency.insert(c, vec![b, d]);
        adjacency.insert(d, vec![c, e]);
        adjacency.insert(e, vec![d]);

        let calc = FocusCalculator::new();

        // 1-hop: A=1.0, B=0.8
        let r1 = calc.calculate(a, Hop::Hop1, &adjacency);
        assert_eq!(r1.opacity_map.get(&a).unwrap().value(), 1.0);
        assert_eq!(r1.opacity_map.get(&b).unwrap().value(), 0.8);
        assert!(!r1.opacity_map.contains_key(&c));

        // 2-hop: + C=0.6
        let r2 = calc.calculate(a, Hop::Hop2, &adjacency);
        assert_eq!(r2.opacity_map.get(&c).unwrap().value(), 0.6);
        assert!(!r2.opacity_map.contains_key(&d));

        // 3-hop: + D=0.4
        let r3 = calc.calculate(a, Hop::Hop3, &adjacency);
        assert_eq!(r3.opacity_map.get(&d).unwrap().value(), 0.4);
        assert!(!r3.opacity_map.contains_key(&e));
    }

    #[test]
    fn focus_isolated_node() {
        // 孤立节点: 仅自身 1.0
        let focus = WorktreeId::from_u128(1);
        let calc = FocusCalculator::new();
        let adj: HashMap<WorktreeId, Vec<WorktreeId>> = HashMap::new();
        let result = calc.calculate(focus, Hop::Hop3, &adj);
        assert_eq!(result.opacity_map.len(), 1);
        assert_eq!(result.opacity_map.get(&focus).unwrap().value(), 1.0);
    }

    #[test]
    fn hop_parse_round_trip() {
        assert_eq!(Hop::parse(1).unwrap(), Hop::Hop1);
        assert_eq!(Hop::parse(2).unwrap(), Hop::Hop2);
        assert_eq!(Hop::parse(3).unwrap(), Hop::Hop3);
        assert!(Hop::parse(0).is_err());
        assert!(Hop::parse(4).is_err());
    }

    #[test]
    fn focus_transparency_clamp() {
        let t = FocusTransparency::new(1.5);
        assert_eq!(t.value(), 1.0);
        let t = FocusTransparency::new(-0.5);
        assert_eq!(t.value(), 0.0);
    }
}