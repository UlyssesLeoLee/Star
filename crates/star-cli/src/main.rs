//! `star` CLI (Phase D.2 MVP 17 核心命令 + ULYS-196 Skill Registry 4 命令)
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/cli/01-cli-spec.md` §2
//!
//! ULYS-196 加 `star skill add/list/show/remove` 4 子命令 (per FR-ORCA-034 §10.2)

use clap::{Parser, Subcommand};

mod commands;
mod error;
mod output;
mod skill_registry;

pub(crate) use error::StarError;

use commands::{
    agent, code, context, issue, mr, pipeline, project, submit, task, test, workspace, worktree,
};
use skill_registry::SkillRegistry;

#[derive(Debug, Parser)]
#[command(name = "star", version, about, long_about = None)]
struct Cli {
    /// 强制 JSON 输出(per `spec/cli/01-cli-spec.md` §3 通用 flags)
    ///
    /// 现状:所有 MVP 17 命令已统一走 `output::json_pretty` 输出 JSON,本 flag
    /// 仅作为 clap global arg 暴露,所有子命令接受但不分支(per D.4 P1-1 修复)。
    #[arg(long, global = true)]
    #[allow(dead_code)] // clap derive 内部读取,运行时不直接用
    json: bool,

    #[command(subcommand)]
    command: TopCommand,
}

#[derive(Debug, Subcommand)]
enum TopCommand {
    #[command(subcommand)]
    Agent(agent::AgentCommand),
    #[command(subcommand)]
    Task(task::TaskCommand),
    Submit(submit::SubmitArgs),
    #[command(subcommand)]
    Project(project::ProjectCommand),
    #[command(subcommand)]
    Issue(issue::IssueCommand),
    #[command(subcommand)]
    Context(context::ContextCommand),
    #[command(subcommand)]
    Code(code::CodeCommand),
    #[command(subcommand)]
    Workspace(workspace::WorkspaceCommand),
    #[command(subcommand)]
    Worktree(worktree::WorktreeCommand),
    #[command(subcommand)]
    Mr(mr::MrCommand),
    #[command(subcommand)]
    Test(test::TestCommand),
    #[command(subcommand)]
    Pipeline(pipeline::PipelineCommand),
    /// ULYS-196: Skill Registry 子命令 (per FR-ORCA-034 §10.2)
    Skill(skill_registry::SkillCommandArgs),
}

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    let mut skill_registry = SkillRegistry::new();
    match run(cli, &mut skill_registry) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::ExitCode::from(e.exit_code())
        }
    }
}

fn run(cli: Cli, skill_registry: &mut SkillRegistry) -> Result<(), StarError> {
    match cli.command {
        TopCommand::Agent(c) => c.run(),
        TopCommand::Task(c) => c.run(),
        TopCommand::Submit(a) => submit::run(a),
        TopCommand::Project(c) => c.run(),
        TopCommand::Issue(c) => c.run(),
        TopCommand::Context(c) => c.run(),
        TopCommand::Code(c) => c.run(),
        TopCommand::Workspace(c) => c.run(),
        TopCommand::Worktree(c) => c.run(),
        TopCommand::Mr(c) => c.run(),
        TopCommand::Test(c) => c.run(),
        TopCommand::Pipeline(c) => c.run(),
        TopCommand::Skill(c) => skill_registry::run(c, skill_registry),
    }
}
