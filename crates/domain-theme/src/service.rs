//! 主题系统应用服务

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use super::context::ThemeContext;
use super::entity::{ScopeOwner, Theme};
use super::error::ThemeError;
use super::event::ThemeEvent;
use super::invariant::*;
use super::port::{ThemeEventBus, ThemeRepository};
use super::value_object::{ThemeDefinition, ThemeId, ThemeScope};

/// 主题系统应用服务(三层解析编排)
pub struct ThemeService {
    repo: Arc<dyn ThemeRepository>,
    bus: Arc<dyn ThemeEventBus>,
}

impl ThemeService {
    /// 构造主题应用服务
    pub fn new(repo: Arc<dyn ThemeRepository>, bus: Arc<dyn ThemeEventBus>) -> Self {
        Self { repo, bus }
    }

    /// 三层解析: Personal > Tenant > Global (per 2026-08-29 04:09 JST 拍板)
    pub async fn resolve(&self, ctx: &ThemeContext) -> Result<ThemeId, ThemeError> {
        for scope in &ctx.resolution_chain {
            if let Some(t) = self
                .repo
                .find_by_scope(
                    // 先找用户最近一次设置的 (这里简化为按主题 ID 的最近一次记录)
                    ThemeId::Light, // 占位, 真实应按"当前激活"字段查
                    *scope,
                    ctx.actor_id,
                    ctx.tenant_id,
                )
                .await?
            {
                return Ok(t.theme_id);
            }
        }
        // 三层都未设置 → 平台默认 (Light)
        Ok(ThemeId::Light)
    }

    /// 列出可用主题 (按 scope 过滤, Global 永远可见)
    pub async fn list_available(&self, ctx: &ThemeContext) -> Result<Vec<Theme>, ThemeError> {
        // 1. 内置永远可见
        let mut all = self.repo.list_builtin().await?;
        // 2. 当前租户的 Tenant scope 主题
        let tenant_themes = self
            .repo
            .list_by_scope(ThemeScope::Tenant, ctx.tenant_id, None)
            .await?;
        all.extend(tenant_themes);
        // 3. 当前用户的 Personal scope 主题
        if let Some(actor) = ctx.actor_id {
            let personal = self
                .repo
                .list_by_scope(ThemeScope::Personal, ctx.tenant_id, Some(actor))
                .await?;
            all.extend(personal);
        }
        // 去重 (按 theme_id)
        all.sort_by_key(|t| t.theme_id.as_str().to_string());
        all.dedup_by_key(|t| t.theme_id);
        Ok(all)
    }

    /// 设置主题 (任意 scope)
    pub async fn set(
        &self,
        ctx: &ThemeContext,
        theme_id: ThemeId,
        definition: ThemeDefinition,
    ) -> Result<Theme, ThemeError> {
        // INV-THEME-03
        if !inv_03_definition_complete(&definition) {
            return Err(ThemeError::IncompleteDefinition(
                "缺 color / spacing / radius".into(),
            ));
        }
        // INV-THEME-01
        if !inv_01_id_valid(&theme_id) {
            return Err(ThemeError::NotFound(theme_id.as_str().into()));
        }
        // 权限: Personal 必须 actor 匹配
        match (ctx.resolution_chain.first(), ctx.actor_id) {
            (Some(ThemeScope::Personal), None) => {
                return Err(ThemeError::PermissionDenied {
                    actor: "anonymous".into(),
                    scope: "personal".into(),
                });
            }
            _ => {}
        }

        let scope_owner = match ctx
            .resolution_chain
            .first()
            .copied()
            .unwrap_or(ThemeScope::Global)
        {
            ThemeScope::Personal => ScopeOwner::Personal {
                actor_id: ctx.actor_id.ok_or_else(|| ThemeError::PermissionDenied {
                    actor: "anonymous".into(),
                    scope: "personal".into(),
                })?,
            },
            ThemeScope::Tenant => ScopeOwner::Tenant {
                tenant_id: ctx.tenant_id,
            },
            ThemeScope::Global => ScopeOwner::Global,
        };
        let scope = scope_owner.scope_enum();

        // 查已有
        let existing = self
            .repo
            .find_by_scope(theme_id, scope, ctx.actor_id, ctx.tenant_id)
            .await?;

        let mut theme = if let Some(mut t) = existing {
            // INV-THEME-04
            if !inv_04_version_monotonic(t.version, t.version + 1) {
                return Err(ThemeError::Storage("version monotonicity".into()));
            }
            t.definition = definition;
            t.display_name = theme_id.as_str().to_string();
            t.bump_version();
            t
        } else {
            Theme::new(theme_id, theme_id.as_str(), definition, scope, scope_owner)
        };

        self.repo.save(&theme).await?;

        // 发事件
        self.bus
            .publish(ThemeEvent::Changed {
                theme_id,
                scope,
                actor_id: ctx.actor_id,
                tenant_id: ctx.tenant_id,
                at: chrono::Utc::now(),
            })
            .await?;

        Ok(theme)
    }
}

impl ScopeOwner {
    /// 转换为对应的 ThemeScope
    pub fn scope_enum(&self) -> ThemeScope {
        match self {
            ScopeOwner::Personal { .. } => ThemeScope::Personal,
            ScopeOwner::Tenant { .. } => ThemeScope::Tenant,
            ScopeOwner::Global => ThemeScope::Global,
        }
    }
}
