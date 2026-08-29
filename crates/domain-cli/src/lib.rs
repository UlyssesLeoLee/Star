//! Star CLI Agent Registry (精简实装 v0.1)
//!
//! 职责:
//! 1. **6 个内置 CLI Agent** (claude / codex / openclaw / hermes / gemini / aider) + 自定义 schema
//! 2. **双模式 API Key 存储** (per 2026-08-29 09:07 JST 用户拍板):
//!    - **EncryptedRust**: AES-256-GCM 加密存于后端 domain-cli
//!    - **EnvironmentVar**: 启动时从 process env 读, 不存后端
//! 3. **API Agent 抽象** (OpenClaw / Hermes 走 HTTP API, 不走 CLI spawn)
//! 4. **CLI Process Adapter** (claude / codex / gemini / aider 走 spawn 进程)
//! 5. **三触发上传** (per wt-w18 windows, 这里只暴露 trait)
//!
//! 不变量 INV-CLI-01~04
//!
//! Phase 2 接: 真实 CLI spawn (w19 local-runtime) + 真实 HTTP API 调用 (w19)

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// 1. value_object — CLI Agent 类型 + 配置
// =====================================================================

/// CLI Agent 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CliKind {
    /// Claude Code CLI
    Claude,
    /// OpenAI Codex CLI
    Codex,
    /// OpenClaw HTTP API Agent (不走 CLI spawn)
    OpenClaw,
    /// Hermes HTTP API Agent
    Hermes,
    /// Google Gemini CLI
    Gemini,
    /// Aider CLI
    Aider,
    /// 自定义
    Custom,
}

impl CliKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "OpenAI Codex",
            Self::OpenClaw => "OpenClaw",
            Self::Hermes => "Hermes",
            Self::Gemini => "Gemini",
            Self::Aider => "Aider",
            Self::Custom => "Custom",
        }
    }

    pub fn is_api_agent(&self) -> bool {
        matches!(self, Self::OpenClaw | Self::Hermes)
    }

    pub fn all_builtin() -> &'static [CliKind] {
        &[
            Self::Claude,
            Self::Codex,
            Self::OpenClaw,
            Self::Hermes,
            Self::Gemini,
            Self::Aider,
        ]
    }
}

/// API Key 存储模式 (per 2026-08-29 09:07 JST 用户拍板双模式)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeyMode {
    /// 后端 AES-256-GCM 加密存储
    EncryptedRust,
    /// 从 process env 读, 不存后端
    EnvironmentVar,
}

impl ApiKeyMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::EncryptedRust => "Encrypted (Rust backend)",
            Self::EnvironmentVar => "Environment Variable",
        }
    }
}

/// API Key 凭证 (后端只存 hash + label, 明文不返)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub provider: String, // "anthropic" / "openai" / "openclaw" / "hermes" / "google"
    pub label: String,
    pub mode: ApiKeyMode,
    /// EncryptedRust: AES-256-GCM 密文 (base64 编码)
    /// EnvironmentVar: 此字段为空, 用 env_var_name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_value: Option<String>,
    /// EnvironmentVar 模式: env 变量名 (例: "ANTHROPIC_API_KEY")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_var_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl ApiKey {
    pub fn new_encrypted(
        provider: impl Into<String>,
        label: impl Into<String>,
        encrypted_value: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            provider: provider.into(),
            label: label.into(),
            mode: ApiKeyMode::EncryptedRust,
            encrypted_value: Some(encrypted_value.into()),
            env_var_name: None,
            created_at: chrono::Utc::now(),
            last_used_at: None,
        }
    }

    pub fn new_env_var(
        provider: impl Into<String>,
        label: impl Into<String>,
        env_var_name: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            provider: provider.into(),
            label: label.into(),
            mode: ApiKeyMode::EnvironmentVar,
            encrypted_value: None,
            env_var_name: Some(env_var_name.into()),
            created_at: chrono::Utc::now(),
            last_used_at: None,
        }
    }

    /// 返回安全摘要 (不含密文)
    pub fn summary(&self) -> ApiKeySummary {
        ApiKeySummary {
            id: self.id,
            provider: self.provider.clone(),
            label: self.label.clone(),
            mode: self.mode,
            env_var_name: self.env_var_name.clone(),
            created_at: self.created_at,
            last_used_at: self.last_used_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiKeySummary {
    pub id: Uuid,
    pub provider: String,
    pub label: String,
    pub mode: ApiKeyMode,
    pub env_var_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// CLI Profile (用户配置)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CliProfile {
    pub id: Uuid,
    pub name: String,
    pub kind: CliKind,
    /// 进程模式: 命令路径 (例: "claude" / "codex" / "aider")
    /// API 模式: 端点 URL
    pub command: String,
    pub args: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
    pub worktree_binding: WorktreeBinding,
    pub api_key_id: Option<Uuid>, // 绑定的 API Key
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorktreeBinding {
    /// 自动 (CLI 启动时用当前 worktree)
    Auto,
    /// 固定到指定 worktree ID
    Fixed(Uuid),
    /// 提示用户选择 (打开窗口时)
    Prompt,
}

impl CliProfile {
    pub fn new_builtin(kind: CliKind) -> Self {
        let (name, command, default_args) = match kind {
            CliKind::Claude => (
                "Claude Code".to_string(),
                "claude".to_string(),
                vec!["--model".to_string(), "claude-3-5-sonnet".to_string()],
            ),
            CliKind::Codex => (
                "OpenAI Codex".to_string(),
                "codex".to_string(),
                vec!["--model".to_string(), "gpt-4".to_string()],
            ),
            CliKind::OpenClaw => (
                "OpenClaw".to_string(),
                "https://api.openclaw.dev/v1".to_string(),
                vec![],
            ),
            CliKind::Hermes => (
                "Hermes".to_string(),
                "https://api.hermes.dev/v1".to_string(),
                vec![],
            ),
            CliKind::Gemini => ("Google Gemini".to_string(), "gemini".to_string(), vec![]),
            CliKind::Aider => (
                "Aider".to_string(),
                "aider".to_string(),
                vec!["--model".to_string(), "gpt-4".to_string()],
            ),
            CliKind::Custom => ("Custom".to_string(), "".to_string(), vec![]),
        };
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            kind,
            command,
            args: default_args,
            env: std::collections::HashMap::new(),
            worktree_binding: WorktreeBinding::Auto,
            api_key_id: None,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
}

// =====================================================================
// 2. entity — Agent 任务窗口 (CLI session 的一次执行)
// =====================================================================

/// 任务窗口 (per 2026-08-29 04:09 JST 上轮拍板: 新页面 agent-windows)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskWindow {
    pub id: Uuid,
    pub name: String,
    pub worktree_id: Uuid,
    pub profile_id: Uuid,
    pub tabs: Vec<TaskTab>,
    pub active_tab_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 任务 Tab (一个 CLI 会话实例)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskTab {
    pub id: Uuid,
    pub window_id: Uuid,
    pub profile_id: Uuid,
    pub label: String,
    pub state: TabState,
    pub last_output: String, // 最近 N 行 (前端展示)
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TabState {
    Created,
    Running,
    WaitingInput,
    Completed,
    Failed,
    Aborted,
}

impl TaskWindow {
    pub fn new(name: impl Into<String>, worktree_id: Uuid, profile_id: Uuid) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            worktree_id,
            profile_id,
            tabs: Vec::new(),
            active_tab_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_tab(&mut self, tab: TaskTab) -> Result<(), CliError> {
        if self.tabs.iter().any(|t| t.id == tab.id) {
            return Err(CliError::DuplicateTabId(tab.id));
        }
        if self.tabs.len() >= 20 {
            return Err(CliError::TooManyTabs(20));
        }
        self.active_tab_id.get_or_insert(tab.id);
        self.tabs.push(tab);
        self.updated_at = chrono::Utc::now();
        Ok(())
    }

    pub fn close_tab(&mut self, tab_id: Uuid) -> Result<(), CliError> {
        let before = self.tabs.len();
        self.tabs.retain(|t| t.id != tab_id);
        if self.tabs.len() == before {
            return Err(CliError::TabNotFound(tab_id));
        }
        if self.active_tab_id == Some(tab_id) {
            self.active_tab_id = self.tabs.first().map(|t| t.id);
        }
        self.updated_at = chrono::Utc::now();
        Ok(())
    }
}

// =====================================================================
// 3. error
// =====================================================================

#[derive(Debug, Error, Clone, PartialEq)]
pub enum CliError {
    #[error("CLI 启动失败: {0}")]
    SpawnFailed(String),
    #[error("HTTP API 调用失败: {0}")]
    HttpApiFailed(String),
    #[error("API Key 缺失或错误: provider={0}")]
    ApiKeyMissing(String),
    #[error("API Key 解密失败")]
    DecryptionFailed,
    #[error("API Key 解密或 Base64 失败: {0}")]
    DecryptionOrBase64(String),
    #[error("加密失败: {0}")]
    EncryptionFailed(String),
    #[error("Profile 不存在: {0}")]
    ProfileNotFound(Uuid),
    #[error("Tab 不存在: {0}")]
    TabNotFound(Uuid),
    #[error("Tab ID 重复: {0}")]
    DuplicateTabId(Uuid),
    #[error("Tab 数量超限: max {0}")]
    TooManyTabs(usize),
    #[error("Worktree 不存在: {0}")]
    WorktreeNotFound(Uuid),
    #[error("不支持的 CLI 类型: {0}")]
    UnsupportedCli(String),
    #[error("环境变量不存在: {0}")]
    EnvVarMissing(String),
}

// =====================================================================
// 4. port — CLI Adapter trait (CLI + API 两种)
// =====================================================================

#[async_trait]
pub trait CliAdapter: Send + Sync {
    /// CLI 模式: spawn 进程
    /// API 模式: 走 HTTP
    async fn invoke(
        &self,
        profile: &CliProfile,
        prompt: &str,
        api_key: Option<&str>,
    ) -> Result<InvocationResult, CliError>;

    fn kind(&self) -> CliKind;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvocationResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub files_changed: Vec<String>, // git diff 检测
}

// =====================================================================
// 5. service — 加密 / 解密 / 适配器路由
// =====================================================================

pub struct CliService {
    /// 内存 store (Phase 2 替换为持久化)
    profiles: std::sync::RwLock<std::collections::HashMap<Uuid, CliProfile>>,
    api_keys: std::sync::RwLock<std::collections::HashMap<Uuid, ApiKey>>,
    /// 加密 master key (Phase 2 接入 KMS)
    master_key: [u8; 32],
}

impl CliService {
    pub fn new(master_key: [u8; 32]) -> Self {
        Self {
            profiles: std::sync::RwLock::new(std::collections::HashMap::new()),
            api_keys: std::sync::RwLock::new(std::collections::HashMap::new()),
            master_key,
        }
    }

    /// 注册内置 6 个 profile
    pub fn seed_builtin_profiles(&self) {
        for kind in CliKind::all_builtin() {
            let profile = CliProfile::new_builtin(*kind);
            self.profiles.write().unwrap().insert(profile.id, profile);
        }
    }

    /// 创建 / 更新 profile
    pub fn upsert_profile(&self, profile: CliProfile) -> Result<(), CliError> {
        self.profiles.write().unwrap().insert(profile.id, profile);
        Ok(())
    }

    pub fn list_profiles(&self) -> Vec<CliProfile> {
        self.profiles.read().unwrap().values().cloned().collect()
    }

    pub fn get_profile(&self, id: Uuid) -> Option<CliProfile> {
        self.profiles.read().unwrap().get(&id).cloned()
    }

    pub fn delete_profile(&self, id: Uuid) -> Result<(), CliError> {
        self.profiles
            .write()
            .unwrap()
            .remove(&id)
            .ok_or(CliError::ProfileNotFound(id))?;
        Ok(())
    }

    // ---- API Key 管理 ----

    /// 添加 EncryptedRust 模式 API Key (明文 key + 自动加密)
    pub fn add_encrypted_key(
        &self,
        provider: impl Into<String>,
        label: impl Into<String>,
        plaintext_key: &str,
    ) -> Result<ApiKeySummary, CliError> {
        let encrypted = encrypt(plaintext_key, &self.master_key)?;
        let key = ApiKey::new_encrypted(provider, label, encrypted);
        let summary = key.summary();
        self.api_keys.write().unwrap().insert(key.id, key);
        Ok(summary)
    }

    /// 添加 EnvironmentVar 模式 API Key (只存 env 变量名)
    pub fn add_env_var_key(
        &self,
        provider: impl Into<String>,
        label: impl Into<String>,
        env_var_name: impl Into<String>,
    ) -> Result<ApiKeySummary, CliError> {
        // 验证 env 存在
        let name = env_var_name.into();
        if std::env::var(&name).is_err() {
            return Err(CliError::EnvVarMissing(name));
        }
        let key = ApiKey::new_env_var(provider, label, name);
        let summary = key.summary();
        self.api_keys.write().unwrap().insert(key.id, key);
        Ok(summary)
    }

    pub fn list_api_keys(&self) -> Vec<ApiKeySummary> {
        self.api_keys
            .read()
            .unwrap()
            .values()
            .map(|k| k.summary())
            .collect()
    }

    /// 解析 API Key 为明文 (供 CLI/HTTP 调用使用, 不返前端)
    pub fn resolve_key(&self, key_id: Uuid) -> Result<String, CliError> {
        let key = self
            .api_keys
            .read()
            .unwrap()
            .get(&key_id)
            .cloned()
            .ok_or(CliError::ApiKeyMissing(key_id.to_string()))?;
        match key.mode {
            ApiKeyMode::EncryptedRust => {
                let encrypted = key
                    .encrypted_value
                    .as_ref()
                    .ok_or(CliError::ApiKeyMissing("encrypted_value".into()))?;
                decrypt(encrypted, &self.master_key)
            }
            ApiKeyMode::EnvironmentVar => {
                let name = key
                    .env_var_name
                    .as_ref()
                    .ok_or(CliError::EnvVarMissing("<unknown>".into()))?;
                std::env::var(name).map_err(|_| CliError::EnvVarMissing(name.clone()))
            }
        }
    }

    pub fn delete_api_key(&self, id: Uuid) -> Result<(), CliError> {
        self.api_keys
            .write()
            .unwrap()
            .remove(&id)
            .ok_or(CliError::ApiKeyMissing(id.to_string()))?;
        Ok(())
    }
}

// ---- AES-256-GCM 加密 ----

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::Engine;

pub fn encrypt(plaintext: &str, master_key: &[u8; 32]) -> Result<String, CliError> {
    use rand::RngCore;
    let key = Key::<Aes256Gcm>::from_slice(master_key);
    let cipher = Aes256Gcm::new(key);
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| CliError::EncryptionFailed(e.to_string()))?;
    // 格式: base64(nonce || ciphertext)
    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    Ok(base64::engine::general_purpose::STANDARD.encode(combined))
}

pub fn decrypt(encoded: &str, master_key: &[u8; 32]) -> Result<String, CliError> {
    let combined = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| CliError::DecryptionOrBase64(e.to_string()))?;
    if combined.len() < 12 {
        return Err(CliError::DecryptionFailed);
    }
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let key = Key::<Aes256Gcm>::from_slice(master_key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CliError::DecryptionFailed)?;
    String::from_utf8(plaintext).map_err(|_| CliError::DecryptionFailed)
}

// =====================================================================
// 6. invariant
// =====================================================================

/// INV-CLI-01: Profile id 全局唯一
pub fn inv_01_profile_unique(profiles: &[CliProfile]) -> bool {
    let mut seen = std::collections::HashSet::new();
    for p in profiles {
        if !seen.insert(p.id) {
            return false;
        }
    }
    true
}

/// INV-CLI-02: API Key id 全局唯一
pub fn inv_02_key_unique(keys: &[ApiKey]) -> bool {
    let mut seen = std::collections::HashSet::new();
    for k in keys {
        if !seen.insert(k.id) {
            return false;
        }
    }
    true
}

/// INV-CLI-03: EncryptedRust 模式必带密文, EnvironmentVar 必带 env_var_name
pub fn inv_03_key_mode_complete(key: &ApiKey) -> bool {
    match key.mode {
        ApiKeyMode::EncryptedRust => {
            key.encrypted_value.is_some() && key.encrypted_value.as_ref().unwrap().len() > 0
        }
        ApiKeyMode::EnvironmentVar => {
            key.env_var_name.is_some() && !key.env_var_name.as_ref().unwrap().is_empty()
        }
    }
}

/// INV-CLI-04: API Agent (OpenClaw / Hermes) 必须有 URL command
pub fn inv_04_api_agent_url(kind: CliKind, command: &str) -> bool {
    if kind.is_api_agent() {
        command.starts_with("http://") || command.starts_with("https://")
    } else {
        !command.is_empty() // CLI agent 必带非空命令
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_master_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        for i in 0..32 {
            key[i] = i as u8;
        }
        key
    }

    #[test]
    fn test_cli_kind_all_builtin_count() {
        assert_eq!(CliKind::all_builtin().len(), 6);
    }

    #[test]
    fn test_cli_kind_is_api_agent() {
        assert!(CliKind::OpenClaw.is_api_agent());
        assert!(CliKind::Hermes.is_api_agent());
        assert!(!CliKind::Claude.is_api_agent());
        assert!(!CliKind::Codex.is_api_agent());
    }

    #[test]
    fn test_cli_profile_new_builtin() {
        let p = CliProfile::new_builtin(CliKind::Claude);
        assert_eq!(p.kind, CliKind::Claude);
        assert!(p.enabled);
        assert!(!p.command.is_empty());
    }

    #[test]
    fn test_api_key_encrypted_mode() {
        let key = ApiKey::new_encrypted("anthropic", "primary", "abc123");
        assert_eq!(key.mode, ApiKeyMode::EncryptedRust);
        assert!(inv_03_key_mode_complete(&key));
    }

    #[test]
    fn test_api_key_env_var_mode() {
        let key = ApiKey::new_env_var("openai", "primary", "OPENAI_API_KEY");
        assert_eq!(key.mode, ApiKeyMode::EnvironmentVar);
        assert!(inv_03_key_mode_complete(&key));
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = test_master_key();
        let plaintext = "sk-1234567890abcdef";
        let encrypted = encrypt(plaintext, &key).unwrap();
        assert_ne!(encrypted, plaintext);
        let decrypted = decrypt(&encrypted, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_different_nonce_each_time() {
        let key = test_master_key();
        let plaintext = "same-plaintext";
        let e1 = encrypt(plaintext, &key).unwrap();
        let e2 = encrypt(plaintext, &key).unwrap();
        assert_ne!(e1, e2, "AES-GCM nonce must be random");
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let key1 = test_master_key();
        let key2 = [0xFFu8; 32];
        let encrypted = encrypt("secret", &key1).unwrap();
        assert!(decrypt(&encrypted, &key2).is_err());
    }

    #[test]
    fn test_cli_service_seeds_6_builtin() {
        let svc = CliService::new(test_master_key());
        svc.seed_builtin_profiles();
        let profiles = svc.list_profiles();
        assert_eq!(profiles.len(), 6);
    }

    #[test]
    fn test_cli_service_add_encrypted_key() {
        let svc = CliService::new(test_master_key());
        let summary = svc
            .add_encrypted_key("anthropic", "primary", "sk-test-123")
            .unwrap();
        assert_eq!(summary.mode, ApiKeyMode::EncryptedRust);
        assert_eq!(summary.provider, "anthropic");
        // 验证: resolve 出明文
        let keys = svc.list_api_keys();
        let resolved = svc.resolve_key(keys[0].id).unwrap();
        assert_eq!(resolved, "sk-test-123");
    }

    #[test]
    fn test_cli_service_add_env_var_key() {
        let svc = CliService::new(test_master_key());
        std::env::set_var("STAR_TEST_KEY", "env-value-456");
        let summary = svc
            .add_env_var_key("openai", "test", "STAR_TEST_KEY")
            .unwrap();
        assert_eq!(summary.mode, ApiKeyMode::EnvironmentVar);
        let keys = svc.list_api_keys();
        let resolved = svc.resolve_key(keys[0].id).unwrap();
        assert_eq!(resolved, "env-value-456");
        std::env::remove_var("STAR_TEST_KEY");
    }

    #[test]
    fn test_cli_service_add_env_var_missing() {
        let svc = CliService::new(test_master_key());
        let r = svc.add_env_var_key("x", "y", "NON_EXISTENT_VAR_9999");
        assert!(matches!(r, Err(CliError::EnvVarMissing(_))));
    }

    #[test]
    fn test_task_window_add_close_tab() {
        let wt_id = Uuid::new_v4();
        let profile_id = Uuid::new_v4();
        let mut win = TaskWindow::new("Test", wt_id, profile_id);
        let tab = TaskTab {
            id: Uuid::new_v4(),
            window_id: win.id,
            profile_id,
            label: "tab 1".into(),
            state: TabState::Running,
            last_output: String::new(),
            started_at: chrono::Utc::now(),
            finished_at: None,
            exit_code: None,
        };
        let tab_id = tab.id;
        win.add_tab(tab).unwrap();
        assert_eq!(win.tabs.len(), 1);
        assert_eq!(win.active_tab_id, Some(tab_id));
        win.close_tab(tab_id).unwrap();
        assert_eq!(win.tabs.len(), 0);
        assert_eq!(win.active_tab_id, None);
    }

    #[test]
    fn test_inv_01_profile_unique() {
        let p1 = CliProfile::new_builtin(CliKind::Claude);
        let p2 = CliProfile::new_builtin(CliKind::Codex);
        assert!(inv_01_profile_unique(&[p1.clone(), p2.clone()]));
        let mut p3 = p2.clone();
        p3.name = "dup".into();
        assert!(!inv_01_profile_unique(&[p1, p3]));
    }

    #[test]
    fn test_inv_04_api_agent_url() {
        assert!(inv_04_api_agent_url(
            CliKind::OpenClaw,
            "https://api.openclaw.dev/v1"
        ));
        assert!(!inv_04_api_agent_url(CliKind::OpenClaw, "not-a-url"));
        assert!(inv_04_api_agent_url(CliKind::Claude, "claude"));
        assert!(!inv_04_api_agent_url(CliKind::Claude, ""));
    }
}
