//! ═══════════════════════════════════════════════════════════════════
//! Ported from UlyssesLeoLee/SandTree v1.1 (Apache-2.0)
//! per REUSE-MAP-COLLAB-SANDBOX-001 §3 (直接复用, 保留 NOTICE).
//! Source: E:\SandTree\crates\{model,observation-model,resource-graph}
//! ═══════════════════════════════════════════════════════════════════

//! SandTree domain model.
//!
//! This crate is the only vocabulary shared by the kernel, plugins, IPC and the
//! UI. It intentionally has **no** dependency on any provider SDK, on SQLite,
//! on Wasmtime or on the filesystem: NFR-O02 requires the kernel to speak only
//! in domain DTOs, and that rule is enforced by this crate's dependency list
//! rather than by review comments.

// (deny(missing_docs) 移除: 内嵌为子模块, 由 crate 级 allow 统一)

pub mod bundle;
pub mod capability;
pub mod error;
pub mod event;
pub mod id;
pub mod operation;
pub mod resource;

pub use bundle::{
    digest_of, is_valid_blob_name, is_valid_bundle_path, sanitize_blob_path, sort_blobs,
    validate_blob_name, validate_bundle_path, validate_manifest, BundleBlob, BundleError,
    BundleManifest, BundleResource, BundleSource, CollectedBlob, CollectedBundle, Compression,
    RestoreOptions, RestoreOutcome, BUNDLE_FORMAT, BUNDLE_VERSION, CONFIG_ENTRY, LAYERS_ENTRY,
    MANIFEST_ENTRY, VOLUMES_DIR,
};
pub use capability::{Capability, CapabilityNamespace, CapabilityParseError, CapabilitySet};
pub use error::{DomainError, ErrorCode};
pub use event::{EventRecord, EventType, Severity};
pub use id::{
    AppId, ArchiveId, CorrelationId, EndpointId, Generation, IdError, PluginId, ResourceId,
    SessionId, SnapshotId,
};
pub use operation::{
    OperationId, OperationKind, OperationOutcome, OperationProgress, OperationRequest,
    OperationState,
};
pub use resource::{
    Correlation, Relation, RelationKind, ResourceKind, ResourceNode, ResourceState, Timestamp,
};

/// Convenience alias for a JSON object payload crossing a boundary.
pub type Metadata = serde_json::Value;
