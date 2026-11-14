//! ═══════════════════════════════════════════════════════════════════
//! Ported from UlyssesLeoLee/SandTree v1.1 (Apache-2.0)
//! per REUSE-MAP-COLLAB-SANDBOX-001 §3 (直接复用, 保留 NOTICE).
//! Source: E:\SandTree\crates\{model,observation-model,resource-graph}
//! ═══════════════════════════════════════════════════════════════════

//! Resource node / kind / state / relation model (DD-SW §2, BD §4).

use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::model::capability::CapabilitySet;
use crate::model::id::{CorrelationId, EndpointId, PluginId, ResourceId};

/// RFC3339 timestamp, the only timestamp form crossing a boundary
/// (`schemas/observation_snapshot_v1.schema.json` uses `date-time`).
pub type Timestamp = String;

/// Resource kinds in the unified topology (BD §4).
///
/// Ordering is meaningful: it is the primary sort key for the topology view, so
/// that `Host → Sandbox/DockerRuntime → Container …` renders consistently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    /// The local host; root of the primary tree.
    Host,
    /// An isolation environment: Windows Sandbox / Docker Sandbox / Multipass instance.
    Sandbox,
    /// A Docker Engine / Moby-compatible endpoint, including one nested inside a sandbox
    /// (FR-078 Docker-in-Sandbox).
    DockerRuntime,
    /// A Docker container.
    Container,
    /// A Docker image.
    Image,
    /// A Docker volume.
    Volume,
    /// A Docker network.
    Network,
    /// A Compose project (FR-030).
    ComposeProject,
    /// A Compose service inside a project (FR-032).
    ComposeService,
    /// A workspace / VFS root inside a sandbox or container (FR-014).
    Workspace,
}

impl ResourceKind {
    /// Wire name used in DB rows and IPC payloads.
    pub fn as_str(self) -> &'static str {
        match self {
            ResourceKind::Host => "host",
            ResourceKind::Sandbox => "sandbox",
            ResourceKind::DockerRuntime => "docker-runtime",
            ResourceKind::Container => "container",
            ResourceKind::Image => "image",
            ResourceKind::Volume => "volume",
            ResourceKind::Network => "network",
            ResourceKind::ComposeProject => "compose-project",
            ResourceKind::ComposeService => "compose-service",
            ResourceKind::Workspace => "workspace",
        }
    }

    /// Wire name as produced by `serde` — **snake_case**.
    ///
    /// # 为什么要显式写出来
    ///
    /// 这个枚举有**两套写法**：
    ///
    /// * [`ResourceKind::as_str`] → **kebab-case**（`docker-runtime`）——
    ///   DB 行里的写法，也是仓内的规范写法；
    /// * `#[serde(rename_all = "snake_case")]` → **snake_case**
    ///   （`docker_runtime`）—— 走 IPC / JSON 的写法。
    ///
    /// 两套都真实存在，界面很容易只认其中一套 —— 而**认错的那一套
    /// 不会报错，只会静默落空**：按 kind 分支的判定全部走不到
    /// （`docker_runtime` 落到「其它」分支，引擎节点被当成容器，
    /// 画布的列分组也乱了）。
    ///
    /// 所以 wire 形式在这里**显式命名**，并且由测试把它和 serde 的实际输出
    /// 钉在一起 —— 改 serde 属性时测试会红，而不是让界面悄悄失配。
    pub fn as_wire(self) -> &'static str {
        match self {
            ResourceKind::Host => "host",
            ResourceKind::Sandbox => "sandbox",
            ResourceKind::DockerRuntime => "docker_runtime",
            ResourceKind::Container => "container",
            ResourceKind::Image => "image",
            ResourceKind::Volume => "volume",
            ResourceKind::Network => "network",
            ResourceKind::ComposeProject => "compose_project",
            ResourceKind::ComposeService => "compose_service",
            ResourceKind::Workspace => "workspace",
        }
    }

    /// 从**任意一种写法**解析种类。
    ///
    /// snake_case（wire）与 kebab-case（DB / 规范写法）**都认** ——
    /// 因为这两种数据来源真的会同时流到同一个消费者：
    /// DB 里存的是 kebab-case，IPC 上跑的是 snake_case。
    ///
    /// 认不出来返回 `None`，**不猜**（RD §9）。
    pub fn from_wire(s: &str) -> Option<Self> {
        let normalized = s.replace('_', "-");
        Some(match normalized.as_str() {
            "host" => ResourceKind::Host,
            "sandbox" => ResourceKind::Sandbox,
            "docker-runtime" => ResourceKind::DockerRuntime,
            "container" => ResourceKind::Container,
            "image" => ResourceKind::Image,
            "volume" => ResourceKind::Volume,
            "network" => ResourceKind::Network,
            "compose-project" => ResourceKind::ComposeProject,
            "compose-service" => ResourceKind::ComposeService,
            "workspace" => ResourceKind::Workspace,
            _ => return None,
        })
    }

    /// Every kind, used by tests and by provider capability tables.
    pub fn all() -> &'static [ResourceKind] {
        &[
            ResourceKind::Host,
            ResourceKind::Sandbox,
            ResourceKind::DockerRuntime,
            ResourceKind::Container,
            ResourceKind::Image,
            ResourceKind::Volume,
            ResourceKind::Network,
            ResourceKind::ComposeProject,
            ResourceKind::ComposeService,
            ResourceKind::Workspace,
        ]
    }
}

/// Normalized resource state (BD §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceState {
    /// Provider has not reported the resource this discovery round.
    ///
    /// Reconcile marks missing resources stale first and only tombstones them after
    /// the grace period (DD-SW §5). Absence is therefore never immediate removal.
    Unknown,
    /// Being created.
    Creating,
    /// Running / active.
    Running,
    /// Stopped but the resource still exists.
    Stopped,
    /// Paused (container semantics).
    Paused,
    /// Exited (container semantics).
    Exited,
    /// Degraded: exists, control works, observation is partial (FR-079).
    Degraded,
    /// Being destroyed.
    Destroying,
    /// Destroyed / removed.
    Destroyed,
    /// Tombstoned after exceeding the reconcile grace period.
    Tombstoned,
}

impl ResourceState {
    /// Wire name used in DB rows.
    pub fn as_str(self) -> &'static str {
        match self {
            ResourceState::Unknown => "unknown",
            ResourceState::Creating => "creating",
            ResourceState::Running => "running",
            ResourceState::Stopped => "stopped",
            ResourceState::Paused => "paused",
            ResourceState::Exited => "exited",
            ResourceState::Degraded => "degraded",
            ResourceState::Destroying => "destroying",
            ResourceState::Destroyed => "destroyed",
            ResourceState::Tombstoned => "tombstoned",
        }
    }

    /// Terminal states are never returned by discovery.
    pub fn is_terminal(self) -> bool {
        matches!(self, ResourceState::Destroyed | ResourceState::Tombstoned)
    }

    /// Parse a DB/IPC state string.
    pub fn from_wire(s: &str) -> Self {
        match s {
            "creating" => ResourceState::Creating,
            "running" => ResourceState::Running,
            "stopped" => ResourceState::Stopped,
            "paused" => ResourceState::Paused,
            "exited" => ResourceState::Exited,
            "degraded" => ResourceState::Degraded,
            "destroying" => ResourceState::Destroying,
            "destroyed" => ResourceState::Destroyed,
            "tombstoned" => ResourceState::Tombstoned,
            _ => ResourceState::Unknown,
        }
    }
}

/// The unified resource node (DD-SW §2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceNode {
    /// Stable opaque identity.
    pub id: ResourceId,
    /// Normalized kind.
    pub kind: ResourceKind,
    /// Owning provider plugin.
    pub provider_id: PluginId,
    /// Display name. Never part of identity.
    pub name: String,
    /// Normalized state.
    pub state: ResourceState,
    /// Primary tree parent. `None` only for the host root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<ResourceId>,
    /// Operations the provider declares as available (FR-061).
    ///
    /// UI must not show an action whose capability is absent, and policy must
    /// refuse it even if the UI is bypassed (FR-061, NFR-U02).
    pub capabilities: CapabilitySet,
    /// Provider-specific non-secret metadata.
    #[serde(default)]
    pub metadata: Json,
    /// Last time this provider confirmed the resource (RD §9: no guessing).
    pub last_seen: Timestamp,
}

impl ResourceNode {
    /// Minimal constructor used by discovery adapters.
    pub fn new(
        id: ResourceId,
        kind: ResourceKind,
        provider_id: PluginId,
        name: impl Into<String>,
        state: ResourceState,
        parent_id: Option<ResourceId>,
        last_seen: Timestamp,
    ) -> Self {
        Self {
            id,
            kind,
            provider_id,
            name: name.into(),
            state,
            parent_id,
            capabilities: CapabilitySet::empty(),
            metadata: Json::Object(Default::default()),
            last_seen,
        }
    }

    /// Builder for capabilities.
    pub fn with_capabilities(mut self, caps: CapabilitySet) -> Self {
        self.capabilities = caps;
        self
    }

    /// Builder for provider metadata.
    pub fn with_metadata(mut self, metadata: Json) -> Self {
        self.metadata = metadata;
        self
    }

    /// Read a single metadata key without leaking provider SDK types into the kernel.
    pub fn meta_str(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).and_then(Json::as_str)
    }

    /// Docker endpoint identity, when this resource belongs to one.
    ///
    /// Docker resource ids are derived from `endpoint_id + native_id` (DD-PLG §5),
    /// so the endpoint must be recoverable from metadata for nested topologies.
    pub fn endpoint_id(&self) -> Option<EndpointId> {
        self.meta_str("endpoint_id")
            .and_then(EndpointId::from_str_ok)
    }
}

/// Who put a lock on a resource, and therefore what the lock is for.
///
/// Both kinds are removed the same way — by a human, with admin verification.
/// If a `policy` lock could be lifted without verification it would not be a
/// policy; and if it could not be lifted at all, an administrator locked out
/// by their own profile would have no way out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LockKind {
    /// A human locked the resource (UI or CLI).
    Manual,
    /// A policy (profile / rule) locked the resource.
    Policy,
}

impl LockKind {
    /// Wire name stored in `resource_lock.lock_kind`.
    pub fn as_str(self) -> &'static str {
        match self {
            LockKind::Manual => "manual",
            LockKind::Policy => "policy",
        }
    }

    /// Parse a DB/wire lock kind. Unknown kinds are rejected, not guessed (RD §9).
    pub fn from_wire(s: &str) -> Option<Self> {
        Some(match s {
            "manual" => LockKind::Manual,
            "policy" => LockKind::Policy,
            _ => return None,
        })
    }
}

/// A lock sitting on one resource.
///
/// A lock applies to **the resource itself** and to nothing else: locking a
/// container does not lock its volumes or its image. Chaining locks is not
/// protection, it is collateral damage — and a delete that would take other
/// locked resources with it is refused by the provider, not silently allowed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockRecord {
    /// The locked resource.
    pub resource_id: ResourceId,
    /// When the lock was taken (RFC3339).
    pub locked_at: Timestamp,
    /// Who took it — the current user's identity.
    pub locked_by: String,
    /// Why. Free text, written by the user, so it is not a secret.
    pub reason: String,
    /// Manual or policy.
    pub kind: LockKind,
}

impl LockRecord {
    /// Build a record.
    pub fn new(
        resource_id: ResourceId,
        locked_at: Timestamp,
        locked_by: impl Into<String>,
        reason: impl Into<String>,
        kind: LockKind,
    ) -> Self {
        Self {
            resource_id,
            locked_at,
            locked_by: locked_by.into(),
            reason: reason.into(),
            kind,
        }
    }
}

/// How an administrator proves they are one.
///
/// The three are **backups for each other**: a machine with no fingerprint
/// reader still unlocks with a PIN or a password.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationMethod {
    /// Windows Hello user-presence check (`UserConsentVerifier`).
    Fingerprint,
    /// A short personal code.
    Pin,
    /// A longer passphrase.
    Password,
}

impl VerificationMethod {
    /// Wire name used on the IPC surface and in `admin_credential.method`.
    pub fn as_str(self) -> &'static str {
        match self {
            VerificationMethod::Fingerprint => "fingerprint",
            VerificationMethod::Pin => "pin",
            VerificationMethod::Password => "password",
        }
    }

    /// Parse a wire method name. Unknown names are rejected, not guessed (RD §9).
    pub fn from_wire(s: &str) -> Option<Self> {
        Some(match s {
            "fingerprint" => VerificationMethod::Fingerprint,
            "pin" => VerificationMethod::Pin,
            "password" => VerificationMethod::Password,
            _ => return None,
        })
    }

    /// Every method, in the order the UI lists them.
    pub fn all() -> &'static [VerificationMethod] {
        &[
            VerificationMethod::Fingerprint,
            VerificationMethod::Pin,
            VerificationMethod::Password,
        ]
    }

    /// Whether the method is checked against a stored secret.
    ///
    /// Fingerprint is **not**: there is no secret to compare — Windows Hello
    /// performs the match inside the trusted execution environment and only
    /// reports whether the check passed.
    pub fn uses_stored_secret(self) -> bool {
        matches!(self, VerificationMethod::Pin | VerificationMethod::Password)
    }
}

/// Why a biometric verification did not succeed.
///
/// These are **distinct** reasons, not one "verification failed": the user
/// has to be told whether the prompt was dismissed, the reader is missing, the
/// fingerprint is not enrolled, or a group policy forbids it — because each of
/// those needs a different thing from the user (RD §9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BiometricFailure {
    /// The user dismissed the prompt.
    Canceled,
    /// No fingerprint reader on this machine.
    DeviceNotPresent,
    /// The signed-in user has no fingerprint enrolled.
    NotConfiguredForUser,
    /// A group policy forbids Windows Hello.
    DisabledByPolicy,
    /// The reader is busy with another request.
    DeviceBusy,
    /// Too many wrong fingers.
    RetriesExhausted,
    /// The Windows Hello API itself could not be reached.
    ///
    /// Distinct from "no reader on this machine": the API failing usually
    /// means the process is not in an interactive session, which is a
    /// different thing to tell the user.
    ApiUnavailable,
    /// A result this build does not recognise — reported as-is, not guessed.
    Unknown(i32),
}

impl BiometricFailure {
    /// Wire name used on the IPC surface.
    pub fn as_str(self) -> &'static str {
        match self {
            BiometricFailure::Canceled => "canceled",
            BiometricFailure::DeviceNotPresent => "device-not-present",
            BiometricFailure::NotConfiguredForUser => "not-configured-for-user",
            BiometricFailure::DisabledByPolicy => "disabled-by-policy",
            BiometricFailure::DeviceBusy => "device-busy",
            BiometricFailure::RetriesExhausted => "retries-exhausted",
            BiometricFailure::ApiUnavailable => "api-unavailable",
            BiometricFailure::Unknown(_) => "unknown",
        }
    }
}

/// Why a biometric method cannot be used at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BiometricAvailability {
    /// It can be used.
    Available,
    /// No fingerprint reader on this machine.
    DeviceNotPresent,
    /// The signed-in user has no fingerprint enrolled.
    NotConfiguredForUser,
    /// A group policy forbids Windows Hello.
    DisabledByPolicy,
    /// The reader is busy right now.
    DeviceBusy,
    /// Not a value this build knows — reported as-is, not guessed.
    Unknown(i32),
}

impl BiometricAvailability {
    /// Wire name used on the IPC surface.
    pub fn as_str(self) -> &'static str {
        match self {
            BiometricAvailability::Available => "available",
            BiometricAvailability::DeviceNotPresent => "device-not-present",
            BiometricAvailability::NotConfiguredForUser => "not-configured-for-user",
            BiometricAvailability::DisabledByPolicy => "disabled-by-policy",
            BiometricAvailability::DeviceBusy => "device-busy",
            BiometricAvailability::Unknown(_) => "unknown",
        }
    }

    /// Whether the method can be offered to the user right now.
    pub fn is_available(self) -> bool {
        matches!(self, BiometricAvailability::Available)
    }
}

/// Why a lock-related operation was refused.
///
/// # Why this is not an error code
///
/// The registry in `schemas/error_codes.csv` is **frozen**: it carries one
/// code per failure *category*, and the design baseline pins its contents
/// (14 codes / 8 families). A lock adds four refusals that a caller must be
/// able to tell apart — "the resource is locked, ask the human to unlock it"
/// is not "the PIN was wrong, try again", and neither is "wait, you are
/// throttled" or "no PIN is configured at all".
///
/// So the code says *that* something was refused, and this enum says *which*
/// refusal it was. It is closed, typed and machine-checkable: free text would
/// not be decidable (RD §9), and a string the UI has to match on is exactly
/// the kind of contract that drifts (ADR-061).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LockFailure {
    /// The resource is locked; a destructive operation was refused.
    ResourceLocked {
        /// Whether a human or a policy put the lock on.
        lock_kind: LockKind,
        /// The reason the lock was put on, as the operator wrote it.
        reason: String,
    },
    /// The administrator's secret did not match.
    VerificationFailed {
        /// Which method was tried.
        method: VerificationMethod,
        /// How many attempts are left before the method is throttled.
        attempts_remaining: u32,
    },
    /// The requested verification method is not configured on this machine.
    MethodNotConfigured {
        /// Which method was asked for.
        method: VerificationMethod,
    },
    /// Too many failed attempts; the method is throttled until `until_ms`.
    Throttled {
        /// Which method is throttled.
        method: VerificationMethod,
        /// Epoch milliseconds until which the method stays throttled.
        until_ms: u64,
    },
    /// The biometric check did not succeed; `cause` says how.
    BiometricFailed {
        /// The platform's own reason.
        cause: BiometricFailure,
    },
}

impl LockFailure {
    /// The wire tag of this variant.
    ///
    /// One name, written once: the serde tag and this string are pinned
    /// together by a round-trip test, so the audit log, the IPC payload and
    /// the enum cannot drift apart.
    pub fn kind_name(&self) -> &'static str {
        match self {
            LockFailure::ResourceLocked { .. } => "resource_locked",
            LockFailure::VerificationFailed { .. } => "verification_failed",
            LockFailure::MethodNotConfigured { .. } => "method_not_configured",
            LockFailure::Throttled { .. } => "throttled",
            LockFailure::BiometricFailed { .. } => "biometric_failed",
        }
    }
}

/// Typed non-tree relations (FR-004, BD §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RelationKind {
    /// container → image (`uses-image`).
    UsesImage,
    /// container → volume (`mounts`).
    Mounts,
    /// container → network (`attached-network`).
    AttachedNetwork,
    /// container → compose project (`member-of-compose`).
    MemberOfCompose,
    /// host ↔ sandbox ↔ container mount/bind source (`workspace-mount`).
    WorkspaceMount,
    /// sandbox → nested docker runtime (`docker-in-sandbox`, FR-078).
    DockerInSandbox,
}

impl RelationKind {
    /// Wire name stored in `resource_relation.kind`.
    pub fn as_str(self) -> &'static str {
        match self {
            RelationKind::UsesImage => "uses-image",
            RelationKind::Mounts => "mounts",
            RelationKind::AttachedNetwork => "attached-network",
            RelationKind::MemberOfCompose => "member-of-compose",
            RelationKind::WorkspaceMount => "workspace-mount",
            RelationKind::DockerInSandbox => "docker-in-sandbox",
        }
    }

    /// Parse a DB relation kind string.
    pub fn from_wire(s: &str) -> Option<Self> {
        Some(match s {
            "uses-image" => RelationKind::UsesImage,
            "mounts" => RelationKind::Mounts,
            "attached-network" => RelationKind::AttachedNetwork,
            "member-of-compose" => RelationKind::MemberOfCompose,
            "workspace-mount" => RelationKind::WorkspaceMount,
            "docker-in-sandbox" => RelationKind::DockerInSandbox,
            _ => return None,
        })
    }
}

/// A typed edge between two resources (DD-SW §2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relation {
    /// Source resource.
    pub from: ResourceId,
    /// Target resource.
    pub to: ResourceId,
    /// Edge kind.
    pub kind: RelationKind,
    /// Non-secret edge metadata.
    #[serde(default)]
    pub metadata: Json,
}

impl Relation {
    /// Construct a relation with empty metadata.
    pub fn new(from: ResourceId, to: ResourceId, kind: RelationKind) -> Self {
        Self {
            from,
            to,
            kind,
            metadata: Json::Object(Default::default()),
        }
    }
}

/// Correlation carrier for operations and events (FR-063).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Correlation(#[serde(with = "correlation_repr")] CorrelationId);

mod correlation_repr {
    use super::CorrelationId;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(id: &CorrelationId, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(id.as_str())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<CorrelationId, D::Error> {
        let raw = String::deserialize(d)?;
        CorrelationId::parse(&raw).map_err(serde::de::Error::custom)
    }
}

impl Correlation {
    /// Fresh correlation id for a new request.
    pub fn generate() -> Self {
        Self(CorrelationId::generate())
    }

    /// Borrow the inner id.
    pub fn id(&self) -> &CorrelationId {
        &self.0
    }

    /// Parse from wire form.
    pub fn parse(raw: &str) -> Option<Self> {
        CorrelationId::parse(raw).ok().map(Self)
    }
}

impl std::fmt::Display for Correlation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `as_wire` 必须**等于 serde 的实际输出**。
    ///
    /// 这条测试是「两套写法」这件事的护栏：改了 `#[serde(rename_all)]`
    /// 却没改 `as_wire`（或反过来）时，界面就会拿着一种写法去比对另一种 ——
    /// 而且**不会报错**，只会静默失配。这里一红就说明失配要发生了。
    #[test]
    fn wire_name_matches_serde_output() {
        for kind in ResourceKind::all() {
            let json = serde_json::to_string(kind).expect("ResourceKind 可序列化");
            assert_eq!(
                json,
                format!("\"{}\"", kind.as_wire()),
                "{kind:?} 的 wire 名与 serde 输出对不上"
            );
        }
    }

    /// `from_wire` 必须同时认 snake_case（wire）与 kebab-case（DB）。
    #[test]
    fn from_wire_accepts_both_spellings() {
        for kind in ResourceKind::all() {
            assert_eq!(
                ResourceKind::from_wire(kind.as_wire()),
                Some(*kind),
                "认不出 wire 写法 {:?}",
                kind.as_wire()
            );
            assert_eq!(
                ResourceKind::from_wire(kind.as_str()),
                Some(*kind),
                "认不出规范写法 {:?}",
                kind.as_str()
            );
        }
    }

    /// 认不出来就说认不出来 —— **不猜**（RD §9）。
    #[test]
    fn from_wire_rejects_unknown_kinds() {
        assert_eq!(ResourceKind::from_wire("kubernetes"), None);
        assert_eq!(ResourceKind::from_wire(""), None);
        assert_eq!(ResourceKind::from_wire("unknown"), None);
    }

    /// 锁失败的**每一个变体**都要有名字、能过线、能回来。
    ///
    /// `kind_name()` 与 serde 的 tag 是**同一件事**写了两遍 ——
    /// 这条测试把它们钉在一起：改了 tag 没改名字（或者反过来），
    /// 审计日志里的原因就会和线上载荷对不上，而**两边都编译得过**。
    #[test]
    fn every_lock_failure_names_itself_and_round_trips() {
        let cases = [
            LockFailure::ResourceLocked {
                lock_kind: LockKind::Manual,
                reason: "还在用".to_string(),
            },
            LockFailure::VerificationFailed {
                method: VerificationMethod::Pin,
                attempts_remaining: 3,
            },
            LockFailure::MethodNotConfigured {
                method: VerificationMethod::Password,
            },
            LockFailure::Throttled {
                method: VerificationMethod::Pin,
                until_ms: 1_700_000_060_000,
            },
            LockFailure::BiometricFailed {
                cause: BiometricFailure::DeviceNotPresent,
            },
        ];
        let mut names: Vec<&str> = Vec::new();
        for case in &cases {
            let json = serde_json::to_value(case).expect("LockFailure 可序列化");
            assert_eq!(
                json["kind"].as_str(),
                Some(case.kind_name()),
                "{case:?} 的 tag 与 kind_name() 对不上"
            );
            let back: LockFailure = serde_json::from_value(json).expect("LockFailure 可反序列化");
            assert_eq!(&back, case, "过线之后原因变了");
            names.push(case.kind_name());
        }
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(names.len(), before, "两个变体共用了同一个名字");
    }

    /// 一个**不认识**的原因不能被解出来。
    ///
    /// 猜一个原因比没有原因更糟：界面会拿它去说「资源已锁定」，
    /// 而 daemon 从没说过这句话（RD §9）。
    #[test]
    fn an_unknown_lock_failure_is_rejected() {
        let raw = serde_json::json!({"kind": "no_such_reason"});
        assert!(serde_json::from_value::<LockFailure>(raw).is_err());
    }

    #[test]
    fn resource_node_round_trips() {
        // UT-001 ResourceNode serialize/deserialize
        let node = ResourceNode::new(
            ResourceId::derive(&["ep", "c1"]),
            ResourceKind::Container,
            PluginId::derive(&["sandtree.provider.docker"]),
            "web",
            ResourceState::Running,
            None,
            "2026-10-07T00:00:00Z".into(),
        )
        .with_capabilities(CapabilitySet::empty());
        let json = serde_json::to_string(&node).unwrap();
        let back: ResourceNode = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn state_wire_names_are_stable() {
        for kind in ResourceKind::all() {
            assert!(!kind.as_str().is_empty());
        }
        assert_eq!(ResourceState::from_wire("running"), ResourceState::Running);
        assert_eq!(ResourceState::from_wire("nonsense"), ResourceState::Unknown);
    }

    #[test]
    fn relation_kind_wire_round_trip() {
        for s in [
            "uses-image",
            "mounts",
            "attached-network",
            "member-of-compose",
            "workspace-mount",
            "docker-in-sandbox",
        ] {
            assert_eq!(RelationKind::from_wire(s).unwrap().as_str(), s);
        }
    }

    #[test]
    fn endpoint_id_recovered_from_metadata() {
        let ep = EndpointId::derive(&["npipe:////./pipe/docker_engine"]);
        let node = ResourceNode::new(
            ResourceId::derive(&["x"]),
            ResourceKind::DockerRuntime,
            PluginId::derive(&["p"]),
            "docker",
            ResourceState::Running,
            None,
            "2026-10-07T00:00:00Z".into(),
        )
        .with_metadata(serde_json::json!({ "endpoint_id": ep.as_str() }));
        assert_eq!(node.endpoint_id().as_ref(), Some(&ep));
    }

    /// 锁种类的写法必须唯一且能往返 —— 数据库行与 IPC 都靠它。
    #[test]
    fn lock_kind_wire_round_trip() {
        for kind in [LockKind::Manual, LockKind::Policy] {
            assert_eq!(LockKind::from_wire(kind.as_str()), Some(kind));
        }
        assert_eq!(LockKind::from_wire("nonsense"), None);
    }

    /// 三种验证方式必须都能解析，且**指纹不是**「比对一个存起来的秘密」。
    ///
    /// 这条断言守的是一条容易搞混的事实：指纹没有哈希可比 ——
    /// 匹配发生在 TPM 里，应用只看得到「通过 / 没通过」。
    #[test]
    fn verification_methods_are_parsed_and_classified() {
        assert_eq!(VerificationMethod::all().len(), 3);
        for m in VerificationMethod::all() {
            assert_eq!(VerificationMethod::from_wire(m.as_str()), Some(*m));
        }
        assert_eq!(VerificationMethod::from_wire("face"), None);
        assert!(VerificationMethod::Pin.uses_stored_secret());
        assert!(VerificationMethod::Password.uses_stored_secret());
        assert!(
            !VerificationMethod::Fingerprint.uses_stored_secret(),
            "指纹没有可比对的秘密可比"
        );
    }

    #[test]
    fn correlation_serializes_as_plain_string() {
        let c = Correlation::generate();
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(json, format!("\"{}\"", c.id().as_str()));
        assert_eq!(serde_json::from_str::<Correlation>(&json).unwrap(), c);
    }
}
