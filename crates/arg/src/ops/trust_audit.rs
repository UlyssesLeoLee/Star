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
//! Ref: DDD-REVIEW-AGENT-RELATIONSHIP-001 §1.2

use crate::error::ARGError;
use crate::models::{Edge, RelationshipType, TrustAuditLog};
use std::sync::Arc;
use uuid::Uuid;

/// TrustAuditOps = trusts 关系 4 重审计操作
///
/// 包装 EdgeOps, 仅对 `RelationshipType::Trusts` 关系应用 4 重审计
#[derive(Debug, Clone)]
pub struct TrustAuditOps {
    /// In-memory audit log buffer (per G-4 拍板 (c) WORM append-only)
    /// 实际生产应该用 audit_audit_event 表 + append-only storage (per ADR-0043)
    audit_logs: Arc<std::sync::Mutex<Vec<TrustAuditLog>>>,
}

impl TrustAuditOps {
    /// Build a new TrustAuditOps
    pub fn new() -> Self {
        Self {
            audit_logs: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
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
        //    A trusts B AND B trusts A 才允许
        //    (这里用 self_flag 占位, 实际生产需要反向查询 EdgeOps)
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

        // 强制写 WORM audit log (per G-4 拍板 (c))
        let mut logs = self.audit_logs.lock().expect("TrustAuditOps lock poisoned");
        logs.push(log.clone());
        // 实际生产应该同步写 audit_audit_event 表 (WORM append-only per ADR-0043)
        // 这里 in-memory buffer 0.1.0 阶段足够 (per 守门 #11 缺标比错标 P3 跨 session 续)

        Ok(log)
    }

    /// 撤销 trusts 关系 (per G-4 拍板 (d) 7 天撤回窗口)
    pub fn revoke_trust(&self, audit_log_id: Uuid) -> Result<(), ARGError> {
        let mut logs = self.audit_logs.lock().expect("TrustAuditOps lock poisoned");
        for log in logs.iter_mut() {
            if log.id == audit_log_id {
                if !log.is_within_revocation_window() {
                    return Err(ARGError::Other(
                        "7 天撤回窗口已过, 不可撤销".into(),
                    ));
                }
                log.revoked = true;
                return Ok(());
            }
        }
        return Err(ARGError::Other(format!(
            "TrustAuditLog {} 未找到",
            audit_log_id
        )));
    }

    /// 列出所有 audit log (per G-4 拍板 (c) 审计查询)
    pub fn list_audit_logs(&self) -> Vec<TrustAuditLog> {
        let logs = self.audit_logs.lock().expect("TrustAuditOps lock poisoned");
        logs.clone()
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
        assert!(ops.list_audit_logs().len() == 1);
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
        let edge = make_trust_edge(0.95_f32, json!({
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
        let edge = make_trust_edge(0.95_f32, json!({
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
        let edge = make_trust_edge(0.95_f32, json!({
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
        let mut edge = make_trust_edge(0.95_f32, json!({}));
        edge.edge_type = RelationshipType::CollaboratesWith;
        let result = ops.audit_trust_relationship(&edge);
        assert!(result.is_err());
    }

    #[test]
    fn trust_audit_revoke_within_window() {
        let ops = TrustAuditOps::new();
        let edge = make_trust_edge(0.95_f32, json!({
            "mutual_trust": true,
            "historical_evidence_count": 10,
            "multi_source_count": 2,
        }));
        let log = ops.audit_trust_relationship(&edge).unwrap();
        ops.revoke_trust(log.id).unwrap();
        let logs = ops.list_audit_logs();
        assert!(logs[0].revoked);
    }
}
