// SPDX-License-Identifier: MIT OR Apache-2.0
//! Schema v1 migration (per DD §3.1).
//!
//! `apply_schema_v1` is a pure function that returns the list of
//! `CREATE CONSTRAINT` / `CREATE INDEX` statements required to bootstrap
//! the Memgraph graph. The real driver call lives in
//! `MemgraphClient::execute_write` (still a P3-C W1 stub).

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_v1_has_5_statements() {
        assert_eq!(apply_schema_v1().len(), 5);
    }
}
