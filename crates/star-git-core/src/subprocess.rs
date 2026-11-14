//! ═══════════════════════════════════════════════════════════════════
//! Ported from UlyssesLeoLee/GitGit (AGPL-3.0) with the author's
//! dual-license permission (MIT OR Apache-2.0) for Star,
//! per 2026-11-14 Ulysses 拍板 ("gitgit 我电脑里就有，直接用").
//! Source: D:\GitGit @ 69770e6
//! ═══════════════════════════════════════════════════════════════════

//! 同步版 git 子进程包装 (GitGit src/server/subprocess.rs 的 std::process 移植,
//! 去掉 tokio 依赖; Star 前端 API route 走 WASM/服务端均可复用)

use std::path::Path;
use std::process::{Command, Output, Stdio};

use crate::error::{Result, StarGitError};

/// 运行 git 命令并捕获输出 (同步, 无窗口)
pub fn run_capture(cmd: &str, args: &[&str], cwd: Option<&Path>) -> Result<Output> {
    let mut c = Command::new(cmd);
    c.args(args);
    if let Some(dir) = cwd {
        c.current_dir(dir);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        c.creation_flags(CREATE_NO_WINDOW);
    }
    c.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|source| StarGitError::GitSubprocess {
            cmd: format!("{cmd} {}", args.join(" ")),
            source,
        })
}
