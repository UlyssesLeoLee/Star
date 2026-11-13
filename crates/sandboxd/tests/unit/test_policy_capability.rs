/*
@cypher schema=1 source_sha256=ade18f6b1b867576c6f3c6e647c55d3f556c6bf077970d1397b0d6d0ba4b3856
MERGE (self:File {path:"crates/sandboxd/tests/unit/test_policy_capability.rs"})
@endcypher
*/
//! tests/unit/test_policy_capability.rs — capability 维策略单测 (per DD §5.1: 5 TC)
//!
//! DropAll / AllowNetBindService / Custom / 字典查询 / 5 域模板 (per D-10)。

use sandboxd::policy::capability::{
    CapabilityPolicy, LinuxCapabilityMode, RiskLevel, KNOWN_CAPABILITIES,
};
use sandboxd::policy::{Domain, SessionSpec};

#[test]
fn default_is_drop_all() {
    let p = CapabilityPolicy::default();
    assert_eq!(p.linux_mode, LinuxCapabilityMode::DropAll);
    p.validate().unwrap();
}

#[test]
fn allow_net_bind_service_valid() {
    let p = CapabilityPolicy::new(LinuxCapabilityMode::AllowNetBindService, vec![]);
    p.validate().unwrap();
}

#[test]
fn custom_requires_list() {
    let p = CapabilityPolicy::new(LinuxCapabilityMode::Custom, vec![]);
    assert!(p.validate().is_err());
    let p2 = CapabilityPolicy::new(LinuxCapabilityMode::Custom, vec!["CAP_NET_RAW".to_string()]);
    p2.validate().unwrap();
}

#[test]
fn custom_unknown_cap_rejected() {
    let p = CapabilityPolicy::new(
        LinuxCapabilityMode::Custom,
        vec!["CAP_NOT_A_THING".to_string()],
    );
    assert!(p.validate().is_err());
}

#[test]
fn dictionary_lookup() {
    // CAP_NET_BIND_SERVICE 默认 Allow (子代理 port listen 需求, per FR-3.4)
    assert_eq!(
        KNOWN_CAPABILITIES
            .iter()
            .find(|e| e.syscall_name == "CAP_NET_BIND_SERVICE")
            .unwrap()
            .risk_level,
        RiskLevel::Low
    );
    // CAP_SYS_ADMIN Critical
    assert_eq!(
        KNOWN_CAPABILITIES
            .iter()
            .find(|e| e.syscall_name == "CAP_SYS_ADMIN")
            .unwrap()
            .risk_level,
        RiskLevel::Critical
    );
    // 未知 → Critical (fail-closed)
    assert_eq!(
        CapabilityPolicy::default().lookup("CAP_UNKNOWN"),
        RiskLevel::Critical
    );
}

#[test]
fn dict_covers_three_platforms() {
    let platforms = ["Linux", "Windows", "macOS"];
    // 通过字段匹配: CapPlatform 枚举断言
    for plat in platforms {
        use sandboxd::policy::capability::CapPlatform;
        let count = KNOWN_CAPABILITIES
            .iter()
            .filter(|e| {
                matches!(
                    (plat, e.platform),
                    ("Linux", CapPlatform::Linux)
                        | ("Windows", CapPlatform::Windows)
                        | ("macOS", CapPlatform::MacOs)
                )
            })
            .count();
        assert!(count > 0, "platform {plat} has no dictionary entries");
    }
}

#[test]
fn domain_templates_differ() {
    // per D-10: player 域 vs admin 域默认值不同
    let player = SessionSpec::for_domain(Domain::Player);
    let admin = SessionSpec::for_domain(Domain::Admin);
    assert_ne!(player.resource.memory_mb, admin.resource.memory_mb);
    assert_eq!(player.resource.memory_mb, 1024);
    assert_eq!(admin.resource.memory_mb, 4096);
    assert!(admin.network.allowlist.len() > player.network.allowlist.len());
    // player/economy/match/social 相同
    for d in [Domain::Economy, Domain::Match, Domain::Social] {
        assert_eq!(
            SessionSpec::for_domain(d).resource,
            player.resource,
            "domain {d:?} should match player defaults"
        );
    }
}

#[test]
fn domain_templates_all_valid() {
    for d in Domain::ALL {
        SessionSpec::for_domain(d).validate().unwrap();
    }
}
