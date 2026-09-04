//! 主题系统事件

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::value_object::{ThemeId, ThemeScope};

/// 主题系统领域事件
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThemeEvent {
    /// 主题被变更
    Changed {
        /// 被变更的主题 ID
        theme_id: ThemeId,
        /// 变更所在作用域
        scope: ThemeScope,
        /// 执行变更的用户(None = 匿名)
        actor_id: Option<Uuid>,
        /// 所属租户
        tenant_id: Uuid,
        /// 变更时间
        at: DateTime<Utc>,
    },
    /// 主题被注册 (admin only)
    Registered {
        /// 被注册的主题 ID
        theme_id: ThemeId,
        /// 执行注册的管理员
        actor_id: Uuid,
        /// 注册时间
        at: DateTime<Utc>,
    },
    /// 主题被废弃
    Deprecated {
        /// 被废弃的主题 ID
        theme_id: ThemeId,
        /// 执行废弃操作的管理员
        actor_id: Uuid,
        /// 废弃时间
        at: DateTime<Utc>,
    },
}
