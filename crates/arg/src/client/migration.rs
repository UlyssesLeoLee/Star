// SPDX-License-Identifier: MIT OR Apache-2.0
//! Schema v1 migration (per DD §3.1).
//!
//! `apply_schema_v1` is a pure function that returns the list of
//! `CREATE CONSTRAINT` / `CREATE INDEX` statements required to bootstrap
//! the Memgraph graph. The real driver call lives in
//! `MemgraphClient::execute_write` (still a P3-C W1 stub).
//!
//! Schema v2: audit_audit_event WORM append-only table (per ADR-0043 +
//! G-4 4 重审计 WORM audit log 落地)

/// The 5 statements that bootstrap the ARG graph (per DD §3.1).
pub const SCHEMA_V1_STATEMENTS: &[&str] = &[
    "CREATE CONSTRAINT ON (a:Agent) ASSERT a.id IS UNIQUE",
    "CREATE CONSTRAINT ON (a:Agent) ASSERT exists(a.tenant_id)",
    "CREATE INDEX ON :Agent(tenant_id)",
    "CREATE INDEX ON :Agent(archetype)",
    "CREATE INDEX ON :Agent(domain)",
];

/// Returns the 5 v1 statements. Real driver call is the caller's job.
pub fn apply_schema_v1() -> &'static [&'static str] {
    SCHEMA_V1_STATEMENTS
}

/// Schema v2: audit_audit_event WORM append-only table (per ADR-0043 + G-4 4 重审计 WORM)
///
/// TrustAuditLog 写入此表 (per DDD-REVIEW-AGENT-RELATIONSHIP-001 §1.2 拍板 (c) + 守门 #13 d Transaction 100% audit)
/// 包含 4 重审计 (trust_score 阈值 0.95 + mutual + evidence + multi-source) + 7 天撤回窗口
pub const SCHEMA_V2_AUDIT_STATEMENTS: &[&str] = &[
    // 1. audit_audit_event 节点表 (per ADR-0043 WORM append-only)
    "CREATE CONSTRAINT ON (a:AuditEvent) ASSERT a.id IS UNIQUE",
    "CREATE INDEX ON :AuditEvent(tenant_id)",
    "CREATE INDEX ON :AuditEvent(event_type)",
    "CREATE INDEX ON :AuditEvent(audited_at)",
    // 2. TrustAuditLog 关系子表 (Trusts 关系审计专用)
    "CREATE CONSTRAINT ON (t:TrustAuditLog) ASSERT t.id IS UNIQUE",
    "CREATE INDEX ON :TrustAuditLog(from_agent_id)",
    "CREATE INDEX ON :TrustAuditLog(to_agent_id)",
    "CREATE INDEX ON :TrustAuditLog(revocation_window_until)",
    // 3. WORM 约束 (per 守门 #13 d + ADR-0043)
    "CREATE CONSTRAINT ON (a:AuditEvent) ASSERT NOT EXISTS (a.deleted)",
];

/// Returns the 9 v2 audit statements. Real driver call is the caller's job.
pub fn apply_schema_v2_audit() -> &'static [&'static str] {
    SCHEMA_V2_AUDIT_STATEMENTS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_v1_has_5_statements() {
        assert_eq!(apply_schema_v1().len(), 5);
    }

    #[test]
    fn test_schema_v2_audit_has_9_statements() {
        // 1 主表 + 1 索引 + 1 索引 + 1 索引 + 1 主表 + 4 索引 + 1 WORM = 9
        assert_eq!(apply_schema_v2_audit().len(), 9);
    }
}
