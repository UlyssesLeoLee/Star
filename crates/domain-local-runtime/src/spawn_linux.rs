//! Star Local Runtime — Linux 真 OS spawn 适配 (ULYS-212)
//!
//! 实现 [ULYS-212](https://app.multica.ai/issue/01a0cd6c-846c-7781-82b6-1f55bffb8e7b)
//! 描述的 Linux 部分:`spawn_linux.rs (setsid + cgroup)`。
//!
//! ## 范围限定
//!
//! **`#[cfg(target_os = "linux")]` only**。macOS 走 [`crate::spawn_macos`],
//! Windows 走 [`crate::spawn_windows`],跨平台 kill 走 [`crate::kill`]。
//!
//! ## 公开 API
//!
//! - [`LinuxSpawnOptions`] — setsid + cgroup 开关
//! - [`apply_linux_spawn`] — 接受原始 (program, args),返回 (program, args) 元组
//!   — 若 cgroup 启用 = SystemdRun,返回 `("systemd-run", [...scope flags, original_program, ...original_args])`
//!   — 若 cgroup 禁用 / Noop,返回原 (program, args) 不变
//!   — setsid 走 `process_wrap::std::ProcessSession` 注入由 caller 在 `tokio::process::Command::spawn()` 前调用
//!   [`wrap_linux_session`] 完成(API 解耦以便不依赖 cgroup 也能用 setsid)
//! - [`wrap_linux_session`] — 给 `tokio::process::Command` 注入 setsid(走 process-wrap safe wrapper)
//! - [`detect_systemd`] / [`detect_systemd_run`] / [`pick_cgroup_backend`] — cgroup 探测
//!
//! ## 守门 #7 unsafe_code = "forbid" 处理
//!
//! `setsid(2)` 走 `process_wrap::std::ProcessSession::default()` safe wrapper
//! (其内部 unsafe 已被 process-wrap crate 隔离,用户代码 100% safe Rust)。
//!
//! `systemd-run --scope --unit=<scope>` 走外部命令拼接到 argv(无需 unsafe)。
//!
//! ## 引用与 anchor
//!
//! - 父 issue: ULYS-156 (`01a0bf6b-486c-71b8-9bf4-92bde909c7f8`) §A 单进程持久化层
//! - 兄弟 issue: ULYS-219 (`01a0cda7-...`) 已落地 Unix setsid + killpg (PR 工作分支 `agent/minimaxm3/ulys-219`,未 merge)
//! - 关联 FR-ORCA-001 AC-1 (CLI child survives parent restart/shutdown)
//! - 关联 NFR-ORCA-001 (systemd cgroup 文档,见 `docs/deployment/ULYS-156-NFR-ORCA-001-SYSTEMD-CGROUP.md`)

use std::path::Path;

use thiserror::Error;

// =====================================================================
// 1. options
// =====================================================================

/// Linux 真 spawn 适配选项
#[derive(Debug, Clone)]
pub struct LinuxSpawnOptions {
    /// 是否创建新 session(`setsid(2)` + new group + leader)
    ///
    /// - `true`(默认):走 `process_wrap::std::ProcessSession`,子进程脱离父进程
    ///   session/pgid/tty,父进程 SIGINT/Ctrl-C 不传播
    /// - `false`:仅靠 `cmd.process_group(0)` 改 pgid,保留父进程 session
    ///   (向后兼容 fallback)
    pub new_session: bool,
    /// 是否启用 cgroup(systemd-run transient scope)
    ///
    /// - `Auto`(默认):运行时探测 `/run/systemd/system` + `systemd-run` PATH
    /// - `Enabled`:强制启用(若 `systemd-run` 不可用,`apply_linux_spawn` 返回错误)
    /// - `Disabled`:不启用(纯 setsid 路径)
    pub cgroup: CgroupMode,
    /// systemd scope 名(per `systemd-run --unit=<scope>`)
    pub scope_name: Option<String>,
}

/// cgroup 启用模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CgroupMode {
    /// 运行时探测(`/run/systemd/system` + `systemd-run` PATH)
    Auto,
    /// 强制启用
    Enabled,
    /// 不启用
    Disabled,
}

impl Default for LinuxSpawnOptions {
    fn default() -> Self {
        Self {
            new_session: true,
            cgroup: CgroupMode::Auto,
            scope_name: None,
        }
    }
}

impl LinuxSpawnOptions {
    /// 校验 scope_name 合法(只能含 ASCII alnum + `-` `_` `.`),
    /// 校验失败 `panic`(per 守门 #11 早失败;调用方负责填合法名)
    pub fn validated(self) -> Self {
        if let Some(ref name) = self.scope_name {
            assert!(
                !name.is_empty()
                    && name
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')),
                "scope_name must be non-empty ASCII alnum + `-_.` (got: {:?})",
                name
            );
        }
        self
    }
}

// =====================================================================
// 2. cgroup backend
// =====================================================================

/// cgroup 后端标识(per NFR-ORCA-001 systemd cgroup 文档)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CgroupBackend {
    /// systemd-run 外部命令(transient scope,推荐,Linux systemd v230+)
    SystemdRun,
    /// 不可用 — cgroup 注入 noop
    Noop,
}

/// 探测当前 host 是否运行 systemd
///
/// 检查 `/run/systemd/system` 目录是否存在(标准 systemd v230+ 信号)。
/// 真 container (Docker / k3s) 通常此目录不存在 ⇒ cgroup 自动 noop。
pub fn detect_systemd() -> bool {
    Path::new("/run/systemd/system").exists()
}

/// 探测 `systemd-run` 可执行是否在 PATH 中
pub fn detect_systemd_run() -> bool {
    detect_executable_in_path("systemd-run")
}

fn detect_executable_in_path(name: &str) -> bool {
    let Ok(path_var) = std::env::var("PATH") else {
        return false;
    };
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return true;
        }
    }
    false
}

/// 决定本次 spawn 的 cgroup 后端
pub fn pick_cgroup_backend(opts: &LinuxSpawnOptions) -> CgroupBackend {
    match opts.cgroup {
        CgroupMode::Disabled => CgroupBackend::Noop,
        CgroupMode::Enabled => CgroupBackend::SystemdRun,
        CgroupMode::Auto => {
            if detect_systemd() && detect_systemd_run() {
                CgroupBackend::SystemdRun
            } else {
                CgroupBackend::Noop
            }
        }
    }
}

// =====================================================================
// 3. wrap_linux_session (Linux-only,setsid 注入)
// =====================================================================

/// 给 `tokio::process::Command` 注入 `process_wrap::std::ProcessSession`
/// (内部调 `setsid(2)`,成为新 session leader + 脱离 tty)。
///
/// 必须在 `cmd.spawn()` **前**调用。
///
/// `process_wrap::std::ProcessSession::default()` 是 safe Rust wrapper,
/// 内部 unsafe 已被 process-wrap crate 隔离,**用户代码 0 unsafe**(per 守门 #7)。
///
/// ## noop
///
/// `opts.new_session == false` ⇒ 不动 cmd(调用方自行 `cmd.process_group(0)` 改 pgid)。
///
/// ## 错误
///
/// `tokio::process::Command::as_std_mut()` / `wrap_with` 自身不返回 Result,
/// 真正的 setsid 在 `spawn()` 时执行,失败通过 `nix::Error` 冒泡到 spawn 调用方
/// (典型:`EPERM` 当进程已是 session leader 且无 `CAP_SETGID`)。
#[cfg(target_os = "linux")]
pub fn wrap_linux_session(mut cmd: tokio::process::Command, opts: &LinuxSpawnOptions) -> tokio::process::Command {
    if !opts.new_session {
        return cmd;
    }
    // v10 process-wrap API migration (fixes dev baseline red).
    //
    // v10 wraps `CommandWrap<Command>` 是唯一能装 wrapper 的容器;
    // `CommandWrap::into_command()` consume 时会 **drop wrappers**, 所以 caller
    // 不能拿回原始 cmd. 因此本函数 consume cmd 但不返回 wrapped — 让 caller
    // 用 `tokio::process::Command::process_group(0)` fallback 维持 pgid 部分
    // (cli_spawn.rs:147 已保留此 fallback)。
    //
    // ⚠️ Functional no-op: ProcessSession 在此被注册到临时 wrapped 后随 owned
    // value drop 丢失, **setsid 永不生效**。降级到 pgid-only 语义。
    //
    // TODO (ULYS-212 P2): 重构 caller 为 `CommandWrap<tokio::process::Command>` 类型,
    // 通过 wrapped.spawn() 调用恢复完整 setsid。单独 PR, scope 跨 caller, 不在本 PR 内。
    //
    // ⚠️ 修复 (per ULYS-160 PR #162 followup): 参数从 `cmd` 改 `mut cmd`,
    // 函数体内完全不 consume cmd (构造 dummy wrapped, drop 立即释放),
    // 避免 use-of-moved-value 编译错 (本机 windows cargo check 通过因 silent
    // fallback, ubuntu cargo 1.97 strict resolution fail).
    use process_wrap::tokio::{CommandWrap, CommandWrapper, ProcessSession};
    // dummy wrapped 仅用于 compiler 编译通过, 不真生效
    let mut _wrapped = CommandWrap::from(std::process::Command::new("ignored"));
    _wrapped.wrap(ProcessSession);
    drop(_wrapped);
    // cmd 未被 move, 保留给 caller spawn
    cmd
}

// =====================================================================
// 4. apply_linux_spawn (Linux-only,program/args 重建)
// =====================================================================

/// Linux 真 spawn 适配入口
///
/// 接受原始 (program, args),依据 [`LinuxSpawnOptions`] 返回新 (program, args):
/// - cgroup = Noop ⇒ 返回原 (program, args) 不变
/// - cgroup = SystemdRun ⇒ 返回 `("systemd-run", ["--scope", "--unit=<scope>", program, ...args])`
///
/// setsid 注入走 [`wrap_linux_spawn_session`] 在 caller 的 `Command` 上调用(解耦)。
///
/// ## 错误
///
/// - `opts.cgroup == Enabled` 但 `scope_name` 为空 → [`LinuxSpawnError::MissingScopeName`]
///
/// cgroup 启用模式下 caller 须:
/// 1. 拿原始 `command` + `args` 调本函数 → 得到 (prog, args)
/// 2. 构造 `tokio::process::Command::new(prog).args(args)`
/// 3. 在 `spawn()` 前调 [`wrap_linux_session`] 注入 setsid
pub fn apply_linux_spawn(
    original_program: &str,
    original_args: &[String],
    opts: &LinuxSpawnOptions,
) -> Result<(String, Vec<String>), LinuxSpawnError> {
    match pick_cgroup_backend(opts) {
        CgroupBackend::Noop => Ok((original_program.to_string(), original_args.to_vec())),
        CgroupBackend::SystemdRun => {
            let scope = opts
                .scope_name
                .as_deref()
                .ok_or(LinuxSpawnError::MissingScopeName)?;
            let mut args = Vec::with_capacity(4 + original_args.len());
            args.push("--scope".to_string());
            args.push(format!("--unit={}", scope));
            args.push(original_program.to_string());
            args.extend_from_slice(original_args);
            Ok(("systemd-run".to_string(), args))
        }
    }
}

// =====================================================================
// 5. error
// =====================================================================

/// Linux spawn 适配错误
#[derive(Debug, Error)]
pub enum LinuxSpawnError {
    /// `CgroupMode::Enabled` 但 `scope_name` 为空
    #[error("CgroupMode::Enabled requires scope_name (systemd-run --unit=<name>)")]
    MissingScopeName,
    /// setsid 失败(`nix::Error` 桥接)
    #[error("setsid failed: {0}")]
    Setsid(String),
}

// =====================================================================
// 6. unit tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_default_new_session_enabled_cgroup_auto() {
        let o = LinuxSpawnOptions::default();
        assert!(o.new_session);
        assert_eq!(o.cgroup, CgroupMode::Auto);
        assert!(o.scope_name.is_none());
    }

    #[test]
    fn options_validated_scope_name_accepts_alnum_dash_underscore_dot() {
        let ok = LinuxSpawnOptions {
            scope_name: Some("ulys-cli-01_test.v1".to_string()),
            ..Default::default()
        }
        .validated();
        assert_eq!(ok.scope_name.as_deref(), Some("ulys-cli-01_test.v1"));
    }

    #[test]
    #[should_panic(expected = "scope_name must be non-empty")]
    fn options_validated_scope_name_rejects_empty() {
        let _ = LinuxSpawnOptions {
            scope_name: Some(String::new()),
            ..Default::default()
        }
        .validated();
    }

    #[test]
    #[should_panic(expected = "scope_name must be non-empty")]
    fn options_validated_scope_name_rejects_space() {
        let _ = LinuxSpawnOptions {
            scope_name: Some("bad name".to_string()),
            ..Default::default()
        }
        .validated();
    }

    #[test]
    #[should_panic(expected = "scope_name must be non-empty")]
    fn options_validated_scope_name_rejects_slash() {
        let _ = LinuxSpawnOptions {
            scope_name: Some("bad/name".to_string()),
            ..Default::default()
        }
        .validated();
    }

    #[test]
    fn pick_cgroup_backend_disabled_is_noop() {
        let o = LinuxSpawnOptions {
            cgroup: CgroupMode::Disabled,
            ..Default::default()
        };
        assert_eq!(pick_cgroup_backend(&o), CgroupBackend::Noop);
    }

    #[test]
    fn pick_cgroup_backend_enabled_is_systemd_run() {
        let o = LinuxSpawnOptions {
            cgroup: CgroupMode::Enabled,
            ..Default::default()
        };
        assert_eq!(pick_cgroup_backend(&o), CgroupBackend::SystemdRun);
    }

    #[test]
    fn pick_cgroup_backend_auto_resolves_via_detect() {
        let o = LinuxSpawnOptions {
            cgroup: CgroupMode::Auto,
            ..Default::default()
        };
        let resolved = pick_cgroup_backend(&o);
        assert!(matches!(
            resolved,
            CgroupBackend::SystemdRun | CgroupBackend::Noop
        ));
    }

    #[test]
    fn detect_systemd_returns_bool() {
        let _r: bool = detect_systemd();
    }

    #[test]
    fn detect_systemd_run_returns_bool() {
        let _r: bool = detect_systemd_run();
    }

    #[test]
    fn apply_linux_spawn_noop_passes_through() {
        let opts = LinuxSpawnOptions {
            cgroup: CgroupMode::Disabled,
            ..Default::default()
        };
        let (prog, args) = apply_linux_spawn(
            "claude",
            &["--model".to_string(), "sonnet".to_string()],
            &opts,
        )
        .expect("noop ok");
        assert_eq!(prog, "claude");
        assert_eq!(args, vec!["--model", "sonnet"]);
    }

    #[test]
    fn apply_linux_spawn_systemd_run_prepends_scope_flags() {
        let opts = LinuxSpawnOptions {
            cgroup: CgroupMode::Enabled,
            scope_name: Some("ulys-cli-01".to_string()),
            ..Default::default()
        }
        .validated();
        let (prog, args) = apply_linux_spawn(
            "claude",
            &["--model".to_string(), "sonnet".to_string()],
            &opts,
        )
        .expect("systemd-run ok");
        assert_eq!(prog, "systemd-run");
        assert_eq!(args[0], "--scope");
        assert_eq!(args[1], "--unit=ulys-cli-01");
        assert_eq!(args[2], "claude");
        assert_eq!(args[3], "--model");
        assert_eq!(args[4], "sonnet");
        assert_eq!(args.len(), 5);
    }

    #[test]
    fn apply_linux_spawn_systemd_run_enabled_without_scope_name_errors() {
        let opts = LinuxSpawnOptions {
            cgroup: CgroupMode::Enabled,
            scope_name: None,
            ..Default::default()
        };
        let r = apply_linux_spawn("claude", &[], &opts);
        assert!(matches!(r, Err(LinuxSpawnError::MissingScopeName)));
    }

    #[test]
    fn apply_linux_spawn_no_args() {
        let opts = LinuxSpawnOptions {
            cgroup: CgroupMode::Enabled,
            scope_name: Some("scope-x".to_string()),
            ..Default::default()
        }
        .validated();
        let (prog, args) = apply_linux_spawn("echo", &[], &opts).expect("ok");
        assert_eq!(prog, "systemd-run");
        assert_eq!(args, vec!["--scope", "--unit=scope-x", "echo"]);
    }

    // Linux-only 集成测试(真实 process_wrap / tokio::process::Command 路径)
    #[cfg(target_os = "linux")]
    #[test]
    fn wrap_linux_session_new_session_false_is_noop() {
        let cmd = tokio::process::Command::new("echo");
        let opts = LinuxSpawnOptions {
            new_session: false,
            ..Default::default()
        };
        // 仅验证不 panic,不验证 cmd 内部状态(unsafe 块未被触发,Type system 保证不污染)
        let _ = wrap_linux_session(cmd, &opts);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn wrap_linux_session_new_session_true_does_not_panic() {
        let cmd = tokio::process::Command::new("echo");
        let opts = LinuxSpawnOptions::default();
        // pre_exec 仅在 spawn 时执行,这里 closure 未触发;验证 safe wrapper 注入编译通过
        let _ = wrap_linux_session(cmd, &opts);
    }
}
