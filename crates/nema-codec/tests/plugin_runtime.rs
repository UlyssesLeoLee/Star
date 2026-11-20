/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"crates/nema-codec/tests/plugin_runtime.rs"})
MERGE (runtime:File {path:"crates/nema-codec/src/plugin_runtime.rs"})
MERGE (plugin:File {path:"crates/nema-codec/src/interpreter_plugin.rs"})
MERGE (four_states:Symbol {id:"crates/nema-codec/tests/plugin_runtime.rs::four_value_states_round_trip_v1_and_v2",kind:"test"})
MERGE (error_paths:Symbol {id:"crates/nema-codec/tests/plugin_runtime.rs::error_paths_report_clear_messages",kind:"test"})
MERGE (parallel:Symbol {id:"crates/nema-codec/tests/plugin_runtime.rs::v1_v2_and_third_profile_run_in_parallel",kind:"test"})
MERGE (lifecycle:Symbol {id:"crates/nema-codec/tests/plugin_runtime.rs::install_lifecycle_abi_digest_and_upgrade",kind:"test"})
MERGE (migration:Symbol {id:"crates/nema-codec/tests/plugin_runtime.rs::migration_preview_preserves_original_and_executes_explicitly",kind:"test"})
MERGE (examples:Symbol {id:"crates/nema-codec/tests/plugin_runtime.rs::examples_conform_to_pinned_snapshot",kind:"test"})
MERGE (self)-[:DEFINES]->(four_states)
MERGE (self)-[:DEFINES]->(error_paths)
MERGE (self)-[:DEFINES]->(parallel)
MERGE (self)-[:DEFINES]->(lifecycle)
MERGE (self)-[:DEFINES]->(migration)
MERGE (self)-[:DEFINES]->(examples)
MERGE (self)-[:IMPORTS]->(runtime)
MERGE (self)-[:IMPORTS]->(plugin)
MERGE (four_states)-[:TESTS]->(runtime)
MERGE (error_paths)-[:TESTS]->(runtime)
MERGE (parallel)-[:TESTS]->(runtime)
MERGE (lifecycle)-[:TESTS]->(runtime)
MERGE (migration)-[:TESTS]->(runtime)
@endcypher
*/
//! Integration tests for the isolated NEMA plugin runtime: value round trips,
//! error paths, parallel profiles, installation lifecycle, migration, and
//! conformance of the docs/nema examples against their expected ASTs.

use nema_codec::interpreter_plugin::{
    bundled_profile_for_revision, InterpreterError, InterpreterProfile, NemaInterpreterRegistry,
    INTERPRETER_ABI_V1, NEMA_AST_SCHEMA_V1, NEMA_STRICT_V1_REVISION, NEMA_STRICT_V2_REVISION,
};
use nema_codec::plugin_runtime::{
    FormatRequest, FormatResponse, NemaFileBinding, NemaPluginHost, NemaPluginInstaller,
    ParseRequest, ParseResponse, PluginArtifact, ResourceLimits, WorkerLaunch,
    NEMA_PLUGIN_PROTOCOL_VERSION,
};
use nema_codec::strict::{format_strict_v2, parse_strict_v2, Cell, Document};
use std::sync::Arc;

/// In-memory protocol host: a *real* worker-protocol round trip without an
/// isolation boundary. This proves request/response encoding and AST
/// validation; process isolation itself is tested in sandboxd (nema_worker).
struct ScriptedV2Host;

impl NemaPluginHost for ScriptedV2Host {
    fn execute(
        &self,
        _launch: &WorkerLaunch,
        request: &[u8],
        _limits: &ResourceLimits,
    ) -> Result<Vec<u8>, InterpreterError> {
        if let Ok(parse) = serde_json::from_slice::<ParseRequest>(request) {
            let response = match parse_strict_v2(&parse.source) {
                Ok(doc) => ParseResponse {
                    protocol_version: NEMA_PLUGIN_PROTOCOL_VERSION,
                    ast: Some(serde_json::to_value(&doc).unwrap()),
                    error: None,
                },
                Err(error) => ParseResponse {
                    protocol_version: NEMA_PLUGIN_PROTOCOL_VERSION,
                    ast: None,
                    error: Some(nema_codec::plugin_runtime::PluginErrorPayload {
                        code: "syntax".to_string(),
                        detail: error.to_string(),
                    }),
                },
            };
            return serde_json::to_vec(&response).map_err(|e| {
                InterpreterError::ProtocolViolation {
                    detail: format!("scripted host: {e}"),
                }
            });
        }
        if let Ok(format) = serde_json::from_slice::<FormatRequest>(request) {
            let document: Document = serde_json::from_value(format.ast).unwrap();
            let response = match format_strict_v2(&document) {
                Ok(text) => FormatResponse {
                    protocol_version: NEMA_PLUGIN_PROTOCOL_VERSION,
                    text: Some(text),
                    error: None,
                },
                Err(error) => FormatResponse {
                    protocol_version: NEMA_PLUGIN_PROTOCOL_VERSION,
                    text: None,
                    error: Some(nema_codec::plugin_runtime::PluginErrorPayload {
                        code: "format".to_string(),
                        detail: error.to_string(),
                    }),
                },
            };
            return serde_json::to_vec(&response).map_err(|e| {
                InterpreterError::ProtocolViolation {
                    detail: format!("scripted host: {e}"),
                }
            });
        }
        Err(InterpreterError::ProtocolViolation {
            detail: "scripted host received an unknown request kind".to_string(),
        })
    }
}

/// Host that always reports a worker-side timeout (resource budget breach).
struct TimeoutHost;

impl NemaPluginHost for TimeoutHost {
    fn execute(
        &self,
        _launch: &WorkerLaunch,
        _request: &[u8],
        _limits: &ResourceLimits,
    ) -> Result<Vec<u8>, InterpreterError> {
        let response = ParseResponse {
            protocol_version: NEMA_PLUGIN_PROTOCOL_VERSION,
            ast: None,
            error: Some(nema_codec::plugin_runtime::PluginErrorPayload {
                code: "timeout".to_string(),
                detail: "cpu budget exhausted".to_string(),
            }),
        };
        serde_json::to_vec(&response).map_err(|e| InterpreterError::ProtocolViolation {
            detail: format!("timeout host: {e}"),
        })
    }
}

/// Host that returns an oversized raw response (output limit breach).
struct OversizedHost;

impl NemaPluginHost for OversizedHost {
    fn execute(
        &self,
        _launch: &WorkerLaunch,
        _request: &[u8],
        _limits: &ResourceLimits,
    ) -> Result<Vec<u8>, InterpreterError> {
        Ok(vec![b'x'; 16 * 1024 * 1024 + 1])
    }
}

fn third_party_profile(plugin_version: &str) -> InterpreterProfile {
    InterpreterProfile {
        format_id: "nema".into(),
        plugin_id: "org.star.nema.demoplugin".into(),
        grammar_revision: NEMA_STRICT_V2_REVISION.into(),
        plugin_version: plugin_version.into(),
        abi_version: INTERPRETER_ABI_V1,
        ast_schema_version: NEMA_AST_SCHEMA_V1,
        grammar_source_sha256: "a".repeat(64),
        artifact_sha256: "b".repeat(64),
    }
}

fn launch() -> WorkerLaunch {
    WorkerLaunch {
        program: "nema-worker".to_string(),
        args: Vec::new(),
        env: Vec::new(),
        cwd: String::new(),
    }
}

const V1_GRAPH: &str = " \\>1 1\nA LoginAPI\n V2\n";
const V2_GRAPH: &str = " \\>1 1\nA LoginAPI\n 1 V2\n";
/// v1-only: two-space indent continuation (v2 forbids consecutive spaces).
const V1_ONLY: &str = " \\>1 1\nA LoginAPI\n  V2\n";
const FOUR_STATES: &str = " \\1 1\nA ~ \\e \\_\n";

#[test]
fn four_value_states_round_trip_v1_and_v2() {
    let registry = NemaInterpreterRegistry::bundled().unwrap();
    let v1 = bundled_profile_for_revision(NEMA_STRICT_V1_REVISION).unwrap();
    let v2 = bundled_profile_for_revision(NEMA_STRICT_V2_REVISION).unwrap();
    for profile in [&v1, &v2] {
        let document = registry.parse(profile, FOUR_STATES).unwrap();
        assert_eq!(
            document.domains[0].records[0],
            vec![
                Cell::Value("A".into()),
                Cell::Null,
                Cell::Empty,
                Cell::Missing
            ],
            "profile {} must keep the four states distinct",
            profile.grammar_revision
        );
        // Null / Empty / Missing / Value are pairwise distinct.
        let cells = &document.domains[0].records[0];
        assert_ne!(cells[0], cells[1]);
        assert_ne!(cells[0], cells[2]);
        assert_ne!(cells[0], cells[3]);
        assert_ne!(cells[1], cells[2]);
        assert_ne!(cells[1], cells[3]);
        assert_ne!(cells[2], cells[3]);
        // parse → format → parse must reproduce the identical AST.
        let formatted = registry.format(profile, &document).unwrap();
        let reparsed = registry.parse(profile, &formatted).unwrap();
        assert_eq!(reparsed, document);
    }
}

#[test]
fn error_paths_report_clear_messages() {
    let registry = NemaInterpreterRegistry::bundled().unwrap();
    let v2 = bundled_profile_for_revision(NEMA_STRICT_V2_REVISION).unwrap();

    // Unknown profile: no fallback, explicit error.
    let unknown = third_party_profile("9.9.9");
    let unknown_error = registry.parse(&unknown, V2_GRAPH).unwrap_err();
    assert_eq!(unknown_error, InterpreterError::UnregisteredProfile);
    assert!(
        unknown_error.to_string().contains("not installed"),
        "message must be explicit: {unknown_error}"
    );

    // Input above MAX_BYTES.
    let oversized_source = " ".repeat(16 * 1024 * 1024 + 1);
    assert_eq!(
        registry.parse(&v2, &oversized_source).unwrap_err(),
        InterpreterError::InputTooLarge
    );

    // Caller-supplied limit below the default: ResourceLimit with key info.
    let limits = ResourceLimits {
        max_input_bytes: 8,
        ..ResourceLimits::default()
    };
    let limited_error = registry.parse_isolated(&v2, V2_GRAPH, &limits).unwrap_err();
    match limited_error {
        InterpreterError::ResourceLimit {
            resource,
            limit,
            actual,
        } => {
            assert_eq!(resource, "input_bytes");
            assert_eq!(limit, 8);
            assert!(actual > 8);
        }
        other => panic!("expected ResourceLimit, got {other:?}"),
    }

    // ABI incompatible profile fails closed at validation.
    let mut bad_abi = third_party_profile("1.0.0");
    bad_abi.abi_version = INTERPRETER_ABI_V1 + 1;
    assert_eq!(
        bad_abi.validate(),
        Err(InterpreterError::UnsupportedAbi(INTERPRETER_ABI_V1 + 1))
    );
    assert!(
        bad_abi
            .validate()
            .unwrap_err()
            .to_string()
            .contains("unsupported interpreter ABI 2"),
        "ABI error must name the version"
    );

    // Digest mismatch at registration: explicit expected/actual hashes.
    let mut tampered = PluginArtifact::from_payload(b"demo-worker".to_vec());
    tampered.payload = b"demo-workar".to_vec();
    let digest_error = NemaInterpreterRegistry::default()
        .register_isolated(
            third_party_profile("1.0.0"),
            &tampered,
            launch(),
            Arc::new(ScriptedV2Host),
        )
        .unwrap_err();
    match digest_error {
        InterpreterError::DigestMismatch { expected, actual } => {
            assert_ne!(expected, actual);
            assert!(expected.len() == 64 && actual.len() == 64);
        }
        other => panic!("expected DigestMismatch, got {other:?}"),
    }

    // Worker-reported timeout maps to ResourceLimit with the worker code.
    let mut timeout_registry = NemaInterpreterRegistry::default();
    timeout_registry
        .register_isolated(
            third_party_profile("1.0.0"),
            &PluginArtifact::from_payload(b"demo-worker".to_vec()),
            launch(),
            Arc::new(TimeoutHost),
        )
        .unwrap();
    let timeout_error = timeout_registry
        .parse(&third_party_profile("1.0.0"), V2_GRAPH)
        .unwrap_err();
    assert!(matches!(
        timeout_error,
        InterpreterError::ResourceLimit { ref resource, .. } if resource == "timeout"
    ));
    assert!(timeout_error.to_string().contains("timeout"));

    // Oversized worker response is rejected before parsing.
    let mut oversized_registry = NemaInterpreterRegistry::default();
    oversized_registry
        .register_isolated(
            third_party_profile("1.0.0"),
            &PluginArtifact::from_payload(b"demo-worker".to_vec()),
            launch(),
            Arc::new(OversizedHost),
        )
        .unwrap();
    let oversized_error = oversized_registry
        .parse(&third_party_profile("1.0.0"), V2_GRAPH)
        .unwrap_err();
    assert!(matches!(
        oversized_error,
        InterpreterError::ResourceLimit { ref resource, .. } if resource == "output_bytes"
    ));
}

#[test]
fn v1_v2_and_third_profile_run_in_parallel() {
    let mut registry = NemaInterpreterRegistry::bundled().unwrap();
    let third = third_party_profile("1.0.0");
    registry
        .register_isolated(
            third.clone(),
            &PluginArtifact::from_payload(b"demo-worker".to_vec()),
            launch(),
            Arc::new(ScriptedV2Host),
        )
        .unwrap();
    let v1 = bundled_profile_for_revision(NEMA_STRICT_V1_REVISION).unwrap();
    let v2 = bundled_profile_for_revision(NEMA_STRICT_V2_REVISION).unwrap();
    assert_eq!(registry.profiles().len(), 3);

    // Each file keeps its own interpreter; the three adapters never leak.
    let v1_doc = registry.parse(&v1, V1_ONLY).unwrap();
    assert_eq!(
        v1_doc.domains[0].records[0],
        vec![
            Cell::Value("A".into()),
            Cell::Value("LoginAPI".into()),
            Cell::Missing,
            Cell::Value("V2".into())
        ]
    );
    let v2_doc = registry.parse(&v2, V2_GRAPH).unwrap();
    assert_eq!(
        v2_doc.domains[0].records[0],
        vec![Cell::Value("A".into()), Cell::Value("LoginAPIV2".into())]
    );
    let third_doc = registry.parse(&third, V2_GRAPH).unwrap();
    assert_eq!(third_doc, v2_doc);

    // Format each under its own profile, re-parse, verify independence.
    for (profile, document) in [(&v1, &v1_doc), (&v2, &v2_doc), (&third, &third_doc)] {
        let formatted = registry.format(profile, document).unwrap();
        assert_eq!(registry.parse(profile, &formatted).unwrap(), *document);
    }
    // v1-only indent continuation is rejected by v2 and vice versa.
    assert!(registry.parse(&v2, V1_ONLY).is_err());
    assert!(registry.parse(&v1, V2_GRAPH).is_err());
}

#[test]
fn install_lifecycle_abi_digest_and_upgrade() {
    let host: Arc<dyn NemaPluginHost> = Arc::new(ScriptedV2Host);
    let mut installer = NemaPluginInstaller::new(NemaInterpreterRegistry::default(), host);
    let profile = third_party_profile("1.0.0");
    let artifact = PluginArtifact::from_payload(b"demo-worker-v1".to_vec());

    // Not installed yet → explicit error, never a built-in fallback.
    assert_eq!(
        installer.registry().parse(&profile, V2_GRAPH).unwrap_err(),
        InterpreterError::UnregisteredProfile
    );

    // ABI-incompatible profile is refused at install (fail closed).
    let mut bad_abi = profile.clone();
    bad_abi.abi_version = INTERPRETER_ABI_V1 + 1;
    let abi_error = installer
        .install_profile(bad_abi, artifact.clone(), launch())
        .unwrap_err();
    assert!(matches!(abi_error, InterpreterError::UnsupportedAbi(2)));

    // Digest mismatch is refused at install.
    let mut tampered = artifact.clone();
    tampered.payload = b"demo-worker-vO".to_vec();
    let digest_error = installer
        .install_profile(profile.clone(), tampered, launch())
        .unwrap_err();
    assert!(matches!(
        digest_error,
        InterpreterError::DigestMismatch { .. }
    ));
    assert!(digest_error.to_string().contains("digest mismatch"));

    // Clean install → activate → parse through the isolated protocol path.
    installer
        .install_profile(profile.clone(), artifact.clone(), launch())
        .unwrap();
    assert_eq!(
        installer
            .install_profile(profile.clone(), artifact.clone(), launch())
            .unwrap_err(),
        InterpreterError::AlreadyInstalled("org.star.nema.demoplugin@1.0.0".to_string())
    );
    installer
        .activate("org.star.nema.demoplugin", "1.0.0")
        .unwrap();
    let parsed = installer.registry().parse(&profile, V2_GRAPH).unwrap();
    assert_eq!(
        parsed.domains[0].records[0],
        vec![Cell::Value("A".into()), Cell::Value("LoginAPIV2".into())]
    );
    let formatted = installer.registry().format(&profile, &parsed).unwrap();
    assert_eq!(
        installer.registry().parse(&profile, &formatted).unwrap(),
        parsed
    );

    // Upgrade: install v2 alongside the active v1 — old stays registered until uninstall.
    let upgraded = third_party_profile("2.0.0");
    installer
        .upgrade(
            "org.star.nema.demoplugin",
            "1.0.0",
            upgraded.clone(),
            PluginArtifact::from_payload(b"demo-worker-v2".to_vec()),
            launch(),
        )
        .unwrap();
    assert_eq!(
        installer.state("org.star.nema.demoplugin", "1.0.0"),
        Some(nema_codec::plugin_runtime::PluginInstallState::Draining)
    );
    // Old version keeps serving its pinned files during drain.
    let old_doc = installer.registry().parse(&profile, V2_GRAPH).unwrap();
    assert_eq!(old_doc.domains[0].records[0].len(), 2);
    installer
        .complete_drain("org.star.nema.demoplugin", "1.0.0")
        .unwrap();
    installer
        .uninstall("org.star.nema.demoplugin", "1.0.0")
        .unwrap();
    // The uninstalled old version is unregistered again — explicit, no fallback.
    assert_eq!(
        installer.registry().parse(&profile, V2_GRAPH).unwrap_err(),
        InterpreterError::UnregisteredProfile
    );

    // Activate the new version; drain semantics with an outstanding lease.
    installer
        .activate("org.star.nema.demoplugin", "2.0.0")
        .unwrap();
    let new_doc = installer.registry().parse(&upgraded, V2_GRAPH).unwrap();
    assert_eq!(new_doc, old_doc);
    let lease = installer
        .admit("org.star.nema.demoplugin", "2.0.0")
        .unwrap();
    installer
        .begin_drain("org.star.nema.demoplugin", "2.0.0")
        .unwrap();
    assert!(matches!(
        installer.complete_drain("org.star.nema.demoplugin", "2.0.0"),
        Err(InterpreterError::InvalidLifecycle { .. })
    ));
    installer.release_lease(lease).unwrap();
    installer
        .complete_drain("org.star.nema.demoplugin", "2.0.0")
        .unwrap();
    assert_eq!(
        installer.versions("org.star.nema.demoplugin"),
        vec!["1.0.0", "2.0.0"]
    );
}

#[test]
fn migration_preview_preserves_original_and_executes_explicitly() {
    let registry = NemaInterpreterRegistry::bundled().unwrap();
    let v1 = bundled_profile_for_revision(NEMA_STRICT_V1_REVISION).unwrap();
    let v2 = bundled_profile_for_revision(NEMA_STRICT_V2_REVISION).unwrap();
    let source = V1_GRAPH;

    // Old file keeps its v1 interpreter; bind it.
    let binding = NemaFileBinding::new(v1.clone(), source).unwrap();
    let original_doc = registry.parse(&v1, source).unwrap();
    assert_eq!(original_doc.domains[0].records[0].len(), 3);

    // Preview to v2: original preserved, structural summary present.
    let preview = registry.preview_migration(&binding, source, &v2).unwrap();
    assert!(preview.original_preserved);
    assert!(preview.source_reparses_under_target);
    assert!(preview.diff_summary.contains("records="));
    assert!(preview.diff_summary.contains("cells="));
    let new_ast: Document = serde_json::from_value(preview.new_ast.clone().unwrap()).unwrap();
    assert_eq!(new_ast, original_doc);
    // The file itself is untouched by the preview.
    assert_eq!(registry.parse(&v1, source).unwrap(), original_doc);

    // Explicit execution returns the original copy and a new binding.
    let executed = registry.execute_migration(&binding, source, &v2).unwrap();
    assert_eq!(executed.original_source, source);
    assert_eq!(executed.new_binding.profile, v2);
    let migrated_doc = registry.parse(&v2, &executed.formatted_text).unwrap();
    assert_eq!(migrated_doc, original_doc);
    // The new binding verifies the new text and rejects the old text.
    assert!(executed
        .new_binding
        .verify(&executed.formatted_text)
        .is_ok());
    assert!(matches!(
        executed.new_binding.verify(source),
        Err(InterpreterError::DigestMismatch { .. })
    ));

    // Migrating to the same profile is refused.
    assert!(matches!(
        registry.preview_migration(&binding, source, &v1),
        Err(InterpreterError::MigrationRefused { .. })
    ));

    // Subset direction: a v2 indexed continuation cannot parse under v1 as-is,
    // but the canonical text can — the preview must say so explicitly.
    let v2_source = V2_GRAPH;
    let v2_binding = NemaFileBinding::new(v2.clone(), v2_source).unwrap();
    let preview_down = registry
        .preview_migration(&v2_binding, v2_source, &v1)
        .unwrap();
    assert!(!preview_down.source_reparses_under_target);
    let down_ast: Document = serde_json::from_value(preview_down.new_ast.unwrap()).unwrap();
    assert_eq!(down_ast, registry.parse(&v2, v2_source).unwrap());
}

macro_rules! example {
    ($name:literal) => {
        (
            $name,
            include_str!(concat!("../../../docs/nema/examples/", $name, ".nema")),
            include_str!(concat!(
                "../../../docs/nema/examples/expected/",
                $name,
                ".ast.json"
            )),
        )
    };
}

#[test]
fn examples_conform_to_pinned_snapshot() {
    let registry = NemaInterpreterRegistry::bundled().unwrap();
    let v1 = bundled_profile_for_revision(NEMA_STRICT_V1_REVISION).unwrap();
    let v2 = bundled_profile_for_revision(NEMA_STRICT_V2_REVISION).unwrap();
    let cases: &[(&str, &InterpreterProfile)] = &[
        ("v1-values", &v1),
        ("v1-null", &v1),
        ("v1-empty", &v1),
        ("v1-missing", &v1),
        ("v1-graph-continuation", &v1),
        ("v2-values", &v2),
        ("v2-null", &v2),
        ("v2-empty", &v2),
        ("v2-missing", &v2),
        ("v2-graph-continuation", &v2),
    ];
    for (name, profile) in cases {
        let (_, source, expected_json) = example_values(*name);
        let expected: serde_json::Value = serde_json::from_str(expected_json).unwrap();
        let document = registry.parse(profile, source).unwrap();
        assert_eq!(
            serde_json::to_value(&document).unwrap(),
            expected,
            "example {name}: parsed AST differs from the pinned expectation"
        );
        let formatted = registry.format(profile, &document).unwrap();
        assert_eq!(
            registry.parse(profile, &formatted).unwrap(),
            document,
            "example {name}: format must round trip"
        );
    }
}

fn example_values(name: &str) -> (&'static str, &'static str, &'static str) {
    match name {
        "v1-values" => example!("v1-values"),
        "v1-null" => example!("v1-null"),
        "v1-empty" => example!("v1-empty"),
        "v1-missing" => example!("v1-missing"),
        "v1-graph-continuation" => example!("v1-graph-continuation"),
        "v2-values" => example!("v2-values"),
        "v2-null" => example!("v2-null"),
        "v2-empty" => example!("v2-empty"),
        "v2-missing" => example!("v2-missing"),
        "v2-graph-continuation" => example!("v2-graph-continuation"),
        _ => panic!("unknown example {name}"),
    }
}
