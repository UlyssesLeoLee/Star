//! 权威 `ActorContext` 定义 (per AGENTS.md §0 一句话硬约束 + P0-1 联动协作修复)
//!
//! **crate**: `star_context::actor`
//! **来源**: 2026-08-31 P0-1 联动审计发现 14 个 domain-* + 3 supporting crate
//!   各自定义 `ActorContext` 重复 17 次, 字段不兼容 (per audit report P0-1)
//!
//! ## 职责
//!
//! 提供仓库内**唯一**的 `ActorContext` 权威定义, 替换 17 处重复.
//!
//! ## 字段 (合并 identity 版 + permission 版)
//!
//! | 字段 | 来源 | 说明 |
//! |---|---|---|
//! | `user_id` | 双版共有 | 当前用户 UUID (各 domain 转 `UserId` 强类型) |
//! | `tenant_id` | 双版共有 | 当前租户 UUID (各 domain 转 `TenantId`) |
//! | `project_ids` | 双版共有 | 当前 Project IDs (Vec<Uuid>, 各 domain 转 `ProjectId`) |
//! | `roles` | 双版共有 | 角色字符串 (`Vec<String>`) — 字符串与 domain 强类型 `Role` 枚举转换 |
//! | `device_id` | api 版独有 | Local Runtime 三重绑定 (per ADR-0024) |
//! | `is_local_runtime` | permission 版独有 | Agent 自身 subject 标志 |
//! | `is_platform_admin` | identity 版独有 | 平台管理员标志 |
//!
//! ## 不变量
//!
//! - INV-ACT-01: `user_id` / `tenant_id` 非 nil UUID
//! - INV-ACT-02: `roles` 元素属于 `Role` 枚举的字符串表示 (5 种: tenant_admin / project_admin / developer / viewer / agent)
//! - INV-ACT-03: `is_local_runtime == true` 时, `roles` 必含 `"agent"`
//!
//! ## 迁移路径
//!
//! - **Before** (per audit P0-1): 14 domain + 3 supporting 各定义一份
//! - **After** (本文件): 全部 17 处改用 `use star_context::ActorContext`
//! - **各 domain 内部**: 收到 `ActorContext` 后做 `UserId::from(actor.user_id)` 等转换
//!
//! Lead 责任: 架构师 (Mavis 接手 agent per DEC-008)

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// **权威 ActorContext** — 跨 crate 统一调用方上下文
///
/// 字段设计: 合并 `domain-identity` 版的 `is_platform_admin` +
/// `domain-permission` 版的 `is_local_runtime` + `api` 版的 `device_id`,
/// 用 `Uuid` 而非强类型 ID (避免 star-context 引入对 domain-* 的依赖).
///
/// **调用方语义** (per ADR-0024 IDE session identity):
/// - HTTP REST 入口: api crate 从 JWT claim 构造
/// - MCP 入口: star-mcp handler 从 JSON-RPC params 构造
/// - CLI 入口: star-cli 从 local config + KMS 解密 JWT 构造
/// - Agent 自身: Local Runtime 三重绑定, `is_local_runtime = true`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorContext {
    /// 当前用户 ID (INV-ACT-01: 非 nil)
    pub user_id: Uuid,

    /// 当前租户 ID (INV-ACT-01: 非 nil, 13 类对象必带 per §6.1)
    pub tenant_id: Uuid,

    /// Local Runtime 三重绑定设备 ID (per ADR-0024, 仅 LRT 三重绑定路径必带)
    pub device_id: Option<Uuid>,

    /// 当前 Project IDs (用于 Project Policy 校验 + Permission check)
    pub project_ids: Vec<Uuid>,

    /// 当前用户角色 (字符串形式, INV-ACT-02 约束枚举)
    ///
    /// 5 种合法值: `tenant_admin` / `project_admin` / `developer` / `viewer` / `agent`
    /// (per `domain_permission::Role` 枚举)
    pub roles: Vec<String>,

    /// Agent 自身 subject 标志 (Local Runtime 路径, per domain-permission §3)
    #[serde(default)]
    pub is_local_runtime: bool,

    /// 平台管理员标志 (跨 tenant 操作能力, per domain-identity INV-ID-01)
    #[serde(default)]
    pub is_platform_admin: bool,
}

impl ActorContext {
    /// 新建最小 ActorContext (user + tenant 必带, 其他 None/空)
    ///
    /// **INV-ACT-01 校验**: user_id / tenant_id 非 nil, 否则 panic (debug) / assert (release)
    pub fn new(user_id: Uuid, tenant_id: Uuid) -> Self {
        assert!(!user_id.is_nil(), "ActorContext::new: user_id 不能为 nil (INV-ACT-01)");
        assert!(
            !tenant_id.is_nil(),
            "ActorContext::new: tenant_id 不能为 nil (INV-ACT-01)"
        );
        Self {
            user_id,
            tenant_id,
            device_id: None,
            project_ids: vec![],
            roles: vec!["developer".to_string()],
            is_local_runtime: false,
            is_platform_admin: false,
        }
    }

    /// 追加角色
    pub fn with_role(mut self, role: &str) -> Self {
        self.roles.push(role.to_string());
        self
    }

    /// 是否持有指定角色字符串
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// 是否平台管理员 (cross-tenant 能力)
    pub fn is_platform_admin(&self) -> bool {
        self.is_platform_admin
    }

    /// 是否 Local Runtime / Agent 自身 (per domain-permission §3)
    pub fn is_local_runtime(&self) -> bool {
        self.is_local_runtime
    }

    /// 解析为 `Role` 枚举兼容的 `Vec<String>` (per domain-permission `from_str_opt`)
    ///
    /// **注意**: 仅做大小写归一, 不做权限裁决.
    /// 权限裁决走 `domain_permission::check` (INV-PM-02 Deny 优先 + INV-PM-05 默认 Deny).
    pub fn parsed_roles(&self) -> Vec<String> {
        self.roles
            .iter()
            .map(|s| s.to_ascii_lowercase())
            .collect()
    }
}

impl Default for ActorContext {
    /// 默认 ActorContext (仅用于测试桩 + 无 actor 内部调用)
    ///
    /// **警告**: `Default::default()` 不满足 INV-ACT-01 (uuid 是 nil), 业务代码
    /// 必须用 `ActorContext::new(user, tenant)` 或显式构造.
    fn default() -> Self {
        Self {
            user_id: Uuid::nil(),
            tenant_id: Uuid::nil(),
            device_id: None,
            project_ids: vec![],
            roles: vec![],
            is_local_runtime: false,
            is_platform_admin: false,
        }
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_developer_role_by_default() {
        let a = ActorContext::new(Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(a.roles, vec!["developer".to_string()]);
        assert!(!a.is_platform_admin);
        assert!(!a.is_local_runtime);
    }

    #[test]
    fn with_role_appends() {
        let a = ActorContext::new(Uuid::new_v4(), Uuid::new_v4())
            .with_role("tenant_admin");
        assert!(a.has_role("tenant_admin"));
        assert!(a.has_role("developer"));
    }

    #[test]
    fn has_role_case_sensitive() {
        let a = ActorContext::new(Uuid::new_v4(), Uuid::new_v4())
            .with_role("Tenant_Admin");
        // has_role 严格大小写 (Role 枚举统一由 domain-permission 转换)
        assert!(!a.has_role("tenant_admin"));
        assert!(a.has_role("Tenant_Admin"));
    }

    #[test]
    fn parsed_roles_lowercases() {
        let a = ActorContext::new(Uuid::new_v4(), Uuid::new_v4())
            .with_role("Tenant_Admin")
            .with_role("Developer");
        let parsed = a.parsed_roles();
        // "developer" (default) + "tenant_admin" + "developer"
        assert_eq!(parsed.iter().filter(|r| *r == "developer").count(), 2);
        assert!(parsed.contains(&"tenant_admin".to_string()));
    }

    #[test]
    #[should_panic(expected = "user_id 不能为 nil")]
    fn new_panics_on_nil_user() {
        ActorContext::new(Uuid::nil(), Uuid::new_v4());
    }

    #[test]
    #[should_panic(expected = "tenant_id 不能为 nil")]
    fn new_panics_on_nil_tenant() {
        ActorContext::new(Uuid::new_v4(), Uuid::nil());
    }

    #[test]
    fn default_has_nil_uuids() {
        // Default 不满足 INV-ACT-01, 仅供测试桩使用
        let a = ActorContext::default();
        assert!(a.user_id.is_nil());
        assert!(a.tenant_id.is_nil());
    }

    #[test]
    fn serde_roundtrip() {
        let a = ActorContext::new(Uuid::new_v4(), Uuid::new_v4())
            .with_role("tenant_admin");
        let json = serde_json::to_string(&a).unwrap();
        let b: ActorContext = serde_json::from_str(&json).unwrap();
        assert_eq!(a.user_id, b.user_id);
        assert_eq!(a.tenant_id, b.tenant_id);
        assert_eq!(a.roles, b.roles);
    }
}
