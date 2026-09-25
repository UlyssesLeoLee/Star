//! Star Local Runtime — macOS 真 OS spawn 适配 (ULYS-212)
//!
//! 实现 [ULYS-212](https://app.multica.ai/issue/01a0cd6c-846c-7781-82b6-1f55bffb8e7b)
//! 描述的 macOS 部分:`spawn_macos.rs (setpgid)`。
//!
//! ## 范围限定
//!
//! **`#[cfg(target_os = "macos")]` only**。Linux 走 [`crate::spawn_linux`],
//! Windows 走 [`crate::spawn_windows`],跨平台 kill 走 [`crate::kill`]。
//!
//! ## 公开 API
//!
//! - [`MacosSpawnOptions`] — setpgid 开关
//! - [`apply_macos_spawn`] — 返回 `(program, args)` 元组 — setpgid 不改 program/args
//!   (走 tokio 的 `cmd.process_group(0)` 注入,由 caller 在 `spawn()` 前调用
//!   [`wrap_macos_setpgid`]),此处 API 与 [`crate::spawn_linux::apply_linux_spawn`] 对齐
//! - [`wrap_macos_setpgid`] — 给 `tokio::process::Command` 注入 `process_group(0)`
//!   (走 tokio safe API,内部 `setpgid(0, 0)`)
//!
//! ## 与 Linux 的差异
//!
//! - **无 cgroup**:macOS 没有 systemd cgroup 等价物;`apply_macos_spawn` 不动 argv
//!   (返回原 (program, args) 不变),cgroup 概念在该平台不存在
//! - **setpgid 而非 setsid**:macOS 习惯 `setpgid(0, 0)`(BSD 风格),虽 `setsid(2)` 在
//!   macOS 10.15+ 也支持,但 `setpgid` 与 `setsid` 的语义差异(session leader vs
//!   process group leader)对 agent CLI 子进程隔离需求都够用
//!   — 选 `setpgid` 与 ULYS-212 description 一致
//!
//! ## 守门 #7 unsafe_code = "forbid" 处理
//!
//! `setpgid(0, 0)` 走 `tokio::process::Command::process_group(0)` — tokio safe API
//! 包装(std `CommandExt::process_group` 是 `unsafe fn`,但 tokio 包了一层 safe)。
//!
//! ## 引用与 anchor
//!
//! - 父 issue: ULYS-156 (`01a0bf6b-486c-71b8-9bf4-92bde909c7f8`) §A 单进程持久化层
//! - 兄弟 issue: ULYS-219 (`01a0cda7-...`) Unix setsid + killpg (未 merge)
//! - 关联 FR-ORCA-001 AC-1 (CLI child survives parent restart/shutdown)

use thiserror::Error;

// =====================================================================
// 1. options
// =====================================================================

/// macOS 真 spawn 适配选项
#[derive(Debug, Clone)]
pub struct MacosSpawnOptions {
    /// 是否创建新 process group(`setpgid(0, 0)` 注入)
    ///
    /// - `true`(默认):子进程成为新 pg leader,pgid == pid
    /// - `false`:保持父进程 pgid(向后兼容 fallback)
    pub new_pg: bool,
}

impl Default for MacosSpawnOptions {
    fn default() -> Self {
        Self { new_pg: true }
    }
}

// =====================================================================
// 2. wrap_macos_setpgid (macOS-only)
// =====================================================================

/// 给 `tokio::process::Command` 注入 `process_group(0)`
/// (内部调 `setpgid(0, 0)`,子进程成为新 process group leader)。
///
/// 必须在 `cmd.spawn()` **前**调用。
///
/// `tokio::process::Command::process_group(i32)` 是 safe API(stable since tokio 1.x)
/// — 内部 unsafe 已被 tokio 隔离,**用户代码 0 unsafe**(per 守门 #7)。
///
/// ## noop
///
/// `opts.new_pg == false` ⇒ 不动 cmd。
///
/// ## 错误
///
/// 自身不返回 Result。`setpgid(0, 0)` 失败通过 `nix::Error` 冒泡到 spawn 调用方
/// (典型:`EACCES` 当父进程已被另一个进程 attach;罕见)。
#[cfg(target_os = "macos")]
pub fn wrap_macos_setpgid(cmd: &mut tokio::process::Command, opts: &MacosSpawnOptions) {
    if !opts.new_pg {
        return;
    }
    // tokio safe API:process_group(0) 等价 setpgid(0, 0)
    // — 子进程以自身 pid 作为新 pgid
    cmd.process_group(0);
}

// =====================================================================
// 3. apply_macos_spawn (macOS-only)
// =====================================================================

/// macOS 真 spawn 适配入口
///
/// macOS 无 cgroup 概念,返回原 (program, args) 不变。
/// setsid/setpgid 注入由 caller 在 `tokio::process::Command::spawn()` 前
/// 调 [`wrap_macos_setpgid`] 完成(API 解耦以便跟 Linux 同款签名对齐)。
pub fn apply_macos_spawn(
    original_program: &str,
    original_args: &[String],
) -> (String, Vec<String>) {
    (original_program.to_string(), original_args.to_vec())
}

// =====================================================================
// 4. error
// =====================================================================

/// macOS spawn 适配错误(预留;当前 setpgid 路径无业务错误)
#[derive(Debug, Error)]
pub enum MacosSpawnError {
    /// setpgid 失败(`nix::Error` 桥接;预留,当前未使用)
    #[error("setpgid failed: {0}")]
    Setpgid(String),
}

// =====================================================================
// 5. unit tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_default_new_pg_enabled() {
        let o = MacosSpawnOptions::default();
        assert!(o.new_pg);
    }

    #[test]
    fn apply_macos_spawn_passes_through_unchanged() {
        let (prog, args) =
            apply_macos_spawn("claude", &["--model".to_string(), "sonnet".to_string()]);
        assert_eq!(prog, "claude");
        assert_eq!(args, vec!["--model", "sonnet"]);
    }

    #[test]
    fn apply_macos_spawn_no_args() {
        let (prog, args) = apply_macos_spawn("echo", &[]);
        assert_eq!(prog, "echo");
        assert!(args.is_empty());
    }

    // macOS-only 集成测试(真实 process_group 注入路径)
    #[cfg(target_os = "macos")]
    #[test]
    fn wrap_macos_setpgid_new_pg_false_is_noop() {
        let mut cmd = tokio::process::Command::new("echo");
        let opts = MacosSpawnOptions { new_pg: false };
        wrap_macos_setpgid(&mut cmd, &opts);
        // 仅验证不 panic;不验证 cmd 内部状态
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn wrap_macos_setpgid_new_pg_true_does_not_panic() {
        let mut cmd = tokio::process::Command::new("echo");
        let opts = MacosSpawnOptions::default();
        wrap_macos_setpgid(&mut cmd, &opts);
    }
}
