// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/domain-llm/src/composer.rs` — W3 (ULYS-98-W3) Composer 多文件 diff
//! 类型 (per `docs/briefs/ulys-98-star-cursor-min-v1.md` §"Sub-task 3.4").
//!
//! **v0.0.1 stub**: 6 类型只声明, 0 业务方法 / 0 真实 LLM 编排. W2
//! (ULYS-98-W2) 不依赖此模块, 但 lib.rs 已 `pub use`, 因此占位让
//! `cargo check -p domain-llm` 在 W2 阶段通过. 阶段 3 (W3) 落真实
//! diff 计算 + 选区校验 + 拼装 [`ComposerEditResponse`].
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #11 + 守门 #14 v2):
//! - 0 `unsafe` blocks (`unsafe_code = "forbid"` workspace lint).
//! - DTOs derive `Serialize` / `Deserialize` (per 守门 #11 缺标比错标).
//! - 5 域 Lead 真人到位前 Mavis 临时代签.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =====================================================================
// ComposerRole (mirrors ChatRole for editor context)
// =====================================================================

/// **ComposerRole** — speaker role in a Composer instruction context.
///
/// Mirrors [`crate::chat::ChatRole`] but kept distinct so the Composer
/// pipeline can evolve independently (e.g. add `Developer` role without
/// affecting Chat).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComposerRole {
    /// User instruction prompt.
    User,
    /// System context (rarely used in Composer).
    System,
    /// Composer assistant (placeholder for future multi-turn).
    Assistant,
}

impl ComposerRole {
    /// Stable string form (matches `serde(rename_all = "lowercase")` wire format).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::System => "system",
            Self::Assistant => "assistant",
        }
    }
}

// =====================================================================
// Selection (cursor range inside one file)
// =====================================================================

/// **Selection** — half-open byte range `[start, end)` inside one file.
///
/// Coordinates are byte offsets (UTF-8 aware consumers must validate
/// boundaries). v0.0.1 stub: no validation; W3 adds line / col pair
/// parsing if Monaco sends line/col coordinates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Selection {
    /// Byte offset of the selection start (inclusive).
    pub start: u32,
    /// Byte offset of the selection end (exclusive).
    pub end: u32,
}

impl Selection {
    /// Construct a selection. Caller validates `start <= end`.
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

// =====================================================================
// ComposerFileContext (per-file input to a Composer edit)
// =====================================================================

/// **ComposerFileContext** — one file's content + optional selection to be
/// included in a Composer edit request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComposerFileContext {
    /// File path (relative to worktree root, UTF-8).
    pub path: String,
    /// Full file content (UTF-8). W3 truncates files > N KiB.
    pub content: String,
    /// Optional cursor / selection inside the file.
    pub selection: Option<Selection>,
}

// =====================================================================
// ComposerEditRequest / ComposerEditResponse
// =====================================================================

/// **ComposerEditRequest** — input to the Composer endpoint.
///
/// W2 (this crate) only declares the type; the route handler lives in
/// `crates/api/src/composer.rs` (W3.4).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComposerEditRequest {
    /// Caller tenant id (per 守门 #13 b RLS 13 類).
    pub tenant_id: Uuid,
    /// Caller actor id (per 守门 #10 author).
    pub actor_id: Uuid,
    /// Per-file contexts (>= 1).
    pub files: Vec<ComposerFileContext>,
    /// User instruction (UTF-8 text).
    pub instruction: String,
    /// Optional request id for idempotency.
    pub request_id: Option<Uuid>,
}

/// **ComposerEdit** — single file edit produced by the Composer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComposerEdit {
    /// Target file path.
    pub path: String,
    /// Unified diff (UTF-8 text).
    pub diff: String,
    /// New full file content (UTF-8).
    pub new_content: String,
}

/// **ComposerEditResponse** — output of the Composer endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComposerEditResponse {
    /// Per-file edits, one entry per [`ComposerEditRequest::files`].
    pub edits: Vec<ComposerEdit>,
    /// Optional overall summary message (e.g. assistant note).
    pub summary: Option<String>,
}

// =====================================================================
// Unit Tests (per 守门 #1 v25 — >= 1 unit test per module)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composer_role_as_str_matches_serde_wire_format() {
        assert_eq!(ComposerRole::User.as_str(), "user");
        assert_eq!(ComposerRole::System.as_str(), "system");
        assert_eq!(ComposerRole::Assistant.as_str(), "assistant");
    }

    #[test]
    fn selection_new_stores_start_and_end() {
        let s = Selection::new(10, 20);
        assert_eq!(s.start, 10);
        assert_eq!(s.end, 20);
    }

    #[test]
    fn composer_file_context_stores_required_fields() {
        let f = ComposerFileContext {
            path: "src/lib.rs".to_string(),
            content: "fn main() {}".to_string(),
            selection: Some(Selection::new(0, 12)),
        };
        assert_eq!(f.path, "src/lib.rs");
        assert_eq!(f.content, "fn main() {}");
        assert_eq!(f.selection.unwrap().start, 0);
    }

    #[test]
    fn composer_edit_request_serialises_roundtrip() {
        let req = ComposerEditRequest {
            tenant_id: Uuid::nil(),
            actor_id: Uuid::nil(),
            files: vec![ComposerFileContext {
                path: "a.rs".into(),
                content: "x".into(),
                selection: None,
            }],
            instruction: "rename x to y".into(),
            request_id: Some(Uuid::new_v4()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let parsed: ComposerEditRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.instruction, "rename x to y");
        assert_eq!(parsed.files.len(), 1);
    }
}