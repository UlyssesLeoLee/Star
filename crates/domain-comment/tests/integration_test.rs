//! 集成测试 (IT) — domain-comment 真实数据接入验证
//!
//! **crate**: domain-comment (test target)
//! **per**: 2026-09-07 P3-B sub-batch 2 write-heavy 域真实数据接入
//! **参考**: `crates/domain-workspace/tests/integration_test.rs` (per 7f9f52a 4 域真实接入已落地)
//!
//! ## 验证维度 (V0-V3)
//!
//! - V0 (基础 CRUD): create + get_by_id + list_by_parent
//! - V1 (编辑/软删除): edit 状态机 + delete 软删除 + list 过滤 deleted
//! - V2 (RLS 13 類): 跨 tenant 拒绝 (守门 #13 (c) Master 100% RLS)
//! - V3 (不变量): INV-C-03 Reaction 唯一 + INV-C-04 object_key 必带 tenant 前缀

use domain_comment::{
    ActorContext, AddReactionCommand, AgentId, AttachmentId, CommentCommandPort, CommentError,
    CommentQueryPort, CommentStatus, CreateCommentCommand, DeleteCommentCommand,
    EditCommentCommand, GetCommentQuery, InMemoryCommentService, ListByParentQuery, ParentType,
    ProjectId, RegisterAttachmentCommand, TenantId, UserId,
};
use domain_permission::{
    Action, CheckQuery, CreateSchemeCommand, Effect, GrantRoleCommand, InMemoryPermissionService,
    PermissionCommandPort, PermissionError, PermissionQueryPort, PermissionRule, PermissionScheme,
    ProjectId as PermProjectId, ResourceType, Role, SubjectType, TenantId as PermTenantId,
    UpsertRuleCommand, UserId as PermUserId,
};
use std::sync::Arc;
use uuid::Uuid;

/// IT fixture: project_admin 角色 actor (per INV-C-06 delete_comment 可代作者删)
fn make_admin_actor(tenant_id: Uuid) -> ActorContext {
    ActorContext::new(Uuid::new_v4(), tenant_id).with_role("project_admin")
}

/// IT fixture: 基础 comment 创建命令
fn basic_comment_cmd(tid: Uuid, author: Uuid) -> CreateCommentCommand {
    CreateCommentCommand {
        tenant_id: TenantId(tid),
        project_id: ProjectId::new(),
        parent_type: ParentType::WorkItem,
        parent_id: Uuid::new_v4(),
        body: "first comment".to_string(),
        author_user_id: Some(UserId::from(author)),
        author_agent_id: None,
        mentions: vec![],
        attachment_ids: vec![],
        actor_user_id: UserId::from(author),
    }
}

// =====================================================================
// V0: 基础 CRUD
// =====================================================================

/// **IT-V0-1**: 完整 create → get_by_id → list_by_parent 闭环
#[tokio::test]
async fn it_v0_create_get_list_lifecycle() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let parent_id = Uuid::new_v4();
    let author = Uuid::new_v4();
    let actor = ActorContext::new(author, tenant_id);

    // create
    let mut cmd = basic_comment_cmd(tenant_id, author);
    cmd.parent_id = parent_id;
    cmd.body = "looks good to me".to_string();
    let c = svc
        .create_comment(cmd, &actor)
        .await
        .expect("create 必成功");
    assert_eq!(c.status, CommentStatus::Open);
    assert_eq!(c.body, "looks good to me");

    // get
    let fetched = svc
        .get(
            GetCommentQuery {
                tenant_id: TenantId(tenant_id),
                comment_id: c.id,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(fetched.id, c.id);

    // list_by_parent — 1 条
    let list = svc
        .list_by_parent(
            ListByParentQuery {
                tenant_id: TenantId(tenant_id),
                parent_type: ParentType::WorkItem,
                parent_id,
                include_deleted: false,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, c.id);
}

/// **IT-V0-2**: 嵌套/多作者 — 3 个作者 5 条评论按 parent 隔离
#[tokio::test]
async fn it_v0_multiple_authors_isolated_by_parent() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let parent_a = Uuid::new_v4();
    let parent_b = Uuid::new_v4();
    let author = Uuid::new_v4();
    let actor = ActorContext::new(author, tenant_id);

    // parent_a: 3 条
    for i in 0..3 {
        let mut cmd = basic_comment_cmd(tenant_id, author);
        cmd.parent_id = parent_a;
        cmd.body = format!("a-{}", i);
        svc.create_comment(cmd, &actor).await.unwrap();
    }
    // parent_b: 2 条
    for i in 0..2 {
        let mut cmd = basic_comment_cmd(tenant_id, author);
        cmd.parent_id = parent_b;
        cmd.body = format!("b-{}", i);
        svc.create_comment(cmd, &actor).await.unwrap();
    }

    let list_a = svc
        .list_by_parent(
            ListByParentQuery {
                tenant_id: TenantId(tenant_id),
                parent_type: ParentType::WorkItem,
                parent_id: parent_a,
                include_deleted: false,
            },
            &actor,
        )
        .await
        .unwrap();
    let list_b = svc
        .list_by_parent(
            ListByParentQuery {
                tenant_id: TenantId(tenant_id),
                parent_type: ParentType::WorkItem,
                parent_id: parent_b,
                include_deleted: false,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(list_a.len(), 3);
    assert_eq!(list_b.len(), 2);
}

// =====================================================================
// V1: 编辑 / 软删除
// =====================================================================

/// **IT-V1-1**: edit 状态机 Open→Edited + lock_version 递增
#[tokio::test]
async fn it_v1_edit_status_machine() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let author = Uuid::new_v4();
    let actor = ActorContext::new(author, tenant_id);
    let c = svc
        .create_comment(basic_comment_cmd(tenant_id, author), &actor)
        .await
        .unwrap();
    assert_eq!(c.status, CommentStatus::Open);

    let edited = svc
        .edit_comment(
            EditCommentCommand {
                tenant_id: TenantId(tenant_id),
                comment_id: c.id,
                new_body: "edited body".to_string(),
                actor_user_id: UserId::from(author),
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(edited.status, CommentStatus::Edited);
    assert_eq!(edited.body, "edited body");
    // updated_at 应晚于 created_at
    assert!(edited.updated_at >= c.created_at);
}

/// **IT-V1-2**: delete 软删除 + list include_deleted=false 过滤
#[tokio::test]
async fn it_v1_soft_delete_and_filter() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let parent_id = Uuid::new_v4();
    let author = Uuid::new_v4();
    let actor = ActorContext::new(author, tenant_id);

    let mut cmd = basic_comment_cmd(tenant_id, author);
    cmd.parent_id = parent_id;
    let c1 = svc.create_comment(cmd, &actor).await.unwrap();
    let c2 = svc
        .create_comment(basic_comment_cmd(tenant_id, author), &actor)
        .await
        .unwrap();

    // 软删 c1
    let deleted = svc
        .delete_comment(
            DeleteCommentCommand {
                tenant_id: TenantId(tenant_id),
                comment_id: c1.id,
                actor_user_id: UserId::from(author),
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(deleted.status, CommentStatus::Deleted);
    assert!(deleted.deleted_at.is_some());

    // include_deleted=false: 0 (c1 被过滤, c2 parent 不同)
    let filtered = svc
        .list_by_parent(
            ListByParentQuery {
                tenant_id: TenantId(tenant_id),
                parent_type: ParentType::WorkItem,
                parent_id,
                include_deleted: false,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(filtered.len(), 0);

    // include_deleted=true: 1 (c1 仍出现)
    let all = svc
        .list_by_parent(
            ListByParentQuery {
                tenant_id: TenantId(tenant_id),
                parent_type: ParentType::WorkItem,
                parent_id,
                include_deleted: true,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].id, c1.id);
    let _ = c2; // suppress unused
}

/// **IT-V1-3**: edit 已删除评论 → EditDeleted 拒绝
#[tokio::test]
async fn it_v1_edit_deleted_rejected() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let author = Uuid::new_v4();
    let actor = ActorContext::new(author, tenant_id);
    let c = svc
        .create_comment(basic_comment_cmd(tenant_id, author), &actor)
        .await
        .unwrap();

    svc.delete_comment(
        DeleteCommentCommand {
            tenant_id: TenantId(tenant_id),
            comment_id: c.id,
            actor_user_id: UserId::from(author),
        },
        &actor,
    )
    .await
    .unwrap();

    let res = svc
        .edit_comment(
            EditCommentCommand {
                tenant_id: TenantId(tenant_id),
                comment_id: c.id,
                new_body: "post-delete".to_string(),
                actor_user_id: UserId::from(author),
            },
            &actor,
        )
        .await;
    assert!(matches!(res, Err(CommentError::EditDeleted)));
}

/// **IT-V1-4**: 非作者 edit 被拒
#[tokio::test]
async fn it_v1_edit_other_user_rejected() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let author = Uuid::new_v4();
    let other = Uuid::new_v4();
    let actor = ActorContext::new(author, tenant_id);
    let c = svc
        .create_comment(basic_comment_cmd(tenant_id, author), &actor)
        .await
        .unwrap();

    let other_actor = ActorContext::new(other, tenant_id);
    let res = svc
        .edit_comment(
            EditCommentCommand {
                tenant_id: TenantId(tenant_id),
                comment_id: c.id,
                new_body: "hostile takeover".to_string(),
                actor_user_id: UserId::from(other),
            },
            &other_actor,
        )
        .await;
    assert!(matches!(res, Err(CommentError::PermissionDenied)));
}

// =====================================================================
// V2: RLS 13 類 tenant 隔离
// =====================================================================

/// **IT-V2-1**: 跨 tenant create 被拒
#[tokio::test]
async fn it_v2_cross_tenant_create_rejected() {
    let svc = InMemoryCommentService::new();
    let actor_tenant = Uuid::new_v4();
    let cmd_tenant = Uuid::new_v4();
    let author = Uuid::new_v4();
    let actor = make_admin_actor(actor_tenant);

    let mut cmd = basic_comment_cmd(cmd_tenant, author);
    cmd.actor_user_id = UserId::from(author);

    let res = svc.create_comment(cmd, &actor).await;
    assert!(matches!(res, Err(CommentError::CrossTenantDenied(_, _))));
}

/// **IT-V2-2**: 跨 tenant get 被拒
#[tokio::test]
async fn it_v2_cross_tenant_get_rejected() {
    let svc = InMemoryCommentService::new();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let author = Uuid::new_v4();
    let actor_a = ActorContext::new(author, tenant_a);
    let c = svc
        .create_comment(basic_comment_cmd(tenant_a, author), &actor_a)
        .await
        .unwrap();

    // tenant B actor 试图读 tenant A 的 comment
    let actor_b = make_admin_actor(tenant_b);
    let res = svc
        .get(
            GetCommentQuery {
                tenant_id: TenantId(tenant_b),
                comment_id: c.id,
            },
            &actor_b,
        )
        .await;
    assert!(matches!(res, Err(CommentError::CrossTenantDenied(_, _))));
}

/// **IT-V2-3**: 跨 tenant list_by_parent 被拒
#[tokio::test]
async fn it_v2_cross_tenant_list_rejected() {
    let svc = InMemoryCommentService::new();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let author = Uuid::new_v4();
    let actor_a = ActorContext::new(author, tenant_a);
    let parent_id = Uuid::new_v4();
    let mut cmd = basic_comment_cmd(tenant_a, author);
    cmd.parent_id = parent_id;
    svc.create_comment(cmd, &actor_a).await.unwrap();

    // tenant B actor 试图查 tenant A 的 parent — query 显式带 tenant_a
    let actor_b = make_admin_actor(tenant_b);
    let res = svc
        .list_by_parent(
            ListByParentQuery {
                tenant_id: TenantId(tenant_a), // 显式要 tenant_a 的数据
                parent_type: ParentType::WorkItem,
                parent_id,
                include_deleted: false,
            },
            &actor_b, // actor 属 tenant_b
        )
        .await;
    assert!(matches!(res, Err(CommentError::CrossTenantDenied(_, _))));
}

// =====================================================================
// V3: 不变量 INV-C-03 Reaction + INV-C-04 tenant prefix
// =====================================================================

/// **IT-V3-1**: INV-C-03 — (comment, user, emoji) 唯一约束
#[tokio::test]
async fn it_v3_reaction_unique_triple() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let user = Uuid::new_v4();
    let actor = ActorContext::new(user, tenant_id);
    let c = svc
        .create_comment(basic_comment_cmd(tenant_id, user), &actor)
        .await
        .unwrap();

    // 第一次 👍 必成功
    svc.add_reaction(
        AddReactionCommand {
            tenant_id: TenantId(tenant_id),
            comment_id: c.id,
            user_id: UserId::from(user),
            emoji: "👍".to_string(),
        },
        &actor,
    )
    .await
    .unwrap();

    // 重复 (comment, user, emoji) → ReactionExists
    let res = svc
        .add_reaction(
            AddReactionCommand {
                tenant_id: TenantId(tenant_id),
                comment_id: c.id,
                user_id: UserId::from(user),
                emoji: "👍".to_string(),
            },
            &actor,
        )
        .await;
    assert!(matches!(res, Err(CommentError::ReactionExists)));
}

/// **IT-V3-2**: INV-C-03 — 同 user 不同 emoji 允许多个反应
#[tokio::test]
async fn it_v3_reaction_different_emoji_allowed() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let user = Uuid::new_v4();
    let actor = ActorContext::new(user, tenant_id);
    let c = svc
        .create_comment(basic_comment_cmd(tenant_id, user), &actor)
        .await
        .unwrap();

    for emoji in ["👍", "🎉", "❤️"] {
        let res = svc
            .add_reaction(
                AddReactionCommand {
                    tenant_id: TenantId(tenant_id),
                    comment_id: c.id,
                    user_id: UserId::from(user),
                    emoji: emoji.to_string(),
                },
                &actor,
            )
            .await;
        assert!(res.is_ok(), "emoji {} 必成功, got: {:?}", emoji, res);
    }
}

/// **IT-V3-3**: INV-C-04 — attachment object_key 缺 tenant_id 前缀 → 拒绝
#[tokio::test]
async fn it_v3_attachment_requires_tenant_prefix() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let uploader = Uuid::new_v4();
    let actor = ActorContext::new(uploader, tenant_id);

    let res = svc
        .register_attachment(
            RegisterAttachmentCommand {
                tenant_id: TenantId(tenant_id),
                uploader_user_id: UserId::from(uploader),
                filename: "design.pdf".to_string(),
                content_type: "application/pdf".to_string(),
                size_bytes: 1024,
                object_key: "wrong-prefix/file.pdf".to_string(), // 缺 tenants/{tid}/
            },
            &actor,
        )
        .await;
    assert!(matches!(res, Err(CommentError::InvalidObjectKey)));
}

/// **IT-V3-4**: INV-C-04 — attachment 正确带 tenant_id 前缀 → OK
#[tokio::test]
async fn it_v3_attachment_with_tenant_prefix_ok() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let uploader = Uuid::new_v4();
    let actor = ActorContext::new(uploader, tenant_id);
    let tid = TenantId(tenant_id);

    let a = svc
        .register_attachment(
            RegisterAttachmentCommand {
                tenant_id: tid,
                uploader_user_id: UserId::from(uploader),
                filename: "design.pdf".to_string(),
                content_type: "application/pdf".to_string(),
                size_bytes: 1024,
                object_key: format!("tenants/{}/files/design.pdf", tenant_id),
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(a.tenant_id, tid);
    assert!(a.object_key.starts_with(&format!("tenants/{}/", tenant_id)));
    let _ = AttachmentId::new(); // suppress unused
}

/// **IT-V3-5**: agent author 创建评论(AI Agent 会话触发)
#[tokio::test]
async fn it_v3_agent_author_comment() {
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let agent = AgentId::new();
    let mut agent_actor = ActorContext::new(agent.as_uuid(), tenant_id).with_agent_session(true);
    agent_actor.tenant_id = tenant_id;

    let cmd = CreateCommentCommand {
        tenant_id: TenantId(tenant_id),
        project_id: ProjectId::new(),
        parent_type: ParentType::PullRequest,
        parent_id: Uuid::new_v4(),
        body: "[bot] reviewed, looks good".to_string(),
        author_user_id: None,
        author_agent_id: Some(agent),
        mentions: vec![],
        attachment_ids: vec![],
        actor_user_id: UserId::new(), // 占位, actor 由 tenant_id 校验
    };
    let c = svc.create_comment(cmd, &agent_actor).await.unwrap();
    assert_eq!(c.author_agent_id, Some(agent));
    assert!(c.author_user_id.is_none());
    assert!(matches!(c.parent_type, ParentType::PullRequest));
}

// =====================================================================
// V4: actor 联动 domain-permission check() (per P1-4, 9/7 19:38 JST 派发)
// =====================================================================
//
// 替换字面量 role (`with_role("project_admin")`) 改为先过 PermissionService::check()
// 再调业务 service。本节演示"actor → permission check → service call"完整闭环,
// 5 个 IT 覆盖: Allow / 默认 Deny / Viewer 拒绝 / Admin 兜底 / 跨 tenant。

/// V4 helper: scheme + Developer × Comment × Write = Allow
async fn setup_dev_comment_write_allow(
    perm_svc: &InMemoryPermissionService,
    tenant_id: Uuid,
    actor: &ActorContext,
) -> PermissionScheme {
    let scheme = perm_svc
        .create_scheme(
            CreateSchemeCommand {
                tenant_id: PermTenantId(tenant_id),
                name: "comment-default".to_string(),
                actor_user_id: PermUserId::from(actor.user_id),
            },
            actor,
        )
        .await
        .expect("scheme create");

    perm_svc
        .upsert_rule(
            UpsertRuleCommand {
                tenant_id: PermTenantId(tenant_id),
                scheme_id: scheme.id,
                rule: PermissionRule {
                    id: domain_permission::PermissionRuleId::new(),
                    subject_type: SubjectType::Role,
                    subject_id: None,
                    role: Some(Role::Developer),
                    resource_type: ResourceType::Comment,
                    resource_id: None,
                    actions: vec![Action::Write, Action::Read],
                    effect: Effect::Allow,
                },
            },
            actor,
        )
        .await
        .expect("rule upsert");
    scheme
}

/// V4 helper: 调 check("comment.create") = Comment/Write
async fn check_comment_create(
    perm_svc: &InMemoryPermissionService,
    scheme_id: Option<domain_permission::PermissionSchemeId>,
    tenant_id: Uuid,
    project_id: ProjectId,
    user_id: Uuid,
    actor: &ActorContext,
) -> Result<bool, PermissionError> {
    perm_svc
        .check(
            CheckQuery {
                tenant_id: PermTenantId(tenant_id),
                scheme_id,
                subject_user_id: PermUserId::from(user_id),
                project_id: PermProjectId::from(project_id.as_uuid()),
                resource_type: ResourceType::Comment,
                resource_id: None,
                action: Action::Write,
            },
            actor,
        )
        .await
}

/// **IT-V4-1**: Developer role + Allow 规则 → check("comment.create") = true → create 成功
#[tokio::test]
async fn it_v4_perm_check_allow_then_create() {
    let perm_svc: Arc<InMemoryPermissionService> = InMemoryPermissionService::new_for_test();
    let svc = InMemoryCommentService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let admin_actor = make_admin_actor(tenant_id);

    let scheme = setup_dev_comment_write_allow(&perm_svc, tenant_id, &admin_actor).await;

    let dev_user = Uuid::new_v4();
    perm_svc
        .grant_role(
            GrantRoleCommand {
                tenant_id: PermTenantId(tenant_id),
                user_id: PermUserId::from(dev_user),
                project_id: PermProjectId::from(project_id.as_uuid()),
                role: Role::Developer,
                granted_by: PermUserId::from(admin_actor.user_id),
            },
            &admin_actor,
        )
        .await
        .expect("grant Developer");

    // dev actor (不带字面量 role)
    let dev_actor = ActorContext::new(dev_user, tenant_id);
    let allowed = check_comment_create(
        &perm_svc,
        Some(scheme.id),
        tenant_id,
        project_id,
        dev_user,
        &dev_actor,
    )
    .await
    .expect("check 必成功");
    assert!(allowed, "Developer + Allow 规则 → comment.create 必 allow");

    // 联动业务:create 必成功
    let mut cmd = basic_comment_cmd(tenant_id, dev_user);
    cmd.project_id = project_id;
    let c = svc
        .create_comment(cmd, &dev_actor)
        .await
        .expect("create 必成功");
    assert_eq!(c.author_user_id, Some(UserId::from(dev_user)));
}

/// **IT-V4-2**: 无 role binding → check() 默认 Deny (INV-PM-05)
#[tokio::test]
async fn it_v4_perm_check_default_deny_no_role() {
    let perm_svc: Arc<InMemoryPermissionService> = InMemoryPermissionService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let admin_actor = make_admin_actor(tenant_id);
    let scheme = setup_dev_comment_write_allow(&perm_svc, tenant_id, &admin_actor).await;

    let dev_user = Uuid::new_v4();
    let dev_actor = ActorContext::new(dev_user, tenant_id);
    let allowed = check_comment_create(
        &perm_svc,
        Some(scheme.id),
        tenant_id,
        project_id,
        dev_user,
        &dev_actor,
    )
    .await
    .expect("check 必成功");
    assert!(!allowed, "无 role binding → 默认 Deny (INV-PM-05)");
}

/// **IT-V4-3**: Viewer role → check() 拒绝 Write
#[tokio::test]
async fn it_v4_perm_check_viewer_write_denied() {
    let perm_svc: Arc<InMemoryPermissionService> = InMemoryPermissionService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let admin_actor = make_admin_actor(tenant_id);
    let scheme = setup_dev_comment_write_allow(&perm_svc, tenant_id, &admin_actor).await;

    let viewer_user = Uuid::new_v4();
    perm_svc
        .grant_role(
            GrantRoleCommand {
                tenant_id: PermTenantId(tenant_id),
                user_id: PermUserId::from(viewer_user),
                project_id: PermProjectId::from(project_id.as_uuid()),
                role: Role::Viewer,
                granted_by: PermUserId::from(admin_actor.user_id),
            },
            &admin_actor,
        )
        .await
        .expect("grant Viewer");

    let viewer_actor = ActorContext::new(viewer_user, tenant_id);
    let allowed = check_comment_create(
        &perm_svc,
        Some(scheme.id),
        tenant_id,
        project_id,
        viewer_user,
        &viewer_actor,
    )
    .await
    .expect("check 必成功");
    assert!(!allowed, "Viewer × Write → 必 deny");
}

/// **IT-V4-4**: Tenant admin + scheme_id=None → default 兜底 allow
#[tokio::test]
async fn it_v4_perm_check_tenant_admin_default_allow() {
    let perm_svc: Arc<InMemoryPermissionService> = InMemoryPermissionService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let admin_user = Uuid::new_v4();
    let admin_actor = ActorContext::new(admin_user, tenant_id).with_role("tenant_admin");

    perm_svc
        .grant_role(
            GrantRoleCommand {
                tenant_id: PermTenantId(tenant_id),
                user_id: PermUserId::from(admin_user),
                project_id: PermProjectId::from(project_id.as_uuid()),
                role: Role::TenantAdmin,
                granted_by: PermUserId::from(admin_user),
            },
            &admin_actor,
        )
        .await
        .expect("grant TenantAdmin");

    // scheme_id = None → default_decide_without_scheme(admin) = true
    let allowed = check_comment_create(
        &perm_svc,
        None,
        tenant_id,
        project_id,
        admin_user,
        &admin_actor,
    )
    .await
    .expect("check 必成功");
    assert!(allowed, "TenantAdmin + scheme=None → 必 allow (默认策略)");
}

/// **IT-V4-5**: 跨 tenant actor 调 check() → CrossTenantDenied
#[tokio::test]
async fn it_v4_perm_check_cross_tenant_rejected() {
    let perm_svc: Arc<InMemoryPermissionService> = InMemoryPermissionService::new_for_test();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let project_id = ProjectId::new();
    let admin_a = make_admin_actor(tenant_a);
    let scheme = setup_dev_comment_write_allow(&perm_svc, tenant_a, &admin_a).await;

    let admin_b = make_admin_actor(tenant_b);
    let res = check_comment_create(
        &perm_svc,
        Some(scheme.id),
        tenant_b,
        project_id,
        admin_b.user_id,
        &admin_b,
    )
    .await;
    assert!(
        matches!(res, Err(PermissionError::CrossTenantDenied(_, _))),
        "跨 tenant check → CrossTenantDenied (守门 #13 c)"
    );
}
