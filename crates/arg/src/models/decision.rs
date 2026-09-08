// SPDX-License-Identifier: MIT OR Apache-2.0
//! Decision / Output / Verdict / EscalationInfo (per DD §3.2.5).
//!
//! These types are passed between the Effect Tier (`arg-effect`) and the
//! LLM clients when running challenges / peer reviews.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 5 decision types used to pick the right challenges prompt (per DD §7).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum DecisionType {
    /// Architectural choice (e.g. crate layout, lock order).
    Architectural,
    /// Business decision (e.g. pricing, gamification rule).
    Business,
    /// Security decision (e.g. RLS policy, KMS rotation).
    Security,
    /// Performance decision (e.g. cache eviction, index).
    Performance,
    /// User-experience decision (e.g. default tab, shortcut).
    Ux,
}

impl DecisionType {
    /// All 5 variants in canonical order.
    pub fn all() -> [DecisionType; 5] {
        [
            Self::Architectural,
            Self::Business,
            Self::Security,
            Self::Performance,
            Self::Ux,
        ]
    }
}

/// A specific decision under review.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Decision {
    /// What kind of decision.
    pub decision_type: DecisionType,
    /// Human-readable description.
    pub description: String,
    /// Free-form context payload (JSON).
    pub context: serde_json::Value,
    /// Tenant id (RLS 13 類).
    pub tenant_id: Uuid,
}

/// Output of a peer_review or challenge round.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Output {
    /// Type label, e.g. `"code.review"`, `"design.proposal"`.
    pub output_type: String,
    /// Free-form content payload.
    pub content: serde_json::Value,
    /// Authoring agent id.
    pub agent_id: Uuid,
    /// Tenant id (RLS 13 類).
    pub tenant_id: Uuid,
}

/// 3-value verdict returned by an LLM challenge round (per DD §4.3.4).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    /// The challenged agent's justification was accepted.
    Accept,
    /// The justification was rejected; the upstream decision is blocked.
    Reject,
    /// The round could not be resolved locally; escalate to a human Lead.
    Escalate,
}

/// Escalation metadata (per DD §3.2.5).
///
/// Used by `ARGOutputEvaluator` when [`Verdict::Escalate`] is returned.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EscalationInfo {
    /// Why escalation is required.
    pub reason: String,
    /// Who the decision is escalated to. Usually a 5 域 Lead 真人.
    pub escalation_target: Uuid,
    /// Deadline for the escalated decision.
    pub deadline: DateTime<Utc>,
}
