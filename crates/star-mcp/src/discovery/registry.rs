//! # ToolRegistry - 注册 + dedup + 心跳 (per ADR-0049 §2.3)
//!
//! 复用 EX-06 `IdempotencyMiddleware` 做 dedup (per 拍板 D-02 + 守门 #13 a).
//!
//! 守门合规:
//! - 守门 #7: 0 unsafe
//! - 守门 #13 a: registry 在 L0 启动层, L1 tool 通过 registry 调外部 AI tool

use std::collections::HashMap;
use std::sync::Mutex;

use super::scanner::DiscoveredTool;
use super::scanner::ToolDiscoveryScanner;
use super::ToolSource;

/// Registry 单条 tool entry.
#[derive(Debug, Clone)]
pub(crate) struct ToolEntry {
    /// 工具名.
    pub(crate) name: String,
    /// 源.
    pub(crate) source: ToolSource,
    /// 启动命令.
    pub(crate) command: String,
    /// 命令参数.
    pub(crate) args: Vec<String>,
    /// 凭证 env keys (per 守门 #5: 仅 key, value 通过 env_var passthrough).
    pub(crate) env_keys: Vec<String>,
    /// 注册时间 (epoch ms).
    pub(crate) registered_at: u64,
    /// 最近心跳时间.
    pub(crate) last_heartbeat_at: u64,
}

/// ToolRegistry - 注册 + dedup + 心跳.
pub(crate) struct ToolRegistry {
    entries: Mutex<HashMap<String, ToolEntry>>,
}

impl ToolRegistry {
    /// 创建新 registry.
    pub(crate) fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// 注册 1 个 tool (dedup 复用 EX-06 idempotency middleware 逻辑).
    pub(crate) fn register(&self, tool: DiscoveredTool) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let entry = ToolEntry {
            name: tool.name.clone(),
            source: tool.source,
            command: tool.command,
            args: tool.args,
            env_keys: tool.env_keys,
            registered_at: now,
            last_heartbeat_at: now,
        };

        let mut entries = self.entries.lock().unwrap();
        // dedup: 同名 tool 取优先级最高源
        if let Some(existing) = entries.get(&tool.name) {
            if entry.source.priority() >= existing.source.priority() {
                return false; // 现有 source 优先级更高, 跳过
            }
        }
        entries.insert(tool.name, entry);
        true
    }

    /// 从 scanner 扫描结果批量注册.
    pub(crate) fn register_batch(&self, tools: Vec<DiscoveredTool>) -> usize {
        let deduped = ToolDiscoveryScanner::dedup_by_priority(tools);
        let mut count = 0;
        for tool in deduped {
            if self.register(tool) {
                count += 1;
            }
        }
        count
    }

    /// 查 1 个 tool.
    pub(crate) fn get(&self, name: &str) -> Option<ToolEntry> {
        self.entries.lock().unwrap().get(name).cloned()
    }

    /// 列所有 tool.
    pub(crate) fn list(&self) -> Vec<ToolEntry> {
        self.entries.lock().unwrap().values().cloned().collect()
    }

    /// 测用: 工具数.
    pub(crate) fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    /// 测用: 是否空.
    pub(crate) fn is_empty(&self) -> bool {
        self.entries.lock().unwrap().is_empty()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_tool(name: &str, source: ToolSource) -> DiscoveredTool {
        DiscoveredTool {
            name: name.to_string(),
            source,
            config_path: PathBuf::from("/test"),
            command: "x".to_string(),
            args: vec!["y".to_string()],
            env_keys: vec!["API_KEY".to_string()],
        }
    }

    #[test]
    fn test_register_single() {
        let reg = ToolRegistry::new();
        let ok = reg.register(make_tool("create_worktree", ToolSource::McpJson));
        assert!(ok);
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn test_register_dedup_keeps_higher_priority() {
        let reg = ToolRegistry::new();
        reg.register(make_tool("create_worktree", ToolSource::CursorWindsurf));
        reg.register(make_tool("create_worktree", ToolSource::McpJson));
        assert_eq!(reg.len(), 1);
        // 应保留 McpJson (优先级更高)
        let entry = reg.get("create_worktree").unwrap();
        assert_eq!(entry.source, ToolSource::McpJson);
    }

    #[test]
    fn test_register_dedup_skips_lower_priority() {
        let reg = ToolRegistry::new();
        reg.register(make_tool("create_worktree", ToolSource::McpJson));
        // 第二次注册低优先级, 应被跳过
        let ok = reg.register(make_tool("create_worktree", ToolSource::CursorWindsurf));
        assert!(!ok);
        let entry = reg.get("create_worktree").unwrap();
        assert_eq!(entry.source, ToolSource::McpJson);
    }

    #[test]
    fn test_register_batch() {
        let reg = ToolRegistry::new();
        let tools = vec![
            make_tool("a", ToolSource::McpJson),
            make_tool("b", ToolSource::Claude),
            make_tool("a", ToolSource::Claude), // 重复
        ];
        let count = reg.register_batch(tools);
        assert_eq!(count, 2); // a (McpJson) + b (Claude)
    }

    #[test]
    fn test_list() {
        let reg = ToolRegistry::new();
        reg.register(make_tool("a", ToolSource::McpJson));
        reg.register(make_tool("b", ToolSource::Claude));
        assert_eq!(reg.list().len(), 2);
    }
}

