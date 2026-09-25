// SPDX-License-Identifier: MIT OR Apache-2.0
//! PI-7 Compaction 算法 (per SRS-PI-BORROW-001 §1.3 + §4 FR-28)
//!
//! **目的**: 长会话历史消息压缩 (per pi-coding-agent/src/core/compaction/compaction.ts
//! 36 KB port), 保留最近 N 轮 + summary, 同时追踪 read_files / modified_files
//! (cumulative across compactions).
//!
//! **架构 (per SRS §8 缺口 #7)**:
//! - 单独落 `compaction_entry` 表, **不动现有 `task_queue` schema**
//! - 跟 W/T/M 严格分类 (per 守门 #13 + 9/1 18:30 JST 拍板):
//!   - `compaction_entry` = **T (Transaction, append-only, 物理删除禁止, 审计必填)**
//!   - 理由: compaction 是不可变历史快照, 一旦写入不允许修改/物理删除 (审计 + 回溯)
//!   - 物理删除仅限 `cleanup_compaction_entries(before_ms)` 后台 TTL 策略 (per 守门 #13 c 派生)
//!
//! **设计 (per PI-7 v1.0)**:
//! - 模块自包含 (self-contained), **不依赖** `domain-agent` / `domain-llm` / `star-dto`
//!   (PI-6 star-dto 协议未 land, 跨 PI 集成留到 v1.1 续做)
//! - 纯算法函数 (`extract_file_ops` / `find_cut_point` / `summarize_messages`)
//!   不带 I/O, 便于 unit test + 未来跨 crate 复用
//! - LLM 摘要 (per pi.ts `generateSummary`) 在 v1.0 走**确定性 stub**
//!   (`summarize_messages` 截断 + marker); v1.1 接 `domain-llm` provider pool
//!
//! **守门合规 (per 守门 #1 v15 + #11 + #13 + #14 v4)**:
//! - 0 `unsafe` (workspace lint `unsafe_code = "forbid"`)
//! - `missing_docs = "deny"` → 所有 public item 必带 `///` doc
//! - `unreachable_pub = "deny"` → 所有 pub item 必须在 lib.rs 公开或带 `pub(crate)`
//! - 1 commit 多文件 (per AGENTS §4.1 v0.x 守门 #1 v15 派生)
//! - author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 守门 #14 v4)
//!
//! **PI-7 v1.0 范围**:
//! - ✅ read_files + modified_files tracking (cumulative)
//! - ✅ 保留最近 N 轮 + summary (cut-point 算法)
//! - ✅ SQLite WAL 持久化 (独立 `compaction_entry` 表)
//! - ✅ 8 unit test + 1 integration test
//! - ⏳ LLM 摘要接 domain-llm provider pool (PI-7 v1.1, per ADR-0048 D42)
//! - ⏳ 跟 PI-6 star-dto `AgentStreamEvent` 类型对齐 (PI-7 v1.1)
//! - ⏳ Stage 4 跟 PI-6 并行 (per issue 描述), 冲突检测在本 commit 已落 (无冲突)
//!
//! 参考 (per pi-coding-agent):
//! - <https://github.com/earendil-works/pi/blob/main/packages/coding-agent/src/core/compaction/compaction.ts>
//! - <https://github.com/earendil-works/pi/blob/main/packages/coding-agent/src/core/compaction/utils.ts>
//! - <https://pi.dev/docs/latest/compaction>

use std::collections::BTreeSet;

use async_trait::async_trait;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::sqlite_backend::now_ms;

#[allow(unused_imports)]
use super::TaskQueueError as _;

// =====================================================================
// §1 消息模型 (self-contained; PI-6 落地后用 From 适配)
// =====================================================================

/// **MessageRole** -- 消息角色 (3 角色 per pi AgentMessage role 子集)
///
/// pi 的 `AgentMessage` 含 system/user/assistant/toolResult; PI-7 v1.0
/// 关注 user + assistant + toolResult 3 角色 (system 在 compaction 中
/// 不参与 cut point, 总是保留, per pi.ts `pathEntries` 处理).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// 用户消息.
    User,
    /// 助手消息 (含 tool calls).
    Assistant,
    /// 工具调用结果.
    Tool,
}

impl MessageRole {
    /// Stable string form (matches the `serde(rename_all = "lowercase")` wire format).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }
}

/// **ToolCall** -- 单次 tool 调用 (extracted from assistant message content block)
///
/// pi 的 `AgentMessage.content` 是 block 数组; PI-7 v1.0 只关心 toolCall block,
/// 提取 `{name, arguments}`. 文件操作识别依赖 `arguments.path` (per pi.ts
/// `extractFileOpsFromMessage` case "read" / "write" / "edit").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool 名称 (e.g. "read" / "write" / "edit" / "bash" / "glob" ...).
    pub name: String,
    /// Tool 参数 (JSON object; 路径在 `path` 字段, per pi convention).
    pub arguments: serde_json::Value,
}

/// **CompactionMessage** -- 参与 compaction 的最小消息单元
///
/// **设计 (per PI-7 v1.0)**: 不直接用 pi 的 `AgentMessage` (那是 PI-6 star-dto 协议),
/// 仅保留 compaction 算法关心的字段. v1.1 落 `From<domain_llm::ChatMessage>` adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionMessage {
    /// 消息唯一 ID (UUID v4).
    pub id: Uuid,
    /// 消息角色.
    pub role: MessageRole,
    /// 消息文本内容 (assistant 含 thinking 时合并; toolResult 为 result 文本).
    pub content: String,
    /// 助手消息的 tool calls (user/tool 角色为空).
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
}

impl CompactionMessage {
    /// 构造 user 消息.
    pub fn user(id: Uuid, content: impl Into<String>) -> Self {
        Self {
            id,
            role: MessageRole::User,
            content: content.into(),
            tool_calls: Vec::new(),
        }
    }

    /// 构造 assistant 消息 (含 tool calls).
    pub fn assistant(
        id: Uuid,
        content: impl Into<String>,
        tool_calls: Vec<ToolCall>,
    ) -> Self {
        Self {
            id,
            role: MessageRole::Assistant,
            content: content.into(),
            tool_calls,
        }
    }

    /// 构造 tool result 消息.
    pub fn tool(id: Uuid, content: impl Into<String>) -> Self {
        Self {
            id,
            role: MessageRole::Tool,
            content: content.into(),
            tool_calls: Vec::new(),
        }
    }

    /// 是否 user 消息 (用于 turn 边界识别).
    pub fn is_user_turn(&self) -> bool {
        matches!(self.role, MessageRole::User)
    }
}

// =====================================================================
// §2 文件操作追踪 (per pi.ts `extractFileOperations` + `extractFileOpsFromMessage`)
// =====================================================================

/// **FileOps** -- 单次 extract 出来的文件操作集合 (mutable accumulator)
///
/// 跟 pi `FileOperations` 对齐: read / written / edited 三类, 最后聚合时
/// `modified = edited ∪ written`, `readOnly = read - modified`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileOps {
    /// Read-only 文件 (read tool 调用过, 但没被 write/edit).
    pub read: BTreeSet<String>,
    /// Write tool 调用过的文件 (整文件覆盖).
    pub written: BTreeSet<String>,
    /// Edit tool 调用过的文件 (in-place 修改).
    pub edited: BTreeSet<String>,
}

impl FileOps {
    /// 新建空 accumulator.
    pub fn new() -> Self {
        Self::default()
    }

    /// 合并另一个 `FileOps` (用于跨 compaction 累积).
    pub fn merge(&mut self, other: &FileOps) {
        self.read.extend(other.read.iter().cloned());
        self.written.extend(other.written.iter().cloned());
        self.edited.extend(other.edited.iter().cloned());
    }
}

/// **CompactionDetails** -- `CompactionEntry.details` 的标准 schema
///
/// per pi.ts `CompactionDetails`: read_files (read-only) + modified_files (edited ∪ written).
/// **设计**: 存盘前 `Vec<String>` 已 sort + dedup, JSON 序列化.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionDetails {
    /// 仅读过但未修改的文件.
    #[serde(default)]
    pub read_files: Vec<String>,
    /// 修改过的文件 (write 或 edit).
    #[serde(default)]
    pub modified_files: Vec<String>,
}

impl CompactionDetails {
    /// 新建空 details.
    pub fn new() -> Self {
        Self::default()
    }

    /// 从 `FileOps` 计算 read_files + modified_files (per pi.ts `computeFileLists`).
    pub fn from_file_ops(ops: &FileOps) -> Self {
        let modified: BTreeSet<String> = ops.edited.iter().chain(ops.written.iter()).cloned().collect();
        let read_only: Vec<String> = ops.read.difference(&modified).cloned().collect();
        let modified_files: Vec<String> = modified.into_iter().collect();
        Self {
            read_files: read_only,
            modified_files,
        }
    }

    /// 累积另一个 details (per pi.ts `extractFileOperations` 的 prevCompaction 分支).
    ///
    /// 用法: 每次新 compaction 时, 先合并上一次的 `details`, 再叠加本次消息新产生的 ops.
    pub fn accumulate(&self, ops: &FileOps) -> Self {
        let prev_modified: BTreeSet<String> = self.modified_files.iter().cloned().collect();
        let prev_read: BTreeSet<String> = self.read_files.iter().cloned().collect();

        let mut all_read = prev_read;
        all_read.extend(ops.read.iter().cloned());

        let mut all_modified = prev_modified;
        all_modified.extend(ops.written.iter().cloned());
        all_modified.extend(ops.edited.iter().cloned());

        let read_only: Vec<String> = all_read.difference(&all_modified).cloned().collect();
        let modified_files: Vec<String> = all_modified.into_iter().collect();
        Self {
            read_files: read_only,
            modified_files,
        }
    }
}

/// 从 assistant 消息的 tool calls 提取文件操作 (per pi.ts `extractFileOpsFromMessage` switch).
///
/// 仅识别 `read` / `write` / `edit` 三种 tool (其他 tool 不算文件操作, per pi convention).
/// `arguments.path` 必须是字符串; 否则忽略 (per pi `if (!path) continue`).
pub fn extract_file_ops_from_message(msg: &CompactionMessage) -> FileOps {
    let mut ops = FileOps::new();
    if msg.role != MessageRole::Assistant {
        return ops;
    }
    for tc in &msg.tool_calls {
        let path = tc
            .arguments
            .as_object()
            .and_then(|m| m.get("path"))
            .and_then(|v| v.as_str());
        let Some(path) = path else { continue };
        match tc.name.as_str() {
            "read" => {
                ops.read.insert(path.to_string());
            }
            "write" => {
                ops.written.insert(path.to_string());
            }
            "edit" => {
                ops.edited.insert(path.to_string());
            }
            _ => {}
        }
    }
    ops
}

/// 从消息列表提取所有文件操作 (累加器).
///
/// **用法**: 配合 `CompactionDetails::accumulate` + 上一次的 `prev.details`, 实现
/// 跨 compaction 的累积追踪 (per pi.ts `extractFileOperations`).
pub fn extract_file_ops(messages: &[CompactionMessage]) -> FileOps {
    let mut ops = FileOps::new();
    for msg in messages {
        ops.merge(&extract_file_ops_from_message(msg));
    }
    ops
}

// =====================================================================
// §3 Cut-point 算法 (per pi.ts `prepareCompaction` + `findCutPoint`)
// =====================================================================

/// **CutPoint** -- compaction 切割点
///
/// per pi.ts: 找出 user 消息作为 turn 边界, 保留最后 N 个 turn 之后的最近消息.
/// `is_split_turn` 表示 cut 点正好在一个 turn 中间 (把当前 turn 的前半也归入 summary).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CutPoint {
    /// 第一个保留消息在 `messages` 里的 idx (`[first_kept_idx..]` 都保留).
    pub first_kept_idx: usize,
    /// 当前 turn 的起始 idx (user 消息 idx). 仅 `is_split_turn=true` 时有值.
    pub turn_start_idx: usize,
    /// 切割点是否把当前 turn 一分为二 (user 消息之后还有 assistant/tool, 都保留).
    pub is_split_turn: bool,
}

impl CutPoint {
    /// 第一个保留消息的 UUID (per pi `firstKeptEntryId`).
    pub fn first_kept_id(&self, messages: &[CompactionMessage]) -> Option<Uuid> {
        messages.get(self.first_kept_idx).map(|m| m.id)
    }

    /// 需要摘要的消息范围 `[..first_kept_idx]` 或 `[..turn_start_idx]` (split 时).
    pub fn summarize_range(&self) -> std::ops::Range<usize> {
        if self.is_split_turn {
            0..self.turn_start_idx
        } else {
            0..self.first_kept_idx
        }
    }

    /// Turn-prefix 摘要范围 (split 时, 描述当前 turn 的前半).
    pub fn turn_prefix_range(&self) -> std::ops::Range<usize> {
        if self.is_split_turn {
            self.turn_start_idx..self.first_kept_idx
        } else {
            self.turn_start_idx..self.turn_start_idx // empty
        }
    }
}

/// **找出 cut-point** (per pi.ts `findCutPoint`).
///
/// 策略:
/// 1. 从尾部向前数 N 个 user 消息 (`keep_last_n_turns`)
/// 2. cut 点 = 第 (N-th-from-last) 个 user 消息的 idx
/// 3. 若 cut 点不是最后一条消息 (说明当前 turn 中间), 标 `is_split_turn=true`,
///    turn-prefix 范围 = cut 之后的当前 turn 前半消息
///
/// **返回 `None`**: 消息数 <= N, 无需 compaction.
pub fn find_cut_point(messages: &[CompactionMessage], keep_last_n_turns: usize) -> Option<CutPoint> {
    if keep_last_n_turns == 0 {
        return None;
    }
    // 收集所有 user 消息 idx (turn 边界)
    let user_idxs: Vec<usize> = messages
        .iter()
        .enumerate()
        .filter_map(|(i, m)| if m.is_user_turn() { Some(i) } else { None })
        .collect();
    if user_idxs.len() <= keep_last_n_turns {
        return None; // 消息数 <= N, 全部保留, 无需 summary
    }
    // 第 N-th-from-last 个 user 消息的 idx (cut 点)
    let cut_user_idx = user_idxs[user_idxs.len() - keep_last_n_turns];
    // 当前 turn 起始 (cut_user_idx 本身), turn-prefix 是 cut_user_idx 之后到 first_kept
    // pi 行为: first_kept = cut_user_idx (user 消息保留), turn_prefix = cut_user_idx 之后
    // 到 first_kept 之前. 但 cut_user_idx 本身 = first_kept 时, turn_prefix = empty.
    // 简化: 我们把 first_kept = cut_user_idx (user 消息保留), 不切 turn,
    // 这样 turn_prefix = empty, is_split_turn = false. 这跟 pi.ts 在
    // `findCutPointFirstKeptEntryId` 主路径一致 (默认不切 turn).
    Some(CutPoint {
        first_kept_idx: cut_user_idx,
        turn_start_idx: cut_user_idx,
        is_split_turn: false,
    })
}

// =====================================================================
// §4 摘要 (per pi.ts `generateSummary`; PI-7 v1.0 = 确定性 stub)
// =====================================================================

/// **Summary truncation limit** -- 摘要文本最大字符数 (per pi `TOOL_RESULT_MAX_CHARS` 同源).
///
/// PI-7 v1.0 用作确定性 stub 的截断阈值; v1.1 接 LLM 后由 LLM 决定.
pub const DEFAULT_SUMMARY_MAX_CHARS: usize = 2_000;

/// **确定性摘要 stub** (per PI-7 v1.0, 替代 LLM)
///
/// pi.ts `generateSummary` 走 LLM provider; PI-7 v1.0 不接 LLM (留待 PI-6 + ADR-0048 D42),
/// 走**截断 + marker** 的纯算法:
/// - 取每条消息的 `[Role]: content` 前 80 字符
/// - 累加直到 `max_chars` 上限
/// - 超长部分标 `<truncated>...</truncated>`
///
/// 未来 v1.1 接 domain-llm 时, 这个函数被替换为 async `generate_summary_with_usage(...)`;
/// 签名 + 返回类型保持兼容 (`String` + `CompactionUsage`), 减小调用方改动.
pub fn summarize_messages(messages: &[CompactionMessage], max_chars: usize) -> String {
    if messages.is_empty() {
        return String::new();
    }
    let max_chars = max_chars.max(80); // 至少 80 char
    let mut out = String::new();
    out.push_str(&format!("[Summary of {} messages]\n\n", messages.len()));
    for msg in messages {
        let role = msg.role.as_str();
        let preview = msg.content.chars().take(80).collect::<String>();
        let line = if msg.content.chars().count() > 80 {
            format!("[{}]: {}...\n", role, preview)
        } else {
            format!("[{}]: {}\n", role, preview)
        };
        if out.len() + line.len() > max_chars {
            out.push_str("\n<truncated>...</truncated>\n");
            break;
        }
        out.push_str(&line);
    }
    out
}

/// **Usage placeholder** -- 摘要 token 计量 (per pi.ts `Usage`).
///
/// PI-7 v1.0 stub 用 char count / 4 估算 tokens; v1.1 接 LLM 后用真实 usage.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionUsage {
    /// Input tokens (per pi `inputTokens`).
    pub input_tokens: u64,
    /// Output tokens (per pi `outputTokens`).
    pub output_tokens: u64,
}

impl CompactionUsage {
    /// 估算 tokens (char / 4, per pi.ts `Math.ceil(chars / 4)`).
    pub fn estimate(chars: usize) -> Self {
        Self {
            input_tokens: chars.div_ceil(4) as u64,
            output_tokens: 0,
        }
    }
}

/// **Token 估算** (per pi.ts `estimateTokens` 简化版).
///
/// PI-7 v1.0 = char count / 4; v1.1 接 tokenizer crate (tiktoken-rs).
pub fn estimate_tokens(messages: &[CompactionMessage]) -> u64 {
    let total_chars: usize = messages.iter().map(|m| m.content.len()).sum();
    total_chars.div_ceil(4) as u64
}

// =====================================================================
// §5 CompactionEntry (持久化模型)
// =====================================================================

/// **CompactionKind** -- compaction 来源 (per pi `fromHook` 字段同源).
///
/// PI-7 v1.0 仅 `Auto` / `Manual`; v1.2 加 `BranchSummary` (per PI-8 worktree fork).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompactionKind {
    /// 自动触发 (token budget 超阈值 / turn count 超阈值).
    Auto,
    /// 手动触发 (用户/上层调用方显式调用).
    Manual,
}

impl CompactionKind {
    /// Stable string form.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Manual => "manual",
        }
    }

    /// 从 SQLite 字符串反序列化.
    fn parse(s: &str) -> Self {
        match s {
            "Manual" => Self::Manual,
            _ => Self::Auto,
        }
    }
}

/// **CompactionEntry** -- 一次 compaction 的不可变快照
///
/// **W/T/M 分类**: T (Transaction, append-only, per 守门 #13 + 9/1 18:30 JST 拍板).
/// 物理删除禁止, 仅 `cleanup_compaction_entries(before_ms)` 后台 TTL 可清理.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionEntry {
    /// Entry UUID v4.
    pub entry_id: Uuid,
    /// Session/Task 关联 ID (grouping key; 暂为 String 兼容 PI-6 star-dto 未 land).
    pub session_id: String,
    /// 第一个保留消息的 ID (per pi `firstKeptEntryId`); None 表示保留全部.
    pub first_kept_message_id: Option<Uuid>,
    /// 摘要文本.
    pub summary: String,
    /// Compaction 前的 token 估算 (per pi `tokensBefore`).
    pub tokens_before: u64,
    /// 文件操作累积追踪 (per pi `details: CompactionDetails`).
    pub details: CompactionDetails,
    /// 创建时间戳 (ms since epoch).
    pub created_at_ms: u64,
    /// Compaction 类型 (Auto / Manual).
    pub kind: CompactionKind,
    /// 摘要生成的 token 计量 (per pi `usage`).
    #[serde(default)]
    pub usage: CompactionUsage,
}

// =====================================================================
// §6 Compactor 算法入口
// =====================================================================

/// **CompactionDecision** -- `prepare` 阶段的中间结果 (per pi.ts `CompactionPreparation`).
///
/// `compact` 阶段消费 `CompactionDecision` 生成 `CompactionEntry`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionDecision {
    /// Cut-point.
    pub cut_point: CutPoint,
    /// 第一个保留消息的 UUID (per pi `firstKeptEntryId`).
    pub first_kept_message_id: Option<Uuid>,
    /// Compaction 前的 token 估算 (per pi `tokensBefore`).
    pub tokens_before: u64,
    /// 文件操作累积追踪 (per pi `fileOps`).
    pub file_ops: FileOps,
}

/// **Compactor** -- compaction 算法配置 + 入口
///
/// v1.0: 同步纯算法; v1.1 接 LLM 后 `compact` 变 async + 接受 provider pool 注入.
#[derive(Debug, Clone)]
pub struct Compactor {
    /// 保留最近 N 个 user turn (per pi `settings.keepLastN` 默认 3).
    pub keep_last_n_turns: usize,
    /// 摘要文本最大字符数.
    pub summary_max_chars: usize,
}

impl Compactor {
    /// 默认配置 (keep_last=3, summary=2000 chars, per pi 默认值).
    pub fn new(keep_last_n_turns: usize) -> Self {
        Self {
            keep_last_n_turns,
            summary_max_chars: DEFAULT_SUMMARY_MAX_CHARS,
        }
    }

    /// 自定义 summary max chars.
    pub fn with_summary_max_chars(mut self, max: usize) -> Self {
        self.summary_max_chars = max;
        self
    }

    /// **prepare 阶段** (per pi.ts `prepareCompaction`):
    /// 1. 找 cut-point
    /// 2. 累积文件操作 (prev.details + messages 的新 ops)
    /// 3. 返回 `CompactionDecision`
    ///
    /// `None` 表示无需 compaction (消息数 <= keep_last_n_turns).
    pub fn prepare(
        &self,
        messages: &[CompactionMessage],
        prev: Option<&CompactionEntry>,
    ) -> Option<CompactionDecision> {
        let cut_point = find_cut_point(messages, self.keep_last_n_turns)?;
        let new_ops = extract_file_ops(&messages[cut_point.summarize_range()]);
        let file_ops = match prev {
            Some(p) => {
                let prev_ops = FileOps {
                    read: p.details.read_files.iter().cloned().collect(),
                    written: BTreeSet::new(),
                    edited: p.details.modified_files.iter().cloned().collect(),
                };
                let mut merged = prev_ops;
                merged.merge(&new_ops);
                merged
            }
            None => new_ops,
        };
        Some(CompactionDecision {
            first_kept_message_id: cut_point.first_kept_id(messages),
            tokens_before: estimate_tokens(messages),
            cut_point,
            file_ops,
        })
    }

    /// **compact 阶段** (per pi.ts `compact`):
    /// 1. `prepare` 拿 decision
    /// 2. summarize 待摘要消息 (v1.0 stub)
    /// 3. 计算 CompactionDetails
    /// 4. 构造 CompactionEntry
    ///
    /// **v1.0 LLM 缺口**: 摘要走确定性 stub, v1.1 接 domain-llm provider pool (per ADR-0048 D42).
    pub fn compact(
        &self,
        messages: &[CompactionMessage],
        session_id: impl Into<String>,
        prev: Option<&CompactionEntry>,
        kind: CompactionKind,
    ) -> Option<CompactionEntry> {
        let decision = self.prepare(messages, prev)?;
        let to_summarize = &messages[decision.cut_point.summarize_range()];
        let summary = summarize_messages(to_summarize, self.summary_max_chars);
        let usage = CompactionUsage::estimate(summary.len());
        let details = CompactionDetails::from_file_ops(&decision.file_ops);
        Some(CompactionEntry {
            entry_id: Uuid::new_v4(),
            session_id: session_id.into(),
            first_kept_message_id: decision.first_kept_message_id,
            summary,
            tokens_before: decision.tokens_before,
            details,
            created_at_ms: now_ms(),
            kind,
            usage,
        })
    }
}

// =====================================================================
// §7 SQLite 持久化 (per SRS §8 缺口 #7: 单独落表, 不动 task_queue schema)
// =====================================================================

/// **CompactionEntry 持久化错误** (per `TaskQueueError` 模式同源).
#[derive(Debug, Error)]
pub enum CompactionError {
    /// Entry 未找到.
    #[error("compaction entry {0} not found")]
    EntryNotFound(Uuid),
    /// SQLite 错误.
    #[error("sqlite: {0}")]
    Sqlite(String),
    /// JSON 序列化/反序列化失败.
    #[error("json: {0}")]
    Json(String),
}

impl From<rusqlite::Error> for CompactionError {
    fn from(e: rusqlite::Error) -> Self {
        CompactionError::Sqlite(e.to_string())
    }
}

impl From<serde_json::Error> for CompactionError {
    fn from(e: serde_json::Error) -> Self {
        CompactionError::Json(e.to_string())
    }
}

/// **CompactionStore trait** -- 抽象存储 (便于 v1.1 替换 PG / remote backend).
#[async_trait]
pub trait CompactionStore: Send + Sync {
    /// 插入一条 compaction entry.
    async fn insert(&self, entry: &CompactionEntry) -> Result<(), CompactionError>;
    /// 按 entry_id 查询.
    async fn get(&self, entry_id: Uuid) -> Result<CompactionEntry, CompactionError>;
    /// 列出指定 session 的所有 entries (按 created_at_ms 升序).
    async fn list_by_session(
        &self,
        session_id: &str,
        limit: u32,
    ) -> Result<Vec<CompactionEntry>, CompactionError>;
    /// 列出指定 session 的最新一条 entry.
    async fn latest_for_session(
        &self,
        session_id: &str,
    ) -> Result<Option<CompactionEntry>, CompactionError>;
    /// 总条目数.
    async fn count(&self) -> Result<u64, CompactionError>;
    /// 删除 created_at_ms < before_ms 的 entries (per 守门 #13 c 派生 TTL).
    async fn cleanup_before(&self, before_ms: u64) -> Result<u64, CompactionError>;
}

/// **SQLite 持久化实现** (per `SqliteTaskQueue` 同源模式).
///
/// 复用 `SqliteTaskQueue::inner` 的 `Connection` 不必要 (compaction 表独立),
/// 这里独立打开 `compaction.db` (跟 `task_queue.db` 物理分离), 跟 `task_queue`
/// schema 完全隔离 (per SRS §8 缺口 #7: 单独落表).
///
/// **生产可考虑**: 跟 `task_queue` 共用同一 SQLite 文件, 走 `ATTACH DATABASE`;
/// PI-7 v1.0 保持独立文件, 简化 W/T/M 横展開.
pub struct SqliteCompactionStore {
    inner: std::sync::Arc<Mutex<Connection>>,
}

impl SqliteCompactionStore {
    /// 新建独立 SQLite 文件 (WAL mode, sync).
    pub fn new(path: &str) -> Result<Self, CompactionError> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        init_compaction_schema(&conn)?;
        Ok(Self {
            inner: std::sync::Arc::new(Mutex::new(conn)),
        })
    }

    /// 内存 SQLite (per unit test).
    pub fn in_memory() -> Result<Self, CompactionError> {
        Self::new(":memory:")
    }
}

/// **初始化 schema** (idempotent, per `init_schema` 同源模式).
///
/// **W/T/M 分类 (per 守门 #13)**: T (Transaction, append-only).
/// - 物理删除禁止 (仅 `cleanup_before` 后台 TTL 可清)
/// - 审计必填: entry_id / session_id / created_at_ms / kind / details 全部 NOT NULL
/// - `first_kept_message_id` 可空 (保留全部消息时)
pub(crate) fn init_compaction_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS compaction_entry (
            entry_id              TEXT PRIMARY KEY NOT NULL,
            session_id            TEXT NOT NULL,
            first_kept_message_id TEXT,
            summary               TEXT NOT NULL,
            tokens_before         INTEGER NOT NULL,
            read_files            TEXT NOT NULL DEFAULT '[]',
            modified_files        TEXT NOT NULL DEFAULT '[]',
            kind                  TEXT NOT NULL,
            created_at_ms         INTEGER NOT NULL,
            input_tokens          INTEGER NOT NULL DEFAULT 0,
            output_tokens         INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_compaction_session_created
            ON compaction_entry(session_id, created_at_ms);
        "#,
    )?;
    Ok(())
}

fn insert_entry(conn: &Connection, entry: &CompactionEntry) -> rusqlite::Result<()> {
    let read_files = serde_json::to_string(&entry.details.read_files)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    let modified_files = serde_json::to_string(&entry.details.modified_files)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    conn.execute(
        r#"INSERT INTO compaction_entry
           (entry_id, session_id, first_kept_message_id, summary, tokens_before,
            read_files, modified_files, kind, created_at_ms, input_tokens, output_tokens)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
        params![
            entry.entry_id.to_string(),
            entry.session_id,
            entry.first_kept_message_id.map(|u| u.to_string()),
            entry.summary,
            entry.tokens_before as i64,
            read_files,
            modified_files,
            format!("{:?}", entry.kind),
            entry.created_at_ms as i64,
            entry.usage.input_tokens as i64,
            entry.usage.output_tokens as i64,
        ],
    )?;
    Ok(())
}

fn row_to_entry(r: &rusqlite::Row<'_>) -> rusqlite::Result<CompactionEntry> {
    let entry_id_str: String = r.get(0)?;
    let session_id: String = r.get(1)?;
    let first_kept_str: Option<String> = r.get(2)?;
    let summary: String = r.get(3)?;
    let tokens_before: i64 = r.get(4)?;
    let read_files: String = r.get(5)?;
    let modified_files: String = r.get(6)?;
    let kind_str: String = r.get(7)?;
    let created_at_ms: i64 = r.get(8)?;
    let input_tokens: i64 = r.get(9)?;
    let output_tokens: i64 = r.get(10)?;

    Ok(CompactionEntry {
        entry_id: Uuid::parse_str(&entry_id_str).unwrap_or(Uuid::nil()),
        session_id,
        first_kept_message_id: first_kept_str.and_then(|s| Uuid::parse_str(&s).ok()),
        summary,
        tokens_before: tokens_before as u64,
        details: CompactionDetails {
            read_files: serde_json::from_str(&read_files).unwrap_or_default(),
            modified_files: serde_json::from_str(&modified_files).unwrap_or_default(),
        },
        created_at_ms: created_at_ms as u64,
        kind: CompactionKind::parse(&kind_str),
        usage: CompactionUsage {
            input_tokens: input_tokens as u64,
            output_tokens: output_tokens as u64,
        },
    })
}

#[allow(dead_code)] // 可选 helper; 当前 `get_entry_optional` 直接用 `query_row(...).optional()`
fn get_entry(conn: &Connection, entry_id: Uuid) -> rusqlite::Result<CompactionEntry> {
    let mut stmt = conn.prepare(
        r#"SELECT entry_id, session_id, first_kept_message_id, summary, tokens_before,
                  read_files, modified_files, kind, created_at_ms, input_tokens, output_tokens
           FROM compaction_entry WHERE entry_id = ?1"#,
    )?;
    stmt.query_row(params![entry_id.to_string()], row_to_entry)
}

/// `get_entry` 的 Option 变体 (per `OptionalExtension`), 用于 `Store::get` 区分 not-found.
fn get_entry_optional(
    conn: &Connection,
    entry_id: Uuid,
) -> rusqlite::Result<Option<CompactionEntry>> {
    let mut stmt = conn.prepare(
        r#"SELECT entry_id, session_id, first_kept_message_id, summary, tokens_before,
                  read_files, modified_files, kind, created_at_ms, input_tokens, output_tokens
           FROM compaction_entry WHERE entry_id = ?1"#,
    )?;
    stmt.query_row(params![entry_id.to_string()], row_to_entry)
        .optional()
}

fn list_entries_by_session(
    conn: &Connection,
    session_id: &str,
    limit: u32,
) -> rusqlite::Result<Vec<CompactionEntry>> {
    let mut stmt = conn.prepare(
        r#"SELECT entry_id, session_id, first_kept_message_id, summary, tokens_before,
                  read_files, modified_files, kind, created_at_ms, input_tokens, output_tokens
           FROM compaction_entry WHERE session_id = ?1 ORDER BY created_at_ms ASC LIMIT ?2"#,
    )?;
    let rows = stmt.query_map(params![session_id, limit as i64], row_to_entry)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

fn latest_entry_for_session(conn: &Connection, session_id: &str) -> rusqlite::Result<Option<CompactionEntry>> {
    let mut stmt = conn.prepare(
        r#"SELECT entry_id, session_id, first_kept_message_id, summary, tokens_before,
                  read_files, modified_files, kind, created_at_ms, input_tokens, output_tokens
           FROM compaction_entry WHERE session_id = ?1
           ORDER BY created_at_ms DESC LIMIT 1"#,
    )?;
    stmt.query_row(params![session_id], row_to_entry).optional()
}

fn count_all_entries(conn: &Connection) -> rusqlite::Result<u64> {
    let n: i64 = conn.query_row(r#"SELECT COUNT(*) FROM compaction_entry"#, [], |r| r.get(0))?;
    Ok(n as u64)
}

fn cleanup_before(conn: &Connection, before_ms: u64) -> rusqlite::Result<u64> {
    // T 类: 物理删除禁止, 仅后台 TTL 策略可清 (per 守门 #13 c 派生).
    // 调用方必须明确传入 before_ms (而不是直接 delete).
    let n = conn.execute(
        r#"DELETE FROM compaction_entry WHERE created_at_ms < ?1"#,
        params![before_ms as i64],
    )?;
    let _ = conn.pragma_update(None, "wal_checkpoint", "PASSIVE");
    Ok(n as u64)
}

#[async_trait]
impl CompactionStore for SqliteCompactionStore {
    async fn insert(&self, entry: &CompactionEntry) -> Result<(), CompactionError> {
        let conn = self.inner.clone();
        let entry = entry.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            insert_entry(&conn, &entry)
        })
        .await
        .map_err(|e| CompactionError::Sqlite(format!("join: {e}")))?
        .map_err(|e: rusqlite::Error| CompactionError::Sqlite(e.to_string()))?;
        Ok(())
    }

    async fn get(&self, entry_id: Uuid) -> Result<CompactionEntry, CompactionError> {
        let conn = self.inner.clone();
        let entry = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            get_entry_optional(&conn, entry_id)
        })
        .await
        .map_err(|e| CompactionError::Sqlite(format!("join: {e}")))?
        .map_err(|e: rusqlite::Error| CompactionError::Sqlite(e.to_string()))?;
        entry.ok_or(CompactionError::EntryNotFound(entry_id))
    }

    async fn list_by_session(
        &self,
        session_id: &str,
        limit: u32,
    ) -> Result<Vec<CompactionEntry>, CompactionError> {
        let conn = self.inner.clone();
        let session_id = session_id.to_string();
        let entries = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            list_entries_by_session(&conn, &session_id, limit)
        })
        .await
        .map_err(|e| CompactionError::Sqlite(format!("join: {e}")))?
        .map_err(|e: rusqlite::Error| CompactionError::Sqlite(e.to_string()))?;
        Ok(entries)
    }

    async fn latest_for_session(
        &self,
        session_id: &str,
    ) -> Result<Option<CompactionEntry>, CompactionError> {
        let conn = self.inner.clone();
        let session_id = session_id.to_string();
        let entry = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            latest_entry_for_session(&conn, &session_id)
        })
        .await
        .map_err(|e| CompactionError::Sqlite(format!("join: {e}")))?
        .map_err(|e: rusqlite::Error| CompactionError::Sqlite(e.to_string()))?;
        Ok(entry)
    }

    async fn count(&self) -> Result<u64, CompactionError> {
        let conn = self.inner.clone();
        let n = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            count_all_entries(&conn)
        })
        .await
        .map_err(|e| CompactionError::Sqlite(format!("join: {e}")))?
        .map_err(|e: rusqlite::Error| CompactionError::Sqlite(e.to_string()))?;
        Ok(n)
    }

    async fn cleanup_before(&self, before_ms: u64) -> Result<u64, CompactionError> {
        let conn = self.inner.clone();
        let n = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            cleanup_before(&conn, before_ms)
        })
        .await
        .map_err(|e| CompactionError::Sqlite(format!("join: {e}")))?
        .map_err(|e: rusqlite::Error| CompactionError::Sqlite(e.to_string()))?;
        Ok(n)
    }
}

// =====================================================================
// §8 now_ms (跟 `sqlite_backend` 共享, 保持 timestamp 一致性)
// =====================================================================

// `now_ms` 从 super 导入; 此处不重复定义 (per `use super::now_ms`).

// `TaskQueueError` 通过 super 提供 (顶部 `use super::TaskQueueError as _`), 保留给 v1.1 cross-store error 统一时使用.

// =====================================================================
// §9 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn tool_call(name: &str, path: &str) -> ToolCall {
        ToolCall {
            name: name.into(),
            arguments: serde_json::json!({ "path": path }),
        }
    }

    fn make_messages() -> Vec<CompactionMessage> {
        // 5 个 user turn, 每个 turn 配 1 个 assistant 调用 read/edit/write
        vec![
            CompactionMessage::user(Uuid::new_v4(), "u1"),
            CompactionMessage::assistant(
                Uuid::new_v4(),
                "a1",
                vec![tool_call("read", "src/foo.rs")],
            ),
            CompactionMessage::user(Uuid::new_v4(), "u2"),
            CompactionMessage::assistant(
                Uuid::new_v4(),
                "a2",
                vec![tool_call("edit", "src/foo.rs"), tool_call("read", "src/bar.rs")],
            ),
            CompactionMessage::user(Uuid::new_v4(), "u3"),
            CompactionMessage::assistant(
                Uuid::new_v4(),
                "a3",
                vec![tool_call("write", "src/baz.rs")],
            ),
            CompactionMessage::user(Uuid::new_v4(), "u4"),
            CompactionMessage::assistant(
                Uuid::new_v4(),
                "a4",
                vec![tool_call("read", "src/foo.rs"), tool_call("edit", "src/qux.rs")],
            ),
            CompactionMessage::user(Uuid::new_v4(), "u5"),
            CompactionMessage::assistant(Uuid::new_v4(), "a5", vec![]),
        ]
    }

    #[test]
    fn extract_file_ops_from_assistant_read_write_edit() {
        let msgs = make_messages();
        let ops = extract_file_ops(&msgs);
        assert_eq!(ops.read, BTreeSet::from(["src/foo.rs".into(), "src/bar.rs".into()]));
        assert_eq!(ops.edited, BTreeSet::from(["src/foo.rs".into(), "src/qux.rs".into()]));
        assert_eq!(ops.written, BTreeSet::from(["src/baz.rs".into()]));
    }

    #[test]
    fn compute_read_only_vs_modified() {
        let msgs = make_messages();
        let ops = extract_file_ops(&msgs);
        let details = CompactionDetails::from_file_ops(&ops);
        // modified = edited ∪ written = {foo, qux, baz}
        assert_eq!(
            details.modified_files,
            vec!["src/baz.rs".to_string(), "src/foo.rs".to_string(), "src/qux.rs".to_string()]
        );
        // read_only = read - modified = {bar}
        assert_eq!(details.read_files, vec!["src/bar.rs".to_string()]);
    }

    #[test]
    fn merge_file_ops_cumulative_across_compactions() {
        // 模拟跨 compaction 累积 (per pi.ts `extractFileOperations` 的 prevCompaction 分支)
        let prev_details = CompactionDetails {
            read_files: vec!["old.rs".into()],
            modified_files: vec!["prev_edit.rs".into()],
        };
        let mut prev_ops = FileOps {
            read: prev_details.read_files.iter().cloned().collect(),
            written: BTreeSet::new(),
            edited: prev_details.modified_files.iter().cloned().collect(),
        };
        // 本次 compaction: 新 read + write
        let new_ops = FileOps {
            read: BTreeSet::from(["new.rs".into()]),
            written: BTreeSet::from(["new_write.rs".into()]),
            edited: BTreeSet::new(),
        };
        prev_ops.merge(&new_ops);
        let merged = CompactionDetails::from_file_ops(&prev_ops);
        assert!(merged.read_files.contains(&"old.rs".to_string()));
        assert!(merged.read_files.contains(&"new.rs".to_string()));
        assert!(merged.modified_files.contains(&"prev_edit.rs".to_string()));
        assert!(merged.modified_files.contains(&"new_write.rs".to_string()));
    }

    #[test]
    fn find_cut_point_keeps_last_n_turns() {
        let msgs = make_messages(); // 5 turns
        // keep_last_n_turns=2 → cut 在第 3 个 user (idx=4, msgs[4]="u3")
        let cp = find_cut_point(&msgs, 2).expect("should find cut point");
        assert_eq!(cp.first_kept_idx, 6);
        assert_eq!(cp.first_kept_id(&msgs), Some(msgs[6].id));
        assert_eq!(cp.summarize_range(), 0..6);
        // keep_last_n_turns=5 → 全部保留, 无需 cut
        assert!(find_cut_point(&msgs, 5).is_none());
        // keep_last_n_turns=0 → 直接返回 None
        assert!(find_cut_point(&msgs, 0).is_none());
    }

    #[test]
    fn summarize_messages_truncates_with_marker() {
        let msgs: Vec<CompactionMessage> = (0..50)
            .map(|i| CompactionMessage::user(Uuid::new_v4(), format!("msg-{}", i)))
            .collect();
        let summary = summarize_messages(&msgs, 500);
        assert!(summary.starts_with("[Summary of 50 messages]"));
        assert!(summary.contains("<truncated>...</truncated>"));
    }

    #[test]
    fn compact_end_to_end_returns_entry() {
        let msgs = make_messages();
        let compactor = Compactor::new(2);
        let entry = compactor
            .compact(&msgs, "session-A", None, CompactionKind::Auto)
            .expect("should compact");
        assert_eq!(entry.session_id, "session-A");
        assert_eq!(entry.kind, CompactionKind::Auto);
        // 摘要了前 4 条消息
        assert!(entry.summary.contains("u1"));
        assert!(entry.summary.contains("u2"));
        // modified_files 应含 foo + baz (a4/qux.rs 在保留区, 不进 details)
        assert!(entry.details.modified_files.contains(&"src/foo.rs".to_string()));
        assert!(entry.details.modified_files.contains(&"src/baz.rs".to_string()));
        assert!(!entry.details.modified_files.contains(&"src/qux.rs".to_string()));
        assert!(!entry.details.modified_files.contains(&"src/bar.rs".to_string()));
        // read_files 应只含 bar (foo 在 modified, 不在 read_only)
        assert_eq!(entry.details.read_files, vec!["src/bar.rs".to_string()]);
        // first_kept_message_id = msgs[6].id (u4)
        assert_eq!(entry.first_kept_message_id, Some(msgs[6].id));
    }

    #[test]
    fn compact_cumulative_with_prev() {
        // 第二次 compaction: 累积上一次的文件操作 (per pi.ts `extractFileOperations` prevCompaction 分支)
        let msgs = make_messages();
        let compactor_first = Compactor::new(4);
        let compactor_second = Compactor::new(2);

        // 第一次 compaction (keep=4) → cut at user_idxs[1]=2 (u2); summarize [0..2] = [u1, a1]
        // file_ops = {read:{foo}}; details.modified_files=[], details.read_files=[foo]
        let first = compactor_first
            .compact(&msgs, "session-B", None, CompactionKind::Auto)
            .expect("first compact should produce entry");
        assert_eq!(first.details.read_files, vec!["src/foo.rs".to_string()]);
        assert!(first.details.modified_files.is_empty());

        // 第二次 compaction (keep=2) → cut at user_idxs[3]=6 (u4); summarize [0..6]
        //   new_ops from [0..6] = {read:{foo,bar}, edited:{foo}, written:{baz}}
        //   prev_ops from first = {read:{foo}, edited:empty, written:empty}
        //   merged = {read:{foo,bar}, edited:{foo}, written:{baz}}
        //   details.modified_files = [foo, baz] (edited union written), read_only = [bar]
        let second = compactor_second
            .compact(&msgs, "session-B", Some(&first), CompactionKind::Auto)
            .expect("second compact should produce entry");
        assert_eq!(second.details.modified_files.len(), 2);
        assert!(second.details.modified_files.contains(&"src/foo.rs".to_string()));
        assert!(second.details.modified_files.contains(&"src/baz.rs".to_string()));
        // read_only 只剩 bar (foo 被标记为 modified)
        assert_eq!(second.details.read_files, vec!["src/bar.rs".to_string()]);
    }

    #[tokio::test]
    async fn compaction_persists_across_reopen() {
        let dir = std::env::temp_dir().join(format!("star-compaction-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("compaction.db");
        let path_str = db_path.to_str().unwrap();

        // 1) 写入 1 条 entry
        let msgs = make_messages();
        let compactor = Compactor::new(2);
        // cut at msgs[6]=u4; summarize [0..6] → modified=[foo,baz], read_only=[bar]
        let entry = compactor
            .compact(&msgs, "session-X", None, CompactionKind::Auto)
            .unwrap();
        assert_eq!(entry.details.modified_files.len(), 2);
        assert_eq!(entry.details.read_files, vec!["src/bar.rs".to_string()]);
        {
            let store = SqliteCompactionStore::new(path_str).unwrap();
            store.insert(&entry).await.unwrap();
            assert_eq!(store.count().await.unwrap(), 1);
        }

        // 2) 重连 → 数据还在 (WAL persistence)
        {
            let store = SqliteCompactionStore::new(path_str).unwrap();
            assert_eq!(store.count().await.unwrap(), 1);
            let latest = store
                .latest_for_session("session-X")
                .await
                .unwrap()
                .expect("should have 1 entry");
            assert_eq!(latest.entry_id, entry.entry_id);
            assert_eq!(latest.details.modified_files.len(), 2);
            assert_eq!(latest.details.read_files, vec!["src/bar.rs".to_string()]);
        }

        let _ = std::fs::remove_dir_all(&dir);
    }
}
