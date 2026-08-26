//! 顶层命令模块聚合(per `docs/.../spec/cli/01-cli-spec.md` §2 缩到 Phase D MVP)
//!
//! Phase D 骨架只暴露 3 个子命令树:
//! - `star agent capabilities`
//! - `star task current`
//! - `star submit`

#![warn(missing_docs)]

pub(crate) mod agent;
pub(crate) mod submit;
pub(crate) mod task;
