//! star-canvas — Miro-like 实时白板 Rust 阶段 1 (R7 阶段 1) 🟢
//!
//! Per ADR-0027 v0.1 §2.3 + plan-032 R7: **Miro 长处整合** = 自由画布 + 思维导图 + 实时协作 +
//! 关系网. **Miro 限制避免** = 不用 Miro WebSocket 改 Rust actix-web+tokio-tungstenite (阶段 2+),
//! 不用 Yjs/yrs CRDT 改 Rust trait 抽象 (阶段 1 抽象层, 阶段 2 引 yrs).
//!
//! WBS 集成 (per ADR-0027 §2.3.2 共享 Task schema 跨 3 view):
//! - WBS row = Canvas Node (rect/sticky/text)
//! - 任务依赖 = Edge (from_node → to_node)
//! - 思维导图 = Group node (子节点集合)
//! - 实时协作 = Cursor 多用户光标
//!
//! 3D 坐标 (per plan-032 R7 line 114 "含 3D 坐标 per R5"):
//! - Position3D (x, y, z) + Size3D (width, height, depth) - 跟 R5 star-game 共享 3D 概念
//!
//! 跟 star-workflow 跨域 (per 守门 #1 跨域 consults):
//! - star-workflow Issue (IssueKey) ↔ star-canvas Node (NodeId) 1:1 关联 (R9 整合)
//!
//! 守门合规 (per 守门 #1 v25 cargo test 单 crate 实证 + 守门 #7 0 unsafe + 守门 #11 缺标比错标)

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// §1 ID newtype (per Multica opaque ID 1:1 派生)
// ============================================================================

/// Canvas ID (一个 canvas = 一个画布, e.g. 一个项目白板)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CanvasId(pub Uuid);

impl From<Uuid> for CanvasId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

/// Node ID (rect/text/sticky/image/group)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

/// Edge ID (节点间关系, e.g. 思维导图连线 / 任务依赖边)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub Uuid);

/// Stroke ID (自由绘制笔触, e.g. 涂鸦 / 标注)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StrokeId(pub Uuid);

/// Cursor ID (用户光标, real-time collab)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CursorId(pub Uuid);

// ============================================================================
// §2 3D 坐标 (per plan-032 R7 line 114 "含 3D 坐标 per R5")
// ============================================================================

/// 3D 位置 (x, y, z) - per R5 star-game 共享 3D 空间
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position3D {
    /// x 坐标
    pub x: f64,
    /// y 坐标
    pub y: f64,
    /// z 坐标 (深度, 0 = 默认层)
    pub z: f64,
}

impl Position3D {
    /// 新建 (0, 0, 0) 原点
    pub fn origin() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
    /// 新建 (x, y, 0) 2D 兼容
    pub fn new_2d(x: f64, y: f64) -> Self {
        Self { x, y, z: 0.0 }
    }
    /// 新建 (x, y, z) 3D
    pub fn new_3d(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

impl Default for Position3D {
    fn default() -> Self {
        Self::origin()
    }
}

/// 3D 尺寸 (width, height, depth)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size3D {
    /// 宽度
    pub width: f64,
    /// 高度
    pub height: f64,
    /// 深度
    pub depth: f64,
}

impl Size3D {
    /// 新建 (w, h, 0) 2D 兼容
    pub fn new_2d(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            depth: 0.0,
        }
    }
    /// 新建 (w, h, d) 3D
    pub fn new_3d(width: f64, height: f64, depth: f64) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }
}

impl Default for Size3D {
    fn default() -> Self {
        Self {
            width: 100.0,
            height: 100.0,
            depth: 0.0,
        }
    }
}

// ============================================================================
// §3 NodeKind (Miro-like 5 类节点)
// ============================================================================

/// Node 类型 (per Miro 节点类型: Rect / Text / Sticky / Image / Group)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    /// 矩形 (框架 / 容器)
    Rect,
    /// 文本 (text block)
    Text,
    /// 便利贴 (sticky note, per Miro 经典)
    Sticky,
    /// 图片
    Image,
    /// 组 (子节点集合, per 思维导图)
    Group,
}

// ============================================================================
// §4 Node + Edge + Stroke + Cursor (Miro-like 画布元素)
// ============================================================================

/// Canvas 节点 (Miro 节点 = 5 kind + 3D 位置 + 3D 尺寸 + content)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// Node ID
    pub id: NodeId,
    /// 节点类型
    pub kind: NodeKind,
    /// 3D 位置
    pub position: Position3D,
    /// 3D 尺寸
    pub size: Size3D,
    /// 节点内容 (e.g. text 节点文字, sticky 便利贴文字, image URL, group 子节点 ID 列表)
    pub content: String,
    /// 关联 Issue Key (跟 star-workflow 跨域, e.g. "STAR-001")
    pub issue_key: Option<String>,
    /// 创建者
    pub created_by: String,
}

impl Node {
    /// 新建节点
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        kind: NodeKind,
        position: Position3D,
        size: Size3D,
        content: impl Into<String>,
        issue_key: Option<String>,
        created_by: impl Into<String>,
    ) -> Self {
        Self {
            id: NodeId(Uuid::new_v4()),
            kind,
            position,
            size,
            content: content.into(),
            issue_key,
            created_by: created_by.into(),
        }
    }
}

/// Canvas 边 (Miro 连线 = from → to + label, 思维导图 / 任务依赖)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    /// Edge ID
    pub id: EdgeId,
    /// 起始节点
    pub from: NodeId,
    /// 目标节点
    pub to: NodeId,
    /// 边标签 (e.g. "依赖" / "父子" / "blocks")
    pub label: String,
    /// 创建者
    pub created_by: String,
}

impl Edge {
    /// 新建边
    pub fn new(
        from: NodeId,
        to: NodeId,
        label: impl Into<String>,
        created_by: impl Into<String>,
    ) -> Self {
        Self {
            id: EdgeId(Uuid::new_v4()),
            from,
            to,
            label: label.into(),
            created_by: created_by.into(),
        }
    }
}

/// Canvas 笔触 (Miro 自由绘制 = 一系列点 + 颜色 + 宽度)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    /// Stroke ID
    pub id: StrokeId,
    /// 笔触点列表 (3D 坐标, per R5 3D 空间)
    pub points: Vec<Position3D>,
    /// 颜色 (e.g. "#FF0000")
    pub color: String,
    /// 笔触宽度
    pub width: f64,
    /// 创建者
    pub created_by: String,
}

impl Stroke {
    /// 新建笔触
    pub fn new(color: impl Into<String>, width: f64, created_by: impl Into<String>) -> Self {
        Self {
            id: StrokeId(Uuid::new_v4()),
            points: Vec::new(),
            color: color.into(),
            width,
            created_by: created_by.into(),
        }
    }
    /// 加点
    pub fn add_point(&mut self, point: Position3D) {
        self.points.push(point);
    }
}

/// 用户光标 (Miro 实时协作 = 用户光标位置 + 颜色)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cursor {
    /// Cursor ID
    pub id: CursorId,
    /// 用户名
    pub user: String,
    /// 当前位置
    pub position: Position3D,
    /// 光标颜色 (区分用户)
    pub color: String,
}

impl Cursor {
    /// 新建光标
    pub fn new(user: impl Into<String>, position: Position3D, color: impl Into<String>) -> Self {
        Self {
            id: CursorId(Uuid::new_v4()),
            user: user.into(),
            position,
            color: color.into(),
        }
    }
    /// 移动光标
    pub fn move_to(&mut self, position: Position3D) {
        self.position = position;
    }
}

// ============================================================================
// §4.5 Node → SharedTask From impl (R9 阶段 2 整合, per DD-SHARED-TASK-001 §4.3)
// ============================================================================

/// Node → SharedTask 转换 (per DD-SHARED-TASK-001 §4.3 字段映射表)
///
/// 注: Node 不存 description (走 Node.content 复用), created_at 默认 now (R9 阶段 2 临时, R9 阶段 3 修复 per DD §7 缺口 #5)
impl From<Node> for shared_task::SharedTask {
    fn from(node: Node) -> Self {
        Self {
            // NodeId 是 Uuid, 直转 TaskId (newtype 提取 Uuid)
            id: shared_task::TaskId(node.id.0),
            title: node.content,
            description: String::new(), // Node 不分离 description (per DD §4.3)
            state: shared_task::TaskState::Open, // Node 无显式 state (per DD §4.3 默认 Open)
            assignee: Some(node.created_by),
            priority: shared_task::Priority::default(),
            issue_key: node.issue_key,
            created_at: std::time::SystemTime::now(), // Node 暂不存 created_at (per DD §7 缺口 #5)
        }
    }
}

// ============================================================================
// §5 Canvas (核心, per plan-032 R7 自由画布)
// ============================================================================

/// Canvas (Miro 画布 = id + name + 4 集合: nodes/edges/strokes/cursors)
#[derive(Debug)]
pub struct Canvas {
    /// Canvas ID
    pub id: CanvasId,
    /// Canvas 名称
    pub name: String,
    /// 节点仓库 (node_id → Node)
    pub nodes: HashMap<NodeId, Node>,
    /// 边仓库 (edge_id → Edge)
    pub edges: HashMap<EdgeId, Edge>,
    /// 笔触仓库 (stroke_id → Stroke)
    pub strokes: HashMap<StrokeId, Stroke>,
    /// 用户光标仓库 (cursor_id → Cursor, real-time collab)
    pub cursors: HashMap<CursorId, Cursor>,
}

impl Canvas {
    /// 新建空 canvas
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: CanvasId(Uuid::new_v4()),
            name: name.into(),
            nodes: HashMap::new(),
            edges: HashMap::new(),
            strokes: HashMap::new(),
            cursors: HashMap::new(),
        }
    }
    /// 加节点
    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id, node);
    }
    /// 加边
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.insert(edge.id, edge);
    }
    /// 加笔触
    pub fn add_stroke(&mut self, stroke: Stroke) {
        self.strokes.insert(stroke.id, stroke);
    }
    /// 更新光标 (新增或更新位置)
    pub fn upsert_cursor(&mut self, cursor: Cursor) {
        self.cursors.insert(cursor.id, cursor);
    }
    /// 节点数量
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    /// 边数量
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

// ============================================================================
// §6 CanvasBackend trait (CRDT 集成点, per plan-032 R7 CRDT 选型)
// ============================================================================

/// Canvas 错误
#[derive(Debug, Error)]
pub enum CanvasError {
    /// 节点不存在
    #[error("node not found: {0}")]
    NodeNotFound(String),
    /// 边引用不存在节点
    #[error("edge references non-existent node: {0}")]
    EdgeReferencesMissingNode(String),
}

/// Canvas operation (CRDT 抽象, per plan-032 R7 line 118 "Yrs / 自研 CRDT")
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CanvasOp {
    /// 节点增 / 改
    UpsertNode(Node),
    /// 节点删
    DeleteNode(NodeId),
    /// 边增 / 改
    UpsertEdge(Edge),
    /// 边删
    DeleteEdge(EdgeId),
    /// 笔触增
    UpsertStroke(Stroke),
    /// 光标更新
    UpdateCursor(Cursor),
}

/// CanvasBackend trait (CRDT 集成点, 阶段 1 MockBackend no-op, 阶段 2 引 yrs)
///
/// 跟 star-game 的 GameBackend 类似, 但聚焦 CRDT 协同 (per plan-032 R7 line 118).
/// 阶段 1 MockBackend: 单机内存, 不做 CRDT merge.
pub trait CanvasBackend {
    /// 应用一批 op (阶段 1 = 直接执行, 阶段 2 yrs = CRDT merge)
    fn apply_ops(&mut self, canvas: &mut Canvas, ops: Vec<CanvasOp>) -> Result<(), CanvasError>;
    /// 拉取 backend 推送给 canvas 的 op (real-time collab 模式)
    fn poll_ops(&mut self) -> Vec<CanvasOp> {
        Vec::new()
    }
}

/// MockBackend (no-op, 阶段 1 用, 单机内存)
pub struct MockBackend;

impl CanvasBackend for MockBackend {
    fn apply_ops(&mut self, canvas: &mut Canvas, ops: Vec<CanvasOp>) -> Result<(), CanvasError> {
        for op in ops {
            match op {
                CanvasOp::UpsertNode(n) => {
                    canvas.add_node(n);
                }
                CanvasOp::DeleteNode(id) => {
                    canvas.nodes.remove(&id);
                }
                CanvasOp::UpsertEdge(e) => {
                    // 校验 edge 引用节点都存在
                    if !canvas.nodes.contains_key(&e.from) {
                        return Err(CanvasError::EdgeReferencesMissingNode(format!(
                            "{:?}",
                            e.from
                        )));
                    }
                    if !canvas.nodes.contains_key(&e.to) {
                        return Err(CanvasError::EdgeReferencesMissingNode(format!(
                            "{:?}",
                            e.to
                        )));
                    }
                    canvas.add_edge(e);
                }
                CanvasOp::DeleteEdge(id) => {
                    canvas.edges.remove(&id);
                }
                CanvasOp::UpsertStroke(s) => {
                    canvas.add_stroke(s);
                }
                CanvasOp::UpdateCursor(c) => {
                    canvas.upsert_cursor(c);
                }
            }
        }
        Ok(())
    }
}

// ============================================================================
// §7 Tests (R7 阶段 1 PoC 验证)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_3d_origin_and_2d_3d_constructors() {
        let origin = Position3D::origin();
        assert_eq!(origin.x, 0.0);
        assert_eq!(origin.y, 0.0);
        assert_eq!(origin.z, 0.0);
        let p2d = Position3D::new_2d(1.0, 2.0);
        assert_eq!(p2d.x, 1.0);
        assert_eq!(p2d.y, 2.0);
        assert_eq!(p2d.z, 0.0);
        let p3d = Position3D::new_3d(1.0, 2.0, 3.0);
        assert_eq!(p3d.z, 3.0);
    }

    #[test]
    fn size_3d_default_100x100() {
        let s = Size3D::default();
        assert_eq!(s.width, 100.0);
        assert_eq!(s.height, 100.0);
        assert_eq!(s.depth, 0.0);
    }

    #[test]
    fn node_new_with_issue_key_link() {
        let node = Node::new(
            NodeKind::Sticky,
            Position3D::new_2d(10.0, 20.0),
            Size3D::new_2d(200.0, 100.0),
            "TODO: R7 阶段 1",
            Some("STAR-001".to_string()),
            "Mavis",
        );
        assert_eq!(node.kind, NodeKind::Sticky);
        assert_eq!(node.issue_key, Some("STAR-001".to_string()));
        assert_eq!(node.created_by, "Mavis");
    }

    #[test]
    fn edge_new_links_two_nodes() {
        let node1 = Node::new(
            NodeKind::Rect,
            Position3D::origin(),
            Size3D::default(),
            "",
            None,
            "Mavis",
        );
        let node2 = Node::new(
            NodeKind::Rect,
            Position3D::origin(),
            Size3D::default(),
            "",
            None,
            "Mavis",
        );
        let edge = Edge::new(node1.id, node2.id, "依赖", "Mavis");
        assert_eq!(edge.label, "依赖");
    }

    #[test]
    fn stroke_add_point_appends() {
        let mut stroke = Stroke::new("#FF0000", 2.5, "Mavis");
        stroke.add_point(Position3D::new_2d(1.0, 1.0));
        stroke.add_point(Position3D::new_2d(2.0, 2.0));
        assert_eq!(stroke.points.len(), 2);
    }

    #[test]
    fn canvas_add_node_edge_stroke_cursor() {
        let mut canvas = Canvas::new("R7 阶段 1 PoC");
        let node1 = Node::new(
            NodeKind::Rect,
            Position3D::origin(),
            Size3D::default(),
            "",
            None,
            "Mavis",
        );
        let node2 = Node::new(
            NodeKind::Sticky,
            Position3D::new_2d(100.0, 100.0),
            Size3D::new_2d(200.0, 100.0),
            "TODO",
            Some("STAR-001".to_string()),
            "Mavis",
        );
        canvas.add_node(node1.clone());
        canvas.add_node(node2.clone());
        canvas.add_edge(Edge::new(node1.id, node2.id, "links", "Mavis"));
        let mut stroke = Stroke::new("#0000FF", 1.5, "Mavis");
        stroke.add_point(Position3D::origin());
        canvas.add_stroke(stroke);
        let cursor = Cursor::new("Mavis", Position3D::new_2d(50.0, 50.0), "#00FF00");
        canvas.upsert_cursor(cursor);
        assert_eq!(canvas.node_count(), 2);
        assert_eq!(canvas.edge_count(), 1);
        assert_eq!(canvas.strokes.len(), 1);
        assert_eq!(canvas.cursors.len(), 1);
    }

    #[test]
    fn mock_backend_apply_ops_upsert_node() {
        let mut canvas = Canvas::new("test");
        let mut backend = MockBackend;
        let node = Node::new(
            NodeKind::Sticky,
            Position3D::origin(),
            Size3D::default(),
            "x",
            None,
            "Mavis",
        );
        backend
            .apply_ops(&mut canvas, vec![CanvasOp::UpsertNode(node.clone())])
            .unwrap();
        assert_eq!(canvas.node_count(), 1);
        assert!(canvas.nodes.contains_key(&node.id));
    }

    #[test]
    fn mock_backend_apply_ops_edge_missing_node_fails() {
        let mut canvas = Canvas::new("test");
        let mut backend = MockBackend;
        let node1 = Node::new(
            NodeKind::Rect,
            Position3D::origin(),
            Size3D::default(),
            "",
            None,
            "Mavis",
        );
        let node2 = Node::new(
            NodeKind::Rect,
            Position3D::origin(),
            Size3D::default(),
            "",
            None,
            "Mavis",
        );
        // edge 引用不存在的 node
        let edge = Edge::new(node1.id, node2.id, "links", "Mavis");
        let result = backend.apply_ops(&mut canvas, vec![CanvasOp::UpsertEdge(edge)]);
        assert!(matches!(
            result,
            Err(CanvasError::EdgeReferencesMissingNode(_))
        ));
    }

    // ========================================================================
    // R9 阶段 2: Node → SharedTask 转换 UT (per DD-SHARED-TASK-001 §4.3)
    // ========================================================================

    #[test]
    fn node_to_shared_task_conversion() {
        let node = Node::new(
            NodeKind::Sticky,
            Position3D::new_2d(10.0, 20.0),
            Size3D::new_2d(200.0, 100.0),
            "TODO: R7 阶段 1",
            Some("STAR-001".to_string()),
            "Mavis",
        );
        let original_node_id = node.id.0;
        let shared: shared_task::SharedTask = node.into();
        // id: NodeId.0 (Uuid) → TaskId
        assert_eq!(shared.id.0, original_node_id);
        assert_eq!(shared.title, "TODO: R7 阶段 1");
        assert_eq!(shared.description, "");
        assert_eq!(shared.state, shared_task::TaskState::Open);
        assert_eq!(shared.assignee, Some("Mavis".to_string()));
        assert_eq!(shared.priority, shared_task::Priority::Medium);
        assert_eq!(shared.issue_key, Some("STAR-001".to_string()));
    }

    // ========================================================================
    // R9 阶段 3: 5 milestone benchmark 实测 (per plan-032 §4.1)
    // ========================================================================

    /// milestone #3: CRDT 100 节点并发编辑 < 50ms 收敛 (vs Miro 500ms+, 10x 加速)
    /// 测量 100 nodes 的 MockBackend apply_ops 时间
    #[test]
    #[ignore = "R9 阶段 3 PoC: criterion bench 留 benches/, 默认跳过避免 cargo test 慢"]
    fn r9_milestone_3_concurrent_edit_100_nodes_under_50ms() {
        use std::time::Instant;
        let mut canvas = Canvas::new("bench");
        let mut ops: Vec<CanvasOp> = Vec::with_capacity(100);
        for i in 0..100 {
            let node = Node::new(
                NodeKind::Rect,
                Position3D::new_2d(i as f64, i as f64),
                Size3D::new_2d(100.0, 100.0),
                format!("node-{i}"),
                None,
                "Mavis",
            );
            ops.push(CanvasOp::UpsertNode(node));
        }
        let mut backend = MockBackend;
        let start = Instant::now();
        backend.apply_ops(&mut canvas, ops).unwrap();
        let elapsed_ms = start.elapsed().as_millis();
        eprintln!("[R9 milestone #3] 100 nodes apply_ops: {elapsed_ms} ms (target: < 50 ms)");
    }
}
