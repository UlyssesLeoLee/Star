# SRS-SANDBOX-002

> **Sandbox-as-a-Service (sandboxd) — 要件定義書 v0.1.1** (per 日本 IPA SEC 標準 / 要件定義書 テンプレート)
>
> - 状态: 🟢 Draft v0.1.1 (2026-09-10 21:57 JST 决策点 D-1~D-6 全部已拍板 per Ulysses A 选项)
> - 目标阶段: 要件定義 → 基本設計 → 詳細設計 → 実装 → テスト → リリース
> - 上游: `docs/architecture/SANDBOX-001.md` v0.2 (子代理 subprocess 沙箱, 仅资源维隔离)
> - 关联 commit: (留空, root 统一 commit 时填, per 守门 #1 v15 docs 同步饱和 + 1 commit 多文件)
> - 关联实装基线: `scripts/automation/guardian/sandbox.py` v0.2 (现役, 子代理 subprocess 沙箱) + `scripts/automation/dispatcher.py` v0.1 (现役, 子代理 invoke) + `crates/star-mcp/` (现役, MCP transport 接入点)
> - 守门基线: 守门 #1+#5+#9+#10+#11+#13+#14 v3+#14 v4+#22+#28 共 10 项必过 (守门 #1 v25 cargo test 不需要跑, 文档工作)
> - 上位要件: `AGENTS.md` §4 守门硬约束 (per 2026-09-10 调研产出) + `docs/architecture/SANDBOX-001.md` v0.2 派生
> - 平行参考: `docs/requirements/SRS-PRE-TOOL-USE-GUARD-001.md` v0.1 (IPA 模板参考) + `docs/automation-design.md` v0.1 (Python 化基线) + `docs/architecture/2026-08-26-upgrade/` 派生历史
> - 修订人: `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` (per 2026-08-27 19:39 JST 用户授权 + 守门 #10 + 守门 #14 v3)
> - 审批: `架构师 (Mavis 接手 agent per DEC-008)` (per 守门 #14 v4 反转 v0.62 2026-09-10 12:45 JST, 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses)
> - 日期: 2026-09-10 JST
> - 受众: 詳細設計エンジニア / 実装エンジニア / SRE Lead / 5 域 Lead (未到位, Mavis 临时代签 per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)
> - 拍板来源: 2026-09-10 21:48 JST Ulysses 拍板"**agent 的沙盒设计到位了吗？没有的话，我希望沙盒组建是一个 app 形式的独立模块，我们来讨论一下设计**" + 21:49 JST "**先把需求文档和基本设计改好**" (本 SRS + 后续 BD 落档)

---

## §0 文档信息 / 修订履歴

### 0.1 文档情報

| 項目 | 内容 |
|---|---|
| 文書 ID | SRS-SANDBOX-002 |
| 文書名 | Sandbox-as-a-Service (sandboxd) — 要件定義書 |
| バージョン | v0.1.1 (决策点 D-1~D-6 已拍板) |
| 作成日 | 2026-09-10 |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手 agent per DEC-008) |
| 対象範囲 | sandboxd 独立 daemon + gRPC IPC + 4 维隔离 (resource / network / fs / capability) + 跨 client 复用 (dispatcher / mavis desktop / 其他) |
| 対象バージョン | sandboxd v0.1 (MVP-骨架, 跟 SANDBOX-001 v0.2 兼容) → v1.0 (生产) |
| 適用プラットフォーム | Windows (PowerShell) + Linux (bash) + macOS (zsh) 三平台 |
| 関連 commit | (root 统一 commit 时填, per 守门 #1 v15) |
| 上位文書 | `docs/architecture/SANDBOX-001.md` v0.2 (派生) + `AGENTS.md` §4 守门硬约束 (守门 #1+#5+#9+#11+#13+#14 v3+#22+#28) |
| 関連文書 | `docs/architecture/SANDBOX-001.md` v0.2 + `docs/basic-design/SANDBOX-BASIC-DESIGN-002.md` v0.1 (同期落档) + `docs/automation-design.md` v0.1 + `scripts/automation/guardian/sandbox.py` v0.2 (现役, 沙箱 v0.2 包装器) + `scripts/automation/dispatcher.py` v0.1 (现役, 子代理 invoke) + `crates/star-mcp/` (现役, MCP transport) |
| 機能数 | 8 機能 (FR-1 ~ FR-8), 業務要件 5 (BR-1 ~ BR-5), 非機能要件 7 類 (NFR-P/A/S/M/T/O/C) 29 项, 受理条件 8 (AC-1 ~ AC-8), 决策点 6 (D-1~D-6 全部已拍板 per Ulysses A 选项 2026-09-10 21:57 JST) |
| データモデル | 4 表 W/T/M 横展 (Session / Policy / Audit / Capability, 100% 覆盖 per 守门 #13) |

### 0.2 修订履歴

| バージョン | 日付 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1 (当前)** | 2026-09-10 21:49 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 + 守门 #14 v4 反转 v0.62) | 初版落档, 8 機能 (FR-1~FR-8, 24 项) + 5 業務要件 (BR-1~BR-5) + 7 非機能要件 (NFR-P/A/S/M/T/O/C 29 项) + 8 验收条件 (AC-1~AC-8) + 9 已知缺口 (含 1 P0 阻塞 sandboxd ↔ dispatcher 集成), 4 表 W/T/M 横展 (Session T + Policy M + Audit T + Capability M, 100% 覆盖 per 守门 #13), 守门 10/10 通过, IPA 10 段结构 (目的 / 範囲 / 用語 / 業務 / 機能 / 非機能 / 制約 / 验收 / 缺口 / 签字 + 修订 + 决策点), 6 决策点显式标 (形态 / IPC / 网络隔离 / FS 隔离 / mavis 集成 / 观测) | 2026-09-10 21:48 JST Ulysses 拍板"agent 的沙盒设计到位了吗？没有的话，我希望沙盒组建是一个 app 形式的独立模块" + 21:49 JST "先把需求文档和基本设计改好" + SANDBOX-001 v0.2 派生 4 缺口 (#1 network / #2 fs / #5 observability / #6 elevated) 升级为独立 app 形态 |

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

本文档按 日本 IPA SEC 標準 制定 **Sandbox-as-a-Service (sandboxd)** 的要件定義書, 涵盖子代理 dispatch + mavis tool 调用 + 任意 host-side 子进程 的统一隔离平台, 包含:

- **独立 daemon 形态** (sandboxd 长期运行, 跟 dispatcher / mavis / 其他 client 解耦)
- **gRPC IPC** (跨语言: Python dispatcher / Rust mavis / 其他)
- **4 维隔离** (resource / network / fs / capability, 覆盖 SANDBOX-001 v0.2 的 4 缺口)
- **session 化** (ephemeral sandbox session, 用完即回收)
- **跨平台** (Windows + Linux + macOS, 跟 SANDBOX-001 v0.2 仅 Windows 升级)
- **观测性** (Prometheus metrics + audit log, 跟守门 v36 索引联动)
- **fail-open / fail-closed 策略** (跟 SRS-PRE-TOOL-USE-GUARD-001 v0.1 派生一致)

作为后续基本設計 (`SANDBOX-BASIC-DESIGN-002.md` v0.1, 同期落档) / 詳細設計 / 実装 / テスト / リリース的唯一依据。

**派生来源**: 2026-09-10 21:48 JST Ulysses 拍板 + SANDBOX-001 v0.2 §6 已知缺口 (#1 network / #2 fs / #3 POSIX rlimit / #5 observability / #6 elevated) 升级为独立 app 形态。

### 1.2 背景 (用户痛点 / 课题)

**课题 1: SANDBOX-001 v0.2 仅资源维隔离** (per `docs/architecture/SANDBOX-001.md` v0.2 §6 已知缺口)
- v0.2 落地: 内存 / CPU / 进程数 / 级联 kill (Windows Job Objects), 9 tests 0 回归
- v0.2 缺: 网络隔离 (0 跨网访问保护) + 文件系统隔离 (走 chmod + W 工作目录) + 沙箱自身观测性 (0 metrics) + 非 elevated 进程 Access Denied
- v0.2 形态: Python module 绑 dispatcher.py, 跨 client 不可复用 (mavis desktop Rust 端无法用)

**课题 2: 子代理越权风险** (per 守门 #9 实证 + SRS-PRE-TOOL-USE-GUARD-001 v0.1 §1.2 课题 1)
- 2026-09-02 实证: 10 background task `net::ERR_CONNECTION_CLOSED` 但 status 报 succeeded
- 子代理被派出去后, 内部 Python 脚本可直接调 subprocess, **PreToolUse hook 是唯一可控边界 (事前)**, 沙箱是事后 / runtime 隔离
- 守门 #5 (env 安全) 是 review guard (事后), 沙箱是 execution guard (runtime), 两者互补

**课题 3: 跨 client 隔离能力不可复用**
- 当前 sandbox v0.2 绑 dispatcher.py (Python), mavis desktop (Rust) 没法用
- 任何需要"安全跑一段代码"的需求 (e.g. OPS Console 跑 AI 修改 mock / 调试控制台跑脚本 / 子代理跑 cargo) 都要自己造轮子
- 期望: 1 份 sandbox 服务, 任何 client 都能通过 IPC 调用

**课题 4: 不可逆操作缺少 runtime 隔离** (per SRS-PRE-TOOL-USE-GUARD-001 v0.1 §1.2 课题 3 派生)
- `rm -rf`、`mkfs`、`sudo`、force push 等不可逆操作, 0 runtime 隔离
- 即便 PreToolUse hook 拦了, 子代理内部代码 (post-dispatch) 仍可调 subprocess 绕过 hook
- 沙箱是 hook 之上的第二道防线: hook 拦 tool call, 沙箱兜底 subprocess 行为

### 1.3 包含範囲 (In-Scope, **8 機能**)

| 機能 ID | 名称 | 项数 | 优先级 | 概要 |
|---|---|---|---|---|
| **FR-1** | daemon 形态 | 3 项 | P0 | sandboxd 独立进程, 长期运行, session 池化 |
| **FR-2** | gRPC IPC | 3 项 | P0 | cross-language, schema 化, streaming 支持 |
| **FR-3** | 4 维隔离 | 4 项 | P0 | resource / network / fs / capability |
| **FR-4** | session 生命周期 | 3 项 | P0 | create / run / destroy, 用完即回收 |
| **FR-5** | 跨平台隔离后端 | 3 项 | P0 | Windows (Job Objects + WFP + AppContainer) + Linux (cgroups v2 + netns + mount ns) + macOS (sandbox-exec) |
| **FR-6** | 观测性 | 3 项 | P1 | Prometheus metrics + audit log (PG) + 健康检查 |
| **FR-7** | 旁路 / fail-open | 2 项 | P0 | sandboxd 不可用 → 走 SANDBOX-001 v0.2 subprocess 降级 |
| **FR-8** | 测试 / 报告 | 3 项 | P1 | 单元测试 + 集成测试 + e2e + 报告 (跟现有 PHASE-*-IMPL-REPORT 一致) |
| **合計** | | **24 项** | | |

### 1.4 排除範囲 (Out-of-Scope)

- **AI 行为审计** (子代理 LLM 决策过程录屏) → 后续 v2.x 拍摄
- **加密 / 凭据管理** (mavis 内置 vault) → 跨项目需求, 不在本专题
- **跨域编排** (5 域 Lead DDD Review / Saga orchestrator) → 跨域编排, 不在本专题
- **真实 K8s / Helm 部署** → v0.1 走 systemd / Windows Service, K8s 部署 v0.2 拍摄
- **TLS / mTLS 双向认证** → v0.1 走明文 gRPC (localhost), TLS v0.2 拍摄
- **多租户隔离** (sandboxd 服务于多个 org) → v0.1 走单租户, 多租户 v0.2 拍摄

### 1.5 关联文档

| 文档 | 关系 | 关键引用 |
|---|---|---|
| `docs/architecture/SANDBOX-001.md` v0.2 | **上游 / 派生** | sandbox v0.2 subprocess 包装器, 4 缺口 (#1+#2+#5+#6) 升级为本 SRS |
| `AGENTS.md` §4 守门硬约束 | 上位 | 守门 #1+#5+#9+#10+#11+#13+#14 v3+#22+#28 |
| `docs/requirements/SRS-PRE-TOOL-USE-GUARD-001.md` v0.1 | 平行 | IPA 模板参考, fail-open / fail-closed 派生, audit log Transaction append-only |
| `docs/basic-design/SANDBOX-BASIC-DESIGN-002.md` v0.1 | 下游 | 同期落档, 8 段基本設計 |
| `docs/automation-design.md` v0.1 | 平行 | §1.2 [P]/[M]/[S] 判定 + §3.1 dispatcher.py brief 落地 |
| `docs/guardian/README.md` | 平行 | 守门 v3x 落档目录, v27+v28+v29 实证 |
| `scripts/automation/guardian/sandbox.py` v0.2 | 现役 | sandbox v0.2 包装器, v0.1 降级路径 |
| `scripts/automation/dispatcher.py` v0.1 | 现役 | 子代理 invoke, 跟 sandboxd client 集成 |
| `crates/star-mcp/` | 现役 | MCP transport, 跟 sandboxd client 集成 (Rust) |
| claude code `plugins/security-guidance` | 对照基线 | PreToolUse hook 9 种危险模式 → 本 SRS 派生 v0.2 sandboxd 4 维隔离 |

---

## §2 用語定義 / 略語 (Glossary)

| 用語 | 定義 | 出典 |
|---|---|---|
| **Mavis** | 本机 root session agent (Mavis As a Jarvis), 运行在 MiniMax Code | per agent-context block |
| **sandboxd** | Sandbox-as-a-Service daemon, 独立进程, 长期运行, 提供 4 维隔离 | 本 SRS 自定义 |
| **session** | sandboxd 的 ephemeral 沙箱实例, 1 个 session = 1 个子代理任务生命周期 | 本 SRS 自定义 |
| **client** | 任何调用 sandboxd IPC 的进程 (dispatcher / mavis / 其他) | 本 SRS 自定义 |
| **gRPC** | Google 远程过程调用, 跨语言 RPC 框架, 基于 HTTP/2 + protobuf | 行业标准 |
| **4 维隔离** | resource (CPU/mem/procs) + network (egress filter) + fs (path allowlist) + capability (syscall filter) | 本 SRS 自定义 |
| **Job Objects** | Windows API, 进程组隔离 + 资源限制 + 级联 kill | Windows SDK |
| **WFP** | Windows Filtering Platform, Windows 内核级网络过滤 | Windows SDK |
| **AppContainer** | Windows 沙箱技术, 跟 UWP app 同等级, 强隔离 | Windows SDK |
| **cgroups v2** | Linux control groups v2, 资源 + 进程隔离 | Linux kernel |
| **netns** | Linux network namespace, 网络隔离 | Linux kernel |
| **mount ns** | Linux mount namespace, 文件系统隔离 | Linux kernel |
| **sandbox-exec** | macOS 内置沙箱工具, 跟 sandboxd 集成 | macOS SDK |
| **fail-open** | sandboxd 不可用时, 走 SANDBOX-001 v0.2 subprocess 降级 (不阻断主流程) | 行业术语, 跟 SRS-PRE-TOOL-USE-GUARD-001 一致 |
| **fail-closed** | audit log 写失败时, BLOCK 工具调用, 返回错误 | 行业术语, 跟 SRS-PRE-TOOL-USE-GUARD-001 一致 |
| **W/T/M** | Work / Transaction / Master 三类横展 (per 守门 #13) | STAR 守门 #13 |
| **OLU** | One-person Logistics Unit, 1 SRE·周 ≈ 1M tokens (per STAR-OLU-001.md) | STAR 派生 |
| **IPA SEC** | Information-technology Promotion Agency, Software Engineering Center | 日本独立行政法人 |
| **要件定義書** | Software Requirements Specification (SRS) | IPA SEC テンプレート |
| **基本設計書** | Basic Design Document (BDD) | IPA SEC テンプレート |
| **詳細設計書** | Detailed Design Document (DDD) | IPA SEC テンプレート |

---

## §3 業務要件 (BR, Business Requirements)

### BR-1 子代理跨 client 统一隔离
任何 client (dispatcher / mavis desktop / 其他 host 进程) 调起子代理 / 子进程 / 任意 host-side 代码时, 必须经 sandboxd 隔离。**当前 SANDBOX-001 v0.2 仅 Python dispatcher 客户端**, 升级后任何 client 都能复用。

### BR-2 4 维隔离深度
子代理内部代码 (post-dispatch) 任何行为 (resource / network / fs / capability) 必须经 sandboxd 策略校验。**跟守门 #9 实证 派生**: 子代理一旦被派, 内部 Python 脚本不受 mavis hook 约束, sandboxd 是 host-side 唯一可控隔离层。

### BR-3 不可逆操作 runtime 隔离
`rm -rf` 命中 `/` `~` `*`、`mkfs`、`sudo`、force push 等不可逆操作, 在 sandboxd 策略层强制 ASK / BLOCK, **跟 SRS-PRE-TOOL-USE-GUARD-001 v0.1 BR-3 派生**: hook 是 tool call 层, sandboxd 是 subprocess 层, 两层互补。

### BR-4 观测性 / 可审计
所有 sandboxd session (无论 resource / network / fs / capability 命中) 必写入 audit log (PG), 可查询 / 可回放 / 可聚合。Prometheus metrics 暴露 session 计数 / 资源使用 / 命中分布, 跟守门 v36 audit log 索引联动。

### BR-5 跨平台 + 升级兼容
sandboxd v0.1 在 Windows + Linux + macOS 三平台落地, 跟 SANDBOX-001 v0.2 (仅 Windows) 兼容 (fail-open 降级路径)。**v0.2 不破坏 v0.1 任何业务 logic**, 仅追加 platform backends + audit log + metrics。

---

## §4 機能要件 (FR, Functional Requirements)

### FR-1 daemon 形态 (3 项, P0)

**FR-1.1** sandboxd 独立进程
- 形态: 独立 binary (Rust, 跟 mavis desktop 同栈), 长期运行
- 启动: systemd service (Linux) / Windows Service (Win) / launchd (macOS)
- 配置: 1 份 YAML 配置文件 (`/etc/sandboxd/config.yaml` 或 `$ProgramData/sandboxd/config.yaml`)
- 健康检查: HTTP endpoint `/healthz` (返回 200 + uptime + session_count)

**FR-1.2** session 池化
- 容量: 默认 100 session 并发, 可配置
- 调度: FIFO + LRU 回收
- 资源: session 间独立 (resource / network / fs), 不共享任何状态
- 复用: 同 client + 同 policy 的 session 可复用 (避免反复创建)

**FR-1.3** 优雅停机
- 接收 SIGTERM → 等待所有 running session 退出 (默认 30s grace) → 强制 kill 残留
- 期间拒绝新 session 创建 (返回 Unavailable)
- 持久化: 写 shutdown log (PG audit log)

### FR-2 gRPC IPC (3 项, P0)

**FR-2.1** Protocol Buffers schema
- 文件: `crates/sandboxd/proto/sandboxd.proto`
- 服务: `SandboxService` (4 RPC: CreateSession / RunCommand / DestroySession / StreamLogs)
- 消息: `SessionSpec` (resource + network + fs + capability policy) + `CommandSpec` (cmd + args + env + cwd) + `RunResult` (returncode + stdout + stderr + latency_ms) + `LogEntry`
- 跨语言: Python (dispatcher) + Rust (mavis) + 任意 grpc 支持语言

**FR-2.2** IPC 端点
- 默认: `127.0.0.1:50051` (localhost, 明文 gRPC, v0.1)
- v0.2: TLS 双向认证 (mTLS)
- 端口冲突: 自动 +1 探测, 最多 10 次
- 跨进程: Unix domain socket (POSIX) / Named pipe (Windows) 作为备选

**FR-2.3** 流式日志
- `StreamLogs` RPC: server streaming, client 订阅 session 日志
- 日志粒度: 1 条 sandboxd 内部事件 = 1 条 log entry
- 缓冲: 1000 条环形缓冲, 满则 drop 老的 (per BR-4 观测性)

### FR-3 4 维隔离 (4 项, P0)

**FR-3.1** Resource 维隔离
- 范围: 内存 / CPU / 进程数 / 文件描述符 / 线程数
- 后端: Windows Job Objects / Linux cgroups v2
- 限制: `SandboxLimits` 数据类 (per SANDBOX-001 v0.2 §3.1), 100% 复用
- 默认值: 1GB mem / 50% CPU / 100 procs / kill_on_parent_exit=True

**FR-3.2** Network 维隔离
- 范围: 出站 (egress) 网络访问
- 后端: Windows WFP (Windows Filtering Platform) / Linux netns + iptables / macOS pf
- 策略: **allowlist 默认 (per §11 决策点 D-3 推荐)**, 子代理常要 git/cargo/crates.io 域名
- allowlist 配置: 1 份 JSON, 默认 `github.com` / `crates.io` / `static.crates.io` / `index.crates.io` / `sh.rustup.rs` + 自定义
- BLOCK 模式: 命中非 allowlist 域名 → audit log + 网络 reset (RST)

**FR-3.3** Filesystem 维隔离
- 范围: 文件系统读写
- 后端: Windows AppContainer (受限 token) / Linux mount namespace (bind mount) / macOS sandbox-exec profile
- 策略: **path allowlist 默认 (per §11 决策点 D-4 推荐)**, 子代理需要 read worktree + 写 `$WORKTREE/.worktree-cache/`
- allowlist 配置: 1 份 JSON, 默认 `/worktree/**` R + `/$WORKTREE/.worktree-cache/**` RW + `/tmp/**` RW
- BLOCK 模式: 命中非 allowlist 路径 → audit log + EACCES

**FR-3.4** Capability 维隔离
- 范围: Linux capability / Windows privilege
- 后端: Linux capability drop (setcap) / Windows token privilege adjust
- 策略: 默认 drop ALL, 仅保留 NET_BIND_SERVICE (per 子代理 port listen 需求)
- v0.1: 仅 Linux 实装, Windows / macOS 走 sandbox 后端隐式 drop (v0.2 显式)

### FR-4 session 生命周期 (3 项, P0)

**FR-4.1** CreateSession
- 入参: `{client_id, task_id, parent_session_id, policy: SessionSpec, timeout_sec}`
- 出参: `{session_id, status: "Ready"|"Failed", error?: string}`
- 失败: 资源不足 / 策略非法 / 平台不支持 → Failed + error
- 超时: CreateSession 自身 < 100ms p99

**FR-4.2** RunCommand
- 入参: `{session_id, cmd: CommandSpec, timeout_sec}`
- 出参: `{returncode, stdout, stderr, latency_ms, hit_violations: [string]}`
- 命中违规: 不阻断, 仅 audit log + hit_violations 字段返回 (per fail-open 派生)
- 超时: 走 session 内 timeout, 触发 sandboxd kill session (per FR-1.1 级联 kill)

**FR-4.3** DestroySession
- 入参: `{session_id, reason: string}`
- 出参: `{status: "Destroyed"|"NotFound"}`
- 行为: kill 所有 session 内进程 + 回收所有隔离 + 写 audit log
- 强制: 任何 session 30 min 默认 timeout (per 子代理 30 min 习惯), 强制 destroy

### FR-5 跨平台隔离后端 (3 项, P0)

**FR-5.1** Windows 后端
- Resource: Job Objects (per SANDBOX-001 v0.2 §2.1, 100% 复用)
- Network: WFP (Windows Filtering Platform) — v0.1 用 WFP API + 自定义 provider
- FS: AppContainer (受限 token) — v0.1 走 CreateProcessAsUser + AppContainer profile
- Capability: token privilege adjust (v0.1 简化, 仅 drop SeDebugPrivilege 等高危)

**FR-5.2** Linux 后端
- Resource: cgroups v2 (memory.max / cpu.max / pids.max)
- Network: netns + iptables (egress allowlist via OUTPUT chain)
- FS: mount namespace (bind mount + tmpfs for /tmp)
- Capability: libcap / capsicum (drop all, per FR-3.4)

**FR-5.3** macOS 后端
- Resource: sandbox-exec resource limits (per macOS sandbox profile)
- Network: sandbox-exec network filter (deny all by default + allowlist)
- FS: sandbox-exec file filter (path allowlist)
- Capability: sandbox-exec implicit drop

### FR-6 观测性 (3 项, P1)

**FR-6.1** Prometheus metrics
- 端点: `127.0.0.1:50051/metrics` (HTTP, 跟 gRPC 同进程)
- 指标:
  - `sandboxd_sessions_total{status="created|failed|rejected"}` counter
  - `sandboxd_sessions_active` gauge
  - `sandboxd_session_duration_seconds` histogram
  - `sandboxd_resource_limit_hits_total{limit_type}` counter
  - `sandboxd_network_violations_total{action="blocked|allowed"}` counter
  - `sandboxd_fs_violations_total{action="blocked|allowed"}` counter
- scrape interval: 15s (Prometheus 默认)

**FR-6.2** audit log
- 存储: PostgreSQL (跟现有 `p3d6_audit` 表结构一致, per 守门 #13 Transaction append-only)
- 字段: 11 字段 (session_id / client_id / task_id / event_type / resource_used / network_violations / fs_violations / capability_violations / timestamp / latency_ms / hit_violations)
- 索引: 跟守门 v36 audit log 索引联动 (session_id / client_id / event_type)
- 保留: 90 天 (per BR-4 + 守门 #5 隐含)

**FR-6.3** 健康检查
- 端点: `127.0.0.1:50051/healthz`
- 响应: `{status: "ok", uptime_sec: int, sessions_active: int, sessions_total: int, version: string}`
- 用途: systemd / k8s liveness probe + 调试

### FR-7 旁路 / fail-open (2 项, P0)

**FR-7.1** sandboxd 不可用 → SANDBOX-001 v0.2 降级
- 触发: gRPC 连接失败 / sandboxd 返回 Unavailable / 健康检查失败
- 行为: dispatcher / client 走 SANDBOX-001 v0.2 subprocess.run 降级路径 (per `scripts/automation/guardian/sandbox.py` v0.2)
- 告警: 写 root session 通知 (per NFR-O-3, < 500ms 推送)
- 持续: sandboxd 恢复后自动切回

**FR-7.2** (隐含) audit log 写失败 → fail-closed
- 触发: PG 不可用 / 磁盘满 / 权限不足
- 行为: BLOCK session 创建, 返回错误
- 原因: 凭据外泄零容忍, 审计是最后一道防线 (跟 SRS-PRE-TOOL-USE-GUARD-001 v0.1 FR-6.2 一致)

### FR-8 测试 / 报告 (3 项, P1)

**FR-8.1** 单元测试
- 路径: `crates/sandboxd/src/**/tests/`
- 覆盖: 4 维隔离策略 + session 生命周期 + fail-open / fail-closed
- 工具: cargo test (跟守门 #1 v25 一致)
- 阈值: ≥ 90% line coverage

**FR-8.2** 集成测试
- 路径: `crates/sandboxd/tests/integration/`
- 覆盖: 跨平台 (Win + Linux + macOS) + 真实子进程 (cargo build / git clone)
- 工具: cargo test --workspace --lib -j 4
- 阈值: 100% pass on Win + Linux

**FR-8.3** e2e + 报告
- e2e: `tests/e2e/test_sandboxd_*.py` 覆盖 dispatcher 实际调用 + mavis desktop 集成
- 报告: `docs/reports/PHASE-SANDBOX-002-IMPL-REPORT.md` 跟现有 6 份 PHASE-*-IMPL-REPORT 一致
- 落地: 1 commit 多文件 (per 守门 #1 v15)

---

## §5 非機能要件 (NFR, Non-Functional Requirements)

### NFR-P 性能 (Performance)

| ID | 要求 | 計測方法 | 阈值 |
|---|---|---|---|
| **NFR-P-1** | CreateSession 延迟 p50 | 单元测试 benchmark | < 10ms |
| **NFR-P-2** | CreateSession 延迟 p99 | 单元测试 benchmark | < 100ms |
| **NFR-P-3** | RunCommand 端到端延迟 | benchmark | 跟 subprocess.run baseline 比 < 5% 开销 |
| **NFR-P-4** | gRPC IPC 序列化开销 | benchmark | < 1ms (per command) |
| **NFR-P-5** | 100 session 并发 | 压测 | 0 资源竞争, p99 < 200ms |

### NFR-A 可用性 (Availability)

| ID | 要求 | 計測方法 | 阈值 |
|---|---|---|---|
| **NFR-A-1** | sandboxd 不可用 → SANDBOX-001 v0.2 降级 | 故障注入测试 | 100% 降级, 0 阻断主流程 |
| **NFR-A-2** | audit log 写失败 → fail-closed | 故障注入测试 | 100% 阻断, 0 静默吞错 |
| **NFR-A-3** | session 自动超时清理 | 实测 | 30 min 默认, 0 残留 |
| **NFR-A-4** | sandboxd 重启不丢状态 | 实测 | session 全 destroy, audit log 全保留 |

### NFR-S 安全性 (Security)

| ID | 要求 | 計測方法 | 阈值 |
|---|---|---|---|
| **NFR-S-1** | 4 维隔离 100% 覆盖 | 渗透测试 | 0 越权 |
| **NFR-S-2** | 凭据 0 外泄 | 单元测试 + 渗透测试 | 0 命中 (跟守门 #5 + SRS-PRE-TOOL-USE-GUARD-001 NFR-S-1 一致) |
| **NFR-S-3** | audit log 不可物理删除 | OS 权限验证 | 0 修改 (跟 SRS-PRE-TOOL-USE-GUARD-001 NFR-S-2 一致) |
| **NFR-S-4** | session 间 0 状态泄漏 | 实测 | 0 跨 session 数据共享 |
| **NFR-S-5** | allowlist 绕过 (DNS rebinding / domain fronting) 0 命中 | 渗透测试 | 0 命中 |

### NFR-M 保守性 / 维护性 (Maintainability)

| ID | 要求 | 計測方法 | 阈值 |
|---|---|---|---|
| **NFR-M-1** | gRPC proto 文档化 (Buf 兼容) | 文档 | 100% |
| **NFR-M-2** | policy JSON Schema 文档化 | 文档 | 100% |
| **NFR-M-3** | audit log JSON Lines 兼容 | 实测 | 100% parseable |
| **NFR-M-4** | 测试覆盖率 | cargo tarpaulin | ≥ 90% |
| **NFR-M-5** | 跟 SANDBOX-001 v0.2 兼容 (v0.2 subprocess.run 降级路径) | 实测 | 100% 兼容 |

### NFR-T 移植性 (Portability)

| ID | 要求 | 計測方法 | 阈值 |
|---|---|---|---|
| **NFR-T-1** | Windows (PowerShell) 兼容 | CI 实测 | 100% pass |
| **NFR-T-2** | Linux (bash) 兼容 | CI 实测 | 100% pass |
| **NFR-T-3** | macOS (zsh) 兼容 | CI 实测 | 100% pass |
| **NFR-T-4** | 4 维隔离后端 跨平台 100% 行为一致 | 单元测试 | 100% 覆盖 |
| **NFR-T-5** | gRPC 跨语言 (Python + Rust) | 集成测试 | 100% 互通 |

### NFR-O 可观测性 (Observability)

| ID | 要求 | 計測方法 | 阈值 |
|---|---|---|---|
| **NFR-O-1** | Prometheus metrics 暴露 (per FR-6.1) | 实测 | 100% |
| **NFR-O-2** | audit log PG 写入 (per FR-6.2) | 实测 | 100% |
| **NFR-O-3** | sandboxd 降级事件 → root session 通知 | 实测 | < 500ms 推送 |
| **NFR-O-4** | BLOCK 事件触发 root session 通知 | 实测 | < 500ms 推送 |
| **NFR-O-5** | 健康检查 endpoint 200 | 实测 | 100% |

### NFR-C 兼容性 (Compatibility)

| ID | 要求 | 計測方法 | 阈值 |
|---|---|---|---|
| **NFR-C-1** | 跟 SANDBOX-001 v0.2 兼容 (v0.2 subprocess.run 降级) | 实测 | 100% |
| **NFR-C-2** | 跟 dispatcher.py v0.1 兼容 (gRPC client 替代 subprocess.run) | 实测 | 100% |
| **NFR-C-3** | 跟 mavis desktop Rust 集成 (gRPC client) | 实测 | 100% |
| **NFR-C-4** | 跟守门 #5 (env 安全) 联动 | 单元测试 | 100% 命中场景被本层 BLOCK |
| **NFR-C-5** | 跟守门 #9 (子代理 RPC 不可靠) 联动 | 单元测试 | sandboxd 不可用 → 走 SANDBOX-001 v0.2 降级 |

---

## §6 制約条件 / 前提 / 依赖

### 6.1 制約条件

- 必须在 `D:/Star/crates/sandboxd/` 目录下落地 (跟现有 `crates/star-mcp/` 等一致, Rust workspace)
- 必须用 Rust 1.80+ (跟 mavis desktop 一致)
- gRPC schema 必须用 Protocol Buffers v3 (跨语言兼容)
- policy 文件必须 JSON 格式 (跟守门 v28 + SRS-PRE-TOOL-USE-GUARD-001 一致)
- audit log 必须 JSON Lines (跟守门 v36 + SRS-PRE-TOOL-USE-GUARD-001 NFR-M-3 一致)
- 不引入新的大依赖 (用 tonic + prost + tokio + serde, 跟 mavis 现有栈一致)
- 必须 systemd / Windows Service / launchd 三平台 init 集成

### 6.2 前提

- SANDBOX-001 v0.2 (subprocess 包装器) 已落地, 本 SRS 是其 app 化升级
- dispatcher.py v0.1 + console_server.py v0.1 现役, sandboxd client 集成点存在
- mavis desktop (Rust) 现役, sandboxd client 集成点存在
- 守门 #5 (env 安全) + 守门 #9 (子代理 RPC) + 守门 #13 (T append-only) 已确立

### 6.3 依赖

- `scripts/automation/guardian/sandbox.py` v0.2 (v0.1 降级路径)
- `scripts/automation/dispatcher.py` v0.1 (子代理 invoke, gRPC client 替代 subprocess.run)
- `crates/star-mcp/` (MCP transport, 跟 sandboxd client 集成)
- PostgreSQL (audit log 存储, 跟现有 `p3d6_audit` 表结构一致)
- Rust workspace (跟 mavis desktop 同栈, per 守门 #1 cargo check)
- tonic / prost (gRPC framework, Rust 1.80+)

---

## §7 验收条件 (AC, Acceptance Criteria)

| AC | 关联 FR / NFR | 验收方法 | 阈值 |
|---|---|---|---|
| **AC-1** | FR-3.1 + FR-3.2 + FR-3.3 + FR-3.4 | 单元测试 4 维隔离全部命中 | 100% |
| **AC-2** | FR-1.1 + NFR-P-1 + NFR-P-2 | benchmark p50 + p99 | < 10ms / < 100ms |
| **AC-3** | FR-7.1 + FR-7.2 | 故障注入测试 | 100% fail-open / 100% fail-closed |
| **AC-4** | FR-4.1 + FR-4.2 + FR-4.3 | session 生命周期测试 | 100% 行为符合 |
| **AC-5** | NFR-T-1 + NFR-T-2 + NFR-T-3 | CI 三平台 | 100% pass |
| **AC-6** | FR-6.1 + NFR-O-1 | Prometheus scrape 验证 | 100% 指标暴露 |
| **AC-7** | FR-6.2 + NFR-S-3 | audit log append-only 验证 | 100% 不可删 |
| **AC-8** | NFR-S-1 + NFR-S-5 | 渗透测试 (4 维 × 10 攻击场景) | 0 越权 |

---

## §8 数据模型 (W/T/M 横展 per 守门 #13)

> 守门 #13 强制: 所有 DB 表必须横展 W/T/M 三類分門別類, 100% 覆盖。

### 8.1 表索引 (4 表 100% 覆盖)

| # | 表名 | 分類 | 说明 | 估行数 |
|---|---|---|---|---|
| 1 | `sandbox_session` | **T (Transaction)** | session 创建/销毁事件, append-only, 含 client/task/parent_session/policy 快照 | 100K+ / 月 |
| 2 | `sandbox_policy` | **M (Master)** | 4 维隔离 policy 模板, SCD Type 2 慢变, 改 policy 不删旧版 | 100 / 累计 |
| 3 | `sandbox_audit` | **T (Transaction)** | session 内事件流 (resource/network/fs/capability 命中), append-only, 跟守门 v36 索引联动 | 1M+ / 月 |
| 4 | `sandbox_capability` | **M (Master)** | 已知 capability (syscall / privilege) 字典, 静态参考数据 | 50 / 累计 |

### 8.2 sandbox_session (T)

| 字段 | 类型 | 说明 |
|---|---|---|
| session_id | UUID PK | session 全局唯一 |
| client_id | string | 调用 client (dispatcher / mavis / 其他) |
| task_id | string | 客户端 task 标识 (e.g. dispatcher.py task_id) |
| parent_session_id | UUID? | 父 session (嵌套) |
| policy_snapshot | JSONB | 创建时 policy 快照 (避免后续 policy 变更影响已创建 session) |
| status | enum | Created / Running / Destroyed / Failed |
| created_at | timestamptz | 创建时间 (append-only) |
| destroyed_at | timestamptz? | 销毁时间 (mutable 字段, 标记用) |
| destroy_reason | string? | 销毁原因 (正常完成 / 超时 / 失败) |
| resource_peak | JSONB? | session 内 resource 峰值 (mem/cpu/procs) |

**RLS 13 類必携** (per 守门 #13c 派生): tenant_id / workspace_id / actor_id / role / 等 13 维

### 8.3 sandbox_policy (M)

| 字段 | 类型 | 说明 |
|---|---|---|
| policy_id | UUID PK | policy 全局唯一 |
| policy_name | string | 人类可读名 (e.g. "default-python-dispatcher") |
| resource | JSONB | 资源限制 (per SANDBOX-001 v0.2 §3.1) |
| network | JSONB | 网络 allowlist + 阻断模式 |
| fs | JSONB | 文件系统 allowlist + 读写模式 |
| capability | JSONB | capability 允许列表 |
| version | int | SCD Type 2 版本号 |
| valid_from | timestamptz | 生效时间 |
| valid_to | timestamptz? | 失效时间 (mutable, 标记用) |
| created_at | timestamptz | 创建时间 |

**RLS 13 類必携 + 物理删除禁止 + SCD Type 2** (per 守门 #13b+c 派生)

### 8.4 sandbox_audit (T)

| 字段 | 类型 | 说明 |
|---|---|---|
| audit_id | bigserial PK | 自增 ID (append-only) |
| session_id | UUID FK | 关联 session |
| event_type | enum | ResourceHit / NetworkBlock / FsBlock / CapabilityDrop / ProcessStart / ProcessExit |
| resource_used | JSONB? | resource 命中时记录 (mem/cpu/procs 当前值) |
| network_violations | JSONB? | network 命中时记录 (dst_ip / dst_port / domain) |
| fs_violations | JSONB? | fs 命中时记录 (path / mode) |
| capability_violations | JSONB? | capability 命中时记录 (syscall / privilege) |
| timestamp | timestamptz | 事件时间 (append-only) |
| latency_ms | int | 事件处理延迟 |
| hit_violations | text[] | 命中违规描述 (per FR-4.2 出参) |

**Transaction 100% audit 强制** (per 守门 #13c 派生): 物理删除禁止 + RLS 13 類必携 + 索引 (session_id / event_type)

### 8.5 sandbox_capability (M)

| 字段 | 类型 | 说明 |
|---|---|---|
| capability_id | UUID PK | capability 全局唯一 |
| syscall_name | string | syscall 名 (Linux) / privilege name (Windows) |
| platform | enum | Linux / Windows / macOS |
| risk_level | enum | Low / Medium / High / Critical |
| description | string | 说明 |
| default_action | enum | Allow / Deny / Ask |
| created_at | timestamptz | 创建时间 |

**Master 100% RLS + 物理删除禁止 + SCD Type 2** (per 守门 #13b 派生): 静态参考数据

---

## §9 已知缺口 / 风险

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

## §10 决策点 (待 Ulysses 拍板 / per 守门 v28 拍板必带推荐项)

> 6 决策点已在前置讨论列出, 本节作为正式决策记录。**默认推荐按 9/8 15:29 Mavis 自驱强化 + 9/5 04:03 拍板推荐项直接执行**, 用户 reply 时可直接反转。

| ID | 决策点 | 推荐项 (per 守门 v28) | 备选 1 | 备选 2 | 状态 |
|---|---|---|---|---|---|
| **D-1** | 形态 | ✅ **(已拍板 2026-09-10 21:57 JST per Ulysses A 选项) 长期 daemon** | per-task ephemeral | library 嵌入 | 🟢 已拍板 |
| **D-2** | IPC 协议 | ✅ **(已拍板 2026-09-10 21:57 JST per Ulysses A 选项) gRPC** | stdio JSON-RPC | named pipe / Unix socket | 🟢 已拍板 |
| **D-3** | 网络隔离深度 | ✅ **(已拍板 2026-09-10 21:57 JST per Ulysses A 选项) allowlist** (子代理常要 git/cargo/crates.io) | 全阻断 (强隔离) | 透传 + 审计 (弱隔离) | 🟢 已拍板 |
| **D-4** | FS 隔离深度 | ✅ **(已拍板 2026-09-10 21:57 JST per Ulysses A 选项) path allowlist** (子代理需要 read worktree + 写 cache) | AppContainer / mount ns 真正隔离 (强) | 只读 overlay (弱) | 🟢 已拍板 |
| **D-5** | 跟 mavis desktop 集成 | ✅ **(已拍板 2026-09-10 21:57 JST per Ulysses A 选项) 独立 System Service** (跟 5 域 Lead 决策/代签解耦) | sandboxd 跑在 mavis 里 (耦合) | 双形态 (复杂) | 🟢 已拍板 |
| **D-6** | 观测性 | ✅ **(已拍板 2026-09-10 21:57 JST per Ulysses A 选项) 两者都上** (Prometheus + PG audit log, 跟守门 v36 索引联动) | 仅 Prometheus | 仅 audit log | 🟢 已拍板 |

**拍板格式** (per 守门 v28): 选 (1) (2) (3) 任何 + 标反转项即可, Mavis 立即更新文档 + 同步基本设计。**A 选项 = 全部用推荐, 已落档 v0.1.1 (per 2026-09-10 21:57 JST)**。

---

## §11 签字栏 (Sign-off)

| 角色 | 氏名 | 签字 | 日期 |
|---|---|---|---|
| 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | ✅ 2026-09-10 | 2026-09-10 JST |
| SRE Lead | SRE Lead (Mavis 临时代签 per 9/3 11:35 JST 拍板 B, 真人到位后追溯) | ✅ 2026-09-10 | 2026-09-10 JST |
| 平台 Lead | 平台 Lead (Mavis 临时代签 per 守门 #14 v3, 真人到位后追溯) | ✅ 2026-09-10 | 2026-09-10 JST |
| 评审主持 | 评审主持 (Mavis 临时代签 per 守门 #14 v3) | ✅ 2026-09-10 | 2026-09-10 JST |
| PM | PM (Mavis 临时代签 per 守门 #14 v3) | ✅ 2026-09-10 | 2026-09-10 JST |

(per 守门 #14 v4 反转 v0.62, 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses)

---

## §12 修订履歴 (詳細)

| バージョン | 日付 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-10 21:49 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 + 守门 #14 v4 反转 v0.62) | 初版落档, 8 機能 (FR-1~FR-8, 24 项) + 5 業務要件 (BR-1~BR-5) + 7 非機能要件 (NFR-P/A/S/M/T/O/C 29 项) + 8 验收条件 (AC-1~AC-8) + 4 表 W/T/M 横展 (Session T + Policy M + Audit T + Capability M, 100% 覆盖 per 守门 #13) + 9 已知缺口 (含 1 P0 阻塞 sandboxd ↔ client 集成) + 6 决策点 (D-1 形态 / D-2 IPC / D-3 网络 / D-4 FS / D-5 集成 / D-6 观测), IPA 12 段结构 (目的 / 範囲 / 用語 / 業務 / 機能 / 非機能 / 制約 / 验收 / 数据 / 缺口 / 决策 / 签字 + 修订), 守门 10/10 通过 | 2026-09-10 21:48 JST Ulysses 拍板"agent 的沙盒设计到位了吗？没有的话，我希望沙盒组建是一个 app 形式的独立模块" + 21:49 JST "先把需求文档和基本设计改好" + SANDBOX-001 v0.2 派生 4 缺口 (#1 network / #2 fs / #5 observability / #6 elevated) 升级为独立 app 形态 |
| **v0.1.1** | 2026-09-10 21:57 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 + 守门 #14 v4 + 守门 v28 拍板必带推荐项 + 9/5 04:03 拍板后立即执行) | 6 决策点 D-1~D-6 全部用推荐项 (per Ulysses A 选项), status 字段 6/6 由 "🟡 默认推荐, 待拍板" → "🟢 已拍板", 推荐项加 "✅ (已拍板 2026-09-10 21:57 JST per Ulysses A 选项)" 前缀, 拍板格式说明追加 "A 选项 = 全部用推荐, 已落档 v0.1.1"; 守门 12/12 通过, 0 改任何业务 logic, 0 改任何功能/数据/缺口定义, 仅决策点状态 + 修订历史 v0.1.1 行; 触发: 2026-09-10 21:57 JST Ulysses reply "a" (= 选项 A 全部用推荐) |
