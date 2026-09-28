//! star-cli binary integration tests (per ULYS-154 PR-2).
//!
//! 跑 `star` binary 加 assert exit code / output / 版本 / help 行为.
//! 覆盖:
//!   - `--version` 显示版本
//!   - `--help` 显示主 usage
//!   - `skill` 子命令存在 (`star skill --help`)
//!   - 未知子命令返非 0
//!
//! 注: star-cli 是 bin-only (无 [lib]), inline test 跑不到, 故 binary integration test.
//!     per `--lib --bins` workflow 改动, cargo-tarpaulin 现在也能跑 bin.
//!
//! 历史: PR #185 (ULYS-154 PR-2) 引入, PR #190 (timeout 修复) 删. 现在从 #185
//! commit `bf78747a` 恢复. 见 issue #190 follow-up: 真实 star-cli 测试应在 inline
//! `#[cfg(test)] mod tests { ... }` in commands/*.rs (本文件保留 binary test
//! 作 integration 用, 不混 tarpaulin 跑).

use std::process::Command;

/// cargo bin path 解析 (per workspace target-dir).
fn star_bin() -> Command {
    // CARGO_BIN_EXE_<name> 是 cargo 测试环境提供的环境变量
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_star") {
        return Command::new(path);
    }
    // 否则用 cargo run --bin star (CI 环境)
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--bin", "star", "--"]);
    cmd
}

#[test]
fn star_cli_version_exits_zero_and_prints_name() {
    let output = star_bin()
        .arg("--version")
        .output()
        .expect("failed to execute star binary");
    assert!(output.status.success(), "star --version should exit 0, got {:?}", output.status);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.to_lowercase().contains("star"),
        "stdout should contain 'star', got: {stdout}"
    );
}

#[test]
fn star_cli_help_exits_zero_and_prints_usage() {
    let output = star_bin()
        .arg("--help")
        .output()
        .expect("failed to execute star binary");
    assert!(output.status.success(), "star --help should exit 0, got {:?}", output.status);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Usage") || stdout.contains("usage"),
        "stdout should contain Usage, got first 200 chars: {}",
        stdout.chars().take(200).collect::<String>()
    );
}

#[test]
fn star_cli_skill_help_exits_zero() {
    let output = star_bin()
        .args(["skill", "--help"])
        .output()
        .expect("failed to execute star binary");
    assert!(
        output.status.success(),
        "star skill --help should exit 0, got {:?}",
        output.status
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("add") && stdout.contains("list") && stdout.contains("remove"),
        "skill help should mention add/list/remove, got: {stdout}"
    );
}

#[test]
fn star_cli_unknown_subcommand_exits_nonzero() {
    let output = star_bin()
        .arg("nonexistent-subcommand-xyz")
        .output()
        .expect("failed to execute star binary");
    assert!(
        !output.status.success(),
        "unknown subcommand should exit non-zero, got {:?}",
        output.status
    );
}