// SPDX-License-Identifier: MIT OR Apache-2.0
//! `star-memory` — Memory Store (Phase G.6, per `docs/briefs/next-session/OPT-NEXT-04-phase-g.md` §3.6)
//!
//! **目的**: 短/长期 memory 分层 (per SRS-001 §3)
//!
//! **架构 (per G.6)**:
//! - `ShortTermMemory`: 短 TTL 作业中, ring buffer 自动 eviction (W 派生, 守门 #13 a)
//! - `LongTermMemory`: 长期 key-value, append-only 审计 (T 派生, 守门 #13 b)
//! - `MemoryStore` 组合两者, 提供统一 API
//!
//! **守门 #13 W/T 派生 (per 2026-09-01 18:30 JST 拍板)**:
//! - short_term 是 W: 物理删除 OK, TTL 失効, retention_period 必填
//! - long_term 是 T: 物理删除禁止, 監査必須, RLS 13 類必携
//!
//! **Mavis 临时代签 5 域 Lead 决策 (per 9/3 11:35 JST 反转)**: Memory schema 决策 Mavis 临时代签, 真人到位后追溯签字

#![allow(missing_docs)] // G.6 PoC 启动, Phase 2 spec 完成后补 doc

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

mod long_term;
mod short_term;

pub use long_term::LongTermMemory;
pub use short_term::ShortTermMemory;

/// Memory 记录 (per G.6)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    /// 记录 ID
    pub record_id: Uuid,
    /// 租户 ID (跨域隔离, INV-ACT-01, 13 類 RLS 之一)
    pub tenant_id: Uuid,
    /// Agent ID (mem 归属)
    pub agent_id: Uuid,
    /// 记忆类型 (e.g. "context" / "tool_call" / "user_pref")
    pub kind: String,
    /// 记忆内容
    pub content: serde_json::Value,
    /// 创建时间戳 (ms since epoch)
    pub created_at_ms: u64,
    /// 权重 (0.0-1.0, 越大越重要, 越不容易 evict)
    pub weight: f32,
}

/// Memory 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryLayer {
    /// 短 TTL 作业中 (ring buffer)
    ShortTerm,
    /// 长期 append-only
    LongTerm,
}

/// Memory 错误
#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("record {0} not found")]
    NotFound(Uuid),
    #[error("storage: {0}")]
    Storage(String),
    #[error("invalid: {0}")]
    Invalid(String),
}

/// Memory 抽象 (per G.6)
#[async_trait]
pub trait Memory: Send + Sync {
    /// 写入一条 memory
    async fn write(&self, record: MemoryRecord) -> Result<Uuid, MemoryError>;
    /// 按 ID 读
    async fn read(&self, record_id: Uuid) -> Result<MemoryRecord, MemoryError>;
    /// 按 agent_id 列出 (按权重降序)
    async fn list_by_agent(
        &self,
        agent_id: Uuid,
        limit: u32,
    ) -> Result<Vec<MemoryRecord>, MemoryError>;
    /// 物理删除 (W 短 TTL 允许, T 长期 append-only 禁止)
    async fn delete(&self, record_id: Uuid) -> Result<(), MemoryError>;
    /// 层类型
    fn layer(&self) -> MemoryLayer;
}

/// MemoryStore 组合短/长期 (per G.6)
pub struct MemoryStore {
    pub short_term: Arc<ShortTermMemory>,
    pub long_term: Arc<LongTermMemory>,
}

impl MemoryStore {
    pub fn new(short_capacity: usize, long_capacity: u64) -> Self {
        Self {
            short_term: Arc::new(ShortTermMemory::new(short_capacity)),
            long_term: Arc::new(LongTermMemory::new(long_capacity)),
        }
    }

    /// 智能写入: weight >= threshold 进 long_term, 否则 short_term
    pub async fn write_smart(
        &self,
        record: MemoryRecord,
        threshold: f32,
    ) -> Result<(Uuid, MemoryLayer), MemoryError> {
        if record.weight >= threshold {
            let id = self.long_term.write(record).await?;
            Ok((id, MemoryLayer::LongTerm))
        } else {
            let id = self.short_term.write(record).await?;
            Ok((id, MemoryLayer::ShortTerm))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(agent: Uuid, weight: f32) -> MemoryRecord {
        MemoryRecord {
            record_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            agent_id: agent,
            kind: "context".into(),
            content: serde_json::json!({"k": "v"}),
            created_at_ms: 1_700_000_000_000,
            weight,
        }
    }

    #[tokio::test]
    async fn smart_write_threshold() {
        let store = MemoryStore::new(10, 10_000);
        let agent = Uuid::new_v4();
        let (id1, layer1) = store
            .write_smart(make_record(agent, 0.3), 0.5)
            .await
            .unwrap();
        assert_eq!(layer1, MemoryLayer::ShortTerm);
        let (id2, layer2) = store
            .write_smart(make_record(agent, 0.8), 0.5)
            .await
            .unwrap();
        assert_eq!(layer2, MemoryLayer::LongTerm);
        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn short_term_ring_eviction() {
        let st = ShortTermMemory::new(3);
        let agent = Uuid::new_v4();
        for _ in 0..5 {
            st.write(make_record(agent, 0.1)).await.unwrap();
        }
        // 容量 3, 应只剩 3
        assert_eq!(st.list_by_agent(agent, 100).await.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn long_term_no_delete() {
        let lt = LongTermMemory::new(10_000);
        let agent = Uuid::new_v4();
        let r = make_record(agent, 0.8);
        let id = lt.write(r).await.unwrap();
        // T 派生: 物理删除禁止
        let res = lt.delete(id).await;
        assert!(res.is_err());
    }
}
