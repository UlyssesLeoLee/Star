//! backend::macos — macOS 隔离后端 (per FR-5.3)
//!
//! macOS 走 `sandbox-exec` (seatbelt) profile, 一份 profile 覆盖 4 维:
//!
//! | 维 | 实装 | 机制 |
//! |---|---|---|
//! | resource | PolicyAuditOnly | sandbox-exec profile 无内存 rlimit primitive → 策略审计 (per 守门 #11 显式标注) |
//! | network | ProcessWrapper | `(deny network*)` + ip_allowlist `(allow network-outbound (remote ip "CIDR"))` |
//! | fs | ProcessWrapper | `(deny file-read*/file-write*)` + per-rule `(allow file-read* (subpath "..."))` |
//! | capability | ProcessWrapper | sandbox-exec implicit drop (profile 默认 deny) |
//!
//! 域名 allowlist 无法在 seatbelt 表达 (仅 IP) → v0.1 用 ip_allowlist + loopback
//! (per SRS §9 缺口 #6, DNS 反查 v0.2 拍摄)。profile 生成为纯逻辑, 全平台编译 + 单测。

use uuid::Uuid;

use super::CommandSpec;
use crate::policy::{FsPathMode, SessionSpec};

/// 生成 sandbox-exec (seatbelt) profile 文本 (per FR-5.3)
///
/// - 默认 deny, `(allow process*)` 放行进程生命周期本身
/// - network 维: BlockAll / Allowlist → `(deny network*)` + ip_allowlist / loopback ACCEPT
/// - fs 维: Allowlist → deny file-read/file-write + per-rule subpath allow
pub fn build_sandbox_profile(policy: &SessionSpec, session_id: &Uuid) -> String {
    let mut p = String::new();
    p.push_str(&format!(
        "(version 1)\n; sandboxd session {session_id} (generated)\n"
    ));
    p.push_str("(deny default)\n");
    p.push_str("(import \"system.sb\")\n");
    p.push_str("(allow process*)\n");
    p.push_str("(allow mach-lookup)\n");

    // network 维 (per FR-5.3 sandbox-exec network filter)
    match policy.network.mode {
        crate::policy::network::NetworkMode::BlockAll => {
            p.push_str("(deny network*)\n");
        }
        crate::policy::network::NetworkMode::Allowlist => {
            p.push_str("(deny network*)\n");
            p.push_str("(allow network-outbound (remote ip \"127.0.0.1\"))\n");
            p.push_str("(allow network-outbound (remote ip \"::1\"))\n");
            for cidr in &policy.network.ip_allowlist {
                // seatbelt 需要纯 IP 段; 合法 CIDR 转 `ip:port` 形式 (port 通配)
                if let Ok(net) = cidr.parse::<ipnet::IpNet>() {
                    if let Some(addr) = net.addr().to_string().split('/').next() {
                        p.push_str(&format!(
                            "(allow network-outbound (remote ip \"{addr}:*\"))\n"
                        ));
                    }
                }
            }
        }
        crate::policy::network::NetworkMode::Passthrough => {}
    }

    // fs 维 (per FR-5.3 sandbox-exec file filter)
    match policy.fs.mode {
        crate::policy::fs::FsMode::Open => {}
        crate::policy::fs::FsMode::ReadOnlyRoot => {
            p.push_str("(deny file-write*)\n");
            for rule in &policy.fs.allowlist {
                if rule.mode == FsPathMode::ReadWrite {
                    p.push_str(&format!(
                        "(allow file-read* (subpath \"{}\"))\n(allow file-write* (subpath \"{}\"))\n",
                        rule.path, rule.path
                    ));
                }
            }
        }
        crate::policy::fs::FsMode::Allowlist => {
            p.push_str("(deny file-read*)\n(deny file-write*)\n");
            for rule in &policy.fs.allowlist {
                p.push_str(&format!("(allow file-read* (subpath \"{}\"))\n", rule.path));
                if rule.mode == FsPathMode::ReadWrite {
                    p.push_str(&format!(
                        "(allow file-write* (subpath \"{}\"))\n",
                        rule.path
                    ));
                }
            }
        }
    }

    // capability 维: profile 默认 deny + (deny default) = sandbox-exec implicit drop (per FR-5.3)
    p
}

/// macOS 包装 argv: `/usr/bin/sandbox-exec -p <profile> -- <cmd...>` (per FR-5.3)
pub fn build_wrap_argv_macos(
    policy: &SessionSpec,
    session_id: &Uuid,
    cmd: &CommandSpec,
) -> Vec<String> {
    let profile = build_sandbox_profile(policy, session_id);
    let mut argv = vec![
        "/usr/bin/sandbox-exec".to_string(),
        "-p".to_string(),
        profile,
        "--".to_string(),
    ];
    argv.extend(cmd.cmd.iter().cloned());
    argv
}

#[cfg(target_os = "macos")]
pub mod platform {
    use std::sync::atomic::{AtomicBool, Ordering};

    use async_trait::async_trait;
    use uuid::Uuid;

    use super::*;
    use crate::backend::{
        build_command_wrapped, check_capability_escalation, check_cwd_fs, run_command_common,
        Backend, BackendHandle, EnforcementDim, EnforcementLevel, EnforcementReport, HealthStatus,
        RunResult,
    };
    use crate::error::SandboxdError;

    /// macOS 后端 (sandbox-exec profile, per FR-5.3)
    pub struct MacosBackend;

    impl MacosBackend {
        /// 构造 macOS 后端
        pub fn new() -> Self {
            Self
        }
    }

    /// macOS backend handle: 1 session = 1 sandbox-exec profile
    pub struct MacosHandle {
        /// 创建时冻结的策略快照
        policy: SessionSpec,
        /// session id (profile 标识)
        session_id: Uuid,
        /// 已销毁标记
        destroyed: AtomicBool,
    }

    #[async_trait]
    impl Backend for MacosBackend {
        async fn apply(
            &self,
            policy: &SessionSpec,
            session_id: Uuid,
        ) -> Result<Box<dyn BackendHandle>, SandboxdError> {
            Ok(Box::new(MacosHandle {
                policy: policy.clone(),
                session_id,
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
                    level: EnforcementLevel::PolicyAuditOnly,
                    detail: "sandbox-exec profile 无内存 rlimit primitive → 策略审计 (per 守门 #11 显式标注)".to_string(),
                },
                EnforcementReport {
                    dim: EnforcementDim::Network,
                    level: EnforcementLevel::ProcessWrapper,
                    detail: "sandbox-exec network filter: (deny network*) + ip_allowlist (域名需 DNS 反查 → v0.2 per 缺口 #6)".to_string(),
                },
                EnforcementReport {
                    dim: EnforcementDim::Fs,
                    level: EnforcementLevel::ProcessWrapper,
                    detail: "sandbox-exec file filter: deny file-read*/file-write* + subpath allowlist".to_string(),
                },
                EnforcementReport {
                    dim: EnforcementDim::Capability,
                    level: EnforcementLevel::ProcessWrapper,
                    detail: "sandbox-exec implicit drop (profile deny default)".to_string(),
                },
            ]
        }
    }

    #[async_trait]
    impl BackendHandle for MacosHandle {
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
            let session_id = self.session_id;
            let wrap = move |spec: &CommandSpec| {
                let argv = build_wrap_argv_macos(&policy, &session_id, spec);
                build_command_wrapped(argv, spec)
            };
            run_command_common(cmd, timeout_sec, wrap, |_child| Ok(()), violations).await
        }

        async fn destroy(&self) -> Result<(), SandboxdError> {
            self.destroyed.store(true, Ordering::SeqCst);
            Ok(())
        }

        async fn health(&self) -> Result<HealthStatus, SandboxdError> {
            Ok(HealthStatus {
                healthy: !self.destroyed.load(Ordering::SeqCst),
                detail: "sandbox-exec profile active".to_string(),
            })
        }
    }
}

#[cfg(target_os = "macos")]
pub use platform::{MacosBackend, MacosHandle};
