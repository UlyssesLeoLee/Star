// ShortTermMemory: W 派生 (per 守门 #13 a 短 TTL 作业中, 物理删除 OK, ring buffer 自动 eviction)
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{Memory, MemoryError, MemoryLayer, MemoryRecord};

pub struct ShortTermMemory {
    /// agent_id -> records (ring buffer)
    by_agent: Arc<RwLock<HashMap<Uuid, VecDeque<MemoryRecord>>>>,
    /// 容量 (per agent)
    capacity: usize,
}

impl ShortTermMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            by_agent: Arc::new(RwLock::new(HashMap::new())),
            capacity,
        }
    }
}

#[async_trait]
impl Memory for ShortTermMemory {
    async fn write(&self, record: MemoryRecord) -> Result<Uuid, MemoryError> {
        let id = record.record_id;
        let agent = record.agent_id;
        let mut by_agent = self.by_agent.write().await;
        let buf = by_agent
            .entry(agent)
            .or_insert_with(|| VecDeque::with_capacity(self.capacity));
        // ring buffer: 超容量则 evict 最早 (front)
        if buf.len() >= self.capacity {
            buf.pop_front();
        }
        buf.push_back(record);
        Ok(id)
    }

    async fn read(&self, record_id: Uuid) -> Result<MemoryRecord, MemoryError> {
        let by_agent = self.by_agent.read().await;
        for buf in by_agent.values() {
            for r in buf {
                if r.record_id == record_id {
                    return Ok(r.clone());
                }
            }
        }
        Err(MemoryError::NotFound(record_id))
    }

    async fn list_by_agent(
        &self,
        agent_id: Uuid,
        limit: u32,
    ) -> Result<Vec<MemoryRecord>, MemoryError> {
        let by_agent = self.by_agent.read().await;
        let mut out: Vec<MemoryRecord> = by_agent
            .get(&agent_id)
            .map(|buf| buf.iter().cloned().collect())
            .unwrap_or_default();
        out.sort_by(|a, b| {
            b.weight
                .partial_cmp(&a.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out.truncate(limit as usize);
        Ok(out)
    }

    async fn delete(&self, record_id: Uuid) -> Result<(), MemoryError> {
        // W 派生: 物理删除 OK
        let mut by_agent = self.by_agent.write().await;
        for buf in by_agent.values_mut() {
            buf.retain(|r| r.record_id != record_id);
        }
        Ok(())
    }

    fn layer(&self) -> MemoryLayer {
        MemoryLayer::ShortTerm
    }
}
