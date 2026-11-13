//! tests/unit/test_policy_network.rs — network 维策略单测 (per DD §5.1: 8 TC)
//!
//! allowlist 匹配 / 通配符 / IP CIDR / BlockAll / Passthrough / 校验。

use std::net::IpAddr;

use sandboxd::policy::network::{NetworkDecision, NetworkMode, NetworkPolicy};

#[test]
fn exact_domain_match() {
    let p = NetworkPolicy::default();
    assert_eq!(p.check_host("github.com"), NetworkDecision::Allowed);
}

#[test]
fn wildcard_subdomain_match() {
    let p = NetworkPolicy::default();
    // *.github.com 匹配任意深度子域
    assert_eq!(p.check_host("api.github.com"), NetworkDecision::Allowed);
    assert_eq!(p.check_host("a.b.github.com"), NetworkDecision::Allowed);
}

#[test]
fn wildcard_does_not_match_apex() {
    // *.github.com 不匹配 github.com 本身 (由显式条目覆盖)
    let p = NetworkPolicy::new(
        NetworkMode::Allowlist,
        vec!["*.github.com".to_string()],
        vec![],
    );
    assert!(matches!(
        p.check_host("github.com"),
        NetworkDecision::Blocked { .. }
    ));
}

#[test]
fn unknown_domain_blocked() {
    let p = NetworkPolicy::default();
    match p.check_host("evil.example.com") {
        NetworkDecision::Blocked { reason } => assert!(reason.contains("evil.example.com")),
        other => panic!("expected blocked, got {other:?}"),
    }
}

#[test]
fn block_all_blocks_everything() {
    let p = NetworkPolicy::new(NetworkMode::BlockAll, vec![], vec![]);
    assert!(matches!(
        p.check_host("github.com"),
        NetworkDecision::Blocked { .. }
    ));
    assert!(matches!(
        p.check_ip("8.8.8.8".parse::<IpAddr>().unwrap()),
        NetworkDecision::Blocked { .. }
    ));
}

#[test]
fn passthrough_allows_everything() {
    let p = NetworkPolicy::new(NetworkMode::Passthrough, vec![], vec![]);
    assert_eq!(
        p.check_host("anything.example.com"),
        NetworkDecision::Allowed
    );
    assert_eq!(
        p.check_ip("8.8.8.8".parse::<IpAddr>().unwrap()),
        NetworkDecision::Allowed
    );
}

#[test]
fn ip_allowlist_cidr() {
    let p = NetworkPolicy::new(
        NetworkMode::Allowlist,
        vec!["github.com".to_string()],
        vec!["10.0.0.0/8".to_string()],
    );
    assert_eq!(
        p.check_ip("10.1.2.3".parse::<IpAddr>().unwrap()),
        NetworkDecision::Allowed
    );
    // 域名 allowlist 无法覆盖裸 IP → 阻断 (per D-3 fail-closed)
    assert!(matches!(
        p.check_ip("8.8.8.8".parse::<IpAddr>().unwrap()),
        NetworkDecision::Blocked { .. }
    ));
}

#[test]
fn validate_rejects_bad_cidr() {
    let p = NetworkPolicy::new(
        NetworkMode::Allowlist,
        vec!["github.com".to_string()],
        vec!["not-a-cidr".to_string()],
    );
    assert!(p.validate().is_err());
}

#[test]
fn case_and_trailing_dot_normalized() {
    let p = NetworkPolicy::default();
    assert_eq!(p.check_host("GitHub.Com."), NetworkDecision::Allowed);
}
