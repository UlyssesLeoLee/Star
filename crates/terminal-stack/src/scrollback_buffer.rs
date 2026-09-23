//! `scrollback_buffer.rs` — Scrollback Ring Buffer (per FR-ORCA-036 §12 v1.0, ULYS-200)
//!
//! **目的**: 终端 scrollback 在 client 重启 / daemon 重启后必须能恢复 (per §12 "terminal
//! scrollback that survives restarts").
//!
//! **MVP v0 范围 (本 issue ULYS-200)**:
//! - `ScrollbackBuffer` 内存 ring buffer (固定容量, append-only)
//! - `ScrollbackLine` 实体 (timestamp + text + 来源 marker)
//! - 序列化 (JSON round-trip, 用于持久化)
//! - Append + 容量溢出 drop-oldest 策略 (per scrollback 语义, 旧行优先丢)
//! - Iter API (reader 端按时间顺序拉)
//! - 8+ 单元测试
//!
//! **不在本 MVP 范围 (P1 followup)**:
//! - SQLite WAL 持久化层 (per `cli_session_registry` 模式扩展)
//! - 与 `cli_session` schema 字段 `scrollback_bytes` 集成
//! - Restart recovery (从 disk replay 到 in-memory)
//! - ANSI escape sequence 解析 (per FR-ORCA-039 TUI Transcript Capture)
//! - Terminal Splits (per spec §12 附加需求)
//! - WebSocket 协议层 (per `crates/<terminal-crate>/` 未来实装)
//!
//! 守门:
//! - #1 v15 cargo test 单 crate 实证
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =====================================================================
// 1. entity
// =====================================================================

/// Scrollback 来源标记 (per FR-ORCA-039 TUI Transcript Capture 派生)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ScrollbackSource {
    /// Process stdout (per default capture)
    Stdout,
    /// Process stderr (per `2>&1` capture)
    Stderr,
    /// Agent 自注入 (per FR-ORCA-035 annotation 渲染)
    AgentInjected,
}

/// 单行 scrollback (per FR-ORCA-036 "terminal pane 的 scrollback buffer")
///
/// MVP v0: 单行 = 1 行 text (换行符已被 strip).
/// Future P1: 含 ANSI escape sequence 字节级还原 (per FR-ORCA-039).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScrollbackLine {
    /// 行 UUID (用于 dedup / diff)
    pub id: Uuid,
    /// 写入时间
    pub timestamp: DateTime<Utc>,
    /// 行文本 (per default 1 行, 换行已 strip)
    pub text: String,
    /// 来源
    pub source: ScrollbackSource,
    /// 字节数 (per `cli_session.scrollback_bytes` 字段兼容, MVP v0 算一次)
    pub byte_len: u32,
}

// =====================================================================
// 2. ring buffer
// =====================================================================

/// Scrollback Ring Buffer (per FR-ORCA-036 容量约束)
///
/// **MVP v0 行为**:
/// - 构造时定容量 (per `ScrollbackBuffer::new(capacity_lines)`)
/// - append: 追加到末尾, 满则 drop-oldest (per scrollback 语义)
/// - 容量以 **行** 计算 (不是字节); byte limit 留 P1 (per scrollback_bytes 兼容)
#[derive(Debug, Clone)]
pub struct ScrollbackBuffer {
    /// 行存储 (FIFO, append-only, drop-oldest 满)
    lines: Vec<ScrollbackLine>,
    /// 容量 (行数)
    capacity_lines: usize,
    /// 总字节数 (per `cli_session.scrollback_bytes` 兼容)
    total_bytes: u64,
}

impl ScrollbackBuffer {
    /// 构造 ring buffer (per 容量约束)
    pub fn new(capacity_lines: usize) -> Self {
        Self {
            lines: Vec::with_capacity(capacity_lines),
            capacity_lines,
            total_bytes: 0,
        }
    }

    /// 当前行数
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// 容量 (行)
    pub fn capacity(&self) -> usize {
        self.capacity_lines
    }

    /// 累积字节数 (per `cli_session.scrollback_bytes` 兼容)
    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    /// 追加一行 (per FR-ORCA-036 append + drop-oldest)
    ///
    /// Returns:
    /// - `Ok(dropped_count)` = 被丢弃的旧行数 (容量满时 > 0)
    /// - `Err(ScrollbackError::EmptyText)` = 拒绝空行 (per invariant)
    pub fn append(
        &mut self,
        text: impl Into<String>,
        source: ScrollbackSource,
    ) -> Result<usize, ScrollbackError> {
        let text = text.into();
        if text.is_empty() {
            return Err(ScrollbackError::EmptyText);
        }
        let byte_len = text.len() as u32;
        let line = ScrollbackLine {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            text,
            source,
            byte_len,
        };
        self.lines.push(line);
        self.total_bytes += byte_len as u64;

        let mut dropped = 0;
        while self.lines.len() > self.capacity_lines {
            let old = self.lines.remove(0);
            self.total_bytes = self.total_bytes.saturating_sub(old.byte_len as u64);
            dropped += 1;
        }
        Ok(dropped)
    }

    /// 追加带显式 timestamp (用于持久化 replay 时保 timestamp)
    ///
    /// **MVP v0**: 不强制覆盖 caller 传入 timestamp, 用于 replay 场景.
    pub fn append_with_timestamp(
        &mut self,
        text: impl Into<String>,
        source: ScrollbackSource,
        timestamp: DateTime<Utc>,
    ) -> Result<usize, ScrollbackError> {
        let text = text.into();
        if text.is_empty() {
            return Err(ScrollbackError::EmptyText);
        }
        let byte_len = text.len() as u32;
        let line = ScrollbackLine {
            id: Uuid::new_v4(),
            timestamp,
            text,
            source,
            byte_len,
        };
        self.lines.push(line);
        self.total_bytes += byte_len as u64;

        let mut dropped = 0;
        while self.lines.len() > self.capacity_lines {
            let old = self.lines.remove(0);
            self.total_bytes = self.total_bytes.saturating_sub(old.byte_len as u64);
            dropped += 1;
        }
        Ok(dropped)
    }

    /// 全部行 (per reader 端按时间顺序拉)
    pub fn lines(&self) -> &[ScrollbackLine] {
        &self.lines
    }

    /// 最新 N 行 (per UI "scroll to bottom" 渲染)
    pub fn tail(&self, n: usize) -> &[ScrollbackLine] {
        let start = self.lines.len().saturating_sub(n);
        &self.lines[start..]
    }

    /// 按时间范围过滤 (per restart recovery "from last_seq 之后 replay")
    pub fn lines_since(&self, since: DateTime<Utc>) -> Vec<&ScrollbackLine> {
        self.lines.iter().filter(|l| l.timestamp >= since).collect()
    }

    /// 清空 (per dismissal / NFR-ORCA-004 "certified PTY exit")
    pub fn clear(&mut self) {
        self.lines.clear();
        self.total_bytes = 0;
    }

    /// JSON 序列化 (per 持久化层落地后接 WAL/sqlite)
    ///
    /// MVP v0 提供 in-memory + JSON round-trip, P1 接 cli_session_registry 的 SQLite WAL 模式.
    pub fn to_json(&self) -> Result<String, ScrollbackError> {
        serde_json::to_string(&self.lines).map_err(ScrollbackError::Serialize)
    }

    /// JSON 反序列化 (per restart recovery)
    pub fn from_json(json: &str, capacity_lines: usize) -> Result<Self, ScrollbackError> {
        let lines: Vec<ScrollbackLine> =
            serde_json::from_str(json).map_err(ScrollbackError::Deserialize)?;
        let mut buf = Self::new(capacity_lines);
        for line in lines {
            // 用 append_with_timestamp 保留原 timestamp, 跳过空文本
            if line.text.is_empty() {
                continue;
            }
            let byte_len = line.byte_len;
            buf.lines.push(line);
            buf.total_bytes += byte_len as u64;
            // drop-oldest 不在此处触发 (caller 控制容量, 直接 restore 一次性)
            while buf.lines.len() > buf.capacity_lines {
                let old = buf.lines.remove(0);
                buf.total_bytes = buf.total_bytes.saturating_sub(old.byte_len as u64);
            }
        }
        Ok(buf)
    }
}

// =====================================================================
// 3. error
// =====================================================================

/// Scrollback 错误 (per 守门 #6 v2 6-field schema 风格)
#[derive(Debug, thiserror::Error)]
pub enum ScrollbackError {
    /// 拒绝空文本行 (per invariant: scrollback line 必须非空)
    #[error("empty text not allowed")]
    EmptyText,
    /// JSON 序列化失败 (per `to_json()`)
    #[error("serialize failed: {0}")]
    Serialize(serde_json::Error),
    /// JSON 反序列化失败 (per `from_json()`)
    #[error("deserialize failed: {0}")]
    Deserialize(serde_json::Error),
}

impl From<serde_json::Error> for ScrollbackError {
    fn from(e: serde_json::Error) -> Self {
        // Default: 标记为 Serialize, 实际 caller 用专门的 from_json 时已用 Deserialize 路径
        Self::Serialize(e)
    }
}

// =====================================================================
// 4. tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    #[test]
    fn empty_buffer_starts_empty() {
        let buf = ScrollbackBuffer::new(100);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.capacity(), 100);
        assert_eq!(buf.total_bytes(), 0);
        assert_eq!(buf.lines().len(), 0);
    }

    #[test]
    fn append_increments_len_and_bytes() {
        let mut buf = ScrollbackBuffer::new(100);
        let dropped = buf.append("hello", ScrollbackSource::Stdout).unwrap();
        assert_eq!(dropped, 0);
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.total_bytes(), 5);
        assert_eq!(buf.lines()[0].text, "hello");
        assert_eq!(buf.lines()[0].byte_len, 5);
    }

    #[test]
    fn append_rejects_empty_text() {
        let mut buf = ScrollbackBuffer::new(100);
        let r = buf.append("", ScrollbackSource::Stdout);
        assert!(matches!(r, Err(ScrollbackError::EmptyText)));
    }

    #[test]
    fn append_drops_oldest_when_capacity_full() {
        // 容量 3, append 5 行 → 前 2 行 drop-oldest
        let mut buf = ScrollbackBuffer::new(3);
        buf.append("line1", ScrollbackSource::Stdout).unwrap();
        buf.append("line2", ScrollbackSource::Stdout).unwrap();
        buf.append("line3", ScrollbackSource::Stdout).unwrap();
        // 满
        let dropped = buf.append("line4", ScrollbackSource::Stdout).unwrap();
        assert_eq!(dropped, 1);
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.lines()[0].text, "line2");
        assert_eq!(buf.lines()[2].text, "line4");

        let dropped = buf.append("line5", ScrollbackSource::Stdout).unwrap();
        assert_eq!(dropped, 1);
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.lines()[0].text, "line3");
        assert_eq!(buf.lines()[2].text, "line5");
    }

    #[test]
    fn total_bytes_recomputes_after_drop() {
        // 容量 2: append "abc"(3) + "de"(2) → 满, total=5
        // append "fgh"(3) → drop "abc", total=5
        let mut buf = ScrollbackBuffer::new(2);
        buf.append("abc", ScrollbackSource::Stdout).unwrap();
        buf.append("de", ScrollbackSource::Stdout).unwrap();
        assert_eq!(buf.total_bytes(), 5);
        buf.append("fgh", ScrollbackSource::Stdout).unwrap();
        assert_eq!(buf.total_bytes(), 5, "drop-oldest 必须减 total_bytes");
    }

    #[test]
    fn tail_returns_last_n_lines() {
        let mut buf = ScrollbackBuffer::new(100);
        for i in 0..10 {
            buf.append(format!("line{i}"), ScrollbackSource::Stdout)
                .unwrap();
        }
        let tail3 = buf.tail(3);
        assert_eq!(tail3.len(), 3);
        assert_eq!(tail3[0].text, "line7");
        assert_eq!(tail3[1].text, "line8");
        assert_eq!(tail3[2].text, "line9");
    }

    #[test]
    fn tail_handles_n_greater_than_len() {
        let mut buf = ScrollbackBuffer::new(100);
        buf.append("only", ScrollbackSource::Stdout).unwrap();
        let tail = buf.tail(10);
        assert_eq!(tail.len(), 1);
        assert_eq!(tail[0].text, "only");
    }

    #[test]
    fn lines_since_filters_by_timestamp() {
        let mut buf = ScrollbackBuffer::new(100);
        let t0 = now();
        buf.append_with_timestamp("a", ScrollbackSource::Stdout, t0)
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let t1 = now();
        buf.append_with_timestamp("b", ScrollbackSource::Stdout, t1)
            .unwrap();
        let since_t1 = t1 + chrono::Duration::milliseconds(1);
        let result = buf.lines_since(since_t1);
        // t1 + 1ms 之后, 只有晚于 t1 的行, 这里 'a' < t1 < 'b' = t1, 'b' > t1
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn clear_resets_buffer() {
        let mut buf = ScrollbackBuffer::new(10);
        buf.append("a", ScrollbackSource::Stdout).unwrap();
        buf.append("b", ScrollbackSource::Stdout).unwrap();
        buf.clear();
        assert!(buf.is_empty());
        assert_eq!(buf.total_bytes(), 0);
    }

    #[test]
    fn json_roundtrip_preserves_content_and_order() {
        let mut buf = ScrollbackBuffer::new(100);
        for i in 0..5 {
            buf.append(format!("line{i}"), ScrollbackSource::Stdout)
                .unwrap();
        }
        let json = buf.to_json().unwrap();
        let restored = ScrollbackBuffer::from_json(&json, 100).unwrap();
        assert_eq!(restored.len(), buf.len());
        for (a, b) in buf.lines().iter().zip(restored.lines().iter()) {
            assert_eq!(a.text, b.text);
            assert_eq!(a.byte_len, b.byte_len);
        }
    }

    #[test]
    fn json_roundtrip_after_drop_respects_capacity() {
        // 容量 2, append 5 → 保留最后 2 行
        let mut buf = ScrollbackBuffer::new(2);
        for i in 0..5 {
            buf.append(format!("line{i}"), ScrollbackSource::Stdout)
                .unwrap();
        }
        let json = buf.to_json().unwrap();
        let restored = ScrollbackBuffer::from_json(&json, 2).unwrap();
        assert_eq!(restored.len(), 2);
        assert_eq!(restored.lines()[0].text, "line3");
        assert_eq!(restored.lines()[1].text, "line4");
    }

    #[test]
    fn append_with_timestamp_preserves_timestamp() {
        let mut buf = ScrollbackBuffer::new(100);
        let explicit_ts = now() - chrono::Duration::days(7); // 7 天前
        buf.append_with_timestamp("old line", ScrollbackSource::Stdout, explicit_ts)
            .unwrap();
        let l = buf.lines()[0].clone();
        assert_eq!(l.timestamp, explicit_ts);
        assert_eq!(l.text, "old line");
    }

    #[test]
    fn different_sources_independent_id() {
        let mut buf = ScrollbackBuffer::new(100);
        buf.append("stdout line", ScrollbackSource::Stdout).unwrap();
        buf.append("stderr line", ScrollbackSource::Stderr).unwrap();
        buf.append("agent line", ScrollbackSource::AgentInjected)
            .unwrap();
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.lines()[0].source, ScrollbackSource::Stdout);
        assert_eq!(buf.lines()[1].source, ScrollbackSource::Stderr);
        assert_eq!(buf.lines()[2].source, ScrollbackSource::AgentInjected);
        // 各 line.id 唯一
        let ids: std::collections::HashSet<_> = buf.lines().iter().map(|l| l.id).collect();
        assert_eq!(ids.len(), 3);
    }
}
