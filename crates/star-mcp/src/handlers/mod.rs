// crates/star-mcp/src/handlers/mod.rs
// SPDX-License-Identifier: MIT OR Apache-2.0
//! 22 domain handler 注册 (Phase H 真实数据接入)
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2
//! + `spec/mcp/02-resources-prompts-spec.md` §2
//! + `spec/cache/01-cache-contract-spec.md` §4
//!
//! 23 domain (= 22 task brief 22 + scm 1, 5 域分类 per spec/agents/02 §2.1):
//! - Agent 运行时: agent / validation (1 of 4; lease/resume 排 Phase H+)
//! - Worktree / Workspace: worktree / scm (2 of 3; workspace 在 Phase E 已硬编码, 留 Phase H+ 切换)
//! - 工作流 / 任务: decision / automation / work_item (3 of 5; flow/event 排 Phase H+)
//! - 权限 / 多租户: audit / identity / permission / tenant (4 of 5; policy 排 Phase H+)
//! - 集成 / 通知: context / feedback / integration / notification / search (5 of 5)
//! - 协作扩展 (per spec/agents/02 §6 #1): board / collaboration / comment
//! - 项目管理扩展: planning / project / relation / development
//!
//! 全部 mock-but-functional, 真实数据源接入 (`crates/domain-*`) 排 Phase H+ 排期。
pub(crate) mod agent;
pub(crate) mod audit;
pub(crate) mod automation;
pub(crate) mod board;
pub(crate) mod collaboration;
pub(crate) mod comment;
pub(crate) mod context;
pub(crate) mod decision;
pub(crate) mod development;
pub(crate) mod feedback;
pub(crate) mod identity;
pub(crate) mod integration;
pub(crate) mod notification;
pub(crate) mod permission;
pub(crate) mod planning;
pub(crate) mod project;
pub(crate) mod relation;
pub(crate) mod scm;
pub(crate) mod search;
pub(crate) mod tenant;
pub(crate) mod validation;
pub(crate) mod work_item;
pub(crate) mod workspace;
pub(crate) mod worktree;

use crate::resources::DynResource;

/// 22 domain handler 一键注册 (per `docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md` §2
/// + `spec/mcp/02-resources-prompts-spec.md` §2)
///
/// 顺序不重要, `ResourcesHandler::read` 按 URI scheme 线性匹配。
/// 全部 22 handler 装入 `Vec<Box<dyn DynResource>>`。
#[allow(dead_code)]
pub(crate) fn all_domain_handlers() -> Vec<Box<dyn DynResource>> {
    vec![
        Box::new(agent::AgentHandler::new()),
        Box::new(audit::AuditHandler),
        Box::new(automation::AutomationHandler),
        Box::new(board::BoardHandler),
        Box::new(collaboration::CollaborationHandler),
        Box::new(comment::CommentHandler),
        Box::new(context::ContextHandler),
        Box::new(decision::DecisionHandler),
        Box::new(development::DevelopmentHandler),
        Box::new(feedback::FeedbackHandler::new()),
        Box::new(identity::IdentityHandler::new()),
        Box::new(integration::IntegrationHandler),
        Box::new(notification::NotificationHandler),
        Box::new(permission::PermissionHandler::new()),
        Box::new(planning::PlanningHandler),
        Box::new(project::ProjectHandler::new()),
        Box::new(relation::RelationHandler),
        Box::new(scm::ScmHandler),
        Box::new(search::SearchHandler),
        Box::new(tenant::TenantHandler::new()),
        Box::new(validation::ValidationHandler),
        Box::new(work_item::WorkItemHandler::new()),
        Box::new(workspace::WorkspaceHandler::new()),
        Box::new(worktree::WorktreeHandler::new()),
    ]
}
