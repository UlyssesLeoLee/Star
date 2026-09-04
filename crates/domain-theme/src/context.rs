//! 主题系统上下文

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 主题操作上下文 (per 2026-08-29 04:09 JST 三层作用域)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeContext {
    /// 当前操作用户 (None = 匿名, 公开访问)
    pub actor_id: Option<Uuid>,
    /// 当前租户
    pub tenant_id: Uuid,
    /// 请求时间戳
    pub requested_at: chrono::DateTime<chrono::Utc>,
    /// 三层解析时按此顺序查找
    pub resolution_chain: Vec<super::value_object::ThemeScope>,
}

impl ThemeContext {
    /// 构造上下文, 使用默认解析顺序(Personal > Tenant > Global)
    pub fn new(actor_id: Option<Uuid>, tenant_id: Uuid) -> Self {
        Self {
            actor_id,
            tenant_id,
            requested_at: chrono::Utc::now(),
            // 默认解析顺序: Personal > Tenant > Global
            resolution_chain: vec![
                super::value_object::ThemeScope::Personal,
                super::value_object::ThemeScope::Tenant,
                super::value_object::ThemeScope::Global,
            ],
        }
    }

    /// 是否为匿名(未登录)访问
    pub fn is_anonymous(&self) -> bool {
        self.actor_id.is_none()
    }
}
