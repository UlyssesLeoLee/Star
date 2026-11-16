/*
@cypher schema=1 source_sha256=a54ed838e760e98c413801d88caa96f8e23808659ce799c84bc76adcf257e409
MERGE (self:File {path:"crates/nema-codec/src/interpreter_plugin.rs"})
MERGE (lib:File {path:"crates/nema-codec/src/strict.rs"})
MERGE (profile:Type {id:"crates/nema-codec/src/interpreter_plugin.rs::InterpreterProfile"})
MERGE (adapter:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::NemaInterpreterAdapter",kind:"trait"})
MERGE (adapter_parse:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::NemaInterpreterAdapter.parse",kind:"method"})
MERGE (adapter_format:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::NemaInterpreterAdapter.format",kind:"method"})
MERGE (registry:Type {id:"crates/nema-codec/src/interpreter_plugin.rs::NemaInterpreterRegistry"})
MERGE (strict:Type {id:"crates/nema-codec/src/interpreter_plugin.rs::StrictV1Interpreter"})
MERGE (strict_v2:Type {id:"crates/nema-codec/src/interpreter_plugin.rs::StrictV2Interpreter"})
MERGE (error:Type {id:"crates/nema-codec/src/interpreter_plugin.rs::InterpreterError"})
MERGE (parse_v1:Symbol {id:"crates/nema-codec/src/strict.rs::parse_strict_v1",kind:"function"})
MERGE (parse_v2:Symbol {id:"crates/nema-codec/src/strict.rs::parse_strict_v2",kind:"function"})
MERGE (format_v1:Symbol {id:"crates/nema-codec/src/strict.rs::format_strict_v1",kind:"function"})
MERGE (format_v2:Symbol {id:"crates/nema-codec/src/strict.rs::format_strict_v2",kind:"function"})
MERGE (document:Type {id:"crates/nema-codec/src/strict.rs::Document"})
MERGE (parser_error:Type {id:"crates/nema-codec/src/strict.rs::Error"})
MERGE (register:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::NemaInterpreterRegistry.register",kind:"method"})
MERGE (dispatch_parse:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::NemaInterpreterRegistry.parse",kind:"method"})
MERGE (dispatch_format:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::NemaInterpreterRegistry.format",kind:"method"})
MERGE (validate:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::InterpreterProfile.validate",kind:"method"})
MERGE (valid_digest:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::valid_sha256",kind:"function"})
MERGE (budget:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::validate_document_budget",kind:"function"})
MERGE (profiles:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::NemaInterpreterRegistry.profiles",kind:"method"})
MERGE (bundled:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::NemaInterpreterRegistry.bundled",kind:"method"})
MERGE (bundled_profile:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::bundled_profile_for_revision",kind:"function"})
MERGE (strict_parse:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::StrictV1Interpreter.parse",kind:"method"})
MERGE (strict_format:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::StrictV1Interpreter.format",kind:"method"})
MERGE (strict_v2_parse:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::StrictV2Interpreter.parse",kind:"method"})
MERGE (strict_v2_format:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::StrictV2Interpreter.format",kind:"method"})
MERGE (tests:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::tests",kind:"module"})
MERGE (profiles_test:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::tests.v1_and_v2_profiles_route_graph_continuations_to_the_pinned_parser",kind:"test"})
MERGE (missing_test:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::tests.v2_continuation_rejects_explicit_missing_but_fills_implicit_missing",kind:"test"})
MERGE (bundled_test:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::tests.bundled_registry_keeps_exact_grammar_profiles_available",kind:"test"})
MERGE (self)-[:IMPORTS]->(lib)
MERGE (self)-[:DEFINES]->(profile)
MERGE (self)-[:DEFINES]->(adapter)
MERGE (self)-[:DEFINES]->(adapter_parse)
MERGE (self)-[:DEFINES]->(adapter_format)
MERGE (self)-[:DEFINES]->(registry)
MERGE (self)-[:DEFINES]->(strict)
MERGE (self)-[:DEFINES]->(strict_v2)
MERGE (self)-[:DEFINES]->(error)
MERGE (self)-[:DEFINES]->(register)
MERGE (self)-[:DEFINES]->(dispatch_parse)
MERGE (self)-[:DEFINES]->(dispatch_format)
MERGE (self)-[:DEFINES]->(validate)
MERGE (self)-[:DEFINES]->(valid_digest)
MERGE (self)-[:DEFINES]->(budget)
MERGE (self)-[:DEFINES]->(profiles)
MERGE (self)-[:DEFINES]->(bundled)
MERGE (self)-[:DEFINES]->(bundled_profile)
MERGE (self)-[:DEFINES]->(strict_parse)
MERGE (self)-[:DEFINES]->(strict_format)
MERGE (self)-[:DEFINES]->(strict_v2_parse)
MERGE (self)-[:DEFINES]->(strict_v2_format)
MERGE (self)-[:DEFINES]->(tests)
MERGE (self)-[:DEFINES]->(profiles_test)
MERGE (self)-[:DEFINES]->(missing_test)
MERGE (self)-[:DEFINES]->(bundled_test)
MERGE (adapter)-[:USES_TYPE]->(profile)
MERGE (adapter)-[:USES_TYPE]->(document)
MERGE (adapter)-[:USES_TYPE]->(error)
MERGE (registry)-[:USES_TYPE]->(profile)
MERGE (registry)-[:USES_TYPE]->(adapter)
MERGE (registry)-[:USES_TYPE]->(error)
MERGE (strict)-[:IMPLEMENTS]->(adapter)
MERGE (strict_v2)-[:IMPLEMENTS]->(adapter)
MERGE (register)-[:CALLS]->(validate)
MERGE (dispatch_parse)-[:CALLS]->(validate)
MERGE (dispatch_parse)-[:CALLS]->(adapter_parse)
MERGE (dispatch_parse)-[:CALLS]->(budget)
MERGE (dispatch_format)-[:CALLS]->(validate)
MERGE (dispatch_format)-[:CALLS]->(adapter_format)
MERGE (dispatch_format)-[:CALLS]->(adapter_parse)
MERGE (dispatch_format)-[:CALLS]->(budget)
MERGE (validate)-[:CALLS]->(valid_digest)
MERGE (strict_parse)-[:CALLS]->(parse_v1)
MERGE (strict_parse)-[:USES_TYPE]->(parser_error)
MERGE (strict_format)-[:CALLS]->(format_v1)
MERGE (strict_format)-[:USES_TYPE]->(parser_error)
MERGE (strict_v2_parse)-[:CALLS]->(parse_v2)
MERGE (strict_v2_parse)-[:USES_TYPE]->(parser_error)
MERGE (strict_v2_format)-[:CALLS]->(format_v2)
MERGE (strict_v2_format)-[:USES_TYPE]->(parser_error)
MERGE (tests)-[:TESTS]->(register)
MERGE (tests)-[:TESTS]->(dispatch_parse)
MERGE (tests)-[:TESTS]->(dispatch_format)
MERGE (profiles_test)-[:TESTS]->(strict_parse)
MERGE (profiles_test)-[:TESTS]->(strict_v2_parse)
MERGE (profiles_test)-[:TESTS]->(dispatch_parse)
MERGE (profiles_test)-[:TESTS]->(dispatch_format)
MERGE (missing_test)-[:TESTS]->(parse_v2)
@endcypher
*/
//! Version-pinned interpreter registry for the NEMA codec.
//!
//! This is a host-side adapter seam. Third-party interpreter code must run in
//! an isolated worker or sandboxed WASM runtime; this crate must not load an
//! arbitrary native library or execute a user-selected shell command.

use crate::strict::{
    format_strict_v1, format_strict_v2, parse_strict_v1, parse_strict_v2, Cell, Document, Error,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

pub const INTERPRETER_ABI_V1: u16 = 1;
pub const NEMA_AST_SCHEMA_V1: u16 = 1;
pub const NEMA_INTERPRETER_PLUGIN_VERSION: &str = "0.1.0";
pub const NEMA_STRICT_V1_REVISION: &str = "matrix-2026-11-10";
pub const NEMA_STRICT_V2_REVISION: &str = "matrix-2026-11-15";
pub const NEMA_STRICT_V1_GRAMMAR_SHA256: &str =
    "0e1ae91167227f595f7586fa2174b19cb0eab4458c72e00c5790de770130f7b3";
pub const NEMA_STRICT_V2_GRAMMAR_SHA256: &str =
    "ec274eda324c8fc88edc88a4bbc5afe3b508d4b4ca953ead173809aca6ae058a";
const NEMA_STRICT_PLUGIN_SOURCE_SHA256: &str =
    "4ae855e1da4c595d0d146183c304bee966c5fb2f34a846fd0da3d224d7311707";

/// Exact interpreter identity persisted with a workspace or import operation.
///
/// Grammar revision and implementation version are separate. The digests pin
/// the grammar snapshot and executable artifact so an upgrade cannot silently
/// reinterpret an existing `.nema` document.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct InterpreterProfile {
    pub format_id: String,
    pub plugin_id: String,
    pub grammar_revision: String,
    pub plugin_version: String,
    pub abi_version: u16,
    pub ast_schema_version: u16,
    pub grammar_source_sha256: String,
    pub artifact_sha256: String,
}

impl InterpreterProfile {
    /// Rejects incomplete identities, unsupported contracts and malformed digests.
    pub fn validate(&self) -> Result<(), InterpreterError> {
        if self.format_id != "nema" {
            return Err(InterpreterError::InvalidProfile("format_id must be nema"));
        }
        for (value, label) in [
            (&self.plugin_id, "plugin_id"),
            (&self.grammar_revision, "grammar_revision"),
            (&self.plugin_version, "plugin_version"),
        ] {
            if value.is_empty()
                || value.len() > 128
                || !value
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'.' | b'_' | b'-' | b'+'))
            {
                return Err(InterpreterError::InvalidProfile(label));
            }
        }
        if !valid_sha256(&self.grammar_source_sha256) {
            return Err(InterpreterError::InvalidProfile("grammar_source_sha256"));
        }
        if !valid_sha256(&self.artifact_sha256) {
            return Err(InterpreterError::InvalidProfile("artifact_sha256"));
        }
        if self.abi_version != INTERPRETER_ABI_V1 {
            return Err(InterpreterError::UnsupportedAbi(self.abi_version));
        }
        if self.ast_schema_version != NEMA_AST_SCHEMA_V1 {
            return Err(InterpreterError::UnsupportedAstSchema(
                self.ast_schema_version,
            ));
        }
        Ok(())
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}

/// Errors from profile validation, interpreter dispatch, and plugin execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpreterError {
    InvalidProfile(&'static str),
    UnsupportedAbi(u16),
    UnsupportedAstSchema(u16),
    DuplicateProfile,
    UnregisteredProfile,
    InputTooLarge,
    OutputTooLarge,
    DocumentOverBudget,
    NonRoundTrippingFormat,
    Parser { line: usize, message: String },
    PluginExecution { code: String, detail: String },
}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProfile(field) => write!(f, "invalid interpreter profile: {field}"),
            Self::UnsupportedAbi(version) => write!(f, "unsupported interpreter ABI {version}"),
            Self::UnsupportedAstSchema(version) => {
                write!(f, "unsupported NEMA AST schema {version}")
            }
            Self::DuplicateProfile => f.write_str("interpreter profile already registered"),
            Self::UnregisteredProfile => f.write_str("exact interpreter profile is not installed"),
            Self::InputTooLarge => f.write_str("interpreter input exceeds the size limit"),
            Self::OutputTooLarge => f.write_str("interpreter output exceeds the size limit"),
            Self::DocumentOverBudget => f.write_str("NEMA document exceeds structural limits"),
            Self::NonRoundTrippingFormat => {
                f.write_str("formatter output does not preserve the NEMA document")
            }
            Self::Parser { line, message } => write!(f, "line {line}: {message}"),
            Self::PluginExecution { code, detail } => {
                write!(f, "interpreter plugin failed ({code}): {detail}")
            }
        }
    }
}

impl std::error::Error for InterpreterError {}

impl From<Error> for InterpreterError {
    fn from(error: Error) -> Self {
        Self::Parser {
            line: error.line,
            message: error.message.to_string(),
        }
    }
}

/// Trusted host-side adapter for a pinned interpreter.
///
/// A third-party plugin is not this Rust trait object: the product adapter must
/// forward calls to the isolated plugin runtime and enforce its capability and
/// resource budgets. Built-in Rust interpreters can implement it directly.
pub trait NemaInterpreterAdapter: Send + Sync {
    /// Returns the immutable identity this adapter serves.
    fn profile(&self) -> &InterpreterProfile;

    /// Parses source with this profile's exact grammar implementation.
    fn parse(&self, source: &str) -> Result<Document, InterpreterError>;

    /// Formats a document using this profile's canonical output rules.
    fn format(&self, document: &Document) -> Result<String, InterpreterError>;
}

/// Exact-profile dispatcher. It has no "latest" or silent fallback lookup.
#[derive(Default)]
pub struct NemaInterpreterRegistry {
    adapters: BTreeMap<InterpreterProfile, Arc<dyn NemaInterpreterAdapter>>,
}

impl NemaInterpreterRegistry {
    /// Registers a built-in adapter or a trusted adapter for an isolated plugin host.
    pub fn register(
        &mut self,
        adapter: Arc<dyn NemaInterpreterAdapter>,
    ) -> Result<(), InterpreterError> {
        let profile = adapter.profile().clone();
        profile.validate()?;
        if self.adapters.contains_key(&profile) {
            return Err(InterpreterError::DuplicateProfile);
        }
        self.adapters.insert(profile, adapter);
        Ok(())
    }

    /// Lists available identities without choosing one implicitly.
    pub fn profiles(&self) -> Vec<InterpreterProfile> {
        self.adapters.keys().cloned().collect()
    }

    /// Creates the two Rust interpreters bundled with this crate. The caller
    /// must still select an exact profile; this method has no latest fallback.
    pub fn bundled() -> Result<Self, InterpreterError> {
        let mut registry = Self::default();
        let v1 = bundled_profile(NEMA_STRICT_V1_REVISION, NEMA_STRICT_V1_GRAMMAR_SHA256);
        let v2 = bundled_profile(NEMA_STRICT_V2_REVISION, NEMA_STRICT_V2_GRAMMAR_SHA256);
        registry.register(Arc::new(StrictV1Interpreter::new(v1)?))?;
        registry.register(Arc::new(StrictV2Interpreter::new(v2)?))?;
        Ok(registry)
    }

    /// Parses only with the exact profile supplied by the caller.
    pub fn parse(
        &self,
        profile: &InterpreterProfile,
        source: &str,
    ) -> Result<Document, InterpreterError> {
        profile.validate()?;
        if source.len() > crate::strict::MAX_SOURCE_BYTES {
            return Err(InterpreterError::InputTooLarge);
        }
        let adapter = self
            .adapters
            .get(profile)
            .ok_or(InterpreterError::UnregisteredProfile)?;
        let document = adapter.parse(source)?;
        validate_document_budget(&document)?;
        Ok(document)
    }

    /// Formats with the pinned profile and verifies the result by parsing it back.
    pub fn format(
        &self,
        profile: &InterpreterProfile,
        document: &Document,
    ) -> Result<String, InterpreterError> {
        profile.validate()?;
        validate_document_budget(document)?;
        let adapter = self
            .adapters
            .get(profile)
            .ok_or(InterpreterError::UnregisteredProfile)?;
        let output = adapter.format(document)?;
        if output.len() > crate::strict::MAX_SOURCE_BYTES {
            return Err(InterpreterError::OutputTooLarge);
        }
        let reparsed = adapter.parse(&output)?;
        validate_document_budget(&reparsed)?;
        if reparsed != *document {
            return Err(InterpreterError::NonRoundTrippingFormat);
        }
        Ok(output)
    }
}

/// Returns the exact identity of one bundled grammar snapshot.
pub fn bundled_profile_for_revision(revision: &str) -> Option<InterpreterProfile> {
    let grammar_sha256 = match revision {
        NEMA_STRICT_V1_REVISION => NEMA_STRICT_V1_GRAMMAR_SHA256,
        NEMA_STRICT_V2_REVISION => NEMA_STRICT_V2_GRAMMAR_SHA256,
        _ => return None,
    };
    Some(bundled_profile(revision, grammar_sha256))
}

fn bundled_profile(revision: &str, grammar_sha256: &str) -> InterpreterProfile {
    InterpreterProfile {
        format_id: "nema".into(),
        plugin_id: "org.star.nema.strict-rust".into(),
        grammar_revision: revision.into(),
        plugin_version: NEMA_INTERPRETER_PLUGIN_VERSION.into(),
        abi_version: INTERPRETER_ABI_V1,
        ast_schema_version: NEMA_AST_SCHEMA_V1,
        grammar_source_sha256: grammar_sha256.into(),
        artifact_sha256: NEMA_STRICT_PLUGIN_SOURCE_SHA256.into(),
    }
}

fn validate_bundled_profile(
    profile: &InterpreterProfile,
    expected_revision: &str,
    expected_grammar_sha256: &str,
) -> Result<(), InterpreterError> {
    profile.validate()?;
    if profile.plugin_id != "org.star.nema.strict-rust"
        || profile.plugin_version != NEMA_INTERPRETER_PLUGIN_VERSION
        || profile.grammar_revision != expected_revision
        || profile.grammar_source_sha256 != expected_grammar_sha256
        || profile.artifact_sha256 != NEMA_STRICT_PLUGIN_SOURCE_SHA256
    {
        return Err(InterpreterError::InvalidProfile(
            "profile does not identify the bundled grammar implementation",
        ));
    }
    Ok(())
}

fn validate_document_budget(document: &Document) -> Result<(), InterpreterError> {
    if document.domains.len() > crate::strict::MAX_DOMAINS {
        return Err(InterpreterError::DocumentOverBudget);
    }
    let mut records = 0usize;
    let mut cells = 0usize;
    let mut text_bytes = 0usize;
    for domain in &document.domains {
        if let Some(columns) = &domain.columns {
            if columns.len() > crate::strict::MAX_COLUMNS {
                return Err(InterpreterError::DocumentOverBudget);
            }
            cells = cells
                .checked_add(columns.len())
                .filter(|count| *count <= crate::strict::MAX_TOTAL_CELLS)
                .ok_or(InterpreterError::DocumentOverBudget)?;
            for column in columns {
                text_bytes = text_bytes
                    .checked_add(column.len())
                    .filter(|size| *size <= crate::strict::MAX_SOURCE_BYTES)
                    .ok_or(InterpreterError::DocumentOverBudget)?;
            }
        }
        records = records
            .checked_add(domain.records.len())
            .filter(|count| *count <= crate::strict::MAX_RECORDS)
            .ok_or(InterpreterError::DocumentOverBudget)?;
        for row in &domain.records {
            if row.len() > crate::strict::MAX_COLUMNS {
                return Err(InterpreterError::DocumentOverBudget);
            }
            cells = cells
                .checked_add(row.len())
                .filter(|count| *count <= crate::strict::MAX_TOTAL_CELLS)
                .ok_or(InterpreterError::DocumentOverBudget)?;
            for cell in row {
                if let Cell::Value(value) = cell {
                    text_bytes = text_bytes
                        .checked_add(value.len())
                        .filter(|size| *size <= crate::strict::MAX_SOURCE_BYTES)
                        .ok_or(InterpreterError::DocumentOverBudget)?;
                }
            }
        }
    }
    Ok(())
}

/// Built-in Rust adapter for the pinned strict-v1 grammar.
pub struct StrictV1Interpreter {
    profile: InterpreterProfile,
}

impl StrictV1Interpreter {
    /// Builds the adapter only for a valid, exact profile descriptor.
    pub fn new(profile: InterpreterProfile) -> Result<Self, InterpreterError> {
        validate_bundled_profile(
            &profile,
            NEMA_STRICT_V1_REVISION,
            NEMA_STRICT_V1_GRAMMAR_SHA256,
        )?;
        Ok(Self { profile })
    }
}

impl NemaInterpreterAdapter for StrictV1Interpreter {
    fn profile(&self) -> &InterpreterProfile {
        &self.profile
    }

    fn parse(&self, source: &str) -> Result<Document, InterpreterError> {
        parse_strict_v1(source).map_err(InterpreterError::from)
    }

    fn format(&self, document: &Document) -> Result<String, InterpreterError> {
        format_strict_v1(document).map_err(InterpreterError::from)
    }
}

/// Built-in Rust adapter for the indexed-continuation grammar snapshot.
pub struct StrictV2Interpreter {
    profile: InterpreterProfile,
}

impl StrictV2Interpreter {
    /// Builds the adapter only for a valid, exact profile descriptor.
    pub fn new(profile: InterpreterProfile) -> Result<Self, InterpreterError> {
        validate_bundled_profile(
            &profile,
            NEMA_STRICT_V2_REVISION,
            NEMA_STRICT_V2_GRAMMAR_SHA256,
        )?;
        Ok(Self { profile })
    }
}

impl NemaInterpreterAdapter for StrictV2Interpreter {
    fn profile(&self) -> &InterpreterProfile {
        &self.profile
    }

    fn parse(&self, source: &str) -> Result<Document, InterpreterError> {
        parse_strict_v2(source).map_err(InterpreterError::from)
    }

    fn format(&self, document: &Document) -> Result<String, InterpreterError> {
        format_strict_v2(document).map_err(InterpreterError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strict::Mode;

    fn profile(grammar_revision: &str, artifact_byte: char) -> InterpreterProfile {
        InterpreterProfile {
            format_id: "nema".into(),
            plugin_id: "org.star.nema.strict".into(),
            grammar_revision: grammar_revision.into(),
            plugin_version: "1.0.0".into(),
            abi_version: INTERPRETER_ABI_V1,
            ast_schema_version: NEMA_AST_SCHEMA_V1,
            grammar_source_sha256: "a".repeat(64),
            artifact_sha256: artifact_byte.to_string().repeat(64),
        }
    }

    fn bundled(revision: &str) -> InterpreterProfile {
        bundled_profile_for_revision(revision).expect("known bundled profile")
    }

    #[test]
    fn bundled_registry_keeps_exact_grammar_profiles_available() {
        let registry = NemaInterpreterRegistry::bundled().unwrap();
        let v1 = bundled(NEMA_STRICT_V1_REVISION);
        let v2 = bundled(NEMA_STRICT_V2_REVISION);
        assert_eq!(registry.profiles(), vec![v1.clone(), v2.clone()]);
        assert!(bundled_profile_for_revision("matrix-unknown").is_none());

        let source = " \\?2 1\nA\n 1 Retry\n";
        let document = registry.parse(&v2, source).unwrap();
        let formatted = registry.format(&v2, &document).unwrap();
        assert_eq!(registry.parse(&v2, &formatted).unwrap(), document);
        assert!(registry.parse(&v1, source).is_err());
    }

    struct MarkerAdapter {
        profile: InterpreterProfile,
        marker: &'static str,
    }

    impl NemaInterpreterAdapter for MarkerAdapter {
        fn profile(&self) -> &InterpreterProfile {
            &self.profile
        }

        fn parse(&self, source: &str) -> Result<Document, InterpreterError> {
            if source == self.marker {
                Ok(Document { domains: vec![] })
            } else {
                Err(InterpreterError::PluginExecution {
                    code: "marker_mismatch".into(),
                    detail: "wrong adapter selected".into(),
                })
            }
        }

        fn format(&self, document: &Document) -> Result<String, InterpreterError> {
            if document.domains.is_empty() {
                Ok(self.marker.into())
            } else {
                Err(InterpreterError::PluginExecution {
                    code: "unexpected_document".into(),
                    detail: "marker fixture accepts an empty document".into(),
                })
            }
        }
    }

    #[test]
    fn exact_profiles_select_compatible_interpreter_plugins() {
        let old = profile("grammar-2026-11-10", 'b');
        let current = profile("grammar-2026-11-15", 'c');
        let mut registry = NemaInterpreterRegistry::default();
        registry
            .register(Arc::new(MarkerAdapter {
                profile: old.clone(),
                marker: "old grammar",
            }))
            .unwrap();
        registry
            .register(Arc::new(MarkerAdapter {
                profile: current.clone(),
                marker: "current grammar",
            }))
            .unwrap();

        let old_doc = registry.parse(&old, "old grammar").unwrap();
        let current_doc = registry.parse(&current, "current grammar").unwrap();
        assert_eq!(registry.format(&old, &old_doc).unwrap(), "old grammar");
        assert_eq!(
            registry.format(&current, &current_doc).unwrap(),
            "current grammar"
        );
        assert_eq!(registry.profiles(), vec![old, current]);
    }

    #[test]
    fn unknown_profiles_never_fall_back_to_latest() {
        let installed = profile("grammar-2026-11-15", 'c');
        let absent = profile("grammar-unknown", 'd');
        let mut registry = NemaInterpreterRegistry::default();
        registry
            .register(Arc::new(MarkerAdapter {
                profile: installed,
                marker: "current grammar",
            }))
            .unwrap();
        assert_eq!(
            registry.parse(&absent, "current grammar"),
            Err(InterpreterError::UnregisteredProfile)
        );
    }

    #[test]
    fn duplicate_profiles_and_unimplemented_contract_versions_fail_closed() {
        let pinned = profile("grammar-2026-11-15", 'c');
        let mut registry = NemaInterpreterRegistry::default();
        registry
            .register(Arc::new(MarkerAdapter {
                profile: pinned.clone(),
                marker: "current grammar",
            }))
            .unwrap();
        assert_eq!(
            registry.register(Arc::new(MarkerAdapter {
                profile: pinned,
                marker: "current grammar",
            })),
            Err(InterpreterError::DuplicateProfile)
        );

        let mut unsupported_abi = profile("grammar-2026-11-15", 'd');
        unsupported_abi.abi_version += 1;
        assert_eq!(
            unsupported_abi.validate(),
            Err(InterpreterError::UnsupportedAbi(INTERPRETER_ABI_V1 + 1))
        );
        let mut unsupported_ast = profile("grammar-2026-11-15", 'e');
        unsupported_ast.ast_schema_version += 1;
        assert_eq!(
            unsupported_ast.validate(),
            Err(InterpreterError::UnsupportedAstSchema(
                NEMA_AST_SCHEMA_V1 + 1
            ))
        );
    }

    #[test]
    fn built_in_rust_interpreter_preserves_four_distinct_cell_states() {
        let pinned = bundled(NEMA_STRICT_V1_REVISION);
        let mut registry = NemaInterpreterRegistry::default();
        registry
            .register(Arc::new(StrictV1Interpreter::new(pinned.clone()).unwrap()))
            .unwrap();
        let document = registry.parse(&pinned, " \\1 1\nA ~ \\e \\_\n").unwrap();
        assert_eq!(
            document.domains[0].records[0],
            vec![
                Cell::Value("A".into()),
                Cell::Null,
                Cell::Empty,
                Cell::Missing
            ]
        );
        let formatted = registry.format(&pinned, &document).unwrap();
        assert_eq!(registry.parse(&pinned, &formatted).unwrap(), document);
    }

    #[test]
    fn v1_and_v2_profiles_route_graph_continuations_to_the_pinned_parser() {
        let old = bundled(NEMA_STRICT_V1_REVISION);
        let current = bundled(NEMA_STRICT_V2_REVISION);
        let mut registry = NemaInterpreterRegistry::default();
        registry
            .register(Arc::new(StrictV1Interpreter::new(old.clone()).unwrap()))
            .unwrap();
        registry
            .register(Arc::new(StrictV2Interpreter::new(current.clone()).unwrap()))
            .unwrap();

        let old_source = " \\>3 1\nA LoginAPI\n V2\n";
        let current_source = " \\>3 1\nA LoginAPI\n 1 V2\n";
        let old_doc = registry.parse(&old, old_source).unwrap();
        let current_doc = registry.parse(&current, current_source).unwrap();
        assert!(registry.parse(&old, current_source).is_err());
        assert_eq!(
            old_doc.domains[0].records[0],
            vec![
                Cell::Value("A".into()),
                Cell::Value("LoginAPI".into()),
                Cell::Value("V2".into())
            ]
        );
        assert_eq!(
            current_doc.domains[0].records[0],
            vec![Cell::Value("A".into()), Cell::Value("LoginAPIV2".into())]
        );
        let old_text = registry.format(&old, &old_doc).unwrap();
        let current_text = registry.format(&current, &current_doc).unwrap();
        assert_eq!(registry.parse(&old, &old_text).unwrap(), old_doc);
        assert_eq!(
            registry.parse(&current, &current_text).unwrap(),
            current_doc
        );
    }

    #[test]
    fn v2_continuation_rejects_explicit_missing_but_fills_implicit_missing() {
        assert!(parse_strict_v2(" \\?2 1\nA \\_\n 1 Retry\n").is_err());
        let document = parse_strict_v2(" \\?2 1\nA\n 1 Retry\n").unwrap();
        assert_eq!(
            document.domains[0].records[0][1],
            Cell::Value("Retry".into())
        );
        let text = format_strict_v2(&document).unwrap();
        assert_eq!(parse_strict_v2(&text).unwrap(), document);

        let sparse = parse_strict_v2(" \\?2 2\nA\n 3 Retry\n").unwrap();
        let sparse_text = format_strict_v2(&sparse).unwrap();
        assert_eq!(parse_strict_v2(&sparse_text).unwrap(), sparse);
    }

    #[test]
    fn plugin_formatter_must_round_trip_document_without_loss() {
        struct LossyAdapter {
            profile: InterpreterProfile,
        }
        impl NemaInterpreterAdapter for LossyAdapter {
            fn profile(&self) -> &InterpreterProfile {
                &self.profile
            }
            fn parse(&self, _: &str) -> Result<Document, InterpreterError> {
                Ok(Document { domains: vec![] })
            }
            fn format(&self, _: &Document) -> Result<String, InterpreterError> {
                Ok(String::new())
            }
        }
        let pinned = profile("grammar-2026-11-15", 'a');
        let mut registry = NemaInterpreterRegistry::default();
        registry
            .register(Arc::new(LossyAdapter {
                profile: pinned.clone(),
            }))
            .unwrap();
        let document = Document {
            domains: vec![crate::strict::Domain {
                mode: Mode::Table,
                id: 1,
                revision: 1,
                columns: None,
                explicit_header: false,
                declared_width: None,
                records: vec![vec![Cell::Value("data".into())]],
            }],
        };
        assert_eq!(
            registry.format(&pinned, &document),
            Err(InterpreterError::NonRoundTrippingFormat)
        );
    }
}
