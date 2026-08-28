//! 主题系统事件

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::value_object::{ThemeId, ThemeScope};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThemeEvent {
    /// 主题被变更
    Changed {
        theme_id: ThemeId,
        scope: ThemeScope,
        actor_id: Option<Uuid>,
        tenant_id: Uuid,
        at: DateTime<Utc>,
    },
    /// 主题被注册 (admin only)
    Registered {
        theme_id: ThemeId,
        actor_id: Uuid,
        at: DateTime<Utc>,
    },
    /// 主题被废弃
    Deprecated {
        theme_id: ThemeId,
        actor_id: Uuid,
        at: DateTime<Utc>,
    },
}
