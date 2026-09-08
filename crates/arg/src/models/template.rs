// SPDX-License-Identifier: MIT OR Apache-2.0
//! Team templates (per DD-AGENT-RELATIONSHIP-001 §3.2.3).
//!
//! 5 templates are pre-defined: `HubAndSpoke`, `Mesh`, `Chain`,
//! `Hierarchical`, `ReviewCouncil`. Each one captures the canonical edge
//! structure (1 lead + N workers, fully-connected, linear, 3-layer or
//! 1+3 reviewer) and the constraints (`min_agents..=max_agents`).

use serde::{Deserialize, Serialize};

use super::edge::RelationshipType;

/// Unique template id (5 variants, per DD §3.2.3).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum TemplateId {
    /// 1 Lead + 4 Worker (5 nodes, 4 edges).
    HubAndSpoke,
    /// N nodes fully connected (`C(N,2)` edges).
    Mesh,
    /// Linear chain A → B → C → D (4 nodes, 3 edges).
    Chain,
    /// 3-layer tree (1 Lead + 2 Sub-Lead + 6 Worker, 9 nodes, 8 edges).
    Hierarchical,
    /// 1 Lead + 3 Reviewer (4 nodes, 3 edges).
    ReviewCouncil,
}

impl TemplateId {
    /// Stable slug used in URLs and template-instance names.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HubAndSpoke => "hub-and-spoke",
            Self::Mesh => "mesh",
            Self::Chain => "chain",
            Self::Hierarchical => "hierarchical",
            Self::ReviewCouncil => "review-council",
        }
    }
}

/// Coarse taxonomy (5 categories, per DD §3.2.3).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TemplateCategory {
    /// Default flat team (e.g. `HubAndSpoke`).
    Standard,
    /// High connectivity (`Mesh`).
    HighDensity,
    /// Sequential pipeline (`Chain`).
    Pipeline,
    /// 3-layer management structure (`Hierarchical`).
    Management,
    /// Decision / review group (`ReviewCouncil`).
    Decision,
}

/// Concrete edge inside a template (refs by `agent_ids[index]`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemplateEdge {
    /// Index of the source agent in the instantiate call's `agent_ids`.
    pub from_index: usize,
    /// Index of the target agent in the same list.
    pub to_index: usize,
    /// Edge semantics.
    pub edge_type: RelationshipType,
    /// Default weight applied when this edge is materialised.
    pub default_weight: f32,
}

/// A reusable team template.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TeamTemplate {
    /// Template id.
    pub id: TemplateId,
    /// Human-readable name.
    pub name: String,
    /// Long-form description.
    pub description: String,
    /// Minimum number of agents required to instantiate.
    pub min_agents: usize,
    /// Maximum number of agents allowed.
    pub max_agents: usize,
    /// Template edge structure (relative to `agent_ids`).
    pub edges: Vec<TemplateEdge>,
    /// Coarse category.
    pub category: TemplateCategory,
}

/// 1 Lead + 4 Worker (5 nodes, 4 `DELEGATES_TO` edges).
pub fn hub_and_spoke() -> TeamTemplate {
    TeamTemplate {
        id: TemplateId::HubAndSpoke,
        name: "Hub-and-Spoke".into(),
        description: "1 Lead + 4 Worker 团队".into(),
        min_agents: 5,
        max_agents: 5,
        edges: vec![
            TemplateEdge {
                from_index: 0,
                to_index: 1,
                edge_type: RelationshipType::DelegatesTo,
                default_weight: 0.7,
            },
            TemplateEdge {
                from_index: 0,
                to_index: 2,
                edge_type: RelationshipType::DelegatesTo,
                default_weight: 0.7,
            },
            TemplateEdge {
                from_index: 0,
                to_index: 3,
                edge_type: RelationshipType::DelegatesTo,
                default_weight: 0.7,
            },
            TemplateEdge {
                from_index: 0,
                to_index: 4,
                edge_type: RelationshipType::DelegatesTo,
                default_weight: 0.7,
            },
        ],
        category: TemplateCategory::Standard,
    }
}

/// N-node fully-connected mesh. Edge count is `C(N, 2)` of undirected
/// `COLLABORATES_WITH` edges.
pub fn mesh() -> TeamTemplate {
    TeamTemplate {
        id: TemplateId::Mesh,
        name: "Mesh".into(),
        description: "N 节点全连接 (C(N,2) 边)".into(),
        min_agents: 3,
        max_agents: 10,
        edges: Vec::new(), // populated at instantiate time based on N
        category: TemplateCategory::HighDensity,
    }
}

/// Linear chain A → B → C → D (4 nodes, 3 `DELEGATES_TO` edges).
pub fn chain() -> TeamTemplate {
    TeamTemplate {
        id: TemplateId::Chain,
        name: "Chain".into(),
        description: "A → B → C → D 链式".into(),
        min_agents: 4,
        max_agents: 4,
        edges: vec![
            TemplateEdge {
                from_index: 0,
                to_index: 1,
                edge_type: RelationshipType::DelegatesTo,
                default_weight: 0.6,
            },
            TemplateEdge {
                from_index: 1,
                to_index: 2,
                edge_type: RelationshipType::DelegatesTo,
                default_weight: 0.6,
            },
            TemplateEdge {
                from_index: 2,
                to_index: 3,
                edge_type: RelationshipType::DelegatesTo,
                default_weight: 0.6,
            },
        ],
        category: TemplateCategory::Pipeline,
    }
}

/// 3-layer tree (9 nodes, 8 `DELEGATES_TO` edges).
pub fn hierarchical() -> TeamTemplate {
    let mut edges = Vec::new();
    // 1 Lead → 2 Sub-Lead
    edges.push(TemplateEdge {
        from_index: 0,
        to_index: 1,
        edge_type: RelationshipType::DelegatesTo,
        default_weight: 0.7,
    });
    edges.push(TemplateEdge {
        from_index: 0,
        to_index: 2,
        edge_type: RelationshipType::DelegatesTo,
        default_weight: 0.7,
    });
    // 2 Sub-Lead → 6 Worker (3 each)
    for sub in 1..=2_usize {
        for w in 0..3_usize {
            let worker_index = 3 + (sub - 1) * 3 + w;
            edges.push(TemplateEdge {
                from_index: sub,
                to_index: worker_index,
                edge_type: RelationshipType::DelegatesTo,
                default_weight: 0.6,
            });
        }
    }
    TeamTemplate {
        id: TemplateId::Hierarchical,
        name: "Hierarchical".into(),
        description: "1 Lead + 2 Sub-Lead + 6 Worker (3 层)".into(),
        min_agents: 9,
        max_agents: 9,
        edges,
        category: TemplateCategory::Management,
    }
}

/// 1 Lead + 3 Reviewer (4 nodes, 3 `CONSULTS` edges pointing at the Lead).
pub fn review_council() -> TeamTemplate {
    TeamTemplate {
        id: TemplateId::ReviewCouncil,
        name: "ReviewCouncil".into(),
        description: "1 Lead + 3 Reviewer 评审团".into(),
        min_agents: 4,
        max_agents: 4,
        edges: vec![
            TemplateEdge {
                from_index: 1,
                to_index: 0,
                edge_type: RelationshipType::Consults,
                default_weight: 0.8,
            },
            TemplateEdge {
                from_index: 2,
                to_index: 0,
                edge_type: RelationshipType::Consults,
                default_weight: 0.8,
            },
            TemplateEdge {
                from_index: 3,
                to_index: 0,
                edge_type: RelationshipType::Consults,
                default_weight: 0.8,
            },
        ],
        category: TemplateCategory::Decision,
    }
}

/// Returns the 5 canonical templates in the order
/// `HubAndSpoke / Mesh / Chain / Hierarchical / ReviewCouncil`.
pub fn all_templates() -> Vec<TeamTemplate> {
    vec![
        hub_and_spoke(),
        mesh(),
        chain(),
        hierarchical(),
        review_council(),
    ]
}
