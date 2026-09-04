//! 主题系统端口 (六边形架构: trait 抽象)

use async_trait::async_trait;
use uuid::Uuid;

use super::entity::Theme;
use super::error::ThemeError;
use super::value_object::{ThemeId, ThemeScope};

/// 主题仓储端口(六边形架构出站端口)
#[async_trait]
pub trait ThemeRepository: Send + Sync {
    /// 按 ID 查询
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Theme>, ThemeError>;

    /// 按 (theme_id, scope, scope_owner) 查
    async fn find_by_scope(
        &self,
        theme_id: ThemeId,
        scope: ThemeScope,
        actor_id: Option<Uuid>,
        tenant_id: Uuid,
    ) -> Result<Option<Theme>, ThemeError>;

    /// 列出某 scope 下的所有主题
    async fn list_by_scope(
        &self,
        scope: ThemeScope,
        tenant_id: Uuid,
        actor_id: Option<Uuid>,
    ) -> Result<Vec<Theme>, ThemeError>;

    /// 列出全部内置主题 (不依赖 scope, 全局可见)
    async fn list_builtin(&self) -> Result<Vec<Theme>, ThemeError>;

    /// 保存
    async fn save(&self, theme: &Theme) -> Result<(), ThemeError>;

    /// 删除
    async fn delete(&self, id: Uuid) -> Result<(), ThemeError>;
}

/// 主题领域事件发布端口
#[async_trait]
pub trait ThemeEventBus: Send + Sync {
    /// 发布主题领域事件
    async fn publish(&self, event: super::event::ThemeEvent) -> Result<(), ThemeError>;
}
