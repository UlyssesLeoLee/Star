//! `nl_translator.rs` — NL → Search DSL + Git Filter 翻译 (per FR-SEARCH-005)
//!
//! MVP 实现: 模板 + keyword matching, 不接真实 LLM.
//! 完整 LLM 调用 (claude / gpt) 留 P2 (per IMPL-PLAN §4.11 T11.3 备注).
//!
//! 输出:
//! - `nl_to_dsl` — NL → DSL string (e.g. "show conflicts" → "show:conflict")
//! - `nl_to_filters` — NL → Vec<Filter> (parsed DSL)
//!
//! 守门:
//! - #1 v25 `cargo test -p query-engine --lib` 4/4 pass
//! - #11 缺标比错标
//!
//! MVP 翻译规则 (per spec §8.7 + BD D-LLM-001):
//! - "conflict" / "冲突" / "conflicts" → show:conflict
//! - "stale" / "停滞" / "idle" → show:stale
//! - "behind" / "落后" → behind:>0 (默认)
//! - "modified" / "改动" / "changed" + filename → modified:<name>
//! - 健康分关键词 → health:<op><num>
//! - 否则 → empty DSL (MVP 不调用 LLM)

use crate::dsl_parser::{DslParser, Filter};
use crate::error::QueryError;

/// NL → DSL string (MVP 模板匹配)
///
/// # Errors
///
/// - `NL_TRANSLATION_FAILED` — 输入为空
pub fn nl_to_dsl(nl: &str) -> Result<String, QueryError> {
    let nl = nl.trim();
    if nl.is_empty() {
        return Err(QueryError::nl_translation(nl, "empty NL input"));
    }

    let lower = nl.to_ascii_lowercase();
    let mut dsl_parts: Vec<String> = Vec::new();

    // show state 关键词
    if lower.contains("conflict") || lower.contains("冲突") {
        dsl_parts.push("show:conflict".to_string());
    } else if lower.contains("stale") || lower.contains("停滞") || lower.contains("idle") {
        dsl_parts.push("show:stale".to_string());
    } else if lower.contains("diverged") || lower.contains("分叉") {
        dsl_parts.push("show:diverged".to_string());
    } else if lower.contains("merged") || lower.contains("已合并") {
        dsl_parts.push("show:merged".to_string());
    } else if lower.contains("ready") || lower.contains("就绪") {
        dsl_parts.push("show:ready".to_string());
    } else if lower.contains("running") || lower.contains("运行中") {
        dsl_parts.push("show:running".to_string());
    } else if lower.contains("waiting") || lower.contains("等待") {
        dsl_parts.push("show:waiting".to_string());
    }

    // agent 关键词
    for agent in ["codex", "claude-code", "claude", "opencode", "gemini", "sonnet"] {
        if lower.contains(agent) {
            dsl_parts.push(format!("agent:{agent}"));
            break;
        }
    }

    // behind / ahead 关键词
    if lower.contains("behind") || lower.contains("落后") {
        // 尝试提取数字
        if let Some(n) = extract_number(&lower) {
            dsl_parts.push(format!("behind:>{n}"));
        } else {
            dsl_parts.push("behind:>0".to_string());
        }
    }
    if lower.contains("ahead") || lower.contains("领先") {
        if let Some(n) = extract_number(&lower) {
            dsl_parts.push(format!("ahead:>{n}"));
        } else {
            dsl_parts.push("ahead:>0".to_string());
        }
    }

    // health 关键词
    if lower.contains("health") || lower.contains("健康") {
        if let Some(n) = extract_number(&lower) {
            let op = if lower.contains("below") || lower.contains("低于") || lower.contains("under") {
                "<"
            } else {
                ">"
            };
            dsl_parts.push(format!("health:{op}{n}"));
        }
    }

    // modified 关键词 + filename
    if lower.contains("modified") || lower.contains("改动") || lower.contains("changed") {
        // 尝试提取文件名 (word with .ext)
        if let Some(file) = extract_filename(&lower) {
            dsl_parts.push(format!("modified:{file}"));
        }
    }

    Ok(dsl_parts.join(" "))
}

/// NL → Vec<Filter> (parse DSL 后返回)
///
/// # Errors
///
/// - `NL_TRANSLATION_FAILED` — 输入为空
/// - `DSL_PARSE_ERROR` — 翻译结果无法 parse
pub fn nl_to_filters(nl: &str) -> Result<Vec<Filter>, QueryError> {
    let dsl = nl_to_dsl(nl)?;
    if dsl.is_empty() {
        return Ok(Vec::new());
    }
    DslParser::new().parse(&dsl)
}

/// 提取数字 (提取第一个数字 token)
fn extract_number(s: &str) -> Option<u32> {
    let mut current = String::new();
    for c in s.chars() {
        if c.is_ascii_digit() {
            current.push(c);
        } else if !current.is_empty() {
            return current.parse().ok();
        }
    }
    if current.is_empty() {
        None
    } else {
        current.parse().ok()
    }
}

/// 提取文件名 (word.word 模式)
fn extract_filename(s: &str) -> Option<String> {
    for word in s.split_whitespace() {
        if word.contains('.') && word.len() > 2 {
            // 跳过 stop words
            if matches!(word, "modified" | "changed" | "etc.") {
                continue;
            }
            // 净化: 去掉标点
            let cleaned: String = word
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/'))
                .collect();
            if !cleaned.is_empty() {
                return Some(cleaned);
            }
        }
    }
    None
}

// re-export for tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl_parser::Operator;

    #[test]
    fn nl_query_translator_graph_query_git_filter() {
        // 守门 UT-3 (T11): NL → DSL + Filter 翻译

        // 1. 简单 NL → DSL
        let dsl = nl_to_dsl("show conflicts modified auth.rs").expect("translate");
        assert!(dsl.contains("show:conflict"));
        assert!(dsl.contains("modified:auth.rs"));

        // 2. NL → Filters (parse 后)
        let filters = nl_to_filters("show stale worktrees").expect("translate");
        assert_eq!(filters.len(), 1);
        match &filters[0] {
            Filter::Show { values } => {
                assert_eq!(values[0], crate::keywords::ShowValue::Stale);
            }
            other => panic!("got {other:?}"),
        }

        // 3. behind + num
        let dsl = nl_to_dsl("worktrees behind 20").expect("translate");
        assert!(dsl.contains("behind:>20"), "got '{dsl}'");

        // 4. agent 翻译
        let dsl = nl_to_dsl("tasks running on codex").expect("translate");
        assert!(dsl.contains("agent:codex"), "got '{dsl}'");

        // 5. health below 50
        let dsl = nl_to_dsl("worktrees with health below 50").expect("translate");
        assert!(dsl.contains("health:<50"), "got '{dsl}'");
    }

    #[test]
    fn nl_empty_returns_error() {
        let err = nl_to_dsl("").expect_err("should fail");
        assert_eq!(err.code, "NL_TRANSLATION_FAILED");
    }

    #[test]
    fn nl_no_match_returns_empty_dsl() {
        let dsl = nl_to_dsl("random gibberish text").expect("ok");
        assert!(dsl.is_empty());
    }

    #[test]
    fn operator_in_filter() {
        // sanity: Operator::parse round-trip (cross-module test)
        let op = Operator::parse("<=").expect("parse");
        assert_eq!(op, Operator::LtEq);
    }
}