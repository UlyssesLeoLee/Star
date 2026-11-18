# F13 沙盒

> 状态摘要: 沙盒由三层组成, 实现程度不一。sandboxd 守护进程已实现 gRPC 四个 RPC、session 池与超时回收、Windows Job Objects 的 best-effort 限额、提权工具拦截与 JSONL 审计; 网络与文件系统两维在运行路径上只做审计或未接入, Linux 与 macOS 后端未在目标平台验证, 降级路径无法执行被请求的命令。/sandbox 页面为常量演示, 仅"接入实体机"表单写入本地 React 状态。

> 代码基线: HEAD 1643559f (the single baseline for every feature doc; cite this exact hash); 核对方法: 对所引路径做 symbol/grep 检索并读取工作树文件; 引用路径的工作树与 HEAD 1643559f 无差异 (路径限定 git diff 为空); 核对日期 2026-11-17。

## 1. 目标与边界

### 1.1 目标

为子代理、任务卡 CLI 与宿主侧子进程提供受控执行环境: 限制内存、CPU、进程数, 限制网络出站与文件系统访问, 记录审计, 并在沙盒服务不可用时降级。同时在 /sandbox 页面展示实体机、虚拟机与 Docker 资源, 以及观测四级 (原生 → 执行 → 探针 → 元数据)。来源: SRS-SANDBOX-001 §1; SRS-SANDBOX-002 §1.1。

### 1.2 范围内 / 范围外

| 范围 | 内容 | 依据 |
|---|---|---|
| 范围内 | sandboxd 守护进程: gRPC、策略、平台后端、审计、指标、健康检查、Rust 客户端与降级 | crates/sandboxd/src/lib.rs:6-26 |
| 范围内 | agent 工具的命令执行入口 (RunCmd, 命令 deny-list) | crates/agent-bridge/src/sandbox.rs:96-183; crates/agent-bridge/src/tool.rs:125-132 |
| 范围内 | Python 子代理软约束包装器 (SANDBOX-001 降级层) | scripts/automation/guardian/sandbox.py:1-20 |
| 范围内 | /sandbox 页面及其 API 门面 | frontend/src/app/(app)/sandbox/page.tsx:86-107; crates/star-api-rest/src/lib.rs:109-112, 219 |
| 范围外 | 运行时构建沙箱 (direct / container / microvm), 属另一设计层 | 名称与 docs/runtime-design.md 的 sandbox 章节重合; 该文件未读取, 不纳入结论 |
| 范围外 | 多租户、mTLS、K8s 部署、AI 行为审计、凭据保险库 | SRS-SANDBOX-002 §1.4 |

## 2. 用户与场景

| 角色 | 场景 | 状态 |
|---|---|---|
| 子代理 / 任务卡 CLI 调用方 | 在隔离 session 中执行命令, 超时被终止 | 🟡 Rust 客户端可调 gRPC (crates/sandboxd/src/client.rs:111-237); Python dispatcher 未接入 gRPC (scripts/automation/dispatcher.py 无 grpc 引用) |
| agent 工具调用方 | 通过 RunCmd 执行 shell 命令, 危险命令被拒绝 | 🟡 deny-list 为子串匹配 (crates/agent-bridge/src/sandbox.rs:96-113), 无资源与网络隔离 |
| 宿主管理员 | 启动 sandboxd, 查看 /healthz 与 /metrics | 🟡 启动与端点存在 (crates/sandboxd/src/bin/sandboxd.rs:49-115; crates/sandboxd/src/health.rs:38-40); 多数配置字段未被读取 |
| 项目负责人 | 在 /sandbox 查看资源与观测, 登记实体机 | 🟡 页面存在, 数据为常量 (frontend/src/app/(app)/sandbox/page.tsx:48-84) |
| 审计人员 | 查询 session 生命周期与命中记录 | 🟡 JSONL 可用 (crates/sandboxd/src/audit.rs:155-192); PG 生命周期写入有静态缺陷 (见 §6) |

## 3. 需求

优先级取值: P0 核心, P1 重要, P2 增强。优先级取自 SRS-SANDBOX-002 §4; 代码既有而 SRS 未列的项标 "代码既有"。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-13-01 | sandboxd 独立守护进程: YAML 配置加载 (缺文件回退默认值), SIGTERM / ctrl_c 优雅停机 | P0 | 🟡 | 入口与加载 bin/sandboxd.rs:71-72, 113; config.rs:262-270; 停机 server.rs:228-247。systemd、Windows Service、launchd 单元文件在受版本控制的文件中不存在 (见 §6 #21) |
| FR-13-02 | 健康检查 GET /healthz (status, uptime_sec, sessions_active, sessions_total, version) 与 GET /metrics | P1 | ✅ | health.rs:38-62, 65-84; 监听 127.0.0.1:50052 (config.rs:170, 306-308)。偏离: SRS 写 50051 (见 §8) |
| FR-13-03 | session 池与容量拒绝 (默认 100 并发) | P0 | 🟡 | 容量拒绝 orchestrator.rs:274-282; 默认值 config.rs:171。FIFO/LRU 回收与同策略复用 ⬜ (无代码) |
| FR-13-04 | 停机期间拒绝新 session, 等待在飞命令 (宽限期 30s), 强制销毁残留 | P0 | 🟡 | orchestrator.rs:266-270, 582-602; server.rs:219-223。拒绝以 Internal 错误返回 (orchestrator.rs:267-268), 映射为 FAILED 而非 Unavailable; 持久化停机日志 ⬜ |
| FR-13-05 | gRPC 四个 RPC: CreateSession, RunCommand, DestroySession, StreamLogs | P0 | ✅ | proto/sandboxd.proto:8-20; server.rs:43-169 |
| FR-13-06 | gRPC 监听 127.0.0.1:50051; 端口冲突 +1 探测; UDS / 命名管道; mTLS | P1 | 🟡 | 地址与默认端口 config.rs:169, 301-303。端口探测、UDS/命名管道、mTLS 为 ⬜ (无代码) |
| FR-13-07 | StreamLogs: 1000 条环形缓冲, 按 offset 重放, 实时推送 | P1 | 🟡 | orchestrator.rs:38, 64-126, 489-509。运行时只推送 ProcessStart 与 ProcessExit (orchestrator.rs:404-408, 430-435); proto 定义的 8 类事件中其余 6 类无生产推送 (sandboxd.proto:117-125) |
| FR-13-08 | CreateSession 时校验策略并冻结快照 (INV-SBX-02) | P0 | ✅ | orchestrator.rs:272, 289-298, 339; policy/mod.rs:99-125 |
| FR-13-09 | 资源维: 内存、CPU、进程数、父进程退出级联 kill | P0 | 🟡 | Windows: 内存为 best-effort, 特权不足时回退为无内存限制 (backend/windows.rs:49-57, 131-139); CPU 仅映射为优先级类 (windows.rs:61-63, 73-79); 未设置 ActiveProcessLimit (windows.rs:49-65); kill_on_job_close 生效 (windows.rs:58-60, 210-217)。Linux: cgroups v2 写入失败被忽略 (backend/linux.rs:317-323, 需 root)。macOS: 仅审计 (backend/macos.rs:165-171) |
| FR-13-10 | 网络维: 域名 allowlist 与 IP allowlist 决策; BLOCK 时 RST 与审计 | P0 | 🟡 | 决策函数 policy/network.rs:101-161, src 内无生产调用点。Linux: 非 CIDR 出站全部 DROP, CIDR 放行 (backend/linux.rs:58-65, 329-356); 域名不映射为 IP (linux.rs:57)。Windows: 仅审计 (windows.rs:161-165)。macOS: 仅 IP 段 (macos.rs:40-61)。RST 与违规审计 ⬜ |
| FR-13-11 | 文件系统维: 路径 allowlist (glob, 最长匹配), 越界阻断与审计 | P0 | 🟡 | 决策 policy/fs.rs:120-165; glob fs.rs:174-211。运行路径只审计 cwd (backend/mod.rs:379-387; windows.rs:193-197; linux.rs:270-274; macos.rs:206-210)。Linux mount ns 绑定 (linux.rs:99-112, 131-141); macOS subpath 规则 (macos.rs:63-89)。Windows AppContainer ⬜ (windows.rs:167-170 仅 PolicyAuditOnly) |
| FR-13-12 | 能力维: 拦截提权工具 (sudo, runas, psexec, gsudo, doas, su, setcap, getcap, regini) | P0 | ✅ | 拦截 backend/mod.rs:357-374; 调用点 windows.rs:189-191, linux.rs:267-269, macos.rs:203-205。名称匹配, 非内核级约束 |
| FR-13-13 | 能力维 (Linux): capsh 降权, DropAll / AllowNetBindService / Custom | P0 | 🟡 | build_capsh_prefix linux.rs:72-93; 应用于 argv linux.rs:142。依赖目标主机安装 capsh; 未在 Linux 上验证 |
| FR-13-14 | 能力维 (Windows, macOS): token privilege 调整 (D-8 决策为立即实装) | P0 | ⬜ | Windows 与 macOS 仅为 spawn 前拦截与隐式 drop (windows.rs:171-175; macos.rs:182-186), 无 token 调整代码 |
| FR-13-15 | 三平台后端工厂: 按编译目标选择 Windows / Linux / macOS | P0 | 🟡 | DefaultBackendFactory backend/mod.rs:187-211; 模块 cfg 门控 mod.rs:26-31。Linux 与 macOS 后端只在对应平台编译, 当前平台 (Windows) 不编译 |
| FR-13-16 | 审计: JSON Lines 追加写, 11 字段 (NFR-M-3) | P0 | 🟡 | 字段与写入 audit.rs:39-76, 155-192; 默认文件 config.rs:233。违规结构化字段 (resource_used 等四列) 在 audit_event 中恒为 None (audit.rs:126-145), 仅 hit_violations 有值 |
| FR-13-17 | 审计: PostgreSQL 写入 sandbox_session 与 sandbox_audit, 写前注入 RLS 三元组 | P0 | 🟡 | audit.rs:222-291 (set_config 231-242; INSERT 243-283)。静态缺陷: Created 与 Destroyed 事件以同一 session_id INSERT 进 sandbox_session (orchestrator.rs:292-302, 472-482), 而该列为主键 (migrations/001_sandbox_session.sql:7)。真实 PG 行为 ❓ |
| FR-13-18 | 审计写失败 fail-closed: 创建路径阻断; 运行与销毁路径不得静默吞错 (NFR-A-2) | P0 | 🟡 | 创建路径 orchestrator.rs:303-309; 启动路径 bin/sandboxd.rs:88-93。运行与销毁路径忽略审计错误: orchestrator.rs:326, 403, 429, 483 |
| FR-13-19 | 审计保留 90 天 | P2 | ⬜ | 配置字段 config.rs:127-128, 仅校验 config.rs:279-283; 无清理代码 (字段在 src 内无读取点) |
| FR-13-20 | 指标: Prometheus 族 (SRS 文字写 9 个) | P1 | 🟡 | 8 个族 metrics.rs:57-112: sessions_total, sessions_active, session_duration_seconds, resource_limit_hits_total, network_violations_total, fs_violations_total, capability_violations_total, rpc_duration_seconds |
| FR-13-21 | 降级: sandboxd 不可用时 client 回落到 SANDBOX-001 子进程路径 (fail-open) | P0 | 🟡 | run_with_failover client.rs:243-337; run_fallback client.rs:340-407。降级程序为 `python sandbox.py <cmd>` (client.rs:47-53, 362-363), 而该文件无命令行入口 (sandbox.py 全文无 `__main__` 与 argv 解析), 被请求命令不会执行 |
| FR-13-22 | 降级事件通知 root session (< 500ms, NFR-O-3) | P1 | ⬜ | 配置字段 config.rs:140 无读取点; 无通知代码 |
| FR-13-23 | 超时: session 默认 1800s 后台扫描强制销毁; 命令超时 kill 并返回码 124 | P1 | ✅ | config.rs:172; orchestrator.rs:41, 550-579 (扫描间隔 30s, :554); backend/mod.rs:93, 255-297 (kill :290-295), 违规记录 :339-343 |
| FR-13-24 | DestroySession: 级联 kill 与回收隔离, 写审计 | P0 | 🟡 | orchestrator.rs:440-486; Windows Job 释放 windows.rs:208-219; Linux 删除 netns 与 cgroup (best-effort) linux.rs:284-298; macOS 仅置标记 macos.rs:220-223; Destroyed 审计错误被丢弃 orchestrator.rs:483 |
| FR-13-25 | 环境变量只记录 KEY 名, 不记录值 (守门 #5, INV-SBX-05) | P0 | ✅ | backend/mod.rs:57-63; orchestrator.rs:407 (ProcessStart 的 env_keys) |
| FR-13-26 | 策略四维模型 (resource / network / fs / capability) 与 proto 互转 | P0 | ✅ | policy/mod.rs:36-46, 153-173; proto/sandboxd.proto:39-76 |
| FR-13-27 | 5 域策略模板 (player / economy / match / social / admin, D-10) | P2 | 🟡 | Domain policy/mod.rs:49-72; SessionSpec::for_domain policy/mod.rs:79-91 (admin: 4096 MB, 100%, 追加 3 个域名)。src 内无调用点 |
| FR-13-28 | 多租户、TLS / mTLS、K8s 部署 | P2 | ⬜ | SRS-SANDBOX-002 §1.4; 无代码 |
| FR-13-29 | 测试: 单元、集成、e2e (SRS-SANDBOX-002 FR-8) 与性能基准 (NFR-P) | P1 | 🟡 | 测试见 §7。基准 ⬜ (crates/sandboxd 无 benches 目录, 受版本控制的文件中无匹配) |
| FR-13-30 | 实施报告 docs/reports/PHASE-SANDBOX-002-IMPL-REPORT.md | P1 | ⬜ | 受版本控制的文件中无该报告 |
| FR-13-31 | /sandbox 页面: 资源树 (实体机、虚拟机、Docker)、观测四级、沙盒详情 | P1 | 🟡 | 资源树与详情为常量 page.tsx:48-84, 153-179, 255-278; 观测级别读自 API page.tsx:182-186 (数据源 integrations.rs:176) |
| FR-13-32 | 页面数据经 argo / sandtree API 获取, 失败回退本地常量 | P1 | 🟡 | 拉取 page.tsx:92-107; 路由 lib.rs:109, 111-112 (挂载于 /api/v1, lib.rs:219); 处理函数返回演示常量 integrations.rs:141-164, 173-180 |
| FR-13-33 | 接入实体机表单 (主机、系统类型、接入方式) 与登记列表 | P1 | 🟡 | 表单 page.tsx:193-245; 提交 submitAttach page.tsx:115-122 仅写入 React 状态, 刷新后丢失; 后端探测 ⬜ (页面文案 page.tsx:236 自述) |
| FR-13-34 | 页面操作: 新建沙盒、启动、停止、快照、销毁、开设 Worktree、登录、同步 | P1 | ⬜ | 按钮无 onClick: page.tsx:129, 268-277, 322-325, 366-368 |
| FR-13-35 | Worker 心跳与任务租约展示 (SRS-SANDBOX-001 FR-SB-005) | P2 | ⬜ | page.tsx 无对应文本; 仅原型文件含 heartbeat / lease 文字 |
| FR-13-36 | 命令 deny-list (RunCmd): 子串匹配拒绝 rm -rf /, mkfs, dd if=, shutdown, reboot, git push --force 等 10 项 | 代码既有 | ✅ | crates/agent-bridge/src/sandbox.rs:96-113 (matching 110-113); 调用 tool.rs:125-132 |
| FR-13-37 | Python 子代理软约束包装器: Windows Job Objects 限额, 不可用时 fail-open | P1 | 🟡 | 可用性 sandbox.py:61-69; Job 配置 sandbox.py:72-113; fail-open 直接 subprocess.run sandbox.py:143-152; dispatcher 导入保护 dispatcher.py:67-77, 默认 use_sandbox dispatcher.py:197-198, 调用 dispatcher.py:302-325。网络与文件系统维未实装 (sandbox.py:15-19) |
| FR-13-38 | dispatcher 切换为 sandboxd gRPC client (BR-1) | P0 | ⬜ | scripts/automation/dispatcher.py 无 grpc 引用 |
| FR-13-39 | mavis desktop 与 star-mcp 作为 Rust 客户端接入 (NFR-C-3) | P0 | ⬜ | 无 crate 依赖 sandboxd (crates/*/Cargo.toml 中仅 crates/sandboxd/Cargo.toml 与根 Cargo.toml 的 workspace 成员声明) |
| FR-13-40 | sandboxd session 与 ARG 集成: actor_id 注入 ARG Edge (D-7) | P2 | ⬜ | orchestrator.rs:155-156 注释为"未来注入"; actor_id 来自配置 config.rs:143-164 |
| FR-13-41 | 复用 star-telemetry、star-pg-adapter、star-eventbus (D-9) | P2 | ⬜ | 直接依赖 prometheus (crates/sandboxd/Cargo.toml:42) 与 sqlx (:56); 未依赖上述 crate |

## 4. 设计

### 4.1 概念模型

| 概念 | 定义 | 证据 |
|---|---|---|
| Session | 隔离执行单元, 由 client_id 与 task_id 标识, 持有冻结的 SessionSpec 与一个 BackendHandle | orchestrator.rs:129-163 |
| SessionSpec | 四维策略快照: resource, network, fs, capability | policy/mod.rs:36-46 |
| BackendHandle | 单个 session 的平台句柄, 提供 run, destroy, health | backend/mod.rs:168-179 |
| EnforcementLevel | 每维强制力声明: Kernel, ProcessWrapper, PolicyAuditOnly | backend/mod.rs:129-138 |
| AuditEvent | 11 字段审计事件, 目标表 Session 或 Audit, 附 RLS 三元组 | audit.rs:31-76 |
| Actor | tenant_id, workspace_id, actor_id, 来自服务端配置 | config.rs:143-164; policy/mod.rs:131-149 |
| Domain | 五个业务域的策略模板标识 | policy/mod.rs:49-61 |
| 观测四级 | native, execution, probe, metadata (页面展示用) | integrations.rs:176; page.tsx:181-188 |

各维强制力 (EnforcementLevel) 按平台:

| 维 | Windows | Linux | macOS |
|---|---|---|---|
| resource | Kernel (windows.rs:154-160) | ProcessWrapper (linux.rs:229-235) | PolicyAuditOnly (macos.rs:165-171) |
| network | PolicyAuditOnly (windows.rs:161-165) | ProcessWrapper (linux.rs:236-240) | ProcessWrapper (macos.rs:172-176) |
| fs | PolicyAuditOnly (windows.rs:166-170) | ProcessWrapper (linux.rs:241-245) | ProcessWrapper (macos.rs:177-181) |
| capability | PolicyAuditOnly (windows.rs:171-175) | ProcessWrapper (linux.rs:246-250) | ProcessWrapper (macos.rs:182-186) |

三层关系: sandboxd (Rust, gRPC) 为主路径; 不可用时 client 降级到 Python 包装器 (sandbox.py); agent 工具的 RunCmd 直接调用 agent-bridge 的 deny-list 执行器, 不经过 sandboxd (tool.rs:125-132)。

### 4.2 数据模型

PostgreSQL 表 (migrations/001-005), 按守门 #13 的 W/T/M 分类:

| 表 | 分类 | 主键 | 关键字段 | 代码写入情况 | 证据 |
|---|---|---|---|---|---|
| sandbox_session | T | session_id UUID | client_id, task_id, parent_session_id, policy_snapshot JSONB, status (Created / Running / Destroyed / Failed), created_at, destroyed_at, destroy_reason, resource_peak, tenant_id, workspace_id, actor_id | 每个生命周期事件 INSERT; destroyed_at、destroy_reason、resource_peak 不写入 | migrations/001_sandbox_session.sql:6-22; audit.rs:243-262 |
| sandbox_policy | M | policy_id UUID | policy_name, resource / network / fs / capability JSONB, version, valid_from, valid_to; UNIQUE (policy_name, version, tenant_id) | 无写入代码 | migrations/002_sandbox_policy.sql:6-22 |
| sandbox_audit | T | audit_id BIGSERIAL | session_id, event_type (8 值 CHECK), 四个 JSONB 违规列, timestamp, latency_ms, hit_violations TEXT[] | INSERT; 事件类型仅 ProcessStart 与 ProcessExit; JSONB 违规列恒为 NULL | migrations/003_sandbox_audit.sql:6-30; audit.rs:263-283; orchestrator.rs:396, 422 |
| sandbox_capability | M | capability_id UUID | syscall_name, platform, risk_level, description, default_action; UNIQUE (syscall_name, platform, tenant_id) | 无写入; 内存字典 KNOWN_CAPABILITIES 52 项 (Linux 41, Windows 8, macOS 3) | migrations/004_sandbox_capability.sql:6-19; policy/capability.rs:100 |

RLS (migrations/005_rls_policies.sql:6-24): 四张表的策略均只比较 tenant_id 与 workspace_id; actor_id 不参与策略。SRS-SANDBOX-002 §8 要求 RLS 13 类, 代码为 3 个 UUID 列。

审计事件的 11 字段结构: audit.rs:39-76。在 audit_event 构造中, resource_used、network_violations、fs_violations、capability_violations 恒为 None (audit.rs:133-137)。

### 4.3 接口

gRPC 服务 SandboxService (crates/sandboxd/proto/sandboxd.proto:8-20):

| RPC | 请求字段 | 响应字段 | 实现 |
|---|---|---|---|
| CreateSession | client_id, task_id, parent_session_id, policy, timeout_sec (proto:24-29) | session_id, status (READY / FAILED), error (proto:32-37) | server.rs:43-79; 失败以 FAILED 字段返回, 不抛 gRPC 错误 (server.rs:73-77) |
| RunCommand | session_id, command (cmd, env, cwd, capture_output), timeout_sec (proto:78-89) | returncode, stdout, stderr, latency_ms, hit_violations (proto:91-97) | server.rs:82-115; 空 argv 返回 INVALID_ARGUMENT (server.rs:97-99); session 不存在返回 NOT_FOUND (server.rs:112) |
| DestroySession | session_id, reason (proto:99-102) | status (DESTROYED / NOT_FOUND) (proto:104-107) | server.rs:118-144 |
| StreamLogs | session_id, from_offset (uint32, proto:109-112) | stream LogEntry (proto:114-130) | server.rs:149-168 |

HTTP (crates/sandboxd/src/health.rs:38-40): GET /healthz、GET /metrics, 监听 127.0.0.1:50052 (config.rs:306-308)。

API 门面 (crates/star-api-rest):
- GET /api/v1/argo/nodes: lib.rs:109 → integrations.rs:141-164, 返回 supported_cli 与 demo_nodes 常量。
- GET /api/v1/sandtree/resources: lib.rs:111-112 → integrations.rs:173-180, 返回 observation_levels 常量。

前端: 路由 /sandbox (page.tsx:86); 导航项 frontend/src/lib/nav/registry.ts:594-602, 描述为"会话列表 / 运行命令"。

Rust 客户端 (crates/sandboxd/src/client.rs): SandboxdClient::is_available (:122), create_session (:127), run_command (:160), destroy_session (:191), stream_logs (:217), run_with_failover (:243)。

Python: run_with_sandbox(cmd, *, timeout, cwd, limits, capture_output) (sandbox.py:133-141); make_default_sandbox_limits (sandbox.py:193); dispatcher.invoke(use_sandbox=True, sandbox_limits=None) (dispatcher.py:197-198)。

agent 工具: ToolCall::RunCmd { cmd, timeout_secs } (crates/agent-bridge/src/tool.rs:125-132) → sandbox::run(cmd, cfg) (crates/agent-bridge/src/sandbox.rs:117)。

配置 (crates/sandboxd/config/sandboxd.yaml): 结构体 config.rs:23-164。运行时实际读取的字段: server.grace_shutdown_sec (server.rs:221), actor.* (config.rs:157-163), observability.audit_log_file 与 audit_log_pg_url (bin/sandboxd.rs:77-82)。isolation.*、failover.* 与 observability.prometheus_enabled、audit_log_retention_days 在 src 内无读取点。

### 4.4 状态机与关键流程

Session 状态 (orchestrator.rs:44-54), 与 crates/sandboxd DD §4.1 对照:

| 迁移 | 设计 (DD §4.1) | 代码 | 证据 |
|---|---|---|---|
| 创建 → Created | 是 | 是 | orchestrator.rs:340 |
| Created → Running (首个 RunCommand) | 是 | 是 | orchestrator.rs:379-390 |
| Running → Destroyed (DestroySession 或超时) | 是 | 是 | orchestrator.rs:440-486; 超时 :550-579 |
| Created → Failed (策略应用失败) | 是 | 否: 失败时 session 不入池, 仅写 Failed 审计事件 | orchestrator.rs:311-333 |
| Running → Failed | 是 | 否: 无赋值点 | orchestrator.rs:53, 59, 382 |

流程:
- CreateSession: 停机检查 (orchestrator.rs:266-270) → 策略校验 (:272) → 容量检查 (:274-282) → Created 审计, 失败则拒绝 (:292-309) → backend.apply (:311) → 失败时写 Failed 审计并返回错误 (:315-331) → 入池 (:334-352)。
- RunCommand: 状态迁移 (:379-390) → ProcessStart 审计, 错误忽略 (:392-403) 与日志 (:404-408) → backend.run (:411) → 违规分类为指标 (:416) → ProcessExit 审计, 错误忽略 (:418-429) 与日志 (:430-435)。Windows 后端的 run 顺序: 提权检查 → cwd 审计 → 启动子进程 → Job 赋值 → 等待或超时 kill (windows.rs:182-206; backend/mod.rs:255-297)。
- DestroySession: 出池 (:445-448) → 状态与时间 (:449-457) → backend.destroy (:459) → 指标 (:461-470) → Destroyed 审计, 错误忽略 (:472-483)。
- 降级: run_with_failover 在连接、CreateSession、RunCommand 任一步失败时调用 run_fallback (client.rs:250-324); RunCommand 失败时不调用 DestroySession (client.rs:319-324), session 残留至超时扫描。降级结果的 hit_violations 带 "failover:" 前缀 (client.rs:398)。
- 超时扫描: 每 30 秒, 销毁 created_at 距今超过 timeout_sec 的 session, 原因 "session_timeout" (orchestrator.rs:554-573)。
- 停机: shutting_down 置位 (:583) → 等待在飞命令至宽限期结束 (:588-594) → 销毁全部剩余 session, 错误仅记录 warn (:595-600)。

### 4.5 权限与安全

- 提权拦截: 按可执行文件基名匹配 deny-list, 命中返回 CapabilityViolation (backend/mod.rs:357-374), gRPC 映射为 PERMISSION_DENIED (crates/sandboxd/src/error.rs:161-162)。
- 命令 deny-list (agent 工具): 大小写不敏感的子串匹配 (crates/agent-bridge/src/sandbox.rs:110-113), 10 项 (:96-108)。等价写法与拼接绕过不在覆盖范围内。
- 环境变量: 审计与日志只记录 KEY (backend/mod.rs:57-63; orchestrator.rs:407); 值仅注入子进程 (backend/mod.rs:230-234)。
- 无 unsafe: crates/sandboxd/src/lib.rs:37 `#![forbid(unsafe_code)]`。
- gRPC 无 TLS、无鉴权, 绑定 127.0.0.1 (config.rs:301-303; server.rs 无认证逻辑)。本机任一能连接 127.0.0.1:50051 的进程可以创建 session 并执行命令。
- RLS: 写入前以 set_config 注入 app.tenant_id, app.workspace_id, app.actor_id (audit.rs:231-242); 策略只检查前两者 (migrations/005_rls_policies.sql)。
- fail-closed 仅覆盖创建路径 (orchestrator.rs:303-309); 运行与销毁路径忽略审计错误 (见 FR-13-18)。
- /sandbox 页面文案称 Agent、插件与任务卡 CLI 无法获得宿主 Docker 或虚拟机控制权 (page.tsx:390-393)。代码中无对应的授权执行路径, 该陈述是否成立 ❓。

### 4.6 依赖关系

| 依赖 | 用途 | 证据 |
|---|---|---|
| tonic 0.12, prost 0.13; 构建期 tonic-build 0.12 | gRPC | crates/sandboxd/Cargo.toml:34-35, 63-64 |
| axum (workspace) | /healthz, /metrics | Cargo.toml:38 |
| prometheus 0.14 | 指标 | Cargo.toml:42 |
| sqlx (workspace) | PostgreSQL 审计 | Cargo.toml:56 |
| dashmap 6, ipnet 2 | session 池, CIDR 判定 | Cargo.toml:50, 53 |
| win32job (仅 Windows) | Job Objects | Cargo.toml:60-61 |
| tokio, tokio-stream, tracing, async-trait, serde_json, futures-util | 运行时 | Cargo.toml:20-31 |
| argo-core, sandtree-core (star-api-rest 门面) | CLI 清单与观测级别 | integrations.rs:17 (注释: 移植自用户自有仓库), :143 (argo_core::SUPPORTED), :175 (sandtree-core 注释) |
| scripts/automation/guardian/sandbox.py | 降级程序 | client.rs:47-53; config.rs:244 |
| star-telemetry, star-pg-adapter, star-eventbus | D-9 复用 | ⬜ 未依赖 |
| star-context (ActorContext) | D-7 与 RLS 三元组 | ⬜ 未依赖; actor 来自配置 config.rs:143-164 |
| 无其他 crate 依赖 sandboxd | 集成点 | ⬜ (见 FR-13-39) |

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 守护进程入口 | crates/sandboxd/src/bin/sandboxd.rs | ✅ | :49-115 |
| gRPC 服务 | crates/sandboxd/src/server.rs | ✅ | :43-169; serve :178-225 |
| Orchestrator | crates/sandboxd/src/orchestrator.rs | 🟡 | 生命周期与日志 ✅; Failed 状态从不赋值 :53; 运行、销毁审计错误被丢弃 :326, 403, 429, 483 |
| 配置 | crates/sandboxd/src/config.rs | 🟡 | 解析与校验 ✅ :250-298; 多数 isolation、failover、observability 字段无读取点 |
| 策略: 资源 | crates/sandboxd/src/policy/resource.rs | ✅ | 字段与默认值 :18-38; 校验 :57-66 |
| 策略: 网络 | crates/sandboxd/src/policy/network.rs | 🟡 | 决策 :101-161; src 内无生产调用点 |
| 策略: 文件系统 | crates/sandboxd/src/policy/fs.rs | 🟡 | 决策 :120-165, glob :174-211; 运行路径仅审计 cwd (backend/mod.rs:379-387) |
| 策略: 能力 | crates/sandboxd/src/policy/capability.rs | 🟡 | 字典 :100; 校验 :480; 无数据库写入 |
| 策略: 5 域模板 | crates/sandboxd/src/policy/mod.rs:49-91 | 🟡 | 无调用点 |
| 后端公共逻辑 | crates/sandboxd/src/backend/mod.rs | ✅ | 提权拦截 :357-374; 超时 kill :255-297; cwd 审计 :379-387 |
| 后端: Windows | crates/sandboxd/src/backend/windows.rs | 🟡 | Job Objects ✅ :49-65, 93-120; 网络、文件系统、能力仅审计 :161-175 |
| 后端: Linux | crates/sandboxd/src/backend/linux.rs | 🟡 | cgroups :312-326; netns :329-356; mount ns :99-112; capsh :72-93; 需 root, 未在 Linux 上运行 |
| 后端: macOS | crates/sandboxd/src/backend/macos.rs | 🟡 | profile 生成 :30-93 (纯函数); 运行路径 :193-218; 未在 macOS 上运行 |
| 审计 | crates/sandboxd/src/audit.rs | 🟡 | JSONL ✅ :155-192; PG 写入有静态缺陷 (§6) |
| 客户端与降级 | crates/sandboxd/src/client.rs | 🟡 | Rust 客户端 ✅ :111-237; 降级程序无入口 (§6) |
| 指标 | crates/sandboxd/src/metrics.rs | ✅ | 8 个族 :57-112 |
| 健康检查 | crates/sandboxd/src/health.rs | ✅ | :38-84 |
| 错误映射 | crates/sandboxd/src/error.rs | ✅ | 错误码 :85-95; tonic 状态 :147-170 |
| 数据库迁移 | crates/sandboxd/migrations/001-005 | 🟡 | 表与 RLS 定义 ✅; sandbox_policy 与 sandbox_capability 无写入 |
| 页面 | frontend/src/app/(app)/sandbox/page.tsx | 🟡 | 常量数据; 操作按钮无处理 :129, 268-277, 322-325, 366-368; 接入表单仅本地状态 :115-122 |
| 页面门面 | crates/star-api-rest/src/routes/integrations.rs:137-180; lib.rs:109-112 | 🟡 | 返回演示常量 |
| 导航项 | frontend/src/lib/nav/registry.ts:594-602 | 🟡 | 描述与页面不符 (:600) |
| Python 包装器 | scripts/automation/guardian/sandbox.py | 🟡 | Windows Job 限额 :72-113; 无命令行入口 |
| Python dispatcher 集成 | scripts/automation/dispatcher.py:67-77, 197-198, 302-325 | 🟡 | 使用包装器; 无 gRPC |
| agent 命令 deny-list | crates/agent-bridge/src/sandbox.rs | 🟡 | deny-list :96-113; run :117-183; 超时分支不终止子进程 (:175-183, 全文无 kill 调用); 无资源或网络隔离 |
| agent RunCmd 工具 | crates/agent-bridge/src/tool.rs:125-132 | ✅ | 调用 sandbox::run, 默认超时 60s (:126-128) |
| 协议定义与生成 | crates/sandboxd/proto/sandboxd.proto; build.rs | ✅ | 服务 :8-20 |
| Rust 调用方 | 其他 crate | ⬜ | 无 crate 依赖 sandboxd |

## 6. 差距与后续

| # | 差距 | 影响 | 下一步 |
|---|---|---|---|
| 1 | 降级程序 sandbox.py 无入口; client 把 Python 进程的返回码当作命令结果 (client.rs:340-407) | 不可用时子代理拿到空输出与返回码 0, 错误被掩盖; FR-13-21 不成立 | 为 sandbox.py 增加入口并运行被请求的命令, 或在 run_fallback 直接执行命令; 测试改为调用真实入口 |
| 2 | test_failover_v02_subprocess 以 echo 替代降级程序 (tests/integration/test_failover_v02_subprocess.rs:21-22) | 测试通过不代表真实降级路径可用 | 用真实 sandbox.py 入口替换 echo 断言 |
| 3 | SessionStatus::Failed 从不赋值 (orchestrator.rs:53, 382) | 失败 session 不可观察; 与 DD §4.1 不一致 | 失败时保留记录并置 Failed, 或修订 DD 状态图 |
| 4 | sandbox_session 的 session_id 为主键 (migrations/001:7), Created 与 Destroyed/Failed 以同一 id INSERT (audit.rs:243-262) | 静态上销毁与失败事件违反主键, 错误被丢弃; 真实 PG 行为 ❓ | 改为事件表或 UPDATE; 在真实 PG 上运行 test_pg_audit_persistence (需 DATABASE_URL) |
| 5 | 运行、销毁、Failed 审计错误被丢弃 (orchestrator.rs:326, 403, 429, 483) | 违反 NFR-A-2; 审计缺口不可见 | 上浮错误; 至少记录 error 日志并计数 |
| 6 | 网络维运行时无决策 (policy/network.rs check_host / check_ip 无生产调用); Linux 域名 allowlist 不可达 (linux.rs:57-65); Windows 无网络控制 (windows.rs:161-165) | 默认 allowlist (github.com, crates.io) 在 Linux 上实际不可达; Windows 与 macOS 无域名控制 | 域名解析为 IP 后写入 OUTPUT 规则, 或在 RunCommand 前检查命令中的域名; Windows 接 WFP 或将强制力标注为 PolicyAuditOnly |
| 7 | 文件系统维运行时仅审计 cwd (backend/mod.rs:379-387) | 子代理可读写任意路径; FR-13-11 不成立 | 检查 argv 与环境中的路径; Windows 接 AppContainer; Linux mount ns 的覆盖范围需 root 环境验证 |
| 8 | Windows 资源维: 内存 best-effort (windows.rs:131-139); CPU 仅优先级 (windows.rs:73-79); 无 ActiveProcessLimit (windows.rs:49-65) | 进程数限制 (SRS FR-3.1) 不成立; CPU 无硬上限 | 补充进程数限制; CPU 硬上限 (CpuRate) 单独立项 |
| 9 | 配置字段大多无读取点 (config.rs; src 内无引用) | 运维修改 isolation、failover、prometheus_enabled 无效果 | 将默认值接入 CreateSession 缺省策略与降级开关 |
| 10 | 降级时 RunCommand 失败不销毁 session (client.rs:319-324) | session 占用池容量直到超时 | 失败分支调用 DestroySession |
| 11 | Linux 与 macOS 后端未在目标平台运行 (backend/linux.rs 与 macos.rs 仅在对应平台编译; test_backend_linux.rs:10, test_backend_macos.rs:10 为平台门控) | 两平台行为不可判定 | 在 Linux 与 macOS 上运行对应测试 |
| 12 | 审计 90 天保留无清理 (config.rs:127-128 无读取) | FR-6.2 不成立 | 增加清理任务, 或在文档中标注未实现 |
| 13 | 指标 8 个族; SRS 文字写 9 个 | 文档与实现不一致 | 以代码为准修订 SRS 文字 |
| 14 | 端口: SRS 写 gRPC 与 HTTP 同为 50051; 代码 HTTP 为 50052 (config.rs:43-48) | 文档偏差 | 文档同步 |
| 15 | 页面操作按钮无处理; 资源数据为常量; 导航描述与页面不符 (registry.ts:600) | 用户以为能操作, 实际不能 | 接入 gRPC 的 CreateSession 与 DestroySession, 或将按钮设为禁用并标注未接入; 修正导航描述 |
| 16 | sandbox_policy 与 sandbox_capability 无写入; 能力字典仅在内存 (capability.rs:100) | 数据模型与代码不一致 | 增加 seed 与写入, 或声明内存字典为权威 |
| 17 | Python dispatcher 与 mavis desktop 未接入 gRPC; 无 crate 依赖 sandboxd | 跨 client 复用未实现 (BR-1) | 增加 gRPC client (SANDBOX-002 SBX-09、SBX-10) |
| 18 | gRPC 无鉴权 (server.rs 无认证) | 本机任意进程可执行命令 | 增加本机 token, 或使用 UDS 配合文件权限 |
| 19 | agent deny-list 为子串匹配; 超时分支不终止子进程 (agent-bridge sandbox.rs:110-113, 175-183) | 可被等价写法绕过; 超时命令可能残留 | 增加进程组 kill 与超时清理; 明确 deny-list 与 sandboxd 的职责边界 |
| 20 | 性能基准、UAT、实施报告缺失 (crates/sandboxd 无 benches; docs/reports 无 PHASE-SANDBOX-002) | NFR-P-1 至 NFR-P-5 未验证 | 按 TEST-DESIGN-SANDBOX-002 §5 增加 benches; 完成报告 |
| 21 | 部署单元 (systemd, Windows Service, launchd) 在受版本控制的文件中不存在 | 无法按 FR-1.1 常驻运行 | 增加单元文件与安装说明 |
| 22 | 多租户、TLS, K8s 为 P2 范围外 | 跨主机不可用 | 保持范围外; 列入后续版本 |

## 7. 验收与测试

### 7.1 现有测试

crates/sandboxd/Cargo.toml 注册 15 个 [[test]] 目标 (:72-131), 另有 client.rs 内联测试。测试函数统计按 `#[test]` 与 `#[tokio::test]` 计数。

| 目录 / 文件 | 测试数 | 平台门控 | 覆盖内容 |
|---|---|---|---|
| tests/unit/test_policy_resource.rs | 8 | 无 | 资源默认值与校验 |
| tests/unit/test_policy_network.rs | 9 | 无 | 域名与 CIDR 判定 |
| tests/unit/test_policy_fs.rs | 10 | 无 | glob 与路径判定 |
| tests/unit/test_policy_capability.rs | 8 | 无 | 字典与校验 |
| tests/unit/test_orchestrator.rs | 10 | 无 | 生命周期、容量、审计失败时 fail-closed (:299) |
| tests/unit/test_metrics.rs | 4 | 无 | 指标注册 |
| tests/unit/test_audit.rs | 5 | 无 | JSONL 与 PG 记录构造 |
| tests/unit/test_config.rs | 8 | 无 | YAML 加载与校验 |
| tests/unit/test_backend_linux.rs | 6 | `#![cfg(target_os = "linux")]` (:10) | cgroup 文本, netns 脚本, capsh 前缀 |
| tests/unit/test_backend_macos.rs | 4 | `#![cfg(target_os = "macos")]` (:10) | sandbox-exec profile 文本 |
| tests/integration/test_grpc_create_run_destroy.rs | 8 | 无 | gRPC 生命周期 |
| tests/integration/test_failover_v02_subprocess.rs | 4 | 无 | 降级路径; 以 echo 替代降级程序 (:21-22) |
| tests/integration/test_pg_audit_persistence.rs | 4 | 需 DATABASE_URL (:8-9, :15), 否则跳过 | PG 审计 |
| tests/integration/test_backend_smoke.rs | 6 | `#[cfg(windows)]` 与 `#[cfg(not(windows))]` 分支 | 后端冒烟 |
| tests/e2e/test_dispatcher_integration.rs | 4 | 无 | 进程内 daemon, 执行 git --version (:21-24) |
| src/client.rs (内联) | 3 | 无 | 降级规格解析 |
| 合计 | 101 | | |

Python: scripts/automation/guardian/tests/test_sandbox.py 含 10 个测试函数 (:36, 52, 59, 70, 81, 91, 97, 102, 116, 140), 覆盖可用性判定、默认与自定义限额、无限额时回退 subprocess.run、基本成功与失败、超时、沙箱不可用时回退, 以及 Access Denied 时级联 kill 的 fail-open。

前端: 按 sandbox 关键字检索 frontend/src 未发现 /sandbox 页面的专属测试; 命中的测试文件为导航相关的 AppHeader.test.tsx 与 Sidebar.test.tsx。

### 7.2 已验证

- 策略判定与校验: 单元测试覆盖资源、网络、文件系统、能力四维 (tests/unit/test_policy_*.rs)。
- session 生命周期与容量: tests/unit/test_orchestrator.rs。
- gRPC 生命周期: tests/integration/test_grpc_create_run_destroy.rs。
- 审计失败时创建被拒绝: tests/unit/test_orchestrator.rs:299。
- Python 包装器行为: scripts/automation/guardian/tests/test_sandbox.py。

以上测试的通过状态 ❓: 需运行 `cargo test -p sandboxd -j 4` 与 pytest 才能确认。

### 7.3 未验证

- 降级的真实执行路径 (现有测试以 echo 替代)。
- PG 生命周期事件的真实行为与主键冲突 (需 DATABASE_URL 与已迁移的数据库)。
- Linux cgroup、netns、mount ns、capsh 的真实运行 (需 root)。
- macOS sandbox-exec 的实际强制力。
- Windows 上网络与文件系统维的强制效果 (当前无强制)。
- 性能目标 NFR-P-1 至 NFR-P-5 (无基准)。
- /sandbox 页面操作与接入流程 (无测试)。
- Windows 特权不足时内存上限的实际效果。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| docs/requirements/SRS-SANDBOX-001.md | /sandbox 页面需求 FR-SB-001 至 005; 观测四级; 实体机接入 | FR-SB-005 心跳与租约代码中不存在; 修订履历所列提交未在本次核对中检查; "浏览器实测已闭环"无法从代码核实 | 页面需求采用; 状态以代码为准; 浏览器实测 ❓ |
| docs/requirements/SRS-SANDBOX-002.md | sandboxd 功能域 FR-1 至 FR-8; BR-1 至 BR-5; NFR; AC-1 至 AC-8; D-1 至 D-10; W/T/M 数据模型 | 指标 8 族 (代码) 与 9 个 (文字) 不符; 端口 50052 (代码 HTTP) 与 50051 不符; RLS 3 维 (代码) 与 13 类不符; 默认网络 allowlist 7 项 (policy/network.rs:46-55) 与 SRS 5 项不符; 审计保留未实装 | 功能需求作为设计意图采用; 状态以代码判定; 数据模型以迁移文件为准 |
| docs/architecture/SANDBOX-001.md | Python 子进程包装器设计; SandboxLimits; Windows Job 配置; 缺口 1 至 6 | 文件与 /sandbox 页面无引用关系; network 与 fs 为已知缺口; 降级入口缺失 | 作为降级层设计采用 (§4.1, §5) |
| docs/architecture/SANDBOX-002.md | 拓扑; 四维后端选型; gRPC 详设; fail-open 时序; 部署形态表; 缺口 G-SBX-01 至 09; D-1 至 D-10 | 部署单元文件不存在 (§6 #21); D-7 ARG 集成与 D-9 复用未实装 | 作为目标架构; 状态以代码为准 |
| docs/test-design/TEST-DESIGN-SANDBOX-002.md | 五级测试设计 (UT 56, IT 17, E2E 11, PT 5, UAT 8); TC 编号 | 代码有 101 个测试函数; 测试设计的 TC 编号与代码测试函数名未建立映射; 无 benches; 无 UAT 文件 | 以实际测试为准; PT 与 UAT 标 ⬜ |
| docs/basic-design/SANDBOX-BASIC-DESIGN-002.md | 模块划分; Backend 抽象; gRPC 契约; 表结构; 部署与监控 (仅读取章节标题与 §4 至 §6 的对应代码条目) | 具体差异以 §6 对应条目为准 | 章节结构作为设计意图采用 |
| docs/detailed-design/DD-SANDBOX-002.md | 状态机 §4.1 (读取图示与守则); DDL §6; 测试矩阵 §5 | §4.1 包含 Running → Failed 与 Created → Failed, 代码未实现; DDL 与迁移文件基本一致 | 状态机以代码为准; Failed 迁移标 🟡 |
| docs/implementation-plans/SANDBOX-IMPL-PLAN-002.md | 5 阶段 16 子项 (P0 4, P1 4, P2 3, P3 3, P4 2) 与风险矩阵 (读取 §1.1 与阶段表) | 计划未与代码状态对齐; WBS 标为计划中 | 作为后续规划依据 (§6) |
| docs/WBS.md (第 90 至 92 行) | 6.5 进行中; 6.5.1 已完成; 6.5.2 标 91 个测试、90% | 代码为 101 个测试函数; 6.5.2 的进度百分比未核实 | 进度以代码为准 |
| docs/guardian/v37_sandbox_guard.md (前 12 行) | 状态 Active v0.2; 五维软约束 (路径、网络、资源、能力、模式标志) | 网络与文件系统维仅声明 (sandbox.py:15-19) | 作为 Python 软约束的设计来源 |
| docs/ci-cd.md | CI 失败条目 (检索命中第 158 行附近) | guardian 覆盖率数字未核实 | 不作为证据 |
| docs/architecture/2026-11-14-ops-console/REUSE-MAP-COLLAB-SANDBOX-001.md | 复用清单 (SandTree, Argo) 与授权 (检索命中) | 正文未读取 | 复用清单内容 ❓; 代码侧仅依据 integrations.rs:17 注释 |
| docs/prototypes/2c · 沙盒-html/README.md | 原型为参考, 不是实现 | 无 | 仅作视觉参考 |
| docs/prototypes/2c · 沙盒-html/Sandbox.dc.html | 原型标签与文案 (检索计数: Docker 12, Multipass 5, heartbeat 2, lease 2, 心跳 1, 租约 1) | 未逐行读取; heartbeat、lease、Multipass 未进入代码 | 仅作视觉与文案参考 |
| docs/prototypes/INDEX.md (第 18 行) | 原型与 SRS-SANDBOX-001、页面、sandtree-core、argo-core 的映射 | 无 | 采用映射关系 |
| crates/sandboxd/README.md | 4 RPC; 三平台后端表; fail-open 与 fail-closed 描述; 数据模型摘要 | README 称降级经 run_with_failover 执行, 降级程序无入口 (client.rs:340-407) | 采用; 降级缺陷见 §6 #1 |
| crates/sandboxd/Cargo.toml; config/sandboxd.yaml; proto/sandboxd.proto; migrations/001 至 005; src 全部模块; tests 全部文件 | 实现与契约 | 无 | 按代码判定状态 |
| crates/star-api-rest/src/routes/integrations.rs; crates/star-api-rest/src/lib.rs (路由表) | 门面 argo 与 sandtree 端点 | 返回演示常量 | 标 🟡 |
| crates/agent-bridge/src/sandbox.rs; crates/agent-bridge/src/tool.rs (RunCmd 分支) | 命令 deny-list 与执行 | 与 sandboxd 无关联; 超时分支不终止子进程 | 作为独立执行层记录 (§5, §6 #19) |
| scripts/automation/guardian/sandbox.py; scripts/automation/dispatcher.py (导入保护与 invoke 集成段); scripts/automation/guardian/tests/test_sandbox.py (测试函数名) | Python 包装器实现与集成 | 无 CLI 入口; dispatcher 无 gRPC | 按代码判定状态 |
| frontend/src/app/(app)/sandbox/page.tsx | 页面结构、常量数据、门面调用、接入表单 | 操作按钮无处理 (:129, 268-277, 322-325, 366-368) | 按代码判定状态 |
| frontend/src/lib/nav/registry.ts (594-602) | 导航项 sandbox | 描述 "会话列表 / 运行命令" 与页面不符 (:600) | 标注差异 |
| docs/features/F13-sandbox.md (同路径既有稿, 基线 cdd00a78) | 章节结构与部分结论 | 页面行号已变动; Python 测试数为 10; 测试目标数为 15 (非 16); 降级测试以 echo 替代 | 以本文件为准 |
| 仅关键字命中、本文未读取正文的文件: docs/runtime-design.md; docs/reviews/*; docs/reports/*; docs/briefs/*; docs/implementation-plans/* (非 SANDBOX-IMPL-PLAN-002); docs/requirements/SRS-AUTOMATION-SCHEDULE-API-001.md; docs/architecture/2026-11-15-lane-workspace/*; docs/architecture/2026-11-14-github-integration/*; docs/automation-design.md; scripts/automation/registry.md; frontend/src/app/(app)/layout.tsx; frontend/src/lib/nav/navStore.ts; frontend/src/components/__tests__/*; crates/domain-local-runtime/src/task_pty.rs; crates/argo-core; crates/sandtree-core; crates/nema-codec | 仅命中关键字 "sandbox" 或 "沙盒" | 与沙盒设计内容无直接依赖 (未逐一核对) | 不纳入结论; runtime-design 属运行时构建层, 不在本功能范围 |
| 项目 AGENTS.md (上下文提供) | 守门 #5 环境变量安全; 守门 #13 W/T/M 分类 | 不含沙盒实现状态 | 仅用于守门口径, 不作状态依据 |
