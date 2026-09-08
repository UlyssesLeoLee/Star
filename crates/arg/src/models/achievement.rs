// SPDX-License-Identifier: MIT OR Apache-2.0
//! Achievements (per DD-AGENT-RELATIONSHIP-001 §3.2.4).
//!
//! 20 achievements across 3 categories:
//! - **8 拓扑** (`Topology`) — static graph structure
//! - **7 行为** (`Behavior`) — runtime event pattern
//! - **5 产出** (`Output`) — aggregated output quality metric
//!
//! Each achievement carries an [`UnlockCondition`] that the
//! `ARGAchievementEngine` (in `crates/arg-effect`) evaluates.

use serde::{Deserialize, Serialize};

/// 3 评估维度 (per DD §3.2.4).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum AchievementCategory {
    /// Static graph structure (Cypher-driven).
    Topology,
    /// Runtime event pattern.
    Behavior,
    /// Aggregated output quality metric.
    Output,
}

/// 4 稀有度 (per DD §3.2.4 + §11.4).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Rarity {
    /// Common.
    Common,
    /// Rare.
    Rare,
    /// Epic.
    Epic,
    /// Legendary.
    Legendary,
}

/// A single achievement definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Achievement {
    /// Stable code (e.g. `TOP-001-MESH-5DOMAIN`).
    pub code: String,
    /// Chinese display name.
    pub name_zh: String,
    /// English display name.
    pub name_en: String,
    /// Long-form description shown in the UI.
    pub description: String,
    /// Category (Topology / Behavior / Output).
    pub category: AchievementCategory,
    /// Rarity bucket.
    pub rarity: Rarity,
    /// Icon URL (placeholder for now).
    pub icon_url: String,
    /// Unlock condition evaluated by `ARGAchievementEngine`.
    pub unlock_condition: UnlockCondition,
}

/// Unlock condition kinds. The `ARGAchievementEngine` (in `arg-effect`)
/// dispatches on this enum to choose its evaluator.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnlockCondition {
    /// Run a Cypher query against Memgraph; unlock when it returns at least
    /// one row.
    CypherQuery {
        /// The query body (parameterised with `$tenant_id`).
        query: String,
        /// Cypher parameters.
        params: serde_json::Value,
    },
    /// Match an event pattern a given number of times within a sliding
    /// window.
    EventPattern {
        /// Pattern string, e.g. `"edge.created.type=DELEGATES_TO"`.
        pattern: String,
        /// Required count.
        count: u32,
    },
    /// Aggregate metric above a threshold for a sustained period.
    AggregateMetric {
        /// Metric name, e.g. `"peer_review_score"`.
        metric: String,
        /// Threshold value.
        threshold: f64,
        /// Required sustained duration.
        time_window: chrono::Duration,
    },
}

/// Build the 8 拓扑成就 (per DD §6 + §3.2.4).
fn topology_achievements() -> Vec<Achievement> {
    vec![
        Achievement {
            code: "TOP-001-MESH-5DOMAIN".into(),
            name_zh: "五域全连接".into(),
            name_en: "5-Domain Mesh".into(),
            description: "5 agents from 5 distinct domains, fully connected".into(),
            category: AchievementCategory::Topology,
            rarity: Rarity::Epic,
            icon_url: "/static/arg/top-001.png".into(),
            unlock_condition: UnlockCondition::CypherQuery {
                query: "MATCH (a:Agent)-[r:COLLABORATES_WITH]-(b:Agent) \
                        WHERE a.tenant_id = $tenant_id AND a.archived = false \
                        WITH count(DISTINCT a) AS node_count, count(DISTINCT r) AS edge_count \
                        WHERE node_count = 5 AND edge_count = 10 \
                        RETURN true AS triggered"
                    .into(),
                params: serde_json::json!({}),
            },
        },
        Achievement {
            code: "TOP-002-HUB-AND-SPOKE".into(),
            name_zh: "轮辐式".into(),
            name_en: "Hub-and-Spoke".into(),
            description: "1 Lead with 4 outgoing DELEGATES_TO edges".into(),
            category: AchievementCategory::Topology,
            rarity: Rarity::Common,
            icon_url: "/static/arg/top-002.png".into(),
            unlock_condition: UnlockCondition::CypherQuery {
                query: "MATCH (lead:Agent)-[:DELEGATES_TO]->(w:Agent) \
                        WHERE lead.tenant_id = $tenant_id \
                        WITH lead, count(DISTINCT w) AS worker_count \
                        WHERE worker_count = 4 \
                        RETURN count(lead) >= 1 AS triggered"
                    .into(),
                params: serde_json::json!({}),
            },
        },
        Achievement {
            code: "TOP-003-MESH-10".into(),
            name_zh: "十人网状".into(),
            name_en: "10-Node Mesh".into(),
            description: "10 nodes, fully connected (45 edges)".into(),
            category: AchievementCategory::Topology,
            rarity: Rarity::Common,
            icon_url: "/static/arg/top-003.png".into(),
            unlock_condition: UnlockCondition::CypherQuery {
                query: "MATCH (a:Agent)-[r:COLLABORATES_WITH]-(b:Agent) \
                        WHERE a.tenant_id = $tenant_id AND a.archived = false \
                        WITH count(DISTINCT a) AS node_count, count(DISTINCT r) AS edge_count \
                        WHERE node_count = 10 AND edge_count = 45 \
                        RETURN true AS triggered"
                    .into(),
                params: serde_json::json!({}),
            },
        },
        Achievement {
            code: "TOP-004-CHAIN-5".into(),
            name_zh: "五步链".into(),
            name_en: "5-Step Chain".into(),
            description: "A → B → C → D → E (4 DELEGATES_TO edges)".into(),
            category: AchievementCategory::Topology,
            rarity: Rarity::Common,
            icon_url: "/static/arg/top-004.png".into(),
            unlock_condition: UnlockCondition::CypherQuery {
                query: "MATCH path = (a:Agent)-[:DELEGATES_TO*4]->(e:Agent) \
                        WHERE a.tenant_id = $tenant_id \
                        WITH a, e, length(path) AS depth \
                        WHERE depth = 4 \
                        RETURN count(*) >= 1 AS triggered"
                    .into(),
                params: serde_json::json!({}),
            },
        },
        Achievement {
            code: "TOP-005-HIERARCHICAL-3".into(),
            name_zh: "三层金字塔".into(),
            name_en: "3-Layer Hierarchical".into(),
            description: "1 Lead → 2 Sub-Lead → 6 Worker".into(),
            category: AchievementCategory::Topology,
            rarity: Rarity::Common,
            icon_url: "/static/arg/top-005.png".into(),
            unlock_condition: UnlockCondition::CypherQuery {
                query: "MATCH (lead:Agent)-[:DELEGATES_TO]->(sub:Agent)-[:DELEGATES_TO]->(worker:Agent) \
                        WHERE lead.tenant_id = $tenant_id \
                        WITH lead, count(DISTINCT sub) AS sub_count, count(DISTINCT worker) AS worker_count \
                        WHERE sub_count = 2 AND worker_count = 6 \
                        RETURN count(lead) >= 1 AS triggered"
                    .into(),
                params: serde_json::json!({}),
            },
        },
        Achievement {
            code: "TOP-006-REVIEW-COUNCIL".into(),
            name_zh: "评审团".into(),
            name_en: "Review Council".into(),
            description: "1 Lead + 3 Reviewer (3 incoming CONSULTS)".into(),
            category: AchievementCategory::Topology,
            rarity: Rarity::Rare,
            icon_url: "/static/arg/top-006.png".into(),
            unlock_condition: UnlockCondition::CypherQuery {
                query: "MATCH (lead:Agent)<-[:CONSULTS]-(reviewer:Agent) \
                        WHERE lead.tenant_id = $tenant_id \
                        WITH lead, count(DISTINCT reviewer) AS reviewer_count \
                        WHERE reviewer_count = 3 \
                        RETURN count(lead) >= 1 AS triggered"
                    .into(),
                params: serde_json::json!({}),
            },
        },
        Achievement {
            code: "TOP-007-NO-SELF-LOOP".into(),
            name_zh: "无自环".into(),
            name_en: "No Self-Loop".into(),
            description: "No self-loops in the graph (health check)".into(),
            category: AchievementCategory::Topology,
            rarity: Rarity::Rare,
            icon_url: "/static/arg/top-007.png".into(),
            unlock_condition: UnlockCondition::CypherQuery {
                query: "MATCH (a:Agent)-[r]->(a) WHERE a.tenant_id = $tenant_id \
                        RETURN count(r) = 0 AS no_self_loop"
                    .into(),
                params: serde_json::json!({}),
            },
        },
        Achievement {
            code: "TOP-008-NO-ISLAND".into(),
            name_zh: "无孤岛".into(),
            name_en: "No Island".into(),
            description: "Every agent has at least one edge (health check)".into(),
            category: AchievementCategory::Topology,
            rarity: Rarity::Rare,
            icon_url: "/static/arg/top-008.png".into(),
            unlock_condition: UnlockCondition::CypherQuery {
                query: "MATCH (a:Agent) WHERE a.tenant_id = $tenant_id AND a.archived = false \
                        OPTIONAL MATCH (a)-[r]-() \
                        WITH a, count(r) AS degree \
                        WHERE degree = 0 \
                        WITH count(a) AS isolated_count \
                        RETURN isolated_count = 0 AS no_isolated"
                    .into(),
                params: serde_json::json!({}),
            },
        },
    ]
}

/// Build the 7 协作行为成就 (per DD §3.2.4, placeholders for P3-E ARG.8).
fn behavior_achievements() -> Vec<Achievement> {
    vec![
        Achievement {
            code: "BEH-001-FIRST-EDGE".into(),
            name_zh: "初次建边".into(),
            name_en: "First Edge".into(),
            description: "Created the first edge in the graph".into(),
            category: AchievementCategory::Behavior,
            rarity: Rarity::Common,
            icon_url: "/static/arg/beh-001.png".into(),
            unlock_condition: UnlockCondition::EventPattern {
                pattern: "edge.created".into(),
                count: 1,
            },
        },
        Achievement {
            code: "BEH-002-10-EDGES".into(),
            name_zh: "十边里程碑".into(),
            name_en: "10 Edges".into(),
            description: "10 edges created in total".into(),
            category: AchievementCategory::Behavior,
            rarity: Rarity::Common,
            icon_url: "/static/arg/beh-002.png".into(),
            unlock_condition: UnlockCondition::EventPattern {
                pattern: "edge.created".into(),
                count: 10,
            },
        },
        Achievement {
            code: "BEH-003-100-EDGES".into(),
            name_zh: "百边".into(),
            name_en: "100 Edges".into(),
            description: "100 edges created in total".into(),
            category: AchievementCategory::Behavior,
            rarity: Rarity::Common,
            icon_url: "/static/arg/beh-003.png".into(),
            unlock_condition: UnlockCondition::EventPattern {
                pattern: "edge.created".into(),
                count: 100,
            },
        },
        Achievement {
            code: "BEH-004-CONSULT-ROUND".into(),
            name_zh: "首次咨询".into(),
            name_en: "First Consult".into(),
            description: "First CONSULTS round completed".into(),
            category: AchievementCategory::Behavior,
            rarity: Rarity::Rare,
            icon_url: "/static/arg/beh-004.png".into(),
            unlock_condition: UnlockCondition::EventPattern {
                pattern: "consult.round.completed".into(),
                count: 1,
            },
        },
        Achievement {
            code: "BEH-005-CHALLENGE-ROUND".into(),
            name_zh: "首次挑战".into(),
            name_en: "First Challenge".into(),
            description: "First CHALLENGES round completed".into(),
            category: AchievementCategory::Behavior,
            rarity: Rarity::Rare,
            icon_url: "/static/arg/beh-005.png".into(),
            unlock_condition: UnlockCondition::EventPattern {
                pattern: "challenge.round.completed".into(),
                count: 1,
            },
        },
        Achievement {
            code: "BEH-006-STAND-IN-RESCUE".into(),
            name_zh: "救场".into(),
            name_en: "Stand-In Rescue".into(),
            description: "Stand-in fallback triggered successfully".into(),
            category: AchievementCategory::Behavior,
            rarity: Rarity::Epic,
            icon_url: "/static/arg/beh-006.png".into(),
            unlock_condition: UnlockCondition::EventPattern {
                pattern: "stand_in.activated".into(),
                count: 1,
            },
        },
        Achievement {
            code: "BEH-007-MENTOR-CHAIN".into(),
            name_zh: "师徒链".into(),
            name_en: "Mentor Chain".into(),
            description: "Built a MENTORS chain of length 3".into(),
            category: AchievementCategory::Behavior,
            rarity: Rarity::Legendary,
            icon_url: "/static/arg/beh-007.png".into(),
            unlock_condition: UnlockCondition::EventPattern {
                pattern: "mentor.context_injected".into(),
                count: 3,
            },
        },
    ]
}

/// Build the 5 产出质量成就 (per DD §3.2.4, placeholders for P3-E ARG.8).
fn output_achievements() -> Vec<Achievement> {
    vec![
        Achievement {
            code: "OUT-001-PEER-REVIEW-80".into(),
            name_zh: "高分评审".into(),
            name_en: "High Peer Review".into(),
            description: "Peer review score >= 0.8".into(),
            category: AchievementCategory::Output,
            rarity: Rarity::Common,
            icon_url: "/static/arg/out-001.png".into(),
            unlock_condition: UnlockCondition::AggregateMetric {
                metric: "peer_review_score".into(),
                threshold: 0.8,
                time_window: chrono::Duration::days(7),
            },
        },
        Achievement {
            code: "OUT-002-CHALLENGE-ACCEPT".into(),
            name_zh: "挑战通过".into(),
            name_en: "Challenge Accepted".into(),
            description: "10 consecutive challenges accepted".into(),
            category: AchievementCategory::Output,
            rarity: Rarity::Rare,
            icon_url: "/static/arg/out-002.png".into(),
            unlock_condition: UnlockCondition::AggregateMetric {
                metric: "challenge_accept_streak".into(),
                threshold: 10.0,
                time_window: chrono::Duration::days(30),
            },
        },
        Achievement {
            code: "OUT-003-FAST-COLLAB".into(),
            name_zh: "协作提速".into(),
            name_en: "Fast Collaboration".into(),
            description: "Wall-clock saved by parallel collaboration >= 1h".into(),
            category: AchievementCategory::Output,
            rarity: Rarity::Rare,
            icon_url: "/static/arg/out-003.png".into(),
            unlock_condition: UnlockCondition::AggregateMetric {
                metric: "wall_clock_savings_seconds".into(),
                threshold: 3600.0,
                time_window: chrono::Duration::days(7),
            },
        },
        Achievement {
            code: "OUT-004-TRUST-VERIFIED".into(),
            name_zh: "信任验证".into(),
            name_en: "Trust Verified".into(),
            description: "Trust skip-verify triggered >= 100 times".into(),
            category: AchievementCategory::Output,
            rarity: Rarity::Epic,
            icon_url: "/static/arg/out-004.png".into(),
            unlock_condition: UnlockCondition::AggregateMetric {
                metric: "trust_skip_verify_total".into(),
                threshold: 100.0,
                time_window: chrono::Duration::days(30),
            },
        },
        Achievement {
            code: "OUT-005-LEGENDARY-AGENT".into(),
            name_zh: "传奇代理".into(),
            name_en: "Legendary Agent".into(),
            description: "An agent reached trust tier VeryHigh".into(),
            category: AchievementCategory::Output,
            rarity: Rarity::Legendary,
            icon_url: "/static/arg/out-005.png".into(),
            unlock_condition: UnlockCondition::AggregateMetric {
                metric: "agents_at_very_high_tier".into(),
                threshold: 1.0,
                time_window: chrono::Duration::days(30),
            },
        },
    ]
}

/// Returns the 20 canonical achievements: 8 拓扑 + 7 行为 + 5 产出.
///
/// The order is stable so callers can use index-based keys if they need
/// to (e.g. for a UI grid).
pub fn all_achievements() -> Vec<Achievement> {
    let mut all = Vec::with_capacity(20);
    all.extend(topology_achievements());
    all.extend(behavior_achievements());
    all.extend(output_achievements());
    all
}

/// Helper for tests: count achievements in a given category.
pub fn count_by_category(category: AchievementCategory) -> usize {
    all_achievements()
        .iter()
        .filter(|a| a.category == category)
        .count()
}
