# UI 规划：Project / Run / Worktree 驱动的高性能桌面端

**版本：** v0.3（新版规划）
**日期：** 2026-10-22
**设计基线：** requirements v6.91、Schedule API SRS v0.64、basic design v6.93、`DD-WORKTREE-GROUP-001.md` v5.16、实施计划 v7.14
**实现基线：** 候选集成代码 `50b67a94`（包含 UI-1 `79c0320c` 与 WLD-5 普通同源链接守卫）；最终本地 `dev` 集成状态见 WBS 2026-10-22 checkpoint。UI 规划文档本身是该代码基线之上的后续提交。
**状态：** 目标交互与分期规划；不是桌面端或所有 Run App 已完成的声明。

## 0. 目的与本轮结论

把当前 Worktree-first 目标整理成可分期实施的产品 UI 方案。产品主线是 Project 内多 Run、多 Agent checkout 的可见归属、冲突与生命周期管理；Run 是协作、任务和分析的工作区；Worktree 是明确可操作的本地 checkout；Branch 是 Worktree 下可信本地观察到的末级状态。导航只有在用户选择具体 Worktree 后才展示该 Run 的 App tabs；只选中 Project 或 Run 时，主区仅展示上下文摘要和可选 Worktree，不提前挂载 Run App。

本轮不把当前 Web/React 页面或 Tauri 原型当作最终桌面体验。Web 端已有导航、授权 RunContext、Task Card 与受控终端的局部实现；Tauri 壳的独立界面仍由 React/Vite 渲染，Rust IPC provider 尚不完整。目标桌面 UI 按 Rust 原生前端规划，先用代表性页面测量渲染器和资源成本，再冻结技术选型。

## 1. 现状基线与差距

| 区域 | 当前证据 | UI 规划中的处理 |
|---|---|---|
| 左侧 Project 目录 | `ProjectRunTree` 按 Project → Engineering Run → Worktree → Branch 展示，并使用固定视口、行高、overscan 和键盘树导航；UI-1 切片分别呈现人工状态、机器状态、Agent 分配、Session/Runtime 绑定、归档状态和 binding 版本。锁、变更详情、观测时间、Branch 来源/新鲜度尚无可信观测，显示 unknown。 | 延续树形导航；补齐有 owner 证据的冲突、锁、freshness 和生命周期状态；Project Worktree Index 只作跨 Run 汇总，不取代主导航。 |
| Run Shell | `RunWorkspaceShell` 当前无 focus 时仍进入 Task Cards，是本轮确认的导航偏差；有 focus 才显示 Run App tabs。当前 Task Cards 与 BI 有受控页面，其余 tabs 仍有待迁移项。 | 修为：Project/Run 选择态只显示摘要和 Worktree 列表；点击具体 Worktree 后才显示该 Run 的 App tabs。按 owner capability 显示可用、只读、不可用或等待状态。 |
| Task Card / CLI | Task Card 有分页创建与读取切片；终端必须对应当前 Task、Worktree 和经服务端授权的 Session。无 provisioner、授权或状态不确定时 fail closed。 | 将 CLI 收入卡内执行面板；补全 Agent 分配、运行状态、取消/重接、验证证据与结果回写的界面闭环。不得显示假终端或把 Session 结束当作任务完成。 |
| Canvas | 有独立 Canvas 页面和 Group Canvas/Worktree 图相关实现，但 Run 内统一入口和完整 owner 生产闭环尚未完成。 | Infinite Canvas 与 Task Card 在 Run 层同级；Canvas 保存几何、布局和 typed 引用，Task/Worktree/关系事实仍由其 owner API 管理。 |
| 聊天 | scope 选择与授权目标目录存在局部实现；当前 Run Shell 的输入和发送仍禁用。 | 固定底栏，明确 `WORKTREE` / `GLOBAL`；发送前展示有效目标、权限和上下文摘要。GLOBAL 不扩大授权。 |
| Hooks | 高级设置内已有可视化 Hook 编辑页。本轮 WLD-5 扩展普通同源内容链接守卫，并通过公开 App Router API 在批准后执行 SPA 导航；Stay 阻止路由和链接回调。定向 4 个测试文件 33 项与 TypeScript typecheck 通过。浏览器 Back/Forward 仍不受保护；真实宿主身份与执行期原生 Hook 链路仍开放。 | 保留“高级设置 → Hooks”；各可控导航出口统一 Leave/Stay；历史前进/后退限制明确显示并作为后续门禁；补足继承/版本差异、模拟评估、发布回滚和决策深链。 |
| Schedule / hostless Run | WLD-3 已增加没有远端 Branch 的 Run 规则读写/配置路径；目标仍须绑定当前本地 Worktree、Task、repository、Profile 和 HookSet，Branch 仅由授权 Run 范围提供。Rust 与 PostgreSQL 门禁未运行，且没有 Schedule worker/trigger/executor。 | 将“规则配置已保存”“规则启用配置态”和“Occurrence/Run 实际执行”显示为不同状态；无远端 Branch 只允许配置，不显示为可触发或可运行。触发仍受 Domain/materializer 与 Run-owner Branch 必需门保护。 |
| 桌面端 | `crates/star-desktop` 是 Tauri 2 + React/Vite 壳和 Rust IPC；Task provider 缺失时失败关闭，旧任务演示回退已移除。当前不能称为 Rust 原生前端闭环。 | 建立 Rust UI 目标架构，与 Web 端共享 API/owner/event 契约；Rust IPC 只做有界 adapter，不拥有第二份业务事实。 |

代码与报告证据：`frontend/src/components/Sidebar.tsx`、`frontend/src/components/run/ProjectRunTree.tsx`、`frontend/src/components/run/RunWorkspaceShell.tsx`、`frontend/src/components/run/RunTaskCardsPanel.tsx`、`frontend/src/lib/group/taskCliPresentation.ts`、`crates/star-desktop/frontend/src/App.tsx`，以及 `docs/reports/WLD1-LIFECYCLE-OP-OUTBOX-20261021.md`、`docs/reports/WLD2-TASK-CLI-UI-SLICE-20261020.md`、`docs/reports/WLD5-HOOK-POLICY-UI-SLICE-20261020.md`。这些证据只覆盖报告声明的代码/测试范围；不证明生产身份、目标 DB、Runtime 或全产品闭环。

## 2. 产品交互原则

1. **从 Project 管起。** 用户先选 Project，再查看该 Project 的 Engineering Run、Worktree 与 Branch 状态；远端 SCM/宿主是否连接不改变层级或可见的本地管理路径。
2. **Run 是工作区边界。** Inbox、Work Item/计划、Task Card、Infinite Canvas、Workflow、BI/Benchmark 与可授权 Plugin Apps 属于 Run。它们共用一个 RunContext 和 owner API，不另造同义列表或事实表。
3. **Worktree 表示具体 checkout。** 选中 Worktree 设置 RunContext focus，并切到它所属 Run 的 App tabs；focus 用于本地文件、Git、CLI 和会话目标，不扩大 Run 授权。
4. **Branch 是状态叶。** 只显示可信本地 checkout observation、来源、时间和 freshness。远端 Branch 是可选的独立身份；缺失/陈旧/未授权显示“未知”，不推断、不创建假状态。
5. **未选 Worktree 时不进入 Run App。** 选择 Project 或展开/选中 Run，只显示身份、成员/策略摘要和该 Run 的 Worktree 清单；不展示 App tabs、Task Cards、Canvas、Inbox 或其他业务 App。只有点击并成功校验具体 Worktree 后，才挂载其所属 Run 的 App tabs。此时 Chat Bar 可选 `GLOBAL`；选择 Worktree 后再启用 `WORKTREE`。
6. **明确区分事实、投影和执行状态。** Session、Task、Schedule、Hook 决策、Infrastructure admission、集群 readiness、BI coverage 各显示自身 owner 的状态；请求已排队、IPC 已返回或插件已加载都不等于业务成功。
7. **故障时保留可理解的状态。** Loading、空结果、权限拒绝、授权过期、provider 不可用、状态未知、版本冲突、离线、取消中、重接中、部分覆盖均使用不同文案和操作入口。
8. **产品文案保持能力中性。** 可显示已配置的真实连接器名称，不使用竞品对照、模仿暗示或仿制文案。
9. **区分已保存配置与可执行能力。** Schedule Rule 写入成功不代表存在 worker、Occurrence、Run admission 或 executor；Hook 草稿离开确认也不代表策略已经发布或执行。

## 3. 信息架构

```text
左侧主导航（管理目录）
Project（首层选择器与根）
├─ Engineering Run
│  ├─ Worktree（Run 绑定的本地 checkout；点击后成为 focus）
│  │  └─ Branch（该 checkout 的本地分支/ref 观察叶）
└─ Project Worktree Index（跨 Run 汇总入口，不改变主层级）

中间主区（不是导航树的新层级）
├─ 未选 Worktree：Project/Run 摘要 + Worktree 清单（无 App tabs/content）
└─ 已选 Worktree：当前 Run 工作区（此时才挂载 App tabs）
   ├─ Inbox / Work Items / Planning
   ├─ Task Cards（卡内 Agent 与 CLI）
   ├─ Infinite Canvas（与 Task Cards 同级）
   ├─ Workflow / Schedule（LangGraph 是编排 adapter）
   ├─ BI / Benchmark
   └─ Plugin Apps（已授权的可插拔 App）

固定底栏：Chat Bar(scope = WORKTREE | GLOBAL)
全局设置：高级设置 → Hooks；插件能力与基础设施策略由有权限的设置入口管理
```

主屏布局采用三块区域：左侧 Project/Run/Worktree/Branch 树；中间 Project/Run 摘要或当前 Run App；按需打开的右侧检查器/详情面板；底部固定 Chat Bar。小屏/窄窗将检查器改为可收起抽屉，主对象与聊天范围不得丢失。Project Worktree Index 提供跨 Run 排序、筛选、状态分组和深链；它不替代主导航，也不维护另一份 Task/Agent 真相。

```text
未选 Worktree
┌──────── Project 树 ────────┬────────── 主区 ──────────┬── 检查器 ──┐
│ Project                    │ Project / Run 摘要       │ 可收起     │
│  Run                       │ Worktree 清单与选择入口 │           │
│   Worktree                 │ （没有 Run App tabs）    │           │
│    Branch                  │                          │           │
└───────────────────────────┴──────────────────────────┴───────────┘
┌──────────────────────── Chat Bar：GLOBAL ─────────────────────────┐

已选 Worktree
┌──────── Project 树 ────────┬────────── 主区 ──────────┬── 检查器 ──┐
│ Project                    │ Run 标题 + Worktree 路径 │ 卡片详情   │
│  Run                       │ Inbox | Tasks | Canvas   │ Hook/Agent │
│   Worktree  ← focus        │ Workflow | BI | Plugins  │ 状态/证据  │
│    Branch                  │ 当前 App                 │           │
└───────────────────────────┴──────────────────────────┴───────────┘
┌──────────── Chat Bar：WORKTREE | GLOBAL ───────────────────────────┘
```

Worktree 选择是从管理目录进入工作区的唯一焦点切换点。用户离开 Worktree 或更换 Project/Run 时，先取消旧请求、停止旧流并复验授权；不能把上一个 Worktree 的 tab、CLI 或聊天上下文带入新的 focus。

### 3.1 导航与焦点行为

| 操作 | 结果 | 必须清理或复验 |
|---|---|---|
| 切换 Project | 加载该 Project 授权目录，保持 Project Worktree Index 入口。 | 取消旧 Project 请求；丢弃旧 Run/Worktree/App 的内存投影和未发送操作；重新解析授权。 |
| 展开 Engineering Run | 懒加载本 Run 下 Worktree；展开本身不设置 focus。 | 处理加载、分页、重试和空列表；不能因缺 Branch/宿主连接隐藏 Run。 |
| 选择 Project 或 Run（无 Worktree focus） | 主区显示 Project/Run 摘要与 Worktree 选择清单；不出现 Run App tabs 或 App 内容；聊天仅可选 GLOBAL。 | 显示“选择一个 Worktree 以打开 Run 工作区”；不显示 Task/Inbox/Canvas/Workflow/BI 页面，不挂载终端或任何依赖 checkout 的操作。 |
| 选择 Worktree | 校验同 Project/Run binding，设置 focus 并进入 canonical Worktree route；显示所属 Run 的完整 App tabs。 | 复验当前授权、binding/version 和 freshness；路由切换取消旧请求；状态不确定时不显示可写控件。 |
| 展开/选中 Branch | 展示本地分支/ref 观察和 freshness；选中只细化代码上下文。 | 不改变 Project/Run/Worktree 层级；远端操作另行检查远端授权。 |
| 切换 Run App | 保持当前授权 RunContext 和 focus，更新 `app_id`。 | 每个 App 独立重新检查 capability；撤权/版本冲突只清理受影响范围。 |
| 切换聊天 scope | `WORKTREE` 指向当前 focus；`GLOBAL` 先显示可授权目标选择/摘要。 | 每个目标分别授权；切 scope 时停止旧流、取消旧命令并防止旧回复混入新会话。 |

## 4. 页面与组件规划

| 页面/组件 | 主要内容与动作 | 空/错误/边界状态 |
|---|---|---|
| Project 选择器 | 最近/收藏 Project、授权角色、名称搜索与 Project 切换。 | 无授权 Project、目录加载失败、授权已撤回。 |
| Project / Worktree Index | 跨 Run Worktree 表格/列表；按 Runtime、Agent、Branch freshness、冲突、Hook 决策、执行占用、基础设施 readiness 筛选；进入 Worktree 或 Run。 | 以 `unknown` 表达缺 provider/陈旧观察；批量动作逐项展示权限和阻塞原因。 |
| Project Run Tree | 懒加载 Project → Run → Worktree → Branch；状态徽标显示 Agent owner、活跃 Session、checkout 锁、未提交变化摘要、最新观测时间。 | 虚拟列表；部分行失败可重试；节点被删除/移动后保持焦点可恢复。 |
| Project/Run 摘要（无 focus） | 身份、授权/成员摘要、Worktree 绑定清单、选择 Worktree 的入口；不展示 Run Apps 或任务列表。 | 仅显示摘要级 capability；明确提示需选择 Worktree 才进入该 Run 工作区；不显示业务 App 空状态来冒充已经进入工作区。 |
| Run Workspace（Worktree focus 已通过校验） | 标题与 breadcrumb；当前 Worktree/Branch 观察；Inbox、Work Items/Planning、Task Cards、Canvas、Workflow/Schedule、BI/Benchmark、Plugin tabs；统一右侧详情面板。 | 每个 tab 独立 loading/error/empty/permission/disabled 状态；不可用时显示未关闭依赖，不渲染假数据。 |
| Task Card 详情 | 目标、优先级、关系、计划、Agent 分配、执行/Session 历史、验证条件、审计和产出证据。 | Task lifecycle 与 Session lifecycle 分开；重复请求、冲突和撤权均可恢复且不丢草稿。 |
| 卡内 CLI 面板 | 绑定 Task + Run + 可选 Worktree；授权状态、启动配置、输出、取消/重接、运行预算、验证/提交结果。 | 无 Session receipt、无 sandbox、binding 不符、Hook 失败、预算未知或状态不确定时不可输入；输出有界、支持暂停/重连。 |
| Infinite Canvas | 无限画布、Task Card/Worktree/Branch/BI fact 的 typed node、布局与连接器；节点侧栏可跳到 owner 页面。 | 视口裁剪和局部加载；断开的引用显示失效原因；拖拽布局不修改业务关系。 |
| Workflow / Schedule | Run 内工作流模板、步骤状态、Schedule Rule 配置与审计。目标 Worktree/Task/Profile/HookSet 必须有效；没有远端 Branch 时仍可保存规则配置。 | 明确区分“规则配置态”与“运行态”；当前无 Schedule worker/trigger。Occurrence 在应用层仍须有 Branch，resume 前重新授权并检查版本、幂等键和当前策略。 |
| BI / Benchmark | Run 级指标、趋势、队列/成功率/成本/延迟、Hook/验证结果和版本化基准 cohort。 | 显示 freshness、coverage、缺口、样本量和公式版本；不完整覆盖保持 partial/unknown。 |
| Plugin Apps | 可插拔 App 列表、版本兼容、Run scope、capabilities、启停/卸载和权限审查。 | Manifest 不可信或 capability 被撤销时隔离/卸载；展示受影响操作与保存状态。 |
| 高级设置 → Hooks | 可视化 typed rule builder、继承来源、Run 快照版本、差异、dry-run、发布/回滚、执行结果深链；所有壳级路由通过统一的未保存草稿守卫。 | Leave/Stay 确认先于导航副作用；失败时保留草稿和焦点。历史前进/后退、外部页面链接和非壳级 imperative navigation 需要逐项接入；策略解析/审计失败 fail closed。 |
| Infrastructure 状态面板 | HostEnvironment、Multipass instance、目标 cluster identity/generation、Run binding、reservation、K3s/Agent sandbox readiness 的分层状态。 | Host 与 cluster 容量分开显示；未知或过期不可启执行；不提供任意宿主 shell/socket。测试 inventory 上限固定 2 个 Multipass 记录，不提供自动 purge。 |

## 5. RunContext、跨 App 合同与状态管理

每个视图只持有 canonical identity 与少量界面状态。授权主体由可信 session/服务端提供，客户端传入的 context 只作为目标引用并由 owner 复验。

```text
RunUiContext = {
  project_id,
  engineering_run_id,
  focus_worktree_id?,
  observed_local_branch_ref?,
  active_app_id?,
  chat_scope: WORKTREE | GLOBAL,
  context_version,
  focus_version,
  authorization_version
}
```

- Task Card、Canvas、Workflow、Plugin 与 BI 使用相同 `engineering_run_id`；引用通过含类型、owner scope 和版本的 `EntityRef` 跳转，不复制数据事实。
- Agent/Task CLI 绑定 `(Project, Run, Work Item, TaskExecutionRun, optional Worktree)`；Worktree 存在时由服务端验证同 Run 绑定。CLI 目标不可由 Branch 名称或前端路径推断。
- Canvas 创建 Task Card 使用 WorkItem owner 的原子复合命令；连接线是 Canvas 表达，不等价于 WorkItem 关系。
- HookSet digest、Infrastructure Profile/Run binding、Plugin manifest/capabilities、BI formula/cohort version 均在详情中显示摘要和来源；变更后关闭旧的敏感会话并重新授权。
- LangGraph checkpoint/thread 不拥有 Task、Git、Canvas、Schedule 或审批事实。恢复执行前重验授权、策略与资源预算，副作用使用稳定幂等身份。
- 业务事实经 owner API 与 transactional Outbox/Inbox 流转；UI 使用可重建 projection/SSE/分页查询。前端缓存、状态机和插件内存不可充当事实源。

## 6. Rust 桌面端目标结构

```text
Rust Desktop Shell
├─ Navigation/Context Store（Project、Run、Worktree focus、App、Chat scope）
├─ Feature Views（Index、Run Apps、Task Card、Canvas、BI、Settings）
├─ UI Query/Command Ports（分页、取消、幂等、版本、授权提示）
├─ Session/Auth Adapter（短期凭据句柄；不持久化秘密）
├─ Run Event Projection（有界游标、去重、重连与 drain）
└─ Plugin UI Host（版本化 manifest + 隔离 capability bridge；无任意 native code）
       ↓ 同一版本化 API / owner events
Rust 服务 owner → DB/RLS + transactional Outbox/Inbox → 其他 owner/BI/Runtime
```

桌面前端必须用 Rust 实现导航、视图状态、分页/虚拟化、异步任务编排与终端生命周期；Web 前端可以继续作为另一客户端，但不得强迫桌面端借用无界 WebView 状态或在两端复制业务逻辑。UI 线程不执行 I/O、解析大日志或同步数据库调用。所有网络/IPC 工作带取消令牌、deadline、最大结果数和字节预算；页面卸载、scope 切换和授权代次变化时取消旧任务，并忽略迟到回调。

具体 Rust 渲染器在 UI-0 阶段以相同的 Project Tree、Task Card 和 Canvas 原型比较：启动/空闲 RSS、增量 RSS、首屏时间、滚动/交互延迟、无障碍、国际化、跨平台输入法、终端嵌入、打包依赖和插件隔离。记录机器规格、数据集、profile 与原始结果后再作 ADR；未完成原型前不把框架选择写成既定事实。

## 7. 内存、并行与响应性能设计

| 热路径 | 设计约束 | 验收证据 |
|---|---|---|
| Project/Run/Worktree 树 | 只加载已展开节点；列表虚拟化、固定/测量行高、少量 overscan；筛选服务端分页或增量索引；禁止一次性构造完整 Project×Run×Worktree×Branch 树。 | 记录不同节点规模下首屏/滚动/展开延迟、峰值 RSS 和取消后残留；性能预算由 UI-0 实测定版。 |
| Task/Inbox/Run 列表 | 保持 Group DD 已定的每页 12 条、单响应 2 MiB、最多 100 页；按需请求，当前页与恢复所需状态有界缓存，不把所有历史页保存在 UI 内存。 | 验证页切换、快速筛选、并发取消、响应过大拒绝及恢复存储额度。 |
| 事件与多 Agent 状态 | 用版本化 cursor/sequence 增量更新；按 Run 公平限流；UI 只留当前 projection 和有界去重窗口；断线使用 resume hint 并从 owner 重取。 | 长时间运行、断线恢复、重复/缺口事件、慢消费者、取消/drain 下的 RSS 与一致性记录。 |
| CLI/终端输出 | 每个 Session 输出采用字节和行双重上限的环形缓冲；不可见面板暂停渲染并按策略限流；消费者慢时施加背压；不保留无限滚屏或把完整日志复制到多个组件。上限由 Runtime/Profile 固定并展示。 | 压力下 bounded RSS、可取消、重接后的游标正确；清理 Session 后释放 PTY、任务、输出 buffer 和订阅。 |
| Infinite Canvas | 基于 viewport 的可见元素索引、视口裁剪、按需加载关系/节点详情；拖动期间只保留合并后的本地几何草稿；提交走版本 CAS。 | 大画布拖动/缩放与保存不阻塞 UI；视口外元素不常驻绘制树。 |
| BI/Benchmark | 服务端聚合、时间窗/维度过滤、序列化响应硬上限；图表组件只保留当前可见范围。 | 明确 coverage/freshness；图表切换可取消旧查询；大区间被分页或拒绝。 |
| 插件 | 插件在独立隔离执行环境；声明每 Run capabilities、事件订阅与资源限额；所有调用经异步队列和背压；热卸载先撤权、取消在途调用、drain、清理 view/cache。 | 越权/恶意 Manifest、超时、无限事件、OOM/崩溃与热撤销演练；主进程不加载任意 native library。 |
| 桌面基础 | 有界 LRU/TTL、共享只读 projection、取消过期 query、异步 I/O、虚拟列表；只缓存必要 ID/version，不复制完整业务对象；内存缓存按身份/授权版本分区并在退出/登出时清理。 | UI-0 冻结冷启动/闲置/目标规模/压力态预算；后续各阶段在同一机器、同一数据集复测并报告差值。 |

性能规划不虚构机器无关的毫秒/RSS目标。UI-0 必须先定义目标机器、代表性数据规模、冷/热缓存与压力场景，测出基线并由产品门禁审批具体数值；在基线批准前先执行硬性不变量：无全量预加载、无无界队列/缓存/日志、无 UI 线程 I/O、可取消、可背压、逐 Run 公平。

## 8. 分阶段实施与依赖

| Phase | 交付 | 依赖 | 完成门 |
|---|---|---|---|
| UI-0 Rust UI 架构与基准 | 角色任务流、低保真页面图、Rust 渲染器技术原型、目标机器/数据集、内存/延迟预算草案、UI Architecture ADR。 | 当前文档与桌面壳审计。 | 同一原型完成树/卡片/画布剖析；预算、技术选型、插件隔离和测试方式经过评审；没有把 WebView 原型误标为 Rust 前端完成。 |
| UI-1 Project 与 Worktree 管理导航 | Project 选择/授权、Run 树、Worktree focus、Branch freshness 叶、Project Worktree Index、键盘与虚拟列表。 | UI-0；Run/Worktree 目录合同；WLD-0/1 的 API 按能力逐步开放。 | 有/无宿主、有/无 Worktree、陈旧 Branch、多人并行/锁/冲突、权限变化及空/错误态均清晰；导航不会将 Worktree 操作错绑到其他 Run。 |
| UI-2 Run Shell 与跨 App 上下文 | 未选 Worktree 的 Project/Run 摘要态；选中 Worktree 后的完整同级 App tabs；右侧检查器；WORKTREE/GLOBAL 底栏聊天；统一 context 切换与取消。 | UI-1；RunContext/EntityRef/owner API。 | 无 focus 时绝不挂载 Run App tabs/content；有效 Worktree focus 后才开放 tabs；每个写动作重新授权；Global 目标逐项校验；旧响应不会覆盖新 scope。 |
| UI-3 Task Card 与卡内 CLI | Inbox/Task Card 详情、Agent 分配、计划/依赖、Session 生命周期、受控终端、取消/恢复、验证与结果证据。 | UI-2；WLD-0/2，Hook/Runtime/sandbox gates。 | 当前 Task/Run/Worktree/Session 匹配才可输入；缺执行准入/provisioner/sandbox/Hook/预算时禁用并说明；独立验证和结果回写未接通前不标“可用”。 |
| UI-4 Infinite Canvas 与协作对象 | Run Canvas、WorkItem/Worktree/Branch/BI typed refs、关系检查器、从 Canvas 建 Task 的复合命令、跳转与回链。 | UI-2；Canvas 与 WorkItem owner contracts；WLD-2 中涉及 Task 的复合命令。 | Canvas 与 Task Card 同级；布局与业务关系分离；引用逐次授权；跨 owner 失败不留孤儿对象；大画布遵守视口/内存界限。 |
| UI-5 Workflow、Schedule、Hooks 与 Plugin Apps | Workflow/Loop 状态；Schedule Rule 配置与执行分态；高级设置 Hooks 和全局未保存草稿守卫；Plugin Registry/热插拔/capability 可视化、授权撤销。 | UI-2；WLD-3/5；Plugin manifest/runtime contract。 | hostless Run 可配置规则只在 Rust/PostgreSQL acceptance 完成后展示为可用配置；明确它没有触发/Occurrence/执行能力。确认守卫覆盖全部离开路径；checkpoint 不作业务事实；resume/retry 复验权限、策略和幂等；核心 Hook 不可被插件削弱；卸载取消并 drain 在途插件调用。 |
| UI-6 BI/Benchmark 与 Infrastructure 可视化 | Coverage-aware BI/Benchmark；Run/Host/Cluster/Instance/Reservation/readiness 分层详情和故障深链。 | UI-2；WLD-4/6；版本化指标和 Infrastructure Profile。 | Unknown/partial 不显示成功；host 与 cluster 容量分开；Run 固定 cluster generation；测试 Multipass inventory 仍最多 2 条记录，无自动清理。 |
| UI-7 发布质量与 Rust 桌面替换 | 主要路径端到端验收、键盘/读屏/缩放、恢复与性能复测、打包/升级/迁移说明、Web/Rust 客户端一致性矩阵。 | UI-0..6；WLD-7 对应产品事实/owner 门关闭。 | Rust 1.99.0 工具链门、授权/目标 DB/runtime、性能与内存预算、恢复/取消、插件隔离和真实数据验收均有证据；未通过的功能在界面中保持禁用。 |

依赖关系：`UI-0 → UI-1 → UI-2`；UI-3/UI-4/UI-5/UI-6 可在共享 RunContext/owner 契约冻结后分开推进；UI-7 汇总所有阶段。UI 视觉实现可以先接受控只读 projection 和禁用状态，不得用 mock/seed 模拟生产可用。WLD owner 后端未通过准入前，UI 开发不解除执行、Hook、BI ingress 或 Infrastructure 操作的 fail-closed 状态。

## 9. 总体验收清单

- 导航顺序恒为 Project → Engineering Run → Worktree → Branch；Project 是第一层，Branch 是 Worktree 子层，不以宿主连接状态改序。
- 选 Run 而无 Worktree focus 时只查看 Project/Run 摘要与 Worktree 清单，不显示任何 Run App tabs/content；点击有效 Worktree 后才显示 Inbox、Work Items、Task Cards、Canvas、Workflow、BI/Benchmark、Plugin 等同级 tabs。
- Task Card 与 Canvas 在 Run 层同级；CLI 只从 Task Card 内进入，并持有正确 Task/Run/Worktree/Session 身份。
- Chat Bar 固定并显式选择 WORKTREE/GLOBAL；GLOBAL 只扩大候选范围，不扩大授权。
- Schedule 面板把 Rule 配置状态与 trigger、Occurrence、Run admission、executor 状态分开；没有远端 Branch 的 Run 可配置规则，但不能据此显示为可执行。目标 Worktree、Task、Profile、HookSet 和 run-as 授权仍需有效。
- Hook 草稿的 Leave/Stay 保护不得在用户选择 Stay 时关闭菜单、改写最近访问记录或丢失焦点；普通同源内容链接批准后走 SPA push，Stay 不调用链接回调。浏览器 Back/Forward 仍未保护，必须显式列为限制并在进一步实现后单独验收。
- Worktree Index、Run、Task、Canvas、Workflow、BI、Plugin、Hook、Infrastructure 状态均指向单一 owner 数据/API；跨 App 变更返回 receipt、version 和可审计结果。
- 多 Agent 工作区能区分运行中、等待、冲突、锁定、授权撤销、资源不足、状态未知和已完成；状态来自可信服务，陈旧数据带时间戳。
- UI 加载、缓存、列表、事件订阅、终端输出、图表和插件资源均有界；所有长任务可取消并在取消后释放资源。
- 关键操作具备键盘可达、焦点恢复、可读状态、错误恢复与确认；颜色不是唯一状态信号。
- 所有尚缺生产 provider、身份、sandbox、目标数据库、Hook、consumer 或 BI coverage 的功能明确标为 unavailable/partial/unknown，不使用演示数据伪装成功。

## 10. 已知差距与待决项

1. Rust 原生桌面渲染器尚未通过原型比较；UI-0 前不固定具体框架。
2. 当前 Tauri + React/Vite 桌面原型的导航、Task、Canvas 和 IPC 需要按本计划重新映射；旧界面只能作为交互参考，不能把它的样例记录带入生产路径。
3. Web Run Shell 现有 no-focus Task Cards 路由与本规划不一致，需收敛为 Project/Run 摘要态；只有选中有效 Worktree 后才挂载所属 Run 的 App tabs。现有聊天和 CLI 有限实现不等于完整 Runtime/App；浏览器 Back/Forward、目标 DB、真实 session provisioner、Hook 生命周期和 BI receiver 仍开放。
4. Rust 前端的跨平台字体/输入法、屏幕阅读、窗口/多显示器、终端渲染以及插件沙箱具体协议，在 UI-0 用目标设备实测确定。
5. 基准所用硬件型号、合成目录规模、绝对 RSS/延迟阈值需在 UI-0 定版；在此之前不得声称满足了性能目标。
6. WLD-3 hostless Schedule Rule source slice 尚未通过 Rust 1.99.0 与 PostgreSQL migration/API acceptance；Occurrence 的 `target_snapshot` 是 JSONB，Branch 必需性由 Domain/materializer/admission 校验，不能在 UI 或文档中暗示数据库存在独立 Branch 列约束。

## 11. 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.3 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 根据 UI-1 Worktree 状态呈现与 WLD-5 普通同源链接保护的本轮验证证据，修正 no-focus Run 误显示 Task Cards 的导航偏差，补充双态布局示意、焦点切换合同、unknown 状态边界及后续 UI 阶段验收。 | 用户要求当前开发完成后规划新版 UI，并确认 App tabs 必须在选择 Worktree 后才显示 |
| v0.2 | 2026-10-21 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 按 dev `1d6b3338` 更新 WLD-3 hostless Rule 配置/Branch-required 执行分态和 WLD-5 全局壳导航草稿保护现状；明确 Rust/PostgreSQL 与浏览器历史导航缺口，细化 UI-5 验收。 | 当前开发轮完成并按用户要求规划新版 UI |
| v0.1 | 2026-10-21 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 基于本地 dev `6f4ebc17` 的导航、Run Shell、Task/CLI、Hooks 与 Tauri 源码证据制定 UI 信息架构、Rust 桌面性能边界、交互合同、分阶段计划和验收门；保留未实现与生产差距。 | 用户要求本轮开发后规划新版 UI |
