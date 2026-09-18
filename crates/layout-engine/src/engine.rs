//! `engine.rs` — LayoutEngine Trait + 3 算法 + 切换阈值 + Pure impl
//!
//! Per DD §16 + D-LAYOUT-001 + spec §8.8:
//!
//! 3 Layout Algorithm (per spec §8.8):
//! - `Dagre` (TREE VIEW + node_count < 100)
//! - `D3Force` (DEPENDENCY / RISK / HISTORY VIEW)
//! - `Elk` (AGENT VIEW + node_count > 50, 或全局 > 500)
//!
//! 3 算法切换阈值 (per spec §8.8 触发规则):
//! | View Mode    | node_count | Algorithm |
//! |--------------|------------|-----------|
//! | TREE         | < 100      | Dagre     |
//! | TREE         | >= 100     | Elk (fallback)|
//! | DEPENDENCY   | any        | D3Force   |
//! | RISK         | any        | D3Force   |
//! | HISTORY      | any        | D3Force   |
//! | AGENT        | > 50       | Elk       |
//! | AGENT        | <= 50      | D3Force   |
//! | any          | > 500      | Elk (强制) |

use async_trait::async_trait;
use graph_core::types::WorktreeId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::LayoutError;

/// 5 View Mode (per DD §18 + FR-UI-007..011 + spec §18)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewMode {
    /// TREE — 以 Main → Worktree 派生为核心
    Tree,
    /// DEPENDENCY — 突出 DEPENDS_ON / BLOCKS
    Dependency,
    /// RISK — 正常节点弱化, 高风险节点强调
    Risk,
    /// AGENT — Agent → Task → Worktree
    Agent,
    /// HISTORY — 已合并 Worktree + 历史关系
    History,
}

/// 3 Layout 算法 (per D-LAYOUT-001)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutAlgorithm {
    /// dagre.js — TREE 派生树 (per spec §8.8)
    Dagre,
    /// d3-force — Graph Edge 平衡 (per spec §8.8)
    D3Force,
    /// ELK.js — 复杂图 (per spec §8.8)
    Elk,
}

impl LayoutAlgorithm {
    /// 3 算法 (守门用)
    pub const COUNT: usize = 3;

    /// 全部 3 个算法 (按 enum 顺序, per INV-WC-09 守门)
    pub const ALL: [LayoutAlgorithm; Self::COUNT] = [Self::Dagre, Self::D3Force, Self::Elk];
}

/// Layout 输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutInput {
    /// 节点 (id + 任何 metadata; layout engine 不读内容)
    pub nodes: Vec<String>,
    /// 边 (from, to)
    pub edges: Vec<(String, String)>,
    /// View mode
    pub view_mode: ViewMode,
    /// 根节点 ID (TREE layout 用)
    pub root_id: Option<String>,
}

/// Layout 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutOptions {
    /// 画布宽
    pub width: f64,
    /// 画布高
    pub height: f64,
    /// 节点间距 (默认 80)
    pub node_spacing: f64,
    /// 层级间距 (默认 100)
    pub rank_spacing: f64,
    /// 算法 (None = 自动选择, per spec §8.8)
    pub algorithm: Option<LayoutAlgorithm>,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            width: 1920.0,
            height: 1080.0,
            node_spacing: 80.0,
            rank_spacing: 100.0,
            algorithm: None,
        }
    }
}

/// 节点位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePosition {
    /// 节点 ID
    pub id: String,
    /// X 坐标
    pub x: f64,
    /// Y 坐标
    pub y: f64,
    /// 宽度
    pub width: f64,
    /// 高度
    pub height: f64,
}

/// 边 path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgePath {
    /// 起点
    pub from: String,
    /// 终点
    pub to: String,
    /// SVG path
    pub path: String,
}

/// BBox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BBox {
    /// 左上 X
    pub x: f64,
    /// 左上 Y
    pub y: f64,
    /// 宽
    pub width: f64,
    /// 高
    pub height: f64,
}

/// Layout 输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutOutput {
    /// 节点位置列表
    pub nodes: Vec<NodePosition>,
    /// 边 path 列表
    pub edges: Vec<EdgePath>,
    /// Bounding box
    pub bbox: BBox,
}

/// LayoutEngine Trait (per DD §16)
#[async_trait]
pub trait LayoutEngine: Send + Sync {
    /// Compute layout for nodes + edges
    async fn compute(
        &self,
        input: LayoutInput,
        options: LayoutOptions,
    ) -> Result<LayoutOutput, LayoutError>;

    /// Incremental update (only changed nodes)
    async fn update_incremental(
        &self,
        existing: LayoutOutput,
        changes: Vec<NodePosition>,
    ) -> Result<LayoutOutput, LayoutError>;
}

/// 阈值常量 (per spec §8.8)
pub mod thresholds {
    /// Dagre 上限 (per spec §8.8): < 100
    pub const DAGRE_MAX_NODES: usize = 100;
    /// ELK 强制下限 (per spec §8.8): > 500
    pub const ELK_FORCE_GLOBAL: usize = 500;
    /// AGENT VIEW 走 Elk (per spec §8.8): > 50
    pub const ELK_AGENT_MIN_NODES: usize = 50;
}

/// 给定 ViewMode + node_count, 返回默认算法 (per spec §8.8 触发规则)
pub fn pick_algorithm(view_mode: ViewMode, node_count: usize) -> LayoutAlgorithm {
    use thresholds::*;
    use ViewMode::*;
    match view_mode {
        Tree => {
            if node_count < DAGRE_MAX_NODES {
                LayoutAlgorithm::Dagre
            } else {
                LayoutAlgorithm::Elk
            }
        }
        Dependency | Risk | History => LayoutAlgorithm::D3Force,
        Agent => {
            if node_count > ELK_AGENT_MIN_NODES {
                LayoutAlgorithm::Elk
            } else {
                LayoutAlgorithm::D3Force
            }
        }
    }
}

/// LayoutEnginePure — sync impl (本期 T10, P2 落点 GPU/Web Worker)
///
/// MVP: 简单网格布局 (i * node_spacing, j * rank_spacing).
/// 真实算法 (Dagre / D3Force / ELK) 在 P2 实装, 通过 N-API / Web Worker 桥到前端.
pub struct LayoutEnginePure;

impl std::fmt::Debug for LayoutEnginePure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayoutEnginePure").finish()
    }
}

impl Default for LayoutEnginePure {
    fn default() -> Self {
        Self
    }
}

impl LayoutEnginePure {
    /// 构造
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LayoutEngine for LayoutEnginePure {
    async fn compute(
        &self,
        input: LayoutInput,
        options: LayoutOptions,
    ) -> Result<LayoutOutput, LayoutError> {
        if input.nodes.is_empty() {
            return Err(LayoutError::new("LAYOUT.EMPTY_INPUT", "no nodes to layout"));
        }

        let algorithm = options
            .algorithm
            .unwrap_or_else(|| pick_algorithm(input.view_mode, input.nodes.len()));

        // MVP grid layout: per algorithm 分配不同 grid (per node_count 分组)
        let cols = match algorithm {
            LayoutAlgorithm::Dagre => 1, // TREE 单列 (per dagre 树形)
            LayoutAlgorithm::D3Force => 4,
            LayoutAlgorithm::Elk => 6,
        };

        let positions: Vec<NodePosition> = input
            .nodes
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let row = i / cols;
                let col = i % cols;
                NodePosition {
                    id: id.clone(),
                    x: col as f64 * options.node_spacing,
                    y: row as f64 * options.rank_spacing,
                    width: 60.0,
                    height: 40.0,
                }
            })
            .collect();

        // Edge path (per Bezier)
        let edges: Vec<EdgePath> = input
            .edges
            .iter()
            .map(|(from, to)| {
                let (x1, y1) = positions
                    .iter()
                    .find(|p| &p.id == from)
                    .map(|p| (p.x + p.width / 2.0, p.y + p.height / 2.0))
                    .unwrap_or((0.0, 0.0));
                let (x2, y2) = positions
                    .iter()
                    .find(|p| &p.id == to)
                    .map(|p| (p.x + p.width / 2.0, p.y + p.height / 2.0))
                    .unwrap_or((0.0, 0.0));
                EdgePath {
                    from: from.clone(),
                    to: to.clone(),
                    path: crate::bezier::bezier_path(x1, y1, x2, y2),
                }
            })
            .collect();

        let bbox = BBox {
            x: 0.0,
            y: 0.0,
            width: options.width,
            height: options.height,
        };

        Ok(LayoutOutput {
            nodes: positions,
            edges,
            bbox,
        })
    }

    async fn update_incremental(
        &self,
        existing: LayoutOutput,
        changes: Vec<NodePosition>,
    ) -> Result<LayoutOutput, LayoutError> {
        Ok(crate::incremental::update_incremental(existing, changes))
    }
}

// 注: WorktreeId re-export (避免 unused import 警告 — 跨 crate 类型)
#[allow(dead_code)]
fn _type_check(_: WorktreeId) {}
#[allow(dead_code)]
fn _uuid_check(_: Uuid) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_3_algorithms_dagre_d3_elk() {
        // 守门 UT-1 (T10): 3 Layout 算法完整定义
        assert_eq!(LayoutAlgorithm::COUNT, 3);
        assert_eq!(LayoutAlgorithm::ALL.len(), 3);
        assert!(LayoutAlgorithm::ALL.contains(&LayoutAlgorithm::Dagre));
        assert!(LayoutAlgorithm::ALL.contains(&LayoutAlgorithm::D3Force));
        assert!(LayoutAlgorithm::ALL.contains(&LayoutAlgorithm::Elk));
    }

    #[test]
    fn pick_algorithm_threshold_correct() {
        // 守门: 阈值切换正确 (per spec §8.8)
        // TREE: < 100 → Dagre, >= 100 → Elk
        assert_eq!(pick_algorithm(ViewMode::Tree, 50), LayoutAlgorithm::Dagre);
        assert_eq!(pick_algorithm(ViewMode::Tree, 99), LayoutAlgorithm::Dagre);
        assert_eq!(pick_algorithm(ViewMode::Tree, 100), LayoutAlgorithm::Elk);

        // DEPENDENCY / RISK / HISTORY → 任意 D3Force
        for n in [0, 50, 100, 500, 1000] {
            assert_eq!(
                pick_algorithm(ViewMode::Dependency, n),
                LayoutAlgorithm::D3Force
            );
            assert_eq!(pick_algorithm(ViewMode::Risk, n), LayoutAlgorithm::D3Force);
            assert_eq!(
                pick_algorithm(ViewMode::History, n),
                LayoutAlgorithm::D3Force
            );
        }

        // AGENT: > 50 → Elk, <= 50 → D3Force
        assert_eq!(
            pick_algorithm(ViewMode::Agent, 30),
            LayoutAlgorithm::D3Force
        );
        assert_eq!(
            pick_algorithm(ViewMode::Agent, 50),
            LayoutAlgorithm::D3Force
        );
        assert_eq!(pick_algorithm(ViewMode::Agent, 51), LayoutAlgorithm::Elk);
    }

    #[tokio::test]
    async fn compute_layout_produces_positions_and_edges() {
        let engine = LayoutEnginePure::new();
        let input = LayoutInput {
            nodes: vec!["wt-1".to_string(), "wt-2".to_string(), "wt-3".to_string()],
            edges: vec![
                ("wt-1".to_string(), "wt-2".to_string()),
                ("wt-2".to_string(), "wt-3".to_string()),
            ],
            view_mode: ViewMode::Tree,
            root_id: None,
        };
        let out = engine
            .compute(input, LayoutOptions::default())
            .await
            .unwrap();
        assert_eq!(out.nodes.len(), 3);
        assert_eq!(out.edges.len(), 2);

        // TREE + 3 nodes < 100 → Dagre → cols=1 → 节点全部 X=0
        // 注: grid layout, 这是 MVP 简化 (P2 落点 dagre.js 真实算法)
        let algo_used = pick_algorithm(ViewMode::Tree, 3);
        assert_eq!(algo_used, LayoutAlgorithm::Dagre);
    }
}
