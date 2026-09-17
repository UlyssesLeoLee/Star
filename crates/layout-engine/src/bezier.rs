//! `bezier.rs` — Bezier path for edge rendering (per DD §31)
//!
//! 三次 Bezier 曲线 (per DD §31 `bezier_path` 函数).

/// 生成 SVG 三次 Bezier 路径字符串 (per DD §31)
///
/// ```
/// let p = bezier_path(0.0, 0.0, 100.0, 100.0);
/// assert!(p.starts_with("M 0 0"));
/// ```
pub fn bezier_path(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    let dx = x2 - x1;
    format!(
        "M {} {} C {} {} {} {} {} {}",
        x1,
        y1,
        x1 + dx * 0.25,
        y1,
        x2 - dx * 0.25,
        y2,
        x2,
        y2,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bezier_path_format_correct() {
        let p = bezier_path(0.0, 0.0, 100.0, 100.0);
        assert!(p.starts_with("M 0 0"));
        assert!(p.contains("C 25 0 75 100 100 100"));
    }
}
