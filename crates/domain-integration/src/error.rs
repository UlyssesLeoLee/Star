//! Integration 域错误(`IntegrationError`)
//!
//! 来源: `docs/api-design.md` §8.3.13(I- 系列),`docs/specs/domain-integration-spec.md` §8
//!
//! **8 标准变体**(I-001~005 + SEC + 内部错误):
//! - `NotFound` — Integration 不存在(I-001,HTTP 404)
//! - `InvalidState` — 违反不变量 / 状态机非法迁移(I-007)
//! - `PermissionDenied` — 跨租户访问 / 角色不足(SEC-001/002/007)
//! - `Conflict` — 同步冲突 / 唯一键冲突(I-003)
//! - `InvalidArgument` — 参数校验失败(I-002,provider 不可用)
//! - `LoopGuardMissing` — Bidirectional Sync 缺 Loop 防护(I-004,HTTP 422)
//! - `CredentialMissing` — Provider Credential 缺失(I-005,HTTP 422)
//! - `Internal` — 内部错误

use crate::value_object::IntegrationId;

/// **Integration 域错误**
#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    /// Integration 不存在(I-001,HTTP 404)
    #[error("integration not found: integration_id={0}")]
    NotFound(IntegrationId),

    /// 违反不变量 / 状态机非法迁移(I-007)
    #[error("integration invalid state: {0}")]
    InvalidState(String),

    /// 权限不足(跨租户访问 / 角色不足,SEC-001/002/007)
    #[error("integration permission denied")]
    PermissionDenied,

    /// 同步冲突 / 唯一键冲突(I-003)
    #[error("integration conflict: {0}")]
    Conflict(String),

    /// 参数校验失败(provider 不可用等,I-002,HTTP 422)
    #[error("integration invalid argument: {0}")]
    InvalidArgument(String),

    /// Bidirectional Sync 缺 Loop 防护(I-004,HTTP 422,INV-I-02)
    #[error("integration loop guard missing: {0}")]
    LoopGuardMissing(String),

    /// Provider Credential 缺失(I-005,HTTP 422,INV-I-04)
    #[error("integration credential missing: {0}")]
    CredentialMissing(String),

    /// 内部错误
    #[error("integration internal error: {0}")]
    Internal(String),
}

impl IntegrationError {
    /// 错误码字符串(供 `crates/api` 映射 HTTP 状态码 / NATS subject)
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "I-001",
            Self::InvalidState(_) => "I-007",
            Self::PermissionDenied => "SEC-007",
            Self::Conflict(_) => "I-003",
            Self::InvalidArgument(_) => "I-002",
            Self::LoopGuardMissing(_) => "I-004",
            Self::CredentialMissing(_) => "I-005",
            Self::Internal(_) => "I-000",
        }
    }

    /// 是否为 5xx 错误
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_))
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
                | Self::LoopGuardMissing(_)
                | Self::CredentialMissing(_)
        )
    }
}
