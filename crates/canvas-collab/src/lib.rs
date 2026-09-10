//! canvas-collab: A12 多人编辑 业务逻辑 (per P3-D.6 实施计划 §4.3)
//!
//! 本 crate 是 P3-D.6 启动实装 阶段 1 基础 任务 1.3, 仅骨架 (0 业务方法),
//! 阶段 2 业务 实装阶段 才落地 A12 多人编辑 8 项业务方法 (create_element / update_cursor / post_comment / grant_permission / record_audit / ...).
//!
//! 派生自 `docs/design/DD-CANVAS-AGENT-001.md` v0.1 §3.1 module 布局 + §4.14 A12 sub-class + `docs/design/DD-CANVAS-001.md` v0.1.1 C-25 `CanvasElementsBackend` + C-26 `CanvasMultiUserAudit`.
//! 5 域 Lead 真人未到位, Mavis 临时代签 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D).
//! 真人到位后追溯签字覆盖修订历史 (per 守门 #1 禁回溯叙事).

#![warn(missing_docs)]

pub mod models;

pub use models::audit::CanvasMultiUserAudit;
pub use models::comment::CanvasComment;
pub use models::element::CanvasElementBackend;
pub use models::permission::CanvasPermission;
pub use models::presence::PresenceCursor;

/// Canvas-collab 错误类型 (per 守门 #11 缺标比错标, 显式定义, 0 业务方法).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasCollabError {
    /// 阶段 1 基础 占位, 阶段 2 业务 落地业务错误.
    NotImplemented,
}
