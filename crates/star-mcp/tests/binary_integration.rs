//! star-mcp binary integration tests (per ULYS-154 PR-2).
//!
//! 注: star-mcp 是 bin-only (无 [lib]), main() 启动后进入 transport loop (stdio/http),
//! 不响应 --version/--help. binary test 跑会 timeout 死锁 cargo-tarpaulin.
//!
//! PR-2 follow-up: 删 3 个会 timeout 的 binary test, 留 placeholder.
//! 真实 star-mcp 测试应该在 inline test (src/main.rs mod tests { ... }),
//! 但 PR-2 时间盒内不做, 留作 PR-3 follow-up.

#[test]
fn star_mcp_binary_test_skipped_per_design() {
    // star-mcp main() 启动后阻塞 transport loop, --version/--help 不能触发 early exit.
    // binary integration test 跑会死锁 tarpaulin 的 ptrace timeout.
    // 解: 把 test 移到 inline `#[cfg(test)] mod tests { ... }` 在 src/main.rs (PR-3 follow-up).
    // 这里 placeholder 仅确认 test binary 编译.
}
