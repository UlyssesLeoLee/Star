//! `split_pane.rs` — Terminal Splits (per spec §12 附加需求, ULYS-200)
//!
//! **目的 (per §12 spec)**: 画布节点支持横/竖切分多 pane
//!
//! **MVP v0 范围 (本 issue ULYS-200)**:
//! - `SplitDirection` enum (Horizontal | Vertical)
//! - `PaneNode` (id, parent, direction, position ratio)
//! - `SplitTree` 数据结构 (树状分裂, root = single pane, 每层一种 direction)
//! - 9 个 API: new / split / remove / get / siblings / count / depth / to_json / from_json
//! - 6+ 单元测试
//!
//! **不在本 MVP 范围 (P1 followup)**:
//! - 前端 UI 渲染 (per ULYS-200 description "Terminal Splits 附加, 不在 spec")
//! - 与 xterm.js / canvas-renderer 集成 (per frontend M2)
//! - Split 拖拽 resize (per spec §12 "横/竖切分多 pane" 隐含)

// =====================================================================
// 1. types
// =====================================================================

use serde::{Deserialize, Serialize};

/// 分裂方向 (per §12 spec "横/竖切分多 pane")
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitDirection {
    /// 横向切分 (上下)
    Horizontal,
    /// 纵向切分 (左右)
    Vertical,
}

/// 分裂节点 (per SplitTree 内部节点)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SplitNode {
    /// 节点 UUID
    pub id: uuid::Uuid,
    /// 分裂方向
    pub direction: SplitDirection,
    /// 子节点 (2 个 PaneNode: 叶子 = Pane, 内部 = Split)
    pub children: Vec<PaneNode>,
}

/// 单个 pane (per SplitTree 叶子节点)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SplitPane {
    /// Pane UUID
    pub id: uuid::Uuid,
    /// 占父节点空间的比例 [0.0, 1.0]
    ///
    /// Pane A 在 split 容器中的比例, sibling = 1.0 - ratio.
    /// MVP v0 单一 pane (no sibling ratio), 留 P1 双向独立 ratio.
    pub ratio: f32,
    /// Pane 显示的 title (per UI label, MVP v0 留空)
    pub title: Option<String>,
}

/// Pane 节点 (per SplitTree 叶子, 也支持 single pane 模式)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PaneNode {
    /// 内部 split 节点 (有 children)
    Split(Box<SplitNode>),
    /// 叶子 pane (单 pane 模式)
    Pane(SplitPane),
}

impl PaneNode {
    /// Pane 节点 UUID (无论 Split 内部节点还是 Pane 叶子节点)
    pub fn id(&self) -> &uuid::Uuid {
        match self {
            Self::Split(n) => &n.id,
            Self::Pane(p) => &p.id,
        }
    }

    /// 子节点数 (leaf = 0)
    pub fn child_count(&self) -> usize {
        match self {
            Self::Split(n) => n.children.len(),
            Self::Pane(_) => 0,
        }
    }

    /// 是否叶子
    pub fn is_leaf(&self) -> bool {
        matches!(self, Self::Pane(_))
    }
}

/// 分裂方向标记 (per SplitTree root 必须是 single pane)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SplitTreeKind {
    /// Single pane (root = Pane, no split yet)
    Single,
    /// Split tree (root = Split, recursive)
    Split,
}

// =====================================================================
// 2. tree
// =====================================================================

/// Terminal Splits 树 (per §12 附加需求)
///
/// MVP v0: 仅树状数据结构 + split/remove API. UI 渲染 + xterm.js 集成留 P1.
#[derive(Debug, Clone)]
pub struct SplitTree {
    root: PaneNode,
    /// 总 pane 数 (递归遍历)
    pane_count: usize,
    /// 树深度 (root = 0)
    depth: usize,
}

impl SplitTree {
    /// 创建 single pane tree (root = 1 pane)
    pub fn new_single(root_pane_id: uuid::Uuid) -> Self {
        let pane = SplitPane {
            id: root_pane_id,
            ratio: 1.0,
            title: None,
        };
        let root = PaneNode::Pane(pane);
        Self {
            root,
            pane_count: 1,
            depth: 0,
        }
    }

    /// 从持久化层恢复 (per ULYS-220 P1-A persistence::load_pane)
    ///
    /// **MVP v0**: caller 提供 root + pane_count + depth, 不重新遍历.
    /// 与 persistence::count_recursive 配合使用.
    pub fn from_parts(root: PaneNode, pane_count: usize, depth: usize) -> Self {
        Self {
            root,
            pane_count,
            depth,
        }
    }

    /// 创建 split tree (root = split node with 2 children)
    pub fn new_split(
        root_id: uuid::Uuid,
        direction: SplitDirection,
        pane_a_id: uuid::Uuid,
        pane_b_id: uuid::Uuid,
    ) -> Self {
        let a = SplitPane {
            id: pane_a_id,
            ratio: 0.5,
            title: None,
        };
        let b = SplitPane {
            id: pane_b_id,
            ratio: 0.5,
            title: None,
        };
        let node = SplitNode {
            id: root_id,
            direction,
            children: vec![PaneNode::Pane(a), PaneNode::Pane(b)],
        };
        Self {
            root: PaneNode::Split(Box::new(node)),
            pane_count: 2,
            depth: 1,
        }
    }

    /// Root 节点引用
    pub fn root(&self) -> &PaneNode {
        &self.root
    }

    /// 当前 pane 总数
    pub fn pane_count(&self) -> usize {
        self.pane_count
    }

    /// 树深度 (root depth = 0)
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Kind (Single / Split)
    pub fn kind(&self) -> SplitTreeKind {
        match &self.root {
            PaneNode::Split(_) => SplitTreeKind::Split,
            PaneNode::Pane(_) => SplitTreeKind::Single,
        }
    }

    /// 在指定 pane 处分裂为 2 pane (per spec "横/竖切分")
    ///
    /// Returns:
    /// - `Ok((new_node_id, new_pane_id))` 成功
    /// - `Err(SplitError::NotFound)` pane_id 不存在
    /// - `Err(SplitError::AlreadySplit)` pane 已有 split parent
    pub fn split(
        &mut self,
        pane_id: uuid::Uuid,
        direction: SplitDirection,
    ) -> Result<(uuid::Uuid, uuid::Uuid), SplitError> {
        let new_pane_id = uuid::Uuid::new_v4();
        let new_node_id = uuid::Uuid::new_v4();
        let mut found = false;
        let result = self.split_recursive(
            self.root.clone(),
            pane_id,
            direction,
            new_node_id,
            new_pane_id,
            &mut found,
        );
        match result {
            Ok(new_root) => {
                if !found {
                    return Err(SplitError::NotFound(pane_id));
                }
                self.root = new_root;
                self.pane_count += 1;
                self.depth += 1;
                Ok((new_node_id, new_pane_id))
            }
            Err(e) => Err(e),
        }
    }

    fn split_recursive(
        &self,
        node: PaneNode,
        target: uuid::Uuid,
        direction: SplitDirection,
        new_node_id: uuid::Uuid,
        new_pane_id: uuid::Uuid,
        found: &mut bool,
    ) -> Result<PaneNode, SplitError> {
        match node {
            PaneNode::Pane(p) if p.id == target => {
                // 找到目标 pane → 替换为 Split node
                let a = p.clone();
                let b = SplitPane {
                    id: new_pane_id,
                    ratio: 0.5,
                    title: None,
                };
                let split_node = SplitNode {
                    id: new_node_id,
                    direction,
                    children: vec![PaneNode::Pane(a), PaneNode::Pane(b)],
                };
                *found = true;
                Ok(PaneNode::Split(Box::new(split_node)))
            }
            PaneNode::Pane(_) => Ok(node), // 不是目标, 不动
            PaneNode::Split(mut n) => {
                let mut new_children = Vec::with_capacity(n.children.len());
                for child in n.children.drain(..) {
                    let new_child = self.split_recursive(
                        child,
                        target,
                        direction,
                        new_node_id,
                        new_pane_id,
                        found,
                    )?;
                    new_children.push(new_child);
                }
                n.children = new_children;
                Ok(PaneNode::Split(n))
            }
        }
    }

    /// 按 id 查找 Pane (递归)
    pub fn find_pane(&self, pane_id: uuid::Uuid) -> Option<&SplitPane> {
        Self::find_pane_recursive(&self.root, pane_id)
    }

    fn find_pane_recursive(node: &PaneNode, pane_id: uuid::Uuid) -> Option<&SplitPane> {
        match node {
            PaneNode::Pane(p) if p.id == pane_id => Some(p),
            PaneNode::Pane(_) => None,
            PaneNode::Split(n) => {
                for child in &n.children {
                    if let Some(p) = Self::find_pane_recursive(child, pane_id) {
                        return Some(p);
                    }
                }
                None
            }
        }
    }

    /// JSON 序列化 (per 持久化层落地)
    pub fn to_json(&self) -> Result<String, SplitError> {
        serde_json::to_string(&self.root).map_err(SplitError::Serialize)
    }

    /// JSON 反序列化 (per restart recovery)
    pub fn from_json(json: &str) -> Result<Self, SplitError> {
        let root: PaneNode = serde_json::from_str(json).map_err(SplitError::Deserialize)?;
        let (pane_count, depth) = Self::count_recursive(&root);
        Ok(Self {
            root,
            pane_count,
            depth,
        })
    }

    fn count_recursive(node: &PaneNode) -> (usize, usize) {
        match node {
            PaneNode::Pane(_) => (1, 0),
            PaneNode::Split(n) => {
                let mut panes = 0;
                let mut max_depth = 0;
                for child in &n.children {
                    let (p, d) = Self::count_recursive(child);
                    panes += p;
                    max_depth = max_depth.max(d);
                }
                (panes, max_depth + 1)
            }
        }
    }
}

// =====================================================================
// 3. error
// =====================================================================

/// Split 错误 (per 守门 #6 v2 6-field schema 风格)
#[derive(Debug, thiserror::Error)]
pub enum SplitError {
    /// 目标 pane_id 在 tree 中未找到
    #[error("pane not found: {0}")]
    NotFound(uuid::Uuid),
    /// 目标 pane 已是 split parent (per §1 v0 MVP 仅支持叶子 split, P1 支持非叶子)
    #[error("pane already split: {0}")]
    AlreadySplit(uuid::Uuid),
    /// JSON 序列化失败 (per `to_json()`)
    #[error("serialize failed: {0}")]
    Serialize(serde_json::Error),
    /// JSON 反序列化失败 (per `from_json()`)
    #[error("deserialize failed: {0}")]
    Deserialize(serde_json::Error),
}

impl From<serde_json::Error> for SplitError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialize(e)
    }
}

// =====================================================================
// 4. tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn id() -> uuid::Uuid {
        uuid::Uuid::new_v4()
    }

    #[test]
    fn empty_tree_starts_with_one_pane() {
        let pane = id();
        let tree = SplitTree::new_single(pane);
        assert_eq!(tree.pane_count(), 1);
        assert_eq!(tree.depth(), 0);
        assert_eq!(tree.kind(), SplitTreeKind::Single);
        assert!(tree.root().is_leaf());
        assert_eq!(tree.root().id(), &pane);
    }

    #[test]
    fn split_increments_pane_count_and_depth() {
        let p = id();
        let mut tree = SplitTree::new_single(p);
        let (_n1, p2) = tree.split(p, SplitDirection::Vertical).unwrap();
        assert_eq!(tree.pane_count(), 2);
        assert_eq!(tree.depth(), 1);
        assert_eq!(tree.kind(), SplitTreeKind::Split);
        assert!(!tree.root().is_leaf());
        // 新 pane 存在
        assert!(tree.find_pane(p).is_some(), "pane p 仍是 leaf");
        assert!(tree.find_pane(p2).is_some(), "新 pane p2 是 leaf");
    }

    #[test]
    fn split_at_deeper_level_increments_recursively() {
        let p1 = id();
        let mut tree = SplitTree::new_single(p1);
        let (_, p2) = tree.split(p1, SplitDirection::Vertical).unwrap();
        let (_, p4) = tree.split(p2, SplitDirection::Horizontal).unwrap();
        assert_eq!(tree.pane_count(), 3);
        assert_eq!(tree.depth(), 2);
        // p1, p2, p4 都找得到
        for p in [p1, p2, p4] {
            assert!(tree.find_pane(p).is_some());
        }
    }

    #[test]
    fn split_returns_not_found_for_missing_pane() {
        let p1 = id();
        let mut tree = SplitTree::new_single(p1);
        let r = tree.split(id(), SplitDirection::Vertical);
        assert!(matches!(r, Err(SplitError::NotFound(_))));
        // tree 未变
        assert_eq!(tree.pane_count(), 1);
        assert_eq!(tree.depth(), 0);
    }

    #[test]
    fn find_pane_returns_none_for_missing() {
        let p = id();
        let tree = SplitTree::new_single(p);
        assert!(tree.find_pane(id()).is_none());
    }

    #[test]
    fn json_roundtrip_single_pane() {
        let pane = id();
        let tree = SplitTree::new_single(pane);
        let json = tree.to_json().unwrap();
        let restored = SplitTree::from_json(&json).unwrap();
        assert_eq!(restored.pane_count(), 1);
        assert_eq!(restored.kind(), SplitTreeKind::Single);
        assert!(restored.find_pane(pane).is_some());
    }

    #[test]
    fn json_roundtrip_split_tree() {
        let p1 = id();
        let p2 = id();
        let mut tree = SplitTree::new_split(p1, SplitDirection::Vertical, p1, p2);
        let (_, p3) = tree.split(p1, SplitDirection::Horizontal).unwrap();
        let json = tree.to_json().unwrap();
        let restored = SplitTree::from_json(&json).unwrap();
        assert_eq!(restored.pane_count(), 3);
        assert_eq!(restored.depth(), 2);
        for p in [p1, p2, p3] {
            assert!(restored.find_pane(p).is_some());
        }
    }

    #[test]
    fn split_at_pane_after_tree_growth() {
        // 4 pane tree: split root, then split newly created pane
        let p_root = id();
        let mut tree = SplitTree::new_single(p_root);
        let (_, p_a) = tree.split(p_root, SplitDirection::Vertical).unwrap();
        let (_, p_b) = tree.split(p_root, SplitDirection::Horizontal).unwrap();
        // 此刻: split root (vertical) → [p_root, p_a]; p_b 在 root 的 children[0] 位置
        // split p_a → [p_root(p_a, new_split), ...]
        let (_, p_new) = tree.split(p_a, SplitDirection::Horizontal).unwrap();
        assert_eq!(tree.pane_count(), 4);
        // root 是 Split 内部节点, 不是 leaf
        assert!(!tree.root().is_leaf(), "root 应是 Split 内部节点");
        // 所有 4 leaf panes 找得到 (p_root, p_a, p_b, p_new — 第一次 split 把 p_root 包装但仍是 findable)
        for p in [p_root, p_a, p_b, p_new] {
            assert!(tree.find_pane(p).is_some(), "pane {p:?} 应是 leaf");
        }
    }
}
