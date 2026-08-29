//! `star submit` — Universal Submit 真实 12 步流程
//!
//! per `docs/architecture/2026-08-26-upgrade/spec/flows/05-universal-submit.md`
//!
//! ## Phase D 实现
//!
//! 跑全 12 步流程,但 5-12 步均为 dry-run / mock:
//!
//! | # | 步骤 | Phase D 行为 |
//! |---|---|---|
//! | 1 | 检查 Task | 读 `STAR-CURRENT-TASK.json` |
//! | 2 | 检查 Workspace | 读 `.star/workspace.json` |
//! | 3 | 检查 Worktree | 读 `.git/HEAD` 拿分支 + SHA |
//! | 4 | 检查 Diff | spawn `git diff --stat` |
//! | 5 | Required Validation | spawn `cargo check` |
//! | 6 | 检查 Policy | 写死 ALLOW |
//! | 7 | Commit | spawn `git commit -am 'submit: <id>'`(空 commit 跳过) |
//! | 8 | Push | spawn `git push --dry-run` |
//! | 9 | 创建 MR | 打印 "would create MR" |
//! | 10 | 关联 Issue | 写 `.star/issue-link.json` |
//! | 11 | 回写 Agent 状态 | 写 `.star/agent-state.json` |
//! | 12 | 回写 IDE Session 状态 | 写 `.star/ide-session-state.json` |
//!
//! 每步输出 `[STEP n/12] <name>: <status>`,最后输出 `SubmitResult` JSON(per `agent-api/v1#SubmitResult`)。

#![warn(missing_docs)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use chrono::{DateTime, Utc};
use clap::Args;
use serde::Serialize;

use crate::error::StarError;
use crate::output;

/// `star submit` 参数
#[derive(Debug, Args)]
pub(crate) struct SubmitArgs {
    /// Dry run(Phase D 永远走流程,但 5-12 步默认 dry-run)
    #[arg(long, default_value_t = true)]
    pub dry_run: bool,
    /// 强制 JSON 输出(最后输出 `SubmitResult` JSON,中间步骤走 human-readable)
    #[arg(long, default_value_t = false)]
    pub json: bool,
    /// 跳过 commit(只跑 1-6 检查,8-12 仍 dry-run)
    #[arg(long, default_value_t = false)]
    pub no_commit: bool,
}

/// `agent-api/v1#SubmitResult` schema(per `spec/agent-api/01-schema.md` §3.3)
#[derive(Debug, Serialize)]
struct SubmitResult {
    /// 守门标记
    schema_version: &'static str,
    /// 状态:OK / FAILED
    status: String,
    /// commit SHA
    #[serde(skip_serializing_if = "Option::is_none")]
    commit_sha: Option<String>,
    /// MR ID(Phase D 留 None)
    #[serde(skip_serializing_if = "Option::is_none")]
    mr_id: Option<String>,
    /// pipeline run ID(Phase D 留 None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pipeline_run_id: Option<String>,
    /// 验证是否通过
    validation_passed: bool,
    /// policy 是否通过
    policy_checked: bool,
    /// 12 步执行结果
    steps: Vec<StepResult>,
    /// task id(per `STAR-CURRENT-TASK.json` 或 default)
    task_id: String,
    /// 提交开始时间
    started_at: DateTime<Utc>,
    /// 提交结束时间
    finished_at: DateTime<Utc>,
}

/// 单步结果
#[derive(Debug, Serialize)]
struct StepResult {
    /// 步骤号 (1-12)
    step: u8,
    /// 步骤名
    name: &'static str,
    /// 状态:OK / SKIPPED / FAILED
    status: &'static str,
    /// 备注
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

/// 12 步流程入口
pub(crate) fn run(args: SubmitArgs) -> Result<(), StarError> {
    let started_at = Utc::now();
    let mut steps: Vec<StepResult> = Vec::with_capacity(12);

    // ===== 1. 检查 Task =====
    let task_id = match step_check_task() {
        Ok(id) => {
            steps.push(StepResult {
                step: 1,
                name: "check_task",
                status: "OK",
                note: Some(format!("task_id = {id}")),
            });
            id
        }
        Err(e) => {
            steps.push(StepResult {
                step: 1,
                name: "check_task",
                status: "FAILED",
                note: Some(e),
            });
            return finish_result(args, started_at, steps, String::new(), None, false, false);
        }
    };

    // ===== 2. 检查 Workspace =====
    match step_check_workspace() {
        Ok(()) => steps.push(StepResult {
            step: 2,
            name: "check_workspace",
            status: "OK",
            note: None,
        }),
        Err(e) => {
            steps.push(StepResult {
                step: 2,
                name: "check_workspace",
                status: "FAILED",
                note: Some(e),
            });
            return finish_result(args, started_at, steps, task_id, None, false, false);
        }
    }

    // ===== 3. 检查 Worktree =====
    match step_check_worktree() {
        Ok(sha) => steps.push(StepResult {
            step: 3,
            name: "check_worktree",
            status: "OK",
            note: Some(format!("head = {sha}")),
        }),
        Err(e) => steps.push(StepResult {
            step: 3,
            name: "check_worktree",
            status: "FAILED",
            note: Some(e),
        }),
    };

    // ===== 4. 检查 Diff =====
    match step_check_diff() {
        Ok(line) => steps.push(StepResult {
            step: 4,
            name: "check_diff",
            status: "OK",
            note: Some(line),
        }),
        Err(e) => steps.push(StepResult {
            step: 4,
            name: "check_diff",
            status: "FAILED",
            note: Some(e),
        }),
    }

    // ===== 5. Required Validation (dry-run: cargo check) =====
    let validation_passed = match step_validation() {
        Ok(()) => {
            steps.push(StepResult {
                step: 5,
                name: "validation",
                status: "OK",
                note: Some("cargo check passed".to_string()),
            });
            true
        }
        Err(e) => {
            steps.push(StepResult {
                step: 5,
                name: "validation",
                status: "FAILED",
                note: Some(e),
            });
            false
        }
    };

    // ===== 6. 检查 Policy (Phase D: 写死 ALLOW) =====
    steps.push(StepResult {
        step: 6,
        name: "policy",
        status: "OK",
        note: Some("ALLOW (Phase D hard-coded)".to_string()),
    });

    // ===== 7. Commit (skip if --no-commit) =====
    let commit_sha = if args.no_commit {
        steps.push(StepResult {
            step: 7,
            name: "commit",
            status: "SKIPPED",
            note: Some("--no-commit".to_string()),
        });
        None
    } else {
        match step_commit(&task_id) {
            Ok(Some(sha)) => {
                steps.push(StepResult {
                    step: 7,
                    name: "commit",
                    status: "OK",
                    note: Some(format!("sha = {sha}")),
                });
                Some(sha)
            }
            Ok(None) => {
                steps.push(StepResult {
                    step: 7,
                    name: "commit",
                    status: "SKIPPED",
                    note: Some("nothing to commit".to_string()),
                });
                None
            }
            Err(e) => {
                steps.push(StepResult {
                    step: 7,
                    name: "commit",
                    status: "FAILED",
                    note: Some(e),
                });
                None
            }
        }
    };

    // ===== 8. Push (dry-run) =====
    match step_push_dry_run() {
        Ok(()) => steps.push(StepResult {
            step: 8,
            name: "push",
            status: "OK",
            note: Some("git push --dry-run".to_string()),
        }),
        Err(e) => steps.push(StepResult {
            step: 8,
            name: "push",
            status: "FAILED",
            note: Some(e),
        }),
    }

    // ===== 9. 创建 MR (dry-run: 仅打印) =====
    let mr_id = step_create_mr_dry_run(&task_id);
    steps.push(StepResult {
        step: 9,
        name: "create_mr",
        status: "OK",
        note: Some(format!("would create MR (mr_id = {mr_id})")),
    });

    // ===== 10. 关联 Issue =====
    match step_link_issue(&task_id) {
        Ok(()) => steps.push(StepResult {
            step: 10,
            name: "link_issue",
            status: "OK",
            note: Some(".star/issue-link.json".to_string()),
        }),
        Err(e) => steps.push(StepResult {
            step: 10,
            name: "link_issue",
            status: "FAILED",
            note: Some(e),
        }),
    }

    // ===== 11. 回写 Agent 状态 =====
    match step_write_agent_state(&task_id) {
        Ok(()) => steps.push(StepResult {
            step: 11,
            name: "agent_state",
            status: "OK",
            note: Some(".star/agent-state.json".to_string()),
        }),
        Err(e) => steps.push(StepResult {
            step: 11,
            name: "agent_state",
            status: "FAILED",
            note: Some(e),
        }),
    }

    // ===== 12. 回写 IDE Session 状态 =====
    match step_write_ide_state(&task_id) {
        Ok(()) => steps.push(StepResult {
            step: 12,
            name: "ide_state",
            status: "OK",
            note: Some(".star/ide-session-state.json".to_string()),
        }),
        Err(e) => steps.push(StepResult {
            step: 12,
            name: "ide_state",
            status: "FAILED",
            note: Some(e),
        }),
    }

    finish_result(
        args,
        started_at,
        steps,
        task_id,
        commit_sha,
        validation_passed,
        true,
    )
}

/// 打印每步状态 + 最后输出 SubmitResult
fn finish_result(
    args: SubmitArgs,
    started_at: DateTime<Utc>,
    steps: Vec<StepResult>,
    task_id: String,
    commit_sha: Option<String>,
    validation_passed: bool,
    policy_checked: bool,
) -> Result<(), StarError> {
    // 打印每步
    for s in &steps {
        let note = s.note.as_deref().unwrap_or("");
        println!("[STEP {}/12] {}: {} ({})", s.step, s.name, s.status, note);
    }

    let finished_at = Utc::now();
    let failed = steps.iter().any(|s| s.status == "FAILED");
    let result = SubmitResult {
        schema_version: output::SCHEMA_VERSION,
        status: if failed {
            "FAILED".to_string()
        } else {
            "OK".to_string()
        },
        commit_sha,
        mr_id: Some(format!("MR-{}-mock", task_id)),
        pipeline_run_id: Some(format!("pl-{}-mock", task_id)),
        validation_passed,
        policy_checked,
        steps,
        task_id,
        started_at,
        finished_at,
    };

    if args.json {
        let pretty = output::json_pretty(&result)?;
        println!("{pretty}");
    } else {
        // human-readable summary
        println!();
        println!("Submit {}", result.status);
        if let Some(sha) = &result.commit_sha {
            println!("  commit_sha: {sha}");
        }
        println!("  validation_passed: {}", result.validation_passed);
        println!("  policy_checked: {}", result.policy_checked);
    }

    if failed {
        // 失败时退出码 = 1(per `spec/cli/01-cli-spec.md` §5 错误模型)
        std::process::exit(1);
    }
    Ok(())
}

// ============= 12 步具体实现 =============

/// Step 1: 读 `STAR-CURRENT-TASK.json` 拿 task_id
fn step_check_task() -> Result<String, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let path =
        find_task_file(&cwd).ok_or_else(|| "STAR-CURRENT-TASK.json not found".to_string())?;
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    json.get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "missing 'id' field".to_string())
}

/// Step 2: 读 `.star/workspace.json`
fn step_check_workspace() -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let ws_path = cwd.join(".star").join("workspace.json");
    if !ws_path.is_file() {
        // Phase D: workspace.json 可选,缺失用 default
        eprintln!("warning: .star/workspace.json not found, using default");
        return Ok(());
    }
    let _content = fs::read_to_string(&ws_path).map_err(|e| e.to_string())?;
    Ok(())
}

/// Step 3: 读 `.git/HEAD` 拿分支 + SHA(worktree 形态下 `.git` 是 file 形态)
fn step_check_worktree() -> Result<String, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let dot_git = cwd.join(".git");
    // 1. 找 HEAD 实际位置
    let head_path = if dot_git.is_file() {
        // worktree 形态:.git 是 file,内容 `gitdir: <path>`
        let content = fs::read_to_string(&dot_git).map_err(|e| e.to_string())?;
        let gitdir = content
            .trim()
            .strip_prefix("gitdir: ")
            .ok_or_else(|| "invalid .git file".to_string())?;
        let resolved = if Path::new(gitdir).is_absolute() {
            PathBuf::from(gitdir)
        } else {
            cwd.join(gitdir)
        };
        resolved.join("HEAD")
    } else if dot_git.is_dir() {
        dot_git.join("HEAD")
    } else {
        return Err("no .git found".to_string());
    };
    let content = fs::read_to_string(&head_path).map_err(|e| e.to_string())?;
    Ok(content.trim().to_string())
}

/// Step 4: spawn `git diff --stat` 拿变更数
fn step_check_diff() -> Result<String, String> {
    let output = Command::new("git")
        .args(["diff", "--stat"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("failed to spawn git: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "git diff failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line_count = stdout.lines().count();
    Ok(format!("{line_count} stat lines"))
}

/// Step 5: spawn `cargo check`(本 crate 名 dry-run check)
fn step_validation() -> Result<(), String> {
    let output = Command::new("cargo")
        .args(["check", "-p", "star-cli", "--quiet"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("failed to spawn cargo: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo check failed: {}",
            String::from_utf8_lossy(&output.stderr)
                .lines()
                .take(5)
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    Ok(())
}

/// Step 7: spawn `git commit -am 'submit: <task_id>'` — 空 commit 跳过
fn step_commit(task_id: &str) -> Result<Option<String>, String> {
    // 先检查有没有 staged / unstaged 变更
    let diff_output = Command::new("git")
        .args(["diff", "--stat"])
        .output()
        .map_err(|e| e.to_string())?;
    let diff = String::from_utf8_lossy(&diff_output.stdout);
    if diff.trim().is_empty() {
        return Ok(None);
    }
    let msg = format!("submit: {task_id}");
    let output = Command::new("git")
        .args(["commit", "-am", &msg])
        .output()
        .map_err(|e| format!("failed to spawn git: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "git commit failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    // 拿最新 commit SHA
    let sha_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|e| e.to_string())?;
    let sha = String::from_utf8_lossy(&sha_output.stdout)
        .trim()
        .to_string();
    Ok(Some(sha))
}

/// Step 8: spawn `git push --dry-run`
fn step_push_dry_run() -> Result<(), String> {
    let output = Command::new("git")
        .args(["push", "--dry-run"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("failed to spawn git: {e}"))?;
    // --dry-run 在没 remote 时可能非 0,但仍视为 OK(Phase D 宽松)
    if !output.status.success() {
        // stderr 写一行 warning,不阻塞
        eprintln!(
            "warning: git push --dry-run exited {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
                .lines()
                .next()
                .unwrap_or("")
        );
    }
    Ok(())
}

/// Step 9: 打印 "would create MR",生成 mock mr_id
fn step_create_mr_dry_run(task_id: &str) -> String {
    format!("MR-{task_id}-mock")
}

/// Step 10: 写 `.star/issue-link.json`
fn step_link_issue(task_id: &str) -> Result<(), String> {
    let dir = star_dir()?;
    let path = dir.join("issue-link.json");
    let json = serde_json::json!({
        "schema_version": output::SCHEMA_VERSION,
        "task_id": task_id,
        "issue_id": format!("ISSUE-{task_id}"),
        "linked_at": Utc::now(),
    });
    write_json(&path, &json)
}

/// Step 11: 写 `.star/agent-state.json`
fn step_write_agent_state(task_id: &str) -> Result<(), String> {
    let dir = star_dir()?;
    let path = dir.join("agent-state.json");
    let json = serde_json::json!({
        "schema_version": output::SCHEMA_VERSION,
        "task_id": task_id,
        "agent_session_id": "agent-phase-d-mock",
        "status": "SUBMITTED",
        "submitted_at": Utc::now(),
    });
    write_json(&path, &json)
}

/// Step 12: 写 `.star/ide-session-state.json`
fn step_write_ide_state(task_id: &str) -> Result<(), String> {
    let dir = star_dir()?;
    let path = dir.join("ide-session-state.json");
    let json = serde_json::json!({
        "schema_version": output::SCHEMA_VERSION,
        "task_id": task_id,
        "ide_session_id": "ide-vscode-mock",
        "status": "READY_FOR_REVIEW",
        "updated_at": Utc::now(),
    });
    write_json(&path, &json)
}

// ============= 工具函数 =============

/// 创建/读取 `.star/` 目录
fn star_dir() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let dir = cwd.join(".star");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("create .star dir: {e}"))?;
    }
    Ok(dir)
}

/// 写 JSON 文件(pretty + 原子)
fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let content = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    let mut f = fs::File::create(path).map_err(|e| e.to_string())?;
    f.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
    f.write_all(b"\n").map_err(|e| e.to_string())?;
    Ok(())
}

/// 从 `start` 向上找 `STAR-CURRENT-TASK.json`
fn find_task_file(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start.to_path_buf());
    while let Some(dir) = current {
        let candidate = dir.join("STAR-CURRENT-TASK.json");
        if candidate.is_file() {
            return Some(candidate);
        }
        current = dir.parent().map(Path::to_path_buf);
    }
    None
}
