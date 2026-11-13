# SRS-STAR-VERCEL-DEPLOY-001

> **STAR Vercel 部署需求说明 v0.4**
>
> - 状态: Requirements Baseline（Draft，未批准，0 实现；v0.4 = 修正 Functions WebSocket GA 范围并同步 VCD-0 兼容性评审结论，详见附录 C）
> - 目标阶段: 需求定义 → 基本设计 (Phase 1 前端; Phase 2 Rust HTTP adapter 为可行性门后的候选)
> - 核心语言: TypeScript (前端) + Rust (后端)
> - 核心运行时: Next.js 16 App Router (前端) + 可选 Vercel Rust Runtime Beta (HTTP adapter, Phase 2 候选)
> - 范围拍板: 2026-10-24 用户拍板 — **① 分阶段** (先前端上 Vercel, 再把 Rust 控制面搬上去) + 落档本 SRS 正式文档
> - 上游: `AGENTS.md` (守门 #1-#17 + 守门派生 v1-v26) / `docs/requirements/SRS-STAR-CLI-001.md` v1.0 (SRS 模板) / `docs/requirements/SRS-STAR-OPS-001.md` v0.1 (IPA 章节映射范式) / `docs/deployment/BFF-DEPLOYMENT-001.md` v0.1 (既有 BFF 部署资产) / `docs/automation-design.md` v2.64 (守门 #19 适用范围判定)
> - 下游: 基本设计 `BD-STAR-VERCEL-DEPLOY-001.md`、详细设计与测试设计尚未在当前 dev 文档树建立；纳入总 WBS 的 VC-0 后再进入实施任务卡
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核**
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-10-24

---

## §0 文档目的

本文档定义 STAR 项目（`D:/Star`）**部署到 Vercel** 的软件需求规格，覆盖 (a) 部署平面切分与范围边界 (b) 平台能力基线 (c) 可执行需求条目 FR-VC-001..FR-VC-021 (d) 非功能需求 NFR-VC-01..NFR-VC-08 (e) 降级与 fail-closed 矩阵 (f) 验收标准 AC-VC-001..AC-VC-013 (g) 已知缺口与待验证项。

**触发**（per 2026-10-24 用户提问「如果我都想把这个项目部署在 vercel 你认为修改用的要件定义是什么样的？」+ 拍板 `ask_36f7d8462d9773f241c35533`）：范围 = ① 分阶段，落档 = `docs/requirements/SRS-STAR-VERCEL-DEPLOY-001.md`。

**本文件的核心命题**：Star 不能整体部署到 Vercel。Phase 1 评估把 Web 呈现面和受限 BFF 放在 Vercel；Phase 2 的 Rust HTTP functions 只是候选，须通过 runtime、数据库/Owner、身份与运维验收后再决定。Star 的 **执行面 / 有状态面**（Agent sandbox、终端 PTY / noVNC、Canvas 实时 WSS、Outbox worker、Multipass / K3s、SQLite WAL 任务队列）继续留在自有运行时。每项能力按 Owner 和持久事实归属平面，未通过的能力显式 fail closed；UI、mock 或预览适配器不能冒充可用闭环。

**证据基线**（本文件所有仓库事实均为 2026-10-24 当日实测，平台事实检索日期同为 2026-10-24）：

| 类别 | 事实 | 证据 |
|---|---|---|
| 仓库 | 前端 = Next.js 16 App Router + React 19 + TypeScript 7.0.2 | `frontend/package.json:39-42, 68` |
| 仓库 | 全仓 **0 个** `vercel.json`（本项目为绿地接入） | `glob **/vercel.json` → No files matched |
| 仓库 | Next.js API route 仅 2 个：`app/api/group/[...path]/route.ts`（同源 BFF 代理）+ `app/automation-debug/api/scripts/route.ts` | `frontend/src/app` 递归 `route.ts` 计数 = 2 |
| 仓库 | 前端直连后端走 **build-time** 变量 `NEXT_PUBLIC_BFF_URL` / `NEXT_PUBLIC_API_BASE_URL` / `NEXT_PUBLIC_USE_REAL_API` | `frontend/src/components/chat/useChatStream.ts:5`、`frontend/src/components/chat/ChatPanel.tsx:34`、`frontend/src/components/agent/AgentRunner.tsx:10`、`frontend/src/lib/store.ts:1645` |
| 仓库 | 上述 build-time 陷阱已在仓内被显式记录（runtime env 注入会静默跑 mock） | `docs/reports/STAR-API-REST-BACKEND-TAKEOVER-WBS-001.md:99` |
| 仓库 | 浏览器侧长连接 4 类：terminal WS / remote file browser WS / ARG WS / chat SSE | `frontend/src/lib/terminal/wsClient.ts:138`、`frontend/src/components/remote/FileBrowser.tsx:110`、`frontend/src/lib/arg/ws.ts:144`、`frontend/src/components/chat/useChatStream.ts:22` |
| 仓库 | Rust workspace members = 111 个唯一 crate（成员声明 116 条, 含 5 组重复声明: `crates/leads` / `crates/shared-task` / `crates/star-canvas` / `crates/star-scheduler` / `crates/star-workflow`）, 含 5 个常驻 axum 进程（bff/canvas-engine `:8080`、domain-canvas `:8081`、canvas-realtime `:8082` + WSS `:8083`、canvas-game `:8084`） | `Cargo.toml:3-175`（workspace members + 各 crate 注释）; 成员计数 = 成员数组解析实测 |
| 仓库 | `crates/star-api-rest` 已含 `routes/`、`group_api/`（engineering_runs / canvas / hook_policies / schedule_rules / execution_profiles 等模块）、`auth/oauth/`（keypair + PKCE）、`middleware/`、`rbac.rs` | `crates/star-api-rest/src` 目录实测 |
| 仓库 | PG 迁移 93 个文件 | `db/migrations` 文件计数 = 93 |
| 仓库 | Rust 工具链固定 1.99.1（2026-10-15 发布），components = rustfmt + clippy | `rust-toolchain.toml:4-6` + AGENTS.md「Rust toolchain baseline」 |
| 仓库 | 既有部署资产全为 K8s / Helm / GitOps / Multipass 轨 | `deploy/k3s-local/`、`deploy/helm/`、`deploy/gitops/`、`deploy/multipass/` |
| 平台 | Vercel Rust Runtime = 官方 Beta, 全套餐, Fluid compute | vercel.com/docs/functions/runtimes/rust（2026-10-24 检索） |
| 平台 | Rust runtime 需 `api/` 下每文件一 function + Cargo.toml `[[bin]]` 逐个声明 + `vercel_runtime` v2 (`service_fn`) | 同上 |
| 平台 | 通用函数计划表列 Hobby 300s、Pro/Ent 标准上限 800s；1800s Beta 明确列 Node.js/Bun/Python，Rust 专属 duration 未确认 | vercel.com/docs/limits；Rust runtime 功能矩阵见 GAP-11 |
| 平台 | 标准函数按 250MB 预算；5GB Large Functions Beta 明确限 Node.js/Python；载荷 4.5MB、fd 1024 并发共享。Rust 专属矩阵仍需实测 | vercel.com/docs/functions/limitations；vercel.com/changelog/vercel-functions-can-now-be-up-to-5-gb-in-package-size |
| 平台 | 通用 Vercel Functions WebSocket 于 2026-10-20 进入 GA；要求 Fluid Compute，单连接固定到一个 Function 实例并受 Function duration/limits 约束，跨实例持久状态须外部协调。该通用能力不证明 Rust Runtime 支持；Rust 专页仍标 Beta，feature-support 表为空且没有 WebSocket 行 | https://vercel.com/changelog/websocket-support-is-now-generally-available；https://vercel.com/docs/functions/websockets；https://vercel.com/docs/functions/runtimes/rust |
| 平台 | Cron: Hobby 每日一次、指定小时内触发；其他团队分钟级。投递 best-effort，可能漏发/重复/重叠、失败不重试；Rollback 会恢复部署对应的 Cron 定义 | vercel.com/docs/cron-jobs/manage-cron-jobs |
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
| §4 | FR 列表（FR-VC-001..021） | ⬜ | 0 实现, 全部待 Phase 1/2 落地 |
| §5 | NFR（NFR-VC-01..08） | ⬜ | 数值已由平台限额反推定档 |
| §6 | 降级与 fail-closed 矩阵 | ⬜ | 随 Phase 1 实施 |
| §7 | 验收（AC-VC-001..013） | ⬜ | 全部未开始 |
| §8 | 已知缺口与待验证项（GAP-01..11） | ⛔ | 11 项待验证, 不得当已实现 |
| §9 | 风险与依赖（R-01..R-08） | ✅ | 风险识别完成 |
| §10 | 角色与责任 + 签字栏 | ✅ | Mavis 接手审核 |

---

## §2 业务目标与平台约束基线

### 2.1 目标

| # | 目标 | 可度量表述 |
|---|---|---|
| G-1 | 全球可达 | 前端静态 / SSR 资源由 Vercel CDN 边缘分发, 消除当前「前端未容器化、仅本地 `npm run dev` 可用」的限制（per `docs/reports/STAR-API-REST-BACKEND-TAKEOVER-WBS-001.md:25`） |
| G-2 | 控制面弹性 | 若 Phase 2 通过 VCD-2 并获准上线，无状态 HTTP adapter 可按流量扩缩；Owner/持久状态/执行资源仍由 Star 权威运行时管理 |
| G-3 | 契约正确 | 消除 build-time env 静默降级为 mock 的线上风险（per 同上 `:99`） |
| G-4 | 边界诚实 | 任何不能在 Vercel 运行的能力, 在 UI 上明确拒绝并说明原因, 不提供假闭环 |
| G-5 | 双轨不互伤 | Vercel 轨与既有 K8s/Helm/GitOps 轨并存, 既有资产 0 删除、0 改写 |

### 2.2 平台约束基线（Vercel 官方资料复核, 2026-10-24）

| 维度 | 约束 | 对 Star 的直接影响 |
|---|---|---|
| 执行模型 | 无状态函数, 实例可回收, 无常驻进程 | 5 个常驻 axum 进程 + 常驻 worker 不可原样搬迁 |
| 函数时长 | 平台计划表列 Hobby 300s、Pro/Ent 默认 300s / 标准上限 800s；1800s Beta 明确列 Node.js/Bun/Python。Rust Runtime 的专属最大时长未在功能支持表中给出 | Rust endpoint 不得预设可用 800s/1800s；记录 GAP-11 并以实际 Runtime/套餐部署验证 |
| 函数包 | 标准函数限制按 250MB 规划；2026-06 Large Functions Beta 的 5GB 资格明确为 Node.js/Python，不假定 Rust 可用该扩展 | 按每个 Rust adapter 的真实依赖闭包测量；超过 250MB 即阻断 Rust 轨（GAP-01/11） |
| 载荷 | 4.5MB 请求 / 响应 | 大文件、大日志不得经函数体过境 |
| 文件描述符 | 1024 在并发执行间共享，Runtime 自身也占用 | DB / HTTP 连接池上限必须显式收敛 |
| 内存 | 平台表列 Hobby 2GB·1vCPU；Pro/Ent 默认 2GB·1vCPU，最高 4GB·2vCPU；Rust 专属资格仍需核实 | 禁进程内大缓存，并以目标 Runtime 实测 RSS/并发 |
| 区域 | 可配置执行区域；平台页面未支持本文件此前写入的固定 `iad1` 默认值 | 部署前显式选择与数据库可达性、数据驻留一致的区域 |
| WebSocket | 通用 Vercel Functions WebSocket 已于 2026-10-20 GA（全套餐）；要求 Fluid Compute，连接固定到接入实例并受 Function duration 限制，跨实例持久状态需外部协调。Rust Runtime 仍为 Beta，官方 feature-support 表为空且没有 WebSocket 行，因此 Rust 专属 WebSocket/Streaming 能力未确认 | PTY/noVNC/Canvas WSS 仍归 Star 自有执行面；不得依赖 Vercel 函数内存作为连接或协作事实（GAP-02/11） |
| Cron | Hobby 每日一次且在指定小时内触发；其他团队为分钟级。投递 best-effort，可能漏发、重复、重叠；失败不自动重试；Instant Rollback 会恢复该部署关联的 Cron 定义 | 仅作可重复唤醒/补偿入口；业务事实必须由持久 owner 状态、幂等收据与对账驱动 |
| 构建 | Rust runtime 走 Cargo 构建 + `[[bin]]` 逐函数声明, **且要求项目根为带 `[package]` 的 manifest** | 仓根是虚拟 manifest, 需先裁决构建形态（GAP-10 / FR-VC-002） |
| 工具链 | 构建镜像 Rust 版本未公开承诺 | 本仓固定 1.99.1（`rust-toolchain.toml:4`）能否在 Vercel 构建镜像内编译 ⛔ 待验证（GAP-03） |

---

## §3 范围与边界

### 3.1 平面切分（本 SRS 的架构前提）

| 平面 | 定义 | Star 现资产 | 归属 |
|---|---|---|---|
| **P-呈现面** | 页面渲染、静态资源、同源代理 | `frontend/`（Next.js 16） | **Vercel**（Phase 1） |
| **P-控制面** | 无状态 HTTP adapter；Star Owner 持有授权判定、业务写入、持久事实与 Outbox | `crates/star-api-rest`（`routes/` + `group_api/` + `auth/oauth/` + `rbac.rs`） | **Vercel Phase 2 候选**（VCD-2 全部验收通过后）；否则保留在 Star 自有 Rust 运行时 |
| **P-执行面** | Agent 执行 / 沙箱 / 原生 Hook 执行 / 终端 PTY / 实时协同 WS / 常驻 worker / 基础设施管理 | `crates/domain-agent*`、`crates/domain-hook`、`crates/multipass-adapter`、`crates/star-taskqueue`（SQLite WAL）、`crates/bi-owner-runtime`、`crates/platform-messaging-runtime`、`crates/canvas-realtime`（WSS）、`crates/terminal-stack` | **自有运行时（不变）** |
| **P-数据面** | PG + RLS 13 类 + 迁移 | `db/migrations`（93 文件）、`crates/star-pg-adapter`、`crates/infrastructure` | Star 数据 Owner 持有；托管 PostgreSQL/连接池仅在 Phase 2 评估通过后选型 |

### 3.2 排除清单（明确不上 Vercel）

| 能力 | 排除理由 | 行为要求 |
|---|---|---|
| Agent 执行 / 沙箱 / 工具调用 | 需真实进程与宿主能力 | fail closed + UI 明示（FR-VC-016） |
| 终端 PTY / noVNC | 长连接 + PTY 会话态 | 留在执行面, 前端直连执行面（FR-VC-013/016） |
| Canvas 实时协同 WSS | 长连接 + CRDT 同步态 | 留在执行面（FR-VC-013/016） |
| 原生 Hook 执行 | 安全关键, 必须 Rust 核心 fail-closed | 留在执行面（FR-VC-016） |
| Multipass / K3s 基础设施管理 | 宿主级能力, 与 AGENTS.md 硬约束绑定 | 留在执行面（FR-VC-016） |
| SQLite WAL 任务队列 | 本地文件态 | 留在执行面（FR-VC-003/016） |
| 常驻 outbox / inbox worker | 需长驻进程 + 租约 / 重试 | 留在执行面（FR-VC-011） |

### 3.3 阶段划分

| 阶段 | 交付物 | 出口条件 |
|---|---|---|
| **Phase 1** | 前端上 Vercel + 同源 BFF 代理 + build-time env 契约重定义 + CI 构建门 | AC-VC-001..006、013 全绿 |
| **Phase 2** | 仅在 Rust Runtime 可行性门通过后，才评估小型 Rust 控制面 adapter；Star Owner、持久状态、授权和执行面边界不迁移 | AC-VC-007..012 全绿 + GAP-01/02/03/04/10/11 关闭 + GAP-09 风险经正式决策接受；否则保持 Phase 1 前端部署形态 |

Phase 1 的「同源 BFF 改造」是 Phase 2 的前置件, 两阶段复用同一套 `/api/*` 代理骨架, 不做重复工。

---

## §4 FR 列表

阶段标注: `P1` = Phase 1 交付; `P2` = Phase 2 交付。状态统一 `⬜ 待实施`（本 SRS 0 实现）。

### 4.1 拓扑与工程（FR-VC-001..004）

| ID | 需求 | 阶段 | 验收方式 | 状态 |
|---|---|---|---|---|
| FR-VC-001 | 部署单元切分: Phase 1 呈现面 = 1 个 Vercel Project (Next.js preset); Rust 控制面 Project 仅为 Phase 2 候选，须通过 VCD-2 并经 Owner/DB/Auth 设计评审后才创建；执行面 = 既有 Star 自有运行时不变。同一 Vercel Project 不得混用 Next.js 与 Rust 两种 framework preset | P1 / P2 conditional | Phase 1 前端项目单独可部署/回滚；若 Phase 2 获批，再验证独立 Rust Project 的部署/回滚 | ⬜ |
| FR-VC-002 | **先裁决 Phase 2 构建形态再动手实现**（前置约束见 GAP-10: 仓根 `Cargo.toml` 为虚拟 manifest, 无 `[package]` 段, 而 Rust Runtime 要求项目根为带 `[[bin]]` 的 package manifest）: 候选 A = git integration + Root Directory 指向构建子目录 + path deps 指向 `crates/*`; 候选 B = CI 预构建二进制 + 按 Build Output API v3 产出 `.vercel/output` + `vercel deploy --prebuilt`。选定后在构建根提供 `Cargo.toml`（`[[bin]]` 逐函数声明, 唯一名）+ `api/*.rs`（每文件一函数）+ `vercel_runtime` v2 `service_fn` 适配层。**只允许新增 adapter 层, 不得复制 `crates/domain-*` 业务逻辑** | P2 | 候选 A/B 至少一条路径部署成功 + 每个控制面 endpoint 一 `[[bin]]`; `cargo test -p star-api-rest --lib -j 4` 0 err（守门 #1 v25 单 crate 口径） | ⬜ |
| FR-VC-003 | 控制面无状态化硬约束: 禁止在函数内实例化 InMemory repo / 进程内 EventBus / `star-taskqueue` / 进程内 cache; 启动自检发现即 fail closed | P2 | 启动日志证明 0 个 in-memory 实例; 负向用例: 注入 in-memory 实现 → 拒绝启动 | ⬜ |
| FR-VC-004 | Rust 构建产物治理: 沿用仓内既有 `[profile.release]`（`Cargo.toml:293-295`: `lto = "thin"` + `codegen-units = 1`）并**新增** strip 符号; 构建根不继承 workspace profile, 需自带等价段（与 Vercel 文档示例 `lto = "fat"` 的差异随 GAP-10 一并裁决）; `.vercelignore` 排除 `target/**` 仅保留 release 二进制; 禁在二进制内 embed 大静态资源（monaco / three 走 dynamic import 或 CDN） | P2 | 函数包实测 ≤ 250MB（GAP-01 关闭依据） | ⬜ |

### 4.2 前端与调用链（FR-VC-005..008）

| ID | 需求 | 阶段 | 验收方式 | 状态 |
|---|---|---|---|---|
| FR-VC-005 | **build-time env 契约重定义**: `NEXT_PUBLIC_BFF_URL` / `NEXT_PUBLIC_API_BASE_URL` / `NEXT_PUBLIC_USE_REAL_API` 必须在 Vercel Project Env 显式声明（Build + Production 两套作用域均需存在）, 缺任一即构建失败而非静默回落 mock | P1 | 构建日志可见变量已注入; 缺变量构建 **必须失败**（负向用例） | ⬜ |
| FR-VC-006 | 后端地址改 **同源代理**: 客户端不得直连后端主机名; 所有后端调用经 `frontend/src/app/api/*` 同源路由; 后端地址仅作为 **服务端** 环境变量（`STAR_API_BASE_URL` 形态）注入, 不进客户端 bundle | P1 | 构建产物 grep 无后端主机名 / 无 `NEXT_PUBLIC_*` 承载的后端地址 | ⬜ |
| FR-VC-007 | 统一同源 BFF 骨架: 在现有 `frontend/src/app/api/group/[...path]/route.ts` 模式上扩展为通用 `/api/*` 代理, 必须保留既有安全约束（设计来源 per `docs/design/DD-WORKTREE-GROUP-001.md:121`, **实现已存在** per `route.ts`: method allowlist `:45`+`:108`、非 GET 同源 Origin `:110-115`、cookie 名校验 + token 长度上限 `:117-124`、生产强制 HTTPS + 拒带凭证 upstream `:135-148`、路径白名单 + 段级 `..`/斜杠校验 `:68-76`+`:150-166`、2,000,000 bytes 双重 body 上限 `:66`+`:179-195`、`cache: no-store` + `redirect: manual` + 3xx 拒绝 `:196-215`、hop-by-hop 与 `set-cookie` 剥离 `:53-65`+`:209`） | P1 | 代理单测覆盖上述约束; 非法 method / path / cookie 名 / 非 HTTPS upstream 均被拒 | ⬜ |
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
| FR-VC-014 | 定时语义: Vercel Cron 只作 best-effort 唤醒/对账入口，不是 Schedule truth、delivery guarantee 或 SLA 级调度器。对 owner 的持久未完成工作按游标重扫；容忍漏发、重复、并发重叠；owner 以幂等键、锁/租约 fencing 和可恢复回执消重。Cron 失败不自动重试；每次 trigger/retry/resume 仍按固化的 `run_as_actor_id` 实时复验 Project/Run 授权，撤权 fail closed。部署回滚会切换 Cron 定义，必须执行 Schedule reconciliation | P1 | 注入漏发、重复、重叠、失败和回滚；证明无重复业务事实、未处理工作可补扫、授权撤销拒绝且审计完整 | ⬜ |
| FR-VC-015 | 后台 worker 不进 Vercel: outbox / inbox 消费端、`bi-owner-runtime`、`platform-messaging-runtime`、`star-dispatcher` 常驻 worker 留在执行面; Vercel 侧只写 owner-local Outbox, 由 owner 侧消费; 禁止在函数内实现租约 / 重试 / 取消语义 | P2 | 事务提交后 Outbox 行被 owner 侧消费; 函数侧 0 worker 循环 | ⬜ |

### 4.5 安全 / 身份 / 供应链 / 可观测（FR-VC-016..021）

| ID | 需求 | 阶段 | 验收方式 | 状态 |
|---|---|---|---|---|
| FR-VC-016 | 本地能力显式禁用: Agent 执行 / 沙箱 / 原生 Hook 执行 / Multipass / 本地 worktree 文件系统在 Vercel 形态下必须 fail closed, 且 UI 明示「该能力需要本地运行时」; 禁以 mock / 预览适配器冒充闭环（per AGENTS.md「Agent/Task CLI 闭环验收」） | P1 | 负向用例: 点「启动执行」→ 明确拒绝 + 原因, 无静默假成功 | ⬜ |
| FR-VC-017 | 授权不得依赖出口 IP: 平台无固定 egress IP（无静态 IP 套餐时）, 仓库根 `ip-list.json` 类 IP 白名单策略在 Vercel 形态下**失效**; 访问控制必须叠加 token / OIDC 校验 | P1 | 无凭据访问被拒; 白名单策略在 Vercel 形态下被显式标注失效 | ⬜ |
| FR-VC-018 | 密钥治理: 平台 Env + 现有 sealed-secrets 等价机制; 禁 secret 进 `NEXT_PUBLIC_*`; 遵守守门 #5（不 dump env value, 不 `cat .env`） | P1 | 产物 / 日志扫描 0 credential | ⬜ |
| FR-VC-019 | 供应链: 出 SPDX / SBOM（Rust + Node 双链）+ 商业许可逐项审查 + 保留各许可证版权 / NOTICE / 专利声明（per AGENTS.md「商业开源依赖硬约束」）；新增依赖须允许不限用途商业使用 | P1 | SBOM 产物存在; 未决许可项 = 0 | ⬜ |
| FR-VC-020 | 可观测与成本: Prometheus exporter 形态不适用于函数 → 改 OTLP 外送; 平台按 Active CPU + Provisioned Memory 计费, 需设成本告警阈值 | P1 | trace 可查 + 成本告警可触发 | ⬜ |
| FR-VC-021 | Preview/Production 环境隔离: Preview 必须有访问保护、独立数据与独立凭据；不得继承 Production secret/数据库角色、写入权限或 Schedule 触发权限。来自未信任分支/Agent 的 Preview 不得对生产 Owner 产生副作用；Production Promotion 仅允许受保护审批流程 | P1 | 分别尝试未授权访问、跨环境凭据使用、Owner 写入与 Cron 调用均 fail closed；验证 Preview 不能访问生产数据，Promotion 留有身份和审计记录 | ⬜ |

---

## §5 NFR

| ID | 指标 | 目标值 | 依据 |
|---|---|---|---|
| NFR-VC-01 | 控制面请求延迟 | p95 < 2s；单请求必须在所选运行时、套餐与配置均已证实的 maxDuration 内结束，业务工作以持久收据异步完成 | Rust Runtime 专属 maxDuration 未证实前不把平台通用值当承诺；禁函数内长轮询 |
| NFR-VC-02 | 载荷上限 | ≤ 4.5MB; 大文件 / 大日志不经函数体 | 平台载荷限额 |
| NFR-VC-03 | 函数包体积 | Rust adapter 以标准 250MB 上限作保守预算；不依赖 5GB Large Functions | 官方 Large Functions Beta 明确覆盖 Node.js/Python，Rust 资格需另行确认；按每 endpoint 依赖闭包实测 |
| NFR-VC-04 | 内存占用 | 预算按目标 Rust Runtime 实测与确认的实例规格制定；平台通用 Pro/Ent 上限 4GB / 2vCPU 不能替代 Rust 资格确认 | 禁进程内大缓存；记录峰值 RSS、并发与成本 |
| NFR-VC-05 | 区域 / 数据驻留 | 固定到已验证且与 PostgreSQL 私网/公网策略、驻留政策相容的函数区域 | 不假定 `iad1` 默认区域；在部署设计中记录选区与故障域 |
| NFR-VC-06 | 冷启动 | 首字节延迟 SLO **实测后定档**, 不得预设 | 每 endpoint 的实际依赖闭包与函数冷启动叠加；当前无实测 |
| NFR-VC-07 | 调度精度 | 不承载 SLA 级调度; Run 级定时精度需求由执行面保证 | Cron 精度限制 |
| NFR-VC-08 | 兼容基线 | Rust MSRV 1.99.1 + edition 2021 保持不变; workspace lint（`unsafe_code = "forbid"` 等）0 放宽 | `rust-toolchain.toml` + `Cargo.toml:178-182` |

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
| AC-VC-001 | Phase 1 前端 Project 可独立构建、部署与回滚；Phase 2 Rust Project 仅在 VCD-2 通过后纳入 | 目标账号上的部署/回滚实测 | P1; P2 conditional | ⬜ |
| AC-VC-002 | 客户端 bundle 无后端主机名、无后端地址 `NEXT_PUBLIC_*` | 构建产物 grep | P1 | ⬜ |
| AC-VC-003 | 同源 BFF 6 项安全约束单测全绿 + 4 类负向用例被拒 | vitest（`frontend/package.json:13` 现有 vitest 口径） | P1 | ⬜ |
| AC-VC-004 | 缺 build-time env 时构建**失败**（负向） | CI 实测 | P1 | ⬜ |
| AC-VC-005 | hosted smoke 验证真实登录、授权 Project/Run/Worktree 读取及同源 BFF；尚未具备 Owner/provider/runtime 的写入、CLI、Canvas、Hook 或 Schedule 能力必须明确不可用，不得使用 seed/mock 假装全链路完成 | 目标账号浏览器实跑 + 权限负向用例 + 每项能力平面/Owner 留证 | P1; deeper owner flow follows WLD gates | ⬜ |
| AC-VC-006 | 本地能力在 Vercel 形态下 fail closed, UI 有明示文案 | 负向实跑截图 | P1 | ⬜ |
| AC-VC-007 | 控制面 endpoint 全部具备 `[[bin]]` + `api/*.rs` 入口, 业务逻辑 0 复制 | `cargo test -p star-api-rest --lib -j 4` 0 err + 代码审查 | P2 | ⬜ |
| AC-VC-008 | 启动自检 0 in-memory 实例（负向: 注入即拒启动） | 启动自检实测 | P2 | ⬜ |
| AC-VC-009 | 双租户并发写入互不可见 + 连接复用无串号 | 并发实测 + RLS 断言 | P2 | ⬜ |
| AC-VC-010 | 迁移独立作业成功, 函数侧 0 DDL | CI 作业日志 + 产物 grep | P2 | ⬜ |
| AC-VC-011 | 函数包实测体积 ≤ 250MB; 冷启动延迟实测值回写 §5 NFR-VC-06 | 部署实测 + 回写文档 | P2 | ⬜ |
| AC-VC-012 | Outbox 跨平面链路闭环: Vercel 侧写 → owner 侧消费 → 回执 | 端到端实测 | P2 | ⬜ |
| AC-VC-013 | 未信任 Preview 无 Production secrets/data/Owner side effects；仅受保护 Promotion 能发布生产部署 | 目标账号隔离凭据、数据和副作用的负向测试 + Promotion 审计记录 | P1 | ⬜ |

---

## §8 已知缺口与待验证项

| ID | 缺口 | 影响 | 验证方式 | 状态 |
|---|---|---|---|---|
| GAP-01 | 按每个 Rust handler 的真实依赖闭包计算函数包体积；不是把 111 个 workspace crate 假定为全部链接进每个函数。标准函数按 250MB 保守预算；Large Functions 5GB Beta 官方只列 Node.js/Python | 影响 Rust adapter 是否能部署；超预算即保留 Rust 控制面在自有运行时 | 最小 handler 与代表性业务 handler 分别构建并测量 `.vercel/output/functions/*`；确认启用的具体限制 | ⛔ |
| GAP-02 | 通用 Functions WebSocket 已于 2026-10-20 GA；Rust Runtime WebSocket/Streaming 能力仍未确认：Rust 页面标 Beta、feature-support 表留空且没有 WebSocket 行；连接仍受 Function duration 限制，跨实例状态需外部持久化 | 通用 Functions GA 不得外推为 Rust 支持；PTY/Canvas WSS 留在 Star 自有执行面。VCD-2 仅在隔离目标账号/计划中验证 Rust streaming/WebSocket、时长、并发、内存和恢复语义 | 官方 Rust Runtime/WebSocket 文档更新或可复现的目标账号最小部署实测；保留通用 GA 与 Rust 专属能力的区分 | ⛔ |
| GAP-03 | Vercel 构建镜像的 Rust 版本能否满足 MSRV 1.99.1 | 决定 Phase 2 可行性 | 最小 Rust 工程部署实测 | ⛔ |
| GAP-04 | 托管 PG 在事务级池化下与 RLS 13 类 + `SET LOCAL` 的兼容性 | FR-VC-010 的实现细节未定 | spike: 池化 + 双租户并发实测 | ⛔ |
| GAP-05 | Next.js 16 + TypeScript 7.0.2 + monaco/three 在平台构建镜像的兼容性与构建时长 | 构建超时 / 失败 | 首次构建实测 | ⛔ |
| GAP-06 | 「Vercel Postgres 已关停并迁 Neon」目前仅第三方来源, 未以 Neon / Marketplace 官方页复核 | 影响 FR-VC-009 的目标选型表述 | 官方页复核 | ⛔ |
| GAP-07 | 类型门禁为 advisory 且平台构建跳过类型检查: CI 已跑 `npx tsc --noEmit` 但 `continue-on-error: true`（ci.yml:143-148, 且注明 4 个 pre-existing 错误）, 同时 `frontend/next.config.js:38` 的 `ignoreBuildErrors: true` 使平台构建也不拦类型错误 → 两道门都不阻塞, 类型缺陷可带入生产 | 类型缺陷进入生产的风险 | 先清零 4 个 pre-existing 错误, 再把 CI typecheck 由 advisory 改回 blocking, 并移除 `ignoreBuildErrors` | ⛔ |
| GAP-08 | `frontend/src/app/automation-debug/api/scripts/route.ts` 依赖本机 Python `scripts/automation/console_server.py`: `FASTAPI_BASE` 默认 `http://127.0.0.1:8080`（`route.ts:10`）, 在函数形态下 loopback 指向函数自身 → 该代理必然 502 | 该调试页在 Vercel 形态下不可用, 且失败原因易被误读为「后端挂了」 | 显式禁用该路由（生产构建不暴露）, 并在 UI 标注为本地专用调试面 | ⛔ |
| GAP-09 | Rust Runtime remains Beta and its official feature table does not establish a stable streaming/WebSocket contract; general Functions WebSocket GA does not remove Rust-specific uncertainty | Runtime or capability changes can invalidate assumptions | Track official Rust runtime revisions and rerun the VCD-2 matrix before enabling any Rust capability; pin evidence to the intended account and plan | ⛔ |
| GAP-10 | **构建根与虚拟 manifest 冲突（Phase 2 可行性关键项）**: 仓根 `Cargo.toml` 段标题实测仅 `[workspace]` / `[workspace.lints.rust]` / `[workspace.package]` / `[workspace.dependencies]` / `[profile.release]` / `[profile.dev]` → **无 `[package]`** ⇒ 虚拟 manifest; 而 Rust Runtime 要求项目根为带 `[[bin]]` 的 package manifest。两种构建根位置各有失败模式: 放仓根 → builder 无 package 可用; 放子目录 → path deps 指向 `crates/*` 是否随部署上传未确认 | 直接决定 Phase 2 能否成立, 决定 FR-VC-002 走候选 A（git integration + Root Directory 子目录 + path deps）还是候选 B（CI 预构建 + Build Output API v3 + `vercel deploy --prebuilt`）; 未裁决前 Phase 2 无法开工, 开工即返工 | 最小工程 spike 分别试 A / B 两条路径, 判据 = 「能部署成功 + 能访问 `crates/star-api-rest` 业务 crate」; 裁决结果回写 FR-VC-002 / 附录 A | ⛔ |
| GAP-11 | Rust Runtime 仍为 Beta，官方 Rust 页面虽称运行在 Fluid Compute，但专属 feature-support 表中 Fluid compute、Active CPU、Streaming、`waitUntil`、日志与请求指标均未给出值；Rust 专属 duration、memory、并发与 WebSocket 映射也未形成可依赖的契约 | Phase 2 不得从 Node/Python/通用 Function 数据推导 Rust 服务承诺；未确认前只允许无敏感数据、无业务写入的隔离 spike | 对照官方 Rust runtime、limits 与 WebSocket 文档，并通过目标计划最小部署实测：duration/streaming/WS、并发、RSS/冷启动、日志/指标、请求体/包体限制及回滚；差异回写 BD/DD/测试设计 | ⛔ |

---

## §9 风险与依赖

| ID | 类别 | 描述 | 缓解 |
|---|---|---|---|
| R-01 | 范围失控 | 「整体上 Vercel」的诉求含不可迁移能力 | §3.2 排除清单为硬边界; FR-VC-016 强制 fail closed |
| R-02 | 契约静默降级 | build-time env 漏配 → 线上跑 mock 无告警 | FR-VC-005 负向验收（构建必须失败） |
| R-03 | 数据串号 | 托管 PG 池化 + 会话级 `SET` → 跨租户泄漏 | FR-VC-010 事务级 `SET LOCAL` + AC-VC-009 双租户并发实测 |
| R-04 | 性能 | 冷启动 + 111 crate 链接 → 首字节劣化 | NFR-VC-06 实测后定档; Phase 1 已提供独立价值, 不阻塞 |
| R-05 | 排期 | Phase 2 依赖 GAP-01/02/03/04/10/11 关闭及 GAP-09 Beta 风险决策 | 分阶段交付, Phase 2 开工前先做隔离 spike |
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
| `vercel-rust/`（新建构建根, 形态待定 — 见 GAP-10 候选 A/B） | `Cargo.toml`（package + `[[bin]]`）+ `api/*.rs` + `vercel.json`; 候选 B 形态下改为 CI 预构建 + `.vercel/output` 产出脚本 | P2 |
| `crates/star-api-rest/src/vercel/`（新建 adapter） | `vercel_runtime` v2 `service_fn` 适配层, 0 业务逻辑 | P2 |
| `crates/star-pg-adapter` / `crates/infrastructure` | 事务级 `SET LOCAL` 语义 + 连接池上限配置 | P2 |

---

## 附录 B 平台事实引用（官方资料检索日期 2026-10-24）

| 事实 | 来源 |
|---|---|
| Rust Runtime 官方 Beta / 全套餐 / Fluid compute | https://vercel.com/docs/functions/runtimes/rust |
| Rust entrypoint 形态（`api/` + `[[bin]]` + `vercel_runtime` v2 `service_fn`） | https://vercel.com/docs/functions/runtimes/rust |
| 函数时长、内存与请求/响应/包体限制（平台表不是 Rust 专属能力承诺） | https://vercel.com/docs/functions/limitations |
| Rust Runtime Beta、Cargo package/`[[bin]]`/`api/` 结构与当前留空的功能支持矩阵 | https://vercel.com/docs/functions/runtimes/rust |
| Large Functions 5GB Beta 的 runtime 范围（Node.js/Python）及标准 250MB 路径 | https://vercel.com/changelog/vercel-functions-can-now-be-up-to-5-gb-in-package-size |
| WebSocket GA 与 Rust Runtime 能力边界、Fluid Compute、实例固定、duration/reconnect 与跨实例外部状态 | https://vercel.com/changelog/websocket-support-is-now-generally-available；https://vercel.com/docs/functions/websockets；https://vercel.com/docs/functions/runtimes/rust |
| Preview/Production environment 与每环境变量隔离 | https://vercel.com/docs/deployments/environments；https://vercel.com/docs/environment-variables |
| Preview 和 Production 部署访问保护的套餐能力与 scope | https://vercel.com/docs/deployment-protection；https://vercel.com/changelog/protect-production-deployments-for-free-on-every-plan |
| Preview 提升到 Production、Instant Rollback 的发布语义 | https://vercel.com/docs/deployments/promoting-a-deployment |
| Cron 漏发/重复/重叠、失败不重试、幂等/锁与 rollback schedule 定义 | https://vercel.com/docs/cron-jobs/manage-cron-jobs |
| Vercel Postgres 关停与 Neon 迁移（第三方, 待官方复核 — GAP-06） | https://kuberns.com/blogs/vercel-postgres-dead-what-replaced-it |

---

## 附录 C 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 初版: 10 段 IPA 章节 + 平面切分 (§3.1) + 排除清单 (§3.2) + FR-VC-001..020 (§4) + NFR-VC-01..08 (§5) + fail-closed 矩阵 (§6) + AC-VC-001..012 (§7) + GAP-01..09 (§8) + R-01..08 (§9)。范围 = ① 分阶段 (Phase 1 前端 / Phase 2 Rust 控制面)。仓库事实均当日实测, 平台事实检索日期 2026-10-24 | 用户 2026-10-24 提问「部署到 Vercel 的要件定义」+ 拍板 `ask_36f7d8462d9773f241c35533`（范围 ① 分阶段 / 落档本文件） |
| **v0.2** | **2026-10-24** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | **自审修订 (5 项)**: (1) **修正 crate 计数** — 原写「~130 crate」为估数, 实测 members 数组 = 116 条 / **111 唯一 crate**（含 5 组重复声明: `leads` / `shared-task` / `star-canvas` / `star-scheduler` / `star-workflow`）, §0/§2.2/§5/§8/§9 全部改为实测值; (2) **新增 GAP-10（Phase 2 可行性关键项）** — 仓根 `Cargo.toml` 实测为**虚拟 manifest**（无 `[package]`）, 而 Rust Runtime 要求项目根为带 `[[bin]]` 的 package manifest → FR-VC-002 改为「先裁决候选 A（git integration + Root Directory 子目录 + path deps）/ 候选 B（CI 预构建 + Build Output API v3 + `--prebuilt`）」, §2.2 构建行 + 附录 A 同步; (3) **FR-VC-004 对齐既有 `[profile.release]`**（`Cargo.toml:293-295` lto=thin / codegen-units=1）并标注构建根不继承 workspace profile; (4) **补实现级证据** — FR-VC-007 追加 `group/[...path]/route.ts` 逐条行号（既有安全约束确已实现, 非仅设计文档）, GAP-08 追加 `FASTAPI_BASE` 默认 `127.0.0.1:8080` 证据（函数形态必然 502）; (5) **并发写入事件** — 自审期间检测到本文件被并发进程追加了 1 行同主题 GAP-10 草案（措辞不同, 含 `⇒` 标记, 非本会话所写）, 导致 §8 出现 2 条 GAP-10; 已**合并为单条**并保留其实质内容（构建根放仓根 → builder 无 package 可用 / 放子目录 → path deps 是否随部署上传未确认 + 二选一 spike 判据）, 未删除其任何信息 | 交付前自审（用户 2026-10-24 指令「自审」）; 全部结论均以当日实测为准, 未沿用记忆 |
| **v0.3** | **2026-10-24** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核** | 基于 Vercel 官方文档复核修订平台能力边界：当时将 Functions WebSocket 记录为 Public Beta（该表述由 v0.4 按 2026-10-20 官方 GA 公告纠正），但 Rust Runtime 的 Streaming/能力矩阵仍留空；将 Rust maxDuration/内存/并发/WebSocket 映射列为实测缺口；修正 Cron best-effort、可能漏发/重复/重叠、失败不自动重试及 Instant Rollback 恢复 Cron 定义；将 5GB Large Functions 范围限定为 Node.js/Python，Rust 按 250MB 保守预算；删除未有官方证据的 `iad1` 固定默认推断；新增 GAP-11 与 FR/AC Preview 环境隔离门，并将基本/详细/测试设计缺失、Phase 1 与有条件 Phase 2 交由总 WBS 跟进。本文仍为 Draft，0 实现。 | 用户要求审核 Vercel 版本设计并纳入 WBS；基于 2026-10-24 Vercel 官方 Rust、Functions limits、WebSocket、Cron、Large Functions 与 Deployment Protection 页面复核 |
| **v0.4** | **2026-10-24** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核** | **纠正 v0.3 的平台快照：通用 Functions WebSocket 已于 2026-10-20 GA；Rust Runtime 仍为 Beta，专属 feature 表为空且没有 WebSocket 项，Rust 能力继续保持未确认。补充官方 GA 公告与现行 WebSocket 文档引用；VCD-0 兼容性审查报告确认候选设计只能选择性吸收，未批准的第二 Owner/数据库/worker/scheduler/VM 执行拓扑仍排除。部署、账户、Rust runtime 与生产能力均未验证，实施状态仍为 0。** | **2026-10-24 当前官方 Vercel 文档复核及 VCD-0 basic/ADR、detailed、test 兼容性审查。** |
