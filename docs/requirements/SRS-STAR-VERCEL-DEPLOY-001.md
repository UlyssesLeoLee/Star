# SRS-STAR-VERCEL-DEPLOY-001

> **STAR Vercel 部署形态软件需求规格说明书 v0.1**
>
> - 状态: Requirements Baseline (Draft — 需求定义完成, 0 实现)
> - 目标阶段: 需求定义 → 基本设计 (分阶段交付: Phase 1 前端 / Phase 2 Rust 控制面)
> - 核心语言: TypeScript (前端) + Rust (后端)
> - 核心运行时: Next.js 16 App Router (前端) + Vercel Rust Runtime Beta (后端, Phase 2)
> - 范围拍板: 2026-10-24 用户拍板 — **① 分阶段** (先前端上 Vercel, 再把 Rust 控制面搬上去) + 落档本 SRS 正式文档
> - 上游: `AGENTS.md` (守门 #1-#17 + 守门派生 v1-v26) / `docs/requirements/SRS-STAR-CLI-001.md` v1.0 (SRS 模板) / `docs/requirements/SRS-STAR-OPS-001.md` v0.1 (IPA 章节映射范式) / `docs/deployment/BFF-DEPLOYMENT-001.md` v0.1 (既有 BFF 部署资产) / `docs/automation-design.md` v2.64 (守门 #19 适用范围判定)
> - 下游: 基本设计 `BD-STAR-VERCEL-DEPLOY-001.md` (待开) + 详设 + Phase 1 / Phase 2 实施任务卡
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核**
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-10-24

---

## §0 文档目的

本文档定义 STAR 项目（`D:/Star`）**部署到 Vercel** 的软件需求规格，覆盖 (a) 部署平面切分与范围边界 (b) 平台能力基线 (c) 可执行需求条目 FR-VC-001..FR-VC-020 (d) 非功能需求 NFR-VC-01..NFR-VC-08 (e) 降级与 fail-closed 矩阵 (f) 验收标准 AC-VC-001..AC-VC-012 (g) 已知缺口与待验证项。

**触发**（per 2026-10-24 用户提问「如果我都想把这个项目部署在 vercel 你认为修改用的要件定义是什么样的？」+ 拍板 `ask_36f7d8462d9773f241c35533`）：范围 = ① 分阶段，落档 = `docs/requirements/SRS-STAR-VERCEL-DEPLOY-001.md`。

**本文件的核心命题**：Star **不能整体部署到 Vercel**。可行形态是把 Star 拆成 **控制面**（前端 + 无状态 API + 定时触发入口）上 Vercel、**执行面 / 有状态面**（Agent sandbox、终端 PTY / noVNC、Canvas 实时 WSS、outbox worker、Multipass / K3s、SQLite WAL 任务队列）留在自有运行时。需求定义的重点因此不是"改多少代码"，而是**逐能力裁定归属平面**，并对不能上 Vercel 的能力**显式 fail closed**（per AGENTS.md 守门 #11 缺标比错标安全 + 「Agent/Task CLI 闭环验收」硬约束：仅有 UI / mock / 预览适配器不得标为可用闭环）。

**证据基线**（本文件所有仓库事实均为 2026-10-24 当日实测，平台事实检索日期同为 2026-10-24）：

| 类别 | 事实 | 证据 |
|---|---|---|
| 仓库 | 前端 = Next.js 16 App Router + React 19 + TypeScript 7.0.2 | `frontend/package.json:39-42, 68` |
| 仓库 | 全仓 **0 个** `vercel.json`（本项目为绿地接入） | `glob **/vercel.json` → No files matched |
| 仓库 | Next.js API route 仅 2 个：`app/api/group/[...path]/route.ts`（同源 BFF 代理）+ `app/automation-debug/api/scripts/route.ts` | `frontend/src/app` 递归 `route.ts` 计数 = 2 |
| 仓库 | 前端直连后端走 **build-time** 变量 `NEXT_PUBLIC_BFF_URL` / `NEXT_PUBLIC_API_BASE_URL` / `NEXT_PUBLIC_USE_REAL_API` | `frontend/src/components/chat/useChatStream.ts:5`、`frontend/src/components/chat/ChatPanel.tsx:34`、`frontend/src/components/agent/AgentRunner.tsx:10`、`frontend/src/lib/store.ts:1645` |
| 仓库 | 上述 build-time 陷阱已在仓内被显式记录（runtime env 注入会静默跑 mock） | `docs/reports/STAR-API-REST-BACKEND-TAKEOVER-WBS-001.md:99` |
| 仓库 | 浏览器侧长连接 4 类：terminal WS / remote file browser WS / ARG WS / chat SSE | `frontend/src/lib/terminal/wsClient.ts:138`、`frontend/src/components/remote/FileBrowser.tsx:110`、`frontend/src/lib/arg/ws.ts:144`、`frontend/src/components/chat/useChatStream.ts:22` |
| 仓库 | Rust workspace ~130 crate，含 5 个常驻 axum 进程（bff/canvas-engine `:8080`、domain-canvas `:8081`、canvas-realtime `:8082` + WSS `:8083`、canvas-game `:8084`） | `Cargo.toml:3-175`（workspace members + 各 crate 注释） |
| 仓库 | `crates/star-api-rest` 已含 `routes/`、`group_api/`（engineering_runs / canvas / hook_policies / schedule_rules / execution_profiles 等模块）、`auth/oauth/`（keypair + PKCE）、`middleware/`、`rbac.rs` | `crates/star-api-rest/src` 目录实测 |
| 仓库 | PG 迁移 93 个文件 | `db/migrations` 文件计数 = 93 |
| 仓库 | Rust 工具链固定 1.99.0（2026-10-01 发布），components = rustfmt + clippy | `rust-toolchain.toml:4-6` + AGENTS.md「Rust toolchain baseline」 |
| 仓库 | 既有部署资产全为 K8s / Helm / GitOps / Multipass 轨 | `deploy/k3s-local/`、`deploy/helm/`、`deploy/gitops/`、`deploy/multipass/` |
| 平台 | Vercel Rust Runtime = 官方 Beta, 全套餐, Fluid compute | vercel.com/docs/functions/runtimes/rust（2026-10-24 检索） |
| 平台 | Rust runtime 需 `api/` 下每文件一 function + Cargo.toml `[[bin]]` 逐个声明 + `vercel_runtime` v2 (`service_fn`) | 同上 |
| 平台 | 函数时长: Hobby 上限 300s; Pro/Ent 默认 300s / 上限 800s / 1800s beta **仅 Node/Bun/Python** | vercel.com/docs/functions/limitations、/functions/configuring-functions/duration |
| 平台 | 其他限额: 载荷 4.5MB / 函数包 250MB（Rust 无 large-function 资格）/ 内存 Pro·Ent 4GB·2vCPU / fd 1024 共享 / 默认单区 iad1 | vercel.com/docs/functions/limitations |
| 平台 | WebSocket = public beta (2026-06-22 起), 全套餐, 需 Fluid Compute; 连接在时长上限处断开; 跨实例广播需 Redis | vercel.com/kb/guide/publish-and-subscribe-to-realtime-data-on-vercel |
| 平台 | Cron: 每项目 100 个; Hobby 每日一次且精度 ±59min; Pro/Ent 每分钟一次、分钟级精度 | vercel.com/docs/cron/usage-and-pricing |
| 平台 | Vercel Postgres 已于 2025-06 关停并全量迁 Neon, 新库走 Marketplace | 第三方报道（kuberns.com，2026-10-24 检索）—⛔ 需以 Neon / Marketplace 官方页复核，见 §8 GAP-06 |

**不做**（本 SRS 范围外）：

- 详细设计（DD）— 随 Phase 1 实施同步开
- 既有 K8s / Helm / GitOps / Multipass 轨的替换或下线 — 双轨并存（per AGENTS.md「本地/CI 多 K3s 集群虚拟机管理约束」不受本 SRS 影响）
- 执行面能力上 Vercel（见 §3.2 明确排除清单）
- 任何数据库表结构变更 — 本 SRS 不新增表（守门 #13 W/T/M 分类不变, RLS 13 类必携）
- 前端功能裁剪 — 本 SRS 不删除任何现有页面 / 能力, 只改部署与调用链

---

## §1 章节映射

需求章节用 IPA 模板 + 状态列：`✅ 已实测` / `⬜ 待实施` / `⛔ 待验证` / `❌ 范围外`。

| # | 章节 | 状态 | 实证 / 待 |
|---|---|---|---|
| §2 | 业务目标与平台约束基线 | ✅ | 2026-10-24 实测 + Vercel 官方文档检索 |
| §3 | 范围与边界（平面切分 / 两阶段 / 排除清单） | ✅ | 用户拍板 ① 分阶段 |
| §4 | FR 列表（FR-VC-001..020） | ⬜ | 0 实现, 全部待 Phase 1/2 落地 |
| §5 | NFR（NFR-VC-01..08） | ⬜ | 数值已由平台限额反推定档 |
| §6 | 降级与 fail-closed 矩阵 | ⬜ | 随 Phase 1 实施 |
| §7 | 验收（AC-VC-001..012） | ⬜ | 全部未开始 |
| §8 | 已知缺口与待验证项（GAP-01..09） | ⛔ | 9 项待验证, 不得当已实现 |
| §9 | 风险与依赖（R-01..R-08） | ✅ | 风险识别完成 |
| §10 | 角色与责任 + 签字栏 | ✅ | Mavis 接手审核 |

---

## §2 业务目标与平台约束基线

### 2.1 目标

| # | 目标 | 可度量表述 |
|---|---|---|
| G-1 | 全球可达 | 前端静态 / SSR 资源由 Vercel CDN 边缘分发, 消除当前「前端未容器化、仅本地 `npm run dev` 可用」的限制（per `docs/reports/STAR-API-REST-BACKEND-TAKEOVER-WBS-001.md:25`） |
| G-2 | 控制面弹性 | Phase 2 后, 无状态控制面 API 随流量自动扩缩, 不再依赖 k3s 节点预留 |
| G-3 | 契约正确 | 消除 build-time env 静默降级为 mock 的线上风险（per 同上 `:99`） |
| G-4 | 边界诚实 | 任何不能在 Vercel 运行的能力, 在 UI 上明确拒绝并说明原因, 不提供假闭环 |
| G-5 | 双轨不互伤 | Vercel 轨与既有 K8s/Helm/GitOps 轨并存, 既有资产 0 删除、0 改写 |

### 2.2 平台约束基线（Vercel, 2026-10-24）

| 维度 | 约束 | 对 Star 的直接影响 |
|---|---|---|
| 执行模型 | 无状态函数, 实例可回收, 无常驻进程 | 5 个常驻 axum 进程 + 常驻 worker 不可原样搬迁 |
| 函数时长 | Rust 最高 800s（Pro/Ent）, Hobby 300s | 禁在函数内做长轮询 / 长流式等待 |
| 函数包 | 250MB 未压缩, Rust 无 large-function 资格 | 130 crate 链接产物体积 ⛔ 未实测（GAP-01） |
| 载荷 | 4.5MB 请求 / 响应 | 大文件、大日志不得经函数体过境 |
| 文件描述符 | 1024 跨并发实例共享 | DB / HTTP 连接池上限必须显式收敛 |
| 内存 | Hobby 2GB·1vCPU; Pro·Ent 4GB·2vCPU | 禁进程内大缓存 |
| 区域 | 默认单区 iad1, Pro/Ent 可多区 | 函数区必须与 PG 数据驻留区一致 |
| WebSocket | public beta, 需 Fluid Compute, 达时长上限即断 | 终端 / 实时协同长连接不迁 Vercel（GAP-02） |
| Cron | Hobby 每日一次 ±59min; Pro/Ent 分钟级 | 无 SLA 级调度; 精度不满足 Run 级定时语义 |
| 构建 | Rust runtime 走 Cargo 构建 + `[[bin]]` 逐函数声明 | 需为 Vercel 准备独立构建入口（FR-VC-002） |
| 工具链 | 构建镜像 Rust 版本未公开承诺 | 本仓固定 1.99.0（`rust-toolchain.toml:4`）能否在 Vercel 构建镜像内编译 ⛔ 待验证（GAP-03） |

---

## §3 范围与边界

### 3.1 平面切分（本 SRS 的架构前提）

| 平面 | 定义 | Star 现资产 | 归属 |
|---|---|---|---|
| **P-呈现面** | 页面渲染、静态资源、同源代理 | `frontend/`（Next.js 16） | **Vercel**（Phase 1） |
| **P-控制面** | 无状态读 / 写 API, 授权判定, outbox 写入 | `crates/star-api-rest`（`routes/` + `group_api/` + `auth/oauth/` + `rbac.rs`） | **Vercel（Phase 2）** |
| **P-执行面** | Agent 执行 / 沙箱 / 原生 Hook 执行 / 终端 PTY / 实时协同 WS / 常驻 worker / 基础设施管理 | `crates/domain-agent*`、`crates/domain-hook`、`crates/multipass-adapter`、`crates/star-taskqueue`（SQLite WAL）、`crates/bi-owner-runtime`、`crates/platform-messaging-runtime`、`crates/canvas-realtime`（WSS）、`crates/terminal-stack` | **自有运行时（不变）** |
| **P-数据面** | PG + RLS 13 类 + 迁移 | `db/migrations`（93 文件）、`crates/star-pg-adapter`、`crates/infrastructure` | 托管 PG + 连接池（Phase 2） |

### 3.2 排除清单（明确不上 Vercel）

| 能力 | 排除理由 | 行为要求 |
|---|---|---|
| Agent 执行 / 沙箱 / 工具调用 | 需真实进程与宿主能力 | fail closed + UI 明示（FR-VC-015） |
| 终端 PTY / noVNC | 长连接 + PTY 会话态 | 留在执行面, 前端直连执行面（FR-VC-009） |
| Canvas 实时协同 WSS | 长连接 + CRDT 同步态 | 留在执行面（FR-VC-009） |
| 原生 Hook 执行 | 安全关键, 必须 Rust 核心 fail-closed | 留在执行面（FR-VC-015） |
| Multipass / K3s 基础设施管理 | 宿主级能力, 与 AGENTS.md 硬约束绑定 | 留在执行面（FR-VC-015） |
| SQLite WAL 任务队列 | 本地文件态 | 留在执行面（FR-VC-003） |
| 常驻 outbox / inbox worker | 需长驻进程 + 租约 / 重试 | 留在执行面（FR-VC-011） |

### 3.3 阶段划分

| 阶段 | 交付物 | 出口条件 |
|---|---|---|
| **Phase 1** | 前端上 Vercel + 同源 BFF 代理 + build-time env 契约重定义 + CI 构建门 | AC-VC-001..006 全绿 |
| **Phase 2** | Rust 控制面上 Vercel Rust Runtime + 无状态化 + 托管 PG / 连接池 / RLS 事务级兼容 | AC-VC-007..012 全绿 + GAP-01/02/03/04 关闭 |

Phase 1 的「同源 BFF 改造」是 Phase 2 的前置件, 两阶段复用同一套 `/api/*` 代理骨架, 不做重复工。

---

## §4 FR 列表

阶段标注: `P1` = Phase 1 交付; `P2` = Phase 2 交付。状态统一 `⬜ 待实施`（本 SRS 0 实现）。

### 4.1 拓扑与工程（FR-VC-001..004）

| ID | 需求 | 阶段 | 验收方式 | 状态 |
|---|---|---|---|---|
| FR-VC-001 | 部署单元切分: 呈现面 = 1 个 Vercel Project (Next.js preset); 控制面 = 1 个独立 Vercel Project (framework preset `Other` + Rust runtime); 执行面 = 既有 k3s/VPS 不变。同一 Vercel Project 不得混用 Next.js 与 Rust 两种 framework preset | P1 | 拓扑表落档 + 2 个 Project 各自可独立部署与独立回滚 | ⬜ |
| FR-VC-002 | 准备 Vercel Rust 构建入口: 在独立构建根下提供 `Cargo.toml`（含 `[[bin]]` 逐函数声明, 唯一名）+ `api/*.rs`（每文件一函数）+ `vercel_runtime` v2 `service_fn` 适配层。**只允许新增 adapter 层, 不得复制 `crates/domain-*` 业务逻辑** | P2 | 每个控制面 endpoint 一 `[[bin]]`; `cargo test -p star-api-rest --lib -j 4` 0 err（守门 #1 v25 单 crate 口径） | ⬜ |
| FR-VC-003 | 控制面无状态化硬约束: 禁止在函数内实例化 InMemory repo / 进程内 EventBus / `star-taskqueue` / 进程内 cache; 启动自检发现即 fail closed | P2 | 启动日志证明 0 个 in-memory 实例; 负向用例: 注入 in-memory 实现 → 拒绝启动 | ⬜ |
| FR-VC-004 | Rust 构建产物治理: `[profile.release]` 关闭 debuginfo / strip 符号; `.vercelignore` 排除 `target/**` 仅保留 release 二进制; 禁在二进制内 embed 大静态资源（monaco / three 走 dynamic import 或 CDN） | P2 | 函数包实测 ≤ 250MB（GAP-01 关闭依据） | ⬜ |

### 4.2 前端与调用链（FR-VC-005..008）

| ID | 需求 | 阶段 | 验收方式 | 状态 |
|---|---|---|---|---|
| FR-VC-005 | **build-time env 契约重定义**: `NEXT_PUBLIC_BFF_URL` / `NEXT_PUBLIC_API_BASE_URL` / `NEXT_PUBLIC_USE_REAL_API` 必须在 Vercel Project Env 显式声明（Build + Production 两套作用域均需存在）, 缺任一即构建失败而非静默回落 mock | P1 | 构建日志可见变量已注入; 缺变量构建 **必须失败**（负向用例） | ⬜ |
| FR-VC-006 | 后端地址改 **同源代理**: 客户端不得直连后端主机名; 所有后端调用经 `frontend/src/app/api/*` 同源路由; 后端地址仅作为 **服务端** 环境变量（`STAR_API_BASE_URL` 形态）注入, 不进客户端 bundle | P1 | 构建产物 grep 无后端主机名 / 无 `NEXT_PUBLIC_*` 承载的后端地址 | ⬜ |
| FR-VC-007 | 统一同源 BFF 骨架: 在现有 `frontend/src/app/api/group/[...path]/route.ts` 模式上扩展为通用 `/api/*` 代理, 必须保留既有安全约束（per `docs/design/DD-WORKTREE-GROUP-001.md:121`）: 拒绝非法 method / path / cookie 名 / 上游 URL、生产 upstream 强制 HTTPS、写操作校验同源 `Origin`、请求体上限 2,000,000 bytes、禁缓存、禁跟随 redirect、剥离 hop-by-hop 与 `Set-Cookie` 响应头 | P1 | 代理单测覆盖上述约束; 非法 method / path / cookie 名 / 非 HTTPS upstream 均被拒 | ⬜ |
| FR-VC-008 | 服务端凭证不外泄: access token 仅存 HttpOnly cookie, 由代理服务端读取并向后端加 `Bearer`; token 不得进入浏览器 JS / URL / 日志 / 终端 | P1 | 前端 bundle 无 token 字面量; 服务端日志无 token 值 | ⬜ |

### 4.3 数据面（FR-VC-009..012）

| ID | 需求 | 阶段 | 验收方式 | 状态 |
|---|---|---|---|---|
| FR-VC-009 | 托管 PG 接入: 走 Vercel Marketplace 提供的托管 PostgreSQL（或等价自托管 PG + 私网可达）, 凭证经平台 Env 注入; **Vercel Postgres 本身已关停**, 不得作为依赖目标 | P2 | 连接建立 + 迁移后 schema 一致 | ⬜ |
| FR-VC-010 | **RLS 与连接池兼容（关键项）**: 租户上下文必须以 `SET LOCAL`（事务级）注入, 禁 `SET`（会话级）; 连接池模式下必须事务包裹「SET LOCAL → 业务语句 → COMMIT」, 防跨租户串号 | P2 | 双租户并发写入实测互不可见 + 连接复用后无串号 | ⬜ |
| FR-VC-011 | 迁移不跑在函数内: 禁在 function / 请求路径内执行 DDL（无 DDL 权限 + 禁长事务 + 禁函数内多步迁移）; 迁移由独立 CI 作业 + 外部 migrator 执行 | P2 | 函数产物 grep 0 DDL 语句; CI 迁移作业独立可跑 | ⬜ |
| FR-VC-012 | 连接预算: 单函数 DB 连接池上限显式配置, 与平台 fd 1024 共享上限 + 托管 PG 连接上限对齐; 禁每请求新建连接 | P2 | 压测无 "too many open files" / 无连接耗尽 | ⬜ |

### 4.4 长连接 / 定时 / 后台（FR-VC-013..015）

| ID | 需求 | 阶段 | 验收方式 | 状态 |
|---|---|---|---|---|
| FR-VC-013 | 长连接归属裁决: terminal WS / PTY / noVNC / Canvas 实时 WSS / ARG WS **判定留在执行面**（前端直连执行面地址或经自有边缘代理）; chat SSE 允许经 Next 路由透传; Vercel 侧**不得**代理 PTY 或 CRDT 会话 | P1 | 拓扑表逐条标注归属平面; 4 类长连接在 Vercel 形态下实测仍可用 | ⬜ |
| FR-VC-014 | 定时语义: Vercel Cron 仅作「唤醒入口」, **不承载 SLA 级调度**（Hobby 每日一次 ±59min 精度不满足 Run 级定时）; 每次 trigger / retry / resume 必须按 Schedule Rule 固化的 `run_as_actor_id` 实时复验当前 Project / Run 权限, 撤权即 fail closed | P1 | 撤权后 trigger 被拒且有审计记录 | ⬜ |
| FR-VC-015 | 后台 worker 不进 Vercel: outbox / inbox 消费端、`bi-owner-runtime`、`platform-messaging-runtime`、`star-dispatcher` 常驻 worker 留在执行面; Vercel 侧只写 owner-local Outbox, 由 owner 侧消费; 禁止在函数内实现租约 / 重试 / 取消语义 | P2 | 事务提交后 Outbox 行被 owner 侧消费; 函数侧 0 worker 循环 | ⬜ |

### 4.5 安全 / 身份 / 供应链 / 可观测（FR-VC-016..020）

| ID | 需求 | 阶段 | 验收方式 | 状态 |
|---|---|---|---|---|
| FR-VC-016 | 本地能力显式禁用: Agent 执行 / 沙箱 / 原生 Hook 执行 / Multipass / 本地 worktree 文件系统在 Vercel 形态下必须 fail closed, 且 UI 明示「该能力需要本地运行时」; 禁以 mock / 预览适配器冒充闭环（per AGENTS.md「Agent/Task CLI 闭环验收」） | P1 | 负向用例: 点「启动执行」→ 明确拒绝 + 原因, 无静默假成功 | ⬜ |
| FR-VC-017 | 授权不得依赖出口 IP: 平台无固定 egress IP（无静态 IP 套餐时）, 仓库根 `ip-list.json` 类 IP 白名单策略在 Vercel 形态下**失效**; 访问控制必须叠加 token / OIDC 校验 | P1 | 无凭据访问被拒; 白名单策略在 Vercel 形态下被显式标注失效 | ⬜ |
| FR-VC-018 | 密钥治理: 平台 Env + 现有 sealed-secrets 等价机制; 禁 secret 进 `NEXT_PUBLIC_*`; 遵守守门 #5（不 dump env value, 不 `cat .env`） | P1 | 产物 / 日志扫描 0 credential | ⬜ |
| FR-VC-019 | 供应链: 出 SPDX / SBOM（Rust + Node 双链）+ 商业许可逐项审查 + 保留各许可证版权 / NOTICE / 专利声明（per AGENTS.md「商业开源依赖硬约束」）；新增依赖须允许不限用途商业使用 | P1 | SBOM 产物存在; 未决许可项 = 0 | ⬜ |
| FR-VC-020 | 可观测与成本: Prometheus exporter 形态不适用于函数 → 改 OTLP 外送; 平台按 Active CPU + Provisioned Memory 计费, 需设成本告警阈值 | P1 | trace 可查 + 成本告警可触发 | ⬜ |

---

## §5 NFR

| ID | 指标 | 目标值 | 依据 |
|---|---|---|---|
| NFR-VC-01 | 控制面请求延迟 | p95 < 2s; 硬上限 300s (Hobby) / 800s (Pro) | 平台函数时长限额; 禁函数内长轮询 |
| NFR-VC-02 | 载荷上限 | ≤ 4.5MB; 大文件 / 大日志不经函数体 | 平台载荷限额 |
| NFR-VC-03 | 函数包体积 | ≤ 250MB (未压缩) | 平台函数包限额; Rust 无 large-function 资格 |
| NFR-VC-04 | 内存占用 | ≤ 4GB / 2vCPU (Pro) | 平台内存限额; 禁进程内大缓存 |
| NFR-VC-05 | 区域 / 数据驻留 | 函数区域与 PG 区域一致（默认 iad1 需与托管 PG region 对齐） | 平台单区默认 + 数据驻留要求 |
| NFR-VC-06 | 冷启动 | 首字节延迟 SLO **实测后定档**, 不得预设 | 130 crate 链接产物 + 函数冷启动叠加, 当前无实测 |
| NFR-VC-07 | 调度精度 | 不承载 SLA 级调度; Run 级定时精度需求由执行面保证 | Cron 精度限制 |
| NFR-VC-08 | 兼容基线 | Rust MSRV 1.99.0 + edition 2021 保持不变; workspace lint（`unsafe_code = "forbid"` 等）0 放宽 | `rust-toolchain.toml` + `Cargo.toml:178-182` |

---

## §6 降级与 fail-closed 矩阵

| 场景 | 期望行为 | 禁止行为 | 关联 FR |
|---|---|---|---|
| Vercel 形态下请求 Agent 执行 | 明确拒绝 + 指出「需本地运行时」 | 静默返回 mock 成功 | FR-VC-016 |
| 平台不可用 / 函数超时 | 明确错误态 + 可重试语义 | 伪装成功 / 半写状态 | FR-VC-003 |
| build-time env 缺失 | **构建失败** | 静默回落 mock 模式 | FR-VC-005 |
| 连接池下 RLS 租户上下文 | 事务级 `SET LOCAL` | 会话级 `SET`（会串号） | FR-VC-010 |
| Schedule 触发时权限已撤销 | fail closed + 审计记录 | 继续执行 | FR-VC-014 |
| 执行面离线 | 呈现「执行面不可达」 | 假装执行成功 / 回填假结果 | FR-VC-015, FR-VC-016 |
| 迁移在函数内被触发 | 拒绝 | 函数内执行 DDL | FR-VC-011 |

---

## §7 验收

| AC | 描述 | 测试方法 | 阶段 | 状态 |
|---|---|---|---|---|
| AC-VC-001 | 2 个 Vercel Project 各自可独立构建 + 部署 + 回滚 | `vercel build` / `vercel deploy --prebuilt` 实测 | P1 | ⬜ |
| AC-VC-002 | 客户端 bundle 无后端主机名、无后端地址 `NEXT_PUBLIC_*` | 构建产物 grep | P1 | ⬜ |
| AC-VC-003 | 同源 BFF 6 项安全约束单测全绿 + 4 类负向用例被拒 | vitest（`frontend/package.json:13` 现有 vitest 口径） | P1 | ⬜ |
| AC-VC-004 | 缺 build-time env 时构建**失败**（负向） | CI 实测 | P1 | ⬜ |
| AC-VC-005 | 端到端 smoke: 登录 → 建 Project → 建 Run → 建 Worktree → 拉 Canvas → 发一条 chat (SSE) → 触发生命周期 Hook, 逐条标注「Vercel 侧 / 执行面侧」 | 浏览器实跑 + 逐条留证 | P1 | ⬜ |
| AC-VC-006 | 本地能力在 Vercel 形态下 fail closed, UI 有明示文案 | 负向实跑截图 | P1 | ⬜ |
| AC-VC-007 | 控制面 endpoint 全部具备 `[[bin]]` + `api/*.rs` 入口, 业务逻辑 0 复制 | `cargo test -p star-api-rest --lib -j 4` 0 err + 代码审查 | P2 | ⬜ |
| AC-VC-008 | 启动自检 0 in-memory 实例（负向: 注入即拒启动） | 启动自检实测 | P2 | ⬜ |
| AC-VC-009 | 双租户并发写入互不可见 + 连接复用无串号 | 并发实测 + RLS 断言 | P2 | ⬜ |
| AC-VC-010 | 迁移独立作业成功, 函数侧 0 DDL | CI 作业日志 + 产物 grep | P2 | ⬜ |
| AC-VC-011 | 函数包实测体积 ≤ 250MB; 冷启动延迟实测值回写 §5 NFR-VC-06 | 部署实测 + 回写文档 | P2 | ⬜ |
| AC-VC-012 | Outbox 跨平面链路闭环: Vercel 侧写 → owner 侧消费 → 回执 | 端到端实测 | P2 | ⬜ |

---

## §8 已知缺口与待验证项

| ID | 缺口 | 影响 | 验证方式 | 状态 |
|---|---|---|---|---|
| GAP-01 | 130 crate 链接后的 Rust 函数包真实体积未实测 | 若 > 250MB 则 Phase 2 需拆包或改走构建产物裁剪 | 构建后量测 `.vercel/output/functions/*` 体积 | ⛔ |
| GAP-02 | Vercel Rust Runtime 下 WebSocket 支持情况未确认（官方文档 feature support 表为空） | 若不支持, FR-VC-013 的 SSE 透传方案需改 | 以官方文档 / 最小工程实测为准 | ⛔ |
| GAP-03 | Vercel 构建镜像的 Rust 版本能否满足 MSRV 1.99.0 | 决定 Phase 2 可行性 | 最小 Rust 工程部署实测 | ⛔ |
| GAP-04 | 托管 PG 在事务级池化下与 RLS 13 类 + `SET LOCAL` 的兼容性 | FR-VC-010 的实现细节未定 | spike: 池化 + 双租户并发实测 | ⛔ |
| GAP-05 | Next.js 16 + TypeScript 7.0.2 + monaco/three 在平台构建镜像的兼容性与构建时长 | 构建超时 / 失败 | 首次构建实测 | ⛔ |
| GAP-06 | 「Vercel Postgres 已关停并迁 Neon」目前仅第三方来源, 未以 Neon / Marketplace 官方页复核 | 影响 FR-VC-009 的目标选型表述 | 官方页复核 | ⛔ |
| GAP-07 | 类型门禁为 advisory 且平台构建跳过类型检查: CI 已跑 `npx tsc --noEmit` 但 `continue-on-error: true`（ci.yml:143-148, 且注明 4 个 pre-existing 错误）, 同时 `frontend/next.config.js:38` 的 `ignoreBuildErrors: true` 使平台构建也不拦类型错误 → 两道门都不阻塞, 类型缺陷可带入生产 | 类型缺陷进入生产的风险 | 先清零 4 个 pre-existing 错误, 再把 CI typecheck 由 advisory 改回 blocking, 并移除 `ignoreBuildErrors` | ⛔ |
| GAP-08 | `frontend/src/app/automation-debug/api/scripts/route.ts` 依赖本机 Python `scripts/automation/console_server.py`（:8080） | 该调试页在 Vercel 形态下不可用 | 明确标注为本地专用调试面, 不进生产路由 | ⛔ |
| GAP-09 | 平台 WS beta 的行为变化 / Rust runtime 仍在 Beta | 接口与限额可能变 | 锁版本 + 季度复核 | ⛔ |

---

## §9 风险与依赖

| ID | 类别 | 描述 | 缓解 |
|---|---|---|---|
| R-01 | 范围失控 | 「整体上 Vercel」的诉求含不可迁移能力 | §3.2 排除清单为硬边界; FR-VC-016 强制 fail closed |
| R-02 | 契约静默降级 | build-time env 漏配 → 线上跑 mock 无告警 | FR-VC-005 负向验收（构建必须失败） |
| R-03 | 数据串号 | 托管 PG 池化 + 会话级 `SET` → 跨租户泄漏 | FR-VC-010 事务级 `SET LOCAL` + AC-VC-009 双租户并发实测 |
| R-04 | 性能 | 冷启动 + 130 crate 链接 → 首字节劣化 | NFR-VC-06 实测后定档; Phase 1 已提供独立价值, 不阻塞 |
| R-05 | 排期 | Phase 2 依赖 GAP-01/03/04 关闭 | 分阶段交付, Phase 2 开工前先做 spike |
| R-06 | 双轨冲突 | Vercel 轨与既有 K8s/Helm/GitOps 轨并存易漂移 | 既有资产 0 改写; 拓扑表单一事实源 |
| R-07 | 安全 | IP 白名单策略在平台形态失效 | FR-VC-017 叠加 token / OIDC |
| R-08 | 治理 | 平台 Beta 变动导致实现返工 | 锁版本 + GAP-09 季度复核 |

**依赖**:

- `crates/star-api-rest`（Phase 2 控制面主体）— 需补 Vercel adapter 层, 不改业务语义
- `crates/domain-permission` / `crates/star-api-rest/src/auth/oauth/`（授权判定留在 Rust）
- `crates/star-pg-adapter` / `crates/infrastructure`（PG 访问层, 需支持 `SET LOCAL` 语义）
- `db/migrations`（93 文件, 迁移执行器化）
- 执行面既有资产（`deploy/k3s-local/` + `deploy/k3s-local/envoy/deployment.yaml` 独立 envoy deployment 形态, 非 istio sidecar — per 用户 2026-09-01 13:03/13:05 JST 边缘层选型偏好）
- 平台账号与套餐（Hobby 仅 300s 函数 + 每日 cron; Phase 2 与 SLA 级定时需 Pro）

---

## §10 角色与责任

| 角色 | 职责 | 状态 |
|---|---|---|
| 架构师 | 平面切分裁决、Phase 2 adapter 边界审核 | 架构师 (Mavis 接手 agent per DEC-008) |
| SRE Lead | 容量 / 成本护栏 / 观测与告警验收 | SRE Lead（Mavis 接手审核） |
| 平台 | 部署工程、CI 构建门、迁移作业编排 | 平台（Mavis 接手审核） |
| 评审主持 | FR / NFR / AC 完备性评审 | 评审主持（Mavis 接手审核） |
| PM | 阶段范围与拍板记录 | PM（Mavis 接手审核） |

5 域 Lead 真人到位后按 AGENTS.md 守门 #14 v4 追溯补签（修订历史追加 1 行, 不改写既有行, per 守门 #1 禁回溯叙事）。

---

## 附录 A 实施位置（预估, 详细设计阶段细化）

| 路径 | 变更性质 | 阶段 |
|---|---|---|
| `frontend/vercel.json`（新建） | 呈现面 Vercel 配置（regions / headers / rewrites） | P1 |
| `frontend/src/app/api/[...path]/route.ts`（新建, 仿现有 `group/[...path]/route.ts`） | 通用同源代理 | P1 |
| `frontend/src/components/{chat,agent,composer}/*` + `frontend/src/lib/store.ts` | 后端地址改同源相对路径 | P1 |
| `frontend/next.config.js` | 评估移除 `ignoreBuildErrors`（与 GAP-07 联动） | P1 |
| `.github/workflows/ci.yml` | 增 Vercel 构建 + `tsc --noEmit` 独立门 | P1 |
| `vercel-rust/`（新建构建根） | `Cargo.toml` + `api/*.rs` + `vercel.json` | P2 |
| `crates/star-api-rest/src/vercel/`（新建 adapter） | `vercel_runtime` v2 `service_fn` 适配层, 0 业务逻辑 | P2 |
| `crates/star-pg-adapter` / `crates/infrastructure` | 事务级 `SET LOCAL` 语义 + 连接池上限配置 | P2 |

---

## 附录 B 平台事实引用（检索日期 2026-10-24）

| 事实 | 来源 |
|---|---|
| Rust Runtime 官方 Beta / 全套餐 / Fluid compute | https://vercel.com/docs/functions/runtimes/rust |
| Rust entrypoint 形态（`api/` + `[[bin]]` + `vercel_runtime` v2 `service_fn`） | https://vercel.com/docs/functions/runtimes/rust |
| 函数时长上限与 Rust 的 maxDuration 配置位（`vercel.json` `functions`） | https://vercel.com/docs/functions/configuring-functions/duration |
| 载荷 4.5MB / 函数包 250MB / 内存 / fd 1024 / 区域 | https://vercel.com/docs/functions/limitations |
| WebSocket public beta 行为与 Redis 跨实例广播要求 | https://vercel.com/kb/guide/publish-and-subscribe-to-realtime-data-on-vercel |
| Cron 套餐差异（Hobby 每日 ±59min / Pro 分钟级）与 100 job 上限 | https://vercel.com/docs/cron/usage-and-pricing |
| Vercel Postgres 关停与 Neon 迁移（第三方, 待官方复核 — GAP-06） | https://kuberns.com/blogs/vercel-postgres-dead-what-replaced-it |

---

## 附录 C 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 初版: 10 段 IPA 章节 + 平面切分 (§3.1) + 排除清单 (§3.2) + FR-VC-001..020 (§4) + NFR-VC-01..08 (§5) + fail-closed 矩阵 (§6) + AC-VC-001..012 (§7) + GAP-01..09 (§8) + R-01..08 (§9)。范围 = ① 分阶段 (Phase 1 前端 / Phase 2 Rust 控制面)。仓库事实均当日实测, 平台事实检索日期 2026-10-24 | 用户 2026-10-24 提问「部署到 Vercel 的要件定义」+ 拍板 `ask_36f7d8462d9773f241c35533`（范围 ① 分阶段 / 落档本文件） |
