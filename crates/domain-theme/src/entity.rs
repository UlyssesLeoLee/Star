//! 主题系统聚合根

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::value_object::{ThemeDefinition, ThemeId, ThemeScope};

/// Theme 聚合根 — 代表"已注册"的主题 (与 ThemeDefinition 区分: Definition 是 schema, Theme 是注册实例)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Theme {
    /// 主键 ID
    pub id: Uuid,
    /// 主题定义 ID
    pub theme_id: ThemeId,
    /// 显示名称
    pub display_name: String,
    /// 主题定义 schema
    pub definition: ThemeDefinition,
    /// 所属作用域
    pub scope: ThemeScope,
    /// 范围主体 (Personal=actor_id, Tenant=tenant_id, Global=None)
    pub scope_owner: ScopeOwner,
    /// 版本号(每次变更递增)
    pub version: u32,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最近更新时间
    pub updated_at: DateTime<Utc>,
    /// 是否已废弃
    pub deprecated: bool,
}

/// 作用域主体标识
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScopeOwner {
    /// 个人作用域
    Personal {
        /// 所属用户 ID
        actor_id: Uuid,
    },
    /// 租户作用域
    Tenant {
        /// 所属租户 ID
        tenant_id: Uuid,
    },
    /// 全局作用域
    Global,
}

impl Theme {
    /// 构造新主题实例
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

    /// 递增版本号并刷新更新时间
    pub fn bump_version(&mut self) {
        self.version += 1;
        self.updated_at = Utc::now();
    }

    /// 标记为废弃
    pub fn deprecate(&mut self) {
        self.deprecated = true;
        self.updated_at = Utc::now();
    }
}
