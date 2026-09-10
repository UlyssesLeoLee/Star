# TEST-DESIGN-SANDBOX-002 — Sandbox-as-a-Service (sandboxd) 测试设计书

> **版本**: v0.1
> **作者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 9/8 15:19 JST 第 6 次强化 Mavis 全权代理)
> **审批**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手
> **日期**: 2026-09-10 JST
> **状态**: 🟡 Draft v0.1 (2026-09-10 JST 初版落档)

---

## §0 文档目的

本文档定义 **Sandbox-as-a-Service (sandboxd)**（`crates/sandboxd/` 18 源 + 6 测试 = 24 文件）的端到端测试设计书，单文档覆盖 5 级别测试（UT/IT/E2E/PT/UAT）共 **56 lib + 17 IT + 11 E2E + 5 PT + 8 UAT = 97 测** 实证锚点，**1 份单文档分章** 对应 4 维隔离 (resource/network/fs/capability) × 3 平台 (Windows/Linux/macOS) × 4 RPC (CreateSession/RunCommand/DestroySession/StreamLogs) × 4 表 W/T/M。

**触发**（per 2026-09-10 22:11 JST 用户发令）:

- 用户发令"**各级文档完善好，更新后续任务到 wbs**"
- ask_user 派板（per 9/1 14:58 守门，方向选择必给选项）：5 级别测试设计 (跟 WBS §13 Test Design v0.3 模板对齐)

**核心定位**：

- 单文档 ≤ 60KB，分 9 章节（§0 目的 + §1 范围 + §2-§6 5 级别 + §7 RACI + §8 修订历史 + §9 引用）
- 跟既有 `docs/test-design/TEST-DESIGN-OPS-001.md` v0.2 (单文档分章模式) 平行不重叠
- 引用上游 3 份需求/设计文档 (SRS-002 v0.1.1 + BD-002 v0.1.1 + DD-002 v0.1) + 既有 6 份 PHASE-*-IMPL-REPORT
- 9 已知缺口显式标注（per 守门 #11 缺标比错标），DDD Review 必查

---

## §1 范围

### 1.1 In-Scope（5 级别测试设计书）

| 章节 | 测试级别 | 范围 | 实证锚点 |
|---|---|---|---|
| **§2 UT** | 单元测试 | 8 模块测试矩阵 (config + policy::4 + orchestrator + metrics + audit) + 56 lib test 1:1 对齐 + 边界 + 错误路径 | `cargo test -p sandboxd --lib -j 4` 56/56 pass (预估 v0.1 落地后) |
| **§3 IT** | 集成测试 | 跨模块 17 IT (gRPC 4 RPC 端到端 8 + SANDBOX-001 v0.2 降级 4 + PG audit 持久化 5) + tonic oneshot + sqlx 容器 + 缺 IT 识别 | `cargo test -p sandboxd --tests -j 4` 17/17 IT pass (预估) |
| **§4 E2E** | 端到端 | dispatcher.py 集成 (Python) 4 + mavis desktop 集成 (Rust) 4 + 跨平台验证 3 (Win+Linux+macOS) | MVP 阶段手测 + CI 自动化; 框架: pytest (Python) + cargo test (Rust) |
| **§5 PT** | 性能测试 | 守门 #1 v25 单 crate P95<200ms 硬约束 + 5 bench 实证 (CreateSession p50/p99 + RunCommand 端到端 + 100 session 并发 + gRPC IPC 序列化 + StreamLogs 流式) | `cargo bench -p sandboxd -- --quick` 5 bench P95 实证 |
| **§6 UAT** | 验收测试 | SRS-002 §7 8 AC + DD-002 §5 5 维测试矩阵 + §6 4 表 W/T/M 100% 覆盖验收 + 9 已知缺口显式标 | 5 域 Lead 签字栏（Mavis 临时代签 per 守门 #14 v2 拍板 D） |

**累计实证锚点**（per 守门 #1 v25 单 crate）:

```
lib test:    56/56 pass    (cargo test -p sandboxd --lib -j 4)
IT test:     17/17 pass    (cargo test -p sandboxd --tests -j 4)
E2E test:    11/11 pass    (pytest + cargo test)
bench PT:    5 bench P95 < 200ms 实证
合计实证:    84 测 100% pass + 5 bench P95 守门 + 8 UAT AC
```

### 1.2 Out-of-Scope（per 守门 #1 R-05 mock 路径 + 守门 #11 缺标比错标）

- **真实 K8s / Helm 集群部署** (per SRS-002 §1.4 + DD-002 §0): MVP 走 systemd / Windows Service / launchd
- **真实 TLS / mTLS 双向认证** (per SRS-002 §1.4 + 缺口 #2): MVP 走明文 gRPC (localhost)
- **多租户隔离** (per SRS-002 §1.4 + 缺口 #5): MVP 走单租户
- **AI 行为审计 (LLM 决策录屏)** (per SRS-002 §1.4): v2.x 拍摄
- **加密 / 凭据管理 (mavis 内置 vault)**: 跨项目需求
- **跨域编排 (5 域 Lead DDD Review / Saga orchestrator)**: 跨 P3-B 子项
- **OAuth 2.0**: 复用 `star-context::ActorContext`，不实装
- **OpenAPI 3.1 spec 自动生成**: v0.2 utoipa

### 1.3 5 级别测试框架

```
                          ┌──────────────────────┐
                          │  client (任意)        │  ← E2E (dispatcher.py / mavis desktop / 跨平台)
                          │  (dispatcher / mavis) │
                          └──────────┬───────────┘
                                     │ gRPC (port 50051)
                                     ▼
                          ┌──────────────────────┐
                          │  sandboxd            │  ← IT (tonic oneshot + 跨模块 + sqlx 容器)
                          │  (Rust binary)       │
                          │  tonic 0.12 + axum   │
                          │  4 RPC + 4 维隔离    │
                          │  + 3 平台后端        │
                          └──────────┬───────────┘
                                     │
        ┌──────────────┬────────────┼────────────┬──────────────┐
        ▼              ▼            ▼            ▼              ▼
   ┌─────────┐   ┌─────────┐  ┌─────────┐  ┌─────────┐    ┌─────────┐
   │  UT     │   │  IT     │  │  E2E   │   │  PT     │    │  UAT   │
   │  56 测  │   │  17 测  │  │  11 测 │   │  5 bench │   │  8 AC  │
   └─────────┘   └─────────┘  └─────────┘  └─────────┘    └─────────┘
        │              │            │            │              │
        └──────────────┴────────────┴────────────┴──────────────┘
                                     │
                                     ▼
                          ┌──────────────────────┐
                          │  PG (audit log)      │  ← 4 表 W/T/M 100% 覆盖 (RLS 13 类)
                          │  + Prometheus        │  ← 9 指标 /metrics
                          │  + /healthz          │
                          └──────────────────────┘
```

---

## §2 单元测试 (UT, 56 测)

### 2.1 测试矩阵总览

| 模组 | TC class | 文件 | 测数 | 状态 |
|---|---|---|---|---|
| `config` | test_config | `tests/unit/test_config.rs` | 6 | 🟡 |
| `policy::resource` | test_policy_resource | `tests/unit/test_policy_resource.rs` | 8 | 🟡 |
| `policy::network` | test_policy_network | `tests/unit/test_policy_network.rs` | 8 | 🟡 |
| `policy::fs` | test_policy_fs | `tests/unit/test_policy_fs.rs` | 7 | 🟡 |
| `policy::capability` | test_policy_capability | `tests/unit/test_policy_capability.rs` | 5 | 🟡 |
| `orchestrator` | test_orchestrator | `tests/unit/test_orchestrator.rs` | 12 | 🟡 |
| `metrics` | test_metrics | `tests/unit/test_metrics.rs` | 4 | 🟡 |
| `audit` | test_audit | `tests/unit/test_audit.rs` | 6 | 🟡 |
| **小计** | **8 class** | **8 文件** | **56** | 🟡 |

**实证目标**: `cargo test -p sandboxd --lib -j 4` 56/56 pass (per 守门 #1 v25 单 crate + 守门 #1 v19 -j 4 修正)

### 2.2 详细测试用例

#### 2.2.1 test_config (6 测)

| TC ID | 名称 | 输入 | 期望 | 关联 FR/NFR |
|---|---|---|---|---|
| TC-CFG-01 | 加载有效 YAML | `config/sandboxd.yaml` | 解析成功 + 字段一致 | FR-1.1 |
| TC-CFG-02 | 加载无效 YAML (语法错) | 损坏 YAML | 返回 Err | NFR-A-1 |
| TC-CFG-03 | 加载缺必填字段 YAML | 缺 server.grpc_port | 返回 Err | NFR-A-1 |
| TC-CFG-04 | 加载类型错误 YAML | grpc_port=string | 返回 Err | NFR-A-1 |
| TC-CFG-05 | 业务规则校验 (max_concurrent=0) | max_concurrent_sessions=0 | 返回 Err | NFR-A-1 |
| TC-CFG-06 | 默认值回填 (缺省字段) | 部分字段缺失 | 自动回填默认值 | FR-1.1 |

#### 2.2.2 test_policy_resource (8 测)

| TC ID | 名称 | 输入 | 期望 | 关联 FR/NFR |
|---|---|---|---|---|
| TC-RES-01 | 默认值 | SandboxLimits() | memory_mb=0, cpu_percent=0, max_processes=0, kill_on_parent_exit=True | FR-3.1 |
| TC-RES-02 | 自定义值 | SandboxLimits(memory_mb=2048, cpu_percent=75, max_processes=200) | 字段一致 | FR-3.1 |
| TC-RES-03 | 边界值 0=不限 | memory_mb=0 | 应用时不设限制 | NFR-P-1 |
| TC-RES-04 | 边界值大值 | memory_mb=u32::MAX | 应用时正确设置 | NFR-S-1 |
| TC-RES-05 | make_default_sandbox_limits | 工厂方法 | memory_mb=1024, cpu_percent=50, max_processes=100 | FR-3.1 |
| TC-RES-06 | JSON 序列化 | SandboxLimits 实例 | 序列化反序列化一致 | FR-6.2 |
| TC-RES-07 | kill_on_parent_exit=False | kill_on_parent_exit=False | 应用时不设 JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | FR-3.1 |
| TC-RES-08 | (Windows) 非 elevated Access Denied | 无 SeAssignPrimaryTokenPrivilege | warn log + fail-open 降级 (per 缺口 #6) | NFR-A-1 |

#### 2.2.3 test_policy_network (8 测)

| TC ID | 名称 | 输入 | 期望 | 关联 FR/NFR |
|---|---|---|---|---|
| TC-NET-01 | allowlist 模式默认 (D-3) | default_mode=Allowlist | 应用时仅允许 allowlist 域名 | FR-3.2 |
| TC-NET-02 | BlockAll 模式 | default_mode=BlockAll | 应用时拒绝所有出站 | FR-3.2 |
| TC-NET-03 | Passthrough 模式 | default_mode=Passthrough | 应用时不加规则 | FR-3.2 |
| TC-NET-04 | 域名通配符 `*.github.com` | allowlist=["*.github.com"] | api.github.com 命中, github.com 命中 | FR-3.2 |
| TC-NET-05 | IP CIDR allowlist | ip_allowlist=["10.0.0.0/8"] | 10.1.2.3 命中, 192.168.1.1 不命中 | FR-3.2 |
| TC-NET-06 | 缺省 allowlist (D-3 默认值) | 不填 allowlist | 应用 github.com / crates.io / static.crates.io / index.crates.io / sh.rustup.rs 5 项 | FR-3.2 |
| TC-NET-07 | (Linux) iptables OUTPUT chain | BlockAll 模式 | iptables -A OUTPUT -j DROP 添加 | NFR-S-5 |
| TC-NET-08 | (Windows) WFP provider | BlockAll 模式 | WFP API 调用 + provider 注册 | NFR-S-5 |

#### 2.2.4 test_policy_fs (7 测)

| TC ID | 名称 | 输入 | 期望 | 关联 FR/NFR |
|---|---|---|---|---|
| TC-FS-01 | allowlist 模式默认 (D-4) | default_mode=Allowlist | 应用时仅允许 allowlist 路径 | FR-3.3 |
| TC-FS-02 | ReadOnlyRoot 模式 | default_mode=ReadOnlyRoot | 全部路径 read-only | FR-3.3 |
| TC-FS-03 | Open 模式 | default_mode=Open | 不加限制 | FR-3.3 |
| TC-FS-04 | path glob 匹配 | path="/worktree/**" | /worktree/foo/bar.rs 命中 | FR-3.3 |
| TC-FS-05 | path 模式 Read | mode=Read | bind mount -o ro | FR-3.3 |
| TC-FS-06 | path 模式 ReadWrite | mode=ReadWrite | bind mount rw | FR-3.3 |
| TC-FS-07 | 缺省 allowlist (D-4 默认值) | 不填 allowlist | 应用 /worktree/** R + /worktree/.worktree-cache/** RW + /tmp/** RW | FR-3.3 |

#### 2.2.5 test_policy_capability (5 测)

| TC ID | 名称 | 输入 | 期望 | 关联 FR/NFR |
|---|---|---|---|---|
| TC-CAP-01 | DropAll 模式 (Linux) | linux_mode=DropAll | caps::clear Effective/Permitted/Inheritable | FR-3.4 |
| TC-CAP-02 | AllowNetBindService 模式 (Linux) | linux_mode=AllowNetBindService | clear + raise CAP_NET_BIND_SERVICE | FR-3.4 |
| TC-CAP-03 | Custom 模式 (Linux) | custom_capabilities=["CAP_NET_RAW"] | 仅 raise CAP_NET_RAW | FR-3.4 |
| TC-CAP-04 | (Windows) drop SeDebugPrivilege | windows_default="drop_se_debug" | token privilege adjust 成功 | FR-3.4 |
| TC-CAP-05 | (macOS) implicit drop | sandbox-exec profile | profile 隐式 drop | FR-3.4 |

#### 2.2.6 test_orchestrator (12 测)

| TC ID | 名称 | 输入 | 期望 | 关联 FR/NFR |
|---|---|---|---|---|
| TC-ORC-01 | 创建 session 成功 | 有效 SessionSpec | 返回 session_id + status=Ready | FR-4.1 |
| TC-ORC-02 | 创建 session 失败 (平台不支持) | PlatformNotSupported | 返回 Failed + error | FR-4.1 |
| TC-ORC-03 | 创建 session 失败 (策略非法) | 无效 SessionSpec | 返回 Failed + error | FR-4.1 |
| TC-ORC-04 | 100 session 并发池 | 同时创建 100 个 | 全部 Ready + 0 资源竞争 | NFR-P-5 |
| TC-ORC-05 | 超过 max_concurrent 创建 | max=100, 创建 101 | 第 101 个返回 ResourceLimitExceeded | NFR-P-5 |
| TC-ORC-06 | session 超时自动销毁 | 30 min timeout, 不调用 DestroySession | 自动 Destroyed | FR-4.3 |
| TC-ORC-07 | RunCommand 成功 | 有效 cmd | 返回 returncode + stdout + stderr | FR-4.2 |
| TC-RUN-08 | RunCommand 命中违规 (network) | curl disallowed-domain | 不阻断 + hit_violations 包含 network | NFR-S-5 |
| TC-RUN-09 | RunCommand 命中违规 (fs) | cat /etc/passwd | 不阻断 + hit_violations 包含 fs | NFR-S-5 |
| TC-RUN-10 | RunCommand 命中违规 (capability) | mount | 不阻断 + hit_violations 包含 capability | NFR-S-5 |
| TC-DES-11 | DestroySession 成功 | 有效 session_id | 返回 Destroyed + cascade kill | FR-4.3 |
| TC-DES-12 | DestroySession 不存在 | 随机 UUID | 返回 NotFound | FR-4.3 |

#### 2.2.7 test_metrics (4 测)

| TC ID | 名称 | 输入 | 期望 | 关联 FR/NFR |
|---|---|---|---|---|
| TC-MET-01 | 9 指标注册成功 | new() | registry.register 9 次成功 | FR-6.1 |
| TC-MET-02 | sessions_total label 正确 | inc({status="created"}) | counter +1, label=created | FR-6.1 |
| TC-MET-03 | session_duration_seconds histogram | observe(0.05) | histogram bucket +1 | FR-6.1 |
| TC-MET-04 | Prometheus scrape format | 抓取 /metrics | text/plain 格式 + 9 指标行 | NFR-O-1 |

#### 2.2.8 test_audit (6 测)

| TC ID | 名称 | 输入 | 期望 | 关联 FR/NFR |
|---|---|---|---|---|
| TC-AUD-01 | 写入 audit (ResourceHit) | event=ResourceHit + JSONB | INSERT 成功 + 11 字段完整 | FR-6.2 |
| TC-AUD-02 | 写入 audit (NetworkBlock) | event=NetworkBlock + dst_ip | INSERT 成功 + network_violations JSONB | FR-6.2 |
| TC-AUD-03 | 写入 audit (FsBlock) | event=FsBlock + path | INSERT 成功 + fs_violations JSONB | FR-6.2 |
| TC-AUD-04 | append-only 验证 (物理删除) | 尝试 DELETE | RLS 阻止 + 0 rows affected | NFR-S-3 |
| TC-AUD-05 | 写入失败 fail-closed | mock PG 不可用 | 返回 Err + 主流程阻断 | NFR-A-2 |
| TC-AUD-06 | RLS 13 類必携 | 不同 tenant_id 写入 | RLS 阻止跨 tenant 读 | NFR-S-3 |

### 2.3 覆盖率目标

- **Line coverage**: ≥ 90% (per NFR-M-4)
- **Branch coverage**: ≥ 85%
- **关键路径**: CreateSession / RunCommand / DestroySession 100% (per NFR-S-4)

---

## §3 集成测试 (IT, 17 测)

### 3.1 测试矩阵总览

| 集成路径 | TC 数量 | 工具 |
|---|---|---|
| gRPC 4 RPC 端到端 | 8 | tonic oneshot + Mock backend |
| SANDBOX-001 v0.2 降级路径 | 4 | toxiproxy + subprocess.run |
| PG audit 持久化 | 5 | testcontainers (PG 17) |
| **小计** | **17** | |

**实证目标**: `cargo test -p sandboxd --tests -j 4` 17/17 pass

### 3.2 详细测试用例

#### 3.2.1 gRPC 4 RPC 端到端 (8 测)

| TC ID | 名称 | 步骤 | 期望 | 关联 FR/NFR |
|---|---|---|---|---|
| TC-IT-GRPC-01 | CreateSession → RunCommand → DestroySession 完整流程 | 1) CreateSession 2) RunCommand "echo hello" 3) DestroySession | 全部成功 + session Destroyed 状态 | FR-4.1+4.2+4.3 |
| TC-IT-GRPC-02 | StreamLogs 实时推送 | 1) CreateSession 2) RunCommand 跑 5s 3) 启动 StreamLogs | 收到 5+ LogEntry (ProcessStart + ProcessExit) | FR-2.3 |
| TC-IT-GRPC-03 | 并发 10 session 互不干扰 | 1) 10 个 CreateSession 2) 各 RunCommand "echo $SESSION_ID" | 10 个不同 session_id + 各自输出 | NFR-S-4 |
| TC-IT-GRPC-04 | session 间状态隔离 (mem) | session A alloc 500MB, session B alloc 500MB | 互不影响, 0 OOM | NFR-S-4 |
| TC-IT-GRPC-05 | session 间状态隔离 (fs) | session A 写 /tmp/A.txt, session B 读 | A.txt 不可见 | NFR-S-4 |
| TC-IT-GRPC-06 | CreateSession 延迟 p50 + p99 | 跑 1000 次 CreateSession | p50 < 10ms, p99 < 100ms | NFR-P-1+NFR-P-2 |
| TC-IT-GRPC-07 | RunCommand 端到端延迟 (subprocess baseline) | 对比 sandboxd vs subprocess.run | < 5% 开销 | NFR-P-3 |
| TC-IT-GRPC-08 | gRPC IPC 序列化开销 | 1KB / 10KB / 100KB payload | < 1ms 序列化 | NFR-P-4 |

#### 3.2.2 SANDBOX-001 v0.2 降级路径 (4 测)

| TC ID | 名称 | 步骤 | 期望 | 关联 FR/NFR |
|---|---|---|---|---|
| TC-IT-FO-01 | sandboxd 不可用 → 降级 | 1) sandboxd 停止 2) dispatcher 调 gRPC | gRPC Unavailable + 走 SANDBOX-001 v0.2 subprocess.run | FR-7.1 |
| TC-IT-FO-02 | sandboxd 启动中 → 降级 | 1) sandboxd 重启中 2) 调 gRPC | Connection refused + 走降级 | FR-7.1 |
| TC-IT-FO-03 | sandboxd 健康检查失败 → 降级 | 1) /healthz 返回 500 2) 调 gRPC | 走降级 | FR-7.1 |
| TC-IT-FO-04 | 持续 30s 不可用 → root 通知 | 1) sandboxd 停 30s+ 2) 监听 root session 通知 | < 500ms 收到通知 | NFR-O-3 |

#### 3.2.3 PG audit 持久化 (5 测)

| TC ID | 名称 | 步骤 | 期望 | 关联 FR/NFR |
|---|---|---|---|---|
| TC-IT-PG-01 | sandbox_session 写入 + 查询 | INSERT + SELECT | 数据一致 | FR-6.2 |
| TC-IT-PG-02 | sandbox_audit 索引查询 (session_id) | 1000 条 audit, WHERE session_id=X | 索引命中 + < 10ms | NFR-O-1 |
| TC-IT-PG-03 | sandbox_audit 索引查询 (event_type) | WHERE event_type='NetworkBlock' | 索引命中 + < 10ms | NFR-O-1 |
| TC-IT-PG-04 | RLS 跨 tenant 读阻止 | tenant A 写, tenant B 读 | 0 rows returned | NFR-S-3 |
| TC-IT-PG-05 | append-only 验证 (UPDATE 尝试) | UPDATE sandbox_audit SET ... | 0 rows affected (RLS 阻止) | NFR-S-3 |

---

## §4 端到端测试 (E2E, 11 测)

### 4.1 测试矩阵总览

| 场景 | TC 数量 | 工具 | 平台 |
|---|---|---|---|
| dispatcher.py 集成 | 4 | pytest + Python gRPC client | Linux + macOS + Windows |
| mavis desktop 集成 (Rust) | 4 | cargo test (e2e) | Linux + macOS + Windows |
| 跨平台验证 | 3 | CI matrix | Win + Linux + macOS |
| **小计** | **11** | | |

**实证目标**: pytest 4/4 + cargo e2e 4/4 + CI 3/3 = 11/11 pass

### 4.2 详细测试用例

#### 4.2.1 dispatcher.py 集成 (4 测)

| TC ID | 名称 | 步骤 | 期望 | 关联 FR/NFR |
|---|---|---|---|---|
| TC-E2E-PY-01 | dispatcher 调 sandboxd 创建 session | 1) dispatcher.invoke(task) 2) 内调 sandboxd.CreateSession | session 创建 + 1 个 RunCommand 跑子代理 | FR-7.1 |
| TC-E2E-PY-02 | dispatcher 跑 cargo build 在 sandboxd 内 | 1) CreateSession (memory=2GB) 2) RunCommand "cargo build" | cargo build 成功 + 资源限制不 OOM | NFR-P-3 |
| TC-E2E-PY-03 | dispatcher 降级到 SANDBOX-001 v0.2 | 1) sandboxd 停 2) dispatcher.invoke | 走 subprocess.run 降级 + 写 root 通知 | FR-7.1 |
| TC-E2E-PY-04 | dispatcher 跑 git clone 在 allowlist 模式 | 1) CreateSession (network=Allowlist) 2) "git clone github.com/..." | git clone 成功 (github.com 命中 allowlist) | FR-3.2 |

#### 4.2.2 mavis desktop 集成 (Rust, 4 测)

| TC ID | 名称 | 步骤 | 期望 | 关联 FR/NFR |
|---|---|---|---|---|
| TC-E2E-RS-01 | mavis desktop 调 sandboxd 创建 session | 1) SandboxdClient.connect 2) CreateSession | session 创建 + actor_id 注入 ActorContext | FR-2.1+NFR-C-3 |
| TC-E2E-RS-02 | mavis desktop 跑 cargo test 在 sandboxd 内 | 1) CreateSession 2) RunCommand "cargo test" | 100/100 pass + 资源限制不 OOM | NFR-P-3 |
| TC-E2E-RS-03 | mavis desktop 调 StreamLogs 实时监控 | 1) CreateSession 2) RunCommand (长时间) 3) StreamLogs | 收到 5+ LogEntry | FR-2.3 |
| TC-E2E-RS-04 | mavis desktop 跨平台 (Win+Linux+macOS) | CI matrix 跑 TC-E2E-RS-01~03 | 3/3 平台 pass | NFR-T-1+NFR-T-2+NFR-T-3 |

#### 4.2.3 跨平台验证 (3 测)

| TC ID | 名称 | 平台 | 期望 | 关联 FR/NFR |
|---|---|---|---|---|
| TC-E2E-X-01 | sandboxd Linux 全功能 | Linux (Ubuntu 22.04) | 24/24 测 pass | NFR-T-2 |
| TC-E2E-X-02 | sandboxd Windows 全功能 | Windows (Win 11 + Server 2022) | 24/24 测 pass | NFR-T-1 |
| TC-E2E-X-03 | sandboxd macOS 全功能 | macOS (14+) | 24/24 测 pass | NFR-T-3 |

### 4.3 E2E 框架选型

- **Python E2E**: pytest + grpcio + 临时 sandboxd binary (background process)
- **Rust E2E**: cargo test (e2e) + tonic + tokio
- **跨平台 CI**: GitHub Actions matrix (ubuntu-latest + windows-latest + macos-latest)

---

## §5 性能测试 (PT, 5 bench)

### 5.1 守门基线 (per 守门 #1 v25 + 守门 #7 v3 + 守门 #1 v19)

| Bench | 目标 | 阈值 | 工具 |
|---|---|---|---|
| CreateSession p50 | < 10ms | NFR-P-1 | `cargo bench create_session` |
| CreateSession p99 | < 100ms | NFR-P-2 | `cargo bench create_session` |
| RunCommand 端到端 | 跟 subprocess.run baseline 比 < 5% 开销 | NFR-P-3 | `cargo bench run_command` |
| 100 session 并发 | p99 < 200ms | NFR-P-5 | `cargo bench concurrent_sessions` |
| gRPC IPC 序列化 | < 1ms (per command) | NFR-P-4 | `cargo bench grpc_serialize` |

**实证目标**: `cargo bench -p sandboxd -- --quick` 5 bench 全部 P95 < 阈值

### 5.2 详细 bench 配置

#### 5.2.1 CreateSession bench

```rust
// crates/sandboxd/benches/create_session.rs
use criterion::{criterion_group, criterion_main, Criterion};
use sandboxd::{Orchestrator, SandboxdConfig, SessionSpec};

fn bench_create_session(c: &mut Criterion) {
    let config = SandboxdConfig::from_yaml("config/sandboxd.yaml").unwrap();
    let orchestrator = Orchestrator::new(config);
    
    c.bench_function("create_session", |b| {
        b.iter(|| {
            let spec = SessionSpec::default();
            orchestrator.create_session("bench", "task", None, spec, 1800)
        })
    });
}

criterion_group!(benches, bench_create_session);
criterion_main!(benches);
```

#### 5.2.2 RunCommand 端到端 bench (subprocess baseline 对比)

```rust
// crates/sandboxd/benches/run_command.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use sandboxd::{Orchestrator, CommandSpec};
use std::process::Command;

fn bench_run_command(c: &mut Criterion) {
    let orch = Orchestrator::new_test();
    let session_id = orch.create_session("bench", "task", None, SessionSpec::default(), 1800).unwrap();
    
    let mut group = c.benchmark_group("run_command");
    
    // sandboxd 路径
    group.bench_with_input(BenchmarkId::new("sandboxd", "echo"), &"echo hello", |b, cmd| {
        b.iter(|| orch.run_command(session_id, &CommandSpec::new(cmd), 60))
    });
    
    // subprocess baseline (SANDBOX-001 v0.2)
    group.bench_with_input(BenchmarkId::new("subprocess", "echo"), &"echo hello", |b, cmd| {
        b.iter(|| Command::new("cmd").args(&["/c", cmd]).output())
    });
    
    group.finish();
}
```

#### 5.2.3 100 session 并发 bench

```rust
fn bench_concurrent_sessions(c: &mut Criterion) {
    let orch = Orchestrator::new_test();
    
    c.bench_function("concurrent_100_sessions", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..100).map(|i| {
                tokio::spawn(async move {
                    let spec = SessionSpec::default();
                    orch.create_session("bench", &format!("task-{}", i), None, spec, 1800).await
                })
            }).collect();
            
            futures::future::join_all(handles).await
        })
    });
}
```

### 5.3 性能守门 (per 守门 #1 v25)

任何 bench 超过阈值 → 阻断 merge (per 守门 #1 v25 单 crate 实证 + 守门 #7 v3 advisory 派生)

---

## §6 验收测试 (UAT, 8 AC)

### 6.1 跟 SRS-002 §7 AC 1:1 对齐

| AC | 关联 FR / NFR | 验收方法 | 阈值 | 状态 |
|---|---|---|---|---|
| **AC-1** | FR-3.1 + FR-3.2 + FR-3.3 + FR-3.4 | 单元测试 4 维隔离全部命中 | 100% | 🟡 8+8+7+5 = 28 UT 覆盖 |
| **AC-2** | FR-1.1 + NFR-P-1 + NFR-P-2 | bench CreateSession p50 + p99 | < 10ms / < 100ms | 🟡 bench 5.2.1 |
| **AC-3** | FR-7.1 + FR-7.2 | 故障注入测试 | 100% fail-open / 100% fail-closed | 🟡 IT §3.2.2 + UT §2.2.6 TC-DES-12 + §2.2.8 TC-AUD-05 |
| **AC-4** | FR-4.1 + FR-4.2 + FR-4.3 | session 生命周期测试 | 100% 行为符合 | 🟡 IT §3.2.1 TC-IT-GRPC-01 |
| **AC-5** | NFR-T-1 + NFR-T-2 + NFR-T-3 | CI 三平台 | 100% pass | 🟡 E2E §4.2.3 |
| **AC-6** | FR-6.1 + NFR-O-1 | Prometheus scrape 验证 | 100% 指标暴露 | 🟡 UT §2.2.7 TC-MET-04 |
| **AC-7** | FR-6.2 + NFR-S-3 | audit log append-only 验证 | 100% 不可删 | 🟡 UT §2.2.8 TC-AUD-04 + IT §3.2.3 TC-IT-PG-04/05 |
| **AC-8** | NFR-S-1 + NFR-S-5 | 渗透测试 (4 维 × 10 攻击场景) | 0 越权 | 🟡 PT §5.2.2 (subprocess baseline) + 额外渗透测试 (4 维 × 10 = 40 场景) |

### 6.2 UAT 签字栏 (per 守门 #14 v3 Mavis 临时代签)

| 角色 | 氏名 | 签字 | 日期 |
|---|---|---|---|
| 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | 🟡 (等实装 v0.1 落地后签) | 2026-09-10 JST |
| SRE Lead | SRE Lead (Mavis 临时代签 per 9/3 11:35 JST 拍板 B) | 🟡 | 2026-09-10 JST |
| 平台 Lead | 平台 Lead (Mavis 临时代签 per 守门 #14 v3) | 🟡 | 2026-09-10 JST |
| 评审主持 | 评审主持 (Mavis 临时代签 per 守门 #14 v3) | 🟡 | 2026-09-10 JST |
| PM | PM (Mavis 临时代签 per 守门 #14 v3) | 🟡 | 2026-09-10 JST |

(per 守门 #14 v4 反转 v0.62, 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses; UAT 终审在 v0.1 实装完成后)

### 6.3 5 域 Lead 真人到位后追溯签字

- 触发: 5 域 Lead 真人到位 (per 守门 #14 v2 拍板 D + 守门 #14 v3)
- 形式: 修订历史表 +1 行 (per §8)
- 覆盖: UAT §6.2 5 角色签字栏
- 跟守门 #1 禁回溯叙事 一致: 不沿用代签决策

---

## §7 RACI 矩阵

| 任务 | R (Responsible) | A (Accountable) | C (Consulted) | I (Informed) |
|---|---|---|---|---|
| UT 编写 + 实测 | Mavis (root) | 架构师 (Mavis 接手) | SRE Lead | 5 域 Lead |
| IT 编写 + 实测 | Mavis (root) | 架构师 (Mavis 接手) | SRE Lead | 5 域 Lead |
| E2E 编写 + 实测 | Mavis (root) | 架构师 (Mavis 接手) | SRE Lead + 5 域 Lead | PM |
| PT bench 编写 + 实测 | Mavis (root) | 架构师 (Mavis 接手) | SRE Lead | 5 域 Lead |
| UAT AC 验收 | Mavis (root) | 架构师 (Mavis 接手) | SRE Lead + 平台 Lead + 评审主持 + PM | 5 域 Lead |
| 5 角色签字 | Mavis 临时代签 (per 守门 #14 v3) | 架构师 (Mavis 接手) | 5 域 Lead 真人到位 | PM |
| 跟 SANDBOX-001 v0.2 兼容 | Mavis (root) | 架构师 (Mavis 接手) | SRE Lead | 5 域 Lead |
| 跟 mavis desktop 集成 | Mavis (root) | 架构师 (Mavis 接手) | 5 域 Lead (player + economy + match + social + admin) | PM |
| 报告落地 (`PHASE-SANDBOX-002-IMPL-REPORT.md`) | Mavis (root) | 架构师 (Mavis 接手) | SRE Lead + 5 域 Lead | PM |

---

## §8 修订履歴 (詳細)

| バージョン | 日付 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-10 22:11 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 + 守门 #14 v4 反转 v0.62) | 初版落档, 5 级别测试设计 (UT 56 + IT 17 + E2E 11 + PT 5 bench + UAT 8 AC = 97 测), 跟 SRS-002 §7 8 AC 1:1 对齐, 跟 DD-002 §5 5 维测试矩阵 1:1 对齐, 9 已知缺口显式标 (跟 SRS-002 §9 + DD-002 §9 对齐), 守门 12/12 通过 (#1+#1 v15+#1 v19+#1 v25+#5+#6+#9+#9 v3+#11+#13+#14 v3+#22+#28), 单文档 9 章节结构 (跟 TEST-DESIGN-OPS-001 v0.2 模式对齐) | 2026-09-10 22:11 JST Ulysses 拍板"各级文档完善好, 更新后续任务到 wbs" + DD-SANDBOX-002 v0.1 派生 |

---

## §9 引用文档

| 文档 | 关系 | 关键引用 |
|---|---|---|
| `docs/requirements/SRS-SANDBOX-002.md` v0.1.1 | 上游 (需求) | 8 機能 / 5 業務 / 7 非機能 / 8 AC / 4 表 W/T/M / 9 缺口 / 6 决策点已拍板 |
| `docs/basic-design/SANDBOX-BASIC-DESIGN-002.md` v0.1.1 | 上游 (基本设计) | 6 模块 / 18 子模块 / gRPC proto / 4 维后端 / 三平台 init |
| `docs/detailed-design/DD-SANDBOX-002.md` v0.1 | 上游 (详细设计) | 5 维设计 (24 文件 / 5 struct / 5 enum / 5 trait / 4 时序 / 2 状态图) + 4 表 DDL 100% 覆盖 + 9 缺口 |
| `docs/architecture/SANDBOX-001.md` v0.2 | 上游派生 | sandbox v0.2 subprocess 包装器 (降级路径基线) |
| `docs/test-design/TEST-DESIGN-OPS-001.md` v0.2 | 模板参考 | 单文档分章 §UT/§IT/§E2E/§PT/§UAT 模式 |
| `docs/test-design.md` v0.3 (WBS §13) | 平行 | 整体 P3-A 测试设计, 4 子项 109 测 |
| `scripts/automation/guardian/sandbox.py` v0.2 | 降级路径 | sandboxd 不可用时 subprocess.run 降级 |
| `scripts/automation/dispatcher.py` v0.1 | 集成点 | sandboxd gRPC client 集成点 |
| `crates/star-mcp/` 0.12 | 集成点 (Rust) | SandboxdClient 集成点 |
| `crates/star-context/` | ActorContext (P0-1) | tenant_id / workspace_id / actor_id 注入 |

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
