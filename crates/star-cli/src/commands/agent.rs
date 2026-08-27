//! `star agent ...` 子命令(per `docs/.../spec/cli/01-cli-spec.md` §4)
//!
//! Phase D 实现 `star agent capabilities --json` 1 个命令。
//!
//! ## Phase D 实现
//!
//! - 读取 workspace `.git/HEAD` 拿到当前分支和 commit
//! - 把 workspace data 嵌入 `Capabilities.workspace` 字段
//! - 完整 capability 列表 / 静态命令清单 / 权限待 Phase D.1 补齐

#![warn(missing_docs)]

use clap::Subcommand;

use crate::error::StarError;
use crate::output;

/// `star agent` 子命令枚举
#[derive(Debug, Subcommand)]
pub(crate) enum AgentCommand {
    /// Capability Discovery(per `spec/acceptance/12-capability-discovery.md` §3)
    Capabilities(CapabilitiesArgs),
}

/// `star agent capabilities` 参数(Phase D 仅支持 `--json`)
#[derive(Debug, clap::Args)]
pub(crate) struct CapabilitiesArgs {
    /// 强制 JSON 输出(Phase D 默认 true,留作未来扩展 hook)
    #[arg(long, default_value_t = true)]
    pub json: bool,
}

impl AgentCommand {
    /// dispatch 到具体子命令
    pub(crate) fn run(self) -> Result<(), StarError> {
        match self {
            Self::Capabilities(args) => capabilities::run(&args),
        }
    }
}

/// `capabilities` 子命令实现模块
pub(crate) mod capabilities {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::{CapabilitiesArgs, StarError, output};
    use serde::Serialize;

    /// `agent-api/v1#Capabilities` 真实实现
    ///
    /// schema 形状 per `spec/acceptance/12-capability-discovery.md` §3,
    /// Phase D 新增 `workspace` 字段(读 `.git/HEAD` 拿真实数据)。
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "snake_case")]
    pub(crate) struct Capabilities {
        /// 守门标记
        pub schema_version: &'static str,
        /// 能力关键词列表
        pub capabilities: Vec<&'static str>,
        /// Workspace 真实数据(Phase D 新增,读 `.git/HEAD`)
        pub workspace: WorkspaceInfo,
        /// 命令清单(per spec §3 `commands` 子树)
        pub commands: Commands,
        /// Resources 模板(per spec §3 `resources` 子树)
        pub resources: Resources,
        /// 权限(per spec §3 `permissions` 子树,Phase D 默认全部 ALLOW)
        pub permissions: Permissions,
    }

    /// Workspace 数据(从 `.git/HEAD` 读)
    #[derive(Debug, Serialize)]
    pub(crate) struct WorkspaceInfo {
        /// Workspace 根路径(`.git` 父目录,或 CWD)
        pub path: String,
        /// 当前分支(如 `wt-phase-d-impl`)
        pub branch: Option<String>,
        /// 当前 HEAD commit SHA(如 `6f3c90a...`)
        pub head: Option<String>,
        /// `.git` 是否存在
        pub git_present: bool,
    }

    /// 命令清单(per spec §3)
    #[derive(Debug, Serialize)]
    pub(crate) struct Commands {
        /// agent 域命令
        pub agent: Vec<CommandEntry>,
        /// ide 域命令(Phase D 留空,IDE 命令待后续 phase 补齐)
        pub ide: Vec<CommandEntry>,
    }

    /// 单条命令条目
    #[derive(Debug, Serialize)]
    pub(crate) struct CommandEntry {
        /// 命令名
        pub name: &'static str,
        /// schema 引用(`agent-api/v1#Foo` 形态)
        pub schema_ref: &'static str,
        /// 一句话说明
        pub description: &'static str,
    }

    /// Resources 模板(per spec §3)
    #[derive(Debug, Serialize)]
    pub(crate) struct Resources {
        /// agent 域 resources
        pub agent: Vec<ResourceEntry>,
    }

    /// 单条 resource 模板
    #[derive(Debug, Serialize)]
    pub(crate) struct ResourceEntry {
        /// URI 模板
        pub uri_template: &'static str,
        /// 一句话说明
        pub description: &'static str,
    }

    /// 权限清单(per spec §3)
    #[derive(Debug, Serialize)]
    #[allow(clippy::struct_field_names)] // 字段名就是权限 key,统一保留原样
    pub(crate) struct Permissions {
        /// 读仓库权限
        pub read_repository: &'static str,
        /// 创建 worktree 权限
        pub create_worktree: &'static str,
        /// 部署生产权限
        pub deploy_production: &'static str,
    }

    /// Real 入口:读 `.git/HEAD` + 输出 JSON
    pub(crate) fn run(args: &CapabilitiesArgs) -> Result<(), StarError> {
        let _ = args.json; // 字段仅留作未来扩展 hook
        let workspace = read_workspace();
        let caps = build_capabilities(workspace);
        let pretty = output::json_pretty(caps)?;
        println!("{pretty}");
        Ok(())
    }

    /// 读 `.git/HEAD` 拿 workspace 真实数据
    ///
    /// - 路径发现:从 CWD 向上找 `.git` 目录(或 `.git` 文件 — worktree 形态)
    /// - `.git/HEAD` 格式:`ref: refs/heads/<branch>` 或 detached `<40-char-sha>`
    /// - 若 `ref:` 形态,继续解析 `refs/heads/<branch>` 拿 SHA
    ///   (worktree 的 ref 可能在 commondir)
    /// - 若 `.git/HEAD` 不存在,字段全 `None`,`git_present = false`
    fn read_workspace() -> WorkspaceInfo {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let (git_dir, workspace_path) = find_git_dir(&cwd);
        match git_dir {
            Some(git) => {
                let head_path = git.join("HEAD");
                let (branch, head) = parse_head(&git, &head_path);
                WorkspaceInfo {
                    path: workspace_path.to_string_lossy().to_string(),
                    branch,
                    head,
                    git_present: true,
                }
            }
            None => WorkspaceInfo {
                path: workspace_path.to_string_lossy().to_string(),
                branch: None,
                head: None,
                git_present: false,
            },
        }
    }

    /// 解析 worktree 形态的 git dir,返回 commondir 路径(若存在)
    ///
    /// - 主仓库:git dir 自身就是 commondir,返回 `Some(git_dir)`(待 join refs)
    /// - worktree:git dir 含 `commondir` 文件,内容是相对路径,拼成绝对路径
    /// - 读不到 `commondir`:`None`
    fn common_dir(git_dir: &Path) -> Option<PathBuf> {
        let commondir_file = git_dir.join("commondir");
        let content = fs::read_to_string(&commondir_file).ok()?;
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Some(git_dir.to_path_buf());
        }
        let resolved = if Path::new(trimmed).is_absolute() {
            PathBuf::from(trimmed)
        } else {
            git_dir.join(trimmed)
        };
        Some(resolved)
    }

    /// 从 `start` 向上查找 `.git` 目录
    ///
    /// - 找到 `.git` 目录(普通仓库)→ 返回 (Some(git_dir), workspace_path)
    /// - 找到 `.git` 文件(worktree 形态,内容是 `gitdir: <path>`)→ 解析 gitdir,
    ///   返回 (Some(resolved_git_dir), workspace_path)
    /// - 找不到 → (None, start)
    fn find_git_dir(start: &Path) -> (Option<PathBuf>, PathBuf) {
        let mut current = Some(start.to_path_buf());
        while let Some(dir) = current {
            let git_path = dir.join(".git");
            if git_path.is_dir() {
                return (Some(git_path), dir);
            }
            if git_path.is_file() {
                // worktree 形态:解析 gitdir: <path>
                if let Ok(content) = fs::read_to_string(&git_path) {
                    if let Some(gitdir) = content.trim().strip_prefix("gitdir: ") {
                        let resolved = if Path::new(gitdir).is_absolute() {
                            PathBuf::from(gitdir)
                        } else {
                            dir.join(gitdir)
                        };
                        return (Some(resolved), dir);
                    }
                }
                return (Some(git_path), dir);
            }
            current = dir.parent().map(Path::to_path_buf);
        }
        (None, start.to_path_buf())
    }

    /// 解析 `.git/HEAD` 内容
    ///
    /// - `ref: refs/heads/<branch>` → 在 commondir 读 packed-refs / loose ref 拿 SHA
    ///   → (Some(branch), Some(sha))
    /// - `<40-char-sha>` (detached) → (None, Some(sha))
    /// - 其他 / 读不到 → (None, None)
    fn parse_head(git_dir: &Path, head_path: &Path) -> (Option<String>, Option<String>) {
        let Ok(content) = fs::read_to_string(head_path) else {
            return (None, None);
        };
        let trimmed = content.trim();
        if let Some(rest) = trimmed.strip_prefix("ref: refs/heads/") {
            let branch = rest.to_string();
            // worktree 形态:refs 在 commondir;主仓库:commondir == git_dir
            let ref_root = common_dir(git_dir).unwrap_or_else(|| git_dir.to_path_buf());
            let ref_name = format!("refs/heads/{branch}");
            let sha = read_packed_ref(&ref_root, &ref_name)
                .or_else(|| read_loose_ref(&ref_root, &ref_name));
            (Some(branch), sha)
        } else if trimmed.len() == 40 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            (None, Some(trimmed.to_string()))
        } else {
            (None, None)
        }
    }

    /// 读 packed-refs 解析 `<ref>` 的 SHA(找不到返回 None)
    fn read_packed_ref(git_dir: &Path, ref_name: &str) -> Option<String> {
        let path = git_dir.join("packed-refs");
        let content = fs::read_to_string(&path).ok()?;
        for line in content.lines() {
            // 跳过注释 / 空白
            if line.is_empty() || line.starts_with('#') || line.starts_with('^') {
                continue;
            }
            // 格式: `<sha> <ref>` 或 `<sha> <ref> <peeled-sha>`(tag)
            let mut parts = line.split_whitespace();
            let sha = parts.next()?;
            let name = parts.next()?;
            if name == ref_name {
                return Some(sha.to_string());
            }
        }
        None
    }

    /// 读 loose ref 文件解析 SHA(找不到返回 None)
    fn read_loose_ref(git_dir: &Path, ref_name: &str) -> Option<String> {
        let path = git_dir.join(ref_name);
        let content = fs::read_to_string(&path).ok()?;
        let trimmed = content.trim();
        if trimmed.len() == 40 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            Some(trimmed.to_string())
        } else {
            // 可能是 `ref: ...` 链式引用,Phase D 不递归
            None
        }
    }

    /// 构造真实 Capabilities(per spec §3 schema)
    fn build_capabilities(workspace: WorkspaceInfo) -> Capabilities {
        Capabilities {
            schema_version: output::SCHEMA_VERSION,
            capabilities: vec![
                "tasks",
                "workspaces",
                "worktrees",
                "merge_requests",
                "context",
            ],
            workspace,
            commands: Commands {
                agent: vec![
                    CommandEntry {
                        name: "task current",
                        schema_ref: "agent-api/v1#CurrentTask",
                        description: "Retrieve the current task assigned to this agent",
                    },
                    CommandEntry {
                        name: "agent capabilities",
                        schema_ref: "agent-api/v1#Capabilities",
                        description: "Discover available STAR capabilities",
                    },
                    CommandEntry {
                        name: "submit",
                        schema_ref: "agent-api/v1#SubmitResult",
                        description: "Universal submit — 12 step flow (per spec/flows/05)",
                    },
                ],
                ide: vec![],
            },
            resources: Resources {
                agent: vec![ResourceEntry {
                    uri_template: "issue://{id}",
                    description: "Issue current state (MCP resource, optional)",
                }],
            },
            permissions: Permissions {
                read_repository: "ALLOW",
                create_worktree: "ALLOW",
                deploy_production: "DENY",
            },
        }
    }
}
