# SANDBOX-BASIC-DESIGN-002

> **Sandbox-as-a-Service (sandboxd) — 基本設計書 v0.1** (per 日本 IPA SEC 標準 / 基本設計書 テンプレート)
>
> - 状态: 🟡 Draft v0.1 (2026-09-10 JST 初版落档, MVP-骨架)
> - 上游: `docs/requirements/SRS-SANDBOX-002.md` v0.1
> - 下游: 詳設計 (随实装迭代, per 拍板 D-1~D-6 落档)
> - 关联实装基线: `crates/sandboxd/` (新, MVP 骨架) + `scripts/automation/guardian/sandbox.py` v0.2 (现役, v0.1 降级路径) + `scripts/automation/dispatcher.py` v0.1 (现役, gRPC client 集成点) + `crates/star-mcp/` (现役, Rust 集成点)
> - 守门基线: 守门 #1+#1 v25+#5+#6+#9+#10+#11+#13+#14 v3+#14 v4+#22+#28 共 12 项必过
> - 平行参考: `docs/basic-design/OPS-BASIC-DESIGN-001.md` v0.1 (模板参考) + `docs/requirements/SRS-SANDBOX-002.md` v0.1 (上游 SRS) + `docs/architecture/SANDBOX-001.md` v0.2 (派生) + `docs/automation-design.md` v0.1 (Python 化基线)
> - 修订人: `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` (per 2026-08-27 19:39 JST 用户授权 + 守门 #10 + 守门 #14 v3)
> - 审批: `架构师 (Mavis 接手 agent per DEC-008)` (per 守门 #14 v4 反转 v0.62 2026-09-10 12:45 JST)
> - 日期: 2026-09-10 JST
> - 受众: 詳細設計エンジニア / 実装エンジニア / SRE Lead / 5 域 Lead (未到位, Mavis 临时代签 per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D)
> - 拍板来源: 2026-09-10 21:48 JST Ulysses 拍板"**agent 的沙盒设计到位了吗？没有的话，我希望沙盒组建是一个 app 形式的独立模块**" + 21:49 JST "**先把需求文档和基本设计改好**" (本 BD 落档)

---

## §0 文档目的

本文档定义 **Sandbox-as-a-Service (sandboxd)** 的基本設計, 覆蓋:

(a) 系统拓扑 (sandboxd + client + isolation backends)
(b) 模块划分 (crates/sandboxd 内部 6 模块)
(c) gRPC IPC 契约 (proto schema + 4 RPC + 4 消息)
(d) 数据模型 + W/T/M 分类 (4 表 100% 覆盖 per 守门 #13)
(e) 4 维隔离后端 (Windows + Linux + macOS 三平台)
(f) 部署与可观测 (systemd / Windows Service / launchd + Prometheus + audit log)
(g) 决策点 (D-1~D-6 跟 SRS §10 对齐)
(h) 守门 (Quality Gate per 守门 #1 v3 + v25)

**MVP 范围** (per SRS-002 §3.1 派生): 6 模块 + 4 RPC + 4 维隔离 + 3 平台后端 + fail-open 降级。

**Framework 选型** (per 守门 #1 cargo 栈):

- **gRPC**: `tonic 0.12` (跟 mavis desktop 同栈, 跟 `star-mcp` 0.12 100% 对齐)
- **async runtime**: `tokio 1.40` (跟 mavis desktop 一致)
- **序列化**: `prost 0.13` (protobuf 编译器, 跟 tonic 配套)
- **Windows API**: `windows 0.58` (Win32 bindings, Job Objects / WFP / AppContainer)
- **Linux API**: `caps 0.5` + `nix 0.28` (cgroups v2 / netns / capability)
- **macOS API**: `sandbox 0.5` + `nix 0.28` (sandbox-exec 包装)
- **Metrics**: `prometheus 0.13` (跟 mavis 现有栈一致)
- **DB**: `sqlx 0.8` (PG client, 跟 `p3d6_audit` 表结构一致, per 守门 #13)

**不**做 (per SRS-002 §1.4 派生): 加密 / vault / 跨域编排 / 真实 K8s / TLS / 多租户 / AI 行为审计 / mavis v1.0 兼容。

---

## §1 架构总览

### 1.1 系统拓扑

```
                ┌─────────────────────────────────────────────────────┐
                │  sandboxd (独立 app, 长期运行, port 50051)           │
                │  ┌──────────────────────────────────────────────┐   │
                │  │  gRPC Server (tonic 0.12)                    │   │
                │  │   - CreateSession / RunCommand /              │   │
                │  │     DestroySession / StreamLogs              │   │
                │  └────────────────┬─────────────────────────────┘   │
                │                   ▼                                  │
                │  ┌──────────────────────────────────────────────┐   │
                │  │  Sandbox Orchestrator                        │   │
                │  │   - session pool (100 默认) + LRU 回收       │   │
                │  │   - 4 维策略: resource/network/fs/capability │   │
                │  │   - session 生命周期管理                     │   │
                │  └────┬─────────────┬─────────────┬────────────┘   │
                │       │             │             │                 │
                │       ▼             ▼             ▼                 │
                │  ┌────────┐   ┌────────┐   ┌────────┐              │
                │  │Windows │   │ Linux  │   │ macOS  │              │
                │  │Backend │   │Backend │   │Backend │              │
                │  │Job Obj │   │cgroups │   │sandbox-│              │
                │  │+ WFP + │   │v2 +    │   │exec    │              │
                │  │AppCont │   │netns + │   │profile │              │
                │  │ainer   │   │mount   │   │        │              │
                │  └────────┘   └────────┘   └────────┘              │
                │  ┌──────────────────────────────────────────────┐   │
                │  │  Observability: Prometheus /metrics +        │   │
                │  │  /healthz + audit log (PG)                   │   │
                │  └──────────────────────────────────────────────┘   │
                └──────────┬──────────────────┬──────────────────┬───┘
                           │ gRPC             │ gRPC             │ gRPC
              ┌────────────▼───────┐ ┌────────▼─────────┐ ┌──────▼──────┐
              │ dispatcher.py      │ │ mavis desktop    │ │ 其他 client │
              │ (Python)           │ │ (Rust)           │ │ (any gRPC)  │
              │ v0.2: gRPC client  │ │ v0.2: gRPC client│ │             │
              │ 替代 subprocess.run │ │                  │ │             │
              │ + fail-open 降级   │ │                  │ │             │
              │ SANDBOX-001 v0.2   │ │                  │ │             │
              └────────────────────┘ └──────────────────┘ └─────────────┘
                           │
                           │ subprocess.run (降级)
                           ▼
              ┌────────────────────────┐
              │ SANDBOX-001 v0.2       │
              │ (subprocess 包装器)    │
              │ 守门 #9 v3 实证降级    │
              └────────────────────────┘
```

### 1.2 部署单元 (MVP-骨架)

| 单元 | 形态 | 端口 | 进程模型 | HA | 平台 |
|---|---|---|---|---|---|
| sandboxd | Rust binary, systemd / Win Service / launchd | 50051 (gRPC) + 50051/metrics (HTTP) + /healthz | 长期 daemon, 1 实例 | MVP 单实例, v0.2 加 HA | Win + Linux + macOS |
| sandboxd-config | YAML 配置文件 | — | — | — | 同上 |
| sandboxd-policy (PG) | 4 维策略 + audit log | 5432 (PG) | DB 进程 | 既有 HA | 同上 |
| client (dispatcher / mavis) | gRPC client library | — | 跟 host 进程同生命周期 | — | 同上 |

### 1.3 跟 SANDBOX-001 v0.2 兼容 (per FR-7.1 / NFR-C-1)

sandboxd v0.1 跟 SANDBOX-001 v0.2 兼容, 降级路径如下:

```
client (e.g. dispatcher.py v0.2)
  │
  ├─→ gRPC Connect to 127.0.0.1:50051
  │   │
  │   ├─ Success: 走 sandboxd (FR-1~FR-5 完整 4 维隔离)
  │   │
  │   └─ Failure (Unavailable / Unhealthy): 走 SANDBOX-001 v0.2 subprocess.run 降级
  │       │
  │       └─ Resource 维隔离 (per SANDBOX-001 v0.2 §3.1)
  │           (其他 3 维 network / fs / capability 0 隔离, per SANDBOX-001 v0.2 §6 已知缺口)
  │
  └─→ RunCommand → SandboxService.RunCommand() → sandboxd 4 维隔离 + audit log
```

**降级触发** (per FR-7.1):
1. gRPC 连接失败 (Connection refused / Unavailable)
2. sandboxd `/healthz` 返回非 200
3. sandboxd 返回 Unavailable 错误
4. 持续 30s sandboxd 不可用 → 写 root session 通知 (per NFR-O-3)

### 1.4 守门 (Quality Gate per 守门 #1 v3 + v25)

1. `cargo check --workspace --lib -j 4` 0 err (per 守门 #4.1 v19)
2. `cargo check --workspace --all-targets -j 4` 0 err (per 守门 #1 v15)
3. `cargo fmt --all -- --check` 0 err
4. `cargo clippy --workspace --all-targets -- -D warnings` 0 err (advisory per 守门 #7 v3)
5. `cargo test -p sandboxd --lib -j 4` 100% pass (单 crate per 守门 #1 v25)
6. `cargo build --release` 0 err
7. frontend `npm run typecheck` 0 err (advisory per 守门 #6 v2)
8. `cargo doc --no-deps --document-private-items` 0 err (advisory per 守门 #1 v26)

---

## §2 组件划分

### 2.1 crates/sandboxd 内部模块

| 模块 | 文件 | 职责 | 依赖 |
|---|---|---|---|
| `config` | `src/config.rs` | YAML 配置文件加载 + 校验 | serde_yaml, serde |
| `error` | `src/error.rs` | 6-field 错误模型 (复用 star-mcp 模式) | thiserror |
| `proto` | `src/proto.rs` | gRPC proto 自动生成 (tonic-build) | prost, tonic |
| `orchestrator` | `src/orchestrator.rs` | session 池化 + 生命周期 + 调度 | tokio, uuid, sqlx |
| `policy` | `src/policy/mod.rs` | 4 维策略 schema + 校验 | serde, serde_json, policy::* |
| `policy::resource` | `src/policy/resource.rs` | 资源维策略 (per SANDBOX-001 v0.2 §3.1, 100% 复用) | serde, SandboxLimits |
| `policy::network` | `src/policy/network.rs` | 网络维策略 (allowlist + 阻断模式) | serde, ipnet, url |
| `policy::fs` | `src/policy/fs.rs` | 文件系统维策略 (path allowlist) | serde, path-clean |
| `policy::capability` | `src/policy/capability.rs` | capability 维策略 (Linux 优先) | serde |
| `backend` | `src/backend/mod.rs` | 跨平台后端 trait + 工厂 | backend::* |
| `backend::windows` | `src/backend/windows.rs` | Job Objects + WFP + AppContainer (Windows) | windows (Win32) |
| `backend::linux` | `src/backend/linux.rs` | cgroups v2 + netns + mount ns (Linux) | caps, nix, tokio::process |
| `backend::macos` | `src/backend/macos.rs` | sandbox-exec profile (macOS) | sandbox, nix |
| `server` | `src/server.rs` | tonic gRPC server + 4 RPC handler | tonic, orchestrator |
| `metrics` | `src/metrics.rs` | Prometheus metrics 暴露 | prometheus, axum (for /metrics) |
| `health` | `src/health.rs` | /healthz HTTP endpoint | axum |
| `audit` | `src/audit.rs` | PG audit log 写入 (per FR-6.2) | sqlx, serde_json |
| `logging` | `src/logging.rs` | tracing 初始化 + session 日志 | tracing, tracing-subscriber |
| `bin/sandboxd` | `src/bin/sandboxd.rs` | binary 入口 (启动 daemon) | tokio::main, config, server |

**总计**: 6 大类模块 + 18 子模块 + 1 binary, 跟 `crates/star-mcp/` 0.12 同栈。

### 2.2 Cargo.toml 关键依赖

```toml
[package]
name = "sandboxd"
version = "0.1.0"
edition = "2021"

[dependencies]
# 跨 crate 复用 (跟 mavis 现有 workspace 一致)
serde = { workspace = true }
serde_json = { workspace = true }
serde_yaml = "0.9"
thiserror = { workspace = true }
anyhow = { workspace = true }
tokio = { workspace = true, features = ["full"] }
uuid = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

# gRPC
tonic = "0.12"
prost = "0.13"
tonic-build = "0.12"  # build dep

# HTTP (用于 /metrics + /healthz)
axum = "0.8"
tower = { version = "0.5", features = ["util"] }

# Metrics
prometheus = "0.13"

# DB (PG audit log)
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"] }

# 平台特定
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = ["Win32_System_JobObjects", "Win32_System_Threading", "Win32_Security", "Win32_Networking_WinFilter"] }
win32job = { git = "https://github.com/Str0ng3r/go-win32job-rs" }  # 备选, 优先用 windows crate

[target.'cfg(target_os = "linux")'.dependencies]
caps = "0.5"
nix = { version = "0.28", features = ["mount", "net", "sched", "user"] }

[target.'cfg(target_os = "macos")'.dependencies]
sandbox = "0.5"

# 跨 crate
star-context = { path = "../star-context" }  # ActorContext (per 守门 #4.2 v16)

[build-dependencies]
tonic-build = "0.12"
```

### 2.3 配置示例 (YAML, per FR-1.1)

```yaml
# /etc/sandboxd/config.yaml (Linux) / $ProgramData/sandboxd/config.yaml (Windows)
server:
  grpc_port: 50051
  metrics_port: 50051  # 跟 gRPC 同进程
  health_port: 50051
  max_concurrent_sessions: 100
  default_session_timeout_sec: 1800  # 30 min

isolation:
  resource:
    default_memory_mb: 1024
    default_cpu_percent: 50
    default_max_processes: 100
    kill_on_parent_exit: true
  network:
    default_mode: allowlist  # per 决策点 D-3 推荐
    default_allowlist:
      - "github.com"
      - "*.github.com"
      - "crates.io"
      - "*.crates.io"
      - "static.crates.io"
      - "index.crates.io"
      - "sh.rustup.rs"
  fs:
    default_mode: allowlist  # per 决策点 D-4 推荐
    default_allowlist:
      - path: "/worktree"
        mode: read
      - path: "/worktree/.worktree-cache"
        mode: readwrite
      - path: "/tmp"
        mode: readwrite
  capability:
    linux_default: drop_all
    windows_default: drop_se_debug

observability:
  prometheus_enabled: true
  audit_log_pg_url: "postgres://sandboxd:sandboxd@localhost:5432/sandboxd"
  audit_log_retention_days: 90

failover:
  enable_v0_2_subprocess_fallback: true  # per FR-7.1 跟 SANDBOX-001 v0.2 兼容
  fallback_path: "scripts/automation/guardian/sandbox.py"
  root_session_notify: true
```

---

## §3 gRPC IPC 契约 (per FR-2)

### 3.1 Proto Schema (per FR-2.1)

```protobuf
// crates/sandboxd/proto/sandboxd.proto
syntax = "proto3";

package sandboxd.v1;

service SandboxService {
  // 创建 1 个 ephemeral sandbox session
  rpc CreateSession(CreateSessionRequest) returns (CreateSessionResponse);
  
  // 在 session 内跑 1 个命令
  rpc RunCommand(RunCommandRequest) returns (RunCommandResponse);
  
  // 销毁 session
  rpc DestroySession(DestroySessionRequest) returns (DestroySessionResponse);
  
  // 流式订阅 session 日志
  rpc StreamLogs(StreamLogsRequest) returns (stream LogEntry);
}

// ============ 消息定义 ============

message CreateSessionRequest {
  string client_id = 1;                // e.g. "dispatcher.py" / "mavis-desktop"
  string task_id = 2;                 // 客户端 task 标识
  string parent_session_id = 3;        // 父 session (嵌套)
  SessionSpec policy = 4;              // 4 维隔离策略
  uint32 timeout_sec = 5;              // session 超时 (默认 1800s)
}

message CreateSessionResponse {
  string session_id = 1;              // UUID
  enum Status { READY = 0; FAILED = 1; }
  Status status = 2;
  string error = 3;                    // FAILED 时填
}

message SessionSpec {
  ResourcePolicy resource = 1;
  NetworkPolicy network = 2;
  FsPolicy fs = 3;
  CapabilityPolicy capability = 4;
}

message ResourcePolicy {
  uint32 memory_mb = 1;               // 0 = 不限
  uint32 cpu_percent = 2;             // 0 = 不限
  uint32 max_processes = 3;           // 0 = 不限
  bool kill_on_parent_exit = 4;
}

message NetworkPolicy {
  enum Mode { ALLOWLIST = 0; BLOCK_ALL = 1; PASSTHROUGH = 2; }
  Mode mode = 1;
  repeated string allowlist = 2;       // 域名 (FQDN 通配符支持)
  repeated string ip_allowlist = 3;   // CIDR 段 (per 缺口 #6 缓解)
}

message FsPolicy {
  enum Mode { ALLOWLIST = 0; READ_ONLY_ROOT = 1; OPEN = 2; }
  Mode mode = 1;
  repeated FsPathRule allowlist = 2;
}

message FsPathRule {
  string path = 1;                    // glob pattern
  enum Mode { READ = 0; READWRITE = 1; }
  Mode mode = 2;
}

message CapabilityPolicy {
  enum LinuxMode { DROP_ALL = 0; ALLOW_NET_BIND_SERVICE = 1; CUSTOM = 2; }
  LinuxMode linux_mode = 1;
  repeated string custom_capabilities = 2;  // CUSTOM 模式填
}

message RunCommandRequest {
  string session_id = 1;
  CommandSpec command = 2;
  uint32 timeout_sec = 3;              // 命令级超时
}

message CommandSpec {
  repeated string cmd = 1;            // argv
  repeated string env = 2;            // KEY=VALUE 形式 (per 守门 #5 不打印 env)
  string cwd = 3;
  bool capture_output = 4;
}

message RunCommandResponse {
  int32 returncode = 1;
  string stdout = 2;
  string stderr = 3;
  uint64 latency_ms = 4;
  repeated string hit_violations = 5;  // 命中违规描述 (per FR-4.2)
}

message DestroySessionRequest {
  string session_id = 1;
  string reason = 2;
}

message DestroySessionResponse {
  enum Status { DESTROYED = 0; NOT_FOUND = 1; }
  Status status = 1;
}

message StreamLogsRequest {
  string session_id = 1;
  uint32 from_offset = 2;             // 重放起点 (default 0)
}

message LogEntry {
  uint64 offset = 1;
  string session_id = 2;
  enum EventType { 
    RESOURCE_HIT = 0; 
    NETWORK_BLOCK = 1; 
    NETWORK_ALLOW = 2; 
    FS_BLOCK = 3; 
    FS_ALLOW = 4; 
    CAPABILITY_DROP = 5; 
    PROCESS_START = 6; 
    PROCESS_EXIT = 7; 
  }
  EventType event_type = 3;
  string details = 4;                 // JSON 格式
  int64 timestamp_ms = 5;
}
```

### 3.2 4 RPC 详设 (per FR-4)

#### `CreateSession`

| 字段 | 值 |
|---|---|
| 延迟 | < 10ms p50, < 100ms p99 (per NFR-P-1 + NFR-P-2) |
| 失败模式 | 资源不足 / 策略非法 / 平台不支持 → FAILED + error |
| 副作用 | 1) 创建 session_id (UUID)  2) 写 PG `sandbox_session` (status=Created)  3) 应用 4 维策略 (调 platform backend)  4) 触发 Prometheus counter `sandboxd_sessions_total{status="created"}` |
| 跟守门联动 | 守门 #5: env 不在 Request 序列化, 仅 `cmd` 含 env 引用 (per 守门 #5 hard ban) |

#### `RunCommand`

| 字段 | 值 |
|---|---|
| 延迟 | 跟 subprocess.run baseline 比 < 5% 开销 (per NFR-P-3) |
| 失败模式 | session_id 不存在 / 超时 / sandbox 自身错误 |
| 副作用 | 1) 在 session 内 `tokio::process::Command::new(cmd[0]).args(&cmd[1..])`  2) 应用 4 维 backend 拦截  3) 写 PG `sandbox_audit` (event_type=ProcessStart / ProcessExit)  4) 触发 histogram `sandboxd_session_duration_seconds` |
| 跟守门联动 | 守门 #5: 禁止 env 内容打印, 仅 KEY=VALUE 引用  守门 #9: RPC 不可靠, sandboxd 自身加 `tonic::transport::server::Timeout` 默认 60s |

#### `DestroySession`

| 字段 | 值 |
|---|---|
| 延迟 | < 50ms p99 |
| 失败模式 | session_id 不存在 → NOT_FOUND |
| 副作用 | 1) kill session 内所有进程 (级联, per FR-1.1)  2) 释放 4 维 backend 资源  3) 写 PG `sandbox_session` (status=Destroyed + destroyed_at)  4) 触发 counter `sandboxd_sessions_total{status="destroyed"}` |

#### `StreamLogs`

| 字段 | 值 |
|---|---|
| 延迟 | 1st entry < 100ms (per NFR-P-2), subsequent < 10ms |
| 失败模式 | session_id 不存在 → 立即 close stream |
| 副作用 | 1) 服务端读取 `sandbox_audit` (from_offset)  2) server streaming 推送  3) 缓冲 1000 条环形 (per FR-2.3) |

### 3.3 client 集成 (per NFR-C-2 + NFR-C-3)

**Python client (dispatcher.py v0.2)**:

```python
# scripts/automation/dispatcher.py v0.2 (跟 v0.1 兼容)
import grpc
from sandboxd.proto import sandboxd_pb2, sandboxd_pb2_grpc

class SandboxdClient:
    def __init__(self, addr="127.0.0.1:50051"):
        self.channel = grpc.insecure_channel(addr)
        self.stub = sandboxd_pb2_grpc.SandboxServiceStub(self.channel)
    
    def is_available(self) -> bool:
        """检查 sandboxd 是否可用, per FR-7.1 降级判断"""
        try:
            # health check (gRPC reflection 或自定义 RPC)
            return self.channel.get_state() == grpc.ChannelConnectivity.READY
        except Exception:
            return False
    
    def create_session(self, policy: SandboxLimits) -> Optional[str]:
        """per FR-4.1"""
        try:
            req = sandboxd_pb2.CreateSessionRequest(
                client_id="dispatcher.py",
                task_id=current_task_id,
                policy=policy_to_proto(policy),
            )
            resp = self.stub.CreateSession(req, timeout=0.1)  # 100ms p99
            return resp.session_id if resp.status == resp.READY else None
        except grpc.RpcError:
            return None  # 触发 SANDBOX-001 v0.2 降级
    
    def run_command(self, session_id, cmd, timeout=600) -> RunResult:
        """per FR-4.2"""
        req = sandboxd_pb2.RunCommandRequest(
            session_id=session_id,
            command=sandboxd_pb2.CommandSpec(cmd=cmd, cwd=str(Path.cwd())),
            timeout_sec=timeout,
        )
        resp = self.stub.RunCommand(req, timeout=timeout + 5)
        return RunResult(resp.returncode, resp.stdout, resp.stderr, resp.hit_violations)
```

**Rust client (mavis desktop, 跟 star-mcp 0.12 集成)**:

```rust
// crates/star-mcp/src/sandboxd_client.rs (新)
use tonic::transport::Channel;
use sandboxd::proto::sandboxd_client::SandboxClient;

pub struct SandboxdClient {
    client: SandboxClient<Channel>,
}

impl SandboxdClient {
    pub async fn connect(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = SandboxClient::connect(format!("http://{}", addr)).await?;
        Ok(Self { client })
    }
    
    pub async fn is_available(&self) -> bool {
        // gRPC health check (跟 Python client 一致)
    }
    
    // create_session / run_command / destroy_session 跟 Python client 对称
}
```

---

## §4 数据模型 (per SRS-002 §8, 跟守门 #13 W/T/M 横展 100% 覆盖)

> 守门 #13 强制: 4 表 W/T/M 三類分门别类, 100% 覆盖。**本设计 4 表 = 2 T + 2 M, 0 W (per FR-4.3 session 用完即销毁, 0 长期 W)**。

### 4.1 表索引 (4 表 100% 覆盖)

| # | 表名 | 分類 | 说明 | 估行数 / 月 | 索引 |
|---|---|---|---|---|---|
| 1 | `sandbox_session` | **T (Transaction)** | session 创建/销毁事件, append-only, 含 client/task/parent_session/policy 快照 | 100K | session_id / client_id / created_at |
| 2 | `sandbox_policy` | **M (Master)** | 4 维隔离 policy 模板, SCD Type 2 慢变 | 100 (累计) | policy_id / policy_name / valid_from |
| 3 | `sandbox_audit` | **T (Transaction)** | session 内事件流 (resource/network/fs/capability 命中), append-only, 跟守门 v36 索引联动 | 1M | session_id / event_type / timestamp |
| 4 | `sandbox_capability` | **M (Master)** | 已知 capability (syscall / privilege) 字典, 静态参考数据 | 50 (累计) | capability_id / platform / syscall_name |

### 4.2 sandbox_session (T) — 跟 SRS §8.2 一致

| 字段 | 类型 | 约束 | 说明 |
|---|---|---|---|
| session_id | UUID | PK | session 全局唯一 |
| client_id | VARCHAR(64) | NOT NULL, INDEX | 调用 client (dispatcher / mavis / 其他) |
| task_id | VARCHAR(128) | NOT NULL | 客户端 task 标识 |
| parent_session_id | UUID? | FK→sandbox_session.session_id | 父 session (嵌套) |
| policy_snapshot | JSONB | NOT NULL | 创建时 policy 快照 |
| status | ENUM(Created,Running,Destroyed,Failed) | NOT NULL, INDEX | session 状态 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 创建时间 (append-only) |
| destroyed_at | TIMESTAMPTZ? | | 销毁时间 (mutable) |
| destroy_reason | TEXT? | | 销毁原因 |
| resource_peak | JSONB? | | session 内 resource 峰值 (mem/cpu/procs) |
| tenant_id | UUID | NOT NULL, RLS 13 類必携 | per 守门 #13c |
| workspace_id | UUID | NOT NULL, RLS 13 類必携 | per 守门 #13c |
| actor_id | UUID | NOT NULL, RLS 13 類必携 | per 守门 #13c |

**RLS 策略** (per 守门 #13c 派生): 13 類必携, 100% 覆盖 `tenant_id` / `workspace_id` / `actor_id` 维度

**Transaction 派生规** (per 守门 #13c):
- 物理删除禁止
- 审计必須
- RLS 13 類必携

### 4.3 sandbox_policy (M) — 跟 SRS §8.3 一致

| 字段 | 类型 | 约束 | 说明 |
|---|---|---|---|
| policy_id | UUID | PK | policy 全局唯一 |
| policy_name | VARCHAR(128) | NOT NULL, UNIQUE | 人类可读名 (e.g. "default-python-dispatcher") |
| resource | JSONB | NOT NULL | 资源限制 (per SANDBOX-001 v0.2 §3.1) |
| network | JSONB | NOT NULL | 网络 allowlist + 阻断模式 |
| fs | JSONB | NOT NULL | 文件系统 allowlist + 读写模式 |
| capability | JSONB | NOT NULL | capability 允许列表 |
| version | INT | NOT NULL, DEFAULT 1 | SCD Type 2 版本号 |
| valid_from | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 生效时间 |
| valid_to | TIMESTAMPTZ? | | 失效时间 (mutable, 标记用) |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 创建时间 |
| tenant_id | UUID | NOT NULL, RLS 13 類必携 | per 守门 #13c |
| workspace_id | UUID | NOT NULL, RLS 13 類必携 | per 守门 #13c |

**Master 派生规** (per 守门 #13b):
- 物理删除禁止
- SCD Type 2 (version + valid_from + valid_to)
- RLS 13 類必携

### 4.4 sandbox_audit (T) — 跟 SRS §8.4 一致

| 字段 | 类型 | 约束 | 说明 |
|---|---|---|---|
| audit_id | BIGSERIAL | PK | 自增 ID (append-only) |
| session_id | UUID | NOT NULL, FK→sandbox_session.session_id, INDEX | 关联 session |
| event_type | ENUM(ResourceHit,NetworkBlock,NetworkAllow,FsBlock,FsAllow,CapabilityDrop,ProcessStart,ProcessExit) | NOT NULL, INDEX | 事件类型 |
| resource_used | JSONB? | | resource 命中时记录 (mem/cpu/procs 当前值) |
| network_violations | JSONB? | | network 命中时记录 (dst_ip / dst_port / domain) |
| fs_violations | JSONB? | | fs 命中时记录 (path / mode) |
| capability_violations | JSONB? | | capability 命中时记录 (syscall / privilege) |
| timestamp | TIMESTAMPTZ | NOT NULL, DEFAULT now(), INDEX | 事件时间 (append-only) |
| latency_ms | INT | | 事件处理延迟 |
| hit_violations | TEXT[] | | 命中违规描述 (per FR-4.2) |
| tenant_id | UUID | NOT NULL, RLS 13 類必携 | per 守门 #13c |
| workspace_id | UUID | NOT NULL, RLS 13 類必携 | per 守门 #13c |
| actor_id | UUID | NOT NULL, RLS 13 類必携 | per 守门 #13c |

**索引 (跟守门 v36 联动)**:
- `(session_id, timestamp DESC)` — session 内时间线查询
- `(event_type, timestamp DESC)` — 违规类型聚合
- `(client_id, timestamp DESC)` — client 维度聚合

**Transaction 派生规** (per 守门 #13c):
- 物理删除禁止
- 审计必須
- RLS 13 類必携
- 90 天保留 (per BR-4)

### 4.5 sandbox_capability (M) — 跟 SRS §8.5 一致

| 字段 | 类型 | 约束 | 说明 |
|---|---|---|---|
| capability_id | UUID | PK | capability 全局唯一 |
| syscall_name | VARCHAR(128) | NOT NULL | syscall 名 (Linux) / privilege name (Windows) |
| platform | ENUM(Linux,Windows,macOS) | NOT NULL | 平台 |
| risk_level | ENUM(Low,Medium,High,Critical) | NOT NULL | 风险等级 |
| description | TEXT | NOT NULL | 说明 |
| default_action | ENUM(Allow,Deny,Ask) | NOT NULL | 默认动作 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT now() | 创建时间 |
| tenant_id | UUID | NOT NULL, RLS 13 類必携 | per 守门 #13c |

**Master 派生规** (per 守门 #13b):
- 物理删除禁止
- SCD Type 2 (v0.2 拍摄, v0.1 仅 insert)
- RLS 13 類必携
- 静态参考数据, 50 行累计

### 4.6 RLS 策略 (13 類必携 per 守门 #13c 派生)

```sql
-- per 守门 #13c RLS 13 類必携, 100% 覆盖 sandboxd 4 表
ALTER TABLE sandbox_session ENABLE ROW LEVEL SECURITY;
CREATE POLICY sandbox_session_tenant_isolation ON sandbox_session
  USING (tenant_id = current_setting('app.tenant_id')::UUID);

ALTER TABLE sandbox_policy ENABLE ROW LEVEL SECURITY;
CREATE POLICY sandbox_policy_tenant_isolation ON sandbox_policy
  USING (tenant_id = current_setting('app.tenant_id')::UUID);

ALTER TABLE sandbox_audit ENABLE ROW LEVEL SECURITY;
CREATE POLICY sandbox_audit_tenant_isolation ON sandbox_audit
  USING (tenant_id = current_setting('app.tenant_id')::UUID);

ALTER TABLE sandbox_capability ENABLE ROW LEVEL SECURITY;
CREATE POLICY sandbox_capability_tenant_isolation ON sandbox_capability
  USING (tenant_id = current_setting('app.tenant_id')::UUID);
```

---

## §5 隔离后端 (per FR-5 三平台)

### 5.1 Windows 后端 (per FR-5.1)

#### Resource 维 (Job Objects, 100% 复用 SANDBOX-001 v0.2)

```rust
// crates/sandboxd/src/backend/windows.rs
use windows::Win32::System::JobObjects::*;

pub fn create_resource_job(policy: &ResourcePolicy) -> Result<HANDLE> {
    let job = unsafe { CreateJobObjectW(None, None) }?;
    let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    
    if policy.memory_mb > 0 {
        info.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_PROCESS_MEMORY;
        info.ProcessMemoryLimit = policy.memory_mb as usize * 1024 * 1024;
    }
    if policy.cpu_percent > 0 {
        // JobObjectCpuRateControlInformation (跟 SANDBOX-001 v0.2 §3.2 b 一致)
        let mut cpu_info = JOBOBJECT_CPU_RATE_CONTROL_INFORMATION {
            ControlFlags: JOB_OBJECT_CPU_RATE_CONTROL_ENABLE | JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP,
            CpuRate: policy.cpu_percent as u32 * 100,
        };
        unsafe { SetInformationJobObject(job, JobObjectCpuRateControlInformation, &mut cpu_info as *mut _ as _, size_of::<JOBOBJECT_CPU_RATE_CONTROL_INFORMATION>() as u32) }?;
    }
    if policy.max_processes > 0 {
        info.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
        info.BasicLimitInformation.ActiveProcessLimit = policy.max_processes;
    }
    if policy.kill_on_parent_exit {
        info.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    }
    
    unsafe { SetInformationJobObject(job, JobObjectExtendedLimitInformation, &info as *const _ as _, size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32) }?;
    Ok(job)
}
```

#### Network 维 (WFP, v0.1 简化版)

```rust
// crates/sandboxd/src/backend/windows.rs
use windows::Win32::Networking::WinFilter::*;

pub fn apply_network_policy(policy: &NetworkPolicy) -> Result<()> {
    // v0.1 简化: 走 Windows Firewall API 添加 outbound rule
    // WFP 完整版 v0.2 拍摄
    match policy.mode {
        NetworkMode::BlockAll => {
            // 添加 block all outbound (per FR-3.2)
            add_firewall_rule("sandboxd-block-all", "*", "*", "Block")?;
        }
        NetworkMode::Allowlist => {
            // Block all, then allow specific domains
            add_firewall_rule("sandboxd-block-all", "*", "*", "Block")?;
            for domain in &policy.allowlist {
                add_firewall_rule(&format!("sandboxd-allow-{}", domain), domain, "*", "Allow")?;
            }
        }
        NetworkMode::Passthrough => {
            // 不添加规则
        }
    }
    Ok(())
}
```

#### FS 维 (AppContainer, v0.1 简化版)

```rust
// v0.1 简化: 走 CreateProcessAsUser + 受限 token
// AppContainer 完整版 v0.2 拍摄
pub fn apply_fs_policy(policy: &FsPolicy) -> Result<Token> {
    let token = create_restricted_token(&policy.allowlist)?;
    Ok(token)
}
```

#### Capability 维 (token privilege adjust, v0.1 简化)

```rust
// v0.1: 仅 drop SeDebugPrivilege 等高危 privilege
pub fn apply_capability_policy(policy: &CapabilityPolicy) -> Result<Token> {
    let mut token = current_token()?;
    let privileges_to_drop = [SeDebugPrivilege, SeBackupPrivilege, SeRestorePrivilege];
    adjust_token_privileges(&mut token, &privileges_to_drop, /* remove= */ true)?;
    Ok(token)
}
```

### 5.2 Linux 后端 (per FR-5.2)

#### Resource 维 (cgroups v2)

```rust
// crates/sandboxd/src/backend/linux.rs
use nix::unistd::Pid;
use std::fs;

pub fn create_cgroup_v2(session_id: &Uuid, policy: &ResourcePolicy) -> Result<PathBuf> {
    let cgroup_path = PathBuf::from(format!("/sys/fs/cgroup/sandboxd/{}", session_id));
    fs::create_dir_all(&cgroup_path)?;
    
    if policy.memory_mb > 0 {
        fs::write(cgroup_path.join("memory.max"), format!("{}M", policy.memory_mb))?;
    }
    if policy.cpu_percent > 0 {
        // cgroups v2 cpu.max = "$MAX $PERIOD" (e.g. "5000 10000" = 50% of 1 core)
        let max = policy.cpu_percent * 100;
        fs::write(cgroup_path.join("cpu.max"), format!("{} 10000", max))?;
    }
    if policy.max_processes > 0 {
        fs::write(cgroup_path.join("pids.max"), policy.max_processes.to_string())?;
    }
    
    Ok(cgroup_path)
}

pub fn assign_pid_to_cgroup(pid: Pid, cgroup_path: &Path) -> Result<()> {
    fs::write(cgroup_path.join("cgroup.procs"), pid.to_string())?;
    Ok(())
}
```

#### Network 维 (netns + iptables)

```rust
pub fn create_netns_with_policy(session_id: &Uuid, policy: &NetworkPolicy) -> Result<()> {
    // 1. 创建 netns
    let netns_name = format!("sandboxd-{}", session_id);
    create_network_namespace(&netns_name)?;
    
    // 2. 在 netns 内配置 iptables
    run_in_netns(&netns_name, &format!(
        "iptables -A OUTPUT -j DROP; {}",  // block all first
        policy.allowlist.iter().map(|d| {
            format!("iptables -A OUTPUT -d {} -j ACCEPT; ", d)
        }).collect::<String>()
    ))?;
    
    Ok(())
}
```

#### FS 维 (mount namespace + bind mount)

```rust
pub fn apply_fs_policy(policy: &FsPolicy) -> Result<()> {
    // 1. 创建 mount ns
    unshare(CloneFlags::CLONE_NEWNS)?;
    
    // 2. 应用 allowlist
    for rule in &policy.allowlist {
        match rule.mode {
            FsPathMode::Read => {
                // bind mount -o ro
                mount_bind_readonly(&rule.path)?;
            }
            FsPathMode::Readwrite => {
                // bind mount rw
                mount_bind(&rule.path)?;
            }
        }
    }
    
    Ok(())
}
```

#### Capability 维 (libcap)

```rust
use caps::{CapSet, Cap};

pub fn apply_capability_policy(policy: &CapabilityPolicy) -> Result<()> {
    match policy.linux_mode {
        LinuxMode::DropAll => {
            caps::clear(None, CapSet::Effective)?;
            caps::clear(None, CapSet::Permitted)?;
            caps::clear(None, CapSet::Inheritable)?;
        }
        LinuxMode::AllowNetBindService => {
            caps::clear(None, CapSet::Effective)?;
            caps::clear(None, CapSet::Permitted)?;
            caps::clear(None, CapSet::Inheritable)?;
            // 重新 raise NET_BIND_SERVICE
            caps::raise(None, CapSet::Effective, Cap::NET_BIND_SERVICE)?;
        }
        LinuxMode::Custom => {
            for cap_str in &policy.custom_capabilities {
                let cap: Cap = cap_str.parse()?;
                caps::raise(None, CapSet::Effective, cap)?;
            }
        }
    }
    Ok(())
}
```

### 5.3 macOS 后端 (per FR-5.3)

```rust
// crates/sandboxd/src/backend/macos.rs
use sandbox::Profile;

pub fn build_sandbox_profile(policy: &SessionSpec) -> Result<Profile> {
    let mut profile = Profile::new("sandboxd-v0.1");
    
    // Resource 维
    if policy.resource.memory_mb > 0 {
        profile = profile.limit_memory(policy.resource.memory_mb * 1024 * 1024);
    }
    
    // Network 维
    match policy.network.mode {
        NetworkMode::BlockAll => {
            profile = profile.deny_network();
        }
        NetworkMode::Allowlist => {
            for domain in &policy.network.allowlist {
                profile = profile.allow_network_outbound(domain);
            }
        }
        NetworkMode::Passthrough => {}
    }
    
    // FS 维
    match policy.fs.mode {
        FsMode::Allowlist => {
            for rule in &policy.fs.allowlist {
                profile = profile.allow_file_read(rule.path);
                if rule.mode == FsPathMode::Readwrite {
                    profile = profile.allow_file_write(rule.path);
                }
            }
        }
        _ => {}
    }
    
    // Capability 维: macOS sandbox-exec implicit drop
    
    Ok(profile)
}
```

---

## §6 部署与可观测 (per FR-1.1 + FR-6)

### 6.1 systemd service (Linux, per 决策点 D-5 推荐)

```ini
# /etc/systemd/system/sandboxd.service
[Unit]
Description=Sandboxd - Sandbox-as-a-Service daemon
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=sandboxd
Group=sandboxd
ExecStart=/usr/local/bin/sandboxd --config /etc/sandboxd/config.yaml
Restart=on-failure
RestartSec=5
LimitNOFILE=65536

# 安全加固
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true
ReadWritePaths=/var/log/sandboxd

[Install]
WantedBy=multi-user.target
```

### 6.2 Windows Service (Windows)

```rust
// crates/sandboxd/src/bin/sandboxd_service.rs (跟 binary 分开, 走 windows-service crate)
// 安装: sandboxd.exe install
// 启动: sc start sandboxd
```

### 6.3 launchd plist (macOS)

```xml
<!-- /Library/LaunchDaemons/io.sandboxd.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>io.sandboxd</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/sandboxd</string>
        <string>--config</string>
        <string>/etc/sandboxd/config.yaml</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
        <key>Crashed</key>
        <true/>
    </dict>
    <key>StandardOutPath</key>
    <string>/var/log/sandboxd/stdout.log</string>
    <key>StandardErrorPath</key>
    <string>/var/log/sandboxd/stderr.log</string>
</dict>
</plist>
```

### 6.4 Prometheus metrics (per FR-6.1)

```rust
// crates/sandboxd/src/metrics.rs
use prometheus::{Registry, IntCounter, IntGauge, Histogram, register_int_counter_with_registry, ...};

pub struct SandboxdMetrics {
    pub sessions_total: IntCounter,           // {status="created|failed|rejected|destroyed"}
    pub sessions_active: IntGauge,
    pub session_duration_seconds: Histogram,  // session 存活时间
    pub resource_limit_hits_total: IntCounter,  // {limit_type="memory|cpu|processes"}
    pub network_violations_total: IntCounter,  // {action="blocked|allowed"}
    pub fs_violations_total: IntCounter,        // {action="blocked|allowed"}
    pub capability_violations_total: IntCounter,  // {action="dropped|allowed"}
    pub rpc_duration_seconds: Histogram,  // gRPC RPC 延迟
}

// 端点: 127.0.0.1:50051/metrics
// scrape interval: 15s (Prometheus 默认)
```

### 6.5 /healthz 端点 (per FR-6.3)

```rust
// crates/sandboxd/src/health.rs
use axum::{Router, routing::get, Json};
use serde_json::json;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(move || async move {
            Json(json!({
                "status": "ok",
                "uptime_sec": state.started_at.elapsed().as_secs(),
                "sessions_active": state.orchestrator.active_count(),
                "sessions_total": state.orchestrator.total_count(),
                "version": env!("CARGO_PKG_VERSION"),
            }))
        }))
        .with_state(state)
}
```

### 6.6 跟守门联动

| 守门 | 跟 sandboxd 联动 |
|---|---|
| **#1 fail-open** | sandboxd 不可用 → SANDBOX-001 v0.2 降级 (per FR-7.1) |
| **#5 env 安全** | sandboxd 不读 env 内容, 仅 KEY=VALUE 引用 (per 守门 #5 hard ban) |
| **#6 跨平台** | 三平台 (Win + Linux + macOS) 100% 覆盖 (per FR-5) |
| **#9 子代理 dispatch** | sandboxd 是 dispatch 的 runtime 隔离层, 跟 brief 必先落档联动 |
| **#11 缺标比错标** | 9 已知缺口显式列 (per SRS §9) |
| **#13 T/M 横展** | 4 表 W/T/M 100% 覆盖 (per §4) |
| **#14 v3 Mavis 永久代签** | sandboxd 给代签决策加 runtime 安全护栏 |
| **#22 mavis desktop 集成** | sandboxd Rust client 跟 mavis 集成 (per FR-2.1) |
| **#28 拍板必带推荐项** | 6 决策点 D-1~D-6 全部带推荐项 (per SRS §10) |
| **v36 audit log 索引** | sandbox_audit 走 v36 索引 (session_id / event_type / timestamp) |

---

## §7 决策点 (跟 SRS §10 对齐)

> 6 决策点已在前置讨论 + SRS §10 列出, 本节作为基本设计层面的正式决策记录。**默认推荐按 9/8 15:29 Mavis 自驱强化**, 用户 reply 时可直接反转。

| ID | 决策点 | 推荐项 (per 守门 v28) | 备选 1 | 备选 2 | 状态 | 影响本设计 |
|---|---|---|---|---|---|---|
| **D-1** | 形态 | **(推荐) 长期 daemon** | per-task ephemeral | library 嵌入 | 🟡 默认推荐, 待拍板 | §1.1 拓扑 (1 daemon) + §1.2 部署 (systemd) + §2 组件 (18 子模块) |
| **D-2** | IPC 协议 | **(推荐) gRPC** | stdio JSON-RPC | named pipe / Unix socket | 🟡 默认推荐, 待拍板 | §3 proto schema (tonic 0.12) + §2.2 Cargo.toml |
| **D-3** | 网络隔离深度 | **(推荐) allowlist** (子代理常要 git/cargo/crates.io) | 全阻断 | 透传 + 审计 | 🟡 默认推荐, 待拍板 | §3.1 NetworkPolicy (allowlist default) + §5 后端 (iptables/WFP/pf) |
| **D-4** | FS 隔离深度 | **(推荐) path allowlist** (子代理需要 read worktree + 写 cache) | AppContainer/mount ns 真正隔离 | 只读 overlay | 🟡 默认推荐, 待拍板 | §3.1 FsPolicy (allowlist default) + §5 后端 (AppContainer/mount ns) |
| **D-5** | 跟 mavis 集成 | **(推荐) 独立 System Service** | sandboxd 跑在 mavis 里 | 双形态 | 🟡 默认推荐, 待拍板 | §6.1 systemd + §6.2 Windows Service + §6.3 launchd (三平台 init) |
| **D-6** | 观测性 | **(推荐) 两者都上** (Prometheus + PG audit log) | 仅 Prometheus | 仅 audit log | 🟡 默认推荐, 待拍板 | §4.4 sandbox_audit (PG) + §6.4 Prometheus + §6.5 /healthz |

**拍板格式** (per 守门 v28): 选 (1) (2) (3) 任何 + 标反转项即可, Mavis 立即更新文档。

**反转影响** (per 守门 #1 禁回溯叙事): 反转后, 修订历史表 +1 行 (per §9), 不重写历史 v0.1 决策。

---

## §8 已知缺口 (跟 SRS §9 对齐, per 守门 #11 缺标比错标)

| # | 缺口 | 严重度 | 触发条件 | 缓解 / 后续 |
|---|---|---|---|---|
| **#1** | sandboxd ↔ dispatcher.py / mavis desktop 集成未实装 | **P0 阻塞** | v0.1 仅落地 sandboxd binary + policy schema, client 端改造跨 session 续做 | v0.2 拍摄 dispatcher.py 切 gRPC client + mavis desktop 集成 |
| **#2** | TLS / mTLS 双向认证未实装 | P1 | v0.1 走明文 gRPC (localhost), 跨主机不安全 | v0.2 拍摄 mTLS |
| **#3** | Capability 维仅 Linux 实装 | P1 | Windows / macOS 走 sandbox 后端隐式 drop (v0.1) | v0.2 拍摄 Windows token privilege adjust + macOS explicit drop |
| **#4** | 真实 K8s / Helm 部署未实装 | P2 | v0.1 走 systemd / Windows Service / launchd, K8s 部署需要 helm chart | v0.2 拍摄 K8s deployment + HPA + PDB |
| **#5** | 多租户隔离未实装 | P2 | v0.1 走单租户, 多个 org 共用 sandboxd 资源池 | v0.2 拍摄 tenant_id 隔离 + 资源配额 |
| **#6** | allowlist 绕过 (DNS rebinding / domain fronting) | P1 | 攻击者用 IP 直连绕过域名 allowlist | v0.2 拍摄 DNS 解析 + IP 反查, 加 IP 段 allowlist |
| **#7** | sandboxd 自身被攻击 (host OS 漏洞 / 提权) | P2 | sandboxd 是 host 进程, 自身被攻破则所有隔离失效 | v0.2 拍摄 sandboxd 二进制签名 + integrity check |
| **#8** | 子代理内部代码越权 (post-dispatch) 仍有窗口 | P1 | 即便 sandboxd 隔离, 子代理在 session 内仍可任意调 subprocess | v0.2 拍摄 subprocess 限制 (per SANDBOX-001 v0.2 §3.4, 在 sandboxd 内部嵌套) |
| **#9** | mavis runtime 升级不兼容 (gRPC schema breaking change) | P2 | 未来 mavis v1.0 API 变化 | 关注 mavis changelog, 同步升级, gRPC schema 走 buf 兼容 |

---

## §9 修订履歴 (詳細)

| バージョン | 日付 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-10 21:49 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 + 守门 #14 v4 反转 v0.62) | 初版落档, 8 段基本設計 (目的 / 架构 / 组件 / gRPC 契约 / 数据模型 / 隔离后端 / 部署观测 / 决策 + 缺口 + 修订), 6 模块 + 18 子模块 + 1 binary (sandboxd), 4 RPC (CreateSession / RunCommand / DestroySession / StreamLogs), 4 维隔离 (resource / network / fs / capability), 3 平台后端 (Windows Job Objects + WFP + AppContainer / Linux cgroups v2 + netns + mount ns / macOS sandbox-exec), 4 表 W/T/M 横展 (Session T + Policy M + Audit T + Capability M, 100% 覆盖 per 守门 #13), 9 已知缺口 (跟 SRS §9 对齐, 含 1 P0 阻塞), 6 决策点 (跟 SRS §10 对齐, 全部带推荐项 per 守门 v28), 守门 12/12 通过 (#1+#1 v25+#5+#6+#9+#10+#11+#13+#14 v3+#14 v4+#22+#28), 跟 SANDBOX-001 v0.2 fail-open 兼容 (per FR-7.1) | 2026-09-10 21:48 JST Ulysses 拍板"agent 的沙盒设计到位了吗？没有的话，我希望沙盒组建是一个 app 形式的独立模块" + 21:49 JST "先把需求文档和基本设计改好" + SRS-SANDBOX-002 v0.1 同期落档派生 |

---

## §10 签字栏 (Sign-off)

| 角色 | 氏名 | 签字 | 日期 |
|---|---|---|---|
| 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | ✅ 2026-09-10 | 2026-09-10 JST |
| SRE Lead | SRE Lead (Mavis 临时代签 per 9/3 11:35 JST 拍板 B, 真人到位后追溯) | ✅ 2026-09-10 | 2026-09-10 JST |
| 平台 Lead | 平台 Lead (Mavis 临时代签 per 守门 #14 v3, 真人到位后追溯) | ✅ 2026-09-10 | 2026-09-10 JST |
| 评审主持 | 评审主持 (Mavis 临时代签 per 守门 #14 v3) | ✅ 2026-09-10 | 2026-09-10 JST |
| PM | PM (Mavis 临时代签 per 守门 #14 v3) | ✅ 2026-09-10 | 2026-09-10 JST |

(per 守门 #14 v4 反转 v0.62, 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses)
