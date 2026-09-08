// SPDX-License-Identifier: MIT OR Apache-2.0
//! `AchievementUnlock` (per DD-AGENT-RELATIONSHIP-001 §3.2.5).
//!
//! A `Transaction`-class row per 守门 #13 b: append-only, audited. The
//! `idempotent` guarantee lives in
//! [`crate::ops::achievement_ops::AchievementOps::unlock`] which checks
//! `(user_id, achievement_code)` before inserting.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single `user × achievement` unlock event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AchievementUnlock {
    /// Unlock row id.
    pub id: Uuid,
    /// Achievement code, e.g. `"TOP-001-MESH-5DOMAIN"`.
    pub achievement_code: String,
    /// User who earned the achievement.
    pub user_id: Uuid,
    /// Agents that participated in the unlock.
    pub agent_ids: Vec<Uuid>,
    /// Free-form trigger metadata (which Cypher / event / metric fired).
    pub trigger_metadata: serde_json::Value,
    /// Tenant id (RLS 13 類).
    pub tenant_id: Uuid,
    /// Unlock timestamp.
    pub unlocked_at: DateTime<Utc>,
}

impl AchievementUnlock {
    /// Build a new unlock row with a fresh id and now() timestamp.
    pub fn new(
        achievement_code: String,
        user_id: Uuid,
        agent_ids: Vec<Uuid>,
        trigger_metadata: serde_json::Value,
        tenant_id: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            achievement_code,
            user_id,
            agent_ids,
            trigger_metadata,
            tenant_id,
            unlocked_at: Utc::now(),
        }
    }
}
