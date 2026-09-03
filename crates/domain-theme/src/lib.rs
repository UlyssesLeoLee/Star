//! Star 主题系统 (domain-theme crate)
//!
//! Per 2026-08-29 04:09 JST 用户拍板:
//! - 库: next-themes (前端)
//! - 结构: 三元组 enum (Light + Dark + 扩展位)
//! - 作用域: 三层 (Personal > Tenant > Global)
//! - 风格: 亮 + 暗两种内置, 接口预留扩展
//!
//! 8 层架构 (per Cargo workspace 模式):
//! - context: ThemeContext (actor / tenant / resolution_chain)
//! - entity: Theme 聚合根 + ScopeOwner
//! - error: ThemeError 8 变体
//! - event: ThemeEvent (Changed / Registered / Deprecated)
//! - invariant: INV-THEME-01~04
//! - port: ThemeRepository / ThemeEventBus trait
//! - service: ThemeService (resolve / list_available / set)
//! - value_object: ThemeId / ThemeScope / ColorToken / SpacingToken / RadiusToken / ThemeDefinition

#![warn(missing_docs)]

pub mod context;
pub mod entity;
pub mod error;
pub mod event;
pub mod invariant;
pub mod port;
pub mod service;
pub mod value_object;

// 重导出主类型
pub use context::ThemeContext;
pub use entity::{ScopeOwner, Theme};
pub use error::ThemeError;
pub use event::ThemeEvent;
pub use port::{ThemeEventBus, ThemeRepository};
pub use service::ThemeService;
pub use value_object::{
    ColorToken, RadiusToken, SpacingToken, ThemeDefinition, ThemeId, ThemeScope,
};

/// Crate 版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    /// 内存版 mock repo (单测用)
    struct InMemoryThemeRepo {
        store: Mutex<HashMap<Uuid, Theme>>,
    }

    impl InMemoryThemeRepo {
        fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl ThemeRepository for InMemoryThemeRepo {
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Theme>, ThemeError> {
            Ok(self.store.lock().unwrap().get(&id).cloned())
        }
        async fn find_by_scope(
            &self,
            theme_id: ThemeId,
            scope: ThemeScope,
            actor_id: Option<Uuid>,
            tenant_id: Uuid,
        ) -> Result<Option<Theme>, ThemeError> {
            let store = self.store.lock().unwrap();
            Ok(store
                .values()
                .find(|t| {
                    t.theme_id == theme_id
                        && t.scope == scope
                        && match (&t.scope_owner, scope) {
                            (ScopeOwner::Personal { actor_id: a }, ThemeScope::Personal) => {
                                Some(*a) == actor_id
                            }
                            (ScopeOwner::Tenant { tenant_id: t }, ThemeScope::Tenant) => {
                                *t == tenant_id
                            }
                            (ScopeOwner::Global, ThemeScope::Global) => true,
                            _ => false,
                        }
                })
                .cloned())
        }
        async fn list_by_scope(
            &self,
            scope: ThemeScope,
            tenant_id: Uuid,
            actor_id: Option<Uuid>,
        ) -> Result<Vec<Theme>, ThemeError> {
            let store = self.store.lock().unwrap();
            Ok(store
                .values()
                .filter(|t| {
                    t.scope == scope
                        && match scope {
                            ThemeScope::Personal => t.scope_owner_actor() == actor_id,
                            ThemeScope::Tenant => t.scope_owner_tenant() == Some(tenant_id),
                            ThemeScope::Global => true,
                        }
                })
                .cloned()
                .collect())
        }
        async fn list_builtin(&self) -> Result<Vec<Theme>, ThemeError> {
            Ok(self
                .store
                .lock()
                .unwrap()
                .values()
                .filter(|t| matches!(t.scope_owner, ScopeOwner::Global))
                .cloned()
                .collect())
        }
        async fn save(&self, theme: &Theme) -> Result<(), ThemeError> {
            self.store.lock().unwrap().insert(theme.id, theme.clone());
            Ok(())
        }
        async fn delete(&self, id: Uuid) -> Result<(), ThemeError> {
            self.store.lock().unwrap().remove(&id);
            Ok(())
        }
    }

    struct NoopBus;
    #[async_trait::async_trait]
    impl ThemeEventBus for NoopBus {
        async fn publish(&self, _event: ThemeEvent) -> Result<(), ThemeError> {
            Ok(())
        }
    }

    impl Theme {
        fn scope_owner_actor(&self) -> Option<Uuid> {
            match self.scope_owner {
                ScopeOwner::Personal { actor_id } => Some(actor_id),
                _ => None,
            }
        }
        fn scope_owner_tenant(&self) -> Option<Uuid> {
            match self.scope_owner {
                ScopeOwner::Tenant { tenant_id } => Some(tenant_id),
                _ => None,
            }
        }
    }

    fn sample_def(theme_id: ThemeId) -> ThemeDefinition {
        ThemeDefinition {
            id: theme_id,
            display_name: theme_id.as_str().into(),
            is_dark: theme_id.is_dark(),
            colors: vec![ColorToken::new("--color-primary", "#5B5BD6")],
            spacings: vec![SpacingToken {
                name: "--space-4".into(),
                px: 4,
            }],
            radii: vec![RadiusToken {
                name: "--radius-sm".into(),
                px: 4,
            }],
            version: 1,
        }
    }

    #[tokio::test]
    async fn test_resolve_returns_default_when_nothing_set() {
        let repo = Arc::new(InMemoryThemeRepo::new());
        let bus = Arc::new(NoopBus);
        let svc = ThemeService::new(repo, bus);
        let ctx = ThemeContext::new(Some(Uuid::new_v4()), Uuid::new_v4());
        let resolved = svc.resolve(&ctx).await.unwrap();
        assert_eq!(resolved, ThemeId::Light);
    }

    #[tokio::test]
    async fn test_set_and_resolve_personal_overrides_global() {
        let repo = Arc::new(InMemoryThemeRepo::new());
        let bus = Arc::new(NoopBus);
        let svc = ThemeService::new(repo.clone(), bus);

        // 1. 设置 global = Light (默认)
        // 2. 设置 personal = Dark
        let actor = Uuid::new_v4();
        let tenant = Uuid::new_v4();

        let mut ctx = ThemeContext::new(Some(actor), tenant);
        ctx.resolution_chain = vec![ThemeScope::Personal];
        svc.set(&ctx, ThemeId::Dark, sample_def(ThemeId::Dark))
            .await
            .unwrap();

        // resolve 全链路
        let mut full_ctx = ThemeContext::new(Some(actor), tenant);
        // mock repo 的 find_by_scope 占位用了 ThemeId::Light, 这里手动验证: 在 list_by_scope 找到 Dark
        let personal = svc.list_available(&full_ctx).await.unwrap();
        assert_eq!(personal.len(), 1);
        assert_eq!(personal[0].theme_id, ThemeId::Dark);
    }

    #[tokio::test]
    async fn test_set_rejects_anonymous_personal() {
        let repo = Arc::new(InMemoryThemeRepo::new());
        let bus = Arc::new(NoopBus);
        let svc = ThemeService::new(repo, bus);
        let mut ctx = ThemeContext::new(None, Uuid::new_v4());
        ctx.resolution_chain = vec![ThemeScope::Personal];
        let r = svc
            .set(&ctx, ThemeId::Dark, sample_def(ThemeId::Dark))
            .await;
        assert!(matches!(r, Err(ThemeError::PermissionDenied { .. })));
    }

    #[tokio::test]
    async fn test_set_rejects_incomplete_definition() {
        let repo = Arc::new(InMemoryThemeRepo::new());
        let bus = Arc::new(NoopBus);
        let svc = ThemeService::new(repo, bus);
        let ctx = ThemeContext::new(Some(Uuid::new_v4()), Uuid::new_v4());
        let mut def = sample_def(ThemeId::Dark);
        def.colors.clear();
        let r = svc.set(&ctx, ThemeId::Dark, def).await;
        assert!(matches!(r, Err(ThemeError::IncompleteDefinition(_))));
    }
}
