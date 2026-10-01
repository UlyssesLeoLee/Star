//! CYPHER STRUCTURAL MANIFEST
//! CREATE
//!   (f:File {name:"cli_sessions.rs",type:"file",language:"rust"}),(m:Module {name:"cli_sessions",type:"module",language:"rust"}),
//!   (p:Interface {name:"TaskCliSessionProvisioner",type:"interface",language:"rust"}),(e:Enum {name:"TaskCliSessionProvisionError",type:"enum",language:"rust"}),(c:Class {name:"TaskCliSessionStartCommand",type:"class",language:"rust"}),(a:Class {name:"TaskCliSessionAccessCommand",type:"class",language:"rust"}),(r:Class {name:"TaskCliSessionReceipt",type:"class",language:"rust"}),(ss:Class {name:"TaskCliSessionStatus",type:"class",language:"rust"}),(state:Enum {name:"TaskCliSessionState",type:"enum",language:"rust"}),(b:Class {name:"StartTaskCliSessionBody",type:"class",language:"rust"}),(w:Class {name:"WorktreeTaskCliScope",type:"class",language:"rust"}),(l:Class {name:"TaskCliLifecycle",type:"class",language:"rust"}),(req:Variable {name:"request_fingerprint",type:"variable",language:"rust"}),
//!   (rt:Function {name:"router",type:"function",language:"rust"}),(st:Function {name:"start_task_cli_session",type:"function",language:"rust"}),(list:Function {name:"list_task_cli_sessions",type:"function",language:"rust"}),(get:Function {name:"get_task_cli_session_status",type:"function",language:"rust"}),(cancel:Function {name:"cancel_task_cli_session",type:"function",language:"rust"}),(attach:Function {name:"reattach_task_cli_session",type:"function",language:"rust"}),(authorize:Function {name:"authorize_task_cli_session_access",type:"function",language:"rust"}),(authorize_parent:Function {name:"authorize_task_cli_session_parent",type:"function",language:"rust"}),(valid:Function {name:"validate_receipt",type:"function",language:"rust"}),(no_store:Function {name:"no_store",type:"function",language:"rust"}),(val:Function {name:"validate_start_body",type:"function",language:"rust"}),(limit:Function {name:"validate_session_list_limit",type:"function",language:"rust"}),(valid_list:Function {name:"session_statuses_are_valid",type:"function",language:"rust"}),(fp:Function {name:"request_fingerprint",type:"function",language:"rust"}),(map:Function {name:"map_provision_error",type:"function",language:"rust"}),(sp:Function {name:"TaskCliSessionProvisioner::start_task_cli_session",type:"function",language:"rust"}),(list_method:Function {name:"TaskCliSessionProvisioner::list_task_cli_sessions",type:"function",language:"rust"}),(status:Function {name:"TaskCliSessionProvisioner::task_cli_session_status",type:"function",language:"rust"}),(stop:Function {name:"TaskCliSessionProvisioner::cancel_task_cli_session",type:"function",language:"rust"}),(ticket:Function {name:"TaskCliSessionProvisioner::reattach_task_cli_session",type:"function",language:"rust"}),(va:Function {name:"validate_actor",type:"function",language:"rust"}),(rs:Function {name:"require_scope",type:"function",language:"rust"}),(ik:Function {name:"idempotency_key",type:"function",language:"rust"}),(tenant:Function {name:"set_tenant",type:"function",language:"rust"}),(ab:Function {name:"active_binding",type:"function",language:"rust"}),(writer:Function {name:"require_task_writer",type:"function",language:"rust"}),(bad:Function {name:"GroupApiError::bad_request",type:"function",language:"rust"}),(internal:Function {name:"GroupApiError::internal",type:"function",language:"rust"}),(not_found:Function {name:"GroupApiError::not_found",type:"function",language:"rust"}),(conflict:Function {name:"GroupApiError::conflict",type:"function",language:"rust"}),(unavailable:Function {name:"GroupApiError::service_unavailable",type:"function",language:"rust"}),(invalid:Function {name:"GroupApiError::invalid_request",type:"function",language:"rust"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(p),(m)-[:CONTAINS]->(e),(m)-[:CONTAINS]->(c),(m)-[:CONTAINS]->(a),(m)-[:CONTAINS]->(r),(m)-[:CONTAINS]->(ss),(m)-[:CONTAINS]->(state),(m)-[:CONTAINS]->(b),(m)-[:CONTAINS]->(w),(m)-[:CONTAINS]->(l),(m)-[:CONTAINS]->(rt),(m)-[:CONTAINS]->(st),(m)-[:CONTAINS]->(get),(m)-[:CONTAINS]->(cancel),(m)-[:CONTAINS]->(attach),(m)-[:CONTAINS]->(authorize),(m)-[:CONTAINS]->(valid),(m)-[:CONTAINS]->(no_store),(m)-[:CONTAINS]->(val),(m)-[:CONTAINS]->(fp),(m)-[:CONTAINS]->(map),(p)-[:HAS_METHOD]->(sp),(p)-[:HAS_METHOD]->(status),(p)-[:HAS_METHOD]->(stop),(p)-[:HAS_METHOD]->(ticket),
//!   (m)-[:CONTAINS]->(list),(m)-[:CONTAINS]->(limit),(m)-[:CONTAINS]->(valid_list),(m)-[:CONTAINS]->(authorize_parent),(p)-[:HAS_METHOD]->(list_method),(list)-[:CALLS]->(authorize_parent),(list)-[:CALLS]->(list_method),(list)-[:CALLS]->(limit),(list)-[:CALLS]->(valid_list),(list)-[:CALLS]->(no_store),
//!   (rt)-[:CALLS]->(st),(rt)-[:CALLS]->(list),(rt)-[:CALLS]->(get),(rt)-[:CALLS]->(cancel),(rt)-[:CALLS]->(attach),(st)-[:CALLS]->(val),(st)-[:CALLS]->(fp),(st)-[:CALLS]->(map),(st)-[:CALLS]->(va),(st)-[:CALLS]->(rs),(st)-[:CALLS]->(ik),(st)-[:CALLS]->(tenant),(st)-[:CALLS]->(ab),(st)-[:CALLS]->(writer),(st)-[:CALLS]->(sp),(st)-[:CALLS]->(bad),(st)-[:CALLS]->(internal),(st)-[:CALLS]->(not_found),(st)-[:CALLS]->(conflict),(st)-[:CALLS]->(unavailable),(st)-[:CALLS]->(invalid),(st)-[:USES]->(req),(list)-[:CALLS]->(authorize_parent),(list)-[:CALLS]->(list_method),(list)-[:CALLS]->(no_store),(get)-[:CALLS]->(authorize),(get)-[:CALLS]->(status),(cancel)-[:CALLS]->(authorize),(cancel)-[:CALLS]->(stop),(attach)-[:CALLS]->(authorize),(attach)-[:CALLS]->(ticket),(attach)-[:CALLS]->(valid),(attach)-[:CALLS]->(no_store),(get)-[:CALLS]->(no_store),(cancel)-[:CALLS]->(no_store),(authorize)-[:CALLS]->(authorize_parent),(val)-[:CALLS]->(invalid),(fp)-[:CALLS]->(internal),(map)-[:CALLS]->(unavailable),(map)-[:CALLS]->(conflict),(map)-[:CALLS]->(not_found),(map)-[:CALLS]->(internal);
//! MATCH (m:Module {name:"cli_sessions",type:"module"}),(st:Function {name:"start_task_cli_session",type:"function"}),(s:Function {name:"GroupApiState::with_task_cli_session_provisioner",type:"function"}),(p:Interface {name:"TaskCliSessionProvisioner",type:"interface"}),(g:Class {name:"GroupApiState",type:"class"});
//! CREATE (m)-[:USES]->(g),(st)-[:USES]->(p),(g)-[:USES]->(p);
//! CYPHER STRUCTURE MANIFEST ADDENDUM
//! MATCH (m:Module {name:"cli_sessions",type:"module"}),(list:Function {name:"list_task_cli_sessions",type:"function"}),(limit:Function {name:"validate_session_list_limit",type:"function"}),(valid:Function {name:"session_statuses_are_valid",type:"function"});
//! CREATE (list_command:Class {name:"TaskCliSessionListCommand",type:"class",language:"rust"}),(list_query:Class {name:"TaskCliSessionListQuery",type:"class",language:"rust"}),(parent:Class {name:"TaskCliSessionParent",type:"class",language:"rust"});
//! CREATE (m)-[:CONTAINS]->(list_command),(m)-[:CONTAINS]->(list_query),(m)-[:CONTAINS]->(parent),(list)-[:CALLS]->(limit),(list)-[:CALLS]->(valid);
//! CYPHER STRUCTURE MANIFEST ADDENDUM
//! MATCH (m:Module {name:"cli_sessions",type:"module"}),(st:Function {name:"start_task_cli_session",type:"function"}),(sp:Interface {name:"TaskCliSessionProvisioner",type:"interface"});
//! CREATE (idem:Class {name:"TaskRunIdempotency",type:"class",language:"rust"}),(lookup:Function {name:"lookup_cli_task_run",type:"function",language:"rust"}),(record:Function {name:"record_cli_task_run",type:"function",language:"rust"}),(append:Function {name:"append_cli_task_run_event",type:"function",language:"rust"}),(actor_scope:Function {name:"set_actor_scope",type:"function",language:"rust"}),(category:Function {name:"provision_error_category",type:"function",language:"rust"}),(task_run_id:Variable {name:"task_run_id",type:"variable",language:"rust"});
//! CREATE (m)-[:CONTAINS]->(idem),(m)-[:CONTAINS]->(lookup),(m)-[:CONTAINS]->(record),(m)-[:CONTAINS]->(append),(m)-[:CONTAINS]->(actor_scope),(m)-[:CONTAINS]->(category),(st)-[:CALLS]->(lookup),(st)-[:CALLS]->(record),(st)-[:CALLS]->(append),(st)-[:CALLS]->(actor_scope),(st)-[:CALLS]->(category),(st)-[:USES]->(task_run_id),(sp)-[:USES]->(task_run_id),(record)-[:CALLS]->(lookup),(lookup)-[:USES]->(idem);
//! CYPHER STRUCTURE MANIFEST ADDENDUM
//! MATCH (m:Module {name:"cli_sessions",type:"module"}),(st:Function {name:"start_task_cli_session",type:"function"}),(sp:Interface {name:"TaskCliSessionProvisioner",type:"interface"});
//! CREATE (readiness:Class {name:"TaskRunAdmissionReadiness",type:"class",language:"rust"}),(readinessCommand:Class {name:"TaskRunAdmissionReadinessCommand",type:"class",language:"rust"}),(loadContext:Function {name:"load_task_cli_start_context",type:"function",language:"rust"}),(freshness:Function {name:"run_admission_readiness_is_fresh",type:"function",language:"rust"}),(appendAdmission:Function {name:"append_run_admission_hook_events",type:"function",language:"rust"});
//! CREATE (m)-[:CONTAINS]->(readiness),(m)-[:CONTAINS]->(readinessCommand),(m)-[:CONTAINS]->(loadContext),(m)-[:CONTAINS]->(freshness),(m)-[:CONTAINS]->(appendAdmission),(st)-[:CALLS]->(loadContext),(st)-[:CALLS]->(freshness),(st)-[:CALLS]->(appendAdmission),(sp)-[:HAS_METHOD]->(readiness);
//! CYPHER STRUCTURE MANIFEST ADDENDUM
//! MATCH (m:Module {name:"cli_sessions",type:"module"}),(st:Function {name:"start_task_cli_session",type:"function"}),(p:Interface {name:"TaskCliSessionProvisioner",type:"interface"}),(body:Class {name:"StartTaskCliSessionBody",type:"class"}),(readiness:Class {name:"TaskRunAdmissionReadinessCommand",type:"class"}),(sessionStart:Class {name:"TaskCliSessionStartCommand",type:"class"}),(fp:Function {name:"request_fingerprint",type:"function"}),(validate:Function {name:"validate_start_body",type:"function"});
//! CREATE (capability:Function {name:"TaskCliSessionProvisioner::supports_profile_bound_run_admission",type:"function",language:"rust"}),(profileId:Variable {name:"execution_profile_id",type:"variable",language:"rust"});
//! CREATE (catalogCapability:Function {name:"TaskCliSessionProvisioner::supports_current_execution_catalogs",type:"function",language:"rust"}),(capability)-[:REQUIRES]->(catalogCapability);
//! CREATE (m)-[:CONTAINS]->(capability),(p)-[:HAS_METHOD]->(capability),(st)-[:CALLS]->(capability),(body)-[:USES]->(profileId),(readiness)-[:USES]->(profileId),(sessionStart)-[:USES]->(profileId),(fp)-[:USES]->(profileId),(validate)-[:USES]->(profileId);

//! CYPHER STRUCTURE MANIFEST ADDENDUM
//! MATCH (m:Module {name:"cli_sessions",type:"module"}),(st:Function {name:"start_task_cli_session",type:"function"}),(readiness:Class {name:"TaskRunAdmissionReadinessCommand",type:"class"}),(load:Function {name:"execution_catalogs::load_current_execution_admission_snapshot",type:"function"}),(recheck:Function {name:"execution_catalogs::recheck_current_execution_admission_snapshot",type:"function"}),(snapshot:Class {name:"CurrentExecutionAdmissionSnapshot",type:"class"});
//! CREATE (readiness)-[:USES]->(snapshot),(st)-[:CALLS]->(load),(st)-[:CALLS]->(recheck),(m)-[:CONTAINS]->(snapshot);
//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (st:Function {name:"start_task_cli_session",type:"function"}),(record:Function {name:"record_cli_task_run",type:"function"}),(snapshot:Class {name:"CurrentExecutionAdmissionSnapshot",type:"class"}),(readiness:Class {name:"TaskRunAdmissionReadiness",type:"class"}),(resources:Module {name:"execution_resources",type:"module"});
//! CREATE (reserve:Function {name:"execution_resources::reserve_project_execution_resources",type:"function",language:"rust"});
//! CREATE (resources)-[:CONTAINS]->(reserve),(record)-[:CALLS]->(reserve),(reserve)-[:USES]->(snapshot),(reserve)-[:USES]->(readiness),(st)-[:USES]->(snapshot),(st)-[:USES]->(readiness);
//! CYPHER STRUCTURE MANIFEST ADDENDUM
//! MATCH (f:File {name:"cli_sessions.rs",type:"file"}),(m:Module {name:"cli_sessions",type:"module"}),(st:Function {name:"start_task_cli_session",type:"function"}),(record:Function {name:"record_cli_task_run",type:"function"}),(append:Function {name:"append_run_admission_hook_events",type:"function"}),(readiness:Class {name:"TaskRunAdmissionReadiness",type:"class"}),(readiness_command:Class {name:"TaskRunAdmissionReadinessCommand",type:"class"}),(session_start:Class {name:"TaskCliSessionStartCommand",type:"class"}),(snapshot:Class {name:"CurrentExecutionAdmissionSnapshot",type:"class"});
//! CREATE (profile_identity:Class {name:"TaskRunProfileRevisionIdentity",type:"class",language:"rust"}),(catalog_identity:Class {name:"TaskRunCatalogRevisionIdentity",type:"class",language:"rust"}),(fence_binding:Class {name:"TaskRunSpawnFenceBinding",type:"class",language:"rust"}),(spawn_fence:Class {name:"TaskRunSpawnFence",type:"class",language:"rust"}),(binding_digest:Function {name:"TaskRunSpawnFenceBinding::binding_digest",type:"function",language:"rust",visibility:"pub"}),(catalog_from:Function {name:"TaskRunCatalogRevisionIdentity::from",type:"function",language:"rust",visibility:"private"}),(build_binding:Function {name:"expected_task_run_spawn_fence_binding",type:"function",language:"rust",visibility:"private"}),(fence_matches:Function {name:"spawn_fence_matches_readiness_command",type:"function",language:"rust",visibility:"private"}),(digest_check:Function {name:"is_lower_hex_sha256_digest",type:"function",language:"rust",visibility:"private"}),(digest_encode:Function {name:"digest_to_lower_hex",type:"function",language:"rust",visibility:"private"}),(fence_digest_value:Variable {name:"spawn_fence_binding_digest",type:"variable",language:"rust"}),(f)-[:CONTAINS]->(profile_identity),(f)-[:CONTAINS]->(catalog_identity),(f)-[:CONTAINS]->(fence_binding),(f)-[:CONTAINS]->(spawn_fence),(f)-[:CONTAINS]->(binding_digest),(f)-[:CONTAINS]->(catalog_from),(f)-[:CONTAINS]->(build_binding),(f)-[:CONTAINS]->(fence_matches),(f)-[:CONTAINS]->(digest_check),(f)-[:CONTAINS]->(digest_encode),(f)-[:CONTAINS]->(fence_digest_value),(fence_binding)-[:HAS_METHOD]->(binding_digest),(catalog_identity)-[:HAS_METHOD]->(catalog_from),(readiness)-[:USES]->(spawn_fence),(session_start)-[:USES]->(spawn_fence),(st)-[:CALLS]->(fence_matches),(fence_matches)-[:CALLS]->(build_binding),(fence_matches)-[:CALLS]->(digest_check),(record)-[:USES]->(spawn_fence),(build_binding)-[:USES]->(readiness_command),(build_binding)-[:USES]->(snapshot),(record)-[:CALLS]->(digest_encode),(append)-[:CALLS]->(digest_encode),(record)-[:USES]->(fence_digest_value),(append)-[:USES]->(fence_digest_value),(digest_encode)-[:USES]->(fence_digest_value);

use async_trait::async_trait;
use axum::{
    extract::{Path, Query, State},
    http::header,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use domain_agent::execution_profile::{
    ExecutionCatalogRevisions, HookSetSnapshot, ResourceBudgetSnapshot,
};
use domain_hook::{
    evaluate as evaluate_hook, HookDecision, HookEventEnvelope, HookPhase, HookScope,
    RetentionLockState, EVENT_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{FromRow, Postgres, Row, Transaction};
use std::sync::Arc;
use uuid::Uuid;

use super::{
    active_binding, require_scope, set_tenant, validate_actor, AuthUser, AuthenticatedUser,
    GroupApiError, GroupApiState,
};

/// A REST-authorized request for provisioning a Task Card CLI session in Local Runtime.
/// The provisioner must resolve the Approved Launch Profile and bind the separately selected
/// Agent Execution Profile from trusted catalogs, then recheck live ACL, Worktree/runtime health
/// and policy immediately before grant issuance and spawn; persist
/// TaskRun intent/result audit against the supplied correlation ID. This command is not itself
/// an execution grant. For a new admitted Run, the provisioner must consume the supplied
/// single-use admission fence and verify its full Task/Worktree/Runtime/Profile/catalog-revision/
/// fingerprint scope and catalog-fence expiry before spawn; `None` is reserved for an already-
/// admitted idempotent replay.
#[derive(Clone)]
pub struct TaskCliSessionStartCommand {
    pub tenant_id: Uuid,
    pub actor_id: Uuid,
    pub project_id: Uuid,
    pub repository_id: Uuid,
    pub worktree_id: Uuid,
    pub work_item_id: Uuid,
    pub task_run_id: Uuid,
    pub runtime_id: Uuid,
    pub expected_lifecycle_version: i32,
    pub approved_launch_profile_id: Uuid,
    /// Agent Execution Profile selected separately from the executable launch policy.
    /// `None` is reserved for a replay of a legacy Run admitted before Profile identity binding.
    pub execution_profile_id: Option<Uuid>,
    pub correlation_id: Uuid,
    pub idempotency_key: String,
    pub request_fingerprint: [u8; 32],
    /// Opaque one-time Runtime fence plus its bounded server-verified identity binding; absent
    /// only for an already-admitted idempotent replay.
    pub spawn_fence: Option<TaskRunSpawnFence>,
}

/// Request to observe and fence Local Runtime before the database admission transaction.
#[derive(Clone, Debug)]
pub struct TaskRunAdmissionReadinessCommand {
    pub tenant_id: Uuid,
    pub actor_id: Uuid,
    pub project_id: Uuid,
    pub repository_id: Uuid,
    pub worktree_id: Uuid,
    pub work_item_id: Uuid,
    pub runtime_id: Uuid,
    pub expected_lifecycle_version: i32,
    pub approved_launch_profile_id: Uuid,
    /// Agent Execution Profile identity, independent from the Approved Launch Profile.
    pub execution_profile_id: Option<Uuid>,
    /// Authorized, reference-scoped Profile and current Provider/Skill/Grant facts.
    pub execution_snapshot: Arc<super::CurrentExecutionAdmissionSnapshot>,
    pub correlation_id: Uuid,
    pub request_fingerprint: [u8; 32],
}

/// Bounded Local Runtime facts and fence returned before native Run admission evaluation.
#[derive(Clone, Debug)]
pub struct TaskRunAdmissionReadiness {
    pub runtime_healthy: bool,
    pub observed_at: DateTime<Utc>,
    /// Runtime-resident, single-consume fence bound to both launch and execution profiles.
    pub spawn_fence: Option<TaskRunSpawnFence>,
}

/// A compact immutable profile identity carried across the Runtime boundary.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TaskRunProfileRevisionIdentity {
    pub profile_id: Uuid,
    pub version: u64,
    pub content_digest: String,
}

/// Only revision facts cross the Runtime boundary; catalog entries remain Arc-backed in REST.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct TaskRunCatalogRevisionIdentity {
    pub provider_catalog_revision: u64,
    pub skill_catalog_revision: u64,
    pub grant_set_id: Uuid,
    pub grant_set_version: u64,
}

impl From<ExecutionCatalogRevisions> for TaskRunCatalogRevisionIdentity {
    fn from(revisions: ExecutionCatalogRevisions) -> Self {
        Self {
            provider_catalog_revision: revisions.provider_catalog_revision,
            skill_catalog_revision: revisions.skill_catalog_revision,
            grant_set_id: revisions.grant_set_id,
            grant_set_version: revisions.grant_set_version,
        }
    }
}

/// Exact immutable facts a Runtime fence must bind before it may create a process.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TaskRunSpawnFenceBinding {
    pub tenant_id: Uuid,
    pub actor_id: Uuid,
    pub project_id: Uuid,
    pub repository_id: Uuid,
    pub worktree_id: Uuid,
    pub work_item_id: Uuid,
    pub runtime_id: Uuid,
    pub expected_lifecycle_version: i32,
    pub approved_launch_profile: TaskRunProfileRevisionIdentity,
    pub execution_profile: TaskRunProfileRevisionIdentity,
    pub catalog_revisions: TaskRunCatalogRevisionIdentity,
    pub hook_set: HookSetSnapshot,
    pub resource_budget: ResourceBudgetSnapshot,
    pub request_fingerprint: [u8; 32],
}

impl TaskRunSpawnFenceBinding {
    /// Hash a versioned canonical serialization so Runtime storage can bind the opaque token to
    /// this exact request and both immutable Profile revisions without copying catalog contents.
    pub fn binding_digest(&self) -> Result<[u8; 32], serde_json::Error> {
        let encoded = serde_json::to_vec(self)?;
        let mut hasher = Sha256::new();
        hasher.update(b"star.task_run_spawn_fence.v1\0");
        hasher.update(encoded);
        Ok(hasher.finalize().into())
    }
}

/// Opaque one-time Runtime token with a small auditable binding, never a process grant.
#[derive(Clone, Debug)]
pub struct TaskRunSpawnFence {
    pub fence_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub binding: TaskRunSpawnFenceBinding,
    pub binding_digest: [u8; 32],
}

/// Current Group authorization context supplied to Local Runtime for a session operation.
#[derive(Clone, Debug)]
pub struct TaskCliSessionAccessCommand {
    pub tenant_id: Uuid,
    pub actor_id: Uuid,
    pub project_id: Uuid,
    pub repository_id: Uuid,
    pub worktree_id: Uuid,
    pub work_item_id: Uuid,
    pub runtime_id: Uuid,
    pub session_id: Uuid,
    pub correlation_id: Uuid,
}

/// Authorized, bounded request for recent sessions attached to one canonical Task/Worktree.
#[derive(Clone, Debug)]
pub struct TaskCliSessionListCommand {
    pub tenant_id: Uuid,
    pub actor_id: Uuid,
    pub project_id: Uuid,
    pub repository_id: Uuid,
    pub worktree_id: Uuid,
    pub work_item_id: Uuid,
    pub runtime_id: Uuid,
    pub correlation_id: Uuid,
    pub limit: usize,
}

/// Successful response data. The attachment ticket is short-lived and returned only once over
/// authenticated HTTPS with `Cache-Control: no-store`; it must never be logged or persisted by UI.
#[derive(Serialize)]
pub struct TaskCliSessionReceipt {
    pub session_id: Uuid,
    pub attachment_ticket: String,
    pub attachment_ticket_expires_at: DateTime<Utc>,
}

/// Public lifecycle states for the Task Card session list; terminal output is never included.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskCliSessionState {
    Starting,
    Running,
    Disconnected,
    Cancelling,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
    Lost,
}

/// Redacted session projection returned to authorized Task Card clients.
#[derive(Clone, Debug, Serialize)]
pub struct TaskCliSessionStatus {
    pub session_id: Uuid,
    pub state: TaskCliSessionState,
    pub exit_code: Option<i32>,
    pub updated_at: DateTime<Utc>,
}

/// Provisioning failures intentionally expose only stable public error categories.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskCliSessionProvisionError {
    RuntimeUnavailable,
    SessionConflict,
    SessionNotFound,
    Internal,
}

/// Trusted bridge from the REST authorization boundary to Local Runtime.
/// Implementations must be idempotent by `(tenant, actor, task_run_id, idempotency_key)`, compare
/// the request fingerprint on retries, and return the same session for the same TaskRun. Return
/// only after a sandboxed session is actually running and a fresh single-use WebSocket attachment
/// ticket has been issued. Every lifecycle operation must compare the complete session binding
/// and recheck current ACL and Runtime health. Lifecycle operations are correlated for TaskRun
/// Audit, cancel is idempotent, and reattach must issue a new ticket rather than return a ticket
/// previously issued to the browser.
#[async_trait]
pub trait TaskCliSessionProvisioner: Send + Sync {
    /// Explicit opt-in is required before Run-admission policies can be published.
    fn supports_run_admission(&self) -> bool {
        false
    }

    /// Opt in only when current Provider, Skill, and Grant reads are authoritative, bounded,
    /// revision-fenced, and rechecked inside the final admission transaction.
    fn supports_current_execution_catalogs(&self) -> bool {
        false
    }

    /// Opt in only when the REST host installs current Profile/catalog resolution, the
    /// transactional Run snapshot writer, and Project-wide resource-quota reservations, and the
    /// Runtime consumes the exact dual-Profile fence once. Before process creation it must
    /// recheck current ACL/scope/lifecycle, both current Profile versions and digests, catalog
    /// revisions, HookSet, resource budget, and request binding. The capability remains false
    /// until reservation activation/release and failed-spawn reconciliation are also installed.
    fn supports_profile_bound_run_admission(&self) -> bool {
        false
    }

    /// Resolve the current Approved Launch Profile, establish a short-lived one-time fence, and
    /// report Runtime health without a database transaction. The fence must echo the exact
    /// request/snapshot binding and attest the launch Profile ID/version/digest it resolved.
    /// `start_task_cli_session` must atomically consume the Runtime-resident token once, recheck
    /// live ACL/scope/lifecycle, both Profile revisions, catalogs, HookSet and budget, then create
    /// the process only after all checks pass. Fence stores must be bounded and expire within the
    /// returned TTL; the fence is not itself an execution grant.
    async fn prepare_run_admission(
        &self,
        _command: TaskRunAdmissionReadinessCommand,
    ) -> Result<TaskRunAdmissionReadiness, TaskCliSessionProvisionError> {
        Err(TaskCliSessionProvisionError::RuntimeUnavailable)
    }

    async fn start_task_cli_session(
        &self,
        command: TaskCliSessionStartCommand,
    ) -> Result<TaskCliSessionReceipt, TaskCliSessionProvisionError>;

    /// Return recent redacted session metadata for this canonical Task binding, without tickets
    /// or terminal output. The implementation must recheck the complete current binding.
    async fn list_task_cli_sessions(
        &self,
        command: TaskCliSessionListCommand,
    ) -> Result<Vec<TaskCliSessionStatus>, TaskCliSessionProvisionError>;

    async fn task_cli_session_status(
        &self,
        access: TaskCliSessionAccessCommand,
    ) -> Result<TaskCliSessionStatus, TaskCliSessionProvisionError>;

    async fn cancel_task_cli_session(
        &self,
        access: TaskCliSessionAccessCommand,
    ) -> Result<TaskCliSessionStatus, TaskCliSessionProvisionError>;

    async fn reattach_task_cli_session(
        &self,
        access: TaskCliSessionAccessCommand,
    ) -> Result<TaskCliSessionReceipt, TaskCliSessionProvisionError>;
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct StartTaskCliSessionBody {
    expected_lifecycle_version: i32,
    approved_launch_profile_id: Uuid,
    correlation_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    execution_profile_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TaskCliSessionListQuery {
    limit: Option<usize>,
}

#[derive(Clone, Debug)]
struct TaskCliSessionParent {
    tenant_id: Uuid,
    actor_id: Uuid,
    project_id: Uuid,
    repository_id: Uuid,
    worktree_id: Uuid,
    work_item_id: Uuid,
    runtime_id: Uuid,
    correlation_id: Uuid,
}

#[derive(Clone, Debug, FromRow)]
struct WorktreeTaskCliScope {
    project_id: Uuid,
    repository_id: Uuid,
    runtime_id: Option<Uuid>,
    branch: String,
    archived: bool,
}

#[derive(Debug, FromRow)]
struct TaskCliLifecycle {
    status: String,
    review_state: String,
    active_worktree_id: Option<Uuid>,
    claimed_by: Option<Uuid>,
    version: i32,
}

struct TaskCliStartContext {
    worktree: WorktreeTaskCliScope,
    runtime_id: Uuid,
    lifecycle: TaskCliLifecycle,
    request_fingerprint: [u8; 32],
    existing_run_id: Option<Uuid>,
}

#[derive(Debug, FromRow)]
struct TaskRunIdempotency {
    request_hash: Vec<u8>,
    run_id: Uuid,
}

pub(super) fn router() -> Router<GroupApiState> {
    Router::new().route(
        "/api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/cli-sessions",
        get(list_task_cli_sessions).post(start_task_cli_session),
    ).route(
        "/api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/cli-sessions/{session_id}",
        get(get_task_cli_session_status).delete(cancel_task_cli_session),
    ).route(
        "/api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/cli-sessions/{session_id}/attachment-tickets",
        post(reattach_task_cli_session),
    )
}

async fn list_task_cli_sessions(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, work_item_id)): Path<(String, String)>,
    Query(query): Query<TaskCliSessionListQuery>,
    headers: axum::http::HeaderMap,
) -> Result<Response, GroupApiError> {
    let correlation_id = required_correlation_id(&headers)?;
    let limit = validate_session_list_limit(query.limit)?;
    let parent = authorize_task_cli_session_parent(
        &state,
        &actor,
        "agent_session:read",
        &worktree_id,
        &work_item_id,
        correlation_id,
    )
    .await?;
    let command = TaskCliSessionListCommand {
        tenant_id: parent.tenant_id,
        actor_id: parent.actor_id,
        project_id: parent.project_id,
        repository_id: parent.repository_id,
        worktree_id: parent.worktree_id,
        work_item_id: parent.work_item_id,
        runtime_id: parent.runtime_id,
        correlation_id: parent.correlation_id,
        limit,
    };
    let provisioner = state
        .task_cli_session_provisioner
        .ok_or_else(GroupApiError::service_unavailable)?;
    let sessions = provisioner
        .list_task_cli_sessions(command)
        .await
        .map_err(map_provision_error)?;
    if sessions.len() > limit || !session_statuses_are_valid(&sessions) {
        return Err(GroupApiError::internal());
    }
    Ok(no_store(Json(json!({ "sessions": sessions }))))
}

async fn start_task_cli_session(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, work_item_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    Json(body): Json<StartTaskCliSessionBody>,
) -> Result<Response, GroupApiError> {
    validate_actor(&actor)?;
    require_scope(&actor, "agent_session:start")?;
    validate_start_body(&body)?;

    let worktree_id = Uuid::parse_str(&worktree_id).map_err(|_| GroupApiError::bad_request())?;
    let work_item_id = Uuid::parse_str(&work_item_id).map_err(|_| GroupApiError::bad_request())?;
    let idempotency_key = super::work_items::idempotency_key(&headers)?;
    let provisioner = state
        .task_cli_session_provisioner
        .clone()
        .ok_or_else(GroupApiError::service_unavailable)?;

    let mut preflight_tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
        .execute(&mut *preflight_tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
    let preflight = load_task_cli_start_context(
        &mut preflight_tx,
        &actor,
        worktree_id,
        work_item_id,
        &body,
        &idempotency_key,
        false,
    )
    .await?;
    let execution_snapshot = if preflight.existing_run_id.is_some() {
        None
    } else {
        if !provisioner.supports_run_admission() {
            return Err(GroupApiError::feature_unavailable(
                "run_admission_producer_unavailable",
            ));
        }
        if !provisioner.supports_current_execution_catalogs() {
            return Err(GroupApiError::feature_unavailable(
                "execution_catalog_provider_unavailable",
            ));
        }
        if !provisioner.supports_profile_bound_run_admission() {
            return Err(GroupApiError::feature_unavailable(
                "execution_profile_admission_unavailable",
            ));
        }
        let execution_profile_id = body.execution_profile_id.ok_or_else(|| {
            GroupApiError::feature_unavailable("execution_profile_selection_required")
        })?;
        if execution_profile_id.is_nil() {
            return Err(GroupApiError::bad_request());
        }
        Some(
            super::execution_catalogs::load_current_execution_admission_snapshot(
                &mut preflight_tx,
                actor.tenant_id,
                preflight.worktree.project_id,
                worktree_id,
                execution_profile_id,
            )
            .await?,
        )
    };
    preflight_tx
        .commit()
        .await
        .map_err(|_| GroupApiError::internal())?;

    let (task_run_id, spawn_fence, worktree, runtime_id, request_fingerprint) =
        if let Some(existing_run_id) = preflight.existing_run_id {
            (
                existing_run_id,
                None,
                preflight.worktree,
                preflight.runtime_id,
                preflight.request_fingerprint,
            )
        } else {
            let execution_profile_id = body.execution_profile_id.ok_or_else(|| {
                GroupApiError::feature_unavailable("execution_profile_selection_required")
            })?;
            if execution_profile_id.is_nil() {
                return Err(GroupApiError::bad_request());
            }
            let execution_snapshot = execution_snapshot
                .as_ref()
                .cloned()
                .ok_or_else(GroupApiError::internal)?;
            let readiness_command = TaskRunAdmissionReadinessCommand {
                tenant_id: actor.tenant_id,
                actor_id: actor.user_id,
                project_id: preflight.worktree.project_id,
                repository_id: preflight.worktree.repository_id,
                worktree_id,
                work_item_id,
                runtime_id: preflight.runtime_id,
                expected_lifecycle_version: body.expected_lifecycle_version,
                approved_launch_profile_id: body.approved_launch_profile_id,
                execution_profile_id: Some(execution_profile_id),
                execution_snapshot: Arc::clone(&execution_snapshot),
                correlation_id: body.correlation_id,
                request_fingerprint: preflight.request_fingerprint,
            };
            let readiness = tokio::time::timeout(
                std::time::Duration::from_secs(2),
                provisioner.prepare_run_admission(readiness_command.clone()),
            )
            .await;
            let readiness = match readiness {
                Ok(Ok(readiness)) => readiness,
                Ok(Err(_)) | Err(_) => {
                    return Err(GroupApiError::feature_unavailable(
                        "run_admission_readiness_unavailable",
                    ));
                }
            };
            if !run_admission_readiness_is_fresh(&readiness)
                || !spawn_fence_matches_readiness_command(&readiness, &readiness_command)
            {
                return Err(GroupApiError::feature_unavailable(
                    "run_admission_fence_binding_invalid",
                ));
            }
            let spawn_fence = readiness
                .spawn_fence
                .as_ref()
                .ok_or_else(|| {
                    GroupApiError::feature_unavailable("run_admission_fence_unavailable")
                })?
                .clone();

            let mut tx = state
                .resolver
                .pool
                .begin()
                .await
                .map_err(|_| GroupApiError::internal())?;
            sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
                .execute(&mut *tx)
                .await
                .map_err(|_| GroupApiError::internal())?;
            let current = load_task_cli_start_context(
                &mut tx,
                &actor,
                worktree_id,
                work_item_id,
                &body,
                &idempotency_key,
                true,
            )
            .await?;
            if let Some(existing_run_id) = current.existing_run_id {
                tx.commit().await.map_err(|_| GroupApiError::internal())?;
                (
                    existing_run_id,
                    None,
                    current.worktree,
                    current.runtime_id,
                    current.request_fingerprint,
                )
            } else {
                if current.worktree.project_id != preflight.worktree.project_id
                    || current.worktree.repository_id != preflight.worktree.repository_id
                    || current.worktree.branch != preflight.worktree.branch
                    || current.runtime_id != preflight.runtime_id
                    || current.request_fingerprint != preflight.request_fingerprint
                {
                    return Err(GroupApiError::conflict("run_admission_context_changed"));
                }

                let Some((policy, current_hook_set)) =
                    super::hook_policies::load_verified_effective_run_snapshot(
                        &mut tx,
                        actor.tenant_id,
                        current.worktree.project_id,
                        worktree_id,
                    )
                    .await?
                else {
                    return Err(GroupApiError::feature_unavailable(
                        "execution_profile_hook_set_unavailable",
                    ));
                };
                super::execution_catalogs::recheck_current_execution_admission_snapshot(
                    &mut tx,
                    &execution_snapshot,
                    &current_hook_set,
                )
                .await?;
                let policy = Some(policy);
                let readiness_is_fresh = run_admission_readiness_is_fresh(&readiness);
                let event = HookEventEnvelope {
                    event_id: Uuid::new_v4().into_bytes(),
                    event_schema_version: EVENT_SCHEMA_VERSION,
                    phase: HookPhase::BeforeRunAdmission,
                    scope: HookScope {
                        tenant_id: actor.tenant_id.into_bytes(),
                        project_id: current.worktree.project_id.into_bytes(),
                        worktree_id: worktree_id.into_bytes(),
                        actor_id: actor.user_id.into_bytes(),
                        correlation_id: body.correlation_id.into_bytes(),
                    },
                    actor_authorized: true,
                    lifecycle_version_matches: current.lifecycle.version
                        == body.expected_lifecycle_version,
                    runtime_healthy: Some(readiness.runtime_healthy && readiness_is_fresh),
                    retention_lock: RetentionLockState::Unknown,
                    active_run_count: 0,
                    active_agent_lease_count: 0,
                    file_claim_count: 0,
                    owned_process_count: 0,
                };
                let evaluation_started = std::time::Instant::now();
                let evaluation = evaluate_hook(&event, policy.as_ref());
                let duration_ms =
                    i64::try_from(evaluation_started.elapsed().as_millis()).unwrap_or(i64::MAX);

                if evaluation.decision != HookDecision::Allow {
                    append_run_admission_hook_events(
                        &mut tx,
                        actor.tenant_id,
                        actor.user_id,
                        current.worktree.project_id,
                        worktree_id,
                        work_item_id,
                        None,
                        &event,
                        &evaluation,
                        policy.as_ref(),
                        duration_ms,
                        &readiness,
                    )
                    .await?;
                    tx.commit().await.map_err(|_| GroupApiError::internal())?;
                    return Err(GroupApiError::conflict("hook_admission_denied"));
                }

                if !run_admission_readiness_is_fresh(&readiness)
                    || !spawn_fence_matches_readiness_command(&readiness, &readiness_command)
                {
                    return Err(GroupApiError::feature_unavailable(
                        "run_admission_fence_binding_invalid",
                    ));
                }
                let hook_snapshot =
                    serde_json::to_value(policy.as_ref().ok_or_else(GroupApiError::internal)?)
                        .map_err(|_| GroupApiError::internal())?;
                let task_run_id = record_cli_task_run(
                    &mut tx,
                    actor.tenant_id,
                    actor.user_id,
                    current.worktree.project_id,
                    current.worktree.repository_id,
                    worktree_id,
                    current.runtime_id,
                    &current.worktree.branch,
                    work_item_id,
                    body.correlation_id,
                    &idempotency_key,
                    &current.request_fingerprint,
                    &hook_snapshot,
                    &execution_snapshot,
                    body.approved_launch_profile_id,
                    &spawn_fence,
                )
                .await?;
                append_run_admission_hook_events(
                    &mut tx,
                    actor.tenant_id,
                    actor.user_id,
                    current.worktree.project_id,
                    worktree_id,
                    work_item_id,
                    Some(task_run_id),
                    &event,
                    &evaluation,
                    policy.as_ref(),
                    duration_ms,
                    &readiness,
                )
                .await?;
                if !run_admission_readiness_is_fresh(&readiness)
                    || !spawn_fence_matches_readiness_command(&readiness, &readiness_command)
                {
                    return Err(GroupApiError::feature_unavailable(
                        "run_admission_fence_expired",
                    ));
                }
                tx.commit().await.map_err(|_| GroupApiError::internal())?;
                (
                    task_run_id,
                    Some(spawn_fence),
                    current.worktree,
                    current.runtime_id,
                    current.request_fingerprint,
                )
            }
        };

    let provision_result = provisioner
        .start_task_cli_session(TaskCliSessionStartCommand {
            tenant_id: actor.tenant_id,
            actor_id: actor.user_id,
            project_id: worktree.project_id,
            repository_id: worktree.repository_id,
            worktree_id,
            work_item_id,
            task_run_id,
            runtime_id,
            expected_lifecycle_version: body.expected_lifecycle_version,
            approved_launch_profile_id: body.approved_launch_profile_id,
            execution_profile_id: body.execution_profile_id,
            correlation_id: body.correlation_id,
            idempotency_key,
            request_fingerprint,
            spawn_fence,
        })
        .await;
    let receipt = match provision_result {
        Ok(receipt) => receipt,
        Err(error) => {
            append_cli_task_run_event(
                &state,
                actor.tenant_id,
                actor.user_id,
                worktree.project_id,
                work_item_id,
                task_run_id,
                "execution_state_changed",
                "failed",
                Some(provision_error_category(error)),
                body.correlation_id,
                json!({ "failure_category": provision_error_category(error) }),
            )
            .await?;
            return Err(map_provision_error(error));
        }
    };
    validate_receipt(&receipt, None)?;
    append_cli_task_run_event(
        &state,
        actor.tenant_id,
        actor.user_id,
        worktree.project_id,
        work_item_id,
        task_run_id,
        "execution_state_changed",
        "running",
        None,
        body.correlation_id,
        json!({ "cli_session_id": receipt.session_id }),
    )
    .await?;

    let mut response = Json(json!({
        "session_id": receipt.session_id,
        "task_run_id": task_run_id,
        "worktree_id": worktree_id,
        "work_item_id": work_item_id,
        "runtime_id": runtime_id,
        "status": "running",
        "attachment_ticket": receipt.attachment_ticket,
        "attachment_ticket_expires_at": receipt.attachment_ticket_expires_at,
        "correlation_id": body.correlation_id,
    }))
    .into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response.headers_mut().insert(
        header::PRAGMA,
        axum::http::HeaderValue::from_static("no-cache"),
    );
    Ok(response)
}

async fn load_task_cli_start_context(
    tx: &mut Transaction<'_, Postgres>,
    actor: &AuthUser,
    worktree_id: Uuid,
    work_item_id: Uuid,
    body: &StartTaskCliSessionBody,
    idempotency_key: &str,
    tolerate_lifecycle_version_change: bool,
) -> Result<TaskCliStartContext, GroupApiError> {
    set_tenant(tx, actor.tenant_id).await?;
    set_actor_scope(tx, actor.user_id).await?;
    let worktree = sqlx::query_as::<_, WorktreeTaskCliScope>(
        r#"
        SELECT p.project_id, w.repo_id AS repository_id, w.runtime_id, w.branch, w.archived
        FROM worktree_canvas_worktree w
        JOIN multica.worktree_project_binding p
          ON p.tenant_id = w.tenant_id AND p.worktree_id = w.id
         AND p.project_id = w.project_id AND p.valid_to IS NULL
        WHERE w.id = $1 AND w.tenant_id = $2
        FOR SHARE OF w, p
        "#,
    )
    .bind(worktree_id)
    .bind(actor.tenant_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    let binding = active_binding(tx, actor, worktree.project_id).await?;
    super::work_items::require_task_writer(&binding.role)?;
    if worktree.archived {
        return Err(GroupApiError::conflict("worktree_archived"));
    }
    let runtime_id = worktree
        .runtime_id
        .ok_or_else(|| GroupApiError::conflict("runtime_not_assigned"))?;
    let request_fingerprint = request_fingerprint(
        actor.tenant_id,
        actor.user_id,
        worktree.project_id,
        worktree.repository_id,
        worktree_id,
        work_item_id,
        runtime_id,
        body,
    )?;
    let existing_run_id = lookup_cli_task_run(
        tx,
        actor.tenant_id,
        actor.user_id,
        idempotency_key,
        &request_fingerprint,
    )
    .await?;
    let lifecycle = sqlx::query_as::<_, TaskCliLifecycle>(
        r#"
        SELECT c.status, c.review_state, c.active_worktree_id, c.claimed_by, c.version
        FROM multica.work_item_worktree l
        JOIN multica.task_metadata m
          ON m.tenant_id = l.tenant_id AND m.work_item_id = l.work_item_id
         AND m.project_id = l.project_id AND m.valid_to IS NULL
        JOIN multica.task_lifecycle_current c
          ON c.tenant_id = m.tenant_id AND c.work_item_id = m.work_item_id
        WHERE l.tenant_id = $1 AND l.project_id = $2 AND l.worktree_id = $3
          AND l.work_item_id = $4 AND l.valid_to IS NULL
        FOR SHARE OF l, m, c
        "#,
    )
    .bind(actor.tenant_id)
    .bind(worktree.project_id)
    .bind(worktree_id)
    .bind(work_item_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    if existing_run_id.is_none() {
        if !tolerate_lifecycle_version_change
            && lifecycle.version != body.expected_lifecycle_version
        {
            return Err(GroupApiError::conflict("version_conflict"));
        }
        if lifecycle.status != "in_progress"
            || lifecycle.review_state == "pending_review"
            || lifecycle.active_worktree_id != Some(worktree_id)
            || lifecycle.claimed_by != Some(actor.user_id)
        {
            return Err(GroupApiError::conflict("task_not_executable_in_worktree"));
        }
    }
    Ok(TaskCliStartContext {
        worktree,
        runtime_id,
        lifecycle,
        request_fingerprint,
        existing_run_id,
    })
}

fn run_admission_readiness_is_fresh(readiness: &TaskRunAdmissionReadiness) -> bool {
    let now = Utc::now();
    let Some(spawn_fence) = readiness.spawn_fence.as_ref() else {
        return false;
    };
    let Ok(expected_binding_digest) = spawn_fence.binding.binding_digest() else {
        return false;
    };
    readiness.runtime_healthy
        && !spawn_fence.fence_id.is_nil()
        && spawn_fence.binding_digest == expected_binding_digest
        && readiness.observed_at <= now
        && now.signed_duration_since(readiness.observed_at) <= Duration::seconds(5)
        && spawn_fence.expires_at > now + Duration::seconds(5)
        && spawn_fence.expires_at <= now + Duration::seconds(30)
}

fn expected_task_run_spawn_fence_binding(
    command: &TaskRunAdmissionReadinessCommand,
    approved_launch_profile: &TaskRunProfileRevisionIdentity,
) -> Option<TaskRunSpawnFenceBinding> {
    let snapshot = command.execution_snapshot.as_ref();
    let execution_profile = &snapshot.profile_identity;
    let scope = &snapshot.scope;
    let revisions = snapshot.catalogs.fence().revisions();
    if approved_launch_profile.profile_id != command.approved_launch_profile_id
        || approved_launch_profile.profile_id.is_nil()
        || approved_launch_profile.version == 0
        || i64::try_from(approved_launch_profile.version).is_err()
        || !is_lower_hex_sha256_digest(&approved_launch_profile.content_digest)
        || execution_profile.profile_id.is_nil()
        || execution_profile.version == 0
        || i64::try_from(execution_profile.version).is_err()
        || !is_lower_hex_sha256_digest(&execution_profile.content_digest)
        || command.execution_profile_id != Some(execution_profile.profile_id)
        || scope.tenant_id != command.tenant_id
        || scope.project_id != command.project_id
        || scope.worktree_id != Some(command.worktree_id)
        || command.expected_lifecycle_version < 1
        || revisions.grant_set_id.is_nil()
        || revisions.grant_set_version == 0
        || snapshot.hook_set.hook_set_id.is_nil()
        || snapshot.hook_set.version == 0
        || !is_lower_hex_sha256_digest(&snapshot.hook_set.effective_digest)
    {
        return None;
    }

    Some(TaskRunSpawnFenceBinding {
        tenant_id: command.tenant_id,
        actor_id: command.actor_id,
        project_id: command.project_id,
        repository_id: command.repository_id,
        worktree_id: command.worktree_id,
        work_item_id: command.work_item_id,
        runtime_id: command.runtime_id,
        expected_lifecycle_version: command.expected_lifecycle_version,
        approved_launch_profile: approved_launch_profile.clone(),
        execution_profile: TaskRunProfileRevisionIdentity {
            profile_id: execution_profile.profile_id,
            version: execution_profile.version,
            content_digest: execution_profile.content_digest.clone(),
        },
        catalog_revisions: revisions.into(),
        hook_set: snapshot.hook_set.clone(),
        resource_budget: snapshot
            .verified_profile
            .document()
            .profile
            .resource_budget
            .clone(),
        request_fingerprint: command.request_fingerprint,
    })
}

fn spawn_fence_matches_readiness_command(
    readiness: &TaskRunAdmissionReadiness,
    command: &TaskRunAdmissionReadinessCommand,
) -> bool {
    let Some(spawn_fence) = readiness.spawn_fence.as_ref() else {
        return false;
    };
    let Some(expected_binding) = expected_task_run_spawn_fence_binding(
        command,
        &spawn_fence.binding.approved_launch_profile,
    ) else {
        return false;
    };
    let Ok(expected_binding_digest) = expected_binding.binding_digest() else {
        return false;
    };
    spawn_fence.binding == expected_binding && spawn_fence.binding_digest == expected_binding_digest
}

fn is_lower_hex_sha256_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn digest_to_lower_hex(digest: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        encoded.push(HEX[(*byte >> 4) as usize] as char);
        encoded.push(HEX[(*byte & 0x0f) as usize] as char);
    }
    encoded
}

#[allow(clippy::too_many_arguments)]
async fn append_run_admission_hook_events(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    actor_id: Uuid,
    project_id: Uuid,
    worktree_id: Uuid,
    work_item_id: Uuid,
    task_run_id: Option<Uuid>,
    event: &HookEventEnvelope,
    evaluation: &domain_hook::HookEvaluation,
    policy: Option<&domain_hook::VerifiedHookPolicySnapshot>,
    duration_ms: i64,
    readiness: &TaskRunAdmissionReadiness,
) -> Result<(), GroupApiError> {
    let project_policy_version = evaluation
        .project_version
        .map(i64::try_from)
        .transpose()
        .map_err(|_| GroupApiError::internal())?;
    let worktree_policy_version = evaluation
        .worktree_version
        .map(i64::try_from)
        .transpose()
        .map_err(|_| GroupApiError::internal())?;
    let event_work_item_id = task_run_id.map(|_| work_item_id);
    let spawn_fence = readiness.spawn_fence.as_ref();
    let details = json!({
        "attempted_work_item_id": work_item_id,
        "readiness_observed_at": readiness.observed_at,
        "admission_fence_expires_at": spawn_fence.map(|fence| fence.expires_at),
        "spawn_fence_binding_digest": spawn_fence.map(|fence| digest_to_lower_hex(&fence.binding_digest)),
        "approved_launch_profile_id": spawn_fence.map(|fence| fence.binding.approved_launch_profile.profile_id),
        "approved_launch_profile_version": spawn_fence.map(|fence| fence.binding.approved_launch_profile.version),
        "approved_launch_profile_digest": spawn_fence.map(|fence| fence.binding.approved_launch_profile.content_digest.as_str()),
        "execution_profile_id": spawn_fence.map(|fence| fence.binding.execution_profile.profile_id),
        "execution_profile_version": spawn_fence.map(|fence| fence.binding.execution_profile.version),
        "execution_profile_digest": spawn_fence.map(|fence| fence.binding.execution_profile.content_digest.as_str()),
        "readiness_runtime_healthy": readiness.runtime_healthy,
        "readiness_fresh_at_evaluation": run_admission_readiness_is_fresh(readiness),
    });
    sqlx::query(
        r#"INSERT INTO multica.hook_execution_event
           (event_id, tenant_id, project_id, worktree_id, work_item_id, run_id,
            actor_id, correlation_id, source_kind, hook_phase, hook_decision,
            hook_reason_code, matched_rule_id, project_policy_version,
            worktree_policy_version, evaluator_api_version, policy_digest,
            evaluated_condition_count, duration_ms, timed_out, details)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'task_run', 'run_admission',
                   $9, $10, $11, $12, $13, $14, $15, $16, $17, false, $18)"#,
    )
    .bind(Uuid::from_bytes(event.event_id))
    .bind(tenant_id)
    .bind(project_id)
    .bind(worktree_id)
    .bind(event_work_item_id)
    .bind(task_run_id)
    .bind(actor_id)
    .bind(Uuid::from_bytes(event.scope.correlation_id))
    .bind(hook_decision_name(evaluation.decision))
    .bind(hook_reason_name(evaluation.reason_code))
    .bind(evaluation.matched_rule_id.map(Uuid::from_bytes))
    .bind(project_policy_version)
    .bind(worktree_policy_version)
    .bind(i16::try_from(evaluation.evaluator_api_version).map_err(|_| GroupApiError::internal())?)
    .bind(evaluation.policy_digest.map(hex::encode))
    .bind(i32::from(evaluation.evaluated_condition_count))
    .bind(duration_ms)
    .bind(details.clone())
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;

    if let Some(task_run_id) = task_run_id {
        let hook_set_version = policy
            .map(|snapshot| snapshot.effective_version())
            .map(i64::try_from)
            .transpose()
            .map_err(|_| GroupApiError::internal())?;
        let hook_rule_version = evaluation
            .matched_rule_id
            .and_then(|rule_id| policy.and_then(|snapshot| snapshot.matched_rule_version(rule_id)))
            .map(i64::try_from)
            .transpose()
            .map_err(|_| GroupApiError::internal())?;
        sqlx::query(
            r#"INSERT INTO multica.task_execution_run_event
               (event_id, tenant_id, project_id, run_id, work_item_id, event_type,
                hook_set_version, hook_rule_id, hook_rule_version,
                hook_evaluator_version, hook_digest, hook_phase, hook_decision,
                hook_reason_class, hook_duration_ms, actor_id, correlation_id, details)
               VALUES ($1, $2, $3, $4, $5, 'hook_evaluated', $6, $7, $8, $9,
                       $10, 'run_admission', $11, $12, $13, $14, $15, $16)"#,
        )
        .bind(Uuid::from_bytes(event.event_id))
        .bind(tenant_id)
        .bind(project_id)
        .bind(task_run_id)
        .bind(work_item_id)
        .bind(hook_set_version)
        .bind(evaluation.matched_rule_id.map(Uuid::from_bytes))
        .bind(hook_rule_version)
        .bind(format!("domain-hook/v{}", evaluation.evaluator_api_version))
        .bind(evaluation.policy_digest.map(hex::encode))
        .bind(hook_decision_name(evaluation.decision))
        .bind(hook_reason_name(evaluation.reason_code))
        .bind(duration_ms)
        .bind(actor_id)
        .bind(Uuid::from_bytes(event.scope.correlation_id))
        .bind(details)
        .execute(&mut **tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
    }
    Ok(())
}

fn hook_decision_name(decision: HookDecision) -> &'static str {
    match decision {
        HookDecision::Allow => "allow",
        HookDecision::Deny => "deny",
        HookDecision::RequireHuman => "require_human",
        HookDecision::Defer => "defer",
    }
}

fn hook_reason_name(reason: domain_hook::HookReasonCode) -> &'static str {
    use domain_hook::HookReasonCode;

    match reason {
        HookReasonCode::AllowedByBuiltinBaseline => "allowed_by_builtin_baseline",
        HookReasonCode::IncompleteScope => "incomplete_scope",
        HookReasonCode::EventSchemaUnsupported => "event_schema_unsupported",
        HookReasonCode::ActorNotAuthorized => "actor_not_authorized",
        HookReasonCode::LifecycleVersionStale => "lifecycle_version_stale",
        HookReasonCode::RuntimeUnhealthyOrUnknown => "runtime_unhealthy_or_unknown",
        HookReasonCode::RetentionLockUnusable => "retention_lock_unusable",
        HookReasonCode::ExecutionNotDrained => "execution_not_drained",
        HookReasonCode::PolicyUnavailable => "policy_unavailable",
        HookReasonCode::PolicyInvalid => "policy_invalid",
        HookReasonCode::RuleDenied => "rule_denied",
        HookReasonCode::HumanApprovalRequired => "human_approval_required",
        HookReasonCode::ExternalConditionPending => "external_condition_pending",
    }
}

async fn lookup_cli_task_run(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    actor_id: Uuid,
    idempotency_key: &str,
    request_hash: &[u8; 32],
) -> Result<Option<Uuid>, GroupApiError> {
    let lock_key = format!("{tenant_id}:{actor_id}:cli_session_start:{idempotency_key}");
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(lock_key)
        .execute(&mut **tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
    sqlx::query(
        r#"
        DELETE FROM multica.task_execution_run_idempotency
        WHERE tenant_id = $1 AND actor_id = $2 AND operation = 'cli_session_start'
          AND idempotency_key = $3 AND expires_at <= now()
        "#,
    )
    .bind(tenant_id)
    .bind(actor_id)
    .bind(idempotency_key)
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    let previous = sqlx::query_as::<_, TaskRunIdempotency>(
        r#"
        SELECT request_hash, run_id
        FROM multica.task_execution_run_idempotency
        WHERE tenant_id = $1 AND actor_id = $2 AND operation = 'cli_session_start'
          AND idempotency_key = $3 AND expires_at > now()
        FOR UPDATE
        "#,
    )
    .bind(tenant_id)
    .bind(actor_id)
    .bind(idempotency_key)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    match previous {
        Some(previous) if previous.request_hash.as_slice() == request_hash => {
            Ok(Some(previous.run_id))
        }
        Some(_) => Err(GroupApiError::conflict("idempotency_key_reused")),
        None => Ok(None),
    }
}

async fn record_cli_task_run(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    actor_id: Uuid,
    project_id: Uuid,
    repository_id: Uuid,
    worktree_id: Uuid,
    runtime_id: Uuid,
    start_ref: &str,
    work_item_id: Uuid,
    correlation_id: Uuid,
    idempotency_key: &str,
    request_hash: &[u8; 32],
    hook_set_snapshot: &serde_json::Value,
    execution_snapshot: &super::CurrentExecutionAdmissionSnapshot,
    expected_approved_launch_profile_id: Uuid,
    spawn_fence: &TaskRunSpawnFence,
) -> Result<Uuid, GroupApiError> {
    if let Some(run_id) =
        lookup_cli_task_run(tx, tenant_id, actor_id, idempotency_key, request_hash).await?
    {
        return Ok(run_id);
    }

    let run_id = Uuid::new_v4();
    let profile_document = execution_snapshot.verified_profile.document();
    if execution_snapshot.profile_identity.content_digest != profile_document.content_digest
        || execution_snapshot.scope.tenant_id != tenant_id
        || execution_snapshot.scope.project_id != project_id
        || execution_snapshot.scope.worktree_id != Some(worktree_id)
    {
        return Err(GroupApiError::conflict(
            "execution_admission_snapshot_changed",
        ));
    }
    let profile_version = i64::try_from(execution_snapshot.profile_identity.version)
        .map_err(|_| GroupApiError::internal())?;
    let approved_launch_profile = &spawn_fence.binding.approved_launch_profile;
    let approved_launch_profile_version =
        i64::try_from(approved_launch_profile.version).map_err(|_| GroupApiError::internal())?;
    if spawn_fence.fence_id.is_nil()
        || approved_launch_profile.profile_id.is_nil()
        || approved_launch_profile.profile_id != expected_approved_launch_profile_id
        || approved_launch_profile.version == 0
        || !is_lower_hex_sha256_digest(&approved_launch_profile.content_digest)
        || spawn_fence.binding.execution_profile.profile_id
            != execution_snapshot.profile_identity.profile_id
        || spawn_fence.binding.execution_profile.version
            != execution_snapshot.profile_identity.version
        || spawn_fence.binding.execution_profile.content_digest
            != execution_snapshot.profile_identity.content_digest
        || spawn_fence.binding.resource_budget != profile_document.profile.resource_budget
    {
        return Err(GroupApiError::conflict(
            "run_admission_fence_binding_invalid",
        ));
    }
    let expected_binding_digest = spawn_fence
        .binding
        .binding_digest()
        .map_err(|_| GroupApiError::internal())?;
    if spawn_fence.binding_digest != expected_binding_digest {
        return Err(GroupApiError::conflict(
            "run_admission_fence_binding_invalid",
        ));
    }
    let spawn_fence_binding_digest = digest_to_lower_hex(&spawn_fence.binding_digest);
    let profile_snapshot =
        serde_json::to_value(profile_document).map_err(|_| GroupApiError::internal())?;
    let resource_budget_snapshot = serde_json::to_value(&profile_document.profile.resource_budget)
        .map_err(|_| GroupApiError::internal())?;
    let loop_policy_snapshot = serde_json::to_value(&profile_document.profile.loop_budget)
        .map_err(|_| GroupApiError::internal())?;
    let inserted = sqlx::query(
        r#"
        INSERT INTO multica.task_execution_run (
            run_id, tenant_id, project_id, work_item_id, initiated_by, execution_channel,
            worktree_id, repository_id, runtime_id, start_ref, task_contract_version,
            task_snapshot, acceptance_snapshot, correlation_id, run_origin, hook_set_snapshot,
            execution_profile_id, execution_profile_version, execution_profile_digest,
            execution_profile_snapshot, resource_budget_snapshot, loop_policy_snapshot,
            approved_launch_profile_id, approved_launch_profile_version,
            approved_launch_profile_digest, spawn_fence_binding_digest
        )
        SELECT $1, $2, $3, $4, $5, 'cli', $6, $7, $8, $9, c.version,
               jsonb_build_object(
                   'metadata_version', m.version,
                   'item_type', m.item_type,
                   'title', m.title,
                   'description', m.description,
                   'priority', m.priority,
                   'labels', to_jsonb(m.labels)
               ),
               CASE WHEN c.contract_id IS NULL THEN NULL ELSE jsonb_build_object(
                   'goal', c.goal,
                   'scope', c.scope,
                   'dependencies', c.dependencies,
                   'acceptance_criteria', c.acceptance_criteria
               ) END,
               $10, 'cli', $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21
        FROM multica.task_metadata m
        LEFT JOIN multica.task_contract c
          ON c.tenant_id = m.tenant_id AND c.project_id = m.project_id
         AND c.work_item_id = m.work_item_id AND c.valid_to IS NULL
        WHERE m.tenant_id = $2 AND m.project_id = $3 AND m.work_item_id = $4
          AND m.valid_to IS NULL
        "#,
    )
    .bind(run_id)
    .bind(tenant_id)
    .bind(project_id)
    .bind(work_item_id)
    .bind(actor_id)
    .bind(worktree_id)
    .bind(repository_id)
    .bind(runtime_id)
    .bind(start_ref)
    .bind(correlation_id)
    .bind(hook_set_snapshot)
    .bind(execution_snapshot.profile_identity.profile_id)
    .bind(profile_version)
    .bind(&execution_snapshot.profile_identity.content_digest)
    .bind(&profile_snapshot)
    .bind(&resource_budget_snapshot)
    .bind(&loop_policy_snapshot)
    .bind(approved_launch_profile.profile_id)
    .bind(approved_launch_profile_version)
    .bind(&approved_launch_profile.content_digest)
    .bind(&spawn_fence_binding_digest)
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    if inserted.rows_affected() != 1 {
        return Err(GroupApiError::not_found());
    }

    super::execution_resources::reserve_project_execution_resources(
        tx,
        tenant_id,
        project_id,
        worktree_id,
        work_item_id,
        run_id,
        actor_id,
        correlation_id,
        spawn_fence.fence_id,
        spawn_fence.expires_at,
        execution_snapshot,
    )
    .await?;

    sqlx::query(
        r#"
        INSERT INTO multica.task_execution_run_event (
            tenant_id, project_id, run_id, work_item_id, event_type, execution_state,
            actor_id, correlation_id
        ) VALUES ($1, $2, $3, $4, 'run_started', 'starting', $5, $6)
        "#,
    )
    .bind(tenant_id)
    .bind(project_id)
    .bind(run_id)
    .bind(work_item_id)
    .bind(actor_id)
    .bind(correlation_id)
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::internal())?;

    sqlx::query(
        r#"
        INSERT INTO multica.task_execution_run_idempotency (
            tenant_id, actor_id, operation, idempotency_key, request_hash, run_id
        ) VALUES ($1, $2, 'cli_session_start', $3, $4, $5)
        "#,
    )
    .bind(tenant_id)
    .bind(actor_id)
    .bind(idempotency_key)
    .bind(request_hash.as_slice())
    .bind(run_id)
    .execute(&mut **tx)
    .await
    .map_err(|_| GroupApiError::conflict("idempotency_race"))?;
    Ok(run_id)
}

async fn append_cli_task_run_event(
    state: &GroupApiState,
    tenant_id: Uuid,
    actor_id: Uuid,
    project_id: Uuid,
    work_item_id: Uuid,
    run_id: Uuid,
    event_type: &str,
    execution_state: &str,
    failure_category: Option<&str>,
    correlation_id: Uuid,
    details: serde_json::Value,
) -> Result<(), GroupApiError> {
    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    set_tenant(&mut tx, tenant_id).await?;
    let run_exists = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT run_id FROM multica.task_execution_run
        WHERE tenant_id = $1 AND project_id = $2 AND work_item_id = $3 AND run_id = $4
        FOR UPDATE
        "#,
    )
    .bind(tenant_id)
    .bind(project_id)
    .bind(work_item_id)
    .bind(run_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .is_some();
    if !run_exists {
        return Err(GroupApiError::internal());
    }

    let previous = sqlx::query(
        r#"
        SELECT execution_state, failure_category, details
        FROM multica.task_execution_run_event
        WHERE tenant_id = $1 AND run_id = $2 AND event_type = $3
        ORDER BY occurred_at DESC, event_id DESC
        LIMIT 1
        "#,
    )
    .bind(tenant_id)
    .bind(run_id)
    .bind(event_type)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    if let Some(previous) = previous {
        let previous_state: Option<String> = previous
            .try_get("execution_state")
            .map_err(|_| GroupApiError::internal())?;
        let previous_category: Option<String> = previous
            .try_get("failure_category")
            .map_err(|_| GroupApiError::internal())?;
        let previous_details: serde_json::Value = previous
            .try_get("details")
            .map_err(|_| GroupApiError::internal())?;
        if previous_state.as_deref() == Some(execution_state)
            && previous_category.as_deref() == failure_category
            && previous_details == details
        {
            tx.commit().await.map_err(|_| GroupApiError::internal())?;
            return Ok(());
        }
    }

    sqlx::query(
        r#"
        INSERT INTO multica.task_execution_run_event (
            tenant_id, project_id, run_id, work_item_id, event_type, execution_state,
            failure_category, actor_id, correlation_id, details
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(tenant_id)
    .bind(project_id)
    .bind(run_id)
    .bind(work_item_id)
    .bind(event_type)
    .bind(execution_state)
    .bind(failure_category)
    .bind(actor_id)
    .bind(correlation_id)
    .bind(details)
    .execute(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;
    Ok(())
}

async fn set_actor_scope(
    tx: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
) -> Result<(), GroupApiError> {
    sqlx::query("SELECT set_config('app.actor_id', $1, true)")
        .bind(actor_id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(|_| GroupApiError::internal())?;
    Ok(())
}

async fn get_task_cli_session_status(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, work_item_id, session_id)): Path<(String, String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Response, GroupApiError> {
    let correlation_id = required_correlation_id(&headers)?;
    let access = authorize_task_cli_session_access(
        &state,
        &actor,
        "agent_session:read",
        &worktree_id,
        &work_item_id,
        &session_id,
        correlation_id,
    )
    .await?;
    let provisioner = state
        .task_cli_session_provisioner
        .ok_or_else(GroupApiError::service_unavailable)?;
    let status = provisioner
        .task_cli_session_status(access.clone())
        .await
        .map_err(map_provision_error)?;
    if status.session_id != access.session_id {
        return Err(GroupApiError::internal());
    }
    Ok(no_store(Json(status)))
}

async fn cancel_task_cli_session(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, work_item_id, session_id)): Path<(String, String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Response, GroupApiError> {
    let correlation_id = required_correlation_id(&headers)?;
    let access = authorize_task_cli_session_access(
        &state,
        &actor,
        "agent_session:cancel",
        &worktree_id,
        &work_item_id,
        &session_id,
        correlation_id,
    )
    .await?;
    let provisioner = state
        .task_cli_session_provisioner
        .ok_or_else(GroupApiError::service_unavailable)?;
    let status = provisioner
        .cancel_task_cli_session(access.clone())
        .await
        .map_err(map_provision_error)?;
    if status.session_id != access.session_id {
        return Err(GroupApiError::internal());
    }
    Ok(no_store(Json(status)))
}

async fn reattach_task_cli_session(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path((worktree_id, work_item_id, session_id)): Path<(String, String, String)>,
    headers: axum::http::HeaderMap,
) -> Result<Response, GroupApiError> {
    let correlation_id = required_correlation_id(&headers)?;
    let access = authorize_task_cli_session_access(
        &state,
        &actor,
        "agent_session:attach",
        &worktree_id,
        &work_item_id,
        &session_id,
        correlation_id,
    )
    .await?;
    let provisioner = state
        .task_cli_session_provisioner
        .ok_or_else(GroupApiError::service_unavailable)?;
    let receipt = provisioner
        .reattach_task_cli_session(access.clone())
        .await
        .map_err(map_provision_error)?;
    validate_receipt(&receipt, Some(access.session_id))?;
    let mut response = Json(json!({
        "session_id": receipt.session_id,
        "worktree_id": access.worktree_id,
        "work_item_id": access.work_item_id,
        "runtime_id": access.runtime_id,
        "status": "attachment_ticket_issued",
        "attachment_ticket": receipt.attachment_ticket,
        "attachment_ticket_expires_at": receipt.attachment_ticket_expires_at,
        "correlation_id": access.correlation_id,
    }))
    .into_response();
    set_no_store_headers(&mut response);
    Ok(response)
}

/// Re-resolve the canonical Task/Worktree relationship and current Project writer membership on
/// every session operation. The Runtime provisioner must repeat live policy and session-binding
/// checks after this transaction commits to close the authorization-to-action race.
async fn authorize_task_cli_session_parent(
    state: &GroupApiState,
    actor: &AuthUser,
    scope: &str,
    worktree_id: &str,
    work_item_id: &str,
    correlation_id: Uuid,
) -> Result<TaskCliSessionParent, GroupApiError> {
    validate_actor(actor)?;
    require_scope(actor, scope)?;
    let worktree_id = Uuid::parse_str(worktree_id).map_err(|_| GroupApiError::bad_request())?;
    let work_item_id = Uuid::parse_str(work_item_id).map_err(|_| GroupApiError::bad_request())?;

    let mut tx = state
        .resolver
        .pool
        .begin()
        .await
        .map_err(|_| GroupApiError::internal())?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    let worktree = sqlx::query_as::<_, WorktreeTaskCliScope>(
        r#"
        SELECT p.project_id, w.repo_id AS repository_id, w.runtime_id, w.archived
        FROM worktree_canvas_worktree w
        JOIN multica.worktree_project_binding p
          ON p.tenant_id = w.tenant_id AND p.worktree_id = w.id
         AND p.project_id = w.project_id AND p.valid_to IS NULL
        WHERE w.id = $1 AND w.tenant_id = $2
        FOR SHARE OF w, p
        "#,
    )
    .bind(worktree_id)
    .bind(actor.tenant_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    let binding = active_binding(&mut tx, actor, worktree.project_id).await?;
    super::work_items::require_task_writer(&binding.role)?;
    if worktree.archived && scope == "agent_session:attach" {
        return Err(GroupApiError::conflict("worktree_archived"));
    }

    let _task_exists = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT 1
        FROM multica.work_item_worktree l
        JOIN multica.task_metadata m
          ON m.tenant_id = l.tenant_id AND m.work_item_id = l.work_item_id
         AND m.project_id = l.project_id AND m.valid_to IS NULL
        JOIN multica.task_lifecycle_current c
          ON c.tenant_id = m.tenant_id AND c.work_item_id = m.work_item_id
        WHERE l.tenant_id = $1 AND l.project_id = $2 AND l.worktree_id = $3
          AND l.work_item_id = $4 AND l.valid_to IS NULL
        FOR SHARE OF l, m, c
        "#,
    )
    .bind(actor.tenant_id)
    .bind(worktree.project_id)
    .bind(worktree_id)
    .bind(work_item_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| GroupApiError::internal())?
    .ok_or_else(GroupApiError::not_found)?;
    let runtime_id = worktree
        .runtime_id
        .ok_or_else(|| GroupApiError::conflict("runtime_not_assigned"))?;
    tx.commit().await.map_err(|_| GroupApiError::internal())?;

    Ok(TaskCliSessionParent {
        tenant_id: actor.tenant_id,
        actor_id: actor.user_id,
        project_id: worktree.project_id,
        repository_id: worktree.repository_id,
        worktree_id,
        work_item_id,
        runtime_id,
        correlation_id,
    })
}

async fn authorize_task_cli_session_access(
    state: &GroupApiState,
    actor: &AuthUser,
    scope: &str,
    worktree_id: &str,
    work_item_id: &str,
    session_id: &str,
    correlation_id: Uuid,
) -> Result<TaskCliSessionAccessCommand, GroupApiError> {
    let session_id = Uuid::parse_str(session_id).map_err(|_| GroupApiError::bad_request())?;
    let parent = authorize_task_cli_session_parent(
        state,
        actor,
        scope,
        worktree_id,
        work_item_id,
        correlation_id,
    )
    .await?;
    Ok(TaskCliSessionAccessCommand {
        tenant_id: parent.tenant_id,
        actor_id: parent.actor_id,
        project_id: parent.project_id,
        repository_id: parent.repository_id,
        worktree_id: parent.worktree_id,
        work_item_id: parent.work_item_id,
        runtime_id: parent.runtime_id,
        session_id,
        correlation_id: parent.correlation_id,
    })
}

fn validate_session_list_limit(limit: Option<usize>) -> Result<usize, GroupApiError> {
    let limit = limit.unwrap_or(20);
    if !(1..=50).contains(&limit) {
        return Err(GroupApiError::bad_request());
    }
    Ok(limit)
}

fn session_statuses_are_valid(sessions: &[TaskCliSessionStatus]) -> bool {
    let mut ids = std::collections::HashSet::with_capacity(sessions.len());
    sessions
        .iter()
        .all(|session| !session.session_id.is_nil() && ids.insert(session.session_id))
}

fn required_correlation_id(headers: &axum::http::HeaderMap) -> Result<Uuid, GroupApiError> {
    let value = headers
        .get("x-correlation-id")
        .and_then(|header| header.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
        .filter(|value| !value.is_nil())
        .ok_or_else(|| GroupApiError::invalid_request("correlation_id_required"))?;
    Ok(value)
}

fn validate_receipt(
    receipt: &TaskCliSessionReceipt,
    expected_session_id: Option<Uuid>,
) -> Result<(), GroupApiError> {
    let now = Utc::now();
    if expected_session_id.is_some_and(|expected| receipt.session_id != expected)
        || receipt.session_id.is_nil()
        || receipt.attachment_ticket.is_empty()
        || receipt.attachment_ticket.len() > 512
        || receipt.attachment_ticket_expires_at <= now
        || receipt.attachment_ticket_expires_at > now + Duration::seconds(60)
    {
        return Err(GroupApiError::internal());
    }
    Ok(())
}

fn no_store(response: impl IntoResponse) -> Response {
    let mut response = response.into_response();
    set_no_store_headers(&mut response);
    response
}

fn set_no_store_headers(response: &mut Response) {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response.headers_mut().insert(
        header::PRAGMA,
        axum::http::HeaderValue::from_static("no-cache"),
    );
}

fn validate_start_body(body: &StartTaskCliSessionBody) -> Result<(), GroupApiError> {
    if body.expected_lifecycle_version < 1
        || body.approved_launch_profile_id.is_nil()
        || body.correlation_id.is_nil()
        || body
            .execution_profile_id
            .is_some_and(|profile_id| profile_id.is_nil())
    {
        return Err(GroupApiError::invalid_request(
            "invalid_task_cli_session_start",
        ));
    }
    Ok(())
}

fn request_fingerprint(
    tenant_id: Uuid,
    actor_id: Uuid,
    project_id: Uuid,
    repository_id: Uuid,
    worktree_id: Uuid,
    work_item_id: Uuid,
    runtime_id: Uuid,
    body: &StartTaskCliSessionBody,
) -> Result<[u8; 32], GroupApiError> {
    let encoded = if body.execution_profile_id.is_some() {
        serde_json::to_vec(&(
            tenant_id,
            actor_id,
            project_id,
            repository_id,
            worktree_id,
            work_item_id,
            runtime_id,
            "cli_session_start_v2",
            body,
        ))
    } else {
        // Keep the exact pre-v2 serialized tuple for retries of legacy admitted Runs.
        serde_json::to_vec(&(
            tenant_id,
            actor_id,
            project_id,
            repository_id,
            worktree_id,
            work_item_id,
            runtime_id,
            body,
        ))
    }
    .map_err(|_| GroupApiError::internal())?;
    Ok(Sha256::digest(encoded).into())
}

fn map_provision_error(error: TaskCliSessionProvisionError) -> GroupApiError {
    match error {
        TaskCliSessionProvisionError::RuntimeUnavailable => GroupApiError::service_unavailable(),
        TaskCliSessionProvisionError::SessionConflict => {
            GroupApiError::conflict("task_cli_session_conflict")
        }
        TaskCliSessionProvisionError::SessionNotFound => GroupApiError::not_found(),
        TaskCliSessionProvisionError::Internal => GroupApiError::internal(),
    }
}

fn provision_error_category(error: TaskCliSessionProvisionError) -> &'static str {
    match error {
        TaskCliSessionProvisionError::RuntimeUnavailable => "runtime_unavailable",
        TaskCliSessionProvisionError::SessionConflict => "session_conflict",
        TaskCliSessionProvisionError::SessionNotFound => "session_not_found",
        TaskCliSessionProvisionError::Internal => "internal",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_list_limit_is_bounded() {
        assert_eq!(validate_session_list_limit(None).unwrap(), 20);
        assert_eq!(validate_session_list_limit(Some(1)).unwrap(), 1);
        assert_eq!(validate_session_list_limit(Some(50)).unwrap(), 50);
        assert!(validate_session_list_limit(Some(0)).is_err());
        assert!(validate_session_list_limit(Some(51)).is_err());
    }

    #[test]
    fn session_receipt_validates_short_lived_ticket_and_optional_binding() {
        let session_id = Uuid::new_v4();
        let make_receipt = |session_id, expires_at| TaskCliSessionReceipt {
            session_id,
            attachment_ticket: "single-use-ticket".to_owned(),
            attachment_ticket_expires_at: expires_at,
        };

        assert!(validate_receipt(
            &make_receipt(session_id, Utc::now() + Duration::seconds(30)),
            None,
        )
        .is_ok());
        assert!(validate_receipt(
            &make_receipt(session_id, Utc::now() + Duration::seconds(30)),
            Some(session_id),
        )
        .is_ok());
        assert!(validate_receipt(
            &make_receipt(session_id, Utc::now() + Duration::seconds(30)),
            Some(Uuid::new_v4()),
        )
        .is_err());
        assert!(validate_receipt(
            &make_receipt(session_id, Utc::now() + Duration::seconds(61)),
            None,
        )
        .is_err());
    }

    #[test]
    fn session_list_rejects_nil_and_duplicate_ids() {
        let session_id = Uuid::new_v4();
        let status = |session_id| TaskCliSessionStatus {
            session_id,
            state: TaskCliSessionState::Running,
            exit_code: None,
            updated_at: Utc::now(),
        };

        assert!(session_statuses_are_valid(&[status(session_id)]));
        assert!(!session_statuses_are_valid(&[
            status(session_id),
            status(session_id)
        ]));
        assert!(!session_statuses_are_valid(&[status(Uuid::nil())]));
    }

    #[test]
    fn run_admission_requires_fresh_readiness_and_a_bounded_fence() {
        let binding = TaskRunSpawnFenceBinding {
            tenant_id: Uuid::new_v4(),
            actor_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            repository_id: Uuid::new_v4(),
            worktree_id: Uuid::new_v4(),
            work_item_id: Uuid::new_v4(),
            runtime_id: Uuid::new_v4(),
            expected_lifecycle_version: 1,
            approved_launch_profile: TaskRunProfileRevisionIdentity {
                profile_id: Uuid::new_v4(),
                version: 1,
                content_digest: "a".repeat(64),
            },
            execution_profile: TaskRunProfileRevisionIdentity {
                profile_id: Uuid::new_v4(),
                version: 1,
                content_digest: "b".repeat(64),
            },
            catalog_revisions: TaskRunCatalogRevisionIdentity {
                provider_catalog_revision: 1,
                skill_catalog_revision: 1,
                grant_set_id: Uuid::new_v4(),
                grant_set_version: 1,
            },
            hook_set: HookSetSnapshot {
                hook_set_id: Uuid::new_v4(),
                version: 1,
                effective_digest: "c".repeat(64),
            },
            resource_budget: ResourceBudgetSnapshot {
                max_rss_bytes: 1024,
                max_cpu_ms: 1000,
                max_runtime_ms: 1000,
                max_child_processes: 1,
                max_parallel_tools: 1,
                max_provider_calls: 1,
                max_output_bytes: 1024,
                max_event_buffer_bytes: 1024,
            },
            request_fingerprint: [7; 32],
        };
        let binding_digest = binding.binding_digest().unwrap();
        let valid = TaskRunAdmissionReadiness {
            runtime_healthy: true,
            observed_at: Utc::now(),
            spawn_fence: Some(TaskRunSpawnFence {
                fence_id: Uuid::new_v4(),
                expires_at: Utc::now() + Duration::seconds(20),
                binding,
                binding_digest,
            }),
        };
        assert!(run_admission_readiness_is_fresh(&valid));

        let mut stale = valid.clone();
        stale.observed_at = Utc::now() - Duration::seconds(6);
        assert!(!run_admission_readiness_is_fresh(&stale));

        let mut missing_fence = valid.clone();
        missing_fence.spawn_fence = None;
        assert!(!run_admission_readiness_is_fresh(&missing_fence));

        let mut short_fence = valid.clone();
        short_fence.spawn_fence.as_mut().unwrap().expires_at = Utc::now() + Duration::seconds(4);
        assert!(!run_admission_readiness_is_fresh(&short_fence));

        let mut unbounded_fence = valid;
        unbounded_fence.spawn_fence.as_mut().unwrap().expires_at =
            Utc::now() + Duration::seconds(31);
        assert!(!run_admission_readiness_is_fresh(&unbounded_fence));
    }
}
