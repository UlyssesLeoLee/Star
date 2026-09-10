// SPDX-License-Identifier: MIT OR Apache-2.0
//! Domain models for the ARG Data Tier.
//!
//! Each submodule corresponds to one of the type families in
//! [`docs/design/DD-AGENT-RELATIONSHIP-001.md`](../../docs/design/DD-AGENT-RELATIONSHIP-001.md)
//! §3.2 / §3.3 / §3.2.5. They are re-exported here so consumers can `use
//! star_arg::models::{Agent, Edge, ...}`.

/// Achievement (20 definitions across 3 categories), per DD §3.2.4.
pub mod achievement;
/// `AchievementUnlock` (Transaction), per DD §3.2.5.
pub mod achievement_unlock;
/// Agent node (16 fields + 3 enums), per DD §3.2.1.
pub mod agent;
/// `Decision` / `DecisionType` / `Output` / `Verdict` / `EscalationInfo`, per DD §3.2.5.
pub mod decision;
/// Edge between two agents (14 fields + 3 enums), per DD §3.2.2.
pub mod edge;
/// `ARGEvent` enum (9 variants), per DD §3.2.5.
pub mod event;
/// `PeerReviewVerdict` / `ChallengeVerdict` / `ChallengePrompt`, per DD §3.2.5.
pub mod peer_review;
/// Team template (5 templates + enums), per DD §3.2.3.
pub mod template;
/// `TemplateInstance` (TTL 30 d, per 守门 #13 a W 类), per DD §3.2.5.
pub mod template_instance;
/// Trust score 5 档 enum + transfer function, per DD §3.3.3.
pub mod trust_score;
/// `TrustAuditLog` (WORM append-only per 守门 #13 d + ADR-0043), per DDD-REVIEW-AGENT-RELATIONSHIP-001 §1.2 G-4 拍板.
pub mod trust_audit;

pub use achievement::{Achievement, AchievementCategory, Rarity, UnlockCondition};
pub use achievement_unlock::AchievementUnlock;
pub use agent::{Agent, AgentArchetype, AgentStatus, Domain};
pub use decision::{Decision, DecisionType, EscalationInfo, Output, Verdict};
pub use edge::{Edge, EdgeDirection, RelationshipType};
pub use event::ARGEvent;
pub use peer_review::{ChallengePrompt, ChallengeVerdict, PeerReviewVerdict};
pub use template::{TeamTemplate, TemplateCategory, TemplateEdge, TemplateId};
pub use template_instance::TemplateInstance;
pub use trust_score::{update_trust_score, TrustScoreTier};
pub use trust_audit::TrustAuditLog;
