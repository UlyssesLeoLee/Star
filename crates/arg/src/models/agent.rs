// SPDX-License-Identifier: MIT OR Apache-2.0
//! Agent node (per DD-AGENT-RELATIONSHIP-001 §3.2.1 + §3.2.5).
//!
//! An [`Agent`] is a single node in the Agent Relationship Graph. It carries
//! 16 fields and is identified by a [`Uuid`]. The `archetype` field uses an
//! [`AgentArchetype`] enum covering 9 SA types + 5 domain leads + a custom
//! catch-all. [`Domain`] is derived from the lead archetype and [`AgentStatus`]
//! implements a 3-state machine (Active ↔ Standby → Archived).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ARGError;

/// 9 SA Archetype + 5 domain Lead + Custom = 15 variants (per DD §3.2.1).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgentArchetype {
    /// Sub-agent archetype #01.
    Sa01,
    /// Sub-agent archetype #02.
    Sa02,
    /// Sub-agent archetype #03.
    Sa03,
    /// Sub-agent archetype #04.
    Sa04,
    /// Sub-agent archetype #05.
    Sa05,
    /// Sub-agent archetype #06.
    Sa06,
    /// Sub-agent archetype #07.
    Sa07,
    /// Sub-agent archetype #08.
    Sa08,
    /// Sub-agent archetype #09.
    Sa09,
    /// Lead for the Player domain.
    LeadPlayer,
    /// Lead for the Economy domain.
    LeadEconomy,
    /// Lead for the Match domain.
    LeadMatch,
    /// Lead for the Social domain.
    LeadSocial,
    /// Lead for the Admin domain.
    LeadAdmin,
    /// User-defined archetype.
    Custom,
}

impl AgentArchetype {
    /// Stable string label used in Cypher / logs / API responses.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sa01 => "SA_01",
            Self::Sa02 => "SA_02",
            Self::Sa03 => "SA_03",
            Self::Sa04 => "SA_04",
            Self::Sa05 => "SA_05",
            Self::Sa06 => "SA_06",
            Self::Sa07 => "SA_07",
            Self::Sa08 => "SA_08",
            Self::Sa09 => "SA_09",
            Self::LeadPlayer => "LEAD_PLAYER",
            Self::LeadEconomy => "LEAD_ECONOMY",
            Self::LeadMatch => "LEAD_MATCH",
            Self::LeadSocial => "LEAD_SOCIAL",
            Self::LeadAdmin => "LEAD_ADMIN",
            Self::Custom => "CUSTOM",
        }
    }
}

/// 5 業務 sub-domains (per DD §3.2.1 — the canonical `player / economy /
/// match / social / admin` taxonomy from `AGENTS.md` §5 仓库拓扑).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Domain {
    /// Player-facing flows.
    Player,
    /// In-game economy.
    Economy,
    /// Match / session lifecycle.
    Match,
    /// Social graph and relationships.
    Social,
    /// Admin and moderation.
    Admin,
}

/// 3-state machine: Active ↔ Standby → Archived (per DD §3.3.2).
///
/// `Archived` is the absorbing state and cannot transition out.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    /// Operational and accepting tasks.
    Active,
    /// Paused, will not be picked up by dispatch.
    Standby,
    /// Terminal state, retained for audit (SCD Type 2, per 守门 #13 c).
    Archived,
}

/// Agent node (16 fields, per DD §3.2.1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Agent {
    /// Globally unique id (Uuid v4).
    pub id: Uuid,
    /// Display name (max 255 chars, validated by [`Self::validate`]).
    pub name: String,
    /// Archetype (9 SA + 5 Lead + Custom).
    pub archetype: AgentArchetype,
    /// Business sub-domain. Derived from `archetype` for `Lead*` variants,
    /// `None` otherwise.
    pub domain: Option<Domain>,
    /// Lifecycle state (3-state machine).
    pub status: AgentStatus,
    /// Trust score in `[0.0, 1.0]`. Bucketed by [`crate::models::trust_score::TrustScoreTier`].
    pub trust_score: f32,
    /// Free-form extension field.
    pub metadata: serde_json::Value,
    /// Tenant id used by RLS 13 類 (per 守门 #13).
    pub tenant_id: Uuid,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
    /// SCD Type 2 version counter (per 守门 #13 c Master).
    pub version: i32,
    /// Author of this row. Filled with `created_by` per 守门 #10 (代签 Ulysses).
    pub created_by: Uuid,
}

impl Agent {
    /// Construct a new agent with sane defaults (trust 0.5, status `Active`,
    /// domain derived from archetype, version 1, now() timestamps).
    pub fn new(name: String, archetype: AgentArchetype, tenant_id: Uuid, created_by: Uuid) -> Self {
        let domain = match archetype {
            AgentArchetype::LeadPlayer => Some(Domain::Player),
            AgentArchetype::LeadEconomy => Some(Domain::Economy),
            AgentArchetype::LeadMatch => Some(Domain::Match),
            AgentArchetype::LeadSocial => Some(Domain::Social),
            AgentArchetype::LeadAdmin => Some(Domain::Admin),
            _ => None,
        };
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            archetype,
            domain,
            status: AgentStatus::Active,
            trust_score: 0.5,
            metadata: serde_json::Value::Null,
            tenant_id,
            created_at: now,
            updated_at: now,
            version: 1,
            created_by,
        }
    }

    /// Run the field-level validation. Returns the first failure as
    /// [`ARGError::ValidationFailed`] or [`ARGError::TrustScoreOutOfRange`].
    pub fn validate(&self) -> Result<(), ARGError> {
        if self.name.is_empty() {
            return Err(ARGError::ValidationFailed("name is empty".into()));
        }
        if self.name.len() > 255 {
            return Err(ARGError::ValidationFailed("name > 255 chars".into()));
        }
        if !(0.0..=1.0).contains(&self.trust_score) {
            return Err(ARGError::TrustScoreOutOfRange(self.trust_score));
        }
        Ok(())
    }

    /// Apply a small `+0.01` / `-0.05` trust score nudge, clamped to
    /// `[0.0, 1.0]`, and bump `updated_at` + `version`. Returns the new score.
    pub fn update_trust_score(&mut self, success: bool) -> Result<f32, ARGError> {
        let new_score = if success {
            (self.trust_score + 0.01).min(1.0)
        } else {
            (self.trust_score - 0.05).max(0.0)
        };
        self.trust_score = new_score;
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(new_score)
    }

    /// State machine guard (per DD §3.3.2).
    ///
    /// `Archived` is absorbing — it has no outgoing transitions.
    pub fn can_transition_to(&self, target: AgentStatus) -> bool {
        use AgentStatus::*;
        matches!(
            (self.status, target),
            (Active, Standby) | (Active, Archived) | (Standby, Active) | (Standby, Archived)
        )
    }

    /// Cypher `MERGE` expression for the agent node. Used by the
    /// [`crate::ops::agent_node::AgentNodeOps`] write path.
    pub fn to_cypher(&self) -> String {
        let domain = self
            .domain
            .map(|d| format!("{:?}", d).to_lowercase())
            .unwrap_or_default();
        let status = format!("{:?}", self.status).to_lowercase();
        format!(
            "MERGE (a:Agent {{id: '{}'}}) \
             SET a.name = '{}', a.archetype = '{}', a.domain = '{}', \
                 a.status = '{}', a.trust_score = {}, a.tenant_id = '{}', \
                 a.version = {}, a.created_by = '{}', \
                 a.created_at = localdatetime(), a.updated_at = localdatetime()",
            self.id,
            self.name,
            self.archetype.as_str(),
            domain,
            status,
            self.trust_score,
            self.tenant_id,
            self.version,
            self.created_by,
        )
    }
}
