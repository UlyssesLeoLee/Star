//! Plugin lifecycle state machine with drain semantics (W12 第一阶段, WBS 7.13).
//!
//! 骨架期不加载任何插件代码进主进程 (W12 验收要求 "插件不能加载进主进程"):
//! 本模块只管理描述元数据 (id/version/digest) 与生命周期状态, 不持有任何
//! 插件实现或二进制。真实插件执行将由隔离 worker + capability broker 调度
//! —— 均属未完成范围。
//!
//! 状态机: `Installed → Active → Draining → Drained → Uninstalled`
//!
//! 热卸载语义 = **先停新准入 → drain 存量 → 再卸载**:
//! - `begin_drain` 将 Active 置为 Draining, 立即拒绝新的 `admit`;
//! - `complete_drain` 只在在途租约 (`active_leases`) 归零后允许 Draining → Drained;
//! - `uninstall` 只允许来自 Drained (或从未激活的 Installed)。
//!
//! 升级路径: `upgrade` 注册新版本 (Installed) 并对当前 Active 版本开始
//! drain; 调用方在租约归零后 complete_drain + uninstall 旧版本, 再激活新版本。

/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"crates/domain-agent/src/plugin_lifecycle.rs"})
MERGE (descriptor:Symbol {id:"crates/domain-agent/src/plugin_lifecycle.rs::PluginDescriptor",kind:"type"})
MERGE (state:Symbol {id:"crates/domain-agent/src/plugin_lifecycle.rs::PluginLifecycleState",kind:"enum"})
MERGE (lease:Symbol {id:"crates/domain-agent/src/plugin_lifecycle.rs::PluginLease",kind:"type"})
MERGE (record:Symbol {id:"crates/domain-agent/src/plugin_lifecycle.rs::PluginRecord",kind:"type"})
MERGE (manager:Symbol {id:"crates/domain-agent/src/plugin_lifecycle.rs::PluginLifecycleManager",kind:"type"})
MERGE (error:Symbol {id:"crates/domain-agent/src/plugin_lifecycle.rs::PluginLifecycleError",kind:"enum"})
MERGE (transition_check:Symbol {id:"crates/domain-agent/src/plugin_lifecycle.rs::check_lifecycle_transition",kind:"function"})
MERGE (install:Symbol {id:"crates/domain-agent/src/plugin_lifecycle.rs::PluginLifecycleManager.install",kind:"method"})
MERGE (activate:Symbol {id:"crates/domain-agent/src/plugin_lifecycle.rs::PluginLifecycleManager.activate",kind:"method"})
MERGE (admit:Symbol {id:"crates/domain-agent/src/plugin_lifecycle.rs::PluginLifecycleManager.admit",kind:"method"})
MERGE (begin_drain:Symbol {id:"crates/domain-agent/src/plugin_lifecycle.rs::PluginLifecycleManager.begin_drain",kind:"method"})
MERGE (complete_drain:Symbol {id:"crates/domain-agent/src/plugin_lifecycle.rs::PluginLifecycleManager.complete_drain",kind:"method"})
MERGE (uninstall:Symbol {id:"crates/domain-agent/src/plugin_lifecycle.rs::PluginLifecycleManager.uninstall",kind:"method"})
MERGE (upgrade:Symbol {id:"crates/domain-agent/src/plugin_lifecycle.rs::PluginLifecycleManager.upgrade",kind:"method"})
MERGE (self)-[:DEFINES]->(descriptor)
MERGE (self)-[:DEFINES]->(state)
MERGE (self)-[:DEFINES]->(lease)
MERGE (self)-[:DEFINES]->(record)
MERGE (self)-[:DEFINES]->(manager)
MERGE (self)-[:DEFINES]->(error)
MERGE (self)-[:DEFINES]->(transition_check)
MERGE (self)-[:DEFINES]->(install)
MERGE (self)-[:DEFINES]->(activate)
MERGE (self)-[:DEFINES]->(admit)
MERGE (self)-[:DEFINES]->(begin_drain)
MERGE (self)-[:DEFINES]->(complete_drain)
MERGE (self)-[:DEFINES]->(uninstall)
MERGE (self)-[:DEFINES]->(upgrade)
MERGE (manager)-[:USES_TYPE]->(record)
MERGE (manager)-[:USES_TYPE]->(lease)
MERGE (record)-[:USES_TYPE]->(descriptor)
MERGE (record)-[:USES_TYPE]->(state)
MERGE (install)-[:CALLS]->(transition_check)
MERGE (activate)-[:CALLS]->(transition_check)
MERGE (begin_drain)-[:CALLS]->(transition_check)
MERGE (complete_drain)-[:CALLS]->(transition_check)
MERGE (uninstall)-[:CALLS]->(transition_check)
MERGE (upgrade)-[:CALLS]->(begin_drain)
MERGE (tests:Symbol {id:"crates/domain-agent/src/plugin_lifecycle.rs::tests",kind:"module"})
MERGE (self)-[:DEFINES]->(tests)
MERGE (tests)-[:TESTS]->(manager)
@endcypher
*/

use crate::capability_profile::CapabilityProfileDigest;
use std::collections::HashMap;
use thiserror::Error;

const MAX_PLUGIN_ID_BYTES: usize = 64;
const MAX_PLUGIN_VERSION_BYTES: usize = 64;

/// Plugin descriptor metadata pinned at install time.
///
/// The descriptor carries no plugin code and no secret material; the digest is
/// the artifact/content digest supplied by the trusted resolver.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginDescriptor {
    plugin_id: String,
    version: String,
    digest: CapabilityProfileDigest,
}

impl PluginDescriptor {
    /// Creates a descriptor after validating its id and version grammar.
    pub fn new(
        plugin_id: impl Into<String>,
        version: impl Into<String>,
        digest: CapabilityProfileDigest,
    ) -> Result<Self, PluginLifecycleError> {
        let plugin_id = plugin_id.into();
        let version = version.into();
        if !valid_plugin_id(&plugin_id) {
            return Err(PluginLifecycleError::InvalidPluginId(plugin_id));
        }
        if !valid_plugin_version(&version) {
            return Err(PluginLifecycleError::InvalidPluginVersion(version));
        }
        Ok(Self {
            plugin_id,
            version,
            digest,
        })
    }

    /// Returns the stable plugin identifier.
    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }

    /// Returns the plugin version.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns the pinned artifact/content digest.
    pub fn digest(&self) -> CapabilityProfileDigest {
        self.digest
    }
}

/// Plugin lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginLifecycleState {
    /// Installed but not yet accepting work.
    Installed,
    /// Accepting new work.
    Active,
    /// New admission stopped; existing leases are draining.
    Draining,
    /// All leases released; ready to be uninstalled.
    Drained,
    /// Terminal state; the record is kept for audit.
    Uninstalled,
}

impl PluginLifecycleState {
    /// Returns the stable wire identifier for this state.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Installed => "INSTALLED",
            Self::Active => "ACTIVE",
            Self::Draining => "DRAINING",
            Self::Drained => "DRAINED",
            Self::Uninstalled => "UNINSTALLED",
        }
    }

    /// Returns whether new work may be admitted in this state.
    pub fn admits_new_work(&self) -> bool {
        matches!(self, Self::Active)
    }

    /// Returns whether this is a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Uninstalled)
    }
}

/// Strict lifecycle transition table.
///
/// Any transition outside this table is rejected as [`PluginLifecycleError::InvalidTransition`].
pub fn check_lifecycle_transition(
    from: PluginLifecycleState,
    to: PluginLifecycleState,
) -> Result<(), PluginLifecycleError> {
    use PluginLifecycleState::*;
    let allowed = matches!(
        (from, to),
        (Installed, Active)
            | (Installed, Uninstalled)
            | (Active, Draining)
            | (Draining, Drained)
            | (Draining, Active)
            | (Drained, Uninstalled)
    );
    if !allowed {
        return Err(PluginLifecycleError::InvalidTransition {
            from: from.as_str().to_string(),
            to: to.as_str().to_string(),
        });
    }
    Ok(())
}

/// An admitted in-flight lease; releasing it returns one unit of capacity.
///
/// A lease is tied to one specific plugin version at admission time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginLease {
    plugin_key: String,
}

impl PluginLease {
    /// Returns the internal plugin key (id@version) this lease was admitted for.
    pub fn plugin_key(&self) -> &str {
        &self.plugin_key
    }
}

/// One plugin version record: descriptor, lifecycle state, and in-flight leases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginRecord {
    descriptor: PluginDescriptor,
    state: PluginLifecycleState,
    active_leases: u32,
}

impl PluginRecord {
    /// Returns the pinned descriptor.
    pub fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    /// Returns the current lifecycle state.
    pub fn state(&self) -> PluginLifecycleState {
        self.state
    }

    /// Returns the number of not-yet-released leases.
    pub fn active_leases(&self) -> u32 {
        self.active_leases
    }
}

/// Pure-logic plugin lifecycle manager (no clocks, no IO, no plugin code).
///
/// Records are keyed by `plugin_id@version`, so several versions of one plugin
/// may coexist during a hot upgrade while the old version drains.
#[derive(Default)]
pub struct PluginLifecycleManager {
    plugins: HashMap<String, PluginRecord>,
}

impl PluginLifecycleManager {
    /// Creates an empty manager.
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Registers a new plugin version in the `Installed` state.
    pub fn install(&mut self, descriptor: PluginDescriptor) -> Result<(), PluginLifecycleError> {
        let key = Self::key(descriptor.plugin_id(), descriptor.version());
        if self.plugins.contains_key(&key) {
            return Err(PluginLifecycleError::AlreadyExists(key));
        }
        self.plugins.insert(
            key,
            PluginRecord {
                descriptor,
                state: PluginLifecycleState::Installed,
                active_leases: 0,
            },
        );
        Ok(())
    }

    /// Activates an installed plugin version (`Installed → Active`).
    pub fn activate(&mut self, plugin_id: &str, version: &str) -> Result<(), PluginLifecycleError> {
        let key = Self::key(plugin_id, version);
        self.transition(&key, PluginLifecycleState::Active)
    }

    /// Admits one unit of work to an active plugin version.
    ///
    /// Fails closed with [`PluginLifecycleError::AdmissionStopped`] unless the
    /// version is `Active`.
    pub fn admit(
        &mut self,
        plugin_id: &str,
        version: &str,
    ) -> Result<PluginLease, PluginLifecycleError> {
        let key = Self::key(plugin_id, version);
        let record = self
            .plugins
            .get_mut(&key)
            .ok_or_else(|| PluginLifecycleError::NotFound(key.clone()))?;
        if !record.state.admits_new_work() {
            return Err(PluginLifecycleError::AdmissionStopped(key));
        }
        record.active_leases = record.active_leases.saturating_add(1);
        Ok(PluginLease { plugin_key: key })
    }

    /// Stops new admission and enters drain (`Active → Draining`).
    pub fn begin_drain(
        &mut self,
        plugin_id: &str,
        version: &str,
    ) -> Result<(), PluginLifecycleError> {
        let key = Self::key(plugin_id, version);
        self.transition(&key, PluginLifecycleState::Draining)
    }

    /// Aborts a drain and resumes admission (`Draining → Active`).
    pub fn cancel_drain(
        &mut self,
        plugin_id: &str,
        version: &str,
    ) -> Result<(), PluginLifecycleError> {
        let key = Self::key(plugin_id, version);
        self.transition(&key, PluginLifecycleState::Active)
    }

    /// Completes a drain (`Draining → Drained`) only when no leases are outstanding.
    pub fn complete_drain(
        &mut self,
        plugin_id: &str,
        version: &str,
    ) -> Result<(), PluginLifecycleError> {
        let key = Self::key(plugin_id, version);
        let record = self
            .plugins
            .get_mut(&key)
            .ok_or_else(|| PluginLifecycleError::NotFound(key.clone()))?;
        if record.state != PluginLifecycleState::Draining {
            return Err(PluginLifecycleError::InvalidTransition {
                from: record.state.as_str().to_string(),
                to: PluginLifecycleState::Drained.as_str().to_string(),
            });
        }
        if record.active_leases > 0 {
            return Err(PluginLifecycleError::LeasesOutstanding {
                plugin_id: key,
                leases: record.active_leases,
            });
        }
        record.state = PluginLifecycleState::Drained;
        Ok(())
    }

    /// Uninstalls a drained (or never-activated) version; terminal state.
    ///
    /// Active or draining versions are rejected: hot unload must drain first.
    pub fn uninstall(
        &mut self,
        plugin_id: &str,
        version: &str,
    ) -> Result<(), PluginLifecycleError> {
        let key = Self::key(plugin_id, version);
        self.transition(&key, PluginLifecycleState::Uninstalled)
    }

    /// Registers a newer version and starts draining the currently active one.
    ///
    /// The old version moves `Active → Draining` (admission stops immediately);
    /// the caller releases outstanding leases, then `complete_drain` +
    /// `uninstall` the old version and `activate` the new one.
    pub fn upgrade(
        &mut self,
        plugin_id: &str,
        current_version: &str,
        new_descriptor: PluginDescriptor,
    ) -> Result<(), PluginLifecycleError> {
        if new_descriptor.plugin_id() != plugin_id {
            return Err(PluginLifecycleError::UpgradePluginIdMismatch(
                new_descriptor.plugin_id().to_string(),
            ));
        }
        let old_key = Self::key(plugin_id, current_version);
        let new_key = Self::key(plugin_id, new_descriptor.version());
        if new_descriptor.version() == current_version {
            return Err(PluginLifecycleError::UpgradeVersionIdentical(old_key));
        }
        let record = self
            .plugins
            .get(&old_key)
            .ok_or_else(|| PluginLifecycleError::NotFound(old_key.clone()))?;
        if record.state != PluginLifecycleState::Active {
            return Err(PluginLifecycleError::UpgradeSourceNotActive(old_key));
        }
        if self.plugins.contains_key(&new_key) {
            return Err(PluginLifecycleError::AlreadyExists(new_key));
        }
        self.plugins.insert(
            new_key,
            PluginRecord {
                descriptor: new_descriptor,
                state: PluginLifecycleState::Installed,
                active_leases: 0,
            },
        );
        self.begin_drain(plugin_id, current_version)
    }

    /// Releases one previously admitted lease back to its plugin version.
    pub fn release_lease(&mut self, lease: PluginLease) -> Result<(), PluginLifecycleError> {
        let record = self
            .plugins
            .get_mut(&lease.plugin_key)
            .ok_or_else(|| PluginLifecycleError::NotFound(lease.plugin_key.clone()))?;
        if record.active_leases == 0 {
            return Err(PluginLifecycleError::InvalidLease(lease.plugin_key));
        }
        record.active_leases -= 1;
        Ok(())
    }

    /// Returns the record for a plugin version, if present.
    pub fn record(&self, plugin_id: &str, version: &str) -> Option<&PluginRecord> {
        self.plugins.get(&Self::key(plugin_id, version))
    }

    /// Returns the lifecycle state of a plugin version, if present.
    pub fn state(&self, plugin_id: &str, version: &str) -> Option<PluginLifecycleState> {
        self.record(plugin_id, version).map(|record| record.state())
    }

    /// Returns the number of outstanding leases of a plugin version, if present.
    pub fn leases(&self, plugin_id: &str, version: &str) -> Option<u32> {
        self.record(plugin_id, version)
            .map(|record| record.active_leases())
    }

    /// Returns all known versions of a plugin in sorted order.
    pub fn versions(&self, plugin_id: &str) -> Vec<String> {
        let prefix = format!("{plugin_id}@");
        let mut versions: Vec<String> = self
            .plugins
            .keys()
            .filter(|key| key.starts_with(&prefix))
            .map(|key| key[prefix.len()..].to_string())
            .collect();
        versions.sort();
        versions
    }

    fn key(plugin_id: &str, version: &str) -> String {
        format!("{plugin_id}@{version}")
    }

    fn transition(
        &mut self,
        key: &str,
        to: PluginLifecycleState,
    ) -> Result<(), PluginLifecycleError> {
        let record = self
            .plugins
            .get_mut(key)
            .ok_or_else(|| PluginLifecycleError::NotFound(key.to_string()))?;
        check_lifecycle_transition(record.state, to)?;
        record.state = to;
        Ok(())
    }
}

/// Validation errors for plugin descriptors and lifecycle operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PluginLifecycleError {
    /// Plugin id did not match the restricted identifier grammar.
    #[error("invalid plugin id: {0}")]
    InvalidPluginId(String),
    /// Plugin version did not match the restricted version grammar.
    #[error("invalid plugin version: {0}")]
    InvalidPluginVersion(String),
    /// No record exists for the referenced plugin version.
    #[error("plugin not found: {0}")]
    NotFound(String),
    /// A record for the same plugin id and version already exists.
    #[error("plugin already exists: {0}")]
    AlreadyExists(String),
    /// The requested state change is outside the lifecycle transition table.
    #[error("invalid plugin state transition: {from} -> {to}")]
    InvalidTransition {
        /// Transition source state.
        from: String,
        /// Transition target state.
        to: String,
    },
    /// New admission was requested for a version that is not `Active`.
    #[error("plugin admission stopped: {0}")]
    AdmissionStopped(String),
    /// A drain cannot complete while leases are still outstanding.
    #[error("plugin {plugin_id} still has {leases} outstanding lease(s)")]
    LeasesOutstanding {
        /// Plugin key with outstanding leases.
        plugin_id: String,
        /// Number of not-yet-released leases.
        leases: u32,
    },
    /// A lease was released twice or never admitted.
    #[error("invalid lease: {0}")]
    InvalidLease(String),
    /// An upgrade was requested for a version that is not `Active`.
    #[error("upgrade requires current version active: {0}")]
    UpgradeSourceNotActive(String),
    /// An upgrade must target a version different from the current one.
    #[error("upgrade target version identical: {0}")]
    UpgradeVersionIdentical(String),
    /// The upgrade descriptor references a different plugin id.
    #[error("upgrade descriptor plugin id mismatch: {0}")]
    UpgradePluginIdMismatch(String),
}

fn valid_plugin_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PLUGIN_ID_BYTES
        && value.as_bytes()[0].is_ascii_lowercase()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b".-".contains(&byte))
}

fn valid_plugin_version(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PLUGIN_VERSION_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._+-".contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest() -> CapabilityProfileDigest {
        CapabilityProfileDigest::from_trusted_resolver([9; 32]).unwrap()
    }

    fn desc(id: &str, version: &str) -> PluginDescriptor {
        PluginDescriptor::new(id, version, digest()).unwrap()
    }

    fn installed_active(id: &str, version: &str) -> (PluginLifecycleManager, PluginLease) {
        let mut manager = PluginLifecycleManager::new();
        manager.install(desc(id, version)).unwrap();
        manager.activate(id, version).unwrap();
        let lease = manager.admit(id, version).unwrap();
        (manager, lease)
    }

    #[test]
    fn descriptor_validates_id_and_version_grammar() {
        assert_eq!(
            PluginDescriptor::new("-bad", "1.0", digest()),
            Err(PluginLifecycleError::InvalidPluginId("-bad".to_string()))
        );
        assert_eq!(
            PluginDescriptor::new("lint", "", digest()),
            Err(PluginLifecycleError::InvalidPluginVersion(String::new()))
        );
        assert_eq!(
            PluginDescriptor::new("lint", "a/b", digest()),
            Err(PluginLifecycleError::InvalidPluginVersion(
                "a/b".to_string()
            ))
        );
        let ok = desc("lint", "1.0.1+build-3");
        assert_eq!(ok.plugin_id(), "lint");
        assert_eq!(ok.version(), "1.0.1+build-3");
    }

    #[test]
    fn install_rejects_duplicate_id_and_version() {
        let mut manager = PluginLifecycleManager::new();
        manager.install(desc("lint", "1.0")).unwrap();
        assert_eq!(
            manager.install(desc("lint", "1.0")),
            Err(PluginLifecycleError::AlreadyExists("lint@1.0".to_string()))
        );
        // 同 id 不同 version 允许并存 (upgrade 路径)。
        assert!(manager.install(desc("lint", "2.0")).is_ok());
        assert_eq!(
            manager.versions("lint"),
            vec!["1.0".to_string(), "2.0".to_string()]
        );
    }

    #[test]
    fn strict_transition_table() {
        use PluginLifecycleState::*;
        let valid = [
            (Installed, Active),
            (Installed, Uninstalled),
            (Active, Draining),
            (Draining, Drained),
            (Draining, Active),
            (Drained, Uninstalled),
        ];
        for (from, to) in valid {
            assert!(
                check_lifecycle_transition(from, to).is_ok(),
                "{from:?} -> {to:?}"
            );
        }
        let invalid = [
            (Installed, Draining),
            (Active, Active),
            (Active, Uninstalled),
            (Draining, Uninstalled),
            (Drained, Active),
            (Uninstalled, Installed),
        ];
        for (from, to) in invalid {
            assert!(
                check_lifecycle_transition(from, to).is_err(),
                "{from:?} -> {to:?} must be rejected"
            );
        }
    }

    #[test]
    fn admit_before_activate_fails_closed() {
        let mut manager = PluginLifecycleManager::new();
        manager.install(desc("lint", "1.0")).unwrap();
        assert_eq!(
            manager.admit("lint", "1.0"),
            Err(PluginLifecycleError::AdmissionStopped(
                "lint@1.0".to_string()
            ))
        );
    }

    #[test]
    fn hot_unload_stops_admission_then_drains_then_uninstalls() {
        let (mut manager, first) = installed_active("lint", "1.0");
        let second = manager.admit("lint", "1.0").unwrap();

        // 热卸载第一步: 停新准入。
        manager.begin_drain("lint", "1.0").unwrap();
        assert_eq!(
            manager.state("lint", "1.0"),
            Some(PluginLifecycleState::Draining)
        );
        assert_eq!(
            manager.admit("lint", "1.0"),
            Err(PluginLifecycleError::AdmissionStopped(
                "lint@1.0".to_string()
            ))
        );

        // 存量未清空前不能完成 drain。
        assert_eq!(
            manager.complete_drain("lint", "1.0"),
            Err(PluginLifecycleError::LeasesOutstanding {
                plugin_id: "lint@1.0".to_string(),
                leases: 2,
            })
        );

        manager.release_lease(first).unwrap();
        assert_eq!(manager.leases("lint", "1.0"), Some(1));
        assert_eq!(
            manager.complete_drain("lint", "1.0"),
            Err(PluginLifecycleError::LeasesOutstanding {
                plugin_id: "lint@1.0".to_string(),
                leases: 1,
            })
        );

        manager.release_lease(second).unwrap();
        assert_eq!(manager.leases("lint", "1.0"), Some(0));
        manager.complete_drain("lint", "1.0").unwrap();
        assert_eq!(
            manager.state("lint", "1.0"),
            Some(PluginLifecycleState::Drained)
        );

        manager.uninstall("lint", "1.0").unwrap();
        assert_eq!(
            manager.state("lint", "1.0"),
            Some(PluginLifecycleState::Uninstalled)
        );
        assert!(manager.record("lint", "1.0").unwrap().state().is_terminal());
    }

    #[test]
    fn double_release_of_lease_fails_closed() {
        let (mut manager, lease) = installed_active("lint", "1.0");
        manager.release_lease(lease.clone()).unwrap();
        assert_eq!(
            manager.release_lease(lease),
            Err(PluginLifecycleError::InvalidLease("lint@1.0".to_string()))
        );
    }

    #[test]
    fn uninstall_requires_drain_or_never_activated() {
        // Active → 拒绝 (必须先 drain)。
        let (mut manager, _) = installed_active("lint", "1.0");
        assert_eq!(
            manager.uninstall("lint", "1.0"),
            Err(PluginLifecycleError::InvalidTransition {
                from: "ACTIVE".to_string(),
                to: "UNINSTALLED".to_string(),
            })
        );

        // 从未激活的 Installed 可以直接卸载。
        let mut fresh = PluginLifecycleManager::new();
        fresh.install(desc("fmt", "1.0")).unwrap();
        assert!(fresh.uninstall("fmt", "1.0").is_ok());
        assert_eq!(
            fresh.state("fmt", "1.0"),
            Some(PluginLifecycleState::Uninstalled)
        );
    }

    #[test]
    fn upgrade_stops_admission_then_new_version_takes_over() {
        let (mut manager, old_lease) = installed_active("lint", "1.0");

        manager.upgrade("lint", "1.0", desc("lint", "2.0")).unwrap();
        // 旧版本立即停新准入; 新版本 Installed 待激活。
        assert_eq!(
            manager.state("lint", "1.0"),
            Some(PluginLifecycleState::Draining)
        );
        assert_eq!(
            manager.state("lint", "2.0"),
            Some(PluginLifecycleState::Installed)
        );
        assert_eq!(
            manager.admit("lint", "1.0"),
            Err(PluginLifecycleError::AdmissionStopped(
                "lint@1.0".to_string()
            ))
        );

        // drain 存量 → 卸载旧版本 → 激活新版本。
        manager.release_lease(old_lease).unwrap();
        manager.complete_drain("lint", "1.0").unwrap();
        manager.uninstall("lint", "1.0").unwrap();
        manager.activate("lint", "2.0").unwrap();

        let new_lease = manager.admit("lint", "2.0").unwrap();
        assert_eq!(new_lease.plugin_key(), "lint@2.0");
        manager.release_lease(new_lease).unwrap();
    }

    #[test]
    fn upgrade_rejects_non_active_source_identical_version_and_id_mismatch() {
        let (mut manager, _) = installed_active("lint", "1.0");

        // 版本相同 → 拒绝。
        assert_eq!(
            manager.upgrade("lint", "1.0", desc("lint", "1.0")),
            Err(PluginLifecycleError::UpgradeVersionIdentical(
                "lint@1.0".to_string()
            ))
        );

        // 升级描述引用其他插件 id → 拒绝。
        assert_eq!(
            manager.upgrade("lint", "1.0", desc("fmt", "2.0")),
            Err(PluginLifecycleError::UpgradePluginIdMismatch(
                "fmt".to_string()
            ))
        );

        // 未激活的源版本 → 拒绝。
        let mut fresh = PluginLifecycleManager::new();
        fresh.install(desc("lint", "1.0")).unwrap();
        assert_eq!(
            fresh.upgrade("lint", "1.0", desc("lint", "2.0")),
            Err(PluginLifecycleError::UpgradeSourceNotActive(
                "lint@1.0".to_string()
            ))
        );
    }

    #[test]
    fn cancel_drain_resumes_admission() {
        let (mut manager, lease) = installed_active("lint", "1.0");
        manager.begin_drain("lint", "1.0").unwrap();
        assert_eq!(
            manager.admit("lint", "1.0"),
            Err(PluginLifecycleError::AdmissionStopped(
                "lint@1.0".to_string()
            ))
        );

        manager.cancel_drain("lint", "1.0").unwrap();
        assert_eq!(
            manager.state("lint", "1.0"),
            Some(PluginLifecycleState::Active)
        );
        let second = manager.admit("lint", "1.0").unwrap();

        // 租约完整归还后仍可正常热卸载。
        manager.release_lease(lease).unwrap();
        manager.release_lease(second).unwrap();
        manager.begin_drain("lint", "1.0").unwrap();
        manager.complete_drain("lint", "1.0").unwrap();
        manager.uninstall("lint", "1.0").unwrap();
    }

    #[test]
    fn unknown_plugin_operations_fail_closed() {
        let mut manager = PluginLifecycleManager::new();
        assert_eq!(
            manager.activate("ghost", "1.0"),
            Err(PluginLifecycleError::NotFound("ghost@1.0".to_string()))
        );
        assert_eq!(
            manager.admit("ghost", "1.0"),
            Err(PluginLifecycleError::NotFound("ghost@1.0".to_string()))
        );
        assert!(manager.record("ghost", "1.0").is_none());
        assert!(manager.versions("ghost").is_empty());
    }
}
