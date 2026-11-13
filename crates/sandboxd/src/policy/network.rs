//! policy::network — 网络维隔离策略 (per FR-3.2, 决策点 D-3 已拍板)
//!
//! 策略: **allowlist 默认** (子代理常要 git / cargo / crates.io 域名)。
//! - 域名 allowlist: 精确 + `*.domain` 通配符 (FQDN 通配符支持)
//! - ip_allowlist: CIDR 段 (per SRS §9 缺口 #6 DNS rebinding 缓解)
//! - BLOCK 模式: 命中非 allowlist 域名 → audit log + 网络 reset (RST)

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

use crate::error::SandboxdError;

/// 网络策略模式 (per BD §3.1 NetworkPolicy.Mode)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NetworkMode {
    /// allowlist (默认 per D-3): 仅放行 allowlist 域名 / CIDR
    #[default]
    Allowlist,
    /// 全阻断: 0 出站
    BlockAll,
    /// 透传: 不隔离 (审计用)
    Passthrough,
}

/// 网络维策略
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkPolicy {
    /// 模式 (默认 Allowlist per D-3)
    pub mode: NetworkMode,
    /// 域名 allowlist (FQDN 通配符: `*.github.com` 匹配任意深度子域)
    pub allowlist: Vec<String>,
    /// IP allowlist (CIDR 段, per 缺口 #6 缓解)
    pub ip_allowlist: Vec<String>,
}

impl Default for NetworkPolicy {
    fn default() -> Self {
        Self {
            mode: NetworkMode::Allowlist,
            // per SRS FR-3.2 D-3 默认 allowlist
            allowlist: vec![
                "github.com".to_string(),
                "*.github.com".to_string(),
                "crates.io".to_string(),
                "*.crates.io".to_string(),
                "static.crates.io".to_string(),
                "index.crates.io".to_string(),
                "sh.rustup.rs".to_string(),
            ],
            ip_allowlist: vec![],
        }
    }
}

/// 网络维判定结果 (per FR-3.2 BLOCK 模式派生)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkDecision {
    /// 放行 (命中 allowlist / Passthrough)
    Allowed,
    /// 阻断 (命中非 allowlist)
    Blocked {
        /// 阻断原因
        reason: String,
    },
}

impl NetworkPolicy {
    /// 构造网络策略
    pub fn new(mode: NetworkMode, allowlist: Vec<String>, ip_allowlist: Vec<String>) -> Self {
        Self {
            mode,
            allowlist,
            ip_allowlist,
        }
    }

    /// 校验域名条目 (空条目非法)
    pub fn validate(&self) -> Result<(), SandboxdError> {
        if self.allowlist.iter().any(|d| d.trim().is_empty()) {
            return Err(SandboxdError::InvalidPolicy(
                "network allowlist entry must not be empty".to_string(),
            ));
        }
        for cidr in &self.ip_allowlist {
            cidr.parse::<ipnet::IpNet>()
                .map_err(|e| SandboxdError::InvalidPolicy(format!("invalid CIDR '{cidr}': {e}")))?;
        }
        Ok(())
    }

    /// 域名判定: host 是否在 allowlist 内
    ///
    /// - `Passthrough` → 全部放行
    /// - `BlockAll` → 全部阻断
    /// - `Allowlist` → 精确匹配 / `*.domain` 后缀匹配, 未命中阻断
    pub fn check_host(&self, host: &str) -> NetworkDecision {
        match self.mode {
            NetworkMode::Passthrough => NetworkDecision::Allowed,
            NetworkMode::BlockAll => NetworkDecision::Blocked {
                reason: "network mode = BLOCK_ALL".to_string(),
            },
            NetworkMode::Allowlist => {
                if self.host_allowed(host) {
                    NetworkDecision::Allowed
                } else {
                    NetworkDecision::Blocked {
                        reason: format!("host '{host}' not in allowlist"),
                    }
                }
            }
        }
    }

    /// IP 判定: ip 是否命中 ip_allowlist (per 缺口 #6 CIDR 缓解)
    ///
    /// `Passthrough` → 放行; `BlockAll` → 阻断; `Allowlist` → 仅 ip_allowlist CIDR 内放行。
    /// 未配置 ip_allowlist 时, 域名 allowlist 无法对裸 IP 生效 → 阻断 (fail-closed per D-3)。
    pub fn check_ip(&self, ip: IpAddr) -> NetworkDecision {
        match self.mode {
            NetworkMode::Passthrough => NetworkDecision::Allowed,
            NetworkMode::BlockAll => NetworkDecision::Blocked {
                reason: "network mode = BLOCK_ALL".to_string(),
            },
            NetworkMode::Allowlist => {
                for cidr in &self.ip_allowlist {
                    if let Ok(net) = cidr.parse::<ipnet::IpNet>() {
                        if net.contains(&ip) {
                            return NetworkDecision::Allowed;
                        }
                    }
                }
                NetworkDecision::Blocked {
                    reason: format!("ip '{ip}' not in ip_allowlist"),
                }
            }
        }
    }

    /// 域名匹配: 精确 or `*.domain` 后缀 (任意深度子域)
    fn host_allowed(&self, host: &str) -> bool {
        let host = host.trim().trim_end_matches('.').to_ascii_lowercase();
        for pattern in &self.allowlist {
            let pattern = pattern.trim().trim_end_matches('.').to_ascii_lowercase();
            if pattern == host {
                return true;
            }
            if let Some(suffix) = pattern.strip_prefix("*.") {
                // *.github.com 匹配 api.github.com / a.b.github.com, 但不匹配 github.com 本身
                // (github.com 由显式条目覆盖, per D-3 默认 allowlist 双条目)
                if host != suffix && host.ends_with(&format!(".{suffix}")) {
                    return true;
                }
            }
        }
        false
    }
}

// ===== proto 互转 (per BD-002 §3.1 NetworkPolicy message) =====

impl From<&NetworkPolicy> for crate::proto::NetworkPolicy {
    fn from(p: &NetworkPolicy) -> Self {
        let mode = match p.mode {
            NetworkMode::Allowlist => crate::proto::network_policy::Mode::Allowlist as i32,
            NetworkMode::BlockAll => crate::proto::network_policy::Mode::BlockAll as i32,
            NetworkMode::Passthrough => crate::proto::network_policy::Mode::Passthrough as i32,
        };
        Self {
            mode,
            allowlist: p.allowlist.clone(),
            ip_allowlist: p.ip_allowlist.clone(),
        }
    }
}

impl From<&crate::proto::NetworkPolicy> for NetworkPolicy {
    fn from(p: &crate::proto::NetworkPolicy) -> Self {
        use crate::proto::network_policy::Mode;
        let mode = match Mode::try_from(p.mode).unwrap_or(Mode::Allowlist) {
            Mode::BlockAll => NetworkMode::BlockAll,
            Mode::Passthrough => NetworkMode::Passthrough,
            Mode::Allowlist => NetworkMode::Allowlist,
        };
        Self {
            mode,
            allowlist: p.allowlist.clone(),
            ip_allowlist: p.ip_allowlist.clone(),
        }
    }
}
