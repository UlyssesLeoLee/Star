//! Task Card CLI execution context and the Local Runtime authorization gate.
/*
CREATE
  (file:File {name:"crates/domain-local-runtime/src/task_execution.rs",type:"file",language:"rust"}),
  (ctx:Class {name:"TaskExecutionContext",type:"class",language:"rust"}),
  (profile:Class {name:"TaskCliLaunchProfile",type:"class",language:"rust"}),
  (error:Class {name:"TaskExecutionError",type:"class",language:"rust"}),
  (resolve:Function {name:"TaskExecutionContext::resolve",type:"function",signature:"resolve(tenant_id, actor_id, project_id, worktree_id, work_item_id, runtime_id, authorization_version, expires_at, launch_profile_id, checkout_root, working_directory) -> Result<Self, TaskExecutionError>",visibility:"public",complexity:"complex"}),
  (validate:Function {name:"TaskExecutionContext::validate_for_start",type:"function",signature:"validate_for_start(&self, runtime_id, launch_profile_id, now) -> Result<&Path, TaskExecutionError>",visibility:"public",complexity:"moderate"}),
  (newProfile:Function {name:"TaskCliLaunchProfile::new",type:"function",signature:"new(id, executable, args) -> Result<Self, TaskExecutionError>",visibility:"public",complexity:"moderate"}),
  (spawn:Function {name:"spawn_authorized_cli",type:"function",signature:"spawn_authorized_cli(runtime, context, runtime_id, profile) -> Result<ProcessHandle, TaskExecutionError>",visibility:"public",complexity:"moderate"}),
  (resolvePath:Function {name:"resolve_checkout_directory",type:"function",signature:"resolve_checkout_directory(checkout_root, working_directory) -> Result<(PathBuf, PathBuf), TaskExecutionError>",visibility:"private",complexity:"moderate"}),
  (file)-[:CONTAINS]->(ctx),
  (file)-[:CONTAINS]->(profile),
  (file)-[:CONTAINS]->(error),
  (file)-[:CONTAINS]->(resolve),
  (file)-[:CONTAINS]->(validate),
  (file)-[:CONTAINS]->(newProfile),
  (file)-[:CONTAINS]->(spawn),
  (file)-[:CONTAINS]->(resolvePath),
  (ctx)-[:HAS_METHOD]->(resolve),
  (ctx)-[:HAS_METHOD]->(validate),
  (profile)-[:HAS_METHOD]->(newProfile),
  (resolve)-[:CALLS]->(resolvePath),
  (validate)-[:CALLS]->(resolvePath),
  (spawn)-[:CALLS]->(validate);
*/

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::process::{LocalRuntime, ProcessHandle, RuntimeError};

/// A short-lived, server-resolved authorization snapshot for one Task Card CLI run.
///
/// Construct this value only after the Group API has authorized the actor and resolved
/// the Worktree mount from trusted runtime state. Never deserialize it directly from a
/// browser request: IDs and paths are bindings, not authorization claims by themselves.
#[derive(Debug, Clone)]
pub struct TaskExecutionContext {
    tenant_id: Uuid,
    actor_id: Uuid,
    project_id: Uuid,
    worktree_id: Uuid,
    work_item_id: Uuid,
    runtime_id: Uuid,
    authorization_version: i64,
    expires_at: DateTime<Utc>,
    launch_profile_id: Uuid,
    checkout_root: PathBuf,
    working_directory: PathBuf,
}

impl TaskExecutionContext {
    /// Resolve a task execution context against the trusted mounted checkout.
    #[allow(clippy::too_many_arguments)]
    pub fn resolve(
        tenant_id: Uuid,
        actor_id: Uuid,
        project_id: Uuid,
        worktree_id: Uuid,
        work_item_id: Uuid,
        runtime_id: Uuid,
        authorization_version: i64,
        expires_at: DateTime<Utc>,
        launch_profile_id: Uuid,
        checkout_root: impl AsRef<Path>,
        working_directory: impl AsRef<Path>,
    ) -> Result<Self, TaskExecutionError> {
        if [
            tenant_id,
            actor_id,
            project_id,
            worktree_id,
            work_item_id,
            runtime_id,
            launch_profile_id,
        ]
        .into_iter()
        .any(|id| id.is_nil())
            || authorization_version <= 0
        {
            return Err(TaskExecutionError::InvalidIdentity);
        }
        if expires_at <= Utc::now() {
            return Err(TaskExecutionError::AuthorizationExpired);
        }

        let (checkout_root, working_directory) =
            resolve_checkout_directory(checkout_root.as_ref(), working_directory.as_ref())?;

        Ok(Self {
            tenant_id,
            actor_id,
            project_id,
            worktree_id,
            work_item_id,
            runtime_id,
            authorization_version,
            expires_at,
            launch_profile_id,
            checkout_root,
            working_directory,
        })
    }

    /// Re-check the expiry, runtime binding and checkout path immediately before spawn.
    pub fn validate_for_start(
        &self,
        runtime_id: Uuid,
        launch_profile_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<&Path, TaskExecutionError> {
        if now >= self.expires_at {
            return Err(TaskExecutionError::AuthorizationExpired);
        }
        if runtime_id != self.runtime_id {
            return Err(TaskExecutionError::RuntimeMismatch);
        }
        if launch_profile_id != self.launch_profile_id {
            return Err(TaskExecutionError::ProfileMismatch);
        }
        if self.authorization_version <= 0
            || [
                self.tenant_id,
                self.actor_id,
                self.project_id,
                self.worktree_id,
                self.work_item_id,
            ]
            .into_iter()
            .any(|id| id.is_nil())
        {
            return Err(TaskExecutionError::InvalidIdentity);
        }

        let (checkout_root, working_directory) =
            resolve_checkout_directory(&self.checkout_root, &self.working_directory)?;
        if checkout_root != self.checkout_root || working_directory != self.working_directory {
            return Err(TaskExecutionError::CheckoutChanged);
        }
        Ok(&self.working_directory)
    }
}

/// An administrator-approved executable and argument profile resolved by the server.
///
/// Arguments are profile-owned. Client-provided command strings, arguments and environment
/// variables must never be copied into this value.
#[derive(Debug, Clone)]
pub struct TaskCliLaunchProfile {
    id: Uuid,
    executable: PathBuf,
    args: Vec<String>,
}

impl TaskCliLaunchProfile {
    /// Create a profile from trusted server configuration and an installed executable.
    pub fn new(
        id: Uuid,
        executable: impl AsRef<Path>,
        args: Vec<String>,
    ) -> Result<Self, TaskExecutionError> {
        if id.is_nil() {
            return Err(TaskExecutionError::InvalidIdentity);
        }
        let executable = fs::canonicalize(executable)
            .map_err(|_| TaskExecutionError::InvalidProfileExecutable)?;
        if !executable.is_file() {
            return Err(TaskExecutionError::InvalidProfileExecutable);
        }
        Ok(Self {
            id,
            executable,
            args,
        })
    }
}

/// Validate the server-resolved scope and start the approved executable in its checkout.
pub async fn spawn_authorized_cli<R: LocalRuntime>(
    runtime: &R,
    context: &TaskExecutionContext,
    runtime_id: Uuid,
    profile: &TaskCliLaunchProfile,
) -> Result<ProcessHandle, TaskExecutionError> {
    let working_directory = context.validate_for_start(runtime_id, profile.id, Utc::now())?;
    let mut handle = runtime
        .spawn_cli(
            profile
                .executable
                .to_str()
                .ok_or(TaskExecutionError::InvalidProfileExecutable)?,
            &profile.args,
            &HashMap::new(),
            working_directory
                .to_str()
                .ok_or(TaskExecutionError::CheckoutUnavailable)?,
        )
        .await?;
    handle.worktree_id = context.worktree_id;
    Ok(handle)
}

fn resolve_checkout_directory(
    checkout_root: &Path,
    working_directory: &Path,
) -> Result<(PathBuf, PathBuf), TaskExecutionError> {
    let root =
        fs::canonicalize(checkout_root).map_err(|_| TaskExecutionError::CheckoutUnavailable)?;
    if !root.is_dir() {
        return Err(TaskExecutionError::CheckoutUnavailable);
    }
    let requested = if working_directory.is_absolute() {
        working_directory.to_path_buf()
    } else {
        root.join(working_directory)
    };
    let working =
        fs::canonicalize(requested).map_err(|_| TaskExecutionError::CheckoutUnavailable)?;
    if !working.is_dir() {
        return Err(TaskExecutionError::CheckoutUnavailable);
    }
    if !working.starts_with(&root) {
        return Err(TaskExecutionError::PathOutsideCheckout);
    }
    Ok((root, working))
}

/// Validation failures for Task Card initiated Local Runtime CLI execution.
#[derive(Debug, Error)]
pub enum TaskExecutionError {
    /// One or more required authorization bindings are absent or invalid.
    #[error("task execution identity is invalid")]
    InvalidIdentity,
    /// The server-resolved authorization snapshot has expired.
    #[error("task execution authorization has expired")]
    AuthorizationExpired,
    /// The requested Local Runtime does not match the authorized runtime.
    #[error("runtime does not match the authorized task context")]
    RuntimeMismatch,
    /// The approved launch profile does not match the authorized task context.
    #[error("launch profile does not match the authorized task context")]
    ProfileMismatch,
    /// The selected working directory resolves outside the authorized checkout.
    #[error("working directory is outside the authorized checkout")]
    PathOutsideCheckout,
    /// The checkout mount is absent, inaccessible, or no longer a directory.
    #[error("authorized checkout is unavailable")]
    CheckoutUnavailable,
    /// The checkout or working-directory path changed after authorization.
    #[error("authorized checkout changed before process start")]
    CheckoutChanged,
    /// The server-approved executable is absent or cannot be resolved.
    #[error("approved CLI executable is unavailable")]
    InvalidProfileExecutable,
    /// The underlying runtime failed to start the process.
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}
