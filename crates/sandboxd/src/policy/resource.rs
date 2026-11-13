/*
@cypher schema=1 source_sha256=86d59719d676005854942434c29e65a39da7828000fed6d284312d7184cad761
MERGE (self:File {path:"crates/sandboxd/src/policy/resource.rs"})
@endcypher
*/
//! policy::resource — 资源维隔离策略 (per FR-3.1)
//!
//! `ResourcePolicy` = SANDBOX-001 v0.2 §3.1 `SandboxLimits` 100% 复用:
//! 内存 (MB) / CPU (%) / 进程数 / kill_on_parent_exit。
//! 默认值 per SRS §3.2 表: 1GB mem / 50% CPU / 100 procs / kill_on_parent_exit=true。

use serde::{Deserialize, Serialize};

use crate::error::SandboxdError;

/// 资源维策略 (per SANDBOX-001 v0.2 §3.1, 100% 复用)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourcePolicy {
    /// 内存上限 (MB, 0 = 不限, 默认 1024 = 1GB)
    pub memory_mb: u32,
    /// CPU 百分比 (0 = 不限, 默认 50)
    pub cpu_percent: u32,
    /// 最大进程数 (0 = 不限, 默认 100)
    pub max_processes: u32,
    /// 父进程退出时级联 kill (默认 true, per FR-1.1 级联 kill)
    pub kill_on_parent_exit: bool,
}

impl Default for ResourcePolicy {
    fn default() -> Self {
        Self {
            memory_mb: 1024,
            cpu_percent: 50,
            max_processes: 100,
            kill_on_parent_exit: true,
        }
    }
}

impl ResourcePolicy {
    /// 构造资源策略
    pub fn new(
        memory_mb: u32,
        cpu_percent: u32,
        max_processes: u32,
        kill_on_parent_exit: bool,
    ) -> Self {
        Self {
            memory_mb,
            cpu_percent,
            max_processes,
            kill_on_parent_exit,
        }
    }

    /// 校验 (per DD §3.1 fail-fast; 0 = 不限)
    pub fn validate(&self) -> Result<(), SandboxdError> {
        if self.cpu_percent > 100 {
            return Err(SandboxdError::InvalidPolicy(format!(
                "cpu_percent {} > 100",
                self.cpu_percent
            )));
        }
        Ok(())
    }
}

// ===== proto 互转 (per BD-002 §3.1 ResourcePolicy message) =====

impl From<&ResourcePolicy> for crate::proto::ResourcePolicy {
    fn from(p: &ResourcePolicy) -> Self {
        Self {
            memory_mb: p.memory_mb,
            cpu_percent: p.cpu_percent,
            max_processes: p.max_processes,
            kill_on_parent_exit: p.kill_on_parent_exit,
        }
    }
}

impl From<&crate::proto::ResourcePolicy> for ResourcePolicy {
    fn from(p: &crate::proto::ResourcePolicy) -> Self {
        Self {
            memory_mb: p.memory_mb,
            cpu_percent: p.cpu_percent,
            max_processes: p.max_processes,
            kill_on_parent_exit: p.kill_on_parent_exit,
        }
    }
}
