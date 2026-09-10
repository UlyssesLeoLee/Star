//! # ToolDiscoveryScanner - 4 源 AI 工具 config 扫描
//!
//! per `ADR-0049 AI 工具自动扫描 + 链接` §2.2 (4 源扫描实现)
//!
//! 守门合规:
//! - 守门 #5: env 字段值不打印, 仅引用
//! - 守门 #7: 0 unsafe
//! - 守门 #13 a: L0 启动层, 经 registry 注册后 L1 可调

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 4 源配置 (per 拍板 D-02).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ToolSource {
    /// `.mcp.json` (Anthropic MCP 标准).
    McpJson,
    /// Claude / Claude Code.
    Claude,
    /// OpenAI / Codex.
    OpenAI,
    /// Cursor / Windsurf / 其他 IDE.
    CursorWindsurf,
}

impl ToolSource {
    /// 优先级 (per ADR-0049 §2.4): 数字越小优先级越高.
    pub(crate) fn priority(&self) -> u8 {
        match self {
            Self::McpJson => 0,
            Self::Claude => 1,
            Self::OpenAI => 2,
            Self::CursorWindsurf => 3,
        }
    }
}

/// 4 源配置路径 (Windows 优先, Unix fallback).
#[derive(Debug, Clone)]
pub(crate) struct ToolSourceConfig {
    /// 项目根 `.mcp.json`.
    pub(crate) mcp_json_project: PathBuf,
    /// 用户级 `~/.mcp.json`.
    pub(crate) mcp_json_user: PathBuf,
    /// `~/.claude/settings.json` + `~/.claude.json`.
    pub(crate) claude_settings: PathBuf,
    /// `~/.config/openai/`.
    pub(crate) openai_config: PathBuf,
    /// `~/.cursor/mcp.json` + `~/.windsurf/mcp.json`.
    pub(crate) cursor_mcp: PathBuf,
    /// `~/.windsurf/mcp.json`.
    pub(crate) windsurf_mcp: PathBuf,
}

impl Default for ToolSourceConfig {
    fn default() -> Self {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| ".".to_string());
        let home_path = PathBuf::from(home);
        Self {
            mcp_json_project: PathBuf::from(".mcp.json"),
            mcp_json_user: home_path.join(".mcp.json"),
            claude_settings: home_path.join(".claude").join("settings.json"),
            openai_config: home_path.join(".config").join("openai"),
            cursor_mcp: home_path.join(".cursor").join("mcp.json"),
            windsurf_mcp: home_path.join(".windsurf").join("mcp.json"),
        }
    }
}

/// 发现的 AI 工具 (1 个 tool 1 个 entry).
#[derive(Debug, Clone)]
pub(crate) struct DiscoveredTool {
    /// 工具名 (e.g. "create_worktree").
    pub(crate) name: String,
    /// 源 (per ToolSource).
    pub(crate) source: ToolSource,
    /// 配置文件路径.
    pub(crate) config_path: PathBuf,
    /// 启动命令 (e.g. "npx", "python").
    pub(crate) command: String,
    /// 命令参数.
    pub(crate) args: Vec<String>,
    /// 环境变量 (key only, value 通过 env_var passthrough, per 守门 #5).
    pub(crate) env_keys: Vec<String>,
}

impl DiscoveredTool {
    /// 唯一标识 (name + source) 用于 dedup.
    pub(crate) fn dedup_key(&self) -> String {
        format!("{}:{}", self.source.priority(), self.name)
    }
}

/// 4 源扫描器.
pub(crate) struct ToolDiscoveryScanner {
    config: ToolSourceConfig,
}

impl ToolDiscoveryScanner {
    /// 创建新 scanner (默认 config).
    pub(crate) fn new() -> Self {
        Self {
            config: ToolSourceConfig::default(),
        }
    }

    /// 创建带自定义 config 的 scanner (测用).
    pub(crate) fn with_config(config: ToolSourceConfig) -> Self {
        Self { config }
    }

    /// 扫描 4 源 (per 拍板 D-02), 10s 内完成.
    pub(crate) fn scan_all(&self) -> Vec<DiscoveredTool> {
        let mut tools = Vec::new();
        tools.extend(self.scan_mcp_json(&self.config.mcp_json_project));
        tools.extend(self.scan_mcp_json(&self.config.mcp_json_user));
        tools.extend(self.scan_claude(&self.config.claude_settings));
        tools.extend(self.scan_openai(&self.config.openai_config));
        tools.extend(self.scan_cursor_windsurf(&self.config.cursor_mcp));
        tools.extend(self.scan_cursor_windsurf(&self.config.windsurf_mcp));
        tools
    }

    /// 扫描 `.mcp.json` (Anthropic MCP 标准).
    ///
    /// JSON 格式: `{mcpServers: {name: {command, args, env}}}`
    fn scan_mcp_json(&self, path: &Path) -> Vec<DiscoveredTool> {
        self.parse_mcp_json_file(path, ToolSource::McpJson)
    }

    fn parse_mcp_json_file(&self, path: &Path, source: ToolSource) -> Vec<DiscoveredTool> {
        let Ok(content) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
            return Vec::new();
        };
        let Some(servers) = json.get("mcpServers").and_then(|v| v.as_object()) else {
            return Vec::new();
        };
        servers
            .iter()
            .filter_map(|(name, value)| {
                let value = value.as_object()?;
                let command = value.get("command")?.as_str()?.to_string();
                let args = value
                    .get("args")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                let env_keys = value
                    .get("env")
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.keys().cloned().collect())
                    .unwrap_or_default();
                Some(DiscoveredTool {
                    name: name.clone(),
                    source,
                    config_path: path.to_path_buf(),
                    command,
                    args,
                    env_keys,
                })
            })
            .collect()
    }

    /// 扫描 Claude / Claude Code.
    ///
    /// JSON 格式: `{mcpServers: {name: {command, args, env}}}` 或 `{apiKey: "..."}`
    fn scan_claude(&self, path: &Path) -> Vec<DiscoveredTool> {
        // Claude 格式跟 mcp.json 类似, 复用 parser
        self.parse_mcp_json_file(path, ToolSource::Claude)
    }

    /// 扫描 OpenAI / Codex.
    ///
    /// TOML/JSON 格式: `{api_key, base_url, tools: [...]}`
    fn scan_openai(&self, path: &Path) -> Vec<DiscoveredTool> {
        // OpenAI 实际是 list of files in dir
        let Ok(entries) = std::fs::read_dir(path) else {
            return Vec::new();
        };
        let mut tools = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            // 简化: JSON 格式
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(tool_array) = json.get("tools").and_then(|v| v.as_array()) {
                    for tool in tool_array {
                        if let Some(tool_obj) = tool.as_object() {
                            if let Some(name) = tool_obj.get("name").and_then(|v| v.as_str()) {
                                let command = tool_obj
                                    .get("command")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("openai")
                                    .to_string();
                                let args = tool_obj
                                    .get("args")
                                    .and_then(|v| v.as_array())
                                    .map(|arr| {
                                        arr.iter()
                                            .filter_map(|x| x.as_str().map(String::from))
                                            .collect()
                                    })
                                    .unwrap_or_default();
                                let env_keys = tool_obj
                                    .get("env")
                                    .and_then(|v| v.as_object())
                                    .map(|obj| obj.keys().cloned().collect())
                                    .unwrap_or_default();
                                tools.push(DiscoveredTool {
                                    name: name.to_string(),
                                    source: ToolSource::OpenAI,
                                    config_path: path.clone(),
                                    command,
                                    args,
                                    env_keys,
                                });
                            }
                        }
                    }
                }
            }
        }
        tools
    }

    /// 扫描 Cursor / Windsurf (复用 mcp.json parser).
    fn scan_cursor_windsurf(&self, path: &Path) -> Vec<DiscoveredTool> {
        self.parse_mcp_json_file(path, ToolSource::CursorWindsurf)
    }

    /// 按优先级去重 (低优先级数字 = 高优先级).
    pub(crate) fn dedup_by_priority(tools: Vec<DiscoveredTool>) -> Vec<DiscoveredTool> {
        let mut map: HashMap<String, DiscoveredTool> = HashMap::new();
        for tool in tools {
            let key = format!("{}", tool.name);
            let should_insert = match map.get(&key) {
                None => true,
                Some(existing) => tool.source.priority() < existing.source.priority(),
            };
            if should_insert {
                map.insert(key, tool);
            }
        }
        map.into_values().collect()
    }
}

impl Default for ToolDiscoveryScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_priority() {
        assert!(ToolSource::McpJson.priority() < ToolSource::Claude.priority());
        assert!(ToolSource::Claude.priority() < ToolSource::OpenAI.priority());
        assert!(ToolSource::OpenAI.priority() < ToolSource::CursorWindsurf.priority());
    }

    #[test]
    fn test_dedup_keeps_higher_priority() {
        let tools = vec![
            DiscoveredTool {
                name: "create_worktree".to_string(),
                source: ToolSource::CursorWindsurf,
                config_path: PathBuf::from("/test"),
                command: "x".to_string(),
                args: vec![],
                env_keys: vec![],
            },
            DiscoveredTool {
                name: "create_worktree".to_string(),
                source: ToolSource::McpJson,
                config_path: PathBuf::from("/test2"),
                command: "y".to_string(),
                args: vec![],
                env_keys: vec![],
            },
        ];
        let dedup = ToolDiscoveryScanner::dedup_by_priority(tools);
        assert_eq!(dedup.len(), 1);
        assert_eq!(dedup[0].source, ToolSource::McpJson);
        assert_eq!(dedup[0].command, "y");
    }

    #[test]
    fn test_dedup_keeps_different_names() {
        let tools = vec![
            DiscoveredTool {
                name: "tool-a".to_string(),
                source: ToolSource::McpJson,
                config_path: PathBuf::from("/test"),
                command: "x".to_string(),
                args: vec![],
                env_keys: vec![],
            },
            DiscoveredTool {
                name: "tool-b".to_string(),
                source: ToolSource::Claude,
                config_path: PathBuf::from("/test2"),
                command: "y".to_string(),
                args: vec![],
                env_keys: vec![],
            },
        ];
        let dedup = ToolDiscoveryScanner::dedup_by_priority(tools);
        assert_eq!(dedup.len(), 2);
    }

    #[test]
    fn test_parse_mcp_json_file_valid() {
        let scanner = ToolDiscoveryScanner::new();
        let content = r#"{
            "mcpServers": {
                "create_worktree": {
                    "command": "npx",
                    "args": ["-y", "@star/mcp-worktree"],
                    "env": {"API_KEY": "${API_KEY}"}
                }
            }
        }"#;
        let tmp = std::env::temp_dir().join("test_mcp.json");
        std::fs::write(&tmp, content).unwrap();
        let tools = scanner.parse_mcp_json_file(&tmp, ToolSource::McpJson);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "create_worktree");
        assert_eq!(tools[0].command, "npx");
        assert_eq!(tools[0].args, vec!["-y", "@star/mcp-worktree"]);
        assert_eq!(tools[0].env_keys, vec!["API_KEY"]);
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_parse_mcp_json_file_missing() {
        let scanner = ToolDiscoveryScanner::new();
        let tools = scanner.parse_mcp_json_file(
            &std::env::temp_dir().join("nonexistent_mcp.json"),
            ToolSource::McpJson,
        );
        assert!(tools.is_empty());
    }
}
