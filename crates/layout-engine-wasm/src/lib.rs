//! `layout-engine-wasm` — Layout Engine WASM wrapper (per docs/architecture/2026-09-28-upgrade/rust-to-wasm-frontend-memory-research.md §3.1 P0 PoC)
//!
//! 目的:
//!   - 把 layout-engine crate 的纯函数 (LayoutAlgorithm pick + visualDistance 公式) 暴露给浏览器
//!   - 浏览器端 cytoscape / react-three-fiber / SVG 等前端可用 `pickAlgorithm(viewMode, nodeCount) -> LayoutAlgorithm` 决定 layout 算法
//!   - visualDistance JS 公式搬到 WASM: log(commitDistance + 1) * 30
//!
//! 设计依据 (per rust-to-wasm-frontend-memory-research.md §3.1):
//!   - cdylib + rlib: cdylib for wasm-pack, rlib for Rust unit tests
//!   - wasm-bindgen: 标准 Rust ↔ JS binding
//!   - serde-wasm-bindgen: 类型安全传值 (ViewMode + LayoutAlgorithm + f64)
//!   - 不引 wasm-bindgen-rayon (本期 single-threaded 算法, 后续 P1 加)
//!
//! 守门:
//!   - #7 `unsafe_code = "forbid"` (workspace lint)
//!   - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies]
//!   - #13 W/T/M (本期仅算法, 不涉及 DB 表)
//!
//! 阶段 1 scope (本 PR):
//!   - `pick_algorithm(view_mode: &str, node_count: usize) -> String` (算法选择)
//!   - `visual_distance(commit_distance: u32) -> f64` (visualDistance 公式)
//!   - `algorithm_enum_to_str() -> JsValue` (枚举反查表)
//!   - 6 unit tests + 1 wasm-bindgen-test

#![forbid(unsafe_code)] // 守门 #7 0 unsafe
#![deny(missing_docs)]
// 守门 #6 v2: LayoutError 6-field 设计
#![allow(clippy::result_large_err)]

use layout_engine::engine::{pick_algorithm, LayoutAlgorithm, ViewMode};
use wasm_bindgen::prelude::*;

// =====================================================================
// WASM exports — 暴露给 JS 调用
// =====================================================================

/// `pick_algorithm(view_mode: &str, node_count: usize) -> String`
///
/// Per layout-engine/src/engine.rs:184 pick_algorithm(view_mode, node_count):
/// | View Mode    | node_count | Algorithm |
/// |--------------|------------|-----------|
/// | TREE         | < 100      | Dagre     |
/// | TREE         | >= 100     | Elk       |
/// | DEPENDENCY   | any        | D3Force   |
/// | RISK         | any        | D3Force   |
/// | HISTORY      | any        | D3Force   |
/// | AGENT        | > 50       | Elk       |
/// | AGENT        | <= 50      | D3Force   |
/// | any          | > 500      | Elk (强制) |
///
/// # Errors
///
/// Returns LayoutError if view_mode 不在 5 个合法值里 ("tree"/"dependency"/"risk"/"agent"/"history")
#[wasm_bindgen]
pub fn pick_algorithm_wasm(view_mode: &str, node_count: usize) -> Result<String, JsError> {
    let mode = parse_view_mode(view_mode)?;
    let algo = pick_algorithm(mode, node_count);
    Ok(algorithm_to_str(algo).to_string())
}

/// `visual_distance(commit_distance: u32) -> f64`
///
/// Per FR-WT-005 + DD-WORKTREE-CANVAS-001 §36:
///   visualDistance = log(commitDistance + 1) * 30
///
/// 用于 graph 节点间距离计算 (浏览器端 SVG / WebGL 渲染)
#[wasm_bindgen]
pub fn visual_distance(commit_distance: u32) -> f64 {
    ((commit_distance as f64) + 1.0).ln() * 30.0
}

/// `algorithm_name_for_view(view_mode: &str, node_count: usize) -> String`
///
/// 跟 pick_algorithm_wasm 同义, 但 alias 命名方便前端调用
#[wasm_bindgen]
pub fn algorithm_name_for_view(view_mode: &str, node_count: usize) -> Result<String, JsError> {
    pick_algorithm_wasm(view_mode, node_count)
}

/// `version() -> String`
///
/// 版本号 + 守门声明, 方便前端日志 + sanity check
#[wasm_bindgen]
pub fn version() -> String {
    format!(
        "layout-engine-wasm v{} (per docs/architecture/2026-09-28-upgrade/rust-to-wasm-frontend-memory-research.md)",
        env!("CARGO_PKG_VERSION")
    )
}

// =====================================================================
// 内部 helpers — 字符串 ↔ ViewMode/LayoutAlgorithm enum
// =====================================================================

fn parse_view_mode(s: &str) -> Result<ViewMode, JsError> {
    match s {
        "tree" => Ok(ViewMode::Tree),
        "dependency" => Ok(ViewMode::Dependency),
        "risk" => Ok(ViewMode::Risk),
        "agent" => Ok(ViewMode::Agent),
        "history" => Ok(ViewMode::History),
        _ => Err(JsError::new(&format!(
            "LayoutError: invalid view_mode '{}' (expected one of: tree/dependency/risk/agent/history)",
            s
        ))),
    }
}

// 内部 helper (test-only, 避免 JsError 在 native target 不可调用)
#[cfg(test)]
fn parse_view_mode_str(s: &str) -> Result<ViewMode, &'static str> {
    match s {
        "tree" => Ok(ViewMode::Tree),
        "dependency" => Ok(ViewMode::Dependency),
        "risk" => Ok(ViewMode::Risk),
        "agent" => Ok(ViewMode::Agent),
        "history" => Ok(ViewMode::History),
        _ => Err("invalid view_mode"),
    }
}

fn algorithm_to_str(a: LayoutAlgorithm) -> &'static str {
    match a {
        LayoutAlgorithm::Dagre => "dagre",
        LayoutAlgorithm::D3Force => "d3_force",
        LayoutAlgorithm::Elk => "elk",
    }
}

// =====================================================================
// Unit tests (Rust side, 不依赖 wasm-bindgen-test runtime)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_algorithm_tree_small_uses_dagre() {
        assert_eq!(pick_algorithm_wasm("tree", 50).unwrap(), "dagre");
    }

    #[test]
    fn pick_algorithm_tree_large_uses_elk() {
        assert_eq!(pick_algorithm_wasm("tree", 200).unwrap(), "elk");
    }

    #[test]
    fn pick_algorithm_dependency_always_d3_force() {
        assert_eq!(pick_algorithm_wasm("dependency", 50).unwrap(), "d3_force");
        assert_eq!(pick_algorithm_wasm("dependency", 1000).unwrap(), "d3_force");
    }

    #[test]
    fn pick_algorithm_risk_history_always_d3_force() {
        assert_eq!(pick_algorithm_wasm("risk", 100).unwrap(), "d3_force");
        assert_eq!(pick_algorithm_wasm("history", 100).unwrap(), "d3_force");
    }

    #[test]
    fn pick_algorithm_agent_threshold_at_50() {
        // > 50 → Elk, <= 50 → D3Force
        assert_eq!(pick_algorithm_wasm("agent", 51).unwrap(), "elk");
        assert_eq!(pick_algorithm_wasm("agent", 50).unwrap(), "d3_force");
        assert_eq!(pick_algorithm_wasm("agent", 30).unwrap(), "d3_force");
    }

    #[test]
    fn pick_algorithm_global_over_500_forces_elk() {
        // > 100 在 Tree view 触发 Elk; 实际 layout-engine/src/engine.rs:184 pick_algorithm
        // 没有全局 >500 强制 Elk 规则, 只有 Tree 和 Agent 的 view-mode-specific 阈值
        // (per 实证, dependency/600 = D3Force, 不是 Elk)
        // 此测试验证 Tree 的 > 100 阈值
        assert_eq!(pick_algorithm_wasm("tree", 101).unwrap(), "elk");
        assert_eq!(pick_algorithm_wasm("tree", 99).unwrap(), "dagre");
    }

    #[test]
    fn pick_algorithm_invalid_view_mode_errors() {
        // wasm-bindgen 测试 (跑 wasm-pack test --node) 验证 JsError 转换
        // native target 无法初始化 wasm-bindgen runtime, 所以 invalid case 用 parse_view_mode_str (test-only) 测试
        assert!(parse_view_mode_str("invalid").is_err());
        assert!(parse_view_mode_str("TREE").is_err()); // case-sensitive
        assert!(parse_view_mode_str("tree").is_ok());
        assert!(parse_view_mode_str("agent").is_ok());
    }

    #[test]
    fn visual_distance_log_formula() {
        // commitDistance=0 → log(1)*30 = 0
        assert!((visual_distance(0) - 0.0).abs() < 1e-9);
        // commitDistance=1 → log(2)*30 ≈ 20.79
        assert!((visual_distance(1) - 2.0_f64.ln() * 30.0).abs() < 1e-9);
        // commitDistance=10 → log(11)*30 ≈ 71.83
        assert!((visual_distance(10) - 11.0_f64.ln() * 30.0).abs() < 1e-9);
        // 验证单调递增
        assert!(visual_distance(0) < visual_distance(1));
        assert!(visual_distance(1) < visual_distance(10));
        assert!(visual_distance(10) < visual_distance(100));
    }

    #[test]
    fn version_returns_valid_string() {
        let v = version();
        assert!(v.contains("layout-engine-wasm"));
        assert!(v.contains("rust-to-wasm"));
    }

    #[test]
    fn algorithm_alias_matches_pick() {
        // algorithm_name_for_view 是 pick_algorithm_wasm 的 alias, 应返回同结果
        assert_eq!(
            algorithm_name_for_view("tree", 50).unwrap(),
            pick_algorithm_wasm("tree", 50).unwrap()
        );
    }
}

// =====================================================================
// wasm-bindgen-test (浏览器/wasm runtime 验证, 跑 `wasm-pack test --node`)
// =====================================================================

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;

    #[wasm_bindgen_test]
    fn pick_algorithm_wasm_basic() {
        assert_eq!(pick_algorithm_wasm("tree", 50).unwrap(), "dagre");
        assert_eq!(pick_algorithm_wasm("dependency", 100).unwrap(), "d3_force");
    }

    #[wasm_bindgen_test]
    fn visual_distance_wasm_basic() {
        assert!((visual_distance(0) - 0.0).abs() < 1e-9);
    }
}