//! `clear.rs` — AI Vault clear_search_index (FR-ORCA-024 spec line 351-353)
//!
//! **目的 (per spec v1.0 line 351-353, ULYS-157 P0-C MVP)**:
//! `aiVault.clearSearchIndex()` 关闭 indexer / 删除 SQLite + sidecars (**不删除原始 transcripts**);
//! 重建若 consent 仍 enabled.
//!
//! **MVP v0 范围 (本 issue ULYS-157 P0-C)**:
//! - `ClearSearchIndexResult` 实体 (deleted: SQLite files, deleted: sidecars, skipped: raw transcripts)
//! - `VaultSearchIndexLayout` (SQLite + sidecars 路径推导)
//! - `clear_search_index(layout)` 纯函数: 删 SQLite + sidecars, 保留 raw
//!
//! **不在本 MVP 范围 (P1 followup)**:
//! - 实际 indexer 生命周期 (per description "实装期走原型对比")
//! - 重建 index 任务调度
//! - 并发保护 (per spec 隐含 — 当前 P1 不做)
//! - Permissions / RBAC (per spec "no-arg, desktop-only preload")
//!
//! 守门:
//! - #1 v15 cargo test 单 crate 实证
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies]

use std::path::{Path, PathBuf};

// =====================================================================
// 1. types
// =====================================================================

/// AI Vault 搜索索引 layout (per FR-ORCA-024)
///
/// MVP v0: 用 path 表示 SQLite + sidecars 位置, raw transcripts 在单独子目录.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultSearchIndexLayout {
    /// Vault 根目录 (e.g. `<data-root>/ai-vault/`)
    pub vault_root: PathBuf,
}

impl VaultSearchIndexLayout {
    /// 构造 layout (per vault 根目录)
    pub fn new(vault_root: impl Into<PathBuf>) -> Self {
        Self {
            vault_root: vault_root.into(),
        }
    }

    /// SQLite DB 路径 (per spec "删除 SQLite")
    ///
    /// MVP v0: 单一文件 `search-index.sqlite` (per spec "SQLite + sidecars" 单数表示)
    pub fn sqlite_path(&self) -> PathBuf {
        self.vault_root.join("search-index.sqlite")
    }

    /// Sidecar 路径 (per spec "sidecars" 复数 → 多文件)
    ///
    /// MVP v0: 单一 `search-index.jsonl` sidecar (per spec "删除 SQLite + sidecars" 隐含)
    /// P1 followup: 多 sidecar (per content type)
    pub fn sidecar_path(&self) -> PathBuf {
        self.vault_root.join("search-index.jsonl")
    }

    /// Raw transcripts 目录 (per spec "不删除原始 transcripts")
    pub fn raw_dir(&self) -> PathBuf {
        self.vault_root.join("raw")
    }
}

/// Clear Search Index 结果 (per spec line 353 隐含 "返回" 信息)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClearSearchIndexResult {
    /// 删除的 SQLite 文件路径
    pub deleted_sqlite: Vec<PathBuf>,
    /// 删除的 sidecar 文件路径
    pub deleted_sidecars: Vec<PathBuf>,
    /// 跳过的 raw transcripts 目录 (per spec "不删除原始 transcripts")
    pub preserved_raw_dir: Option<PathBuf>,
}

impl ClearSearchIndexResult {
    /// 总共删除文件数
    pub fn total_deleted(&self) -> usize {
        self.deleted_sqlite.len() + self.deleted_sidecars.len()
    }

    /// 是否完全 clean (无文件可删)
    pub fn is_clean(&self) -> bool {
        self.total_deleted() == 0
    }
}

// =====================================================================
// 2. error
// =====================================================================

/// Clear Index 错误 (per 守门 #6 v2 6-field schema 风格)
#[derive(Debug, thiserror::Error)]
pub enum ClearIndexError {
    /// IO 错误 (删除文件失败)
    #[error("io error at {path}: {source}")]
    Io {
        /// 出错的文件路径
        path: PathBuf,
        /// IO 错误源
        #[source]
        source: std::io::Error,
    },
}

// =====================================================================
// 3. clear function
// =====================================================================

/// `aiVault.clearSearchIndex()` — 删除 SQLite + sidecars, 保留 raw transcripts.
///
/// MVP v0 行为 (per spec line 353):
/// - 删除 `search-index.sqlite` (如果存在)
/// - 删除 `search-index.jsonl` (如果存在)
/// - **不触碰** `raw/` 子目录
/// - 缺失文件 = no-op (idempotent)
///
/// **不** 重建 index — 重建逻辑由 caller (indexer) 在 P1 后根据 SearchPolicy.enabled 触发.
pub fn clear_search_index(
    layout: &VaultSearchIndexLayout,
) -> Result<ClearSearchIndexResult, ClearIndexError> {
    let mut result = ClearSearchIndexResult::default();

    // 1. 删除 SQLite (per spec "删除 SQLite")
    let sqlite_path = layout.sqlite_path();
    if sqlite_path.exists() {
        delete_file(&sqlite_path)?;
        result.deleted_sqlite.push(sqlite_path);
    }

    // 2. 删除 sidecar (per spec "sidecars")
    let sidecar_path = layout.sidecar_path();
    if sidecar_path.exists() {
        delete_file(&sidecar_path)?;
        result.deleted_sidecars.push(sidecar_path);
    }

    // 3. 标记 raw 保留 (per spec "不删除原始 transcripts")
    let raw_dir = layout.raw_dir();
    if raw_dir.exists() {
        result.preserved_raw_dir = Some(raw_dir);
    }

    Ok(result)
}

/// 删除单个文件 (per `clear_search_index` helper)
fn delete_file(path: &Path) -> Result<(), ClearIndexError> {
    std::fs::remove_file(path).map_err(|source| ClearIndexError::Io {
        path: path.to_path_buf(),
        source,
    })
}

// =====================================================================
// 4. tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_layout() -> (TempDir, VaultSearchIndexLayout) {
        let tmp = tempfile::tempdir().unwrap();
        let vault_root = tmp.path().join("ai-vault");
        fs::create_dir_all(&vault_root).unwrap();
        fs::create_dir_all(vault_root.join("raw")).unwrap();
        (tmp, VaultSearchIndexLayout::new(vault_root))
    }

    #[test]
    fn layout_derives_paths_from_vault_root() {
        let (_tmp, layout) = make_layout();
        assert!(layout.sqlite_path().ends_with("search-index.sqlite"));
        assert!(layout.sidecar_path().ends_with("search-index.jsonl"));
        assert!(layout.raw_dir().ends_with("raw"));
    }

    #[test]
    fn clear_search_index_no_files_is_clean() {
        let (_tmp, layout) = make_layout();
        let result = clear_search_index(&layout).unwrap();
        assert!(result.is_clean());
        assert_eq!(result.total_deleted(), 0);
        assert!(result.deleted_sqlite.is_empty());
        assert!(result.deleted_sidecars.is_empty());
        assert!(result.preserved_raw_dir.is_some());
    }

    #[test]
    fn clear_search_index_deletes_sqlite() {
        let (_tmp, layout) = make_layout();
        let sqlite_path = layout.sqlite_path();
        fs::write(&sqlite_path, b"sqlite-content").unwrap();
        assert!(sqlite_path.exists());

        let result = clear_search_index(&layout).unwrap();
        assert_eq!(result.deleted_sqlite.len(), 1);
        assert_eq!(result.deleted_sqlite[0], sqlite_path);
        assert!(!sqlite_path.exists(), "SQLite 已被删除");
    }

    #[test]
    fn clear_search_index_deletes_sidecar() {
        let (_tmp, layout) = make_layout();
        let sidecar_path = layout.sidecar_path();
        fs::write(&sidecar_path, b"sidecar-content\n").unwrap();

        let result = clear_search_index(&layout).unwrap();
        assert_eq!(result.deleted_sidecars.len(), 1);
        assert!(!sidecar_path.exists());
    }

    #[test]
    fn clear_search_index_preserves_raw_transcripts() {
        let (_tmp, layout) = make_layout();
        let raw_path = layout.raw_dir().join("transcript-001.jsonl");
        fs::write(&raw_path, b"raw transcript content\n").unwrap();
        assert!(raw_path.exists());

        let result = clear_search_index(&layout).unwrap();
        assert_eq!(
            result.preserved_raw_dir,
            Some(layout.raw_dir()),
            "raw 目录被标记保留"
        );
        assert!(raw_path.exists(), "原始 transcripts 不被删除 (per spec)");
    }

    #[test]
    fn clear_search_index_deletes_both_files() {
        let (_tmp, layout) = make_layout();
        fs::write(layout.sqlite_path(), b"sqlite").unwrap();
        fs::write(layout.sidecar_path(), b"sidecar\n").unwrap();
        let raw_path = layout.raw_dir().join("t.jsonl");
        fs::write(&raw_path, b"raw\n").unwrap();

        let result = clear_search_index(&layout).unwrap();
        assert_eq!(result.total_deleted(), 2);
        assert_eq!(result.deleted_sqlite.len(), 1);
        assert_eq!(result.deleted_sidecars.len(), 1);
        assert!(raw_path.exists());
    }

    #[test]
    fn clear_search_index_is_idempotent() {
        let (_tmp, layout) = make_layout();
        fs::write(layout.sqlite_path(), b"sqlite").unwrap();
        fs::write(layout.sidecar_path(), b"sidecar\n").unwrap();

        // 第一次: 删除
        let r1 = clear_search_index(&layout).unwrap();
        assert_eq!(r1.total_deleted(), 2);

        // 第二次: idempotent no-op
        let r2 = clear_search_index(&layout).unwrap();
        assert!(r2.is_clean());
        assert_eq!(r2.total_deleted(), 0);
    }

    #[test]
    fn clear_search_index_missing_vault_dir_returns_clean() {
        // vault 目录不存在 → 不报错, 返回 clean (idempotent 设计)
        let tmp = tempfile::tempdir().unwrap();
        let missing_vault = tmp.path().join("non-existent-vault");
        let layout = VaultSearchIndexLayout::new(missing_vault);

        let result = clear_search_index(&layout).unwrap();
        assert!(result.is_clean());
        assert!(result.preserved_raw_dir.is_none());
    }

    #[test]
    fn result_total_deleted_counts_both() {
        let mut r = ClearSearchIndexResult::default();
        r.deleted_sqlite.push(PathBuf::from("/a.sqlite"));
        r.deleted_sqlite.push(PathBuf::from("/b.sqlite"));
        r.deleted_sidecars.push(PathBuf::from("/c.jsonl"));
        assert_eq!(r.total_deleted(), 3);
        assert!(!r.is_clean());
    }

    #[test]
    fn result_default_is_clean() {
        let r = ClearSearchIndexResult::default();
        assert!(r.is_clean());
        assert_eq!(r.total_deleted(), 0);
    }
}
