//! star-mcp binary integration tests (per ULYS-154 PR-2).
//!
//! 跑 `star-mcp` binary 加 assert 版本 / help.

use std::process::Command;

fn mcp_bin() -> Command {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_star-mcp") {
        return Command::new(path);
    }
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--bin", "star-mcp", "--"]);
    cmd
}

#[test]
fn star_mcp_version_exits_zero() {
    let output = mcp_bin()
        .arg("--version")
        .output()
        .expect("failed to execute star-mcp binary");
    assert!(
        output.status.success(),
        "star-mcp --version should exit 0, got {:?}",
        output.status
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.to_lowercase().contains("star-mcp") || stdout.to_lowercase().contains("star_mcp"),
        "stdout should mention star-mcp, got: {stdout}"
    );
}

#[test]
fn star_mcp_help_exits_zero() {
    let output = mcp_bin()
        .arg("--help")
        .output()
        .expect("failed to execute star-mcp binary");
    assert!(
        output.status.success(),
        "star-mcp --help should exit 0, got {:?}",
        output.status
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Usage") || stdout.contains("usage"),
        "stdout should contain Usage, got first 200: {}",
        stdout.chars().take(200).collect::<String>()
    );
}

#[test]
fn star_mcp_unknown_flag_exits_nonzero() {
    let output = mcp_bin()
        .arg("--unknown-flag-xyz")
        .output()
        .expect("failed to execute star-mcp binary");
    assert!(
        !output.status.success(),
        "unknown flag should exit non-zero, got {:?}",
        output.status
    );
}
