/*
@cypher schema=1 source_sha256=421a53da64bacf269cb63708abeac01f448825f90a91b15b107577c92176c947
MERGE (self:File {path:"crates/sandboxd/src/backend/windows.rs"})
@endcypher
*/
//! backend::windows — Windows 隔离后端 (per FR-5.1)
//!
//! | 维 | v0.1 实装 | v0.2 拍摄 (per DD §5.1 v0.1 简化标注) |
//! |---|---|---|
//! | resource | Job Objects (win32job safe wrapper): working-set 内存上限 + kill_on_job_close 级联 kill + cpu_percent→priority class | ProcessMemoryLimit / ActiveProcessLimit / CpuRate hard cap (需 unsafe Win32) |
//! | network | policy+audit (spawn 前 cwd / 域名判定由 sandboxd 层审计) | WFP provider 完整版 |
//! | fs | policy+audit | AppContainer (CreateProcessAsUser + 受限 token) |
//! | capability | spawn 前提权工具拦截 (per D-8) + 隐式 drop | token privilege adjust (drop SeDebug 等) |
//!
//! 全部 0 unsafe (per 守门 #7), win32job 内部 unsafe 已被 crate 隔离。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use async_trait::async_trait;
use tracing::warn;
use uuid::Uuid;
use win32job::{ExtendedLimitInfo, Job, PriorityClass};

use super::{
    check_capability_escalation, check_cwd_fs, run_command_common, Backend, BackendHandle,
    CommandSpec, EnforcementDim, EnforcementLevel, EnforcementReport, HealthStatus, RunResult,
};
use crate::error::SandboxdError;
use crate::policy::SessionSpec;

/// Windows 后端 (Job Objects + policy/audit, per FR-5.1)
#[derive(Default)]
pub struct WindowsBackend;

impl WindowsBackend {
    /// 构造 Windows 后端
    pub fn new() -> Self {
        Self
    }

    /// 依 policy 构造 Job Object 限制信息 (per SANDBOX-001 v0.2 §3.1 复用)
    ///
    /// `with_memory`: 是否附加 working-set 内存上限。
    /// v0.1 实测: `JOB_OBJECT_LIMIT_WORKINGSET` 在本机需要
    /// `SE_INCREASE_WORKING_SET` 特权 (0 unsafe 约束下无法 enable), 故内存上限为
    /// best-effort — 失败时以 `with_memory=false` 重试 (保留 kill_on_job_close + priority class),
    /// 并标注 PolicyAuditOnly (硬 ProcessMemoryLimit 需 unsafe Win32 → v0.2)。
    fn build_limit_info(policy: &SessionSpec, with_memory: bool) -> ExtendedLimitInfo {
        let mut info = ExtendedLimitInfo::new();
        let r = &policy.resource;
        if with_memory && r.memory_mb > 0 {
            // min 必须 ≥ 系统最小 working set (实测 1MB 可过参数校验)
            let max = r.memory_mb as usize * 1024 * 1024;
            let min = max.min(1024 * 1024);
            info.limit_working_memory(min, max);
        }
        if r.kill_on_parent_exit {
            info.limit_kill_on_job_close();
        }
        if r.cpu_percent > 0 {
            info.limit_priority_class(cpu_priority_class(r.cpu_percent));
        }
        info
    }
}

/// cpu_percent → priority class (v0.1 软限, hard cap CpuRate v0.2)
///
/// - 1..=33 → Idle
/// - 34..=66 → BelowNormal
/// - 67..=100 → Normal (不惩罚)
pub fn cpu_priority_class(cpu_percent: u32) -> PriorityClass {
    match cpu_percent {
        1..=33 => PriorityClass::Idle,
        34..=66 => PriorityClass::BelowNormal,
        _ => PriorityClass::Normal,
    }
}

/// Windows backend handle: 1 session = 1 Job Object
pub struct WindowsHandle {
    /// Job Object (drop 时 KILL_ON_JOB_CLOSE 级联 kill 所有 session 内进程, per FR-4.3)
    job: Mutex<Option<Job>>,
    /// 创建时冻结的策略快照
    policy: SessionSpec,
    /// 已销毁标记 (destroy 后拒绝新命令)
    destroyed: AtomicBool,
    /// 内存上限是否成功内核强制 (false = PolicyAuditOnly 降级)
    memory_enforced: bool,
}

impl WindowsHandle {
    /// spawn 后把子进程加入 Job (per FR-5.1 Job Objects)
    fn post_spawn(&self, child: &mut tokio::process::Child) -> Result<(), SandboxdError> {
        // 子进程已退出 → 无需 assign (避免 assign 已死进程报错)
        match child.try_wait() {
            Ok(Some(_)) => return Ok(()),
            Ok(None) => {}
            Err(e) => return Err(SandboxdError::Internal(format!("try_wait failed: {e}"))),
        }
        // RawHandle = *mut c_void → cast usize → cast isize 给 win32job (per domain-local-runtime 同款)
        let raw: isize = match child.raw_handle() {
            Some(h) => h as usize as isize,
            None => 0,
        };
        let guard = self
            .job
            .lock()
            .map_err(|_| SandboxdError::Internal("job mutex poisoned".to_string()))?;
        if let Some(job) = guard.as_ref() {
            job.assign_process(raw)
                .map_err(|e| SandboxdError::Internal(format!("assign job failed: {e}")))?;
        } else {
            return Err(SandboxdError::Internal(
                "session destroyed: job handle gone".to_string(),
            ));
        }
        Ok(())
    }
}

#[async_trait]
impl Backend for WindowsBackend {
    async fn apply(
        &self,
        policy: &SessionSpec,
        _session_id: Uuid,
    ) -> Result<Box<dyn BackendHandle>, SandboxdError> {
        // 内存上限 best-effort: working-set 限制失败 → 无内存限制重试 (per v0.1 实测特权约束)
        let mut memory_enforced = policy.resource.memory_mb > 0;
        let mut job = Job::create_with_limit_info(&Self::build_limit_info(policy, true));
        if job.is_err() && policy.resource.memory_mb > 0 {
            memory_enforced = false;
            warn!(
                "working-set memory limit unavailable (privilege/validation) — falling back to no-memory-limit job (PolicyAuditOnly)"
            );
            job = Job::create_with_limit_info(&Self::build_limit_info(policy, false));
        }
        let job =
            job.map_err(|e| SandboxdError::Internal(format!("CreateJobObjectW failed: {e}")))?;
        Ok(Box::new(WindowsHandle {
            job: Mutex::new(Some(job)),
            policy: policy.clone(),
            destroyed: AtomicBool::new(false),
            memory_enforced,
        }))
    }

    fn is_supported(&self) -> bool {
        true
    }

    fn enforcement(&self) -> Vec<EnforcementReport> {
        vec![
            EnforcementReport {
                dim: EnforcementDim::Resource,
                level: EnforcementLevel::Kernel,
                detail: "Job Objects (win32job safe wrapper): kill_on_job_close 级联 kill + priority class 内核强制; working-set 内存上限 best-effort (特权不足自动降级 PolicyAuditOnly, 硬 ProcessMemoryLimit v0.2)".to_string(),
            },
            EnforcementReport {
                dim: EnforcementDim::Network,
                level: EnforcementLevel::PolicyAuditOnly,
                detail: "WFP provider 完整版 v0.2 拍摄 (per DD §5.1 v0.1 简化)".to_string(),
            },
            EnforcementReport {
                dim: EnforcementDim::Fs,
                level: EnforcementLevel::PolicyAuditOnly,
                detail: "AppContainer 受限 token v0.2 拍摄 (per DD §5.1 v0.1 简化)".to_string(),
            },
            EnforcementReport {
                dim: EnforcementDim::Capability,
                level: EnforcementLevel::PolicyAuditOnly,
                detail: "spawn 前提权工具拦截 (per D-8) + 隐式 drop; token privilege adjust v0.2".to_string(),
            },
        ]
    }
}

#[async_trait]
impl BackendHandle for WindowsHandle {
    async fn run(&self, cmd: &CommandSpec, timeout_sec: u32) -> Result<RunResult, SandboxdError> {
        if self.destroyed.load(Ordering::SeqCst) {
            return Err(SandboxdError::Internal(
                "session destroyed: reject new commands".to_string(),
            ));
        }
        // capability 维: 提权工具 BLOCK (per FR-3.4 drop 语义, 跟 Linux capsh drop 等价)
        if let Some(cap) = check_capability_escalation(cmd) {
            return Err(SandboxdError::CapabilityViolation { cap });
        }
        // fs 维: cwd 审计 (fail-open, per FR-4.2)
        let violations = if cmd.cwd.is_empty() {
            Vec::new()
        } else {
            check_cwd_fs(&self.policy.fs, &cmd.cwd)
        };
        run_command_common(
            cmd,
            timeout_sec,
            super::build_command,
            |child| self.post_spawn(child),
            violations,
        )
        .await
    }

    async fn destroy(&self) -> Result<(), SandboxdError> {
        self.destroyed.store(true, Ordering::SeqCst);
        // drop Job handle → KILL_ON_JOB_CLOSE 触发 OS 级联 kill 所有 session 内进程 (per FR-4.3)
        let mut guard = self
            .job
            .lock()
            .map_err(|_| SandboxdError::Internal("job mutex poisoned".to_string()))?;
        if guard.take().is_none() {
            warn!("windows backend: destroy called twice (job already gone)");
        }
        Ok(())
    }

    async fn health(&self) -> Result<HealthStatus, SandboxdError> {
        Ok(HealthStatus {
            healthy: !self.destroyed.load(Ordering::SeqCst),
            detail: if self.destroyed.load(Ordering::SeqCst) {
                "job object released (session destroyed)".to_string()
            } else {
                format!(
                    "job object active (memory_enforced={}, kill_on_job_close={})",
                    self.memory_enforced, self.policy.resource.kill_on_parent_exit
                )
            },
        })
    }
}
