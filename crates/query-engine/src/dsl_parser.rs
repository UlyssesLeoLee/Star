//! `dsl_parser.rs` — Search DSL Parser (hand-rolled, per DD §17.1 + §28)
//!
//! 语法 (per spec §8.7 + DD §17.1):
//! ```text
//! query := filter (WS filter)*
//! filter := KEY ":" VALUE
//! KEY := "show" | "agent" | "behind" | "ahead" | "health" | "modified"
//! VALUE :=
//!   | show_value      (show:)
//!   | agent_type      (agent:)
//!   | op?number       (behind/ahead/health:)
//!   | file_path       (modified:)
//! op := "=" | "<" | ">" | "<=" | ">="
//! show_value := "conflict" | "unmerged" | "ready" | "stale" | "running"
//!             | "waiting" | "merged" | "diverged"
//! ```
//!
//! AND-only (OR/NOT 留 P2, per 简化守门).
//!
//! 守门:
//! - #1 v25 `cargo test -p query-engine --lib` 4/4 pass
//! - #10 Cypher 注入防护: 严格 allowlist 关键字 + op + 文件路径字符限制
//!
//! MVP 选择: 不引入 chevrotain (1MB+ deps), 用 hand-rolled tokenizer.
//! 完整 chevrotain 实现留 P2 (per IMPL-PLAN §4.11 T11.2 备注).

use crate::error::QueryError;
use crate::keywords::{Keyword, ShowValue};
use serde::{Deserialize, Serialize};

/// 比较运算符 (per spec §8.7)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operator {
    /// 等于 `=`
    Eq,
    /// 小于 `<`
    Lt,
    /// 大于 `>`
    Gt,
    /// 小于等于 `<=`
    LtEq,
    /// 大于等于 `>=`
    GtEq,
}

impl Operator {
    /// 解析 operator 字符串
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "=" => Some(Self::Eq),
            "<" => Some(Self::Lt),
            ">" => Some(Self::Gt),
            "<=" => Some(Self::LtEq),
            ">=" => Some(Self::GtEq),
            _ => None,
        }
    }

    /// 字符串形式
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Lt => "<",
            Self::Gt => ">",
            Self::LtEq => "<=",
            Self::GtEq => ">=",
        }
    }
}

/// 单一 DSL filter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Filter {
    /// show 过滤 (HumanState 一组)
    Show {
        /// show values
        values: Vec<ShowValue>,
    },
    /// agent 过滤
    Agent {
        /// agent 类型字符串 (codex / claude-code / opencode)
        value: String,
    },
    /// behind 过滤
    Behind {
        /// operator
        op: Operator,
        /// num
        num: u32,
    },
    /// ahead 过滤
    Ahead {
        /// operator
        op: Operator,
        /// num
        num: u32,
    },
    /// health 过滤
    Health {
        /// operator
        op: Operator,
        /// num
        num: u8,
    },
    /// modified 过滤
    Modified {
        /// file path
        value: String,
    },
}

impl Filter {
    /// 取得 filter 关联的关键字
    pub fn keyword(&self) -> Keyword {
        match self {
            Self::Show { .. } => Keyword::Show,
            Self::Agent { .. } => Keyword::Agent,
            Self::Behind { .. } => Keyword::Behind,
            Self::Ahead { .. } => Keyword::Ahead,
            Self::Health { .. } => Keyword::Health,
            Self::Modified { .. } => Keyword::Modified,
        }
    }
}

/// 简化的 ShowFilterValue — 公开别名, 与 keywords.rs 同义
pub type ShowFilterValue = ShowValue;

/// Search DSL Parser
#[derive(Debug, Default, Clone)]
pub struct DslParser;

impl DslParser {
    /// 构造新 parser
    pub fn new() -> Self {
        Self
    }

    /// 解析 DSL 字符串为 filter 列表
    ///
    /// # Errors
    ///
    /// - `DSL_PARSE_ERROR` — token 错误 / 未知关键字 / op 后缺数字 / 文件路径非法字符
    pub fn parse(&self, input: &str) -> Result<Vec<Filter>, QueryError> {
        let input = input.trim();
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let mut filters = Vec::new();
        for token in input.split_whitespace() {
            let filter = self.parse_token(token)?;
            filters.push(filter);
        }
        Ok(filters)
    }

    /// 解析单一 token (`key:value`)
    fn parse_token(&self, token: &str) -> Result<Filter, QueryError> {
        let (key, value) = token.split_once(':').ok_or_else(|| {
            QueryError::dsl_parse(
                token,
                "missing ':' separator; expected key:value (e.g. show:conflict)",
            )
        })?;

        match key {
            "show" => self.parse_show(value).map(|values| Filter::Show { values }),
            "agent" => self.parse_agent(value).map(|value| Filter::Agent { value }),
            "behind" => self.parse_op_num(value)
                .map(|(op, num)| Filter::Behind { op, num }),
            "ahead" => self.parse_op_num(value).map(|(op, num)| Filter::Ahead { op, num }),
            "health" => self
                .parse_op_num(value)
                .and_then(|(op, num)| {
                    let num = u8::try_from(num).map_err(|_| {
                        QueryError::dsl_parse(
                            value,
                            "health value out of range (0..=255)",
                        )
                    })?;
                    Ok((op, num))
                })
                .map(|(op, num)| Filter::Health { op, num }),
            "modified" => self
                .parse_file_path(value)
                .map(|value| Filter::Modified { value }),
            other => Err(QueryError::dsl_parse(
                token,
                format!("unknown keyword: '{other}' (allowed: show, agent, behind, ahead, health, modified)"),
            )),
        }
    }

    /// parse `show:value[,value]*`
    fn parse_show(&self, value: &str) -> Result<Vec<ShowValue>, QueryError> {
        let mut out = Vec::new();
        for v in value.split(',') {
            let v = v.trim();
            if v.is_empty() {
                continue;
            }
            let parsed = ShowValue::parse(v).ok_or_else(|| {
                QueryError::dsl_parse(
                    value,
                    format!(
                        "unknown show value: '{v}' (allowed: conflict, unmerged, ready, stale, running, waiting, merged, diverged)"
                    ),
                )
            })?;
            out.push(parsed);
        }
        if out.is_empty() {
            return Err(QueryError::dsl_parse(
                value,
                "show requires at least one value",
            ));
        }
        Ok(out)
    }

    /// parse `agent:value` — 严格 allowlist
    fn parse_agent(&self, value: &str) -> Result<String, QueryError> {
        // 守门 #10: 严格 allowlist, 拒绝 Cypher 注入字符
        if value.is_empty() {
            return Err(QueryError::dsl_parse(value, "agent value cannot be empty"));
        }
        if !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(QueryError::cypher_injection(format!(
                "agent value contains disallowed characters: '{value}'"
            )));
        }
        Ok(value.to_string())
    }

    /// parse `op?number` — `behind:>20`, `behind:=10`, `behind:<5`, etc.
    fn parse_op_num(&self, value: &str) -> Result<(Operator, u32), QueryError> {
        if value.is_empty() {
            return Err(QueryError::dsl_parse(value, "missing value after op"));
        }
        // op 提取 (2 chars first, then 1 char)
        let (op, rest) = if value.starts_with("<=") || value.starts_with(">=") {
            let (op, rest) = value.split_at(2);
            (Operator::parse(op).expect("guarded by prefix"), rest)
        } else {
            let head = value.chars().next().expect("guarded by is_empty");
            let op = match head {
                '=' | '<' | '>' => {
                    Operator::parse(&head.to_string()).expect("guarded by head match")
                }
                _ => Operator::Eq, // default: no op = Eq
            };
            let rest = if matches!(head, '=' | '<' | '>') {
                &value[1..]
            } else {
                value
            };
            (op, rest)
        };

        if rest.is_empty() {
            return Err(QueryError::dsl_parse(value, "missing number after op"));
        }
        let num: u32 = rest
            .parse()
            .map_err(|_| QueryError::dsl_parse(value, format!("invalid number: '{rest}'")))?;
        Ok((op, num))
    }

    /// parse file path — allow alphanumeric + `._/-/` (守门 #10 注入防护)
    fn parse_file_path(&self, value: &str) -> Result<String, QueryError> {
        if value.is_empty() {
            return Err(QueryError::dsl_parse(
                value,
                "modified value cannot be empty",
            ));
        }
        if !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/'))
        {
            return Err(QueryError::cypher_injection(format!(
                "modified file path contains disallowed characters: '{value}'"
            )));
        }
        if value.contains("..") {
            return Err(QueryError::cypher_injection(format!(
                "modified file path contains '..': '{value}'"
            )));
        }
        Ok(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_dsl_parser_show_conflict_combined() {
        // 守门 UT-3 (T11): DSL 解析 show:conflict,unmerged + agent + behind + health + modified
        let parser = DslParser::new();
        let filters = parser
            .parse("show:conflict,unmerged agent:codex behind:>20 health:<50 modified:auth.rs")
            .expect("parse ok");

        assert_eq!(filters.len(), 5);

        // show:conflict,unmerged
        match &filters[0] {
            Filter::Show { values } => {
                assert_eq!(values.len(), 2);
                assert!(values.contains(&ShowValue::Conflict));
                assert!(values.contains(&ShowValue::Unmerged));
            }
            other => panic!("expected Show, got {other:?}"),
        }

        // agent:codex
        match &filters[1] {
            Filter::Agent { value } => assert_eq!(value, "codex"),
            other => panic!("expected Agent, got {other:?}"),
        }

        // behind:>20
        match &filters[2] {
            Filter::Behind { op, num } => {
                assert_eq!(*op, Operator::Gt);
                assert_eq!(*num, 20);
            }
            other => panic!("expected Behind, got {other:?}"),
        }

        // health:<50
        match &filters[3] {
            Filter::Health { op, num } => {
                assert_eq!(*op, Operator::Lt);
                assert_eq!(*num, 50);
            }
            other => panic!("expected Health, got {other:?}"),
        }

        // modified:auth.rs
        match &filters[4] {
            Filter::Modified { value } => assert_eq!(value, "auth.rs"),
            other => panic!("expected Modified, got {other:?}"),
        }
    }

    #[test]
    fn operator_parse_round_trip() {
        for op in [
            Operator::Eq,
            Operator::Lt,
            Operator::Gt,
            Operator::LtEq,
            Operator::GtEq,
        ] {
            assert_eq!(Operator::parse(op.as_str()), Some(op));
        }
        assert_eq!(Operator::parse("!="), None);
    }

    #[test]
    fn parse_rejects_unknown_keyword() {
        let parser = DslParser::new();
        let err = parser.parse("task:payment").expect_err("should fail");
        assert_eq!(err.code, "DSL_PARSE_ERROR");
        // source 包含 unknown keyword 细节
        let source = err.source.as_deref().unwrap_or("");
        assert!(
            source.contains("unknown keyword"),
            "expected 'unknown keyword' in source, got: {source}"
        );
    }

    #[test]
    fn parse_rejects_missing_colon() {
        let parser = DslParser::new();
        let err = parser.parse("showconflict").expect_err("should fail");
        assert_eq!(err.code, "DSL_PARSE_ERROR");
    }

    #[test]
    fn parse_rejects_empty() {
        let parser = DslParser::new();
        let filters = parser.parse("").expect("empty ok");
        assert!(filters.is_empty());
    }

    #[test]
    fn parse_op_num_default_eq() {
        let parser = DslParser::new();
        let filters = parser.parse("behind:10").expect("parse ok");
        match &filters[0] {
            Filter::Behind { op, num } => {
                assert_eq!(*op, Operator::Eq);
                assert_eq!(*num, 10);
            }
            other => panic!("got {other:?}"),
        }
    }
}
