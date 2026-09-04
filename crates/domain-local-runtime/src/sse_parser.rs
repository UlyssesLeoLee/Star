//! Star Local Runtime — OpenAI SSE 响应解析器 (wt-w25)
//!
//! Per 2026-08-29 10:25 JST Phase 2 候选 1:
//! 解析 OpenAI-compatible ChatCompletion 流式响应 (data: {json}\n\n)
//! 提取 choices[0].delta.content
//!
//! 同时支持 SSE 注释 (`: keep-alive`) 和 [DONE] 标记.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// =====================================================================
// 1. value_object
// =====================================================================

/// 解析后的单个 chunk
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SseChunk {
    /// delta 内容 (拼接后即完整响应)
    pub content: String,
    /// 角色 (仅首 chunk)
    pub role: Option<String>,
    /// finish_reason (仅末 chunk, e.g. "stop" / "length")
    pub finish_reason: Option<String>,
    /// 模型名
    pub model: Option<String>,
}

/// OpenAI ChatCompletion 流式 chunk schema
#[derive(Debug, Clone, PartialEq, Deserialize)]
struct OpenAiStreamChunk {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    choices: Vec<OpenAiChoice>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct OpenAiChoice {
    #[serde(default)]
    index: Option<u32>,
    #[serde(default)]
    delta: OpenAiDelta,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
struct OpenAiDelta {
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    content: Option<String>,
}

// =====================================================================
// 2. parser
// =====================================================================

/// SSE 流式解析器
/// 输入: 原始 SSE 字符串 (单行或多行)
/// 输出: 解析后的 SseChunk 列表
pub struct SseParser {
    /// 累积 buffer (处理跨 chunk 边界)
    buffer: String,
}

impl SseParser {
    /// 构造空解析器
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// 喂入一段原始字节, 返回完整事件列表
    pub fn feed(&mut self, chunk: &str) -> Vec<Result<SseChunk, SseParseError>> {
        self.buffer.push_str(chunk);
        let mut results = Vec::new();

        // 按 \n\n 切分事件
        while let Some(end) = self.buffer.find("\n\n") {
            let event = self.buffer[..end].to_string();
            self.buffer = self.buffer[end + 2..].to_string();
            if let Some(parsed) = parse_event(&event) {
                results.push(parsed);
            }
        }
        results
    }

    /// 收尾 (流结束, 处理残余)
    pub fn finish(&mut self) -> Vec<Result<SseChunk, SseParseError>> {
        if self.buffer.is_empty() {
            return vec![];
        }
        let event = std::mem::take(&mut self.buffer);
        match parse_event(&event) {
            Some(r) => vec![r],
            None => vec![],
        }
    }
}

impl Default for SseParser {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_event(event: &str) -> Option<Result<SseChunk, SseParseError>> {
    // SSE 事件: 多行 "field: value", 通常 "data: <json>"
    let mut data = String::new();
    for line in event.lines() {
        if let Some(rest) = line.strip_prefix("data:") {
            let value = rest.trim_start();
            // [DONE] 哨兵
            if value == "[DONE]" {
                return None;
            }
            // 多 data 行: 拼接 (实际少见)
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(value);
        }
        // 忽略 event:/id:/retry:/: 注释
    }
    if data.is_empty() {
        return None;
    }

    // 解析 JSON
    match serde_json::from_str::<OpenAiStreamChunk>(&data) {
        Ok(c) => {
            if c.choices.is_empty() {
                return None;
            }
            let choice = &c.choices[0];
            Some(Ok(SseChunk {
                content: choice.delta.content.clone().unwrap_or_default(),
                role: choice.delta.role.clone(),
                finish_reason: choice.finish_reason.clone(),
                model: c.model.clone(),
            }))
        }
        Err(e) => Some(Err(SseParseError::Json(e.to_string()))),
    }
}

// =====================================================================
// 3. error
// =====================================================================

/// SSE 解析错误
#[derive(Debug, Error, Clone, PartialEq)]
pub enum SseParseError {
    /// SSE JSON 解析失败
    #[error("SSE JSON 解析失败: {0}")]
    Json(String),
    /// SSE 格式错误
    #[error("SSE 格式错误: {0}")]
    Format(String),
}

// =====================================================================
// 4. invariant
// =====================================================================

/// INV-SSE-01: data 字段必非空
pub fn inv_01_data_not_empty(event: &str) -> bool {
    for line in event.lines() {
        if line.trim_start().starts_with("data:") {
            return line.trim_start().len() > 5;
        }
    }
    false
}

/// INV-SSE-02: [DONE] 哨兵表示流结束
pub fn inv_02_done_sentinel(event: &str) -> bool {
    event.contains("data: [DONE]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_content_chunk() {
        let raw = "data: {\"id\":\"chatcmpl-1\",\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n";
        let chunks = SseParser::feed_str(raw);
        assert_eq!(chunks.len(), 1, "raw: {:?}", raw);
        let c = chunks[0].as_ref().unwrap();
        assert_eq!(c.content, "Hello");
        assert_eq!(c.model.as_deref(), Some("gpt-4"));
    }

    #[test]
    fn test_parse_role_chunk() {
        let raw = "data: {\"choices\":[{\"delta\":{\"role\":\"assistant\"}}]}\n\n";
        let chunks = SseParser::feed_str(raw);
        assert_eq!(chunks.len(), 1);
        let c = chunks[0].as_ref().unwrap();
        assert_eq!(c.role.as_deref(), Some("assistant"));
        assert_eq!(c.content, "");
    }

    #[test]
    fn test_parse_finish_chunk() {
        let raw = "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n";
        let chunks = SseParser::feed_str(raw);
        assert_eq!(chunks.len(), 1);
        let c = chunks[0].as_ref().unwrap();
        assert_eq!(c.finish_reason.as_deref(), Some("stop"));
    }

    #[test]
    fn test_parse_done_sentinel() {
        let raw = "data: [DONE]\n\n";
        let chunks = SseParser::feed_str(raw);
        assert_eq!(chunks.len(), 0); // [DONE] 返回 None
    }

    #[test]
    fn test_parse_ignore_sse_comments() {
        let raw = ": keep-alive\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"hi\"}}]}\n\n";
        let chunks = SseParser::feed_str(raw);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].as_ref().unwrap().content, "hi");
    }

    #[test]
    fn test_parse_invalid_json() {
        let raw = "data: {invalid json\n\n";
        let chunks = SseParser::feed_str(raw);
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].is_err());
    }

    #[test]
    fn test_buffer_split_events() {
        let mut p = SseParser::new();
        // 第一个完整事件
        let c1 = p.feed(
            r#"data: {"choices":[{"delta":{"content":"a"}}]}

"#,
        );
        assert_eq!(c1.len(), 1);
        assert_eq!(c1[0].as_ref().unwrap().content, "a");
        // 跨 chunk: 半个事件
        let c2 = p.feed(r#"data: {"choices":[{"delta":{"con"#);
        assert_eq!(c2.len(), 0);
        // 完成另一半
        let c3 = p.feed(
            r#"tent":"b"}}]}

"#,
        );
        assert_eq!(c3.len(), 1);
        assert_eq!(c3[0].as_ref().unwrap().content, "b");
    }

    #[test]
    fn test_finish_remaining() {
        let mut p = SseParser::new();
        p.feed(r#"data: {"choices":[{"delta":{"content":"x"}}]}"#);
        let r = p.finish();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].as_ref().unwrap().content, "x");
    }

    #[test]
    fn test_inv_01_data_not_empty() {
        assert!(!inv_01_data_not_empty("data:"));
        assert!(!inv_01_data_not_empty(""));
        assert!(!inv_01_data_not_empty("event: ping"));
    }

    #[test]
    fn test_inv_02_done_sentinel() {
        assert!(inv_02_done_sentinel("data: [DONE]"));
        assert!(!inv_02_done_sentinel("data: [done]"));
    }
}

// 测试 helper
impl SseParser {
    /// 测试便捷方法
    pub fn feed_str(s: &str) -> Vec<Result<SseChunk, SseParseError>> {
        let mut p = SseParser::new();
        p.feed(s)
    }
}
