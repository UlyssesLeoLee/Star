//! ═══════════════════════════════════════════════════════════════════
//! Ported from UlyssesLeoLee/SandTree v1.1 (Apache-2.0)
//! per REUSE-MAP-COLLAB-SANDBOX-001 §3 (直接复用, 保留 NOTICE).
//! Source: E:\SandTree\crates\{model,observation-model,resource-graph}
//! ═══════════════════════════════════════════════════════════════════

//! sandtree-core — SandTree 沙盒资源拓扑内核 (Star)
//!
//! 从 UlyssesLeoLee/SandTree (Apache-2.0) 移植:
//! - model: 资源/操作/事件/能力/ID 基础模型 (sandtree-model)
//! - graph: 统一资源拓扑 (in-memory tree + 关系索引 + reconcile, FR-002~004)
//! - observation: 观测四级 (原生→执行→探针→元数据, 来源/可信度/新鲜度)
#![allow(missing_docs, unreachable_pub)]

pub mod graph;
pub mod model;
pub mod observation;

pub use graph::{Change, ChangeKind, Metadata, ResourceFilter, ResourceGraph, TreeNode};
pub use model::id::{PluginId, ResourceId};
pub use model::resource::{Relation, ResourceKind, ResourceNode, ResourceState};
