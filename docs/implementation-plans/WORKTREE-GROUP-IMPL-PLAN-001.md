# WORKTREE-GROUP-IMPL-PLAN-001

> **渡口 Project Worktree 群组实施计划 v1.0**
>
> - 状态：🟡 执行中（Phase 0/1、2A 完成；2B/2C/2D API 与 additive migrations 已实现并编译；Phase 3A scope guard 与 3B Canvas persistence/API 代码切片已落地；Phase 3 UI API 接线、Outbox consumer、Canvas WorkItem Command、数据库部署、成员 provisioning 与 ACL/RLS 负向验收待完成）
> - 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> - 日期：2026-09-29
> - 关联需求：`docs/requirements.md` v2.4 §50
> - 关联基本设计：`docs/basic-design.md` v0.5 §16
> - 关联详细设计：`docs/design/DD-WORKTREE-GROUP-001.md` v0.6、`docs/design/DD-MULTICA-TASK-001.md` v0.3、`docs/design/DD-WORKTREE-CANVAS-001.md` v1.3

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
| Project / Worktree | `/worktree?project_id=...` 是 Project Worktree Index；Project 页 Worktrees 视图传入选定项目；展开 Worktree 显示当前同级 App 导航；UI seed 任务不构成数据库关联 | Project 选择限定 Index 范围；生产命令必须拒绝缺少 Master binding 或 WorkItem-Worktree relation 的目标 |
| Worktree ownership | Phase 2B/2D migrations 定义 `worktree_project_binding`、`worktree_owner_assignment` Master/SCD2 与 owner projection；前端仍用 seed，旧 Worktree rows 未 reconciliation | 未知 owner 不猜；管理变更走 plan/confirm；Session/Runtime 当前只有 ref 投影，不伪造活跃状态 |
| Task Card | Phase 2C 增加 PostgreSQL metadata + Multica lifecycle + Worktree association API；UI、Review Gate、Jira alias/sync 和旧三态 REST 尚未接入 | `work_item_id` 是 canonical key；Task Card ID 与其相同；旧 seed 和 Jira key 不自动合并 |
| Canvas | Group UI 只加载 `ref_kind=worktree` 且 `ref_id=current_worktree` 的 Canvas；旧 Project/Free Canvas 与裸 `work_item_id` 不自动继承 | typed `EntityRef(ref_type,ref_id,worktree_id)` 才能跨 App 深链；服务端必须验证当前 Worktree association。Canvas persistence/API/Outbox 仍未接通 |
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
| Task Card 与卡内 CLI | Canonical WorkItem / Multica lifecycle + Agent/Local Runtime | Card 是 WorkItem projection；新写 API 只提供任务生命周期，不提供 CLI session；启动执行仍须服务端构造 TaskExecutionContext |
| Infinite Canvas | Canvas Domain | 保存布局和 EntityRef；任务创建/改状态通过 Lifecycle Command，不直写 WorkItem 表 |
| Workflow / LangGraph | Workflow Runtime / LangGraph | Run 固定不可变 scope snapshot；resume 重新检查当前 ACL、插件状态和 Worktree |
| Plugin Apps | Group App Registry + capability bridge | 插件入口与 agent graph type 分离；禁用立即拒绝新调用，保留业务事实与 disabled EntityRef |
| 底部聊天栏 | Group Shell Chat Adapter + L0 | 同一栏切换 WORKTREE/GLOBAL；GLOBAL 写必须明确目标、逐目标 preflight、逐目标结果和审计 |

## §3 分阶段实施与验收门

| Phase | 交付范围 | 依赖 | 完成门 |
|---|---|---|---|
| **0 设计基线** | 需求/基本/详细设计统一 Project 选择 → Worktree 管理 → 展开后同级 App、canonical ID、ownership、W/T/M 和权限规则 | `requirements.md` v2.4 §50、`basic-design.md` v0.5 §16、`DD-WORKTREE-GROUP-001.md` v0.5 | ✅ 已收口；SDK、Plugin sandbox 与在途撤权仍作为兼容门 |
| **1 Group Shell 原型** | 项目选择、项目 Worktree 清单与状态管理面板；展开 Worktree 后显示 Multica/Jira/Task Card/Canvas/Workflow/Plugins 同级导航；Canvas ↔ Task Card 深链；底部 scope selector | 现有 project nav store / frontend store / CanvasView | 🟡 预览交付：Project Worktrees 入口携带 `project_id` 进入 Index；浏览器可见按 Project 过滤的 Worktree 管理信号和同级 App 树；Task Card / CLI preview / WORKTREE-GLOBAL 选择可见，未连接的发送和命令保持禁用；owner 数据与服务端操作仍待接入 |
| **2 安全上下文与 canonical 任务闭环** | 2A GroupContext / 跨 App 契约；2B 认证 Actor、Project/Worktree ACL、GroupContextResolver；2C Postgres canonical WorkItem + Multica 生命周期、版本/幂等/审计；2D Cursor Index、owner/archive plan-confirm | 2A 设计基线；2B-2D migrations、membership provisioning、历史 reconciliation、ACL/RLS 与并发验收 | 🟡 2B/2C/2D API 与迁移代码已完成切片；DB 迁移和集成验收未完成，不能启用生产写路径；Jira sync、Domain adapter、Runtime/Git 状态接入仍是 blocker |
| **3 Canvas 双向联动** | 3A UI Worktree scope guard / 旧 seed 隔离；3B Worktree Canvas/Element/EntityRef persistence、版本化 API、Audit/Outbox 原子写；3C UI 接真实 API、Outbox consumer 与 Canvas→WorkItem Command | Phase 2B ACL + Phase 2C persistence / WorkItem API + Canvas migration/API + GroupContext | 3A/3B 代码切片完成但未部署；Canvas 不得直接写 Multica lifecycle；前端、Outbox consumer、Canvas 创建/关联任务 Command、旧 Canvas 冲突分类和 ACL/RLS 运行验收通过后才关闭 Phase 3 |
| **4 Task Card CLI** | `TaskExecutionContext`、本地 Runtime session 绑定、卡内 CLI、运行/取消/断线恢复 | Local Runtime 身份、Worktree checkout、工具与 secret capability ACL | CLI cwd 必须解析为授权 checkout；缺失/过期 scope 拒绝执行；执行记录可追溯，退出后清理 Work 状态 |
| **5 Group Chat + LangGraph** | Chat scope/targets DTO、服务端 GroupContext middleware、L0 router、Flow draft、LangGraph run/checkpoint、resume 再授权 | Phase 2B ACL、Phase 4 TaskExecutionContext、锁定的 LangGraph SDK compatibility review | WORKTREE 只读写本 Worktree；GLOBAL 写需显式授权目标；未授权目标零副作用；checkpoint 恢复不复活撤销授权；chat session、WorkItem、Task Card、LangGraph thread ID 分离 |
| **6 Plugin 热插拔** | manifest/version/capability、Group App Registry 动态发现、enable/disable/uninstall、热撤权和 UI 更新 | Phase 2B ACL + Phase 5 L0 capability bridge | disable/uninstall 后新调用即时失败；在途操作有确定取消/排空语义；业务 Transaction 与 Canvas ref 保留；agent registry 不冒充 app registry |
| **7 跨应用验收与发布** | Worktree → 每个 app 的端到端路径、权限矩阵、审计、故障注入、性能和迁移演练 | Phase 1-6 | §4 验收全过；已知缺口逐项关闭或保留为有 owner 的 release blocker；发布说明不把 preview 标为生产能力 |

Project 选择 → Project Worktree Index → 展开 Worktree → 同级 Apps 是产品主导航，Worktree 管理是多 Agent 协作核心。Phase 2B/2C/2D 的代码切片已完成，生产启用仍依赖数据库部署、受控 membership provisioning、归属 reconciliation 和负向集成验收。Phase 2D 当前不创建 Git Worktree、不探测 Runtime 活跃状态或执行物理清理。Phase 3-6 继续复用相同 GroupContext 和 canonical WorkItem；UI seed 不能代替生产 API。

**Phase 2B-2D 生产启用门（进入 Phase 3 生产联动前）**：先在目标数据库评审并按 2B → 2C → 2D 顺序演练/应用 additive migrations；建立 Project SoR 与受控 membership provisioning 后，只对有证据的旧 Worktree / Owner / WorkItem 关联做审计 reconciliation，歧义项隔离；随后验证跨 tenant/project 的 404/拒绝行为、scope/role 矩阵、RLS、复合外键、审计不可变、TTL、幂等并发重放和过期 version 冲突。上述任一项未验收前，新接口只视为未部署代码切片，不能启用生产写路径。另需完成 REST coordinator 到 Domain Command Port / PostgreSQL adapter 的架构门，以及真实 UI/API 接线，才可将 Phase 2 的产品验收标为完成。

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

1. **旧 WorkItem REST 路由仍不具备生产安全性**：legacy build_router() 保留 no-op auth / 默认 Actor / in-memory 服务，仅供旧测试；生产 binary 只挂载 JWT + ACL Group API。新 Group API 尚未从 REST SQL coordinator 抽为 Domain Command Port / PostgreSQL adapter。
2. **canonical task API 只完成首个持久化切片**：新接口持久化 Master metadata、六态 lifecycle、Worktree relation、version、幂等响应与 Transaction audit；Review Gate、metadata update、Jira external alias/sync 与 Outbox 尚未完成。
3. **服务端 scope-aware Chat 尚不存在**：当前 Chat W1 DTO 只有 `session_id/user_id/content`，`user_id` 由调用方传入，Transcript 在进程内 map；无 Worktree scope、授权目标或 GroupContext ACL。Phase 5 之前 composer 必须保持禁用或明确 preview。
4. **CLI 仍是 placeholder**：没有从 Task Card 到 Local Runtime 的可信 session provisioning；Phase 4 之前禁止实际执行命令。
5. **历史 WorkItem / Worktree seed 未绑定证据**：新建任务会写 canonical Worktree relation；既有 mock seed 和 Worktree legacy task_id 不构成生产关联证据，迁移歧义须隔离，不自动合并。
6. **Canvas 持久化与 EntityRef API 未接通**：Group 页面已拒绝自动显示 project/free Canvas，只接受显式 Worktree ref；没有 Group Canvas registry/element persistence API 时当前只呈现空状态。需实现 Worktree-scoped query、创建/关联命令与 Outbox，并将歧义旧 Canvas 隔离。
7. **LangGraph SDK/checkpointer 版本**：实施前读取仓库锁定版本并验证 Runtime/context/checkpointer API；Tier 3 按 ADR-0047 的有效前置门槛执行，未满足时采用已批准的较低 Tier 或将生产验收标阻塞。
8. **Plugin 热撤权语义**：需确定在途 run 的 cancel/drain 策略、manifest trust、签名与 API version compatibility；Phase 6 通过 ADR/详细设计冻结。
9. **开发 Group 守门未完成**：`group_guard.py` 已有脚本，但 `check_cross_ref` 仍 placeholder；`NEXT_PUBLIC_GROUP_ID` 仍是开发分组变量，未显示产品 GroupContext。两者分别在开发工具与 Group Shell 阶段处理。
10. **Phase 2B-2D 数据环境尚未启用**：migrations 尚未应用；Project Role Binding provisioning API / Project SoR 缺失，历史 Worktree 的 Project/Owner/WorkItem link 未 reconciliation；数据库 RLS、外键、隔离和负向 ACL 尚未真实验证。
11. **Phase 2D 管理能力边界**：当前 owner/归档 plan-confirm 有版本和审计，但 Session/Runtime 活跃检测只按 ref 存在保守拒绝；没有 Git lock 状态源、agent drain、Worktree create/import 或物理 cleanup。不能把归档 API 当 Git worktree 清理。
12. **W/T/M 与安全门未全通过**：新迁移逐表标注主分类、RLS 与保留规则；尚需完整 RLS policy 分类检查、真实 PG migration rehearsal 和 append-only / TTL 验收。

## §6 Change Scope 与最小验收顺序

1. 本阶段涉及 `crates/star-api-rest/`、`db/migrations/`、`docs/`；只在 managed Worktree 修改，不碰原始工作区主题改动。
2. Phase 1 使用现有 Zustand seed 做可交互原型；mock-only 功能需持续显示“预览 / 未接服务端”，不能模拟授权成功。
3. 先完成 TypeScript typecheck 与新增交互路径的 focused checks；之后再执行目标端到端路径。生产阶段追加 Rust workspace、DB migration、ACL 和 LangGraph 各自的守门。
4. 任何 Phase 变更须同步本计划的状态、验收和 release blocker；跨 W/T/M 的新增表按项目 DB 分类规则逐表归类。

### 6.1 本次阶段结果（2026-09-29）

| Phase | 结果 | 证据/限制 |
|---|---|---|
| 0 设计基线 | 🟢 完成 | 需求 v2.4 §50、基本设计 v0.5 §16、Group 详细设计 v0.5 与 Multica 详细设计 v0.3 对齐 Project → Project Worktree Index → Worktree → 同级 Apps；导航将 Worktree 管理作为多 Agent 协作主工作面 |
| 1 Group Shell | 🟡 预览完成 | Project 页 Worktrees 视图 → `/worktree?project_id=...` → Worktree Group 路由已在浏览器渲染；Index 显示 Agent/Runtime、branch、status、PR、lock 和最近活动；Group Shell 显示 Task Card、卡内 CLI 预览和 WORKTREE/GLOBAL 选择。数据仍为 seed/local；Worktree owner 字段、服务端 GroupContext、Chat API、真实 terminal session、创建/归档与 plugin runtime 尚未接通 |
| 2A GroupContext / 跨 App 契约 | 🟢 详细设计完成 | DD-WORKTREE-GROUP-001.md v0.5 定义认证 actor、Project/Worktree ACL、Worktree 管理 plan/confirm、TaskExecutionContext、Worktree typed EntityRef、Outbox、Chat/LangGraph 与 Plugin 撤权契约；后端只实现其中 2B-2D 子集 |
| 2B 认证 Actor / ACL / GroupContextResolver | 🟡 代码与 additive migration 完成，部署/ACL 验收待做 | production-only Group API；JWT sub/user_id 一致性；Project active membership；独立 Project Worktree Master binding；owner/task/link projection 不猜历史值。migration 未部署、membership 未 provisioning、跨 tenant/project/RLS 负向用例未跑 |
| 2C canonical 任务闭环 / 持久化 | 🟡 API 与 additive migration 完成切片，Domain adapter/数据库验收待做 | 新 Group API 写 canonical WorkItem Master、Multica 六态 Work current、Worktree SCD2 link、version、Idempotency-Key 和 append-only audit；`cargo check -p star-api-rest --all-targets -j 4` 通过。未应用 migration；REST crate 仍持有 SQL coordinator，旧 Domain service 保持 3 态 in-memory；Review/Jira alias/UI 没接通 |
| 2D Worktree Index 投影 / 管理 API | 🟡 Cursor Index 与 plan/confirm API 完成切片，数据库/运行状态验收待做 | stable keyset cursor + owner/state/archive filters；owner SCD2；归档/恢复 plan-confirm + expected version + audit；有 session/runtime ref 时保守拒绝 archive。没有 Git lock/Runtime active state source、create/import/physical cleanup；migration 未部署，UI 仍 seed |
| 3A Canvas scope guard | 🟢 前端边界代码切片完成 | Group 页面只查显式 Worktree Canvas；Canvas task link 要求 EntityRef 和任务关联都指向当前 Worktree；未绑定任务无 CLI 入口；UI 数据仍为 mock |
| 3B Canvas persistence/API | 🟡 后端代码切片完成，DB 未部署 | 新增 Worktree-scoped migration 与 Group API：Canvas list/create，Element list/create/versioned update/soft delete，typed EntityRef set/replace/clear；Project membership + scope/role、RLS GUC、当前 WorkItem association resolver、transactional Audit/Outbox 与幂等；EntityRef 变更同步推进 Element version，Outbox aggregate version 单调；`cargo check -p star-api-rest --all-targets -j 4` 通过；未应用 migration，未做 SQL runtime / ACL/RLS 负向验收 |
| 3C Canvas 跨 App UI 与事件消费 | ⚪ 待开始 | 前端尚未切到新 API；Outbox 无 consumer/realtime projection；Canvas 创建任务、Jira relation 尚未调用事实 owner Command |
| 4-7 | ⚪ 未开始 | 按 §3 依赖顺序推进；不得把 Phase 1 展示当作生产验收 |

本次 HTTP 验证：`/worktree?project_id=prj-mobile` 与 `/worktree/wt-003/group?app=task-card&work_item_id=wi-001&cli=1` 均返回 200，Index 响应保留 `project_id`，Group 响应保留 `wt-003`，没有 redirect。Phase 1 的浏览器预览记录：页面可见同级 App、任务详情、卡内 CLI 预览和禁用的 Chat 发送。`pnpm typecheck` 未通过，仍有 Monaco 子组件、terminal store、push client 和 MSW 重复导出等仓库诊断；定向 Vitest 在收集用例前被 `src/mocks/handlers/index.ts` 中重复导出的 `annotationsHandlers` 阻断。没有把 preview/seed 行为标成生产能力。

Phase 3A 已完成前端 Worktree scope guard；Phase 3B 已增加服务端 migration/API 的代码切片，涵盖 Canvas registry、元素、typed EntityRef、版本条件写入、幂等、Audit 与 append-only Outbox。前端 Group Canvas 仍来自 mock store，Outbox consumer / Canvas→WorkItem Command 未接入。本轮未运行浏览器或测试，未应用数据库 migration；`cargo check -p star-api-rest --all-targets -j 4`、改动文件 `rustfmt --check` 与 `git diff --check` 用于本轮静态验证，数据库 DDL / RLS / ACL 运行行为仍未验证。

---

Phase 2B-2D 本地验证：`cargo check -p star-api-rest --all-targets -j 4` 通过；改动 Rust 文件 `rustfmt --check` 与 `git diff --check` 通过。未运行测试，未应用数据库 migration，ACL/RLS 负向集成和数据库 SQL runtime 行为未验证。全仓 `cargo fmt --all --check` 已知因多处既存格式差异失败；CodeRabbit CLI 在当前 Windows/WSL 环境不可执行（`/root/.local/bin/coderabbit: Permission denied`），完成了本地自审。

## 修订履历

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-28 | Ulysses — Mavis 接手审核 | 建立 Worktree Group 分阶段实施路径、跨应用验收矩阵及当前服务端 blocker；明确开发 Group ID 与产品 Worktree scope 不同 | 用户要求基于 Worktree 顶层体系继续推进至完成 |
| v0.2 | 2026-09-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Phase 0 设计收口与 Phase 1 预览原型交付；补充阶段结果、typecheck 基线缺口和生产验收边界 | 用户要求继续推进到完成 |
| v0.3 | 2026-09-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将导航改为 Project selector → Project Worktree Index → Worktree → 同级 Group Apps；确认多 Agent 可见性与服务端安全门槛 | 用户澄清核心痛点为多 Agent Worktree 混乱与内部管理不可控 |
| v0.4 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 固定 `/worktree` 为 Index canonical route，从 Project Worktrees 视图建立入口；移除其指向 Sprint 树视图的 redirect；补充 Worktree owner projection 缺口 | 本地 HTTP 检查显示 `/worktree` 返回 307 至 Sprint |
| v0.5 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 GroupContext 与跨 App 详细设计；将 Phase 2 拆成 2A 设计、2B 认证/ACL、2C 持久化任务闭环、2D Worktree Index 投影；Project Index 路由保留 `project_id` | REST WorkItem 路由复核确认默认 Actor、no-op auth 与 in-memory blocker |
| v0.6 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 完成 Phase 2B JWT actor、Project ACL / GroupContext resolver、生产只读 Group API 与 SCD2 membership migration；继续推进 Phase 2C，显式记录迁移未部署、历史关联待 reconciliation 和权限负向集成缺口 | 用户要求继续完成 Phase 2B/2C/2D，并澄清 Worktree-first 导航及多 Agent Worktree 管理痛点 |
| v0.7 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 完成 Phase 2B/2C/2D 的 Group API 与 additive migration 代码切片：Project/Owner SCD2、canonical WorkItem/Multica 六态、version/idempotency/audit、stable cursor Index 与 owner/archive plan-confirm；明确数据库未部署、membership/reconciliation、Domain adapter、Jira sync、真实 Session/Runtime/Git 检查与 UI 接线仍未完成 | 用户要求连续推进 2B/2C/2D，并重申 Project Worktree Index 是解决多 Agent Worktree 混乱的主入口 |
| v0.8 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 明确进入 Phase 3 生产联动前的数据库迁移、Project SoR/membership、历史归属、RLS/ACL/幂等/TTL 验收门；记录 Task Project 关联校验与 failed 终态保留修正，避免把代码切片误报为生产完成 | Phase 2B/2C/2D 自审补强跨 Project 隔离和生产启用条件 |
| v0.9 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 加入 Phase 3 Canvas scope guard 结果：只接受 Worktree 显式绑定和 typed EntityRef，Project/Free seed 隔离，未绑定任务关闭 CLI 入口；将 Canvas persistence/API/Outbox 列为当前未完成工作 | Canvas 页面复用全局 Project seed 且跨 Worktree 任务深链未校验当前 scope |
| v1.0 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 分拆 Phase 3A/3B/3C；记录 3B migration/API 代码切片、实体引用/版本/幂等/审计/Outbox 边界和验证状态；明确 UI 接线、Outbox consumer、Canvas WorkItem Command 与 DB 部署仍是未关闭门 | 用户要求继续推进到完成，进入 Phase 3B Canvas 服务端联动 |
