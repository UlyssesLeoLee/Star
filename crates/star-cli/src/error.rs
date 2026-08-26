//! `star` CLI 错误类型(per `docs/.../spec/cli/01-cli-spec.md` §5 错误模型)
//!
//! Phase D 骨架只暴露 stub 错误(序列化失败 / IO 失败),
//! 完整 9 类错误模型待 Phase D.1 增量补齐。

#![warn(missing_docs)]

use thiserror::Error;

/// CLI 顶层错误(per spec §5)
#[derive(Debug, Error)]
pub(crate) enum StarError {
    /// JSON 序列化失败(serde_json 抛出的错误)
    #[error("json serialization failed: {0}")]
    Json(#[from] serde_json::Error),

    /// IO 错误(当前未使用,留作未来 stub 扩展)
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl StarError {
    /// 退出码(0=OK,1=用户错误,2=内部错误)
    pub(crate) const fn exit_code(&self) -> u8 {
        match self {
            Self::Json(_) | Self::Io(_) => 2,
        }
    }
}
