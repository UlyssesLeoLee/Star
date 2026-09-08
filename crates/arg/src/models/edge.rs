// SPDX-License-Identifier: MIT OR Apache-2.0
//! Edge between two agents (per DD-AGENT-RELATIONSHIP-001 §3.2.2 + §3.3.1).
//!
//! [`Edge`] has 14 fields. The 11 mandatory fields are required on every
//! create; the 3 derived fields (`direction`, `archived`, `version`) are
//! either inferred from the [`RelationshipType`] or maintained by the
//! [`crate::ops::edge_ops::EdgeOps`] SCD Type 2 layer.
//!
//! The 4 core + 6 extension relationship types = 10 variants in
//! [`RelationshipType`]. Two of them (`CollaboratesWith`, `PeerReviews`)
//! are undirected; the rest are directed.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ARGError;

/// 10 relationship types (4 核心 + 6 扩展, per DD §3.2.2).
///
/// `CollaboratesWith` and `PeerReviews` are undirected; all others are
/// directed. Use [`Self::is_directed`] or [`Self::cypher_label`] to query
/// the semantics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelationshipType {
    // ===== 4 核心 =====
    /// Directed: A delegates a task to B.
    DelegatesTo,
    /// Directed: A consults B for an opinion.
    Consults,
    /// Undirected: A and B collaborate in parallel.
    CollaboratesWith,
    /// Directed: A reports to B.
    ReportsTo,
    // ===== 6 扩展 =====
    /// Directed: A mentors B (B receives historical context, per DD §4.3.2).
    Mentors,
    /// Undirected: A and B review each other's outputs (per DD §4.3.4).
    PeerReviews,
    /// Directed: A stands in for B when B is unavailable.
    StandInFor,
    /// Directed: A shadows B and observes B's events.
    Shadows,
    /// Directed: A challenges B's decision (per DD §4.3.4 challenges).
    Challenges,
    /// Directed: A trusts B (used by TrustEngine skip-verify, per DD §4.3.3).
    Trusts,
}

impl RelationshipType {
    /// Returns `false` for the two undirected types, `true` otherwise.
    pub fn is_directed(&self) -> bool {
        !matches!(self, Self::CollaboratesWith | Self::PeerReviews)
    }

    /// Cypher edge label (uppercase, snake-screaming).
    pub fn cypher_label(&self) -> &'static str {
        match self {
            Self::DelegatesTo => "DELEGATES_TO",
            Self::Consults => "CONSULTS",
            Self::CollaboratesWith => "COLLABORATES_WITH",
            Self::ReportsTo => "REPORTS_TO",
            Self::Mentors => "MENTORS",
            Self::PeerReviews => "PEER_REVIEWS",
            Self::StandInFor => "STAND_IN_FOR",
            Self::Shadows => "SHADOWS",
            Self::Challenges => "CHALLENGES",
            Self::Trusts => "TRUSTS",
        }
    }

    /// Returns all 10 variants in the canonical order. Used by enumerations
    /// and by the 8 拓扑成就 Cypher queries.
    pub fn all() -> [RelationshipType; 10] {
        [
            Self::DelegatesTo,
            Self::Consults,
            Self::CollaboratesWith,
            Self::ReportsTo,
            Self::Mentors,
            Self::PeerReviews,
            Self::StandInFor,
            Self::Shadows,
            Self::Challenges,
            Self::Trusts,
        ]
    }
}

/// Direction of the edge. Only meaningful when the
/// [`RelationshipType`] is undirected — for directed types this is always
/// [`Self::Directed`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EdgeDirection {
    /// Standard arrow `A → B`.
    Directed,
    /// Undirected (e.g. `COLLABORATES_WITH`).
    Undirected,
}

/// Edge (14 fields, per DD §3.2.2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Edge {
    /// Globally unique id.
    pub id: Uuid,
    /// Source agent id.
    pub from_agent: Uuid,
    /// Target agent id.
    pub to_agent: Uuid,
    /// Relationship semantics.
    pub edge_type: RelationshipType,
    /// Relationship weight in `[0.0, 1.0]` (validated by [`Self::validate`]).
    pub weight: f32,
    /// Direction — must agree with `edge_type.is_directed()`.
    pub direction: EdgeDirection,
    /// Soft-delete flag (SCD Type 2 friendly, per 守门 #13 c).
    pub archived: bool,
    /// Free-form extension field.
    pub metadata: serde_json::Value,
    /// Tenant id (RLS 13 類).
    pub tenant_id: Uuid,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
    /// SCD Type 2 version counter.
    pub version: i32,
    /// Author of this row (代签 Ulysses, 守门 #10).
    pub created_by: Uuid,
}

impl Edge {
    /// Field-level validation. Self-loops, OOB weights and direction/type
    /// mismatches are rejected with [`ARGError::ValidationFailed`].
    pub fn validate(&self) -> Result<(), ARGError> {
        if self.from_agent == self.to_agent {
            return Err(ARGError::ValidationFailed("self-loop not allowed".into()));
        }
        if !(0.0..=1.0).contains(&self.weight) {
            return Err(ARGError::ValidationFailed("weight out of [0,1]".into()));
        }
        if !self.edge_type.is_directed() && self.direction != EdgeDirection::Undirected {
            return Err(ARGError::ValidationFailed(
                "undirected type but direction is directed".into(),
            ));
        }
        Ok(())
    }

    /// 3-state machine for edges (per DD §3.3.1).
    /// `Archived` is the absorbing state.
    pub fn can_transition(from: EdgeState, to: EdgeState) -> bool {
        use EdgeState::*;
        matches!(
            (from, to),
            (Created, Updated) | (Created, Archived) | (Updated, Updated) | (Updated, Archived)
        )
    }
}

/// 3-state machine for edges (Created / Updated / Archived).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeState {
    /// Newly created edge, version 1.
    Created,
    /// Edge that has been updated at least once (`version > 1`).
    Updated,
    /// Soft-deleted edge (`archived = true`).
    Archived,
}
