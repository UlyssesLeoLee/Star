//! `sandboxd` 配置模块 (per FR-1.1, DD-SANDBOX-002 §2.3)
//!
//! YAML 配置文件加载 + 校验:
//! - Linux: `/etc/sandboxd/config.yaml`
//! - Windows: `$ProgramData/sandboxd/config.yaml`
//! - 缺文件走 [`SandboxdConfig::default`] (开发模式, 默认端口 50051 gRPC + 50052 HTTP)

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;

use crate::error::SandboxdError;
use crate::policy::{ActorIds, FsMode, FsPathMode, FsPathRule, LinuxCapabilityMode, NetworkMode};
use uuid::Uuid;

/// 顶层配置 (per BD-002 §2.3)
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct SandboxdConfig {
    /// server 段
    pub server: ServerConfig,
    /// isolation 段 (4 维默认值)
    pub isolation: IsolationConfig,
    /// observability 段
    pub observability: ObservabilityConfig,
    /// failover 段 (FR-7.1)
    pub failover: FailoverConfig,
    /// actor 三元组 (RLS 13 類必携, v0.1 服务端注入)
    pub actor: ActorConfig,
}

/// server 段 (per FR-1.1 / FR-1.2 / FR-1.3)
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// gRPC 监听端口 (默认 50051, per FR-2.2)
    pub grpc_port: u16,
    /// HTTP 监听端口 (/metrics + /healthz, 默认 50052)
    ///
    /// 偏差标注: BD §2.3 期望 metrics/health 跟 gRPC 同端口 (50051), tonic 0.12 无
    /// `accept_http1` (0.14 才引入), 故 v0.1 同进程不同端口, "跟 gRPC 同进程" 保持。
    pub http_port: u16,
    /// 最大并发 session 数 (默认 100, per FR-1.2)
    pub max_concurrent_sessions: usize,
    /// session 默认超时 (秒, 默认 1800 = 30 min, per FR-4.3)
    pub default_session_timeout_sec: u32,
    /// 优雅停机等待 running session 的宽限期 (秒, 默认 30, per FR-1.3)
    pub grace_shutdown_sec: u64,
}

/// isolation 段 (4 维默认值, per SRS §3.2 表)
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct IsolationConfig {
    /// resource 维默认
    pub resource: ResourceConfig,
    /// network 维默认
    pub network: NetworkConfig,
    /// fs 维默认
    pub fs: FsConfig,
    /// capability 维默认
    pub capability: CapabilityConfig,
}

/// resource 维默认 (per FR-3.1: 1GB / 50% CPU / 100 procs / kill_on_parent_exit=true)
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ResourceConfig {
    /// 默认内存 (MB)
    pub default_memory_mb: u32,
    /// 默认 CPU 百分比
    pub default_cpu_percent: u32,
    /// 默认最大进程数
    pub default_max_processes: u32,
    /// 父进程退出级联 kill
    pub kill_on_parent_exit: bool,
}

/// network 维默认 (per D-3 allowlist 默认)
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    /// 默认模式
    pub default_mode: NetworkMode,
    /// 默认域名 allowlist
    pub default_allowlist: Vec<String>,
    /// 默认 IP allowlist (CIDR)
    pub default_ip_allowlist: Vec<String>,
}

/// fs 维默认 (per D-4 path allowlist 默认)
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct FsConfig {
    /// 默认模式
    pub default_mode: FsMode,
    /// 默认 path allowlist
    pub default_allowlist: Vec<FsPathRule>,
}

/// capability 维默认 (per FR-3.4: drop ALL, 仅保留 NET_BIND_SERVICE)
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CapabilityConfig {
    /// Linux 默认模式
    pub linux_default: LinuxCapabilityMode,
    /// Windows 默认 (v0.1: "drop_se_debug" 语义 → spawn 前提权工具拦截)
    pub windows_default: String,
}

/// observability 段 (per FR-6.1 / FR-6.2)
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ObservabilityConfig {
    /// Prometheus /metrics 开关
    pub prometheus_enabled: bool,
    /// audit log 文件路径 (JSON Lines, per NFR-M-3; 空 = 不写文件)
    pub audit_log_file: String,
    /// PG audit log URL (空 = 不启用 PG, per FR-6.2)
    pub audit_log_pg_url: String,
    /// audit 保留天数 (默认 90, per BR-4)
    pub audit_log_retention_days: u32,
}

/// failover 段 (per FR-7.1 跟 SANDBOX-001 v0.2 兼容)
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct FailoverConfig {
    /// 启用 v0.2 subprocess 降级
    pub enable_v0_2_subprocess_fallback: bool,
    /// 降级路径 (scripts/automation/guardian/sandbox.py)
    pub fallback_path: PathBuf,
    /// 降级事件是否推送 root session 通知 (NFR-O-3)
    pub root_session_notify: bool,
}

/// actor 三元组 (per 守门 #13 RLS 13 類必携, v0.1 服务端注入; client 传播 v0.2)
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct ActorConfig {
    /// tenant_id (UUID 字符串, 空 = nil)
    pub tenant_id: String,
    /// workspace_id (UUID 字符串, 空 = nil)
    pub workspace_id: String,
    /// actor_id (UUID 字符串, 空 = nil; per D-7 未来注入 ARG Edge)
    pub actor_id: String,
}

impl ActorConfig {
    /// 转 [`ActorIds`] (非法 UUID → nil, 不阻断启动)
    pub fn to_ids(&self) -> ActorIds {
        ActorIds {
            tenant_id: self.tenant_id.parse().unwrap_or_else(|_| Uuid::nil()),
            workspace_id: self.workspace_id.parse().unwrap_or_else(|_| Uuid::nil()),
            actor_id: self.actor_id.parse().unwrap_or_else(|_| Uuid::nil()),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            grpc_port: 50051,
            http_port: 50052,
            max_concurrent_sessions: 100,
            default_session_timeout_sec: 1800,
            grace_shutdown_sec: 30,
        }
    }
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            default_memory_mb: 1024,
            default_cpu_percent: 50,
            default_max_processes: 100,
            kill_on_parent_exit: true,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            default_mode: NetworkMode::Allowlist,
            default_allowlist: vec![
                "github.com".to_string(),
                "*.github.com".to_string(),
                "crates.io".to_string(),
                "*.crates.io".to_string(),
                "static.crates.io".to_string(),
                "index.crates.io".to_string(),
                "sh.rustup.rs".to_string(),
            ],
            default_ip_allowlist: vec![],
        }
    }
}

impl Default for FsConfig {
    fn default() -> Self {
        Self {
            default_mode: FsMode::Allowlist,
            default_allowlist: vec![
                FsPathRule::new("/worktree/**", FsPathMode::Read),
                FsPathRule::new("/worktree/.worktree-cache/**", FsPathMode::ReadWrite),
                FsPathRule::new("/tmp/**", FsPathMode::ReadWrite),
            ],
        }
    }
}

impl Default for CapabilityConfig {
    fn default() -> Self {
        Self {
            linux_default: LinuxCapabilityMode::DropAll,
            windows_default: "drop_se_debug".to_string(),
        }
    }
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            prometheus_enabled: true,
            audit_log_file: "sandboxd-audit.jsonl".to_string(),
            audit_log_pg_url: String::new(),
            audit_log_retention_days: 90,
        }
    }
}

impl Default for FailoverConfig {
    fn default() -> Self {
        Self {
            enable_v0_2_subprocess_fallback: true,
            fallback_path: PathBuf::from("scripts/automation/guardian/sandbox.py"),
            root_session_notify: true,
        }
    }
}

impl SandboxdConfig {
    /// YAML 加载 (per FR-1.1 配置文件; 缺文件 → 默认配置)
    pub fn from_yaml(path: &Path) -> Result<Self, SandboxdError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| SandboxdError::Internal(format!("read config {}: {e}", path.display())))?;
        let config: SandboxdConfig = serde_yaml::from_str(&content)
            .map_err(|e| SandboxdError::Internal(format!("parse config: {e}")))?;
        config.validate()?;
        Ok(config)
    }

    /// 加载或回退默认 (配置文件不存在 → 默认 + warn, per FR-7.1 fail-open 精神)
    pub fn load_or_default(path: &Path) -> SandboxdConfig {
        match Self::from_yaml(path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("config load failed ({e}), using defaults");
                SandboxdConfig::default()
            }
        }
    }

    /// 校验 (per DD §2.3 validate 业务规则)
    pub fn validate(&self) -> Result<(), SandboxdError> {
        if self.server.max_concurrent_sessions == 0 {
            return Err(SandboxdError::InvalidPolicy(
                "max_concurrent_sessions must be > 0".to_string(),
            ));
        }
        if self.observability.audit_log_retention_days < 1 {
            return Err(SandboxdError::InvalidPolicy(
                "audit_log_retention_days must be >= 1".to_string(),
            ));
        }
        if self.observability.audit_log_file.is_empty()
            && self.observability.audit_log_pg_url.is_empty()
        {
            return Err(SandboxdError::InvalidPolicy(
                "audit 必须至少配置一个 sink (audit_log_file 或 audit_log_pg_url), per FR-7.2 fail-closed"
                    .to_string(),
            ));
        }
        if self.server.grpc_port == self.server.http_port {
            return Err(SandboxdError::InvalidPolicy(
                "grpc_port and http_port must differ".to_string(),
            ));
        }
        Ok(())
    }

    /// gRPC 监听地址 (per FR-2.2 默认 127.0.0.1:50051)
    pub fn grpc_addr(&self) -> String {
        format!("127.0.0.1:{}", self.server.grpc_port)
    }

    /// HTTP 监听地址 (/metrics + /healthz)
    pub fn http_addr(&self) -> String {
        format!("127.0.0.1:{}", self.server.http_port)
    }

    /// gRPC 连接超时 (client 侧探测, per FR-7.1 降级判断)
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_millis(100)
    }
}
