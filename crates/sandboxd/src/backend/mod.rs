/*
@cypher schema=1 source_sha256=299bb6a368a04ec687d2ab0c00f8fb19fe8a3a862a1e75f542bb2ae107c52272
MERGE (self:File {path:"crates/sandboxd/src/backend/mod.rs"})
@endcypher
*/
//! `sandboxd` backend 模块 — 跨平台隔离后端 (per FR-5, DD-SANDBOX-002 §2.1)
//!
//! | 平台 | Resource | Network | FS | Capability |
//! |---|---|---|---|---|
//! | Windows | Job Objects (win32job safe wrapper) | v0.1 policy+audit (WFP v0.2) | v0.1 policy+audit (AppContainer v0.2) | v0.1 spawn 拦截 + 隐式 drop |
//! | Linux | cgroups v2 (fs 直写) | netns + iptables (`ip` CLI) | mount ns (`unshare` CLI) | `capsh` drop |
//! | macOS | sandbox-exec profile | sandbox-exec network filter | sandbox-exec file filter | sandbox-exec implicit drop |
//!
//! [`DefaultBackendFactory`] 按编译目标平台选择后端 (per DD §2.1)。
//! 三平台实装的纯逻辑部分 (profile 生成 / wrap argv 构造) 全平台编译, 平台特定
//! spawn 集成走 `#[cfg]` (Windows 本机开发 → win32job 路径全量验证)。

use std::time::{Duration, Instant};

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::SandboxdError;
use crate::policy::SessionSpec;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(windows)]
pub mod windows;

/// 命令规格 (domain 类型, 跟 proto `CommandSpec` 1:1, per FR-4.2 入参)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    /// argv (cmd[0] = executable)
    pub cmd: Vec<String>,
    /// env KEY=VALUE 形式 (per 守门 #5: 内容永不打印, 仅 KEY 名进审计)
    pub env: Vec<String>,
    /// 工作目录 (空 = 继承 sandboxd)
    pub cwd: String,
    /// 是否捕获 stdout/stderr
    pub capture_output: bool,
}

impl CommandSpec {
    /// 构造命令规格
    pub fn new(cmd: Vec<String>, env: Vec<String>, cwd: String, capture_output: bool) -> Self {
        Self {
            cmd,
            env,
            cwd,
            capture_output,
        }
    }

    /// env KEY 名列表 (per 守门 #5: 审计/日志仅用 KEY 名, 不含 value)
    pub fn env_keys(&self) -> Vec<String> {
        self.env
            .iter()
            .filter_map(|kv| kv.split_once('=').map(|(k, _)| k.to_string()))
            .collect()
    }
}

impl From<&crate::proto::CommandSpec> for CommandSpec {
    fn from(p: &crate::proto::CommandSpec) -> Self {
        Self {
            cmd: p.cmd.clone(),
            env: p.env.clone(),
            cwd: p.cwd.clone(),
            capture_output: p.capture_output,
        }
    }
}

/// 命令执行结果 (domain 类型, per FR-4.2 出参)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunResult {
    /// 退出码 (超时 = 124, per GNU timeout 惯例)
    pub returncode: i32,
    /// stdout
    pub stdout: String,
    /// stderr
    pub stderr: String,
    /// 端到端延迟 ms (per FR-4.2 latency_ms)
    pub latency_ms: u64,
    /// 命中违规描述 (per FR-4.2 hit_violations)
    pub hit_violations: Vec<String>,
}

/// 命令超时退出码 (per GNU coreutils timeout 惯例)
pub const TIMEOUT_EXIT_CODE: i32 = 124;

/// backend 健康状态 (per BackendHandle::health)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthStatus {
    /// 是否健康
    pub healthy: bool,
    /// 详情 (不健康原因 / 平台说明)
    pub detail: String,
}

/// 隔离维度 (per FR-3 4 维)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnforcementDim {
    /// resource 维
    Resource,
    /// network 维
    Network,
    /// fs 维
    Fs,
    /// capability 维
    Capability,
}

impl EnforcementDim {
    /// 维度名 (审计前缀, per orchestrator hit_violations 分类)
    pub fn as_str(&self) -> &'static str {
        match self {
            EnforcementDim::Resource => "resource",
            EnforcementDim::Network => "network",
            EnforcementDim::Fs => "fs",
            EnforcementDim::Capability => "capability",
        }
    }
}

/// 强制等级 (per 守门 #11 缺标比错标: 显式标注每维实际强制力)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnforcementLevel {
    /// 内核级强制 (Job Objects / WFP / AppContainer)
    Kernel,
    /// 进程包装器强制 (cgroup / netns / unshare / capsh / sandbox-exec)
    ProcessWrapper,
    /// 策略层审计 (spawn 前检查 + audit, 运行时无法内核强制)
    PolicyAuditOnly,
}

/// 单维强制力报告 (per 守门 #11: 每平台 × 每维显式标注 v0.1 实际强制力)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnforcementReport {
    /// 维度
    pub dim: EnforcementDim,
    /// 强制等级
    pub level: EnforcementLevel,
    /// 详情 (e.g. "WFP 完整版 v0.2 拍摄")
    pub detail: String,
}

/// 平台隔离后端 (per DD §2.1 Backend trait)
#[async_trait]
pub trait Backend: Send + Sync {
    /// 应用 4 维隔离策略, 返回 backend handle (per DD §3.1 CreateSession 时序点 3)
    async fn apply(
        &self,
        policy: &SessionSpec,
        session_id: Uuid,
    ) -> Result<Box<dyn BackendHandle>, SandboxdError>;

    /// 检查当前平台是否支持
    fn is_supported(&self) -> bool;

    /// 各维强制力报告 (per 守门 #11 显式标注)
    fn enforcement(&self) -> Vec<EnforcementReport>;
}

/// backend handle (1 session = 1 handle, per DD §2.1 BackendHandle trait)
#[async_trait]
pub trait BackendHandle: Send + Sync {
    /// 在 sandbox 内跑命令 (per FR-4.2)
    async fn run(&self, cmd: &CommandSpec, timeout_sec: u32) -> Result<RunResult, SandboxdError>;

    /// 销毁 sandbox (级联 kill, per FR-4.3)
    async fn destroy(&self) -> Result<(), SandboxdError>;

    /// 健康检查
    async fn health(&self) -> Result<HealthStatus, SandboxdError>;
}

/// backend 工厂 (per DD §2.1 BackendFactory trait)
pub trait BackendFactory: Send + Sync {
    /// 创建当前平台 backend
    fn create(&self) -> Result<Box<dyn Backend>, SandboxdError>;
}

/// 默认工厂: 按编译目标平台选择 (per DD §2.1 DefaultBackendFactory)
pub struct DefaultBackendFactory;

impl BackendFactory for DefaultBackendFactory {
    fn create(&self) -> Result<Box<dyn Backend>, SandboxdError> {
        #[cfg(windows)]
        {
            Ok(Box::new(windows::WindowsBackend::new()))
        }
        #[cfg(target_os = "linux")]
        {
            Ok(Box::new(linux::LinuxBackend::new()))
        }
        #[cfg(target_os = "macos")]
        {
            Ok(Box::new(macos::MacosBackend::new()))
        }
        #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
        {
            Err(SandboxdError::PlatformNotSupported(
                std::env::consts::OS.to_string(),
            ))
        }
    }
}

/// 基础命令构造: argv + env (KEY=VALUE) + cwd + stdio 管线 (per FR-4.2)
///
/// env 值只注入子进程, 永不打印 (per 守门 #5)。
pub(crate) fn build_command(spec: &CommandSpec) -> Result<tokio::process::Command, SandboxdError> {
    build_command_wrapped(spec.cmd.clone(), spec)
}

/// 包装版命令构造: 平台后端给入完整 argv (netns / unshare / capsh 前缀), 其余同 [`build_command`]
pub(crate) fn build_command_wrapped(
    argv: Vec<String>,
    spec: &CommandSpec,
) -> Result<tokio::process::Command, SandboxdError> {
    let (exe, args) = argv.split_first().ok_or_else(|| {
        SandboxdError::InvalidPolicy("command argv must not be empty".to_string())
    })?;
    let mut cmd = tokio::process::Command::new(exe);
    cmd.args(args);
    for kv in &spec.env {
        if let Some((k, v)) = kv.split_once('=') {
            cmd.env(k, v);
        }
    }
    if !spec.cwd.is_empty() {
        cmd.current_dir(&spec.cwd);
    }
    if spec.capture_output {
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
    } else {
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());
    }
    cmd.stdin(std::process::Stdio::null());
    cmd.kill_on_drop(true);
    Ok(cmd)
}

/// 等待子进程完成 (含命令级超时, per FR-4.2 走 session 内 timeout)
///
/// - 超时: 跨平台 kill 子进程 (Windows `taskkill /T /F` 级联 / Unix `kill -9`),
///   返回 [`TIMEOUT_EXIT_CODE`] (124) + `timed_out=true`
/// - 未超时: 返回真实退出码 + stdout/stderr
pub(crate) async fn await_child(
    child: tokio::process::Child,
    timeout_sec: u32,
) -> (i32, String, String, bool) {
    if timeout_sec == 0 {
        return match child.wait_with_output().await {
            Ok(o) => (
                o.status.code().unwrap_or(TIMEOUT_EXIT_CODE),
                String::from_utf8_lossy(&o.stdout).into_owned(),
                String::from_utf8_lossy(&o.stderr).into_owned(),
                false,
            ),
            Err(e) => (-1, String::new(), format!("wait error: {e}"), false),
        };
    }
    let pid = child.id();
    let mut wait_task = tokio::task::spawn(async move { child.wait_with_output().await });
    let out = tokio::select! {
        r = &mut wait_task => Some(r),
        _ = tokio::time::sleep(Duration::from_secs(timeout_sec as u64)) => None,
    };
    match out {
        Some(Ok(Ok(o))) => (
            o.status.code().unwrap_or(TIMEOUT_EXIT_CODE),
            String::from_utf8_lossy(&o.stdout).into_owned(),
            String::from_utf8_lossy(&o.stderr).into_owned(),
            false,
        ),
        Some(Ok(Err(e))) => (-1, String::new(), format!("wait error: {e}"), false),
        Some(Err(e)) => (
            -1,
            String::new(),
            format!("wait task join error: {e}"),
            false,
        ),
        None => {
            // 超时: kill 子进程 (级联), wait_task 随子进程退出正常收尾
            kill_by_pid(pid);
            let _ = tokio::time::timeout(Duration::from_secs(3), wait_task).await;
            (TIMEOUT_EXIT_CODE, String::new(), String::new(), true)
        }
    }
}

/// 跨平台 kill 子进程 (含进程树): Windows `taskkill /PID <pid> /T /F` / Unix `kill -9 <pid>`
fn kill_by_pid(pid: Option<u32>) {
    let Some(pid) = pid else { return };
    #[cfg(windows)]
    let mut cmd = std::process::Command::new("taskkill");
    #[cfg(not(windows))]
    let mut cmd = std::process::Command::new("kill");
    #[cfg(windows)]
    cmd.args(["/PID", &pid.to_string(), "/T", "/F"]);
    #[cfg(not(windows))]
    cmd.args(["-9", &pid.to_string()]);
    let _ = cmd
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

/// 共享 run 实现 (三平台 handle 复用, per DD §3.2 RunCommand 时序)
///
/// - `wrap`: 平台包装 (netns / unshare / capsh / sandbox-exec 前缀)
/// - `post_spawn`: spawn 后集成 (Job Object assign / cgroup.procs 写入)
/// - `pre_check`: spawn 前策略检查, 返回违规列表 (fs cwd / capability 拦截)
pub(crate) async fn run_command_common<F, G>(
    spec: &CommandSpec,
    timeout_sec: u32,
    wrap: F,
    post_spawn: G,
    mut violations: Vec<String>,
) -> Result<RunResult, SandboxdError>
where
    F: Fn(&CommandSpec) -> Result<tokio::process::Command, SandboxdError>,
    G: Fn(&mut tokio::process::Child) -> Result<(), SandboxdError>,
{
    let t0 = Instant::now();
    let mut cmd = wrap(spec)?;
    let mut child = cmd
        .spawn()
        .map_err(|e| SandboxdError::Internal(format!("spawn failed: {e}")))?;
    post_spawn(&mut child)?;
    let (code, stdout, stderr, timed_out) = await_child(child, timeout_sec).await;
    if timed_out {
        violations.push(format!(
            "resource: command timed out after {timeout_sec}s (child killed)"
        ));
    }
    Ok(RunResult {
        returncode: code,
        stdout,
        stderr,
        latency_ms: t0.elapsed().as_millis() as u64,
        hit_violations: violations,
    })
}

/// capability 维 spawn 前拦截: 提权工具 deny-list (per FR-5.1 v0.1 简化 + D-8)
///
/// Linux 后端走 `capsh` drop (kernel 级), Windows / macOS v0.1 无法安全调整 token,
/// 故以 spawn 前拦截提权可执行文件作为等价防线 (BLOCK, per FR-3.4 drop 语义)。
pub(crate) fn check_capability_escalation(spec: &CommandSpec) -> Option<String> {
    const ESCALATION_TOOLS: &[&str] = &[
        "sudo", "runas", "psexec", "gsudo", "doas", "su", "setcap", "getcap", "regini",
    ];
    let exe = spec.cmd.first()?;
    let base = exe
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(exe.as_str())
        .to_ascii_lowercase();
    if ESCALATION_TOOLS.contains(&base.as_str()) {
        Some(format!(
            "privilege escalation tool '{exe}' intercepted (capability dimension drop)"
        ))
    } else {
        None
    }
}

/// fs 维 spawn 前检查: cwd 读访问判定 (audit-only, per FR-4.2 fail-open 派生)
///
/// v0.1 仅审计不阻断; cwd 写强制 AppContainer / mount ns 级 v0.2 拍摄。
pub(crate) fn check_cwd_fs(policy: &crate::policy::fs::FsPolicy, cwd: &str) -> Vec<String> {
    use crate::policy::fs::FsDecision;
    match policy.check_path(cwd, false) {
        FsDecision::Blocked { reason } => {
            vec![format!("fs: cwd '{cwd}' not in fs allowlist ({reason})")]
        }
        _ => vec![],
    }
}
