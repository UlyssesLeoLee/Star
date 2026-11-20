/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"crates/nema-codec/src/plugin_runtime.rs"})
MERGE (plugin:File {path:"crates/nema-codec/src/interpreter_plugin.rs"})
MERGE (protocol:Type {id:"crates/nema-codec/src/plugin_runtime.rs::NEMA_PLUGIN_PROTOCOL_VERSION",kind:"type"})
MERGE (parse_req:Type {id:"crates/nema-codec/src/plugin_runtime.rs::ParseRequest",kind:"struct"})
MERGE (parse_resp:Type {id:"crates/nema-codec/src/plugin_runtime.rs::ParseResponse",kind:"struct"})
MERGE (format_req:Type {id:"crates/nema-codec/src/plugin_runtime.rs::FormatRequest",kind:"struct"})
MERGE (format_resp:Type {id:"crates/nema-codec/src/plugin_runtime.rs::FormatResponse",kind:"struct"})
MERGE (limits:Type {id:"crates/nema-codec/src/plugin_runtime.rs::ResourceLimits",kind:"struct"})
MERGE (artifact:Type {id:"crates/nema-codec/src/plugin_runtime.rs::PluginArtifact",kind:"struct"})
MERGE (launch:Type {id:"crates/nema-codec/src/plugin_runtime.rs::WorkerLaunch",kind:"struct"})
MERGE (host:Type {id:"crates/nema-codec/src/plugin_runtime.rs::NemaPluginHost",kind:"trait"})
MERGE (isolated:Type {id:"crates/nema-codec/src/plugin_runtime.rs::IsolatedNemaAdapter",kind:"struct"})
MERGE (binding:Type {id:"crates/nema-codec/src/plugin_runtime.rs::NemaFileBinding",kind:"struct"})
MERGE (preview:Type {id:"crates/nema-codec/src/plugin_runtime.rs::MigrationPreview",kind:"struct"})
MERGE (executed:Type {id:"crates/nema-codec/src/plugin_runtime.rs::ExecutedMigration",kind:"struct"})
MERGE (installer:Type {id:"crates/nema-codec/src/plugin_runtime.rs::NemaPluginInstaller",kind:"struct"})
MERGE (register_isolated:Symbol {id:"crates/nema-codec/src/plugin_runtime.rs::NemaInterpreterRegistry.register_isolated",kind:"method"})
MERGE (parse_isolated:Symbol {id:"crates/nema-codec/src/plugin_runtime.rs::NemaInterpreterRegistry.parse_isolated",kind:"method"})
MERGE (format_isolated:Symbol {id:"crates/nema-codec/src/plugin_runtime.rs::NemaInterpreterRegistry.format_isolated",kind:"method"})
MERGE (preview_migration:Symbol {id:"crates/nema-codec/src/plugin_runtime.rs::NemaInterpreterRegistry.preview_migration",kind:"method"})
MERGE (execute_migration:Symbol {id:"crates/nema-codec/src/plugin_runtime.rs::NemaInterpreterRegistry.execute_migration",kind:"method"})
MERGE (install:Symbol {id:"crates/nema-codec/src/plugin_runtime.rs::NemaPluginInstaller.install_profile",kind:"method"})
MERGE (activate:Symbol {id:"crates/nema-codec/src/plugin_runtime.rs::NemaPluginInstaller.activate",kind:"method"})
MERGE (self)-[:IMPORTS]->(plugin)
MERGE (self)-[:DEFINES]->(protocol)
MERGE (self)-[:DEFINES]->(parse_req)
MERGE (self)-[:DEFINES]->(parse_resp)
MERGE (self)-[:DEFINES]->(format_req)
MERGE (self)-[:DEFINES]->(format_resp)
MERGE (self)-[:DEFINES]->(limits)
MERGE (self)-[:DEFINES]->(artifact)
MERGE (self)-[:DEFINES]->(launch)
MERGE (self)-[:DEFINES]->(host)
MERGE (self)-[:DEFINES]->(isolated)
MERGE (self)-[:DEFINES]->(binding)
MERGE (self)-[:DEFINES]->(preview)
MERGE (self)-[:DEFINES]->(executed)
MERGE (self)-[:DEFINES]->(installer)
MERGE (self)-[:DEFINES]->(register_isolated)
MERGE (self)-[:DEFINES]->(parse_isolated)
MERGE (self)-[:DEFINES]->(format_isolated)
MERGE (self)-[:DEFINES]->(preview_migration)
MERGE (self)-[:DEFINES]->(execute_migration)
MERGE (self)-[:DEFINES]->(install)
MERGE (self)-[:DEFINES]->(activate)
@endcypher
*/
//! Isolated NEMA interpreter plugin runtime: fixed request/response protocol,
//! digest-verified artifacts, resource limits, independent installation, and
//! file-level profile binding with explicit migration.
//!
//! This module owns the *host-side* half of the plugin contract. It never
//! executes foreign code in the Star process: every parse/format of an
//! installed plugin travels through [`NemaPluginHost`], a trait seam whose
//! implementations run the worker behind an isolation boundary (the
//! `sandboxd`-backed host maps this contract to a policy-restricted session;
//! see `crates/sandboxd/src/nema_worker.rs`). There is no `dlopen`, no shell,
//! and no silent fallback to the built-in interpreters anywhere on this path.

use crate::interpreter_plugin::{
    valid_sha256, validate_document_budget, InterpreterError, InterpreterProfile,
    NemaInterpreterAdapter, NemaInterpreterRegistry,
};
use crate::sha256::sha256_hex;
use crate::strict::{
    Document, MAX_COLUMNS, MAX_DOMAINS, MAX_RECORDS, MAX_SOURCE_BYTES, MAX_TOTAL_CELLS,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Fixed NEMA plugin request/response protocol version.
///
/// A host or worker that speaks a different version is rejected, never probed.
pub const NEMA_PLUGIN_PROTOCOL_VERSION: u32 = 1;

/// Worker contract: argv = `[program] + args + [<request-file>, <response-file>]`.
///
/// The request JSON is staged to `<request-file>` by the host; the worker must
/// write its response JSON to `<response-file>` and exit 0. The host executes
/// the program directly (exec, never a shell) inside a restricted session.
pub const WORKER_TRAILING_ARGS: usize = 2;

/// Resource budgets for one plugin invocation.
///
/// The host must enforce memory and CPU/time; this crate enforces the
/// byte-level and structural budgets itself before and after the worker runs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum source bytes accepted for parse.
    pub max_input_bytes: usize,
    /// Maximum response/output bytes accepted from a worker.
    pub max_output_bytes: usize,
    /// Memory budget the host must apply to the worker (bytes).
    pub memory_budget_bytes: u64,
    /// CPU/time budget the host must apply to the worker (milliseconds).
    pub cpu_budget_millis: u64,
    /// Structural budgets mirroring the strict parser.
    pub max_domains: usize,
    pub max_records: usize,
    pub max_total_cells: usize,
    pub max_columns: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: MAX_SOURCE_BYTES,
            max_output_bytes: MAX_SOURCE_BYTES,
            memory_budget_bytes: 256 * 1024 * 1024,
            cpu_budget_millis: 10_000,
            max_domains: MAX_DOMAINS,
            max_records: MAX_RECORDS,
            max_total_cells: MAX_TOTAL_CELLS,
            max_columns: MAX_COLUMNS,
        }
    }
}

fn resource_limit(resource: &str, limit: u64, actual: u64) -> InterpreterError {
    InterpreterError::ResourceLimit {
        resource: resource.to_string(),
        limit,
        actual,
    }
}

/// A parse request on the fixed NEMA plugin protocol.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseRequest {
    pub protocol_version: u32,
    pub profile: InterpreterProfile,
    pub source: String,
    pub limits: ResourceLimits,
}

/// A parse response; exactly one of `ast` / `error` must be present.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParseResponse {
    pub protocol_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ast: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<PluginErrorPayload>,
}

/// A format request: the AST travels as tagged JSON (see `strict::Document`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormatRequest {
    pub protocol_version: u32,
    pub profile: InterpreterProfile,
    pub ast: serde_json::Value,
    pub limits: ResourceLimits,
}

/// A format response; exactly one of `text` / `error` must be present.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormatResponse {
    pub protocol_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<PluginErrorPayload>,
}

/// Worker-reported failure payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginErrorPayload {
    /// Stable machine code. `resource_limit` / `timeout` / `out_of_memory`
    /// map to [`InterpreterError::ResourceLimit`]; everything else maps to
    /// [`InterpreterError::PluginExecution`].
    pub code: String,
    /// Human-readable detail, preserved verbatim in the error.
    pub detail: String,
}

fn check_worker_response(
    protocol_version: u32,
    error: &Option<PluginErrorPayload>,
) -> Result<(), InterpreterError> {
    if protocol_version != NEMA_PLUGIN_PROTOCOL_VERSION {
        return Err(InterpreterError::ProtocolViolation {
            detail: format!(
                "worker speaks protocol {protocol_version}, host requires {NEMA_PLUGIN_PROTOCOL_VERSION}"
            ),
        });
    }
    if let Some(payload) = error {
        return match payload.code.as_str() {
            "resource_limit" | "timeout" | "out_of_memory" => {
                Err(InterpreterError::ResourceLimit {
                    resource: payload.code.clone(),
                    limit: 0,
                    actual: 0,
                })
            }
            _ => Err(InterpreterError::PluginExecution {
                code: payload.code.clone(),
                detail: payload.detail.clone(),
            }),
        };
    }
    Ok(())
}

/// A plugin artifact: opaque bytes plus the SHA-256 digest they must match.
///
/// The payload may be a WASM module or a worker executable bundle; this crate
/// never interprets it, it only pins it. Digest semantics are shared with
/// `domain-agent::PluginDescriptor` (id/version/digest) — see the reuse note
/// in `NemaPluginInstaller`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginArtifact {
    /// Lowercase hex SHA-256 of `payload`.
    pub sha256: String,
    /// Raw artifact bytes.
    pub payload: Vec<u8>,
}

impl PluginArtifact {
    /// Builds an artifact from raw payload bytes, computing the pinned digest.
    pub fn from_payload(payload: Vec<u8>) -> Self {
        let sha256 = sha256_hex(&payload);
        Self { sha256, payload }
    }

    /// Verifies the pinned digest against the payload before any use.
    pub fn verify(&self) -> Result<(), InterpreterError> {
        if !valid_sha256(&self.sha256) {
            return Err(InterpreterError::ArtifactRejected {
                detail: "artifact sha256 must be a 64-char lowercase hex digest".to_string(),
            });
        }
        if self.payload.is_empty() {
            return Err(InterpreterError::ArtifactRejected {
                detail: "artifact payload must not be empty".to_string(),
            });
        }
        let actual = sha256_hex(&self.payload);
        if actual != self.sha256 {
            return Err(InterpreterError::DigestMismatch {
                expected: self.sha256.clone(),
                actual,
            });
        }
        Ok(())
    }
}

/// Describes how to launch a digest-verified worker.
///
/// Maps 1:1 onto `sandboxd::backend::CommandSpec`:
/// `cmd = [program] ++ args`, `env`, `cwd`, `capture_output = true`. The
/// isolation host appends the request/response file paths as the trailing
/// [`WORKER_TRAILING_ARGS`] arguments and executes the program directly —
/// never through a shell. `program` must be a real executable path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerLaunch {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<String>,
    pub cwd: String,
}

/// Isolation seam for executing a plugin worker.
///
/// Implementations must run `launch` behind a real isolation boundary
/// (sandboxed process session or WASM runtime) with the given resource
/// budgets, write the serialized request for the worker to read, and return
/// the raw response bytes. Returning bytes that exceed
/// `limits.max_output_bytes` is a host defect; this crate re-checks anyway.
pub trait NemaPluginHost: Send + Sync {
    /// Runs the worker and returns its raw response bytes.
    fn execute(
        &self,
        launch: &WorkerLaunch,
        request: &[u8],
        limits: &ResourceLimits,
    ) -> Result<Vec<u8>, InterpreterError>;
}

/// [`NemaInterpreterAdapter`] that routes parse/format through the isolated
/// plugin protocol instead of calling built-in Rust code.
pub struct IsolatedNemaAdapter {
    profile: InterpreterProfile,
    launch: WorkerLaunch,
    host: Arc<dyn NemaPluginHost>,
}

impl IsolatedNemaAdapter {
    /// Builds the adapter only for a valid profile and a non-empty launch program.
    pub fn new(
        profile: InterpreterProfile,
        launch: WorkerLaunch,
        host: Arc<dyn NemaPluginHost>,
    ) -> Result<Self, InterpreterError> {
        profile.validate()?;
        if launch.program.is_empty() {
            return Err(InterpreterError::ArtifactRejected {
                detail: "worker launch program must not be empty".to_string(),
            });
        }
        Ok(Self {
            profile,
            launch,
            host,
        })
    }
}

impl NemaInterpreterAdapter for IsolatedNemaAdapter {
    fn profile(&self) -> &InterpreterProfile {
        &self.profile
    }

    fn parse(&self, source: &str) -> Result<Document, InterpreterError> {
        let limits = ResourceLimits::default();
        if source.len() > limits.max_input_bytes {
            return Err(resource_limit(
                "input_bytes",
                limits.max_input_bytes as u64,
                source.len() as u64,
            ));
        }
        let request = ParseRequest {
            protocol_version: NEMA_PLUGIN_PROTOCOL_VERSION,
            profile: self.profile.clone(),
            source: source.to_string(),
            limits: limits.clone(),
        };
        let request_bytes =
            serde_json::to_vec(&request).map_err(|e| InterpreterError::ProtocolViolation {
                detail: format!("cannot encode parse request: {e}"),
            })?;
        let response_bytes = self.host.execute(&self.launch, &request_bytes, &limits)?;
        if response_bytes.len() > limits.max_output_bytes {
            return Err(resource_limit(
                "output_bytes",
                limits.max_output_bytes as u64,
                response_bytes.len() as u64,
            ));
        }
        let response: ParseResponse = serde_json::from_slice(&response_bytes).map_err(|e| {
            InterpreterError::ProtocolViolation {
                detail: format!("malformed worker parse response: {e}"),
            }
        })?;
        check_worker_response(response.protocol_version, &response.error)?;
        let ast: Document = serde_json::from_value(response.ast.ok_or_else(|| {
            InterpreterError::ProtocolViolation {
                detail: "worker parse response carries neither ast nor error".to_string(),
            }
        })?)
        .map_err(|e| InterpreterError::ProtocolViolation {
            detail: format!("worker returned an invalid NEMA AST: {e}"),
        })?;
        validate_document_budget(&ast)?;
        Ok(ast)
    }

    fn format(&self, document: &Document) -> Result<String, InterpreterError> {
        validate_document_budget(document)?;
        let limits = ResourceLimits::default();
        let request = FormatRequest {
            protocol_version: NEMA_PLUGIN_PROTOCOL_VERSION,
            profile: self.profile.clone(),
            ast: serde_json::to_value(document).map_err(|e| {
                InterpreterError::ProtocolViolation {
                    detail: format!("cannot encode format request: {e}"),
                }
            })?,
            limits: limits.clone(),
        };
        let request_bytes =
            serde_json::to_vec(&request).map_err(|e| InterpreterError::ProtocolViolation {
                detail: format!("cannot encode format request: {e}"),
            })?;
        let response_bytes = self.host.execute(&self.launch, &request_bytes, &limits)?;
        if response_bytes.len() > limits.max_output_bytes {
            return Err(resource_limit(
                "output_bytes",
                limits.max_output_bytes as u64,
                response_bytes.len() as u64,
            ));
        }
        let response: FormatResponse = serde_json::from_slice(&response_bytes).map_err(|e| {
            InterpreterError::ProtocolViolation {
                detail: format!("malformed worker format response: {e}"),
            }
        })?;
        check_worker_response(response.protocol_version, &response.error)?;
        let text = response
            .text
            .ok_or_else(|| InterpreterError::ProtocolViolation {
                detail: "worker format response carries neither text nor error".to_string(),
            })?;
        if text.len() > limits.max_output_bytes {
            return Err(resource_limit(
                "output_bytes",
                limits.max_output_bytes as u64,
                text.len() as u64,
            ));
        }
        Ok(text)
    }
}

fn validate_limits(document: &Document, limits: &ResourceLimits) -> Result<(), InterpreterError> {
    if document.domains.len() > limits.max_domains {
        return Err(resource_limit(
            "domains",
            limits.max_domains as u64,
            document.domains.len() as u64,
        ));
    }
    let mut records = 0usize;
    let mut cells = 0usize;
    for domain in &document.domains {
        if let Some(columns) = &domain.columns {
            if columns.len() > limits.max_columns {
                return Err(resource_limit(
                    "columns",
                    limits.max_columns as u64,
                    columns.len() as u64,
                ));
            }
            cells = cells.checked_add(columns.len()).ok_or_else(|| {
                resource_limit("total_cells", limits.max_total_cells as u64, u64::MAX)
            })?;
            if cells > limits.max_total_cells {
                return Err(resource_limit(
                    "total_cells",
                    limits.max_total_cells as u64,
                    cells as u64,
                ));
            }
        }
        records = records
            .checked_add(domain.records.len())
            .ok_or_else(|| resource_limit("records", limits.max_records as u64, u64::MAX))?;
        if records > limits.max_records {
            return Err(resource_limit(
                "records",
                limits.max_records as u64,
                records as u64,
            ));
        }
        for row in &domain.records {
            if row.len() > limits.max_columns {
                return Err(resource_limit(
                    "columns",
                    limits.max_columns as u64,
                    row.len() as u64,
                ));
            }
            cells = cells.checked_add(row.len()).ok_or_else(|| {
                resource_limit("total_cells", limits.max_total_cells as u64, u64::MAX)
            })?;
            if cells > limits.max_total_cells {
                return Err(resource_limit(
                    "total_cells",
                    limits.max_total_cells as u64,
                    cells as u64,
                ));
            }
        }
    }
    Ok(())
}

impl NemaInterpreterRegistry {
    /// Registers an isolated plugin: digest verification, profile validation
    /// (ABI / AST schema fail closed) and adapter construction happen before
    /// anything becomes dispatchable. The registry never falls back from this
    /// adapter to a built-in one.
    pub fn register_isolated(
        &mut self,
        profile: InterpreterProfile,
        artifact: &PluginArtifact,
        launch: WorkerLaunch,
        host: Arc<dyn NemaPluginHost>,
    ) -> Result<(), InterpreterError> {
        artifact.verify()?;
        profile.validate()?;
        let adapter = IsolatedNemaAdapter::new(profile, launch, host)?;
        self.register(Arc::new(adapter))
    }

    /// Parses through the exact installed profile with explicit resource limits.
    ///
    /// An unknown or not-installed profile returns
    /// [`InterpreterError::UnregisteredProfile`]; there is deliberately no
    /// "latest" or built-in fallback lookup.
    pub fn parse_isolated(
        &self,
        profile: &InterpreterProfile,
        source: &str,
        limits: &ResourceLimits,
    ) -> Result<Document, InterpreterError> {
        profile.validate()?;
        if source.len() > limits.max_input_bytes {
            return Err(resource_limit(
                "input_bytes",
                limits.max_input_bytes as u64,
                source.len() as u64,
            ));
        }
        let document = self.parse(profile, source)?;
        validate_limits(&document, limits)?;
        Ok(document)
    }

    /// Formats through the exact installed profile with explicit resource limits.
    pub fn format_isolated(
        &self,
        profile: &InterpreterProfile,
        document: &Document,
        limits: &ResourceLimits,
    ) -> Result<String, InterpreterError> {
        profile.validate()?;
        validate_limits(document, limits)?;
        let text = self.format(profile, document)?;
        if text.len() > limits.max_output_bytes {
            return Err(resource_limit(
                "output_bytes",
                limits.max_output_bytes as u64,
                text.len() as u64,
            ));
        }
        Ok(text)
    }

    /// Previews migrating one file from its bound profile to a target profile.
    ///
    /// The original file text and binding are never modified; the preview
    /// re-parses the source with the old interpreter, canonicalizes with the
    /// target, and reports a structural diff summary. Migration must be made
    /// real by an explicit [`Self::execute_migration`] call.
    pub fn preview_migration(
        &self,
        binding: &NemaFileBinding,
        source: &str,
        target: &InterpreterProfile,
    ) -> Result<MigrationPreview, InterpreterError> {
        binding.profile.validate()?;
        target.validate()?;
        binding.verify(source)?;
        if &binding.profile == target {
            return Err(InterpreterError::MigrationRefused {
                detail: "target profile is identical to the bound profile".to_string(),
            });
        }
        // The old interpreter reads the file; no auto-upgrade happens.
        let old_doc = self.parse(&binding.profile, source)?;
        let formatted = self.format(target, &old_doc)?;
        let new_ast = self.parse(target, &formatted)?;
        let source_reparses_under_target =
            matches!(self.parse(target, source), Ok(doc) if doc == old_doc);
        let diff_summary = summarize_diff(&old_doc, &new_ast, source_reparses_under_target);
        Ok(MigrationPreview {
            original_preserved: true,
            source_reparses_under_target,
            diff_summary,
            new_ast: Some(serde_json::to_value(&new_ast).map_err(|e| {
                InterpreterError::ProtocolViolation {
                    detail: format!("cannot encode migrated AST: {e}"),
                }
            })?),
            formatted_text: Some(formatted),
        })
    }

    /// Executes a migration explicitly and returns the original source for the
    /// caller to persist as a backup copy.
    pub fn execute_migration(
        &self,
        binding: &NemaFileBinding,
        source: &str,
        target: &InterpreterProfile,
    ) -> Result<ExecutedMigration, InterpreterError> {
        let preview = self.preview_migration(binding, source, target)?;
        let formatted = preview
            .formatted_text
            .expect("preview carries formatted text");
        let new_binding = NemaFileBinding::new(target.clone(), &formatted)?;
        Ok(ExecutedMigration {
            new_binding,
            formatted_text: formatted,
            original_source: source.to_string(),
        })
    }
}

/// A file's pinned interpreter identity plus the source digest at bind time.
///
/// `profile` already pins grammar revision, interpreter (plugin) version,
/// ABI, AST schema and both digests; `source_digest` is the only field the
/// profile does not carry, added here so a changed file cannot silently
/// re-bind itself.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NemaFileBinding {
    pub profile: InterpreterProfile,
    /// SHA-256 (lowercase hex) of the `.nema` source text at bind time.
    pub source_digest: String,
}

impl NemaFileBinding {
    /// Creates a binding for the given exact profile and source text.
    pub fn new(profile: InterpreterProfile, source: &str) -> Result<Self, InterpreterError> {
        profile.validate()?;
        Ok(Self {
            profile,
            source_digest: sha256_hex(source.as_bytes()),
        })
    }

    /// Verifies that `source` is exactly the text this binding was made for.
    pub fn verify(&self, source: &str) -> Result<(), InterpreterError> {
        let actual = sha256_hex(source.as_bytes());
        if actual != self.source_digest {
            return Err(InterpreterError::DigestMismatch {
                expected: self.source_digest.clone(),
                actual,
            });
        }
        Ok(())
    }
}

/// Result of a dry-run migration; nothing has been written anywhere.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MigrationPreview {
    /// Always `true` for a preview: the original text and binding are untouched.
    pub original_preserved: bool,
    /// Whether the current source text parses under the target profile with
    /// an identical AST (continuation-style migrations usually do).
    pub source_reparses_under_target: bool,
    /// Compact structural diff: `domains=a/b; records=c/d; cells=e/f;
    /// canonical_text_changed=<bool>`.
    pub diff_summary: String,
    /// The AST the target profile produces from the canonicalized text.
    pub new_ast: Option<serde_json::Value>,
    /// Canonical text under the target profile.
    pub formatted_text: Option<String>,
}

/// Result of an explicit migration; the caller persists both sides.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutedMigration {
    /// The new binding for the migrated (canonical) text.
    pub new_binding: NemaFileBinding,
    /// Canonical text under the target profile.
    pub formatted_text: String,
    /// The original source, returned so the caller can keep a backup copy.
    pub original_source: String,
}

fn summarize_diff(
    old_doc: &Document,
    new_doc: &Document,
    source_reparses_under_target: bool,
) -> String {
    let mut records = 0usize;
    let mut cells = 0usize;
    for domain in &new_doc.domains {
        records += domain.records.len();
        cells += domain.records.iter().map(Vec::len).sum::<usize>();
    }
    let mut old_records = 0usize;
    let mut old_cells = 0usize;
    for domain in &old_doc.domains {
        old_records += domain.records.len();
        old_cells += domain.records.iter().map(Vec::len).sum::<usize>();
    }
    let canonical_text_changed = old_doc != new_doc;
    format!(
        "domains={}/{}; records={}/{}; cells={}/{}; source_reparses_under_target={}; canonical_text_changed={}",
        new_doc.domains.len(),
        old_doc.domains.len(),
        records,
        old_records,
        cells,
        old_cells,
        source_reparses_under_target,
        canonical_text_changed
    )
}

/// Lifecycle state of one installed interpreter plugin version.
///
/// Same state machine as `domain-agent::PluginLifecycleManager`
/// (`Installed → Active → Draining → Drained → Uninstalled`), ported here
/// because this lane cannot add a cross-crate dependency while `Cargo.lock`
/// is frozen. The reuse contract (id@version key, drain-before-uninstall,
/// upgrade drains the active version) is preserved 1:1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginInstallState {
    Installed,
    Active,
    Draining,
    Drained,
    Uninstalled,
}

impl PluginInstallState {
    /// Stable wire identifier for this state.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Installed => "INSTALLED",
            Self::Active => "ACTIVE",
            Self::Draining => "DRAINING",
            Self::Drained => "DRAINED",
            Self::Uninstalled => "UNINSTALLED",
        }
    }
}

/// An admitted lease for one active plugin version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NemaPluginLease {
    key: String,
}

struct InstallRecord {
    profile: InterpreterProfile,
    artifact: PluginArtifact,
    launch: WorkerLaunch,
    state: PluginInstallState,
    active_leases: u32,
}

/// Independent installation of NEMA interpreter plugins.
///
/// `install_profile` verifies the artifact digest and ABI compatibility and
/// stores the version; `activate` makes it dispatchable in the owned
/// registry. An upgrade installs the new version and starts draining the old
/// one — the old profile stays registered and keeps serving pinned files
/// until an explicit uninstall.
pub struct NemaPluginInstaller {
    registry: NemaInterpreterRegistry,
    records: BTreeMap<String, InstallRecord>,
    host: Arc<dyn NemaPluginHost>,
}

fn install_key(plugin_id: &str, version: &str) -> String {
    format!("{plugin_id}@{version}")
}

impl NemaPluginInstaller {
    /// Creates an installer owning `registry`; all isolated plugins run on `host`.
    pub fn new(registry: NemaInterpreterRegistry, host: Arc<dyn NemaPluginHost>) -> Self {
        Self {
            registry,
            records: BTreeMap::new(),
            host,
        }
    }

    /// Installs one plugin version: ABI/AST validation and digest
    /// verification happen before any record exists. A duplicate
    /// `plugin_id@version` is rejected with a dedicated error.
    pub fn install_profile(
        &mut self,
        profile: InterpreterProfile,
        artifact: PluginArtifact,
        launch: WorkerLaunch,
    ) -> Result<(), InterpreterError> {
        profile.validate()?;
        artifact.verify()?;
        if launch.program.is_empty() {
            return Err(InterpreterError::ArtifactRejected {
                detail: "worker launch program must not be empty".to_string(),
            });
        }
        let key = install_key(&profile.plugin_id, &profile.plugin_version);
        if self.records.contains_key(&key) {
            return Err(InterpreterError::AlreadyInstalled(key));
        }
        self.records.insert(
            key,
            InstallRecord {
                profile,
                artifact,
                launch,
                state: PluginInstallState::Installed,
                active_leases: 0,
            },
        );
        Ok(())
    }

    /// Activates an installed version (`Installed → Active`) and registers
    /// its isolated adapter in the owned registry (digest re-verified).
    pub fn activate(&mut self, plugin_id: &str, version: &str) -> Result<(), InterpreterError> {
        let key = install_key(plugin_id, version);
        let record = self
            .records
            .get_mut(&key)
            .ok_or_else(|| InterpreterError::NotInstalled(key.clone()))?;
        if record.state != PluginInstallState::Installed {
            return Err(InterpreterError::InvalidLifecycle {
                detail: format!(
                    "activate requires INSTALLED, current state is {}",
                    record.state.as_str()
                ),
            });
        }
        let adapter = IsolatedNemaAdapter::new(
            record.profile.clone(),
            record.launch.clone(),
            Arc::clone(&self.host),
        )?;
        // Re-verify the stored artifact at activation time.
        record.artifact.verify()?;
        if let Err(error) = self.registry.register(Arc::new(adapter)) {
            return Err(InterpreterError::InvalidLifecycle {
                detail: format!("activation failed to register adapter: {error}"),
            });
        }
        record.state = PluginInstallState::Active;
        Ok(())
    }

    /// Stops new admission for an active version (`Active → Draining`).
    pub fn begin_drain(&mut self, plugin_id: &str, version: &str) -> Result<(), InterpreterError> {
        let key = install_key(plugin_id, version);
        let record = self
            .records
            .get_mut(&key)
            .ok_or_else(|| InterpreterError::NotInstalled(key.clone()))?;
        if record.state != PluginInstallState::Active {
            return Err(InterpreterError::InvalidLifecycle {
                detail: format!(
                    "begin_drain requires ACTIVE, current state is {}",
                    record.state.as_str()
                ),
            });
        }
        record.state = PluginInstallState::Draining;
        Ok(())
    }

    /// Completes a drain (`Draining → Drained`) only with zero outstanding leases.
    pub fn complete_drain(
        &mut self,
        plugin_id: &str,
        version: &str,
    ) -> Result<(), InterpreterError> {
        let key = install_key(plugin_id, version);
        let record = self
            .records
            .get_mut(&key)
            .ok_or_else(|| InterpreterError::NotInstalled(key.clone()))?;
        if record.state != PluginInstallState::Draining {
            return Err(InterpreterError::InvalidLifecycle {
                detail: format!(
                    "complete_drain requires DRAINING, current state is {}",
                    record.state.as_str()
                ),
            });
        }
        if record.active_leases > 0 {
            return Err(InterpreterError::InvalidLifecycle {
                detail: format!(
                    "cannot complete drain: {} lease(s) outstanding",
                    record.active_leases
                ),
            });
        }
        record.state = PluginInstallState::Drained;
        Ok(())
    }

    /// Uninstalls a drained (or never-activated) version and removes its
    /// adapter from the registry so the profile becomes unregistered again.
    pub fn uninstall(&mut self, plugin_id: &str, version: &str) -> Result<(), InterpreterError> {
        let key = install_key(plugin_id, version);
        let record = self
            .records
            .get_mut(&key)
            .ok_or_else(|| InterpreterError::NotInstalled(key.clone()))?;
        match record.state {
            PluginInstallState::Drained | PluginInstallState::Installed => {}
            other => {
                return Err(InterpreterError::InvalidLifecycle {
                    detail: format!(
                        "uninstall requires DRAINED or INSTALLED, current state is {}",
                        other.as_str()
                    ),
                });
            }
        }
        let removed = self.registry.unregister(&record.profile);
        match removed {
            Ok(_) => {}
            Err(InterpreterError::UnregisteredProfile) => {}
            Err(error) => {
                return Err(InterpreterError::InvalidLifecycle {
                    detail: format!("uninstall failed to remove adapter: {error}"),
                });
            }
        }
        record.state = PluginInstallState::Uninstalled;
        Ok(())
    }

    /// Registers a newer version (`Installed`) and starts draining the active one.
    ///
    /// The old version remains registered and keeps serving its pinned files
    /// until the caller explicitly drains, uninstalls and migrates them.
    pub fn upgrade(
        &mut self,
        plugin_id: &str,
        current_version: &str,
        new_profile: InterpreterProfile,
        new_artifact: PluginArtifact,
        new_launch: WorkerLaunch,
    ) -> Result<(), InterpreterError> {
        if new_profile.plugin_id != plugin_id {
            return Err(InterpreterError::InvalidLifecycle {
                detail: format!(
                    "upgrade plugin id mismatch: {} != {}",
                    new_profile.plugin_id, plugin_id
                ),
            });
        }
        if new_profile.plugin_version == current_version {
            return Err(InterpreterError::AlreadyInstalled(install_key(
                plugin_id,
                current_version,
            )));
        }
        let old_key = install_key(plugin_id, current_version);
        let old_record = self
            .records
            .get(&old_key)
            .ok_or_else(|| InterpreterError::NotInstalled(old_key.clone()))?;
        if old_record.state != PluginInstallState::Active {
            return Err(InterpreterError::InvalidLifecycle {
                detail: format!(
                    "upgrade requires current version ACTIVE, current state is {}",
                    old_record.state.as_str()
                ),
            });
        }
        self.install_profile(new_profile, new_artifact, new_launch)?;
        self.begin_drain(plugin_id, current_version)
    }

    /// Admits one unit of work to an active version; fails closed otherwise.
    pub fn admit(
        &mut self,
        plugin_id: &str,
        version: &str,
    ) -> Result<NemaPluginLease, InterpreterError> {
        let key = install_key(plugin_id, version);
        let record = self
            .records
            .get_mut(&key)
            .ok_or_else(|| InterpreterError::NotInstalled(key.clone()))?;
        if record.state != PluginInstallState::Active {
            return Err(InterpreterError::InvalidLifecycle {
                detail: format!("admission stopped: state is {}", record.state.as_str()),
            });
        }
        record.active_leases = record.active_leases.saturating_add(1);
        Ok(NemaPluginLease { key })
    }

    /// Releases one previously admitted lease.
    pub fn release_lease(&mut self, lease: NemaPluginLease) -> Result<(), InterpreterError> {
        let record = self
            .records
            .get_mut(&lease.key)
            .ok_or_else(|| InterpreterError::NotInstalled(lease.key.clone()))?;
        if record.active_leases == 0 {
            return Err(InterpreterError::InvalidLifecycle {
                detail: format!("invalid lease release: {}", lease.key),
            });
        }
        record.active_leases -= 1;
        Ok(())
    }

    /// Returns the lifecycle state of a plugin version, if present.
    pub fn state(&self, plugin_id: &str, version: &str) -> Option<PluginInstallState> {
        self.records
            .get(&install_key(plugin_id, version))
            .map(|record| record.state)
    }

    /// Returns all known versions of a plugin, sorted.
    pub fn versions(&self, plugin_id: &str) -> Vec<String> {
        let prefix = format!("{plugin_id}@");
        let mut versions: Vec<String> = self
            .records
            .keys()
            .filter(|key| key.starts_with(&prefix))
            .map(|key| key[prefix.len()..].to_string())
            .collect();
        versions.sort();
        versions
    }

    /// The registry owned by this installer (dispatchable profiles only).
    pub fn registry(&self) -> &NemaInterpreterRegistry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_requests_serialize_round_trip() {
        let limits = ResourceLimits::default();
        let request = ParseRequest {
            protocol_version: NEMA_PLUGIN_PROTOCOL_VERSION,
            profile: crate::interpreter_plugin::bundled_profile_for_revision(
                crate::interpreter_plugin::NEMA_STRICT_V2_REVISION,
            )
            .unwrap(),
            source: " \\?2 1\nA\n 1 Retry\n".to_string(),
            limits: limits.clone(),
        };
        let bytes = serde_json::to_vec(&request).unwrap();
        let decoded: ParseRequest = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded, request);
        assert_eq!(decoded.protocol_version, NEMA_PLUGIN_PROTOCOL_VERSION);
    }

    #[test]
    fn worker_error_payload_maps_resource_codes() {
        let resource = Some(PluginErrorPayload {
            code: "timeout".to_string(),
            detail: "cpu budget exhausted".to_string(),
        });
        assert!(matches!(
            check_worker_response(NEMA_PLUGIN_PROTOCOL_VERSION, &resource),
            Err(InterpreterError::ResourceLimit { .. })
        ));
        let plain = Some(PluginErrorPayload {
            code: "syntax".to_string(),
            detail: "bad escape".to_string(),
        });
        assert!(matches!(
            check_worker_response(NEMA_PLUGIN_PROTOCOL_VERSION, &plain),
            Err(InterpreterError::PluginExecution { .. })
        ));
        assert!(matches!(
            check_worker_response(NEMA_PLUGIN_PROTOCOL_VERSION + 1, &None),
            Err(InterpreterError::ProtocolViolation { .. })
        ));
    }

    #[test]
    fn artifact_digest_mismatch_is_explicit() {
        let artifact = PluginArtifact {
            sha256: sha256_hex(b"payload"),
            payload: b"payload".to_vec(),
        };
        assert!(artifact.verify().is_ok());
        let tampered = PluginArtifact {
            sha256: sha256_hex(b"payload"),
            payload: b"payloax".to_vec(),
        };
        match tampered.verify() {
            Err(InterpreterError::DigestMismatch { expected, actual }) => {
                assert_eq!(expected, sha256_hex(b"payload"));
                assert_eq!(actual, sha256_hex(b"payloax"));
                assert_ne!(expected, actual);
            }
            other => panic!("expected DigestMismatch, got {other:?}"),
        }
    }

    #[test]
    fn binding_verifies_source_digest() {
        let source = " \\\\1 1\nx\n";
        let binding = NemaFileBinding::new(
            crate::interpreter_plugin::bundled_profile_for_revision(
                crate::interpreter_plugin::NEMA_STRICT_V1_REVISION,
            )
            .unwrap(),
            source,
        )
        .unwrap();
        assert!(binding.verify(source).is_ok());
        assert!(matches!(
            binding.verify(" \\\\1 1\nx\n "),
            Err(InterpreterError::DigestMismatch { .. })
        ));
    }
}
