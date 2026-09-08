// SPDX-License-Identifier: MIT OR Apache-2.0
//! 8 拓扑成就 Cypher templates (per DD-AGENT-RELATIONSHIP-001 §6).
//!
//! Each entry is the `(code, query)` pair consumed by the
//! `ARGAchievementEngine` topology evaluator. The Cypher bodies are
//! parameterised by `$tenant_id`; no APOC dependency (per DD §6
//! self-review fix F-11).

/// A `(code, cypher)` pair for one 拓扑 achievement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopologyCypher {
    /// Achievement code (e.g. `"TOP-001-MESH-5DOMAIN"`).
    pub code: String,
    /// Cypher body, parameterised by `$tenant_id`.
    pub cypher: String,
}

/// Returns the 8 拓扑成就 Cypher pairs in canonical order
/// (`TOP-001` … `TOP-008`).
pub fn all_topology_cyphers() -> Vec<TopologyCypher> {
    vec![
        TopologyCypher {
            code: "TOP-001-MESH-5DOMAIN".into(),
            cypher: "MATCH (a:Agent)-[r:COLLABORATES_WITH]-(b:Agent) \
                     WHERE a.tenant_id = $tenant_id AND a.archived = false \
                     WITH a, b, r \
                     WITH count(DISTINCT a) AS node_count, count(DISTINCT r) AS edge_count, \
                          collect(DISTINCT a.domain) + collect(DISTINCT b.domain) AS all_domains \
                     WITH node_count, edge_count, \
                          size([d IN all_domains WHERE d IS NOT NULL | d]) AS total_domain_refs \
                     WHERE node_count = 5 AND edge_count = 10 AND total_domain_refs >= 5 \
                     RETURN true AS triggered \
                     LIMIT 1"
                .into(),
        },
        TopologyCypher {
            code: "TOP-002-HUB-AND-SPOKE".into(),
            cypher: "MATCH (lead:Agent)-[:DELEGATES_TO]->(w:Agent) \
                     WHERE lead.tenant_id = $tenant_id \
                     WITH lead, count(DISTINCT w) AS worker_count \
                     WHERE worker_count = 4 \
                     RETURN count(lead) >= 1 AS triggered"
                .into(),
        },
        TopologyCypher {
            code: "TOP-003-MESH-10".into(),
            cypher: "MATCH (a:Agent)-[r:COLLABORATES_WITH]-(b:Agent) \
                     WHERE a.tenant_id = $tenant_id AND a.archived = false \
                     WITH count(DISTINCT a) AS node_count, count(DISTINCT r) AS edge_count \
                     WHERE node_count = 10 AND edge_count = 45 \
                     RETURN true AS triggered"
                .into(),
        },
        TopologyCypher {
            code: "TOP-004-CHAIN-5".into(),
            cypher: "MATCH path = (a:Agent)-[:DELEGATES_TO*4]->(e:Agent) \
                     WHERE a.tenant_id = $tenant_id \
                     WITH a, e, length(path) AS depth \
                     WHERE depth = 4 \
                     RETURN count(*) >= 1 AS triggered \
                     LIMIT 1"
                .into(),
        },
        TopologyCypher {
            code: "TOP-005-HIERARCHICAL-3".into(),
            cypher: "MATCH (lead:Agent)-[:DELEGATES_TO]->(sub:Agent)-[:DELEGATES_TO]->(worker:Agent) \
                     WHERE lead.tenant_id = $tenant_id \
                     WITH lead, count(DISTINCT sub) AS sub_count, count(DISTINCT worker) AS worker_count \
                     WHERE sub_count = 2 AND worker_count = 6 \
                     RETURN count(lead) >= 1 AS triggered"
                .into(),
        },
        TopologyCypher {
            code: "TOP-006-REVIEW-COUNCIL".into(),
            cypher: "MATCH (lead:Agent)<-[:CONSULTS]-(reviewer:Agent) \
                     WHERE lead.tenant_id = $tenant_id \
                     WITH lead, count(DISTINCT reviewer) AS reviewer_count \
                     WHERE reviewer_count = 3 \
                     RETURN count(lead) >= 1 AS triggered"
                .into(),
        },
        TopologyCypher {
            code: "TOP-007-NO-SELF-LOOP".into(),
            cypher: "MATCH (a:Agent)-[r]->(a) WHERE a.tenant_id = $tenant_id \
                     RETURN count(r) = 0 AS no_self_loop"
                .into(),
        },
        TopologyCypher {
            code: "TOP-008-NO-ISLAND".into(),
            cypher: "MATCH (a:Agent) WHERE a.tenant_id = $tenant_id AND a.archived = false \
                     OPTIONAL MATCH (a)-[r]-() \
                     WITH a, count(r) AS degree \
                     WHERE degree = 0 \
                     WITH count(a) AS isolated_count \
                     RETURN isolated_count = 0 AS no_isolated"
                .into(),
        },
    ]
}
