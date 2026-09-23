//! `shared_dir_types.rs` — 类型定义 (per ULYS-104.3 FR-ORCA-007)
//!
//! Orca v1.0 §3 FR-ORCA-007 字面用 3 机制: (1) per-user config, (2) `orca.yaml`
//! `worktree.sharedDirectories`, (3) `.worktreeinclude` 文件 (copy 而非 symlink)。
//!
//! 9/22 D-Boy 拍板 TBD-0044-01 C 选项 (per comment `01a0c8e7-5df4-7b4c-af8d-528351eb4a22`):
//! **第 3 机制从依赖 Multica CLI 升版改为 本仓库直读 `<workspace>/multica-config/config.json`**。
//! 本仓库派生 spec:
//! - 机制 1 = per-user (PerUserSource, 读 `~/.star/shared` 默认)
//! - 机制 2 = workspace-level (WorkspaceSource, 读 `worktree_shared_dir` PG 表)
//! - 机制 3 = `<workspace>/multica-config/config.json` (ConfigSource, **FileBackedConfigSource 本仓库实装**)
//!
//! 不复用 `.worktreeinclude` (那是 Orca 设计, 不适合 Rust 体系的本仓库)。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Shared Directory 来源 (per FR-ORCA-007 3 机制)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SharedDirSource {
    /// 机制 1: per-user (per repo / per user 配置, 优先级 P3 = 最低)
    PerUser,
    /// 机制 2: workspace-level (PG 表 `worktree_shared_dir`, 优先级 P1/P2 = 最高)
    Workspace,
    /// 机制 3: `<workspace>/multica-config/config.json` 的
    /// `worktree_shared_directories` 字段 (FileBackedConfigSource 直读, 不走 CLI)
    MulticaConfig,
}

/// Mount 策略 (per FR-ORCA-007 AC-1/AC-3 + Orca 实测: APFS clone-copy on macOS)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SharedMountStrategy {
    /// git worktree add (默认, MVP 兜底 per 9/22 D-Boy 决策)
    WorktreeAdd,
    /// Symlink (per-user + workspace-level 都缺 symlink 优先)
    Symlink,
    /// Hardlink (per-file hardlink, 适合大文件 + 频繁读)
    Hardlink,
    /// Bind mount (Linux/macOS mount 集成, P1 followup)
    BindMount,
    /// Copy (`.worktreeinclude` 模式, Orca 默认用于 .env / .vscode/settings.json)
    Copy,
}

/// 优先级 (P1 最高 = workspace-level 主共享; P3 最低 = per-user 兜底)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SharedDirPriority {
    /// P1 = 最高 (workspace-level 主共享)
    P1 = 1,
    /// P2 = 中等 (workspace-level 次共享)
    P2 = 2,
    /// P3 = 最低 (per-user 兜底 + multica-config/config.json)
    P3 = 3,
}

/// 单条 Shared Directory 配置 (per FR-ORCA-007 1 row = 1 shared path)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedDirectory {
    /// Source 归属
    pub source: SharedDirSource,
    /// 共享路径 (绝对路径, 在 worktree 间共享)
    pub path: PathBuf,
    /// Mount 策略
    pub mount_strategy: SharedMountStrategy,
    /// 显示标签 (e.g. "node_modules", ".env")
    pub label: String,
    /// 优先级
    pub priority: SharedDirPriority,
    /// 是否启用
    pub enabled: bool,
}

/// Picker 候选 (per FR-ORCA-009, P2 推到下一阶段实装)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PickerCandidate {
    /// 候选 ID
    pub id: String,
    /// 显示名
    pub label: String,
    /// 描述 (e.g. "Linear issue #123", "Local branch", "Commit SHA abc123")
    pub description: String,
    /// 候选类型
    pub kind: PickerCandidateKind,
}

/// Picker 候选类型 (per FR-ORCA-009 4 选 1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PickerCandidateKind {
    /// Repo base ref (默认 origin/main)
    RepoBase,
    /// Local branch (另一个本地分支)
    LocalBranch,
    /// 特定 commit SHA
    CommitSha,
    /// 远程 branch (fetch 后 checkout)
    RemoteBranch,
}

/// `<workspace>/multica-config/config.json` 的 `worktree_shared_directories` 字段反序列化结果.
///
/// FileBackedConfigSource 直读 `<workspace>/multica-config/config.json`, 不依赖 Multica CLI.
/// 即使 CLI v0.4.42 的 16-key whitelist 不含本字段, 运维仍可手写 config.json 启用本机制.
///
/// ## serde 映射
///
/// JSON 字段名是 `worktree_shared_directories` (per 9/22 D-Boy 拍板 snake_case flat key),
/// Rust 字段名是 `shared_directories` (语义更明确). 用 `#[serde(rename)]` 桥接二者.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedDirConfig {
    /// `worktree_shared_directories` 字段: JSON array of strings
    #[serde(rename = "worktree_shared_directories", default)]
    pub shared_directories: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_dir_priority_ordering_p1_highest() {
        // P1 < P2 < P3 in enum variant order (rust Ord for u8 backing)
        assert!(SharedDirPriority::P1 < SharedDirPriority::P2);
        assert!(SharedDirPriority::P2 < SharedDirPriority::P3);
    }

    #[test]
    fn shared_dir_source_serde_snake_case() {
        let s = serde_json::to_string(&SharedDirSource::PerUser).unwrap();
        assert_eq!(s, "\"per_user\"");
        let s = serde_json::to_string(&SharedDirSource::MulticaConfig).unwrap();
        assert_eq!(s, "\"multica_config\"");
    }

    #[test]
    fn shared_mount_strategy_5_variants() {
        // Per FR-ORCA-007 AC-1 + Orca 实测: 5 档
        let all = [
            SharedMountStrategy::WorktreeAdd,
            SharedMountStrategy::Symlink,
            SharedMountStrategy::Hardlink,
            SharedMountStrategy::BindMount,
            SharedMountStrategy::Copy,
        ];
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn shared_dir_config_default_empty() {
        let cfg = SharedDirConfig::default();
        assert!(cfg.shared_directories.is_empty());
    }

    #[test]
    fn shared_dir_config_round_trip_json_with_array() {
        let raw = r#"{"worktree_shared_directories":["/opt/x","shared"]}"#;
        let cfg: SharedDirConfig = serde_json::from_str(raw).unwrap();
        assert_eq!(cfg.shared_directories.len(), 2);
        assert_eq!(cfg.shared_directories[0], "/opt/x");
        assert_eq!(cfg.shared_directories[1], "shared");
    }

    #[test]
    fn shared_directory_round_trip_serde() {
        let sd = SharedDirectory {
            source: SharedDirSource::Workspace,
            path: PathBuf::from("/opt/shared"),
            mount_strategy: SharedMountStrategy::Symlink,
            label: "shared".to_string(),
            priority: SharedDirPriority::P1,
            enabled: true,
        };
        let j = serde_json::to_string(&sd).unwrap();
        let back: SharedDirectory = serde_json::from_str(&j).unwrap();
        assert_eq!(sd, back);
    }
}
