//! `star agent ...` 子命令(per `docs/.../spec/cli/01-cli-spec.md` §4)
//!
//! Phase D 骨架只实现 `star agent capabilities --json` 1 个命令。
//! 完整命令(`describe` / `instructions` / `permissions`)待 Phase D.1 增量补齐。

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
    /// 强制 JSON 输出(Phase D 默认 true,mock 阶段只暴露 JSON 形态)
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
    use super::{CapabilitiesArgs, StarError, output};
    use serde::Serialize;

    /// `agent-api/v1#Capabilities` mock 实现
    ///
    /// schema 形状 per `spec/acceptance/12-capability-discovery.md` §3
    /// Phase D 骨架只暴露 stub — 完整 capability 列表待 Phase D.1 补齐
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "snake_case")]
    pub(crate) struct Capabilities {
        /// 守门标记
        pub schema_version: &'static str,
        /// 能力关键词列表(Phase D 全部标记 `false` 留空)
        pub capabilities: Vec<&'static str>,
        /// 命令清单(per spec §3 `commands` 子树)
        pub commands: Commands,
        /// Resources 模板(per spec §3 `resources` 子树)
        pub resources: Resources,
        /// 权限(per spec §3 `permissions` 子树,Phase D 默认全部 ALLOW)
        pub permissions: Permissions,
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

    /// Mock 入口
    pub(crate) fn run(args: &CapabilitiesArgs) -> Result<(), StarError> {
        let _ = args.json; // Phase D 永远 JSON 输出,字段仅留作未来扩展 hook
        let caps = mock_capabilities();
        let pretty = output::json_pretty(caps)?;
        println!("{pretty}");
        Ok(())
    }

    /// 构造 mock Capabilities(per spec §3 schema)
    fn mock_capabilities() -> Capabilities {
        Capabilities {
            schema_version: output::SCHEMA_VERSION,
            capabilities: vec![
                "tasks",
                "workspaces",
                "worktrees",
                "merge_requests",
                "context",
            ],
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
                        description: "Universal submit — 11/12 step flow (per spec/flows/05)",
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
