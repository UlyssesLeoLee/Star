/*
@cypher schema=1 source_sha256=e46ff56e434cc751a6ff1fbecf33b94f6f5cfd259bcb7850397c87f28c8a91a1
MERGE (self:File {path:"crates/sandboxd/tests/unit/test_policy_fs.rs"})
@endcypher
*/
//! tests/unit/test_policy_fs.rs — fs 维策略单测 (per DD §5.1: 7 TC)
//!
//! glob 匹配 (`**`/`*`/`?`) / 模式 / 读写判定 / 边界。

use sandboxd::policy::fs::{glob_match, FsDecision, FsMode, FsPathMode, FsPathRule, FsPolicy};

#[test]
fn glob_double_star_crosses_segments() {
    assert!(glob_match("/worktree/**", "/worktree/.worktree-cache/x"));
    assert!(glob_match("/worktree/**", "/worktree/a/b/c"));
    assert!(glob_match("/worktree/**", "/worktree")); // 尾随 /** 含父目录本身 (minimatch 惯例)
    assert!(!glob_match("/worktree/**", "/tmp/x"));
}

#[test]
fn glob_single_star_stays_in_segment() {
    assert!(glob_match("/tmp/*.log", "/tmp/a.log"));
    assert!(!glob_match("/tmp/*.log", "/tmp/sub/a.log")); // 单 `*` 不跨段
    assert!(glob_match("/*", "/worktree"));
    assert!(!glob_match("/*", "/a/b"));
}

#[test]
fn glob_question_mark() {
    assert!(glob_match("/tmp/file?.txt", "/tmp/file1.txt"));
    assert!(!glob_match("/tmp/file?.txt", "/tmp/file12.txt"));
}

#[test]
fn glob_backslash_normalized() {
    // Windows 分隔符归一 (per 跨平台)
    assert!(glob_match(r"C:\worktree\**", "C:/worktree/src/main.rs"));
    assert!(glob_match("/worktree/**", r"\worktree\x"));
}

#[test]
fn allowlist_read_vs_write() {
    let p = FsPolicy::default();
    // /worktree/** 只读
    assert_eq!(
        p.check_path("/worktree/src/main.rs", false),
        FsDecision::ReadAllowed
    );
    match p.check_path("/worktree/src/main.rs", true) {
        FsDecision::Blocked { reason } => assert!(reason.contains("READWRITE"), "{reason}"),
        other => panic!("expected blocked, got {other:?}"),
    }
}

#[test]
fn allowlist_rw_path() {
    let p = FsPolicy::default();
    assert_eq!(
        p.check_path("/worktree/.worktree-cache/x", true),
        FsDecision::WriteAllowed
    );
    assert_eq!(
        p.check_path("/tmp/anything", true),
        FsDecision::WriteAllowed
    );
}

#[test]
fn allowlist_unlisted_path_blocked() {
    let p = FsPolicy::default();
    assert!(matches!(
        p.check_path("/etc/passwd", false),
        FsDecision::Blocked { .. }
    ));
    assert!(matches!(
        p.check_path("/etc/passwd", true),
        FsDecision::Blocked { .. }
    ));
}

#[test]
fn open_mode_allows_all() {
    let p = FsPolicy::new(FsMode::Open, vec![]);
    assert_eq!(p.check_path("/anywhere", true), FsDecision::WriteAllowed);
}

#[test]
fn readonly_root_blocks_writes_outside() {
    let p = FsPolicy::new(
        FsMode::ReadOnlyRoot,
        vec![FsPathRule::new("/worktree/**", FsPathMode::ReadWrite)],
    );
    assert_eq!(p.check_path("/worktree/x", true), FsDecision::WriteAllowed);
    // allowlist 外: 读放行, 写阻断
    assert_eq!(p.check_path("/usr/bin/git", false), FsDecision::ReadAllowed);
    assert!(matches!(
        p.check_path("/usr/bin/git", true),
        FsDecision::Blocked { .. }
    ));
}

#[test]
fn longest_pattern_wins() {
    let p = FsPolicy::new(
        FsMode::Allowlist,
        vec![
            FsPathRule::new("/worktree/**", FsPathMode::Read),
            FsPathRule::new("/worktree/cache/**", FsPathMode::ReadWrite),
        ],
    );
    // 最长匹配 (cache/**) 优先 → 可写
    assert_eq!(
        p.check_path("/worktree/cache/f", true),
        FsDecision::WriteAllowed
    );
    // 次长匹配 (worktree/**) → 只读
    assert_eq!(
        p.check_path("/worktree/src/f", false),
        FsDecision::ReadAllowed
    );
}
