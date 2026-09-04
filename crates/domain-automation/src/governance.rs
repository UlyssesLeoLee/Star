//! Star Automation — 治理 (wt-w8-gov 扩展)
//!
//! 治理能力:
//! - RBAC 编辑组: 控制哪些群组可创建/编辑规则
//! - Pause-all: 全局紧急暂停
//! - 死信队列: 失败 3 次入队, 可 replay / dismiss
//! - 维护窗口: 时间段内自动暂停
//! - 限流: 每规则每小时最多 N 次
//! - 阻止动作: 全局禁用某些 Action 类型
//! - 审计: 规则创建/编辑/执行/失败全记录

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// 1. RBAC
// =====================================================================

/// RBAC 编辑组配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RbacConfig {
    /// 可创建 / 编辑规则的群组
    pub editor_groups: Vec<String>,
    /// 是否允许非 admin 创建规则
    pub allow_non_admin: bool,
}

impl Default for RbacConfig {
    fn default() -> Self {
        Self {
            editor_groups: vec!["jira-administrators".into()],
            allow_non_admin: false,
        }
    }
}

/// 审批流程配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovalFlow {
    /// 是否启用审批流程
    pub enabled: bool,
    /// 需要审批的群组列表
    pub required_approver_groups: Vec<String>,
}

// =====================================================================
// 2. Pause-all
// =====================================================================

/// 全局暂停状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PauseState {
    /// 是否处于暂停状态
    pub paused: bool,
    /// 暂停时间
    pub paused_at: Option<DateTime<Utc>>,
    /// 执行暂停的用户
    pub paused_by: Option<Uuid>,
    /// 暂停原因
    pub reason: Option<String>,
}

impl PauseState {
    /// 构造未暂停的初始状态
    pub fn new() -> Self {
        Self {
            paused: false,
            paused_at: None,
            paused_by: None,
            reason: None,
        }
    }

    /// 触发全局暂停
    pub fn pause(&mut self, actor: Uuid, reason: impl Into<String>) {
        self.paused = true;
        self.paused_at = Some(Utc::now());
        self.paused_by = Some(actor);
        self.reason = Some(reason.into());
    }

    /// 恢复(解除暂停)
    pub fn resume(&mut self) {
        self.paused = false;
        self.paused_at = None;
        self.paused_by = None;
        self.reason = None;
    }
}

impl Default for PauseState {
    fn default() -> Self {
        Self::new()
    }
}

// =====================================================================
// 3. 限流
// =====================================================================

/// 限流配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThrottleConfig {
    /// 每规则每小时最多执行次数
    pub max_per_rule_per_hour: u32,
    /// 每规则每次执行最大动作数
    pub max_actions_per_run: u32,
    /// 全局每小时最大执行次数
    pub max_global_per_hour: u32,
    /// 错误告警阈值 (连续失败次数)
    pub error_alert_threshold: u32,
}

impl Default for ThrottleConfig {
    fn default() -> Self {
        Self {
            max_per_rule_per_hour: 1000,
            max_actions_per_run: 50,
            max_global_per_hour: 10_000,
            error_alert_threshold: 5,
        }
    }
}

/// 限流计数器(按规则 + 小时分桶)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThrottleCounter {
    /// 规则 ID
    pub rule_id: Uuid,
    /// 小时分桶, 如 "2026-08-29T04"
    pub hour_bucket: String, // "2026-08-29T04" (按小时)
    /// 当前小时内已执行次数
    pub count: u32,
}

// =====================================================================
// 4. 阻止动作
// =====================================================================

/// 全局阻止的 Action 类型集合
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockedActions {
    /// 禁用的 Action 类型列表
    pub blocked: Vec<String>,
}

impl Default for BlockedActions {
    fn default() -> Self {
        Self { blocked: vec![] }
    }
}

impl BlockedActions {
    /// 判断指定 Action 类型是否被阻止
    pub fn is_blocked(&self, action_type: &str) -> bool {
        self.blocked.iter().any(|b| b == action_type)
    }
}

// =====================================================================
// 5. 维护窗口
// =====================================================================

/// 维护窗口(时间段内自动暂停)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceWindow {
    /// 窗口 ID
    pub id: Uuid,
    /// 窗口名称
    pub name: String,
    /// 起始时间
    pub start_at: DateTime<Utc>,
    /// 结束时间
    pub end_at: DateTime<Utc>,
    /// 是否每周循环
    pub recurring: bool, // 每周循环
}

impl MaintenanceWindow {
    /// 判断给定时间点是否落在窗口内
    pub fn is_active(&self, now: DateTime<Utc>) -> bool {
        now >= self.start_at && now <= self.end_at
    }
}

// =====================================================================
// 6. 死信队列
// =====================================================================

/// 死信队列条目
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeadLetterEntry {
    /// 条目 ID
    pub id: Uuid,
    /// 关联规则 ID
    pub rule_id: Uuid,
    /// 失败原因
    pub failure_reason: String,
    /// 已重试次数
    pub attempts: u32,
    /// 首次失败时间
    pub first_failed_at: DateTime<Utc>,
    /// 最近一次失败时间
    pub last_failed_at: DateTime<Utc>,
    /// 当前状态
    pub status: DlqStatus,
    /// 失败时的原始 payload
    pub payload: serde_json::Value,
}

/// 死信队列条目状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DlqStatus {
    /// 待重放
    Pending, // 待重放
    /// 已重放
    Replayed, // 已重放
    /// 已忽略
    Dismissed, // 已忽略
}

// =====================================================================
// 7. 审计日志
// =====================================================================

/// 审计日志条目
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEntry {
    /// 条目 ID
    pub id: Uuid,
    /// 审计事件
    pub event: AuditEvent,
    /// 触发事件的用户(系统触发时为 None)
    pub actor_id: Option<Uuid>,
    /// 租户 ID
    pub tenant_id: Uuid,
    /// 事件发生时间
    pub at: DateTime<Utc>,
}

/// 审计事件类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuditEvent {
    /// 规则被创建
    RuleCreated {
        /// 规则 ID
        rule_id: Uuid,
        /// 规则名称
        name: String,
    },
    /// 规则被编辑
    RuleEdited {
        /// 规则 ID
        rule_id: Uuid,
    },
    /// 规则被执行
    RuleExecuted {
        /// 规则 ID
        rule_id: Uuid,
        /// 是否执行成功
        success: bool,
    },
    /// 规则执行失败
    RuleFailed {
        /// 规则 ID
        rule_id: Uuid,
        /// 错误信息
        error: String,
    },
    /// 全局暂停状态被切换
    PauseToggled {
        /// 切换后是否处于暂停
        paused: bool,
        /// 操作用户
        by: Uuid,
    },
    /// 配置项被修改
    SettingsChanged {
        /// 配置项 key
        key: String,
        /// 操作用户
        by: Uuid,
    },
    /// 某 Action 类型被阻止
    ActionBlocked {
        /// 被阻止的 Action 类型
        action: String,
        /// 操作用户
        by: Uuid,
    },
}

// =====================================================================
// 8. GovernanceService (聚合)
// =====================================================================

/// 治理服务(聚合 RBAC / Pause / 限流 / 阻止动作 / 维护窗口 / 死信队列 / 审计)
pub struct GovernanceService {
    /// RBAC 编辑组配置
    pub rbac: RbacConfig,
    /// 审批流程配置
    pub approval: ApprovalFlow,
    /// 全局暂停状态
    pub pause: PauseState,
    /// 限流配置
    pub throttle: ThrottleConfig,
    /// 全局阻止的 Action 类型
    pub blocked: BlockedActions,
    /// 维护窗口列表
    pub maintenance: Vec<MaintenanceWindow>,
    /// 死信队列
    pub dlq: Vec<DeadLetterEntry>,
    /// 审计日志
    pub audit: Vec<AuditEntry>,
}

impl GovernanceService {
    /// 构造默认治理服务
    pub fn new() -> Self {
        Self {
            rbac: RbacConfig::default(),
            approval: ApprovalFlow {
                enabled: false,
                required_approver_groups: vec![],
            },
            pause: PauseState::new(),
            throttle: ThrottleConfig::default(),
            blocked: BlockedActions::default(),
            maintenance: vec![],
            dlq: vec![],
            audit: vec![],
        }
    }

    /// 检查规则是否可执行 (RBAC + Pause + 维护窗口 + 阻止动作)
    pub fn can_execute(
        &self,
        rule_id: Uuid,
        action_types: &[String],
        actor_groups: &[String],
        now: DateTime<Utc>,
    ) -> Result<(), GovernanceError> {
        if self.pause.paused {
            return Err(GovernanceError::Paused);
        }
        for w in &self.maintenance {
            if w.is_active(now) {
                return Err(GovernanceError::InMaintenance {
                    window: w.name.clone(),
                });
            }
        }
        if !self.rbac.allow_non_admin
            && !actor_groups
                .iter()
                .any(|g| self.rbac.editor_groups.contains(g))
        {
            return Err(GovernanceError::PermissionDenied);
        }
        for action in action_types {
            if self.blocked.is_blocked(action) {
                return Err(GovernanceError::ActionBlocked {
                    action: action.clone(),
                });
            }
        }
        // 限流 (简化: 假设 per-hour counter 已经在外部维护)
        let _ = rule_id;
        Ok(())
    }

    /// 添加死信条目
    pub fn add_dlq(&mut self, entry: DeadLetterEntry) {
        self.dlq.push(entry);
    }

    /// 重放死信条目
    pub fn replay_dlq(&mut self, id: Uuid) -> Result<(), GovernanceError> {
        if let Some(entry) = self.dlq.iter_mut().find(|e| e.id == id) {
            entry.status = DlqStatus::Replayed;
            Ok(())
        } else {
            Err(GovernanceError::DlqNotFound(id))
        }
    }

    /// 忽略死信条目
    pub fn dismiss_dlq(&mut self, id: Uuid) -> Result<(), GovernanceError> {
        if let Some(entry) = self.dlq.iter_mut().find(|e| e.id == id) {
            entry.status = DlqStatus::Dismissed;
            Ok(())
        } else {
            Err(GovernanceError::DlqNotFound(id))
        }
    }

    /// 添加审计
    pub fn add_audit(&mut self, entry: AuditEntry) {
        self.audit.push(entry);
    }
}

impl Default for GovernanceService {
    fn default() -> Self {
        Self::new()
    }
}

/// 治理相关错误
#[derive(Debug, Error, Clone, PartialEq)]
pub enum GovernanceError {
    /// 全局暂停中
    #[error("全局暂停中")]
    Paused,
    /// 处于维护窗口内
    #[error("维护窗口: {window}")]
    InMaintenance {
        /// 维护窗口名称
        window: String,
    },
    /// 权限拒绝
    #[error("权限拒绝")]
    PermissionDenied,
    /// 动作被阻止
    #[error("动作被阻止: {action}")]
    ActionBlocked {
        /// 被阻止的 Action 类型
        action: String,
    },
    /// 死信条目未找到
    #[error("死信条目未找到: {0}")]
    DlqNotFound(Uuid),
    /// 限流触发
    #[error("限流触发: {0}")]
    Throttled(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pause_state() {
        let mut p = PauseState::new();
        assert!(!p.paused);
        p.pause(Uuid::new_v4(), "migrate DB");
        assert!(p.paused);
        p.resume();
        assert!(!p.paused);
    }

    #[test]
    fn test_can_execute_default() {
        let g = GovernanceService::new();
        let r = g.can_execute(
            Uuid::new_v4(),
            &["EditField".into()],
            &["jira-administrators".into()],
            Utc::now(),
        );
        assert!(r.is_ok());
    }

    #[test]
    fn test_can_execute_paused() {
        let mut g = GovernanceService::new();
        g.pause.pause(Uuid::new_v4(), "incident");
        let r = g.can_execute(
            Uuid::new_v4(),
            &["EditField".into()],
            &["jira-administrators".into()],
            Utc::now(),
        );
        assert!(matches!(r, Err(GovernanceError::Paused)));
    }

    #[test]
    fn test_can_execute_blocked_action() {
        let mut g = GovernanceService::new();
        g.blocked.blocked.push("DeleteIssue".into());
        let r = g.can_execute(
            Uuid::new_v4(),
            &["DeleteIssue".into()],
            &["jira-administrators".into()],
            Utc::now(),
        );
        assert!(matches!(r, Err(GovernanceError::ActionBlocked { .. })));
    }

    #[test]
    fn test_can_execute_permission_denied() {
        let g = GovernanceService::new();
        let r = g.can_execute(
            Uuid::new_v4(),
            &["EditField".into()],
            &["jira-users".into()], // 非 admin
            Utc::now(),
        );
        assert!(matches!(r, Err(GovernanceError::PermissionDenied)));
    }

    #[test]
    fn test_maintenance_window_active() {
        let now = Utc::now();
        let w = MaintenanceWindow {
            id: Uuid::new_v4(),
            name: "Weekly".into(),
            start_at: now - chrono::Duration::hours(1),
            end_at: now + chrono::Duration::hours(1),
            recurring: true,
        };
        assert!(w.is_active(now));
    }

    #[test]
    fn test_dlq_replay_dismiss() {
        let mut g = GovernanceService::new();
        let id = Uuid::new_v4();
        g.add_dlq(DeadLetterEntry {
            id,
            rule_id: Uuid::new_v4(),
            failure_reason: "timeout".into(),
            attempts: 3,
            first_failed_at: Utc::now(),
            last_failed_at: Utc::now(),
            status: DlqStatus::Pending,
            payload: serde_json::json!({}),
        });
        g.replay_dlq(id).unwrap();
        assert_eq!(g.dlq[0].status, DlqStatus::Replayed);
        g.dismiss_dlq(id).unwrap();
        assert_eq!(g.dlq[0].status, DlqStatus::Dismissed);
    }

    #[test]
    fn test_audit_log() {
        let mut g = GovernanceService::new();
        g.add_audit(AuditEntry {
            id: Uuid::new_v4(),
            event: AuditEvent::RuleCreated {
                rule_id: Uuid::new_v4(),
                name: "test".into(),
            },
            actor_id: None,
            tenant_id: Uuid::new_v4(),
            at: Utc::now(),
        });
        assert_eq!(g.audit.len(), 1);
    }
}
