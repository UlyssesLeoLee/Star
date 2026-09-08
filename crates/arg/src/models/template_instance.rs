// SPDX-License-Identifier: MIT OR Apache-2.0
//! `TemplateInstance` (per DD-AGENT-RELATIONSHIP-001 §3.2.5).
//!
//! A `Work`-class row per 守门 #13 a: physical deletion is allowed, the
//! row has a `30-day TTL` after which it is treated as expired by
//! [`crate::ops::template_ops::TemplateOps`].

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::template::TemplateEdge;
use super::template::TemplateId;

/// A materialised team template.
///
/// `expires_at = created_at + 30 days` per 守门 #13 a Work. Callers should
/// not rely on `expires_at` for security — it's a soft TTL.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemplateInstance {
    /// Instance id.
    pub id: Uuid,
    /// Which template was instantiated.
    pub template_id: TemplateId,
    /// Human-readable name (chosen by the user at instantiate time).
    pub instance_name: String,
    /// Concrete agent ids in instantiate order (matches `TemplateEdge.from_index`).
    pub agent_ids: Vec<Uuid>,
    /// Serialised edge list (Work class, can be GC'd after the TTL).
    pub edges_json: serde_json::Value,
    /// Tenant id (RLS 13 類).
    pub tenant_id: Uuid,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Soft TTL (now + 30 days, per 守门 #13 a Work).
    pub expires_at: DateTime<Utc>,
    /// Author of the instance (代签 Ulysses, 守门 #10).
    pub created_by: Uuid,
}

impl TemplateInstance {
    /// Build a new instance with a fresh id, now() timestamps, and a
    /// 30-day TTL.
    pub fn new(
        template_id: TemplateId,
        instance_name: String,
        agent_ids: Vec<Uuid>,
        edges: Vec<TemplateEdge>,
        tenant_id: Uuid,
        created_by: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            template_id,
            instance_name,
            agent_ids,
            edges_json: serde_json::to_value(&edges).unwrap_or(serde_json::Value::Null),
            tenant_id,
            created_at: now,
            expires_at: now + Duration::days(30),
            created_by,
        }
    }

    /// `true` iff `Utc::now() > expires_at`.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}
