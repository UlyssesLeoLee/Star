# F13 沙盒

> 状态摘要: sandboxd 守护进程已实现 gRPC 四个 RPC、session 池与超时回收、JSONL 审计，以及 Windows Job Objects 的资源限额与提权工具拦截；网络与文件系统维在运行路径上只做审计或未接入，Linux 与 macOS 后端仅在对应平台编译且未经目标平台验证，Rust 客户端的降级程序没有命令行入口。/sandbox 页面当前为常量演示，接入表单只写入本地状态，操作按钮没有处理器。

> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

为子代理、任务卡 CLI 与宿主侧子进程提供受控执行环境：限制内存、CPU 与进程数，限制网络出站与文件系统访问，记录审计，并在沙盒服务不可用时降级到 SANDBOX-001 子进程包装器。同时在 /sandbox 页面展示实体机、虚拟机与 Docker 资源，以及观测四级（原生 → 执行 → 探针 → 元数据）。来源：SRS-SANDBOX-002 §1.1；SRS-SANDBOX-001 §1。

### 1.2 范围内 / 范围外

| 范围 | 内容 | 依据 |
|---|---|---|
| 范围内 | sandboxd 守护进程：gRPC 服务、四维策略、三平台后端、审计、指标、健康检查、Rust 客户端与降级 | crates/sandboxd/src/lib.rs:6-26 |
| 范围内 | /sandbox 页面及其 argo、sandtree 门面端点 | frontend/src/app/(app)/sandbox/page.tsx:86-107；crates/star-api-rest/src/lib.rs:221-225 |
| 范围内 | Python 子进程软约束包装器，作为守护进程不可用时的降级层 | scripts/automation/guardian/sandbox.py:1-20 |
| 范围内（相邻） | agent 工具 RunCmd 的命令 deny-list 执行器，记录其与 sandboxd 无关联 | crates/agent-bridge/src/sandbox.rs:94-115；crates/agent-bridge/src/tool.rs:125-132 |
| 范围外 | 运行时构建沙箱（direct、container、microvm） | 不属于 SRS-SANDBOX-001 与 SRS-SANDBOX-002 |
| 范围外 | 多租户、TLS 与 mTLS、K8s 部署、AI 行为审计、凭据保险库 | SRS-SANDBOX-002 §1.4 |
| 范围外 | SandTree 与 Argo 的完整产品，仅其内核 crate 作为页面数据源 | crates/sandtree-core/src/lib.rs:1-21；crates/argo-core/src/lib.rs:1-16 |

## 2. 用户与场景

| 角色 | 场景 | 状态 |
|---|---|---|
| 子代理与任务卡 CLI 调用方 | 在隔离 session 中执行命令，超时被终止 | 🟡 Rust 客户端可调用 gRPC（crates/sandboxd/src/client.rs:111-237）；dispatcher.py 无 gRPC 引用，仍使用 SANDBOX-001 包装器（scripts/automation/dispatcher.py:67-77） |
| agent 工具调用方 | 通过 RunCmd 执行 shell，危险命令被拒绝 | 🟡 子串匹配 deny-list（crates/agent-bridge/src/sandbox.rs:94-115），无资源或网络隔离 |
| 宿主管理员 | 启动守护进程，查看 /healthz 与 /metrics | 🟡 启动入口与端点存在（crates/sandboxd/src/bin/sandboxd.rs:49-115；crates/sandboxd/src/health.rs:37-42）；随仓库发布的配置样例无法解析（❓，见 FR-13-02） |
| 项目负责人 | 在 /sandbox 查看资源、登记实体机、开设 Worktree | 🟡 资源为常量，登记为本地状态，开设 Worktree 无动作（page.tsx:48-73、115-122、366-368） |
| 审计人员 | 查询 session 生命周期与命中记录 | 🟡 JSONL 可写（crates/sandboxd/src/audit.rs:155-192）；PG 生命周期写入存在冲突疑点（§6 #4） |

## 3. 需求

优先级取自 SRS-SANDBOX-002 §4；SRS-SANDBOX-001 未标注优先级，此处按页面需求取 P1 或 P2。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-13-01 | 守护进程独立运行，支持 SIGTERM 与 ctrl_c 优雅停机（SRS-002 FR-1.1、FR-1.3） | P0 | 🟡 入口与停机存在；系统服务单元缺失 | crates/sandboxd/src/bin/sandboxd.rs:49-115；server.rs:178-225、228-246；orchestrator.rs:582-602；仓库内无 systemd、launchd 或 Windows Service 单元文件（§6 #22） |
| FR-13-02 | YAML 配置加载与校验，文件缺失或解析失败时回退默认值（FR-1.1） | P0 | 🟡 加载与校验存在；随仓库的样例配置无法解析（❓）；多数字段无读取点 | crates/sandboxd/src/config.rs:250-298、262-270；bin/sandboxd.rs:72；config/sandboxd.yaml:20、31、40 |
| FR-13-03 | 健康检查 GET /healthz，返回 status、uptime_sec、sessions_active、sessions_total、version（FR-6.3） | P1 | ✅ 监听端口为 50052，SRS 写 50051 | crates/sandboxd/src/health.rs:37-62；config.rs:169-170、44-48 |
| FR-13-04 | Prometheus 指标 GET /metrics（FR-6.1） | P1 | 🟡 注册 8 个指标族；网络与文件系统违规计数无运行时来源 | crates/sandboxd/src/metrics.rs:57-112；orchestrator.rs:607-611；backend/mod.rs:383 |
| FR-13-05 | session 池，默认 100 并发，超限拒绝（FR-1.2） | P0 | 🟡 容量拒绝存在；FIFO、LRU 回收与同策略复用未实现 | orchestrator.rs:274-282；config.rs:171 |
| FR-13-06 | 停机期间拒绝新 session，宽限期 30 秒，强制销毁残留（FR-1.3） | P0 | 🟡 拒绝以 Internal 错误返回，映射为 FAILED 而非 UNAVAILABLE；停机日志未持久化 | orchestrator.rs:266-270；server.rs:73-77；orchestrator.rs:582-602 |
| FR-13-07 | session 默认 1800 秒超时，后台每 30 秒扫描并强制销毁（FR-4.3） | P0 | ✅ | config.rs:172；orchestrator.rs:41、550-579；bin/sandboxd.rs:111 |
| FR-13-08 | SandboxService 四个 RPC：CreateSession、RunCommand、DestroySession、StreamLogs（FR-2.1） | P0 | ✅ | crates/sandboxd/proto/sandboxd.proto:8-20；server.rs:43-169 |
| FR-13-09 | CreateSession 失败以 FAILED 与 error 字段返回，不抛 gRPC 错误（FR-4.1） | P0 | ✅ 延迟目标 p50、p99 未经基准验证（❓） | server.rs:43-79 |
| FR-13-10 | RunCommand 拒绝空 argv；session 不存在时返回 NOT_FOUND（FR-4.2） | P0 | ✅ | server.rs:97-99、112 |
| FR-13-11 | DestroySession 级联 kill 并回收隔离，返回 DESTROYED 或 NOT_FOUND（FR-4.3） | P0 | 🟡 Windows 经 Job 释放实现级联 kill；Linux 与 macOS 回收为 best-effort；销毁审计错误被丢弃 | server.rs:118-144；backend/windows.rs:208-219；backend/linux.rs:284-298；backend/macos.rs:220-223；orchestrator.rs:483 |
| FR-13-12 | StreamLogs：1000 条环形缓冲、按 offset 重放、实时推送（FR-2.3） | P1 | 🟡 运行时只推送 ProcessStart 与 ProcessExit；proto 定义的八类事件中六类无生产来源 | orchestrator.rs:38、64-126、489-509、404-408、430-435；proto:117-125 |
| FR-13-13 | 监听 127.0.0.1，端口可配（FR-2.2） | P1 | 🟡 地址固定为 127.0.0.1；端口冲突探测、UDS 与命名管道未实现 | config.rs:300-303 |
| FR-13-14 | TLS、mTLS 与 gRPC 鉴权（SRS-002 §1.4，v0.2） | P2 | ⬜ | crates/sandboxd/src 中无拦截器与鉴权代码（server.rs） |
| FR-13-15 | 资源维：内存、CPU、进程数限额，父进程退出级联（FR-3.1） | P0 | 🟡 Job 关闭时级联 kill 已实现（windows.rs:58-60、210-217）；内存为 best-effort；CPU 仅映射为优先级类；进程数未设限；Linux 写入失败被忽略；macOS 仅审计 | backend/windows.rs:49-65、73-79、130-139；backend/linux.rs:313-326；backend/macos.rs:165-171 |
| FR-13-16 | 网络维：域名与 IP allowlist 决策，命中非白名单时阻断、RST 与审计（FR-3.2） | P0 | 🟡 决策函数存在但运行路径无调用；Linux 仅放行 CIDR；域名未解析为 IP；Windows 与 macOS 不做域名控制；RST 与违规审计未实现 | policy/network.rs:101-142；backend/linux.rs:57-65；backend/macos.rs:40-58；backend/windows.rs:161-165；error.rs:38（NetworkViolation 无构造点） |
| FR-13-17 | 文件系统维：路径 allowlist（glob，最长匹配）、越界阻断与审计（FR-3.3） | P0 | 🟡 决策函数存在；运行路径只审计 cwd；Linux mount namespace 与 macOS subpath 规则存在；Windows AppContainer 未实现 | policy/fs.rs:118、120-140；backend/mod.rs:376-387；backend/linux.rs:99-112、131-141；backend/macos.rs:64-89；backend/windows.rs:166-170 |
| FR-13-18 | 能力维：拦截提权工具（sudo、runas、psexec、gsudo、doas、su、setcap、getcap、regini）（FR-3.4，D-8） | P0 | ✅ 按可执行文件基名匹配 | backend/mod.rs:353-374；调用于 windows.rs:189-191、linux.rs:267-269、macos.rs:203-205 |
| FR-13-19 | 能力维（Linux）：capsh 的 DropAll、AllowNetBindService、Custom 三种模式（FR-3.4） | P0 | 🟡 包装 argv 已生成；依赖目标主机安装 capsh；未在 Linux 验证 | backend/linux.rs:67-93、142 |
| FR-13-20 | 能力维（Windows 与 macOS）：token 权限调整（D-8 立即实装）（FR-3.4） | P0 | ⬜ 仅有 spawn 前拦截与隐式 drop，无 token 调整代码 | backend/windows.rs:171-175；backend/macos.rs:182-186 |
| FR-13-21 | 后端工厂按编译目标选择 Windows、Linux 或 macOS 后端（FR-5） | P0 | 🟡 工厂存在；Linux 与 macOS 模块只在对应平台编译 | backend/mod.rs:26-31、187-211 |
| FR-13-22 | CreateSession 时策略校验 fail-fast 并冻结快照（INV-SBX-02，DD §3.1） | P0 | ✅ | policy/mod.rs:99-125；orchestrator.rs:272、289-298、339 |
| FR-13-23 | 5 域策略模板：admin 放宽，其余四域标准默认（D-10） | P2 | 🟡 模板函数存在；无调用点 | policy/mod.rs:49-91 |
| FR-13-24 | 命令超时终止子进程，返回码 124（FR-4.2） | P1 | ✅ | backend/mod.rs:92-93、255-297、339-343 |
| FR-13-25 | 策略命中以 hit_violations 返回，命中不阻断（FR-4.2，fail-open） | P0 | 🟡 超时与 cwd 审计进入 hit_violations；能力拦截以错误返回；网络违规不产生 | server.rs:82-115；backend/mod.rs:339-343、376-387；backend/windows.rs:189-191 |
| FR-13-26 | 降级：守护进程不可用时回落到 SANDBOX-001 子进程路径，结果标注 degraded（FR-7.1） | P0 | 🟡 降级逻辑存在；降级程序无入口，被请求的命令不会执行；dispatcher 未调用该客户端 | crates/sandboxd/src/client.rs:243-337、340-407、361-363；scripts/automation/guardian/sandbox.py（全文无 `__main__` 与 argv 解析） |
| FR-13-27 | 降级事件通知 root session，500 毫秒内推送（NFR-O-3） | P1 | ⬜ 配置字段 root_session_notify 无读取点 | crates/sandboxd/src/config.rs:140 |
| FR-13-28 | 审计 JSONL 追加写，11 个业务字段（FR-6.2、NFR-M-3） | P0 | ✅ 写入可用；违规结构列恒为 NULL | crates/sandboxd/src/audit.rs:155-192、133-137 |
| FR-13-29 | 审计 PostgreSQL 写入 sandbox_session 与 sandbox_audit，写前注入 RLS 三元组（FR-6.2） | P0 | 🟡 注入与插入存在；生命周期事件与主键的冲突未经 PG 验证（❓） | audit.rs:223-291（set_config 231-242；INSERT 245-283）；migrations/001_sandbox_session.sql:7 |
| FR-13-30 | 审计写失败 fail-closed：创建路径阻断，启动路径拒绝（FR-7.2、NFR-A-2） | P0 | 🟡 创建与启动路径存在；运行与销毁路径忽略错误 | orchestrator.rs:292-309；bin/sandboxd.rs:89-93；orchestrator.rs:403、429、483 |
| FR-13-31 | 审计保留 90 天（BR-4、FR-6.2） | P2 | ⬜ 配置仅校验，无清理代码 | config.rs:127-128、279-283 |
| FR-13-32 | 环境变量只记录 KEY，不记录值（守门 #5，INV-SBX-05） | P0 | ✅ | backend/mod.rs:57-63、230-234；orchestrator.rs:407 |
| FR-13-33 | 数据模型四表按 W/T/M 分类，RLS 必携列（守门 #13） | P0 | 🟡 四表迁移存在；policy 与 capability 无写入；RLS 只比较 tenant_id 与 workspace_id；无 REVOKE 与触发器约束 append-only | migrations/001-005（005:6-24） |
| FR-13-34 | 能力字典：52 项（Linux 41、Windows 8、macOS 3）（FR-3.4） | P2 | 🟡 字典在内存中，未写入 sandbox_capability | policy/capability.rs:100、389-441 |
| FR-13-35 | Python 子进程软约束包装器（SANDBOX-001）：Windows Job Objects 限额，不可用时 fail-open | P1 | 🟡 包装器与 dispatcher 集成存在；网络与文件系统维未实装 | scripts/automation/guardian/sandbox.py:61-191、14-19；dispatcher.py:67-77、197-198、302-305 |
| FR-13-36 | dispatcher 切换为 sandboxd gRPC 客户端（BR-1、NFR-C-2） | P0 | ⬜ | scripts/automation/dispatcher.py 中无 grpc 引用 |
| FR-13-37 | mavis desktop 与 star-mcp 作为 Rust 客户端接入（NFR-C-3） | P0 | ⬜ | 仓库内无 crate 依赖 sandboxd（各 crates/*/Cargo.toml 无引用） |
| FR-13-38 | session 的 actor 注入 ARG Edge（D-7） | P2 | ⬜ | orchestrator.rs:155-156 注释为未来注入；actor 来自配置（config.rs:143-164） |
| FR-13-39 | 复用 star-telemetry、star-pg-adapter、star-eventbus（D-9） | P2 | ⬜ 直接依赖 prometheus 与 sqlx | crates/sandboxd/Cargo.toml:42、56 |
| FR-13-40 | 配置 isolation、failover、observability 字段生效（FR-1.1，D-3、D-4 默认值） | P1 | 🟡 server、actor 与审计文件、PG 地址有读取点；isolation、failover、prometheus_enabled、retention 无读取点 | bin/sandboxd.rs:77-109；config.rs:157-163；config.rs:23-141 |
| FR-13-41 | 单元、集成与端到端测试（FR-8.1 至 FR-8.3） | P1 | 🟡 测试属性 101 个，未执行；failover 测试以 echo 替代降级程序 | §7 |
| FR-13-42 | 性能基准 NFR-P-1 至 NFR-P-5 | P1 | ⬜ crates/sandboxd 中无 benches 目录 | git ls-tree（crates/sandboxd）无 benches 条目 |
| FR-13-43 | 实施报告 docs/reports/PHASE-SANDBOX-002-IMPL-REPORT.md（FR-8.3） | P1 | ⬜ 仓库内无该文件 | docs/reports 目录 |
| FR-13-44 | /sandbox 资源树：实体机、虚拟机、Docker 与观测四级（SRS-001 FR-SB-001） | P1 | 🟡 资源为常量；观测级别来自门面 | page.tsx:48-73、127-190、181-188 |
| FR-13-45 | 沙盒详情：类型、系统、资源、接入方式、网络出站；启动、停止、快照、销毁（SRS-001 FR-SB-002） | P1 | 🟡 有名称、系统与资源文字及四个按钮；无类型、接入方式、网络出站字段；按钮无处理器 | page.tsx:255-278 |
| FR-13-46 | 新建沙盒与接入实体机入口（SRS-001 FR-SB-003） | P1 | 🟡 接入对话框存在且写入本地状态；新建沙盒按钮无处理器 | page.tsx:115-122、129、132、193-245 |
| FR-13-47 | 页面经 argo 与 sandtree 门面取数，失败时回退（SRS-001 FR-SB-004） | P1 | 🟡 请求与回退存在；门面返回演示常量；argoCli 未渲染 | page.tsx:88-107、182-186；star-api-rest lib.rs:221-225；routes/integrations.rs:141-164、173-179 |
| FR-13-48 | Worker 心跳与任务租约展示（SRS-001 FR-SB-005） | P2 | ⬜ 页面无对应内容 | page.tsx（无 heartbeat 或 lease 文字） |
| FR-13-49 | 已安装 Agent CLI 列表（SRS-001 §1 与原型） | P2 | ⬜ 页面无对应内容 | page.tsx；原型 Sandbox.dc.html 含"已安装的 Agent CLI" |
| FR-13-50 | 沙盒操作可用：启动、停止、快照、销毁、开设 Worktree、登录、同步 | P1 | ⬜ 按钮无处理器 | page.tsx:129、274、322-324、366 |
| FR-13-51 | 页面标签：概览、Worktree、终端、Docker、观测各有内容（原型） | P2 | 🟡 标签切换只改变样式；内容固定为 Worktree | page.tsx:38、281-299 |
| FR-13-52 | 命令 deny-list：子串匹配，拒绝 10 类危险命令（代码既有） | P2 | ✅ | crates/agent-bridge/src/sandbox.rs:94-115；tool.rs:125-132 |
| FR-13-53 | RunCmd 超时终止子进程（代码既有行为的补充） | P2 | ⬜ 超时分支不终止子进程，文件中无 kill 调用，也未设置 kill_on_drop | crates/agent-bridge/src/sandbox.rs:154-184 |

## 4. 设计

### 4.1 概念模型

| 概念 | 定义 | 代码位置 |
|---|---|---|
| Session | 隔离执行单元，持有创建时冻结的 SessionSpec 与一个平台 BackendHandle；带 client_id、task_id、parent_session_id 与 tenant、workspace、actor 三元组 | orchestrator.rs:130-163 |
| SessionSpec | 四维策略快照：resource、network、fs、capability | policy/mod.rs:36-46 |
| Backend、BackendHandle、BackendFactory | 平台后端抽象：apply 生成 handle；handle 提供 run、destroy、health | backend/mod.rs:151-185 |
| EnforcementLevel | 每维强制力声明：Kernel、ProcessWrapper、PolicyAuditOnly | backend/mod.rs:129-138 |
| AuditEvent、AuditSink | 审计事件（目标表 Session 或 Audit）与三种 sink：JSONL、PG、Composite | audit.rs:31-76、149-153、294-314 |
| ActorIds | 服务端注入的 RLS 三元组，来自配置 | policy/mod.rs:131-149；config.rs:143-164 |
| Domain | 五个业务域的策略模板标识：player、economy、match、social、admin | policy/mod.rs:49-72 |
| 观测四级 | native、execution、probe、metadata，页面展示用常量 | routes/integrations.rs:176 |

各平台声明的强制力（代码中为固定声明，不随运行时降级变化）：

| 维 | Windows | Linux | macOS |
|---|---|---|---|
| resource | Kernel（Job Objects；内存为 best-effort）windows.rs:154-160 | ProcessWrapper（cgroup 写入失败被忽略）linux.rs:229-235、313-326 | PolicyAuditOnly macos.rs:165-171 |
| network | PolicyAuditOnly windows.rs:161-165 | ProcessWrapper（仅 CIDR）linux.rs:236-240 | ProcessWrapper（IP 段）macos.rs:172-176 |
| fs | PolicyAuditOnly windows.rs:166-170 | ProcessWrapper linux.rs:241-245 | ProcessWrapper macos.rs:177-181 |
| capability | PolicyAuditOnly（提权工具拦截）windows.rs:171-175 | ProcessWrapper（capsh）linux.rs:246-250 | ProcessWrapper（profile 默认拒绝）macos.rs:182-186 |

### 4.2 数据模型

| 表 | 分类 | 键与关键字段 | 代码写入 | 证据 |
|---|---|---|---|---|
| sandbox_session | T | session_id UUID 主键；status VARCHAR CHECK（Created、Running、Destroyed、Failed）；policy_snapshot JSONB NOT NULL；RLS 三列 | 每个生命周期事件一条 INSERT；destroyed_at、destroy_reason、resource_peak 不写入 | migrations/001_sandbox_session.sql:6-22；audit.rs:245-262 |
| sandbox_audit | T | audit_id BIGSERIAL；event_type CHECK（八值）；四个 JSONB 违规列；hit_violations TEXT[]；RLS 三列 | INSERT；事件类型只有 ProcessStart、ProcessExit；JSONB 违规列恒为 NULL | migrations/003_sandbox_audit.sql:5-30；audit.rs:264-283；orchestrator.rs:396、422 |
| sandbox_policy | M | policy_id UUID 主键；policy_name；四维 JSONB；version；valid_from、valid_to；UNIQUE(policy_name, version, tenant_id) | 无写入 | migrations/002_sandbox_policy.sql:5-23 |
| sandbox_capability | M | capability_id UUID 主键；syscall_name；platform、risk_level、default_action 的 CHECK | 无写入，无种子数据；能力字典仅在内存 | migrations/004_sandbox_capability.sql:5-20；policy/capability.rs:100 |

RLS 策略只比较 tenant_id 与 workspace_id，actor_id 不参与（migrations/005_rls_policies.sql:6-24）。迁移中没有 REVOKE 或 TRIGGER，append-only 只是约定。

### 4.3 接口

gRPC 服务 SandboxService（crates/sandboxd/proto/sandboxd.proto:8-20）：

| RPC | 请求字段 | 响应字段 | 实现 |
|---|---|---|---|
| CreateSession | client_id、task_id、parent_session_id、policy、timeout_sec（proto:24-30） | session_id、status（READY、FAILED）、error（proto:32-37） | server.rs:43-79 |
| RunCommand | session_id、command（cmd、env、cwd、capture_output）、timeout_sec（proto:78-89） | returncode、stdout、stderr、latency_ms、hit_violations（proto:91-97） | server.rs:82-115 |
| DestroySession | session_id、reason（proto:99-102） | status（DESTROYED、NOT_FOUND）（proto:104-107） | server.rs:118-144 |
| StreamLogs | session_id、from_offset（proto:109-112） | stream LogEntry（proto:114-130） | server.rs:146-168 |

HTTP 端点（监听 127.0.0.1:50052，config.rs:169-170、306-308）：GET /healthz（health.rs:37-62）；GET /metrics（health.rs:65-84）。

Rust 客户端（crates/sandboxd/src/client.rs）：SandboxdClient 的 is_available（:122）、create_session（:127）、run_command（:160）、destroy_session（:191）、stream_logs（:217）、run_with_failover（:243）。

Python：run_with_sandbox(cmd, *, timeout, cwd, limits)（scripts/automation/guardian/sandbox.py:133-141）；make_default_sandbox_limits（sandbox.py:193）；dispatcher.invoke 的 use_sandbox 与 sandbox_limits 参数（dispatcher.py:197-198）。

agent 工具：ToolCall::RunCmd { cmd, timeout_secs } 调用 sandbox::run（crates/agent-bridge/src/tool.rs:125-132；sandbox.rs:117）。

REST 门面（挂载于 /api/v1，lib.rs:137、182）：GET /api/v1/argo/nodes（lib.rs:221；routes/integrations.rs:141-164）；GET /api/v1/sandtree/resources（lib.rs:222-225；integrations.rs:173-179）。两个端点返回常量；crates/star-api-rest/tests/production_router.rs:98-99 只检查挂载。

配置：运行时读取的字段为 server.grpc_port、http_port、max_concurrent_sessions、default_session_timeout_sec、grace_shutdown_sec、observability.audit_log_file、observability.audit_log_pg_url 与 actor.*（bin/sandboxd.rs:77-109；config.rs:157-163）。isolation.*、failover.*、observability.prometheus_enabled 与 audit_log_retention_days 在 src 内无读取点（已 grep）。

### 4.4 状态机与关键流程

Session 状态（orchestrator.rs:44-60）：

| 迁移 | 设计（DD §4.1） | 代码 | 证据 |
|---|---|---|---|
| 创建后为 Created | 是 | ✅ | orchestrator.rs:334-352（status 为 Created，:340） |
| Created → Running（首个 RunCommand） | 是 | ✅ | orchestrator.rs:379-390 |
| Running → Destroyed（DestroySession 或超时） | 是 | ✅ | orchestrator.rs:440-486；550-579 |
| Created → Failed（策略应用失败） | 是 | 🟡 失败时 session 不入池，只写 Failed 审计并返回错误 | orchestrator.rs:311-333 |
| Running → Failed | 是 | ⬜ 无赋值点 | orchestrator.rs:53、382（仅枚举与匹配） |
| Destroyed、Failed 为终态 | 是 | ✅ | orchestrator.rs:58-60 |

关键流程：

- CreateSession：停机检查（orchestrator.rs:266-270）→ 策略校验（:272）→ 容量检查（:274-282）→ 写 Created 审计，失败即拒绝（:292-309）→ backend.apply（:311）→ 失败时写 Failed 审计并返回错误（:315-331）→ 入池并计数（:334-358）。
- RunCommand：状态迁移（:379-390）→ ProcessStart 审计，错误忽略（:392-403）与日志（:404-408）→ backend.run（:411）→ 违规分类为指标（:416）→ ProcessExit 审计（:418-429）与日志（:430-435）。
- Windows 后端 run 顺序：提权检查（backend/windows.rs:189-191）→ cwd 审计（:193-197）→ spawn 后加入 Job（backend/windows.rs:95-120；backend/mod.rs:316-351）→ 等待或超时 kill（backend/mod.rs:255-297）。
- DestroySession：出池（orchestrator.rs:445-448）→ 写状态与时间（:449-457）→ backend.destroy（:459）→ 指标（:461-470）→ Destroyed 审计，错误忽略（:472-483）。
- 超时扫描：每 30 秒（orchestrator.rs:41、554）销毁 created_at 距今超过 timeout_sec 的 session，原因为 session_timeout（:560-577）。扫描任务由 bin/sandboxd.rs:111 启动。
- 降级：run_with_failover 在连接、CreateSession、RunCommand 任一步失败时调用 run_fallback（client.rs:250-324）。CreateSession 固定使用 SessionSpec::default()（client.rs:267），不使用调用方策略。RunCommand 失败分支不调用 DestroySession（client.rs:298-324）。降级结果的 hit_violations 带 failover: 前缀（client.rs:398）。降级程序以 `python sandbox.py <cmd>` 形式启动（client.rs:361-363）。
- 停机：shutting_down 置位（orchestrator.rs:582-583）→ 等待在飞命令至宽限期结束（:588-594）→ 销毁剩余 session，错误只记录（:595-600）。server.rs:219-223 在 gRPC 退出后调用。

### 4.5 权限与安全

- 无 unsafe：crates/sandboxd/src/lib.rs:37 `#![forbid(unsafe_code)]`。
- gRPC 无 TLS 与鉴权，绑定 127.0.0.1（config.rs:300-303；server.rs 无拦截器）。本机任一能连接 127.0.0.1:50051 的进程可以创建 session 并执行命令。
- 环境变量：审计与日志只记录 KEY（backend/mod.rs:57-63；orchestrator.rs:407），值只注入子进程（backend/mod.rs:230-234）。
- 提权拦截按可执行文件基名匹配 sudo、runas、psexec、gsudo、doas、su、setcap、getcap、regini（backend/mod.rs:357-369），命中返回 CapabilityViolation，gRPC 映射为 PERMISSION_DENIED（error.rs:159-162）。匹配是名称匹配，不是内核约束。
- RLS：写入前以 set_config 注入 app.tenant_id、app.workspace_id、app.actor_id（audit.rs:231-242）；策略只比较前两者（migrations/005_rls_policies.sql:6-24）。
- fail-closed 只覆盖创建路径（orchestrator.rs:292-309）与启动路径（bin/sandboxd.rs:89-93）；运行与销毁路径忽略审计错误（orchestrator.rs:403、429、483）。
- agent deny-list 为大小写不敏感子串匹配（sandbox.rs:110-115）；等价写法不在覆盖范围内。
- macOS seatbelt profile 把路径直接插入文本，未做转义（backend/macos.rs:70-72、80-83）。
- /sandbox 页面文案称 Agent、插件与任务卡 CLI 拿不到宿主 Docker 或虚拟机控制权（page.tsx:390-393）。代码中没有对应的授权执行路径，该陈述 ❓。

### 4.6 依赖关系

| 依赖 | 用途 | 证据 |
|---|---|---|
| tonic 0.12、prost 0.13 | gRPC | crates/sandboxd/Cargo.toml:34-35 |
| axum（workspace） | /healthz、/metrics | crates/sandboxd/Cargo.toml:38 |
| prometheus 0.14 | 指标 | crates/sandboxd/Cargo.toml:42 |
| sqlx（workspace） | PG 审计 | crates/sandboxd/Cargo.toml:56 |
| dashmap 6、ipnet 2 | session 池、CIDR 判定 | crates/sandboxd/Cargo.toml:50、53 |
| serde-saphyr 2.1 | YAML 配置解析 | crates/sandboxd/Cargo.toml:47 |
| win32job（仅 Windows） | Job Objects | crates/sandboxd/Cargo.toml:59-60 |
| argo-core、sandtree-core | 页面门面的常量来源；argo 的 SUPPORTED 被读取，sandtree 的 ResourceGraph 未被路由使用 | routes/integrations.rs:143、172-175；argo-core/src/model.rs:11 |
| scripts/automation/guardian/sandbox.py | 降级程序（无命令行入口） | client.rs:47-53、361-363 |
| star-telemetry、star-pg-adapter、star-eventbus | D-9 复用 | ⬜ 未依赖 |
| star-context（ActorContext） | D-7 与 RLS 三元组 | ⬜ 未依赖；actor 来自配置（config.rs:143-164） |
| 其他 crate | 集成点 | ⬜ 无 crate 依赖 sandboxd |

### 4.7 页面设计

#### 4.7.1 /sandbox

**文件**：frontend/src/app/(app)/sandbox/page.tsx（默认导出 SandboxPage，:86）。

**布局**：

| 区域 | 提供方 | 证据 |
|---|---|---|
| 顶栏 | AppHeader，包在 Suspense 中 | frontend/src/components/AppShell.tsx:46-50 |
| 侧栏（应用级） | AppShell 不渲染应用侧栏；页面内左侧为资源树 aside，宽 w-72 | page.tsx:127 |
| 主区 | AppShell main；/sandbox 在全高前缀列表中，走 fullBleed 样式 | AppShell.tsx:57-73；frontend/src/app/(app)/layout.tsx:28-31、45-47 |
| 右侧栏 | RightChatRail，全局挂载 | AppShell.tsx:77 |
| 底栏 | AppShell 不渲染底栏，本页无底栏 | AppShell.tsx:31-79 |

**组件树**：

- SandboxPage（page.tsx:86）
  - aside data-testid=sandbox-resource-tree（:127-190）：新建沙盒按钮（:129）、接入实体机按钮（:132）、已接入列表（:138-152）、RESOURCES 三组列表（:153-179）、观测说明（:181-188）
  - 接入实体机对话框（:193-245）；完成提示（:246-250）
  - main（:253-396）
    - 沙盒头部 section data-testid=sandbox-detail（:255-278）：标题（:261）、四个操作按钮（:268-277）
    - 标签栏（:281-296）
    - Worktree 表（:300-330）
    - 开设 Worktree 表单（:333-369）
    - 操作任务列表（:372-387）
    - 安全说明（:390-393）

**状态矩阵**：

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 页面加载 | 常量资源树、Worktree 表、操作任务 | 🟡 数据为常量（page.tsx:48-84） |
| 加载 | 门面请求进行中 | 无加载指示；观测说明行在 backendInfo 为 null 时不显示级别 | 🟡（page.tsx:88-107、182） |
| 空 | 无已接入实体机 | 已接入列表不渲染 | ✅（page.tsx:138） |
| 错误 | 门面请求失败或返回非 2xx | 保留常量数据，无错误提示 | 🟡（page.tsx:99-105，catch 中无动作） |
| 未授权 | 无权限用户 | 无鉴权检查 | ⬜ |
| 未接入 | 后端未接入 | 接入对话框提示"当前登记本地记录"（page.tsx:235-237）；其他操作无未接入态 | 🟡 |
| 禁用 | 操作不可用 | 只有提交按钮在主机为空时禁用（page.tsx:241）；其他操作无禁用态 | ⬜ |

**交互**：

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 新建沙盒（:129） | 无 | 无处理器 | 无 | ⬜ |
| 接入实体机（:132） | 无 | 打开对话框 | 本地 state（:110） | ✅ |
| 取消或点击遮罩（:194、240） | 对话框打开 | 关闭 | 本地 state | ✅ |
| 接入（:241） | 主机地址非空 | 追加到已接入列表，状态为"验证中"，4 秒后隐藏提示 | 本地 state（:115-122） | 🟡 不写后端；"验证中"不会转换 |
| 标签切换（:286） | 无 | 切换标签样式 | 本地 state（:108） | 🟡 内容固定为 Worktree（:299） |
| 启动、停止、快照、销毁（:274） | 无 | 无处理器 | 无 | ⬜ |
| 登录、同步（:323-324） | 无 | 无处理器 | 无 | ⬜ |
| 开设 Worktree（:366） | 无 | 无处理器；代码进入方式单选只设置 defaultChecked（:358） | 无 | ⬜ |

**数据**：

| 来源 | 字段 | 用途 | 证据 |
|---|---|---|---|
| 页面常量 | RESOURCES（实体机、虚拟机、Docker 三组）、WORKTREES、OP_TASKS | 资源树、Worktree 表、操作任务 | page.tsx:48-84 |
| REST /api/v1/argo/nodes | supported_cli → argoCli | 保存在状态中，未渲染 | page.tsx:89、95、102 |
| REST /api/v1/sandtree/resources | observation_levels | 观测说明行 | page.tsx:96、103、182-186；integrations.rs:176 |
| 本地 state | attachForm、attachedHosts | 接入表单与登记列表，刷新后丢失 | page.tsx:111-113、117-118 |
| 无 | 沙盒详情字段 | 标题与副标题为常量 | page.tsx:261-262 |

**与文档设计的差距**：

- FR-SB-001（SRS-SANDBOX-001 §2）：资源树为常量，观测级别只来自门面返回的常量数组（FR-13-44）。
- FR-SB-002：详情缺类型、接入方式、网络出站字段，这三项在 SRS-SANDBOX-001 §1 与原型中都有（FR-13-45）。
- FR-SB-003：新建沙盒没有动作；接入表单只做本地登记（FR-13-46）。
- FR-SB-004：门面调用存在，但返回的是演示常量（FR-13-47）。
- FR-SB-005：页面无心跳与租约内容（FR-13-48）。
- SRS-SANDBOX-001 §1 与原型的"已安装 Agent CLI"：页面无对应内容（FR-13-49）。
- 原型标签"概览、终端、Docker、观测"：只有 Worktree 内容（FR-13-51）。
- 操作按钮均无处理器（FR-13-50）。
- 导航描述"会话列表 / 运行命令（sandboxd gRPC）"（registry.ts:600）与页面内容不符。i18n 中的 sandbox 字典块（zh-CN.ts:669-679）描述的是 sandboxd 控制台，其键在 frontend/src 中没有引用（grep 无命中）。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 守护进程入口 | crates/sandboxd/src/bin/sandboxd.rs | ✅ | :49-115 |
| gRPC 服务 | crates/sandboxd/src/server.rs | ✅ | :43-169；serve :178-225 |
| Orchestrator | crates/sandboxd/src/orchestrator.rs | 🟡 | 生命周期与日志 ✅；Failed 不被赋值（:53）；运行与销毁审计错误被丢弃（:403、429、483） |
| 配置 | crates/sandboxd/src/config.rs | 🟡 | 解析与校验 ✅（:250-298）；isolation、failover 及多数 observability 字段无读取点 |
| 策略：资源 | crates/sandboxd/src/policy/resource.rs | ✅ | :16-38、57-65 |
| 策略：网络 | crates/sandboxd/src/policy/network.rs | 🟡 | 决策 :101-142，无生产调用 |
| 策略：文件系统 | crates/sandboxd/src/policy/fs.rs | 🟡 | 决策 :118-140、glob；运行路径只审计 cwd（backend/mod.rs:376-387） |
| 策略：能力 | crates/sandboxd/src/policy/capability.rs | 🟡 | 字典 :100（52 项）；校验 :480；无持久化 |
| 策略：5 域模板 | crates/sandboxd/src/policy/mod.rs:49-91 | 🟡 | 无调用点 |
| 后端公共逻辑 | crates/sandboxd/src/backend/mod.rs | ✅ | 提权拦截 :353-374；超时 kill :255-297；cwd 审计 :376-387 |
| 后端：Windows | crates/sandboxd/src/backend/windows.rs | 🟡 | Job Objects ✅（:49-65、95-120、208-219）；网络、文件系统、能力仅审计（:161-175） |
| 后端：Linux | crates/sandboxd/src/backend/linux.rs | 🟡 | cgroups（:313-326）、netns（:329-356）、mount ns（:99-112）、capsh（:67-93）；未在 Linux 验证 |
| 后端：macOS | crates/sandboxd/src/backend/macos.rs | 🟡 | profile 纯函数（:30-93）；运行（:193-218）；未在 macOS 验证 |
| 审计 JSONL | crates/sandboxd/src/audit.rs | ✅ | :155-192；违规结构列恒为 NULL（:133-137，见 §6 #16） |
| 审计 PG | crates/sandboxd/src/audit.rs | 🟡 | RLS 注入 ✅（:231-242）；生命周期 INSERT 与主键冲突疑点（❓，§6 #4） |
| 客户端与降级 | crates/sandboxd/src/client.rs | 🟡 | Rust 客户端 ✅（:111-237）；降级程序无入口（§6 #1） |
| 指标 | crates/sandboxd/src/metrics.rs | ✅ | 8 个族（:57-112） |
| 健康检查 | crates/sandboxd/src/health.rs | ✅ | :37-84 |
| 错误映射 | crates/sandboxd/src/error.rs | ✅ | :147-173 |
| 协议与生成 | crates/sandboxd/proto/sandboxd.proto；crates/sandboxd/src/lib.rs | ✅ | 服务 proto:8-20；生成 lib.rs:52-58 |
| 数据库迁移 | crates/sandboxd/migrations/001-005 | 🟡 | 表与 RLS 存在；sandbox_policy 与 sandbox_capability 无写入 |
| 页面 | frontend/src/app/(app)/sandbox/page.tsx | 🟡 | 常量数据；操作无处理器（:129、274、322-324、366） |
| 页面门面 | crates/star-api-rest/src/routes/integrations.rs:141-179；lib.rs:221-225 | 🟡 | 返回演示常量 |
| 导航项 | frontend/src/lib/nav/registry.ts:593-602；navStore.ts:94 | 🟡 | 描述与页面不符（registry.ts:600）；分析组与默认顶栏标签中存在 |
| Python 包装器 | scripts/automation/guardian/sandbox.py | 🟡 | Windows Job 限额（:72-113）；无命令行入口 |
| Python dispatcher 集成 | scripts/automation/dispatcher.py:67-77、197-198、302-305 | 🟡 | 使用包装器；无 gRPC |
| agent 命令 deny-list | crates/agent-bridge/src/sandbox.rs | 🟡 | deny-list :94-115；超时分支不终止子进程（:175-184） |
| agent RunCmd 工具 | crates/agent-bridge/src/tool.rs:125-132 | ✅ | 调用 sandbox::run，默认超时 60 秒（:127） |
| Rust 调用方 | 其他 crate | ⬜ | 无 crate 依赖 sandboxd |

## 6. 差距与后续

| # | 差距 | 影响 | 闭合它的下一步 |
|---|---|---|---|
| 1 | 降级程序 sandbox.py 无命令行入口；client 把 Python 进程的返回码当作命令结果（client.rs:340-407） | 守护进程不可用时，请求的命令不执行，调用方得到空输出与返回码（FR-13-26 不成立） | 为 sandbox.py 增加入口并执行 argv；或在 run_fallback 中直接执行命令；测试改为调用真实入口 |
| 2 | 降级测试以 echo 替代降级程序（tests/integration/test_failover_v02_subprocess.rs:21-30） | 测试通过不证明降级路径可用 | 用真实入口替换 echo 断言 |
| 3 | 随仓库的配置样例 config/sandboxd.yaml 使用小写枚举值（:20、31、40），与 serde 变体名不一致（network.rs:19-23、fs.rs:21-35、capability.rs:18-27 均无 rename）；加载失败时 load_or_default 回退默认值（config.rs:262-270） | 样例被静默忽略（❓，需用 serde-saphyr 实际解析确认）；README 的启动命令（README.md:14）因此无效 | 统一枚举写法（或加 serde rename）；启动时记录解析错误；增加样例解析测试 |
| 4 | sandbox_session 的 session_id 为主键（migrations/001:7），Created 与 Destroyed、Failed 事件以同一 session_id INSERT（audit.rs:245-262；orchestrator.rs:292-302、472-482）；销毁与失败的错误被丢弃（orchestrator.rs:483） | 销毁与失败事件可能不落库（❓，需 PG 实测） | 改为事件表或 UPDATE；在已迁移的 PG 上运行 test_pg_audit_persistence，并为 Destroyed 增加断言 |
| 5 | 运行、销毁、Failed 审计错误被丢弃（orchestrator.rs:326、403、429、483） | 违反 NFR-A-2，审计缺口不可见 | 上浮错误，至少记录 error 日志并计数 |
| 6 | 网络维运行时无决策：check_host 与 check_ip 无生产调用；NetworkViolation 无构造点（error.rs:38） | 默认 allowlist 的域名在 Linux 上不生效（linux.rs:57）；Windows 与 macOS 无域名控制 | 域名解析为 IP 后写入 OUTPUT 规则，或在 RunCommand 前检查命令中的域名；Windows 接 WFP 或将强制力标注为 PolicyAuditOnly 并在运行路径调用决策函数 |
| 7 | 文件系统维运行时只审计 cwd（backend/mod.rs:376-387） | 子代理可访问 allowlist 之外的路径（FR-13-17 不成立） | 检查 argv 与环境中的路径；Windows 接 AppContainer；Linux mount ns 的覆盖范围需在 root 环境验证 |
| 8 | Linux 与 macOS 后端未在目标平台验证；Linux 强制力依赖 root，失败时静默降级（linux.rs:313-326、329-356），声明的 ProcessWrapper 不随之变化（linux.rs:229-252） | 两平台的实际强制力不可判定 | 在对应平台运行 test_backend_linux、test_backend_macos 与集成测试；降级时写审计，并让 EnforcementReport 反映实际级别 |
| 9 | Windows 资源维：内存为 best-effort（windows.rs:130-139）；CPU 仅映射优先级类（:73-79）；无进程数限制（:49-65） | FR-3.1 进程数限制不成立；CPU 无硬上限 | 补充 ActiveProcessLimit；CPU 硬上限单独立项 |
| 10 | macOS seatbelt profile 的路径未转义（macos.rs:70-72、80-83） | 含引号或特殊字符的路径会破坏 profile（❓，需验证） | 对路径做 seatbelt 字符串转义 |
| 11 | 降级路径中 RunCommand 失败不销毁 session（client.rs:298-324） | session 占用池容量直至超时 | 失败分支调用 DestroySession |
| 12 | Session 的 Failed 状态从不作为池内状态存在（orchestrator.rs:53、382） | 失败 session 不可观察；与 DD §4.1 不一致 | 保留失败记录并置 Failed，或修订 DD 状态图 |
| 13 | 配置字段多数无读取点：isolation.*、failover.*、observability.prometheus_enabled、audit_log_retention_days（config.rs:23-141） | 运维修改这些字段无效果 | 把默认值接入缺省策略与降级开关；或从配置结构移除未使用字段 |
| 14 | 降级通知 root session（NFR-O-3）未实现（config.rs:140 无读取点） | 降级无告警 | 接入通知通道 |
| 15 | 审计保留 90 天无清理（config.rs:127-128，只做校验） | FR-6.2 不成立 | 增加清理任务，或在文档中标注未实现 |
| 16 | 审计违规结构列恒为 NULL（audit.rs:133-137）；StreamLogs 只推送两类事件（orchestrator.rs:404-408、430-435） | 审计与日志缺少违规细节 | 在命中时填充结构化列与对应事件类型 |
| 17 | gRPC 无鉴权、无 TLS（config.rs:300-303；server.rs 无拦截器） | 本机任一进程可执行命令 | 增加本机 token，或改用 UDS 配合文件权限 |
| 18 | sandbox_policy 与 sandbox_capability 无写入、无种子；能力字典只在内存（capability.rs:100） | 数据模型与代码不一致 | 增加种子与写入；或声明内存字典为权威 |
| 19 | RLS 仅比较 tenant_id 与 workspace_id（migrations/005:6-24）；无 REVOKE 或 TRIGGER 禁止 UPDATE、DELETE | 与守门 #13 的 RLS 13 类与 append-only 要求不一致 | 补齐 actor 维度策略；以权限或触发器禁止更新与删除 |
| 20 | dispatcher.py、mavis desktop、star-mcp 未接入 gRPC；仓库内无 crate 依赖 sandboxd | 跨 client 复用未实现（BR-1） | 增加 Python gRPC 客户端与 Rust 客户端接入点 |
| 21 | 页面操作按钮无处理器，资源数据为常量，导航描述与页面不符（page.tsx:129、274、322-324、366；registry.ts:600） | 用户以为能操作，实际不能 | 接入 CreateSession 与 DestroySession；或将按钮设为禁用并标注未接入；修正导航描述 |
| 22 | 部署单元（systemd、Windows Service、launchd）与实施报告 PHASE-SANDBOX-002-IMPL-REPORT.md 在仓库中不存在 | 无法按 FR-1.1 常驻运行；交付无记录 | 增加单元文件与安装说明；完成实施报告 |
| 23 | 性能基准与 UAT 缺失（crates/sandboxd 无 benches；TEST-DESIGN-SANDBOX-002 §5、§6） | NFR-P-1 至 NFR-P-5 未验证 | 按 TEST-DESIGN §5.2 增加 benches |
| 24 | 超时扫描的测试未经过 sweeper（test_orchestrator.rs:310-318 手动调用 destroy） | 超时扫描无直接测试 | 为 sweeper 增加可注入时钟的测试 |
| 25 | agent deny-list 为子串匹配；超时分支不终止子进程（agent-bridge sandbox.rs:110-115、175-184；无 kill_on_drop） | 可被等价写法绕过；超时命令残留 | 增加进程组终止与超时清理；明确 deny-list 与 sandboxd 的职责边界 |
| 26 | 文档与代码的数值偏差：SRS 写 9 个指标，代码 8 族（metrics.rs:57-112；模块注释 :8 亦写 9）；SRS 写 HTTP 为 50051，代码为 50052（config.rs:169-170、44-48）；默认网络 allowlist SRS 写 5 项，代码 7 项（network.rs:46-54） | 文档与实现不一致 | 以代码为准修订文档数值 |
| 27 | 多租户、TLS 与 mTLS、K8s 为 v0.1 范围外 | 跨主机不可用 | 保持范围外，列入后续版本 |

## 7. 验收与测试

### 7.1 现有测试

crates/sandboxd/Cargo.toml 注册 15 个 [[test]] 目标，另有 client.rs 内联测试。下表按 `#[test]` 与 `#[tokio::test]` 属性计数。

| 目录 / 文件 | 测试数 | 平台门控 | 覆盖内容 | 备注 |
|---|---|---|---|---|
| tests/unit/test_policy_resource.rs | 8 | 无 | 资源默认值与校验 | |
| tests/unit/test_policy_network.rs | 9 | 无 | 域名与 CIDR 判定 | 判定函数的测试；运行路径无调用 |
| tests/unit/test_policy_fs.rs | 10 | 无 | glob 与路径判定 | |
| tests/unit/test_policy_capability.rs | 8 | 无 | 字典与校验 | |
| tests/unit/test_orchestrator.rs | 10 | 无 | 生命周期、容量、审计失败时创建被拒（:295-300） | 使用 mock 后端；超时用例手动调用 destroy（:310-318） |
| tests/unit/test_metrics.rs | 4 | 无 | 指标注册 | |
| tests/unit/test_audit.rs | 5 | 无 | JSONL 与记录构造 | |
| tests/unit/test_config.rs | 8 | 无 | YAML 加载与校验 | 无测试加载 config/sandboxd.yaml |
| tests/unit/test_backend_linux.rs | 6 | `#![cfg(target_os = "linux")]`（:10） | cgroup 文本、netns 脚本、capsh 前缀 | |
| tests/unit/test_backend_macos.rs | 4 | `#![cfg(target_os = "macos")]`（:10） | sandbox-exec profile 文本 | |
| tests/integration/test_grpc_create_run_destroy.rs | 8 | 分支 cfg（windows 与非 windows） | gRPC 生命周期 | |
| tests/integration/test_failover_v02_subprocess.rs | 4 | 分支 cfg | 降级路径 | 以 echo 替代降级程序（:21-30） |
| tests/integration/test_pg_audit_persistence.rs | 4 | 需 DATABASE_URL（:8-9、:14-19），否则跳过 | PG 审计 | 成功路径只输出日志，不做断言（:50-58） |
| tests/integration/test_backend_smoke.rs | 6 | 分支 cfg | 后端冒烟 | |
| tests/e2e/test_dispatcher_integration.rs | 4 | 分支 cfg | 进程内守护进程，执行 git --version（:21-24、:103） | |
| src/client.rs（内联） | 3 | 无 | 降级规格解析 | |
| 合计 | 101 | | | |

Python：scripts/automation/guardian/tests/test_sandbox.py 含 10 个测试函数，覆盖可用性判定、默认与自定义限额、不可用时回退 subprocess.run，以及超时与 Access Denied 时的级联 kill。

前端：没有 /sandbox 页面的专属测试。frontend/src/components/__tests__/AppHeader.test.tsx:68 只断言顶栏存在 sandbox 标签。

### 7.2 已覆盖的领域（代码存在，通过状态 ❓）

- 策略判定与校验：tests/unit/test_policy_*.rs 覆盖资源、网络、文件系统、能力四维。
- session 生命周期与容量：tests/unit/test_orchestrator.rs。
- 审计失败时创建被拒绝：tests/unit/test_orchestrator.rs:295-300。
- gRPC 生命周期：tests/integration/test_grpc_create_run_destroy.rs。
- Python 包装器行为：scripts/automation/guardian/tests/test_sandbox.py。

通过状态需执行 `cargo test -p sandboxd -j 4` 与 pytest 才能确认。本文件未执行测试。

### 7.3 未验证

- 降级的真实执行路径（现有测试以 echo 替代降级程序）。
- PG 生命周期事件的真实行为与主键冲突（需 DATABASE_URL 与已迁移的数据库）。
- Linux cgroup、netns、mount ns、capsh 的真实运行（需 root）。
- macOS sandbox-exec 的实际强制力。
- Windows 上网络与文件系统维的强制效果（当前无强制）。
- 性能目标 NFR-P-1 至 NFR-P-5（无基准）。
- 超时扫描本身（sweeper）的行为。
- /sandbox 页面操作与接入流程（无测试）。
- Windows 特权不足时内存上限的实际效果。
- 配置样例的解析结果（§6 #3）。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| docs/requirements/SRS-SANDBOX-001.md | /sandbox 页面需求 FR-SB-001 至 005；观测四级；实体机接入；数据源说明（§1、§2） | FR-SB-005 心跳与租约代码中不存在；页面未渲染类型、接入方式、网络出站字段；文档称"浏览器实测闭环"，在本基线无法核实；SRS 未标优先级 | 页面需求采用；状态以代码判定；实测记录 ❓ |
| docs/requirements/SRS-SANDBOX-002.md | FR-1 至 FR-8；BR-1 至 BR-5；NFR；AC-1 至 AC-8；D-1 至 D-10；W/T/M 数据模型 | 指标写 9 个，代码 8 族（metrics.rs:57-112）；HTTP 端口写 50051，代码 50052（config.rs:169-170）；RLS 写 13 类，迁移为三个 UUID 列（005:6-24）；默认 allowlist 写 5 项，代码 7 项（network.rs:46-54）；审计保留与部署单元未实现 | 功能需求作为设计意图采用；数值以代码为准；数据模型以迁移文件为准 |
| docs/architecture/SANDBOX-001.md | SANDBOX-001 子进程包装器设计：SandboxLimits、Windows Job 配置、缺口 1 至 6 | 与 SRS-SANDBOX-001 同名易混淆：本文件描述 Python 子进程包装器，不是 /sandbox 页面；测试数文字写 9，实际为 10 | 作为降级层设计采用 |
| docs/architecture/SANDBOX-002.md | 拓扑；三平台四维后端选型；gRPC 详设；fail-open 时序；部署形态表；G-SBX-01 至 09；D-1 至 D-10 | 部署单元文件不存在；D-7 与 D-9 未实装 | 作为目标架构；状态以代码为准 |
| docs/test-design/TEST-DESIGN-SANDBOX-002.md | 五级测试设计（UT 56、IT 17、E2E 11、PT 5、UAT 8）与 AC 对应 | 代码测试属性 101 个；无 benches；无 UAT 文件；TC 编号与测试函数名无映射 | 测试设计作为目标；验收状态按代码标注 ⬜ 或 🟡 |
| docs/basic-design/SANDBOX-BASIC-DESIGN-002.md | 章节结构：模块划分、Backend 抽象、gRPC 契约、表结构、部署与监控（仅读取章节标题） | 具体条目未逐条核对 | 章节结构作为设计意图；条目以代码为准 |
| docs/detailed-design/DD-SANDBOX-002.md | §4.1 状态图；§5.1 UT 矩阵；§6 DDL；§1.1 文件数 | 状态图含 Running → Failed 与 Created → Failed；代码仅在池外 apply 失败时写 Failed 审计；§1.1 写 24 个文件，实际 src 20 个、tests 15 个 | 状态机以代码为准 |
| docs/implementation-plans/SANDBOX-IMPL-PLAN-002.md | 5 阶段 16 子项与风险矩阵（章节标题与阶段表） | 计划未与代码状态对齐；实施报告不存在 | 作为后续规划依据（§6） |
| docs/guardian/v37_sandbox_guard.md | Python 软约束的五维声明；与 sandboxd 的关系（文件头） | 网络与文件系统维仅声明（sandbox.py:14-19） | 作为 Python 层设计来源 |
| docs/WBS.md 第 90 至 92 行 | 6.5.1 标为已完成（Python 软约束）；6.5.2 标为进行中，写 91 个测试、90% | 代码为 101 个测试属性；6.5.2 百分比未核实 | 进度以代码与测试为准 |
| docs/prototypes/2c · 沙盒-html/README.md | 原型为参考，不是实现 | 无 | 仅作视觉参考 |
| docs/prototypes/2c · 沙盒-html/Sandbox.dc.html | 详情字段（类型、接入方式、网络出站）、已安装 Agent CLI、Worker 心跳与任务租约文案（经定向检索读取） | 其余布局与样式未逐行读取；多数元素未进入代码 | 仅作设计与文案参考 |
| docs/prototypes/INDEX.md 第 18 行 | 2c 原型与 SRS-SANDBOX-001、页面、sandtree-core、argo-core 的映射 | 无 | 采用映射关系 |
| docs/architecture/2026-11-14-ops-console/REUSE-MAP-COLLAB-SANDBOX-001.md §3 与许可行 | 沙盒页面与 SandTree、Argo 功能映射（资源树、观测、心跳与租约、CLI 适配） | 许可行对 Argo 写 AGPL-3.0 并注"同上"（与 Apache 同列）；argo-core 移植头注释记录权利人以 MIT 复用的决定（argo-core/src/lib.rs:1-6）；该决定的独立记录本文未核实 | 功能映射采用；许可条目 ❓ |
| crates/sandboxd/README.md | 四个 RPC；三平台后端表；fail-open 与 fail-closed 描述；数据模型摘要 | README 称降级经 run_with_failover 执行，但降级程序无入口；启动命令引用 config/sandboxd.yaml（:14），该文件无法解析（§6 #3） | 结构采用；降级与配置条目以代码为准 |
| crates/sandboxd/Cargo.toml、config/sandboxd.yaml、config/policy_default.json、proto/sandboxd.proto、migrations/001 至 005、src 全部模块、tests 全部文件 | 实现与契约 | policy_default.json 与 Rust 默认值一致（network.rs:46-54、fs.rs:74-78）；sandboxd.yaml 枚举写法与 serde 变体名不一致 | 按代码判定状态 |
| crates/star-api-rest/src/routes/integrations.rs；crates/star-api-rest/src/lib.rs（路由 :137、:182、:221-225）；crates/star-api-rest/tests/production_router.rs:98-99 | argo 与 sandtree 门面端点及挂载检查 | 门面返回演示常量；测试只检查挂载 | 标 🟡 |
| crates/sandtree-core/src/lib.rs；crates/argo-core/src/lib.rs；crates/argo-core/src/model.rs | sandtree 的 ResourceGraph 模块导出；argo 的 SUPPORTED 常量 | 门面未使用 ResourceGraph；只读取了 SUPPORTED | 作为门面数据来源说明 |
| crates/agent-bridge/src/sandbox.rs；crates/agent-bridge/src/tool.rs（RunCmd 分支） | 命令 deny-list 与执行 | 与 sandboxd 无关联；超时分支不终止子进程 | 作为独立执行层记录 |
| scripts/automation/guardian/sandbox.py；scripts/automation/guardian/tests/test_sandbox.py（函数计数 10）；scripts/automation/dispatcher.py（导入保护、默认 use_sandbox、invoke 段） | Python 包装器实现、集成与测试 | 无命令行入口；dispatcher 无 gRPC | 按代码判定状态 |
| frontend/src/app/(app)/sandbox/page.tsx；frontend/src/app/(app)/layout.tsx；frontend/src/components/AppShell.tsx | 页面结构、常量数据、门面调用、接入表单、布局区域 | 操作按钮无处理器 | 按代码判定状态；布局区域按 AppShell 判定 |
| frontend/src/lib/nav/registry.ts（593-602、738、755-783）；frontend/src/lib/nav/navStore.ts:94；frontend/src/lib/i18n/zh-CN.ts:669-679（en、ja 同键） | 导航项、分析组、默认顶栏标签、sandbox 字典块 | 描述与页面不符（registry.ts:600）；字典键在 frontend/src 中无引用 | 标注差异 |
| frontend/src/components/__tests__/AppHeader.test.tsx（:64-86）；Sidebar.test.tsx（:43） | 顶栏包含沙盒标签的断言 | 只断言导航存在，不涉及页面行为 | 作为导航测试引用 |
| docs/runtime-design.md | 未读取 | 运行时构建沙箱属另一设计层，不在本功能范围 | 不纳入结论 |
| docs/reports/*、docs/briefs/*、docs/reviews/*、docs/ci-cd.md | 仅关键字命中，未读取正文 | 与本功能的直接依赖未核对 | 不作为证据 |
| scripts/automation/registry.md、aggregate_v02.py、wbs_section15_sync.py | 仅关键字命中 | 状态表的自述不作为证据 | 不作为证据 |
| frontend/src/lib/agents/agentsApi.ts、frontend/src/app/(app)/agents/page.tsx、agents/__tests__/agents.test.tsx、(app)/__tests__/panels.test.tsx | argo 端点关键字命中 | 属 Agent 中心页面，不是本功能页面；仅作为 /api/v1/argo/nodes 的使用方引用 | 不纳入本功能结论 |
| frontend/src/app/(app)/lane/[laneId]/settings/page.tsx（:61、:94、:347） | Lane 路径类型中的"沙盒路径"选项 | 不调用 sandboxd 或 /sandbox 页面；未读取其数据流 | 不纳入本功能结论 |
| crates/nema-codec/NEMA_INTERPRETER_PLUGIN_CONTRACT.md（:53） | 关键字命中，将 sandboxd 描述为独立隔离服务 | 仅为外部描述 | 不作依据 |
