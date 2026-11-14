//! ═══════════════════════════════════════════════════════════════════
//! Ported from UlyssesLeoLee/SandTree v1.1 (Apache-2.0)
//! per REUSE-MAP-COLLAB-SANDBOX-001 §3 (直接复用, 保留 NOTICE).
//! Source: E:\SandTree\crates\{model,observation-model,resource-graph}
//! ═══════════════════════════════════════════════════════════════════

//! Resource bundle format (ADR-057; `docs/45` §3).
//!
//! A **bundle** is the single intermediate form behind both "move a resource to
//! another endpoint" and "archive a resource compressed" (ADR-057). The two
//! products look different to the user, but they need the same thing: a
//! portable, self-describing copy of a resource that can be rebuilt on another
//! engine. Implementing the collection and the rebuild **once** is what makes
//! the two features share a code path instead of merely resemble each other.
//!
//! This module owns the **DTO** and the **layout rules**. The rules are pure
//! functions on purpose: they are the part that keeps a hostile archive from
//! turning into a path-traversal vector on the machine that restores it, and
//! they are testable without a Docker daemon, a disk or a clock.

use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::model::id::{EndpointId, PluginId, ResourceId};
use crate::model::resource::ResourceKind;

/// Format tag written into every bundle manifest.
pub const BUNDLE_FORMAT: &str = "sandtree.bundle";

/// Current bundle format version.
pub const BUNDLE_VERSION: u32 = 1;

/// Name of the manifest entry inside a bundle.
pub const MANIFEST_ENTRY: &str = "manifest.json";

/// Bundle path of the captured provider configuration.
pub const CONFIG_ENTRY: &str = "config/container.json";

/// Bundle path of the captured writable layer.
pub const LAYERS_ENTRY: &str = "layers/image.tar";

/// Directory that holds captured volume data.
pub const VOLUMES_DIR: &str = "volumes";

/// Compression codec a bundle container is stored with.
///
/// The codec applies to the **whole container**, not to individual blobs
/// (`docs/45` §6.1). One compressed stream is one CAS object, which is what
/// makes an archive a single atomic commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Compression {
    /// zstd — the default: smaller than gzip and much faster.
    Zstd,
    /// gzip — for archives that generic tools must be able to read.
    Gzip,
}

impl Compression {
    /// Wire name stored in the manifest and in the `archive` table.
    pub fn as_str(self) -> &'static str {
        match self {
            Compression::Zstd => "zstd",
            Compression::Gzip => "gzip",
        }
    }

    /// Parse a wire name.
    pub fn from_wire(raw: &str) -> Option<Self> {
        match raw {
            "zstd" => Some(Compression::Zstd),
            "gzip" => Some(Compression::Gzip),
            _ => None,
        }
    }

    /// Container-level codec identifier.
    ///
    /// The container carries this byte in its header so a reader knows how to
    /// decompress **before** it can read the manifest that also names the codec.
    pub fn codec_id(self) -> u8 {
        match self {
            Compression::Zstd => 1,
            Compression::Gzip => 2,
        }
    }

    /// Parse a container-level codec identifier.
    pub fn from_codec_id(id: u8) -> Option<Self> {
        match id {
            1 => Some(Compression::Zstd),
            2 => Some(Compression::Gzip),
            _ => None,
        }
    }
}

/// Where a bundle was captured.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleSource {
    /// Endpoint URI of the source engine.
    pub endpoint_uri: String,
    /// Provider that produced the bundle.
    pub provider_id: PluginId,
    /// Endpoint identity, when the source is known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint_id: Option<EndpointId>,
}

/// Which resource a bundle holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleResource {
    /// Resource kind at capture time (the enum is frozen).
    pub kind: ResourceKind,
    /// Display name at capture time.
    pub name: String,
    /// Resource id on the source endpoint. Provenance only — the id is not
    /// portable, which is exactly why the bundle carries the content.
    pub origin_id: ResourceId,
    /// Endpoint the resource was captured from.
    pub origin_endpoint_id: Option<EndpointId>,
}

/// One content blob inside a bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleBlob {
    /// Bundle-internal path, e.g. `layers/image.tar`.
    pub name: String,
    /// Size of the **uncompressed** blob, in bytes.
    pub size: u64,
    /// BLAKE3 digest of the **uncompressed** blob.
    ///
    /// Over the uncompressed bytes on purpose: re-compressing the same content
    /// with another codec must not change content identity (S-14-03).
    pub digest: String,
    /// Provider-side name of the thing this blob holds, e.g. the volume name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}

/// The manifest of a bundle: what is inside, where it came from, how to
/// rebuild it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BundleManifest {
    /// Format tag; always [`BUNDLE_FORMAT`].
    pub format: String,
    /// Format version.
    pub version: u32,
    /// Capture time, RFC3339.
    pub created_at: String,
    /// Where the bundle was captured.
    pub source: BundleSource,
    /// Which resource the bundle holds.
    pub resource: BundleResource,
    /// Captured provider configuration (image reference, command, environment,
    /// ports, mounts, labels, limits, network).
    #[serde(default)]
    pub config: Json,
    /// Content blobs, **sorted by name**.
    pub blobs: Vec<BundleBlob>,
    /// Codec the container is stored with.
    pub compression: Compression,
}

impl BundleManifest {
    /// Total uncompressed size of every blob in the bundle.
    pub fn total_uncompressed_size(&self) -> u64 {
        self.blobs.iter().map(|b| b.size).sum()
    }

    /// The blob entry with the given bundle-internal path.
    pub fn blob(&self, name: &str) -> Option<&BundleBlob> {
        self.blobs.iter().find(|b| b.name == name)
    }
}

/// Raw content destined for one bundle entry.
///
/// This is the shape a **provider** hands over: the bytes are still
/// uncompressed, and the name is the bundle-internal path they will get.
#[derive(Debug, Clone)]
pub struct CollectedBlob {
    /// Bundle-internal path. Must satisfy the layout rules.
    pub name: String,
    /// Provider-side name of the thing this blob holds, e.g. the volume name.
    pub origin: Option<String>,
    /// Raw, uncompressed bytes.
    pub data: Vec<u8>,
}

/// A collected bundle: the manifest plus the content it describes.
///
/// A provider produces this; the store turns it into one compressed CAS
/// object. The two steps are separate on purpose: **collection** is
/// provider-specific, **packing** is not (ADR-057).
#[derive(Debug, Clone)]
pub struct CollectedBundle {
    /// What the bundle holds.
    pub manifest: BundleManifest,
    /// The blob contents.
    pub blobs: Vec<CollectedBlob>,
}

/// What a restore is allowed to overwrite.
///
/// # Why the caller has to say so
///
/// A restore writes into the target engine, and some of what it writes lands on
/// top of something that is already there — a named volume with data in it, a
/// container name that is taken. Overwriting those silently would make a
/// restore look successful while quietly destroying something else, so the
/// default is to **refuse** and let the caller opt in.
///
/// The opt-in is the confirmation NFR-U02 asks for: the destructive part is
/// never implied, it is stated.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreOptions {
    /// Replace volumes that already exist on the target engine.
    ///
    /// `false` (the default) makes a restore that would overwrite an existing
    /// volume **fail**, naming the volume.
    pub replace_volumes: bool,
    /// Override the restored resource's name.
    ///
    /// The captured name is used when this is `None`; when the target already
    /// has a resource with that name, the restore fails rather than silently
    /// renaming or deleting anything.
    pub name: Option<String>,
}

/// What a restore actually did.
///
/// The two lists exist so that a restore can be **honest about what it could
/// not carry over**. A bind mount points at a path on the *source* machine;
/// its contents are not in the bundle, and a restore that reported plain
/// success would be claiming otherwise.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreOutcome {
    /// The id of the resource that was created on the target.
    pub resource_id: ResourceId,
    /// Things whose previous content was replaced by the bundle.
    pub replaced: Vec<String>,
    /// Things that could not be carried over, and why.
    pub not_carried: Vec<String>,
}

impl RestoreOutcome {
    /// An outcome with nothing replaced and nothing missing.
    pub fn new(resource_id: ResourceId) -> Self {
        Self {
            resource_id,
            replaced: Vec::new(),
            not_carried: Vec::new(),
        }
    }
}

/// Why a bundle was rejected.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BundleError {
    /// The bundle-internal path was empty.
    #[error("bundle path must not be empty")]
    EmptyPath,
    /// The bundle-internal path was absolute.
    #[error("bundle path must be relative: {0}")]
    AbsolutePath(String),
    /// The bundle-internal path contained a `..` segment.
    #[error("bundle path must not contain a `..` segment: {0}")]
    ParentSegment(String),
    /// The bundle-internal path used a character outside `[a-z0-9._-]`.
    #[error("bundle path uses a forbidden character: {0}")]
    IllegalCharacter(String),
    /// The bundle-internal path contained an empty or `.` segment.
    #[error("bundle path contains an empty or `.` segment: {0}")]
    IllegalSegment(String),
    /// The name is reserved for the manifest.
    #[error("bundle path is reserved: {0}")]
    ReservedPath(String),
    /// The same blob name appeared twice.
    #[error("duplicate bundle blob: {0}")]
    DuplicateBlob(String),
    /// The blob list was not sorted by name.
    #[error("bundle blobs must be sorted by name: {0} follows {1}")]
    UnsortedBlobs(String, String),
    /// A blob digest was not a BLAKE3 hex digest.
    #[error("bundle blob {0:?} has a malformed digest")]
    MalformedDigest(String),
    /// The manifest declared a format this build does not know.
    #[error("unsupported bundle format {0:?}")]
    UnsupportedFormat(String),
    /// The manifest declared a version this build does not know.
    #[error("unsupported bundle version {0}")]
    UnsupportedVersion(u32),
    /// The bundle content did not match what the manifest declared.
    #[error("bundle content does not match the manifest: {0}")]
    ContentMismatch(String),
    /// A blob's bytes did not match its digest.
    #[error("bundle blob {0:?} is corrupt: digest mismatch")]
    DigestMismatch(String),
}

/// Validate a bundle-internal path.
///
/// The rules are the same ones the VFS applies to workspace paths (NFR-S05),
/// because a bundle gets unpacked on a machine that never saw it:
///
/// * relative — no leading `/`, no `\` root, no `C:` drive prefix;
/// * no `..` segment;
/// * no empty segment (`a//b`) and no `.` segment;
/// * every character in `[a-z0-9._-]`, with `/` as the only separator.
///
/// Upper case is excluded on purpose: the same content must produce the same
/// bytes on every machine that packs it, and case-folding rules differ
/// between file systems.
pub fn validate_bundle_path(path: &str) -> Result<(), BundleError> {
    if path.is_empty() {
        return Err(BundleError::EmptyPath);
    }
    if path.starts_with('/') || path.starts_with('\\') {
        return Err(BundleError::AbsolutePath(path.to_string()));
    }
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return Err(BundleError::AbsolutePath(path.to_string()));
    }
    for segment in path.split('/') {
        if segment.is_empty() || segment == "." {
            return Err(BundleError::IllegalSegment(path.to_string()));
        }
        if segment == ".." {
            return Err(BundleError::ParentSegment(path.to_string()));
        }
        if !segment.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        }) {
            return Err(BundleError::IllegalCharacter(path.to_string()));
        }
    }
    Ok(())
}

/// Whether a bundle-internal path is legal.
pub fn is_valid_bundle_path(path: &str) -> bool {
    validate_bundle_path(path).is_ok()
}

/// Validate a blob name: a bundle-internal path that is not the manifest.
pub fn validate_blob_name(name: &str) -> Result<(), BundleError> {
    validate_bundle_path(name)?;
    if name == MANIFEST_ENTRY {
        return Err(BundleError::ReservedPath(name.to_string()));
    }
    Ok(())
}

/// Whether a blob name is legal.
pub fn is_valid_blob_name(name: &str) -> bool {
    validate_blob_name(name).is_ok()
}

/// Sort blob entries by name.
///
/// The manifest's blob list must be sorted: it is what makes the packed bytes
/// deterministic, and a deterministic bundle is what lets the CAS recognise
/// two archives of the same content as the same object.
pub fn sort_blobs(blobs: &mut [BundleBlob]) {
    blobs.sort_by(|a, b| a.name.cmp(&b.name));
}

/// The digest of a blob's **uncompressed** content.///
/// # One algorithm, one place
///
/// The bundle format, the CAS and the provider-side capture all have to agree
/// on what a blob's digest is, or a restore cannot tell a corrupt archive from
/// an intact one. Computing it here — instead of at each call site — is what
/// keeps that agreement from drifting into "two ways to spell the same
/// digest".
///
/// The digest covers the **uncompressed** bytes on purpose: the archive's
/// codec is a property of the container, not of the content, so the same
/// content has the same digest whether it was stored with zstd or with gzip.
pub fn digest_of(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

/// Pair a bundle's manifest with its blob bytes.
///
/// A stored archive keeps the two apart: the manifest names each blob and
/// records where it came from, the CAS object holds the bytes. A restore needs
/// them **together**, and the pairing rule has to live in one place — if the
/// origin were dropped on the way out of the archive, the restored image would
/// have no name to be restored under.
///
/// Blobs the manifest does not mention are **kept** rather than dropped: the
/// manifest is the index, but the bytes are the truth, and a restore that
/// silently ignored an unnamed blob would lose data without saying so.
pub fn blobs_of(
    manifest: &BundleManifest,
    bytes: std::collections::BTreeMap<String, Vec<u8>>,
) -> Vec<CollectedBlob> {
    bytes
        .into_iter()
        .map(|(name, data)| CollectedBlob {
            origin: manifest.blob(&name).and_then(|b| b.origin.clone()),
            name,
            data,
        })
        .collect()
}

/// Map a provider-side name onto a legal bundle path.
///
/// Volume names are chosen by the user and routinely contain characters the
/// bundle layout forbids (upper case, spaces, separators). The mapping has to
/// be **deterministic** — the same name must always land on the same path, or
/// a restore cannot find the blob again — and **collision free**: two
/// different names that fold onto the same path would silently overwrite each
/// other inside the archive.
///
/// Folding alone cannot do that (`PGDATA` and `pgdata` both fold to
/// `pgdata`), so the digest of the *original* name is appended. The suffix is
/// what carries the uniqueness; the folded stem only carries the readability.
pub fn sanitize_blob_path(dir: &str, origin: &str) -> Result<String, BundleError> {
    validate_bundle_path(dir)?;
    if dir.contains('/') {
        return Err(BundleError::IllegalSegment(dir.to_string()));
    }

    let mut folded = String::with_capacity(origin.len());
    let mut previous_dash = false;
    for ch in origin.chars() {
        let ch = if ch.is_ascii_alphanumeric() {
            ch.to_ascii_lowercase()
        } else {
            '-'
        };
        if ch == '-' {
            if previous_dash {
                continue;
            }
            previous_dash = true;
        } else {
            previous_dash = false;
        }
        folded.push(ch);
    }
    let stem = folded.trim_matches('-');
    let stem = if stem.is_empty() { "blob" } else { stem };

    let digest = blake3::hash(origin.as_bytes()).to_hex().to_string();
    Ok(format!("{dir}/{stem}-{}{}", &digest[..8], ".tar"))
}

/// Whether a string is a BLAKE3 hex digest.
fn is_digest(raw: &str) -> bool {
    raw.len() == 64
        && raw
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Validate a whole manifest: format, version and layout.
///
/// The layout checks are the ones that matter for safety: a bundle is
/// untrusted input on the restore side, so its own bookkeeping is checked
/// against the same rules the packer obeys.
pub fn validate_manifest(manifest: &BundleManifest) -> Result<(), BundleError> {
    if manifest.format != BUNDLE_FORMAT {
        return Err(BundleError::UnsupportedFormat(manifest.format.clone()));
    }
    if manifest.version != BUNDLE_VERSION {
        return Err(BundleError::UnsupportedVersion(manifest.version));
    }

    let mut previous: Option<&str> = None;
    for blob in &manifest.blobs {
        validate_blob_name(&blob.name)?;
        if !is_digest(&blob.digest) {
            return Err(BundleError::MalformedDigest(blob.name.clone()));
        }
        if let Some(prev) = previous {
            if blob.name == prev {
                return Err(BundleError::DuplicateBlob(blob.name.clone()));
            }
            if blob.name.as_str() < prev {
                return Err(BundleError::UnsortedBlobs(
                    blob.name.clone(),
                    prev.to_string(),
                ));
            }
        }
        previous = Some(&blob.name);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::id::ResourceId;

    fn manifest_with(blobs: Vec<BundleBlob>) -> BundleManifest {
        BundleManifest {
            format: BUNDLE_FORMAT.to_string(),
            version: BUNDLE_VERSION,
            created_at: "2026-11-04T00:00:00Z".to_string(),
            source: BundleSource {
                endpoint_uri: "npipe:////./pipe/docker_engine".to_string(),
                provider_id: PluginId::derive(&["sandtree", "provider", "docker"]),
                endpoint_id: None,
            },
            resource: BundleResource {
                kind: ResourceKind::Container,
                name: "rgs-postgres".to_string(),
                origin_id: ResourceId::derive(&["ep", "c1"]),
                origin_endpoint_id: None,
            },
            config: serde_json::json!({ "image": "postgres:16" }),
            blobs,
            compression: Compression::Zstd,
        }
    }

    fn blob(name: &str) -> BundleBlob {
        BundleBlob {
            name: name.to_string(),
            size: 12,
            digest: "a".repeat(64),
            origin: None,
        }
    }

    // --- 布局规则（S-14-01） ---

    #[test]
    fn relative_paths_are_accepted() {
        for path in [
            "manifest.json",
            "config/container.json",
            "layers/image.tar",
            "volumes/data.tar",
            "a/b/c.bin",
            "name.with.dots",
            "name_with_underscores",
            "name-with-dashes",
            "0",
        ] {
            assert!(
                is_valid_bundle_path(path),
                "{path} should be a legal bundle path"
            );
        }
    }

    #[test]
    fn absolute_paths_are_rejected() {
        // A bundle gets unpacked on a machine that never saw it, so a path that
        // leaves the archive root must never be accepted (NFR-S05).
        for path in [
            "/etc/passwd",
            "\\windows\\system32",
            "C:/windows",
            "c:\\temp",
        ] {
            assert!(
                !is_valid_bundle_path(path),
                "{path} must be rejected as absolute"
            );
        }
    }

    #[test]
    fn parent_segments_are_rejected() {
        for path in ["../evil", "a/../../evil", "..", "a/.."] {
            assert!(
                !is_valid_bundle_path(path),
                "{path} must be rejected: a bundle must not escape its root"
            );
        }
    }

    #[test]
    fn empty_and_dot_segments_are_rejected() {
        for path in ["", "a//b", "./a", "a/./b"] {
            assert!(
                !is_valid_bundle_path(path),
                "{path} must be rejected: empty or dot segment"
            );
        }
    }

    #[test]
    fn characters_outside_the_allowed_set_are_rejected() {
        for path in [
            "Layers/Image.tar",
            "image tar",
            "a\\b",
            "a:b",
            "a*b",
            "a?b",
            "a|b",
            "a\x00b",
        ] {
            assert!(
                !is_valid_bundle_path(path),
                "{path} must be rejected: character outside [a-z0-9._-]"
            );
        }
    }

    #[test]
    fn the_manifest_entry_name_is_reserved() {
        assert!(is_valid_bundle_path(MANIFEST_ENTRY));
        assert!(
            !is_valid_blob_name(MANIFEST_ENTRY),
            "a blob may not shadow the manifest"
        );
        assert!(
            is_valid_blob_name("config/manifest.json"),
            "only the top-level manifest name is reserved"
        );
    }

    #[test]
    fn sorted_blob_lists_are_accepted_and_unsorted_ones_are_not() {
        let ok = manifest_with(vec![blob("a.tar"), blob("b.tar"), blob("c.tar")]);
        assert!(validate_manifest(&ok).is_ok());

        let unsorted = manifest_with(vec![blob("b.tar"), blob("a.tar")]);
        assert_eq!(
            validate_manifest(&unsorted),
            Err(BundleError::UnsortedBlobs(
                "a.tar".to_string(),
                "b.tar".to_string()
            ))
        );
    }

    #[test]
    fn duplicate_blob_names_are_rejected() {
        let dup = manifest_with(vec![blob("a.tar"), blob("a.tar")]);
        assert_eq!(
            validate_manifest(&dup),
            Err(BundleError::DuplicateBlob("a.tar".to_string()))
        );
    }

    #[test]
    fn malformed_digests_are_rejected() {
        let bad = manifest_with(vec![BundleBlob {
            name: "a.tar".to_string(),
            size: 1,
            digest: "not-a-digest".to_string(),
            origin: None,
        }]);
        assert_eq!(
            validate_manifest(&bad),
            Err(BundleError::MalformedDigest("a.tar".to_string()))
        );
    }

    #[test]
    fn unknown_format_or_version_is_rejected() {
        let mut m = manifest_with(vec![]);
        m.format = "something.else".to_string();
        assert_eq!(
            validate_manifest(&m),
            Err(BundleError::UnsupportedFormat("something.else".to_string()))
        );

        let mut m = manifest_with(vec![]);
        m.version = BUNDLE_VERSION + 1;
        assert_eq!(
            validate_manifest(&m),
            Err(BundleError::UnsupportedVersion(BUNDLE_VERSION + 1))
        );
    }

    #[test]
    fn sort_blobs_puts_the_list_in_name_order() {
        let mut blobs = vec![blob("c.tar"), blob("a.tar"), blob("b.tar")];
        sort_blobs(&mut blobs);
        let names: Vec<&str> = blobs.iter().map(|b| b.name.as_str()).collect();
        assert_eq!(names, ["a.tar", "b.tar", "c.tar"]);
    }

    // --- 名字消毒 ---

    #[test]
    fn sanitized_paths_are_legal_and_deterministic() {
        let a = sanitize_blob_path(VOLUMES_DIR, "PGDATA").unwrap();
        let b = sanitize_blob_path(VOLUMES_DIR, "PGDATA").unwrap();
        assert_eq!(a, b, "the same name must always land on the same path");
        assert!(is_valid_bundle_path(&a), "{a} must be a legal bundle path");
        assert!(a.starts_with("volumes/pgdata-"), "{a}");
        assert!(a.ends_with(".tar"), "{a}");
    }

    #[test]
    fn sanitizing_distinguishes_names_that_fold_alike() {
        // Folding alone would map both of these onto `volumes/pgdata-*.tar`.
        let upper = sanitize_blob_path(VOLUMES_DIR, "PGDATA").unwrap();
        let lower = sanitize_blob_path(VOLUMES_DIR, "pgdata").unwrap();
        assert_ne!(upper, lower, "different names must not collide");
    }

    #[test]
    fn sanitizing_handles_names_with_no_usable_characters() {
        let path = sanitize_blob_path(VOLUMES_DIR, "///").unwrap();
        assert!(is_valid_bundle_path(&path), "{path}");
        assert!(path.starts_with("volumes/blob-"), "{path}");
    }

    #[test]
    fn sanitizing_rejects_an_illegal_directory() {
        assert!(sanitize_blob_path("../evil", "x").is_err());
        assert!(sanitize_blob_path("a/b", "x").is_err());
    }

    // --- 压缩编解码标识 ---

    #[test]
    fn compression_wire_and_codec_ids_round_trip() {
        for codec in [Compression::Zstd, Compression::Gzip] {
            assert_eq!(Compression::from_wire(codec.as_str()), Some(codec));
            assert_eq!(
                Compression::from_codec_id(codec.codec_id()),
                Some(codec),
                "the container header and the manifest must name the same codec"
            );
        }
        assert_eq!(Compression::from_wire("lzma"), None);
        assert_eq!(Compression::from_codec_id(0), None);
        assert_eq!(Compression::from_codec_id(3), None);
    }

    #[test]
    fn the_manifest_keeps_the_uncompressed_size() {
        let m = manifest_with(vec![
            BundleBlob {
                name: "a.tar".to_string(),
                size: 10,
                digest: "a".repeat(64),
                origin: None,
            },
            BundleBlob {
                name: "b.tar".to_string(),
                size: 32,
                digest: "b".repeat(64),
                origin: None,
            },
        ]);
        assert_eq!(m.total_uncompressed_size(), 42);
        assert_eq!(m.blob("b.tar").map(|b| b.size), Some(32));
        assert!(m.blob("missing.tar").is_none());
    }
}
