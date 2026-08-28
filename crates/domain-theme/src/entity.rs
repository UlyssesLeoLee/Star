//! 主题系统聚合根

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::value_object::{ThemeDefinition, ThemeId, ThemeScope};

/// Theme 聚合根 — 代表"已注册"的主题 (与 ThemeDefinition 区分: Definition 是 schema, Theme 是注册实例)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Theme {
    pub id: Uuid,
    pub theme_id: ThemeId,
    pub display_name: String,
    pub definition: ThemeDefinition,
    pub scope: ThemeScope,
    /// 范围主体 (Personal=actor_id, Tenant=tenant_id, Global=None)
    pub scope_owner: ScopeOwner,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deprecated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScopeOwner {
    Personal { actor_id: Uuid },
    Tenant { tenant_id: Uuid },
    Global,
}

impl Theme {
    pub fn new(
        theme_id: ThemeId,
        display_name: impl Into<String>,
        definition: ThemeDefinition,
        scope: ThemeScope,
        scope_owner: ScopeOwner,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            theme_id,
            display_name: display_name.into(),
            definition,
            scope,
            scope_owner,
            version: 1,
            created_at: now,
            updated_at: now,
            deprecated: false,
        }
    }

    pub fn bump_version(&mut self) {
        self.version += 1;
        self.updated_at = Utc::now();
    }

    pub fn deprecate(&mut self) {
        self.deprecated = true;
        self.updated_at = Utc::now();
    }
}
