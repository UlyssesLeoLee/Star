//! `renderer.rs` — `CanvasRenderer` Trait + 8 方法 (per spec §4.1)
//!
//! Trait 定义 (per DD §11 + spec §4.1):
//! - `render_node` — 渲染单一节点
//! - `render_edge` — 渲染边
//! - `update_viewport` — 更新视口
//! - `hit_test` — 点命中测试 (P2 stub)
//! - `apply_lod` — 应用 LOD
//! - `apply_focus` — 应用 Focus Mode
//! - `apply_view_mode` — 应用 View Mode
//! - `to_reactflow_json` — 输出 JSON
//!
//! MVP: Trait + 简单 RenderState + 默认实现.

use crate::error::CanvasError;
use crate::focus::{FocusCalculator, FocusResult, Hop};
use crate::lod::{LodLevel, LodSelector};
use crate::reactflow::{ReactFlowAdapter, ReactFlowNode, ReactFlowSpec};
use crate::view_mode::ViewMode;
use crate::viewport::{NodePosition, Viewport, VirtualizationPolicy};
use graph_core::node::NodePayload;
use graph_core::types::WorktreeId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 渲染状态 (per DD §11)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderState {
    /// 当前 LOD
    pub lod: LodLevel,
    /// 当前 View Mode
    pub view_mode: ViewMode,
    /// 当前视口
    pub viewport: Viewport,
    /// 当前 zoom
    pub zoom: f32,
    /// 当前 focus (None = 无 focus)
    pub focus: Option<FocusResult>,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            lod: LodLevel::L1RepoWorktree,
            view_mode: ViewMode::Dependency,
            viewport: Viewport::new(0.0, 0.0, 1920.0, 1080.0),
            zoom: 0.5,
            focus: None,
        }
    }
}

/// CanvasRenderer Trait (per DD §11 + spec §4.1)
pub trait CanvasRenderer: Send + Sync {
    /// 渲染单一节点
    fn render_node(
        &self,
        payload: &NodePayload,
        position: NodePosition,
    ) -> Result<ReactFlowNode, CanvasError>;

    /// 渲染边
    fn render_edge(
        &self,
        source: WorktreeId,
        target: WorktreeId,
        edge_kind: &str,
    ) -> Result<crate::reactflow::ReactFlowEdge, CanvasError>;

    /// 更新视口
    fn update_viewport(&mut self, viewport: Viewport) -> Result<(), CanvasError>;

    /// 命中测试 (P2 stub)
    fn hit_test(&self, _x: f32, _y: f32) -> Result<Option<WorktreeId>, CanvasError> {
        Ok(None)
    }

    /// 应用 LOD
    fn apply_lod(&mut self, zoom: f32) -> Result<LodLevel, CanvasError>;

    /// 应用 Focus Mode (N-hop)
    fn apply_focus(
        &mut self,
        focus_id: WorktreeId,
        hop: Hop,
        adjacency: &HashMap<WorktreeId, Vec<WorktreeId>>,
    ) -> Result<FocusResult, CanvasError>;

    /// 应用 View Mode
    fn apply_view_mode(&mut self, mode: ViewMode) -> Result<(), CanvasError>;

    /// 输出 React-Flow JSON
    fn to_reactflow_json(&self, spec: &ReactFlowSpec) -> Result<String, CanvasError>;
}

/// 标准 CanvasRenderer 实现
pub struct StandardCanvasRenderer {
    state: RenderState,
}

impl Default for StandardCanvasRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl StandardCanvasRenderer {
    /// 构造新 renderer
    pub fn new() -> Self {
        Self {
            state: RenderState::default(),
        }
    }

    /// 取得当前 RenderState
    pub fn state(&self) -> &RenderState {
        &self.state
    }
}

impl CanvasRenderer for StandardCanvasRenderer {
    fn render_node(
        &self,
        payload: &NodePayload,
        position: NodePosition,
    ) -> Result<ReactFlowNode, CanvasError> {
        // 节点 kind 是否在当前 LOD 可见
        let kind = payload.kind();
        if !LodSelector::is_node_kind_in_lod(kind, self.state.lod) {
            return Err(CanvasError::new(
                "LOD_INVALID",
                format!("node kind {:?} not visible at LOD {:?}", kind, self.state.lod),
            ));
        }
        // viewport virtualization
        let policy = VirtualizationPolicy::new();
        if !policy.is_in_viewport(position, self.state.viewport) {
            return Err(CanvasError::new(
                "VIEWPORT_INVALID",
                "node position outside viewport",
            ));
        }
        let adapter = ReactFlowAdapter::new();
        adapter.convert_node(payload, (position.x, position.y))
    }

    fn render_edge(
        &self,
        source: WorktreeId,
        target: WorktreeId,
        edge_kind: &str,
    ) -> Result<crate::reactflow::ReactFlowEdge, CanvasError> {
        let adapter = ReactFlowAdapter::new();
        Ok(adapter.build_edge(source, target, edge_kind))
    }

    fn update_viewport(&mut self, viewport: Viewport) -> Result<(), CanvasError> {
        if !viewport.is_valid() {
            return Err(CanvasError::new(
                "VIEWPORT_INVALID",
                "viewport width/height must be > 0",
            ));
        }
        self.state.viewport = viewport;
        Ok(())
    }

    fn apply_lod(&mut self, zoom: f32) -> Result<LodLevel, CanvasError> {
        let lod = LodSelector::new().from_zoom(zoom)?;
        self.state.lod = lod;
        self.state.zoom = zoom;
        Ok(lod)
    }

    fn apply_focus(
        &mut self,
        focus_id: WorktreeId,
        hop: Hop,
        adjacency: &HashMap<WorktreeId, Vec<WorktreeId>>,
    ) -> Result<FocusResult, CanvasError> {
        let result = FocusCalculator::new().calculate(focus_id, hop, adjacency);
        self.state.focus = Some(result.clone());
        Ok(result)
    }

    fn apply_view_mode(&mut self, mode: ViewMode) -> Result<(), CanvasError> {
        self.state.view_mode = mode;
        Ok(())
    }

    fn to_reactflow_json(&self, spec: &ReactFlowSpec) -> Result<String, CanvasError> {
        ReactFlowAdapter::new().to_json(spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_default_state() {
        let r = StandardCanvasRenderer::new();
        assert_eq!(r.state().lod, LodLevel::L1RepoWorktree);
        assert_eq!(r.state().view_mode, ViewMode::Dependency);
    }

    #[test]
    fn renderer_apply_lod_updates_state() {
        let mut r = StandardCanvasRenderer::new();
        let lod = r.apply_lod(0.1).expect("ok");
        assert_eq!(lod, LodLevel::L0Project);
        assert_eq!(r.state().lod, LodLevel::L0Project);
    }

    #[test]
    fn renderer_apply_view_mode() {
        let mut r = StandardCanvasRenderer::new();
        r.apply_view_mode(ViewMode::Risk).expect("ok");
        assert_eq!(r.state().view_mode, ViewMode::Risk);
    }

    #[test]
    fn renderer_update_viewport_rejects_invalid() {
        let mut r = StandardCanvasRenderer::new();
        let bad = Viewport::new(0.0, 0.0, 0.0, 0.0);
        assert!(r.update_viewport(bad).is_err());
    }

    #[test]
    fn renderer_to_json_empty_spec() {
        let r = StandardCanvasRenderer::new();
        let spec = ReactFlowSpec {
            nodes: vec![],
            edges: vec![],
        };
        let json = r.to_reactflow_json(&spec).expect("ok");
        assert!(json.contains("\"nodes\""));
        assert!(json.contains("\"edges\""));
    }
}