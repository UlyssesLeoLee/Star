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

use domain_work_item::{
    ActorContext, AiTaskData, AssignCommand, CreateAcceptanceCriterionCommand,
    CreateRequirementCommand, CreateWorkItemCommand, GetWorkItemQuery, InMemoryWorkItemService,
    ListByProjectQuery, Priority, ProjectId, RepositoryId, RequirementId, Severity,
    TransitionStatusCommand, UserId, WorkItemCommandPort, WorkItemError, WorkItemQueryPort,
    WorkItemStatus, WorkItemType, WorkspaceId,
};
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
    let item = svc.create_work_item(cmd, &actor).await.expect("create 必成功");
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
    assert_eq!(ac.coverage_status, domain_work_item::CoverageStatus::Uncovered);
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
    assert_eq!(ai_data.objective, "migrate session storage from in-memory to redis");
    assert_eq!(ai_data.repository_scope.len(), 1);
}
