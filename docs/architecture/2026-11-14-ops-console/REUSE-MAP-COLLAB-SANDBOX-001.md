# REUSE-MAP-COLLAB-SANDBOX-001 — 协同与沙盒的本机项目复用清单

> 状态：🟡 Draft v0.1（复用盘点，未开始移植）
> 日期：2026-11-14
> 依据：AGENTS.md §0「本机 Rust 服务端按模块复用」
> 设计原型：Claude Design 画布「2b · 协同」「2c · 沙盒」

## §0 目的

Star 的「协同」页签（文件即任务 × 多人多 Agent 共同编程）与「沙盒」页签（虚拟机、实体机、Docker、沙盒内 Worktree）在本机已有可复用实现。本文按模块列出来源、状态、许可与复用方式，作为实装前的依据。**本文只做盘点，不复制任何代码。**

## §1 来源仓库状态（2026-11-14 实测）

| 仓库 | 路径 | HEAD | 工作区 | 许可证 | 用途 |
|---|---|---|---|---|---|
| IDE1.0 | `E:\IDE1.0`（dev） | `a200e5d` | 有未提交改动（18 项，集中在 `ide-fast/` 与 `ide-agent-router` 新增文件）；本文引用的文件均未改动 | AGPL-3.0 | 协同、.nema、多 Agent 流水线 |
| SandTree | `E:\SandTree` | `3f6eda2` | 有未提交改动（4 项） | Apache-2.0 | 沙盒 / Docker 控制面与观测面 |
| Argo | `E:\Argo` | `2941596` | 干净 | AGPL-3.0 | 多机 Controller / Worker、CLI 适配 |
| Star | `D:\Star` | — | — | MIT | 目标 |

按 §0 规则：来源仓库有未提交改动时只读，并以文件 hash 固定引用版本（§2 表中列出）。

## §2 协同：来自 IDE1.0

| 来源模块 | 文件（sha256 前 16 位） | 能力 | Star 落点 | 方式 |
|---|---|---|---|---|
| `nema-codec` | `src/lib.rs` `1a71ad20b81a161c`；`src/interpreter.rs` `57d51a5e756ef30f` | .nema 编解码（Table / 5W1H / Graph、共享词典、.nemat 压缩传输、字节级往返校验） | 替换前端 `lib/nema/nema.ts` 的规范子集解析；后端新增 Rust 实现 | 直接复用（许可确认后） |
| `ide-agent-router::collaboration` | `12f22fdfb88ba106` | 协作会话：远程光标、按键广播、`AnchorOffsetTable` 分区锚点、按行归属 | 协同页的光标、分区与归属显示 | 适配（接入 Star 的 owner revision / Outbox 推送） |
| `ide-agent-router::coordinator` | `5b4e3fbf028c39f8` | 文件提交协调：文件状态、任务状态、Outbox 事件、补丁提交 | 协同写入路径与任务卡回写 | 适配 |
| `ide-agent-router::nema_pipeline` | `8f9368d91237a0e5` | 路由拆分（协作组 / 独立组）、`.nema` 语义三方合并、`.nema → Rust` 生成与 SourceMap、漂移检查、按 Agent 分支、验证与验收证书、迭代单 | 协同页顶部五步流水线与右侧「协作组 / 合并 / 验证」 | 适配 |
| `ide-agent-router::reviewer` | `1f12b8759f8df0fc` | 按段指导区、归属到人的 hunk、评审会话 | 评审闸门与合并面板 | 参考 |
| `ide-agent-router::tenant` | `7184743270bde689` | 租户内协作隔离 | 与 `domain-tenant` 对齐 | 参考 |
| `editcore::file_lock` | `11c7deb64250f3d8` | 文件锁 | 文件租约（Star 要求带递增 fencing token） | 适配 |
| `editcore::history` | `e87cc55354de6544` | 编辑历史 | 协同版本面板 | 参考 |
| 设计文档 | `docs/01_基本设计/BD-002_Multi-Agent-NEMA-Collab-Platform.md` | 四阶段流水线、六大支柱、系统不变量 | 协同页需求与设计 | 参考 |

与 Star 约束的对齐点：
- 冲突判定以服务端 owner revision / CAS 为准，不用客户端时钟（AGENTS.md「实时协作与冲突保护」）；IDE1.0 的按键广播需改为由 owner 事务事实驱动推送。
- 1 秒内到达的两份完整提案立即进入冲突事务并保留双方草稿，不允许后写覆盖前写。
- 文件即任务：协同页的工作集按「任务卡」或「Branch」选择，文件、.nema 侧车与任务卡绑定沿用 Star 现有 `nemaStore` 规则（≥2 人共同编辑自动生成侧车）。

## §3 沙盒：来自 SandTree 与 Argo

| 来源 | 模块 | 能力 | Star 落点 | 方式 |
|---|---|---|---|---|
| SandTree | `resource-graph`、`kernel`（resources / operations / lock） | 统一资源拓扑、操作任务（OperationJob）、锁 | 沙盒页资源树与「操作任务」栏 | 适配 |
| SandTree | `observation-core`、`observation-model` | 观测四级：原生 → 执行 → 探针 → 元数据，每个值带来源、可信度、新鲜度 | 沙盒页「观测」页签与资源树上的方式标记 | 直接复用 |
| SandTree | `policy`、`credential` | 严格出站封印、凭据隔离 | 沙盒网络出站与凭据策略 | 适配 |
| SandTree | `vfs`、`store` | 受限根目录文件访问、快照 | 「限路径同步」与快照 | 适配 |
| SandTree | Docker / Compose、Multipass、Windows Sandbox、Docker Sandboxes provider | 各类运行时的发现与生命周期 | 沙盒页「虚拟机」「Docker」分组 | 适配 |
| Argo | `crates/argo`（`server.rs` Controller、`worker.rs` Worker） | 多机 Controller / Worker、心跳、任务租约、审批 | 沙盒页「实体机」分组与 Worker 心跳 / 租约显示 | 适配 |
| Argo | CLI 适配（Claude / Codex / Hermes 等） | 调用各机器上已安装的 CLI | 沙盒概览「已安装的 Agent CLI」、任务卡 CLI 页签 | 参考 |
| Argo | 跨机快照 / 补丁 / 审阅工作树 | 在另一台机器上改代码并带回审阅 | 「在此沙盒开设 Worktree」与同步 | 参考 |

与 Star 约束的对齐点：
- 只有受信的 Rust 宿主管理进程可以调用 Multipass、Docker 守护进程；Agent、插件、任务卡 CLI 拿不到宿主 socket 或任意宿主 shell（AGENTS.md「多 K3s 集群」约束）。
- 测试阶段 Multipass 实例最多保留 2 台，创建走带跨进程锁的受控脚本，达上限时拒绝。
- Worktree 进入沙盒只能经显式、限路径、可审计的桥接；沙盒页提供「沙盒内克隆」「限路径同步」「只读挂载」三种方式。
- 离线主机仍列出并标明，不把不可用当作空结果。

## §4 许可证与权属

| 来源 | 许可 | 进入 Star（MIT）前需要的处理 |
|---|---|---|
| SandTree | Apache-2.0 | 可直接复用，保留版权声明与 NOTICE |
| IDE1.0 | AGPL-3.0 | 二选一：权利人把要复用的模块改为与 Star 兼容的许可后再移植；或按 AGPL 履行义务（含网络服务提供对应源码），并评估对 Star 整体交付形态的影响 |
| Argo | AGPL-3.0 | 同上 |

三个仓库与 Star 均为同一作者，许可调整属于权利人的决定，**实装前需要你确认采用哪种方式**。确认前只按「参考」使用，不复制代码。

## §5 已知缺口

| # | 缺口 |
|---|---|
| G-1 | IDE1.0 与 SandTree 有未提交改动，移植前需你确认以哪个提交为准 |
| G-2 | IDE1.0 的协作广播是进程内实现，需改造为持久事实 + Outbox 推送才能满足 Star 的实时协作约束 |
| G-3 | Argo 的 Rust/Tauri 与双机联调在其仓库内尚未实跑（见其 README） |
| G-4 | SandTree 的桌面 GUI 未实现（见其 `11_画面与交互设计.md`），Star 沙盒页为新设计 |
| G-5 | AGPL 模块的许可处理待定（§4） |

## §6 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-11-14 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 初版：IDE1.0 / SandTree / Argo 模块盘点、hash 固定、许可处理与缺口 |
