//! tests/unit/test_backend_macos.rs — macOS 后端纯逻辑单测 (仅 macOS CI 跑)
//!
//! sandbox-exec profile 生成: network / fs 维过滤器 (per FR-5.3)。

#![cfg(target_os = "macos")]

use sandboxd::backend::macos::build_sandbox_profile;
use sandboxd::policy::{
    FsMode, FsPathMode, FsPathRule, FsPolicy, NetworkMode, NetworkPolicy, SessionSpec,
};
use uuid::Uuid;

#[test]
fn profile_deny_default() {
    let profile = build_sandbox_profile(&SessionSpec::default(), &Uuid::new_v4());
    assert!(profile.contains("(deny default)"));
    assert!(profile.contains("(allow process*)"));
}

#[test]
fn profile_network_allowlist() {
    let mut policy = SessionSpec::default();
    policy.network = NetworkPolicy::new(
        NetworkMode::Allowlist,
        vec!["github.com".to_string()],
        vec!["10.0.0.0/8".to_string()],
    );
    let profile = build_sandbox_profile(&policy, &Uuid::new_v4());
    assert!(profile.contains("(deny network*)"));
    assert!(profile.contains("(allow network-outbound (remote ip \"127.0.0.1\"))"));
    assert!(profile.contains("(allow network-outbound (remote ip \"10.0.0.0:*\"))"));
    // 域名无法在 seatbelt 表达 → 仅 IP + loopback (per 缺口 #6)
    assert!(!profile.contains("github.com"));
}

#[test]
fn profile_network_block_all() {
    let mut policy = SessionSpec::default();
    policy.network = NetworkPolicy::new(NetworkMode::BlockAll, vec![], vec![]);
    let profile = build_sandbox_profile(&policy, &Uuid::new_v4());
    assert!(profile.contains("(deny network*)"));
    assert!(!profile.contains("(allow network-outbound"));
}

#[test]
fn profile_fs_allowlist() {
    let mut policy = SessionSpec::default();
    policy.fs = FsPolicy::new(
        FsMode::Allowlist,
        vec![
            FsPathRule::new("/worktree", FsPathMode::Read),
            FsPathRule::new("/tmp", FsPathMode::ReadWrite),
        ],
    );
    let profile = build_sandbox_profile(&policy, &Uuid::new_v4());
    assert!(profile.contains("(deny file-read*)"));
    assert!(profile.contains("(deny file-write*)"));
    // Read → 仅 file-read
    assert!(profile.contains("(allow file-read* (subpath \"/worktree\"))"));
    assert!(!profile.contains("(allow file-write* (subpath \"/worktree\"))"));
    // ReadWrite → read + write
    assert!(profile.contains("(allow file-read* (subpath \"/tmp\"))"));
    assert!(profile.contains("(allow file-write* (subpath \"/tmp\"))"));
}
