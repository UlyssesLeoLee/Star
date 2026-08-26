//! `star task ...` 子命令(per `docs/.../spec/cli/01-cli-spec.md` §2)
//!
//! Phase D 骨架只实现 `star task current --json` 1 个命令,
//! 返回 mock `agent-api/v1#CurrentTask`(per `spec/agent-api/01-schema.md` §3.1 Task)。

#![warn(missing_docs)]

use clap::Subcommand;

use crate::error::StarError;
use crate::output;

/// `star task` 子命令枚举
#[derive(Debug, Subcommand)]
pub(crate) enum TaskCommand {
    /// `star task current` — 返回 mock CurrentTask
    Current(CurrentArgs),
}

/// `star task current` 参数
#[derive(Debug, clap::Args)]
pub(crate) struct CurrentArgs {
    /// 强制 JSON 输出(Phase D 默认 true)
    #[arg(long, default_value_t = true)]
    pub json: bool,
}

impl TaskCommand {
    /// dispatch 到具体子命令
    pub(crate) fn run(self) -> Result<(), StarError> {
        match self {
            Self::Current(args) => current::run(&args),
        }
    }
}

/// `current` 子命令实现模块
pub(crate) mod current {
    use super::{CurrentArgs, StarError, output};
    use chrono::{DateTime, Utc};
    use serde::Serialize;

    /// `agent-api/v1#CurrentTask` mock 实现
    ///
    /// schema 形状 per `spec/agent-api/01-schema.md` §3.1
    #[derive(Debug, Serialize)]
    pub(crate) struct CurrentTask {
        /// 守门标记
        pub schema_version: &'static str,
        /// 任务 ID(Phase D 固定 mock 值)
        pub id: &'static str,
        /// 任务标题
        pub title: &'static str,
        /// 任务状态
        pub status: &'static str,
        /// 分配给的 agent
        pub assigned_to: &'static str,
        /// 上下文引用(REQ / ADR / MR 列表)
        pub context_refs: Vec<&'static str>,
        /// 验收条件(Phase D 留空)
        pub acceptance_criteria: Vec<&'static str>,
        /// 标签
        pub labels: Vec<&'static str>,
        /// 最后更新时间
        pub updated_at: DateTime<Utc>,
    }

    /// Mock 入口
    pub(crate) fn run(args: &CurrentArgs) -> Result<(), StarError> {
        let _ = args.json;
        let task = mock_current_task();
        let pretty = output::json_pretty(task)?;
        println!("{pretty}");
        Ok(())
    }

    /// 构造 mock CurrentTask
    fn mock_current_task() -> CurrentTask {
        CurrentTask {
            schema_version: output::SCHEMA_VERSION,
            id: "STAR-1024",
            title: "Phase D 骨架 — STAR CLI / MCP / Context 三 crate 落地",
            status: "IN_PROGRESS",
            assigned_to: "agent-mock",
            context_refs: vec!["DEC-008", "arch/03-star-ai-compat-arch.md"],
            acceptance_criteria: vec![
                "cargo build -p star-cli -p star-mcp -p star-context 通过",
                "cargo clippy --workspace ... RUSTFLAGS=-D warnings 通过",
            ],
            labels: vec!["phase-d", "skeleton", "mvp"],
            updated_at: Utc::now(),
        }
    }
}
