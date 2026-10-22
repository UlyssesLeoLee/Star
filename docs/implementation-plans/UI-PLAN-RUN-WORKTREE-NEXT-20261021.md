# UI 规划：Project / Run / Worktree 驱动的高性能桌面端

**版本：** v1.6（Project/Run 选择与展开分离；UI-1/UI-2 focused tests 共 9/9、typecheck 通过；production build 因 Node OOM 未通过；UI-0 generator/runner lane 已启动）
**日期：** 2026-10-22
**设计基线：** 本地 dev 起始提交 a5b4404a；requirements v6.91、Schedule API SRS v0.65、basic design v6.94、Schedule BD v0.63、Schedule DD v0.77、DD-WORKTREE-GROUP-001.md v5.16、实施计划 v7.14、WBS v3.7、UI 规划 v0.6。
**实现基线：** 本轮以本地 main/dev 与缓存 origin/dev 共同提交 5a051bfd8f76fbfbe8ff49a994579c60166d6ec6 为起点。WLD-1 Run-scoped API 仍无默认可信 provider、Run↔Worktree binding writer、目标 DB/RLS 验收，前端旧 Project-scoped 路径继续 fail closed。UI-1 Project/Run 树现将选择与展开分开：Project 链接进入 Project Worktree Index 并设定 Project scope，Run 链接进入无焦点概览并清除旧 Worktree focus；UI-1 三项 focused tests 与 UI-2 六项 focused tests 共 9/9 通过，TypeScript typecheck 通过。Next.js production build 已尝试但以 Node OOM/exit 134 失败，采样时主机约有 0.81 GB 可用内存。真实浏览器导航、屏幕阅读器和 owner-backed 完整闭环未验收；组件级键盘 disclosure 用例通过。UI-0 benchmark generator/runner 已在登记的独立分支和 worktree 启动，实测与 renderer 决策仍开放；workload manifest SHA-256 为 D06D57B7CA0D30FDB9960E9FC8757AA936C991ADE1993F2F59EA0CE9B5118328，目标机静态规格与性能测量状态见 WBS 和 Rust Desktop ADR。
**状态：** 目标交互与分期规划；不是桌面端或所有 Run App 已完成的声明。

## 0. 目的与本轮结论

把当前 Worktree-first 目标整理成可分期实施的产品 UI 方案。产品主线是 Project 内多 Run、多 Agent checkout 的可见归属、冲突与生命周期管理；Run 是协作、任务和分析的工作区；Worktree 是明确可操作的本地 checkout；Branch 是 Worktree 下可信本地观察到的末级状态。导航只有在用户选择具体 Worktree 后才展示该 Run 的 App tabs；只选中 Project 或 Run 时，主区仅展示上下文摘要和可选 Worktree，不提前挂载 Run App。

本轮不把当前 Web/React 页面或 Tauri 原型当作最终桌面体验。Web 端已有导航、授权 RunContext、Task Card 与受控终端的局部实现；Tauri 壳的独立界面仍由 React/Vite 渲染，Rust IPC provider 尚不完整。目标桌面 UI 按 Rust 原生前端规划，先用代表性页面测量渲染器和资源成本，再冻结技术选型。

v0.8 以 v0.7 的屏幕规格为底稿，加入 WLD-1 Run-scoped lifecycle API 的当前实现边界，细化 Worktree 生命周期检查器、操作预览、Run API 接入门和分期 WBS。当前仍缺 Run↔Worktree 持久绑定 owner、真实 provider、目标 PostgreSQL/RLS 验收及前端 Run API 迁移；本文将这些写为接口/实现前置条件，不把现有 API seam、预览或不完整投影描述成已可用产品能力。

v0.4 的导航决策落到清晰的双态：左侧 Project → Engineering Run → Worktree → Branch 树始终承担目录管理；只选 Project/Run 时，右侧工作区显示授权摘要和 Worktree 选择，不挂载 Run App；选中并重新验证 Worktree 后，右侧工作区才显示该 Run 的同级 App tabs 与内容。UI-2 的 no-focus 门已在 dev 集成；右侧 picker 已支持最多 4 页/200 条、已加载项搜索、重复游标与重复 Worktree ID 保护，并拒绝超过 50 条单页或 200 条总量的响应；服务端全目录搜索仍需 owner/API 合同。GLOBAL 聊天范围已显示，但聊天 Runtime 和发送仍禁用。

## 1. 现状基线与差距

| 区域 | 当前证据 | UI 规划中的处理 |
|---|---|---|
| 左侧 Project 目录 | `ProjectRunTree` 按 Project → Engineering Run → Worktree → Branch 展示，并使用固定视口、行高、overscan 和键盘树导航；UI-1 切片分别呈现人工状态、机器状态、Agent 分配、Session/Runtime 绑定、归档状态和 binding 版本。锁、变更详情、观测时间、Branch 来源/新鲜度尚无可信观测，显示 unknown。 | 延续树形导航；补齐有 owner 证据的冲突、锁、freshness 和生命周期状态；Project Worktree Index 只作跨 Run 汇总，不取代主导航。 |
| Run Shell | UI-2 已改为无 focus 时只读取该 Run 授权 Worktree 目录、显示摘要/选择器和 GLOBAL 范围；不挂载 Task Cards、BI、App tabs 或 App 数据请求。focused Run Shell tests 6/6 与 TypeScript typecheck 通过；Next.js production build 尝试因 Node OOM 退出 134（执行时约 0.81 GB 可用内存），浏览器/e2e、键盘/读屏和窄视口验收仍开放。选择器在首屏与后续页均拒绝超量响应和页内/跨页重复身份。选中 Worktree 后，目标规范路由再次获取并验证 RunContext 才挂载现有 App 面板；当前只有 Task Cards 与 BI 面板，其他 tabs 明确待迁移。 | 保持 Worktree-first 焦点门；补齐分页/搜索和右侧工作区布局；按 owner capability 呈现可用、只读、不可用或等待状态。GLOBAL 聊天仍无 Runtime/send。 |
| Worktree 生命周期 | WLD-1 已增加 Run-scoped repository discovery、import candidate、create/import API 与显式 provider 注入边界；缺少 provider 或 Run 当前 Repository binding 时 fail closed。Ready receipt 需 Run↔Worktree binding 身份/版本，但此切片不创建该绑定。前端仍调用旧 Project-scoped paths，API/DB/Rust/provider 验收未完成。 | 迁移 UI 到 Run-scoped API 合同；provider、binding owner、目标 DB/RLS 与持久 receipt 未验收前，操作显示不可用原因，不回退到 Project API、不做乐观成功。 |
| Task Card / CLI | Task Card 有分页创建与读取切片；终端必须对应当前 Task、Worktree 和经服务端授权的 Session。无 provisioner、授权或状态不确定时 fail closed。 | 将 CLI 收入卡内执行面板；补全 Agent 分配、运行状态、取消/重接、验证证据与结果回写的界面闭环。不得显示假终端或把 Session 结束当作任务完成。 |
| Canvas | 有独立 Canvas 页面和 Group Canvas/Worktree 图相关实现，但 Run 内统一入口和完整 owner 生产闭环尚未完成。 | Infinite Canvas 与 Task Card 在 Run 层同级；Canvas 保存几何、布局和 typed 引用，Task/Worktree/关系事实仍由其 owner API 管理。 |
| 聊天 | scope 选择与授权目标目录存在局部实现；当前 Run Shell 的输入和发送仍禁用。 | 固定底栏，明确 `WORKTREE` / `GLOBAL`；发送前展示有效目标、权限和上下文摘要。GLOBAL 不扩大授权。 |
| Hooks | 高级设置内已有可视化 Hook 编辑页。本轮 WLD-5 扩展普通同源内容链接守卫，并通过公开 App Router API 在批准后执行 SPA 导航；Stay 阻止路由和链接回调。定向 4 个测试文件 33 项与 TypeScript typecheck 通过。浏览器 Back/Forward 仍不受保护；真实宿主身份与执行期原生 Hook 链路仍开放。 | 保留“高级设置 → Hooks”；各可控导航出口统一 Leave/Stay；历史前进/后退限制明确显示并作为后续门禁；补足继承/版本差异、模拟评估、发布回滚和决策深链。 |
| Schedule / hostless Run | WLD-3 已增加没有远端 Branch 的 Run 规则读写/配置路径；目标仍须绑定当前本地 Worktree、Task、repository、Profile 和 HookSet，Branch 仅由授权 Run 范围提供。Rust 与 PostgreSQL 门禁未运行，且没有 Schedule worker/trigger/executor。 | 将“规则配置已保存”“规则启用配置态”和“Occurrence/Run 实际执行”显示为不同状态；无远端 Branch 只允许配置，不显示为可触发或可运行。触发仍受 Domain/materializer 与 Run-owner Branch 必需门保护。 |
| 桌面端 | `crates/star-desktop` 是 Tauri 2 + React/Vite 壳和 Rust IPC；Task provider 缺失时失败关闭，旧任务演示回退已移除。当前不能称为 Rust 原生前端闭环。 | 建立 Rust UI 目标架构，与 Web 端共享 API/owner/event 契约；Rust IPC 只做有界 adapter，不拥有第二份业务事实。 |

代码与报告证据：`frontend/src/components/Sidebar.tsx`、`frontend/src/components/run/ProjectRunTree.tsx`、`frontend/src/components/run/RunWorkspaceShell.tsx`、`frontend/src/components/run/RunTaskCardsPanel.tsx`、`frontend/src/lib/group/taskCliPresentation.ts`、`crates/star-desktop/frontend/src/App.tsx`，以及 `docs/reports/WLD1-LIFECYCLE-OP-OUTBOX-20261021.md`、`docs/reports/WLD1-LIFECYCLE-PROVIDER-20261022.md`、`docs/reports/WLD2-TASK-CLI-UI-SLICE-20261020.md`、`docs/reports/WLD5-HOOK-POLICY-UI-SLICE-20261020.md`。这些证据只覆盖报告声明的代码/测试范围；不证明生产身份、目标 DB、Runtime 或全产品闭环。

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

右侧工作区（随左树焦点变化，不是导航树的新层级）
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
| 选择 Project 或 Run（无 Worktree focus） | v0.4 实现已显示 Run 摘要、授权 Worktree 首页面和 GLOBAL 范围；不会挂载 Run App 面板或读取其数据。 | 选择器有下一页时明确提示左侧目录加载更多；后续补右侧搜索/分页，错误可重试、空列表说明受控创建路径。 |
| 选择 Worktree | UI-2 提供不预取的规范 Worktree 链接；导航后由目标路由重新校验同 Project/Run binding，服务端 focus 与路由 Worktree 一致后才显示所属 Run App tabs。 | 不在列表点击时乐观设置 focus；复验当前授权、binding/version 和 freshness；路由切换取消旧请求；状态不确定时不显示可写控件。 |
| 展开/选中 Branch | 展示本地分支/ref 观察和 freshness；选中只细化代码上下文。 | 不改变 Project/Run/Worktree 层级；远端操作另行检查远端授权。 |
| 切换 Run App | 保持当前授权 RunContext 和 focus，更新 `app_id`。 | 每个 App 独立重新检查 capability；撤权/版本冲突只清理受影响范围。 |
| 切换聊天 scope | `WORKTREE` 指向当前 focus；`GLOBAL` 先显示可授权目标选择/摘要。 | 每个目标分别授权；切 scope 时停止旧流、取消旧命令并防止旧回复混入新会话。 |

### 3.2 下一轮状态矩阵

| UI 状态 | 左侧 Project 树 | 中间工作区 | Chat Bar | 进入下一状态的条件 |
|---|---|---|---|---|
| Project 已选、Run 未聚焦 | Project → Run → Worktree → Branch 常驻；展开仅加载目录。 | Project 摘要和 Run/Worktree 管理入口；不展示 Run Apps。 | `GLOBAL` 可见；消息发送仍受 Runtime 能力门控。 | 选择一个 Run 或展开其目录。 |
| Run 已选、无 Worktree | 树继续承担跨 Run 管理；选中行可见但不设置 checkout focus。 | Run 摘要与已授权 Worktree 搜索/分页清单；没有 App tabs、App 内容或 App 数据查询。 | `GLOBAL`；无 Runtime 时显示不可用原因。 | 从清单或左树打开 Worktree 规范路由。 |
| Worktree 正在验证/切换 | 标识目标 Worktree，旧 focus 的敏感动作立即禁用。 | 只显示验证进度；不保留旧 Worktree 的 App 或 CLI 输入。 | 暂停 `WORKTREE` 发送并停止旧流；`GLOBAL` 保持独立授权。 | 当前 Run binding、Worktree binding/version、身份和 freshness 全部复验成功。 |
| Worktree 已验证 | Worktree 行成为明确 focus；Branch 仍是 checkout 下的观察叶。 | 展示所属 Run 的同级 App tabs；每个 App 再检查自己的 capability。 | `WORKTREE` 仅绑定已验证 checkout；可切到 `GLOBAL`。 | 导航到另一 Worktree/Run，或授权过期。 |
| 授权拒绝、版本过期、provider 不可用 | 保留目录身份并标记失败/未知，不推断可写状态。 | 隐藏或卸载 Run Apps；提供重试、重新授权或回到目录入口。 | 关闭失效的 `WORKTREE` scope；旧流/命令不得继续。 | 明确恢复条件达成后重新走验证态。 |

状态矩阵的关键验收是切换失败时没有短暂显示旧 App、旧 CLI 输出或旧 Worktree 聊天；取消中的查询、事件订阅和终端输出都应释放。后续视觉稿至少覆盖 Run 无焦点、有效 Worktree、切换/验证失败三种宽屏与窄屏布局，并包含键盘焦点顺序和可读状态文案。

## 4. 页面与组件规划

| 页面/组件 | 主要内容与动作 | 空/错误/边界状态 |
|---|---|---|
| Project 选择器 | 最近/收藏 Project、授权角色、名称搜索与 Project 切换。 | 无授权 Project、目录加载失败、授权已撤回。 |
| Project / Worktree Index | 跨 Run Worktree 表格/列表；按 Runtime、Agent、Branch freshness、冲突、Hook 决策、执行占用、基础设施 readiness 筛选；进入 Worktree 或 Run。 | 以 `unknown` 表达缺 provider/陈旧观察；批量动作逐项展示权限和阻塞原因。 |
| Project Run Tree | 懒加载 Project → Run → Worktree → Branch；状态徽标显示 Agent owner、活跃 Session、checkout 锁、未提交变化摘要、最新观测时间。 | 虚拟列表；部分行失败可重试；节点被删除/移动后保持焦点可恢复。 |
| Project/Run 摘要（无 focus） | 身份、授权/成员摘要、Worktree 绑定清单、选择 Worktree 的入口；不展示 Run Apps 或任务列表。 | 仅显示摘要级 capability；明确提示需选择 Worktree 才进入该 Run 工作区；不显示业务 App 空状态来冒充已经进入工作区。 |
| Run Workspace（Worktree focus 已通过校验） | 产品目标是右侧工作区展示标题与 breadcrumb、当前 Worktree/Branch 观察、Inbox、Work Items/Planning、Task Cards、Canvas、Workflow/Schedule、BI/Benchmark、Plugin tabs 和统一详情面板。当前 Run Shell 只挂载 Task Cards/BI 面板，其余 tabs 禁用并标待迁移。 | 每个 tab 独立 loading/error/empty/permission/disabled 状态；不可用时显示未关闭依赖，不渲染假数据。 |
| Task Card 详情 | 目标、优先级、关系、计划、Agent 分配、执行/Session 历史、验证条件、审计和产出证据。 | Task lifecycle 与 Session lifecycle 分开；重复请求、冲突和撤权均可恢复且不丢草稿。 |
| 卡内 CLI 面板 | 绑定 Task + Run + 可选 Worktree；授权状态、启动配置、输出、取消/重接、运行预算、验证/提交结果。 | 无 Session receipt、无 sandbox、binding 不符、Hook 失败、预算未知或状态不确定时不可输入；输出有界、支持暂停/重连。 |
| Infinite Canvas | 无限画布、Task Card/Worktree/Branch/BI fact 的 typed node、布局与连接器；节点侧栏可跳到 owner 页面。 | 视口裁剪和局部加载；断开的引用显示失效原因；拖拽布局不修改业务关系。 |
| Workflow / Schedule | Run 内工作流模板、步骤状态、Schedule Rule 配置与审计。目标 Worktree/Task/Profile/HookSet 必须有效；没有远端 Branch 时仍可保存规则配置。新 Occurrence 的 `target_snapshot.rule_snapshot.target.branch_id` 由 SQL `NOT VALID` CHECK 限制为非 nil UUID，旧行不回填。 | 明确区分“规则配置态”与“运行态”；当前无 Schedule worker/trigger。resume 前重新授权并检查版本、幂等键和当前策略；PostgreSQL migration/API 与 Rust gates 通过前不显示为可用执行。 |
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

### 8.1 v0.5 UI 规划：先后顺序与可审查产物

| 顺序 | 工作包 | 设计/实现重点 | 验收与停止条件 |
|---|---|---|---|
| P0 | UI-0 Rust 桌面 UI 基准与 ADR | 先比较候选 Rust 渲染路径；用 Project 树、Worktree 清单和 Task 卡片作代表样本，记录目标机器、数据规模、冷/热状态、启动/滚动/切换延迟及 RSS。 | 先给出可重复测量的基线、内存上限与帧/交互预算，再决定桌面 UI 技术栈；禁止从当前 React/Tauri 壳直接推导 Rust UI 已完成。 |
| P1 | UI-1/2 Worktree 导航闭环 | 左树分页/虚拟化、Worktree 搜索与右侧选择器分页、稳定焦点与切换中状态；补充 back/forward 与失效授权下的页面恢复设计。 | Project → Run → Worktree → Branch 顺序不变；未验证 focus 时零 App 数据请求；切换时清理旧身份与流；目录大规模加载必须按页、可取消。 |
| P1 | UI-2 Chat Bar 状态与范围 | 明示 `GLOBAL`/`WORKTREE` 目标身份、授权范围和不可用原因；规划全局目标选择及发送前确认。 | 当前 Runtime/send 未接通时按钮保持禁用；启用前须证明新 scope 单独授权、切换取消旧流、失败不重放变更。 |
| P2 | UI-3/4 Task Card、卡内 CLI 与 Canvas | Task/CLI 显示绑定身份、Agent 分配、预算、取消/重接和验证证据；Canvas 与 Task Card 在 Run 下同级，以 typed reference 打开 owner 页面。 | CLI 仅绑定真实 Task/Run/Worktree/Session；没有 provider、sandbox、Hook 或独立验证时不能输入/标可用。画布拖动不直接改业务关系。 |
| P2 | UI-5/6 Hook、Plugin、Schedule、BI 与基础设施 | Hooks 放在高级设置并用 typed 表单可视化；Run App 只显示 capability 摘要和 owner 深链；Schedule 配置态与执行态分开；BI 标 freshness/coverage；Multipass/Cluster 分层。 | 核心 Hook 由 Rust 原生门控；Plugin 隔离且可撤销/drain；Run Infrastructure Binding 固定 cluster generation；测试 inventory 最多 2 条且无自动 purge。 |
| P3 | UI-7 发布与迁移 | 汇总键盘/读屏/窗口缩放、错误恢复、性能、升级和 Web/Rust 一致性。 | 只对已过身份、数据、执行、Rust 工具链和目标部署门的能力开放；其他状态可解释且 fail closed。 |

实现进入 P1 前先交付三态交互稿、焦点/授权转换图和状态文案表；UI-0 通过前可以继续 Web 端安全修正，但不得冻结 Rust UI 框架，也不得绕过 Run owner 或用 seed/mock 充当结果。

依赖关系：`UI-0 → UI-1 → UI-2`；UI-3/UI-4/UI-5/UI-6 可在共享 RunContext/owner 契约冻结后分开推进；UI-7 汇总所有阶段。UI 视觉实现可以先接受控只读 projection 和禁用状态，不得用 mock/seed 模拟生产可用。WLD owner 后端未通过准入前，UI 开发不解除执行、Hook、BI ingress 或 Infrastructure 操作的 fail-closed 状态。

本轮交付状态：UI-2 的 no-focus 挂载门、授权 Worktree 首页面、禁预取规范链接和 WORKTREE/GLOBAL 焦点门已实现；两项聚焦 Vitest、TypeScript、CypherGraph Guardian 和 diff whitespace checks 通过。UI-2 仍未完成完整右侧工作区、跨 App inspector、目录分页/搜索、聊天发送及 Browser Back/Forward。WLD-3 本轮仅加 Occurrence 快照持久化 CHECK；静态契约测试通过 3/3，数据库与 Rust 验收未运行。以上均不替代 UI-0 的 Rust 渲染器与资源基准；本文件 v0.5 新增 UI 状态矩阵和下一轮排序，不代表这些工作已实现。

## 8.2 v0.6 新版 UI 规划：多 Agent Worktree 控制台

### 规划目标与版本边界

本版优先解决 Project 内多个 Agent 同时开发时 Worktree 难以归属、运行状态分散、冲突与生命周期操作不可控的问题。主导航仍固定为 Project → Engineering Run → Worktree → Branch；Project Worktree Index 是跨 Run 汇总入口，不改变该树形层级。用户先选 Project，再进入 Run；只有选择并验证某个 Worktree 后，右侧才加载该 Run 的 App 工作区。没有远端宿主身份时，Project、Run、Worktree 与本地 Branch 的管理路径仍完整可用。

本版规划覆盖导航、Worktree 状态与动作、Run 工作区、Task Card/CLI、Canvas、聊天范围以及插件/Hook/Loop/BI 的信息呈现。实施按 owner API、身份和 Runtime 能力逐项开放；缺少 provider、权限、持久化回执或沙箱时，界面显示明确的不可用原因，不提供假的成功状态。

### 版本首页结构

| 区域 | 职责 | 主要信息 |
|---|---|---|
| Project 导航 | 选择 Project，进入该 Project 的 Run 树；提供 Project Worktree Index 汇总入口。 | Project 身份与权限；Run 数量；跨 Run 的 Worktree 摘要、过滤器和更新时间。 |
| Run 树 | 管理 Project 下的 Engineering Run。 | Run 身份、成员/策略摘要、Host/SCM 关联（可选元数据）、Worktree 数量与待处理风险。 |
| Worktree 树节点 | 管理 Run 绑定的本地 checkout，是多 Agent 状态的主要可见单元。 | checkout 所有者、Agent/Session、关联 Work Item、工作目录、运行态、锁/租约、变更摘要、最近观察时间与 freshness。每类状态来自各自 owner，未知值显示为 unknown。 |
| Branch 叶节点 | 展示 Worktree 当前本地 checkout branch/ref 与提交观察。 | 本地 ref 名、HEAD、相对基线 ahead/behind、最近同步/观察时间；远端 Branch 身份另列且可为空，不推导远端状态。 |
| Worktree 工作区 | 用户选中并验证 Worktree 后出现，标题显示 Project / Run / Worktree 与本地 Branch 摘要。 | Run 级同级 App tabs：Inbox/Work Items、Task Cards、Canvas、Workflow/Schedule、BI/Benchmark、授权 Plugin Apps。Task Cards 与 Canvas 是同级 App；Worktree focus 只是 checkout 上下文，不扩大 Run 授权。 |
| 详情检查器 | 按需显示选中实体详情和可审计动作。 | Agent/Session/Task 绑定、HookSet/Profile 版本、锁与冲突事实、来源/时间、命令预览与最终 receipt。 |
| 固定 Chat Bar | 在全局和 Worktree 之间显式选作用域。 | GLOBAL 或 WORKTREE 标签、目标身份、授权摘要和 Runtime 可用状态；切换 scope 会取消旧流并丢弃旧身份响应。 |

### Worktree 控制视图

Project Worktree Index 提供跨 Run 的表格/列表、搜索、状态筛选和风险排序；树仍是首要导航。每条 Worktree 记录将状态分开显示，不压成单个含糊的健康灯：

- **归属**：Project、Run、Worktree ID、负责人、当前 Agent 与关联任务。
- **执行**：Agent/Session 生命周期、Runtime 与执行预算状态；提交/排队/启动/取消/恢复各显示自己的 owner 事实。
- **checkout**：本地目录标识、Branch/HEAD、dirty/untracked 摘要、相对基线差异与最近观测时间。
- **并行冲突**：文件/资源冲突、锁/租约 owner、阻塞操作及新鲜度；不确定或过期状态带时间戳并禁止危险操作。
- **策略与资源**：HookSet/Profile 版本、授权结果、资源上限和等待原因；敏感策略通过深链进入高级设置 → Hooks。
- **动作**：创建/导入、绑定/重绑、交接、归档/恢复、合并准备、清理。危险动作先预览影响和阻塞项，提交后显示 owner receipt、审计结果与刷新状态。

自动 Agent 分配在 Worktree/Task 详情里显示候选 Agent、负载/能力/冲突理由、被分配的 Run 与 Worktree、预算及可取消状态；用户可确认、重新分配或查看阻塞原因。UI 不直接修改 Agent/Task/Worktree 的 owner 事实，不将建议状态展示为已分配。

清理或删除路径必须重新读取 retention lock、绑定/租约、Agent 进程、PTY 与 Runtime 操作状态。dirty/untracked 内容、活动执行、未知 freshness、权限缺失或状态冲突时默认阻止清理，给出可恢复的下一步；不得用 UI 直接 purge Worktree、Multipass 记录或宿主数据来绕过门禁。合并准备界面区分本地 Branch/ref、变更集、目标基线与验证结果；远端 SCM 操作单独检查连接器和权限。

### 核心用户流程

~~~mermaid
flowchart TD
  P[选择 Project] --> R[展开 Engineering Run]
  R --> W[查看 Worktree 目录与跨 Run 汇总]
  W --> V{选择并验证具体 Worktree}
  V -->|授权与绑定有效| A[打开该 Run 的 App tabs]
  V -->|过期/拒绝/未知| E[保留目录并显示恢复原因]
  A --> T[Task Card 与卡内 CLI]
  A --> C[Canvas 与 typed owner references]
  A --> L[Workflow / Schedule / Loop 状态]
  A --> B[BI / Benchmark coverage]
  A --> H[授权的 Plugin Apps]
  A --> G[Chat Bar: WORKTREE 或 GLOBAL]
  T -->|取消/恢复/验证| A
  C -->|打开 canonical owner 页面| A
  L -->|resume 前复验授权/版本| A
~~~

| 用户任务 | 交互步骤 | 安全与可恢复门 |
|---|---|---|
| 找出哪个 Agent 正在修改哪个 checkout | Project Worktree Index 筛选 Agent/Session/任务；打开 Worktree 查看归属、Branch、锁和更新时间。 | 状态按 owner 来源分栏；陈旧或未知信号不显示为空闲。 |
| 为任务分配 Agent 和 Worktree | 打开 Run 的 Task Card；查看候选 Agent 和 Worktree 负载/冲突；确认后显示已提交的 assignment receipt。 | 先复验 Project/Run 权限、Worktree binding、HookSet/Profile 和预算；UI 不在本地乐观标记分配成功。 |
| 在任务卡中运行 CLI | Task Card 显示 Task/Run/Worktree/Session 身份，进入卡内 CLI 面板并查看受控输出。 | 只有服务端授权 Session receipt、sandbox、原生 Hook、预算和当前 binding 全部有效才可输入；否则保持只读/禁用并解释原因。 |
| 准备合并或交接 Worktree | 检查 Branch/HEAD、变更、锁/租约、活动执行和验证证据；预览目标并提交明确的操作。 | 每步由对应 owner API 执行，使用幂等键、审计/Outbox 和可追溯 receipt；冲突或不确定时停止，不丢弃文件。 |
| 从 Canvas 管理协作对象 | 搜索/放置 Task、Worktree、Branch 或分析卡片；点击节点打开其 owner 页面。 | Canvas 保存布局和 typed 引用；业务关系只经对应 owner 的 API 变更，跨 owner 命令失败可恢复。 |
| 查看 Loop 与分析反馈 | Run 工作区打开 Workflow/Schedule 或 BI/Benchmark；沿事件/coverage 深链到 Task、Hook 决策、Worktree 与基础设施事实。 | checkpoint 不作为 Task/Run/Git 业务事实；未 materialize/未接纳/未执行分别显示；BI 明示 freshness、coverage、样本量和公式版本。 |

### 焦点切换与危险操作状态

切换顺序固定为 current focus → stop/cancel old streams → clear old scoped view state → resolve target → recheck auth/binding/version → mount target App tabs。只有最后一步成功才展示目标 Run App。旧 Worktree 的 CLI 输入、终端流、Canvas selection、Inspector selection 和 chat response 都不能跨焦点残留。

| 操作 | 预览内容 | 确认后回执 | 必须阻止条件 |
|---|---|---|---|
| 创建/导入 Worktree | Project/Run、源目录/仓库、目标本地 ref、资源预算、冲突检测。 | 新 Worktree ID、provider operation ID、初始观测与审计事件。 | Project/Run 不同域、目录冲突、provider/容量未知、未经授权。 |
| 重绑/交接 | 原/新 Run、Agent/Task、Session/PTY、锁/租约、策略与权限影响。 | 新 binding version、原/新 owner receipt、Outbox/event ID。 | 活跃运行未 drain、跨 Project、版本冲突或任何 owner 状态未知。 |
| 合并准备 | 本地源 ref、目标 ref、提交差异、工作区 dirty 状态、验证/Hook 结果。 | 操作方、目标 identity、提交结果或可恢复冲突记录。 | 有未解决冲突、dirty data 未确认、验证失败、目标授权不完整。 |
| 归档/恢复/清理 | retention lock、Run/Task/Agent 绑定、存活进程、PTY、Runtime、文件保留和目标路径。 | 独立 operation ID、状态迁移、Audit 与 Outbox 结果。 | active lease/session/process、freshness 未知、未授权或历史连续性缺失。 |

### 新版本交付顺序

以下顺序复用 UI-0..UI-7，不把所有 App 同时塞入首个可用版本：

| 次序 | 交付包 | 设计产物 | 实施门 |
|---|---|---|---|
| 0 | **UI-0：Rust 桌面基准与架构决策** | 项目树、Worktree 列表、Task 卡/Canvas 代表性原型；输入/滚动/切焦/窗口变化、冷启动与闲置/压力测量计划；Rust UI renderer 与 IPC/数据层 ADR。 | 测量目标机器与真实数据集后再定 RSS/交互预算；无 UI 线程 I/O、无限日志缓存或主进程任意插件加载。 |
| 1 | **UI-1：Project/Run/Worktree 控制台** | Project 树、Worktree Index、Run 无 focus 摘要、Worktree 状态详情与危险操作预览；键盘/窄屏/空态稿。 | Project → Run → Worktree → Branch 稳定；状态来源/观测时间可见；未知/陈旧/活动任务不能触发破坏操作。 |
| 2 | **UI-2：Run 工作区与跨 App 焦点** | 右侧同级 App tabs、分页/搜索 Worktree picker、Inspector、Chat scope 与焦点切换图。 | Worktree 授权成功后才挂载 App；每个写操作重新授权；切换取消旧流/响应；失效后清理旧敏感状态。 |
| 3 | **UI-3：Task Card 与受控 CLI** | 任务详情、Agent 选择/分配、Session、CLI、取消/恢复、验证证据和结果回写。 | 无 provisioner/sandbox/Hook/当前授权/预算时不开放输入；只认 Task owner 的最终状态。 |
| 4 | **UI-4/5：Canvas、Workflow、Schedule、Hooks 与 Plugin Apps** | Canvas/Task sibling tabs；Loop timeline；Rule config 与执行状态；可视化 Hooks 和 plugin capability/lifecycle 管理。 | Typed EntityRef 与 owner API；核心 Hook 不可被 Plugin 绕过；插件撤销先停止新调用并 drain 在途操作。 |
| 5 | **UI-6/7：BI/Benchmark、Infrastructure 与发布** | Coverage-aware dashboard、Host/Cluster/Reservation 分层视图、无障碍/升级/跨客户端一致性矩阵。 | WLD-4/6/7 各自验收；未知/部分覆盖不显示成功；测试 VM inventory 保持最多 2 个记录且无自动 purge。 |

### Rust 桌面性能设计输入

1. **分层按需加载**：启动先加载 Project/Run 摘要，展开 Worktree 时按页请求，选中 Worktree 后才加载其 Run App；Canvas 按视口取数，Task transcript/CLI 输出按需订阅。
2. **有界 view model**：客户端只缓存可见行、少量前后页、当前焦点与当前 App 数据；每个 Worktree 的日志/事件/Canvas scene 都有上限和淘汰策略；Project 切换清理旧 Run scope 缓存。
3. **UI 与 I/O 分离**：Rust UI 主线程只处理输入与小型 view diff；API、数据库、Git observation、文件扫描、图表聚合和插件 IPC 放在可取消后台任务。所有队列有界、可观测、带 backpressure 和 owner fairness。
4. **事件合并**：Agent stdout、文件变更与 Run 事件进行批量合并/节流；维护最终状态的 owner event 不可丢，纯展示进度可 coalesce，记录丢弃计数和重同步 cursor。
5. **资源分层**：按 Project、Run、Worktree、Agent、Plugin 分层预算 CPU、RAM、打开句柄、流和渲染对象；低优先级 BI/Benchmark 与缩略图工作不可饿死当前交互。
6. **可复测指标**：UI-0 记录冷/热启动、Project Tree 可交互时间、Worktree 滚动/切换 p50/p95、键盘输入到反馈延迟、空闲/峰值 RSS、终端缓冲、Canvas 可见对象数、事件积压和取消 drain 时间；经目标机器测量后再批准绝对阈值。

### 本轮验收与后续交付物

本次只完成 UI 规划更新，不实现新页面、不放开任务/计划执行或 BI ingress。下一实现批次的审查包应包含：

1. 5 张低保真状态稿：Project/Worktree Index、Run 无 focus、Worktree focused workspace、Worktree 生命周期操作预览、Task Card 中 CLI disabled/enabled 两态。
2. Worktree 归属/锁/dirty/Agent/Session/Task/Branch/freshness 的状态字典及来源 owner 映射表。
3. Project/Run/Worktree/Branch 的键盘树交互、焦点管理、窄屏折叠和读屏顺序说明。
4. UI-0 Rust renderer/IPC/插件隔离 ADR 与可重复性能测量方案；测量前不设置没有基线依据的 RSS/毫秒硬值。
5. 每个界面动作到 API/owner、授权/version、幂等键、receipt/审计、失败恢复与独立验证的追踪矩阵。

开工顺序：先评审并冻结第 1–4 项的设计输入，再按 UI-0 → UI-1 → UI-2 推进基础导航/焦点，然后 UI-3/UI-4/5/UI-6/7 与 WLD owner 门并行。此阶段不代表视觉稿、Rust 前端、运行时或生产业务功能已经完成。


## 8.3 v0.7 屏幕级交互规格：Project 选择、Run 概览与 Worktree 控制

### 本版范围和证据边界

v0.7 是 UI 规划规格，不是页面实现。Project、Run、Worktree 必须分别表达“选中”与“展开/加载”；只有通过 RunContext 验证的 Worktree 才建立代码 checkout focus。选中 Project 或 Run 不得隐式聚焦某个 Worktree，不得挂载 Run App，也不得请求 Task、Canvas、BI 或 Plugin 内容。

本版审查到的当前状态限定如下：Project 与 Run 各自有独立的展开按钮和选择链接；Project 选择进入 Project Worktree Index，Run 选择进入无焦点概览，Worktree 才建立验证态焦点。当前仅有组件测试覆盖导航目标与展开不触发选择，浏览器导航、完整 roving focus、读屏和视觉验收仍开放；Project 投影只含 ID/role，界面可能只能显示 ID。Run Worktree 投影含 Agent、Session、Runtime ID 和 binding 版本，但没有可信 lock、dirty、ahead/behind、观测时间、Branch freshness 或 assignment receipt。旧 Project Worktree Index 有部分 checkout 字段，却没有 Engineering Run ID，不能据此把记录归入某个 Run 或构造 Run 深链。创建/导入入口以 Project/Repository 为范围且没有 Run binding receipt；归档/恢复只改变管理状态，不执行 Git checkout 删除或宿主数据清理。当前 Run Shell 的 Task Cards 与任务事件页是局部面板；事件页不能表述为完整 BI/Benchmark。旧 Group 路由含本地 store 与预览回退，不作为生产工作区证据。以上字段/API 能力补齐并验收前，UI 只能显示明确的 unavailable/unknown 状态。

性能源码边界也按实况记录：共享 Web 树虽虚拟化已挂载行，发布更新仍会重建完整扁平快照；页大小为 50、父节点缓存最多 16 个且每个最多 4 页。当前 Tauri Worktree 视图默认生成 120 个 fixture 节点、上限 500，并逐节点绘制 SVG；Worktree/Canvas IPC 仍有 MockDb fixture。现有终端投影上限按每 pane 1,000,000 字符计算，输出时拼接已有字符串；字符数不能限制 UTF-8 字节或峰值 RSS。共享 Canvas 路径会遍历完整元素/缩略图并对每条连接逐次查找端点。当前任务事件页每页 50 条、最多保留 100 个分页游标、响应上限 512 KiB；它仍是事件浏览而非服务端 BI aggregation。这些都是待测旧实现的参考，不是未来架构预算或生产能力证明。

### 主界面布局

| 区域 | 规划职责 | 关键行为与状态 |
|---|---|---|
| 左侧 Project 目录 | 作为唯一主导航根，显示 Project → Engineering Run → Worktree → Branch。顶部可切换/搜索 Project，树中 Project 是第一层。 | Project 名称由 Project owner 投影提供；缺少名称时显示稳定 ID，不造假名称。行上的展开按钮和选择按钮分开。树宽度可调整；小屏可折叠为目录抽屉。 |
| 中间工作区 | 根据当前选择显示 Project 概览、无焦点 Run 概览、Worktree 验证态、已聚焦 Run 工作区或 Branch 只读详情。 | Project/Run 概览可显示授权范围内的摘要和 Worktree 入口；未验证 Worktree 时没有 App tabs 和 App 数据。 |
| 右侧检查器 | 按需显示当前 Project、Run、Worktree 或 Branch 的事实来源、版本、观测时间和可审计操作。 | 检查器可收起；只显示 owner 提供的字段。Unknown、stale、unavailable 和 denied 分开呈现，不压成一个健康灯。 |
| 固定底部聊天栏 | 显式显示 GLOBAL 或 WORKTREE 目标与目标身份。 | 无 Worktree focus 时仅可选 GLOBAL；WORKTREE 必须绑定当前已验证 Worktree。Runtime/send 未通过能力门时输入禁用并说明阻断原因。 |

Project Worktree Index 作为 Project 概览中的跨 Run 汇总视图/过滤入口，不增加树形层级，也不替代 Project → Run 导航。每一行必须有 owner 返回的 Project、Engineering Run、Worktree 身份和 binding/version；现有缺少 Run ID 的聚合投影不得被拼接成 Run 行。字段不完整时该聚合视图标为部分/不可用，不能用它触发归属、重绑或清理操作。

### 选择、展开与焦点转换

| 对象 | 展开/选择 | 选择后的屏幕行为 | 不得隐式发生 |
|---|---|---|---|
| Project | 独立的展开控制加载 Run；Project 名称/选择控制进入 Project 概览。 | 显示 Project 身份、权限和可证明的 Run 摘要；索引入口仅按上一节的数据完整性门开放。 | 不自动选择第一个 Run/Worktree；不请求 Run App 数据。 |
| Engineering Run | 独立的展开控制加载 Worktree；Run 名称/选择控制进入 Run 概览。 | 显示 Run 身份、授权/策略摘要及有权限的 Worktree 清单；可显示有证据的空态和下一页状态。 | 不使用“打开 Task Cards”捷径绕过焦点门；Run 选择不等于代码 checkout focus。 |
| Worktree | 激活 Worktree 目标，进入解析与验证态。 | 校验 URL/Project/Run/Worktree 身份、当前授权、binding/version 和 freshness 后，才进入已聚焦工作区。 | 点击时不乐观挂载 App；不继承旧 Worktree 的 CLI、Canvas selection、Inspector 或聊天响应。 |
| Branch | 选择本地 checkout ref 的只读观察叶；保留当前 Worktree focus。 | 显示本地 ref/HEAD、来源、观测时间和 freshness；没有观测则明示 Unknown。 | 不将 Branch 叶改成 Worktree 或 Project 的父级；不推断远端 SCM 状态，不单独授权代码执行。 |

键盘树采用可预期的 roving focus：上下键移动当前行，左右键展开/收起或进入/返回父层，Home/End 移至当前已加载区间首尾；Enter 选择，展开按钮有独立可访问名称和 aria-expanded。选中态、键盘焦点、加载态和 Worktree App focus 使用不同视觉/语义状态。分页追加或窗口替换后恢复到稳定的实体 ID；删除/撤权时将焦点移到最近有效父级并播报原因。读屏顺序与视觉层级一致，状态不能只依赖颜色或图标。

Project、Run、Worktree 搜索只有在 owner API 提供相同授权范围的服务端过滤与分页时才可声称覆盖完整范围；如果只过滤当前已加载页，界面必须标出该范围，不能隐藏后续页中的匹配项或显示全局结果计数。

### 屏幕状态与 Run App 挂载门

| 状态 | 中间工作区 | Run App / 数据查询 | 底部聊天 | 离开/进入条件 |
|---|---|---|---|---|
| 尚未选择 Project | Project 选择/搜索与授权说明。 | 全部不挂载。 | GLOBAL 目标选择；Runtime 不可用时禁用输入。 | 选择有权访问的 Project。 |
| Project 已选、Run 未选 | Project 概览、授权 Run 目录和可证明的计数；计数明确区分总量与已加载数量。 | 全部不挂载。 | GLOBAL。 | 选择/展开 Run。 |
| Run 已选、无 Worktree focus | Run 概览、已授权 Worktree picker、目录分页状态、受控创建入口状态。 | 不挂载 App tabs、Task/Canvas/BI/Plugin 内容，也不发这些内容的请求。 | GLOBAL。 | 通过 Worktree 规范路由进入验证态。 |
| Worktree 正在验证 | 当前目标的身份和验证进度；旧工作区立即卸载。 | 目标与旧 App 都不挂载。 | 暂停 WORKTREE；GLOBAL 独立授权。 | 校验成功进入 focus；拒绝/过期/未知进入错误态。 |
| Worktree 已聚焦 | 显示 Project / Run / Worktree breadcrumb、本地 Branch 摘要、Run 同级 App tabs 和可收起检查器。 | 按当前 owner capability 懒加载用户选中的 App；不能预加载所有 tabs。 | WORKTREE 或 GLOBAL；两种身份和授权分开显示。 | 切焦、撤权、版本变化或 session invalidation 时卸载受影响的内容。 |
| Branch 只读详情 | 显示本地 Branch/ref 观察；Worktree focus 保持不变。 | 只保留已验证 Worktree 的当前 App 状态，不以 Branch 选择提升权限。 | 原 scope 保持并标识当前 Worktree。 | 返回 Worktree 或选择其他 Branch。 |
| 401、403、stale、provider 不可用 | 保留可读目录身份；区分会话失效、scope 拒绝、事实陈旧和 provider 缺失。 | 卸载被拒绝/过期范围的 App 内容；只显示允许的恢复操作。 | 清除失效 WORKTREE scope；403 不等于可切换到未授权身份。 | 恢复后重新解析身份、授权和 binding，不恢复旧会话写入。 |

切换顺序固定为：停止旧流/任务 → 取消可取消请求 → 清理旧 Task、Canvas、Inspector 和 WORKTREE chat 投影 → 解析目标 → 复验授权和版本 → 成功后挂载目标 App。HTTP AbortController 只能证明 HTTP 请求被取消；Tauri IPC 必须另有请求 ID、generation/token 和取消命令，并证明执行端停止或结果被丢弃。401 清除失效 session/focus；403 保留会话但拒绝该 scope；两者都不得保留可交互的旧 Worktree 内容。

### Worktree 状态检查器与操作契约

检查器对每类字段提供来源 owner、实体 ID、版本、观察时间和新鲜度。字段至少分为：归属（Project/Run/Worktree/owner）、执行（Agent/Session/Runtime/Task）、checkout（本地路径标识、Branch/HEAD、dirty 与 ahead/behind）、并行保护（lock/lease/conflict）、策略（HookSet/Profile/资源预算）和生命周期操作状态。Agent ID 不代表 Agent 健康、负载或正在工作；Task 负责人不代表自动 Agent assignment。没有 owner 事实时显示 unknown/unavailable，不显示为空闲或成功。

| 操作组 | 进入确认前必须展示 | 当前边界与默认阻断 |
|---|---|---|
| 创建/导入 | Project/Run/repository、源和目标 ref、目录/容量检查、预计绑定、稳定幂等 key 与可恢复 operation ID。 | 仅当 API 建立 Run binding 并返回对应 receipt 后才可在 Run UI 中启用。当前 Project/Repository 范围 API 不足以证明 Run 归属，保持 disabled/unavailable。未知结果的重试复用原 key。 |
| 绑定/重绑/交接 | 当前及目标 Run/Worktree/Agent/Task、锁与 lease、策略/profile、expected version、影响和冲突。 | 必须由对应 owner 的 preview/command/receipt 流程执行；没有预览合同、权限或版本时阻止提交。 |
| 合并准备 | 本地 Branch/HEAD、目标基线、dirty/untracked、冲突、验证证据、远端连接器权限（若涉及远端）。 | 预览不等于 merge；未有真实观测或 owner 操作 API 时只读，不出现成功确认。 |
| 归档/恢复 | retention lock、绑定、活动 Agent/Session/PTY/Runtime、保留对象与逻辑状态变化。 | 明示当前归档/恢复是逻辑管理状态，不删除 Git checkout、Multipass 记录或宿主文件。 |
| 清理 | 新鲜 retention/lock/lease/process/provider 观察、目标路径、回滚/恢复方案和最终 receipt。 | 任一观测 unknown/stale、dirty/untracked 未保全、执行仍活动或授权不足时 fail closed；不得从 UI 直接 purge。 |

每个生命周期操作采用预览 → 明确确认 → owner command → accepted/provisioning/completed/failed/unknown receipt → 刷新/对账的状态机。预览显示目标、影响、阻断项、expected version 和复用的幂等身份；UI 不以点击或 HTTP 202 推断完成。命令结果不确定时使用同一幂等 key 查询/重放；Audit/Outbox 是 owner 事实，UI 只展示其状态与深链，不能在客户端造一条成功记录。

### Run App、Task CLI、Canvas 与聊天

- Run App tab 由 owner capability 列表驱动；只在 Worktree focus 验证后挂载当前 App。当前 Task Cards 与任务事件面板按局部实现展示；未验收的 Inbox、Canvas、Workflow/Schedule、Hook 管理、Plugin、完整 BI/Benchmark、基础设施面板保持规划或不可用，不用预览数据填充生产状态。
- Task Card 与 Canvas 是 Run 下的同级 App。Task Card 内 CLI 始终显示绑定的 Task/Run/Worktree/Session；只有当前授权、执行 admission、受控 provisioner、sandbox、原生 Hook、资源预算和可取消 Runtime 都满足时才可输入。输出限字节并带截断/丢弃计数；取消、重接、独立验证和 owner 回写缺一时不标记为可用闭环。
- Canvas 节点保存 typed owner reference 与布局分离；搜索、展开或变更关系均走对应 owner API。节点打开 owner 页面前重新授权。只加载 viewport 加 overscan 范围；连线端点查找使用索引，缩略图/minimap 使用轻量摘要，不复制全量节点对象。
- BI/Benchmark 仅在数据源 owner、时间窗、coverage、freshness、样本量、公式/模型版本齐全时显示分析结论。现有任务事件列表使用“任务事件”语义，不能称为完整 BI 或 Benchmark；查询聚合与绘制不得阻塞焦点切换或终端交互。
- Chat Bar 始终明确显示目标身份。WORKTREE scope 要求当前 focus 和授权都有效；GLOBAL scope 的候选目标仍逐项授权，不表示全局管理员。scope/focus 切换会停止旧流、取消请求并丢弃旧 generation 的回调。Runtime 未接通时保留不可用原因，不显示假发送成功。

### UI-0 性能规划与可复测基准

UI-0 分开采集三条结果：当前 Tauri + React/Vite 壳的特征基线、共享 Web 组件参考基线、Rust 原生 Worktree/Task-CLI/Canvas parity prototype。当前 Tauri mock 页面不代表未来真实负载；三条结果使用相同语义的 fixture 和操作脚本，但报告不得混为一条成绩，也不能在 Rust parity 测量前宣布 renderer 已选定。

| 测量面 | 代表工作负载 | 必采指标 |
|---|---|---|
| Project/Run/Worktree 目录 | 确定性 fixture 超过当前 50 条分页、200 条替换窗口与 16-parent 缓存边界；含展开/折叠、切 Project/Run、busy/unknown/stale/dirty/lock 等状态。 | 首屏可交互、展开到绘制、滚动/键盘到呈现 p50/p95；可见行/挂载 widget、页/缓存字节、请求并发、RSS/CPU。 |
| Task/终端 | 12 条任务分页；多 pane 的 UTF-8/ANSI、长行、突发/持续输出、重连快照与缺口；按字节而非字符验证终端保留上限。 | 收到到呈现 p50/p95、每 pane 保留/丢弃/截断字节、队列深度/字节/最大年龄、取消到 drain 时间与取消后残留句柄。 |
| Canvas/BI | Canvas 分层包含当前 500 元素和 2k/10k 压力档、稀疏/稠密连线；BI 分开测 50 条页和 100 页游览及后台聚合。 | pan/zoom 到呈现 p50/p95、视口内/外元素与连线数、索引成本、请求/缓存字节、BI backlog 与交互公平性。 |
| 多 Agent 并发 | 使用配置允许的最大 Agent/订阅/终端负载，同时运行目录交互、Canvas 与 BI 背景任务；事件区分 owner 事实和展示进度。 | 交互尾延迟、队列高水位/年龄、事件到可见投影延迟、拒绝/合并/重同步计数、CPU/RSS 和每 Run 等待公平性。 |

每轮记录设备型号、CPU/RAM、OS/build、屏幕刷新率/DPI、电源模式、commit、renderer/runtime/toolchain、debug/release、fixture hash、重复次数和热/冷状态。报告整个应用进程树的峰值/稳定 RSS（不重复计数 WebView 子进程），并分列 Rust allocator/cache/view-model/queue/terminal/Canvas 用量；JS heap/DOM 仅作旧壳诊断。记录冷启动到 Project 可交互、分页绘制、键盘/滚动、Worktree focus、事件可见、terminal render、Canvas pan/zoom、BI result 和 cancel/drain 的 p50/p95。目标设备及真实负载分布确认前不设置机器无关的 RSS/ms 阈值。

关键 owner 事件不能静默丢弃；可合并的进度按实体/字段保留最新值，并记录 produced/coalesced/rejected/resync 数。队列按条数与字节双重设上限，设置 Run/Agent 公平配额；饱和时暂停读取/降低频率并用 durable cursor 请求快照重同步，不把 overflow 当成功。分页、模型缓存、Canvas 可见数据、终端 buffer 和 BI backlog 均有独立容量、淘汰/拒绝策略和取消 drain 门。重复展开/折叠、增删 terminal pane、focus 往返用于检查资源是否单调增长；切 Project/Run 时检查在途请求、订阅、socket/IPC handle 和旧 generation 响应全部排空或被拒收。

### 下一批可审查设计交付物与验收

UI-0/UI-1 开发前提交：

1. 低保真屏幕组：Project 目录/概览、Run 无焦点、Worktree picker、Worktree 验证/失败、已聚焦工作区、Branch 详情、Worktree 检查器与生命周期预览、Task Card CLI 禁用/启用态。
2. Project/Run/Worktree/Branch 状态字典，逐字段映射 owner、API、scope、版本、观测时间、freshness 和未知/拒绝表现；单列当前缺失字段与解除条件。
3. 键盘树、选择/展开分离、焦点恢复、读屏顺序、窄屏抽屉及 Worktree/Project/Run 切换时的取消/清理流程。
4. lifecycle preview/command/receipt/idempotency/reconciliation 的 API-to-UI 矩阵，确认 Project Worktree Index 和 Run Worktree projection 的 run binding 来源。
5. 三条 UI-0 测试 lane 的 fixture manifest、脚本、设备信息采集、RSS/CPU/内存分类和 p50/p95 报告格式；阈值由目标机实测与评审决定。

UI-0 完成门：三条 lane 可重复运行，边界及负载已说明，Rust parity 原型完成同语义交互，取消与背压可观测，资源结果可归因；评审确认渲染架构后才冻结技术选择和预算。UI-1 完成门：Project 与 Run 可分别选择/展开；名字缺失不伪造；主层级恒定；Project Index 不产生无证据的 Run 归属；无 focus 为零 Run App 数据请求；键盘/读屏、分页、旧焦点清理与错误恢复通过。生命周期、CLI、Hook、Plugin、BI、Infrastructure 未通过各自 WLD owner/API 和运行时门前保持 disabled/partial。当前整轮只完成规划，不声明上述界面、Rust renderer、Runtime 或生产闭环已经实现。

## 8.4 v0.8 下一版 UI 规划：Run 生命周期接入与 Worktree 控制闭环

### 规划基线和目标

v0.8 以 dev `8245b90c` 为实施事实基线，承接 WLD-1 Run-scoped lifecycle API 源码切片，并将其明确限制为“路由/授权/Provider seam 已有，真实 checkout 操作未开放”。旧 Project-scoped 前端路径仍返回 unavailable；前端不得把 Run API 源码存在描述为创建、导入、归档、恢复或清理已经可用。Run 当前 Repository binding、持久化 Run↔Worktree relation、目标数据库授权和可信 provider 均是开放门。

本版目标是把多 Agent checkout 管理路径做成可审查、可恢复的桌面体验方案：用户从 Project 目录定位 Run 和 Worktree，观察归属、执行、checkout、并行保护与 freshness；选择 Worktree 并重新验证后才进入该 Run 的工作区；每个危险动作从授权预览开始，直到可追溯 receipt 和重新观察完成。规划先冻结状态语义与 API-to-UI 合同，再进入实现。

### 核心屏幕与状态

| 屏幕 | 关键内容 | 明确边界 |
|---|---|---|
| Project Worktree Index | 跨 Run 筛选/排序；每行含 Project、Run、Worktree、Branch、owner 来源和观测时间。 | Run ID/binding 缺失时显示部分或不可用；不推断归属，不维护第二份事实。 |
| Run 概览（无 Worktree focus） | Run 身份、授权摘要、Worktree 目录分页、受控动作能力状态、GLOBAL 聊天范围。 | 不挂载 App tabs，不读取 Task/Canvas/BI/Plugin 内容；无焦点时不得启用 WORKTREE 聊天。 |
| Worktree focused workspace | 面包屑、checkout/Branch 摘要、Run 同级 tabs、按需打开的 inspector。 | 每次 focus 变化先取消旧请求/流、清理旧 scope，再复验授权、binding 和版本。 |
| Worktree inspector | Project/Run/Worktree owner、Agent/Session/Task 关联、Branch/HEAD、dirty/ahead/behind、lock/lease、HookSet/Profile、freshness。 | 只展示 owner-backed 字段；unknown、stale、denied、unavailable 分别表达，绝不当作 idle。 |
| Lifecycle operation preview | Create/import/rebind/archive/restore/cleanup 的目标、影响范围、授权、策略、dirty 数据、冲突、幂等身份和阻塞项。 | 未具备可信 provider、Run Repository binding、当前授权和 receipt owner 前，确认按钮保持禁用。 |
| Task Card 与 Canvas | 两个同级 Run App tabs；Task 卡内可显示 CLI 面板。Canvas 通过 typed reference 链接到 Task/Worktree/Branch 及其他 Run 对象。 | CLI 必须关联 Task/Run/Worktree/Session 并经过 sandbox/Hook/预算门；Canvas 只保存布局和引用，不能代写业务 owner 事实。 |
| Chat Bar | 固定在底栏，显式选择 `WORKTREE` 或 `GLOBAL`，显示作用域身份与 Runtime 状态。 | 每次切换清理旧流和响应；GLOBAL 不扩展已授予权限；未装 Runtime 时输入禁用。 |

### 生命周期可用状态与交互门

生命周期按钮只依据 capability 和 owner receipt 开放，状态至少区分 `available`、`preview-only`、`pending`、`unavailable`、`denied`、`stale`、`conflict` 和 `recovering`。API 的 202/queued、前端导航成功、provider 进程启动和 guest/VM 就绪都不得映射为 checkout ready。

Run API 可读取或配置不等于可以执行本地文件系统操作。用户确认操作时，服务端仍需实时重验 Project/Run 授权、Run 当前 Repository binding、HookSet/Profile 版本、idempotency key 和资源预算；provider 回执应能识别 Run↔Worktree binding ID/version、操作状态、审计和重放。失败后刷新 owner 状态并展示恢复入口，不能以旧 Project 路由、preview store 或 seed 数据补成功。

| 交互 | 前置事实 | 成功反馈 | 未满足时 |
|---|---|---|---|
| 打开 Worktree | Project/Run/Worktree identity、当前授权、binding/version 和观测 freshness。 | 设置 focus，按需挂载当前 Run App。 | 保留安全目录信息，清除旧焦点内容，显示重试或重新授权原因。 |
| 创建/导入 checkout | Run repository binding、trusted provider、路径解析结果、并发冲突与资源 admission。 | 显示 operation receipt 和新的 Run↔Worktree binding；ready 经独立观察确认。 | 按 unavailable/denied/conflict 分类；不调用 Project-scoped legacy path。 |
| 归档/恢复/清理 | retention/lease/claim、Agent/PTY/Runtime drain、dirty/untracked 和确认的保留计划。 | 显示对应 owner 的状态迁移、审计/Outbox 与对账结果。 | 未知状态/陈旧状态/未 drain/数据未保留时停止并给恢复步骤。 |
| 合并准备 | 本地 Branch、目标基线、dirty/conflict、Hook/验证证据和用户授权。 | 显示提交/冲突结果及新鲜度。 | 不把 Worktree 下的本地 Branch 与远端 SCM identity 混为一谈；不强制清理。 |

### 新版实施阶段和并行边界

本版不改变 UI-0..UI-7 命名，细化每阶段的交付物与前置门，见 [`WBS-UI-PLAN-NEXT-20261022.md`](WBS-UI-PLAN-NEXT-20261022.md)。UI-0 设计和性能协议先行；UI-1 目录投影与 UI-2 Run 焦点壳完成后，生命周期、Task CLI、Canvas 和分析/Plugin 面板可在 API/owner 合同冻结、文件边界互斥时分开实现。不得把未验收 provider/Rust Runtime 的执行按钮作为视觉占位。

Rust 桌面目标以有界内存与可取消并行为硬条件：首屏只取 Project/Run 摘要；Worktree 目录分页和虚拟化；Canvas 只载入视口元素；Task transcript/terminal output 采用有界 ring buffer；事件进度允许合并、owner 事实事件不丢失；API、文件扫描、Git 观察和渲染资源均支持取消/drain。性能规划分开记录 Tauri+React 现状、共享 Web 参考和 Rust-native parity 原型，使用确定规模数据、process-tree RSS、缓存/队列/绘制计数、交互 p50/p95 与取消 drain 数据；目标设备测量前不编造硬阈值。Multipass/K3s 交互属于 Run Infrastructure App 的 owner 事实，测试最多 2 个 Multipass inventory 实例，绝不要求 Docker Desktop。

### 本轮完成定义和未完成项

- 完成定义：实现基线更新到 `8245b90c`；UI-2 集成状态、WLD-1 API/Provider gap 和所有后续 UI 阶段在计划与 WBS 中一致；Project → Run → Worktree → Branch 导航和无焦点 App mount 门不被改写；无 mock/seed 生产成功表述。
- 本轮只更新规划文档，不实现新页面、不开放生命周期、任务执行、聊天发送或 BI ingress。
- Rust 1.99.0、目标 PostgreSQL/RLS、真实 provider、Run↔Worktree 持久绑定、Runtime sandbox 与 UI 性能实测仍未验收；相应 UI capability 必须显示 disabled/unavailable。

## 8.5 v0.9 UI-0 设计包与下一版工作顺序

本版本把 UI-0 从方向性文字收敛为可审查的屏幕/状态合同与桌面架构边界：

- [UI0-SCREEN-STATE-20261022.md](UI0-SCREEN-STATE-20261022.md) 定义 Project/Run 目录、Worktree focus 验证、Run 同层 App、Task Card 内 CLI、Canvas、Chat Bar 的状态、owner/API 映射、切换清理、无障碍要求和 fail-closed 表现。
- [UI0-RUST-DESKTOP-ADR-20261022.md](UI0-RUST-DESKTOP-ADR-20261022.md) 固定 Rust 桌面职责、类型化 IPC scope/version/cancellation 边界、有界并发与缓存、原生 Hook/Plugin 隔离要求，以及可复测的性能报告协议。
- [WBS-UI-PLAN-NEXT-20261022.md](WBS-UI-PLAN-NEXT-20261022.md) 将 UI-0..UI-7 排成依赖明确的交付批次。

本轮已完成 UI-2 picker 的有限分页/已加载项搜索实现和 focused test/typecheck；UI-0 generator/runner 与实际性能比较转入下一批。UI-0 的设计包可供实现评审；目标设备库存与压力负载尚未采集，因此渲染 crate、绝对内存/延迟阈值和 Rust parity 结论仍未决定。不得把设计完成写作桌面实现或性能验收完成。

后续顺序：

1. **UI-1：目录与可观测性。** 先补 Worktree 到 Run 的可信归属、binding/version、observed-at/freshness、分页和键盘可达；owner 未提供的 lock、dirty、ahead/behind、assignment 继续显示 unknown。
2. **UI-2：焦点工作区。** 完成可搜索/分页 Worktree picker、focus 代次切换、旧请求/订阅清理、Run App lazy mount 和固定 WORKTREE/GLOBAL Chat Bar。无 Worktree focus 时不请求 App 数据。
3. **UI-3：生命周期控制。** 仅在 provider、Run↔Worktree binding owner、目标 DB/RLS、权限复验、审计和 durable receipt 都通过后开放预览/执行/恢复流程。
4. **UI-4/5：CLI 与 Canvas。** CLI 经 Task/Run/Worktree/Session、Hook、budget、sandbox、验证和结果回写闭环；Canvas 使用视口数据与 typed references，不直接更改其他 owner 的事实。
5. **UI-6：Workflow、Hooks、BI/Benchmark、Plugins 和 Infrastructure。** 展示规则与执行分态、原生 Hook 决策、coverage/freshness、plugin capability/revoke/drain，以及 Host/Multipass/K3s 分层身份。
6. **UI-7：桌面候选比较和发布门。** 在目标机 inventory、固定 fixture 和 renderer parity 原型齐备后比较 Rust 前端；记录进程树内存、队列/缓存高水位、交互 p50/p95、取消 drain 和无障碍。先测量，再由评审定阈值。

所有性能敏感后台工作继续遵守按 Project/Run 公平份额、有界队列与字节、优先级、背压、generation、取消/drain；工作流事实由业务 owner 持久化，LangGraph/checkpoint 或本地投影不替代 Run 授权与业务事实。
## 9. 总体验收清单

- 导航顺序恒为 Project → Engineering Run → Worktree → Branch；Project 是第一层，Branch 是 Worktree 子层，不以宿主连接状态改序。
- 选 Run 而无 Worktree focus 时只查看 Project/Run 摘要与 Worktree 清单，不显示任何 Run App tabs/content；点击有效 Worktree 后才显示 Inbox、Work Items、Task Cards、Canvas、Workflow、BI/Benchmark、Plugin 等同级 tabs。
- Task Card 与 Canvas 在 Run 层同级；CLI 只从 Task Card 内进入，并持有正确 Task/Run/Worktree/Session 身份。
- Chat Bar 固定并显式选择 WORKTREE/GLOBAL；GLOBAL 只扩大候选范围，不扩大授权。
- Schedule 面板把 Rule 配置状态与 trigger、Occurrence、Run admission、executor 状态分开；没有远端 Branch 的 Run 可配置规则，但不能据此显示为可执行。目标 Worktree、Task、Profile、HookSet 和 run-as 授权仍需有效。
- Hook 草稿的 Leave/Stay 保护不得在用户选择 Stay 时关闭菜单、改写最近访问记录或丢失焦点；普通同源内容链接批准后走 SPA push，Stay 不调用链接回调。浏览器 Back/Forward 仍未保护，必须显式列为限制并在进一步实现后单独验收。
- Worktree Index、Run、Task、Canvas、Workflow、BI、Plugin、Hook、Infrastructure 状态均指向单一 owner 数据/API；跨 App 变更返回 receipt、version 和可审计结果。
- Worktree 生命周期按钮仅连接 Run-scoped API；无真实 provider、Run Repository binding、目标 DB/RLS 与有效 receipt owner 时保持 disabled，不回退到已关闭的 Project-scoped API。
- 多 Agent 工作区能区分运行中、等待、冲突、锁定、授权撤销、资源不足、状态未知和已完成；状态来自可信服务，陈旧数据带时间戳。
- UI 加载、缓存、列表、事件订阅、终端输出、图表和插件资源均有界；所有长任务可取消并在取消后释放资源。
- 关键操作具备键盘可达、焦点恢复、可读状态、错误恢复与确认；颜色不是唯一状态信号。
- 所有尚缺生产 provider、身份、sandbox、目标数据库、Hook、consumer 或 BI coverage 的功能明确标为 unavailable/partial/unknown，不使用演示数据伪装成功。

## 10. 已知差距与待决项

1. Rust 原生桌面渲染器尚未通过原型比较；UI-0 前不固定具体框架。
2. 当前 Tauri + React/Vite 桌面原型的 Worktree/Canvas 数据与部分 IPC 仍是 fixture/preview 路径；Worktree 可视化没有视口裁剪。旧界面仅作交互参考，不能把样例记录带入生产路径或当作 Rust 桌面性能基线。
3. UI-2 已关闭 Web Run Shell 的 no-focus Task Cards 偏差；UI-1 已实现 Project/Run 选择与展开分离，组件测试覆盖 Project Index 导航、Project scope 更新、Run no-focus 导航及展开不隐式选择。浏览器/键盘/读屏仍未验收。Run Worktree picker 最多呈现 4 页/200 条并支持已加载项搜索。右侧完整 App 工作区、Inspector、服务端范围搜索、聊天 Runtime/send、浏览器 Back/Forward、目标 DB、真实 session provisioner、Hook 生命周期和 BI receiver 仍开放。
4. Rust 前端的跨平台字体/输入法、屏幕阅读、窗口/多显示器、终端渲染以及插件沙箱具体协议，在 UI-0 用目标设备实测确定。
5. 基准所用硬件型号、合成目录规模、绝对 RSS/延迟阈值需在 UI-0 定版；在此之前不得声称满足了性能目标。
6. WLD-3 hostless Schedule Rule 与 Occurrence snapshot CHECK source slices 尚未通过 Rust 1.99.0 和 PostgreSQL migration/API acceptance；Occurrence 仍只有 JSONB `target_snapshot`，CHECK 约束 JSON path，不新增 Branch 列，也不实现 worker/materializer/admission/executor。
7. Project 投影当前只有 ID/role；Run Worktree projection 缺 lock、dirty/ahead/behind、observed-at、Branch freshness 与 assignment receipt。Project Worktree Index 缺 Engineering Run ID，创建/导入也没有 Run binding receipt。补齐 owner/API 合同前，不得把这些字段作为生产事实或开放 Run 级生命周期操作。
8. 旧 Group 路由可回退至本地 store/预览；当前 BI 面板是任务事件视图，Canvas 未装配到 canonical Run Shell。仅凭这些现象不得声称完整 App 集成、BI/Benchmark 或生产数据闭环。
9. WLD-1 Run-scoped lifecycle API 还没有真实 provider、Run↔Worktree binding writer 和目标数据库验收；现有 UI 未迁移至这些路径。Run API 与旧 Project API 的表面存在不可作为 UI 可用能力证明。

## 11. 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v1.6 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 实现 UI-1 Project/Run 展开与选择分离；Project 进入 Project Worktree Index，Run 进入 no-focus 概览。UI-1/UI-2 focused tests 共 9/9、typecheck 与 CypherGraph Guardian 通过；组件级 ArrowRight disclosure 用例通过，浏览器/读屏和 production build 仍未通过/未验收。 | 按 WBS 推进 UI-1 选择态切片 |
| v1.5 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | WBS UI-0 generator/runner 已登记独立 worktree 并开始实现；实际性能测量和 renderer 决策仍保持开放，dev integration 等待 Orca owner 状态允许。 | WBS UI-0 并行 lane 启动 |
| v1.4 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 UI-2 6/6 focused tests/typecheck 通过，以及 production build 在主机可用内存约 0.81 GB 时 Node OOM/exit 134；build 不计为通过，浏览器验收仍开放。 | 本轮 UI-2 build validation |
| v1.3 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修复 UI-2 页内重复 ID 和超量响应边界；首屏与后续页共用 50 条单页/200 条总量检查。focused Vitest 6/6、TypeScript typecheck 通过；生产 build、浏览器/e2e/无障碍仍开放。 | 按 WBS 继续并完成分页边界自审 |
| v1.2 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 为分页 Worktree picker 增加搜索/游标回归测试；focused Vitest 3/3、TypeScript typecheck、CypherGraph Guardian 和 git diff --check 通过，浏览器/e2e/无障碍仍开放；更新 WBS UI-0 下一步。 | 按 WBS 继续 UI-0 与 UI-2 验证 |
| v1.1 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 固定 UI0-BENCH-001 workload parameters 和目标机静态 inventory；记录 unknown 元数据标记及 UI-1 缺失 owner freshness facts，generator/runner 与实际测量仍开放。 | UI-0/UI-1 静态审计 |
| v1.0 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 以 5a051bfd 为实现基线；记录 no-focus picker 的有限分页、已加载项搜索、200 条上限和 fail-closed 边界。 | 最新 dev 上的 UI/UX 改进 |
| v0.9 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 基于实际 dev 17b3d065 更新规划基线；增加 UI-0 屏幕/状态包、Rust 桌面架构 ADR 与 UI-1..7 的下一版实施顺序。明确此次仅完成规划，renderer/目标机性能门仍开放，未修改 UI 实现。 | 本轮设计交付及与 dev 实际源码状态核对 |
| v0.8 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将规划基线更新到 dev `8245b90c`；纳入 WLD-1 Run-scoped lifecycle API seam、provider 默认缺失、Run↔Worktree binding owner/目标 DB/Rust 门未关闭以及前端仍使用旧 Project routes 的事实；新增 Worktree 检查器、操作预览与 receipt 屏幕状态，细化 UI-0..7 下一版阶段、并行边界和低内存/可取消性能要求。此次为规划，未实现 UI 或解除执行 fail-closed。 | WLD-1 源码切片合入 dev 后，用户要求针对新版本开展 UI 规划 |
| v0.7 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 在已合并的 dev `30c6b975` 基线上把 UI 规划升级为屏幕级规格：区分 Project/Run 选择与 Worktree focus，修正无焦点 Run 快捷入口、Project Index 缺 Run 归属、Run Worktree 状态字段和 lifecycle API 的证据边界；增加 App mount 状态、键盘/读屏、owner receipt 与幂等恢复合同，并细化 Tauri/Web/Rust parity 三条 UI-0 性能测量 lane、fixture、计数和 p50/p95 门。没有把规划写成已实现能力。 | 用户要求当前开发轮结束后，对新版本进行一轮 UI 规划；UX 与性能只读评审 |
| v0.6 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 基于本地 dev a5b4404a 与已整合的 UI-2/WLD-3/WLD-5 切片，将下一版 UI 主线聚焦到多 Agent Worktree 可见管理与安全生命周期控制；细化页面、状态、用户流程、操作预览/回执、性能输入和下一批交付物。 | 当前开发轮已合并到 dev，用户要求随后规划新版本 UI |
| v0.5 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 在 UI-2 实际交付后更新实现基线；增加 Run/Worktree 焦点状态矩阵、切换/失败清理约束和下一轮 UI-0..7 排序、交付物及停止条件。 | UI-2 与 WLD-3 本轮实现及验证结果 |
| v0.4 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 以 UI-2 实际 no-focus 门更新 Run Shell 状态；明确左侧 Project/Run/Worktree/Branch 常驻、右侧工作区仅在 Worktree 授权后出现、当前 Worktree 选择仅第一页，并把 Rust UI-0 基准、搜索分页、完整右侧 App/Inspector 作为后续计划；同步 WLD-3 Occurrence Branch CHECK 与未验证门禁。 | UI-2 与 WLD-3 本轮实现及验证结果 |
| v0.3 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 根据 UI-1 Worktree 状态呈现与 WLD-5 普通同源链接保护的本轮验证证据，修正 no-focus Run 误显示 Task Cards 的导航偏差，补充双态布局示意、焦点切换合同、unknown 状态边界及后续 UI 阶段验收。 | 用户要求当前开发完成后规划新版 UI，并确认 App tabs 必须在选择 Worktree 后才显示 |
| v0.2 | 2026-10-21 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 按 dev `1d6b3338` 更新 WLD-3 hostless Rule 配置/Branch-required 执行分态和 WLD-5 全局壳导航草稿保护现状；明确 Rust/PostgreSQL 与浏览器历史导航缺口，细化 UI-5 验收。 | 当前开发轮完成并按用户要求规划新版 UI |
| v0.1 | 2026-10-21 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 基于本地 dev `6f4ebc17` 的导航、Run Shell、Task/CLI、Hooks 与 Tauri 源码证据制定 UI 信息架构、Rust 桌面性能边界、交互合同、分阶段计划和验收门；保留未实现与生产差距。 | 用户要求本轮开发后规划新版 UI |
