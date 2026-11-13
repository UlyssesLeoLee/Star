/*
@cypher schema=1 source_sha256=a3fa6e3d68b6923d6982a6b87c7383a797a1ff90268d174be5ac911bb8fd21dd
MERGE (self:File {path:"crates/sandboxd/tests/unit/test_policy_resource.rs"})
@endcypher
*/
//! tests/unit/test_policy_resource.rs — resource 维策略单测 (per DD §5.1: 8 TC)
//!
//! 默认值 / 边界 / 0=不限 / 大值 / 校验 / JSON 序列化 / proto 互转。

use sandboxd::policy::resource::ResourcePolicy;
use serde_json::json;

#[test]
fn defaults_match_srs() {
    // per SRS FR-3.1: 1GB mem / 50% CPU / 100 procs / kill_on_parent_exit=True
    let p = ResourcePolicy::default();
    assert_eq!(p.memory_mb, 1024);
    assert_eq!(p.cpu_percent, 50);
    assert_eq!(p.max_processes, 100);
    assert!(p.kill_on_parent_exit);
}

#[test]
fn zero_means_unlimited() {
    let p = ResourcePolicy::new(0, 0, 0, false);
    assert_eq!(p.memory_mb, 0);
    p.validate().unwrap();
}

#[test]
fn cpu_boundary_100_ok() {
    let p = ResourcePolicy::new(1024, 100, 100, true);
    p.validate().unwrap();
}

#[test]
fn cpu_over_100_invalid() {
    let p = ResourcePolicy::new(1024, 101, 100, true);
    assert!(p.validate().is_err());
}

#[test]
fn large_values_ok() {
    let p = ResourcePolicy::new(u32::MAX, 100, u32::MAX, true);
    p.validate().unwrap();
}

#[test]
fn kill_on_parent_exit_false_ok() {
    let p = ResourcePolicy::new(512, 50, 10, false);
    assert!(!p.kill_on_parent_exit);
    p.validate().unwrap();
}

#[test]
fn json_roundtrip() {
    let p = ResourcePolicy::default();
    let v = serde_json::to_value(p).unwrap();
    assert_eq!(v["memory_mb"], json!(1024));
    let back: ResourcePolicy = serde_json::from_value(v).unwrap();
    assert_eq!(back, p);
}

#[test]
fn proto_roundtrip() {
    let p = ResourcePolicy::default();
    let proto: sandboxd::proto::ResourcePolicy = (&p).into();
    assert_eq!(proto.memory_mb, 1024);
    assert!(proto.kill_on_parent_exit);
    let back = ResourcePolicy::from(&proto);
    assert_eq!(back, p);
}
