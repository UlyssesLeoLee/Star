// SPDX-License-Identifier: MIT OR Apache-2.0
//! V2EdgeSink = ARG Schema V2 edge sink (per DDD-REVIEW §1.3 G-10 拍板)
//!
//! v0.82 阶段 1 双写 (per G-10 4 阶段渐进式迁移):
//! - 阶段 1 (2 周): V1 + V2 schema 并存, 双写 (V1 旧 path + V2 新 path)
//! - 阶段 2 (2 周): 读路径全部走 V2, 写路径继续双写
//! - 阶段 3 (1 周): 写路径切到 V2 only, V1 表 mark read-only
//! - 阶段 4 (1 周): V1 表 archive 到 _archive schema
//!
//! Ref: scripts/automation/arg_migration.py v1.0 (V1Edge → V2Edge 转换)

use crate::error::ARGError;
use crate::models::edge::Edge;
use std::sync::Arc;
use uuid::Uuid;

/// V2EdgeSink = V2 schema edge 写入目标 (per G-10 拍板 §1.3)
///
/// v0.82 阶段 1 双写: EdgeOps::create() 同时写 V1 + V2 sink
/// v0.83 阶段 2 V2 优先: EdgeOps::get/list 走 V2 sink (v0.83 跨 session 续)
/// v0.85 阶段 4 V1 退役: EdgeOps 停止写 V1 sink
pub trait V2EdgeSink: std::fmt::Debug + Send + Sync {
    /// 追加 V2 edge (per G-10 拍板 §1.3 stage 1)
    fn append_v2(&self, v1_edge: &Edge) -> Result<(), ARGError>;

    /// 根据 ID 查询 V2 edge
    fn get_v2(&self, id: Uuid) -> Result<Option<Edge>, ARGError>;

    /// 列出所有 V2 edge
    fn list_v2(&self) -> Result<Vec<Edge>, ARGError>;
}

/// InMemoryV2EdgeSink = 内存版 V2 edge sink (v0.82 阶段, per 守门 #11 缺标比错标 P3 跨 session 续)
///
/// 实际生产应该换成 MemgraphV2EdgeSink (per r2d2-memgraph G-1 落地)
#[derive(Debug, Clone, Default)]
pub struct InMemoryV2EdgeSink {
    edges: Arc<std::sync::Mutex<Vec<Edge>>>,
}

impl InMemoryV2EdgeSink {
    /// Create a new in-memory V2 edge sink
    pub fn new() -> Self {
        Self::default()
    }
}

impl V2EdgeSink for InMemoryV2EdgeSink {
    fn append_v2(&self, v1_edge: &Edge) -> Result<(), ARGError> {
        // v0.82 阶段: V1 edge 写到 V2 sink (V2 schema 字段映射由 caller 处理)
        // 实际生产: V2 schema 字段 (edge_type_v2 + v1_migration_status) 由 arg_migration.py
        // 处理, 这里仅保留 V1 edge 引用
        let mut edges = self.edges.lock().expect("InMemoryV2EdgeSink lock poisoned");
        edges.push(v1_edge.clone());
        Ok(())
    }

    fn get_v2(&self, id: Uuid) -> Result<Option<Edge>, ARGError> {
        let edges = self.edges.lock().expect("InMemoryV2EdgeSink lock poisoned");
        Ok(edges.iter().find(|e| e.id == id).cloned())
    }

    fn list_v2(&self) -> Result<Vec<Edge>, ARGError> {
        let edges = self.edges.lock().expect("InMemoryV2EdgeSink lock poisoned");
        Ok(edges.clone())
    }
}
