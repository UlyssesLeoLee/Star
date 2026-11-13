//! tests/unit/test_backend_linux.rs — Linux 后端纯逻辑单测 (仅 Linux CI 跑)
//!
//! cgroup 限制构造 / netns iptables 脚本 / capsh argv / fs bind 脚本 / wrap argv。

#![cfg(target_os = "linux")]

use sandboxd::backend::linux::{
    build_capsh_prefix, build_fs_bind_script, build_netns_setup_script, build_wrap_argv,
    cgroup_cpu_max, cgroup_memory_max, cgroup_path_for, cgroup_pids_max, netns_name_for,
};
use sandboxd::backend::CommandSpec;
use sandboxd::policy::{
    CapabilityPolicy, FsMode, FsPathMode, FsPathRule, FsPolicy, LinuxCapabilityMode, NetworkMode,
    NetworkPolicy, SessionSpec,
};

#[test]
fn cgroup_limits_format() {
    assert_eq!(cgroup_memory_max(1024), Some("1024M".to_string()));
    assert_eq!(cgroup_memory_max(0), None);
    // per BD §5.2: cpu.max = "max 10000" (50% = 5000)
    assert_eq!(cgroup_cpu_max(50), Some("5000 10000".to_string()));
    assert_eq!(cgroup_cpu_max(0), None);
    assert_eq!(cgroup_pids_max(100), Some("100".to_string()));
    assert_eq!(cgroup_pids_max(0), None);
}

#[test]
fn cgroup_and_netns_names() {
    let id = uuid::Uuid::new_v4();
    assert!(cgroup_path_for(&id).starts_with("/sys/fs/cgroup/sandboxd/sandboxd-"));
    assert!(netns_name_for(&id).starts_with("sandboxd-"));
}

#[test]
fn netns_script_blocks_all_then_allows() {
    let policy = NetworkPolicy::new(
        NetworkMode::Allowlist,
        vec!["github.com".to_string()],
        vec!["10.0.0.0/8".to_string()],
    );
    let script = build_netns_setup_script(&policy);
    assert!(script.contains("iptables -A OUTPUT -o lo -j ACCEPT"));
    assert!(script.contains("iptables -A OUTPUT -j DROP"));
    assert!(script.contains("iptables -A OUTPUT -d 10.0.0.0/8 -j ACCEPT"));
    // 域名 allowlist 需 ipset → 脚本内无 (per 缺口 #6)
    assert!(!script.contains("github.com"));
}

#[test]
fn capsh_prefix_modes() {
    assert_eq!(
        build_capsh_prefix(&CapabilityPolicy::default()),
        vec!["capsh", "--drop=all", "--"]
    );
    assert_eq!(
        build_capsh_prefix(&CapabilityPolicy::new(
            LinuxCapabilityMode::AllowNetBindService,
            vec![]
        )),
        vec!["capsh", "--caps=cap_net_bind_service+ep", "--"]
    );
    assert_eq!(
        build_capsh_prefix(&CapabilityPolicy::new(
            LinuxCapabilityMode::Custom,
            vec!["CAP_NET_RAW".to_string(), "CAP_SYS_NICE".to_string()]
        )),
        vec!["capsh", "--caps=CAP_NET_RAW,CAP_SYS_NICE+ep", "--"]
    );
}

#[test]
fn fs_bind_script_read_vs_rw() {
    let policy = FsPolicy::new(
        FsMode::Allowlist,
        vec![
            FsPathRule::new("/worktree", FsPathMode::Read),
            FsPathRule::new("/tmp", FsPathMode::ReadWrite),
        ],
    );
    let script = build_fs_bind_script(&policy);
    // Read → bind + remount ro
    assert!(script.contains("mount --bind '/worktree' '/worktree'"));
    assert!(script.contains("mount -o remount,ro,bind '/worktree'"));
    // ReadWrite → 仅 bind
    assert!(script.contains("mount --bind '/tmp' '/tmp'"));
    assert!(script.contains("exec \"$@\""));
}

#[test]
fn wrap_argv_full_chain() {
    let policy = SessionSpec::default();
    let cmd = CommandSpec::new(
        vec!["echo".to_string(), "hi".to_string()],
        vec![],
        String::new(),
        true,
    );
    let argv = build_wrap_argv(&policy, Some("sandboxd-test"), &cmd);
    // netns 前缀
    assert_eq!(&argv[0..4], &["ip", "netns", "exec", "sandboxd-test"]);
    // mount ns (默认 fs allowlist 非空)
    assert!(argv.contains(&"unshare".to_string()));
    // capsh drop-all
    assert!(argv.contains(&"--drop=all".to_string()));
    // 原命令在末尾
    assert_eq!(argv[argv.len() - 2], "echo");
    assert_eq!(argv[argv.len() - 1], "hi");

    // netns None → 无 ip 前缀
    let argv2 = build_wrap_argv(&policy, None, &cmd);
    assert_ne!(argv2[0], "ip");
}
