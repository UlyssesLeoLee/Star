//! Star Workflow — 可视化导出 (wt-w7-viz 扩展)
//!
//! 把工作流状态机导出为 SVG / DOT / Mermaid 格式, 给前端 w4 编辑器对接.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =====================================================================
// SVG 导出
// =====================================================================

/// 工作流节点 (状态)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VizNode {
    pub id: String,
    pub label: String,
    pub category: NodeCategory, // TODO/DOING/DONE
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeCategory {
    Todo,
    Doing,
    Done,
}

/// 工作流边 (转换)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VizEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub label: String, // 转换名
    pub condition: Option<String>,
}

/// 工作流可视化
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowViz {
    pub nodes: Vec<VizNode>,
    pub edges: Vec<VizEdge>,
}

impl WorkflowViz {
    /// 导出 SVG (固定 800x600, 简单布局)
    pub fn to_svg(&self) -> String {
        let mut svg = String::from(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="800" height="600" viewBox="0 0 800 600">"##,
        );
        svg.push_str(r##"<defs><marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M 0 0 L 10 5 L 0 10 z" fill="#475569"/></marker></defs>"##);
        svg.push_str(r##"<rect width="100%" height="100%" fill="#F8FAFC"/>"##);

        // 边
        for edge in &self.edges {
            if let (Some(from), Some(to)) = (
                self.nodes.iter().find(|n| n.id == edge.from),
                self.nodes.iter().find(|n| n.id == edge.to),
            ) {
                let color = match edge.condition.as_deref() {
                    Some(_) => "#C77B30", // 条件转换: 橙
                    None => "#5B5BD6",    // 普通: 主色
                };
                svg.push_str(&format!(
                    r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="2" marker-end="url(#arrow)"/>"##,
                    from.x + 60.0, from.y + 30.0, to.x + 60.0, to.y + 30.0, color
                ));
                if !edge.label.is_empty() {
                    let mx = (from.x + to.x) / 2.0 + 60.0;
                    let my = (from.y + to.y) / 2.0 + 30.0;
                    svg.push_str(&format!(
                        r##"<text x="{}" y="{}" font-size="11" fill="#475569" text-anchor="middle">{}</text>"##,
                        mx, my, edge.label
                    ));
                }
            }
        }

        // 节点
        for node in &self.nodes {
            let fill = match node.category {
                NodeCategory::Todo => "#EEF2F7",
                NodeCategory::Doing => "#5B5BD6",
                NodeCategory::Done => "#3D8B5F",
            };
            let stroke = match node.category {
                NodeCategory::Todo => "#CBD5E1",
                NodeCategory::Doing => "#5B5BD6",
                NodeCategory::Done => "#3D8B5F",
            };
            let text_color = if matches!(node.category, NodeCategory::Doing | NodeCategory::Done) {
                "#FFFFFF"
            } else {
                "#0F172A"
            };
            svg.push_str(&format!(
                r##"<rect x="{}" y="{}" width="120" height="60" rx="8" fill="{}" stroke="{}" stroke-width="2"/>"##,
                node.x, node.y, fill, stroke
            ));
            svg.push_str(&format!(
                r##"<text x="{}" y="{}" font-size="14" font-weight="600" fill="{}" text-anchor="middle" dominant-baseline="central">{}</text>"##,
                node.x + 60.0, node.y + 30.0, text_color, node.label
            ));
        }
        svg.push_str("</svg>");
        svg
    }

    /// 导出 Mermaid
    pub fn to_mermaid(&self) -> String {
        let mut m = String::from("stateDiagram-v2\n");
        for node in &self.nodes {
            m.push_str(&format!("    {}: {}\n", node.id, node.label));
        }
        for edge in &self.edges {
            m.push_str(&format!(
                "    {} --> {}: {}\n",
                edge.from, edge.to, edge.label
            ));
        }
        m
    }

    /// 导出 DOT (Graphviz)
    pub fn to_dot(&self) -> String {
        let mut d = String::from("digraph workflow {\n");
        d.push_str("  rankdir=LR;\n");
        for node in &self.nodes {
            let shape = match node.category {
                NodeCategory::Todo => "box",
                NodeCategory::Doing => "box,style=filled,fillcolor=blue",
                NodeCategory::Done => "box,style=filled,fillcolor=green",
            };
            d.push_str(&format!(
                "  {} [label=\"{}\",{}];\n",
                node.id, node.label, shape
            ));
        }
        for edge in &self.edges {
            d.push_str(&format!(
                "  {} -> {} [label=\"{}\"];\n",
                edge.from, edge.to, edge.label
            ));
        }
        d.push_str("}\n");
        d
    }
}

/// 简单布局算法 (从左到右 3 列, 每列最多 5 个节点)
pub fn auto_layout(nodes: Vec<VizNode>) -> Vec<VizNode> {
    let col_width = 220.0;
    let row_height = 100.0;
    let start_x = 40.0;
    let start_y = 40.0;

    let mut sorted = nodes;
    sorted.sort_by_key(|n| match n.category {
        NodeCategory::Todo => 0,
        NodeCategory::Doing => 1,
        NodeCategory::Done => 2,
    });

    let mut by_col: std::collections::HashMap<u8, Vec<VizNode>> = std::collections::HashMap::new();
    for n in sorted {
        let col = match n.category {
            NodeCategory::Todo => 0u8,
            NodeCategory::Doing => 1u8,
            NodeCategory::Done => 2u8,
        };
        by_col.entry(col).or_insert_with(Vec::new).push(n);
    }

    let mut result = Vec::new();
    for (col, mut col_nodes) in by_col {
        for (i, mut n) in col_nodes.drain(..).enumerate() {
            n.x = start_x + col as f32 * col_width;
            n.y = start_y + i as f32 * row_height;
            result.push(n);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_viz() -> WorkflowViz {
        WorkflowViz {
            nodes: vec![
                VizNode {
                    id: "todo".into(),
                    label: "To Do".into(),
                    category: NodeCategory::Todo,
                    x: 0.0,
                    y: 0.0,
                },
                VizNode {
                    id: "doing".into(),
                    label: "In Progress".into(),
                    category: NodeCategory::Doing,
                    x: 0.0,
                    y: 0.0,
                },
                VizNode {
                    id: "done".into(),
                    label: "Done".into(),
                    category: NodeCategory::Done,
                    x: 0.0,
                    y: 0.0,
                },
            ],
            edges: vec![
                VizEdge {
                    id: "e1".into(),
                    from: "todo".into(),
                    to: "doing".into(),
                    label: "Start".into(),
                    condition: None,
                },
                VizEdge {
                    id: "e2".into(),
                    from: "doing".into(),
                    to: "done".into(),
                    label: "Complete".into(),
                    condition: Some("all subtasks done".into()),
                },
            ],
        }
    }

    #[test]
    fn test_to_svg() {
        let svg = sample_viz().to_svg();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("To Do"));
        assert!(svg.contains("In Progress"));
        assert!(svg.contains("Done"));
    }

    #[test]
    fn test_to_mermaid() {
        let m = sample_viz().to_mermaid();
        assert!(m.contains("stateDiagram-v2"));
        assert!(m.contains("todo --> doing"));
    }

    #[test]
    fn test_to_dot() {
        let d = sample_viz().to_dot();
        assert!(d.contains("digraph workflow"));
        assert!(d.contains("rankdir=LR"));
    }

    #[test]
    fn test_auto_layout() {
        let nodes = vec![
            VizNode {
                id: "a".into(),
                label: "A".into(),
                category: NodeCategory::Todo,
                x: 0.0,
                y: 0.0,
            },
            VizNode {
                id: "b".into(),
                label: "B".into(),
                category: NodeCategory::Doing,
                x: 0.0,
                y: 0.0,
            },
            VizNode {
                id: "c".into(),
                label: "C".into(),
                category: NodeCategory::Done,
                x: 0.0,
                y: 0.0,
            },
        ];
        let laid = auto_layout(nodes);
        assert_eq!(laid.len(), 3);
        // 三个不同 col
        assert_ne!(laid[0].x, laid[1].x);
        assert_ne!(laid[1].x, laid[2].x);
    }

    #[test]
    fn test_svg_includes_arrow_marker() {
        let svg = sample_viz().to_svg();
        assert!(svg.contains("marker"));
    }
}
