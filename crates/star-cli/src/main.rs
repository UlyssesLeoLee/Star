//! `star` CLI — Phase D 骨架入口
//!
//! 极简实现,只暴露 3 个核心命令(per `docs/architecture/2026-08-26-upgrade/spec/cli/01-cli-spec.md` Phase D 范围):
//!
//! 1. `star agent capabilities --json` — 返回 `agent-api/v1#Capabilities` schema
//! 2. `star task current --json` — 返回 mock `agent-api/v1#CurrentTask`
//! 3. `star submit` — 占位实现(退出码 0)
//!
//! 完整 23 命令 spec 见 `docs/architecture/2026-08-26-upgrade/spec/cli/01-cli-spec.md` §2。
//! 本 crate 当前仅交付 Phase D 骨架,后续 Phase D.1 起按 spec 增量扩展。
//!
//! ## 设计原则
//!
//! - 0 unsafe(per Physis 同期工程基线)
//! - --json 输出必走稳定 `agent-api/v1` schema(即使 mock 也守门)
//! - 错误用 `thiserror`,不混 `anyhow`
//! - lib 层 0 `unwrap()`(main 顶层允许 `expect` 终止)

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use clap::{Parser, Subcommand};

mod commands;
mod error;
mod output;

pub(crate) use error::StarError;

use commands::{agent, submit, task};

/// `star` — STAR 工程协作平台 CLI(Phase D 骨架)
#[derive(Debug, Parser)]
#[command(name = "star", version, about, long_about = None)]
struct Cli {
    /// 顶层子命令
    #[command(subcommand)]
    command: TopCommand,
}

/// 顶层子命令枚举(per `docs/.../spec/cli/01-cli-spec.md` §2 缩到 Phase D MVP)
#[derive(Debug, Subcommand)]
enum TopCommand {
    /// Agent 域命令(`star agent capabilities` per §4)
    #[command(subcommand)]
    Agent(agent::AgentCommand),
    /// `star task current`(per §2 核心命令)
    #[command(subcommand)]
    Task(task::TaskCommand),
    /// `star submit` — Universal Submit 占位(per `spec/flows/05-universal-submit.md` 11/12 步流程)
    Submit(submit::SubmitArgs),
}

/// CLI 入口 — 解析 → dispatch → 退出码
fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::ExitCode::from(e.exit_code())
        }
    }
}

/// 内部 dispatch,方便测试
fn run(cli: Cli) -> Result<(), StarError> {
    match cli.command {
        TopCommand::Agent(agent_cmd) => agent_cmd.run(),
        TopCommand::Task(task_cmd) => task_cmd.run(),
        TopCommand::Submit(submit_args) => submit::run(submit_args),
    }
}
