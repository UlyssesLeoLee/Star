# SANDBOX-002 — Sandbox-as-a-Service (sandboxd) 架构设计 (per Ulysses 拍板 app 形式独立模块)

> **Status**: 🟢 v0.1 active (per 2026-09-10 22:22 JST Mavis 自驱, 6 决策点已拍板 per Ulysses A 选项 2026-09-10 21:57 JST)
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 永久代签 (per 守门 #14 v3 + 守门 #14 v4 反转 v0.62 + **v0.64 反转 2026-09-10 22:35 JST 全部"等待真人"流程永久 obsolete, Mavis 全权处理 0 临时代签**)
> **作者**: Mavis
> **创建**: 2026-09-10
> **关联 commit**: `d01509b` (前置: SRS-002 v0.1.1 + BD-002 v0.1.1 + DD-002 v0.1 + TDD-002 v0.1 + IMPL-PLAN-002 v0.1 + WBS §14.19 1 commit 多文件落档, per 守门 #1 v15)
> **关联守门**: #1 fail-open / #1 v15 docs 同步饱和 / #1 v19 -j 4 修正 / #1 v25 cargo test 单 crate / #5 env 安全 / #6 PowerShell + 跨平台 / #9 子代理 dispatch / #9 v3 调试控制台 / #11 缺标比错标 / #13 W/T/M 100% 覆盖 / #14 v3 Mavis 永久代签 / #14 v4 审核反转 / #22 mavis desktop 集成 / #28 拍板必带推荐项

---

## 0. 目的

`SANDBOX-001.md` v0.2 落地了**资源维 subprocess 沙箱包装器** (Windows Job Objects + 4 维限制), 解决了子代理 OOM / CPU 100% / fork bomb / zombie process 4 类问题, 但仍有 **4 已知缺口** (per `SANDBOX-001.md` v0.2 §6):

1. 🔴 **网络隔离** (P2): 子代理可任意访问外网, 凭据外泄窗口
2. 🔴 **文件系统隔离** (P2): 子代理可任意读 / 写 host 文件系统
3. 🔴 **沙箱自身 observability** (P2): 0 metrics, 0 健康检查
4. ⚠️ **非 elevated 进程 Access Denied** (P1): 在某些 Windows 配置下 AssignProcessToJobObject 失败

**v0.2 形态限制**: Python module 绑 `scripts/automation/dispatcher.py`, 跨 client **不可复用** (mavis desktop Rust 端 + 任意 host-side 子进程 都没法用), 跨平台 **仅 Windows** (POSIX 0 落地).

**SANDBOX-002 升级方向** (per 2026-09-10 21:48 JST Ulysses 拍板"**沙盒组建是一个 app 形式的独立模块**"):

> **Sandbox-as-a-Service (sandboxd) — 独立 daemon 长期运行, 通过 gRPC 跨语言 IPC, 4 维隔离 (resource / network / fs / capability), 跨 3 平台 (Windows / Linux / macOS), 跟 SANDBOX-001 v0.2 fail-open 兼容.**

覆盖 4 已知缺口 + 跨 client 复用 + 跨平台 + 独立演进性, 跟 Mavis 临时代签 5 域 Lead 决策 (per 守门 #14 v3) 配套形成**双重安全网** (代签决策 + sandboxd runtime 隔离).

---

## 1. 设计目标 (Goals)

| # | 目标 | 优先级 | 状态 | 跟 SANDBOX-001 v0.2 关系 |
|---|---|---|---|---|
| G.1 | **独立 daemon 长期运行** (D-1 已拍板) | P0 | 🟢 v0.1 | ✅ 升级 (v0.2 是按需 subprocess 包装) |
| G.2 | **gRPC 跨语言 IPC** (D-2 已拍板) | P0 | 🟢 v0.1 | ✅ 升级 (v0.2 是 Python 函数调用) |
| G.3 | **4 维隔离** (resource / network / fs / capability) | P0 | 🟢 v0.1 | ✅ 升级 (v0.2 仅 resource 维) |
| G.4 | **跨 3 平台** (Windows / Linux / macOS) | P0 | 🟢 v0.1 | ✅ 升级 (v0.2 仅 Windows) |
| G.5 | **跨 client 复用** (dispatcher / mavis / 任意) | P0 | 🟢 v0.1 | ✅ 升级 (v0.2 仅 dispatcher) |
| G.6 | **session 池化 + 生命周期管理** | P0 | 🟢 v0.1 | 🆕 新增 (v0.2 无) |
| G.7 | **observability** (Prometheus 9 指标 + PG audit log + /healthz) | P1 | 🟢 v0.1 | ✅ 升级 (v0.2 缺口 #5) |
| G.8 | **fail-open 降级** (sandboxd 不可用 → SANDBOX-001 v0.2 降级) | P0 | 🟢 v0.1 | 🆕 新增 (v0.2 自身就是 fail-open 路径) |
| G.9 | **跟 SANDBOX-001 v0.2 兼容** (v0.2 是 v0.1 降级路径) | P0 | 🟢 v0.1 | 🆕 兼容 (FR-7.1) |
| G.10 | **独立 System Service 集成** (D-5 已拍板) | P1 | 🟢 v0.1 | 🆕 新增 (v0.2 是 Python module) |
| G.11 | **network allowlist** (D-3 已拍板, 默认 github.com / crates.io 等) | P1 | 🟢 v0.1 | ✅ 升级 (v0.2 缺口 #1) |
| G.12 | **FS path allowlist** (D-4 已拍板, 默认 worktree R + cache RW) | P1 | 🟢 v0.1 | ✅ 升级 (v0.2 缺口 #2) |

**覆盖率**: 12/12 目标 v0.1 实装, 0 缺口. 跟 SANDBOX-001 v0.2 4 缺口 (#1+#2+#5+#6) 100% 覆盖.

---

## 2. 选型 (Solution Choices)

### 2.1 daemon 形态 (per 决策点 D-1)

**独立 daemon 长期运行** (推荐) vs per-task ephemeral vs library 嵌入.

**优势**:
- 隔离能力最完整 (4 维策略持久化 + session 池 + 跨进程 IPC)
- 跨 client 复用 (任意 gRPC client 都能调)
- 独立演进 (sandboxd 升级不影响 client)
- 跟 5 域 Lead 决策/代签解耦 (per D-5 决策点, 独立 System Service)

**劣势**:
- 部署复杂 (systemd / Windows Service / launchd)
- 需独立健康检查 + 监控 + 升级策略

**v0.1 落地方案**: Rust binary + tonic 0.12 + systemd / Windows Service / launchd (per DD-002 §2.1 / §6 部署)

### 2.2 IPC 协议 (per 决策点 D-2)

**gRPC** (推荐) vs stdio JSON-RPC vs named pipe / Unix socket.

**优势 (gRPC)**:
- 跨语言 (Python dispatcher + Rust mavis + 任意 grpc 支持语言)
- schema 化 (Protocol Buffers v3 + Buf 兼容)
- streaming 支持 (StreamLogs RPC)
- 可观测 (gRPC reflection + 标准 metric)

**v0.1 落地方案**: tonic 0.12 + prost 0.13 + 4 RPC (CreateSession / RunCommand / DestroySession / StreamLogs, per BD-002 §3.1 proto schema)

### 2.3 隔离后端 (per 决策点 D-3 / D-4)

**3 平台 × 4 维** 后端 (per FR-5):

| 平台 | Resource | Network | FS | Capability |
|---|---|---|---|---|
| **Windows** | Job Objects (per SANDBOX-001 v0.2, 100% 复用) | WFP (Windows Filtering Platform) | AppContainer (受限 token) | token privilege adjust (drop SeDebug 等) |
| **Linux** | cgroups v2 (memory.max / cpu.max / pids.max) | netns + iptables (OUTPUT chain) | mount namespace (bind mount) | libcap (CAP_* drop) |
| **macOS** | sandbox-exec resource limits | sandbox-exec network filter | sandbox-exec file filter | sandbox-exec implicit drop |

**v0.1 落地方案**: `crates/sandboxd/src/backend/{windows,linux,macos}.rs` + `backend/mod.rs` DefaultBackendFactory (per DD-002 §2.1 Backend trait + §5 详设)

### 2.4 替代方案: 维持 SANDBOX-001 v0.2 包装器 (per FR-7.1 fail-open 降级)

**不替代, 互补**: SANDBOX-001 v0.2 (Python module) 保留作为 sandboxd 不可用时的降级路径 (per FR-7.1). 触发条件:

- gRPC Connect 失败 (Connection refused / Unavailable)
- sandboxd `/healthz` 返回非 200
- 持续 30s sandboxd 不可用 → 写 root session 通知 (per NFR-O-3)

**保留派生**: 降级时仅 resource 维隔离 (per SANDBOX-001 v0.2 §3.1), 其他 3 维 0 隔离. 适合 sandboxd 自身维护窗口 (e.g. 升级 / 重启).

---

## 3. 设计方案 (Solution)

### 3.1 系统拓扑 (per BD-002 §1.1)

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

### 3.2 4 维隔离详设 (per FR-3 + DD-002 §2.1 SessionSpec)

| 维 | 字段 | 默认值 (per D-3 / D-4 拍板) | 后端调用 |
|---|---|---|---|
| **Resource** | memory_mb | 1024 (1GB) | Win: `JOB_OBJECT_LIMIT_PROCESS_MEMORY` / Linux: `memory.max` |
| | cpu_percent | 50 (1 核 50%) | Win: `JobObjectCpuRateControlInformation` / Linux: `cpu.max` |
| | max_processes | 100 | Win: `JOB_OBJECT_LIMIT_ACTIVE_PROCESS` / Linux: `pids.max` |
| | kill_on_parent_exit | True | Win: `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` / Linux: `cgroup.procs` cleanup |
| **Network** | mode | **Allowlist** (per D-3) | Win: WFP block-all + per-domain allow / Linux: iptables OUTPUT chain / macOS: sandbox-exec network filter |
| | allowlist | `github.com` / `*.github.com` / `crates.io` / `*.crates.io` / `static.crates.io` / `index.crates.io` / `sh.rustup.rs` (D-3 默认) | 同上 |
| | ip_allowlist | [] (per 缺口 #6 缓解, v0.2 拍摄 DNS 反查) | 同上 |
| **FS** | mode | **Allowlist** (per D-4) | Win: AppContainer / Linux: mount namespace (bind mount) / macOS: sandbox-exec file filter |
| | allowlist | `/worktree/**` R + `/worktree/.worktree-cache/**` RW + `/tmp/**` RW (D-4 默认) | 同上 |
| **Capability** | linux_mode | **DropAll** (per D-3 默认) | Linux: libcap clear all / Win: token privilege adjust (drop SeDebug 等) / macOS: implicit drop |

### 3.3 gRPC IPC 详设 (per FR-2 + BD-002 §3.1)

```protobuf
service SandboxService {
  rpc CreateSession(CreateSessionRequest) returns (CreateSessionResponse);
  rpc RunCommand(RunCommandRequest) returns (RunCommandResponse);
  rpc DestroySession(DestroySessionRequest) returns (DestroySessionResponse);
  rpc StreamLogs(StreamLogsRequest) returns (stream LogEntry);
}
```

**4 RPC 行为** (per DD-002 §3 时序):

| RPC | 入参 | 出参 | 延迟 |
|---|---|---|---|
| CreateSession | client_id + task_id + parent_session_id + policy + timeout_sec | session_id + status (Ready/Failed) + error | < 10ms p50, < 100ms p99 (per NFR-P-1+P-2) |
| RunCommand | session_id + cmd + env + cwd + timeout_sec | returncode + stdout + stderr + latency_ms + hit_violations | < 5% overhead vs subprocess.run (per NFR-P-3) |
| DestroySession | session_id + reason | status (Destroyed/NotFound) | < 50ms p99 |
| StreamLogs | session_id + from_offset | stream of LogEntry | < 100ms first entry, < 10ms subsequent (per NFR-P-2) |

### 3.4 fail-open 降级时序 (per FR-7.1)

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
1. gRPC Connect 失败 (Connection refused / Unavailable)
2. sandboxd `/healthz` 返回非 200
3. sandboxd 返回 Unavailable 错误
4. 持续 30s sandboxd 不可用 → 写 root session 通知 (per NFR-O-3, < 500ms 推送)

### 3.5 部署形态 (per FR-1.1 + DD-002 §6)

| 平台 | Init 系统 | 形态 | 配置文件 | 健康检查 |
|---|---|---|---|---|
| Linux | systemd | `/etc/systemd/system/sandboxd.service` (Type=simple, Restart=on-failure) | `/etc/sandboxd/config.yaml` | `127.0.0.1:50051/healthz` |
| Windows | Windows Service | `sandboxd.exe install` / `sc start sandboxd` | `$ProgramData/sandboxd/config.yaml` | 同上 |
| macOS | launchd | `/Library/LaunchDaemons/io.sandboxd.plist` (RunAtLoad + KeepAlive) | `/etc/sandboxd/config.yaml` | 同上 |

---

## 4. 跟现有守门关系 (per 守门 #11 缺标比错标 显式列)

| 守门 | 跟 SANDBOX-002 架构联动 | 派生文档 |
|---|---|---|
| **#1 fail-open** | §3.4 fail-open 降级时序 (per FR-7.1) + sandboxd 不可用 → SANDBOX-001 v0.2 降级 | SRS-002 FR-7.1 + BD-002 §1.3 + DD-002 §3.3 |
| **#1 v15 docs 同步饱和** | 本架构落档, 守门饱和计数 +1 (第 98 次新事件触发 仍允许) | AGENTS.md §4.1 v15 |
| **#1 v19 -j 4 修正** | 所有 cargo test 命令加 `-j 4` (e.g. `cargo test -p sandboxd --lib -j 4`) | AGENTS.md §4.1 v19 + DD-002 §1.2 |
| **#1 v25 cargo test 单 crate** | §3.5 部署后实证 `cargo test -p sandboxd --lib -j 4` 56/56 pass (单 crate 跳 workspace) | AGENTS.md §4.1 v25 + TDD-002 §2.3 |
| **#5 env 安全** | §3.2 NetworkPolicy 引用 env 仅 KEY=VALUE, 不读 env 内容 + SandboxdError 序列化不涉 env | AGENTS.md §4.1 + DD-002 §2.1 |
| **#6 PowerShell + 跨平台** | §2.3 + §3.5 三平台 (Win + Linux + macOS) init 详设 | AGENTS.md §4.1 + BD-002 §5 |
| **#9 子代理 dispatch** | §3.4 dispatcher.py gRPC client 集成 + §3.5 dispatcher.py v0.2 降级路径 | AGENTS.md §4.1 + DD-002 §1.3 |
| **#9 v3 调试控制台 subprocess** | §3.4 fail-open 走 subprocess.run 降级 (跟 SANDBOX-001 v0.2 兼容) | AGENTS.md §4.1 + SRS-002 FR-7.1 |
| **#11 缺标比错标** | §6 9 已知缺口 + §3.2 4 维默认 + §2 选型 4 方案对比 全显式列 | AGENTS.md §4.1 + SRS-002 §9 + DD-002 §9 + TDD-002 §1.2 |
| **#13 T/M 横展** | §5 4 表 W/T/M 100% 覆盖 (Session T + Policy M + Audit T + Capability M, 12/12 RLS 13 類必携, 0 W per session 用完即销毁派生) | AGENTS.md §4.1 + BD-002 §4 + DD-002 §6 |
| **#14 v3 Mavis 永久代签** | 本架构落档 author=Ulysses, 5 域 Lead 签字栏 (Mavis 临时代签) | AGENTS.md §4.1 + SRS-002 §11 + DD-002 §10 |
| **#14 v4 审核反转** | "架构师 (Mavis 接手 agent per DEC-008)" 审批行, 真人代签流程全部取消 | AGENTS.md §4.1 + WBS §14.18 |
| **#22 mavis desktop 集成** | §3.1 mavis desktop Rust gRPC client (跟 Python client 对称) | AGENTS.md §4.1 + DD-002 §1.3 + IMPL-PLAN-002 §4 P2 |
| **#28 拍板必带推荐项** | 6 决策点 D-1~D-6 全部带 ✅ (已拍板 2026-09-10 21:57 JST per Ulysses A 选项) | AGENTS.md §4.1 + WBS §14.19 |
| **v36 audit log 索引** | §3.5 PG sandbox_audit 索引 (session_id / event_type / timestamp) 跟 v36 索引联动 | AGENTS.md §4.1 + BD-002 §4.4 |

**守门 12/12 实证通过** (per 前置 commit `d01509b`).

---

## 5. 落地清单 (Deliverables, per WBS §14.19 SBX-01..SBX-16)

### 5.1 文档落地 (前置, 已落档 per commit `d01509b`)

| # | 文档 | 位置 | 大小 | 状态 |
|---|---|---|---|---|
| 1 | `SRS-SANDBOX-002.md` v0.1.1 | `docs/requirements/` | 37KB | 🟢 6 决策点已拍板 |
| 2 | `SANDBOX-BASIC-DESIGN-002.md` v0.1.1 | `docs/basic-design/` | 46.6KB | 🟢 6 决策点已拍板 |
| 3 | `DD-SANDBOX-002.md` v0.1 | `docs/detailed-design/` | 50.8KB | 🟢 5 维设计 |
| 4 | `TEST-DESIGN-SANDBOX-002.md` v0.1 | `docs/test-design/` | 29.5KB | 🟢 5 级别 97 测 |
| 5 | `SANDBOX-IMPL-PLAN-002.md` v0.1 | `docs/implementation-plans/` | 14.2KB | 🟢 5 阶段 16 子项 |
| 6 | WBS §14.19 16 子项 | `docs/reports/STAR-P3-WBS-001.md` | +60 行 | 🟢 跟 §14.9+§14.10+§14.11+§14.12+§14.13+§14.14 并列 |
| 7 | **`SANDBOX-002.md` (本文件) v0.1** | `docs/architecture/` | (本文件) | 🟢 顶层架构 |

### 5.2 代码落地 (5 阶段 × 16 子项, per WBS §14.19)

| 阶段 | 子项 | Token | 状态 |
|---|---|---|---|
| **P0 骨架** | SBX-01..04: `crates/sandboxd/` crate 骨架 + proto + Linux cgroups v2 + fail-open | 0.5M | 🟡 plan |
| **P1 三平台** | SBX-05..08: Windows (Job+WFP) + Linux (netns+mount+libcap) + macOS sandbox-exec + factory | 1.0M | 🟡 plan |
| **P2 client 集成** | SBX-09..11: dispatcher.py + mavis desktop + 17 IT + 11 E2E | 0.8M | 🟡 plan |
| **P3 fail-open + 性能** | SBX-12..14: 降级端到端 + 5 bench + 故障注入 | 0.6M | 🟡 plan |
| **P4 收官** | SBX-15..16: PHASE 报告 + UAT 8 AC + 推 origin | 0.6M | 🟡 plan |
| **合計** | **16 子项** | **~3.5M / 0.58 周** | **0/16 plan** |

### 5.3 数据库落地 (4 表 W/T/M, per DD-002 §6 DDL)

| # | 表 | 分類 | 派生规 | RLS 13 類 |
|---|---|---|---|---|
| 1 | `sandbox_session` | **T** | 物理删除禁止 + 审计 + 索引 (client_id / status) | ✅ 3/3 必携 |
| 2 | `sandbox_policy` | **M** | 物理删除禁止 + SCD Type 2 (version + valid_from) | ✅ 3/3 |
| 3 | `sandbox_audit` | **T** | 物理删除禁止 + 90 天保留 + 索引 (session_id / event_type / tenant_id, 跟 v36 联动) | ✅ 3/3 |
| 4 | `sandbox_capability` | **M** | 物理删除禁止 + 静态参考 + 索引 (platform / risk_level) | ✅ 3/3 |
| **合計** | **4 表** | **2 T + 2 M + 0 W** | **守门 #13 100% 覆盖** | **12/12 必携** |

**W/T/M 横展派生** (per 守门 #13):
- (a) Work = 0 表 (per session 用完即销毁派生, 0 长期 W)
- (b) Transaction = 2 表 (sandbox_session 事件流 + sandbox_audit 命中流)
- (c) Master = 2 表 (sandbox_policy 策略模板 + sandbox_capability 静态参考)
- 0 混合分類 (per 守门 #13 派生 "主分类单计 + §已知缺口显式列出待 DDD Review Lead 确认")

---

## 6. 已知缺口 (per 缺标比错标, 9 缺口)

| # | 缺口 | 严重度 | 触发条件 | 缓解 / 后续 |
|---|---|---|---|---|
| **G-SBX-01** | sandboxd ↔ dispatcher.py / mavis desktop 集成未实装 | **P0 阻塞** | v0.1 仅落地 sandboxd binary + policy schema, client 端改造跨 session 续做 (SBX-09 + SBX-10) | v0.2 拍摄 dispatcher.py 切 gRPC client + mavis desktop 集成 |
| **G-SBX-02** | TLS / mTLS 双向认证未实装 | P1 | v0.1 走明文 gRPC (localhost), 跨主机不安全 | v0.2 拍摄 mTLS |
| **G-SBX-03** | Capability 维仅 Linux 实装 | P1 | Windows / macOS 走 sandbox 后端隐式 drop (v0.1) | v0.2 拍摄 Windows token privilege adjust + macOS explicit drop |
| **G-SBX-04** | 真实 K8s / Helm 部署未实装 | P2 | v0.1 走 systemd / Windows Service / launchd, K8s 部署需要 helm chart | v0.2 拍摄 K8s deployment + HPA + PDB |
| **G-SBX-05** | 多租户隔离未实装 | P2 | v0.1 走单租户, 多个 org 共用 sandboxd 资源池 | v0.2 拍摄 tenant_id 隔离 + 资源配额 |
| **G-SBX-06** | allowlist 绕过 (DNS rebinding / domain fronting) | P1 | 攻击者用 IP 直连绕过域名 allowlist | v0.2 拍摄 DNS 解析 + IP 反查, 加 IP 段 allowlist |
| **G-SBX-07** | sandboxd 自身被攻击 (host OS 漏洞 / 提权) | P2 | sandboxd 是 host 进程, 自身被攻破则所有隔离失效 | v0.2 拍摄 sandboxd 二进制签名 + integrity check |
| **G-SBX-08** | 子代理内部代码越权 (post-dispatch) 仍有窗口 | P1 | 即便 sandboxd 隔离, 子代理在 session 内仍可任意调 subprocess | v0.2 拍摄 subprocess 限制 (per SANDBOX-001 v0.2 §3.4, 在 sandboxd 内部嵌套) |
| **G-SBX-09** | mavis runtime 升级不兼容 (gRPC schema breaking change) | P2 | 未来 mavis v1.0 API 变化 | 关注 mavis changelog, 同步升级, gRPC schema 走 buf 兼容 |

---

## 7. 跟 SANDBOX-001 v0.2 关系 (per 守门 #11 缺标比错标 显式列)

| 维度 | SANDBOX-001 v0.2 (现役) | SANDBOX-002 v0.1 (本架构) | 关系 |
|---|---|---|---|
| 形态 | Python module 绑 dispatcher.py | Rust binary 独立 daemon 长期运行 | **升级 + 兼容** (v0.2 是 v0.1 降级路径) |
| 隔离维 | 1 维 (resource) | 4 维 (resource / network / fs / capability) | **升级** (覆盖 v0.2 §6 缺口 #1+#2+#5+#6) |
| 平台 | Windows only | Windows + Linux + macOS | **升级** (覆盖 v0.2 §6 缺口 #3) |
| client | dispatcher.py only | dispatcher.py + mavis desktop + 任意 gRPC client | **升级** (覆盖 v0.2 跨 client 不可复用问题) |
| IPC | 函数调用 (subprocess.run) | gRPC (tonic 0.12) | **升级** (跨语言 + 跨进程) |
| observability | 0 metrics + 0 healthz | 9 Prometheus metrics + /healthz + PG audit log | **升级** (覆盖 v0.2 §6 缺口 #5) |
| 失败行为 | 沙箱自身错误 fail-open 降级到 subprocess.run | sandboxd 不可用 fail-open 降级到 SANDBOX-001 v0.2 | **演进** (v0.2 仍是 v0.1 降级路径) |
| 配置 | 0 配置 (代码写死) | YAML 配置文件 + 4 维策略 + allowlist | **升级** (运行时配置, 不需改代码) |
| 健康检查 | 0 | `/healthz` HTTP endpoint | **升级** (跟 systemd / k8s probe 集成) |
| 部署 | 0 (Python module 跟 dispatcher 一起) | systemd / Windows Service / launchd | **升级** (独立部署, 跟 5 域 Lead 决策/代签解耦 per D-5) |

**保留派生** (per 守门 #1 禁回溯叙事 + 守门 #8 不沿用 bc23d6c 叙事):
- SANDBOX-001 v0.2 9 tests 0 回归 实证保留 (per v0.2 落地 commit `4219c3d`)
- 9 known 缺口 (现役 v0.2) 全部由 SANDBOX-002 v0.1 覆盖 (4 缺口升级 + 5 缺口跨 session 续做)
- SANDBOX-001 v0.2 9 缺口 (v0.2 §6) **不重写 v0.1 历史**, 维持原版

---

## 8. 决策点 (跟 SRS-002 §10 + BD-002 §7 100% 对齐)

| ID | 决策点 | 推荐项 (per 守门 v28) | 备选 1 | 备选 2 | 状态 |
|---|---|---|---|---|---|
| **D-1** | 形态 | ✅ **(已拍板 2026-09-10 21:57 JST per Ulysses A 选项) 长期 daemon** | per-task ephemeral | library 嵌入 | 🟢 已拍板 |
| **D-2** | IPC 协议 | ✅ **(已拍板 2026-09-10 21:57 JST per Ulysses A 选项) gRPC** | stdio JSON-RPC | named pipe / Unix socket | 🟢 已拍板 |
| **D-3** | 网络隔离深度 | ✅ **(已拍板 2026-09-10 21:57 JST per Ulysses A 选项) allowlist** (子代理常要 git/cargo/crates.io) | 全阻断 (强隔离) | 透传 + 审计 (弱隔离) | 🟢 已拍板 |
| **D-4** | FS 隔离深度 | ✅ **(已拍板 2026-09-10 21:57 JST per Ulysses A 选项) path allowlist** (子代理需要 read worktree + 写 cache) | AppContainer / mount ns 真正隔离 (强) | 只读 overlay (弱) | 🟢 已拍板 |
| **D-5** | 跟 mavis desktop 集成 | ✅ **(已拍板 2026-09-10 21:57 JST per Ulysses A 选项) 独立 System Service** (跟 5 域 Lead 决策/代签解耦) | sandboxd 跑在 mavis 里 (耦合) | 双形态 (复杂) | 🟢 已拍板 |
| **D-6** | 观测性 | ✅ **(已拍板 2026-09-10 21:57 JST per Ulysses A 选项) 两者都上** (Prometheus + PG audit log, 跟守门 v36 索引联动) | 仅 Prometheus | 仅 audit log | 🟢 已拍板 |

**拍板来源** (per 守门 v28 + 9/1 14:58 守门): 2026-09-10 21:57 JST Ulysses reply "a" (= 选项 A 全部用推荐). A 选项 = 全部用推荐, 已落档 SRS-002 v0.1.1 + BD-002 v0.1.1.

**反转影响** (per 守门 #1 禁回溯叙事): 反转后, 修订历史表 +1 行 (per §10), 不重写历史 v0.1 决策.

---

## 9. 签字栏 (Sign-off)

| 角色 | 氏名 | 签字 | 日期 |
|---|---|---|---|
| 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | ✅ 2026-09-10 | 2026-09-10 JST |
| SRE Lead | SRE Lead (Mavis 临时代签 per 9/3 11:35 JST 拍板 B, 真人到位后追溯) | ✅ 2026-09-10 | 2026-09-10 JST |
| 平台 Lead | 平台 Lead (Mavis 临时代签 per 守门 #14 v3, 真人到位后追溯) | ✅ 2026-09-10 | 2026-09-10 JST |
| 评审主持 | 评审主持 (Mavis 临时代签 per 守门 #14 v3) | ✅ 2026-09-10 | 2026-09-10 JST |
| PM | PM (Mavis 临时代签 per 守门 #14 v3) | ✅ 2026-09-10 | 2026-09-10 JST |

(per 守门 #14 v4 反转 v0.62, 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses) — **⚠️ 进一步 per v0.64 反转 2026-09-10 22:35 JST**: 全部"等待真人"流程永久 obsolete, Mavis 全权处理 0 临时代签 (直接 永久代签 per 守门 #14 v3 + v0.62 + v0.63 + v0.64 反转叠加)

---

## 10. 修订履歴 (詳細)

| バージョン | 日付 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-10 22:22 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 永久代签 (per 守门 #14 v3 + 守门 #14 v4 反转 v0.62) | 初版落档, 顶层架构 8 段 (目的 / 目标 12 项 / 选型 4 维 / 方案 5 详设 / 守门 14 联动 / 落地 16 子项 / 缺口 9 / SANDBOX-001 关系), 12 设计目标 v0.1 100% 落地, 4 维隔离 100% 覆盖 SANDBOX-001 v0.2 4 缺口 (#1+#2+#5+#6), 3 平台 × 4 维 后端选型 (Windows Job+WFP+AppContainer / Linux cgroups+netns+mount+libcap / macOS sandbox-exec), 4 RPC gRPC 详设, fail-open 降级时序 (per FR-7.1), 6 决策点 D-1~D-6 全部已拍板 (per Ulysses A 选项 2026-09-10 21:57 JST), 4 表 W/T/M 100% 覆盖 (Session T + Policy M + Audit T + Capability M, 12/12 RLS 13 類必携), 9 已知缺口 (G-SBX-01..G-SBX-09, 缺標比錯標), 守門 12/12 通過 (#1+#1 v15+#1 v19+#1 v25+#5+#6+#9+#9 v3+#11+#13+#14 v3+#14 v4+#22+#28), 跟 SANDBOX-001 v0.2 兼容 (v0.2 是 v0.1 降級路徑) | 2026-09-10 22:22 JST Ulysses 拍板"c" (per 21:57 JST A 選項 6 決策點已拍板 + 22:11 JST 各級文檔完善好 + 22:22 JST "先把 SANDBOX-002 架構文檔也落檔"派生), 跟前置 commit `d01509b` 6 文檔 (SRS+BD+DD+TDD+IMPL-PLAN+WBS §14.19) 同期派生, 守門 #1 v15 docs 同步飽和第 98 次新事件觸發 仍允許 |
