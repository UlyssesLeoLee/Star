//! 集成测试 (IT) — worktree-shared-dir path C reader 真实配置接入验证
//!
//! **crate**: worktree-service (integration test target)
//! **per**: ULYS-177 (per ULYS-158 §3.3 path C 实装)
//! **覆盖**:
//! - V0: 缺失 config.json → empty (per FR-ORCA-007 fallback 语义)
//! - V1: 缺失字段 / null 字段 → empty
//! - V2: 三类配置项 (绝对路径 / 相对路径 / 混合) 正确归一化
//! - V3: 重复目录去重
//! - V4: 非字符串元素静默丢弃
//! - V5: 跨平台路径语义 (POSIX 路径在 Windows 下也作为 root-anchored 接受)
//! - V6: live multica config 烟雾测试 — 把测试用 config.json 写到临时目录,
//!   跑真实 reader, 断言读到的 entry 数量与内容
//!
//! 与 unit test (shared_dir_sources.rs::tests) 的区别:
//! - IT 走的是 crate public API (collect_cli_config_shared_dirs), 而非 inner fn
//! - IT 验证 fs layout (`<workspace>/multica-config/config.json`),
//!   而非逻辑分支

use std::fs;
use std::path::Path;

use worktree_service::{
    collect_cli_config_shared_dirs, collect_cli_config_shared_dirs_strict, CliConfigError,
    CLI_CONFIG_DIR_NAME, CLI_CONFIG_FILE_NAME, CLI_CONFIG_FIELD, SOURCE_CLI_CONFIG,
};

/// RAII tempdir helper to avoid pulling `tempfile` as a new dev-dep.
struct TempDir(std::path::PathBuf);

impl TempDir {
    fn new(prefix: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let p = std::env::temp_dir().join(format!("wtsd-it-{}-{}", prefix, nanos));
        fs::create_dir_all(&p).expect("create tempdir");
        Self(p)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write_config(dir: &Path, body: &str) {
    let cfg_dir = dir.join(CLI_CONFIG_DIR_NAME);
    fs::create_dir_all(&cfg_dir).expect("mkdir multica-config");
    fs::write(cfg_dir.join(CLI_CONFIG_FILE_NAME), body).expect("write config.json");
}

#[test]
fn it_v0_missing_config_file_yields_empty() {
    let tmp = TempDir::new("v0");
    let out = collect_cli_config_shared_dirs(tmp.path());
    assert!(
        out.is_empty(),
        "V0: missing config.json must NOT panic; got {:?}",
        out
    );
}

#[test]
fn it_v1_absent_field_or_null_yields_empty() {
    let tmp = TempDir::new("v1");
    write_config(tmp.path(), r#"{"server_url":"https://example"}"#);
    assert!(collect_cli_config_shared_dirs(tmp.path()).is_empty());

    write_config(
        tmp.path(),
        r#"{"worktree_shared_directories": null}"#,
    );
    assert!(collect_cli_config_shared_dirs(tmp.path()).is_empty());

    write_config(tmp.path(), r#"{"worktree_shared_directories": []}"#);
    assert!(collect_cli_config_shared_dirs(tmp.path()).is_empty());
}

#[test]
fn it_v2_mixed_paths_normalize_against_workspace_root() {
    let tmp = TempDir::new("v2");
    write_config(
        tmp.path(),
        r#"{"worktree_shared_directories": ["shared", "/opt/team/x", "../outside"]}"#,
    );
    let out = collect_cli_config_shared_dirs(tmp.path());
    assert_eq!(out.len(), 3, "got {:?}", out);
    assert!(out.contains(&tmp.path().join("shared")));
    let expected_x: std::path::PathBuf = if cfg!(windows) {
        std::path::PathBuf::from(r"\opt\team\x")
    } else {
        std::path::PathBuf::from("/opt/team/x")
    };
    assert!(out.contains(&expected_x));
    assert!(out.contains(
        &tmp.path()
            .parent()
            .expect("tempdir parent")
            .join("outside")
    ));
}

#[test]
fn it_v3_duplicates_collapse() {
    let tmp = TempDir::new("v3");
    write_config(
        tmp.path(),
        r#"{"worktree_shared_directories": ["a", "./a", "/x/y", "/x/./y"]}"#,
    );
    let out = collect_cli_config_shared_dirs(tmp.path());
    assert_eq!(out.len(), 2, "got {:?}", out);
}

#[test]
fn it_v4_non_string_entries_silently_skipped() {
    let tmp = TempDir::new("v4");
    write_config(
        tmp.path(),
        r#"{"worktree_shared_directories": ["ok", 42, null, true, {}, []]}"#,
    );
    let out = collect_cli_config_shared_dirs(tmp.path());
    assert_eq!(out, vec![tmp.path().join("ok")]);
}

#[test]
fn it_v5_strict_propagates_errors_while_lenient_swallows() {
    let tmp = TempDir::new("v5");
    write_config(tmp.path(), r#"{this is not json"#);
    // Lenient: empty
    assert!(collect_cli_config_shared_dirs(tmp.path()).is_empty());
    // Strict: error
    let strict = collect_cli_config_shared_dirs_strict(tmp.path());
    assert!(matches!(strict, Err(CliConfigError::MalformedJson(_))));

    write_config(
        tmp.path(),
        r#"{"worktree_shared_directories": "not-an-array"}"#,
    );
    assert!(collect_cli_config_shared_dirs(tmp.path()).is_empty());
    let strict = collect_cli_config_shared_dirs_strict(tmp.path());
    assert!(matches!(
        strict,
        Err(CliConfigError::FieldTypeMismatch { .. })
    ));
}

#[test]
fn it_v6_smoke_test_realistic_config_layout() {
    // Mimic a realistic per-workspace multica-config/config.json with
    // several types of entries mixed together.
    let tmp = TempDir::new("v6");
    let body = r#"{
        "server_url": "https://api.example.com",
        "device_name": "smoke-tester",
        "worktree_shared_directories": [
            "shared",
            "./shared",
            "/opt/team/shared",
            "/opt/team/./shared",
            ""
        ]
    }"#;
    write_config(tmp.path(), body);

    // Lenient: empty-string entry drops the whole read → empty
    let out = collect_cli_config_shared_dirs(tmp.path());
    assert!(
        out.is_empty(),
        "empty-string entry is rejected; whole read returns empty (got {:?})",
        out
    );

    // Remove the empty entry, retry; expect 2 unique entries
    let body2 = r#"{
        "worktree_shared_directories": ["shared", "/opt/team/shared"]
    }"#;
    write_config(tmp.path(), body2);
    let out = collect_cli_config_shared_dirs(tmp.path());
    assert_eq!(out.len(), 2, "dedup should leave 2; got {:?}", out);
}

#[test]
fn it_v7_public_constants_match_spec() {
    // These constants are part of the public API: orchestrator crates
    // (ULYS-158 shared_dir_resolver) will reference them. Lock the values.
    assert_eq!(CLI_CONFIG_FIELD, "worktree_shared_directories");
    assert_eq!(CLI_CONFIG_FILE_NAME, "config.json");
    assert_eq!(CLI_CONFIG_DIR_NAME, "multica-config");
    assert_eq!(SOURCE_CLI_CONFIG, "cli_config");
}

#[test]
fn it_v8_path_resolution_uses_workspace_root_not_cwd() {
    // The reader MUST resolve relative entries against workspace_root,
    // not the process CWD. We verify by handing a workspace_root that's
    // different from CWD and checking the entries land inside workspace_root.
    let tmp = TempDir::new("v8");
    write_config(
        tmp.path(),
        r#"{"worktree_shared_directories": ["relative-entry"]}"#,
    );

    // Change CWD to the temp dir's PARENT (not the temp dir itself).
    let parent = tmp.path().parent().expect("parent").to_path_buf();
    let prev_cwd = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(&parent).expect("set cwd");

    let out = collect_cli_config_shared_dirs(tmp.path());

    std::env::set_current_dir(&prev_cwd).expect("restore cwd");

    assert_eq!(out, vec![tmp.path().join("relative-entry")]);
}