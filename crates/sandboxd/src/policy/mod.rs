/*
@cypher schema=1 source_sha256=1be3279e210dee83122edadc2934a46036a07edbe96b87df749276ae7e0b73fe
MERGE (self:File {path:"crates/sandboxd/src/policy/mod.rs"})
@endcypher
*/
//! `sandboxd` policy 模块 — 4 维隔离策略 (per FR-3, DD-SANDBOX-002 §2.1)
//!
//! - [`resource`](resource) 资源维 (per SANDBOX-001 v0.2 §3.1 SandboxLimits, 100% 复用)
//! - [`network`](network) 网络维 (域名 allowlist + CIDR ip_allowlist, per 决策点 D-3)
//! - [`fs`](fs) 文件系统维 (path allowlist + glob, per 决策点 D-4)
//! - [`capability`](capability) capability 维 (Linux 优先 + 静态字典, per FR-3.4)
//!
//! ## 5 域 policy 模板 (per 决策点 D-10)
//!
//! [`Domain`] 枚举提供 player / economy / match / social / admin 5 域默认策略,
//! player 域走标准默认值, admin 域放宽 (内存 4GB + 更多 allowlist 域名),
//! 模板值见 [`SessionSpec::for_domain`]。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod capability;
pub mod fs;
pub mod network;
pub mod resource;

pub use capability::{CapabilityPolicy, LinuxCapabilityMode};
pub use fs::{FsMode, FsPathMode, FsPathRule, FsPolicy};
pub use network::{NetworkMode, NetworkPolicy};
pub use resource::ResourcePolicy;

/// 4 维隔离策略快照 (per FR-3 + DD §2.1 `SessionSpec`)
///
/// CreateSession 时冻结 (policy_snapshot), 后续 policy 变更不影响已创建 session
/// (per INV-SBX-02)。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SessionSpec {
    /// resource 维 (内存 / CPU / 进程数 / 级联 kill)
    pub resource: ResourcePolicy,
    /// network 维 (出站 allowlist)
    pub network: NetworkPolicy,
    /// fs 维 (path allowlist)
    pub fs: FsPolicy,
    /// capability 维 (Linux capability / Windows privilege)
    pub capability: CapabilityPolicy,
}

/// 5 域 Lead 分类 (per 决策点 D-10 + 守门 #3 8/21 JST 5 域独立 Lead)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Domain {
    /// player 域 (标准默认)
    Player,
    /// economy 域 (标准默认)
    Economy,
    /// match 域 (标准默认)
    Match,
    /// social 域 (标准默认)
    Social,
    /// admin 域 (放宽: 4GB 内存 + 更多 allowlist)
    Admin,
}

impl Domain {
    /// 5 域全枚举 (供测试 / 遍历)
    pub const ALL: [Domain; 5] = [
        Domain::Player,
        Domain::Economy,
        Domain::Match,
        Domain::Social,
        Domain::Admin,
    ];
}

impl SessionSpec {
    /// 5 域 policy 模板 (per D-10: player 域 vs admin 域默认值不同)
    ///
    /// - player / economy / match / social: 标准默认 (1GB / 50% CPU / 100 procs / D-3+D-4 allowlist)
    /// - admin: 放宽 (4GB 内存 / 100% CPU, allowlist 追加 `docs.rs` / `static.rust-lang.org` / `api.github.com`)
    pub fn for_domain(domain: Domain) -> Self {
        let mut spec = Self::default();
        if domain == Domain::Admin {
            spec.resource.memory_mb = 4096;
            spec.resource.cpu_percent = 100;
            spec.network.allowlist.extend([
                "docs.rs".to_string(),
                "static.rust-lang.org".to_string(),
                "api.github.com".to_string(),
            ]);
        }
        spec
    }

    /// 校验策略合法性 (per DD §3.1 时序点 1 fail-fast)
    ///
    /// - CPU 百分比 > 100 → 非法
    /// - ALLOWLIST 模式网络策略 allowlist 为空 → 非法 (allowlist 空即全阻断, 显式用 BLOCK_ALL)
    /// - fs 规则 path 为空 → 非法
    /// - ip_allowlist 含非法 CIDR → 非法
    pub fn validate(&self) -> Result<(), crate::error::SandboxdError> {
        use crate::error::SandboxdError;
        if self.resource.cpu_percent > 100 {
            return Err(SandboxdError::InvalidPolicy(format!(
                "cpu_percent {} > 100",
                self.resource.cpu_percent
            )));
        }
        if self.network.mode == NetworkMode::Allowlist && self.network.allowlist.is_empty() {
            return Err(SandboxdError::InvalidPolicy(
                "network mode=ALLOWLIST requires non-empty allowlist (use BLOCK_ALL instead)"
                    .to_string(),
            ));
        }
        for cidr in &self.network.ip_allowlist {
            cidr.parse::<ipnet::IpNet>()
                .map_err(|e| SandboxdError::InvalidPolicy(format!("invalid CIDR '{cidr}': {e}")))?;
        }
        for rule in &self.fs.allowlist {
            if rule.path.is_empty() {
                return Err(SandboxdError::InvalidPolicy(
                    "fs allowlist rule path must not be empty".to_string(),
                ));
            }
        }
        Ok(())
    }
}

/// 内部便利: session 三元组 (tenant / workspace / actor, per 守门 #13 RLS 13 類必携)
///
/// v0.1 从服务端配置注入 (config.actor), client 端 actor 传播 v0.2 拍摄 (per 缺口 #1)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorIds {
    /// tenant_id (RLS 13 類)
    pub tenant_id: Uuid,
    /// workspace_id (RLS 13 類)
    pub workspace_id: Uuid,
    /// actor_id (RLS 13 類 + per D-7 未来注入 ARG Edge)
    pub actor_id: Uuid,
}

impl Default for ActorIds {
    fn default() -> Self {
        Self {
            tenant_id: Uuid::nil(),
            workspace_id: Uuid::nil(),
            actor_id: Uuid::nil(),
        }
    }
}

// ===== SessionSpec proto 互转 (per BD-002 §3.1 SessionSpec message) =====

impl From<&SessionSpec> for crate::proto::SessionSpec {
    fn from(p: &SessionSpec) -> Self {
        Self {
            resource: Some((&p.resource).into()),
            network: Some((&p.network).into()),
            fs: Some((&p.fs).into()),
            capability: Some((&p.capability).into()),
        }
    }
}

impl From<&crate::proto::SessionSpec> for SessionSpec {
    fn from(p: &crate::proto::SessionSpec) -> Self {
        Self {
            resource: p.resource.as_ref().map(Into::into).unwrap_or_default(),
            network: p.network.as_ref().map(Into::into).unwrap_or_default(),
            fs: p.fs.as_ref().map(Into::into).unwrap_or_default(),
            capability: p.capability.as_ref().map(Into::into).unwrap_or_default(),
        }
    }
}
