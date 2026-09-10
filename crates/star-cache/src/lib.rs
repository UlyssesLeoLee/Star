// SPDX-License-Identifier: MIT OR Apache-2.0
//! crates/star-cache — Cache layer (per spec/cache/01 + ADR-0035 §8 Phase G)
//!
//! 提供统一的缓存抽象：
//! - `CacheBackend` trait 覆盖 6 个核心操作（get / set / del / exists / incr / expire）
//! - `InMemoryBackend` (per spec/cache/01 §5) — Tokio RwLock + HashMap + TTL 过期
//! - `ValkeyBackend` (per spec/cache/01 §5) — Phase G+ stub, 占位 URL 解析
//!   选型 Valkey 替代 Redis (per docs/operation-design.md §4.4 + data-design.md §13.1):
//!   Redis 7.4+ 改 RSALv2/SSPL 不再 OSI 开源, Valkey (LF) 保持 BSD-3-Clause
//! - `KeyBuilder` (per spec/cache/01 §3) — 三类键名规范（resource / list / field）
//!
//! 缺标比错标安全：Valkey backend 当前为 Phase G+ stub，所有方法返回 CacheError::Other。

/// 统一键名构造器 (per spec/cache/01 §3)
pub mod cache_trait;
/// 进程内 LRU 后端 (per spec/cache/01 §5)
pub mod in_memory_backend;
/// Valkey 后端 (per spec/cache/01 §5) — Phase G+ stub (替代原 RedisBackend, 选型见模块注释)
pub mod valkey_backend;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 缓存错误 (per spec/cache/01 §6 — 5 类错误)
#[derive(Debug, Error)]
pub enum CacheError {
    /// Backend 暂未实装 (Phase G+ stub)。
    /// Phase G+ 完整实装 (per P4-WBS Phase G.4 拍板启动)
    #[error("not_implemented: feature={feature}, suggestion={suggestion}")]
    NotImplemented {
        /// 未实装的特性名 (e.g. "in_memory_incr")
        feature: String,
        /// 推荐替代方案 (含 current_phase / p4_phase 标签)
        suggestion: String,
    },
    /// 连接错误 (e.g. Valkey URL unset / 拨号失败)
    #[error("connection: {0}")]
    Connection(String),
    /// 键不存在
    #[error("not_found")]
    NotFound,
    /// 序列化/反序列化失败
    #[error("decode: {0}")]
    Decode(String),
    /// 网络层错误
    #[error("network: {0}")]
    Network(String),
    /// 其他错误 (含 stub 未实装)
    #[error("other: {0}")]
    Other(String),
}

impl CacheError {
    /// 错误码 (per spec/cache/01 §6 — 单一错误码)
    pub fn code(&self) -> &'static str {
        "CACHE_ERROR"
    }

    /// 是否可重试 (per spec/cache/01 §6.2)
    pub fn retriable(&self) -> bool {
        matches!(self, Self::Network(_) | Self::Connection(_))
    }
}

/// 后端类型枚举 (per spec/cache/01 §2)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Backend {
    /// 进程内 LRU (测试 + 单实例)
    InMemory,
    /// 分布式 Valkey (生产) — 替代 Redis (per docs/operation-design.md §4.4)
    Valkey,
}

/// 缓存后端 trait (per spec/cache/01 §4 — 6 个核心方法)
#[async_trait]
pub trait CacheBackend: Send + Sync {
    /// 取值 (TTL 过期返回 Ok(None))
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError>;
    /// 写值 (ttl_sec=0 表示不设过期,部分实现可能禁用)
    async fn set(&self, key: &str, value: &[u8], ttl_sec: u32) -> Result<(), CacheError>;
    /// 删除 (不存在也返回 Ok)
    async fn del(&self, key: &str) -> Result<(), CacheError>;
    /// 存在性检查
    async fn exists(&self, key: &str) -> Result<bool, CacheError>;
    /// 原子自增 (delta 可为负)
    async fn incr(&self, key: &str, delta: i64) -> Result<i64, CacheError>;
    /// 重设 TTL
    async fn expire(&self, key: &str, ttl_sec: u32) -> Result<(), CacheError>;
}

pub use cache_trait::KeyBuilder;
pub use in_memory_backend::InMemoryBackend;
pub use valkey_backend::ValkeyBackend;
