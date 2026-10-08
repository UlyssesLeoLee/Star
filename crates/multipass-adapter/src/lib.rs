/*
@cypher schema=1 source_sha256=fc6774aabbe5c36e1d0533dfb2d126ff84123f3f67be5227ed75f40fcd8874b2
MERGE (self:File {path:"crates/multipass-adapter/src/lib.rs"})
MERGE (module:Symbol {id:"crates/multipass-adapter/src/lib.rs::module", kind:"module"})
MERGE (read_only_command:Type {id:"crates/multipass-adapter/src/lib.rs::MultipassReadOnlyCommand", kind:"enum"})
MERGE (probe:Type {id:"crates/multipass-adapter/src/lib.rs::MultipassInventoryProbe", kind:"struct"})
MERGE (capability_probe:Type {id:"crates/multipass-adapter/src/lib.rs::MultipassCapabilityProbe", kind:"struct"})
MERGE (capability_observation:Type {id:"crates/multipass-adapter/src/lib.rs::MultipassCapabilityObservation", kind:"struct"})
MERGE (snapshot:Type {id:"crates/multipass-adapter/src/lib.rs::MultipassInventorySnapshot", kind:"struct"})
MERGE (instance:Type {id:"crates/multipass-adapter/src/lib.rs::MultipassInstance", kind:"struct"})
MERGE (state:Type {id:"crates/multipass-adapter/src/lib.rs::MultipassInstanceState", kind:"enum"})
MERGE (error:Type {id:"crates/multipass-adapter/src/lib.rs::MultipassInventoryError", kind:"enum"})
MERGE (document:Type {id:"crates/multipass-adapter/src/lib.rs::MultipassListDocument", kind:"enum"})
MERGE (rows:Type {id:"crates/multipass-adapter/src/lib.rs::MultipassListRows", kind:"struct"})
MERGE (row:Type {id:"crates/multipass-adapter/src/lib.rs::MultipassListRow", kind:"struct"})
MERGE (record:Type {id:"crates/multipass-adapter/src/lib.rs::MultipassInstanceRecord", kind:"struct"})
MERGE (named_instances:Type {id:"crates/multipass-adapter/src/lib.rs::MultipassNamedInstances", kind:"struct"})
MERGE (visitor:Type {id:"crates/multipass-adapter/src/lib.rs::MultipassNamedInstances::NamedInstancesVisitor", kind:"struct"})
MERGE (new:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassInventoryProbe.new", kind:"method"})
MERGE (list:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassInventoryProbe.list_instances", kind:"method"})
MERGE (command_args:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassReadOnlyCommand.args", kind:"method"})
MERGE (command_stdout:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassReadOnlyCommand.max_stdout_bytes", kind:"method"})
MERGE (capability_new:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassCapabilityProbe.new", kind:"method"})
MERGE (capability_run:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassCapabilityProbe.probe", kind:"method"})
MERGE (client_version:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassCapabilityObservation.client_version", kind:"method"})
MERGE (daemon_version:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassCapabilityObservation.daemon_version", kind:"method"})
MERGE (local_driver:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassCapabilityObservation.local_driver", kind:"method"})
MERGE (host_os:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassCapabilityObservation.host_os", kind:"method"})
MERGE (host_arch:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassCapabilityObservation.host_arch", kind:"method"})
MERGE (snapshot_instances:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassInventorySnapshot.instances", kind:"method"})
MERGE (snapshot_parse_json:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassInventorySnapshot.parse_json", kind:"method"})
MERGE (registered_count:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassInventorySnapshot.registered_count", kind:"method"})
MERGE (instance_name:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassInstance.name", kind:"method"})
MERGE (instance_state:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassInstance.state", kind:"method"})
MERGE (deserialize:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassNamedInstances.deserialize", kind:"method"})
MERGE (expecting:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassNamedInstances::NamedInstancesVisitor.expecting", kind:"method"})
MERGE (visit_map:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassNamedInstances::NamedInstancesVisitor.visit_map", kind:"method"})
MERGE (parse:Symbol {id:"crates/multipass-adapter/src/lib.rs::parse_multipass_list_json", kind:"function"})
MERGE (parse_versions:Symbol {id:"crates/multipass-adapter/src/lib.rs::parse_multipass_versions", kind:"function"})
MERGE (parse_driver:Symbol {id:"crates/multipass-adapter/src/lib.rs::parse_multipass_driver", kind:"function"})
MERGE (valid:Symbol {id:"crates/multipass-adapter/src/lib.rs::valid_label", kind:"function"})
MERGE (valid_version:Symbol {id:"crates/multipass-adapter/src/lib.rs::valid_version_token", kind:"function"})
MERGE (valid_driver:Symbol {id:"crates/multipass-adapter/src/lib.rs::valid_driver_token", kind:"function"})
MERGE (read:Symbol {id:"crates/multipass-adapter/src/lib.rs::read_bounded", kind:"function"})
MERGE (stop:Symbol {id:"crates/multipass-adapter/src/lib.rs::stop_child", kind:"function"})
MERGE (acquire:Symbol {id:"crates/multipass-adapter/src/lib.rs::acquire_probe_permit", kind:"function"})
MERGE (run_command:Symbol {id:"crates/multipass-adapter/src/lib.rs::run_multipass_command", kind:"function"})
MERGE (max_stdout:Symbol {id:"crates/multipass-adapter/src/lib.rs::MAX_STDOUT_BYTES", kind:"constant"})
MERGE (max_stderr:Symbol {id:"crates/multipass-adapter/src/lib.rs::MAX_STDERR_BYTES", kind:"constant"})
MERGE (max_rows:Symbol {id:"crates/multipass-adapter/src/lib.rs::MAX_INVENTORY_ROWS", kind:"constant"})
MERGE (deadline:Symbol {id:"crates/multipass-adapter/src/lib.rs::PROBE_DEADLINE", kind:"constant"})
MERGE (read_chunk:Symbol {id:"crates/multipass-adapter/src/lib.rs::READ_CHUNK_BYTES", kind:"constant"})
MERGE (max_name:Symbol {id:"crates/multipass-adapter/src/lib.rs::MAX_INSTANCE_NAME_BYTES", kind:"constant"})
MERGE (max_state:Symbol {id:"crates/multipass-adapter/src/lib.rs::MAX_STATE_BYTES", kind:"constant"})
MERGE (max_capability_output:Symbol {id:"crates/multipass-adapter/src/lib.rs::MAX_CAPABILITY_OUTPUT_BYTES", kind:"constant"})
MERGE (single_flight:Symbol {id:"crates/multipass-adapter/src/lib.rs::MULTIPASS_PROBE_SINGLE_FLIGHT", kind:"static"})
MERGE (tests:Symbol {id:"crates/multipass-adapter/src/lib.rs::tests", kind:"module"})
MERGE (deleted_test:Symbol {id:"crates/multipass-adapter/src/lib.rs::tests::deleted_instances_remain_in_inventory_count", kind:"test"})
MERGE (shape_test:Symbol {id:"crates/multipass-adapter/src/lib.rs::tests::malformed_inventory_fails_closed", kind:"test"})
MERGE (duplicate_test:Symbol {id:"crates/multipass-adapter/src/lib.rs::tests::duplicate_instance_names_fail_closed", kind:"test"})
MERGE (limit_test:Symbol {id:"crates/multipass-adapter/src/lib.rs::tests::inventory_row_limit_is_enforced", kind:"test"})
MERGE (state_test:Symbol {id:"crates/multipass-adapter/src/lib.rs::tests::unknown_state_is_preserved_as_unclassified", kind:"test"})
MERGE (stdout_test:Symbol {id:"crates/multipass-adapter/src/lib.rs::tests::stdout_bound_is_enforced", kind:"test"})
MERGE (stderr_test:Symbol {id:"crates/multipass-adapter/src/lib.rs::tests::stderr_bound_uses_its_own_error", kind:"test"})
MERGE (exact_test:Symbol {id:"crates/multipass-adapter/src/lib.rs::tests::exact_output_bound_is_accepted", kind:"test"})
MERGE (path_test:Symbol {id:"crates/multipass-adapter/src/lib.rs::tests::relative_executable_path_is_rejected", kind:"test"})
MERGE (map_test:Symbol {id:"crates/multipass-adapter/src/lib.rs::tests::official_map_inventory_is_parsed", kind:"test"})
MERGE (duplicate_map_test:Symbol {id:"crates/multipass-adapter/src/lib.rs::tests::duplicate_map_keys_fail_closed", kind:"test"})
MERGE (single_flight_test:Symbol {id:"crates/multipass-adapter/src/lib.rs::tests::single_flight_is_shared_across_probe_instances", kind:"test"})
MERGE (capability_command_test:Symbol {id:"crates/multipass-adapter/src/lib.rs::tests::capability_commands_are_fixed_read_only_and_bounded", kind:"test"})
MERGE (version_parse_test:Symbol {id:"crates/multipass-adapter/src/lib.rs::tests::documented_version_output_is_parsed_without_classifying_compatibility", kind:"test"})
MERGE (version_invalid_test:Symbol {id:"crates/multipass-adapter/src/lib.rs::tests::malformed_or_ambiguous_version_output_fails_closed", kind:"test"})
MERGE (driver_unknown_test:Symbol {id:"crates/multipass-adapter/src/lib.rs::tests::driver_probe_preserves_unknown_values_as_observations", kind:"test"})
MERGE (driver_invalid_test:Symbol {id:"crates/multipass-adapter/src/lib.rs::tests::malformed_or_ambiguous_driver_output_fails_closed", kind:"test"})
MERGE (empty_inventory_test:Symbol {id:"crates/multipass-adapter/src/lib.rs::tests::empty_inventory_is_a_valid_observation", kind:"test"})
MERGE (oversized_json_test:Symbol {id:"crates/multipass-adapter/src/lib.rs::tests::public_inventory_parser_enforces_output_bound", kind:"test"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(read_only_command)
MERGE (module)-[:DEFINES]->(probe)
MERGE (module)-[:DEFINES]->(capability_probe)
MERGE (module)-[:DEFINES]->(capability_observation)
MERGE (module)-[:DEFINES]->(snapshot)
MERGE (module)-[:DEFINES]->(instance)
MERGE (module)-[:DEFINES]->(state)
MERGE (module)-[:DEFINES]->(error)
MERGE (module)-[:DEFINES]->(document)
MERGE (module)-[:DEFINES]->(rows)
MERGE (module)-[:DEFINES]->(row)
MERGE (module)-[:DEFINES]->(record)
MERGE (module)-[:DEFINES]->(named_instances)
MERGE (named_instances)-[:DEFINES]->(visitor)
MERGE (module)-[:DEFINES]->(parse)
MERGE (module)-[:DEFINES]->(parse_versions)
MERGE (module)-[:DEFINES]->(parse_driver)
MERGE (module)-[:DEFINES]->(valid)
MERGE (module)-[:DEFINES]->(valid_version)
MERGE (module)-[:DEFINES]->(valid_driver)
MERGE (module)-[:DEFINES]->(read)
MERGE (module)-[:DEFINES]->(stop)
MERGE (module)-[:DEFINES]->(acquire)
MERGE (module)-[:DEFINES]->(run_command)
MERGE (module)-[:DEFINES]->(max_stdout)
MERGE (module)-[:DEFINES]->(max_stderr)
MERGE (module)-[:DEFINES]->(max_rows)
MERGE (module)-[:DEFINES]->(deadline)
MERGE (module)-[:DEFINES]->(read_chunk)
MERGE (module)-[:DEFINES]->(max_name)
MERGE (module)-[:DEFINES]->(max_state)
MERGE (module)-[:DEFINES]->(max_capability_output)
MERGE (module)-[:DEFINES]->(single_flight)
MERGE (module)-[:DEFINES]->(tests)
MERGE (probe)-[:DEFINES]->(new)
MERGE (probe)-[:DEFINES]->(list)
MERGE (read_only_command)-[:DEFINES]->(command_args)
MERGE (read_only_command)-[:DEFINES]->(command_stdout)
MERGE (capability_probe)-[:DEFINES]->(capability_new)
MERGE (capability_probe)-[:DEFINES]->(capability_run)
MERGE (capability_observation)-[:DEFINES]->(client_version)
MERGE (capability_observation)-[:DEFINES]->(daemon_version)
MERGE (capability_observation)-[:DEFINES]->(local_driver)
MERGE (capability_observation)-[:DEFINES]->(host_os)
MERGE (capability_observation)-[:DEFINES]->(host_arch)
MERGE (snapshot)-[:DEFINES]->(snapshot_instances)
MERGE (snapshot)-[:DEFINES]->(snapshot_parse_json)
MERGE (snapshot)-[:DEFINES]->(registered_count)
MERGE (instance)-[:DEFINES]->(instance_name)
MERGE (instance)-[:DEFINES]->(instance_state)
MERGE (named_instances)-[:DEFINES]->(deserialize)
MERGE (visitor)-[:DEFINES]->(expecting)
MERGE (visitor)-[:DEFINES]->(visit_map)
MERGE (tests)-[:DEFINES]->(deleted_test)
MERGE (tests)-[:DEFINES]->(shape_test)
MERGE (tests)-[:DEFINES]->(duplicate_test)
MERGE (tests)-[:DEFINES]->(limit_test)
MERGE (tests)-[:DEFINES]->(state_test)
MERGE (tests)-[:DEFINES]->(stdout_test)
MERGE (tests)-[:DEFINES]->(stderr_test)
MERGE (tests)-[:DEFINES]->(exact_test)
MERGE (tests)-[:DEFINES]->(path_test)
MERGE (tests)-[:DEFINES]->(map_test)
MERGE (tests)-[:DEFINES]->(duplicate_map_test)
MERGE (tests)-[:DEFINES]->(single_flight_test)
MERGE (tests)-[:DEFINES]->(capability_command_test)
MERGE (tests)-[:DEFINES]->(version_parse_test)
MERGE (tests)-[:DEFINES]->(version_invalid_test)
MERGE (tests)-[:DEFINES]->(driver_unknown_test)
MERGE (tests)-[:DEFINES]->(driver_invalid_test)
MERGE (tests)-[:DEFINES]->(empty_inventory_test)
MERGE (tests)-[:DEFINES]->(oversized_json_test)
MERGE (list)-[:CALLS]->(acquire)
MERGE (list)-[:CALLS]->(run_command)
MERGE (list)-[:CALLS]->(parse)
MERGE (list)-[:CALLS]->(snapshot_parse_json)
MERGE (snapshot_parse_json)-[:CALLS]->(parse)
MERGE (capability_run)-[:CALLS]->(acquire)
MERGE (capability_run)-[:CALLS]->(run_command)
MERGE (capability_run)-[:CALLS]->(parse_versions)
MERGE (capability_run)-[:CALLS]->(parse_driver)
MERGE (capability_run)-[:USES_TYPE]->(capability_observation)
MERGE (command_stdout)-[:USES_TYPE]->(max_stdout)
MERGE (command_stdout)-[:USES_TYPE]->(max_capability_output)
MERGE (run_command)-[:CALLS]->(command_args)
MERGE (run_command)-[:CALLS]->(command_stdout)
MERGE (run_command)-[:CALLS]->(read)
MERGE (run_command)-[:CALLS]->(stop)
MERGE (run_command)-[:USES_TYPE]->(deadline)
MERGE (run_command)-[:USES_TYPE]->(max_stderr)
MERGE (acquire)-[:READS]->(single_flight)
MERGE (parse)-[:USES_TYPE]->(document)
MERGE (parse)-[:USES_TYPE]->(max_rows)
MERGE (parse)-[:USES_TYPE]->(max_name)
MERGE (parse)-[:USES_TYPE]->(max_state)
MERGE (parse)-[:CALLS]->(valid)
MERGE (parse_versions)-[:CALLS]->(valid_version)
MERGE (parse_driver)-[:CALLS]->(valid_driver)
MERGE (valid_version)-[:CALLS]->(valid)
MERGE (valid_driver)-[:CALLS]->(valid)
MERGE (deserialize)-[:CALLS]->(visit_map)
MERGE (read)-[:USES_TYPE]->(read_chunk)
MERGE (tests)-[:CALLS]->(parse)
MERGE (deleted_test)-[:CALLS]->(parse)
MERGE (shape_test)-[:CALLS]->(parse)
MERGE (duplicate_test)-[:CALLS]->(parse)
MERGE (map_test)-[:CALLS]->(parse)
MERGE (duplicate_map_test)-[:CALLS]->(parse)
MERGE (limit_test)-[:CALLS]->(parse)
MERGE (state_test)-[:CALLS]->(parse)
MERGE (stdout_test)-[:CALLS]->(read)
MERGE (stderr_test)-[:CALLS]->(read)
MERGE (exact_test)-[:CALLS]->(read)
MERGE (path_test)-[:CALLS]->(new)
MERGE (capability_command_test)-[:CALLS]->(command_args)
MERGE (capability_command_test)-[:CALLS]->(command_stdout)
MERGE (version_parse_test)-[:CALLS]->(parse_versions)
MERGE (version_invalid_test)-[:CALLS]->(parse_versions)
MERGE (driver_unknown_test)-[:CALLS]->(parse_driver)
MERGE (driver_invalid_test)-[:CALLS]->(parse_driver)
MERGE (empty_inventory_test)-[:CALLS]->(snapshot_parse_json)
MERGE (oversized_json_test)-[:CALLS]->(snapshot_parse_json)
MERGE (single_flight_test)-[:CALLS]->(new)
MERGE (single_flight_test)-[:CALLS]->(list)
MERGE (single_flight_test)-[:CALLS]->(capability_new)
MERGE (single_flight_test)-[:CALLS]->(capability_run)
MERGE (single_flight_test)-[:READS]->(single_flight)
@endcypher
*/

//! Bounded, read-only observation of the local Multipass instance inventory.
//!
//! This crate is only a process adapter. It does not establish instance ownership,
//! HostEnvironment or cluster identity, capacity, daemon readiness, guest readiness,
//! or authorization. It deliberately exposes no lifecycle or guest-execution command.
//! Version and driver facts are observations only; they do not prove readiness.

use std::collections::HashSet;
use std::fmt;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::OnceLock;
use std::time::Duration;

use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::{Child, Command};
use tokio::sync::Semaphore;
use tokio::time::timeout;

/// Maximum bytes accepted from Multipass on stdout.
pub const MAX_STDOUT_BYTES: usize = 128 * 1024;

/// Maximum bytes accepted from Multipass on stderr.
pub const MAX_STDERR_BYTES: usize = 4 * 1024;

/// Maximum number of rows accepted from a single inventory response.
pub const MAX_INVENTORY_ROWS: usize = 512;

const PROBE_DEADLINE: Duration = Duration::from_secs(5);
const READ_CHUNK_BYTES: usize = 4096;
const MAX_INSTANCE_NAME_BYTES: usize = 128;
const MAX_STATE_BYTES: usize = 64;

static MULTIPASS_PROBE_SINGLE_FLIGHT: OnceLock<Semaphore> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MultipassReadOnlyCommand {
    Inventory,
    Version,
    LocalDriver,
}

impl MultipassReadOnlyCommand {
    fn args(self) -> &'static [&'static str] {
        match self {
            Self::Inventory => &["list", "--format", "json"],
            Self::Version => &["version"],
            Self::LocalDriver => &["get", "local.driver"],
        }
    }

    fn max_stdout_bytes(self) -> usize {
        match self {
            Self::Inventory => MAX_STDOUT_BYTES,
            Self::Version | Self::LocalDriver => MAX_CAPABILITY_OUTPUT_BYTES,
        }
    }
}

const MAX_CAPABILITY_OUTPUT_BYTES: usize = 4 * 1024;

/// A trusted, single-flight Multipass inventory probe.
///
/// Construct it with the absolute path resolved by the trusted Host Manager. The
/// adapter invokes exactly `list --format json`; callers cannot supply arguments.
pub struct MultipassInventoryProbe {
    executable: PathBuf,
}

impl MultipassInventoryProbe {
    /// Create a probe for an already-resolved absolute Multipass executable path.
    pub fn new(executable: impl Into<PathBuf>) -> Result<Self, MultipassInventoryError> {
        let executable = executable.into();
        if !executable.is_absolute() {
            return Err(MultipassInventoryError::ExecutablePathMustBeAbsolute);
        }

        Ok(Self { executable })
    }

    /// Read all listed instances without changing Multipass state.
    ///
    /// Stopped and Deleted entries are retained. The result is an observation only;
    /// the caller must not use it as ownership, capacity-reservation, or readiness proof.
    pub async fn list_instances(
        &self,
    ) -> Result<MultipassInventorySnapshot, MultipassInventoryError> {
        let _permit = acquire_probe_permit()?;
        let stdout =
            run_multipass_command(&self.executable, MultipassReadOnlyCommand::Inventory).await?;
        MultipassInventorySnapshot::parse_json(&stdout)
    }
}

/// A bounded read-only probe for local Multipass host facts.
///
/// It reports the CLI/daemon versions, configured driver, and the Rust host target. These
/// observations are not an authorization, capacity, guest, K3s, or Runtime-readiness proof.
pub struct MultipassCapabilityProbe {
    executable: PathBuf,
}

impl MultipassCapabilityProbe {
    /// Create a probe for a trusted, already-resolved absolute Multipass executable path.
    pub fn new(executable: impl Into<PathBuf>) -> Result<Self, MultipassInventoryError> {
        let executable = executable.into();
        if !executable.is_absolute() {
            return Err(MultipassInventoryError::ExecutablePathMustBeAbsolute);
        }

        Ok(Self { executable })
    }

    /// Read the CLI version, daemon version, selected local driver, host OS, and architecture.
    ///
    /// The two fixed CLI calls are serialized with inventory probes and cannot mutate instances.
    pub async fn probe(&self) -> Result<MultipassCapabilityObservation, MultipassInventoryError> {
        let _permit = acquire_probe_permit()?;
        let versions =
            run_multipass_command(&self.executable, MultipassReadOnlyCommand::Version).await?;
        let (client_version, daemon_version) = parse_multipass_versions(&versions)?;
        let driver =
            run_multipass_command(&self.executable, MultipassReadOnlyCommand::LocalDriver).await?;
        let local_driver = parse_multipass_driver(&driver)?;

        Ok(MultipassCapabilityObservation {
            client_version,
            daemon_version,
            local_driver,
            host_os: std::env::consts::OS,
            host_arch: std::env::consts::ARCH,
        })
    }
}

/// Bounded host/provider facts observed from Multipass and the current Rust target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultipassCapabilityObservation {
    client_version: String,
    daemon_version: String,
    local_driver: String,
    host_os: &'static str,
    host_arch: &'static str,
}

impl MultipassCapabilityObservation {
    /// Return the CLI version reported by Multipass.
    pub fn client_version(&self) -> &str {
        &self.client_version
    }

    /// Return the daemon version reported by Multipass.
    pub fn daemon_version(&self) -> &str {
        &self.daemon_version
    }

    /// Return the configured `local.driver` value without classifying unknown drivers.
    pub fn local_driver(&self) -> &str {
        &self.local_driver
    }

    /// Return the operating-system target of this Rust process.
    pub fn host_os(&self) -> &'static str {
        self.host_os
    }

    /// Return the architecture target of this Rust process.
    pub fn host_arch(&self) -> &'static str {
        self.host_arch
    }
}

/// A complete bounded snapshot from `multipass list --format json`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultipassInventorySnapshot {
    instances: Vec<MultipassInstance>,
}

impl MultipassInventorySnapshot {
    /// Parse a bounded Multipass JSON inventory without running a process.
    ///
    /// This validates the document shape and row bounds; it does not authenticate the source.
    pub fn parse_json(bytes: &[u8]) -> Result<Self, MultipassInventoryError> {
        if bytes.len() > MAX_STDOUT_BYTES {
            return Err(MultipassInventoryError::StdoutLimitExceeded);
        }

        parse_multipass_list_json(bytes)
    }

    /// Return all observed rows, including Stopped and Deleted entries.
    pub fn instances(&self) -> &[MultipassInstance] {
        &self.instances
    }

    /// Return the number of registered inventory rows, without filtering by state.
    pub fn registered_count(&self) -> usize {
        self.instances.len()
    }
}

/// One observed Multipass instance row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultipassInstance {
    name: String,
    state: MultipassInstanceState,
}

impl MultipassInstance {
    /// Return the Multipass display name; this is not a durable instance identity.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the state reported by Multipass.
    pub fn state(&self) -> &MultipassInstanceState {
        &self.state
    }
}

/// A state reported by Multipass, preserving values this adapter does not classify.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MultipassInstanceState {
    /// The guest is reported as running.
    Running,
    /// The guest is reported as stopped.
    Stopped,
    /// The guest is deleted but its recoverable record may retain disk data.
    Deleted,
    /// A newer or otherwise unclassified state; it is never treated as ready.
    Other(String),
}

/// Errors from the bounded read-only Multipass inventory probe.
#[derive(Debug, Error)]
pub enum MultipassInventoryError {
    /// The executable path was not absolute.
    #[error("Multipass executable path must be absolute")]
    ExecutablePathMustBeAbsolute,
    /// A previous read-only Multipass probe is still running.
    #[error("Multipass read-only probe is already running")]
    ProbeAlreadyRunning,
    /// The operating system could not start Multipass.
    #[error("could not start Multipass read-only command: {0}")]
    Spawn(#[source] std::io::Error),
    /// The child did not expose the expected stdout pipe.
    #[error("Multipass stdout pipe is unavailable")]
    MissingStdoutPipe,
    /// The child did not expose the expected stderr pipe.
    #[error("Multipass stderr pipe is unavailable")]
    MissingStderrPipe,
    /// Multipass exited unsuccessfully; raw stderr is intentionally not retained.
    #[error("Multipass read-only command exited unsuccessfully (code {0:?})")]
    CommandFailed(Option<i32>),
    /// The command did not finish within its fixed deadline.
    #[error("Multipass read-only command exceeded its 5 second deadline")]
    DeadlineExceeded,
    /// The operating system reported an output-pipe read failure.
    #[error("could not read Multipass output: {0}")]
    Read(#[source] std::io::Error),
    /// The child process could not be reaped.
    #[error("could not wait for Multipass read-only command: {0}")]
    Wait(#[source] std::io::Error),
    /// Multipass produced more stdout than the configured bound.
    #[error("Multipass stdout exceeded its byte limit")]
    StdoutLimitExceeded,
    /// Multipass produced more stderr than the configured bound.
    #[error("Multipass stderr exceeded its byte limit")]
    StderrLimitExceeded,
    /// The JSON response could not be decoded.
    #[error("Multipass inventory response is not valid JSON")]
    InvalidJson,
    /// The version command returned an unknown or ambiguous report.
    #[error("Multipass version report is malformed or unsupported")]
    InvalidVersionOutput,
    /// The local.driver setting was missing, oversized, or ambiguous.
    #[error("Multipass local driver report is malformed or unsupported")]
    InvalidDriverOutput,
    /// A row had an empty, oversized, or control-character identity/state.
    #[error("Multipass inventory contains an invalid instance row")]
    InvalidInstance,
    /// The response contained the same instance name more than once.
    #[error("Multipass inventory contains a duplicate instance name")]
    DuplicateInstance,
    /// The response exceeded the row-count bound.
    #[error("Multipass inventory exceeded its row-count limit")]
    RowLimitExceeded,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum MultipassListDocument {
    Rows(MultipassListRows),
    Instances(MultipassNamedInstances),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MultipassListRows {
    list: Vec<MultipassListRow>,
}

#[derive(Deserialize)]
struct MultipassListRow {
    name: String,
    state: String,
}

#[derive(Deserialize)]
struct MultipassInstanceRecord {
    state: String,
}

struct MultipassNamedInstances(Vec<(String, Vec<MultipassInstanceRecord>)>);

impl<'de> Deserialize<'de> for MultipassNamedInstances {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NamedInstancesVisitor;

        impl<'de> Visitor<'de> for NamedInstancesVisitor {
            type Value = MultipassNamedInstances;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an object mapping instance names to state records")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut instances = Vec::new();
                while let Some(entry) = map.next_entry::<String, Vec<MultipassInstanceRecord>>()? {
                    instances.push(entry);
                }
                Ok(MultipassNamedInstances(instances))
            }
        }

        deserializer.deserialize_map(NamedInstancesVisitor)
    }
}

fn parse_multipass_list_json(
    bytes: &[u8],
) -> Result<MultipassInventorySnapshot, MultipassInventoryError> {
    let document: MultipassListDocument =
        serde_json::from_slice(bytes).map_err(|_| MultipassInventoryError::InvalidJson)?;
    let rows = match document {
        MultipassListDocument::Rows(rows) => rows.list,
        MultipassListDocument::Instances(MultipassNamedInstances(instances)) => {
            let mut rows = Vec::new();
            for (name, records) in instances {
                if records.is_empty() {
                    return Err(MultipassInventoryError::InvalidInstance);
                }
                rows.extend(records.into_iter().map(|record| MultipassListRow {
                    name: name.clone(),
                    state: record.state,
                }));
            }
            rows
        }
    };
    if rows.len() > MAX_INVENTORY_ROWS {
        return Err(MultipassInventoryError::RowLimitExceeded);
    }

    let mut names = HashSet::with_capacity(rows.len());
    let mut instances = Vec::with_capacity(rows.len());
    for row in rows {
        if !valid_label(&row.name, MAX_INSTANCE_NAME_BYTES)
            || !valid_label(&row.state, MAX_STATE_BYTES)
        {
            return Err(MultipassInventoryError::InvalidInstance);
        }
        if !names.insert(row.name.clone()) {
            return Err(MultipassInventoryError::DuplicateInstance);
        }

        let state = match row.state.as_str() {
            "Running" => MultipassInstanceState::Running,
            "Stopped" => MultipassInstanceState::Stopped,
            "Deleted" => MultipassInstanceState::Deleted,
            other => MultipassInstanceState::Other(other.to_owned()),
        };
        instances.push(MultipassInstance {
            name: row.name,
            state,
        });
    }

    Ok(MultipassInventorySnapshot { instances })
}

fn valid_label(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && !value.chars().any(char::is_control)
        && value.trim() == value
}

fn parse_multipass_versions(bytes: &[u8]) -> Result<(String, String), MultipassInventoryError> {
    let output =
        std::str::from_utf8(bytes).map_err(|_| MultipassInventoryError::InvalidVersionOutput)?;
    let mut client_version = None;
    let mut daemon_version = None;

    for line in output.lines() {
        let mut fields = line.split_ascii_whitespace();
        let name = fields
            .next()
            .ok_or(MultipassInventoryError::InvalidVersionOutput)?;
        let version = fields
            .next()
            .ok_or(MultipassInventoryError::InvalidVersionOutput)?;
        if fields.next().is_some() || !valid_version_token(version) {
            return Err(MultipassInventoryError::InvalidVersionOutput);
        }

        let target = match name {
            "multipass" => &mut client_version,
            "multipassd" => &mut daemon_version,
            _ => return Err(MultipassInventoryError::InvalidVersionOutput),
        };
        if target.replace(version.to_owned()).is_some() {
            return Err(MultipassInventoryError::InvalidVersionOutput);
        }
    }

    match (client_version, daemon_version) {
        (Some(client), Some(daemon)) => Ok((client, daemon)),
        _ => Err(MultipassInventoryError::InvalidVersionOutput),
    }
}

fn parse_multipass_driver(bytes: &[u8]) -> Result<String, MultipassInventoryError> {
    let output =
        std::str::from_utf8(bytes).map_err(|_| MultipassInventoryError::InvalidDriverOutput)?;
    let output = output.strip_suffix('\n').unwrap_or(output);
    let output = output.strip_suffix('\r').unwrap_or(output);
    if output
        .chars()
        .any(|character| character == '\n' || character == '\r')
        || !valid_driver_token(output)
    {
        return Err(MultipassInventoryError::InvalidDriverOutput);
    }

    Ok(output.to_owned())
}

fn valid_version_token(value: &str) -> bool {
    valid_label(value, 64)
        && value.is_ascii()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '+')
        })
}

fn valid_driver_token(value: &str) -> bool {
    valid_label(value, 64)
        && value.is_ascii()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_')
        })
}

async fn read_bounded<R>(
    mut reader: R,
    max_bytes: usize,
    output_limit_error: MultipassInventoryError,
) -> Result<Vec<u8>, MultipassInventoryError>
where
    R: AsyncRead + Unpin,
{
    let mut output = Vec::with_capacity(max_bytes.min(READ_CHUNK_BYTES));
    let mut chunk = [0_u8; READ_CHUNK_BYTES];
    loop {
        let room = max_bytes.saturating_add(1).saturating_sub(output.len());
        if room == 0 {
            return Err(output_limit_error);
        }
        let read_limit = room.min(chunk.len());
        let bytes_read = reader
            .read(&mut chunk[..read_limit])
            .await
            .map_err(MultipassInventoryError::Read)?;
        if bytes_read == 0 {
            return Ok(output);
        }
        output.extend_from_slice(&chunk[..bytes_read]);
        if output.len() > max_bytes {
            return Err(output_limit_error);
        }
    }
}

async fn stop_child(child: &mut Child) {
    let _ = child.kill().await;
    let _ = timeout(Duration::from_secs(1), child.wait()).await;
}

fn acquire_probe_permit() -> Result<tokio::sync::SemaphorePermit<'static>, MultipassInventoryError>
{
    MULTIPASS_PROBE_SINGLE_FLIGHT
        .get_or_init(|| Semaphore::new(1))
        .try_acquire()
        .map_err(|_| MultipassInventoryError::ProbeAlreadyRunning)
}

async fn run_multipass_command(
    executable: &std::path::Path,
    kind: MultipassReadOnlyCommand,
) -> Result<Vec<u8>, MultipassInventoryError> {
    let mut child = Command::new(executable)
        .args(kind.args())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(MultipassInventoryError::Spawn)?;

    let stdout = child
        .stdout
        .take()
        .ok_or(MultipassInventoryError::MissingStdoutPipe)?;
    let stderr = child
        .stderr
        .take()
        .ok_or(MultipassInventoryError::MissingStderrPipe)?;

    let result = timeout(PROBE_DEADLINE, async {
        let (stdout, _stderr) = tokio::try_join!(
            read_bounded(
                stdout,
                kind.max_stdout_bytes(),
                MultipassInventoryError::StdoutLimitExceeded,
            ),
            read_bounded(
                stderr,
                MAX_STDERR_BYTES,
                MultipassInventoryError::StderrLimitExceeded,
            ),
        )?;
        let status = child.wait().await.map_err(MultipassInventoryError::Wait)?;
        Ok::<_, MultipassInventoryError>((stdout, status))
    })
    .await;

    let (stdout, status) = match result {
        Ok(Ok(output)) => output,
        Ok(Err(error)) => {
            stop_child(&mut child).await;
            return Err(error);
        }
        Err(_) => {
            stop_child(&mut child).await;
            return Err(MultipassInventoryError::DeadlineExceeded);
        }
    };

    if !status.success() {
        return Err(MultipassInventoryError::CommandFailed(status.code()));
    }

    Ok(stdout)
}

#[cfg(test)]
mod tests {
    use super::{
        parse_multipass_driver, parse_multipass_list_json, parse_multipass_versions, read_bounded,
        MultipassCapabilityProbe, MultipassInstanceState, MultipassInventoryError,
        MultipassInventoryProbe, MultipassInventorySnapshot, MultipassReadOnlyCommand,
        MAX_CAPABILITY_OUTPUT_BYTES, MAX_INVENTORY_ROWS, MAX_STDOUT_BYTES,
        MULTIPASS_PROBE_SINGLE_FLIGHT,
    };
    use tokio::io::{duplex, AsyncWriteExt};

    #[test]
    fn deleted_instances_remain_in_inventory_count() {
        let snapshot = parse_multipass_list_json(
            br#"{"list":[{"name":"agent-a","state":"Stopped"},{"name":"agent-b","state":"Deleted"}]}"#,
        )
        .expect("documented instance rows should parse");

        assert_eq!(snapshot.registered_count(), 2);
        assert_eq!(snapshot.instances()[0].name(), "agent-a");
        assert_eq!(
            snapshot.instances()[1].state(),
            &MultipassInstanceState::Deleted
        );
    }

    #[test]
    fn official_map_inventory_is_parsed() {
        let snapshot = parse_multipass_list_json(
            br#"{"agent-b":[{"state":"Deleted","ipv4":[],"release":"Not Available"}],"agent-a":[{"state":"Running","ipv4":["10.0.0.2"],"release":"Ubuntu 24.04 LTS"}]}"#,
        )
        .expect("official name-to-record inventory should parse");

        assert_eq!(snapshot.registered_count(), 2);
        let running = snapshot
            .instances()
            .iter()
            .find(|instance| instance.name() == "agent-a")
            .expect("the named Running instance should be retained");
        assert_eq!(running.state(), &MultipassInstanceState::Running);
        let deleted = snapshot
            .instances()
            .iter()
            .find(|instance| instance.name() == "agent-b")
            .expect("the named Deleted instance should be retained");
        assert_eq!(deleted.state(), &MultipassInstanceState::Deleted);
    }

    #[test]
    fn empty_inventory_is_a_valid_observation() {
        let snapshot = MultipassInventorySnapshot::parse_json(br#"{"list":[]}"#)
            .expect("an empty list is a valid zero-instance observation");

        assert_eq!(snapshot.registered_count(), 0);
        assert!(snapshot.instances().is_empty());
    }

    #[test]
    fn public_inventory_parser_enforces_output_bound() {
        let bytes = vec![b' '; MAX_STDOUT_BYTES + 1];

        assert!(matches!(
            MultipassInventorySnapshot::parse_json(&bytes),
            Err(MultipassInventoryError::StdoutLimitExceeded)
        ));
    }

    #[test]
    fn malformed_inventory_fails_closed() {
        assert!(matches!(
            parse_multipass_list_json(br#"{"instances":null}"#),
            Err(MultipassInventoryError::InvalidJson)
        ));
        assert!(matches!(
            parse_multipass_list_json(br#"{"list":[{"name":"vm","state":5}]}"#),
            Err(MultipassInventoryError::InvalidJson)
        ));
        assert!(matches!(
            parse_multipass_list_json(br#"{"list":[{"name":"bad\nname","state":"Running"}]}"#),
            Err(MultipassInventoryError::InvalidInstance)
        ));
        assert!(matches!(
            parse_multipass_list_json(br#"{"vm":[]}"#),
            Err(MultipassInventoryError::InvalidInstance)
        ));
    }

    #[test]
    fn duplicate_instance_names_fail_closed() {
        let error = parse_multipass_list_json(
            br#"{"list":[{"name":"vm-a","state":"Running"},{"name":"vm-a","state":"Stopped"}]}"#,
        )
        .expect_err("ambiguous inventory must not be accepted");

        assert!(matches!(error, MultipassInventoryError::DuplicateInstance));
    }

    #[test]
    fn duplicate_map_keys_fail_closed() {
        let error = parse_multipass_list_json(
            br#"{"vm-a":[{"state":"Running"}],"vm-a":[{"state":"Stopped"}]}"#,
        )
        .expect_err("duplicate JSON object keys must not be silently overwritten");

        assert!(matches!(error, MultipassInventoryError::DuplicateInstance));
    }

    #[test]
    fn inventory_row_limit_is_enforced() {
        let rows = (0..=MAX_INVENTORY_ROWS)
            .map(|index| format!(r#"{{"name":"vm-{index}","state":"Stopped"}}"#))
            .collect::<Vec<_>>()
            .join(",");
        let json = format!(r#"{{"list":[{rows}]}}"#);

        let error = parse_multipass_list_json(json.as_bytes())
            .expect_err("unbounded row counts must be rejected");
        assert!(matches!(error, MultipassInventoryError::RowLimitExceeded));
    }

    #[test]
    fn unknown_state_is_preserved_as_unclassified() {
        let snapshot =
            parse_multipass_list_json(br#"{"list":[{"name":"vm-a","state":"Migrating"}]}"#)
                .expect("unknown states remain observable");

        assert_eq!(
            snapshot.instances()[0].state(),
            &MultipassInstanceState::Other("Migrating".to_owned())
        );
    }

    #[test]
    fn capability_commands_are_fixed_read_only_and_bounded() {
        assert_eq!(
            MultipassReadOnlyCommand::Inventory.args(),
            &["list", "--format", "json"]
        );
        assert_eq!(MultipassReadOnlyCommand::Version.args(), &["version"]);
        assert_eq!(
            MultipassReadOnlyCommand::LocalDriver.args(),
            &["get", "local.driver"]
        );
        assert_eq!(
            MultipassReadOnlyCommand::Version.max_stdout_bytes(),
            MAX_CAPABILITY_OUTPUT_BYTES
        );
        assert_eq!(
            MultipassReadOnlyCommand::LocalDriver.max_stdout_bytes(),
            MAX_CAPABILITY_OUTPUT_BYTES
        );
    }

    #[test]
    fn documented_version_output_is_parsed_without_classifying_compatibility() {
        let (client, daemon) = parse_multipass_versions(b"multipass 1.17.0\nmultipassd 1.16.2\n")
            .expect("the documented client/daemon report should parse");

        assert_eq!(client, "1.17.0");
        assert_eq!(daemon, "1.16.2");
    }

    #[test]
    fn malformed_or_ambiguous_version_output_fails_closed() {
        for bytes in [
            b"multipass 1.17.0\n".as_slice(),
            b"multipass 1.17.0\nmultipass 1.17.0\nmultipassd 1.17.0\n",
            b"multipass 1.17.0\nmultipassd 1.17.0 extra\n",
            b"multipass 1.17.0\nother 1.17.0\n",
            b"multipass 1.17.0\nmultipassd bad/version\n",
        ] {
            assert!(matches!(
                parse_multipass_versions(bytes),
                Err(MultipassInventoryError::InvalidVersionOutput)
            ));
        }
    }

    #[test]
    fn driver_probe_preserves_unknown_values_as_observations() {
        assert_eq!(
            parse_multipass_driver(b"future_driver-2\r\n")
                .expect("future driver names remain observable"),
            "future_driver-2"
        );
    }

    #[test]
    fn malformed_or_ambiguous_driver_output_fails_closed() {
        for bytes in [
            b"".as_slice(),
            b"  qemu\n",
            b"qemu\nvirtualbox\n",
            b"qemu/slash\n",
            b"qemu\0\n",
        ] {
            assert!(matches!(
                parse_multipass_driver(bytes),
                Err(MultipassInventoryError::InvalidDriverOutput)
            ));
        }
    }

    #[tokio::test]
    async fn stdout_bound_is_enforced() {
        let (mut writer, reader) = duplex(16);
        writer
            .write_all(b"12345")
            .await
            .expect("fixture pipe should accept the bounded payload");
        drop(writer);

        let error = read_bounded(reader, 4, MultipassInventoryError::StdoutLimitExceeded)
            .await
            .expect_err("bytes beyond the stdout cap must fail closed");
        assert!(matches!(
            error,
            MultipassInventoryError::StdoutLimitExceeded
        ));
    }

    #[tokio::test]
    async fn stderr_bound_uses_its_own_error() {
        let (mut writer, reader) = duplex(16);
        writer
            .write_all(b"12345")
            .await
            .expect("fixture pipe should accept the bounded payload");
        drop(writer);

        let error = read_bounded(reader, 4, MultipassInventoryError::StderrLimitExceeded)
            .await
            .expect_err("bytes beyond the stderr cap must fail closed");
        assert!(matches!(
            error,
            MultipassInventoryError::StderrLimitExceeded
        ));
    }

    #[tokio::test]
    async fn exact_output_bound_is_accepted() {
        let (mut writer, reader) = duplex(16);
        writer
            .write_all(b"1234")
            .await
            .expect("fixture pipe should accept the exact-limit payload");
        drop(writer);

        let output = read_bounded(reader, 4, MultipassInventoryError::StdoutLimitExceeded)
            .await
            .expect("the exact byte limit should be accepted");
        assert_eq!(output, b"1234");
    }

    #[test]
    fn relative_executable_path_is_rejected() {
        assert!(matches!(
            MultipassInventoryProbe::new("multipass.exe"),
            Err(MultipassInventoryError::ExecutablePathMustBeAbsolute)
        ));
    }

    #[tokio::test]
    async fn single_flight_is_shared_across_probe_instances() {
        let semaphore =
            MULTIPASS_PROBE_SINGLE_FLIGHT.get_or_init(|| tokio::sync::Semaphore::new(1));
        let _permit = semaphore
            .try_acquire()
            .expect("the fixture should acquire the process-wide probe gate");
        let executable = std::env::current_exe().expect("the test executable path is absolute");
        let first =
            MultipassInventoryProbe::new(executable.clone()).expect("absolute path accepted");
        let second = MultipassInventoryProbe::new(executable).expect("absolute path accepted");

        assert!(matches!(
            first.list_instances().await,
            Err(MultipassInventoryError::ProbeAlreadyRunning)
        ));
        assert!(matches!(
            second.list_instances().await,
            Err(MultipassInventoryError::ProbeAlreadyRunning)
        ));

        let capability =
            MultipassCapabilityProbe::new(std::env::current_exe().expect("absolute test path"))
                .expect("absolute path accepted");
        assert!(matches!(
            capability.probe().await,
            Err(MultipassInventoryError::ProbeAlreadyRunning)
        ));
    }
}
