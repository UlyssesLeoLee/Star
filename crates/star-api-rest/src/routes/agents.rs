/*
@cypher schema=1 source_sha256=ab1e4bff9aa563bede0a2f22baba98b1f06cc4b75e7de9f3826dde86876de8b5
MERGE (self:File {path:"crates/star-api-rest/src/routes/agents.rs"})
MERGE (agent:Type {id:"crates/agent-domain/src/models/agent.rs::Agent",kind:"struct"})
MERGE (self)-[:USES]->(agent)
@endcypher
*/
// SPDX-License-Identifier: MIT OR Apache-2.0
// =====================================================================
// agents.rs — Agent 列表端点 (agent-domain 门面)
// 2026-11-15 新增, per agent-domain P3-D.6
//
//   GET /api/v1/agents — 返回 Agent 列表
//
// 内存样例, 宿主运行时接入后替换 (无 DB 时返回常量样例; 真实数据源:
// agent-domain + PostgreSQL, per Phase B.2.6 Tier 3)
// =====================================================================

use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use uuid::Uuid;

use agent_domain::{Agent, AgentKind, AgentRole, AgentState, Domain};

use crate::error::RestError;
use crate::response::RestResponse;

/// 内存样例, 宿主运行时接入后替换 (3 个常量样例; 真实实现查 agent-domain + PostgreSQL)
fn sample_agents() -> Vec<Agent> {
    vec![
        Agent {
            id: Uuid::parse_str("00000000-0000-0000-0000-00000000a101").expect("fixed uuid"),
            name: "SA-01 LLM 推理".to_string(),
            avatar_url: None,
            role: AgentRole::Worker,
            kind: AgentKind::Sa01,
            domain: Some(Domain::Player),
            status: AgentState::Running,
            token_usage: 42_000,
            token_budget: 1_200_000,
            parent_session_id: None,
            pipeline_agent_ids: vec![],
            started_at: chrono::Utc::now(),
            tenant_id: Uuid::nil(),
            version: 3,
        },
        Agent {
            id: Uuid::parse_str("00000000-0000-0000-0000-00000000a104").expect("fixed uuid"),
            name: "SA-04 任务编排".to_string(),
            avatar_url: None,
            role: AgentRole::Supervisor,
            kind: AgentKind::Sa04,
            domain: Some(Domain::Admin),
            status: AgentState::Running,
            token_usage: 8_000,
            token_budget: 1_200_000,
            parent_session_id: None,
            pipeline_agent_ids: vec![],
            started_at: chrono::Utc::now(),
            tenant_id: Uuid::nil(),
            version: 5,
        },
        Agent {
            id: Uuid::parse_str("00000000-0000-0000-0000-00000000a105").expect("fixed uuid"),
            name: "SA-05 评审审查".to_string(),
            avatar_url: None,
            role: AgentRole::Reviewer,
            kind: AgentKind::Sa05,
            domain: Some(Domain::Social),
            status: AgentState::Stopped,
            token_usage: 0,
            token_budget: 1_200_000,
            parent_session_id: None,
            pipeline_agent_ids: vec![],
            started_at: chrono::Utc::now(),
            tenant_id: Uuid::nil(),
            version: 2,
        },
    ]
}

/// `GET /api/v1/agents` — Agent 列表 (内存样例, 宿主运行时接入后替换)
pub async fn list_agents() -> Result<Json<RestResponse<Value>>, RestError> {
    // 内存样例, 宿主运行时接入后替换 (真实实现: agent-domain + PostgreSQL 查询)
    let agents = sample_agents();
    let total = agents.len();
    Ok(Json(RestResponse::ok(json!({
        "total": total,
        "agents": agents,
        "note": "内存样例, 宿主运行时接入后替换 (无 DB)",
    }))))
}

/// 嵌套路由 (无状态, 可直接 nest)
pub fn router() -> Router {
    Router::new().route("/", get(list_agents))
}
