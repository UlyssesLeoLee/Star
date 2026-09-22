//! `shared_dir_sources.rs` — Worktree Shared Directories source collectors (per ULYS-158 / ULYS-177).
//!
//! Per FR-ORCA-007 (Orca borrow §3, per `docs/ecosystem-survey/orca-design-survey.md`):
//! 3 路并存 (path C, per 2026-09-22 JST 路径 C 拍板):
//!
//! ```text
//! 优先级 (高 → 低):
//! 1. per-user        (~/.star/worktree_shared_dirs.txt)
//! 2. workspace-level (<workspace>/.star/worktree_shared_dirs.txt)
//! 3. CLI config      (<workspace>/multica-config/config.json.worktree_shared_directories)
//! ```
//!
//! ## ULYS-177 范围 (本模块的本次实装切片)
//!
//! ULYS-177 = ULYS-158.1 = path C (per-workspace multica-config/config.json 适配)
//! **只实装第 3 路** CLI config reader;per-user / workspace-level 两路留给 ULYS-158
//! worker 在后续 sprint 完成 (per ULYS-177 §2 范围改写 + §3.1 三路优先级描述).
//!
//! 三路合并 + 优先级排序 + 路径归一化的 orchestration 留给 ULYS-158 主 issue 的
//! `shared_dir_resolver` 模块 (per §3.3 4 个新模块表). 本 crate 仅暴露
//! `collect_cli_config_shared_dirs` 单一 source, 调用方 merge 三路结果.
//!
//! ## 设计原则
//!
//! - **零失败 panic**: 任何 IO / parse 错误必须降级为 `Vec::new()`, 不阻断 worktree 创建
//!   (per FR-ORCA-007 AC 语义: CLI config 是 fallback, 缺/坏 = 不参与)
//! - **路径归一化**: 输出前用 `Path::new(p).components()` 折叠 `.` / `..` 并 absolute 化
//!   (per FR-ORCA-006 并行 worktree 隔离保证: 重复 / 含 `..` 的路径必须去重规范化)
//! - **snake_case field name**: `worktree_shared_directories` (per 9/22 D-Boy 拍板)
//! - **JSON array of strings**: 任何非 string 元素被静默丢弃 (CLI free-form, 不做 schema validation)
//!
//! ## 守门
//!
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #13 c/d 不引入新 schema (本模块无持久化, 仅 read-only FS)
//! - #11 缺标比错标: 不引入新依赖, 用现有的 `serde_json`

use std::path::{Path, PathBuf};

/// Field name in the per-workspace multica CLI config file.
///
/// Per 9/22 D-Boy 拍板: snake_case flat key. CLI schema is flat key=value (no nested namespace).
pub const CLI_CONFIG_FIELD: &str = "worktree_shared_directories";

/// Default per-workspace multica CLI config file name (relative to workspace root).
pub const CLI_CONFIG_FILE_NAME: &str = "config.json";

/// Source identifier for the CLI config path (per FR-ORCA-007 三路优先级第 3 路).
///
/// Used by the eventual `shared_dir_resolver` (per ULYS-158 §3.3) to label entries
/// when surfacing diagnostic info to operators / logs.
pub const SOURCE_CLI_CONFIG: &str = "cli_config";

/// Per-workspace multica CLI config directory (relative to workspace root).
///
/// Per `multica config list` 实测: `<MULTICA_TASK_CONFIG_ROOT>/config.json`,
/// where `MULTICA_TASK_CONFIG_ROOT` = parent of the git worktree by convention.
pub const CLI_CONFIG_DIR_NAME: &str = "multica-config";

/// Error returned by the path-C reader.
///
/// Note: this error type is **not propagated to callers** — the public API returns
/// `Vec<PathBuf>` and silently degrades to empty on any failure. The error type
/// exists for tests and for an optional stricter mode that ULYS-158's
/// `shared_dir_resolver` may adopt later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliConfigError {
    /// The resolved config file path is invalid (e.g. parent dir missing and not creatable).
    InvalidPath(String),
    /// The config file exists but is not readable (IO error).
    Io(String),
    /// The config file content is not valid JSON.
    MalformedJson(String),
    /// The `worktree_shared_directories` field exists but is not a JSON array.
    FieldTypeMismatch {
        /// The JSON value kind observed (best-effort)
        observed: String,
    },
    /// A path entry inside the array could not be normalized.
    InvalidEntry(String),
}

impl std::fmt::Display for CliConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliConfigError::InvalidPath(s) => write!(f, "invalid config path: {}", s),
            CliConfigError::Io(s) => write!(f, "io error: {}", s),
            CliConfigError::MalformedJson(s) => write!(f, "malformed JSON: {}", s),
            CliConfigError::FieldTypeMismatch { observed } => {
                write!(f, "field type mismatch: expected array, got {}", observed)
            }
            CliConfigError::InvalidEntry(s) => write!(f, "invalid path entry: {}", s),
        }
    }
}

impl std::error::Error for CliConfigError {}

/// Locate the per-workspace multica CLI config file.
///
/// ## Resolution rule
///
/// 1. If `<workspace_root>/multica-config/config.json` exists → return that path.
/// 2. Otherwise return `<workspace_root>/multica-config/config.json` even if missing
///    (callers can decide whether to error or fall through).
///
/// ## Why not read `MULTICA_TASK_CONFIG_ROOT` env var?
///
/// That env var is the runtime's contract with the multica CLI daemon; it identifies
/// the *task slot's* config dir, which may differ from the **repo workspace root**
/// (the root that worktree-service runs in). For shared-directory resolution we
/// want the workspace the worktrees belong to, so we walk up to the workspace root
/// supplied by the caller. This makes the reader pure and testable.
pub fn resolve_cli_config_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(CLI_CONFIG_DIR_NAME).join(CLI_CONFIG_FILE_NAME)
}

/// Read the `worktree_shared_directories` array from a per-workspace multica CLI
/// config file, normalize each entry to an absolute `PathBuf`, and return a
/// deduplicated `Vec<PathBuf>`.
///
/// ## Failure mode
///
/// On any error (missing file, malformed JSON, missing field, wrong type, invalid
/// path entry), this function returns `Ok(Vec::new())` and logs a `tracing::warn!`.
/// Returning an empty vector matches the spec semantics: the CLI config is the
/// lowest-priority fallback, so its absence must never block worktree creation.
///
/// Callers that need stricter failure semantics (e.g. a one-shot CLI validation
/// tool) can use [`collect_cli_config_shared_dirs_strict`] instead.
pub fn collect_cli_config_shared_dirs(workspace_root: &Path) -> Vec<PathBuf> {
    match collect_cli_config_shared_dirs_inner(workspace_root) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(
                target: "worktree_service::shared_dir_sources",
                workspace = %workspace_root.display(),
                error = %e,
                "path C (CLI config) shared-dir read failed; falling back to empty (per FR-ORCA-007 fallback semantics)"
            );
            Vec::new()
        }
    }
}

/// Strict variant of [`collect_cli_config_shared_dirs`]: propagates the first error.
///
/// Intended for the optional pre-flight validator (per ULYS-158 §6 实装收尾验证
/// `multica config set worktree_shared_directories '[...]' + get`) and for tests.
pub fn collect_cli_config_shared_dirs_strict(
    workspace_root: &Path,
) -> Result<Vec<PathBuf>, CliConfigError> {
    collect_cli_config_shared_dirs_inner(workspace_root)
}

/// Inner implementation, shared by both strict and lenient entry points.
fn collect_cli_config_shared_dirs_inner(
    workspace_root: &Path,
) -> Result<Vec<PathBuf>, CliConfigError> {
    let config_path = resolve_cli_config_path(workspace_root);
    // Missing file is NOT an error here — it's the "field not set" path.
    let raw = match std::fs::read_to_string(&config_path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(CliConfigError::Io(format!("{}: {}", config_path.display(), e))),
    };
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| CliConfigError::MalformedJson(format!("{}: {}", config_path.display(), e)))?;
    let field = parsed.get(CLI_CONFIG_FIELD);
    let arr = match field {
        None => return Ok(Vec::new()),
        Some(serde_json::Value::Null) => return Ok(Vec::new()),
        Some(serde_json::Value::Array(a)) => a,
        Some(other) => {
            return Err(CliConfigError::FieldTypeMismatch {
                observed: type_name(other),
            })
        }
    };
    let mut out: Vec<PathBuf> = Vec::with_capacity(arr.len());
    for (idx, item) in arr.iter().enumerate() {
        match item {
            serde_json::Value::String(s) => match normalize_path(s, workspace_root) {
                Ok(p) => out.push(p),
                Err(e) => {
                    return Err(CliConfigError::InvalidEntry(format!(
                        "index {}: {} ({})",
                        idx, e, s
                    )))
                }
            },
            // Non-string entries are silently skipped: CLI is free-form, no schema validation.
            // Spec semantics: "重复目录去重" — non-string is not a directory, so skip.
            _ => continue,
        }
    }
    // Dedup while preserving order (first occurrence wins)
    let mut seen: Vec<PathBuf> = Vec::new();
    out.retain(|p| {
        if seen.iter().any(|q| q == p) {
            false
        } else {
            seen.push(p.clone());
            true
        }
    });
    Ok(out)
}

/// Normalize a single shared-dir entry to an absolute, canonicalized `PathBuf`.
///
/// - Empty string → error.
/// - Relative path → resolved against `workspace_root`.
/// - Absolute path (POSIX `/...` or Windows `C:\...` / `\\server\share`) → returned
///   as-is after component normalization. Note: on Windows, `Path::is_absolute()`
///   returns `false` for POSIX-style paths like `/opt/team/x`, so we also accept
///   paths whose first component is `RootDir`.
/// - Path containing `..` or `.` is collapsed via `Path::components` (path-only
///   normalization; we intentionally do NOT touch the filesystem via
///   `canonicalize()` so this works for paths that don't yet exist).
fn normalize_path(raw: &str, workspace_root: &Path) -> Result<PathBuf, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("empty path entry".to_string());
    }
    let p = Path::new(trimmed);
    // Treat as absolute if either:
    // 1. Path::is_absolute() (covers Windows C:\..., \\server\share, / on POSIX)
    // 2. First component is RootDir — catches POSIX-looking paths on Windows
    //    (`/opt/team/x`) which Rust considers relative but which clearly are.
    let looks_absolute = p.is_absolute()
        || matches!(
            p.components().next(),
            Some(std::path::Component::RootDir)
        );
    let joined: PathBuf = if looks_absolute {
        p.to_path_buf()
    } else {
        workspace_root.join(p)
    };
    // Collapse `.` / `..` purely lexically (no FS touch)
    let mut normalized = PathBuf::new();
    for comp in joined.components() {
        match comp {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::CurDir => {
                // skip
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err("path resolves to empty".to_string());
    }
    Ok(normalized)
}

/// Best-effort JSON value type tag for error messages (we don't depend on serde_json's internals).
fn type_name(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(_) => "bool".to_string(),
        serde_json::Value::Number(_) => "number".to_string(),
        serde_json::Value::String(_) => "string".to_string(),
        serde_json::Value::Array(_) => "array".to_string(),
        serde_json::Value::Object(_) => "object".to_string(),
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile_lite::TempDir;

    /// Tiny RAII tempdir helper to avoid pulling `tempfile` as a new dev-dep.
    /// Falls back to `std::env::temp_dir()` + cleanup-on-drop.
    ///
    /// Visibility is `pub(super)` to satisfy the workspace's `unreachable_pub`
    /// lint without exporting the helper type from the crate.
    mod tempfile_lite {
        use std::path::PathBuf;

        pub(super) struct TempDir(pub(super) PathBuf);
        impl TempDir {
            pub(super) fn new(prefix: &str) -> Self {
                let nanos = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0);
                let p = std::env::temp_dir().join(format!("wtsd-{}-{}", prefix, nanos));
                std::fs::create_dir_all(&p).expect("create tempdir");
                Self(p)
            }
            pub(super) fn path(&self) -> &std::path::Path {
                &self.0
            }
        }
        impl Drop for TempDir {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
    }

    fn write_config(dir: &Path, body: &str) {
        let cfg_dir = dir.join(CLI_CONFIG_DIR_NAME);
        fs::create_dir_all(&cfg_dir).expect("mkdir multica-config");
        fs::write(cfg_dir.join(CLI_CONFIG_FILE_NAME), body).expect("write config.json");
    }

    #[test]
    fn resolve_cli_config_path_appends_multica_config_and_config_json() {
        let p = resolve_cli_config_path(Path::new("/tmp/repo"));
        assert_eq!(p, PathBuf::from("/tmp/repo/multica-config/config.json"));
    }

    #[test]
    fn missing_file_returns_empty() {
        let tmp = TempDir::new("missing");
        let out = collect_cli_config_shared_dirs(tmp.path());
        assert!(out.is_empty(), "missing config.json must NOT panic; got {:?}", out);
    }

    #[test]
    fn missing_field_returns_empty() {
        let tmp = TempDir::new("nofield");
        write_config(tmp.path(), r#"{"server_url": "https://example"}"#);
        let out = collect_cli_config_shared_dirs(tmp.path());
        assert!(out.is_empty(), "absent worktree_shared_directories field → empty");
    }

    #[test]
    fn null_field_returns_empty() {
        let tmp = TempDir::new("nullfield");
        write_config(tmp.path(), r#"{"worktree_shared_directories": null}"#);
        let out = collect_cli_config_shared_dirs(tmp.path());
        assert!(out.is_empty(), "explicit null → empty");
    }

    #[test]
    fn empty_array_returns_empty() {
        let tmp = TempDir::new("emptyarr");
        write_config(tmp.path(), r#"{"worktree_shared_directories": []}"#);
        let out = collect_cli_config_shared_dirs(tmp.path());
        assert!(out.is_empty());
    }

    #[test]
    fn absolute_paths_passed_through_normalized() {
        let tmp = TempDir::new("abspath");
        write_config(
            tmp.path(),
            r#"{"worktree_shared_directories": ["/opt/team/shared", "/var/cache/."]}"#,
        );
        let out = collect_cli_config_shared_dirs(tmp.path());
        // `.` collapsed; the POSIX-looking paths survive as their cross-platform
        // PathBuf representation (Windows: `\opt\team\shared`, POSIX: `/opt/team/shared`).
        assert_eq!(out.len(), 2);
        let expected_a: PathBuf = if cfg!(windows) {
            PathBuf::from(r"\opt\team\shared")
        } else {
            PathBuf::from("/opt/team/shared")
        };
        let expected_b: PathBuf = if cfg!(windows) {
            PathBuf::from(r"\var\cache")
        } else {
            PathBuf::from("/var/cache")
        };
        assert!(out.contains(&expected_a), "missing {:?} in {:?}", expected_a, out);
        assert!(out.contains(&expected_b), "missing {:?} in {:?}", expected_b, out);
    }

    #[test]
    fn relative_paths_resolved_against_workspace_root() {
        let tmp = TempDir::new("relpath");
        write_config(
            tmp.path(),
            r#"{"worktree_shared_directories": ["shared", "../sibling-shared"]}"#,
        );
        let out = collect_cli_config_shared_dirs(tmp.path());
        assert_eq!(out.len(), 2, "got {:?}", out);
        assert!(out.contains(&tmp.path().join("shared")));
        // "../sibling-shared" is collapsed lexically: workspace_root + ".."
        // resolves to workspace_root.parent(), so the entry lands at
        // workspace_root.parent()/sibling-shared.
        let expected_sibling = tmp.path().parent().expect("tempdir parent").join("sibling-shared");
        assert!(
            out.contains(&expected_sibling),
            "missing {:?} in {:?}",
            expected_sibling,
            out
        );
    }

    #[test]
    fn duplicate_paths_deduplicated() {
        let tmp = TempDir::new("dedup");
        write_config(
            tmp.path(),
            r#"{"worktree_shared_directories": ["shared", "./shared", "/opt/team/x", "/opt/team/./x"]}"#,
        );
        let out = collect_cli_config_shared_dirs(tmp.path());
        assert_eq!(out.len(), 2, "duplicates must collapse; got {:?}", out);
        assert!(out.contains(&tmp.path().join("shared")));
        let expected: PathBuf = if cfg!(windows) {
            PathBuf::from(r"\opt\team\x")
        } else {
            PathBuf::from("/opt/team/x")
        };
        assert!(out.contains(&expected), "missing {:?} in {:?}", expected, out);
    }

    #[test]
    fn non_string_entries_silently_skipped() {
        let tmp = TempDir::new("nonstr");
        write_config(
            tmp.path(),
            r#"{"worktree_shared_directories": ["valid", 42, null, true, {"nested": "obj"}, ["arr"]]}"#,
        );
        let out = collect_cli_config_shared_dirs(tmp.path());
        assert_eq!(out.len(), 1, "only the string entry survives; got {:?}", out);
        assert_eq!(out[0], tmp.path().join("valid"));
    }

    #[test]
    fn empty_string_entry_is_rejected_by_strict() {
        let tmp = TempDir::new("emptystr");
        write_config(
            tmp.path(),
            r#"{"worktree_shared_directories": ["valid", "", "  "]}"#,
        );
        // Lenient: empty / whitespace-only are invalid → strict returns Err → lenient returns empty
        let out = collect_cli_config_shared_dirs(tmp.path());
        assert!(out.is_empty(), "empty entries must drop the whole read; got {:?}", out);
        // Strict: confirm the error is surfaced
        let strict = collect_cli_config_shared_dirs_strict(tmp.path());
        assert!(matches!(strict, Err(CliConfigError::InvalidEntry(_))));
    }

    #[test]
    fn malformed_json_returns_empty_lenient() {
        let tmp = TempDir::new("badjson");
        write_config(tmp.path(), "{this is not valid json");
        let out = collect_cli_config_shared_dirs(tmp.path());
        assert!(out.is_empty(), "malformed JSON must not panic");
        let strict = collect_cli_config_shared_dirs_strict(tmp.path());
        assert!(matches!(strict, Err(CliConfigError::MalformedJson(_))));
    }

    #[test]
    fn wrong_field_type_returns_empty_lenient() {
        let tmp = TempDir::new("wrongtype");
        write_config(tmp.path(), r#"{"worktree_shared_directories": "not-an-array"}"#);
        let out = collect_cli_config_shared_dirs(tmp.path());
        assert!(out.is_empty(), "field type mismatch must not panic");
        let strict = collect_cli_config_shared_dirs_strict(tmp.path());
        assert!(matches!(strict, Err(CliConfigError::FieldTypeMismatch { .. })));
    }

    #[test]
    fn dot_dot_components_collapsed_lexically() {
        let tmp = TempDir::new("dotdot");
        write_config(
            tmp.path(),
            r#"{"worktree_shared_directories": ["/opt/a/./b/../c"]}"#,
        );
        let out = collect_cli_config_shared_dirs(tmp.path());
        let expected: PathBuf = if cfg!(windows) {
            PathBuf::from(r"\opt\a\c")
        } else {
            PathBuf::from("/opt/a/c")
        };
        assert_eq!(out, vec![expected]);
    }

    #[test]
    fn source_label_constant_stable() {
        // Lock the public label so ULYS-158's orchestrator can rely on it.
        assert_eq!(SOURCE_CLI_CONFIG, "cli_config");
        assert_eq!(CLI_CONFIG_FIELD, "worktree_shared_directories");
        assert_eq!(CLI_CONFIG_FILE_NAME, "config.json");
        assert_eq!(CLI_CONFIG_DIR_NAME, "multica-config");
    }
}