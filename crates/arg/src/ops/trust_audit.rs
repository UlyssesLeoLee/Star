// SPDX-License-Identifier: MIT OR Apache-2.0
//! TrustAuditOps = trusts 关系 4 重审计 (per DDD-REVIEW-AGENT-RELATIONSHIP-001 §1.2 G-4 拍板)
//!
//! 4 重审计:
//! 1. trust_score 阈值 0.95 (per G-4 拍板 (a))
//! 2. mutual trust 验证 (per G-4 拍板 (b).1)
//! 3. historical evidence 30 天 ≥ 10 次 (per G-4 拍板 (b).2)
//! 4. multi-source attestation ≥ 2 个独立源头 (per G-4 拍板 (b).3)
//! + WORM audit log (per G-4 拍板 (c) + 守门 #13 d)
//! + 7 天撤回窗口 (per G-4 拍板 (d))
//!
//! G-4 生产化 (per v0.78 跨 session 续 + v0.79 sink 抽象):
//! - AuditEventSink trait 抽象 WORM append-only audit log
//! - InMemoryAuditEventSink (v0.1 阶段, per 守门 #11 缺标比错标 P3 跨 session 续)
//! - AuditEventTableSink (生产, per ADR-0043 audit_audit_event 表) 跨 session 续
//!
//! Ref: DDD-REVIEW-AGENT-RELATIONSHIP-001 §1.2 + ADR-0043 audit_audit_event WORM

use crate::client::MemgraphClient;
use crate::error::ARGError;
use crate::models::{Edge, RelationshipType, TrustAuditLog};
use std::sync::Arc;
use uuid::Uuid;

/// AuditEventSink = audit log 写入目标 (per G-4 拍板 (c) WORM append-only)
///
/// v0.1 默认 InMemoryAuditEventSink (测试 + dev 环境)
/// 生产应该用 AuditEventTableSink (per ADR-0043 audit_audit_event 表, 跨 session 续)
pub trait AuditEventSink: std::fmt::Debug + Send + Sync {
    /// 追加一个 TrustAuditLog (WORM append-only, per 守门 #13 d)
    fn append(&self, log: &TrustAuditLog) -> Result<(), ARGError>;

    /// 根据 ID 查询 TrustAuditLog
    fn get(&self, id: Uuid) -> Result<Option<TrustAuditLog>, ARGError>;

    /// 列出所有 TrustAuditLog
    fn list(&self) -> Result<Vec<TrustAuditLog>, ARGError>;

    /// 标记 TrustAuditLog 为已撤销 (per G-4 拍板 (d) 7 天撤回窗口)
    fn revoke(&self, id: Uuid) -> Result<(), ARGError>;
}

/// InMemoryAuditEventSink = 内存版 audit log (v0.1 阶段, per 守门 #11 缺标比错标 P3 跨 session 续)
///
/// 生产应该替换为 AuditEventTableSink (per ADR-0043 audit_audit_event 表)
#[derive(Debug, Clone, Default)]
pub struct InMemoryAuditEventSink {
    logs: Arc<std::sync::Mutex<Vec<TrustAuditLog>>>,
}

impl InMemoryAuditEventSink {
    /// Create a new in-memory audit event sink
    pub fn new() -> Self {
        Self::default()
    }
}

/// AuditEventTableSink = 真实 audit_audit_event 表 sink (per ADR-0043 WORM append-only)
///
/// v0.80 阶段: 提供生产接口 + 内部 in-memory buffer (per 守门 #11 缺标比错标 P3 跨 session 续
///             等 G-1 r2d2-memgraph 实装后再切到真实 Memgraph Bolt)
///
/// 写路径: append() 内部 buffer 写入 (跟 InMemoryAuditEventSink 一样)
///         G-1 落地后切到 client.execute_write(cypher) 写 audit_audit_event 表
/// 读路径: get/list 从 buffer 读
/// 撤销: revoke 标记 buffer 中 log 的 revoked 字段
/// Cypher queries (cypher_append/cypher_get/cypher_revoke) 提供生产路径, 等待 G-1 落地
#[derive(Debug, Clone)]
pub struct AuditEventTableSink {
    #[allow(dead_code)] // 等 G-1 落地后用, 现在仅保留 client 引用
    client: Arc<MemgraphClient>,
    /// In-memory buffer (等 G-1 落地后改 client.execute_write)
    buffer: Arc<std::sync::Mutex<Vec<TrustAuditLog>>>,
}

impl AuditEventTableSink {
    /// Build a new AuditEventTableSink
    pub fn new(client: Arc<MemgraphClient>) -> Self {
        Self {
            client,
            buffer: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Cypher: 写 audit_audit_event 表 (per ADR-0043 WORM append-only + 守门 #13 d)
    /// 等 G-1 落地后, append() 切到调此 cypher
    #[allow(dead_code)]
    fn cypher_append(log: &TrustAuditLog) -> String {
        format!(
            "CREATE (a:AuditEvent:TrustAuditLog {{ \
             id: '{}', \
             from_agent_id: '{}', \
             to_agent_id: '{}', \
             tenant_id: '{}', \
             skip_reason: '{}', \
             trust_score: {}, \
             mutual_trust_verified: {}, \
             historical_evidence_count: {}, \
             multi_source_count: {}, \
             audited_at: '{}', \
             revocation_window_until: '{}', \
             revoked: {} \
             }})",
            log.id,
            log.from_agent_id,
            log.to_agent_id,
            log.tenant_id,
            log.skip_reason,
            log.trust_score,
            log.mutual_trust_verified,
            log.historical_evidence_count,
            log.multi_source_count,
            log.audited_at.to_rfc3339(),
            log.revocation_window_until.to_rfc3339(),
            log.revoked,
        )
    }

    /// Cypher: 按 ID 查询 (per G-4 拍板 (c) 审计查询)
    #[allow(dead_code)]
    fn cypher_get(id: Uuid) -> String {
        format!(
            "MATCH (a:AuditEvent:TrustAuditLog {{id: '{}'}}) RETURN a",
            id
        )
    }

    /// Cypher: 撤销 (per G-4 拍板 (d) 7 天撤回窗口)
    #[allow(dead_code)]
    fn cypher_revoke(id: Uuid) -> String {
        format!(
            "MATCH (a:AuditEvent:TrustAuditLog {{id: '{}'}}) SET a.revoked = true",
            id
        )
    }
}

impl AuditEventSink for AuditEventTableSink {
    fn append(&self, log: &TrustAuditLog) -> Result<(), ARGError> {
        // v0.80 阶段: 写 buffer (per 守门 #11 缺标比错标 P3 跨 session 续)
        // G-1 r2d2-memgraph 落地后: 改成 client.execute_write(cypher_append(log))
        let mut buffer = self.buffer.lock().expect("AuditEventTableSink buffer lock poisoned");
        buffer.push(log.clone());
        Ok(())
    }

    fn get(&self, id: Uuid) -> Result<Option<TrustAuditLog>, ARGError> {
        // v0.80: 从 buffer 读
        // G-1 落地后: client.execute(cypher_get(id)) 读 Bolt
        let buffer = self.buffer.lock().expect("AuditEventTableSink buffer lock poisoned");
        Ok(buffer.iter().find(|l| l.id == id).cloned())
    }

    fn list(&self) -> Result<Vec<TrustAuditLog>, ARGError> {
        // v0.80: 从 buffer 读
        // G-1 落地后: client.execute(cypher_list_all) 读 Bolt
        let buffer = self.buffer.lock().expect("AuditEventTableSink buffer lock poisoned");
        Ok(buffer.clone())
    }

    fn revoke(&self, id: Uuid) -> Result<(), ARGError> {
        // v0.80: 更新 buffer
        // G-1 落地后: client.execute_write(cypher_revoke(id))
        let mut buffer = self.buffer.lock().expect("AuditEventTableSink buffer lock poisoned");
        for log in buffer.iter_mut() {
            if log.id == id {
                log.revoked = true;
                return Ok(());
            }
        }
        Err(ARGError::Other(format!("TrustAuditLog {} 未找到", id)))
    }
}

impl AuditEventSink for InMemoryAuditEventSink {
    fn append(&self, log: &TrustAuditLog) -> Result<(), ARGError> {
        let mut logs = self.logs.lock().expect("InMemoryAuditEventSink lock poisoned");
        logs.push(log.clone());
        Ok(())
    }

    fn get(&self, id: Uuid) -> Result<Option<TrustAuditLog>, ARGError> {
        let logs = self.logs.lock().expect("InMemoryAuditEventSink lock poisoned");
        Ok(logs.iter().find(|l| l.id == id).cloned())
    }

    fn list(&self) -> Result<Vec<TrustAuditLog>, ARGError> {
        let logs = self.logs.lock().expect("InMemoryAuditEventSink lock poisoned");
        Ok(logs.clone())
    }

    fn revoke(&self, id: Uuid) -> Result<(), ARGError> {
        let mut logs = self.logs.lock().expect("InMemoryAuditEventSink lock poisoned");
        for log in logs.iter_mut() {
            if log.id == id {
                log.revoked = true;
                return Ok(());
            }
        }
        Err(ARGError::Other(format!("TrustAuditLog {} 未找到", id)))
    }
}

/// TrustAuditOps = trusts 关系 4 重审计操作
///
/// 包装 EdgeOps, 仅对 `RelationshipType::Trusts` 关系应用 4 重审计
#[derive(Debug, Clone)]
pub struct TrustAuditOps {
    /// WORM audit log sink (per G-4 拍板 (c) + 守门 #13 d)
    /// v0.1 默认 InMemoryAuditEventSink, 生产可换 AuditEventTableSink
    sink: Arc<dyn AuditEventSink>,
}

impl TrustAuditOps {
    /// Build a new TrustAuditOps with default InMemoryAuditEventSink
    pub fn new() -> Self {
        Self {
            sink: Arc::new(InMemoryAuditEventSink::new()),
        }
    }

    /// Build a new TrustAuditOps with custom sink (per G-4 生产化 v0.79)
    pub fn with_sink(sink: Arc<dyn AuditEventSink>) -> Self {
        Self { sink }
    }

    /// 验证 trusts 关系是否符合 4 重审计 (per G-4 拍板 §1.2 (a)(b))
    ///
    /// Returns Ok(TrustAuditLog) if all 4 audits pass, Err(ARGError) otherwise
    pub fn audit_trust_relationship(
        &self,
        edge: &Edge,
    ) -> Result<TrustAuditLog, ARGError> {
        // 仅对 Trusts 关系应用 4 重审计
        if edge.edge_type != RelationshipType::Trusts {
            return Err(ARGError::Other(
                "TrustAuditOps::audit_trust_relationship 仅支持 Trusts 关系".into(),
            ));
        }

        // 1. trust_score 阈值检查 (per G-4 拍板 (a) 阈值 0.95)
        if edge.weight < 0.95 {
            return Err(ARGError::Other(format!(
                "trust_score {} < 0.95 阈值 (G-4 拍板 a)",
                edge.weight
            )));
        }

        // 2. mutual trust 验证 (per G-4 拍板 (b).1)
        let mutual_trust_verified = edge.metadata.get("mutual_trust")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !mutual_trust_verified {
            return Err(ARGError::Other(
                "mutual trust 未验证 (A trusts B AND B trusts A, G-4 拍板 b.1)".into(),
            ));
        }

        // 3. historical evidence 检查 (per G-4 拍板 (b).2 过去 30 天 ≥ 10 次)
        let historical_evidence_count = edge.metadata.get("historical_evidence_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        if historical_evidence_count < 10 {
            return Err(ARGError::Other(format!(
                "historical evidence count {} < 10 阈值 (G-4 拍板 b.2)",
                historical_evidence_count
            )));
        }

        // 4. multi-source attestation 检查 (per G-4 拍板 (b).3 ≥ 2 个独立源头)
        let multi_source_count = edge.metadata.get("multi_source_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        if multi_source_count < 2 {
            return Err(ARGError::Other(format!(
                "multi-source attestation count {} < 2 阈值 (G-4 拍板 b.3)",
                multi_source_count
            )));
        }

        // 创建 WORM audit log (per G-4 拍板 (c) + 守门 #13 d)
        let log = TrustAuditLog::new(
            edge.from_agent,
            edge.to_agent,
            edge.tenant_id,
            "all_4_audit_pass".to_string(),
            edge.weight as f64,
            mutual_trust_verified,
            historical_evidence_count,
            multi_source_count,
        );

        // 强制写 WORM audit log (per G-4 拍板 (c)) 通过 sink
        self.sink.append(&log)?;
        // 实际生产: AuditEventTableSink 同步写 audit_audit_event 表 (per ADR-0043)
        // 当前 v0.1: InMemoryAuditEventSink (per 守门 #11 缺标比错标 P3 跨 session 续)

        Ok(log)
    }

    /// 撤销 trusts 关系 (per G-4 拍板 (d) 7 天撤回窗口)
    pub fn revoke_trust(&self, audit_log_id: Uuid) -> Result<(), ARGError> {
        // 7 天撤回窗口检查
        if let Some(log) = self.sink.get(audit_log_id)? {
            if !log.is_within_revocation_window() {
                return Err(ARGError::Other(
                    "7 天撤回窗口已过, 不可撤销".into(),
                ));
            }
        }
        self.sink.revoke(audit_log_id)
    }

    /// 列出所有 audit log (per G-4 拍板 (c) 审计查询)
    pub fn list_audit_logs(&self) -> Vec<TrustAuditLog> {
        self.sink.list().unwrap_or_default()
    }
}

impl Default for TrustAuditOps {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::edge::EdgeDirection;
    use chrono::Utc;
    use serde_json::json;

    fn make_trust_edge(weight: f32, metadata: serde_json::Value) -> Edge {
        Edge {
            id: Uuid::new_v4(),
            from_agent: Uuid::new_v4(),
            to_agent: Uuid::new_v4(),
            edge_type: RelationshipType::Trusts,
            direction: EdgeDirection::Directed,
            weight,
            archived: false,
            metadata,
            tenant_id: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: 1,
            created_by: Uuid::new_v4(),
        }
    }

    #[test]
    fn trust_audit_passes_all_4() {
        let ops = TrustAuditOps::new();
        let edge = make_trust_edge(0.99_f32, json!({
            "mutual_trust": true,
            "historical_evidence_count": 10,
            "multi_source_count": 2,
        }));
        let log = ops.audit_trust_relationship(&edge).unwrap();
        assert!(log.passes_4_audit());
        assert_eq!(ops.list_audit_logs().len(), 1);
    }

    #[test]
    fn trust_audit_fails_low_trust_score() {
        let ops = TrustAuditOps::new();
        let edge = make_trust_edge(0.94_f32, json!({
            "mutual_trust": true,
            "historical_evidence_count": 10,
            "multi_source_count": 2,
        }));
        let result = ops.audit_trust_relationship(&edge);
        assert!(result.is_err());
    }

    #[test]
    fn trust_audit_fails_no_mutual() {
        let ops = TrustAuditOps::new();
        let edge = make_trust_edge(0.99_f32, json!({
            "mutual_trust": false,
            "historical_evidence_count": 10,
            "multi_source_count": 2,
        }));
        let result = ops.audit_trust_relationship(&edge);
        assert!(result.is_err());
    }

    #[test]
    fn trust_audit_fails_low_evidence() {
        let ops = TrustAuditOps::new();
        let edge = make_trust_edge(0.99_f32, json!({
            "mutual_trust": true,
            "historical_evidence_count": 9,
            "multi_source_count": 2,
        }));
        let result = ops.audit_trust_relationship(&edge);
        assert!(result.is_err());
    }

    #[test]
    fn trust_audit_fails_low_multi_source() {
        let ops = TrustAuditOps::new();
        let edge = make_trust_edge(0.99_f32, json!({
            "mutual_trust": true,
            "historical_evidence_count": 10,
            "multi_source_count": 1,
        }));
        let result = ops.audit_trust_relationship(&edge);
        assert!(result.is_err());
    }

    #[test]
    fn trust_audit_rejects_non_trust_relationship() {
        let ops = TrustAuditOps::new();
        let mut edge = make_trust_edge(0.99_f32, json!({}));
        edge.edge_type = RelationshipType::CollaboratesWith;
        let result = ops.audit_trust_relationship(&edge);
        assert!(result.is_err());
    }

    #[test]
    fn trust_audit_revoke_within_window() {
        let ops = TrustAuditOps::new();
        let edge = make_trust_edge(0.99_f32, json!({
            "mutual_trust": true,
            "historical_evidence_count": 10,
            "multi_source_count": 2,
        }));
        let log = ops.audit_trust_relationship(&edge).unwrap();
        ops.revoke_trust(log.id).unwrap();
        let logs = ops.list_audit_logs();
        assert!(logs[0].revoked);
    }

    // G-4 生产化新测试 (per v0.79 sink 抽象)
    #[test]
    fn trust_audit_with_custom_sink() {
        let custom_sink = Arc::new(InMemoryAuditEventSink::new());
        let ops = TrustAuditOps::with_sink(custom_sink.clone());

        let edge = make_trust_edge(0.99_f32, json!({
            "mutual_trust": true,
            "historical_evidence_count": 10,
            "multi_source_count": 2,
        }));
        let log = ops.audit_trust_relationship(&edge).unwrap();
        assert!(log.passes_4_audit());

        // 自定义 sink 应该收到 audit log
        let logs = custom_sink.list().unwrap();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].passes_4_audit());
    }

    #[test]
    fn in_memory_sink_revoke_marks_revoked() {
        let sink = InMemoryAuditEventSink::new();
        let ops = TrustAuditOps::with_sink(Arc::new(sink));

        let edge = make_trust_edge(0.99_f32, json!({
            "mutual_trust": true,
            "historical_evidence_count": 10,
            "multi_source_count": 2,
        }));
        let log = ops.audit_trust_relationship(&edge).unwrap();
        ops.revoke_trust(log.id).unwrap();

        let fetched = ops.sink.get(log.id).unwrap().unwrap();
        assert!(fetched.revoked);
    }

    // v0.80 AuditEventTableSink 测试 (per G-4 生产化 跨 session 续)
    #[test]
    fn audit_event_table_sink_appends() {
        // 创建 stub MemgraphClient (不连真实 Memgraph, 仅走 stub 路径)
        let client = Arc::new(MemgraphClient::new(
            "bolt://stub:7687".to_string(),
            "stub".to_string(),
            "stub".to_string(),
            1,
        ));
        let sink = Arc::new(AuditEventTableSink::new(client));
        let ops = TrustAuditOps::with_sink(sink.clone());

        let edge = make_trust_edge(0.99_f32, json!({
            "mutual_trust": true,
            "historical_evidence_count": 10,
            "multi_source_count": 2,
        }));
        let log = ops.audit_trust_relationship(&edge).unwrap();
        assert!(log.passes_4_audit());

        // 验证: AuditEventTableSink 收到 audit log
        let logs = sink.list().unwrap();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].passes_4_audit());
    }

    #[test]
    fn audit_event_table_sink_cypher_format() {
        // 验证 cypher_append 生成的 cypher 包含必要字段 (per SCHEMA_V2_AUDIT_STATEMENTS 9 statements)
        let log = TrustAuditLog::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "test_4_audit_pass".to_string(),
            0.99,
            true,
            10,
            2,
        );
        let cypher = AuditEventTableSink::cypher_append(&log);
        assert!(cypher.contains("CREATE"));
        assert!(cypher.contains(":AuditEvent:TrustAuditLog"));
        assert!(cypher.contains("id:"));
        assert!(cypher.contains("trust_score:"));
        assert!(cypher.contains("revoked:"));
    }
}
