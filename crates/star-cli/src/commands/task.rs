//! `star task ...` 子命令(per `docs/.../spec/cli/01-cli-spec.md` §2)
//!
//! Phase D 实现 `star task current --json` 1 个命令。
//!
//! ## Phase D 实现
//!
//! - 读 workspace 根目录 `STAR-CURRENT-TASK.json`(per `spec/.../flows/01-agent-task-lifecycle.md`)
//! - 文件不存在 → 用 default mock(STAR-1024 / IN_PROGRESS / Phase D 骨架)
//! - 解析失败 → 退到 default(打印 stderr warning,但不阻塞)
//! - 符合 `agent-api/v1#CurrentTask` schema

use clap::Subcommand;

use crate::error::StarError;
use crate::output;

/// `star task` 子命令枚举
#[derive(Debug, Subcommand)]
pub(crate) enum TaskCommand {
    /// `star task current` — 返回 CurrentTask
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
    use std::fs;
    use std::path::{Path, PathBuf};

    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    use super::{output, CurrentArgs, StarError};

    /// `agent-api/v1#CurrentTask` schema(per `spec/agent-api/01-schema.md` §3.1)
    #[derive(Debug, Serialize)]
    pub(crate) struct CurrentTask {
        /// 守门标记
        pub schema_version: &'static str,
        /// 任务 ID
        pub id: String,
        /// 任务标题
        pub title: String,
        /// 任务状态(IN_PROGRESS / TODO / DONE)
        pub status: String,
        /// 分配给的 agent
        pub assigned_to: String,
        /// 上下文引用(REQ / ADR / MR 列表)
        pub context_refs: Vec<String>,
        /// 验收条件
        pub acceptance_criteria: Vec<String>,
        /// 标签
        pub labels: Vec<String>,
        /// 最后更新时间
        pub updated_at: DateTime<Utc>,
        /// 数据来源(per spec/flows/01 §3)
        /// - `file`:从 `STAR-CURRENT-TASK.json` 读
        /// - `default_mock`:文件缺失,fallback 到 default
        pub source: &'static str,
    }

    /// `STAR-CURRENT-TASK.json` 文件 schema(per `spec/flows/01-agent-task-lifecycle.md` §3)
    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct TaskFile {
        /// 任务 ID(必填)
        id: String,
        /// 任务标题(可选)
        #[serde(default)]
        title: Option<String>,
        /// 任务状态(可选,默认 IN_PROGRESS)
        #[serde(default)]
        status: Option<String>,
        /// 分配给的 agent(可选)
        #[serde(default)]
        assigned_to: Option<String>,
        /// 上下文引用(可选,默认空)
        #[serde(default)]
        context_refs: Vec<String>,
        /// 验收条件(可选,默认空)
        #[serde(default)]
        acceptance_criteria: Vec<String>,
        /// 标签(可选,默认空)
        #[serde(default)]
        labels: Vec<String>,
        /// 最后更新时间(可选,默认 now)
        #[serde(default)]
        updated_at: Option<DateTime<Utc>>,
    }

    /// `STAR-CURRENT-TASK.json` 在 workspace 根目录的相对路径
    const TASK_FILE: &str = "STAR-CURRENT-TASK.json";

    /// Real 入口:读 `STAR-CURRENT-TASK.json` 或 fallback default
    pub(crate) fn run(args: &CurrentArgs) -> Result<(), StarError> {
        let _ = args.json; // 字段仅留作未来扩展 hook
        let task_path = locate_task_file();
        let task = match task_path.as_ref() {
            Some(path) => match load_task_file(path) {
                Ok(parsed) => parsed,
                Err(e) => {
                    eprintln!("warning: failed to parse {TASK_FILE}: {e}; using default");
                    default_current_task()
                }
            },
            None => default_current_task(),
        };
        let pretty = output::json_pretty(task)?;
        println!("{pretty}");
        Ok(())
    }

    /// 在 CWD 向上找 `STAR-CURRENT-TASK.json`
    ///
    /// - 从 CWD 开始,逐级向上查
    /// - 找到 → `Some(path)`
    /// - 找不到 → `None`
    fn locate_task_file() -> Option<PathBuf> {
        let cwd = std::env::current_dir().ok()?;
        let mut current = Some(cwd);
        while let Some(dir) = current {
            let candidate = dir.join(TASK_FILE);
            if candidate.is_file() {
                return Some(candidate);
            }
            current = dir.parent().map(Path::to_path_buf);
        }
        None
    }

    /// 解析 `STAR-CURRENT-TASK.json` 到 `CurrentTask`
    fn load_task_file(path: &Path) -> Result<CurrentTask, StarError> {
        let content = fs::read_to_string(path)?;
        let parsed: TaskFile = serde_json::from_str(&content)?;
        Ok(CurrentTask {
            schema_version: output::SCHEMA_VERSION,
            id: parsed.id,
            title: parsed.title.unwrap_or_default(),
            status: parsed.status.unwrap_or_else(|| "IN_PROGRESS".to_string()),
            assigned_to: parsed
                .assigned_to
                .unwrap_or_else(|| "agent-unassigned".to_string()),
            context_refs: parsed.context_refs,
            acceptance_criteria: parsed.acceptance_criteria,
            labels: parsed.labels,
            updated_at: parsed.updated_at.unwrap_or_else(Utc::now),
            source: "file",
        })
    }

    /// Default mock(Phase D 骨架,文件不存在时 fallback)
    fn default_current_task() -> CurrentTask {
        CurrentTask {
            schema_version: output::SCHEMA_VERSION,
            id: "STAR-1024".to_string(),
            title: "Phase D 骨架 — STAR CLI / MCP / Context 三 crate 落地".to_string(),
            status: "IN_PROGRESS".to_string(),
            assigned_to: "agent-mock".to_string(),
            context_refs: vec![
                "DEC-008".to_string(),
                "arch/03-star-ai-compat-arch.md".to_string(),
            ],
            acceptance_criteria: vec![
                "cargo build -p star-cli -p star-mcp -p star-context 通过".to_string(),
                "cargo clippy --workspace ... RUSTFLAGS=-D warnings 通过".to_string(),
            ],
            labels: vec![
                "phase-d".to_string(),
                "skeleton".to_string(),
                "mvp".to_string(),
            ],
            updated_at: Utc::now(),
            source: "default_mock",
        }
    }
}
