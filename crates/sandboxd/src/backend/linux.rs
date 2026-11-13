//! backend::linux — Linux 隔离后端 (per FR-5.2)
//!
//! | 维 | 实装 | 机制 |
//! |---|---|---|
//! | resource | cgroups v2 (fs 直写, 0 unsafe) | `memory.max` / `cpu.max` / `pids.max` + `cgroup.procs` |
//! | network | netns + iptables (`ip` CLI) | OUTPUT 链 BlockAll + ip_allowlist CIDR ACCEPT |
//! | fs | mount ns (`unshare -m` CLI) | allowlist bind mount (Read → `-o remount,ro`) |
//! | capability | `capsh` drop (CLI) | DropAll / AllowNetBindService / Custom |
//!
//! 需 root 特权; rootless 环境自动降级 PolicyAuditOnly 并写 audit (fail-open, per FR-7.1)。
//! 纯逻辑函数 (脚本构造 / argv 包装) 全平台编译 + 单测, 平台集成走 `#[cfg(target_os = "linux")]`。

use std::path::PathBuf;

use uuid::Uuid;

use super::CommandSpec;
use crate::policy::{CapabilityPolicy, FsPathMode, FsPolicy, NetworkPolicy, SessionSpec};

/// cgroups v2 根目录 (sandboxd 专用子树)
pub const CGROUP_ROOT: &str = "/sys/fs/cgroup/sandboxd";

/// cgroup 子目录名: `sandboxd-<session_id>` (per BD §5.2)
pub fn cgroup_path_for(session_id: &Uuid) -> PathBuf {
    PathBuf::from(CGROUP_ROOT).join(format!("sandboxd-{session_id}"))
}

/// netns 名: `sandboxd-<session_id>` (per BD §5.2)
pub fn netns_name_for(session_id: &Uuid) -> String {
    format!("sandboxd-{session_id}")
}

/// memory.max 内容 (0 = 不限 → None)
pub fn cgroup_memory_max(memory_mb: u32) -> Option<String> {
    (memory_mb > 0).then(|| format!("{memory_mb}M"))
}

/// cpu.max 内容: `"{max} 10000"` (per BD §5.2, 10000 = 1 核周期, 0 = 不限 → None)
pub fn cgroup_cpu_max(cpu_percent: u32) -> Option<String> {
    (cpu_percent > 0).then(|| format!("{} 10000", cpu_percent * 100))
}

/// pids.max 内容 (0 = 不限 → None)
pub fn cgroup_pids_max(max_processes: u32) -> Option<String> {
    (max_processes > 0).then(|| max_processes.to_string())
}

/// netns 内 iptables 初始化脚本 (per BD §5.2: OUTPUT 先 DROP 再逐条 ACCEPT)
///
/// - loopback 必放行 (进程间 IPC / 本地 DNS)
/// - ip_allowlist CIDR → ACCEPT
/// - 域名 allowlist 需 ipset + DNS 解析 (per SRS §9 缺口 #6) → v0.2 拍摄
pub fn build_netns_setup_script(policy: &NetworkPolicy) -> String {
    let mut script =
        String::from("iptables -A OUTPUT -o lo -j ACCEPT; iptables -A OUTPUT -j DROP; ");
    for cidr in &policy.ip_allowlist {
        script.push_str(&format!("iptables -A OUTPUT -d {cidr} -j ACCEPT; "));
    }
    script
}

/// capsh 包装 argv (per BD §5.2 capability 维)
///
/// - DropAll → `capsh --drop=all --`
/// - AllowNetBindService → `capsh --caps=cap_net_bind_service+ep --`
/// - Custom → `capsh --caps=<caps...>+ep --`
pub fn build_capsh_prefix(policy: &CapabilityPolicy) -> Vec<String> {
    use crate::policy::LinuxCapabilityMode;
    match policy.linux_mode {
        LinuxCapabilityMode::DropAll => {
            vec![
                "capsh".to_string(),
                "--drop=all".to_string(),
                "--".to_string(),
            ]
        }
        LinuxCapabilityMode::AllowNetBindService => vec![
            "capsh".to_string(),
            "--caps=cap_net_bind_service+ep".to_string(),
            "--".to_string(),
        ],
        LinuxCapabilityMode::Custom => vec![
            "capsh".to_string(),
            format!("--caps={}+ep", policy.custom_capabilities.join(",")),
            "--".to_string(),
        ],
    }
}

/// mount ns bind mount 脚本 (per BD §5.2 fs 维, fail-open: 单条失败不阻断)
///
/// 每条 allowlist 规则: Read → bind + remount ro; ReadWrite → bind。
/// `2>/dev/null || true` 保证源路径缺失时仍执行命令 (audit 由 sandboxd 层兜底)。
pub fn build_fs_bind_script(policy: &FsPolicy) -> String {
    let mut script = String::new();
    for rule in &policy.allowlist {
        let p = rule.path.replace('\'', "'\\''");
        script.push_str(&format!("mount --bind '{p}' '{p}' 2>/dev/null || true; "));
        if rule.mode == FsPathMode::Read {
            script.push_str(&format!(
                "mount -o remount,ro,bind '{p}' 2>/dev/null || true; "
            ));
        }
    }
    script.push_str("exec \"$@\"");
    script
}

/// Linux 完整包装 argv: netns 前缀 + mount ns 前缀 + capsh 前缀 + 原命令 (per FR-5.2)
///
/// `netns_name` 由 apply 阶段创建成功后传入 (None = netns 不可用, 降级)。
pub fn build_wrap_argv(
    policy: &SessionSpec,
    netns_name: Option<&str>,
    cmd: &CommandSpec,
) -> Vec<String> {
    let mut argv = Vec::new();
    if let Some(name) = netns_name {
        argv.extend([
            "ip".to_string(),
            "netns".to_string(),
            "exec".to_string(),
            name.to_string(),
        ]);
    }
    if policy.fs.mode != crate::policy::fs::FsMode::Open && !policy.fs.allowlist.is_empty() {
        argv.extend([
            "unshare".to_string(),
            "-m".to_string(),
            "--".to_string(),
            "sh".to_string(),
            "-c".to_string(),
            build_fs_bind_script(&policy.fs),
            "sh".to_string(),
        ]);
    }
    argv.extend(build_capsh_prefix(&policy.capability));
    argv.extend(cmd.cmd.iter().cloned());
    argv
}

#[cfg(target_os = "linux")]
pub use platform::{LinuxBackend, LinuxHandle};

#[cfg(target_os = "linux")]
pub mod platform {
    use std::fs;
    use std::sync::atomic::{AtomicBool, Ordering};

    use async_trait::async_trait;
    use tracing::warn;
    use uuid::Uuid;

    use super::*;
    use crate::backend::{
        build_command_wrapped, check_capability_escalation, check_cwd_fs, run_command_common,
        Backend, BackendHandle, EnforcementDim, EnforcementLevel, EnforcementReport, HealthStatus,
        RunResult,
    };
    use crate::error::SandboxdError;

    /// Linux 后端 (cgroups v2 + netns + mount ns + capsh, per FR-5.2)
    pub struct LinuxBackend;

    impl LinuxBackend {
        /// 构造 Linux 后端
        pub fn new() -> Self {
            Self
        }
    }

    /// Linux backend handle: 1 session = 1 cgroup 子树 + 1 netns
    pub struct LinuxHandle {
        /// 创建时冻结的策略快照
        policy: SessionSpec,
        /// cgroup 子树路径 (root 可用时 Some)
        cgroup: Option<std::path::PathBuf>,
        /// netns 名 (创建成功时 Some)
        netns: Option<String>,
        /// 已销毁标记
        destroyed: AtomicBool,
    }

    impl LinuxHandle {
        /// spawn 后写 cgroup.procs (per BD §5.2 assign_pid_to_cgroup)
        fn post_spawn(&self, child: &mut tokio::process::Child) -> Result<(), SandboxdError> {
            if let Some(cg) = &self.cgroup {
                if let Some(pid) = child.id() {
                    if let Err(e) = fs::write(cg.join("cgroup.procs"), pid.to_string()) {
                        warn!("cgroup.procs write failed (rootless?): {e}");
                    }
                }
            }
            Ok(())
        }
    }

    #[async_trait]
    impl Backend for LinuxBackend {
        async fn apply(
            &self,
            policy: &SessionSpec,
            session_id: Uuid,
        ) -> Result<Box<dyn BackendHandle>, SandboxdError> {
            // 1. cgroups v2 (best-effort, 需 root; 失败降级 PolicyAuditOnly)
            let cgroup = setup_cgroup(policy, &session_id);
            // 2. netns + iptables (best-effort; 失败降级)
            let netns = setup_netns(policy, &session_id).await;
            Ok(Box::new(LinuxHandle {
                policy: policy.clone(),
                cgroup,
                netns,
                destroyed: AtomicBool::new(false),
            }))
        }

        fn is_supported(&self) -> bool {
            true
        }

        fn enforcement(&self) -> Vec<EnforcementReport> {
            vec![
                EnforcementReport {
                    dim: EnforcementDim::Resource,
                    level: EnforcementLevel::ProcessWrapper,
                    detail: "cgroups v2 (memory.max/cpu.max/pids.max, 需 root, 失败自动降级 PolicyAuditOnly)".to_string(),
                },
                EnforcementReport {
                    dim: EnforcementDim::Network,
                    level: EnforcementLevel::ProcessWrapper,
                    detail: "netns + iptables OUTPUT (BlockAll + ip_allowlist; 域名 allowlist 需 ipset/DNS → v0.2 per 缺口 #6)".to_string(),
                },
                EnforcementReport {
                    dim: EnforcementDim::Fs,
                    level: EnforcementLevel::ProcessWrapper,
                    detail: "mount ns (unshare -m) bind mount allowlist (Read → remount ro)".to_string(),
                },
                EnforcementReport {
                    dim: EnforcementDim::Capability,
                    level: EnforcementLevel::ProcessWrapper,
                    detail: "capsh drop (DropAll/AllowNetBindService/Custom)".to_string(),
                },
            ]
        }
    }

    #[async_trait]
    impl BackendHandle for LinuxHandle {
        async fn run(
            &self,
            cmd: &CommandSpec,
            timeout_sec: u32,
        ) -> Result<RunResult, SandboxdError> {
            if self.destroyed.load(Ordering::SeqCst) {
                return Err(SandboxdError::Internal(
                    "session destroyed: reject new commands".to_string(),
                ));
            }
            if let Some(cap) = check_capability_escalation(cmd) {
                return Err(SandboxdError::CapabilityViolation { cap });
            }
            let violations = if cmd.cwd.is_empty() {
                Vec::new()
            } else {
                check_cwd_fs(&self.policy.fs, &cmd.cwd)
            };
            let policy = self.policy.clone();
            let netns_name = self.netns.clone();
            let wrap = move |spec: &CommandSpec| {
                let argv = build_wrap_argv(&policy, netns_name.as_deref(), spec);
                build_command_wrapped(argv, spec)
            };
            run_command_common(cmd, timeout_sec, wrap, |c| self.post_spawn(c), violations).await
        }

        async fn destroy(&self) -> Result<(), SandboxdError> {
            self.destroyed.store(true, Ordering::SeqCst);
            // netns 回收 (best-effort)
            if let Some(name) = &self.netns {
                let _ = tokio::process::Command::new("ip")
                    .args(["netns", "del", name])
                    .status()
                    .await;
            }
            // cgroup 子树回收 (best-effort)
            if let Some(cg) = &self.cgroup {
                let _ = fs::remove_dir_all(cg);
            }
            Ok(())
        }

        async fn health(&self) -> Result<HealthStatus, SandboxdError> {
            Ok(HealthStatus {
                healthy: !self.destroyed.load(Ordering::SeqCst),
                detail: format!(
                    "cgroup={}, netns={}",
                    self.cgroup.is_some(),
                    self.netns.is_some()
                ),
            })
        }
    }

    /// cgroups v2 子树创建 + 限制写入 (best-effort, 需 root)
    fn setup_cgroup(policy: &SessionSpec, session_id: &Uuid) -> Option<std::path::PathBuf> {
        let path = cgroup_path_for(session_id);
        fs::create_dir_all(&path).ok()?;
        if let Some(v) = cgroup_memory_max(policy.resource.memory_mb) {
            fs::write(path.join("memory.max"), v).ok();
        }
        if let Some(v) = cgroup_cpu_max(policy.resource.cpu_percent) {
            fs::write(path.join("cpu.max"), v).ok();
        }
        if let Some(v) = cgroup_pids_max(policy.resource.max_processes) {
            fs::write(path.join("pids.max"), v).ok();
        }
        Some(path)
    }

    /// netns 创建 + iptables 初始化 (best-effort, 需 root + iproute2)
    async fn setup_netns(policy: &SessionSpec, session_id: &Uuid) -> Option<String> {
        if policy.network.mode == crate::policy::network::NetworkMode::Passthrough {
            return None;
        }
        let name = netns_name_for(session_id);
        let add = tokio::process::Command::new("ip")
            .args(["netns", "add", &name])
            .status()
            .await;
        if add.map(|s| !s.success()).unwrap_or(true) {
            warn!("ip netns add failed (rootless?): network isolation degraded");
            return None;
        }
        let script = build_netns_setup_script(&policy.network);
        let exec = tokio::process::Command::new("ip")
            .args(["netns", "exec", &name, "sh", "-c", &script])
            .status()
            .await;
        if exec.map(|s| !s.success()).unwrap_or(true) {
            warn!("iptables init in netns failed: network isolation degraded");
            let _ = tokio::process::Command::new("ip")
                .args(["netns", "del", &name])
                .status()
                .await;
            return None;
        }
        Some(name)
    }
}
