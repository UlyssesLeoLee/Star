// SPDX-License-Identifier: MIT OR Apache-2.0
//! `star-ops` 6-field 错误模型 (per agent-api/v1#Error §3.14, 简化版)
//!
//! per `docs/requirements/SRS-STAR-OPS-001.md` §5 + §6.5
//! per `docs/basic-design/OPS-BASIC-DESIGN-001.md` §3.5
//!
//! MVP-骨架阶段: 仅暴露 6 字段 struct, 5 个核心错误码
//! (NOT_IMPLEMENTED / UNAUTHORIZED / RATE_LIMITED / INTERNAL / BAD_REQUEST).
//! 24 个 SCREAMING_SNAKE_CASE 错误码在 star-mcp 已有完整定义, 本 crate MVP 仅
//! 暴露 ops 域常用的 5 个, 实装阶段扩到完整 24+ 码时复用 star-mcp::error_code 模式.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 错误来源分类 (简化版, 跟 star-mcp 6 source_kind 对齐)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorSourceKind {
    /// 内部逻辑错误 (MVP stub 未实装)
    Internal,
    /// 外部系统错误 (K8s/LLM 调用失败, 实装阶段触发)
    External,
    /// 策略层拒绝 (权限/限流, MVP 简化为 stub)
    Policy,
    /// 参数 / schema 校验失败
    Validation,
}

/// 6-field 错误响应 (per agent-api/v1#Error §3.14)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpsErrorBody {
    /// SCREAMING_SNAKE_CASE 错误码
    pub code: String,
    /// 人类可读错误描述
    pub message: String,
    /// 错误来源 crate::module (e.g. "star_ops::ops_api::cluster")
    pub source_module: String,
    /// 错误分类 (per ErrorSourceKind)
    pub source_kind: ErrorSourceKind,
    /// 是否可重试
    pub retriable: bool,
    /// 修复提示 (e.g. "见 docs/requirements/SRS-STAR-OPS-001.md §3.1 落档清单")
    pub hint: Option<String>,
}

/// Ops 域错误类型 (MVP 5 个核心 variant)
#[derive(Debug, Error)]
pub enum OpsError {
    #[error("NOT_IMPLEMENTED: {0}")]
    NotImplemented(String),
    #[error("UNAUTHORIZED: {0}")]
    Unauthorized(String),
    #[error("RATE_LIMITED: {0}")]
    RateLimited(String),
    #[error("BAD_REQUEST: {0}")]
    BadRequest(String),
    #[error("INTERNAL: {0}")]
    Internal(String),
}

impl OpsError {
    /// 转换为 6-field 错误响应
    pub fn to_body(&self, source_module: &str) -> OpsErrorBody {
        match self {
            OpsError::NotImplemented(msg) => OpsErrorBody {
                code: "NOT_IMPLEMENTED".to_string(),
                message: msg.clone(),
                source_module: source_module.to_string(),
                source_kind: ErrorSourceKind::Internal,
                retriable: false,
                hint: Some(
                    "见 docs/requirements/SRS-STAR-OPS-001.md §3.1 落档清单 + 拍板 [M] 子项"
                        .to_string(),
                ),
            },
            OpsError::Unauthorized(msg) => OpsErrorBody {
                code: "UNAUTHORIZED".to_string(),
                message: msg.clone(),
                source_module: source_module.to_string(),
                source_kind: ErrorSourceKind::Policy,
                retriable: false,
                hint: Some("检查 Authorization header + tenant_admin 角色".to_string()),
            },
            OpsError::RateLimited(msg) => OpsErrorBody {
                code: "RATE_LIMITED".to_string(),
                message: msg.clone(),
                source_module: source_module.to_string(),
                source_kind: ErrorSourceKind::Policy,
                retriable: true,
                hint: Some("60 req/min per key, 等待后重试".to_string()),
            },
            OpsError::BadRequest(msg) => OpsErrorBody {
                code: "BAD_REQUEST".to_string(),
                message: msg.clone(),
                source_module: source_module.to_string(),
                source_kind: ErrorSourceKind::Validation,
                retriable: false,
                hint: Some("检查请求 body / query 参数".to_string()),
            },
            OpsError::Internal(msg) => OpsErrorBody {
                code: "INTERNAL".to_string(),
                message: msg.clone(),
                source_module: source_module.to_string(),
                source_kind: ErrorSourceKind::Internal,
                retriable: true,
                hint: Some("查看 tracing 日志 + ops_cluster_action_log 审计表".to_string()),
            },
        }
    }

    /// 便捷构造: NOT_IMPLEMENTED
    pub fn not_implemented(what: &str) -> Self {
        OpsError::NotImplemented(format!("{} 端到端实装待 [M] 子项拍板", what))
    }

    /// 便捷构造: 无可用 AI 通道
    pub fn no_channel_available() -> Self {
        OpsError::Internal(
            "所有 AI 通道均不可用, 检查 mock subprocess 路径 + 真实通道配置".to_string(),
        )
    }
}

/// 顶层错误响应包装 (per OPS-BASIC-DESIGN-001.md §3.5)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpsErrorResponse {
    pub error: OpsErrorBody,
}

impl axum::response::IntoResponse for OpsError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            OpsError::NotImplemented(_) => axum::http::StatusCode::NOT_IMPLEMENTED,
            OpsError::Unauthorized(_) => axum::http::StatusCode::UNAUTHORIZED,
            OpsError::RateLimited(_) => axum::http::StatusCode::TOO_MANY_REQUESTS,
            OpsError::BadRequest(_) => axum::http::StatusCode::BAD_REQUEST,
            OpsError::Internal(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = OpsErrorResponse {
            error: self.to_body("star_ops"),
        };
        (status, axum::Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_implemented_returns_501() {
        let err = OpsError::not_implemented("F-01 集群更新");
        let body = err.to_body("star_ops::ops_api::cluster");
        assert_eq!(body.code, "NOT_IMPLEMENTED");
        assert_eq!(body.source_kind, ErrorSourceKind::Internal);
        assert!(!body.retriable);
        assert!(body.hint.is_some());
    }

    #[test]
    fn unauthorized_returns_401_with_policy_source() {
        let err = OpsError::Unauthorized("missing api key".to_string());
        let body = err.to_body("star_ops::ops_api::auth");
        assert_eq!(body.code, "UNAUTHORIZED");
        assert_eq!(body.source_kind, ErrorSourceKind::Policy);
    }

    #[test]
    fn rate_limited_is_retriable() {
        let err = OpsError::RateLimited("60 req/min exceeded".to_string());
        let body = err.to_body("star_ops::ops_api::rate_limit");
        assert_eq!(body.code, "RATE_LIMITED");
        assert!(body.retriable);
    }
}
