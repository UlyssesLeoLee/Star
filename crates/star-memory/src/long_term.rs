// LongTermMemory: T 派生 (per 守门 #13 b append-only, 物理删除禁止, RLS 13 類必携)
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{Memory, MemoryError, MemoryLayer, MemoryRecord};

pub struct LongTermMemory {
    /// record_id -> MemoryRecord
    by_id: Arc<RwLock<HashMap<Uuid, MemoryRecord>>>,
    /// agent_id -> record_ids (用于 list_by_agent)
    by_agent: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
    /// 容量上限 (字节, 0 = 无限)
    capacity_bytes: u64,
    /// 已使用字节
    used_bytes: Arc<RwLock<u64>>,
}

impl LongTermMemory {
    pub fn new(capacity_bytes: u64) -> Self {
        Self {
            by_id: Arc::new(RwLock::new(HashMap::new())),
            by_agent: Arc::new(RwLock::new(HashMap::new())),
            capacity_bytes,
            used_bytes: Arc::new(RwLock::new(0)),
        }
    }

    fn estimate_size(r: &MemoryRecord) -> u64 {
        // 粗略估算 (JSON 长度)
        serde_json::to_string(r)
            .map(|s| s.len() as u64)
            .unwrap_or(0)
    }
}

#[async_trait]
impl Memory for LongTermMemory {
    async fn write(&self, record: MemoryRecord) -> Result<Uuid, MemoryError> {
        let id = record.record_id;
        let agent = record.agent_id;
        let size = Self::estimate_size(&record);
        if self.capacity_bytes > 0 {
            let mut used = self.used_bytes.write().await;
            if *used + size > self.capacity_bytes {
                return Err(MemoryError::Storage(format!(
                    "long_term capacity exceeded: used={}, cap={}, new={}",
                    *used, self.capacity_bytes, size
                )));
            }
            *used += size;
        }
        let mut by_id = self.by_id.write().await;
        let mut by_agent = self.by_agent.write().await;
        by_id.insert(id, record);
        by_agent.entry(agent).or_default().push(id);
        Ok(id)
    }

    async fn read(&self, record_id: Uuid) -> Result<MemoryRecord, MemoryError> {
        let by_id = self.by_id.read().await;
        by_id
            .get(&record_id)
            .cloned()
            .ok_or(MemoryError::NotFound(record_id))
    }

    async fn list_by_agent(
        &self,
        agent_id: Uuid,
        limit: u32,
    ) -> Result<Vec<MemoryRecord>, MemoryError> {
        let by_id = self.by_id.read().await;
        let by_agent = self.by_agent.read().await;
        let mut out: Vec<MemoryRecord> = by_agent
            .get(&agent_id)
            .map(|ids| ids.iter().filter_map(|i| by_id.get(i).cloned()).collect())
            .unwrap_or_default();
        out.sort_by(|a, b| {
            b.weight
                .partial_cmp(&a.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out.truncate(limit as usize);
        Ok(out)
    }

    async fn delete(&self, _record_id: Uuid) -> Result<(), MemoryError> {
        // T 派生: 物理删除禁止 (per 守门 #13 b)
        Err(MemoryError::Storage(
            "LongTermMemory 不允许物理删除, per 守门 #13 b T 派生".into(),
        ))
    }

    fn layer(&self) -> MemoryLayer {
        MemoryLayer::LongTerm
    }
}
