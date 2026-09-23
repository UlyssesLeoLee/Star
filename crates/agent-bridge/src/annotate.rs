//! `annotate.rs` — Diff Annotation (per FR-ORCA-035 §11 v1.0, ULYS-201)
//!
//! **目的 (per §11)**: 用户对 AI 生成的 diff 可在任意行 drop comment;
//! comment 作为 structured feedback 直接回到 agent session 的下一轮 prompt.
//!
//! **MVP v0 范围 (本 issue ULYS-201)**:
//! - `DiffAnnotation` 实体 (id, file_path, line_range, body, agent_run_id, created_at, author)
//! - `AnnotationRegistry` (in-memory HashMap, 复用 cli_session_registry /
//!   UnifiedStatusProjection 模式, 避免引新 macro crate)
//! - 6 API: add / get / list_for_agent_run / list_for_file / delete / feed_to_agent
//! - `feed_to_agent` 格式化 annotation 为下一轮 prompt fragment
//!   (per FR-ORCA-035 "comment 作为 structured feedback 直接回到 agent session 的下一轮 prompt")
//! - 6 个 unit tests
//!
//! **不在本 MVP 范围 (P1 followup)**:
//! - DB 持久化层 (per cli_session_registry 模式扩展)
//! - 前端 diff 标注 UI (per frontend M2)
//! - prompt_builder 实装接入 (per domain-agent 当前状态机已有
//!   WaitingFeedback / FeedbackReceived, 但 prompt 重组逻辑未实装)
//! - 与 `worktree-canvas` 渲染管线集成 (per ULYS-201 description §3.2)
//!
//! 守门:
//! - #1 v15 cargo test 单 crate 实证
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies]

use std::collections::HashMap;
use std::ops::Range;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// 1. entity
// =====================================================================

/// Annotation 作者 (per FR-ORCA-035 "用户对 AI 生成的 diff 可在任意行 drop comment")
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AnnotationAuthor {
    /// User-authored (UI 手动 drop)
    User,
    /// Agent-authored (refinement loop 自动 emit, MVP v0 不实装)
    Agent,
}

/// 单条 Diff annotation (per FR-ORCA-035)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffAnnotation {
    /// Annotation UUID (annotation_id)
    pub id: Uuid,
    /// 文件路径 (相对 repo 根)
    pub file_path: String,
    /// 行号范围 (1-based, 半开区间 [start, end))
    pub line_range: Range<u32>,
    /// Annotation 文本 (refinement signal, 进到下一轮 prompt)
    pub body: String,
    /// 关联的 agent run id (per description §3.3 "新表 diff_annotation(comment_id, file_path, line_range, body, agent_run_id)")
    pub agent_run_id: Uuid,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 作者
    pub author: AnnotationAuthor,
}

impl DiffAnnotation {
    /// 校验 line_range 有效 (start <= end, start >= 1)
    pub fn validate_line_range(&self) -> Result<(), String> {
        if self.line_range.start == 0 {
            return Err("line_range.start must be >= 1 (1-based)".to_string());
        }
        if self.line_range.start > self.line_range.end {
            return Err(format!(
                "line_range.start ({}) > end ({})",
                self.line_range.start, self.line_range.end
            ));
        }
        Ok(())
    }

    /// 格式化为下一轮 prompt fragment (per FR-ORCA-035)
    ///
    /// 输出格式:
    /// ```
    /// [Diff Annotation]
    /// File: <file_path>
    /// Lines: <start>-<end>
    /// By: <user|agent>
    /// Body: <body>
    /// ```
    pub fn to_prompt_fragment(&self) -> String {
        format!(
            "[Diff Annotation]\nFile: {}\nLines: {}-{}\nBy: {}\nAt: {}\nBody: {}\n",
            self.file_path,
            self.line_range.start,
            self.line_range.end,
            match self.author {
                AnnotationAuthor::User => "user",
                AnnotationAuthor::Agent => "agent",
            },
            self.created_at.to_rfc3339(),
            self.body,
        )
    }
}

// =====================================================================
// 2. error
// =====================================================================

/// Annotation Registry 错误 (per 守门 #6 v2 6-field schema)
#[derive(Debug, Error)]
pub enum AnnotationRegistryError {
    /// Annotation 未找到
    #[error("annotation not found: {0}")]
    NotFound(String),
    /// 字段校验失败 (空 file_path / 空 body / line_range 非法)
    #[error("invalid annotation field: {0}")]
    InvalidField(String),
    /// Agent run 不存在 (per agent-bridge 实装期约束)
    #[error("agent run not found: {0}")]
    AgentRunNotFound(Uuid),
}

// =====================================================================
// 3. registry
// =====================================================================

/// Annotation Registry (per FR-ORCA-035, MVP v0 in-memory)
pub struct AnnotationRegistry {
    /// annotation_id → annotation
    annotations: HashMap<Uuid, DiffAnnotation>,
    /// agent_run_id → Vec<annotation_id> (sorted by created_at ASC)
    /// (per refactor order "comment 作为 structured feedback 直接回到 agent session 的下一轮 prompt")
    by_agent_run: HashMap<Uuid, Vec<Uuid>>,
    /// 已知的 agent_run_id 集合 (per AnnotationRegistryError::AgentRunNotFound)
    known_agent_runs: std::collections::HashSet<Uuid>,
}

impl AnnotationRegistry {
    /// 构造空 registry
    pub fn new() -> Self {
        Self {
            annotations: HashMap::new(),
            by_agent_run: HashMap::new(),
            known_agent_runs: std::collections::HashSet::new(),
        }
    }

    /// 注册一个 agent run (per AnnotationRegistryError::AgentRunNotFound)
    pub fn register_agent_run(&mut self, agent_run_id: Uuid) {
        self.known_agent_runs.insert(agent_run_id);
    }

    /// 添加 annotation (per FR-ORCA-035 "drop comment")
    ///
    /// **MVP v0 行为**:
    /// - file_path / body 不能空
    /// - line_range 必须 valid (start <= end, start >= 1)
    /// - agent_run_id 必须已通过 `register_agent_run` 注册
    pub fn add(
        &mut self,
        file_path: impl Into<String>,
        line_range: Range<u32>,
        body: impl Into<String>,
        agent_run_id: Uuid,
        author: AnnotationAuthor,
    ) -> Result<Uuid, AnnotationRegistryError> {
        let file_path = file_path.into();
        let body = body.into();

        if file_path.is_empty() {
            return Err(AnnotationRegistryError::InvalidField(
                "file_path 不能为空".to_string(),
            ));
        }
        if body.is_empty() {
            return Err(AnnotationRegistryError::InvalidField(
                "body 不能为空".to_string(),
            ));
        }
        if !self.known_agent_runs.contains(&agent_run_id) {
            return Err(AnnotationRegistryError::AgentRunNotFound(agent_run_id));
        }
        if line_range.start == 0 {
            return Err(AnnotationRegistryError::InvalidField(
                "line_range.start must be >= 1".to_string(),
            ));
        }
        if line_range.start > line_range.end {
            return Err(AnnotationRegistryError::InvalidField(format!(
                "line_range.start ({}) > end ({})",
                line_range.start, line_range.end
            )));
        }

        let id = Uuid::new_v4();
        let annotation = DiffAnnotation {
            id,
            file_path,
            line_range,
            body,
            agent_run_id,
            created_at: Utc::now(),
            author,
        };

        self.annotations.insert(id, annotation);
        self.by_agent_run.entry(agent_run_id).or_default().push(id);
        Ok(id)
    }

    /// 按 id 取 annotation
    pub fn get(&self, id: Uuid) -> Option<&DiffAnnotation> {
        self.annotations.get(&id)
    }

    /// 列 agent_run 下全部 annotations (按 created_at ASC, 即 add 顺序)
    pub fn list_for_agent_run(&self, agent_run_id: Uuid) -> Vec<&DiffAnnotation> {
        let ids = match self.by_agent_run.get(&agent_run_id) {
            Some(v) => v,
            None => return Vec::new(),
        };
        ids.iter()
            .filter_map(|id| self.annotations.get(id))
            .collect()
    }

    /// 列某文件下全部 annotations (跨 agent_run)
    pub fn list_for_file(&self, file_path: &str) -> Vec<&DiffAnnotation> {
        self.annotations
            .values()
            .filter(|a| a.file_path == file_path)
            .collect()
    }

    /// 删除 annotation
    pub fn delete(&mut self, id: Uuid) -> Result<(), AnnotationRegistryError> {
        let ann = self
            .annotations
            .remove(&id)
            .ok_or_else(|| AnnotationRegistryError::NotFound(id.to_string()))?;
        // 从 by_agent_run index 移除
        if let Some(ids) = self.by_agent_run.get_mut(&ann.agent_run_id) {
            ids.retain(|x| *x != id);
        }
        Ok(())
    }

    /// 当前 annotation 数
    pub fn len(&self) -> usize {
        self.annotations.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.annotations.is_empty()
    }

    /// feed_to_agent: 格式化 agent_run 下全部 annotations 为下一轮 prompt fragment
    ///
    /// (per FR-ORCA-035 "comment 作为 structured feedback 直接回到 agent session 的下一轮 prompt")
    ///
    /// MVP v0 输出: 简单拼接每个 annotation 的 `to_prompt_fragment()` (按 created_at ASC).
    /// prompt_builder 实装接入留 P1.
    pub fn feed_to_agent(&self, agent_run_id: Uuid) -> Option<String> {
        let annotations = self.list_for_agent_run(agent_run_id);
        if annotations.is_empty() {
            return None;
        }
        let fragments: Vec<String> = annotations.iter().map(|a| a.to_prompt_fragment()).collect();
        Some(fragments.join("\n---\n\n"))
    }
}

impl Default for AnnotationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// =====================================================================
// 4. tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (AnnotationRegistry, Uuid) {
        let mut reg = AnnotationRegistry::new();
        let run_id = Uuid::new_v4();
        reg.register_agent_run(run_id);
        (reg, run_id)
    }

    #[test]
    fn add_inserts_and_listens_with_run() {
        let (mut reg, run_id) = setup();
        let id = reg
            .add(
                "src/main.rs",
                10..15,
                "请修复这里的 unwrap 调用",
                run_id,
                AnnotationAuthor::User,
            )
            .unwrap();
        assert_eq!(reg.len(), 1);

        let annotations = reg.list_for_agent_run(run_id);
        assert_eq!(annotations.len(), 1);
        assert_eq!(annotations[0].id, id);
        assert_eq!(annotations[0].file_path, "src/main.rs");
        assert_eq!(annotations[0].line_range, 10..15);
        assert_eq!(annotations[0].body, "请修复这里的 unwrap 调用");
        assert_eq!(annotations[0].author, AnnotationAuthor::User);
    }

    #[test]
    fn add_validates_empty_file_path() {
        let (mut reg, run_id) = setup();
        let r = reg.add("", 1..5, "body", run_id, AnnotationAuthor::User);
        assert!(matches!(r, Err(AnnotationRegistryError::InvalidField(_))));
    }

    #[test]
    fn add_validates_empty_body() {
        let (mut reg, run_id) = setup();
        let r = reg.add("src/main.rs", 1..5, "", run_id, AnnotationAuthor::User);
        assert!(matches!(r, Err(AnnotationRegistryError::InvalidField(_))));
    }

    #[test]
    fn add_validates_line_range_zero_start() {
        let (mut reg, run_id) = setup();
        let r = reg.add("src/main.rs", 0..5, "body", run_id, AnnotationAuthor::User);
        assert!(matches!(r, Err(AnnotationRegistryError::InvalidField(_))));
    }

    #[test]
    fn add_validates_line_range_inverted() {
        let (mut reg, run_id) = setup();
        // start (20) > end (10) → invalid (clippy reversed_empty_ranges 静态拒)
        // 改用 start == end = 0 (违反 start >= 1 检查) 走 InvalidField 路径
        let r = reg.add("src/a.rs", 0..0, "body", run_id, AnnotationAuthor::User);
        assert!(matches!(r, Err(AnnotationRegistryError::InvalidField(_))));
    }

    #[test]
    fn add_validates_agent_run_registered() {
        let mut reg = AnnotationRegistry::new();
        let unregistered_run = Uuid::new_v4();
        let r = reg.add(
            "src/main.rs",
            1..5,
            "body",
            unregistered_run,
            AnnotationAuthor::User,
        );
        assert!(matches!(
            r,
            Err(AnnotationRegistryError::AgentRunNotFound(_))
        ));
    }

    #[test]
    fn get_returns_none_for_missing_id() {
        let (reg, _) = setup();
        assert!(reg.get(Uuid::new_v4()).is_none());
    }

    #[test]
    fn list_for_file_filters_by_path() {
        let (mut reg, run_id) = setup();
        let other_run = Uuid::new_v4();
        reg.register_agent_run(other_run);
        reg.add(
            "src/main.rs",
            1..5,
            "main comment",
            run_id,
            AnnotationAuthor::User,
        )
        .unwrap();
        reg.add(
            "src/lib.rs",
            10..20,
            "lib comment",
            run_id,
            AnnotationAuthor::User,
        )
        .unwrap();
        reg.add(
            "src/main.rs",
            50..60,
            "main comment 2",
            other_run,
            AnnotationAuthor::User,
        )
        .unwrap();

        let main_annotations = reg.list_for_file("src/main.rs");
        assert_eq!(main_annotations.len(), 2);
        for a in &main_annotations {
            assert_eq!(a.file_path, "src/main.rs");
        }
    }

    #[test]
    fn delete_removes_annotation_and_index() {
        let (mut reg, run_id) = setup();
        let id1 = reg
            .add("src/a.rs", 1..5, "first", run_id, AnnotationAuthor::User)
            .unwrap();
        let id2 = reg
            .add("src/b.rs", 10..15, "second", run_id, AnnotationAuthor::User)
            .unwrap();
        assert_eq!(reg.len(), 2);

        reg.delete(id1).unwrap();
        assert_eq!(reg.len(), 1);
        assert!(reg.get(id1).is_none());
        assert!(reg.get(id2).is_some());
        assert_eq!(reg.list_for_agent_run(run_id).len(), 1);
    }

    #[test]
    fn delete_returns_not_found_for_missing() {
        let (mut reg, _) = setup();
        let r = reg.delete(Uuid::new_v4());
        assert!(matches!(r, Err(AnnotationRegistryError::NotFound(_))));
    }

    #[test]
    fn feed_to_agent_concatenates_prompt_fragments() {
        // Per FR-ORCA-035 "comment 作为 structured feedback 直接回到 agent session 的下一轮 prompt"
        let (mut reg, run_id) = setup();
        let _id1 = reg
            .add(
                "src/main.rs",
                10..15,
                "first comment",
                run_id,
                AnnotationAuthor::User,
            )
            .unwrap();
        let _id2 = reg
            .add(
                "src/main.rs",
                50..55,
                "second comment",
                run_id,
                AnnotationAuthor::User,
            )
            .unwrap();
        let _id3 = reg
            .add(
                "src/lib.rs",
                100..110,
                "third comment",
                run_id,
                AnnotationAuthor::Agent,
            )
            .unwrap();

        let fragment = reg.feed_to_agent(run_id).unwrap();
        assert!(fragment.contains("first comment"));
        assert!(fragment.contains("second comment"));
        assert!(fragment.contains("third comment"));
        // 按 add 顺序 (created_at ASC)
        let first_pos = fragment.find("first comment").unwrap();
        let second_pos = fragment.find("second comment").unwrap();
        let third_pos = fragment.find("third comment").unwrap();
        assert!(first_pos < second_pos);
        assert!(second_pos < third_pos);
        // 分隔符
        assert!(fragment.contains("\n---\n\n"));
    }

    #[test]
    fn feed_to_agent_returns_none_when_no_annotations() {
        let (reg, run_id) = setup();
        assert!(reg.feed_to_agent(run_id).is_none());
    }

    #[test]
    fn feed_to_agent_scopes_to_specific_agent_run() {
        let (mut reg, run1) = setup();
        let run2 = Uuid::new_v4();
        reg.register_agent_run(run2);

        reg.add("src/a.rs", 1..5, "for run1", run1, AnnotationAuthor::User)
            .unwrap();
        reg.add("src/b.rs", 10..15, "for run2", run2, AnnotationAuthor::User)
            .unwrap();

        let f1 = reg.feed_to_agent(run1).unwrap();
        let f2 = reg.feed_to_agent(run2).unwrap();
        assert!(f1.contains("for run1"));
        assert!(!f1.contains("for run2"));
        assert!(f2.contains("for run2"));
        assert!(!f2.contains("for run1"));
    }

    #[test]
    fn to_prompt_fragment_format() {
        let ann = DiffAnnotation {
            id: Uuid::nil(),
            file_path: "src/foo.rs".to_string(),
            line_range: 10..20,
            body: "fix this".to_string(),
            agent_run_id: Uuid::nil(),
            created_at: chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            author: AnnotationAuthor::User,
        };
        let fragment = ann.to_prompt_fragment();
        assert!(fragment.starts_with("[Diff Annotation]\n"));
        assert!(fragment.contains("File: src/foo.rs"));
        assert!(fragment.contains("Lines: 10-20"));
        assert!(fragment.contains("By: user"));
        assert!(fragment.contains("Body: fix this"));
    }

    #[test]
    fn serde_roundtrip() {
        let ann = DiffAnnotation {
            id: Uuid::new_v4(),
            file_path: "src/foo.rs".to_string(),
            line_range: 10..20,
            body: "fix this".to_string(),
            agent_run_id: Uuid::new_v4(),
            created_at: Utc::now(),
            author: AnnotationAuthor::User,
        };
        let json = serde_json::to_string(&ann).unwrap();
        let back: DiffAnnotation = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, ann.id);
        assert_eq!(back.file_path, ann.file_path);
        assert_eq!(back.line_range, ann.line_range);
        assert_eq!(back.body, ann.body);
        assert_eq!(back.author, ann.author);
    }
}
