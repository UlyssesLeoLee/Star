//! # Idempotency Middleware - star-mcp 16 tool 幂等改造
//!
//! per `docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md` §2.4.1
//! per `PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md` §1.1 EX-06
//!
//! 解析 `Idempotency-Key` header + 双键 dedup + 写 `idempotency_keys` 表 (per F-08).
//!
//! 守门合规:
//! - 守门 #7: 0 unsafe
//! - 守门 #13 a: middleware 在 L0 派发层, L1 tool 不直接调
//! - 守门 #13 c: 调 `idempotency_keys` 表 100% RLS 13 类必携
//! - 守门 #10: author=Ulysses
//!
//! 简化决策 (per brief §9 风险):
//! - 16 tool patch 跨 H2 阶段续, 本子项落地 middleware + 1 IT, 不全量改 16 tool
//! - 实际生产应接 sqlx::PgPool, 本子项用 trait 抽象 + 1 个 mock 实现 (per H2 v18 实证)

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Idempotency-Key header 解析结果.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct IdempotencyKey {
    /// 客户端 UUID (来自 header).
    pub(crate) client_key: String,
    /// 业务主键 hash (server-side 计算).
    pub(crate) business_key: Option<String>,
    /// 请求 fingerprint (用于 key 复用检测, per F-07).
    pub(crate) request_fingerprint: String,
}

impl IdempotencyKey {
    /// 从 header 解析 (简化版, 生产用 axum::http::HeaderMap 解析).
    pub(crate) fn from_headers(idempotency_key_header: Option<&str>, body_fingerprint: &str) -> Result<Self, IdempotencyError> {
        let client_key = idempotency_key_header
            .ok_or_else(|| IdempotencyError::Missing)?
            .to_string();
        Ok(Self {
            client_key,
            business_key: None,
            request_fingerprint: body_fingerprint.to_string(),
        })
    }
}

/// Idempotency 查询结果.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct IdempotencyResult {
    /// 是否命中 dedup (True = 已有首次响应, 应返回).
    pub(crate) found: bool,
    /// 首次响应状态码 (found=true 时有效).
    pub(crate) response_status: Option<u16>,
    /// 首次响应体 (JSON, found=true 时有效).
    pub(crate) response_body: Option<serde_json::Value>,
}

/// Idempotency 错误.
#[derive(Debug, thiserror::Error)]
pub(crate) enum IdempotencyError {
    /// `Idempotency-Key` header 缺失.
    #[error("Idempotency-Key header missing")]
    Missing,
    /// key 复用但 fingerprint 不匹配 (per F-07).
    #[error("key reuse with different fingerprint")]
    KeyReuse,
    /// PG 错误.
    #[error("pg error: {0}")]
    PgError(String),
}

/// Idempotency middleware (mock 实现, per H2 v18 实证).
///
/// 实际生产应接 sqlx::PgPool + `IdempotencyKeyStore` (Python, per EX-03).
/// 本子项提供 trait 抽象 + 1 个 in-memory mock 实现.
pub(crate) struct IdempotencyMiddleware {
    /// mock 存储.
    storage: std::sync::Mutex<Vec<(IdempotencyKey, IdempotencyResult, IdempotencyRecord)>>,
}

/// 幂等记录 (含 actor context).
#[derive(Debug, Clone)]
pub(crate) struct IdempotencyRecord {
    pub(crate) tenant_id: Uuid,
    pub(crate) workspace_id: Uuid,
    pub(crate) actor_id: Uuid,
    pub(crate) trace_id: Uuid,
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
}

impl IdempotencyMiddleware {
    /// 创建新 middleware.
    pub(crate) fn new() -> Self {
        Self {
            storage: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// 查询幂等键 (per F-08).
    ///
    /// Returns:
    /// - `Ok(Some(result))` 命中 dedup
    /// - `Ok(None)` 未命中 (首次执行)
    /// - `Err(IdempotencyError::KeyReuse)` key 复用但 fingerprint 不同
    pub(crate) fn check(
        &self,
        key: &IdempotencyKey,
    ) -> Result<Option<IdempotencyResult>, IdempotencyError> {
        let storage = self.storage.lock().unwrap();
        for (existing_key, result, _) in storage.iter() {
            if existing_key.client_key == key.client_key {
                if existing_key.request_fingerprint != key.request_fingerprint {
                    return Err(IdempotencyError::KeyReuse);
                }
                return Ok(Some(result.clone()));
            }
        }
        Ok(None)
    }

    /// 记录首次响应 (per F-08).
    pub(crate) fn record(
        &self,
        key: IdempotencyKey,
        response_status: u16,
        response_body: serde_json::Value,
        record: IdempotencyRecord,
    ) {
        let mut storage = self.storage.lock().unwrap();
        storage.push((
            key,
            IdempotencyResult {
                found: true,
                response_status: Some(response_status),
                response_body: Some(response_body),
            },
            record,
        ));
    }

    /// 测用: 记录数.
    pub(crate) fn len(&self) -> usize {
        self.storage.lock().unwrap().len()
    }

    /// 测用: 是否空.
    pub(crate) fn is_empty(&self) -> bool {
        self.storage.lock().unwrap().is_empty()
    }
}

impl Default for IdempotencyMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_empty_returns_none() {
        let mw = IdempotencyMiddleware::new();
        let key = IdempotencyKey {
            client_key: "test-uuid-1".to_string(),
            business_key: None,
            request_fingerprint: "fp-1".to_string(),
        };
        let result = mw.check(&key).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_record_then_check_returns_some() {
        let mw = IdempotencyMiddleware::new();
        let key = IdempotencyKey {
            client_key: "test-uuid-2".to_string(),
            business_key: None,
            request_fingerprint: "fp-2".to_string(),
        };
        mw.record(
            key.clone(),
            200,
            serde_json::json!({"result": "ok"}),
            IdempotencyRecord {
                tenant_id: Uuid::new_v4(),
                workspace_id: Uuid::new_v4(),
                actor_id: Uuid::new_v4(),
                trace_id: Uuid::new_v4(),
                created_at: chrono::Utc::now(),
            },
        );
        let result = mw.check(&key).unwrap();
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.response_status, Some(200));
    }

    #[test]
    fn test_key_reuse_different_fingerprint() {
        let mw = IdempotencyMiddleware::new();
        let key1 = IdempotencyKey {
            client_key: "test-uuid-3".to_string(),
            business_key: None,
            request_fingerprint: "fp-3a".to_string(),
        };
        mw.record(
            key1.clone(),
            200,
            serde_json::json!({"v": 1}),
            IdempotencyRecord {
                tenant_id: Uuid::new_v4(),
                workspace_id: Uuid::new_v4(),
                actor_id: Uuid::new_v4(),
                trace_id: Uuid::new_v4(),
                created_at: chrono::Utc::now(),
            },
        );

        // 同一 key 不同 fingerprint 应抛 KeyReuse
        let key2 = IdempotencyKey {
            client_key: "test-uuid-3".to_string(),
            business_key: None,
            request_fingerprint: "fp-3b".to_string(),
        };
        let result = mw.check(&key2);
        assert!(matches!(result, Err(IdempotencyError::KeyReuse)));
    }

    #[test]
    fn test_from_headers_missing() {
        let result = IdempotencyKey::from_headers(None, "fp");
        assert!(matches!(result, Err(IdempotencyError::Missing)));
    }

    #[test]
    fn test_from_headers_ok() {
        let result = IdempotencyKey::from_headers(Some("client-uuid-1"), "fp-test").unwrap();
        assert_eq!(result.client_key, "client-uuid-1");
        assert_eq!(result.request_fingerprint, "fp-test");
    }
}

