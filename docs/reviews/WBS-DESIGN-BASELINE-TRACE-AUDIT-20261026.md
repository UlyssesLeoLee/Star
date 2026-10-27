# WBS W1 设计基线追踪审计

**审计日期：** 2026-10-26
**审计基线：** 当前隔离 checkout 的 `dev@b1aa0f8f3536f5ab49b2a2af3f02ad6b4f707aa4`
**范围：** 只读核对需求、基本设计、相关详细设计及 WBS 文档；本报告是文档追踪证据，不是实施或产品验收。

## 0. 目的与范围

按 WBS 的 W1「设计基线对账」检查用户已确认的 Project/Run/Worktree/Branch 架构、跨 App 边界、无宿主 Run、Multipass/K3s、Agent/Hook/Schedule、事件协作及 Rust 桌面性能要求是否在当前文档间一致。审计依据仅为本 checkout 的固定基线；没有读取主工作树未提交文件、其他工作树、代码或运行产物。

## 1. 需求到设计追踪矩阵

| 需求主题 | 需求基线 | 基本设计 | 详细设计 | WBS 跟踪 | 文档审计结果 |
|---|---|---|---|---|---|
| 主导航与工作区归属 | [requirements.md §50.1](../requirements.md)：Project → Engineering Run → Worktree → Branch；Branch 是本地 checkout 状态叶 | [basic-design.md §16.2、§16.68](../basic-design.md)：Run 独立于远端身份；选择 Worktree 后才显示完整 Run tabs | [DD-WORKTREE-GROUP-001.md §§1.1–1.2](../design/DD-WORKTREE-GROUP-001.md)：主树、路由、Index 聚合视图与中性 UI 名称 | WBS §1 W1；§2 W13 | **主体一致。** Infrastructure 详细设计 §0 有一处旧层级表述，见 §3.1。 |
| Project、Repository 与 hostless Run | requirements §50.3.4、§50.8AP、§50.8AV：Project 自有；Repository 关联可为 0..N；解绑前迁移引用；Run 可不连远端 | basic-design §§16.2、16.66–16.68、16.77–16.77.1：hostless Run/Task Cards 与 repository 绑定边界 | Group DD §§1.1、3.4–3.5：Run 权限和 Worktree binding 分开 | WBS §2 W2/W3；§13；§23.2 W11-A | **产品语义一致，交付仍开放。** 文档明确目标库、Project↔Repository 持久 owner、Run binding/Host root 与生产授权验收未闭合；不等同功能可用。 |
| Run Apps、Task Card、CLI、Canvas、Chat | requirements §§50.3–50.6：Task Card 与 Canvas 同级，CLI 从卡内打开，底栏显式 WORKTREE/GLOBAL | basic-design §§16.2、16.6–16.9、16.66 | Group DD §§7–8：Run scope、任务/画布互动、聊天与插件接口 | WBS §2 W6/W7/W13 | **目标结构一致，执行门未关闭。** WBS 将真实 CLI/runtime/sandbox、跨 App owner 与 UI 数据链路列为 OPEN；hostless Task Cards 入口不等于执行能力。 |
| Hook 强制策略与可视化配置 | requirements §50.8D：Rust 原生 fail-closed、继承/快照、BI 证据及高级设置可视化 | basic-design §16.17、§16.20：HookSet 版本与统一执行门 | Group DD §0、§13：Hooks 位于 Advanced Settings，与其他能力并列；仅显示策略及结果 | WBS §2 W4–W5 | **设计方向一致。** UI、所有写入入口接线、生产授权、策略证据到 BI 的完整闭环仍为 OPEN；本次未验证产品行为。 |
| Multipass 管理 K3s 与资源边界 | requirements §§50.8H–50.8I：不依赖 Docker Desktop；当前测试总 inventory 上限为 2 条，含停止/可恢复记录；选择基础设施 placement 才需要精确 cluster identity/generation | basic-design §§16.21、16.27–16.28 | [DD-LOCAL-INFRASTRUCTURE-001.md §§0、1.1、2–3、7](../design/DD-LOCAL-INFRASTRUCTURE-001.md)：Multipass guest/K3s、双层容量、管理权限、两记录锁与无自动清理 | WBS §23.1–23.2 | **VM 测试约束对齐；Run 绑定与旧层级未对齐。** 见 §3.1–3.2。Manager/provider、目标 DB、readiness、sandbox、bridge 与端到端仍 OPEN。 |
| Schedule、run-as 与 Loop | requirements §50.8B、§50.8AP 及 Phase 9F4B–9F4D gates：固定创建者身份，每次触发/重试/恢复复验；admission 成功前不执行 | basic-design §16.16、§16.25–16.28 | Group DD §§7–8；Infrastructure DD §§3–4 | WBS §2 W9、§23.2 | **run-as 原则与 fail-closed 目标明确；occurrence 的 Branch 语义互相冲突。** worker/admission/执行链不应因文档中的局部 source 状态而标为已完成。 |
| Agent 扩展、插件、工作流与事件 | requirements §§50.8A、50.8C、50.8E：版本化、可替换、有预算与隔离；业务事件与传输 adapter 解耦 | basic-design §§16.15、16.19 | Group DD §§2、7–8；Infrastructure DD §§4、7 | WBS §2 W4/W8/W12 | **架构原则有映射。** 生产 provider composition、durable owner consumer/settlement、热卸载 drain 与恢复仍 OPEN；报告不推断实现。 |
| Rust 桌面低内存、并行与多人协作 | requirements §§50.8A、50.8AW：有界资源、背压、公平调度、持久在线稿/冲突恢复与延迟目标 | basic-design §§16.15、16.78 | Group DD §8.27：owner 事件触发推送、冲突草稿；队列/订阅数字属于候选预算 | WBS §15–16、§20.2；W14/W16 | **目标和待测项有记录。** 目标设备、workload、RSS/CPU/延迟、跨实例投递与双稿恢复尚未完成；协作设计不能表述成运行验收。 |

## 2. 已确认的跨文档一致项

1. requirements §50.1、basic-design §16.2/§16.68 与 Group DD §1.1 都把 Project 置于左侧首层，并按 Run、Worktree、Branch 展开；Worktree Index 是汇总管理视图，不替代主树。
2. Run 是 App、任务与分析的授权范围；Worktree 负责 checkout 焦点。Task Card 与 Canvas 同属 Run，CLI 是卡内操作面板，底栏聊天范围需明确选择。
3. Project/Run 的普通目录权限不以远端 Branch 为前提；SCM 操作和 Worktree 生命周期另外要求对应授权、绑定与可信来源。
4. Hook 的配置入口在高级设置；安全策略由原生核心执行，插件只能在受限能力边界内扩展。
5. Infrastructure 详细设计明确 Multipass 管理 Linux guest、K3s 在 guest 中运行，并把测试 inventory 限定为两条、含停止/可恢复记录，以互斥锁串行检查和创建，禁止自动 purge。
6. 需求和 WBS 都区分设计、局部代码切片和生产验收；Schedule 执行、Agent CLI、BI/Benchmark 与桌面容量均不得由局部设计或静态证据推断为已完成。

## 3. 矛盾与缺口

### 3.1 Infrastructure 详细设计保留旧导航次序

`DD-LOCAL-INFRASTRUCTURE-001.md §0` 写作「Project → Branch → Run → Worktree」，与 requirements §50.1、basic-design §16.2、Group DD §1.1 的固定顺序冲突。原意应是 Infrastructure Environment 不增加主导航层级；建议保留该意思，将树序改为 Project → Engineering Run → Worktree → Branch，并在涉及 Infrastructure binding 的段落继续区分计算 placement 与导航归属。

### 3.2 hostless Run 与基础设施绑定仍被 Branch 强制耦合

requirements §50.8AP 定义 Run 可脱离远端 host/repository/Branch 创建、授权和列出；WBS §23.2 又把「仅选择基础设施 placement 的 Run 才固定 cluster」列为 W11-A 完成条件。但 requirements §50.8AA 将 Branch 写入每个 canonical Run 的必需 Infrastructure Binding scope；basic-design §§16.27–16.28 与 DD-LOCAL-INFRASTRUCTURE-001 §§2–3 仍把 Project/Branch/Run 固定写入 binding/workspace tuple。

这让“hostless Run 不需要 Branch 或 infrastructure binding”与“Run 必须绑定 Branch 才能有基础设施快照”同时留在活动设计中。应明确把 binding 定义为**可选的执行 placement 事实**：未选择基础设施 placement 时，不要求远端 Branch、cluster 或 binding；选择 placement 后才固定精确 Host/cluster generation、profile 与预算。随后同步需求、基本/详细设计，以及 API、schema、migration、digest/replay 与测试设计的兼容规则。WBS §23.2 已把此项排在 provider 实施之前。

### 3.3 hostless Schedule occurrence 的持久化契约自相矛盾

basic-design §16.16 的 WLD-3 子节允许在本地 canonical identity 完整时为无远端 Branch 的 Run 持久化 occurrence，Branch 可缺省或为 JSON null，且明确此边界不授予 admission。紧接的 basic-design §§16.25–16.26 又要求 occurrence `target_snapshot` 含非 nil Branch UUID，并规定缺 Branch 的 materialization/admission/runtime fail closed。两者分别把 Branchless occurrence 定义为允许和禁止。

需要先冻结三个不同门的语义：Rule 配置/读取、durable occurrence 物化、admission/实际执行。若按 §16.16 保留 Branchless 物化，§16.25–16.26 的快照约束及对应需求、迁移、domain/API/test contract 必须同步改成 nullable/无远端身份可表达；同时继续明确 occurrence 不等于执行授权，任何未满足当前 run-as、Hook、预算、Runtime 或可选 placement 的情况仍 fail closed。若产品要求所有 Schedule 都执行在有基础设施的环境，则应显式区分 occurrence 身份所需的本地 Branch 与执行 placement 的 cluster binding，而不能用远端 Cloud Branch 充当本地 checkout Branch。

### 3.4 追踪范围之外的实施门仍开放

WBS §1 的 W1 验收要求各活动文档统一层级、owner、名词与状态；本次已发现 §3 中的冲突，故 W1 不能因本报告产出而关闭。WBS §22.4 将 WLD-7 Canvas 设计同步列在下一步首位；§23.2 要求 W11-A 在 Provider/Manager 实施前完成。WBS 的 W7/W8/W9/W12/W13/W14/W16 仍分别保留执行、消息结算、调度、能力插件、跨 App UI、桌面测量和协作运行门。

## 4. 后续工作顺序与依赖

| 顺序 | 工作 | 依赖与退出条件 |
|---|---|---|
| 首先 | **完成 WLD-7 Canvas 设计同步。** 按 WBS §22.4 的现行顺序对齐 Canvas SRS/基本/详细设计与 WBS；DB、跨实例推送和恢复仍独立验收。 | 设计同步先于其余后续实现门；不得把文档同步标成运行验收。 |
| 并行候选，P1 | **W11-A：统一可选 Infrastructure Binding 与 hostless Run 契约。** 修改 requirements、basic、Infrastructure DD 和相关 API/schema/digest 设计。 | 可与 WLD-7 设计工作分开准备；必须在 Provider/Manager 实施前完成。退出条件：不选 placement 的 hostless Run 无 Branch/cluster/binding 前提；选择 placement 时绑定精确 cluster identity+generation。对应 WBS §23.2。 |
| P1 | **统一 WLD-3 occurrence 的 nullable Branch 规则。** 将 rule、occurrence、admission、execution 分开定义并贯穿快照/replay/验收。 | 需冻结 occurrence 身份最低字段和 Schedule 的本地/基础设施执行路径；随后更新 Schedule 需求/基本/详细及迁移、测试设计。执行仍需单独门禁。 |
| P1 | **修正 Infrastructure DD §0 的旧导航文字并扫查同类残留。** | 依赖固定的 Project → Engineering Run → Worktree → Branch 术语；复核不把 Infrastructure Environment 加入左树。可与 W11-A 同一设计修订处理。 |
| P1 | **完成全量 W1 cross-layer trace 与修订记录。** 核对每个活动设计的版本、owner、状态和历史冲突标记。 | 在优先设计对账完成后由独立 reviewer 复核；验收状态只能由相应运行证据推进，不能由文档同步代替。 |
| 后续 | **按既有 WBS 推进独立运行门。** WBS §22.4 中 Vercel 设计追踪随后；W8 owner settlement、W14 设备/workload/profile 冻结与测量，以及 W9/W11/W12/W13/W16 均保持各自 owner 与依赖。 | 依赖 WBS §§2、20、22–23；本审计未替代这些工作，也未改 WBS。 |

## 5. 验证摘要

- **已完成：** 核对 brief、Read COMMENTS（无留言）、本 checkout HEAD 和上述文档章节；确认工作目录的 baseline commit 与 brief 指定 SHA 完全相同。
- **报告范围：** 本文件是唯一允许的产物；未编辑需求、基本/详细设计、WBS、源代码或测试。
- **未运行 / NOT_RUN：** Cargo、前端测试、数据库/RLS、Multipass/K3s、部署、UI、协作推送、RSS/CPU/延迟基准。此类验收状态继续 OPEN/NOT_RUN。
- **变更校验：** 报告为新文件，执行了针对该文件的 no-index whitespace 检查和 trailing-space 搜索，并对引用的路径及章节标题做本 checkout 内核验；未运行测试。

## 6. 限制与当前结论

这是固定 dev 文档快照上的只读 trace audit。它能指出文档之间的映射和矛盾，不能证明实现与文档一致，也不能证明代码、安全策略、数据库、消息投递、VM/K3s、性能或产品验收。未检查代码结构或外部部署证据，也未核验远端分支状态。

**W1：OPEN。** 本报告仅提供一次文档审计输入；至少需要完成 §3 的两项优先对账、修复旧层级表述，并由后续独立复核确认活动文档和验收状态一致。WLD-7、W11、W8、VCD-0、W14 及产品整体验收均未因此关闭。

## 7. 修订记录

| 版本 | 日期 | 修订 | 触发 |
|---|---|---|---|
| v0.1 | 2026-10-26 | 在固定 dev 文档快照上新增 WBS W1 需求到设计追踪审计，记录对齐项、矛盾、OPEN/NOT_RUN 边界与有依赖的后续顺序 | WBS-DESIGN-BASELINE-TRACE-AUDIT-20261026 brief |
| v0.2 | 2026-10-26 | 终审修正后续顺序，明确 WLD-7 设计同步按 WBS §22.4 优先，W11-A 可并行准备但须先于 Provider/Manager 实施；准确说明新文件 whitespace 检查范围 | Mavis 终审发现顺序表与 WBS §22.4 不一致，并区分了新文件与 tracked-file 检查 |
