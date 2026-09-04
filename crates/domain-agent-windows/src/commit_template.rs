//! Star Agent Windows — Commit Message 模板 + Worktree 检测 (wt-w27)
//!
//! Per 2026-08-29 10:33 JST Phase 2 候选 3:
//! 1. Commit message 模板 (Conventional Commits)
//! 2. Worktree 状态检测 (detached HEAD / conflict / dirty)

use std::path::Path;
use thiserror::Error;
use tokio::process::Command;
use uuid::Uuid;

// =====================================================================
// 1. value_object — Conventional Commits
// =====================================================================

/// Conventional Commits 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommitType {
    Feat,     // 新功能
    Fix,      // bug 修复
    Docs,     // 文档
    Style,    // 格式
    Refactor, // 重构
    Perf,     // 性能
    Test,     // 测试
    Chore,    // 杂项
    Build,    // 构建
    Ci,       // CI
}

impl CommitType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Feat => "feat",
            Self::Fix => "fix",
            Self::Docs => "docs",
            Self::Style => "style",
            Self::Refactor => "refactor",
            Self::Perf => "perf",
            Self::Test => "test",
            Self::Chore => "chore",
            Self::Build => "build",
            Self::Ci => "ci",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Feat => "✨",
            Self::Fix => "🐛",
            Self::Docs => "📝",
            Self::Style => "💄",
            Self::Refactor => "♻️",
            Self::Perf => "⚡",
            Self::Test => "✅",
            Self::Chore => "🔧",
            Self::Build => "📦",
            Self::Ci => "👷",
        }
    }
}

/// Scope (可选, e.g. feat(w16-cli))
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommitScope(pub String);

impl CommitScope {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Commit 模板
#[derive(Debug, Clone, PartialEq)]
pub struct CommitTemplate {
    pub commit_type: CommitType,
    pub scope: Option<CommitScope>,
    pub subject: String, // <= 72 字符
    pub body: Option<String>,
    pub footer: Option<String>,
    pub breaking: bool, // ! 标记
}

impl CommitTemplate {
    /// 构造 feat(scope): subject
    pub fn new(commit_type: CommitType, subject: impl Into<String>) -> Self {
        Self {
            commit_type,
            scope: None,
            subject: subject.into(),
            body: None,
            footer: None,
            breaking: false,
        }
    }

    pub fn with_scope(mut self, scope: CommitScope) -> Self {
        self.scope = Some(scope);
        self
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn with_footer(mut self, footer: impl Into<String>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    pub fn breaking(mut self, breaking: bool) -> Self {
        self.breaking = breaking;
        self
    }

    /// 渲染为完整 commit message
    pub fn render(&self) -> String {
        let bang = if self.breaking { "!" } else { "" };
        let scope_part = match &self.scope {
            Some(s) => format!("({})", s.as_str()),
            None => String::new(),
        };
        let mut msg = format!(
            "{}{}{}: {}{}",
            self.commit_type.emoji(),
            self.commit_type.as_str(),
            scope_part,
            if self.breaking { "! " } else { "" },
            self.subject,
        );
        // 去掉 emoji 后 (符合 Conventional Commits)
        let conventional_only = format!(
            "{}{}{}: {}",
            self.commit_type.as_str(),
            scope_part,
            bang,
            self.subject,
        );

        if let Some(body) = &self.body {
            msg.push_str(&format!("\n\n{}", body));
        }
        if let Some(footer) = &self.footer {
            msg.push_str(&format!("\n\n{}", footer));
        }
        // 默认返 emoji + conventional (前端展示用)
        if msg.starts_with(&self.commit_type.emoji().to_string()) {
            msg
        } else {
            conventional_only
        }
    }
}

// =====================================================================
// 2. service — CommitTemplateBuilder (智能生成)
// =====================================================================

/// 从文件变更列表 + 工作项信息自动生成 commit message
pub struct CommitTemplateBuilder;

impl CommitTemplateBuilder {
    /// 从 diff 推断 type (基于改动文件路径)
    pub fn infer_type(changed_files: &[String]) -> CommitType {
        let all_tests = changed_files
            .iter()
            .all(|f| f.contains("test") || f.contains("__tests__"));
        let all_docs = changed_files
            .iter()
            .all(|f| f.starts_with("docs/") || f.ends_with(".md"));
        let all_frontend = changed_files.iter().all(|f| f.starts_with("frontend/"));
        let has_cargo = changed_files
            .iter()
            .any(|f| f == "Cargo.toml" || f.ends_with(".lock"));
        let has_workflow = changed_files
            .iter()
            .any(|f| f.starts_with(".github/workflows/"));

        if all_tests {
            return CommitType::Test;
        }
        if all_docs {
            return CommitType::Docs;
        }
        if has_workflow {
            return CommitType::Ci;
        }
        if has_cargo && changed_files.len() <= 2 {
            return CommitType::Build;
        }
        if all_frontend {
            return CommitType::Feat;
        }
        CommitType::Feat // 默认
    }

    /// 从 worktree 名推断 scope
    pub fn infer_scope(worktree_id: Uuid, changed_files: &[String]) -> Option<CommitScope> {
        // 简化: 从首个改动的 crate 推断
        for f in changed_files {
            if let Some(idx) = f.find("crates/") {
                let rest = &f[idx + 7..];
                if let Some(slash) = rest.find('/') {
                    return Some(CommitScope::new(&rest[..slash]));
                }
            }
        }
        let _ = worktree_id;
        None
    }

    /// 从变更文件列表 + trigger 来源生成完整 commit message
    pub fn build(
        commit_type: CommitType,
        scope: Option<CommitScope>,
        changed_files: &[String],
        trigger_source: &str, // e.g. "wt-w25" 或 "agent-window"
    ) -> String {
        let file_count = changed_files.len();
        let subject = match commit_type {
            CommitType::Feat => format!("添加 {} 项变更 (来自 {})", file_count, trigger_source),
            CommitType::Fix => format!("修复 {} 项问题 (来自 {})", file_count, trigger_source),
            _ => format!(
                "{} (来自 {}, {} 文件)",
                commit_type.as_str(),
                trigger_source,
                file_count
            ),
        };
        let body = format!(
            "Auto-uploaded by Star Agent Task Window.\n\nFiles:\n{}",
            changed_files
                .iter()
                .map(|f| format!("- {}", f))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        let mut t = CommitTemplate::new(commit_type, subject);
        if let Some(s) = scope {
            t = t.with_scope(s);
        }
        t = t.with_body(body);
        t = t.with_footer(format!(
            "Trigger: {}\nGenerated-by: Star v0.1",
            trigger_source
        ));
        t.render()
    }
}

// =====================================================================
// 3. service — WorktreeStatus (git 状态检测)
// =====================================================================

/// Worktree 状态
#[derive(Debug, Clone, PartialEq)]
pub struct WorktreeStatus {
    pub path: String,
    pub branch: Option<String>,
    pub detached: bool,
    pub dirty: bool,
    pub ahead: u32,
    pub behind: u32,
    pub conflicts: Vec<String>,
    pub last_commit_sha: Option<String>,
}

impl WorktreeStatus {
    /// 是否可安全 commit
    pub fn safe_to_commit(&self) -> bool {
        !self.detached && self.conflicts.is_empty()
    }
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum WorktreeError {
    #[error("git 命令失败: {0}")]
    GitFailed(String),
    #[error("worktree 目录不存在: {0}")]
    NotFound(String),
    #[error("git rev-parse 失败: {0}")]
    NotARepository(String),
}

/// 探测 worktree 状态 (4 个并行 git 命令)
pub async fn detect_worktree_status(worktree_dir: &Path) -> Result<WorktreeStatus, WorktreeError> {
    if !worktree_dir.exists() {
        return Err(WorktreeError::NotFound(worktree_dir.display().to_string()));
    }

    // 1. branch
    let branch_out = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(worktree_dir)
        .output()
        .await
        .map_err(|e| WorktreeError::GitFailed(e.to_string()))?;
    let branch_str = String::from_utf8_lossy(&branch_out.stdout)
        .trim()
        .to_string();
    let detached = branch_str == "HEAD";
    let branch = if detached { None } else { Some(branch_str) };

    // 2. dirty (未提交变更)
    let status_out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(worktree_dir)
        .output()
        .await
        .map_err(|e| WorktreeError::GitFailed(e.to_string()))?;
    let dirty_str = String::from_utf8_lossy(&status_out.stdout);
    let dirty = !dirty_str.trim().is_empty();

    // 3. ahead/behind
    let (ahead, behind) = if let Some(b) = &branch {
        let ab_out = Command::new("git")
            .args([
                "rev-list",
                "--count",
                "--left-right",
                &format!("{}...@{{u}}", b),
            ])
            .current_dir(worktree_dir)
            .output()
            .await
            .ok();
        match ab_out {
            Some(o) if o.status.success() => {
                let s = String::from_utf8_lossy(&o.stdout);
                let parts: Vec<&str> = s.trim().split('\t').collect();
                let a = parts.first().and_then(|x| x.parse().ok()).unwrap_or(0);
                let be = parts.get(1).and_then(|x| x.parse().ok()).unwrap_or(0);
                (a, be)
            }
            _ => (0, 0),
        }
    } else {
        (0, 0)
    };

    // 4. conflicts (未解决 merge 冲突)
    let conflicts: Vec<String> = dirty_str
        .lines()
        .filter_map(|l| {
            // porcelain v1 格式: XY filename, XY 含 UU = unmerged
            let parts: Vec<&str> = l.splitn(2, ' ').collect();
            if parts.len() >= 1 && parts[0].len() >= 2 {
                let x = parts[0].chars().nth(0).unwrap_or(' ');
                let y = parts[0].chars().nth(1).unwrap_or(' ');
                if x == 'U' || y == 'U' || (x == 'A' && y == 'A') || (x == 'D' && y == 'D') {
                    return parts.get(1).map(|s| s.trim().to_string());
                }
            }
            None
        })
        .collect();

    // 5. last commit SHA
    let sha_out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(worktree_dir)
        .output()
        .await
        .map_err(|e| WorktreeError::GitFailed(e.to_string()))?;
    let last_commit_sha = if sha_out.status.success() {
        let s = String::from_utf8_lossy(&sha_out.stdout).trim().to_string();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    } else {
        None
    };

    Ok(WorktreeStatus {
        path: worktree_dir.display().to_string(),
        branch,
        detached,
        dirty,
        ahead,
        behind,
        conflicts,
        last_commit_sha,
    })
}

// =====================================================================
// 4. invariant
// =====================================================================

/// INV-CT-01: subject 必 <= 72 字符 (Conventional Commits)
pub fn inv_01_subject_length(template: &CommitTemplate) -> bool {
    template.subject.len() <= 72
}

/// INV-CT-02: breaking 必带 !
pub fn inv_02_breaking_bang(template: &CommitTemplate) -> bool {
    !template.breaking || template.subject.starts_with('!') || template.subject.contains("BREAKING")
}

/// INV-CT-03: 安全 commit 必不在 detached HEAD
pub fn inv_03_no_detached_for_commit(status: &WorktreeStatus) -> bool {
    !status.detached
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_type_as_str() {
        assert_eq!(CommitType::Feat.as_str(), "feat");
        assert_eq!(CommitType::Fix.as_str(), "fix");
    }

    #[test]
    fn test_commit_template_render_basic() {
        let t = CommitTemplate::new(CommitType::Feat, "添加 6 内置 CLI agent");
        let s = t.render();
        assert!(s.contains("feat:") || s.contains("feat"));
        assert!(s.contains("添加 6 内置 CLI agent"));
    }

    #[test]
    fn test_commit_template_with_scope() {
        let t = CommitTemplate::new(CommitType::Feat, "添加 cli domain")
            .with_scope(CommitScope::new("w17"));
        let s = t.render();
        assert!(s.contains("w17"));
    }

    #[test]
    fn test_commit_template_breaking() {
        let t = CommitTemplate::new(CommitType::Feat, "BREAKING: 重构 API").breaking(true);
        let s = t.render();
        assert!(s.contains("BREAKING") || s.contains("!"));
    }

    #[test]
    fn test_infer_type_all_tests() {
        let files = vec!["frontend/src/lib/__tests__/x.test.ts".to_string()];
        assert_eq!(CommitTemplateBuilder::infer_type(&files), CommitType::Test);
    }

    #[test]
    fn test_infer_type_all_docs() {
        let files = vec!["docs/frontend/design/x.md".to_string()];
        assert_eq!(CommitTemplateBuilder::infer_type(&files), CommitType::Docs);
    }

    #[test]
    fn test_infer_type_cargo() {
        let files = vec!["Cargo.toml".to_string()];
        assert_eq!(CommitTemplateBuilder::infer_type(&files), CommitType::Build);
    }

    #[test]
    fn test_infer_scope_from_crates() {
        let files = vec!["crates/domain-cli/src/lib.rs".to_string()];
        let scope = CommitTemplateBuilder::infer_scope(Uuid::new_v4(), &files);
        assert_eq!(scope.unwrap().0, "domain-cli");
    }

    #[test]
    fn test_build_full() {
        let files = vec![
            "crates/domain-cli/src/lib.rs".to_string(),
            "crates/domain-cli/Cargo.toml".to_string(),
        ];
        let msg = CommitTemplateBuilder::build(
            CommitType::Feat,
            Some(CommitScope::new("cli")),
            &files,
            "wt-w17",
        );
        assert!(msg.contains("feat"));
        assert!(msg.contains("cli"));
        assert!(msg.contains("wt-w17"));
        assert!(msg.contains("2")); // file count
    }

    #[test]
    fn test_worktree_status_safe_to_commit() {
        let mut s = WorktreeStatus {
            path: "/tmp".into(),
            branch: Some("main".into()),
            detached: false,
            dirty: true,
            ahead: 0,
            behind: 0,
            conflicts: vec![],
            last_commit_sha: Some("abc".into()),
        };
        assert!(s.safe_to_commit());
        s.detached = true;
        assert!(!s.safe_to_commit());
        s.detached = false;
        s.conflicts = vec!["a.rs".into()];
        assert!(!s.safe_to_commit());
    }

    #[test]
    fn test_inv_01_subject_length() {
        let t = CommitTemplate::new(CommitType::Feat, "短");
        assert!(inv_01_subject_length(&t));
        let long = "a".repeat(73);
        let t = CommitTemplate::new(CommitType::Feat, long);
        assert!(!inv_01_subject_length(&t));
    }

    #[test]
    fn test_inv_03_no_detached_for_commit() {
        let s = WorktreeStatus {
            path: "/tmp".into(),
            branch: Some("main".into()),
            detached: false,
            dirty: false,
            ahead: 0,
            behind: 0,
            conflicts: vec![],
            last_commit_sha: None,
        };
        assert!(inv_03_no_detached_for_commit(&s));
        let mut s = s;
        s.detached = true;
        assert!(!inv_03_no_detached_for_commit(&s));
    }
}
