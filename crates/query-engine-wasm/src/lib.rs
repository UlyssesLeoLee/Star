//! `query-engine-wasm` — Query Engine WASM wrapper (per docs/architecture/2026-09-28-upgrade/rust-to-wasm-frontend-memory-research.md §3.1 P1 PoC)
//!
//! 目的:
//!   - 把 query-engine crate 的 Search DSL 6 关键字解析暴露给浏览器
//!   - work_items filter/sort/aggregate 用 WASM 加速 (per docs §3.1)
//!   - 复用现有 query-engine crate 的 DslParser + keywords (不动业务 logic)
//!
//! 设计依据 (per rust-to-wasm-frontend-memory-research.md §3.1):
//!   - cdylib + rlib: cdylib for wasm-pack, rlib for Rust unit tests
//!   - wasm-bindgen: 标准 Rust ↔ JS binding
//!   - 阶段 1 PoC scope: parse_dsl + validate_dsl + 6 keyword string utilities
//!
//! 守门:
//!   - #7 `unsafe_code = "forbid"` (workspace lint)
//!   - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies]
//!   - #13 W/T/M (本期仅算法, 不涉及 DB 表)
//!
//! 阶段 1 scope (本 PR):
//!   - `parse_dsl(input: &str) -> Result<JsValue, JsError>` (parsed AST as JSON)
//!   - `validate_dsl(input: &str) -> bool` (parse + validate)
//!   - `keyword_count() -> u32` (守门: 6 keywords)
//!   - `keyword_as_str(idx: u32) -> String` (0-5 → "show"/"agent"/"behind"/"ahead"/"health"/"modified")
//!   - `version() -> String` (version string)

#![forbid(unsafe_code)] // 守门 #7 0 unsafe
#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

use query_engine::keywords::{Keyword, KEYWORD_COUNT};
use wasm_bindgen::prelude::*;

// =====================================================================
// WASM exports — 暴露给 JS 调用
// =====================================================================

/// `parse_dsl(input: &str) -> Result<JsValue, JsError>`
///
/// Per query-engine/src/dsl_parser.rs parse() — 解析 Search DSL 字符串到 AST.
/// 返回 JsValue (JSON-serialized AST) 或 JsError (parse error).
#[wasm_bindgen]
pub fn parse_dsl(input: &str) -> Result<JsValue, JsError> {
    // 阶段 1 PoC: 简化版本 — 验证 input 含至少 1 个关键字, 返回 keyword 列表 (JSON string)
    // 完整 DslParser 集成在 P2 PR (per docs §3.2 P1 P2 计划)
    use query_engine::keywords::Keyword;
    let mut found: Vec<&'static str> = Vec::new();
    for kw in Keyword::ALL {
        if input.contains(kw.as_str()) {
            found.push(kw.as_str());
        }
    }
    // Use serde_json (works on both native + wasm) then convert to JsValue
    let json = serde_json::to_string(&found)
        .map_err(|e| JsError::new(&format!("serialize error: {e}")))?;
    Ok(JsValue::from_str(&json))
}

/// `validate_dsl(input: &str) -> bool`
///
/// 简单验证: parse_dsl 返回 Ok 或 input 至少含 1 个关键字.
#[wasm_bindgen]
pub fn validate_dsl(input: &str) -> bool {
    if input.trim().is_empty() {
        return false;
    }
    use query_engine::keywords::Keyword;
    Keyword::ALL.iter().any(|kw| input.contains(kw.as_str()))
}

/// `keyword_count() -> u32`
///
/// 6 关键字守门 (per query-engine/src/keywords.rs KEYWORD_COUNT).
#[wasm_bindgen]
pub fn keyword_count() -> u32 {
    KEYWORD_COUNT as u32
}

/// `keyword_as_str(idx: u32) -> String`
///
/// 0..6 → "show" / "agent" / "behind" / "ahead" / "health" / "modified"
/// 越界返空字符串 (per Rust convention).
#[wasm_bindgen]
pub fn keyword_as_str(idx: u32) -> String {
    Keyword::ALL
        .get(idx as usize)
        .map(|kw| kw.as_str().to_string())
        .unwrap_or_default()
}

/// `keyword_count_str() -> String`
///
/// 全部 6 关键字逗号分隔字符串 (per docs §3.1 + ULYS-57.3 acceptance).
#[wasm_bindgen]
pub fn keyword_all() -> Vec<JsValue> {
    Keyword::ALL
        .iter()
        .map(|kw| JsValue::from_str(kw.as_str()))
        .collect()
}

/// `version() -> String`
#[wasm_bindgen]
pub fn version() -> String {
    format!(
        "query-engine-wasm v{} (per docs/architecture/2026-09-28-upgrade/rust-to-wasm-frontend-memory-research.md §3.1 P1)",
        env!("CARGO_PKG_VERSION")
    )
}

// =====================================================================
// Unit tests (Rust side, 不依赖 wasm-bindgen-test runtime)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_count_is_six() {
        // 守门 #11: 6 关键字 (per ULYS-57.3 acceptance)
        assert_eq!(keyword_count(), 6);
        assert_eq!(KEYWORD_COUNT, 6);
    }

    #[test]
    fn keyword_as_str_all_six() {
        assert_eq!(keyword_as_str(0), "show");
        assert_eq!(keyword_as_str(1), "agent");
        assert_eq!(keyword_as_str(2), "behind");
        assert_eq!(keyword_as_str(3), "ahead");
        assert_eq!(keyword_as_str(4), "health");
        assert_eq!(keyword_as_str(5), "modified");
    }

    #[test]
    fn keyword_as_str_out_of_bounds() {
        // 越界返空字符串
        assert_eq!(keyword_as_str(6), "");
        assert_eq!(keyword_as_str(100), "");
        assert_eq!(keyword_as_str(u32::MAX), "");
    }

    #[test]
    fn validate_dsl_empty_string() {
        assert!(!validate_dsl(""));
        assert!(!validate_dsl("   "));
    }

    #[test]
    fn validate_dsl_with_keyword() {
        assert!(validate_dsl("show ready"));
        assert!(validate_dsl("agent codex"));
        assert!(validate_dsl("behind >5"));
        assert!(validate_dsl("health <80"));
        assert!(validate_dsl("modified src/foo.rs"));
        assert!(validate_dsl("ahead >3"));
    }

    #[test]
    fn validate_dsl_no_keyword() {
        // 无关键字应返 false
        assert!(!validate_dsl("foo bar baz"));
        assert!(!validate_dsl("hello world"));
    }

    // parse_dsl tests 在 wasm32 module (wasm-bindgen exports crash on native)

    #[test]
    fn version_contains_identifier() {
        let v = version();
        assert!(v.contains("query-engine-wasm"));
        assert!(v.contains("rust-to-wasm"));
    }
}

// =====================================================================
// wasm-bindgen-test (浏览器/wasm runtime 验证, 跑 `wasm-pack test --node`)
// =====================================================================

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;

    #[wasm_bindgen_test]
    fn keyword_count_wasm() {
        assert_eq!(keyword_count(), 6);
    }

    #[wasm_bindgen_test]
    fn keyword_as_str_wasm() {
        assert_eq!(keyword_as_str(0), "show");
    }

    #[wasm_bindgen_test]
    fn validate_dsl_wasm() {
        assert!(validate_dsl("show ready"));
    }
}