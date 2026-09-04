//! CLI 子命令模块聚合
//!
//! Phase D.2 MVP 17 核心命令 (3 已有 + 14 新)
//! Phase D 极简骨架 3 命令: agent / task / submit
//! Phase D.2 新增 14 命令: project / issue / context / code / workspace / worktree / mr / test / pipeline

pub(crate) mod agent;
pub(crate) mod code;
pub(crate) mod context;
pub(crate) mod issue;
pub(crate) mod mr;
pub(crate) mod pipeline;
pub(crate) mod project;
pub(crate) mod submit;
pub(crate) mod task;
pub(crate) mod test;
pub(crate) mod workspace;
pub(crate) mod worktree;
