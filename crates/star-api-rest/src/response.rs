// SPDX-License-Identifier: MIT OR Apache-2.0
//! 统一响应封装 (per spec §2.4)
//!
//! 成功响应:
//! ```json
//! {
//!   "data": { /* resource */ },
//!   "meta": { "request_id": "req_abc", "timestamp": "...", "version": "v1" }
//! }
//! ```
//!
//! 列表 + 分页:
//! ```json
//! {
//!   "data": [ /* items */ ],
//!   "meta": { ... },
//!   "pagination": { "page": 1, "page_size": 20, "total": 142, ... }
//! }
//! ```

use chrono::Utc;
use serde::{Deserialize, Serialize};

/// 响应元数据 (per spec §2.4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    /// request id (从上游中间件 `AuditLayer` 透传, P2 阶段实装)
    pub request_id: String,
    /// RFC 3339 timestamp
    pub timestamp: String,
    /// API version (固定 "v1")
    pub version: String,
}

impl ResponseMeta {
    /// stub request_id (P2 阶段由 `AuditLayer` 注入真实 ID)
    pub fn stub() -> Self {
        Self {
            request_id: "req_stub".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            version: "v1".to_string(),
        }
    }
}

/// 统一成功响应 (per spec §2.4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestResponse<T> {
    /// 业务数据
    pub data: T,
    /// 响应元数据
    pub meta: ResponseMeta,
}

impl<T: Serialize> RestResponse<T> {
    /// 包装业务数据 + stub meta
    pub fn ok(data: T) -> Self {
        Self {
            data,
            meta: ResponseMeta::stub(),
        }
    }
}

/// 分页元数据 (per spec §2.4 列表 + 分页)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// 当前页 (1-indexed)
    pub page: u32,
    /// 每页条数
    pub page_size: u32,
    /// 总条数
    pub total: u64,
    /// 总页数
    pub total_pages: u32,
    /// 是否有下一页
    pub has_next: bool,
}

/// 列表 + 分页响应 (per spec §2.4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedResponse<T> {
    /// 业务数据 (列表)
    pub data: Vec<T>,
    /// 响应元数据
    pub meta: ResponseMeta,
    /// 分页元数据
    pub pagination: Pagination,
}
