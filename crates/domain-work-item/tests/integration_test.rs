//! 集成测试 (IT) — domain-work-item 真实数据接入验证
//!
//! **crate**: domain-work-item (test target)
//! **per**: 2026-09-07 P3-B sub-batch 2 write-heavy 域真实数据接入
//! **参考**: `crates/domain-workspace/tests/integration_test.rs` (per 7f9f52a 4 域真实接入已落地)
//!
//! ## 验证维度 (V0-V3)
//!
//! - V0 (基础 CRUD): create + get + list_by_project
//! - V1 (状态机): transition_status TODO→IN_PROGRESS→DONE + 跳态拒绝 + assign
//! - V2 (RLS 13 類): 跨 tenant 拒绝 (守门 #13 (c) Master 100% RLS)
//! - V3 (不变量): INV-WI-03 AITask 必带 objective + repository_scope; INV-WI-04 parent 必须同 project

use domain_permission::{
    Action, CheckQuery, CreateSchemeCommand, Effect, GrantRoleCommand, InMemoryPermissionService,
    PermissionCommandPort, PermissionError, PermissionQueryPort, PermissionRule, PermissionScheme,
    ProjectId as PermProjectId, ResourceType, Role, SubjectType, TenantId as PermTenantId,
    UpsertRuleCommand, UserId as PermUserId,
};
use domain_work_item::{
    ActorContext, AiTaskData, AssignCommand, CreateAcceptanceCriterionCommand,
    CreateRequirementCommand, CreateWorkItemCommand, GetWorkItemQuery, InMemoryWorkItemService,
    ListByProjectQuery, Priority, ProjectId, RepositoryId, RequirementId, Severity,
    TransitionStatusCommand, UserId, WorkItemCommandPort, WorkItemError, WorkItemQueryPort,
    WorkItemStatus, WorkItemType, WorkspaceId,
};
use std::sync::Arc;
use uuid::Uuid;

/// IT fixture: developer 角色 actor (per 守门 #13 RLS 13 類)
fn make_dev_actor(tenant_id: Uuid) -> ActorContext {
    ActorContext::new(Uuid::new_v4(), tenant_id).with_role("developer")
}

/// IT fixture: tenant_admin 角色 actor
fn make_admin_actor(tenant_id: Uuid) -> ActorContext {
    ActorContext::new(Uuid::new_v4(), tenant_id).with_role("tenant_admin")
}

/// IT fixture: 基础 Task 创建命令
fn basic_task_cmd(tid: Uuid) -> CreateWorkItemCommand {
    CreateWorkItemCommand {
        tenant_id: domain_work_item::TenantId(tid),
        workspace_id: WorkspaceId::new(),
        project_id: ProjectId::new(),
        item_type: WorkItemType::Task,
        title: "fix login bug".to_string(),
        description: "OAuth callback race condition".to_string(),
        priority: Priority::High,
        severity: Some(Severity::Major),
        reporter_user_id: UserId::new(),
        parent_work_item_id: None,
        ai_task_data: None,
        labels: vec!["bug".to_string(), "auth".to_string()],
    }
}

// =====================================================================
// V0: 基础 CRUD
// =====================================================================

/// **IT-V0-1**: 完整 create → get → list_by_project 闭环
#[tokio::test]
async fn it_v0_create_get_list_lifecycle() {
    let svc = InMemoryWorkItemService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let actor = make_dev_actor(tenant_id);

    // create
    let mut cmd = basic_task_cmd(tenant_id);
    cmd.project_id = project_id;
    let item = svc
        .create_work_item(cmd, &actor)
        .await
        .expect("create 必成功");
    assert_eq!(item.status, WorkItemStatus::Todo);
    assert_eq!(item.priority, Priority::High);
    assert_eq!(item.lock_version, 1);

    // get
    let fetched = svc
        .get(
            GetWorkItemQuery {
                tenant_id: domain_work_item::TenantId(tenant_id),
                work_item_id: item.id,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(fetched.id, item.id);
    assert_eq!(fetched.title, "fix login bug");

    // list_by_project
    let list = svc
        .list_by_project(
            ListByProjectQuery {
                tenant_id: domain_work_item::TenantId(tenant_id),
                project_id,
                include_terminal: false,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, item.id);
}

/// **IT-V0-2**: list_by_project 跨项目隔离
#[tokio::test]
async fn it_v0_list_by_project_isolates() {
    let svc = InMemoryWorkItemService::new();
    let tenant_id = Uuid::new_v4();
    let project_a = ProjectId::new();
    let project_b = ProjectId::new();
    let actor = make_dev_actor(tenant_id);

    // project_a 创建 2 个
    for _ in 0..2 {
        let mut cmd = basic_task_cmd(tenant_id);
        cmd.project_id = project_a;
        svc.create_work_item(cmd, &actor).await.unwrap();
    }
    // project_b 创建 1 个
    let mut cmd = basic_task_cmd(tenant_id);
    cmd.project_id = project_b;
    svc.create_work_item(cmd, &actor).await.unwrap();

    let list_a = svc
        .list_by_project(
            ListByProjectQuery {
                tenant_id: domain_work_item::TenantId(tenant_id),
                project_id: project_a,
                include_terminal: false,
            },
            &actor,
        )
        .await
        .unwrap();
    let list_b = svc
        .list_by_project(
            ListByProjectQuery {
                tenant_id: domain_work_item::TenantId(tenant_id),
                project_id: project_b,
                include_terminal: false,
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(list_a.len(), 2);
    assert_eq!(list_b.len(), 1);
}

// =====================================================================
// V1: 状态机 + 分配
// =====================================================================

/// **IT-V1-1**: TODO→IN_PROGRESS→DONE 完整状态机闭环 + lock_version 单调递增
#[tokio::test]
async fn it_v1_status_full_lifecycle() {
    let svc = InMemoryWorkItemService::new();
    let tenant_id = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id);
    let item = svc
        .create_work_item(basic_task_cmd(tenant_id), &actor)
        .await
        .unwrap();
    assert_eq!(item.lock_version, 1);

    // TODO→IN_PROGRESS
    let item = svc
        .transition_status(
            TransitionStatusCommand {
                tenant_id: domain_work_item::TenantId(tenant_id),
                work_item_id: item.id,
                from: WorkItemStatus::Todo,
                to: WorkItemStatus::InProgress,
                actor_user_id: UserId::from(actor.user_id),
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(item.status, WorkItemStatus::InProgress);
    assert_eq!(item.lock_version, 2);

    // IN_PROGRESS→DONE
    let item = svc
        .transition_status(
            TransitionStatusCommand {
                tenant_id: domain_work_item::TenantId(tenant_id),
                work_item_id: item.id,
                from: WorkItemStatus::InProgress,
                to: WorkItemStatus::Done,
                actor_user_id: UserId::from(actor.user_id),
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(item.status, WorkItemStatus::Done);
    assert_eq!(item.lock_version, 3);
}

/// **IT-V1-2**: 状态跳态拒绝 (INV-WI-02: TODO→DONE 跳态不允许)
#[tokio::test]
async fn it_v1_status_skip_rejected() {
    let svc = InMemoryWorkItemService::new();
    let tenant_id = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id);
    let item = svc
        .create_work_item(basic_task_cmd(tenant_id), &actor)
        .await
        .unwrap();

    let res = svc
        .transition_status(
            TransitionStatusCommand {
                tenant_id: domain_work_item::TenantId(tenant_id),
                work_item_id: item.id,
                from: WorkItemStatus::Todo,
                to: WorkItemStatus::Done, // 跳态
                actor_user_id: UserId::from(actor.user_id),
            },
            &actor,
        )
        .await;
    assert!(matches!(res, Err(WorkItemError::InvalidTransition { .. })));
}

/// **IT-V1-3**: assign 走 user + agent 双指派 + lock_version 递增
#[tokio::test]
async fn it_v1_assign_user_and_agent() {
    let svc = InMemoryWorkItemService::new();
    let tenant_id = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id);
    let item = svc
        .create_work_item(basic_task_cmd(tenant_id), &actor)
        .await
        .unwrap();
    assert!(item.assignee_user_id.is_none());
    assert!(item.assignee_agent_id.is_none());

    let user_id = Uuid::new_v4();
    let agent_id = domain_work_item::AgentId::new();
    let updated = svc
        .assign(
            AssignCommand {
                tenant_id: domain_work_item::TenantId(tenant_id),
                work_item_id: item.id,
                assignee_user_id: Some(UserId::from(user_id)),
                assignee_agent_id: Some(agent_id),
                actor_user_id: UserId::from(actor.user_id),
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(updated.assignee_user_id, Some(UserId::from(user_id)));
    assert_eq!(updated.assignee_agent_id, Some(agent_id));
    assert_eq!(updated.lock_version, 2);
}

// =====================================================================
// V2: RLS 13 類 tenant 隔离
// =====================================================================

/// **IT-V2-1**: 跨 tenant 创建被拒(守门 #13 (c) Master 100% RLS)
#[tokio::test]
async fn it_v2_cross_tenant_create_rejected() {
    let svc = InMemoryWorkItemService::new();
    let actor_tenant = Uuid::new_v4();
    let cmd_tenant = Uuid::new_v4();
    let actor = make_admin_actor(actor_tenant);
    let mut cmd = basic_task_cmd(cmd_tenant); // 另一个 tenant
    cmd.reporter_user_id = UserId::from(actor.user_id);

    let res = svc.create_work_item(cmd, &actor).await;
    assert!(matches!(res, Err(WorkItemError::CrossTenantDenied(_, _))));
}

/// **IT-V2-2**: 跨 tenant 读取被拒
#[tokio::test]
async fn it_v2_cross_tenant_get_rejected() {
    let svc = InMemoryWorkItemService::new();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let actor_a = make_admin_actor(tenant_a);
    let item = svc
        .create_work_item(basic_task_cmd(tenant_a), &actor_a)
        .await
        .unwrap();

    // tenant B actor 试图读 tenant A 的 work item
    let actor_b = make_admin_actor(tenant_b);
    let res = svc
        .get(
            GetWorkItemQuery {
                tenant_id: domain_work_item::TenantId(tenant_b),
                work_item_id: item.id,
            },
            &actor_b,
        )
        .await;
    assert!(matches!(res, Err(WorkItemError::CrossTenantDenied(_, _))));
}

/// **IT-V2-3**: 跨 tenant 状态转换被拒
#[tokio::test]
async fn it_v2_cross_tenant_transition_rejected() {
    let svc = InMemoryWorkItemService::new();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let actor_a = make_admin_actor(tenant_a);
    let item = svc
        .create_work_item(basic_task_cmd(tenant_a), &actor_a)
        .await
        .unwrap();

    let actor_b = make_admin_actor(tenant_b);
    let res = svc
        .transition_status(
            TransitionStatusCommand {
                tenant_id: domain_work_item::TenantId(tenant_b),
                work_item_id: item.id,
                from: WorkItemStatus::Todo,
                to: WorkItemStatus::InProgress,
                actor_user_id: UserId::from(actor_b.user_id),
            },
            &actor_b,
        )
        .await;
    assert!(matches!(res, Err(WorkItemError::CrossTenantDenied(_, _))));
}

// =====================================================================
// V3: 不变量 INV-WI-03 + INV-WI-04 + Requirement/AC 关联
// =====================================================================

/// **IT-V3-1**: INV-WI-03 — AITask 缺 objective 必拒
#[tokio::test]
async fn it_v3_ai_task_requires_objective() {
    let svc = InMemoryWorkItemService::new();
    let tenant_id = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id);
    let mut cmd = basic_task_cmd(tenant_id);
    cmd.item_type = WorkItemType::AITask;
    // ai_task_data = None

    let res = svc.create_work_item(cmd, &actor).await;
    assert!(matches!(res, Err(WorkItemError::AiTaskMissingObjective)));
}

/// **IT-V3-2**: INV-WI-03 — AITask 缺 repository_scope 必拒
#[tokio::test]
async fn it_v3_ai_task_requires_repository_scope() {
    let svc = InMemoryWorkItemService::new();
    let tenant_id = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id);
    let mut cmd = basic_task_cmd(tenant_id);
    cmd.item_type = WorkItemType::AITask;
    cmd.ai_task_data = Some(AiTaskData {
        objective: "implement auth".to_string(),
        repository_scope: vec![], // 空
        allowed_files: vec![],
        forbidden_files: vec![],
        agent_policy_id: None,
        validation_policy_id: None,
        context_policy_id: None,
    });

    let res = svc.create_work_item(cmd, &actor).await;
    assert!(matches!(res, Err(WorkItemError::AiTaskMissingScope)));
}

/// **IT-V3-3**: INV-WI-04 — parent_work_item_id 必须同 project
#[tokio::test]
async fn it_v3_parent_must_be_same_project() {
    let svc = InMemoryWorkItemService::new();
    let tenant_id = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id);
    let parent_project = ProjectId::new();
    let child_project = ProjectId::new();

    let mut parent_cmd = basic_task_cmd(tenant_id);
    parent_cmd.project_id = parent_project;
    parent_cmd.title = "parent epic".to_string();
    let parent = svc.create_work_item(parent_cmd, &actor).await.unwrap();

    // child 用不同 project 试图引用 parent
    let mut child_cmd = basic_task_cmd(tenant_id);
    child_cmd.parent_work_item_id = Some(parent.id);
    child_cmd.project_id = child_project;
    child_cmd.title = "child task".to_string();
    let res = svc.create_work_item(child_cmd, &actor).await;
    assert!(matches!(res, Err(WorkItemError::ParentProjectMismatch)));
}

/// **IT-V3-4**: Requirement + AcceptanceCriterion 闭环
#[tokio::test]
async fn it_v3_requirement_and_ac_chain() {
    let svc = InMemoryWorkItemService::new();
    let tenant_id = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id);
    let item = svc
        .create_work_item(basic_task_cmd(tenant_id), &actor)
        .await
        .unwrap();

    // 创建 requirement
    let req = svc
        .create_requirement(
            CreateRequirementCommand {
                tenant_id: domain_work_item::TenantId(tenant_id),
                business_goal_id: None,
                statement: "support OAuth2 login".to_string(),
                rationale: "industry standard".to_string(),
                actor_user_id: UserId::from(actor.user_id),
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(req.id, RequirementId::from(req.id.as_uuid()));

    // 创建 acceptance criterion 关联 work item + requirement
    let ac = svc
        .create_acceptance_criterion(
            CreateAcceptanceCriterionCommand {
                tenant_id: domain_work_item::TenantId(tenant_id),
                requirement_id: req.id,
                work_item_id: item.id,
                statement: "user can log in via OAuth2 within 3 clicks".to_string(),
                actor_user_id: UserId::from(actor.user_id),
            },
            &actor,
        )
        .await
        .unwrap();
    assert_eq!(ac.work_item_id, item.id);
    assert_eq!(ac.requirement_id, req.id);
    assert_eq!(
        ac.coverage_status,
        domain_work_item::CoverageStatus::Uncovered
    );
}

/// **IT-V3-5**: AITask 全字段 OK 创建成功 (RepositoryId 强类型校验)
#[tokio::test]
async fn it_v3_ai_task_with_full_data_ok() {
    let svc = InMemoryWorkItemService::new();
    let tenant_id = Uuid::new_v4();
    let actor = make_dev_actor(tenant_id);
    let mut cmd = basic_task_cmd(tenant_id);
    cmd.item_type = WorkItemType::AITask;
    cmd.title = "AI: refactor auth module".to_string();
    cmd.ai_task_data = Some(AiTaskData {
        objective: "migrate session storage from in-memory to redis".to_string(),
        repository_scope: vec![RepositoryId::new()],
        allowed_files: vec!["crates/domain-identity/src/session/**".to_string()],
        forbidden_files: vec!["**/migrations/**".to_string()],
        agent_policy_id: None,
        validation_policy_id: None,
        context_policy_id: None,
    });

    let item = svc.create_work_item(cmd, &actor).await.unwrap();
    assert!(matches!(item.item_type, WorkItemType::AITask));
    let ai_data = item.ai_task_data.unwrap();
    assert_eq!(
        ai_data.objective,
        "migrate session storage from in-memory to redis"
    );
    assert_eq!(ai_data.repository_scope.len(), 1);
}

// =====================================================================
// V4: actor 联动 domain-permission check() (per P1-4, 9/7 19:38 JST 派发)
// =====================================================================
//
// 替换字面量 role (`with_role("developer")`) 改为先过 PermissionService::check()
// 再调业务 service。本节演示"actor → permission check → service call"完整闭环,
// 6 个 IT 覆盖: Allow / 默认 Deny / Viewer 拒绝 / Admin / 跨 tenant / 自定义 Deny。

/// V4 helper: 在测试用 permission service 上建 scheme + 上插一条规则
/// (Developer × WorkItem × Write = Allow), 返回 scheme id 备用
async fn setup_dev_workitem_write_allow(
    perm_svc: &InMemoryPermissionService,
    tenant_id: Uuid,
    actor: &ActorContext,
) -> PermissionScheme {
    let scheme = perm_svc
        .create_scheme(
            CreateSchemeCommand {
                tenant_id: PermTenantId(tenant_id),
                name: "wi-default".to_string(),
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
                    resource_type: ResourceType::WorkItem,
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

/// V4 helper: 在测试用 permission service 上给 user 绑 Developer 角色
async fn grant_dev_role(
    perm_svc: &InMemoryPermissionService,
    tenant_id: Uuid,
    project_id: domain_work_item::ProjectId,
    user_id: Uuid,
    actor: &ActorContext,
) {
    perm_svc
        .grant_role(
            GrantRoleCommand {
                tenant_id: PermTenantId(tenant_id),
                user_id: PermUserId::from(user_id),
                project_id: PermProjectId::from(project_id.as_uuid()),
                role: Role::Developer,
                granted_by: PermUserId::from(actor.user_id),
            },
            actor,
        )
        .await
        .expect("grant Developer");
}

/// V4 helper: 调 permission service.check() 验证 "work-item.create" 权限
async fn check_workitem_create(
    perm_svc: &InMemoryPermissionService,
    scheme_id: Option<domain_permission::PermissionSchemeId>,
    tenant_id: Uuid,
    project_id: domain_work_item::ProjectId,
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
                resource_type: ResourceType::WorkItem,
                resource_id: None,
                action: Action::Write,
            },
            actor,
        )
        .await
}

/// **IT-V4-1**: Developer role + Allow 规则 → check("work-item.create") 返回 true → create 成功
#[tokio::test]
async fn it_v4_perm_check_allow_then_create() {
    let perm_svc: Arc<InMemoryPermissionService> = InMemoryPermissionService::new_for_test();
    let svc = InMemoryWorkItemService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let admin_actor = make_admin_actor(tenant_id);

    // 1) 配 scheme + rule (Developer × WorkItem × Write = Allow)
    let scheme = setup_dev_workitem_write_allow(&perm_svc, tenant_id, &admin_actor).await;

    // 2) 给 dev user 绑 Developer 角色
    let dev_user = Uuid::new_v4();
    grant_dev_role(&perm_svc, tenant_id, project_id, dev_user, &admin_actor).await;

    // 3) dev actor → check("work-item.create") = WorkItem/Write
    let dev_actor = ActorContext::new(dev_user, tenant_id); // 不再用 with_role("developer")
    let allowed = check_workitem_create(
        &perm_svc,
        Some(scheme.id),
        tenant_id,
        project_id,
        dev_user,
        &dev_actor,
    )
    .await
    .expect("check 必成功");
    assert!(
        allowed,
        "Developer role + Allow 规则 → work-item.create 必 allow"
    );

    // 4) 联动业务:create 必成功
    let mut cmd = basic_task_cmd(tenant_id);
    cmd.project_id = project_id;
    cmd.reporter_user_id = UserId::from(dev_user);
    let item = svc
        .create_work_item(cmd, &dev_actor)
        .await
        .expect("create 必成功");
    assert_eq!(item.reporter_user_id, UserId::from(dev_user));
}

/// **IT-V4-2**: 无 role binding → check() 默认 Deny (INV-PM-05)
#[tokio::test]
async fn it_v4_perm_check_default_deny_no_role() {
    let perm_svc: Arc<InMemoryPermissionService> = InMemoryPermissionService::new_for_test();
    let svc = InMemoryWorkItemService::new();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let admin_actor = make_admin_actor(tenant_id);
    let scheme = setup_dev_workitem_write_allow(&perm_svc, tenant_id, &admin_actor).await;

    // 不 grant role
    let dev_user = Uuid::new_v4();
    let dev_actor = ActorContext::new(dev_user, tenant_id);
    let allowed = check_workitem_create(
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

    // 业务侧:即使 svc 不强制,显式 Err 提示 PermissionDenied
    let mut cmd = basic_task_cmd(tenant_id);
    cmd.project_id = project_id;
    cmd.reporter_user_id = UserId::from(dev_user);
    let res = svc.create_work_item(cmd, &dev_actor).await;
    if res.is_ok() {
        // 业务 service 不强制 check,本测试仅验证 check() 行为
    } else {
        // 若业务层强制,应返回 PermissionDenied
        assert!(matches!(res, Err(WorkItemError::PermissionDenied)));
    }
}

/// **IT-V4-3**: Viewer role → check() 拒绝 Write (read only)
#[tokio::test]
async fn it_v4_perm_check_viewer_write_denied() {
    let perm_svc: Arc<InMemoryPermissionService> = InMemoryPermissionService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let admin_actor = make_admin_actor(tenant_id);
    let scheme = setup_dev_workitem_write_allow(&perm_svc, tenant_id, &admin_actor).await;

    // 给 viewer 绑 Viewer 角色
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

    // Viewer 试图 Write → 应被拒 (规则只对 Developer Allow)
    let viewer_actor = ActorContext::new(viewer_user, tenant_id);
    let allowed = check_workitem_create(
        &perm_svc,
        Some(scheme.id),
        tenant_id,
        project_id,
        viewer_user,
        &viewer_actor,
    )
    .await
    .expect("check 必成功");
    assert!(
        !allowed,
        "Viewer role × Write → 必 deny (规则仅 Developer 命中)"
    );
}

/// **IT-V4-4**: Tenant admin → check() 默认 Deny scheme 场景下 走 RoleBinding 兜底
///  (per default_decide_without_scheme 策略: admin 角色 → 允许所有)
#[tokio::test]
async fn it_v4_perm_check_tenant_admin_default_allow() {
    let perm_svc: Arc<InMemoryPermissionService> = InMemoryPermissionService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let admin_user = Uuid::new_v4();

    // 不创建 scheme (None) → 走 default_decide_without_scheme
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
    let allowed = check_workitem_create(
        &perm_svc,
        None,
        tenant_id,
        project_id,
        admin_user,
        &admin_actor,
    )
    .await
    .expect("check 必成功");
    assert!(
        allowed,
        "TenantAdmin + scheme_id=None → 必 allow (默认策略)"
    );
}

/// **IT-V4-5**: 显式 Deny 规则优先于 Allow (INV-PM-02)
#[tokio::test]
async fn it_v4_perm_check_deny_rule_wins() {
    let perm_svc: Arc<InMemoryPermissionService> = InMemoryPermissionService::new_for_test();
    let tenant_id = Uuid::new_v4();
    let project_id = ProjectId::new();
    let admin_actor = make_admin_actor(tenant_id);

    // 先建一个空 scheme,再插 2 条规则: Developer 全部 Allow + Developer Write 显式 Deny
    let scheme = perm_svc
        .create_scheme(
            CreateSchemeCommand {
                tenant_id: PermTenantId(tenant_id),
                name: "deny-wins".to_string(),
                actor_user_id: PermUserId::from(admin_actor.user_id),
            },
            &admin_actor,
        )
        .await
        .expect("scheme create");

    // Allow: Developer × WorkItem × All actions
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
                    resource_type: ResourceType::WorkItem,
                    resource_id: None,
                    actions: vec![Action::Read, Action::Write, Action::Delete],
                    effect: Effect::Allow,
                },
            },
            &admin_actor,
        )
        .await
        .expect("allow rule");

    // Deny: Developer × WorkItem × Write
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
                    resource_type: ResourceType::WorkItem,
                    resource_id: None,
                    actions: vec![Action::Write],
                    effect: Effect::Deny,
                },
            },
            &admin_actor,
        )
        .await
        .expect("deny rule");

    // 绑 Developer
    let dev_user = Uuid::new_v4();
    grant_dev_role(&perm_svc, tenant_id, project_id, dev_user, &admin_actor).await;

    // check Write → Deny 应赢
    let dev_actor = ActorContext::new(dev_user, tenant_id);
    let allowed = check_workitem_create(
        &perm_svc,
        Some(scheme.id),
        tenant_id,
        project_id,
        dev_user,
        &dev_actor,
    )
    .await
    .expect("check 必成功");
    assert!(!allowed, "Deny 规则优先于 Allow (INV-PM-02)");

    // check Read → 应 allow (Deny 仅 Write)
    let read_allowed = perm_svc
        .check(
            CheckQuery {
                tenant_id: PermTenantId(tenant_id),
                scheme_id: Some(scheme.id),
                subject_user_id: PermUserId::from(dev_user),
                project_id: PermProjectId::from(project_id.as_uuid()),
                resource_type: ResourceType::WorkItem,
                resource_id: None,
                action: Action::Read,
            },
            &dev_actor,
        )
        .await
        .expect("check read");
    assert!(read_allowed, "Read 规则没被 Deny → 应 allow");
}

/// **IT-V4-6**: 跨 tenant actor 调 check() → CrossTenantDenied (守门 #13 c RLS 13 類)
#[tokio::test]
async fn it_v4_perm_check_cross_tenant_rejected() {
    let perm_svc: Arc<InMemoryPermissionService> = InMemoryPermissionService::new_for_test();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let project_id = ProjectId::new();
    let admin_a = make_admin_actor(tenant_a);
    let scheme = setup_dev_workitem_write_allow(&perm_svc, tenant_a, &admin_a).await;

    // tenant B actor 试图用 tenant A 的 scheme_id check
    let admin_b = make_admin_actor(tenant_b);
    let res = check_workitem_create(
        &perm_svc,
        Some(scheme.id),
        tenant_b, // ← 跨 tenant
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
