//! Hermes 实体层 (B.2 4 层精简: entity 层)
//!
//! 含:
//!   1. **AuthToken**: 5 endpoint 1: auth 响应 (access_token + expires_at)
//!   2. **QueryRequest**: 5 endpoint 2: query 请求 (filter: status / priority / created_after)
//!   3. **Task**: 通用 task 实体 (id / name / status / priority / payload / created_at / updated_at)
//!   4. **TaskStatus**: task 状态机 (Pending / Running / Completed / Failed / Cancelled)
//!   5. **CancelResponse**: 5 endpoint 5: cancel 响应 (cancelled: bool + cancelled_at)
//!
//! 与 B.1 OpenClaw GenerateRequest / GenerateResponse 的差异:
//!   - B.1: OpenAI 兼容 chat completions schema (model / messages / choices / usage)
//!   - B.2: Hermes task queue schema (id / name / status / priority / payload) — 不同业务域
//!
//! 与 B.6 hermes_client.rs (chat completions) 的差异:
//!   - B.6 跟 B.1 同 schema (OpenAI 兼容), 单 endpoint /chat/completions
//!   - B.2 完全不同: 5 endpoint task queue, 状态机驱动 (per Hermes spec)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =====================================================================
// Task 状态机
// =====================================================================

/// Task 状态机 (per Hermes spec 5 状态, 不是 4 状态)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// 已创建, 等待执行
    Pending,
    /// 正在执行
    Running,
    /// 成功完成
    Completed,
    /// 执行失败 (transient 或 permanent, 不区分)
    Failed,
    /// 已被用户取消
    Cancelled,
}

impl TaskStatus {
    /// 是否是终态 (Completed / Failed / Cancelled)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
        )
    }

    /// 是否是进行中 (Pending / Running)
    pub fn is_in_progress(&self) -> bool {
        matches!(self, TaskStatus::Pending | TaskStatus::Running)
    }

    /// 字符串字面值 (per Hermes API spec JSON 字段值)
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Running => "running",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
            TaskStatus::Cancelled => "cancelled",
        }
    }
}

// =====================================================================
// Task 实体
// =====================================================================

/// Task 实体 (Hermes 任务/作业主数据)
///
/// 字段:
///   - id: 全局唯一 UUID
///   - name: 任务名 (e.g. "build-package", "run-tests")
///   - status: 5 状态机
///   - priority: 0-9, 0 = 最高, 9 = 最低 (per Hermes spec 数字优先级)
///   - payload: 任务输入 (JSON 字符串, Hermes 不强加 schema)
///   - created_at: 创建时间
///   - updated_at: 最后更新时间
///   - result: 任务结果 (仅 Completed 时有值)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    /// 任务 ID
    pub id: Uuid,
    /// 任务名称
    pub name: String,
    /// 任务状态
    pub status: TaskStatus,
    /// 优先级(0-9, 0 最高)
    pub priority: u8,
    /// 任务输入载荷
    pub payload: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    /// 任务结果(仅 Completed 时有值)
    #[serde(default)]
    pub result: Option<String>,
}

impl Task {
    /// 新建 pending task (factory)
    pub fn new_pending(name: impl Into<String>, priority: u8, payload: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            status: TaskStatus::Pending,
            priority,
            payload: payload.into(),
            created_at: now,
            updated_at: None,
            result: None,
        }
    }
}

// =====================================================================
// 5 endpoint 1: auth
// =====================================================================

/// auth 响应: access_token + expires_at
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthToken {
    /// 访问令牌
    pub access_token: String,
    /// 令牌类型, 通常 "Bearer"
    pub token_type: String,
    /// 过期时间
    pub expires_at: DateTime<Utc>,
}

// =====================================================================
// 5 endpoint 2: query
// =====================================================================

/// query 请求: 过滤条件 (status / priority / created_after / limit)
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryRequest {
    /// 状态过滤 (e.g. 仅看 Running)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskStatus>,
    /// 优先级过滤 (0-9)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<u8>,
    /// 创建时间过滤 (created_at >= created_after)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_after: Option<DateTime<Utc>>,
    /// 限制返回条数 (1-100, 默认 20)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

// =====================================================================
// 5 endpoint 5: cancel
// =====================================================================

/// cancel 响应
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelResponse {
    /// 是否成功取消 (true = 已取消, false = 已是终态无法取消)
    pub cancelled: bool,
    /// 取消时间 (cancelled=true 时有值)
    #[serde(default)]
    pub cancelled_at: Option<DateTime<Utc>>,
    /// 任务当前状态 (cancel 后可能仍是 Running, 等 worker 实际停下)
    pub current_status: TaskStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_status_is_terminal() {
        assert!(TaskStatus::Completed.is_terminal());
        assert!(TaskStatus::Failed.is_terminal());
        assert!(TaskStatus::Cancelled.is_terminal());
        assert!(!TaskStatus::Pending.is_terminal());
        assert!(!TaskStatus::Running.is_terminal());
    }

    #[test]
    fn task_status_is_in_progress() {
        assert!(TaskStatus::Pending.is_in_progress());
        assert!(TaskStatus::Running.is_in_progress());
        assert!(!TaskStatus::Completed.is_in_progress());
        assert!(!TaskStatus::Failed.is_in_progress());
        assert!(!TaskStatus::Cancelled.is_in_progress());
    }

    #[test]
    fn task_status_as_str_matches_serde() {
        for s in [
            TaskStatus::Pending,
            TaskStatus::Running,
            TaskStatus::Completed,
            TaskStatus::Failed,
            TaskStatus::Cancelled,
        ] {
            // 序列化/反序列化 roundtrip
            let json = serde_json::to_string(&s).unwrap();
            let back: TaskStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back, "roundtrip fail for {:?}", s);
            // 字面值一致
            assert_eq!(json.trim_matches('"'), s.as_str());
        }
    }

    #[test]
    fn task_new_pending_defaults() {
        let t = Task::new_pending("build", 3, r#"{"input":"value"}"#);
        assert_eq!(t.name, "build");
        assert_eq!(t.priority, 3);
        assert_eq!(t.payload, r#"{"input":"value"}"#);
        assert_eq!(t.status, TaskStatus::Pending);
        assert!(t.result.is_none());
        assert!(t.updated_at.is_none());
        assert_ne!(t.id, Uuid::nil());
    }

    #[test]
    fn task_serde_roundtrip_with_all_fields() {
        let t = Task {
            id: Uuid::new_v4(),
            name: "run-tests".into(),
            status: TaskStatus::Completed,
            priority: 5,
            payload: "{}".into(),
            created_at: Utc::now(),
            updated_at: Some(Utc::now()),
            result: Some("all-passed".into()),
        };
        let json = serde_json::to_string(&t).unwrap();
        let back: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(t, back);
    }

    #[test]
    fn auth_token_serde_roundtrip() {
        let token = AuthToken {
            access_token: "abc-123".into(),
            token_type: "Bearer".into(),
            expires_at: Utc::now(),
        };
        let json = serde_json::to_string(&token).unwrap();
        let back: AuthToken = serde_json::from_str(&json).unwrap();
        assert_eq!(token, back);
    }

    #[test]
    fn query_request_default_is_empty() {
        let q = QueryRequest::default();
        assert!(q.status.is_none());
        assert!(q.priority.is_none());
        assert!(q.created_after.is_none());
        assert!(q.limit.is_none());
        // skip_serializing_if = "Option::is_none" → 序列化后空 JSON 对象
        let json = serde_json::to_string(&q).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn query_request_with_status_serializes_correctly() {
        let q = QueryRequest {
            status: Some(TaskStatus::Running),
            priority: Some(2),
            limit: Some(50),
            ..Default::default()
        };
        let json = serde_json::to_string(&q).unwrap();
        assert!(json.contains("\"status\":\"running\""));
        assert!(json.contains("\"priority\":2"));
        assert!(json.contains("\"limit\":50"));
        // created_after 仍 skip
        assert!(!json.contains("created_after"));
    }

    #[test]
    fn cancel_response_serde_roundtrip() {
        let resp = CancelResponse {
            cancelled: true,
            cancelled_at: Some(Utc::now()),
            current_status: TaskStatus::Cancelled,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: CancelResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp, back);
    }
}
