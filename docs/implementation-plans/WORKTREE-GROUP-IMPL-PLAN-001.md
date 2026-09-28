# WORKTREE-GROUP-IMPL-PLAN-001

> **渡口 Project Worktree 群组实施计划 v0.5**
>
> - 状态：🟡 执行中（Phase 0/1 已完成设计与预览；Phase 2A GroupContext / 跨 App 契约详细设计完成；下一步 Phase 2B 实装认证 Actor、Project/Worktree ACL 与 GroupContextResolver）
> - 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> - 日期：2026-09-29
> - 关联需求：`docs/requirements.md` v2.4 §50
> - 关联基本设计：`docs/basic-design.md` v0.5 §16
> - 关联详细设计：`docs/design/DD-WORKTREE-GROUP-001.md` v0.1、`docs/design/DD-MULTICA-TASK-001.md` v0.3、`docs/design/DD-WORKTREE-CANVAS-001.md` v1.3

---

## §0 目的与完成定义

本文把 Worktree 群组架构拆成可独立验收的阶段。产品信息树固定为：

```text
Project（先选择，限定可见工作范围）
└── Project Worktree Index（多 Agent Worktree 管理中心）
    └── Worktree（展开，选为当前 GroupContext）
        ├── Multica 任务管理
        ├── Jira 等价视图
        ├── Task Card Index / Task Card Detail（canonical WorkItem）
        ├── Infinite Canvas（Miro 等价视图，可引用和创建任务）
        ├── Workflow / LangGraph
        └── Plugin Apps（可启停的同级应用入口）
```

`/worktree?project_id={project_id}` 是项目 Worktree 管理入口，Project 页的 Worktrees 视图传递当前 Project ID 进入 Index；`/worktree/{worktree_id}/group` 展开指定 Worktree 的同级应用。所有生产入口均消费同一个服务端解析、授权后的 `GroupContext(worktree_id, tenant_id, actor_id, permission_snapshot_ref, correlation_id)`。应用导航由 Group Shell 持有；应用只负责自己的领域视图，不得创建第二棵 Worktree 树或自己的底部聊天栏。

“完成”分两级报告：

1. **可交互原型完成**：Worktree → 同级应用导航、共享 canonical WorkItem 预览、Canvas ↔ Task Card 导航、卡内 CLI 入口状态、WORKTREE/GLOBAL 聊天 scope 选择、插件注册表预览均可演示；所有未连接后端的动作显式标为预览/不可用。
2. **生产验收完成**：GroupContext 服务端鉴权、任务 lifecycle 写入、Local Runtime CLI、Canvas EntityRef 持久化、scope-aware Chat/LangGraph、插件 capability 热插拔与撤权全部接通并通过跨应用验收。原型通过不等于生产完成。

## §1 当前基线与不可混淆边界

| 能力 | 仓库当前事实 | 实施影响 |
|---|---|---|
| Project / Worktree | `/worktree` 是 Project Worktree Index；Project 页 Worktrees 视图可进入该 Index；展开 Worktree 显示同级 App 入口；seed 任务没有 Worktree 绑定 | Project 选择限定列表；演示需显式标记项目级 seed；生产命令必须拒绝未绑定或不匹配的 WorkItem |
| Worktree ownership | 前端 `Worktree` projection 含 `agent_session_id` / `local_runtime_id`，但没有 `owner_user_id`；AgentSession 可显示当前 Agent 名称和状态 | Index 不伪造 owner；Phase 2 需定义负责人投影、转派审计及 Agent Session 多次执行的历史显示 |
| Task Card | Multica/Jira 视图尚未共享一套 Worktree Group Shell 路由 | 以 `work_item_id` 为唯一任务身份，外部 Jira key 仅作 alias |
| Canvas | 当前 Canvas 以 `project` / `free` 为 ref；元素含 `work_item_id`，双击沿用旧 `/work-item` 路由 | 原型可复用 CanvasView；生产改为 canonical `EntityRef(type,id,worktree_id)` 和 GroupContext ACL |
| CLI | `TerminalStackContainer` 无 session 时是 UI placeholder；没有按 TaskExecutionContext 启动的命令通道 | CLI 面板可以先做状态原型；不得显示为已连接 shell 或接受可执行命令 |
| Chat | `ChatPanel` / BFF W1 contract 只覆盖普通 session/message stub，不携带 GroupContext、GLOBAL targets 或 ACL | UI scope selector 可先落地；发送、检索、工具调用必须等 scope-aware API 和服务端 ACL |
| LangGraph | 既有设计区分 Group App Registry 与 `SubAgentRegistry`；ADR-0047 把 Postgres Tier 3 实装列为有前置门槛的工作 | LangGraph 负责执行图，不负责应用导航；执行恢复必须重新授权，checkpoint 不恢复历史权限 |
| Plugins | Group App Registry / capability bridge 在详细设计中有契约，前端没有运行时注册和撤权路径 | 先 manifest/registry contract，再做 enable/disable、调用拦截、撤权和卸载保留引用 |
| Group ID | `NEXT_PUBLIC_GROUP_ID=canvas/domain/frontend/core` 属于仓库协作/分支守门分类；不是用户产品群组身份 | 产品 Worktree scope 必须从路由实体 + 服务端认证解析；不得把此构建期变量当授权凭据 |

`docs/worktree-group-guard.md` 与 `scripts/automation/group_guard.py` 继续服务开发分组守门。守门脚本已有实现，但 `check_cross_ref` 仍是 placeholder；此守门不代替产品级 GroupContext。

## §2 目标组件与事实所有权

| 组件 | 所有权 | 必须遵守 |
|---|---|---|
| Project Worktree Index / Group Shell | Worktree UI / shell | 按选定 Project 展示和管理 Worktree；展开后解析当前 GroupContext；挂载同级 App Registry；拥有唯一底栏 |
| Multica / Jira 等价视图 | WorkItem Domain + 各 UI adapter | 共享 canonical `work_item_id`、同一个 lifecycle version；看板不得私有化状态事实 |
| Task Card 与卡内 CLI | WorkItem Domain + Agent/Local Runtime | Card 是 WorkItem 投影；启动执行时由服务端构造 TaskExecutionContext 并绑定唯一 Worktree |
| Infinite Canvas | Canvas Domain | 保存布局和 EntityRef；任务创建/改状态通过 Lifecycle Command，不直写 WorkItem 表 |
| Workflow / LangGraph | Workflow Runtime / LangGraph | Run 固定不可变 scope snapshot；resume 重新检查当前 ACL、插件状态和 Worktree |
| Plugin Apps | Group App Registry + capability bridge | 插件入口与 agent graph type 分离；禁用立即拒绝新调用，保留业务事实与 disabled EntityRef |
| 底部聊天栏 | Group Shell Chat Adapter + L0 | 同一栏切换 WORKTREE/GLOBAL；GLOBAL 写必须明确目标、逐目标 preflight、逐目标结果和审计 |

## §3 分阶段实施与验收门

| Phase | 交付范围 | 依赖 | 完成门 |
|---|---|---|---|
| **0 设计基线** | 需求/基本/详细设计统一 Project 选择 → Worktree 管理 → 展开后同级 App、canonical ID、ownership、W/T/M 和权限规则 | `requirements.md` v2.4 §50、`basic-design.md` v0.5 §16、`DD-WORKTREE-GROUP-001.md` v0.1 | ✅ 已收口；SDK、Plugin sandbox 与在途撤权仍作为兼容门 |
| **1 Group Shell 原型** | 项目选择、项目 Worktree 清单与状态管理面板；展开 Worktree 后显示 Multica/Jira/Task Card/Canvas/Workflow/Plugins 同级导航；Canvas ↔ Task Card 深链；底部 scope selector | 现有 project nav store / frontend store / CanvasView | 🟡 预览交付：Project Worktrees 入口携带 `project_id` 进入 Index；浏览器可见按 Project 过滤的 Worktree 管理信号和同级 App 树；Task Card / CLI preview / WORKTREE-GLOBAL 选择可见，未连接的发送和命令保持禁用；owner 数据与服务端操作仍待接入 |
| **2 安全上下文与 canonical 任务闭环** | 2A GroupContext / 跨 App 详细契约；2B 认证 Actor、Project/Worktree ACL、GroupContextResolver；2C PostgreSQL 持久化与 WorkItem / Worktree 归属 API、Multica/Jira adapter、Task Card 读写及 version/idempotency；2D Index 投影与受权管理操作 | 2A 设计基线；认证与 ACL；DB migration、alias reconciliation、Worktree owner / Agent / Runtime / Git 状态事实源 | 🟡 2A 设计完成（`DD-WORKTREE-GROUP-001.md` v0.1）；⏸️ 2B 尚未实装：当前 `star-api-rest` WorkItem handler 使用 `InMemoryWorkItemService` / stub auth / 默认 nil-tenant ActorContext，不能接到 Group Shell。依次通过认证 Actor + ACL 负向验收、持久化同 ID/version 与幂等验收、Index 数据源和管理动作验收 |
| **3 Canvas 双向联动** | `EntityRef`、Worktree-scoped Canvas 查询、Canvas 创建任务/关联任务/打开任务、跨域 outbox | Phase 2B ACL + Phase 2C persistence / WorkItem API + Canvas persistence/API | Canvas 和所有任务视图读到同一 canonical ID；Canvas 不能绕过 Lifecycle Service；项目级旧 Canvas migration 冲突隔离 |
| **4 Task Card CLI** | `TaskExecutionContext`、本地 Runtime session 绑定、卡内 CLI、运行/取消/断线恢复 | Local Runtime 身份、Worktree checkout、工具与 secret capability ACL | CLI cwd 必须解析为授权 checkout；缺失/过期 scope 拒绝执行；执行记录可追溯，退出后清理 Work 状态 |
| **5 Group Chat + LangGraph** | Chat scope/targets DTO、服务端 GroupContext middleware、L0 router、Flow draft、LangGraph run/checkpoint、resume 再授权 | Phase 2B ACL、Phase 4 TaskExecutionContext、锁定的 LangGraph SDK compatibility review | WORKTREE 只读写本 Worktree；GLOBAL 写需显式授权目标；未授权目标零副作用；checkpoint 恢复不复活撤销授权；chat session、WorkItem、Task Card、LangGraph thread ID 分离 |
| **6 Plugin 热插拔** | manifest/version/capability、Group App Registry 动态发现、enable/disable/uninstall、热撤权和 UI 更新 | Phase 2B ACL + Phase 5 L0 capability bridge | disable/uninstall 后新调用即时失败；在途操作有确定取消/排空语义；业务 Transaction 与 Canvas ref 保留；agent registry 不冒充 app registry |
| **7 跨应用验收与发布** | Worktree → 每个 app 的端到端路径、权限矩阵、审计、故障注入、性能和迁移演练 | Phase 1-6 | §4 验收全过；已知缺口逐项关闭或保留为有 owner 的 release blocker；发布说明不把 preview 标为生产能力 |

Phase 2B 是安全基础门；2C 与 Canvas 数据层可在命令契约稳定后并行开发，2D 依赖 owner / Session / Runtime / Git 状态的权威数据源。Phase 4、5 共享授权与执行身份，未完成对应 API 前不能用 UI stub 替代。Phase 6 必须复用 Phase 5 的 L0 capability bridge。

## §4 跨应用验收矩阵

| ID | 场景 | 通过条件 |
|---|---|---|
| WG-ACC-01 | 从 Worktree Index 打开群组 | 页面上下文唯一指向该 Worktree；顶栏、应用导航和底栏一致显示 scope |
| WG-ACC-02 | Multica ↔ Jira ↔ Task Card | 三者读取相同 `work_item_id` / lifecycle version；状态写入只经 Lifecycle Service |
| WG-ACC-03 | Canvas ↔ Task Card | 双向跳转保留当前 `worktree_id`；EntityRef 指向 canonical WorkItem；WorkItem 关联多个 Worktree 时仍按当前 GroupContext 授权，未知/跨 Worktree ref 不被隐式改绑 |
| WG-ACC-04 | Task Card CLI | 只能使用服务端解析的 Worktree checkout；用户输入的 URL/路径不能选择任意目录 |
| WG-ACC-05 | 底栏 WORKTREE scope | 消息、检索和工具调用均绑定当前 Worktree；切换 Worktree 开新 scope session |
| WG-ACC-06 | 底栏 GLOBAL scope | 写操作无显式目标返回 `target_required`；逐目标 ACL preflight 后返回独立结果，不允许静默扩权 |
| WG-ACC-07 | Plugin disable/uninstall | capability bridge 阻止新请求；应用入口立即更新；Transaction/audit/EntityRef 保留 |
| WG-ACC-08 | LangGraph resume | 每次执行/恢复加载当前 GroupContext 与权限；旧 checkpoint 只恢复计算状态，不作为授权证据 |
| WG-ACC-09 | 失败与重试 | 超时、部分目标失败、重复消息、重复命令可观察且幂等；没有未记录的副作用 |
| WG-ACC-10 | 数据保留 | Work checkpoint/draft 按 TTL 清理；Task lifecycle、chat message、execution history、审计和 Master 按所属 W/T/M 规则保留 |

## §5 当前 Release Blocker / 已知缺口

1. **GroupContext / WorkItem API 不具备生产安全性**：`star-api-rest` WorkItem handler 使用 `InMemoryWorkItemService`，以 `ActorContext::default().with_role("developer")` 处理请求；`auth_layer_stub` 只透传请求，不注入认证 actor。不能把该接口接成 Group Shell 的生产数据源。
2. **持久化与生命周期契约不完整**：当前 `domain-work-item` 只有 in-memory service、3 态状态机；Multica 详细设计定义 6 态执行 lifecycle 与独立 review state。Phase 2 必须先解决 canonical 写入/版本/事务映射和 PostgreSQL repository。
3. **服务端 scope-aware Chat 尚不存在**：当前 Chat W1 DTO 只有 `session_id/user_id/content`，`user_id` 由调用方传入，Transcript 在进程内 map；无 Worktree scope、授权目标或 GroupContext ACL。Phase 5 之前 composer 必须保持禁用或明确 preview。
4. **CLI 仍是 placeholder**：没有从 Task Card 到 Local Runtime 的可信 session provisioning；Phase 4 之前禁止实际执行命令。
5. **WorkItem seed 未绑定 Worktree**：现有 mock seed 不构成生产关联证据；Phase 2 migration 需要权威 alias/relation 来源，歧义记录隔离，不自动合并。
6. **Canvas 仍是 project/free 参考模型**：现有 Canvas 列表不能被推断为 Worktree-owned；Phase 3 定义绑定与回填方案。
7. **LangGraph SDK/checkpointer 版本**：实施前读取仓库锁定版本并验证 Runtime/context/checkpointer API；Tier 3 按 ADR-0047 的有效前置门槛执行，未满足时采用已批准的较低 Tier 或将生产验收标阻塞。
8. **Plugin 热撤权语义**：需确定在途 run 的 cancel/drain 策略、manifest trust、签名与 API version compatibility；Phase 6 通过 ADR/详细设计冻结。
9. **开发 Group 守门未完成**：`group_guard.py` 已有脚本，但 `check_cross_ref` 仍 placeholder；`NEXT_PUBLIC_GROUP_ID` 仍是开发分组变量，未显示产品 GroupContext。两者分别在开发工具与 Group Shell 阶段处理。

## §6 Change Scope 与最小验收顺序

1. 本计划及本次 UI 原型涉及 `frontend/src/`、`frontend/e2e/`、`docs/`；不碰当前主工作区中的主题改动。
2. Phase 1 使用现有 Zustand seed 做可交互原型；mock-only 功能需持续显示“预览 / 未接服务端”，不能模拟授权成功。
3. 先完成 TypeScript typecheck 与新增交互路径的 focused checks；之后再执行目标端到端路径。生产阶段追加 Rust workspace、DB migration、ACL 和 LangGraph 各自的守门。
4. 任何 Phase 变更须同步本计划的状态、验收和 release blocker；跨 W/T/M 的新增表按项目 DB 分类规则逐表归类。

### 6.1 本次阶段结果（2026-09-29）

| Phase | 结果 | 证据/限制 |
|---|---|---|
| 0 设计基线 | 🟢 完成 | 需求 v2.4 §50、基本设计 v0.5 §16、Group 详细设计 v0.1 与 Multica 详细设计 v0.3 对齐 Project → Worktree → Group Apps；实体授权、生命周期、事件、W/T/M 与兼容门已明确 |
| 1 Group Shell | 🟡 预览完成 | Project 页 Worktrees 视图 → `/worktree?project_id=...` → Worktree Group 路由已在浏览器渲染；Index 显示 Agent/Runtime、branch、status、PR、lock 和最近活动；Group Shell 显示 Task Card、卡内 CLI 预览和 WORKTREE/GLOBAL 选择。数据仍为 seed/local；Worktree owner 字段、服务端 GroupContext、Chat API、真实 terminal session、创建/归档与 plugin runtime 尚未接通 |
| 2A GroupContext / 跨 App 契约 | 🟢 详细设计完成 | `DD-WORKTREE-GROUP-001.md` v0.1 定义认证 actor、Project/Worktree ACL、Worktree 管理 plan/confirm、TaskExecutionContext、EntityRef、Outbox、Chat/LangGraph 与 Plugin 撤权契约；这不代表后端已实现 |
| 2B 认证 Actor / ACL / GroupContextResolver | ⚪ 下一阶段 | 当前 REST `auth_layer_stub` 不注入认证身份，默认 Actor 不能承载生产请求；先接入验证后的 Actor 与 Project/Worktree ACL，并实现跨租户、跨项目负向拒绝 |
| 2C canonical 任务闭环 / 持久化 | ⏸️ 等待 2B | WorkItem REST 路由仍使用内存服务；2B 安全上下文完成后，落 PostgreSQL repository、统一 ID/version、幂等命令和生命周期映射 |
| 2D Worktree Index 投影 / 管理 API | ⚪ 未开始 | 需要 owner、Session、Runtime、PR、Git lock/conflict 等权威投影来源；管理动作须 plan/confirm、乐观锁与 Audit |
| 3-7 | ⚪ 未开始 | 按 §3 依赖顺序推进；不得把 Phase 1 展示当作生产验收 |

本次 HTTP 验证：`/worktree?project_id=prj-mobile` 与 `/worktree/wt-003/group?app=task-card&work_item_id=wi-001&cli=1` 均返回 200，Index 响应保留 `project_id`，Group 响应保留 `wt-003`，没有 redirect。Phase 1 的浏览器预览记录：页面可见同级 App、任务详情、卡内 CLI 预览和禁用的 Chat 发送。`pnpm typecheck` 未通过，仍有 Monaco 子组件、terminal store、push client 和 MSW 重复导出等仓库诊断；定向 Vitest 在收集用例前被 `src/mocks/handlers/index.ts` 中重复导出的 `annotationsHandlers` 阻断。没有把 preview/seed 行为标成生产能力。

---

## 修订履历

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-28 | Ulysses — Mavis 接手审核 | 建立 Worktree Group 分阶段实施路径、跨应用验收矩阵及当前服务端 blocker；明确开发 Group ID 与产品 Worktree scope 不同 | 用户要求基于 Worktree 顶层体系继续推进至完成 |
| v0.2 | 2026-09-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Phase 0 设计收口与 Phase 1 预览原型交付；补充阶段结果、typecheck 基线缺口和生产验收边界 | 用户要求继续推进到完成 |
| v0.3 | 2026-09-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将导航改为 Project selector → Project Worktree Index → Worktree → 同级 Group Apps；确认多 Agent 可见性与服务端安全门槛 | 用户澄清核心痛点为多 Agent Worktree 混乱与内部管理不可控 |
| v0.4 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 固定 `/worktree` 为 Index canonical route，从 Project Worktrees 视图建立入口；移除其指向 Sprint 树视图的 redirect；补充 Worktree owner projection 缺口 | 本地 HTTP 检查显示 `/worktree` 返回 307 至 Sprint |
| v0.5 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 GroupContext 与跨 App 详细设计；将 Phase 2 拆成 2A 设计、2B 认证/ACL、2C 持久化任务闭环、2D Worktree Index 投影；Project Index 路由保留 `project_id` | REST WorkItem 路由复核确认默认 Actor、no-op auth 与 in-memory blocker |
