/*
@cypher schema=1 source_sha256=b0edb419272bfbd5673c83860e90babc57a5c77d30fba0a8380628dee9e0d07e
MERGE (self:File {path:"crates/multipass-adapter/src/lib.rs"})
MERGE (module:Symbol {id:"crates/multipass-adapter/src/lib.rs::module", kind:"module"})
MERGE (probe:Type {id:"crates/multipass-adapter/src/lib.rs::MultipassInventoryProbe", kind:"struct"})
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
MERGE (snapshot_instances:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassInventorySnapshot.instances", kind:"method"})
MERGE (registered_count:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassInventorySnapshot.registered_count", kind:"method"})
MERGE (instance_name:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassInstance.name", kind:"method"})
MERGE (instance_state:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassInstance.state", kind:"method"})
MERGE (deserialize:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassNamedInstances.deserialize", kind:"method"})
MERGE (expecting:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassNamedInstances::NamedInstancesVisitor.expecting", kind:"method"})
MERGE (visit_map:Symbol {id:"crates/multipass-adapter/src/lib.rs::MultipassNamedInstances::NamedInstancesVisitor.visit_map", kind:"method"})
MERGE (parse:Symbol {id:"crates/multipass-adapter/src/lib.rs::parse_multipass_list_json", kind:"function"})
MERGE (valid:Symbol {id:"crates/multipass-adapter/src/lib.rs::valid_label", kind:"function"})
MERGE (read:Symbol {id:"crates/multipass-adapter/src/lib.rs::read_bounded", kind:"function"})
MERGE (stop:Symbol {id:"crates/multipass-adapter/src/lib.rs::stop_child", kind:"function"})
MERGE (max_stdout:Symbol {id:"crates/multipass-adapter/src/lib.rs::MAX_STDOUT_BYTES", kind:"constant"})
MERGE (max_stderr:Symbol {id:"crates/multipass-adapter/src/lib.rs::MAX_STDERR_BYTES", kind:"constant"})
MERGE (max_rows:Symbol {id:"crates/multipass-adapter/src/lib.rs::MAX_INVENTORY_ROWS", kind:"constant"})
MERGE (deadline:Symbol {id:"crates/multipass-adapter/src/lib.rs::PROBE_DEADLINE", kind:"constant"})
MERGE (read_chunk:Symbol {id:"crates/multipass-adapter/src/lib.rs::READ_CHUNK_BYTES", kind:"constant"})
MERGE (max_name:Symbol {id:"crates/multipass-adapter/src/lib.rs::MAX_INSTANCE_NAME_BYTES", kind:"constant"})
MERGE (max_state:Symbol {id:"crates/multipass-adapter/src/lib.rs::MAX_STATE_BYTES", kind:"constant"})
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
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(probe)
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
MERGE (module)-[:DEFINES]->(new)
MERGE (module)-[:DEFINES]->(parse)
MERGE (module)-[:DEFINES]->(valid)
MERGE (module)-[:DEFINES]->(read)
MERGE (module)-[:DEFINES]->(stop)
MERGE (module)-[:DEFINES]->(max_stdout)
MERGE (module)-[:DEFINES]->(max_stderr)
MERGE (module)-[:DEFINES]->(max_rows)
MERGE (module)-[:DEFINES]->(deadline)
MERGE (module)-[:DEFINES]->(read_chunk)
MERGE (module)-[:DEFINES]->(max_name)
MERGE (module)-[:DEFINES]->(max_state)
MERGE (module)-[:DEFINES]->(single_flight)
MERGE (module)-[:DEFINES]->(tests)
MERGE (probe)-[:DEFINES]->(list)
MERGE (snapshot)-[:DEFINES]->(snapshot_instances)
MERGE (snapshot)-[:DEFINES]->(registered_count)
MERGE (instance)-[:DEFINES]->(instance_name)
MERGE (instance)-[:DEFINES]->(instance_state)
MERGE (named_instances)-[:DEFINES]->(deserialize)
MERGE (visitor)-[:DEFINES]->(expecting)
MERGE (visitor)-[:DEFINES]->(visit_map)
MERGE (list)-[:CALLS]->(read)
MERGE (list)-[:USES_TYPE]->(max_stdout)
MERGE (list)-[:USES_TYPE]->(max_stderr)
MERGE (list)-[:USES_TYPE]->(deadline)
MERGE (list)-[:READS]->(single_flight)
MERGE (list)-[:CALLS]->(stop)
MERGE (list)-[:CALLS]->(parse)
MERGE (parse)-[:USES_TYPE]->(document)
MERGE (parse)-[:USES_TYPE]->(max_rows)
MERGE (parse)-[:USES_TYPE]->(max_name)
MERGE (parse)-[:USES_TYPE]->(max_state)
MERGE (parse)-[:CALLS]->(valid)
MERGE (deserialize)-[:CALLS]->(visit_map)
MERGE (read)-[:USES_TYPE]->(read_chunk)
MERGE (tests)-[:CALLS]->(parse)
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
MERGE (single_flight_test)-[:CALLS]->(new)
MERGE (single_flight_test)-[:CALLS]->(list)
MERGE (single_flight_test)-[:READS]->(single_flight)
@endcypher
*/

//! Bounded, read-only observation of the local Multipass instance inventory.
//!
//! This crate is only a process adapter. It does not establish instance ownership,
//! HostEnvironment or cluster identity, capacity, daemon readiness, guest readiness,
//! or authorization. It deliberately exposes no lifecycle or guest-execution command.

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
        let _permit = MULTIPASS_PROBE_SINGLE_FLIGHT
            .get_or_init(|| Semaphore::new(1))
            .try_acquire()
            .map_err(|_| MultipassInventoryError::ProbeAlreadyRunning)?;

        let mut child = Command::new(&self.executable)
            .arg("list")
            .arg("--format")
            .arg("json")
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
                    MAX_STDOUT_BYTES,
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

        parse_multipass_list_json(&stdout)
    }
}

/// A complete bounded snapshot from `multipass list --format json`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultipassInventorySnapshot {
    instances: Vec<MultipassInstance>,
}

impl MultipassInventorySnapshot {
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
    /// A previous probe is still running.
    #[error("Multipass inventory probe is already running")]
    ProbeAlreadyRunning,
    /// The operating system could not start Multipass.
    #[error("could not start Multipass inventory command: {0}")]
    Spawn(#[source] std::io::Error),
    /// The child did not expose the expected stdout pipe.
    #[error("Multipass inventory stdout pipe is unavailable")]
    MissingStdoutPipe,
    /// The child did not expose the expected stderr pipe.
    #[error("Multipass inventory stderr pipe is unavailable")]
    MissingStderrPipe,
    /// Multipass exited unsuccessfully; raw stderr is intentionally not retained.
    #[error("Multipass inventory command exited unsuccessfully (code {0:?})")]
    CommandFailed(Option<i32>),
    /// The command did not finish within its fixed deadline.
    #[error("Multipass inventory command exceeded its 5 second deadline")]
    DeadlineExceeded,
    /// The operating system reported an output-pipe read failure.
    #[error("could not read Multipass inventory output: {0}")]
    Read(#[source] std::io::Error),
    /// The child process could not be reaped.
    #[error("could not wait for Multipass inventory command: {0}")]
    Wait(#[source] std::io::Error),
    /// Multipass produced more stdout than the configured bound.
    #[error("Multipass inventory stdout exceeded its byte limit")]
    StdoutLimitExceeded,
    /// Multipass produced more stderr than the configured bound.
    #[error("Multipass inventory stderr exceeded its byte limit")]
    StderrLimitExceeded,
    /// The JSON response could not be decoded.
    #[error("Multipass inventory response is not valid JSON")]
    InvalidJson,
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
        MultipassListDocument::Rows(rows) => {
            if rows.list.is_empty() {
                return Err(MultipassInventoryError::InvalidInstance);
            }
            rows.list
        }
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

#[cfg(test)]
mod tests {
    use super::{
        parse_multipass_list_json, read_bounded, MultipassInstanceState, MultipassInventoryError,
        MultipassInventoryProbe, MAX_INVENTORY_ROWS, MULTIPASS_PROBE_SINGLE_FLIGHT,
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
        assert!(matches!(
            parse_multipass_list_json(br#"{"list":[]}"#),
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
    }
}
