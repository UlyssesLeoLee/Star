// @cypher schema=1 source_sha256=ced9a873e3bfea03811883d8ea6c87429be6dad6e4f217e3f23f72febd806bb3
// MERGE (self:File {path:"crates/domain-local-runtime/src/spawn_linux.rs"})
// MERGE (options:Type {id:"crates/domain-local-runtime/src/spawn_linux.rs::LinuxSpawnOptions",kind:"struct"})
// MERGE (session:Type {id:"process_wrap::tokio::ProcessSession",kind:"external"})
// MERGE (command_wrap:Type {id:"process_wrap::tokio::CommandWrap",kind:"external"})
// MERGE (wrap:Symbol {id:"crates/domain-local-runtime/src/spawn_linux.rs::wrap_linux_session",kind:"function"})
// MERGE (apply:Symbol {id:"crates/domain-local-runtime/src/spawn_linux.rs::apply_linux_spawn",kind:"function"})
// MERGE (disabled_test:Symbol {id:"crates/domain-local-runtime/src/spawn_linux.rs::tests.wrap_linux_session_new_session_false_is_noop",kind:"test"})
// MERGE (enabled_test:Symbol {id:"crates/domain-local-runtime/src/spawn_linux.rs::tests.wrap_linux_session_new_session_true_registers_session_wrapper",kind:"test"})
// MERGE (leader_test:Symbol {id:"crates/domain-local-runtime/src/spawn_linux.rs::tests.wrap_linux_session_makes_child_the_session_leader",kind:"test"})
// MERGE (self)-[:DEFINES]->(options)
// MERGE (self)-[:DEFINES]->(wrap)
// MERGE (self)-[:DEFINES]->(apply)
// MERGE (wrap)-[:USES_TYPE]->(options)
// MERGE (wrap)-[:USES_TYPE]->(command_wrap)
// MERGE (wrap)-[:USES_TYPE]->(session)
// MERGE (disabled_test)-[:TESTS]->(wrap)
// MERGE (enabled_test)-[:TESTS]->(wrap)
// MERGE (leader_test)-[:TESTS]->(wrap)
// @endcypher
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
//!   — setsid 由 [`wrap_linux_session`] 通过 Tokio `CommandWrap` 在 spawn 时执行
//! - [`wrap_linux_session`] — 将已配置的 Tokio command 包入 `ProcessSession`
//! - [`detect_systemd`] / [`detect_systemd_run`] / [`pick_cgroup_backend`] — cgroup 探测
//!
//! ## 守门 #7 unsafe_code = "forbid" 处理
//!
//! `setsid(2)` 走 `process_wrap::tokio::ProcessSession` safe wrapper
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

/// 将已配置的 Tokio command 包进 `CommandWrap`，并按需注册 `ProcessSession`。
///
/// `CommandWrap::spawn()` 会执行 wrapper 的 spawn hooks；Linux `ProcessSession`
/// 让子进程成为新 session/process-group leader，并把 wrapped child 保留给调用方，
/// 以便后续 wait、cancel 和进程组清理继续使用同一生命周期句柄。
/// `opts.new_session == false` 时不注册 wrapper，由调用方自行选择 `process_group(0)`。
#[cfg(target_os = "linux")]
pub fn wrap_linux_session(
    command: tokio::process::Command,
    opts: &LinuxSpawnOptions,
) -> process_wrap::tokio::CommandWrap {
    use process_wrap::tokio::{CommandWrap, ProcessSession};

    let mut wrapped = CommandWrap::from(command);
    if opts.new_session {
        wrapped.wrap(ProcessSession);
    }
    wrapped
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
        let wrapped = wrap_linux_session(cmd, &opts);
        assert!(!wrapped.has_wrap::<process_wrap::tokio::ProcessSession>());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn wrap_linux_session_new_session_true_registers_session_wrapper() {
        let cmd = tokio::process::Command::new("echo");
        let opts = LinuxSpawnOptions::default();
        let wrapped = wrap_linux_session(cmd, &opts);
        assert!(wrapped.has_wrap::<process_wrap::tokio::ProcessSession>());
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn wrap_linux_session_makes_child_the_session_leader() {
        use nix::unistd::{getsid, Pid};
        use process_wrap::tokio::ChildWrapper;

        let mut command = tokio::process::Command::new("/bin/sleep");
        command.arg("30").kill_on_drop(false);
        let opts = LinuxSpawnOptions {
            cgroup: CgroupMode::Disabled,
            ..Default::default()
        };
        let mut wrapped = wrap_linux_session(command, &opts);
        let mut child = wrapped.spawn().expect("spawn with ProcessSession");
        let pid = child.id().expect("spawned child has pid");
        let session_id = getsid(Some(Pid::from_raw(pid as i32))).map(|sid| sid.as_raw());

        // Reap the native child directly so this test does not exercise the separate group-wait path.
        let kill_result = child.inner_mut().start_kill();
        let wait_result = child.inner_mut().wait().await;

        assert_eq!(session_id.ok(), Some(pid as i32));
        assert!(kill_result.is_ok(), "child cleanup failed: {kill_result:?}");
        assert!(wait_result.is_ok(), "child reap failed: {wait_result:?}");
    }
}
