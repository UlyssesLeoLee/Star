// SPDX-License-Identifier: MIT OR Apache-2.0
//! CRUD operation layer (per DD-AGENT-RELATIONSHIP-001 §4.2-§4.4).
//!
//! Five submodules:
//! - [`agent_node`] — `AgentNodeOps` (create / get / list / update / archive)
//! - [`edge_ops`] — `EdgeOps` (create / get / list / update / archive / outgoing / incoming / find)
//! - [`template_ops`] — `TemplateOps` (list_templates / instantiate)
//! - [`event_writer`] — `EventWriter` (append-only, Transaction class)
//! - [`achievement_ops`] — `AchievementOps` (unlock + query)
//!
//! Per 守门 #13 b Transaction, the `EventWriter` and
//! `AchievementOps::unlock` paths are append-only.

/// Achievement unlock + query operations.
pub mod achievement_ops;
/// Agent CRUD operations.
pub mod agent_node;
/// Edge CRUD + traversal operations.
pub mod edge_ops;
/// Append-only event writer.
pub mod event_writer;
/// Team template operations.
pub mod template_ops;

pub use achievement_ops::AchievementOps;
pub use agent_node::AgentNodeOps;
pub use edge_ops::EdgeOps;
pub use event_writer::EventWriter;
pub use template_ops::TemplateOps;
