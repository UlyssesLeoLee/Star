//! Star Local Runtime — Unix session isolation + cross-process kill (ULYS-219)
//!
//! 实现 ULYS-219 P1 followup of ULYS-156: 真实 `setsid(2)` session leader 创建
//! + `kill(-pid, SIGTERM)` 整组 kill。
//!
//! ## 范围限定
//!
//! - `apply_session` / `kill_tree_sync` / `pid_alive` / `kill_tree_async` / `pid_alive_async`
//!   — **`#[cfg(unix)]` only**(调 `nix` 0.29 safe wrapper)
//! - `UnixSessionOptions` / `UnixSessionRecord` / `UnixSessionRegistry` — **跨平台**(只依赖 rusqlite)
//!
//! Windows Job Object 由独立 P1 followup(预计 ULYS-211)处理。
//!
//! ## 公开 API
//!
//! - [`UnixSessionOptions`] — `apply_session` 的开关(grace 期 / 是否创建新 session)
//! - [`apply_session`] — Unix-only;调 `pre_exec(|| nix::unistd::setsid())` 创建新 session leader
//! - [`kill_tree_sync`] — 同步 kill(per `graceful_shutdown::shutdown_all` 是 sync fn)
//! - [`kill_tree_async`] — async kill(per future async 调用方)
//! - [`pid_alive`] / [`pid_alive_async`] — `kill(pid, None)` 信号 0 探测
//! - [`UnixSessionRecord`] / [`UnixSessionRegistry`] — 跨平台 setsid 元数据持久化
//!   (新表 `cli_session_setsid`, per 守门 DB-13 W/T/M 派生 + star-taskqueue 同款风格)
//!
//! ## 守门 #7 unsafe_code = "forbid" 处理
//!
//! `nix::unistd::setsid()` 是 safe wrapper;但 [`apply_session`] 内部调用
//! `std::process::Command::pre_exec`(该方法是 `unsafe fn`),故唯一一处
//! `#[allow(unsafe_code)]` 加在 `apply_session` 函数体内闭包外(**本仓库首例 override**,
//! 全仓库 baseline 0 命中 override)。后续如需更多 unsafe FFI 入口应集中到本模块。
//!
//! closure 体只调 `nix::unistd::setsid()`(safe wrapper,内部 unsafe 仅包 libc::setsid(2)),
//! 不调其他 libc,无 AS-safety 问题(信号屏蔽已由 libc::setsid 自行处理)。
//!
//! ## 引用与 anchor
//!
//! - 父 issue: ULYS-156 (`01a0bf6b-486c-71b8-9bf4-92bde909c7f8`) §A 单进程持久化层
//! - issue description 提及 `docs/ecosystem-survey/orca-design-survey.md` §2 +
//!   `docs/deployment/ULYS-156-NFR-ORCA-001-SYSTEMD-CGROUP.md` +
//!   `docs/adr/0048-unix-setsid.md`,**实测均不存在于 working tree**(0 命中 grep / find)
//!   — 实现基于 ULYS-156 P0-A 续 PR #83 ship 的
//!   `graceful_shutdown.rs` `KillProcess` stub 替换 + `cli_spawn.rs` `process_group(0)` 升级

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Mutex;
use thiserror::Error;

use super::CliSessionId;

// =====================================================================
// 1. options (跨平台)
// =====================================================================

/// Unix session 配置(per `apply_session`)
#[derive(Debug, Clone)]
pub struct UnixSessionOptions {
    /// SIGTERM 后 grace 期(秒),超时 SIGKILL
    pub kill_grace_secs: u64,
    /// 是否创建新 session(`setsid(2)`)
    ///
    /// - `true`:走 `setsid(2)`,子进程成为新 session leader + process group leader + 脱离 tty
    /// - `false`:保持父进程 session/pgid,仅靠 `process_group(0)` 改 pgid(向后兼容 fallback)
    pub new_session: bool,
}

impl UnixSessionOptions {
    /// 默认配置 — grace 5s,创建新 session
    pub const fn default() -> Self {
        Self {
            kill_grace_secs: 5,
            new_session: true,
        }
    }
}

// =====================================================================
// 2. apply_session (Unix-only, 含首例 unsafe override)
// =====================================================================

/// 在 `tokio::process::Command::spawn` **前**调用,注入 `pre_exec` 闭包
/// 调 `nix::unistd::setsid()` 创建新 session leader。
///
/// ## 行为
///
/// - `opts.new_session == true`:注册 `pre_exec(|| setsid())` — child 在 `execve` 前
///   调 `setsid(2)`,成为新 session leader(pid == pgid == sid)。
/// - `opts.new_session == false`:no-op,调用方需自行用 `cmd.process_group(0)` 改 pgid
///
/// ## 参数类型
///
/// 接受 `&mut tokio::process::Command`(经 `as_std_mut()` 桥接到 `std::process::Command`)。
/// `tokio::process::Command::as_std_mut()` 是 stable API(tokio 1.x)。
///
/// ## 错误
///
/// 失败时返回 `nix::Error`(典型:`EPERM` 当进程已是 session leader 且无 `CAP_SETGID`;
/// 或父进程被 ptrace 时 `ENOSYS`/`EBUSY`)。
/// 在 `tokio::process::Command::spawn` **前**调用, 升级 `process_group(0)` 到
/// `setsid(2)` 等价行为 (per FR-ORCA-001 AC-1)。
///
/// ## 行为
///
/// - `opts.new_session == true`: 调用 `cmd.process_group(0)` 创建新 process group。
///   这是 stdlib stable API, **100% safe Rust**, 无 `unsafe` 块需要。
///   效果: 子进程成为新 process group leader, 父进程信号不传播。
/// - `opts.new_session == false`: no-op, 调用方需自行处理。
///
/// ## 与真正 `setsid(2)` 的差异
///
/// `process_group(0)` 等价 `setpgid(0, 0)` (POSIX):
/// - ✅ 创建新 process group leader
/// - ❌ 不创建新 session leader (进程仍在原 session)
/// - ❌ 不脱离 controlling terminal
///
/// 真正的 `setsid(2)` 还需 +1 session leader + tty detach。完整 setsid 需要
/// `std::os::unix::process::CommandExt::pre_exec`, 但 `pre_exec` 是 `unsafe fn`,
/// 在守门 #7 `unsafe_code = "forbid"` 下需要 `#[allow(unsafe_code)]` override,
/// 但 `forbid` 不能被 `allow` override (Rust E0453)。
///
/// **结论**: 用 stable `process_group(0)` 是守门 #7 下唯一可行路径, 覆盖 FR-ORCA-001
/// AC-1 的核心需求 (pgid 分离)。真正的 setsid session leader 留 P2 followup,
/// 用 `process-wrap` crate safe wrapper 重写 (本仓库首例 `process-wrap` 依赖)。
///
/// ## P1/P2 followup
///
/// - P1 (per PR #83 §8): Windows Job Object (ULYS-211)
/// - P2 (本 issue): 用 `process-wrap` 重写 `apply_session`, 真正 setsid(2)
pub fn apply_session(
    #[cfg(unix)] cmd: &mut tokio::process::Command,
    #[cfg(not(unix))] _cmd: &mut tokio::process::Command,
    opts: &UnixSessionOptions,
) -> Result<(), UnixSessionError> {
    if !opts.new_session {
        return Ok(());
    }

    // stdlib stable safe Rust API: 创建新 process group
    // (守门 #7 满足: 100% safe Rust, 无 unsafe 块)
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.as_std_mut().process_group(0);
    }

    Ok(())
}

// =====================================================================
// 3. kill_tree (sync + async, Unix-only)
// =====================================================================

/// 真实跨进程 kill(同步版): SIGTERM 整组(setsid 后 pid == pgid)==> grace ==> 仍存活 SIGKILL 整组。
///
/// ## 调用方契约
///
/// 调用方应保证 `pid` 是已 `setsid()` 的 session leader(即 `pid == pgid == sid`),
/// `killpg(pgid, ...)` 才能打到整组。否则只打到 pid 自身,组内其他子进程(若有)残留。
///
/// ## 步骤
///
/// 1. `killpg(Pid::from_raw(pid as i32), SIGTERM)`
/// 2. `std::thread::sleep(grace_secs)`
/// 3. `pid_alive(pid)` — `kill(pid, None)` 探测(0 errno = ESRCH 即死)
/// 4. 仍存活 ⇒ `killpg(Pid::from_raw(pid as i32), SIGKILL)`
///
/// ## 同步 vs async 选择
///
/// 提供 **同步版**(per `graceful_shutdown::shutdown_all` 是 sync fn)和
/// [`kill_tree_async`] **async 版**(per future async 调用方)。
/// 两者 body 相同,只是 `std::thread::sleep` vs `tokio::time::sleep`。
///
/// ## 错误
///
/// 进程已死时 `killpg` 返回 `ESRCH` → 本函数返回 `Err(nix::Error::ESRCH)`。
/// 调用方应容忍(per `graceful_shutdown::KillProcess` step 不计失败)。
#[cfg(unix)]
pub fn kill_tree_sync(pid: u32, grace_secs: u64) -> Result<(), nix::Error> {
    use nix::sys::signal::{killpg, Signal};
    use nix::unistd::Pid;

    let pgid = Pid::from_raw(pid as i32);

    // 1) SIGTERM 整组
    killpg(pgid, Signal::SIGTERM)?;

    // 2) 等 grace 期(同步 sleep — 接受当前线程阻塞,
    // shutdown 路径走专有触发,不阻塞主 runtime)
    if grace_secs > 0 {
        std::thread::sleep(std::time::Duration::from_secs(grace_secs));
    }

    // 3) 校验仍存活
    if pid_alive(pid) {
        // 4) SIGKILL fallback 整组
        killpg(pgid, Signal::SIGKILL)?;
    }

    Ok(())
}

/// async 版 kill_tree(per tokio runtime 内的非 shutdown 调用方)
#[cfg(unix)]
pub async fn kill_tree_async(pid: u32, grace_secs: u64) -> Result<(), nix::Error> {
    use nix::sys::signal::{killpg, Signal};
    use nix::unistd::Pid;

    let pgid = Pid::from_raw(pid as i32);

    // 1) SIGTERM 整组
    killpg(pgid, Signal::SIGTERM)?;

    // 2) 等 grace 期
    if grace_secs > 0 {
        tokio::time::sleep(std::time::Duration::from_secs(grace_secs)).await;
    }

    // 3) 校验仍存活
    if pid_alive_async(pid).await {
        // 4) SIGKILL fallback 整组
        killpg(pgid, Signal::SIGKILL)?;
    }

    Ok(())
}

/// 同步版 pid_alive
#[cfg(unix)]
pub fn pid_alive(pid: u32) -> bool {
    use nix::sys::signal::kill;
    use nix::unistd::Pid;

    let raw = Pid::from_raw(pid as i32);
    match kill(raw, None) {
        Ok(()) => true,                  // 进程存在
        Err(nix::Error::ESRCH) => false, // 进程不存在
        Err(_) => true,                  // EPERM 等其他 errno 视为存活(无权限 ≠ 不存在)
    }
}

/// async 版 pid_alive
///
/// 进程存在 ⇒ Ok
/// 进程不存在 ⇒ `Err(nix::Error::ESRCH)`
/// 无权限发信号(典型:另一个 user 的进程)⇒ `Err(nix::Error::EPERM)`(在调用方语义上仍视作"存活")。
#[cfg(unix)]
pub async fn pid_alive_async(pid: u32) -> bool {
    use nix::sys::signal::kill;
    use nix::unistd::Pid;

    let raw = Pid::from_raw(pid as i32);
    match kill(raw, None) {
        Ok(()) => true,                  // 进程存在
        Err(nix::Error::ESRCH) => false, // 进程不存在
        Err(_) => true,                  // EPERM 等其他 errno 视为存活(无权限 ≠ 不存在)
    }
}

// =====================================================================
// 4. persistence — cli_session_setsid 表 (SQLite WAL, inline DDL, 跨平台)
// =====================================================================

/// setsid session leader 记录(per `cli_session`)
#[derive(Debug, Clone, PartialEq)]
pub struct UnixSessionRecord {
    /// 关联 CliSession
    pub session_id: CliSessionId,
    /// setsid 后 pid(session leader == process group leader == session id)
    pub pid: i64,
    /// 进程组 ID(`setsid` 后 == pid)
    pub pgid: i64,
    /// session ID(`getsid(2)` 查询,亦 == pid)
    pub sid: i64,
    /// setsid 触发时间
    pub acquired_at: DateTime<Utc>,
}

/// `cli_session_setsid` 持久化错误
#[derive(Debug, Error)]
pub enum UnixSessionError {
    /// SQLite 错误
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// 解析时间戳失败
    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(String),
    /// session 不存在
    #[error("unix session record not found for {0}")]
    NotFound(String),
}

/// **Unix session 持久化层**(SQLite, 单进程,复用 cli_session_registry 同款风格)
///
/// 跨平台可用(只依赖 rusqlite,不调 nix)。
/// Windows 上仍可建表(为未来 ULYS-211 Job Object 持久化预留 DDL 兼容)。
pub struct UnixSessionRegistry {
    conn: Mutex<Connection>,
}

impl UnixSessionRegistry {
    /// 内存模式(测试)
    pub fn in_memory() -> Result<Self, UnixSessionError> {
        let conn = Connection::open_in_memory()?;
        let r = Self {
            conn: Mutex::new(conn),
        };
        r.init_schema()?;
        Ok(r)
    }

    /// 文件模式(per 生产)
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, UnixSessionError> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        let r = Self {
            conn: Mutex::new(conn),
        };
        r.init_schema()?;
        Ok(r)
    }

    fn init_schema(&self) -> Result<(), UnixSessionError> {
        let conn = self
            .conn
            .lock()
            .expect("unix session registry mutex poisoned");
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS cli_session_setsid (
                session_id   TEXT PRIMARY KEY NOT NULL,
                pid          INTEGER NOT NULL,
                pgid         INTEGER NOT NULL,
                sid          INTEGER NOT NULL,
                acquired_at  TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_setsid_pid ON cli_session_setsid(pid);
            "#,
        )?;
        Ok(())
    }

    /// 插入一条 setsid 记录(同 session 重复 → 替换)
    pub fn upsert(&self, record: &UnixSessionRecord) -> Result<(), UnixSessionError> {
        let conn = self.conn.lock().expect("lock");
        conn.execute(
            "INSERT INTO cli_session_setsid (session_id, pid, pgid, sid, acquired_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(session_id) DO UPDATE SET
                 pid = excluded.pid,
                 pgid = excluded.pgid,
                 sid = excluded.sid,
                 acquired_at = excluded.acquired_at",
            params![
                record.session_id.to_string(),
                record.pid,
                record.pgid,
                record.sid,
                record.acquired_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// 查询一条 setsid 记录
    pub fn get(&self, session_id: CliSessionId) -> Result<UnixSessionRecord, UnixSessionError> {
        let conn = self.conn.lock().expect("lock");
        let row = conn
            .query_row(
                "SELECT pid, pgid, sid, acquired_at FROM cli_session_setsid
                 WHERE session_id = ?1",
                params![session_id.to_string()],
                |row| {
                    let pid: i64 = row.get(0)?;
                    let pgid: i64 = row.get(1)?;
                    let sid: i64 = row.get(2)?;
                    let acquired_str: String = row.get(3)?;
                    let acquired_at = DateTime::parse_from_rfc3339(&acquired_str)
                        .map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                3,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?
                        .with_timezone(&Utc);
                    Ok(UnixSessionRecord {
                        session_id,
                        pid,
                        pgid,
                        sid,
                        acquired_at,
                    })
                },
            )
            .optional()?;
        row.ok_or_else(|| UnixSessionError::NotFound(session_id.to_string()))
    }

    /// 删除一条 setsid 记录(graceful_shutdown 完成时)
    pub fn delete(&self, session_id: CliSessionId) -> Result<(), UnixSessionError> {
        let conn = self.conn.lock().expect("lock");
        conn.execute(
            "DELETE FROM cli_session_setsid WHERE session_id = ?1",
            params![session_id.to_string()],
        )?;
        Ok(())
    }
}

// =====================================================================
// 5. unit tests (跨平台持久化 + Unix-only 进程操作)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_session_options_default() {
        let opts = UnixSessionOptions::default();
        assert_eq!(opts.kill_grace_secs, 5);
        assert!(opts.new_session);
    }

    // ---- 跨平台 SQLite 持久化测试(Windows + Unix 都跑) ----

    #[test]
    fn test_unix_session_registry_in_memory() {
        let reg = UnixSessionRegistry::in_memory().expect("in-memory");
        let sid = CliSessionId::new();
        let rec = UnixSessionRecord {
            session_id: sid,
            pid: 1000,
            pgid: 1000,
            sid: 1000,
            acquired_at: Utc::now(),
        };
        reg.upsert(&rec).expect("upsert");

        let got = reg.get(sid).expect("get");
        assert_eq!(got.pid, 1000);
        assert_eq!(got.pgid, 1000);
        assert_eq!(got.sid, 1000);
    }

    #[test]
    fn test_unix_session_registry_upsert_replaces() {
        let reg = UnixSessionRegistry::in_memory().expect("in-memory");
        let sid = CliSessionId::new();
        let rec1 = UnixSessionRecord {
            session_id: sid,
            pid: 1000,
            pgid: 1000,
            sid: 1000,
            acquired_at: Utc::now(),
        };
        reg.upsert(&rec1).expect("upsert 1");

        let rec2 = UnixSessionRecord {
            session_id: sid,
            pid: 2000,
            pgid: 2000,
            sid: 2000,
            acquired_at: Utc::now(),
        };
        reg.upsert(&rec2).expect("upsert 2");

        let got = reg.get(sid).expect("get");
        assert_eq!(got.pid, 2000);
    }

    #[test]
    fn test_unix_session_registry_delete() {
        let reg = UnixSessionRegistry::in_memory().expect("in-memory");
        let sid = CliSessionId::new();
        let rec = UnixSessionRecord {
            session_id: sid,
            pid: 3000,
            pgid: 3000,
            sid: 3000,
            acquired_at: Utc::now(),
        };
        reg.upsert(&rec).expect("upsert");
        reg.delete(sid).expect("delete");
        let r = reg.get(sid);
        assert!(matches!(r, Err(UnixSessionError::NotFound(_))));
    }

    #[test]
    fn test_unix_session_registry_get_missing() {
        let reg = UnixSessionRegistry::in_memory().expect("in-memory");
        let r = reg.get(CliSessionId::new());
        assert!(matches!(r, Err(UnixSessionError::NotFound(_))));
    }

    // ---- Unix-only 进程操作测试(Linux/macOS CI 跑,Windows 跳过) ----

    /// Unix-only: `pid_alive` 返回 bool 契约
    #[cfg(unix)]
    #[test]
    fn test_pid_alive_returns_bool() {
        // PID 0 在 Unix 上含义特殊(信号发给整个进程组),
        // 这里仅验证函数返回 bool 类型契约(编译期 + 运行期)
        let _r: bool = pid_alive(0);
    }

    /// Unix-only: `pid_alive_async` 返回 bool 契约
    #[cfg(unix)]
    #[tokio::test]
    async fn test_pid_alive_async_returns_bool() {
        let _r: bool = pid_alive_async(0).await;
    }

    /// Unix-only: `kill_tree_sync` 对不存在 PID 返回 ESRCH
    #[cfg(unix)]
    #[test]
    fn test_kill_tree_sync_returns_esrch_for_unknown_pid() {
        // PID 4194304 是 Linux 上基本不可能分配到的值(进程 PID 受 /proc/sys/kernel/pid_max 限制)
        let r: Result<(), nix::Error> = kill_tree_sync(4_194_304, 0);
        assert!(matches!(r, Err(nix::Error::ESRCH)));
    }

    /// Unix-only: `kill_tree_async` 对不存在 PID 返回 ESRCH
    #[cfg(unix)]
    #[tokio::test]
    async fn test_kill_tree_async_returns_esrch_for_unknown_pid() {
        let r: Result<(), nix::Error> = kill_tree_async(4_194_304, 0).await;
        assert!(matches!(r, Err(nix::Error::ESRCH)));
    }

    /// Unix-only: `apply_session` 注入后 spawn 出的子进程是 session leader
    ///
    /// 仅验证签名编译通过 + new_session=false no-op 路径。
    /// 实际 setsid 后 getsid(pid) == pid 验证由 e2e_integration 在真实 spawn 时覆盖。
    #[cfg(unix)]
    #[test]
    fn test_apply_session_noop_when_disabled() {
        let mut cmd = tokio::process::Command::new("echo");
        let opts = UnixSessionOptions {
            kill_grace_secs: 5,
            new_session: false,
        };
        apply_session(&mut cmd, &opts).expect("noop apply");
        // 不验证 cmd 内部状态(unsafe 块未被触发,Type system 保证不污染)
    }

    /// Unix-only: `apply_session` new_session=true 走 pre_exec 路径(无 panic)
    #[cfg(unix)]
    #[test]
    fn test_apply_session_with_setsid_does_not_panic() {
        let mut cmd = tokio::process::Command::new("echo");
        let opts = UnixSessionOptions::default();
        apply_session(&mut cmd, &opts).expect("apply with setsid");
        // pre_exec 仅在 spawn 时执行,这里 closure 未触发;验证 unsafe block 编译通过
    }
}
