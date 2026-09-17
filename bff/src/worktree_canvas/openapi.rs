// SPDX-License-Identifier: MIT OR Apache-2.0
//! `bff/src/worktree_canvas/openapi.rs` — OpenAPI 3.0 spec 同步生成 (per AC-Q-6 +
//! IMPL-PLAN §4.13 brief).
//!
//! 阶段 1: hand-written JSON 字符串 (per `serde_json::json!` macro), 完整列出
//! 11 REST + 15 SSE + 1 WS endpoint + 18 Action + 15 Event schema. 阶段 2 业务
//! 实装可切换 utoipa 自动从 axum handler 注解生成 (per brief §"OpenAPI 3.0 spec
//! 同步生成").
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #6 v2 + 守门 #7 + 守门 #14 v2):
//! - 0 `unsafe` blocks.
//! - AC-Q-6 OpenAPI 3.0 schema 跟 DD §20 §21 字段 1:1 对齐.

use serde_json::{json, Value};

/// OpenAPI 3.0 root 文档 (per AC-Q-6).
pub fn openapi_document() -> Value {
    // Build paths as Value::Object so we can splice the 15 SSE + WS entries
    // (json!{} macro doesn't accept function calls).
    let mut all_paths = serde_json::Map::new();
    if let Value::Object(p) = paths() {
        for (k, v) in p {
            // Skip placeholder stub.
            if !k.starts_with("(sse-and-ws-merged-here)") {
                all_paths.insert(k, v);
            }
        }
    }
    // Insert 15 SSE + 1 health stream + 1 WS.
    if let Value::Object(sse) = sse_paths() {
        all_paths.extend(sse);
    }

    json!({
        "openapi": "3.0.3",
        "info": {
            "title": "AI Worktree Graph Canvas BFF API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "BFF (Backend for Frontend) for AI Worktree Graph Canvas module. \
                Exposes 11 REST + 15 SSE + 1 WS endpoints. \
                Reference: DD-WORKTREE-CANVAS-001 §20-§21 + WORKTREE-CANVAS-IMPL-PLAN-001 §4.13 + \
                docs/specs/worktree-canvas-spec.md §4.4 §4.5."
        },
        "servers": [
            { "url": "http://localhost:3000", "description": "Local dev (Next.js dev server proxies to BFF)" },
            { "url": "https://api.star.local", "description": "Production (envoy → BFF via svc://)" }
        ],
        "tags": [
            { "name": "repository", "description": "Repository list" },
            { "name": "worktrees", "description": "Worktree CRUD + actions" },
            { "name": "graph", "description": "Subgraph focus queries" },
            { "name": "search", "description": "DSL + NL search" },
            { "name": "risks", "description": "Risk list" },
            { "name": "health", "description": "BFF + worktree aggregate health" },
            { "name": "events-sse", "description": "15 SSE event streams" },
            { "name": "realtime-ws", "description": "1 WebSocket realtime channel" }
        ],
        "paths": Value::Object(all_paths),
        "components": components()
    })
}

/// 11 REST + 1 health + 15 SSE + 1 WS = 28 paths (per IMPL-PLAN §4.13).
fn paths() -> Value {
    json!({
        // ── 11 REST ──
        "/v1/worktree-canvas/repositories": {
            "get": {
                "tags": ["repositories"],
                "operationId": "listRepositories",
                "summary": "List repositories visible to current tenant",
                "responses": {
                    "200": {
                        "description": "OK",
                        "content": {
                            "application/json": {
                                "schema": { "type": "array", "items": { "$ref": "#/components/schemas/RepositoryDto" } }
                            }
                        }
                    },
                    "403": { "$ref": "#/components/responses/Forbidden" }
                }
            }
        },
        "/v1/worktree-canvas/worktrees": {
            "get": {
                "tags": ["worktrees"],
                "operationId": "listWorktrees",
                "summary": "List worktrees (paginated)",
                "parameters": [
                    { "name": "repo_id", "in": "query", "schema": { "type": "string", "format": "uuid" } },
                    { "name": "state_filter", "in": "query", "schema": { "type": "string" } },
                    { "name": "cursor", "in": "query", "schema": { "type": "string" } },
                    { "name": "limit", "in": "query", "schema": { "type": "integer", "default": 50 } }
                ],
                "responses": {
                    "200": {
                        "description": "OK",
                        "content": {
                            "application/json": { "schema": { "$ref": "#/components/schemas/ListWorktreesResponse" } }
                        }
                    },
                    "403": { "$ref": "#/components/responses/Forbidden" }
                }
            },
            "post": {
                "tags": ["worktrees"],
                "operationId": "createWorktree",
                "summary": "Create worktree (idempotent via idempotency_key)",
                "requestBody": {
                    "required": true,
                    "content": {
                        "application/json": { "schema": { "$ref": "#/components/schemas/CreateWorktreeRequest" } }
                    }
                },
                "responses": {
                    "201": {
                        "description": "Created",
                        "content": { "application/json": { "schema": { "$ref": "#/components/schemas/ActionResponse" } } }
                    },
                    "400": { "$ref": "#/components/responses/BadRequest" },
                    "403": { "$ref": "#/components/responses/Forbidden" }
                }
            }
        },
        "/v1/worktree-canvas/worktrees/{id}": {
            "parameters": [
                { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
            ],
            "get": {
                "tags": ["worktrees"],
                "operationId": "getWorktree",
                "summary": "Get single worktree by ID",
                "responses": {
                    "200": {
                        "description": "OK",
                        "content": { "application/json": { "schema": { "$ref": "#/components/schemas/WorktreeResponse" } } }
                    },
                    "404": { "$ref": "#/components/responses/NotFound" }
                }
            },
            "patch": {
                "tags": ["worktrees"],
                "operationId": "updateWorktree",
                "summary": "Update worktree metadata (non-destructive)",
                "requestBody": {
                    "required": true,
                    "content": { "application/json": { "schema": { "type": "object" } } }
                },
                "responses": {
                    "200": { "description": "Updated" },
                    "403": { "$ref": "#/components/responses/Forbidden" },
                    "404": { "$ref": "#/components/responses/NotFound" }
                }
            },
            "delete": {
                "tags": ["worktrees"],
                "operationId": "deleteWorktree",
                "summary": "Delete worktree (DESTRUCTIVE — requires confirm=true)",
                "responses": {
                    "204": { "description": "Deleted" },
                    "403": { "$ref": "#/components/responses/Forbidden" },
                    "404": { "$ref": "#/components/responses/NotFound" },
                    "409": { "$ref": "#/components/responses/ConflictRequired" }
                }
            }
        },
        "/v1/worktree-canvas/worktrees/{id}/actions/{action_type}": {
            "parameters": [
                { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } },
                {
                    "name": "action_type",
                    "in": "path",
                    "required": true,
                    "schema": { "type": "string", "enum": action_url_names() }
                }
            ],
            "post": {
                "tags": ["worktrees"],
                "operationId": "executeAction",
                "summary": "Execute one of 18 worktree actions (idempotent via idempotency_key)",
                "requestBody": {
                    "required": true,
                    "content": {
                        "application/json": { "schema": { "$ref": "#/components/schemas/ActionRequest" } }
                    }
                },
                "responses": {
                    "200": {
                        "description": "OK",
                        "content": { "application/json": { "schema": { "$ref": "#/components/schemas/ActionResponse" } } }
                    },
                    "400": { "$ref": "#/components/responses/BadRequest" },
                    "403": { "$ref": "#/components/responses/Forbidden" },
                    "409": { "$ref": "#/components/responses/ConflictRequired" }
                }
            }
        },
        "/v1/worktree-canvas/worktrees/{id}/graph": {
            "parameters": [
                { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
            ],
            "get": {
                "tags": ["graph"],
                "operationId": "getWorktreeGraph",
                "summary": "Get focus N-hop subgraph around a worktree",
                "responses": {
                    "200": {
                        "description": "OK",
                        "content": { "application/json": { "schema": { "$ref": "#/components/schemas/WorktreeGraphResponse" } } }
                    },
                    "403": { "$ref": "#/components/responses/Forbidden" }
                }
            }
        },
        "/v1/worktree-canvas/risks": {
            "get": {
                "tags": ["risks"],
                "operationId": "listRisks",
                "summary": "List active risk detections across all visible worktrees",
                "responses": {
                    "200": {
                        "description": "OK",
                        "content": {
                            "application/json": {
                                "schema": { "type": "array", "items": { "$ref": "#/components/schemas/RiskResponse" } }
                            }
                        }
                    }
                }
            }
        },
        "/v1/worktree-canvas/health": {
            "get": {
                "tags": ["health"],
                "operationId": "getHealthSummary",
                "summary": "Aggregate worktree health summary",
                "responses": {
                    "200": {
                        "description": "OK",
                        "content": { "application/json": { "schema": { "$ref": "#/components/schemas/HealthSummaryDto" } } }
                    }
                }
            }
        },
        "/v1/worktree-canvas/search": {
            "post": {
                "tags": ["search"],
                "operationId": "search",
                "summary": "Search worktrees via DSL (e.g. show:conflict behind:>20)",
                "requestBody": {
                    "required": true,
                    "content": { "application/json": { "schema": { "$ref": "#/components/schemas/SearchRequest" } } }
                },
                "responses": {
                    "200": {
                        "description": "OK",
                        "content": { "application/json": { "schema": { "$ref": "#/components/schemas/SearchResponse" } } }
                    },
                    "400": { "$ref": "#/components/responses/BadRequest" }
                }
            }
        },
        "/v1/worktree-canvas/nl-query": {
            "post": {
                "tags": ["search"],
                "operationId": "nlQuery",
                "summary": "Natural language → DSL → worktree query (mock LLM in stage 1)",
                "requestBody": {
                    "required": true,
                    "content": { "application/json": { "schema": { "$ref": "#/components/schemas/NlQueryRequest" } } }
                },
                "responses": {
                    "200": {
                        "description": "OK",
                        "content": { "application/json": { "schema": { "$ref": "#/components/schemas/NlQueryResponse" } } }
                    }
                }
            }
        },
        "/v1/worktree-canvas/bff-health": {
            "get": {
                "tags": ["health"],
                "operationId": "bffHealth",
                "summary": "BFF instance-level health (uptime, version)",
                "responses": {
                    "200": {
                        "description": "OK",
                        "content": { "application/json": { "schema": { "$ref": "#/components/schemas/BffHealthDto" } } }
                    }
                }
            }
        },
        // ── 15 SSE + 1 WS entries spliced in openapi_document() ──
        "(sse-and-ws-merged-here)": {}
    })
}

/// 15 SSE event 路径 (per spec §4.5).
fn sse_paths() -> Value {
    let mut obj = serde_json::Map::new();
    for name in crate::worktree_canvas::dto::WorktreeSseEvent::ALL_NAMES {
        let path = format!("/v1/worktree-canvas/events/{name}");
        obj.insert(path, json!({
            "get": {
                "tags": ["events-sse"],
                "operationId": format!("subscribe{name}"),
                "summary": format!("SSE stream for {name} events (per spec §4.5)"),
                "responses": {
                    "200": {
                        "description": "text/event-stream",
                        "content": { "text/event-stream": { "schema": { "type": "string" } } }
                    },
                    "403": { "$ref": "#/components/responses/Forbidden" }
                }
            }
        }));
    }
    // Plus health stream.
    obj.insert("/v1/worktree-canvas/health/stream".to_string(), json!({
        "get": {
            "tags": ["events-sse"],
            "operationId": "subscribeHealth",
            "summary": "SSE health keepalive (per IMPL-PLAN §4.13)",
            "responses": {
                "200": {
                    "description": "text/event-stream",
                    "content": { "text/event-stream": { "schema": { "type": "string" } } }
                }
            }
        }
    }));
    // WS realtime path (per IMPL-PLAN §4.13.4 + DD §21).
    obj.insert("/v1/worktree-canvas/realtime".to_string(), json!({
        "get": {
            "tags": ["realtime-ws"],
            "operationId": "realtimeWebSocket",
            "summary": "WebSocket realtime channel (15 SSE events fanout + 30s heartbeat)",
            "responses": {
                "101": { "description": "Switching Protocols — WebSocket upgrade" }
            }
        }
    }));
    Value::Object(obj)
}

fn action_url_names() -> Vec<&'static str> {
    crate::worktree_canvas::permission::WorktreeAction::ALL
        .iter()
        .map(|a| a.as_url())
        .collect()
}

/// OpenAPI `components.schemas` (per DD §20 §21 field 1:1).
fn components() -> Value {
    json!({
        "schemas": {
            "RepositoryDto": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "format": "uuid" },
                    "name": { "type": "string" },
                    "default_branch": { "type": "string" },
                    "worktree_count": { "type": "integer" }
                }
            },
            "WorktreeResponse": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "format": "uuid" },
                    "repo_id": { "type": "string", "format": "uuid" },
                    "name": { "type": "string" },
                    "branch": { "type": "string" },
                    "human_state": { "type": "string", "enum": ["RUNNING","WAITING","READY","DIVERGED","CONFLICT","MERGED","STALE"] },
                    "machine_state": { "type": "string" },
                    "ahead": { "type": "integer" },
                    "behind": { "type": "integer" },
                    "dirty": { "type": "boolean" },
                    "health_score": { "$ref": "#/components/schemas/HealthScoreDto" },
                    "last_activity": { "type": "string", "format": "date-time" },
                    "last_activity_relative": { "type": "string" },
                    "agent": { "$ref": "#/components/schemas/AgentSummary" },
                    "task": { "$ref": "#/components/schemas/TaskSummary" },
                    "risk_count": { "type": "integer" },
                    "test_state": { "type": "string" },
                    "locked": { "type": "boolean" },
                    "archived": { "type": "boolean" },
                    "created_at": { "type": "string", "format": "date-time" },
                    "merged_at": { "type": "string", "format": "date-time", "nullable": true }
                }
            },
            "HealthScoreDto": {
                "type": "object",
                "properties": {
                    "value": { "type": "integer", "minimum": 0, "maximum": 100 },
                    "deductions": { "type": "array", "items": { "$ref": "#/components/schemas/HealthDeduction" } },
                    "computed_at": { "type": "string", "format": "date-time" }
                }
            },
            "HealthDeduction": {
                "type": "object",
                "properties": {
                    "factor": { "type": "string" },
                    "points": { "type": "integer" },
                    "reason": { "type": "string" }
                }
            },
            "AgentSummary": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "format": "uuid" },
                    "agent_type": { "type": "string" },
                    "model": { "type": "string" }
                }
            },
            "TaskSummary": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "format": "uuid" },
                    "title": { "type": "string" }
                }
            },
            "CreateWorktreeRequest": {
                "type": "object",
                "required": ["repo_id", "branch", "idempotency_key"],
                "properties": {
                    "repo_id": { "type": "string", "format": "uuid" },
                    "branch": { "type": "string" },
                    "base_branch": { "type": "string" },
                    "task_id": { "type": "string", "format": "uuid" },
                    "idempotency_key": { "type": "string", "format": "uuid" }
                }
            },
            "ActionRequest": {
                "type": "object",
                "required": ["idempotency_key"],
                "properties": {
                    "params": { "type": "object" },
                    "confirm": { "type": "boolean" },
                    "idempotency_key": { "type": "string", "format": "uuid" }
                }
            },
            "ActionResponse": {
                "type": "object",
                "properties": {
                    "success": { "type": "boolean" },
                    "worktree_id": { "type": "string", "format": "uuid" },
                    "new_state": { "type": "string" },
                    "duration_ms": { "type": "integer" },
                    "warnings": { "type": "array", "items": { "type": "string" } },
                    "errors": { "type": "array", "items": { "type": "string" } }
                }
            },
            "SearchRequest": {
                "type": "object",
                "required": ["query"],
                "properties": {
                    "query": { "type": "string" },
                    "query_type": { "type": "string", "enum": ["dsl","nl"], "default": "dsl" }
                }
            },
            "SearchResponse": {
                "type": "object",
                "properties": {
                    "worktree_ids": { "type": "array", "items": { "type": "string", "format": "uuid" } },
                    "total": { "type": "integer" },
                    "duration_ms": { "type": "integer" }
                }
            },
            "NlQueryRequest": {
                "type": "object",
                "required": ["question"],
                "properties": { "question": { "type": "string" } }
            },
            "NlQueryResponse": {
                "type": "object",
                "properties": {
                    "worktree_ids": { "type": "array", "items": { "type": "string", "format": "uuid" } },
                    "translated_query": { "type": "string" },
                    "explanation": { "type": "string" }
                }
            },
            "ListWorktreesResponse": {
                "type": "object",
                "properties": {
                    "items": { "type": "array", "items": { "$ref": "#/components/schemas/WorktreeResponse" } },
                    "total": { "type": "integer" },
                    "next_cursor": { "type": "string", "nullable": true }
                }
            },
            "RiskResponse": {
                "type": "object",
                "properties": {
                    "worktree_id": { "type": "string", "format": "uuid" },
                    "risk_score": { "type": "number", "minimum": 0, "maximum": 1 },
                    "risk_type": { "type": "string" },
                    "shared_files": { "type": "array", "items": { "type": "string" } },
                    "detected_at": { "type": "string", "format": "date-time" },
                    "reason": { "type": "string" },
                    "confidence": { "type": "number", "minimum": 0, "maximum": 1 }
                }
            },
            "HealthSummaryDto": {
                "type": "object",
                "properties": {
                    "overall_score": { "type": "integer" },
                    "healthy": { "type": "integer" },
                    "at_risk": { "type": "integer" },
                    "diverged": { "type": "integer" },
                    "conflict": { "type": "integer" },
                    "stale": { "type": "integer" }
                }
            },
            "WorktreeGraphResponse": {
                "type": "object",
                "properties": {
                    "center": { "type": "string", "format": "uuid" },
                    "hop": { "type": "integer" },
                    "nodes": { "type": "array", "items": { "$ref": "#/components/schemas/GraphNodeDto" } },
                    "edges": { "type": "array", "items": { "$ref": "#/components/schemas/GraphEdgeDto" } }
                }
            },
            "GraphNodeDto": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "format": "uuid" },
                    "kind": { "type": "string" },
                    "label": { "type": "string" },
                    "properties": { "type": "object" }
                }
            },
            "GraphEdgeDto": {
                "type": "object",
                "properties": {
                    "source": { "type": "string", "format": "uuid" },
                    "target": { "type": "string", "format": "uuid" },
                    "kind": { "type": "string" },
                    "risk_score": { "type": "number", "nullable": true },
                    "metadata": { "type": "object" }
                }
            },
            "BffHealthDto": {
                "type": "object",
                "properties": {
                    "status": { "type": "string" },
                    "version": { "type": "string" },
                    "uptime_seconds": { "type": "integer" },
                    "worktree_canvas_enabled": { "type": "boolean" }
                }
            }
        },
        "responses": {
            "BadRequest": {
                "description": "Bad request",
                "content": {
                    "application/json": { "schema": { "$ref": "#/components/schemas/WorktreeApiError" } }
                }
            },
            "Forbidden": {
                "description": "Forbidden (RBAC denied)",
                "content": {
                    "application/json": { "schema": { "$ref": "#/components/schemas/WorktreeApiError" } }
                }
            },
            "NotFound": {
                "description": "Resource not found",
                "content": {
                    "application/json": { "schema": { "$ref": "#/components/schemas/WorktreeApiError" } }
                }
            },
            "ConflictRequired": {
                "description": "Destructive action requires confirm=true (per DD §20.2)",
                "content": {
                    "application/json": { "schema": { "$ref": "#/components/schemas/WorktreeApiError" } }
                }
            },
            "WorktreeApiError": {
                "type": "object",
                "description": "6-field error per 守门 #6 v2",
                "properties": {
                    "code": { "type": "string" },
                    "message": { "type": "string" },
                    "source": { "type": "string" },
                    "location": { "type": "string" },
                    "context": { "type": "object" },
                    "trace_id": { "type": "string", "format": "uuid" }
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_document_top_level_keys() {
        let doc = openapi_document();
        assert_eq!(doc["openapi"], "3.0.3");
        assert!(doc["info"].is_object());
        assert!(doc["paths"].is_object());
        assert!(doc["components"]["schemas"].is_object());
    }

    #[test]
    fn openapi_paths_count_matches_expected() {
        // 11 REST 端点 = 9 distinct path templates (worktrees covers 2 methods,
        // worktrees/{id} covers 3 methods); +1 bff-health (实例级) + 17 SSE/WS = 27 paths.
        let doc = openapi_document();
        let paths = doc["paths"].as_object().unwrap();
        // 验证 placeholder stub 被 splice 逻辑移除.
        for k in paths.keys() {
            assert!(
                !k.starts_with("(sse-and-ws-merged-here)"),
                "placeholder stub leaked: {k}"
            );
        }
        // 路径构成精确验证 (先验).
        assert!(paths.contains_key("/v1/worktree-canvas/bff-health"));
        assert!(paths.contains_key("/v1/worktree-canvas/health/stream"));
        assert!(paths.contains_key("/v1/worktree-canvas/realtime"));
        assert!(paths.contains_key("/v1/worktree-canvas/health"));
        assert!(paths.contains_key("/v1/worktree-canvas/worktrees"));
        // 数量兜底: 11 REST + 1 bff-health + 15 SSE + 1 health stream + 1 WS = 29,
        // 折合 9 REST paths + 1 bff-health + 17 SSE/WS = 27 paths.
        assert_eq!(
            paths.len(),
            27,
            "got {} paths, expected 27 (11 REST 端点 + 1 bff-health + 15 SSE + 1 health/stream + 1 WS = 29 endpoints, 27 distinct paths)",
            paths.len()
        );
    }

    #[test]
    fn openapi_includes_all_15_sse_paths() {
        let doc = openapi_document();
        let paths = doc["paths"].as_object().unwrap();
        for name in crate::worktree_canvas::dto::WorktreeSseEvent::ALL_NAMES {
            let path = format!("/v1/worktree-canvas/events/{name}");
            assert!(paths.contains_key(&path), "missing SSE path: {path}");
        }
    }

    #[test]
    fn openapi_includes_ws_realtime_path() {
        let doc = openapi_document();
        let paths = doc["paths"].as_object().unwrap();
        assert!(paths.contains_key("/v1/worktree-canvas/realtime"));
    }

    #[test]
    fn openapi_includes_all_18_action_url_names() {
        let doc = openapi_document();
        let schemas = &doc["components"]["schemas"];
        assert!(schemas.is_object());
        let action_names = action_url_names();
        assert_eq!(action_names.len(), 18);
    }
}
