// SPDX-License-Identifier: MIT OR Apache-2.0
//! `ARGEvent` enum (per DD-AGENT-RELATIONSHIP-001 §3.2.5).
//!
//! 9 variants that flow through the in-process `EventBus` and the
//! `MemgraphEventListener` (in `crates/arg-bridge`). 7 of them carry the
//! full row payload (Agent / Edge / TemplateInstance / AchievementUnlock);
//! the 2 trust-score events carry before/after deltas.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::achievement_unlock::AchievementUnlock;
use super::agent::Agent;
use super::edge::Edge;
use super::template_instance::TemplateInstance;

/// 9-variant event enum (per DD §3.2.5).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ARGEvent {
    /// New agent node created.
    AgentCreated(Agent),
    /// Existing agent node updated (SCD Type 2).
    AgentUpdated {
        /// Agent id.
        id: Uuid,
        /// Snapshot before the update.
        before: Agent,
        /// Snapshot after the update.
        after: Agent,
    },
    /// Agent soft-deleted (status → Archived).
    AgentArchived {
        /// Archived agent id.
        id: Uuid,
    },
    /// New edge created.
    EdgeCreated(Edge),
    /// Existing edge updated.
    EdgeUpdated {
        /// Edge id.
        id: Uuid,
        /// Snapshot before the update.
        before: Edge,
        /// Snapshot after the update.
        after: Edge,
    },
    /// Edge soft-deleted.
    EdgeArchived {
        /// Archived edge id.
        id: Uuid,
    },
    /// A team template was instantiated.
    TemplateInstantiated(TemplateInstance),
    /// Trust score for a given agent changed.
    TrustScoreChanged {
        /// Agent id.
        agent_id: Uuid,
        /// Score before the change.
        before: f32,
        /// Score after the change.
        after: f32,
        /// Signed delta applied (`+0.01` / `-0.05`).
        delta: f32,
    },
    /// An achievement was unlocked.
    AchievementUnlocked(AchievementUnlock),
}

impl ARGEvent {
    /// Stable string code for log / metric labels.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::AgentCreated(_) => "agent.created",
            Self::AgentUpdated { .. } => "agent.updated",
            Self::AgentArchived { .. } => "agent.archived",
            Self::EdgeCreated(_) => "edge.created",
            Self::EdgeUpdated { .. } => "edge.updated",
            Self::EdgeArchived { .. } => "edge.archived",
            Self::TemplateInstantiated(_) => "template.instantiated",
            Self::TrustScoreChanged { .. } => "trust_score.changed",
            Self::AchievementUnlocked(_) => "achievement.unlocked",
        }
    }
}
