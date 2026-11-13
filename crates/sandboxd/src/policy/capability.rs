//! policy::capability — capability 维隔离策略 (per FR-3.4, 决策点 D-8 已拍板)
//!
//! - Linux: libcap 三模式 (DropAll / AllowNetBindService / Custom), 后端经 `capsh` 应用
//! - Windows / macOS: v0.1 走 sandbox 后端隐式 drop (per SRS §9 缺口 #3 / D-8 立即实装 Win+macOS
//!   实装于 backend 层, 本策略模块提供 privilege 静态字典与 spawn 前高危命令拦截)
//! - 静态字典 [`KNOWN_CAPABILITIES`] 跟 `sandbox_capability` M 表 seed 1:1 (per DD §6.4)

use serde::{Deserialize, Serialize};

use crate::error::SandboxdError;

/// Linux capability 模式 (per BD §3.1 CapabilityPolicy.LinuxMode)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LinuxCapabilityMode {
    /// drop ALL (默认)
    #[default]
    DropAll,
    /// drop all 后仅保留 NET_BIND_SERVICE (子代理 port listen 需求)
    AllowNetBindService,
    /// 自定义保留列表 (custom_capabilities)
    Custom,
}

/// capability 维策略 (per FR-3.4)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityPolicy {
    /// Linux 模式 (默认 DropAll per D-3)
    pub linux_mode: LinuxCapabilityMode,
    /// CUSTOM 模式保留列表 (e.g. `CAP_NET_RAW`)
    pub custom_capabilities: Vec<String>,
}

impl Default for CapabilityPolicy {
    fn default() -> Self {
        Self {
            linux_mode: LinuxCapabilityMode::DropAll,
            custom_capabilities: vec![],
        }
    }
}

/// 平台 (per sandbox_capability M 表 platform 枚举)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapPlatform {
    /// Linux syscall / capability
    Linux,
    /// Windows privilege
    Windows,
    /// macOS entitlement / operation
    MacOs,
}

/// 风险等级 (per sandbox_capability M 表 risk_level 枚举)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 危急
    Critical,
}

/// 默认动作 (per sandbox_capability M 表 default_action 枚举)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DefaultAction {
    /// 放行
    Allow,
    /// 拒绝
    Deny,
    /// 询问
    Ask,
}

/// 已知 capability 静态字典条目 (跟 `sandbox_capability` M 表 1:1)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityEntry {
    /// syscall 名 (Linux) / privilege 名 (Windows)
    pub syscall_name: &'static str,
    /// 平台
    pub platform: CapPlatform,
    /// 风险等级
    pub risk_level: RiskLevel,
    /// 说明
    pub description: &'static str,
    /// 默认动作
    pub default_action: DefaultAction,
}

/// 已知 capability 字典 (per DD §6.4 seed, Linux 40 + Windows 8 + macOS 3)
///
/// 高风险 (Critical/High) 项在 capability 维后端隐式 drop 或 spawn 前拦截 (per FR-3.4)。
pub const KNOWN_CAPABILITIES: &[CapabilityEntry] = &[
    // ===== Linux (40 标准 CAP_*) =====
    CapabilityEntry {
        syscall_name: "CAP_CHOWN",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Medium,
        description: "修改文件属主",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_DAC_OVERRIDE",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Critical,
        description: "绕过文件权限检查",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_DAC_READ_SEARCH",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::High,
        description: "绕过文件读/目录搜索权限",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_FOWNER",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::High,
        description: "绕过文件属主检查",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_FSETID",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::High,
        description: "设置文件 setuid/setgid 位",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_KILL",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::High,
        description: "向任意进程发信号",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_SETGID",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Medium,
        description: "任意组 ID 变更",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_SETUID",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Critical,
        description: "任意用户 ID 变更 (提权)",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_SETPCAP",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Critical,
        description: "传递 capability 给其他进程",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_LINUX_IMMUTABLE",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::High,
        description: "设置 immutable 文件属性",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_NET_BIND_SERVICE",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Low,
        description: "绑定 <1024 端口 (子代理 port listen)",
        default_action: DefaultAction::Allow,
    },
    CapabilityEntry {
        syscall_name: "CAP_NET_BROADCAST",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Low,
        description: "网络广播",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_NET_ADMIN",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Critical,
        description: "网络配置 (ifconfig/iptables)",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_NET_RAW",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Medium,
        description: "原始网络包访问 (ping)",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_IPC_LOCK",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Medium,
        description: "锁定共享内存页",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_IPC_OWNER",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Medium,
        description: "IPC 对象属主检查绕过",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_SYS_MODULE",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Critical,
        description: "内核模块加载/卸载",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_SYS_RAWIO",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Critical,
        description: "原始 I/O 端口访问",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_SYS_CHROOT",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Medium,
        description: "chroot 调用",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_SYS_PTRACE",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Critical,
        description: "任意进程调试 (内存读取)",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_SYS_PACCT",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Medium,
        description: "进程记账配置",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_SYS_ADMIN",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Critical,
        description: "系统管理操作 (mount 等)",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_SYS_BOOT",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Critical,
        description: "系统重启",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_SYS_NICE",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Medium,
        description: "进程优先级提升",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_SYS_RESOURCE",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::High,
        description: "资源限制绕过 (rlimit)",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_SYS_TIME",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::High,
        description: "系统时钟修改",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_SYS_TTY_CONFIG",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::High,
        description: "tty 设备配置",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_MKNOD",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Medium,
        description: "创建设备节点",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_LEASE",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Low,
        description: "文件租约",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_AUDIT_WRITE",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Medium,
        description: "写内核审计日志",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_AUDIT_CONTROL",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Critical,
        description: "内核审计配置",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_SETFCAP",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Critical,
        description: "设置文件 capability",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_MAC_OVERRIDE",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Critical,
        description: "MAC 强制访问控制绕过",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_MAC_ADMIN",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Critical,
        description: "MAC 配置管理",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_SYSLOG",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Critical,
        description: "内核日志读取 (dmesg)",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_WAKE_ALARM",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Low,
        description: "唤醒闹钟",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_BLOCK_SUSPEND",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Low,
        description: "阻止系统挂起",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_AUDIT_READ",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Medium,
        description: "读内核审计日志",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_PERFMON",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::High,
        description: "性能监控 (eBPF 等)",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_BPF",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::Critical,
        description: "BPF 程序加载",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "CAP_CHECKPOINT_RESTORE",
        platform: CapPlatform::Linux,
        risk_level: RiskLevel::High,
        description: "进程检查点恢复",
        default_action: DefaultAction::Deny,
    },
    // ===== Windows (8 高危 privilege) =====
    CapabilityEntry {
        syscall_name: "SeDebugPrivilege",
        platform: CapPlatform::Windows,
        risk_level: RiskLevel::Critical,
        description: "任意进程调试/内存读取",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "SeBackupPrivilege",
        platform: CapPlatform::Windows,
        risk_level: RiskLevel::Critical,
        description: "绕过 ACL 备份读取",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "SeRestorePrivilege",
        platform: CapPlatform::Windows,
        risk_level: RiskLevel::Critical,
        description: "绕过 ACL 恢复写入",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "SeTakeOwnershipPrivilege",
        platform: CapPlatform::Windows,
        risk_level: RiskLevel::Critical,
        description: "任意对象所有权接管",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "SeLoadDriverPrivilege",
        platform: CapPlatform::Windows,
        risk_level: RiskLevel::Critical,
        description: "内核驱动加载",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "SeTcbPrivilege",
        platform: CapPlatform::Windows,
        risk_level: RiskLevel::Critical,
        description: "可信计算基 (最高特权)",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "SeCreateTokenPrivilege",
        platform: CapPlatform::Windows,
        risk_level: RiskLevel::Critical,
        description: "创建主令牌 (提权)",
        default_action: DefaultAction::Deny,
    },
    CapabilityEntry {
        syscall_name: "SeAssignPrimaryTokenPrivilege",
        platform: CapPlatform::Windows,
        risk_level: RiskLevel::High,
        description: "替换进程令牌",
        default_action: DefaultAction::Deny,
    },
    // ===== macOS (3 常见) =====
    CapabilityEntry {
        syscall_name: "com.apple.security.network.client",
        platform: CapPlatform::MacOs,
        risk_level: RiskLevel::Low,
        description: "出站网络 (entitlement)",
        default_action: DefaultAction::Ask,
    },
    CapabilityEntry {
        syscall_name: "com.apple.security.files.user-selected.read-write",
        platform: CapPlatform::MacOs,
        risk_level: RiskLevel::Medium,
        description: "用户选择文件读写 (entitlement)",
        default_action: DefaultAction::Ask,
    },
    CapabilityEntry {
        syscall_name: "com.apple.security.cs.disable-library-validation",
        platform: CapPlatform::MacOs,
        risk_level: RiskLevel::High,
        description: "禁用代码签名校验",
        default_action: DefaultAction::Deny,
    },
];

impl CapabilityPolicy {
    /// 构造 capability 策略
    pub fn new(linux_mode: LinuxCapabilityMode, custom_capabilities: Vec<String>) -> Self {
        Self {
            linux_mode,
            custom_capabilities,
        }
    }

    /// 校验 (CUSTOM 模式必须给保留列表; 未知 capability 名 → 非法)
    pub fn validate(&self) -> Result<(), SandboxdError> {
        if self.linux_mode == LinuxCapabilityMode::Custom && self.custom_capabilities.is_empty() {
            return Err(SandboxdError::InvalidPolicy(
                "capability mode=CUSTOM requires non-empty custom_capabilities".to_string(),
            ));
        }
        for cap in &self.custom_capabilities {
            if !KNOWN_CAPABILITIES
                .iter()
                .any(|e| e.syscall_name == cap.as_str())
            {
                return Err(SandboxdError::InvalidPolicy(format!(
                    "unknown capability '{cap}'"
                )));
            }
        }
        Ok(())
    }

    /// 字典查询: 已知 capability 的风险等级 (未知 → 按 Critical fail-closed 处理)
    pub fn lookup(&self, name: &str) -> RiskLevel {
        KNOWN_CAPABILITIES
            .iter()
            .find(|e| e.syscall_name == name)
            .map(|e| e.risk_level)
            .unwrap_or(RiskLevel::Critical)
    }
}

// ===== proto 互转 (per BD-002 §3.1 CapabilityPolicy message) =====

impl From<&CapabilityPolicy> for crate::proto::CapabilityPolicy {
    fn from(p: &CapabilityPolicy) -> Self {
        let linux_mode = match p.linux_mode {
            LinuxCapabilityMode::DropAll => {
                crate::proto::capability_policy::LinuxMode::DropAll as i32
            }
            LinuxCapabilityMode::AllowNetBindService => {
                crate::proto::capability_policy::LinuxMode::AllowNetBindService as i32
            }
            LinuxCapabilityMode::Custom => {
                crate::proto::capability_policy::LinuxMode::Custom as i32
            }
        };
        Self {
            linux_mode,
            custom_capabilities: p.custom_capabilities.clone(),
        }
    }
}

impl From<&crate::proto::CapabilityPolicy> for CapabilityPolicy {
    fn from(p: &crate::proto::CapabilityPolicy) -> Self {
        use crate::proto::capability_policy::LinuxMode;
        let linux_mode = match LinuxMode::try_from(p.linux_mode).unwrap_or(LinuxMode::DropAll) {
            LinuxMode::AllowNetBindService => LinuxCapabilityMode::AllowNetBindService,
            LinuxMode::Custom => LinuxCapabilityMode::Custom,
            LinuxMode::DropAll => LinuxCapabilityMode::DropAll,
        };
        Self {
            linux_mode,
            custom_capabilities: p.custom_capabilities.clone(),
        }
    }
}
