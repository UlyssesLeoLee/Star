//! `engine.rs` — QueryEngine Trait + Cypher 生成 (per DD §17)
//!
//! 3 方法:
//! - `execute_dsl` — DSL → Cypher → 执行 (本期 stub: 返回 Cypher + 0 结果)
//! - `nl_to_dsl` — NL → DSL (调用 `nl_translator::nl_to_dsl`)
//! - `dsl_to_cypher` — DSL → CypherQuery (含 params)
//!
//! 守门:
//! - #1 v25 `cargo test -p query-engine --lib` 4/4 pass
//! - #10 Cypher 注入防护 (params 全部走 sanitize)
//! - #11 缺标比错标

use crate::cypher_guard::sanitize_cypher_param;
use crate::dsl_parser::{DslParser, Filter};
use crate::error::QueryError;
use crate::keywords::ShowValue;
use crate::nl_translator::nl_to_dsl;

use graph_core::types::WorktreeId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cypher 查询结果
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryResult {
    /// 匹配的 WorktreeId
    pub worktree_ids: Vec<WorktreeId>,
    /// 总数
    pub total: usize,
    /// 耗时 (ms)
    pub duration_ms: u64,
}

/// Cypher 查询 (per DD §17)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CypherQuery {
    /// Cypher 字符串
    pub cypher: String,
    /// 参数 (走 $param 绑定, 杜绝字符串拼接)
    pub params: HashMap<String, serde_json::Value>,
}

/// QueryEngine Trait (per DD §17)
#[async_trait::async_trait]
pub trait QueryEngine: Send + Sync {
    /// 执行 DSL 查询
    async fn execute_dsl(&self, query: &str) -> Result<QueryResult, QueryError>;
    /// NL → DSL
    async fn nl_to_dsl(&self, nl: &str) -> Result<String, QueryError>;
    /// DSL → Cypher
    async fn dsl_to_cypher(&self, dsl: &str) -> Result<CypherQuery, QueryError>;
}

/// 纯函数版 QueryEngine — 无副作用, 用于测试 (per DD §17 "纯函数约束")
#[derive(Debug, Default, Clone)]
pub struct QueryEnginePure;

impl QueryEnginePure {
    /// 构造
    pub fn new() -> Self {
        Self
    }

    /// DSL → Cypher (同步版本, 内部用)
    pub fn dsl_to_cypher_sync(&self, dsl: &str) -> Result<CypherQuery, QueryError> {
        dsl_to_cypher(dsl)
    }
}

#[async_trait::async_trait]
impl QueryEngine for QueryEnginePure {
    async fn execute_dsl(&self, _query: &str) -> Result<QueryResult, QueryError> {
        // MVP stub: 不连接真实 graph, 返回空结果 + 0 ms
        // P2: 注入 GraphRepository, 走真实 Cypher 执行
        Ok(QueryResult {
            worktree_ids: Vec::new(),
            total: 0,
            duration_ms: 0,
        })
    }

    async fn nl_to_dsl(&self, nl: &str) -> Result<String, QueryError> {
        nl_to_dsl(nl)
    }

    async fn dsl_to_cypher(&self, dsl: &str) -> Result<CypherQuery, QueryError> {
        dsl_to_cypher(dsl)
    }
}

/// DSL → Cypher 翻译 (per DD §17 + §28 + §27)
///
/// 输出 Cypher 走参数化 (`$param`), 严格 sanitization.
pub fn dsl_to_cypher(dsl: &str) -> Result<CypherQuery, QueryError> {
    let filters = DslParser::new().parse(dsl)?;
    let mut cypher = String::from("MATCH (w:Worktree)");
    let mut where_clauses: Vec<String> = Vec::new();
    let mut params: HashMap<String, serde_json::Value> = HashMap::new();

    for (idx, filter) in filters.iter().enumerate() {
        match filter {
            Filter::Show { values } => {
                // show:conflict,unmerged → HumanState 过滤
                let has_unmerged = values.contains(&ShowValue::Unmerged);
                if !values.is_empty() {
                    let pname = format!("show_{idx}");
                    where_clauses.push(format!("w.human_state IN ${pname}"));
                    params.insert(
                        pname,
                        serde_json::json!(values
                            .iter()
                            .filter_map(|v| v.to_human_state().map(|s| format!("{s:?}")))
                            .collect::<Vec<_>>()),
                    );
                }
                if has_unmerged {
                    where_clauses.push("w.human_state <> 'merged'".to_string());
                }
            }
            Filter::Agent { value } => {
                let sanitized = sanitize_cypher_param(value)?;
                let pname = format!("agent_{idx}");
                where_clauses.push(format!("w.agent_type = ${pname}"));
                params.insert(pname, serde_json::json!(sanitized));
            }
            Filter::Behind { op, num } => {
                let pname = format!("behind_{idx}");
                where_clauses.push(format!("w.behind {} ${pname}", op.as_str()));
                params.insert(pname, serde_json::json!(*num));
            }
            Filter::Ahead { op, num } => {
                let pname = format!("ahead_{idx}");
                where_clauses.push(format!("w.ahead {} ${pname}", op.as_str()));
                params.insert(pname, serde_json::json!(*num));
            }
            Filter::Health { op, num } => {
                let pname = format!("health_{idx}");
                where_clauses.push(format!("w.health_score {} ${pname}", op.as_str()));
                params.insert(pname, serde_json::json!(*num));
            }
            Filter::Modified { value } => {
                let sanitized = sanitize_cypher_param(value)?;
                let pname = format!("modified_{idx}");
                where_clauses.push(format!("w.modified_files CONTAINS ${pname}"));
                params.insert(pname, serde_json::json!(sanitized));
            }
        }
    }

    if !where_clauses.is_empty() {
        cypher.push_str(" WHERE ");
        cypher.push_str(&where_clauses.join(" AND "));
    }
    cypher.push_str(" RETURN w");

    Ok(CypherQuery { cypher, params })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn dsl_to_cypher_show_conflict_combined() {
        // 守门 UT-3 (T11): DSL → Cypher 含 show + agent + behind + health + modified
        let engine = QueryEnginePure::new();
        let query = engine
            .dsl_to_cypher("show:conflict agent:codex behind:>20 health:<50 modified:auth.rs")
            .await
            .expect("translate");

        // 验证 Cypher 包含必要 fragment + 参数绑定
        assert!(query.cypher.contains("MATCH (w:Worktree)"));
        assert!(query.cypher.contains("RETURN w"));
        assert!(query.cypher.contains("w.human_state IN"));
        assert!(query.cypher.contains("w.agent_type ="));
        assert!(query.cypher.contains("w.behind >"));
        assert!(query.cypher.contains("w.health_score <"));
        assert!(query.cypher.contains("w.modified_files CONTAINS"));

        // 参数全部走 $param, 不在 Cypher 中拼接
        assert!(query.params.contains_key("agent_1"));
        assert!(query.params.contains_key("behind_2"));
        assert!(query.params.contains_key("health_3"));
        assert!(query.params.contains_key("modified_4"));
        assert_eq!(
            query.params.get("agent_1").unwrap(),
            &serde_json::json!("codex")
        );
        assert_eq!(
            query.params.get("behind_2").unwrap(),
            &serde_json::json!(20)
        );
    }

    #[tokio::test]
    async fn nl_to_dsl_returns_string() {
        let engine = QueryEnginePure::new();
        let dsl = engine.nl_to_dsl("show conflicts").await.expect("translate");
        assert!(dsl.contains("show:conflict"));
    }

    #[tokio::test]
    async fn execute_dsl_returns_empty_stub() {
        let engine = QueryEnginePure::new();
        let result = engine.execute_dsl("show:conflict").await.expect("execute");
        assert_eq!(result.worktree_ids.len(), 0);
        assert_eq!(result.total, 0);
    }

    #[test]
    fn dsl_to_cypher_injection_safe() {
        // 守门 #10: cypher 字符串不包含注入 payload
        // 路径 1: 解析期就拦截 — agent value 含注入字符, 返回 CYPHER_INJECTION_DETECTED
        let query = dsl_to_cypher("agent:code'; DROP TABLE--").expect_err("should fail");
        assert_eq!(query.code, "CYPHER_INJECTION_DETECTED");

        // 路径 2: 合法输入, agent value 必须参数化 (不出现在 cypher 字符串中)
        let query = dsl_to_cypher("agent:codex").expect("ok");
        assert!(
            !query.cypher.contains("codex"),
            "value must be parameterized, not inlined"
        );

        // 路径 3: modified 路径注入
        let err = dsl_to_cypher("modified:../etc/passwd").expect_err("should fail");
        assert_eq!(err.code, "CYPHER_INJECTION_DETECTED");
    }
}
