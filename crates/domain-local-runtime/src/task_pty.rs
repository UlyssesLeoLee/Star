//! CYPHER STRUCTURAL MANIFEST
//! CREATE
//!   (f:File {name:"task_pty.rs",type:"file",language:"rust"}),(m:Module {name:"task_pty",type:"module",language:"rust"}),
//!   (manager:Class {name:"TaskPtyManager",type:"class",language:"rust"}),(session:Class {name:"TaskPtySession",type:"class",language:"rust"}),(output:Class {name:"TaskPtyOutput",type:"class",language:"rust"}),(error:Enum {name:"TaskPtyError",type:"enum",language:"rust"}),
//!   (max_input:Variable {name:"MAX_INPUT_BYTES",type:"variable",language:"rust"}),(max_dimension:Variable {name:"MAX_PTY_DIMENSION",type:"variable",language:"rust"}),(max_chunk:Variable {name:"OUTPUT_CHUNK_BYTES",type:"variable",language:"rust"}),(output_capacity:Variable {name:"OUTPUT_QUEUE_CAPACITY",type:"variable",language:"rust"}),
//!   (manager_new:Function {name:"TaskPtyManager::new",type:"function",language:"rust"}),(manager_spawn:Function {name:"TaskPtyManager::spawn",type:"function",language:"rust"}),(manager_get:Function {name:"TaskPtyManager::get",type:"function",language:"rust"}),(manager_terminate:Function {name:"TaskPtyManager::terminate",type:"function",language:"rust"}),(manager_drop:Function {name:"TaskPtyManager::drop",type:"function",language:"rust"}),
//!   (session_spawn:Function {name:"TaskPtySession::spawn",type:"function",language:"rust"}),(subscribe_output:Function {name:"TaskPtySession::subscribe_output",type:"function",language:"rust"}),(subscribe_exit:Function {name:"TaskPtySession::subscribe_exit",type:"function",language:"rust"}),(write_input:Function {name:"TaskPtySession::write_input",type:"function",language:"rust"}),(resize:Function {name:"TaskPtySession::resize",type:"function",language:"rust"}),(try_wait:Function {name:"TaskPtySession::try_wait",type:"function",language:"rust"}),(terminate:Function {name:"TaskPtySession::terminate",type:"function",language:"rust"}),(session_id:Function {name:"TaskPtySession::session_id",type:"function",language:"rust"}),
//!   (build_command:Function {name:"build_command",type:"function",language:"rust"}),(validate_size:Function {name:"validate_size",type:"function",language:"rust"}),(start_reader:Function {name:"start_output_reader",type:"function",language:"rust"}),(monitor_child:Function {name:"start_child_monitor",type:"function",language:"rust"}),(cleanup_child:Function {name:"terminate_child",type:"function",language:"rust"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(manager),(m)-[:CONTAINS]->(session),(m)-[:CONTAINS]->(output),(m)-[:CONTAINS]->(error),(m)-[:CONTAINS]->(max_input),(m)-[:CONTAINS]->(max_dimension),(m)-[:CONTAINS]->(max_chunk),(m)-[:CONTAINS]->(output_capacity),(m)-[:CONTAINS]->(manager_new),(m)-[:CONTAINS]->(manager_spawn),(m)-[:CONTAINS]->(manager_get),(m)-[:CONTAINS]->(manager_terminate),(m)-[:CONTAINS]->(manager_drop),(m)-[:CONTAINS]->(session_spawn),(m)-[:CONTAINS]->(subscribe_output),(m)-[:CONTAINS]->(subscribe_exit),(m)-[:CONTAINS]->(write_input),(m)-[:CONTAINS]->(resize),(m)-[:CONTAINS]->(try_wait),(m)-[:CONTAINS]->(terminate),(m)-[:CONTAINS]->(session_id),(m)-[:CONTAINS]->(build_command),(m)-[:CONTAINS]->(validate_size),(m)-[:CONTAINS]->(start_reader),(m)-[:CONTAINS]->(monitor_child),(m)-[:CONTAINS]->(cleanup_child),
//!   (manager)-[:HAS_METHOD]->(manager_new),(manager)-[:HAS_METHOD]->(manager_spawn),(manager)-[:HAS_METHOD]->(manager_get),(manager)-[:HAS_METHOD]->(manager_terminate),(manager)-[:HAS_METHOD]->(manager_drop),(session)-[:HAS_METHOD]->(session_spawn),(session)-[:HAS_METHOD]->(subscribe_output),(session)-[:HAS_METHOD]->(subscribe_exit),(session)-[:HAS_METHOD]->(write_input),(session)-[:HAS_METHOD]->(resize),(session)-[:HAS_METHOD]->(try_wait),(session)-[:HAS_METHOD]->(terminate),(session)-[:HAS_METHOD]->(session_id),
//!   (manager_spawn)-[:CALLS]->(session_spawn),(manager_terminate)-[:CALLS]->(terminate),(session_spawn)-[:CALLS]->(validate_size),(session_spawn)-[:CALLS]->(build_command),(session_spawn)-[:CALLS]->(start_reader),(session_spawn)-[:CALLS]->(monitor_child),(session_spawn)-[:CALLS]->(cleanup_child),(resize)-[:CALLS]->(validate_size),(start_reader)-[:USES]->(max_chunk),(start_reader)-[:USES]->(output_capacity),(write_input)-[:USES]->(max_input),(validate_size)-[:USES]->(max_dimension);

use std::{
    collections::HashMap,
    io::{Read, Write},
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::Duration,
};

use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use tokio::sync::{mpsc, watch};
use uuid::Uuid;

use super::PreparedTaskCliExecution;

const MAX_INPUT_BYTES: usize = 64 * 1024;
const MAX_PTY_DIMENSION: u16 = 500;
const OUTPUT_CHUNK_BYTES: usize = 16 * 1024;
const OUTPUT_QUEUE_CAPACITY: usize = 256;

/// Owns interactive PTY processes for already prepared Task CLI executions.
/// This adapter provides terminal I/O only; it does not create an OS sandbox or recheck ACL.
pub(crate) struct TaskPtyManager {
    sessions: RwLock<HashMap<Uuid, Arc<TaskPtySession>>>,
}

impl TaskPtyManager {
    /// Create an empty in-process PTY session manager.
    pub(crate) fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    /// Spawn one interactive process from a previously authorized and prepared execution.
    /// Callers must enforce current ACL, runtime health, audit intent, grant verification,
    /// nonce consumption, and OS sandbox policy before calling this method.
    pub(crate) fn spawn(
        &self,
        session_id: Uuid,
        execution: &PreparedTaskCliExecution,
        cols: u16,
        rows: u16,
    ) -> Result<Arc<TaskPtySession>, TaskPtyError> {
        if session_id.is_nil() {
            return Err(TaskPtyError::InvalidSessionId);
        }
        let mut sessions = self
            .sessions
            .write()
            .map_err(|_| TaskPtyError::Unavailable)?;
        if sessions.contains_key(&session_id) {
            return Err(TaskPtyError::DuplicateSession);
        }
        let session = Arc::new(TaskPtySession::spawn(session_id, execution, cols, rows)?);
        sessions.insert(session_id, Arc::clone(&session));
        Ok(session)
    }

    /// Find an in-process session by its opaque session ID.
    pub(crate) fn get(&self, session_id: Uuid) -> Option<Arc<TaskPtySession>> {
        self.sessions.read().ok()?.get(&session_id).cloned()
    }

    /// Terminate a session and remove it from the in-process registry.
    pub(crate) fn terminate(&self, session_id: Uuid) -> Result<(), TaskPtyError> {
        let session = self
            .sessions
            .read()
            .map_err(|_| TaskPtyError::Unavailable)?
            .get(&session_id)
            .cloned()
            .ok_or(TaskPtyError::SessionNotFound)?;
        session.terminate()?;
        self.sessions
            .write()
            .map_err(|_| TaskPtyError::Unavailable)?
            .remove(&session_id);
        Ok(())
    }
}

impl Drop for TaskPtyManager {
    fn drop(&mut self) {
        if let Ok(sessions) = self.sessions.get_mut() {
            for session in sessions.values() {
                let _ = session.terminate();
            }
        }
    }
}

impl Default for TaskPtyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Raw PTY output chunk with a monotonically increasing per-session sequence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TaskPtyOutput {
    /// Output sequence number, starting at one.
    pub sequence: u64,
    /// Unmodified bytes read from the PTY master.
    pub data: Vec<u8>,
}

/// A running interactive PTY session.
pub(crate) struct TaskPtySession {
    session_id: Uuid,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    master: Mutex<Box<dyn MasterPty + Send>>,
    child: Arc<Mutex<Box<dyn Child + Send>>>,
    output_rx: Mutex<Option<mpsc::Receiver<TaskPtyOutput>>>,
    exit_tx: watch::Sender<Option<i32>>,
}

impl TaskPtySession {
    fn spawn(
        session_id: Uuid,
        execution: &PreparedTaskCliExecution,
        cols: u16,
        rows: u16,
    ) -> Result<Self, TaskPtyError> {
        validate_size(cols, rows)?;
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|_| TaskPtyError::PtyUnavailable)?;
        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|_| TaskPtyError::PtyUnavailable)?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|_| TaskPtyError::PtyUnavailable)?;
        let command = build_command(execution)?;
        let (output_tx, output_rx) = mpsc::channel(OUTPUT_QUEUE_CAPACITY);
        let child: Box<dyn Child + Send> = pair
            .slave
            .spawn_command(command)
            .map_err(|_| TaskPtyError::SpawnFailed)?;
        drop(pair.slave);

        let (exit_tx, _) = watch::channel(None);
        let child = Arc::new(Mutex::new(child));
        if let Err(error) = start_output_reader(session_id, reader, output_tx.clone()) {
            terminate_child(&child);
            return Err(error);
        }
        if let Err(error) = start_child_monitor(Arc::clone(&child), exit_tx.clone()) {
            terminate_child(&child);
            return Err(error);
        }

        Ok(Self {
            session_id,
            writer: Arc::new(Mutex::new(writer)),
            master: Mutex::new(pair.master),
            child,
            output_rx: Mutex::new(Some(output_rx)),
            exit_tx,
        })
    }

    /// Take the bounded PTY byte stream once. Backpressure pauses PTY draining until the
    /// consumer catches up, preserving output before attachment; the caller persists scrollback.
    pub(crate) fn subscribe_output(&self) -> Result<mpsc::Receiver<TaskPtyOutput>, TaskPtyError> {
        self.output_rx
            .lock()
            .map_err(|_| TaskPtyError::Unavailable)?
            .take()
            .ok_or(TaskPtyError::OutputAlreadySubscribed)
    }

    /// Subscribe to the child exit code; `None` means the child is still running.
    pub(crate) fn subscribe_exit(&self) -> watch::Receiver<Option<i32>> {
        self.exit_tx.subscribe()
    }

    /// Write a bounded byte chunk to the interactive process.
    pub(crate) async fn write_input(&self, data: Vec<u8>) -> Result<(), TaskPtyError> {
        if data.len() > MAX_INPUT_BYTES {
            return Err(TaskPtyError::InputTooLarge);
        }
        let writer = Arc::clone(&self.writer);
        tokio::task::spawn_blocking(move || {
            let mut writer = writer.lock().map_err(|_| TaskPtyError::Unavailable)?;
            writer
                .write_all(&data)
                .and_then(|()| writer.flush())
                .map_err(|_| TaskPtyError::InputFailed)
        })
        .await
        .map_err(|_| TaskPtyError::Unavailable)?
    }

    /// Resize the PTY after validating both dimensions.
    pub(crate) fn resize(&self, cols: u16, rows: u16) -> Result<(), TaskPtyError> {
        validate_size(cols, rows)?;
        self.master
            .lock()
            .map_err(|_| TaskPtyError::Unavailable)?
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|_| TaskPtyError::ResizeFailed)
    }

    /// Return the exit code if the PTY child has exited.
    pub(crate) fn try_wait(&self) -> Result<Option<i32>, TaskPtyError> {
        let mut child = self.child.lock().map_err(|_| TaskPtyError::Unavailable)?;
        child
            .try_wait()
            .map(|status| status.map(|status| i32::try_from(status.exit_code()).unwrap_or(-1)))
            .map_err(|_| TaskPtyError::Unavailable)
    }

    /// Terminate the PTY child. The monitor thread records its final status.
    pub(crate) fn terminate(&self) -> Result<(), TaskPtyError> {
        let mut child = self.child.lock().map_err(|_| TaskPtyError::Unavailable)?;
        match child.try_wait().map_err(|_| TaskPtyError::Unavailable)? {
            Some(_) => Ok(()),
            None => child.kill().map_err(|_| TaskPtyError::TerminateFailed),
        }
    }

    /// Return the opaque Local Runtime session identity.
    pub(crate) fn session_id(&self) -> Uuid {
        self.session_id
    }
}

fn build_command(execution: &PreparedTaskCliExecution) -> Result<CommandBuilder, TaskPtyError> {
    if !execution.command.is_absolute() || !execution.worktree_dir.is_absolute() {
        return Err(TaskPtyError::InvalidExecution);
    }
    let mut command = CommandBuilder::new(execution.command.as_os_str());
    command.env_clear();
    command.cwd(execution.worktree_dir.as_os_str());
    command.env("TERM", "xterm-256color");
    for (key, value) in &execution.static_environment {
        command.env(key, value);
    }
    for arg in &execution.args {
        command.arg(arg);
    }
    Ok(command)
}

fn validate_size(cols: u16, rows: u16) -> Result<(), TaskPtyError> {
    if cols == 0 || rows == 0 || cols > MAX_PTY_DIMENSION || rows > MAX_PTY_DIMENSION {
        return Err(TaskPtyError::InvalidSize);
    }
    Ok(())
}

fn start_output_reader(
    session_id: Uuid,
    mut reader: Box<dyn Read + Send>,
    output_tx: mpsc::Sender<TaskPtyOutput>,
) -> Result<(), TaskPtyError> {
    let sequence = AtomicU64::new(1);
    thread::Builder::new()
        .name(format!("task-pty-output-{session_id}"))
        .spawn(move || {
            let mut buffer = vec![0_u8; OUTPUT_CHUNK_BYTES];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => break,
                    Ok(length) => {
                        let item = TaskPtyOutput {
                            sequence: sequence.fetch_add(1, Ordering::Relaxed),
                            data: buffer[..length].to_vec(),
                        };
                        if output_tx.blocking_send(item).is_err() {
                            break;
                        }
                    }
                }
            }
        })
        .map_err(|_| TaskPtyError::Unavailable)?;
    Ok(())
}

fn start_child_monitor(
    child: Arc<Mutex<Box<dyn Child + Send>>>,
    exit_tx: watch::Sender<Option<i32>>,
) -> Result<(), TaskPtyError> {
    thread::Builder::new()
        .name("task-pty-child-monitor".to_string())
        .spawn(move || {
            loop {
                let result = child
                    .lock()
                    .map_err(|_| ())
                    .and_then(|mut child| child.try_wait().map_err(|_| ()));
                match result {
                    Ok(Some(status)) => {
                        exit_tx.send_replace(Some(i32::try_from(status.exit_code()).unwrap_or(-1)));
                        break;
                    }
                    Ok(None) => thread::sleep(Duration::from_millis(100)),
                    Err(()) => {
                        exit_tx.send_replace(Some(-1));
                        break;
                    }
                }
            }
        })
        .map_err(|_| TaskPtyError::Unavailable)?;
    Ok(())
}

fn terminate_child(child: &Arc<Mutex<Box<dyn Child + Send>>>) {
    if let Ok(mut process) = child.lock() {
        let _ = process.kill();
        let _ = process.wait();
    }
}

/// Stable PTY adapter failures; underlying OS diagnostics are intentionally not exposed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum TaskPtyError {
    /// A nil session identity cannot be registered.
    #[error("invalid task CLI session")]
    InvalidSessionId,
    /// The session identity already exists in this Runtime process.
    #[error("task CLI session already exists")]
    DuplicateSession,
    /// The session is not present in this Runtime process.
    #[error("task CLI session not found")]
    SessionNotFound,
    /// A session's output stream has already been claimed by its sink.
    #[error("task CLI output stream already has a consumer")]
    OutputAlreadySubscribed,
    /// The PTY dimensions are outside supported bounds.
    #[error("invalid terminal dimensions")]
    InvalidSize,
    /// The command or checkout path is not absolute.
    #[error("invalid prepared task CLI execution")]
    InvalidExecution,
    /// A terminal input chunk exceeded the configured bound.
    #[error("terminal input is too large")]
    InputTooLarge,
    /// The operating system could not create or configure a PTY.
    #[error("PTY is unavailable")]
    PtyUnavailable,
    /// The approved executable could not be started.
    #[error("task CLI process could not be started")]
    SpawnFailed,
    /// Writing terminal input failed.
    #[error("terminal input could not be written")]
    InputFailed,
    /// Resizing the PTY failed.
    #[error("terminal resize failed")]
    ResizeFailed,
    /// Terminating the child process failed.
    #[error("task CLI process could not be terminated")]
    TerminateFailed,
    /// Runtime state is unavailable or poisoned.
    #[error("task PTY runtime is unavailable")]
    Unavailable,
}
