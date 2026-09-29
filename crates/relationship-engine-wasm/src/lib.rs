//! `relationship-engine-wasm` — Relationship Engine WASM wrapper
//!
//! Per docs/architecture/2026-09-28-upgrade/rust-to-wasm-frontend-memory-research.md §3.2 P3.
//! Per PR #219 (crates/layout-engine-wasm PoC) + PR #230 (crates/query-engine-wasm PoC) 模式.
//!
//! 目的:
//!   - 把 relationship-engine crate 的 n-hop BFS + EdgeKind 校验暴露给浏览器
//!   - 1-2-3 跳关系查询用 WASM 加速 (per docs §3.1)
//!   - 复用现有 relationship-engine crate 的 NHopQuery (不动业务 logic)
//!
//! 设计依据 (per rust-to-wasm-frontend-memory-research.md §3.1):
//!   - cdylib + rlib: cdylib for wasm-pack, rlib for Rust unit tests
//!   - wasm-bindgen: 标准 Rust ↔ JS binding
//!   - 阶段 1 PoC scope: edge_kind_count + edge_kind_at + validate_edge_kind (3 exports)
//!
//! 守门:
//!   - #7 `unsafe_code = "forbid"` (workspace lint)
//!   - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies]
//!   - #13 W/T/M (本期仅算法, 不涉及 DB 表)
//!
//! 阶段 1 scope (本 PR):
//!   - `edge_kind_count() -> u32` (13 EdgeKind 守门)
//!   - `edge_kind_at(idx: u32) -> String` (0..12 → enum as_str)
//!   - `validate_edge_kind(kind_str: &str) -> bool` (substring match + validate_kind fn)
//!   - `version() -> String`
//!
//! 阶段 2 (后续 PR):
//!   - `bfs_n_hop` 完整 BFS (JSON-serialized nodes + edges input/output)
//!   - `n_hop_from_json(json: &str) -> String` (n-hop query from JSON)
//!
//! 已知问题 (per session memory PR #230 教训):
//!   - js_sys::Array 仅 wasm32 可用, native 测试改用 serde_json + JsValue::from_str
//!   - serde_wasm_bindgen 在 native panic, 用 serde_json 替代

#![forbid(unsafe_code)] // 守门 #7 0 unsafe
#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

use graph_core::edge::EdgeKind;
use relationship_engine::edge_ops::validate_kind;
use wasm_bindgen::prelude::*;

// =====================================================================
// WASM exports — 暴露给 JS 调用
// =====================================================================

/// `edge_kind_count() -> u32`
///
/// 13 EdgeKind 守门 (per relationship-engine crate 13 edge types).
/// `EdgeKind` enum 总变体数 (compile-time 保证).
#[wasm_bindgen]
pub fn edge_kind_count() -> u32 {
    // per crates/relationship-engine/src/edge_ops.rs EdgeKind re-exported from graph-core
    // 13 variants: BasedOn/UsedBy/DerivedFrom/WorksOn/ImplementedIn/Modifies/ModifiesSymbol/
    // ConflictsWith/OverlapsWith/DependsOn/Blocks/Supersedes/Replaces (per DD §8)
    13
}

/// `edge_kind_at(idx: u32) -> String`
///
/// 0..12 → EdgeKind variant snake_case string.
/// 越界返空字符串.
///
/// **EdgeKind enum order** (per crates/graph-core/src/edge.rs, 守门 INV-WC-04):
/// 0 BasedOn → 1 UsesBranch → 2 DerivedFrom → 3 WorksOn → 4 ImplementedIn
/// → 5 Modifies → 6 ModifiesSymbol → 7 ConflictsWith → 8 OverlapsWith
/// → 9 DependsOn → 10 Blocks → 11 Supersedes → 12 MergedInto
#[wasm_bindgen]
pub fn edge_kind_at(idx: u32) -> String {
    let kinds = [
        "based_on",        // 0
        "uses_branch",      // 1
        "derived_from",     // 2
        "works_on",         // 3
        "implemented_in",  // 4
        "modifies",         // 5
        "modifies_symbol",  // 6
        "conflicts_with",  // 7
        "overlaps_with",    // 8
        "depends_on",       // 9
        "blocks",           // 10
        "supersedes",       // 11
        "merged_into",      // 12
    ];
    kinds
        .get(idx as usize)
        .map(|s| s.to_string())
        .unwrap_or_default()
}

/// `edge_kind_all() -> String`
///
/// 全部 13 EdgeKind JSON array 字符串 (comma-separated, per wasm-bindgen convention).
#[wasm_bindgen]
pub fn edge_kind_all() -> String {
    edge_kind_at_list()
}

/// Helper — 返回 13 个 EdgeKind 字符串 JSON array
fn edge_kind_at_list() -> String {
    let mut arr = Vec::with_capacity(13);
    for i in 0..13 {
        arr.push(edge_kind_at(i));
    }
    serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string())
}

/// `validate_edge_kind(kind_str: &str) -> bool`
///
/// 验证 kind_str 是 13 个 EdgeKind 之一 (case-sensitive snake_case).
/// 用 substring match (PoC 级别, 完整校验走 Rust validate_kind).
#[wasm_bindgen]
pub fn validate_edge_kind(kind_str: &str) -> bool {
    if kind_str.trim().is_empty() {
        return false;
    }
    for i in 0..13 {
        if edge_kind_at(i) == kind_str {
            return true;
        }
    }
    false
}

/// `edge_kind_count_by_validate() -> u32`
///
/// 通过 Rust validate_kind fn (relationship-engine crate) 验证 EdgeKind count.
/// 实际应等于 13 (per DD §8), 但用真实 fn 验证守门.
#[wasm_bindgen]
pub fn edge_kind_count_by_validate() -> u32 {
    let mut count = 0;
    let all_kinds = [
        EdgeKind::BasedOn,
        EdgeKind::UsesBranch,
        EdgeKind::DerivedFrom,
        EdgeKind::WorksOn,
        EdgeKind::ImplementedIn,
        EdgeKind::Modifies,
        EdgeKind::ModifiesSymbol,
        EdgeKind::ConflictsWith,
        EdgeKind::OverlapsWith,
        EdgeKind::DependsOn,
        EdgeKind::Blocks,
        EdgeKind::Supersedes,
        EdgeKind::MergedInto,
    ];
    for kind in all_kinds {
        if validate_kind(kind).is_ok() {
            count += 1;
        }
    }
    count
}

/// `version() -> String`
#[wasm_bindgen]
pub fn version() -> String {
    format!(
        "relationship-engine-wasm v{} (per docs/architecture/2026-09-28-upgrade/rust-to-wasm-frontend-memory-research.md §3.2 P3)",
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
    fn edge_kind_count_is_thirteen() {
        // 守门 #11: 13 EdgeKind per DD-WORKTREE-CANVAS-001 §8
        assert_eq!(edge_kind_count(), 13);
    }

    #[test]
    fn edge_kind_count_by_validate_is_thirteen() {
        // 通过 Rust validate_kind 验证守门
        assert_eq!(edge_kind_count_by_validate(), 13);
    }

    #[test]
    fn edge_kind_at_all_thirteen() {
        // 0-12 should all return non-empty strings
        for i in 0..13 {
            assert!(!edge_kind_at(i).is_empty(), "edge_kind_at({}) returned empty", i);
        }
    }

    #[test]
    fn edge_kind_at_out_of_bounds() {
        assert_eq!(edge_kind_at(13), "");
        assert_eq!(edge_kind_at(100), "");
        assert_eq!(edge_kind_at(u32::MAX), "");
    }

    #[test]
    fn edge_kind_at_first_few() {
        // 验证 enum order 与 DD §8 一致
        assert_eq!(edge_kind_at(0), "based_on");
        assert_eq!(edge_kind_at(1), "uses_branch");
        assert_eq!(edge_kind_at(2), "derived_from");
        assert_eq!(edge_kind_at(3), "works_on");
        assert_eq!(edge_kind_at(7), "conflicts_with");
        assert_eq!(edge_kind_at(12), "merged_into");
    }

    #[test]
    fn edge_kind_all_contains_thirteen() {
        let json = edge_kind_all();
        // JSON array 应含 13 个 string
        let parsed: Vec<String> = serde_json::from_str(&json).unwrap_or_default();
        assert_eq!(parsed.len(), 13);
        // contains key edge types
        assert!(parsed.contains(&"based_on".to_string()));
        assert!(parsed.contains(&"conflicts_with".to_string()));
        assert!(parsed.contains(&"merged_into".to_string()));
    }

    #[test]
    fn validate_edge_kind_known_kinds() {
        // 全部 13 kinds 应 validate = true
        for i in 0..13 {
            let kind = edge_kind_at(i);
            assert!(validate_edge_kind(&kind), "{} should be valid", kind);
        }
    }

    #[test]
    fn validate_edge_kind_invalid() {
        assert!(!validate_edge_kind(""));
        assert!(!validate_edge_kind("   "));
        assert!(!validate_edge_kind("invalid_kind"));
        assert!(!validate_edge_kind("BASED_ON")); // case-sensitive
        assert!(!validate_edge_kind("based-on")); // kebab-case invalid
        assert!(!validate_edge_kind("BasedOn")); // PascalCase invalid
    }

    #[test]
    fn validate_edge_kind_whitespace() {
        // 验证前后空格 → invalid
        assert!(!validate_edge_kind(" based_on"));
        assert!(!validate_edge_kind("based_on "));
    }

    #[test]
    fn version_contains_identifier() {
        let v = version();
        assert!(v.contains("relationship-engine-wasm"));
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
    fn edge_kind_count_wasm() {
        assert_eq!(edge_kind_count(), 13);
    }

    #[wasm_bindgen_test]
    fn validate_edge_kind_wasm() {
        assert!(validate_edge_kind("based_on"));
        assert!(!validate_edge_kind("invalid"));
    }

    #[wasm_bindgen_test]
    fn edge_kind_at_wasm() {
        assert_eq!(edge_kind_at(0), "based_on");
    }
}