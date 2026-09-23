//! `skill_registry.rs` — Skill Registry (per FR-ORCA-034 Skill Registry, §10 v1.0 行 469-471)
//!
//! Orca 借鉴点 9 的 Skill Registry 实现: Orca 的能力作为 installable skills
//! 发布, agent 通过 `npx skills add <url> --skill <name>` 安装.
//!
//! **MVP v0.1 范围 (本 issue ULYS-196)**:
//! - Skill 实体 + in-memory registry (与 `domain-local-runtime` CLI Session
//!   registry 模式一致, 避免引 sqlx-macros / refinery 等重型迁移框架)
//! - CLI subcommands: `star skill add/list/show/remove`
//! - 跨 agent 复用接口: `list_for_agent(agent_type) -> Vec<Skill>`
//!
//! **不在 MVP 范围 (P1 followup)**:
//! - SQLite WAL 持久化 (per cli_session_registry 模式扩展)
//! - Network 下载 + 校验 (sha256 / signature)
//! - Skill 版本语义化升级
//! - `npx skills add` CLI 包装 (实测需 Node.js + npm, 不在本 crate 依赖)
//!
//! 守门:
//! - #1 v15 cargo test 单 crate 实证
//! - #6 v2 StarError 6-field (复用 crates/star-cli/src/error.rs)
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标: 所有 dep 来自 [workspace.dependencies]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::error::StarError;

// =====================================================================
// 1. entity
// =====================================================================

/// Skill 来源 URL 类型(per FR-ORCA-034 npx skills add <url> --skill <name>)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub(crate) struct SkillSourceUrl(pub(crate) String);

impl SkillSourceUrl {
    /// 构造 + URL 格式校验
    pub(crate) fn new(url: impl Into<String>) -> Result<Self, SkillRegistryError> {
        let url = url.into();
        // MVP v0 限制: 仅允许 http/https/git 协议
        if !(url.starts_with("http://")
            || url.starts_with("https://")
            || url.starts_with("git://")
            || url.starts_with("git+"))
        {
            return Err(SkillRegistryError::InvalidSource(format!(
                "only http/https/git URLs allowed, got: {url}"
            )));
        }
        Ok(Self(url))
    }

    /// 返回 URL 字符串
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SkillSourceUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Skill (per FR-ORCA-034)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Skill {
    /// Skill 名(per `npx skills add --skill <name>`)
    pub(crate) name: String,
    /// 来源 URL(per `npx skills add <url>`)
    pub(crate) source_url: SkillSourceUrl,
    /// 版本字符串(semver 推荐, MVP v0 仅存不校验)
    pub(crate) version: String,
    /// 描述(供 UI / agent 展示)
    pub(crate) description: Option<String>,
    /// 兼容 agent 列表(per `SkillRegistry::list_for_agent(agent_type)`
    /// 调用, MVP v0 默认包含 ["claude-code", "codex", "multica"])
    pub(crate) agent_compat: Vec<String>,
    /// 安装时间
    pub(crate) installed_at: DateTime<Utc>,
    /// 元数据键值对(per Skill 安装时附带的 metadata, MVP v0 留空 map)
    pub(crate) metadata: HashMap<String, String>,
}

// =====================================================================
// 2. error
// =====================================================================

/// Skill Registry 错误 (per 守门 #6 v2 StarError 6-field 风格扩展)
#[derive(Debug, Error)]
pub(crate) enum SkillRegistryError {
    /// Skill 已存在(同 name)
    #[error("skill already exists: {0}")]
    AlreadyExists(String),
    /// Skill 未找到
    #[error("skill not found: {0}")]
    NotFound(String),
    /// URL 格式不合法
    #[error("invalid source URL: {0}")]
    InvalidSource(String),
    /// 参数校验失败(空 name / 空 version)
    #[error("invalid skill field: {0}")]
    InvalidField(String),
}

impl From<SkillRegistryError> for StarError {
    fn from(e: SkillRegistryError) -> Self {
        StarError::Skill(e.to_string())
    }
}

// =====================================================================
// 3. registry
// =====================================================================

/// Skill Registry (MVP v0 in-memory, 复用 cli_session_registry 模式)
pub(crate) struct SkillRegistry {
    skills: HashMap<String, Skill>,
}

impl SkillRegistry {
    /// 构造空 registry
    pub(crate) fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    /// 安装 Skill (per `star skill add <url> --name <n>`)
    ///
    /// **MVP v0 行为**: 直接插入, 不实际下载 source_url
    /// (Network 下载 + 校验留 P1 followup)
    pub(crate) fn install(
        &mut self,
        name: impl Into<String>,
        source_url: SkillSourceUrl,
        version: impl Into<String>,
        description: Option<String>,
        agent_compat: Vec<String>,
    ) -> Result<&Skill, SkillRegistryError> {
        let name = name.into();
        let version = version.into();

        if name.is_empty() {
            return Err(SkillRegistryError::InvalidField(
                "name 不能为空".to_string(),
            ));
        }
        if version.is_empty() {
            return Err(SkillRegistryError::InvalidField(
                "version 不能为空".to_string(),
            ));
        }
        if self.skills.contains_key(&name) {
            return Err(SkillRegistryError::AlreadyExists(name));
        }

        // MVP v0 默认 agent_compat: 若调用方未指定, 包含全部 3 个
        let agent_compat = if agent_compat.is_empty() {
            vec![
                "claude-code".to_string(),
                "codex".to_string(),
                "multica".to_string(),
            ]
        } else {
            agent_compat
        };

        let skill = Skill {
            name: name.clone(),
            source_url,
            version,
            description,
            agent_compat,
            installed_at: Utc::now(),
            metadata: HashMap::new(),
        };
        self.skills.insert(name.clone(), skill);
        Ok(self.skills.get(&name).expect("just inserted"))
    }

    /// 按 name 取 Skill
    pub(crate) fn get(&self, name: &str) -> Result<&Skill, SkillRegistryError> {
        self.skills
            .get(name)
            .ok_or_else(|| SkillRegistryError::NotFound(name.to_string()))
    }

    /// 列出全部 Skill (per `star skill list`)
    pub(crate) fn list(&self) -> Vec<&Skill> {
        let mut v: Vec<&Skill> = self.skills.values().collect();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v
    }

    /// 列出兼容指定 agent 的 Skill (per `SkillRegistry::list_for_agent`)
    pub(crate) fn list_for_agent(&self, agent_type: &str) -> Vec<&Skill> {
        self.list()
            .into_iter()
            .filter(|s| s.agent_compat.iter().any(|a| a == agent_type))
            .collect()
    }

    /// 卸载 Skill (per `star skill remove <name>`)
    pub(crate) fn remove(&mut self, name: &str) -> Result<Skill, SkillRegistryError> {
        self.skills
            .remove(name)
            .ok_or_else(|| SkillRegistryError::NotFound(name.to_string()))
    }

    /// 当前 Skill 数
    pub(crate) fn len(&self) -> usize {
        self.skills.len()
    }

    /// 是否为空
    pub(crate) fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// =====================================================================
// 4. CLI subcommands (per `star skill add/list/show/remove`)
// =====================================================================

/// `star skill ...` 子命令 (per clap 嵌套语法, top-level Skill variant 直接 4 子命令)
#[derive(Debug, clap::Args)]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct SkillCommandArgs {
    #[command(subcommand)]
    pub(crate) cmd: SkillCommand,
}

/// `star skill <subcmd>` 4 子命令
#[derive(Debug, clap::Subcommand)]
pub(crate) enum SkillCommand {
    /// 安装 Skill (per `npx skills add <url> --skill <name>` 的 star-side 等价)
    Add {
        /// 来源 URL (per `npx skills add <url>`)
        #[arg(value_name = "URL")]
        url: String,
        /// Skill 名 (per `npx skills add --skill <name>`)
        #[arg(long)]
        name: String,
        /// 版本字符串 (semver 推荐, MVP v0 仅存)
        #[arg(short, long, default_value = "0.1.0")]
        version: String,
        /// 描述 (可选)
        #[arg(long)]
        description: Option<String>,
        /// 兼容 agent 列表 (逗号分隔, MVP v0 默认全部)
        #[arg(long, value_delimiter = ',')]
        agent_compat: Vec<String>,
    },
    /// 列出全部 Skill (per `npx skills list` 的 star-side 等价)
    List {
        /// 仅列兼容指定 agent 的 Skill (per `list_for_agent(agent_type)`)
        #[arg(long)]
        agent: Option<String>,
    },
    /// 显示 Skill 详情
    Show {
        /// Skill 名
        name: String,
    },
    /// 卸载 Skill
    Remove {
        /// Skill 名
        name: String,
    },
}

/// Skill 子命令入口
pub(crate) fn run(args: SkillCommandArgs, registry: &mut SkillRegistry) -> Result<(), StarError> {
    run_cmd(args.cmd, registry)
}

/// Skill 子命令实际分发
fn run_cmd(cmd: SkillCommand, registry: &mut SkillRegistry) -> Result<(), StarError> {
    match cmd {
        SkillCommand::Add {
            url,
            name,
            version,
            description,
            agent_compat,
        } => {
            let source_url = SkillSourceUrl::new(url).map_err(SkillRegistryError::from)?;
            let skill = registry
                .install(name, source_url, version, description, agent_compat)
                .map_err(SkillRegistryError::from)?;
            // CLI 输出: 跟其他命令一致走 output::json_pretty
            println!(
                "{}",
                crate::output::json_pretty(serde_json::json!({
                    "schema_version": crate::output::SCHEMA_VERSION,
                    "operation": "skill add",
                    "skill": skill,
                }))?
            );
            Ok(())
        }
        SkillCommand::List { agent } => {
            let skills: Vec<&Skill> = if let Some(agent_type) = agent {
                registry.list_for_agent(&agent_type)
            } else {
                registry.list()
            };
            println!(
                "{}",
                crate::output::json_pretty(serde_json::json!({
                    "schema_version": crate::output::SCHEMA_VERSION,
                    "operation": "skill list",
                    "count": skills.len(),
                    "skills": skills,
                }))?
            );
            Ok(())
        }
        SkillCommand::Show { name } => {
            let skill = registry.get(&name)?;
            println!(
                "{}",
                crate::output::json_pretty(serde_json::json!({
                    "schema_version": crate::output::SCHEMA_VERSION,
                    "operation": "skill show",
                    "skill": skill,
                }))?
            );
            Ok(())
        }
        SkillCommand::Remove { name } => {
            let skill = registry.remove(&name)?;
            println!(
                "{}",
                crate::output::json_pretty(serde_json::json!({
                    "schema_version": crate::output::SCHEMA_VERSION,
                    "operation": "skill remove",
                    "removed": skill.name,
                }))?
            );
            Ok(())
        }
    }
}

// =====================================================================
// 5. tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_url() -> SkillSourceUrl {
        SkillSourceUrl::new("https://github.com/stablyai/skills").unwrap()
    }

    #[test]
    fn skill_source_url_validates_protocol() {
        // 合法: http / https / git / git+
        assert!(SkillSourceUrl::new("http://example.com").is_ok());
        assert!(SkillSourceUrl::new("https://example.com/x").is_ok());
        assert!(SkillSourceUrl::new("git://example.com/x").is_ok());
        assert!(SkillSourceUrl::new("git+https://example.com/x").is_ok());
        // 非法: file / ftp / raw path
        assert!(SkillSourceUrl::new("file:///etc/passwd").is_err());
        assert!(SkillSourceUrl::new("ftp://example.com").is_err());
        assert!(SkillSourceUrl::new("/local/path").is_err());
        assert!(SkillSourceUrl::new("example.com").is_err());
    }

    #[test]
    fn skill_install_inserts_and_deduplicates() {
        let mut reg = SkillRegistry::new();
        let url = dummy_url();
        let s1 = reg
            .install("orca-cli", url.clone(), "0.1.0", None, vec![])
            .unwrap();
        assert_eq!(s1.name, "orca-cli");
        assert_eq!(s1.version, "0.1.0");
        // 默认 agent_compat: 全部 3 个
        assert_eq!(s1.agent_compat.len(), 3);
        // 同名重复 → AlreadyExists
        let r2 = reg.install("orca-cli", url, "0.2.0", None, vec![]);
        assert!(matches!(r2, Err(SkillRegistryError::AlreadyExists(_))));
    }

    #[test]
    fn skill_install_validates_required_fields() {
        let mut reg = SkillRegistry::new();
        let url = dummy_url();
        // 空 name
        assert!(matches!(
            reg.install("", url.clone(), "0.1.0", None, vec![]),
            Err(SkillRegistryError::InvalidField(_))
        ));
        // 空 version
        assert!(matches!(
            reg.install("foo", url, "", None, vec![]),
            Err(SkillRegistryError::InvalidField(_))
        ));
    }

    #[test]
    fn skill_get_returns_not_found() {
        let reg = SkillRegistry::new();
        assert!(matches!(
            reg.get("nope"),
            Err(SkillRegistryError::NotFound(_))
        ));
    }

    #[test]
    fn skill_list_sorts_alphabetically() {
        let mut reg = SkillRegistry::new();
        let url = dummy_url();
        reg.install("zebra", url.clone(), "0.1.0", None, vec![])
            .unwrap();
        reg.install("alpha", url.clone(), "0.1.0", None, vec![])
            .unwrap();
        reg.install("mike", url, "0.1.0", None, vec![]).unwrap();
        let names: Vec<&str> = reg.list().iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "mike", "zebra"]);
    }

    #[test]
    fn skill_list_for_agent_filters_correctly() {
        let mut reg = SkillRegistry::new();
        let url = dummy_url();
        // 仅 codex 兼容
        reg.install(
            "codex-only",
            url.clone(),
            "0.1.0",
            None,
            vec!["codex".to_string()],
        )
        .unwrap();
        // 仅 claude-code 兼容
        reg.install(
            "claude-only",
            url.clone(),
            "0.1.0",
            None,
            vec!["claude-code".to_string()],
        )
        .unwrap();
        // 默认全部兼容
        reg.install("all-agents", url, "0.1.0", None, vec![])
            .unwrap();

        let for_codex: Vec<&str> = reg
            .list_for_agent("codex")
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        assert_eq!(for_codex, vec!["all-agents", "codex-only"]);

        let for_claude: Vec<&str> = reg
            .list_for_agent("claude-code")
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        assert_eq!(for_claude, vec!["all-agents", "claude-only"]);

        let for_unknown: Vec<&str> = reg
            .list_for_agent("unknown-agent")
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        // "all-agents" 默认 agent_compat 是 ["claude-code", "codex", "multica"]
        // 不含 "unknown-agent", 所以 unknown-agent filter 应为空
        assert_eq!(for_unknown, Vec::<&str>::new());
    }

    #[test]
    fn skill_remove_returns_skill_and_clears() {
        let mut reg = SkillRegistry::new();
        let url = dummy_url();
        reg.install("foo", url, "0.1.0", None, vec![]).unwrap();
        assert_eq!(reg.len(), 1);

        let removed = reg.remove("foo").unwrap();
        assert_eq!(removed.name, "foo");
        assert_eq!(reg.len(), 0);

        // 重复 remove → NotFound
        assert!(matches!(
            reg.remove("foo"),
            Err(SkillRegistryError::NotFound(_))
        ));
    }

    #[test]
    fn skill_serde_roundtrip() {
        let mut reg = SkillRegistry::new();
        let url = dummy_url();
        reg.install(
            "orca-cli",
            url,
            "0.1.0",
            Some("Orca CLI skill".to_string()),
            vec!["codex".to_string()],
        )
        .unwrap();

        // serde JSON round-trip via registry.list()
        let json = serde_json::to_string(&reg.list()).unwrap();
        let back: Vec<Skill> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].name, "orca-cli");
        assert_eq!(back[0].description.as_deref(), Some("Orca CLI skill"));
        assert_eq!(back[0].agent_compat, vec!["codex".to_string()]);
    }
}
