//! `shared_dir_resolver.rs` — `SharedDirResolver` Trait + 3 Source + 真实 file-backed ConfigSource
//!
//! Per ULYS-104.3 (ULYS-158 + ULYS-177) P0-D + FR-ORCA-007 AC-4:
//! "per-user + workspace-level + multica-config/config.json 并集；symlinked 的不重复 copy"
//!
//! 优先级合并规则 (per FR-ORCA-007 AC-4 + 9/22 派生):
//! 1. workspace-level (P1/P2) 优先
//! 2. per-user (P3) 兜底
//! 3. multica-config/config.json (P3, 跟 per-user 并级, FileBackedConfigSource 直读)
//! 4. 同 path 不同 mount_strategy: workspace-level 覆盖 per-user
//! 5. 同 source 内 priority 排序: P1 > P2 > P3
//!
//! ## 路径 C 自决 (per D-Boy 11:36 JST 2026-09-22 comment `01a0c8e7-5df4-7b4c-af8d-528351eb4a22`)
//!
//! ConfigSource **不依赖 Multica CLI v1.0 schema**, 直读
//! `<workspace>/multica-config/config.json` 文件. 即使 CLI v0.4.42 的 16-key whitelist
//! 不含 `worktree_shared_directories`, 运维手写 config.json 即可启用第 3 机制.
//!
//! 实现细节: 本 crate 提供 FileBackedConfigSource (真实文件读) + NoopConfigSource + InMemorySharedDirConfigSource.
//!
//! 本仓库适配原则 (per D-Boy 9/22 "适合我 rust 体系的版本不算重复造轮子"):
//! - 不引新依赖, 用现有 `serde_json` (workspace dep)
//! - 不重写 3 路合并, 复用 ULYS-158 v1.0 MVP 的 `InMemorySharedDirResolver::merge`

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::error::{SharedDirError, SharedDirResult};
use crate::shared_dir_types::{
    SharedDirConfig, SharedDirPriority, SharedDirSource, SharedDirectory, SharedMountStrategy,
};

/// 默认 trace id (per 守门 #6 v2)
const DEFAULT_TRACE_ID: &str = "wsd-default-trace";

// =====================================================================
// CLI config 路径 / 字段常量 (per 9/22 D-Boy 路径 C 拍板 + 11:36 自决)
// =====================================================================

/// Source identifier for the multica-config/config.json path (per FR-ORCA-007 三路优先级第 3 路).
pub const SOURCE_CLI_CONFIG: &str = "cli_config";

/// Field name in the per-workspace multica config file.
pub const CLI_CONFIG_FIELD: &str = "worktree_shared_directories";

/// Per-workspace multica config file name (relative to `<workspace>/multica-config/`).
pub const CLI_CONFIG_FILE_NAME: &str = "config.json";

/// Per-workspace multica config directory (relative to workspace root).
pub const CLI_CONFIG_DIR_NAME: &str = "multica-config";

/// Resolve the per-workspace multica config file path from a workspace root.
///
/// Resolution: `<workspace_root>/<CLI_CONFIG_DIR_NAME>/<CLI_CONFIG_FILE_NAME>`.
/// Returned unconditionally (even if file missing) so callers can decide whether
/// to error or fall through.
///
/// ## Why not read `MULTICA_TASK_CONFIG_ROOT` env var?
///
/// That env var is the runtime's contract with the multica CLI daemon; it identifies
/// the *task slot's* config dir, which may differ from the **repo workspace root**
/// (the root that worktree-shared-dir runs in). For shared-directory resolution we
/// want the workspace the worktrees belong to, so we walk up to the workspace root
/// supplied by the caller. This makes the reader pure and testable.
pub fn resolve_cli_config_path(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join(CLI_CONFIG_DIR_NAME)
        .join(CLI_CONFIG_FILE_NAME)
}

/// Resolver 解析结果 (按 priority 升序排列, P1 在前)
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedSharedDirs {
    /// 已合并去重的 SharedDirectory 列表 (按 priority 升序)
    pub entries: Vec<SharedDirectory>,
    /// 触发的 source 列表 (用于 audit / debug)
    pub sources_hit: Vec<SharedDirSource>,
}

/// Per-user Source (机制 1, FR-ORCA-007 #1)
#[async_trait]
pub trait PerUserSource: Send + Sync {
    /// 列出 per-user 配置的所有 SharedDirectory
    async fn list(&self) -> SharedDirResult<Vec<SharedDirectory>>;

    /// 列出 per-user 文件路径源 (用于 audit / dry-run).
    ///
    /// Per ULYS-158.2 §2.3: 新增方法暴露给 audit / 故障排查场景.
    /// - `FileBackedPerUserSource` 返回其 `~/.star/worktree_shared_dirs.txt` 路径.
    /// - `NoopPerUserSource` / `InMemorySharedDirPerUserSource` 返回 `<empty>` (无外部源).
    async fn source_path(&self) -> std::path::PathBuf;
}

/// Workspace-level Source (机制 2, FR-ORCA-007 #2) — MVP 阶段 Noop 占位, PG impl 留 P1 followup
#[async_trait]
pub trait WorkspaceSource: Send + Sync {
    /// 列出 workspace-level 配置的所有 SharedDirectory (从 `worktree_shared_dir` PG 表读)
    async fn list(&self) -> SharedDirResult<Vec<SharedDirectory>>;
}

/// `.multica/config.yaml` Source (机制 3, FR-ORCA-007 #3) — FileBacked 本仓库实装
#[async_trait]
pub trait ConfigSource: Send + Sync {
    /// 读 `SharedDirConfig` (FileBacked 直读 config.json; Noop 返回 default empty)
    async fn load(&self) -> SharedDirResult<SharedDirConfig>;
}

/// `SharedDirResolver` Trait (per FR-ORCA-007 AC-4)
#[async_trait]
pub trait SharedDirResolver: Send + Sync {
    /// 给一个 repo + worktree, 解析出全部共享目录 (按 priority 升序)
    async fn resolve(&self, repo_id: uuid::Uuid) -> SharedDirResult<ResolvedSharedDirs>;
}

// =====================================================================
// Noop impls (MVP, 后续 P1 followup 切真实 PG / IO)
// =====================================================================

/// Noop per-user (MVP, 后续从 `~/.star/shared` 读)
#[derive(Debug, Default, Clone)]
pub struct NoopPerUserSource;

#[async_trait]
impl PerUserSource for NoopPerUserSource {
    async fn list(&self) -> SharedDirResult<Vec<SharedDirectory>> {
        Ok(Vec::new())
    }

    async fn source_path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from("<noop>")
    }
}

/// Noop workspace (MVP, 后续从 PG `worktree_shared_dir` 读)
#[derive(Debug, Default, Clone)]
pub struct NoopWorkspaceSource;

#[async_trait]
impl WorkspaceSource for NoopWorkspaceSource {
    async fn list(&self) -> SharedDirResult<Vec<SharedDirectory>> {
        Ok(Vec::new())
    }
}

/// Noop config (默认空, 给不需要 config 注入的 caller 用)
#[derive(Debug, Default, Clone)]
pub struct NoopConfigSource;

#[async_trait]
impl ConfigSource for NoopConfigSource {
    async fn load(&self) -> SharedDirResult<SharedDirConfig> {
        Ok(SharedDirConfig::default())
    }
}

// =====================================================================
// FileBackedConfigSource (本仓库实装, 不依赖 Multica CLI)
// =====================================================================

/// 真实文件读 `ConfigSource` — 直读 `<workspace>/multica-config/config.json`
/// 的 `worktree_shared_directories` 字段.
///
/// Per D-Boy 11:36 JST 2026-09-22 「完成后续工作，我不需要Multica公司配合」:
/// **不依赖 Multica CLI 升版**. 即便 CLI v0.4.42 的 16-key whitelist 不含
/// `worktree_shared_directories`, 运维仍可手写 `<workspace>/multica-config/config.json`
/// 启用本机制. reader 完全自包含在 Rust 仓库内, 零外部依赖.
///
/// ## 行为约定 (per FR-ORCA-007 fallback 语义)
///
/// - 文件不存在 → 返回 `Ok(empty SharedDirConfig)`, **不报错**
/// - 文件存在但字段缺失/为 `null` → 返回 `Ok(empty SharedDirConfig)`
/// - 字段类型错 (期望 array, 实际 string/object/number) → 返回 `Err(WSD.CONFIG_TYPE_FAIL)`
/// - JSON 解析失败 → 返回 `Err(WSD.CONFIG_PARSE_FAIL)`
/// - 其它 IO 错误 (permission denied 等) → 返回 `Err(WSD.CONFIG_IO_FAIL)`
///
/// Lenient 入口 `load_lenient()` 把上述错误降级为 `Ok(empty)`, 适合生产路径上
/// 不希望被坏 config 阻断 worktree 创建的场景.
#[derive(Debug, Clone)]
pub struct FileBackedConfigSource {
    workspace_root: PathBuf,
}

impl FileBackedConfigSource {
    /// 构造: 给定 workspace root.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
        }
    }

    /// workspace root 引用.
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// 严格读: 任何 IO/parse/type 错误都向上抛 `SharedDirError`.
    pub async fn load_strict(&self) -> SharedDirResult<SharedDirConfig> {
        Self::load_strict_sync(&self.workspace_root)
    }

    /// 同步严格读 (内部使用 + 测试).
    fn load_strict_sync(workspace_root: &Path) -> SharedDirResult<SharedDirConfig> {
        let config_path = resolve_cli_config_path(workspace_root);
        // 缺失文件按 "字段未设置" 处理 (非错误, 与 9/22 spec 一致)
        let raw = match std::fs::read_to_string(&config_path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(SharedDirConfig::default());
            }
            Err(e) => {
                return Err(SharedDirError::new(
                    "WSD.CONFIG_IO_FAIL",
                    format!("failed to read {}: {}", config_path.display(), e),
                    DEFAULT_TRACE_ID,
                )
                .with_source(format!("io: {e}")));
            }
        };
        let parsed: serde_json::Value = serde_json::from_str(&raw).map_err(|e| {
            SharedDirError::new(
                "WSD.CONFIG_PARSE_FAIL",
                format!("failed to parse {}: {}", config_path.display(), e),
                DEFAULT_TRACE_ID,
            )
            .with_source(format!("json: {e}"))
        })?;
        let field = parsed.get(CLI_CONFIG_FIELD);
        let arr = match field {
            None => return Ok(SharedDirConfig::default()),
            Some(serde_json::Value::Null) => return Ok(SharedDirConfig::default()),
            Some(serde_json::Value::Array(a)) => a,
            Some(other) => {
                return Err(SharedDirError::new(
                    "WSD.CONFIG_TYPE_FAIL",
                    format!(
                        "{} field must be a JSON array of strings, got {}",
                        CLI_CONFIG_FIELD,
                        type_name(other)
                    ),
                    DEFAULT_TRACE_ID,
                ));
            }
        };
        let mut shared_directories: Vec<String> = Vec::with_capacity(arr.len());
        for (idx, item) in arr.iter().enumerate() {
            match item {
                serde_json::Value::String(s) => {
                    let trimmed = s.trim();
                    if trimmed.is_empty() {
                        return Err(SharedDirError::new(
                            "WSD.CONFIG_PARSE_FAIL",
                            format!("index {}: empty path entry", idx),
                            DEFAULT_TRACE_ID,
                        ));
                    }
                    // 立即归一化 (折叠 `.` / `..` + 保留绝对路径), 让后续 dedup 命中
                    // (per FR-ORCA-006 并行 worktree 隔离保证: 重复 / 含 `..` 的路径必须去重规范化).
                    // 相对路径在这里直接拒: FileBackedConfigSource 不持有 workspace_root 的解析能力,
                    // 路径归一化的 caller-side 解析留给 `merge()` + `normalize_path()` 配合 caller 的 workspace_root.
                    let normalized_str = match normalize_path(trimmed) {
                        Ok(p) => p.to_string_lossy().to_string(),
                        Err(_) => {
                            // 相对路径在 file reader 阶段保留原 trim, 让 merge() 时报错 (per WSD.PATH_NOT_ABSOLUTE)
                            // 不在 reader 阶段硬拒, 给 caller 一个机会用 workspace_root 解析.
                            trimmed.to_string()
                        }
                    };
                    shared_directories.push(normalized_str);
                }
                // 非 string 元素静默丢弃 (spec §3.2 "JSON array of strings").
                _ => continue,
            }
        }
        // 去重保序 (first occurrence wins)
        let mut seen: Vec<String> = Vec::new();
        shared_directories.retain(|p| {
            if seen.iter().any(|q| q == p) {
                false
            } else {
                seen.push(p.clone());
                true
            }
        });
        Ok(SharedDirConfig { shared_directories })
    }
}

#[async_trait]
impl ConfigSource for FileBackedConfigSource {
    async fn load(&self) -> SharedDirResult<SharedDirConfig> {
        // lenient 模式: 任何错误降级为 Ok(empty), 仅 warn 日志
        match self.load_strict().await {
            Ok(cfg) => Ok(cfg),
            Err(e) => {
                warn!(
                    target: "worktree_shared_dir::FileBackedConfigSource",
                    workspace = %self.workspace_root.display(),
                    error = %e,
                    "FileBackedConfigSource read failed; falling back to empty (per FR-ORCA-007 fallback semantics)"
                );
                Ok(SharedDirConfig::default())
            }
        }
    }
}

// =====================================================================
// FileBackedPerUserSource (本仓库实装, ULYS-158.2 P0)
// =====================================================================

/// 默认 per-user 配置文件名 (per ULYS-158.2 §2.1, 路径 C 派生自
/// `docs/ecosystem-survey/orca-design-survey.md` v1.0 §3 FR-ORCA-007).
///
/// 完整路径默认是 `<home>/.star/worktree_shared_dirs.txt`, 通过
/// [`default_per_user_config_path`] 拼装. 测试时可注入临时目录.
pub const PERUSER_CONFIG_FILE_NAME: &str = "worktree_shared_dirs.txt";

/// 默认 per-user 配置所在目录名 (`~/.star`).
pub const PERUSER_CONFIG_DIR_NAME: &str = ".star";

/// 拼装默认 per-user 配置文件路径 (`<home>/.star/worktree_shared_dirs.txt`).
///
/// - POSIX: `$HOME/.star/worktree_shared_dirs.txt`
/// - Windows: `%USERPROFILE%\.star\worktree_shared_dirs.txt`
///
/// 返回的路径即使 `<home>` 不存在或 `.star/` 目录不存在也合法 (per FR-ORCA-007
/// fallback: 文件缺失 = `Ok(Vec::new())`).
pub fn default_per_user_config_path() -> std::path::PathBuf {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    home.join(PERUSER_CONFIG_DIR_NAME)
        .join(PERUSER_CONFIG_FILE_NAME)
}

/// 真实文件读 per-user Source — 直读 `<config_path>` (默认 `~/.star/worktree_shared_dirs.txt`).
///
/// Per ULYS-158.2 §2 (FR-ORCA-007 机制 1):
/// - **格式**: 一行一个绝对路径; 空行 / `#` 开头行 = 注释; 空格 trim.
/// - **缺失文件 / 字段缺失** → `Ok(Vec::new())` (lenient, per FR-ORCA-007 fallback).
/// - **解析失败** (非注释行空 / `..` 后路径无法解析 / 权限拒绝) →
///   `Err(WSD.PERUSER_IO_FAIL | WSD.PERUSER_PARSE_FAIL)`. `list()` lenient 入口
///   把错误降级为 `Ok(Vec::new())`, 仅 `warn!` 日志.
///
/// ## 为什么不是 JSON?
///
/// Orca AC-1 用 `git config --file-list` 风格的纯路径文件 (`worktree.sharedDirectories`
/// 已在机制 2 占位). 本仓库派生 spec 走纯文本, 避免和机制 3 (`config.json`)
/// 重复, 也方便运维 `cat >> ~/.star/worktree_shared_dirs.txt` 追加.
///
/// ## 严格模式
///
/// `load_strict()` 把任何错误向上抛, 适合 audit / dry-run 场景.
/// `list()` 默认走 lenient (per FR-ORCA-007 fallback).
#[derive(Debug, Clone)]
pub struct FileBackedPerUserSource {
    config_path: std::path::PathBuf,
}

impl FileBackedPerUserSource {
    /// 默认构造: 配置文件路径为 `~/.star/worktree_shared_dirs.txt`.
    pub fn new() -> Self {
        Self::with_path(default_per_user_config_path())
    }

    /// 给定配置文件路径 (测试 / 注入用).
    pub fn with_path(config_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            config_path: config_path.into(),
        }
    }

    /// 配置文件路径引用.
    pub fn config_path(&self) -> &std::path::Path {
        &self.config_path
    }

    /// 严格读: 任何 IO/parse 错误都向上抛 `SharedDirError`.
    pub async fn load_strict(&self) -> SharedDirResult<Vec<SharedDirectory>> {
        Self::load_strict_sync(&self.config_path)
    }

    /// 同步严格读 (内部使用 + 测试).
    fn load_strict_sync(config_path: &Path) -> SharedDirResult<Vec<SharedDirectory>> {
        // 缺失文件 → 空 (per FR-ORCA-007 fallback, 不是错误)
        let raw = match std::fs::read_to_string(config_path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Vec::new());
            }
            Err(e) => {
                return Err(SharedDirError::new(
                    "WSD.PERUSER_IO_FAIL",
                    format!("failed to read {}: {}", config_path.display(), e),
                    DEFAULT_TRACE_ID,
                )
                .with_source(format!("io: {e}")));
            }
        };

        // 行解析: skip 注释 / 空行, 每行 trim, 相对路径拒
        let mut entries: Vec<SharedDirectory> = Vec::new();
        for (idx, raw_line) in raw.lines().enumerate() {
            let line = raw_line.trim();
            // 空行 / `#` 注释 / `//` 注释 → skip
            if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                continue;
            }
            // 路径归一化 (复用 crate-internal normalize_path, 拒绝相对路径)
            let normalized = match normalize_path(line) {
                Ok(p) => p,
                Err(e) => {
                    return Err(SharedDirError::new(
                        "WSD.PERUSER_PARSE_FAIL",
                        format!(
                            "line {} of {}: invalid path {:?}: {}",
                            idx + 1,
                            config_path.display(),
                            line,
                            e
                        ),
                        DEFAULT_TRACE_ID,
                    )
                    .with_source(format!("parse: {e}")));
                }
            };
            entries.push(SharedDirectory {
                source: SharedDirSource::PerUser,
                path: normalized,
                mount_strategy: SharedMountStrategy::WorktreeAdd,
                label: derive_label(line),
                priority: SharedDirPriority::P3,
                enabled: true,
            });
        }

        // 去重保序 (first occurrence wins)
        let mut seen: Vec<std::path::PathBuf> = Vec::new();
        entries.retain(|sd| {
            if seen.iter().any(|q| q == &sd.path) {
                false
            } else {
                seen.push(sd.path.clone());
                true
            }
        });

        Ok(entries)
    }
}

impl Default for FileBackedPerUserSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PerUserSource for FileBackedPerUserSource {
    async fn list(&self) -> SharedDirResult<Vec<SharedDirectory>> {
        // lenient 模式: 任何错误降级为 Ok(empty), 仅 warn 日志
        match self.load_strict().await {
            Ok(v) => Ok(v),
            Err(e) => {
                warn!(
                    target: "worktree_shared_dir::FileBackedPerUserSource",
                    config_path = %self.config_path.display(),
                    error = %e,
                    "FileBackedPerUserSource read failed; falling back to empty (per FR-ORCA-007 fallback semantics)"
                );
                Ok(Vec::new())
            }
        }
    }

    async fn source_path(&self) -> std::path::PathBuf {
        self.config_path.clone()
    }
}

// =====================================================================
// InMemory impls (测试 / 本地开发用)
// =====================================================================

/// In-memory per-user source
#[derive(Debug, Default, Clone)]
pub struct InMemorySharedDirPerUserSource {
    entries: Vec<SharedDirectory>,
}

impl InMemorySharedDirPerUserSource {
    /// 构造
    pub fn new(entries: Vec<SharedDirectory>) -> Self {
        Self { entries }
    }
}

#[async_trait]
impl PerUserSource for InMemorySharedDirPerUserSource {
    async fn list(&self) -> SharedDirResult<Vec<SharedDirectory>> {
        Ok(self.entries.clone())
    }

    async fn source_path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from("<in-memory>")
    }
}

/// In-memory workspace source
#[derive(Debug, Default, Clone)]
pub struct InMemorySharedDirWorkspaceSource {
    entries: Vec<SharedDirectory>,
}

impl InMemorySharedDirWorkspaceSource {
    /// 构造
    pub fn new(entries: Vec<SharedDirectory>) -> Self {
        Self { entries }
    }
}

#[async_trait]
impl WorkspaceSource for InMemorySharedDirWorkspaceSource {
    async fn list(&self) -> SharedDirResult<Vec<SharedDirectory>> {
        Ok(self.entries.clone())
    }
}

/// In-memory config source
#[derive(Debug, Default, Clone)]
pub struct InMemorySharedDirConfigSource {
    config: SharedDirConfig,
}

impl InMemorySharedDirConfigSource {
    /// 构造
    pub fn new(config: SharedDirConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ConfigSource for InMemorySharedDirConfigSource {
    async fn load(&self) -> SharedDirResult<SharedDirConfig> {
        Ok(self.config.clone())
    }
}

// =====================================================================
// InMemorySharedDirResolver (聚合 3 source)
// =====================================================================

/// In-memory `SharedDirResolver` (聚合 per-user + workspace + config)
pub struct InMemorySharedDirResolver {
    per_user: Arc<dyn PerUserSource>,
    workspace: Arc<dyn WorkspaceSource>,
    config: Arc<dyn ConfigSource>,
}

impl std::fmt::Debug for InMemorySharedDirResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemorySharedDirResolver")
            .finish_non_exhaustive()
    }
}

impl InMemorySharedDirResolver {
    /// 构造 (3 source 都可独立注入, 默认 Noop)
    pub fn new(
        per_user: Arc<dyn PerUserSource>,
        workspace: Arc<dyn WorkspaceSource>,
        config: Arc<dyn ConfigSource>,
    ) -> Self {
        Self {
            per_user,
            workspace,
            config,
        }
    }

    /// 默认构造 (3 Noop source)
    pub fn default_noop() -> Self {
        Self::new(
            Arc::new(NoopPerUserSource),
            Arc::new(NoopWorkspaceSource),
            Arc::new(NoopConfigSource),
        )
    }

    /// 合并 3 source 结果 (per FR-ORCA-007 AC-4)
    fn merge(
        per_user: Vec<SharedDirectory>,
        workspace: Vec<SharedDirectory>,
        config: SharedDirConfig,
    ) -> SharedDirResult<ResolvedSharedDirs> {
        let mut by_path: HashMap<PathBuf, SharedDirectory> = HashMap::new();
        let mut sources_hit = Vec::new();

        // workspace-level 优先 (P1/P2 通常)
        if !workspace.is_empty() {
            sources_hit.push(SharedDirSource::Workspace);
            for sd in workspace {
                validate(&sd)?;
                by_path.insert(sd.path.clone(), sd);
            }
        }

        // per-user 兜底 (P3), 同 path 不覆盖 workspace-level
        if !per_user.is_empty() {
            sources_hit.push(SharedDirSource::PerUser);
            for sd in per_user {
                validate(&sd)?;
                by_path.entry(sd.path.clone()).or_insert(sd);
            }
        }

        // .multica/config.yaml (FileBackedConfigSource 直读, 实装 OK)
        // 空数组不计入 sources_hit (per FR-ORCA-007 AC-4: empty config 不参与)
        if !config.shared_directories.is_empty() {
            sources_hit.push(SharedDirSource::MulticaConfig);
            for raw_path in &config.shared_directories {
                let normalized = normalize_path(raw_path).map_err(|e| {
                    SharedDirError::new(
                        "WSD.PATH_NOT_ABSOLUTE",
                        format!("path normalization failed for {raw_path:?}: {e}"),
                        DEFAULT_TRACE_ID,
                    )
                })?;
                let sd = SharedDirectory {
                    source: SharedDirSource::MulticaConfig,
                    path: normalized,
                    mount_strategy: SharedMountStrategy::WorktreeAdd,
                    label: derive_label(raw_path),
                    priority: SharedDirPriority::P3,
                    enabled: true,
                };
                validate(&sd)?;
                by_path.entry(sd.path.clone()).or_insert(sd);
            }
        }

        // 按 priority 升序 (P1 在前)
        let mut entries: Vec<SharedDirectory> = by_path.into_values().collect();
        entries.sort_by_key(|e| e.priority);
        entries.retain(|e| e.enabled);

        Ok(ResolvedSharedDirs {
            entries,
            sources_hit,
        })
    }
}

#[async_trait]
impl SharedDirResolver for InMemorySharedDirResolver {
    async fn resolve(&self, _repo_id: uuid::Uuid) -> SharedDirResult<ResolvedSharedDirs> {
        let per_user = self.per_user.list().await?;
        let workspace = self.workspace.list().await?;
        let config = self.config.load().await?;
        Self::merge(per_user, workspace, config)
    }
}

/// 校验单条 SharedDirectory
fn validate(sd: &SharedDirectory) -> SharedDirResult<()> {
    // path 必须绝对 (跨平台: Unix `/foo` 或 Windows `C:\foo` 都算绝对)
    let p = &sd.path;
    let is_absolute = p.is_absolute()
        || p.to_string_lossy().starts_with('/')
        || p.to_string_lossy().starts_with('\\');
    if !is_absolute {
        return Err(SharedDirError::new(
            "WSD.PATH_NOT_ABSOLUTE",
            format!("path must be absolute: {}", sd.path.display()),
            DEFAULT_TRACE_ID,
        ));
    }
    // priority 强制 P1/P2/P3 (enum 已经保证, 兜底)
    let p: SharedDirPriority = sd.priority;
    if !matches!(
        p,
        SharedDirPriority::P1 | SharedDirPriority::P2 | SharedDirPriority::P3
    ) {
        return Err(SharedDirError::new(
            "WSD.INVALID_PRIORITY",
            format!("priority must be P1/P2/P3, got {:?}", sd.priority),
            DEFAULT_TRACE_ID,
        ));
    }
    Ok(())
}

/// 路径归一化 (per FR-ORCA-006 并行 worktree 隔离保证):
/// - 空字符串 → 错误
/// - 绝对路径 (POSIX `/...` 或 Windows `C:\...` / `\\server\share`) → 直接
/// - 相对路径 → 当前调用者负责 workspace_root 解析 (本函数只做 lexical 折叠)
///
/// 注: 与 `worktree-service/src/shared_dir_sources.rs::normalize_path` 的语义不同
/// (后者用 workspace_root 解析相对路径), 这里只做纯 lexical 折叠, 因为
/// FileBackedConfigSource 持有 workspace_root 字段, 由 caller 在装配时统一解析。
///
/// 这里仅做最小实现: 折叠 `.` / `..`, 不做 workspace_root 注入 (per-source 各自的契约).
fn normalize_path(raw: &str) -> Result<PathBuf, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("empty path entry".to_string());
    }
    let p = Path::new(trimmed);
    let looks_absolute =
        p.is_absolute() || matches!(p.components().next(), Some(std::path::Component::RootDir));
    let joined: PathBuf = if looks_absolute {
        p.to_path_buf()
    } else {
        return Err(format!(
            "relative path requires workspace_root context: {raw}"
        ));
    };
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

/// 从 path 派生一个默认 label (basename, 兜底 "shared").
fn derive_label(raw: &str) -> String {
    let p = Path::new(raw.trim());
    p.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "shared".to_string())
}

/// Best-effort JSON value type tag for error messages.
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

    /// Tiny RAII tempdir helper to avoid pulling `tempfile` as a new dev-dep.
    /// Falls back to `std::env::temp_dir()` + cleanup-on-drop.
    mod tempfile_lite {
        use std::path::PathBuf;

        pub(super) struct TempDir(pub(super) PathBuf);
        impl TempDir {
            pub(super) fn new(prefix: &str) -> Self {
                let nanos = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0);
                let p = std::env::temp_dir().join(format!("wsd-{}-{}", prefix, nanos));
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

    use tempfile_lite::TempDir;

    fn write_config(dir: &Path, body: &str) {
        let cfg_dir = dir.join(CLI_CONFIG_DIR_NAME);
        fs::create_dir_all(&cfg_dir).expect("mkdir multica-config");
        fs::write(cfg_dir.join(CLI_CONFIG_FILE_NAME), body).expect("write config.json");
    }

    fn ws(s: &str, p: SharedDirPriority) -> SharedDirectory {
        SharedDirectory {
            source: SharedDirSource::Workspace,
            path: PathBuf::from(s),
            mount_strategy: SharedMountStrategy::Symlink,
            label: "label".to_string(),
            priority: p,
            enabled: true,
        }
    }

    fn pu(s: &str, p: SharedDirPriority) -> SharedDirectory {
        SharedDirectory {
            source: SharedDirSource::PerUser,
            path: PathBuf::from(s),
            mount_strategy: SharedMountStrategy::Copy,
            label: "label".to_string(),
            priority: p,
            enabled: true,
        }
    }

    // ---- path C constants ----

    #[test]
    fn resolve_cli_config_path_appends_multica_config_and_config_json() {
        let p = resolve_cli_config_path(Path::new("/tmp/repo"));
        assert_eq!(p, PathBuf::from("/tmp/repo/multica-config/config.json"));
    }

    #[test]
    fn source_label_constant_stable() {
        assert_eq!(SOURCE_CLI_CONFIG, "cli_config");
        assert_eq!(CLI_CONFIG_FIELD, "worktree_shared_directories");
        assert_eq!(CLI_CONFIG_FILE_NAME, "config.json");
        assert_eq!(CLI_CONFIG_DIR_NAME, "multica-config");
    }

    // ---- FileBackedConfigSource ----

    #[tokio::test]
    async fn file_backed_missing_file_returns_empty_lenient() {
        let tmp = TempDir::new("missing");
        let src = FileBackedConfigSource::new(tmp.path());
        let cfg = src.load().await.unwrap();
        assert!(cfg.shared_directories.is_empty());
    }

    #[tokio::test]
    async fn file_backed_missing_file_strict_returns_empty() {
        let tmp = TempDir::new("missing-strict");
        let cfg = FileBackedConfigSource::load_strict_sync(tmp.path()).unwrap();
        assert!(cfg.shared_directories.is_empty());
    }

    #[tokio::test]
    async fn file_backed_missing_field_returns_empty() {
        let tmp = TempDir::new("nofield");
        write_config(tmp.path(), r#"{"server_url": "https://example"}"#);
        let cfg = FileBackedConfigSource::load_strict_sync(tmp.path()).unwrap();
        assert!(cfg.shared_directories.is_empty());
    }

    #[tokio::test]
    async fn file_backed_null_field_returns_empty() {
        let tmp = TempDir::new("nullfield");
        write_config(tmp.path(), r#"{"worktree_shared_directories": null}"#);
        let cfg = FileBackedConfigSource::load_strict_sync(tmp.path()).unwrap();
        assert!(cfg.shared_directories.is_empty());
    }

    #[tokio::test]
    async fn file_backed_empty_array_returns_empty() {
        let tmp = TempDir::new("emptyarr");
        write_config(tmp.path(), r#"{"worktree_shared_directories": []}"#);
        let cfg = FileBackedConfigSource::load_strict_sync(tmp.path()).unwrap();
        assert!(cfg.shared_directories.is_empty());
    }

    #[tokio::test]
    async fn file_backed_absolute_paths_passed_through() {
        let tmp = TempDir::new("abspath");
        write_config(
            tmp.path(),
            r#"{"worktree_shared_directories": ["/opt/team/shared", "/var/cache/."]}"#,
        );
        let cfg = FileBackedConfigSource::load_strict_sync(tmp.path()).unwrap();
        assert_eq!(cfg.shared_directories.len(), 2);
        // 跨平台: `/opt/team/shared` 在 Windows 上会被转成 `\opt\team\shared`,
        // 在 POSIX 上保留 `/opt/team/shared`. 用 PathBuf 验证.
        let expected_a = std::path::PathBuf::from(if cfg!(windows) {
            r"\opt\team\shared"
        } else {
            "/opt/team/shared"
        });
        let expected_b = std::path::PathBuf::from(if cfg!(windows) {
            r"\var\cache"
        } else {
            "/var/cache"
        });
        let actual_a = std::path::PathBuf::from(&cfg.shared_directories[0]);
        let actual_b = std::path::PathBuf::from(&cfg.shared_directories[1]);
        assert!(
            cfg.shared_directories
                .iter()
                .map(std::path::PathBuf::from)
                .any(|p| p == expected_a),
            "missing {:?} in {:?}",
            expected_a,
            cfg.shared_directories
        );
        assert!(
            cfg.shared_directories
                .iter()
                .map(std::path::PathBuf::from)
                .any(|p| p == expected_b),
            "missing {:?} in {:?}",
            expected_b,
            cfg.shared_directories
        );
        // 抑制 unused 警告 (实际使用在闭包里)
        let _ = (actual_a, actual_b);
    }

    #[tokio::test]
    async fn file_backed_duplicate_paths_deduplicated() {
        let tmp = TempDir::new("dedup");
        write_config(
            tmp.path(),
            r#"{"worktree_shared_directories": ["shared", "  shared  ", "/opt/x", "/opt/x"]}"#,
        );
        let cfg = FileBackedConfigSource::load_strict_sync(tmp.path()).unwrap();
        assert_eq!(cfg.shared_directories.len(), 2);
        // "shared" 和 "  shared  ".trim() 都是 "shared"
        assert!(cfg.shared_directories.contains(&"shared".to_string()));
        // 跨平台: `/opt/x` 在 Windows 上变 `\opt\x`, POSIX 上保留 `/opt/x`.
        let expected_x = std::path::PathBuf::from(if cfg!(windows) { r"\opt\x" } else { "/opt/x" });
        assert!(
            cfg.shared_directories
                .iter()
                .map(std::path::PathBuf::from)
                .any(|p| p == expected_x),
            "missing {:?} in {:?}",
            expected_x,
            cfg.shared_directories
        );
    }

    #[tokio::test]
    async fn file_backed_non_string_entries_silently_skipped() {
        let tmp = TempDir::new("nonstr");
        write_config(
            tmp.path(),
            r#"{"worktree_shared_directories": ["valid", 42, null, true, {"nested": "obj"}, ["arr"]]}"#,
        );
        let cfg = FileBackedConfigSource::load_strict_sync(tmp.path()).unwrap();
        assert_eq!(cfg.shared_directories.len(), 1);
        assert_eq!(cfg.shared_directories[0], "valid");
    }

    #[tokio::test]
    async fn file_backed_empty_string_rejected_by_strict() {
        let tmp = TempDir::new("emptystr");
        write_config(
            tmp.path(),
            r#"{"worktree_shared_directories": ["valid", "", "  "]}"#,
        );
        let strict = FileBackedConfigSource::load_strict_sync(tmp.path());
        assert!(matches!(strict, Err(ref e) if e.code == "WSD.CONFIG_PARSE_FAIL"));
    }

    #[tokio::test]
    async fn file_backed_malformed_json_strict_errors() {
        let tmp = TempDir::new("badjson");
        write_config(tmp.path(), "{this is not valid json");
        let strict = FileBackedConfigSource::load_strict_sync(tmp.path());
        assert!(matches!(strict, Err(ref e) if e.code == "WSD.CONFIG_PARSE_FAIL"));
        // lenient mode → empty
        let src = FileBackedConfigSource::new(tmp.path());
        let cfg = src.load().await.unwrap();
        assert!(cfg.shared_directories.is_empty());
    }

    #[tokio::test]
    async fn file_backed_wrong_field_type_strict_errors() {
        let tmp = TempDir::new("wrongtype");
        write_config(
            tmp.path(),
            r#"{"worktree_shared_directories": "not-an-array"}"#,
        );
        let strict = FileBackedConfigSource::load_strict_sync(tmp.path());
        assert!(matches!(strict, Err(ref e) if e.code == "WSD.CONFIG_TYPE_FAIL"));
        // lenient → empty
        let src = FileBackedConfigSource::new(tmp.path());
        let cfg = src.load().await.unwrap();
        assert!(cfg.shared_directories.is_empty());
    }

    #[tokio::test]
    async fn file_backed_does_not_depend_on_multica_cli() {
        // 关键不变性: 本测试不需要 CLI, 不调用 multica config set/get.
        // 构造一个 workspace, 写 config.json, FileBackedConfigSource 直读.
        let tmp = TempDir::new("no-cli-dep");
        write_config(
            tmp.path(),
            r#"{"worktree_shared_directories": ["/opt/cli-free"]}"#,
        );
        let src = FileBackedConfigSource::new(tmp.path());
        let cfg = src.load().await.unwrap();
        assert_eq!(cfg.shared_directories.len(), 1);
        // 跨平台: `/opt/cli-free` 在 Windows 上变 `\opt\cli-free`, POSIX 上保留.
        let expected = std::path::PathBuf::from(if cfg!(windows) {
            r"\opt\cli-free"
        } else {
            "/opt/cli-free"
        });
        let actual = std::path::PathBuf::from(&cfg.shared_directories[0]);
        assert_eq!(actual, expected);
    }

    // ---- FileBackedPerUserSource ----

    fn write_per_user(dir: &Path, body: &str) -> PathBuf {
        // 写到 `dir/.star/worktree_shared_dirs.txt`, 跟生产路径布局一致
        let cfg_dir = dir.join(PERUSER_CONFIG_DIR_NAME);
        fs::create_dir_all(&cfg_dir).expect("mkdir .star");
        let p = cfg_dir.join(PERUSER_CONFIG_FILE_NAME);
        fs::write(&p, body).expect("write worktree_shared_dirs.txt");
        p
    }

    #[tokio::test]
    async fn per_user_backed_missing_file_returns_empty_lenient() {
        let tmp = TempDir::new("peruser-missing");
        let src = FileBackedPerUserSource::with_path(
            tmp.path().join(PERUSER_CONFIG_DIR_NAME).join(PERUSER_CONFIG_FILE_NAME),
        );
        let entries = src.list().await.unwrap();
        assert!(entries.is_empty());
        // source_path 暴露给 audit
        assert_eq!(
            src.source_path().await,
            tmp.path()
                .join(PERUSER_CONFIG_DIR_NAME)
                .join(PERUSER_CONFIG_FILE_NAME)
        );
    }

    #[tokio::test]
    async fn per_user_backed_missing_file_strict_returns_empty() {
        let tmp = TempDir::new("peruser-missing-strict");
        let cfg = tmp
            .path()
            .join(PERUSER_CONFIG_DIR_NAME)
            .join(PERUSER_CONFIG_FILE_NAME);
        let entries = FileBackedPerUserSource::load_strict_sync(&cfg).unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn per_user_backed_parses_absolute_paths_with_p3() {
        let tmp = TempDir::new("peruser-abs");
        write_per_user(
            tmp.path(),
            "# Per-user worktree shared dirs\n\
             /opt/team/shared-cache\n\
             /var/cache/build-output\n\n\
             # trailing comment\n",
        );
        let cfg = tmp
            .path()
            .join(PERUSER_CONFIG_DIR_NAME)
            .join(PERUSER_CONFIG_FILE_NAME);
        let entries = FileBackedPerUserSource::load_strict_sync(&cfg).unwrap();
        assert_eq!(entries.len(), 2);
        // 跨平台: `/opt/team/shared-cache` 在 Windows 上变 `\opt\team\shared-cache`
        let expected_a = std::path::PathBuf::from(if cfg!(windows) {
            r"\opt\team\shared-cache"
        } else {
            "/opt/team/shared-cache"
        });
        let expected_b = std::path::PathBuf::from(if cfg!(windows) {
            r"\var\cache\build-output"
        } else {
            "/var/cache/build-output"
        });
        let paths: Vec<std::path::PathBuf> = entries.iter().map(|e| e.path.clone()).collect();
        assert!(paths.contains(&expected_a), "missing {expected_a:?} in {paths:?}");
        assert!(paths.contains(&expected_b), "missing {expected_b:?} in {paths:?}");
        // 每条都标 P3 / PerUser / enabled / WorktreeAdd
        for e in &entries {
            assert_eq!(e.source, SharedDirSource::PerUser);
            assert_eq!(e.priority, SharedDirPriority::P3);
            assert!(e.enabled);
            assert_eq!(e.mount_strategy, SharedMountStrategy::WorktreeAdd);
        }
    }

    #[tokio::test]
    async fn per_user_backed_empty_file_returns_empty() {
        let tmp = TempDir::new("peruser-empty");
        write_per_user(tmp.path(), "");
        let cfg = tmp
            .path()
            .join(PERUSER_CONFIG_DIR_NAME)
            .join(PERUSER_CONFIG_FILE_NAME);
        let entries = FileBackedPerUserSource::load_strict_sync(&cfg).unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn per_user_backed_comments_and_blanks_skipped() {
        let tmp = TempDir::new("peruser-comments");
        write_per_user(
            tmp.path(),
            "# header comment\n\
             \n\
             // C-style comment also skipped\n\
                \n\
             /opt/team/shared\n\
             # inline comment\n\
             /opt/team/other\n",
        );
        let cfg = tmp
            .path()
            .join(PERUSER_CONFIG_DIR_NAME)
            .join(PERUSER_CONFIG_FILE_NAME);
        let entries = FileBackedPerUserSource::load_strict_sync(&cfg).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[tokio::test]
    async fn per_user_backed_duplicate_paths_deduplicated() {
        let tmp = TempDir::new("peruser-dedup");
        write_per_user(
            tmp.path(),
            "/opt/shared\n  /opt/shared  \n/opt/shared\n/opt/other\n",
        );
        let cfg = tmp
            .path()
            .join(PERUSER_CONFIG_DIR_NAME)
            .join(PERUSER_CONFIG_FILE_NAME);
        let entries = FileBackedPerUserSource::load_strict_sync(&cfg).unwrap();
        assert_eq!(entries.len(), 2);
        // 跨平台: `/opt/shared` 在 Windows 上变 `\opt\shared`
        let expected = std::path::PathBuf::from(if cfg!(windows) {
            r"\opt\shared"
        } else {
            "/opt/shared"
        });
        let expected_other = std::path::PathBuf::from(if cfg!(windows) {
            r"\opt\other"
        } else {
            "/opt/other"
        });
        let paths: Vec<std::path::PathBuf> = entries.iter().map(|e| e.path.clone()).collect();
        assert!(paths.contains(&expected));
        assert!(paths.contains(&expected_other));
    }

    #[tokio::test]
    async fn per_user_backed_relative_path_rejected_strict() {
        let tmp = TempDir::new("peruser-rel");
        write_per_user(tmp.path(), "/opt/abs\nrelative/path\n");
        let cfg = tmp
            .path()
            .join(PERUSER_CONFIG_DIR_NAME)
            .join(PERUSER_CONFIG_FILE_NAME);
        let strict = FileBackedPerUserSource::load_strict_sync(&cfg);
        assert!(
            matches!(strict, Err(ref e) if e.code == "WSD.PERUSER_PARSE_FAIL"),
            "expected WSD.PERUSER_PARSE_FAIL, got {strict:?}"
        );
    }

    #[tokio::test]
    async fn per_user_backed_relative_path_lenient_falls_back_to_empty() {
        let tmp = TempDir::new("peruser-rel-lenient");
        write_per_user(tmp.path(), "relative/path\n");
        let cfg = tmp
            .path()
            .join(PERUSER_CONFIG_DIR_NAME)
            .join(PERUSER_CONFIG_FILE_NAME);
        let src = FileBackedPerUserSource::with_path(cfg);
        let entries = src.list().await.unwrap();
        // lenient: 解析失败 → 降级为空 (per FR-ORCA-007 fallback)
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn per_user_backed_default_path_uses_home_star() {
        // 不依赖环境变量; 验证常量 + 拼装函数形态稳定.
        assert_eq!(PERUSER_CONFIG_FILE_NAME, "worktree_shared_dirs.txt");
        assert_eq!(PERUSER_CONFIG_DIR_NAME, ".star");
        let p = default_per_user_config_path();
        // 末位 component 必须是文件名
        assert_eq!(
            p.file_name().and_then(|s| s.to_str()),
            Some(PERUSER_CONFIG_FILE_NAME)
        );
        // 倒数第二 component 必须是 `.star`
        let parent = p.parent().expect("path has parent");
        assert_eq!(
            parent.file_name().and_then(|s| s.to_str()),
            Some(PERUSER_CONFIG_DIR_NAME)
        );
    }

    #[tokio::test]
    async fn per_user_backed_integration_with_resolver_three_sources() {
        // 端到端: FileBackedPerUserSource + FileBackedConfigSource + workspace 三路并存.
        let tmp = TempDir::new("peruser-three");
        write_per_user(tmp.path(), "/opt/peruser/a\n/opt/peruser/b\n");
        write_config(
            tmp.path(),
            r#"{"worktree_shared_directories": ["/opt/cli/x"]}"#,
        );
        let per_user_src =
            FileBackedPerUserSource::with_path(tmp.path().join(PERUSER_CONFIG_DIR_NAME).join(PERUSER_CONFIG_FILE_NAME));
        let resolver = InMemorySharedDirResolver::new(
            Arc::new(per_user_src),
            Arc::new(InMemorySharedDirWorkspaceSource::new(vec![ws(
                "/opt/ws/y",
                SharedDirPriority::P1,
            )])),
            Arc::new(FileBackedConfigSource::new(tmp.path())),
        );
        let r = resolver.resolve(uuid::Uuid::nil()).await.unwrap();
        // 3 source 命中: ws/y (P1) + cli/x (P3 multica) + peruser/a,b (P3)
        assert_eq!(r.entries.len(), 4);
        assert_eq!(r.entries[0].priority, SharedDirPriority::P1);
        assert_eq!(r.entries[0].source, SharedDirSource::Workspace);
        // per-user 的两条按 P3 排在最后, source 标 PerUser
        let per_user_entries: Vec<&SharedDirectory> = r
            .entries
            .iter()
            .filter(|e| e.source == SharedDirSource::PerUser)
            .collect();
        assert_eq!(per_user_entries.len(), 2);
        assert!(r.sources_hit.contains(&SharedDirSource::PerUser));
        assert!(r.sources_hit.contains(&SharedDirSource::MulticaConfig));
        assert!(r.sources_hit.contains(&SharedDirSource::Workspace));
    }

    // ---- InMemorySharedDirResolver 集成 ----

    #[tokio::test]
    async fn resolve_workspace_only_returns_workspace_entries_sorted() {
        let resolver = InMemorySharedDirResolver::new(
            Arc::new(NoopPerUserSource),
            Arc::new(InMemorySharedDirWorkspaceSource::new(vec![
                ws("/opt/shared/p2", SharedDirPriority::P2),
                ws("/opt/shared/p1", SharedDirPriority::P1),
            ])),
            Arc::new(NoopConfigSource),
        );
        let r = resolver.resolve(uuid::Uuid::nil()).await.unwrap();
        assert_eq!(r.entries.len(), 2);
        assert_eq!(r.entries[0].priority, SharedDirPriority::P1);
        assert_eq!(r.entries[1].priority, SharedDirPriority::P2);
        assert_eq!(r.sources_hit, vec![SharedDirSource::Workspace]);
    }

    #[tokio::test]
    async fn resolve_per_user_does_not_overwrite_workspace_at_same_path() {
        let resolver = InMemorySharedDirResolver::new(
            Arc::new(InMemorySharedDirPerUserSource::new(vec![pu(
                "/opt/shared/p3",
                SharedDirPriority::P3,
            )])),
            Arc::new(InMemorySharedDirWorkspaceSource::new(vec![ws(
                "/opt/shared/p3",
                SharedDirPriority::P1,
            )])),
            Arc::new(NoopConfigSource),
        );
        let r = resolver.resolve(uuid::Uuid::nil()).await.unwrap();
        assert_eq!(r.entries.len(), 1);
        assert_eq!(r.entries[0].source, SharedDirSource::Workspace);
        assert_eq!(r.entries[0].priority, SharedDirPriority::P1);
    }

    #[tokio::test]
    async fn resolve_disabled_entries_filtered() {
        let mut entry = ws("/opt/shared/p1", SharedDirPriority::P1);
        entry.enabled = false;
        let resolver = InMemorySharedDirResolver::new(
            Arc::new(NoopPerUserSource),
            Arc::new(InMemorySharedDirWorkspaceSource::new(vec![entry])),
            Arc::new(NoopConfigSource),
        );
        let r = resolver.resolve(uuid::Uuid::nil()).await.unwrap();
        assert_eq!(r.entries.len(), 0);
    }

    #[tokio::test]
    async fn resolve_config_noop_returns_empty() {
        let resolver = InMemorySharedDirResolver::default_noop();
        let r = resolver.resolve(uuid::Uuid::nil()).await.unwrap();
        assert_eq!(r.entries.len(), 0);
        // config source Noop 不计入 sources_hit (per FR-ORCA-007 AC-4: empty config 不参与)
        assert!(r.sources_hit.is_empty());
    }

    #[tokio::test]
    async fn resolve_per_user_only_p3() {
        let resolver = InMemorySharedDirResolver::new(
            Arc::new(InMemorySharedDirPerUserSource::new(vec![pu(
                "/home/u/shared",
                SharedDirPriority::P3,
            )])),
            Arc::new(NoopWorkspaceSource),
            Arc::new(NoopConfigSource),
        );
        let r = resolver.resolve(uuid::Uuid::nil()).await.unwrap();
        assert_eq!(r.entries.len(), 1);
        assert_eq!(r.entries[0].source, SharedDirSource::PerUser);
        assert_eq!(r.sources_hit, vec![SharedDirSource::PerUser]);
    }

    #[tokio::test]
    async fn resolve_invalid_path_rejected() {
        let resolver = InMemorySharedDirResolver::new(
            Arc::new(NoopPerUserSource),
            Arc::new(InMemorySharedDirWorkspaceSource::new(vec![
                SharedDirectory {
                    source: SharedDirSource::Workspace,
                    path: PathBuf::from("relative/path"),
                    mount_strategy: SharedMountStrategy::Symlink,
                    label: "bad".to_string(),
                    priority: SharedDirPriority::P1,
                    enabled: true,
                },
            ])),
            Arc::new(NoopConfigSource),
        );
        let err = resolver.resolve(uuid::Uuid::nil()).await.unwrap_err();
        assert_eq!(err.code, "WSD.PATH_NOT_ABSOLUTE");
    }

    #[tokio::test]
    async fn resolve_combined_workspace_p1_p2_per_user_p3() {
        let resolver = InMemorySharedDirResolver::new(
            Arc::new(InMemorySharedDirPerUserSource::new(vec![pu(
                "/home/u/shared",
                SharedDirPriority::P3,
            )])),
            Arc::new(InMemorySharedDirWorkspaceSource::new(vec![
                ws("/opt/shared/p2", SharedDirPriority::P2),
                ws("/opt/shared/p1", SharedDirPriority::P1),
            ])),
            Arc::new(NoopConfigSource),
        );
        let r = resolver.resolve(uuid::Uuid::nil()).await.unwrap();
        assert_eq!(r.entries.len(), 3);
        assert_eq!(r.entries[0].priority, SharedDirPriority::P1);
        assert_eq!(r.entries[1].priority, SharedDirPriority::P2);
        assert_eq!(r.entries[2].priority, SharedDirPriority::P3);
        assert_eq!(
            r.sources_hit,
            vec![SharedDirSource::Workspace, SharedDirSource::PerUser]
        );
    }

    #[tokio::test]
    async fn resolve_file_backed_config_three_sources_combine() {
        // 端到端: FileBackedConfigSource 直读 config.json, 与 workspace-level 合并.
        let tmp = TempDir::new("three-sources");
        write_config(
            tmp.path(),
            r#"{"worktree_shared_directories": ["/opt/cli/x"]}"#,
        );
        let file_src = FileBackedConfigSource::new(tmp.path());
        let resolver = InMemorySharedDirResolver::new(
            Arc::new(NoopPerUserSource),
            Arc::new(InMemorySharedDirWorkspaceSource::new(vec![ws(
                "/opt/ws/y",
                SharedDirPriority::P1,
            )])),
            Arc::new(file_src),
        );
        let r = resolver.resolve(uuid::Uuid::nil()).await.unwrap();
        assert_eq!(r.entries.len(), 2);
        assert_eq!(r.entries[0].priority, SharedDirPriority::P1);
        assert_eq!(r.entries[0].source, SharedDirSource::Workspace);
        assert_eq!(r.entries[1].priority, SharedDirPriority::P3);
        assert_eq!(r.entries[1].source, SharedDirSource::MulticaConfig);
        assert!(
            r.sources_hit.contains(&SharedDirSource::MulticaConfig),
            "MulticaConfig should be in sources_hit"
        );
    }
}
