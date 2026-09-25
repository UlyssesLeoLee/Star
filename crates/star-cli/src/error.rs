//! `star` CLI 错误类型(per `docs/.../spec/cli/01-cli-spec.md` §5 错误模型)
//!
//! Phase D 骨架只暴露 stub 错误(序列化失败 / IO 失败),
//! 完整 9 类错误模型待 Phase D.1 增量补齐。
//!
//! ULYS-218.2 加 `Picker(String)` variant: Start-from Picker 子命令(per FR-ORCA-009 CLI)
//! 调 backend `worktree-shared-dir::StartFromPicker`, 该 crate 抛 `SharedDirError`,
//! CLI 侧 catch 后转 `StarError::Picker(<message>)` 上报用户. 退出码 2 (内部错).

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

    /// Skill Registry 错误 (per FR-ORCA-034, ULYS-196)
    #[error("skill registry: {0}")]
    Skill(String),

    /// Start-from Picker 错误 (per FR-ORCA-009 CLI, ULYS-218.2)
    ///
    /// backend `worktree-shared-dir::StartFromPicker::list_candidates` / `resolve`
    /// 失败时上报. 典型原因: repo path 不存在 / git 二进制找不到 / 不是 git 仓库 /
    /// branch 不存在等. 退出码归 2 (内部错).
    #[error("start-from picker: {0}")]
    Picker(String),
}

impl StarError {
    /// 退出码(0=OK,1=用户错误,2=内部错误)
    pub(crate) const fn exit_code(&self) -> u8 {
        match self {
            Self::Json(_) | Self::Io(_) => 2,
            // Skill 错误分类: 既有用户错(AlreadyExists / NotFound / InvalidSource)
            // 也有内部错(mvp 当前归 2)
            Self::Skill(_) => 1,
            // Picker 错误当前归 2 (内部错); 后续若细分 user/input vs internal
            // 可仿 Skill 模式按错误码分流.
            Self::Picker(_) => 2,
        }
    }
}
