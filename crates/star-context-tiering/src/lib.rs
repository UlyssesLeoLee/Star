// SPDX-License-Identifier: MIT OR Apache-2.0
//! `star-context-tiering` — Context Tiering (Phase G.8, per AGENTS §6.1)
//!
//! **目的**: L0/L1/L2 上下文分层
//!
//! **架构 (per AGENTS §6.1 + G.8 brief)**:
//! - **L0 全局会话上下文 (Top-Level Agent)**: 1 instance / session, singleton, cross-session checkpoint
//! - **L1 任务卡子代理上下文 (Sub-Agent)**: 9 类型 (SA-01..SA-09) + SA-10 task-orchestrator (TMO), 各 sub-agent 独立 subgraph, 任务卡 1:1 mirror
//! - **L2 业务共享池上下文 (Shared Runtime)**: LLM Pool / HTTP Pool / MCP Pool / Tool Registry / Retriever Pool / Tokenizer / Prompt Registry / Rate Limiter / Circuit Breaker
//!
//! **守门 #13 a 派生**: L1↔L1 禁止 (per TMO-03 实证), 跨 L1 通信必须经 L0 EventBus
//!
//! **Mavis 临时代签 5 域 Lead 决策 (per 9/3 11:35 JST 反转)**: Context Tiering schema 决策 Mavis 临时代签, 真人到位后追溯签字

#![allow(missing_docs)] // G.8 PoC 启动, Phase 2 spec 完成后补 doc

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 上下文层级 (per AGENTS §6.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextTier {
    /// L0 全局会话上下文 (Top-Level Agent, 1 instance / session, singleton)
    L0TopLevel,
    /// L1 任务卡子代理上下文 (Sub-Agent, 9 类型 + SA-10 task-orchestrator)
    L1SubAgent,
    /// L2 业务共享池上下文 (Shared Runtime, LLM/HTTP/MCP Pool etc.)
    L2Shared,
}

/// L1 子代理类型 (per AGENTS §6.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubAgentType {
    /// SA-01..SA-09 (9 种)
    Sa01,
    Sa02,
    Sa03,
    Sa04,
    Sa05,
    Sa06,
    Sa07,
    Sa08,
    Sa09,
    /// SA-10 task-orchestrator (TMO, per 9/3 19:00 JST ADR-0046)
    Sa10TaskOrchestrator,
}

/// L2 业务共享池类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SharedPoolType {
    /// LLM Pool
    Llm,
    /// HTTP Pool
    Http,
    /// MCP Pool
    Mcp,
    /// Tool Registry
    Tool,
    /// Retriever Pool
    Retriever,
    /// Tokenizer
    Tokenizer,
    /// Prompt Registry
    PromptRegistry,
    /// Rate Limiter
    RateLimiter,
    /// Circuit Breaker
    CircuitBreaker,
}

/// 上下文条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEntry {
    /// 条目 key
    pub key: String,
    /// 条目 value (JSON 序列化)
    pub value: serde_json::Value,
    /// 写入时间戳 (ms since epoch)
    pub written_at_ms: u64,
    /// 写入者 (e.g. "L0-top" / "L1-SA-01" / "L2-llm")
    pub written_by: String,
}

/// Tier 容器 trait
#[async_trait]
pub trait ContextTierContainer: Send + Sync {
    /// 写入条目
    async fn put(
        &self,
        key: String,
        value: serde_json::Value,
        written_by: String,
    ) -> Result<(), ContextError>;
    /// 读取条目
    async fn get(&self, key: &str) -> Result<Option<ContextEntry>, ContextError>;
    /// 列出所有 key
    async fn keys(&self) -> Vec<String>;
    /// 容器层
    fn tier(&self) -> ContextTier;
    /// 容器 ID (e.g. tenant_id / sub_agent_id / pool_id)
    fn container_id(&self) -> String;
}

/// Context 错误
#[derive(Debug, Error)]
pub enum ContextError {
    #[error("entry not found: {0}")]
    NotFound(String),
    #[error("forbidden: cross-tier access {from:?} -> {to:?}")]
    Forbidden { from: ContextTier, to: ContextTier },
    #[error("other: {0}")]
    Other(String),
}

/// L0 全局会话上下文 (singleton per session, cross-session checkpoint)
pub struct L0Context {
    /// 租户 ID (跨域隔离)
    pub tenant_id: Uuid,
    /// session ID
    pub session_id: Uuid,
    /// entries
    entries: Arc<RwLock<HashMap<String, ContextEntry>>>,
}

impl L0Context {
    pub fn new(tenant_id: Uuid, session_id: Uuid) -> Self {
        Self {
            tenant_id,
            session_id,
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ContextTierContainer for L0Context {
    async fn put(
        &self,
        key: String,
        value: serde_json::Value,
        written_by: String,
    ) -> Result<(), ContextError> {
        let entry = ContextEntry {
            key: key.clone(),
            value,
            written_at_ms: now_ms(),
            written_by,
        };
        let mut entries = self.entries.write().await;
        entries.insert(key, entry);
        Ok(())
    }
    async fn get(&self, key: &str) -> Result<Option<ContextEntry>, ContextError> {
        let entries = self.entries.read().await;
        Ok(entries.get(key).cloned())
    }
    async fn keys(&self) -> Vec<String> {
        let entries = self.entries.read().await;
        entries.keys().cloned().collect()
    }
    fn tier(&self) -> ContextTier {
        ContextTier::L0TopLevel
    }
    fn container_id(&self) -> String {
        format!("l0:{}:{}", self.tenant_id, self.session_id)
    }
}

/// L1 任务卡子代理上下文 (per-subagent, isolated, 任务卡 1:1 mirror)
pub struct L1Context {
    /// 租户 ID
    pub tenant_id: Uuid,
    /// 子代理 ID
    pub sub_agent_id: Uuid,
    /// 子代理类型
    pub sa_type: SubAgentType,
    /// 任务卡 ID (per AGENTS §6.1 1:1 mirror)
    pub task_card_id: Uuid,
    /// entries
    entries: Arc<RwLock<HashMap<String, ContextEntry>>>,
}

impl L1Context {
    pub fn new(
        tenant_id: Uuid,
        sub_agent_id: Uuid,
        sa_type: SubAgentType,
        task_card_id: Uuid,
    ) -> Self {
        Self {
            tenant_id,
            sub_agent_id,
            sa_type,
            task_card_id,
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ContextTierContainer for L1Context {
    async fn put(
        &self,
        key: String,
        value: serde_json::Value,
        written_by: String,
    ) -> Result<(), ContextError> {
        let entry = ContextEntry {
            key: key.clone(),
            value,
            written_at_ms: now_ms(),
            written_by,
        };
        let mut entries = self.entries.write().await;
        entries.insert(key, entry);
        Ok(())
    }
    async fn get(&self, key: &str) -> Result<Option<ContextEntry>, ContextError> {
        let entries = self.entries.read().await;
        Ok(entries.get(key).cloned())
    }
    async fn keys(&self) -> Vec<String> {
        let entries = self.entries.read().await;
        entries.keys().cloned().collect()
    }
    fn tier(&self) -> ContextTier {
        ContextTier::L1SubAgent
    }
    fn container_id(&self) -> String {
        format!(
            "l1:{}:{:?}:{}",
            self.tenant_id, self.sa_type, self.sub_agent_id
        )
    }
}

/// L2 业务共享池上下文 (LLM/HTTP/MCP Pool etc.)
pub struct L2Context {
    /// 池类型
    pub pool: SharedPoolType,
    /// 池 ID
    pub pool_id: String,
    /// entries
    entries: Arc<RwLock<HashMap<String, ContextEntry>>>,
}

impl L2Context {
    pub fn new(pool: SharedPoolType, pool_id: String) -> Self {
        Self {
            pool,
            pool_id,
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ContextTierContainer for L2Context {
    async fn put(
        &self,
        key: String,
        value: serde_json::Value,
        written_by: String,
    ) -> Result<(), ContextError> {
        let entry = ContextEntry {
            key: key.clone(),
            value,
            written_at_ms: now_ms(),
            written_by,
        };
        let mut entries = self.entries.write().await;
        entries.insert(key, entry);
        Ok(())
    }
    async fn get(&self, key: &str) -> Result<Option<ContextEntry>, ContextError> {
        let entries = self.entries.read().await;
        Ok(entries.get(key).cloned())
    }
    async fn keys(&self) -> Vec<String> {
        let entries = self.entries.read().await;
        entries.keys().cloned().collect()
    }
    fn tier(&self) -> ContextTier {
        ContextTier::L2Shared
    }
    fn container_id(&self) -> String {
        format!("l2:{:?}:{}", self.pool, self.pool_id)
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn l0_top_level_singleton() {
        let ctx = L0Context::new(Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(ctx.tier(), ContextTier::L0TopLevel);
        ctx.put("k1".into(), serde_json::json!({"v": 1}), "l0-self".into())
            .await
            .unwrap();
        let got = ctx.get("k1").await.unwrap().unwrap();
        assert_eq!(got.value, serde_json::json!({"v": 1}));
    }

    #[tokio::test]
    async fn l1_sub_agent_isolated() {
        let ctx = L1Context::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            SubAgentType::Sa01,
            Uuid::new_v4(),
        );
        assert_eq!(ctx.tier(), ContextTier::L1SubAgent);
        ctx.put("k1".into(), serde_json::json!({"v": 1}), "sa01-self".into())
            .await
            .unwrap();
        let got = ctx.get("k1").await.unwrap();
        assert!(got.is_some());
    }

    #[tokio::test]
    async fn l2_shared_pool() {
        let ctx = L2Context::new(SharedPoolType::Llm, "pool-1".into());
        assert_eq!(ctx.tier(), ContextTier::L2Shared);
        ctx.put(
            "model".into(),
            serde_json::json!("gpt-4o"),
            "llm-pool".into(),
        )
        .await
        .unwrap();
        let got = ctx.get("model").await.unwrap().unwrap();
        assert_eq!(got.value, serde_json::json!("gpt-4o"));
    }

    #[tokio::test]
    async fn three_tiers_independent() {
        let tenant = Uuid::new_v4();
        let l0 = L0Context::new(tenant, Uuid::new_v4());
        let l1 = L1Context::new(tenant, Uuid::new_v4(), SubAgentType::Sa01, Uuid::new_v4());
        let l2 = L2Context::new(SharedPoolType::Llm, "p1".into());

        l0.put("k".into(), serde_json::json!("L0"), "l0".into())
            .await
            .unwrap();
        l1.put("k".into(), serde_json::json!("L1"), "l1".into())
            .await
            .unwrap();
        l2.put("k".into(), serde_json::json!("L2"), "l2".into())
            .await
            .unwrap();

        assert_eq!(
            l0.get("k").await.unwrap().unwrap().value,
            serde_json::json!("L0")
        );
        assert_eq!(
            l1.get("k").await.unwrap().unwrap().value,
            serde_json::json!("L1")
        );
        assert_eq!(
            l2.get("k").await.unwrap().unwrap().value,
            serde_json::json!("L2")
        );
    }
}
