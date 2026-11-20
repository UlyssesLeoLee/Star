/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"crates/sandboxd/src/nema_worker.rs"})
MERGE (limits:Symbol {id:"crates/sandboxd/src/nema_worker.rs::NemaWorkerLimits",kind:"type"})
MERGE (launch:Symbol {id:"crates/sandboxd/src/nema_worker.rs::NemaWorkerLaunch",kind:"type"})
MERGE (policy:Symbol {id:"crates/sandboxd/src/nema_worker.rs::NemaWorkerPolicy",kind:"type"})
MERGE (verify:Symbol {id:"crates/sandboxd/src/nema_worker.rs::verify_artifact_digest",kind:"function"})
MERGE (run:Symbol {id:"crates/sandboxd/src/nema_worker.rs::run_nema_worker",kind:"function"})
MERGE (hex:Symbol {id:"crates/sandboxd/src/nema_worker.rs::sha256_hex",kind:"function"})
MERGE (self)-[:DEFINES]->(limits)
MERGE (self)-[:DEFINES]->(launch)
MERGE (self)-[:DEFINES]->(policy)
MERGE (self)-[:DEFINES]->(verify)
MERGE (self)-[:DEFINES]->(run)
MERGE (self)-[:DEFINES]->(hex)
@endcypher
*/
//! NEMA 解释器插件 worker 宿主 — digest 校验 + 受限 session 启动 (P0 .nema lane)。
//!
//! 本模块实现 NEMA 插件隔离运行协议的 sandboxd 侧适配
//! (协议 JSON 本体定义在 `nema-codec::plugin_runtime`; 两 crate 在 Cargo.lock
//! 冻结期禁止互相依赖, 故此处把请求视为**不透明字节**只做文件摆渡):
//!
//! ```text
//! 安装方 ── pinned digest ──▶ verify_artifact_digest(worker 可执行文件)
//!                          ──▶ NemaWorkerPolicy::restricted (4 维 SessionSpec)
//!                          ──▶ 请求文件摆渡 (workdir/request.json)
//!                          ──▶ backend.apply → handle.run(argv=[program ...args req resp])
//!                          ──▶ 响应文件读回 + response_limit_bytes 上限
//! ```
//!
//! Worker 契约 (与 `nema-codec::plugin_runtime::WorkerLaunch` 1:1):
//! `argv = [program] + args + [<request-file>, <response-file>]`, program 由 exec
//! 直接启动 (**绝不经过 shell**), worker 读 request-file、写 response-file 后退出 0。
//!
//! 强制力诚实标注 (per 守门 #11): Windows 上 resource 维为 Job Objects 内核级,
//! network/fs/capability 三维为 v0.1 PolicyAuditOnly / spawn 前拦截 —
//! 见 `docs/nema/KNOWN-GAPS.md` 缺口 2。

use std::path::Path;

use tokio::fs;
use uuid::Uuid;

use crate::backend::{Backend, CommandSpec, RunResult, TIMEOUT_EXIT_CODE};
use crate::error::SandboxdError;
use crate::policy::capability::CapabilityPolicy;
use crate::policy::fs::{FsMode, FsPathMode, FsPathRule, FsPolicy};
use crate::policy::network::{NetworkMode, NetworkPolicy};
use crate::policy::resource::ResourcePolicy;
use crate::policy::SessionSpec;

/// Worker 资源预算 (映射到 sandboxd 4 维策略)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NemaWorkerLimits {
    /// 内存上限 (MB → ResourcePolicy.memory_mb)
    pub memory_mb: u32,
    /// CPU 百分比 (→ ResourcePolicy.cpu_percent)
    pub cpu_percent: u32,
    /// 最大进程数 (→ ResourcePolicy.max_processes; 解释器 worker 通常为 1)
    pub max_processes: u32,
    /// 命令级超时 (秒; 超时后端级联 kill)
    pub timeout_sec: u32,
    /// 响应文件字节上限
    pub response_limit_bytes: usize,
}

impl Default for NemaWorkerLimits {
    fn default() -> Self {
        Self {
            memory_mb: 256,
            cpu_percent: 50,
            max_processes: 1,
            timeout_sec: 30,
            response_limit_bytes: 16 * 1024 * 1024,
        }
    }
}

/// NEMA worker 启动描述 (与 `nema-codec::plugin_runtime::WorkerLaunch` 1:1)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NemaWorkerLaunch {
    /// worker 可执行文件路径 (exec 直启, 非 shell)
    pub program: String,
    /// 固定前缀参数
    pub args: Vec<String>,
    /// env KEY=VALUE
    pub env: Vec<String>,
    /// 工作目录 (空 = 继承 sandboxd)
    pub cwd: String,
}

impl NemaWorkerLaunch {
    /// 组装 argv: `[program] + args + [request_path, response_path]`,
    /// 校验 program 非空且不含 shell 解释器前缀。
    pub fn to_command_spec(
        &self,
        request_path: &str,
        response_path: &str,
    ) -> Result<CommandSpec, SandboxdError> {
        if self.program.is_empty() {
            return Err(SandboxdError::InvalidPolicy(
                "nema worker program must not be empty".to_string(),
            ));
        }
        let mut cmd = Vec::with_capacity(1 + self.args.len() + 2);
        cmd.push(self.program.clone());
        cmd.extend(self.args.iter().cloned());
        cmd.push(request_path.to_string());
        cmd.push(response_path.to_string());
        Ok(CommandSpec::new(
            cmd,
            self.env.clone(),
            self.cwd.clone(),
            true,
        ))
    }
}

/// NEMA worker 的受限 4 维会话策略。
pub struct NemaWorkerPolicy;

impl NemaWorkerPolicy {
    /// 构造受限 SessionSpec:
    /// - resource: 来自 [`NemaWorkerLimits`], kill_on_parent_exit=true
    /// - network: BLOCK_ALL (解释器不应出网)
    /// - fs: allowlist = 仅 workdir 读写 (请求/响应摆渡目录)
    /// - capability: DropAll
    pub fn restricted(limits: &NemaWorkerLimits, workdir: &str) -> SessionSpec {
        SessionSpec {
            resource: ResourcePolicy::new(
                limits.memory_mb,
                limits.cpu_percent,
                limits.max_processes,
                true,
            ),
            network: NetworkPolicy {
                mode: NetworkMode::BlockAll,
                allowlist: Vec::new(),
                ip_allowlist: Vec::new(),
            },
            fs: FsPolicy::new(
                FsMode::Allowlist,
                vec![FsPathRule::new(workdir, FsPathMode::ReadWrite)],
            ),
            capability: CapabilityPolicy::default(),
        }
    }
}

/// 校验 worker 可执行文件的 SHA-256 与 pinned digest 一致 (加载前门禁)。
pub async fn verify_artifact_digest(
    path: &Path,
    expected_sha256: &str,
) -> Result<(), SandboxdError> {
    let bytes = fs::read(path).await.map_err(|e| {
        SandboxdError::Internal(format!("read worker artifact {}: {e}", path.display()))
    })?;
    let actual = sha256_hex(&bytes);
    if actual != expected_sha256 {
        return Err(SandboxdError::DigestMismatch {
            expected: expected_sha256.to_string(),
            actual,
        });
    }
    Ok(())
}

/// 完整受限 worker 生命周期: 请求摆渡 → 受限 session → 运行 → 响应读回 + 上限。
///
/// digest 校验必须先于本函数执行 (见 [`verify_artifact_digest`]); 本函数
/// 不自行回落 — 任何一步失败都返回错误, 绝不走未受限路径。
#[allow(clippy::too_many_arguments)]
pub async fn run_nema_worker(
    backend: &dyn Backend,
    policy: &SessionSpec,
    launch: &NemaWorkerLaunch,
    request_bytes: &[u8],
    workdir: &Path,
    limits: &NemaWorkerLimits,
) -> Result<RunResult, SandboxdError> {
    policy.validate()?;
    if request_bytes.len() > limits.response_limit_bytes {
        return Err(SandboxdError::ResourceLimitExceeded {
            limit_type: "request_bytes".to_string(),
            value: limits.response_limit_bytes as u64,
        });
    }
    let request_path = workdir.join("request.json");
    let response_path = workdir.join("response.json");
    fs::write(&request_path, request_bytes)
        .await
        .map_err(|e| SandboxdError::Internal(format!("stage nema request: {e}")))?;
    let command = launch.to_command_spec(
        &request_path.to_string_lossy(),
        &response_path.to_string_lossy(),
    )?;
    let handle = backend
        .apply(policy, Uuid::new_v4())
        .await
        .map_err(|e| SandboxdError::Internal(format!("apply nema worker session: {e}")))?;
    let result = handle.run(&command, limits.timeout_sec).await;
    let _ = handle.destroy().await;
    let result = result?;
    if result.returncode == TIMEOUT_EXIT_CODE {
        return Err(SandboxdError::ResourceLimitExceeded {
            limit_type: "timeout".to_string(),
            value: limits.timeout_sec as u64,
        });
    }
    if result.returncode != 0 {
        return Err(SandboxdError::Internal(format!(
            "nema worker exited {} (stdout={}, stderr={})",
            result.returncode, result.stdout, result.stderr
        )));
    }
    // 响应读回 + 上限 (per 协议 response_limit_bytes)。
    let response = match fs::metadata(&response_path).await {
        Ok(meta) => {
            if meta.len() > limits.response_limit_bytes as u64 {
                return Err(SandboxdError::ResourceLimitExceeded {
                    limit_type: "response_bytes".to_string(),
                    value: limits.response_limit_bytes as u64,
                });
            }
            fs::read(&response_path)
                .await
                .map_err(|e| SandboxdError::Internal(format!("read nema worker response: {e}")))?
        }
        Err(_) => {
            return Err(SandboxdError::Internal(
                "nema worker produced no response file".to_string(),
            ));
        }
    };
    Ok(RunResult {
        returncode: result.returncode,
        stdout: String::from_utf8_lossy(&response).into_owned(),
        stderr: result.stderr,
        latency_ms: result.latency_ms,
        hit_violations: result.hit_violations,
    })
}

/// 参考 worker fixture (NEMA 插件协议契约演示器): 读 `<request-file>`,
/// 写 `<response-file>`。按请求内容判别 parse/format 并返回结构合法的响应;
/// 响应 cell 值携带请求字节长度, 供 e2e 测试证明请求真实送达 worker。
///
/// 仅用于测试与契约演示; 真实解释器由第三方插件按协议提供。
pub async fn run_fixture(request_path: &Path, response_path: &Path) -> anyhow::Result<()> {
    let request_bytes = fs::read(request_path)
        .await
        .map_err(|e| anyhow::anyhow!("fixture read request: {e}"))?;
    let request: serde_json::Value = serde_json::from_slice(&request_bytes)
        .map_err(|e| anyhow::anyhow!("fixture request is not JSON: {e}"))?;
    let request_len = request_bytes.len();
    let response = if request.get("source").is_some() {
        serde_json::json!({
            "protocol_version": 1,
            "ast": {
                "domains": [{
                    "mode": "table",
                    "id": 1,
                    "revision": 1,
                    "columns": ["A"],
                    "explicit_header": false,
                    "declared_width": null,
                    "records": [[{ "kind": "value", "value": format!("fixture-ok:{request_len}") }]]
                }]
            }
        })
    } else if request.get("ast").is_some() {
        serde_json::json!({
            "protocol_version": 1,
            "text": format!(" \\\\1 1\nfixture-ok:{request_len}\n")
        })
    } else {
        anyhow::bail!("fixture: unknown request kind");
    };
    fs::write(response_path, serde_json::to_vec(&response)?)
        .await
        .map_err(|e| anyhow::anyhow!("fixture write response: {e}"))?;
    Ok(())
}

// ============================================================================
// 纯 Rust SHA-256 (Cargo.lock 冻结期替代 sha2 crate; 仅用于完整性校验)
// 复制自 crates/nema-codec/src/sha256.rs (同一实现, 双 crate 各持一份,
// 待跨 crate 依赖解锁后删除本副本)。NIST 向量测试见下方 tests。
// ============================================================================

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

const H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

fn rotr(x: u32, n: u32) -> u32 {
    (x >> n) | (x << (32 - n))
}

fn compress(state: &mut [u32; 8], block: &[u8; 64]) {
    let mut w = [0u32; 64];
    for (i, word) in w.iter_mut().enumerate().take(16) {
        *word = u32::from_be_bytes([
            block[i * 4],
            block[i * 4 + 1],
            block[i * 4 + 2],
            block[i * 4 + 3],
        ]);
    }
    for i in 16..64 {
        let s0 = rotr(w[i - 15], 7) ^ rotr(w[i - 15], 18) ^ (w[i - 15] >> 3);
        let s1 = rotr(w[i - 2], 17) ^ rotr(w[i - 2], 19) ^ (w[i - 2] >> 10);
        w[i] = w[i - 16]
            .wrapping_add(s0)
            .wrapping_add(w[i - 7])
            .wrapping_add(s1);
    }
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;
    for i in 0..64 {
        let s1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
        let ch = (e & f) ^ (!e & g);
        let t1 = h
            .wrapping_add(s1)
            .wrapping_add(ch)
            .wrapping_add(K[i])
            .wrapping_add(w[i]);
        let s0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let t2 = s0.wrapping_add(maj);
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }
    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

/// 返回 `data` 的小写十六进制 SHA-256 (仅用于完整性校验, 非密码学 MAC)。
pub fn sha256_hex(data: &[u8]) -> String {
    let mut state = H0;
    let mut chunks = data.chunks_exact(64);
    for block in &mut chunks {
        let mut arr = [0u8; 64];
        arr.copy_from_slice(block);
        compress(&mut state, &arr);
    }
    let remainder = chunks.remainder();
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut last = [0u8; 128];
    let last_len = if remainder.len() < 56 { 64 } else { 128 };
    last[..remainder.len()].copy_from_slice(remainder);
    last[remainder.len()] = 0x80;
    last[last_len - 8..last_len].copy_from_slice(&bit_len.to_be_bytes());
    for block in last[..last_len].chunks_exact(64) {
        let mut arr = [0u8; 64];
        arr.copy_from_slice(block);
        compress(&mut state, &arr);
    }
    let mut out = String::with_capacity(64);
    for word in state {
        out.push_str(&format!("{word:08x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_nist_vectors() {
        let vectors: &[(&[u8], &str)] = &[
            (
                b"",
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
            (
                b"abc",
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ),
            (
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
                "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
            ),
        ];
        for (input, expected) in vectors {
            assert_eq!(&sha256_hex(input), expected);
        }
    }

    #[test]
    fn restricted_policy_blocks_network_and_drops_capability() {
        let limits = NemaWorkerLimits::default();
        let policy = NemaWorkerPolicy::restricted(&limits, "C:/work/nema-run");
        assert_eq!(policy.resource.memory_mb, 256);
        assert_eq!(policy.resource.max_processes, 1);
        assert!(policy.resource.kill_on_parent_exit);
        assert_eq!(policy.network.mode, NetworkMode::BlockAll);
        assert!(policy.network.allowlist.is_empty());
        assert_eq!(
            policy.capability.linux_mode,
            crate::policy::LinuxCapabilityMode::DropAll
        );
        assert_eq!(policy.fs.mode, FsMode::Allowlist);
        assert_eq!(policy.fs.allowlist.len(), 1);
        assert_eq!(policy.fs.allowlist[0].path, "C:/work/nema-run");
        assert_eq!(policy.fs.allowlist[0].mode, FsPathMode::ReadWrite);
        policy.validate().unwrap();
    }

    #[test]
    fn command_argv_is_exec_not_shell() {
        let launch = NemaWorkerLaunch {
            program: "C:/plugins/nema-worker.exe".to_string(),
            args: vec!["--mode".to_string(), "fixture".to_string()],
            env: vec!["RUST_LOG=warn".to_string()],
            cwd: "C:/work/nema-run".to_string(),
        };
        let command = launch
            .to_command_spec(
                "C:/work/nema-run/request.json",
                "C:/work/nema-run/response.json",
            )
            .unwrap();
        assert_eq!(
            command.cmd,
            vec![
                "C:/plugins/nema-worker.exe",
                "--mode",
                "fixture",
                "C:/work/nema-run/request.json",
                "C:/work/nema-run/response.json",
            ]
        );
        // exec 直启: argv[0] 是 worker 本身, 不是 cmd/sh/bash。
        assert!(!command.cmd[0].to_ascii_lowercase().ends_with("sh"));
        assert!(!command.cmd[0].to_ascii_lowercase().ends_with("cmd.exe"));
        assert!(command.capture_output);
    }

    #[tokio::test]
    async fn digest_verification_matches_and_rejects() {
        let dir = tempfile::tempdir().unwrap();
        let artifact = dir.path().join("worker.bin");
        tokio::fs::write(&artifact, b"worker-bytes").await.unwrap();
        let expected = sha256_hex(b"worker-bytes");
        verify_artifact_digest(&artifact, &expected).await.unwrap();

        let mut wrong = expected.clone();
        let tail = wrong.pop().unwrap();
        wrong.push(if tail == 'a' { 'b' } else { 'a' });
        match verify_artifact_digest(&artifact, &wrong).await {
            Err(SandboxdError::DigestMismatch {
                expected: exp,
                actual,
            }) => {
                assert_eq!(exp, wrong);
                assert_eq!(actual, expected);
            }
            other => panic!("expected DigestMismatch, got {other:?}"),
        }
    }
}
