// SPDX-License-Identifier: MIT OR Apache-2.0
//! TrustAuditLog 共享类型 (per DDD-REVIEW-AGENT-RELATIONSHIP-001 §1.2 G-4 拍板)
//!
//! 4 重审计: trust_score 阈值 0.95 + mutual + evidence + multi-source attestation
//! + WORM audit log (per ADR-0043) + 7 天撤回窗口
//!
//! Ref: DDD-REVIEW-AGENT-RELATIONSHIP-001 §1.2 + ADR-0043 audit_audit_event WORM

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// TrustAuditLog = trusts 跳过 verify 的审计记录 (per G-4 拍板 (c))
///
/// 强制写 audit_audit_event 表 (WORM append-only per ADR-0043 + 守门 #13 d Transaction 100% audit)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustAuditLog {
    /// Audit log UUID
    pub id: Uuid,
    /// Source agent (truster)
    pub from_agent_id: Uuid,
    /// Target agent (trustee)
    pub to_agent_id: Uuid,
    /// Tenant context (守门 #13 c Master 100% RLS)
    pub tenant_id: Uuid,
    /// Skip verify 原因 (e.g. "high_trust_score" / "mutual_trust" / "historical_evidence" / "multi_source")
    pub skip_reason: String,
    /// Trust score at creation time (per G-4 拍板 (a) 阈值 0.95)
    pub trust_score: f64,
    /// 是否 mutual trust (A trusts B AND B trusts A, per G-4 拍板 (b).1)
    pub mutual_trust_verified: bool,
    /// Historical evidence count (过去 30 天真实协作记录, per G-4 拍板 (b).2)
    pub historical_evidence_count: u32,
    /// Multi-source attestation count (≥ 2 个独立源头, per G-4 拍板 (b).3)
    pub multi_source_count: u32,
    /// Audit timestamp (UTC, per ADR-0043 WORM append-only)
    pub audited_at: DateTime<Utc>,
    /// 撤回窗口到期时间 (per G-4 拍板 (d) 7 天)
    pub revocation_window_until: DateTime<Utc>,
    /// 是否已撤销 (true = 已被撤回, trust_score 降回 0.5)
    pub revoked: bool,
}

impl TrustAuditLog {
    /// 创建新的 TrustAuditLog (G-4 拍板 (c) 强制写 WORM audit log)
    pub fn new(
        from_agent_id: Uuid,
        to_agent_id: Uuid,
        tenant_id: Uuid,
        skip_reason: String,
        trust_score: f64,
        mutual_trust_verified: bool,
        historical_evidence_count: u32,
        multi_source_count: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            from_agent_id,
            to_agent_id,
            tenant_id,
            skip_reason,
            trust_score,
            mutual_trust_verified,
            historical_evidence_count,
            multi_source_count,
            audited_at: now,
            // 7 天撤回窗口 (per G-4 拍板 (d))
            revocation_window_until: now + chrono::Duration::days(7),
            revoked: false,
        }
    }

    /// 检查是否在 7 天撤回窗口内 (per G-4 拍板 (d))
    pub fn is_within_revocation_window(&self) -> bool {
        Utc::now() < self.revocation_window_until
    }

    /// 检查 4 重审计是否全部通过 (per G-4 拍板 (a)(b))
    ///
    /// 满足 4 条件:
    /// 1. trust_score >= 0.95 (G-4 拍板 (a) 阈值收紧)
    /// 2. mutual_trust_verified = true (G-4 拍板 (b).1 mutual trust)
    /// 3. historical_evidence_count >= 10 (G-4 拍板 (b).2 过去 30 天真实协作记录)
    /// 4. multi_source_count >= 2 (G-4 拍板 (b).3 ≥ 2 个独立源头)
    pub fn passes_4_audit(&self) -> bool {
        self.trust_score >= 0.95
            && self.mutual_trust_verified
            && self.historical_evidence_count >= 10
            && self.multi_source_count >= 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trust_audit_log_new_sets_revocation_window_7_days() {
        let log = TrustAuditLog::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "high_trust_score".to_string(),
            0.95,
            true,
            10,
            2,
        );
        assert!(log.is_within_revocation_window());
        assert!(!log.revoked);
    }

    #[test]
    fn trust_audit_log_passes_4_audit_all_true() {
        let log = TrustAuditLog::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "all_4_pass".to_string(),
            0.95,
            true,
            10,
            2,
        );
        assert!(log.passes_4_audit());
    }

    #[test]
    fn trust_audit_log_fails_4_audit_below_threshold() {
        let log = TrustAuditLog::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "below_threshold".to_string(),
            0.94, // 阈值 0.95 不达标
            true,
            10,
            2,
        );
        assert!(!log.passes_4_audit());
    }

    #[test]
    fn trust_audit_log_fails_4_audit_no_mutual() {
        let log = TrustAuditLog::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "no_mutual".to_string(),
            0.95,
            false, // mutual trust 不达标
            10,
            2,
        );
        assert!(!log.passes_4_audit());
    }

    #[test]
    fn trust_audit_log_fails_4_audit_low_evidence() {
        let log = TrustAuditLog::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "low_evidence".to_string(),
            0.95,
            true,
            9, // < 10 阈值
            2,
        );
        assert!(!log.passes_4_audit());
    }

    #[test]
    fn trust_audit_log_fails_4_audit_low_multi_source() {
        let log = TrustAuditLog::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "low_multi_source".to_string(),
            0.95,
            true,
            10,
            1, // < 2 阈值
        );
        assert!(!log.passes_4_audit());
    }
}
