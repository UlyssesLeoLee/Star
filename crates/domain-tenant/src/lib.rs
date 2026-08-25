//! Tenant 最高安全边界
//!
//! **crate**: `domain-tenant`
//! **上游 spec**: docs/specs/domain-tenant-spec.md §4.10.2 / §6.1 (13 类对象 1-2)
//! **基本设计**: docs/basic-design.md §2.1(表 18) / §4.10.2 / §5.7
//! **数据设计**: docs/data-design.md §4.1 (`tenant` / `tenant_policy` / `tenant_quota` schema) / §7 (RLS)
//! **API 设计**: docs/api-design.md §3.2 (domain-tenant 端点) / §5.5 / §8.3.7
//!
//! ## 职责
//!
//! Tenant 是 §6.1 列出的 13 类对象之首,本 crate 负责:
//! - 颁发 `tenant_id`(UUIDv4,§5.7,security-design §4.1)
//! - 定义 3 个核心实体(`Tenant` / `TenantPolicy` / `TenantQuota`)
//! - 3 个核心 Domain Event(CloudEvents 1.0)
//! - 2 个端口(`TenantCommandPort` × 4 方法 / `TenantQueryPort` × 5 方法) + 1 个仓库端口
//! - 3 条不变量检查(INV-TEN-01~02 + INV-AUX-01)
//! - 1 个 `InMemoryTenantService` 真实实现
//!
//! ## 关键不变量
//!
//! - 任何聚合根 INSERT/UPDATE 必须携带 tenant_id(§6.1,REQ-SEC-001)
//! - tenant_id 由本 crate 颁发(UUIDv4),不可调用方传入(§5.7,security-design §4.1)
//! - 跨 tenant 访问返回 SEC-007 + Audit(security-design §3.5.4)
//! - `tenant_key` 平台内全局唯一(INV-TEN-01)
//! - Tenant 状态机迁移合法(INV-TEN-02)
//!
//! ## 上游依赖(basic-design §2.3)
//!
//! 本 crate 仅依赖 `crates/domain-tenant` 自身的外部 crate 依赖
//! (serde / uuid / chrono / async-trait / thiserror / tokio)。
//!
//! **禁止反向依赖** 任何其他 `domain-*` crate
//! (由 `crates/application` 或 `crates/infrastructure` 在适配层组合)。
//!
//! ## 关键引用
//!
//! 13 类 tenant_id 对象(§6.1,§4.10.4,已修复 F-06):本 crate 颁发 tenant_id

#![allow(missing_docs)]
#![warn(rust_2018_idioms)]

// =====================================================================
// 子模块装载
// =====================================================================

pub mod context;
pub mod entity;
pub mod error;
pub mod event;
pub mod invariants;
pub mod macros;
pub mod port;
pub mod service;
pub mod value_object;

// =====================================================================
// 便捷 re-export
// =====================================================================

pub use context::ActorContext;
pub use entity::{Tenant, TenantPolicy, TenantQuota};
pub use error::TenantError;
pub use event::{
    EventMeta, TenantCreated, TenantEvent, TenantPolicyUpdated, TenantStatusChanged,
};
pub use invariants::{
    check_invariant_01_tenant_key_unique, check_invariant_02_status_transition,
    check_invariant_required_fields, run_invariants, ALL_INVARIANT_CHECKS,
};
pub use port::{
    ChangeTenantStatusCommand, CreateTenantCommand, ListTenantQuery, TenantCommandPort,
    TenantPolicySpec, TenantQueryPort, TenantRepository, UpdateTenantCommand,
    UpdateTenantPolicyCommand,
};
pub use service::InMemoryTenantService;
pub use value_object::{
    roles, TenantId, TenantPolicyId, TenantQuotaId, TenantStatus, TenantTier,
};

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_object::{TenantId, TenantTier};

    // -------- 测试夹具 --------

    fn make_test_actor() -> ActorContext {
        let tenant_id = TenantId::new();
        ActorContext::new(uuid::Uuid::new_v4(), tenant_id)
            .with_role(roles::TENANT_ADMIN)
    }

    /// 创建一个 platform_operator + 与指定 tenant_id 绑定的 actor
    fn make_platform_actor_for(tenant_id: TenantId) -> ActorContext {
        let mut actor = ActorContext::new(uuid::Uuid::new_v4(), tenant_id);
        actor.roles.push(roles::PLATFORM_OPERATOR.to_string());
        actor
    }

    fn make_platform_actor() -> ActorContext {
        make_platform_actor_for(TenantId::new())
    }

    // -------- 1. ActorContext + 强类型 ID smoke test --------

    #[test]
    fn actor_context_typed_ids() {
        let actor = make_test_actor();
        assert!(!actor.tenant_id.as_uuid().is_nil());
        assert!(actor.is_tenant_admin());
        assert!(!actor.is_platform_operator());
    }

    // -------- 2. Tenant 字段数审计 --------

    #[test]
    fn tenant_field_count_audit() {
        assert_eq!(Tenant::FIELD_COUNT, 9);
        assert_eq!(TenantPolicy::FIELD_COUNT, 10);
        assert_eq!(TenantQuota::FIELD_COUNT, 14);
    }

    // -------- 3. create_tenant 成功路径 --------

    #[tokio::test]
    async fn create_tenant_success() {
        let svc = InMemoryTenantService::new_for_test();
        let actor = make_platform_actor();
        let cmd = CreateTenantCommand {
            tenant_key: "acme-corp".to_string(),
            name: "Acme Corporation".to_string(),
            tier: TenantTier::Pro,
            contact_email: Some("admin@acme.com".to_string()),
            initial_policy: Some(TenantPolicySpec::default()),
        };
        let t = svc.create_tenant(cmd, actor).await.expect("创建成功");
        assert_eq!(t.status, TenantStatus::Active);
        assert_eq!(t.tier, TenantTier::Pro);
        assert_eq!(t.version, 1);
        assert_eq!(svc.count().await, 1);

        // 同时创建了 TenantPolicy 和 TenantQuota
        let viewer = make_platform_actor();
        let policy = svc.get_tenant_policy(t.id, viewer.clone()).await.unwrap();
        assert!(policy.cloud_ai_allowed);
        let quota = svc.get_tenant_quota(t.id, viewer).await.unwrap();
        assert_eq!(quota.max_users, 10);
    }

    // -------- 4. tenant_key 冲突被拒 --------

    #[tokio::test]
    async fn create_tenant_key_conflict() {
        let svc = InMemoryTenantService::new_for_test();
        let actor = make_platform_actor();
        let cmd = CreateTenantCommand {
            tenant_key: "dup".to_string(),
            name: "First".to_string(),
            tier: TenantTier::Free,
            contact_email: None,
            initial_policy: None,
        };
        svc.create_tenant(cmd, actor.clone()).await.unwrap();
        // 第二次同 key → Conflict
        let cmd2 = CreateTenantCommand {
            tenant_key: "dup".to_string(),
            name: "Second".to_string(),
            tier: TenantTier::Free,
            contact_email: None,
            initial_policy: None,
        };
        let res = svc.create_tenant(cmd2, actor).await;
        assert!(matches!(res, Err(TenantError::Conflict(_))));
    }

    // -------- 5. INV-AUX-01:空 tenant_key 被拒 --------

    #[tokio::test]
    async fn invariant_required_fields_empty_key() {
        let svc = InMemoryTenantService::new_for_test();
        let actor = make_platform_actor();
        let cmd = CreateTenantCommand {
            tenant_key: "".to_string(),
            name: "Empty key".to_string(),
            tier: TenantTier::Free,
            contact_email: None,
            initial_policy: None,
        };
        let res = svc.create_tenant(cmd, actor).await;
        assert!(matches!(res, Err(TenantError::InvalidState(_))));
    }

    // -------- 6. INV-TEN-02:非法状态迁移被拒 --------

    #[tokio::test]
    async fn invariant_02_illegal_status_transition() {
        let svc = InMemoryTenantService::new_for_test();
        // 用 platform_operator 跨租户创建
        let platform = make_platform_actor();
        let cmd = CreateTenantCommand {
            tenant_key: "t1".to_string(),
            name: "T1".to_string(),
            tier: TenantTier::Free,
            contact_email: None,
            initial_policy: None,
        };
        let t = svc.create_tenant(cmd, platform.clone()).await.unwrap();
        // 改用与 t.id 绑定的 tenant_admin actor 做后续操作
        let admin = make_platform_actor_for(t.id);
        // Active → Active 幂等 OK
        let t2 = svc
            .change_status(
                ChangeTenantStatusCommand {
                    tenant_id: t.id,
                    target_status: TenantStatus::Active,
                    expected_version: 1,
                    reason: None,
                },
                admin.clone(),
            )
            .await
            .unwrap();
        assert_eq!(t2.status, TenantStatus::Active);
        // 迁移到 Suspended
        let t3 = svc
            .change_status(
                ChangeTenantStatusCommand {
                    tenant_id: t.id,
                    target_status: TenantStatus::Suspended,
                    expected_version: 2,
                    reason: None,
                },
                admin.clone(),
            )
            .await
            .unwrap();
        assert_eq!(t3.status, TenantStatus::Suspended);
        // Suspended → Active OK(恢复)
        let t4 = svc
            .change_status(
                ChangeTenantStatusCommand {
                    tenant_id: t.id,
                    target_status: TenantStatus::Active,
                    expected_version: 3,
                    reason: None,
                },
                admin,
            )
            .await
            .unwrap();
        assert_eq!(t4.status, TenantStatus::Active);
    }

    // -------- 7. 跨租户访问被拒 --------

    #[tokio::test]
    async fn cross_tenant_access_denied() {
        let svc = InMemoryTenantService::new_for_test();
        // 创建 tenant_a
        let platform = make_platform_actor();
        let cmd_a = CreateTenantCommand {
            tenant_key: "tenant-a".to_string(),
            name: "A".to_string(),
            tier: TenantTier::Free,
            contact_email: None,
            initial_policy: None,
        };
        let ta = svc.create_tenant(cmd_a, platform.clone()).await.unwrap();
        // 用 tenant_b 的 actor 尝试访问 tenant_a
        let tenant_b_id = TenantId::new();
        let actor_b = ActorContext::new(uuid::Uuid::new_v4(), tenant_b_id).with_role(roles::TENANT_ADMIN);
        let res = svc.get_by_id(ta.id, actor_b).await;
        assert!(matches!(res, Err(TenantError::PermissionDenied)));
    }

    // -------- 8. 事件总线烟囱测试 --------

    #[tokio::test]
    async fn event_bus_receives_created() {
        let (svc, mut rx) = InMemoryTenantService::new();
        let actor = make_platform_actor();
        let cmd = CreateTenantCommand {
            tenant_key: "evt-test".to_string(),
            name: "Event Test".to_string(),
            tier: TenantTier::Free,
            contact_email: None,
            initial_policy: None,
        };
        svc.create_tenant(cmd, actor).await.unwrap();
        let evt = rx.try_recv().expect("应收到 Created 事件");
        assert!(matches!(evt, TenantEvent::Created(_)));
        assert_eq!(evt.subject(), "star.events.tenant.tenant.created.v1");
    }

    // -------- 9. 乐观锁冲突 --------

    #[tokio::test]
    async fn update_tenant_version_conflict() {
        let svc = InMemoryTenantService::new_for_test();
        let platform = make_platform_actor();
        let cmd = CreateTenantCommand {
            tenant_key: "v-test".to_string(),
            name: "V".to_string(),
            tier: TenantTier::Free,
            contact_email: None,
            initial_policy: None,
        };
        let t = svc.create_tenant(cmd, platform.clone()).await.unwrap();
        let admin = make_platform_actor_for(t.id);
        let res = svc
            .update_tenant(
                UpdateTenantCommand {
                    tenant_id: t.id,
                    expected_version: 99, // 错的
                    name: Some("New".to_string()),
                    contact_email: None,
                    tier: None,
                },
                admin,
            )
            .await;
        assert!(matches!(res, Err(TenantError::Conflict(_))));
    }

    // -------- 10. 非 tenant_admin 更新被拒 --------

    #[tokio::test]
    async fn update_tenant_permission_denied() {
        let svc = InMemoryTenantService::new_for_test();
        let platform = make_platform_actor();
        let cmd = CreateTenantCommand {
            tenant_key: "perm".to_string(),
            name: "P".to_string(),
            tier: TenantTier::Free,
            contact_email: None,
            initial_policy: None,
        };
        let t = svc.create_tenant(cmd, platform).await.unwrap();
        // 普通 viewer 角色
        let mut actor = ActorContext::new(uuid::Uuid::new_v4(), t.id);
        actor.roles.push("viewer".to_string());
        let res = svc
            .update_tenant(
                UpdateTenantCommand {
                    tenant_id: t.id,
                    expected_version: 1,
                    name: Some("Hacker".to_string()),
                    contact_email: None,
                    tier: None,
                },
                actor,
            )
            .await;
        assert!(matches!(res, Err(TenantError::PermissionDenied)));
    }
}
