//! SCM 域错误(`ScmError`)
//!
//! 来源: docs/api-design.md §8.3.12(SC- 系列),docs/specs/domain-scm-spec.md §8
//!
//! **7 标准变体**(SC-001~006 + 内部错误):
//! - `NotFound` — 资源不存在(SC-001,HTTP 404)
//! - `InvalidState` — 违反不变量 / 状态机非法迁移
//! - `PermissionDenied` — 跨租户访问 / 角色不足(SEC-001/002/007)
//! - `Conflict` — 同步冲突 / 唯一键冲突(SC-003 / SC-004)
//! - `InvalidArgument` — 参数校验失败
//! - `ExternalError` — 厂商 API 错误 / 限流(SC-002 / SC-005)
//! - `CredentialMissing` — Provider Credential 缺失(SC-006)
//! - `Internal` — 内部错误(IO / 序列化 / 事件总线等)
//!
//! Phase 3 由 `crates/api` 实现 `Into<ApiError>` 完成 HTTP 状态码映射。

use crate::value_object::RepositoryId;

/// **SCM 域错误**
#[derive(Debug, thiserror::Error)]
pub enum ScmError {
    /// 资源不存在(SC-001,HTTP 404)
    #[error("scm resource not found: repository_id={0}")]
    NotFound(RepositoryId),

    /// 违反不变量或状态机非法迁移(INV-SCM-01~08 / §7.5 PR 状态机)
    #[error("scm invalid state: {0}")]
    InvalidState(String),

    /// 权限不足(跨租户访问 / 角色不足,SEC-001/002/007)
    #[error("scm permission denied")]
    PermissionDenied,

    /// 唯一键冲突 / 乐观锁失败 / 重复 Webhook 事件(SC-003 / SC-004)
    #[error("scm conflict: {0}")]
    Conflict(String),

    /// 参数校验失败(空字符串、超长等)
    #[error("scm invalid argument: {0}")]
    InvalidArgument(String),

    /// 厂商 API 错误 / 限流(SC-002 / SC-005)
    #[error("scm external error: {0}")]
    ExternalError(String),

    /// Provider Credential 缺失(SC-006,HTTP 403)
    #[error("scm credential missing: {0}")]
    CredentialMissing(String),

    /// 内部错误
    #[error("scm internal error: {0}")]
    Internal(String),
}

impl ScmError {
    /// 错误码字符串(供 `crates/api` 映射 HTTP 状态码 / NATS subject)
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "SC-001",
            Self::InvalidState(_) => "SC-007",
            Self::PermissionDenied => "SC-006",
            Self::Conflict(_) => "SC-003",
            Self::InvalidArgument(_) => "SC-008",
            Self::ExternalError(_) => "SC-005",
            Self::CredentialMissing(_) => "SC-006",
            Self::Internal(_) => "SC-000",
        }
    }

    /// 是否为 5xx 错误
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_) | Self::ExternalError(_))
    }

    /// 是否为 4xx 错误
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::NotFound(_)
                | Self::InvalidState(_)
                | Self::PermissionDenied
                | Self::Conflict(_)
                | Self::InvalidArgument(_)
                | Self::CredentialMissing(_)
        )
    }
}

impl From<uuid::Error> for ScmError {
    fn from(e: uuid::Error) -> Self {
        Self::Internal(format!("uuid error: {e}"))
    }
}
