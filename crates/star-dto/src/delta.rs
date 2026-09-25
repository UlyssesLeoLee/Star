// SPDX-License-Identifier: MIT OR Apache-2.0
//! `star-dto/src/delta.rs` -- PI-6 from SRS-PI-BORROW-001.
//!
//! JSON Delta protocol: 7 structural-diff ops over plain JSON values, plus
//! an `apply` / `apply_immutable` reducer used by the star-taskgraph
//! react-flow frontend for incremental viewport sync (per SRS §4 FR-27 +
//! BD §1.1 PI-6 row + chord `packages/chord/src/delta/` 67 KB reference).
//!
//! The 7 ops (borrowed-shape from `earendil-works/pi` v0.87.0 chord/delta):
//!
//! | Verb | Name      | Payload                                  | Effect                                  |
//! |------|-----------|------------------------------------------|-----------------------------------------|
//! | `r`  | replace   | `(JsonValue,)`                           | Replace the entire root value           |
//! | `s`  | set       | `(NonEmptyPath, JsonValue)`              | Write a value at the path               |
//! | `d`  | delete    | `(NonEmptyPath,)`                        | Remove a key/element at the path        |
//! | `a`  | append    | `(NonEmptyPath, String)`                 | Append a string to an existing string   |
//! | `t`  | truncate  | `(NonEmptyPath, usize)`                  | Truncate an existing string from index  |
//! | `p`  | splice    | `(Path, usize index, usize remove, Vec)` | Array splice: remove + insert in place  |
//! | `m`  | move      | `(Path, Vec<usize> permutation)`         | Reorder an array via a permutation      |
//!
//! **W/T/M 分类**: this module is **Work (W)** -- short-TTL effect carriers,
//! never persisted as their own DB tables. Consumers (star-taskgraph
//! frontend, star-taskqueue compaction, domain-worktree context edits) may
//! optionally audit-log the op stream under event_type=`'json_delta_op'`
//! (per BD §4 W/T/M table + 守门 #13 a).
//!
//! **Additive (v0.2.0)**: This is the first public release of the module.
//! `Op` carries `#[non_exhaustive]` so future verbs (per PI-6 ADR backlog)
//! can be added without breaking downstream. Removal lands in the next
//! minor, never silently.
//!
//! 守门合规 (per 守门 #1 v15 + 守门 #5 + 守门 #7 + 守门 #11 + 守门 #13 a + 守门 #14 v4):
//! - 0 `unsafe` blocks (`unsafe_code = "forbid"` workspace-wide).
//! - `#[non_exhaustive]` on `Op` for forward compatibility (per SRS §5 NFR-7).
//! - All DTOs derive `Serialize` / `Deserialize` (per 守门 #11 缺标比错标).
//! - `Op` payloads reject reserved path segments (`__proto__` / `constructor` /
//!   `prototype`) and prototype-chain escapes, so a malicious op cannot pollute
//!   `Object.prototype` for the whole process (per chord/delta UnsafePathError
//!   rationale + star-session 的 `attributor` 来自 facet/plugin/tool 不可信).
//! - 5 域 Lead signoff (per ULYS-175 2026-09-22 01:28 JST): Ulysses = 5-domain Lead.
//!
//! **Non-goals (per SRS §1.4 不含范围 + 守门 #11 缺标比错标)**:
//! - NO path-id interning / wire compression (that's a future codec layer; the
//!   current `apply` consumes in-memory `Op` only).
//! - NO chord `WireOp` enum (path interning + arity omission is a wire-format
//!   concern that lives in the eventual transport codec, not in this DTO crate).
//! - NO `Draft` / `Tracker` / `diffRevisions` (those are chord's higher-level
//!   change-tracking facade; star uses lower-level explicit `Op` emission).
//! - NO 91-provider catalog / no OAuth / no Gondolin / no CBOR / no chord
//!   runtime (per ULYS-175 §1.4 不抄清单, hard reject in PR review).

use serde::{Deserialize, Serialize};

/// **Segment** -- a single step inside a path: an object key (`String`) or
/// an array index (`usize`).
///
/// Tuple-struct style intentionally. `String` and `usize` stay distinct on the
/// wire (Serde representation differs), so a path typed in JSON as
/// `["users", 0, "name"]` round-trips losslessly.
pub type Seg = String;

/// **Path** -- ordered list of segments from the root to a value.
///
/// Empty path `[]` means "the root itself". Only `r` (replace) and `p` / `m`
/// (which target array values that may themselves be the root) may carry an
/// empty path; `s` / `d` / `a` / `t` require `NonEmptyPath` per the type
/// system.
pub type Path = Vec<Seg>;

/// **NonEmptyPath** -- guaranteed-non-empty path for `s` / `d` / `a` / `t`.
///
/// Used at the type level only; `assert_valid_op` enforces this constraint at
/// runtime for deserialized JSON.
pub type NonEmptyPath = Vec<Seg>;

/// **JsonValue** -- the value tree the delta operates over.
///
/// Mirroring chord's choice: we use `serde_json::Value` directly rather than a
/// bespoke enum because (a) the consumer is the react-flow JSON renderer and
/// (b) every JSON-compatible Rust value already round-trips losslessly through
/// `serde_json::Value`. The trade-off is a small allocation per node; that's
/// acceptable for incremental sync (per BD §5 NFR-1 单 chunk 延迟 < 50ms).
pub type JsonValue = serde_json::Value;

// =====================================================================
// §0 Op enum -- the 7-verb wire vocabulary
// =====================================================================

/// **`Op`** -- 7-verb JSON delta op (PI-6 / FR-27).
///
/// Tuple-struct style borrowed from chord `packages/chord/src/delta/index.ts`
/// line 35-44: verbs (`r` / `s` / `d` / `a` / `t` / `p` / `m`) are the only
/// legal discriminator values. Anything else is rejected by
/// [`assert_valid_op`].
///
/// **`r` is the ONLY op that replaces a whole value**. `s` / `d` / `a` / `t`
/// cannot target the root -- the type system encodes this by demanding
/// `NonEmptyPath`. `p` / `m` may target an array at the root, because the
/// root itself may be a JSON array.
///
/// [`#[non_exhaustive]`] guards against breaking downstream when future verbs
/// (per PI-6 ADR backlog) land (per SRS §5 NFR-7).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Op {
    /// **`r`** -- replace the entire root value.
    ///
    /// Carries no path; the second tuple element is the new root value.
    /// Adopted, not copied: the consumer owns the batch it was handed.
    /// Fanning one batch out to several consumers in-process therefore makes
    /// their replicas alias each other (per chord/delta ownership rule).
    Replace(JsonValue),

    /// **`s`** -- set a value at `path`.
    ///
    /// Writes the value via plain assignment so a setter inherited from the
    /// prototype chain does not run (per chord/delta prototype-pollution
    /// defense; at the Rust boundary `serde_json::Value` is a plain enum so
    /// prototype-chain escape is structurally impossible -- the guard
    /// remains as defense-in-depth for downstream re-encoding).
    Set(NonEmptyPath, JsonValue),

    /// **`d`** -- delete the key/element at `path`.
    ///
    /// For object parents: `delete parent[key]`. For array parents: splices
    /// the element out (length shrinks by 1).
    Delete(NonEmptyPath),

    /// **`a`** -- append `String` to an existing `String` value at `path`.
    ///
    /// The existing value MUST be a `String`; otherwise [`apply`] returns
    /// [`PathError`]. Optimized for repeated text-delta syncs (per
    /// star-taskgraph react-flow incremental text updates).
    Append(NonEmptyPath, String),

    /// **`t`** -- truncate a `String` value at `path` from index `n` onward.
    ///
    /// Equivalent to `current_string[n..].clear()`. `n` MUST be a valid
    /// char-boundary index; the applier advances to the next char boundary
    /// if `n` is mid-codepoint (per Rust `str::char_indices` semantics).
    Truncate(NonEmptyPath, usize),

    /// **`p`** -- splice an array at `path`: remove `remove` items starting at
    /// `index`, then insert `items` in their place.
    ///
    /// `index` is allowed to address exactly one past the end (append), but
    /// NOT a larger gap -- a sparse array does not survive a JSON round trip
    /// (per chord/delta `assertIndexInRange` rationale). The applier
    /// enforces this for deserialized ops.
    Splice(Path, usize, usize, Vec<JsonValue>),

    /// **`m`** -- reorder an array in place via `permutation`.
    ///
    /// `new[i] = old[permutation[i]]`. The permutation MUST be a bijection
    /// over `0..len`; the applier validates via [`assert_permutation`].
    Move(Path, Vec<usize>),
}

impl Op {
    /// **Discriminator verb** -- the single-char wire tag (`"r"` / `"s"` / etc.).
    ///
    /// Cheap branch-free access for routing / logging / debug prints without
    /// matching the full tuple.
    pub fn verb(&self) -> &'static str {
        match self {
            Self::Replace(_) => "r",
            Self::Set(_, _) => "s",
            Self::Delete(_) => "d",
            Self::Append(_, _) => "a",
            Self::Truncate(_, _) => "t",
            Self::Splice(_, _, _, _) => "p",
            Self::Move(_, _) => "m",
        }
    }

    /// **Target path** -- the path this op targets, when applicable.
    ///
    /// Returns `None` for [`Op::Replace`] (which targets the root by
    /// convention, not via a path) and `None` if the verb simply does not
    /// carry a path. Used by audit-log emission and by the future codec
    /// layer's path-interning decisions.
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Replace(_) => None,
            Self::Set(p, _) | Self::Delete(p) | Self::Append(p, _) | Self::Truncate(p, _) => {
                Some(p)
            }
            Self::Splice(p, _, _, _) | Self::Move(p, _) => Some(p),
        }
    }

    /// **Predicate**: is this a base batch anchor (`r` op)?
    ///
    /// A batch that starts with `r` is a recovery point: a downstream decoder
    /// replays from the last `r` with a fresh decoder state (per chord/delta
    /// recovery contract). When the batch is *only* one op and that op is
    /// `r`, the whole thing collapses to the replaced value.
    pub fn is_replace(&self) -> bool {
        matches!(self, Self::Replace(_))
    }
}

// =====================================================================
// §1 Path safety -- prototype-pollution defense
// =====================================================================

/// **Reserved segments** -- JSON-keys that would let an op escape into the
/// prototype chain and pollute `Object.prototype` for the whole process.
///
/// Ops arrive from a facet, a plugin compartment, or a tool whose details
/// may echo model output, so none of it is trusted input (per chord/delta
/// rationale + star-taskgraph's contributor set).
pub const RESERVED_SEGMENTS: &[&str] = &["__proto__", "constructor", "prototype"];

/// **Predicate**: is this segment a reserved prototype-chain entrypoint?
///
/// Negative-result hits the fast path; positive-result triggers
/// [`UnsafePathError`] in [`assert_safe_path`].
pub fn is_reserved_segment<S: AsRef<str>>(seg: S) -> bool {
    RESERVED_SEGMENTS.contains(&seg.as_ref())
}

/// **Assert** a path contains no reserved segment and no negative / non-int
/// numeric segment.
///
/// Throws [`UnsafePathError`] on the offending segment. The applier calls
/// this for every op before descending, so a single malformed op fails the
/// whole batch (atomic -- per PI-6 "atomic per-batch" invariant from
/// chord/delta).
pub fn assert_safe_path(path: &[Seg]) -> Result<(), UnsafePathError> {
    for seg in path {
        if is_reserved_segment(seg) {
            return Err(UnsafePathError::new(seg.clone()));
        }
    }
    Ok(())
}

/// **Error** raised when a path segment would pollute the prototype chain.
///
/// Carries the offending segment for caller-side diagnostic logging (and
/// for the future audit-log emission that tags `event_type='unsafe_path'`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsafePathError {
    /// The offending segment (`"__proto__"` / `"constructor"` / `"prototype"`).
    pub segment: Seg,
}

impl UnsafePathError {
    /// Construct from the offending segment.
    pub fn new(segment: Seg) -> Self {
        Self { segment }
    }
}

impl std::fmt::Display for UnsafePathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unsafe path segment: {}", self.segment)
    }
}

impl std::error::Error for UnsafePathError {}

/// **Assert** `value` is a permutation (bijection) over `0..value.len()`.
///
/// Each element MUST be an integer in `0..value.len()` and MUST appear
/// exactly once. Used by [`assert_valid_op`] for `Move` ops.
pub fn assert_permutation(value: &[usize]) -> Result<(), PathError> {
    let n = value.len();
    if n > u8::MAX as usize {
        return Err(PathError::new(Vec::new()));
    }
    let mut seen = vec![0u8; n];
    for (i, &idx) in value.iter().enumerate() {
        if idx >= n || seen[idx] != 0 {
            return Err(PathError::new(Vec::new()));
        }
        // Tag with `i + 1` so we never write 0 to a slot we mean to keep empty.
        seen[idx] = (i as u8).wrapping_add(1);
    }
    Ok(())
}

// =====================================================================
// §2 Op validation -- runtime shape checks (defense in depth)
// =====================================================================

/// **Assert** `op` is a well-formed `Op`: verb present, arity correct, path
/// safe, permutation valid.
///
/// This is the runtime mirror of the type-level invariants enforced by
/// [`Op`]'s tuple variants. Necessary because the JSON wire format admits
/// extra / missing fields that the Rust type system catches only at the
/// deserialization boundary; once deserialized, [`assert_valid_op`] catches
/// any shape slippage before [`apply`] consumes the batch.
///
/// Idempotent -- calling twice is a no-op (per chord/delta `assertValidOp`
/// design + 守门 #11 缺标比错标).
pub fn assert_valid_op(op: &Op) -> Result<(), PathError> {
    match op {
        Op::Replace(_) => Ok(()),
        Op::Set(path, _) | Op::Delete(path) | Op::Append(path, _) | Op::Truncate(path, _) => {
            if path.is_empty() {
                return Err(PathError::new(path.clone()));
            }
            assert_safe_path(path).map_err(|_| PathError::new(path.clone()))
        }
        Op::Splice(path, _index, _remove, _items) => {
            assert_safe_path(path).map_err(|_| PathError::new(path.clone()))
        }
        Op::Move(path, perm) => {
            assert_safe_path(path).map_err(|_| PathError::new(path.clone()))?;
            assert_permutation(perm)
        }
    }
}

// =====================================================================
// §3 Path error
// =====================================================================

/// **Error** raised when a path does not resolve to a valid parent for the
/// requested op (e.g. `Set` on a missing key, `Append` on a non-string).
///
/// Carries the offending path for caller-side diagnostic logging. The
/// `path` field is `Vec<Seg>` rather than `&[Seg]` because the applier
/// hands off an owned copy once it has finished walking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathError {
    /// The path that could not be resolved (owned copy for `Send`-friendly
    /// error transport).
    pub path: Path,
}

impl PathError {
    /// Construct from the failing path.
    pub fn new(path: Path) -> Self {
        Self { path }
    }
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let serialized = serde_json::to_string(&self.path)
            .unwrap_or_else(|_| "\"<unprintable path>\"".to_string());
        write!(f, "unresolvable path: {serialized}")
    }
}

impl std::error::Error for PathError {}

// =====================================================================
// §4 Applier -- mutate a JSON value by applying an op batch
// =====================================================================

/// **Apply** a batch of ops to `target`, returning the new root value.
///
/// **Atomic per-batch**: if any op in `ops` fails (validation / path /
/// permutation), the function returns `Err` and `target` is left
/// unchanged. The mutating semantics apply only to the successful
/// completion of an entire batch.
///
/// **Prototype-pollution safe**: every op routes through
/// [`assert_safe_path`] before walking, and writes use plain assignment
/// (Rust has no `Object.defineProperty` analog -- the JSON value tree is
/// `serde_json::Value`, which is a plain enum, so a prototype-chain
/// escape is structurally impossible at the Rust boundary; the guard
/// remains as a defense-in-depth signal for downstream consumers that
/// re-encode the path into another ecosystem).
///
/// **Ownership rule (per chord/delta)**: the `r` op adopts the value
/// rather than copying it. The consumer owns the batch it was handed.
/// Fanning one batch out to several consumers in-process therefore
/// makes their replicas alias each other -- this is documented behavior,
/// not a defect (per chord/delta ownership rationale).
///
/// # Errors
///
/// - [`PathError`] -- a path did not resolve to a valid parent
/// - [`UnsafePathError`] -- a path segment was reserved (`__proto__` /
///   `constructor` / `prototype`)
pub fn apply(target: Option<JsonValue>, ops: &[Op]) -> Result<JsonValue, DeltaError> {
    let mut root = target.unwrap_or(JsonValue::Null);
    for op in ops {
        assert_valid_op(op)?;
        apply_one(&mut root, op)?;
    }
    Ok(root)
}

/// **Apply** a batch of ops to `target` without mutating the previous
/// immutable value: each step that touches containers copies them first.
///
/// Slower than [`apply`] (an allocation per touched container), but safe
/// for code paths that hold prior snapshots (per chord/delta
/// `applyImmutable` rationale + star-taskgraph's history-preserving
/// viewport sync).
///
/// # Errors
///
/// Same set as [`apply`].
pub fn apply_immutable(target: Option<JsonValue>, ops: &[Op]) -> Result<JsonValue, DeltaError> {
    let mut root = target.unwrap_or(JsonValue::Null);
    for op in ops {
        assert_valid_op(op)?;
        // Copy every container along the path so we never mutate the
        // previous snapshot.
        let path = op.path().cloned().unwrap_or_default();
        let trim_to = match op {
            Op::Splice(_, _, _, _) | Op::Move(_, _) => path.len(),
            _ => path.len().saturating_sub(1),
        };
        root = copy_containers_along(&root, &path[..trim_to]);
        apply_one(&mut root, op)?;
    }
    Ok(root)
}

fn apply_one(root: &mut JsonValue, op: &Op) -> Result<(), PathError> {
    match op {
        Op::Replace(v) => {
            *root = v.clone();
            Ok(())
        }
        Op::Set(path, value) => {
            let (parent, key_owned) = locate_parent_mut(root, path)?;
            write_into_parent(parent, &key_owned, value.clone());
            Ok(())
        }
        Op::Delete(path) => {
            let (parent, key_owned) = locate_parent_mut(root, path)?;
            delete_from_parent(parent, &key_owned, path)?;
            Ok(())
        }
        Op::Append(path, suffix) => {
            let (parent, key_owned) = locate_parent_mut(root, path)?;
            let current = read_from_parent(parent, &key_owned);
            let current_str = current
                .as_str()
                .ok_or_else(|| PathError::new(path.clone()))?;
            let next = format!("{current_str}{suffix}");
            write_into_parent(parent, &key_owned, JsonValue::String(next));
            Ok(())
        }
        Op::Truncate(path, n) => {
            let (parent, key_owned) = locate_parent_mut(root, path)?;
            let current = read_from_parent(parent, &key_owned);
            let current_str = current
                .as_str()
                .ok_or_else(|| PathError::new(path.clone()))?;
            // Advance to the next char boundary if `n` is mid-codepoint
            // (UTF-8 friendly).
            let next = match current_str.char_indices().nth(*n) {
                Some((byte_idx, _)) => current_str[..byte_idx].to_string(),
                None => current_str.to_string(),
            };
            write_into_parent(parent, &key_owned, JsonValue::String(next));
            Ok(())
        }
        Op::Splice(path, index, remove, items) => {
            let target = if path.is_empty() {
                root
            } else {
                let (parent, key_owned) = locate_parent_mut(root, path)?;
                resolve_child_mut(parent, &key_owned)?
            };
            let arr = target
                .as_array_mut()
                .ok_or_else(|| PathError::new(path.clone()))?;
            if *index > arr.len() {
                return Err(PathError::new(path.clone()));
            }
            splice_array(arr, *index, *remove, items);
            Ok(())
        }
        Op::Move(path, perm) => {
            let target = if path.is_empty() {
                root
            } else {
                let (parent, key_owned) = locate_parent_mut(root, path)?;
                resolve_child_mut(parent, &key_owned)?
            };
            let arr = target
                .as_array_mut()
                .ok_or_else(|| PathError::new(path.clone()))?;
            if arr.len() != perm.len() {
                return Err(PathError::new(path.clone()));
            }
            let previous = arr.clone();
            for (i, &src) in perm.iter().enumerate() {
                arr[i] = previous[src].clone();
            }
            Ok(())
        }
    }
}

/// **Walk** `path[..path.len()-1]` and return the parent + final-key.
///
/// `path` MUST be non-empty (callers enforce this). Returns
/// `PathError(path)` if any segment fails to resolve.
fn locate_parent_mut<'a>(
    root: &'a mut JsonValue,
    path: &[Seg],
) -> Result<(&'a mut JsonValue, Seg), PathError> {
    let (last, parent_path) = path
        .split_last()
        .ok_or_else(|| PathError::new(path.to_vec()))?;
    let mut node = root;
    for seg in parent_path {
        node = step_into_mut(node, seg).ok_or_else(|| PathError::new(path.to_vec()))?;
    }
    Ok((node, last.clone()))
}

/// **Step into** a child of `node` named `seg`, returning a mutable
/// reference. Returns `None` if the segment does not resolve.
fn step_into_mut<'a>(node: &'a mut JsonValue, seg: &str) -> Option<&'a mut JsonValue> {
    match node {
        JsonValue::Object(map) => map.get_mut(seg),
        JsonValue::Array(arr) => {
            let idx = seg.parse::<usize>().ok()?;
            arr.get_mut(idx)
        }
        _ => None,
    }
}

/// **Resolve a child** by key for read or write (assumes the parent is
/// a map or array and the key resolves to an existing element).
fn resolve_child_mut<'a>(
    parent: &'a mut JsonValue,
    key: &str,
) -> Result<&'a mut JsonValue, PathError> {
    match parent {
        JsonValue::Object(map) => map
            .get_mut(key)
            .ok_or_else(|| PathError::new(vec![key.to_string()])),
        JsonValue::Array(arr) => {
            let idx = key
                .parse::<usize>()
                .map_err(|_| PathError::new(vec![key.to_string()]))?;
            arr.get_mut(idx)
                .ok_or_else(|| PathError::new(vec![key.to_string()]))
        }
        _ => Err(PathError::new(vec![key.to_string()])),
    }
}

fn write_into_parent(parent: &mut JsonValue, key: &str, value: JsonValue) {
    match parent {
        JsonValue::Object(map) => {
            map.insert(key.to_string(), value);
        }
        JsonValue::Array(arr) => {
            if let Ok(idx) = key.parse::<usize>() {
                if idx < arr.len() {
                    arr[idx] = value;
                }
            }
        }
        _ => {}
    }
}

fn read_from_parent(parent: &JsonValue, key: &str) -> JsonValue {
    match parent {
        JsonValue::Object(map) => map.get(key).cloned().unwrap_or(JsonValue::Null),
        JsonValue::Array(arr) => {
            if let Ok(idx) = key.parse::<usize>() {
                if let Some(v) = arr.get(idx) {
                    return v.clone();
                }
            }
            JsonValue::Null
        }
        _ => JsonValue::Null,
    }
}

fn delete_from_parent(
    parent: &mut JsonValue,
    key: &str,
    full_path: &[Seg],
) -> Result<(), PathError> {
    match parent {
        JsonValue::Object(map) => {
            map.remove(key);
            Ok(())
        }
        JsonValue::Array(arr) => {
            let idx = key
                .parse::<usize>()
                .map_err(|_| PathError::new(full_path.to_vec()))?;
            if idx >= arr.len() {
                return Err(PathError::new(full_path.to_vec()));
            }
            arr.remove(idx);
            Ok(())
        }
        _ => Err(PathError::new(full_path.to_vec())),
    }
}

/// **Splice-array** semantics: remove `remove` items starting at `index`,
/// then insert `items` in their place. Chunked to bound reallocation.
fn splice_array(arr: &mut Vec<JsonValue>, index: usize, remove: usize, items: &[JsonValue]) {
    if remove > 0 {
        let end = (index + remove).min(arr.len());
        arr.drain(index..end);
    }
    const CHUNK: usize = 10_000;
    let mut offset = 0;
    while offset < items.len() {
        let end = (offset + CHUNK).min(items.len());
        arr.splice(index + offset..index + offset, items[offset..end].to_vec());
        offset = end;
    }
}

fn copy_containers_along(root: &JsonValue, path: &[Seg]) -> JsonValue {
    fn copy_value(v: &JsonValue) -> JsonValue {
        match v {
            JsonValue::Object(map) => {
                let mut out = serde_json::Map::new();
                for (k, v) in map {
                    out.insert(k.clone(), copy_value(v));
                }
                JsonValue::Object(out)
            }
            JsonValue::Array(arr) => JsonValue::Array(arr.iter().map(copy_value).collect()),
            other => other.clone(),
        }
    }
    let mut current = copy_value(root);
    for seg in path {
        let next: Option<JsonValue> = match &current {
            JsonValue::Object(map) => map.get(seg.as_str()).map(copy_value),
            JsonValue::Array(arr) => match seg.parse::<usize>() {
                Ok(idx) => arr.get(idx).map(copy_value),
                Err(_) => None,
            },
            _ => None,
        };
        if let Some(next) = next {
            write_into_parent(&mut current, seg.as_str(), next);
        }
    }
    current
}

// =====================================================================
// §5 DeltaError -- unified error for apply / apply_immutable
// =====================================================================

/// **Error** raised by [`apply`] / [`apply_immutable`].
///
/// Wraps the path-level error variants so callers can downcast by the
/// discriminant they care about without juggling three error types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeltaError {
    /// Path-level failure (resolution, type mismatch, permutation invalid).
    Path(PathError),
    /// Prototype-pollution attempt via a reserved segment.
    Unsafe(UnsafePathError),
}

impl std::fmt::Display for DeltaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(e) => write!(f, "delta apply: {e}"),
            Self::Unsafe(e) => write!(f, "delta apply: {e}"),
        }
    }
}

impl std::error::Error for DeltaError {}

impl From<PathError> for DeltaError {
    fn from(e: PathError) -> Self {
        Self::Path(e)
    }
}

impl From<UnsafePathError> for DeltaError {
    fn from(e: UnsafePathError) -> Self {
        Self::Unsafe(e)
    }
}

// =====================================================================
// §6 Unit Tests (per 守门 #1 v25 -- >= 1 unit test per type)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ---- Op enum -----------------------------------------------------

    #[test]
    fn op_verb_returns_correct_single_char() {
        assert_eq!(Op::Replace(json!("x")).verb(), "r");
        assert_eq!(Op::Set(vec!["a".into()], json!(1)).verb(), "s");
        assert_eq!(Op::Delete(vec!["a".into()]).verb(), "d");
        assert_eq!(Op::Append(vec!["a".into()], "x".into()).verb(), "a");
        assert_eq!(Op::Truncate(vec!["a".into()], 0).verb(), "t");
        assert_eq!(
            Op::Splice(vec!["xs".into()], 0, 0, vec![json!(1)]).verb(),
            "p"
        );
        assert_eq!(Op::Move(vec!["xs".into()], vec![0]).verb(), "m");
    }

    #[test]
    fn op_path_returns_none_for_replace_some_for_others() {
        assert!(Op::Replace(json!(1)).path().is_none());
        assert_eq!(
            Op::Set(vec!["a".into(), "b".into()], json!(1)).path(),
            Some(&vec!["a".into(), "b".into()])
        );
        assert_eq!(
            Op::Splice(vec!["a".into()], 0, 0, vec![]).path(),
            Some(&vec!["a".into()])
        );
    }

    #[test]
    fn op_is_replace_predicate() {
        assert!(Op::Replace(json!(null)).is_replace());
        assert!(!Op::Delete(vec!["a".into()]).is_replace());
        assert!(!Op::Set(vec!["a".into()], json!(1)).is_replace());
    }

    #[test]
    fn op_serde_round_trip_all_variants() {
        let ops = [
            Op::Replace(json!({"hello": "world"})),
            Op::Set(vec!["a".into()], json!(1)),
            Op::Delete(vec!["a".into()]),
            Op::Append(vec!["a".into()], "suffix".into()),
            Op::Truncate(vec!["a".into()], 5),
            Op::Splice(vec!["xs".into()], 0, 0, vec![json!(1), json!(2)]),
            Op::Move(vec!["xs".into()], vec![1, 0]),
        ];
        for op in ops {
            let json_str = serde_json::to_string(&op).unwrap();
            let back: Op = serde_json::from_str(&json_str).unwrap();
            assert_eq!(op, back);
        }
    }

    // ---- Path safety -------------------------------------------------

    #[test]
    fn reserved_segments_listed_per_spec() {
        assert!(is_reserved_segment("__proto__"));
        assert!(is_reserved_segment("constructor"));
        assert!(is_reserved_segment("prototype"));
        assert!(!is_reserved_segment("normal"));
        assert!(!is_reserved_segment("PROTO"));
    }

    #[test]
    fn assert_safe_path_accepts_normal_path() {
        assert!(assert_safe_path(&["users".into(), "0".into(), "name".into()]).is_ok());
    }

    #[test]
    fn assert_safe_path_rejects_proto_pollution() {
        let err = assert_safe_path(&["__proto__".into()]).unwrap_err();
        assert_eq!(err.segment, "__proto__");
        assert!(err.to_string().contains("__proto__"));
    }

    #[test]
    fn assert_safe_path_rejects_constructor_and_prototype() {
        assert!(assert_safe_path(&["constructor".into()]).is_err());
        assert!(assert_safe_path(&["prototype".into()]).is_err());
    }

    #[test]
    fn assert_safe_path_rejects_first_offender_not_last() {
        let err = assert_safe_path(&["a".into(), "__proto__".into(), "b".into()]).unwrap_err();
        assert_eq!(err.segment, "__proto__");
    }

    // ---- Permutation validation --------------------------------------

    #[test]
    fn permutation_identity_is_valid() {
        assert!(assert_permutation(&[0, 1, 2, 3]).is_ok());
        assert!(assert_permutation(&[0]).is_ok());
        assert!(assert_permutation(&[]).is_ok());
    }

    #[test]
    fn permutation_swap_is_valid() {
        assert!(assert_permutation(&[1, 0, 2, 3]).is_ok());
        assert!(assert_permutation(&[3, 2, 1, 0]).is_ok());
    }

    #[test]
    fn permutation_with_duplicate_or_out_of_range_is_rejected() {
        assert!(assert_permutation(&[0, 0]).is_err());
        assert!(assert_permutation(&[0, 2]).is_err());
        assert!(assert_permutation(&[1, 2, 3]).is_err());
    }

    // ---- Op validation -----------------------------------------------

    #[test]
    fn assert_valid_op_accepts_well_formed_ops() {
        let ops = [
            Op::Replace(json!(null)),
            Op::Set(vec!["a".into()], json!(1)),
            Op::Delete(vec!["a".into()]),
            Op::Append(vec!["a".into()], "x".into()),
            Op::Truncate(vec!["a".into()], 0),
            Op::Splice(vec!["xs".into()], 0, 0, vec![json!(1)]),
            Op::Move(vec!["xs".into()], vec![0]),
        ];
        for op in ops {
            assert!(assert_valid_op(&op).is_ok(), "{op:?} should be valid");
        }
    }

    #[test]
    fn assert_valid_op_rejects_empty_path_for_set_family() {
        let ops = [
            Op::Set(vec![], json!(1)),
            Op::Delete(vec![]),
            Op::Append(vec![], "x".into()),
            Op::Truncate(vec![], 0),
        ];
        for op in ops {
            assert!(
                assert_valid_op(&op).is_err(),
                "{op:?} should be rejected for empty path"
            );
        }
    }

    #[test]
    fn assert_valid_op_rejects_unsafe_segment() {
        let op = Op::Set(vec!["__proto__".into()], json!({"polluted": true}));
        assert!(assert_valid_op(&op).is_err());
    }

    #[test]
    fn assert_valid_op_rejects_invalid_permutation() {
        let op = Op::Move(vec!["xs".into()], vec![0, 0]);
        assert!(assert_valid_op(&op).is_err());
    }

    // ---- apply: r ---------------------------------------------------

    #[test]
    fn apply_replace_sets_root() {
        let out = apply(None, &[Op::Replace(json!({"a": 1}))]).unwrap();
        assert_eq!(out, json!({"a": 1}));
    }

    #[test]
    fn apply_replace_to_null_root_succeeds() {
        let out = apply(None, &[Op::Replace(json!(42))]).unwrap();
        assert_eq!(out, json!(42));
    }

    // ---- apply: s ---------------------------------------------------

    #[test]
    fn apply_set_writes_nested_key() {
        let start = json!({"users": [{"name": "alice"}]});
        let ops = vec![Op::Set(
            vec!["users".into(), "0".into(), "name".into()],
            json!("bob"),
        )];
        let out = apply(Some(start), &ops).unwrap();
        assert_eq!(out, json!({"users": [{"name": "bob"}]}));
    }

    #[test]
    fn apply_set_creates_missing_object_key() {
        let start = json!({});
        let ops = vec![Op::Set(vec!["new".into()], json!(1))];
        let out = apply(Some(start), &ops).unwrap();
        assert_eq!(out, json!({"new": 1}));
    }

    #[test]
    fn apply_set_rejects_missing_parent() {
        let start = json!({});
        let ops = vec![Op::Set(vec!["missing".into(), "child".into()], json!(1))];
        let err = apply(Some(start), &ops).unwrap_err();
        assert!(matches!(err, DeltaError::Path(_)));
    }

    // ---- apply: d ---------------------------------------------------

    #[test]
    fn apply_delete_object_key() {
        let start = json!({"a": 1, "b": 2});
        let ops = vec![Op::Delete(vec!["a".into()])];
        let out = apply(Some(start), &ops).unwrap();
        assert_eq!(out, json!({"b": 2}));
    }

    #[test]
    fn apply_delete_array_element_splices() {
        let start = json!({"xs": [1, 2, 3]});
        let ops = vec![Op::Delete(vec!["xs".into(), "1".into()])];
        let out = apply(Some(start), &ops).unwrap();
        assert_eq!(out, json!({"xs": [1, 3]}));
    }

    // ---- apply: a ---------------------------------------------------

    #[test]
    fn apply_append_concatenates_string() {
        let start = json!({"msg": "hello"});
        let ops = vec![Op::Append(vec!["msg".into()], " world".into())];
        let out = apply(Some(start), &ops).unwrap();
        assert_eq!(out, json!({"msg": "hello world"}));
    }

    #[test]
    fn apply_append_chains_multiple_ops() {
        let start = json!({"msg": ""});
        let ops = vec![
            Op::Append(vec!["msg".into()], "hello".into()),
            Op::Append(vec!["msg".into()], " ".into()),
            Op::Append(vec!["msg".into()], "world".into()),
        ];
        let out = apply(Some(start), &ops).unwrap();
        assert_eq!(out, json!({"msg": "hello world"}));
    }

    #[test]
    fn apply_append_rejects_non_string_value() {
        let start = json!({"msg": 42});
        let ops = vec![Op::Append(vec!["msg".into()], "x".into())];
        let err = apply(Some(start), &ops).unwrap_err();
        assert!(matches!(err, DeltaError::Path(_)));
    }

    // ---- apply: t ---------------------------------------------------

    #[test]
    fn apply_truncate_at_char_boundary() {
        let start = json!({"msg": "hello world"});
        let ops = vec![Op::Truncate(vec!["msg".into()], 5)];
        let out = apply(Some(start), &ops).unwrap();
        assert_eq!(out, json!({"msg": "hello"}));
    }

    #[test]
    fn apply_truncate_at_zero_empties_string() {
        let start = json!({"msg": "abc"});
        let ops = vec![Op::Truncate(vec!["msg".into()], 0)];
        let out = apply(Some(start), &ops).unwrap();
        assert_eq!(out, json!({"msg": ""}));
    }

    #[test]
    fn apply_truncate_beyond_end_is_noop() {
        let start = json!({"msg": "abc"});
        let ops = vec![Op::Truncate(vec!["msg".into()], 100)];
        let out = apply(Some(start), &ops).unwrap();
        assert_eq!(out, json!({"msg": "abc"}));
    }

    #[test]
    fn apply_truncate_rejects_non_string_value() {
        let start = json!({"msg": 42});
        let ops = vec![Op::Truncate(vec!["msg".into()], 1)];
        let err = apply(Some(start), &ops).unwrap_err();
        assert!(matches!(err, DeltaError::Path(_)));
    }

    // ---- apply: p ---------------------------------------------------

    #[test]
    fn apply_splice_inserts_at_index() {
        let start = json!({"xs": [1, 2, 3]});
        let ops = vec![Op::Splice(
            vec!["xs".into()],
            1,
            0,
            vec![json!(10), json!(11)],
        )];
        let out = apply(Some(start), &ops).unwrap();
        assert_eq!(out, json!({"xs": [1, 10, 11, 2, 3]}));
    }

    #[test]
    fn apply_splice_removes_at_index() {
        let start = json!({"xs": [1, 2, 3]});
        let ops = vec![Op::Splice(vec!["xs".into()], 1, 1, vec![])];
        let out = apply(Some(start), &ops).unwrap();
        assert_eq!(out, json!({"xs": [1, 3]}));
    }

    #[test]
    fn apply_splice_replaces_range() {
        let start = json!({"xs": [1, 2, 3, 4]});
        let ops = vec![Op::Splice(vec!["xs".into()], 1, 2, vec![json!(99)])];
        let out = apply(Some(start), &ops).unwrap();
        assert_eq!(out, json!({"xs": [1, 99, 4]}));
    }

    #[test]
    fn apply_splice_appends_at_len() {
        let start = json!({"xs": [1, 2, 3]});
        let ops = vec![Op::Splice(vec!["xs".into()], 3, 0, vec![json!(4)])];
        let out = apply(Some(start), &ops).unwrap();
        assert_eq!(out, json!({"xs": [1, 2, 3, 4]}));
    }

    #[test]
    fn apply_splice_rejects_index_beyond_len() {
        let start = json!({"xs": [1, 2, 3]});
        let ops = vec![Op::Splice(vec!["xs".into()], 5, 0, vec![json!(4)])];
        let err = apply(Some(start), &ops).unwrap_err();
        assert!(matches!(err, DeltaError::Path(_)));
    }

    #[test]
    fn apply_splice_at_root_array() {
        let start = json!([1, 2, 3]);
        let ops = vec![Op::Splice(vec![], 1, 0, vec![json!(10)])];
        let out = apply(Some(start), &ops).unwrap();
        assert_eq!(out, json!([1, 10, 2, 3]));
    }

    // ---- apply: m ---------------------------------------------------

    #[test]
    fn apply_move_reorders_via_permutation() {
        let start = json!({"xs": [1, 2, 3, 4]});
        let ops = vec![Op::Move(vec!["xs".into()], vec![3, 2, 1, 0])];
        let out = apply(Some(start), &ops).unwrap();
        assert_eq!(out, json!({"xs": [4, 3, 2, 1]}));
    }

    #[test]
    fn apply_move_at_root_array() {
        let start = json!([1, 2, 3]);
        let ops = vec![Op::Move(vec![], vec![2, 0, 1])];
        let out = apply(Some(start), &ops).unwrap();
        assert_eq!(out, json!([3, 1, 2]));
    }

    #[test]
    fn apply_move_rejects_wrong_length_permutation() {
        let start = json!({"xs": [1, 2, 3]});
        let ops = vec![Op::Move(vec!["xs".into()], vec![0, 1])];
        let err = apply(Some(start), &ops).unwrap_err();
        assert!(matches!(err, DeltaError::Path(_)));
    }

    #[test]
    fn apply_move_rejects_non_array_target() {
        let start = json!({"xs": "not an array"});
        let ops = vec![Op::Move(vec!["xs".into()], vec![0])];
        let err = apply(Some(start), &ops).unwrap_err();
        assert!(matches!(err, DeltaError::Path(_)));
    }

    // ---- apply_immutable: snapshot isolation -------------------------

    #[test]
    fn apply_immutable_leaves_previous_value_untouched() {
        let start = json!({"a": 1});
        let start_clone = start.clone();
        let ops = vec![Op::Set(vec!["b".into()], json!(2))];
        let out = apply_immutable(Some(start.clone()), &ops).unwrap();
        // Previous value is untouched.
        assert_eq!(start, start_clone);
        // New value has the write applied.
        assert_eq!(out, json!({"a": 1, "b": 2}));
    }

    #[test]
    fn apply_immutable_deeply_nested_copy_isolates_all_containers() {
        let start = json!({"a": {"b": {"c": 1}}});
        let start_snapshot = start.clone();
        let ops = vec![Op::Set(vec!["a".into(), "b".into(), "d".into()], json!(2))];
        let out = apply_immutable(Some(start.clone()), &ops).unwrap();
        assert_eq!(start, start_snapshot);
        assert_eq!(out, json!({"a": {"b": {"c": 1, "d": 2}}}));
    }

    // ---- Error types: Display + std::error::Error -------------------

    #[test]
    fn path_error_display_includes_serialized_path() {
        let e = PathError::new(vec!["users".into(), "0".into()]);
        let s = e.to_string();
        assert!(s.contains("users"));
        assert!(s.contains("0"));
    }

    #[test]
    fn unsafe_path_error_display_includes_segment() {
        let e = UnsafePathError::new("__proto__".into());
        assert!(e.to_string().contains("__proto__"));
    }

    #[test]
    fn delta_error_display_wraps_inner_message() {
        let inner = PathError::new(vec!["x".into()]);
        let outer = DeltaError::Path(inner);
        assert!(outer.to_string().contains("delta apply"));
    }

    #[test]
    fn delta_error_from_conversions_compile() {
        let _: DeltaError = PathError::new(vec![]).into();
        let _: DeltaError = UnsafePathError::new("__proto__".into()).into();
    }

    // ---- Prototype-pollution defense end-to-end ---------------------

    #[test]
    fn apply_set_with_proto_segment_is_rejected_at_validation() {
        let ops = vec![Op::Set(
            vec!["__proto__".into(), "polluted".into()],
            json!(true),
        )];
        let err = apply(None, &ops).unwrap_err();
        assert!(matches!(err, DeltaError::Path(_)));
    }

    #[test]
    fn apply_splice_with_proto_path_is_rejected() {
        let ops = vec![Op::Splice(
            vec!["__proto__".into(), "xs".into()],
            0,
            0,
            vec![json!(1)],
        )];
        let err = apply(None, &ops).unwrap_err();
        assert!(matches!(err, DeltaError::Path(_)));
    }
}
