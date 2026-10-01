# WORKTREE-GROUP-IMPL-PLAN-001

> **渡口 Project Worktree 群组实施计划 v5.55**
>
> - 状态：🟡 执行中（Phase 0/1、2A 完成；Phase 2B/2C/2D、Phase 3A-3F 有多项 API/UI/migration 代码切片，但宿主认证 provider、目标数据库部署、membership provisioning/reconciliation、ACL/RLS 运行验收、Domain adapter 与 durable realtime 仍未关闭；Phase 2D 已有 Git retention-lock observer/interface/UI 与认证 create/import API contract；Index 条件式 create/import controls 已接入脱敏 Repository/candidate API 并消费受理 receipt、刷新 Index，但 production main 未安装 lifecycle/Host Runtime provider，Project-Repository SoR 与 durable writer 未接通；活跃状态源、drain 与物理 cleanup 未实现；Phase 4A signed grant helper、4B1 Session start seam、4B2 PTY adapter、4B3 Task Card start/status/cancel/manual reattach UI、4B4 bounded Session listing/recovery seam 已实现，生产 provisioner、签名/nonce spawn wiring、实时 ACL/Runtime health、OS sandbox、terminal sink/scrollback、TaskRun Audit 仍缺；Phase 5/6 migrations 已在隔离 PostgreSQL 库重复执行并通过 12 表 FORCE RLS/策略/append-only 验证（事务临时 grants 已回滚）；目标库与 runtime role grants 未部署。Phase 5 已有逐目标 GroupContext 授权、加密 Transcript/W payload persistence seam 与 GLOBAL 目标目录；生产未接真实 protector/key lifecycle、outbox/L0/LangGraph、stream UI、provider 或目标 DB/RLS；Phase 6 已有五表 Master/SCD2 + append-only Audit migration、生产 main 装配的 PostgreSQL 只读 Registry provider、fail-closed API 和 Group UI live consumer，仍缺目标 DB 部署、受信任 manifest ingest/trust root、lifecycle writer、capability gateway/runtime、热撤权/在途 drain 与真实 RLS 验收；Phase 7 跨 App 生产验收未开始）
> - 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> - 日期：2026-10-01
> - 本轮补充：生产 Project 选择通过当前 actor 的服务端 membership 目录加载；API session generation 切换时同步隐藏旧 Project/Index/member-role 投影；分页与深链权限状态均 fail closed，缺失 Project 名称 SoR 时不借本地 seed 补名。
> - Phase 8 更新：Run migration 在隔离临时 PostgreSQL 重复应用并通过 6 张表 FORCE RLS 检查，目标 DB/runtime grants 未部署；CLI Run writer 与 Worktree/Task-scoped Run list/detail API、Task Card Run History 面板已形成条件式代码切片；Contract 写命令、完整事件/Evidence producer、真实 Runtime 与目标环境端到端验收仍未完成。
> - Phase 9A 更新：新增 `domain-hook` bounded Rust evaluator core 和 8 个单测，仅覆盖 Worktree archive/cleanup gate；未接入 lifecycle Domain Command、HookSet 持久化/发布、Advanced Settings UI 或 RunEvent/BI，Phase 9 仍开放。定向 test/Clippy 使用临时解析后的 lock 通过；原始锁文件仍有既存 `objc2 0.6.3` 与 Wry `^0.6.4` 冲突，完整刷新会改动 240 个 package，未混入本阶段。使用刷新锁进行的 `cargo check --workspace --all-targets -j 4` 随后被既有 `star-desktop/src-tauri/src/main.rs` 截断占位阻断。
> - Phase 9B1 更新：policy JSON 有界解码（≤64 KiB）、unknown-field 拒绝、scope/version/rule/action 校验、canonical SHA-256 核验和只读 verified snapshot 已加入 `domain-hook`；13 个定向单测与 Clippy 通过。此为运行时快照 contract，不是数据库策略存储/发布 API，也没有接 lifecycle command。原始 Cargo.lock 的 workspace 解析仍需临时生成锁才可运行目标 crate；Phase 9B2 继续完成 DB/RLS/store/publish 与实际双门。
> - Phase 9B2A/9B2B 更新：`db/migrations/2026-09-30-multica-hook-policy.sql` 写入 Project/Worktree SCD2 policy Master、短 TTL Draft 和 append-only policy audit 三表及 tenant FORCE RLS；迁移已在一次性隔离 PostgreSQL 18 集群重复应用，三表 FORCE RLS、Project/Worktree policy close/rebase、审计 scope 和 UPDATE/DELETE/TRUNCATE 不可变约束场景通过。9B2B 新增 8 条 Project/Worktree scoped policy REST routes，支持 effective read、Draft CAS/TTL、admin-gated publish/rollback、Audit 和 Project baseline 发布时的事务内 overlay rebase；定向 `cargo check -p star-api-rest --lib -j 4` 与 99 个 crate library tests 通过。原始 workspace lock 在 `--locked` 下仍要求重新解析，关联既存 Wry/objc2 依赖冲突；本阶段只补 `star-api-rest -> domain-hook` 直接依赖边，未引入约 240 个 package 的全量 lock churn。目标 DB/runtime grants/API DB-RLS integration 未就绪；correlation ID 是追踪值而非 replay key，响应超时后先重读 revision/Audit。9B2C production provider/DB integration、9C UI 条件式代码切片已加入但宿主 session Provider 未接、9D BI 与 9E Agent/Loop 继续开放。
> - Phase 9B2C 更新：archive-confirm REST gate 已读取当前 Project/Worktree verified policy 并调用 Rust evaluator；先在数据库行锁外检查 Git lock（最多 2 秒），仅新鲜 Unlocked 才请求最多 2 秒的 Host Runtime drain/readiness，并为稳定 operation ID 建立 admission fence；返回后重开短事务、复验 manager authorization、Worktree/plan 状态、effective policy、lifecycle version 与事实新鲜度。fence 必须至少留有 5 秒提交余量并在最终写入前复核；生产 provider 必须确保 fence 覆盖命令完成窗口，并与有界事务时限联合验收。Runtime provider 缺失/不可用或 fence 余量不足时 fail-closed；Deny 记录 Worktree management audit projection 并取消 plan，RequireHuman/Defer 保持 pending。现有 production main 未安装 readiness provider，且目标 DB/runtime grants/API RLS integration、物理 checkout cleanup、通用 Hook RunEvent/outbox 仍未完成。定向 check/test/Clippy evidence 与已知限制见 §6.45。
> - Phase 9D 首个事件切片：新增 Worktree archive HookEvent Transaction ledger、archive-confirm 同事务 producer、Project-scoped keyset read API 与 Advanced Settings → Hooks 页执行事件面板；当前仅 instrument `worktree_archive`，Run-linked producers、异步 Outbox delivery state、统一 BI read model/Run Detail 下钻、app root auth Provider、目标 DB migration/grants/RLS integration 仍开放；覆盖率输出 `partial` 且比例 `null`。Rust crate tests 106/106、Hooks/API 前端定向 25/25、TypeScript 与一次性 PostgreSQL 18 双次应用/append-only/RLS 场景通过。实现状态与验证结果见 §6.47。
> - Phase 9D-2 更新：新增 Project-scoped hook-events/summary API，默认 30 天、窗口 1–90 天，按 phase/decision 汇总 archive ledger 并返回 hook_execution_summary_v1、公式和 partial/null coverage；source-only API 不代表 RunEvent/outcome join 或完整 BI。定向 crate cargo check 在临时排除 star-desktop workspace member 后通过，未运行 tests；原始 Cargo.toml/Cargo.lock 已恢复，见 §6.48。
> - Phase 9D-3 更新：将 summary card 接入既有 Advanced Settings → Hooks 标签页，在内容区选择 7/30/90 天；导航遵循 ULYS-235，不新增 Worktree/主侧栏节点。Phase 9D-4 已将其数据 contract 升至 v2，见下文。
> - Phase 9D-4/9D-5a/9D-5b 更新：summary v2 合并字段完整的 Hook ledger/RunEvent 投影，按 tenant+event_id 去重并以 tenant/project/task/run 键关联最新 Run 状态；9D-5a 将原生 evaluator API 扩展为 phase-scoped v2，增加 BeforeRunAdmission 与 BeforeWorktreeArchiveCleanup，保留 v1 archive-only policy digest 兼容并拒绝 Run admission archive-only facts。9D-5b 增加条件式 Run admission producer seam：锁外 readiness/fencing，锁内最终重授权与 native Hook evaluation；Allow 原子写 Run HookSet snapshot、Run start、Hook ledger 和共享 event_id 的 RunEvent；Deny 只写无 Run/Task FK 的 ledger。policy publish/rollback、Builder 和事件 coverage 共用服务端 producer capability；当前 production adapter 未装配，能力仍 false，不能宣称 Runtime spawn 已有 production Hook protection，见 §6.52。
> - Phase 9E 更新：9E-1 Rust immutable Profile verifier、9E-2 bounded dependency resolver、9E-3 Profile Master/SCD2 + append-only Audit migration substrate 与 9E-4A Run/Profile guard migration 已交付并完成隔离 DB 验收；9E-4B1 已加入 Worktree-scoped bounded current Profile GET list/detail API 和 4 个 Rust 单测；9E-4B2 已加入 Project/Worktree Profile publish/disable/reenable/rollback 生命周期写 API 代码切片；9E-4B3 将 current verified Project/Worktree Hook policy 映射为 Profile admission 所需的 HookSet ID/version/digest；9E-4B4 明确双 Profile identity 与当前 Run writer 缺口；9E-4C1 已加入 Task Card Profile picker、request identity 与 versioned fingerprint/legacy replay 兼容；9E-4C2 已加入 Provider/Skill/GrantSet Master/SCD2 与 audit migration、reference-scoped SQL reader、Arc snapshot 传递和 final transaction fence recheck。隔离 PostgreSQL 18 与 targeted Rust 验证通过，但 catalog publisher/生产 Runtime provisioner、目标 DB/RLS grants 未部署；C3 atomic Run/resource writer 与 C4 Runtime fence 未接入，新 Run 继续 fail closed。occurrence/Loop runtime 仍开放，见 §6.54-§6.62。
> - Phase 9E-4C5 更新：新增 `star-dto::task_run` strict fence DTO；Local Runtime 使用签名 v2 认证双 Profile fence，并提供 current-binding compare 与 nonce/fence 同事务一次性消费；WAL receipt 上限 50,000 条，过期超过 5 分钟窗口后清理。此为 Runtime consume foundation，不连接生产 ACL/authority/catalog/Reservation/OS spawn/BI；profile-bound producer capability 继续默认关闭，见 §6.65。
> - 关联需求：docs/requirements.md v5.41 §50
> - 关联基本设计：docs/basic-design.md v5.37 §16.14-16.18
> - 关联详细设计：docs/design/DD-WORKTREE-GROUP-001.md v4.24、docs/design/DD-MULTICA-TASK-001.md v1.15、docs/requirements/SRS-MULTICA-HOOK-001.md v0.5.6、docs/design/BD-MULTICA-HOOK-001.md v0.5.8、docs/detailed-design/DD-MULTICA-HOOK-001.md v0.5.14、docs/design/DD-WORKTREE-CANVAS-001.md v1.3

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
| Worktree ownership | Phase 2B/2D migrations 定义 `worktree_project_binding`、`worktree_owner_assignment` Master/SCD2 与 owner projection；Index 已有条件式 live API projection/归档 UI，但宿主 provider 未安装、运行态仍 preview，旧 Worktree rows 未 reconciliation | 未知 owner 不猜；管理变更走 plan/confirm；Session/Runtime 当前只有 ref 投影，不伪造活跃状态 |
| Task Card | Phase 2C 增加 PostgreSQL metadata + Multica lifecycle + Worktree association API；UI、Review Gate、Jira alias/sync 和旧三态 REST 尚未接入 | `work_item_id` 是 canonical key；Task Card ID 与其相同；旧 seed 和 Jira key 不自动合并 |
| Canvas | 当前提交已有 Worktree-scoped Canvas/Element/EntityRef、Document API、Canvas→WorkItem 原子创建命令、受保护 Outbox API 和注入 JWT provider 后读取真实 GroupContext/WorkItem/Canvas 的 UI projection；默认运行态仍是 mock preview，因为宿主 provider 未装配 | typed `EntityRef(ref_type,ref_id,worktree_id)` 才能跨 App 深链；Element 内容、位置、几何与删除已有条件式 versioned write UI/API；DB 未部署、宿主 JWT provider、durable consumer/realtime 与 Canvas/Jira relation adapter 仍未完成 |
| CLI | `TerminalStackContainer` 无 session 时是 UI placeholder；没有按 TaskExecutionContext 启动的命令通道 | CLI 面板可以先做状态原型；不得显示为已连接 shell 或接受可执行命令 |
| Chat | `ChatPanel` / BFF W1 contract 只覆盖普通 session/message stub，不携带 GroupContext、GLOBAL targets 或 ACL | UI scope selector 可先落地；发送、检索、工具调用必须等 scope-aware API 和服务端 ACL |
| LangGraph | 既有设计区分 Group App Registry 与 `SubAgentRegistry`；ADR-0047 把 Postgres Tier 3 实装列为有前置门槛的工作 | LangGraph 负责执行图，不负责应用导航；执行恢复必须重新授权，checkpoint 不恢复历史权限 |
| Plugins | Group App Registry / capability bridge 在详细设计中有契约，前端没有运行时注册和撤权路径 | 先 manifest/registry contract，再做 enable/disable、调用拦截、撤权和卸载保留引用 |
| Group ID | `NEXT_PUBLIC_GROUP_ID=canvas/domain/frontend/core` 属于仓库协作/分支守门分类；不是用户产品群组身份 | 产品 Worktree scope 必须从路由实体 + 服务端认证解析；不得把此构建期变量当授权凭据 |

`docs/worktree-group-guard.md` 与 `scripts/automation/group_guard.py` 继续服务开发分组守门。#33 已由完整 Cargo resolved graph 检查 Canvas 与 Domain 源 crate 的双向直接依赖；它只约束开发分组 ownership，不代替产品级 GroupContext。

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
| **0 设计基线** | 需求/基本/详细设计统一 Project 选择 → Worktree 管理 → 展开后同级 App、canonical ID、ownership、W/T/M 和权限规则 | `requirements.md` v4.1 §50、`basic-design.md` v2.5 §16、`DD-WORKTREE-GROUP-001.md` v3.0 | ✅ 已收口；SDK、Plugin sandbox 与在途撤权仍作为兼容门 |
| **1 Group Shell 原型** | 项目选择、项目 Worktree 清单与状态管理面板；展开 Worktree 后显示 Multica/Jira/Task Card/Canvas/Workflow/Plugins 同级导航；Canvas ↔ Task Card 深链；底部 scope selector | 现有 project nav store / frontend store / CanvasView | 🟡 预览交付：Project Worktree Index 的 live API 消费代码仅在 provider 注入时启用；当前应用未安装宿主 provider，默认仍为带标记的 preview；同级 App 树、Task Card / CLI preview / WORKTREE-GLOBAL 选择可见，发送和真实命令保持禁用 |
| **2 安全上下文与 canonical 任务闭环** | 2A GroupContext / 跨 App 契约；2B 认证 Actor、Project/Worktree ACL、GroupContextResolver；2C Postgres canonical WorkItem + Multica 生命周期、review、版本/幂等/审计；2D Project 授权目录、Cursor Index、owner/archive/owner-transfer/create-import plan-confirm | 2A 设计基线；2B-2D migrations、membership provisioning、历史 reconciliation、ACL/RLS 与并发验收 | 🟡 `/api/v1/projects` 当前 actor membership 目录与生产 selector 分页/重试已实现，API session generation 切换时旧 Project/Index/member-role 投影立即隐藏；其余 Index、成员目录、归档/恢复/负责人转派、create/import 与 review command 有条件式切片；宿主 provider 未装配、DB 部署与 ACL/RLS 验收仍缺；Review Domain port、Jira sync、metadata update、Runtime/Git 状态接入仍是 blocker |
| **3 Canvas 双向联动** | 3A UI Worktree scope guard / 旧 seed 隔离；3B Canvas/Element/EntityRef API；3C Canvas Document viewport/frames/connectors SCD2 + CAS；3D Canvas→WorkItem 原子创建命令；3E Group UI 认证/API 接线、多 Canvas 列表/选择/初始化、Task Card 新建/关联、viewport/Element 位置与几何 CAS、Frame 与视觉连线的 Document 编辑、Outbox consumer 和实时投影 | Phase 2B ACL + Phase 2C persistence / WorkItem API + Canvas migrations/API + GroupContext | 3A-3D、授权 Outbox API、JWT adapter、provider-backed projection、多 Canvas `canvas_id` 选择/新建、Canvas→Task Card 新建与既有任务卡原子 EntityRef 关联、viewport/Element 位置与几何 CAS、Frame 创建/删除/元素归属/展示属性、纯视觉连线创建/删除/样式、无实体引用便笺编辑、安全 Element 删除及本地持久游标 poller 已有条件式 UI/API 切片；其他 Element 内容、宿主 provider、目标 DB、服务端 durable offset/realtime 未完成；Canvas 不得直接写 Multica lifecycle；部署、membership、Canvas 历史冲突分类和 ACL/RLS 运行验收全过后才关闭 Phase 3 |
| **3F WorkItem lifecycle UI** | Multica/Jira/Task Card 共用版本化 lifecycle API；六态操作按当前状态、review gate 和 active Worktree 过滤；CAS/幂等/冲突刷新 | Phase 2C WorkItem API + provider-backed projection | lifecycle 与 review submit/accept/reject 有条件式 UI/API 切片；服务端 ACL / writer role / lease / reviewer role 仍为最终守门；provider、DB deployment 与跨角色运行验收未过前不算生产完成 |
| **4 Task Card CLI** | 4A `TaskExecutionContext` 与 runtime mount / canonical checkout 一致性门；4B1 authenticated session start / grant seam；4B2 Local Runtime PTY adapter；4B3 start/status/cancel/reattach API 与 ticket-first xterm UI；4B4 Session listing/recovery | Local Runtime 身份、Worktree checkout、OS sandbox/path jail、Profile Registry、工具与 secret capability ACL、前端用户 Bearer session | Session start/status/cancel/reattach 与 bounded listing REST contract 已实现；Task Card 在页面刷新后可读最近 Session 并手动恢复。每个请求重验 Project writer membership / canonical Task/Worktree link 并委托 provisioner；listing 默认 20、服务端上限 50、脱敏/no-store 且不返回 ticket/output。仍缺真实 provisioner、signer/verifier/nonce spawn wiring、实时 ACL/health、OS sandbox、PTY sink/scrollback、Audit、宿主 UI provider 与 DB/ACL 运行验收，Phase 4 未关闭 |
| **5 Group Chat + LangGraph** | scope/targets DTO、受权目标目录与 UI、多目标选择、逐目标 GroupContext preflight、原子 Transcript metadata/encrypted payload/Run/idempotency/outbox persistence、L0 router、LangGraph run/checkpoint/resume 再授权与流式 Group Chat UI | Phase 2B ACL、Phase 4 TaskExecutionContext、LangGraph package/deployment/database-role compatibility review | 已有 `/chat/targets` 成员过滤目录、稳定 UUID 分页和底栏 Global 多选；`/chat/messages` 逐目标授权；`PgScopedChatWorkflow` 与七张 W/T + FORCE RLS schema 已实现原子 Session/Transcript metadata/queued Run/幂等 receipt/dispatch event/Audit，且强制注入 `TranscriptBodyProtector` seam。LangGraph 官方 Python checkpoint/thread/resume/replay API 语义已核对。Phase 5 migration 在隔离 PostgreSQL 库可重复执行，7 表 FORCE RLS、tenant/actor 隔离和 append-only trigger 已用事务内临时授权验证；目标 DB/runtime role grants 未部署。production `main.rs` 未安装 adapter；缺具体 KMS protector/密钥轮换销毁、payload cleanup/export-delete、outbox consumer/L0 worker、LangGraph runtime/checkpoint/resume、目标级 stream/发送 UI、宿主 provider，composer 保持禁用，Phase 5 未完成 |
| **6 Plugin 热插拔** | manifest/version/capability、Group App Registry 动态发现、enable/disable/uninstall、热撤权和 UI 更新 | Phase 2B ACL + Phase 5 L0 capability bridge | 五表 Master/SCD2 + append-only Audit/FORCE RLS migration 已落档，production main 装配只读 PostgreSQL provider，按 membership version、Worktree binding/archive、verified/current compatible manifest 与 actor `group_app:open` grant 复验；Group UI 消费投影、校验授权条目并 fail closed，WG-ACC-19 聚焦测试 3/3 通过。Phase 6 migration 在隔离 PostgreSQL 库可重复执行，5 表 FORCE RLS、scope policy 和不可变 trigger 已验证；目标 DB/runtime role grants 未部署。trusted manifest ingest、lifecycle writer、capability gateway/runtime 隔离、hot revoke/drain/cancel 仍未完成。agent registry 不冒充 app registry |
| **7 跨应用验收与发布** | Worktree → 每个 app 的端到端路径、权限矩阵、审计、故障注入、性能和迁移演练 | Phase 1-6 | 本地 Group/terminal 与 REST 回归基线已建立；生产验收仍未开始，须完成真实宿主身份、目标 DB/RLS、各 App E2E、故障注入与迁移演练；已知缺口逐项关闭或保留为有 owner 的 release blocker；发布说明不把 preview 标为生产能力 |

| **8A Run foundation** | Task Contract / Run / Event / Evidence migration；CLI start 创建 Run 并返回 Run ID | migration/RLS/append-only/idempotency 运行验收；重放同键与新尝试语义 | CLI writer 与 migration 已有条件式切片；migration 在隔离临时 PostgreSQL 重复应用，6 张 Run 表通过 FORCE RLS 检查；目标 DB、正式 runtime grants 和 production writer 验收未完成 |
| **8B Task Card Run integration** | Contract commands、Run list/detail、Task Card Run history、CLI/Agent/LangGraph 状态与 Evidence producer | tenant/project ACL、RLS、状态事实分离、evidence authorization/retention | 有界 list/detail API 与 Task Card 历史面板已实现；Contract 写命令、CLI 后续状态和 Agent/LangGraph/Validation/Review/Integration/Cost/Evidence producer、目标 DB/RLS E2E 仍缺 |
| **9 Agent Execution & Loop** | 9A bounded Rust Hook evaluator；9B1 policy decoder/verifier；9B2A SCD2 Master/Draft/Audit schema；9B2B scoped policy API；9B2C Worktree archive-confirm gate；9C Advanced Settings → Hooks Builder；9D Hook/RunEvent/BI seam；9E-1 Profile verifier、9E-2 dependency resolver、9E-3 Profile Master/SCD2 + Audit migration；9E-4A Run Profile snapshot all-or-none 与 scope/digest guard migration（隔离 PostgreSQL 验收通过）；9E-4B1 bounded current Profile GET list/detail 与 Rust read-time verifier（4 个单测）；9E-4B2 Project/Worktree Profile lifecycle write API 代码切片；9E-4B3 verified effective HookSet identity adapter；current Provider/Skill/Grant catalog adapter、生产 Run writer、Rust CLI adapter、唯一 Automation occurrence dispatcher、Schedule/Engineering Loop runtime 与公平有界并行调度 | profile/provider version/digest，scope/ACL，DB/RLS/SCD2/append-only，fair admission/backpressure，occurrence fencing/idempotency，Loop budgets/cancel/drain，Hook fail-closed 与 lifecycle command 二次校验，CLI process cleanup；Git lock 与 Agent lease 分离 | 9A/9B1 有 13 个单测与 targeted Clippy；9B2A migration 在一次性 PostgreSQL 18 隔离集群双次应用并验证三表 FORCE RLS、SCD2/rebase、Audit 不可变约束；9B2B 8 条 scoped routes 支持 Draft CAS/TTL、publish/rollback Audit 与原子 baseline overlay rebase；9B2C archive-confirm 的锁外 Git/Runtime 预检、admission fence、锁内重授权与 policy recheck 已实现，103 REST library tests/check/Clippy 通过，但 production readiness provider、目标 DB/RLS、physical cleanup 与通用 RunEvent/outbox 未验收；9C宿主认证和实环境仍开放；9D Run-linked producer/Outbox/full BI 仍开放。9E-1/2 domain core 完成，129/129 domain-agent tests 与 targeted Clippy 通过；9E-3 migration 在临时 PostgreSQL 双次应用，3 revisions/3 Audit、8 类负例与两表 tenant RLS 通过；组合 Run/Profile migration 确认 v1 Run 快照在 v2 successor 后不变，测试 DB/role 已清理。9E-4A guard migration 在隔离 PostgreSQL 重复应用通过；旧 Run 与 Project/Worktree 快照成功，5 类负例拒绝，零 Profile FK，临时库已清理。current Provider/Skill/Grant catalog adapter、Run writer/目标 DB部署/原子 reservation、CLI adapter、occurrence/Loop runtime、公平 scheduler 与生产验收仍开放。9E-4B2 Profile lifecycle write API 已有代码切片。ULYS-235 Hooks tab 位于 `/settings/advanced/hooks` 的高级设置内容区，与 Skills/MCP/Plugins 并列，不新增 Worktree 节点 |
| **10 Project Quality BI** | versioned Run/Loop/Hook/Worktree read model、coverage、cohort 与 authorized drilldown | 指标可复算，unknown 保留，不跨 Project 泄漏 | 设计定义；未实现 |
| **11 Benchmark / Improvement** | 固定 task/repo commit/environment replay、benchmark sets、Proposal adoption/rollback | tuning 与 holdout 隔离，评分/规则版本固定，重复性证据 | 设计定义；未实现 |
| **12 Rust desktop performance** | bounded projection/event/cache/terminal/evidence，Worktree/Run virtualization 与 Canvas culling | 目标设备 peak RSS、CPU、p95 更新延迟/frame time 与并发压力实测 | 指标 TBD，未实现 |
| **13 Integrated release** | Worktree/Task/Run/Loop/Hook/BI/Plugin 端到端、权限/故障/迁移/性能验收 | host identity、目标 DB/RLS grants、provider/runtime、安全门、性能基线全部有证据 | 尚未开始；外部依赖未就绪时明确保留 blocker |

Project 选择 → Project Worktree Index → 展开 Worktree → 同级 Apps 是产品主导航，Worktree 管理是多 Agent 协作核心。生产 Project Selector 查询 `/api/v1/projects` 的当前 actor 有效 membership 目录；该接口目前只能返回 Project ID/role，因为仓库未找到持久 Project 名称 SoR。Phase 2B/2C/2D 已有代码切片，生产启用仍依赖数据库部署、受控 membership provisioning、归属 reconciliation 和负向集成验收。Phase 2D 的 Index 控件已消费当前 Project 脱敏 Repository、候选发现和 create/import API，并校验回执后刷新；Host Runtime lifecycle provider、Project-Repository SoR、durable writer 和宿主认证仍未接通，所以生产 API 返回非成功且不会创建 Git Worktree。当前也不探测独立 Runtime 活跃状态或执行物理清理。Phase 3-6 继续复用相同 GroupContext 和 canonical WorkItem；生产 API 错误时 UI 不以 seed 冒充授权目录。

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
| WG-ACC-11 | Canvas Document 并发写入 | Frame / connector 仅引用当前 Worktree Canvas 的当前 element；过期 `expected_version` 返回 409；同 key 同请求重放原结果；visual connector 不写 WorkItem relation |
| WG-ACC-12 | Canvas Element 位置编辑 | live Canvas 仅拖动未锁定元素；更新绑定当前 Worktree/Canvas/Element，使用 Element `expected_version`、幂等键与 correlation ID；只改变坐标；冲突刷新授权 projection |
| WG-ACC-13 | Multica/Jira/Task Card 生命周期 | 同一 canonical WorkItem version；仅允许合法状态动作；review gate 和 active Worktree guard 有效；成功/冲突都刷新授权 projection，命令幂等且有 audit |
| WG-ACC-14 | Canvas Document 草稿 | viewport、Frame 创建/删除、Frame 元素归属和纯视觉连线创建/删除共享一个固定版本的 Document draft；显式 CAS 保存、稳定幂等键、冲突不自动重基；视觉连线不创建 WorkItem/Jira relation；可放弃冲突草稿并重载当前授权版本 |
| WG-ACC-15 | GLOBAL Chat 目标目录 | 仅列出当前 actor 有效成员所属的非归档 Worktree 最小投影；UUID 游标分页，最多选择 20 个；目录不授予权限，提交再逐目标授权；无 provider/workflow 时不得用 seed 或伪造发送 |
| WG-ACC-16 | Chat Transcript privacy | T 表只留不含正文的 append-only 元数据；正文经受信任 protector 加密后进入 W payload，应用 Project Policy TTL（默认最多 90 天），到期可物理清除/crypto-erase；验证 key rotation、授权导出/删除、RLS、无明文日志，缺 protector/cleanup 时生产入口关闭 |
| WG-ACC-17 | Plugin Registry projection | 仅投影当前兼容、active 且授权有效的插件；查询重验当前 membership/binding/grant，返回最小唯一稳定列表；无 provider 时 503；本地预览不得作为生产入口 |
| WG-ACC-18 | Plugin Registry schema / RLS | manifest、registry revision、binding、actor open grant 四类 Master 均有 SCD2 current-row 唯一约束、no-delete/immutable-history guard；Audit 是 append-only Transaction；五表 tenant/worktree FORCE RLS；真实 PostgreSQL 验证和 migration deployment 另为生产门 |
| WG-ACC-19 | Plugin 授权导航 UI | 有 API provider 时只从当前 Worktree 服务端 projection 生成同级入口；验证 Worktree ID、UUID correlation、Registry version、最多 100 条、唯一/受限 plugin ID、无控制字符且限长的字段和 int32 sort order，并稳定排序；加载、错误和会话切换期间不显示旧入口或预览，失败可重试；无 provider 时才展示醒目标记的本地预览；导航可见不授予 runtime capability |
| WG-ACC-20 | PostgreSQL 运行角色与 RLS | migration owner 与 runtime/worker principal 分离；以目标环境真实有效角色验证 schema/table 最小 grants、tenant/actor/worktree RLS 正反用例、append-only trigger 与无 policy 写入拒绝；禁止 superuser/BYPASSRLS；临时隔离库 grants 不作为生产通过证据 |

## §5 当前 Release Blocker / 已知缺口

1. **旧 WorkItem REST 路由仍不具备生产安全性**：legacy build_router() 保留 no-op auth / 默认 Actor / in-memory 服务，仅供旧测试；生产 binary 只挂载 JWT + ACL Group API。新 Group API 尚未从 REST SQL coordinator 抽为 Domain Command Port / PostgreSQL adapter。
2. **canonical task API 只完成首个持久化切片**：新接口持久化 Master metadata、六态 lifecycle、Worktree relation、version、幂等响应与 Transaction audit；Task review submit/accept/reject REST command 与条件式 Group UI 已有切片；Review Domain adapter、metadata update、Jira external alias/sync 与 Outbox projection 尚未完成。
3. **服务端 scope-aware Chat 仍未形成执行闭环**：新增 `/chat/targets` 在当前已授权路径 Worktree 下列出 actor 有效 Project membership 所属的非归档 Worktree，返回最小字段并稳定分页；GLOBAL 底栏可多选，提交 API 仍逐目标重新解析 GroupContext/Project membership 和 EntityRef Worktree scope。`PgScopedChatWorkflow` 已提供 PostgreSQL 持久化 adapter 与事务 outbox，但 production provider 未注入、目标 DB 未部署，且没有 outbox consumer/L0/LangGraph/checkpoint/resume、retention cleanup、streaming 与真实发送 UI。宿主 provider 缺失时不显示 seed 目标、composer 仍禁用；不得复用旧 Chat W1 的 caller-supplied `user_id` / in-memory transcript。
4. **CLI session 尚未接通**：Phase 4A 已添加 Local Runtime 执行上下文、approved profile、canonical checkout 校验、durable grant nonce consumption helper 与 Ed25519 signed-grant verifier；Phase 4B 已有 start/status/cancel/reattach REST contracts、ticket ledger、受保护路由 seam 和底层 `TaskPtyManager` PTY adapter，但这些仍未由生产 provisioner 编排。当前签名/nonce spawn wiring、ACL / Runtime health 重验、OS sandbox/path jail、TaskRun Audit、PTY 到 `terminal-stack` 的 sink/scrollback 接线、Task Card UI session provider / reattach 调用与端到端输出 redaction 未完成；未由可信 sandbox/provisioner 调用前不得启动命令。
5. **历史 WorkItem / Worktree seed 未绑定证据**：新建任务会写 canonical Worktree relation；既有 mock seed 和 Worktree legacy task_id 不构成生产关联证据，迁移歧义须隔离，不自动合并。
6. **Canvas 后端代码未部署、宿主认证未装配**：Worktree Canvas/Element/EntityRef、versioned Document CAS、Canvas→WorkItem 原子创建命令、受保护 Outbox API、依赖宿主 `GroupAccessTokenProvider` 的授权投影和 Canvas/Task Card 命令入口已有代码切片；Group UI 可暂存 viewport、Frame / connector 编辑、便笺内容、Element 位置/几何和删除，采用 Document 或 Element CAS 并在成功后刷新投影，失败时保留命令/草稿。宿主 JWT session provider 未接入，因此页面默认仍用 mock preview。Canvas route 有本地持久游标 at-least-once polling helper，但没有服务端 durable offset / realtime；其他 Element 内容、Canvas/Jira relation adapter、旧 Canvas 冲突分类、DB 部署、宿主 provider 和 ACL/RLS 负向验收未完成。
7. **LangGraph SDK/checkpointer 版本**：实施前读取仓库锁定版本并验证 Runtime/context/checkpointer API；Tier 3 按 ADR-0047 的有效前置门槛执行，未满足时采用已批准的较低 Tier 或将生产验收标阻塞。
8. **Plugin 热撤权语义**：需确定在途 run 的 cancel/drain 策略、manifest trust、签名与 API version compatibility；Phase 6 通过 ADR/详细设计冻结。
9. **开发 Group 守门仍有剩余项**：守门 #33 已实现 Cargo resolved graph 检查，验证结论与脚本状态见 §6.41；`NEXT_PUBLIC_GROUP_ID` 仍是开发分组变量，不代表产品 GroupContext。开发环境变量注入与产品服务端身份解析分别验收。
10. **Phase 2B-2D 目标数据环境尚未启用**：这些 migrations 尚未应用到目标数据库；Project Role Binding provisioning API / Project SoR 缺失，历史 Worktree 的 Project/Owner/WorkItem link 未 reconciliation；Phase 2 的数据库 RLS、外键、隔离和负向 ACL 尚未真实验证。Phase 5/6 的隔离库验证不覆盖 Phase 2 schema 或目标环境。
11. **Phase 2D 管理能力边界**：当前 owner/归档 plan-confirm 有版本和审计，Session/Runtime 活跃检测只按 ref 存在保守拒绝。Project Index 已有可注入的 Git Worktree retention lock observer seam 与 unknown-safe UI，但 production main 尚未安装 Host Runtime observer，因此真实 Git 保留锁状态仍未知；Git 保留锁也不表示 Agent 活跃或编辑互斥，unlocked 不等于空闲。当前没有可靠 agent 状态和 drain、Worktree create/import 或物理 cleanup。不能把归档 API 当 Git worktree 清理；unknown/unlocked 都不能通过 cleanup guard，清理必须读取独立活跃状态、drain 后重观测。
12. **W/T/M 与安全门未全通过**：新迁移逐表标注主分类、RLS 与保留规则；Phase 5/6 的隔离库已经完成迁移重放、12 表 FORCE RLS、scope 与 append-only trigger 验收；仍需完整 RLS policy 分类检查、Phase 2/3 及目标数据库 migration rehearsal、TTL/cleanup 与并发验收。
13. **数据库运行角色尚无可验证的授权配置**：本地隔离库只确认 `star_app` 角色存在且对 `multica` / `plugin` 无 schema `USAGE`；`star_app_role` 在该库不存在。迁移当前不给 schema/table grants，真实生产 DATABASE_URL 对应身份及连接池有效角色未确认。目标部署前必须建立 migration owner 与最小权限 runtime/worker roles，补齐逐表 grants，并用真实 service identity 验证 SQL privilege + FORCE RLS；不得以 superuser 或临时测试授权关闭此项。

## §6 Change Scope 与最小验收顺序

1. 本阶段涉及 `frontend/`、`crates/star-api-rest/`、`crates/domain-local-runtime/`、`crates/terminal-stack/`、`db/migrations/` 与 `docs/`；只在当前 managed Worktree 修改，不碰原始工作区主题改动。
2. Phase 1 使用现有 Zustand seed 做可交互原型；mock-only 功能需持续显示“预览 / 未接服务端”，不能模拟授权成功。
3. 先完成 TypeScript typecheck 与新增交互路径的 focused checks；之后再执行目标端到端路径。生产阶段追加 Rust workspace、DB migration、ACL 和 LangGraph 各自的守门。
4. 任何 Phase 变更须同步本计划的状态、验收和 release blocker；跨 W/T/M 的新增表按项目 DB 分类规则逐表归类。

### 6.1 本次阶段结果（2026-09-29）

| Phase | 结果 | 证据/限制 |
|---|---|---|
| 0 设计基线 | 🟢 完成 | 需求 v2.6 §50、基本设计 v1.0 §16、Group 详细设计 v1.3 与 Multica 详细设计 v0.3 对齐 Project → Project Worktree Index → Worktree → 同级 Apps；Canvas Document SCD2/CAS、Canvas→Task Card 原子创建契约、CLI grant nonce 与 attachment ticket W 分类已同步，导航将 Worktree 管理作为多 Agent 协作主工作面 |
| 1 Group Shell | 🟡 预览完成 | Project 页 Worktrees 视图 → `/worktree?project_id=...` → Worktree Group 路由已在浏览器渲染；Index 有授权 API 投影消费 seam 和 seed preview 双模式，Group Shell 显示 Task Card、卡内 CLI 预览和 WORKTREE/GLOBAL 选择。当前路由树没有安装宿主 provider，因此应用实际仍显示本地预览；真实 Project directory/session provider、Chat API、terminal session、创建/导入与 plugin runtime 尚未接通 |
| 2A GroupContext / 跨 App 契约 | 🟢 详细设计完成 | DD-WORKTREE-GROUP-001.md v0.5 定义认证 actor、Project/Worktree ACL、Worktree 管理 plan/confirm、TaskExecutionContext、Worktree typed EntityRef、Outbox、Chat/LangGraph 与 Plugin 撤权契约；后端只实现其中 2B-2D 子集 |
| 2B 认证 Actor / ACL / GroupContextResolver | 🟡 代码与 additive migration 完成，部署/ACL 验收待做 | production-only Group API；JWT sub/user_id 一致性；Project active membership；独立 Project Worktree Master binding；owner/task/link projection 不猜历史值。migration 未部署、membership 未 provisioning、跨 tenant/project/RLS 负向用例未跑 |
| 2C canonical 任务闭环 / 持久化 | 🟡 API 与 additive migration 完成切片，Domain adapter/数据库验收待做 | 新 Group API 写 canonical WorkItem Master、Multica 六态 Work current、Worktree SCD2 link、version、Idempotency-Key 和 append-only audit；`cargo check -p star-api-rest --all-targets -j 4` 通过。未应用 migration；REST crate 仍持有 SQL coordinator，旧 Domain service 保持 3 态 in-memory；Review/Jira alias/UI 没接通 |
| 2D Worktree Index 投影 / 管理 API | 🟡 API/条件式 UI 切片完成，数据库与宿主集成验收待做 | stable keyset cursor + owner/state/archive filters；owner SCD2；成员目录 API 只返回当前 Project 有效成员 ID/role；Index live projection、筛选、续页、owner reassignment 和 archive/restore plan-confirm UI 已接 API client；新增独立 `git_lock` observation DTO、注入式 Host Runtime observer、≤30 秒 freshness、单项 2 秒 / 每页 3 秒与最多 8 路并发；无 provider/runtime、错误、超时、无效时间戳均为 unknown；UI 将其与历史持久化 `locked` 标记分开。生产 main 尚未装配 observer，运行态仍 seed preview/lock unknown；agent drain、create/import/physical cleanup、migration 部署和 RLS/ACL 验收仍缺 |
| 3A Canvas scope guard | 🟢 前端边界代码切片完成 | Group 页面只查显式 Worktree Canvas；Canvas task link 要求 EntityRef 和任务关联都指向当前 Worktree；未绑定任务无 CLI 入口；UI 数据仍为 mock |
| 3B Canvas persistence/API | 🟡 后端代码切片完成，DB 未部署 | 新增 Worktree-scoped migration 与 Group API：Canvas list/create，Element list/create/versioned update/soft delete，typed EntityRef set/replace/clear；Project membership + scope/role、RLS GUC、当前 WorkItem association resolver、transactional Audit/Outbox 与幂等；EntityRef 变更同步推进 Element version，Outbox aggregate version 单调；`cargo check -p star-api-rest --all-targets -j 4` 通过；未应用 migration，未做 SQL runtime / ACL/RLS 负向验收 |
| 3C Canvas Document CAS persistence | 🟡 后端代码切片完成，DB 未部署 | 新增 additive migration：registry SCD2 保存 viewport/frames/connectors；PUT Document 执行 Worktree ACL、writer role、expected_version、幂等、同 Canvas 元素引用校验与 Audit/Outbox 原子提交；connector 不代表 canonical task relation；未应用 migration / 未做 PG、RLS 与 ACL 运行验收 |
| 3D Canvas→WorkItem 原子创建命令 | 🟡 后端 API 代码切片已实现并编译 | 同一事务创建 canonical WorkItem / Multica lifecycle / Worktree link / Canvas Element / EntityRef / Task Audit / Canvas Audit + Outbox；`cargo check -p star-api-rest --all-targets -j 4` 通过；未应用 DB migration，未验证 PostgreSQL/RLS runtime 行为 |
| 3E Group UI / Outbox consumer / realtime | 🟡 授权事件读取 + JWT API adapter + SQL validation | 新增受认证的 Canvas Outbox metadata polling API、Canvas-scope keyset index migration，以及宿主注入 `GroupAccessTokenProvider` 的 `WorktreeGroupApiClient`；adapter 10/10 Vitest、`pnpm typecheck` 通过。`CanvasOutboxPoller` 只有在 refresh callback 成功后才持久化并前移本地 Worktree/Canvas 游标，失败会重放当前页；此浏览器 consumer 仍不具备服务端 durable offset。Phase 2B-2D、3B、3C、3E additive migrations 在隔离验证库通过。`star_dev` 仍为空库；Group 页面仍 mock-only、宿主 JWT provider 未装配；缺真实数据 projection、服务端 durable consumer offset/realtime、前端真实 provider 接线与 Canvas/Jira Relation adapter |
| 4A TaskExecutionContext / checkout gate | 🟡 Local Runtime helper + durable nonce store 代码切片完成 | `TaskExecutionContext` 校验 tenant/project/repository/Worktree/runtime/profile 绑定、≤5 分钟 grant window、policy version、non-nil nonce 与 approved profile 非敏感静态参数、runtime mount 与 canonical checkout 精确匹配；新增专用 SQLite WAL `synchronous=FULL` 原子 nonce consumption；新增 Ed25519 grant signer/verifier 与验签后才校验上下文、consume nonce 的 helper；尚未由 Task Card API 调用，current ACL / Runtime health recheck 未接入 |
| 4B Task Card CLI Session | 🟡 受保护 transport、ticket ledger 与底层 PTY adapter 切片 | 新增首帧 `authorize` frame 路由：首帧校验成功后才注册 pane / 载入 snapshot；受保护路由要求注入 `TerminalEventSink`，ticket 由 Local Runtime SQLite WAL `synchronous=FULL` 保存 SHA-256 摘要并单次原子消费，绑定完整 Group / Task / Runtime / Session / policy version，TTL ≤ 60 秒。前端支持每次连接/重连获取新 ticket 并在收到 Hello 后报告已授权。新增 `TaskPtyManager` 提供交互输入/输出/resize/退出；仍未接 Task Session provisioner、GroupApiState 运行配置、真实 ACL authorizer、TaskRun Audit、terminal-stack sink/scrollback、OS sandbox 或 Group UI session/ticket，因此不能作为真实 CLI 使用 |
| 5-7 | ⚪ 未开始 | 按 §3 依赖顺序推进；不得把 Phase 1 展示或未部署代码切片当作生产验收 |

本次 HTTP 验证：`/worktree?project_id=prj-mobile` 与 `/worktree/wt-003/group?app=task-card&work_item_id=wi-001&cli=1` 均返回 200，Index 响应保留 `project_id`，Group 响应保留 `wt-003`，没有 redirect。Phase 1 的浏览器预览记录：页面可见同级 App、任务详情、卡内 CLI 预览和禁用的 Chat 发送。初次 typecheck 与 Vitest 阻塞记录为历史状态；Monaco/terminal 类型、MSW WebSocket mock 问题已在 §6.4 修复，当前验证结果以 §6.4 为准。没有把 preview/seed 行为标成生产能力。

Phase 3A 已完成前端 Worktree scope guard；Phase 3B/3C 已增加服务端 Canvas/Element/EntityRef 与 Document API/migration 代码切片；Phase 3D 增加一个事务内创建 WorkItem、Multica lifecycle、Worktree link、Canvas Element/EntityRef、Task Audit 与 Canvas Audit/Outbox 的命令。前端 Group Canvas 仍来自 mock store，Outbox consumer/realtime 未接入。本轮未运行浏览器或测试，未应用数据库 migration；`cargo check -p star-api-rest --all-targets -j 4`、改动文件 `rustfmt --check` 与 `git diff --check` 用于静态验证，数据库 DDL / RLS / ACL 运行行为仍未验证。

---

Phase 2B-2D 本地验证：`cargo check -p star-api-rest --all-targets -j 4` 通过；改动 Rust 文件 `rustfmt --check` 与 `git diff --check` 通过。未运行测试，未应用数据库 migration，ACL/RLS 负向集成和数据库 SQL runtime 行为未验证。全仓 `cargo fmt --all --check` 已知因多处既存格式差异失败；CodeRabbit CLI 在当前 Windows/WSL 环境不可执行（`/root/.local/bin/coderabbit: Permission denied`），完成了本地自审。

### 6.2 本轮阶段结果（Phase 3C/3D 与 4A，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| 3C Canvas Document API | 🟡 后端代码切片完成 | `group_canvas_registry` 扩展 viewport/frames/connectors，Document 使用 Canvas Master version 的 SCD2 序列；PUT 请求执行 JSON/范围/数量校验、当前 Canvas element reference 校验、`expected_version`、`Idempotency-Key` 和同事务 Audit/Outbox；迁移尚未应用，SQL/RLS 行为未在 PostgreSQL 验证 |
| 3D Canvas→WorkItem 原子创建命令 | 🟡 后端 API 代码切片已实现并编译 | 同一事务创建 canonical WorkItem / Multica lifecycle / Worktree link / Canvas Element / EntityRef / Task Audit / Canvas Audit + Outbox；`cargo check -p star-api-rest --all-targets -j 4` 通过，未应用 DB migration / 未验证 PostgreSQL 和 RLS runtime |
| 3E Group UI / Outbox consumer / realtime | 🟡 授权事件读取 + 浏览器本地游标恢复 | §6.5 已新增 Canvas Outbox metadata polling API 与复合索引 migration；生产 UI 仍需认证系统向 Next.js BFF 提供安全 session/token 契约；浏览器本地游标恢复已实现，仍需服务端 durable consumer offset 与 realtime projection |
| 4A TaskExecutionContext / checkout gate | 🟡 Local Runtime helper + durable nonce store 代码切片完成 | `prepare_task_cli_execution` 校验可信服务端上下文/profile/Worktree binding；`prepare_and_consume_task_cli_execution` 再通过 `CliSessionRegistry` 专用 FULL-sync 连接原子消费 W nonce，expires + 5 分钟后懒清理；低层 context helper 本身不验签；新 signed-grant wrapper 已执行验签，但仍未接入 Session API，也未重验当前 ACL/Runtime health 或执行进程；路径校验尚不能单独消除 TOCTOU |
| 4B Task Card CLI Session | 🟡 设计/基础设施盘点完成，尚未集成 | 复用 terminal-stack 协议/hub/snapshot；需替换 Noop sink、取消未授权 lazy pane、由 Local Runtime 提供真实 PTY 和 OS-enforced sandbox/path jail；Task Card API start 后返回短时单次 attachment ticket，WebSocket 首帧校验并消费；前端用户 Bearer token provider 与 session ref 未接入 |

本轮代码编译校验：`cargo check -p star-api-rest --all-targets -j 4` 与 `cargo check -p domain-local-runtime --all-targets -j 4` 通过；检查输出含仓库既有 deprecation/unused/dead-code 警告。改动 Rust 文件 `rustfmt --check` 与 `git diff --check` 通过；未运行测试、未启动浏览器、未应用数据库 migration。Canvas 仍是未接生产 API 的 mock 页面，TaskExecutionContext helper 仍未接 session API，代码切片均未计为生产验收完成。

### 6.3 本轮阶段结果（Phase 4B ticket / transport，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| 4B attachment ticket ledger | 🟡 Local Runtime helper 代码切片完成 | `task_cli_attachment_ticket` 保存 64 位十六进制 ticket 的 SHA-256 摘要、完整 Group/Task/Runtime/Session/policy binding 和 expiry；TTL 限制 ≤ 60 秒；SQLite `BEGIN IMMEDIATE` 原子消费，拒绝错误绑定、过期和重放；ticket 记录在 expiry + 5 分钟后懒清理；`cargo test -p domain-local-runtime --lib -j 4` 168/168 通过。尚无 session API 调用该 helper |
| 4B protected WebSocket route | 🟡 transport seam 完成 | 必须以首个文本帧提交授权 ticket；首帧仅允许 2 KiB，整条终端连接上限 1 MiB；授权成功后才注册 pane、创建 `WsTerminalSession` 并回放 snapshot；受保护状态强制注入 authorizer 与 `TerminalEventSink`。`cargo test -p terminal-stack --lib -j 4` 82/82 通过；路由未挂入 Group API，真实 ACL authorizer 与 PTY sink 未实现 |
| 4B browser transport | 🟡 客户端能力切片 | `TerminalWsClient` 支持每次连接/重连获取新 ticket、首帧提交和 Hello 后才报告 authorized；Group Task Card 页面尚未提供 session/ticket / Bearer provider。初始类型与 Vitest 阻塞已在 §6.4 修复；当前测试状态以 §6.4 为准 |
| 整体 Phase 4B | 🟡 未完成 | Session start/cancel/reattach/status API、签名 grant 验证、spawn 时 nonce 消费、当前 ACL/Runtime health recheck、真实 PTY、OS-enforced sandbox/path jail、TaskRun Audit、终端输出 redaction、Group router 和浏览器会话接线仍是必要完成门 |

本轮全仓 `cargo fmt --all --check` 被多处非本轮触及文件的格式差异阻断；本轮触及 Rust 源文件使用单文件 `rustfmt --edition 2024` 格式化。未应用 Canvas/PostgreSQL migration，未运行 SQL/RLS/ACL 生产验收。6.3 中初次记录的 frontend typecheck / Vitest 阻塞由 §6.4 后续结果取代。

### 6.4 本轮阶段结果（Phase 4B browser transport verification，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Ticket-first WebSocket client | 🟢 前端传输单测通过 | 修正 Vitest 中 MSW 全局 WebSocket 拦截与测试替身的装配顺序；覆盖授权帧必须先发送、收到 Hello 前拒绝 stdin、Hello 后开放输入，以及断线重连重新获取并提交新 ticket。`pnpm exec vitest run src/lib/terminal/wsClient.test.ts` 14/14 通过 |
| Task Card terminal preview state | 🟢 未绑定 Session 保持断开 | 没有 session 时不再把 preview pane 状态置为已连接；`TerminalSplitPane.test.tsx` 覆盖该条件 |
| Frontend typecheck | 🟢 全量 TypeScript 检查通过 | 将 `monaco-editor` 0.56.0 显式登记为现有 React wrapper 的 peer dependency；修复 Monaco type namespace 与 VAPID `Uint8Array<ArrayBuffer>` 声明。`pnpm typecheck` 通过；terminal、terminal preview 与 inline completion 三组定向测试共 33/33 通过 |
| 整体 Phase 4B | 🟡 未完成 | 已有 Session start REST seam、grant/ticket helper、受保护 transport 与底层 PTY adapter；仍缺生产 provisioner、真实 Group ACL/session authorizer、grant 签名调用与 spawn 时 nonce 消费、OS sandbox、PTY/scrollback sink、审计/redaction 及 Group UI session provider，不能启用真实 CLI |

此轮只改变终端 preview 的连接状态和前端验证装配，不改变需求/基本设计/详细设计中的授权边界；规范层仍沿用 requirements v2.6、basic design v1.0、DD-WORKTREE-GROUP-001 v1.3。本地 `pnpm install --frozen-lockfile` 与 `pnpm add monaco-editor@0.56.0 --save-exact` 后 `pnpm typecheck` 与 33 项定向 Vitest 通过。数据库 migration 未应用；Phase 4B 服务端接线和 Phase 3E/5-7 仍未完成。

### 6.5 本轮阶段结果（Phase 3E Canvas Outbox authorized polling，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Canvas Outbox events API | 🟡 受保护后端读取切片 | 新增 `GET /api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/events`；每个请求经认证 actor、`canvas:read`、当前 Project membership / Worktree scope 和 Canvas 存在性校验；游标 `(cursor_at,cursor_event_id)` 必须成对，分页默认 100、限制 1–200；仅返回事件元数据，不返回 payload / EntityRef 目标；新增 `2026-09-29-worktree-group-phase-3e-canvas-outbox-index.sql`，已在隔离验证库执行 |
| Cursor / page validation | 🟢 Rust 单元测试通过 | `cargo test -p star-api-rest --lib -j 4`：73/73 通过，其中 endpoint helper 覆盖默认/最大页长、超限页长、游标不完整与游标完整组合 |
| Group UI / consumer / realtime | ⚪ 未完成 | Group Canvas 仍为 mock-only，前端无 JWT session/token provider；尚无 Outbox consumer、持久 offset、projection refresh 接线或 SSE/WebSocket 实时推送；Canvas/Jira relation adapter 未完成 |
| 整体 Phase 3 | 🟡 未完成 | Group UI 联动、Canvas 历史冲突分类、真实 `star_dev` 数据库部署、Project membership provisioning、应用层 ACL / RLS 端到端与跨应用实时验收仍未关闭 |

本轮改动将 requirements v2.7 AC-CAN-004、basic design v1.1 §16.5、Group detailed design v1.4 API / Canvas 契约同步至该轮代码；implementation plan 更新至 v2.0。`cargo test -p star-api-rest --lib -j 4` 通过（73/73），`cargo check -p star-api-rest --all-targets -j 4`、本文件 `rustfmt --edition 2024 --check` 和 `git diff --check` 通过；Cargo 输出有仓库已有 deprecation / unused / dead-code warnings。实际 PostgreSQL 位于 localhost:5555。项目 `star_dev` 当前只有 `public` schema、0 张业务表，未应用生产/开发目标 migrations。为验证 DDL，我新建隔离数据库 `star_worktree_phase_validation_20260929`，用最小 Worktree fixture 成功应用 Phase 2B-2D、3B、3C、3E migrations；非特权角色的 tenant/worktree RLS scope 查询通过，跨 scope insert 被拒绝，Outbox UPDATE 被 append-only trigger 拒绝，复合游标在相同 timestamp 下按 event ID 稳定续页，`EXPLAIN` 选择 `idx_canvas_group_outbox_canvas_cursor` 的 Index Only Scan。此验证不代替完整 Star 基线、应用认证/ACL 集成或目标库部署。Group UI、consumer/realtime 仍未接，因此 Phase 3E / Phase 3 未完成。

### 6.6 本轮阶段结果（Phase 3E Group API auth adapter，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Group REST API adapter | 🟢 JWT adapter 切片完成 | 新增 `frontend/src/lib/group/worktreeGroupApi.ts`；token provider 按请求调用、不持久化、不经 URL/WS，Bearer 请求使用 `credentials: omit` 和 `cache: no-store`；拒绝远程 HTTP 与 scheme-relative host；无 token 在网络调用前返回 `session_required`；暴露 Project Worktree Index/filter、owner/archive plan-confirm、GroupContext、WorkItem、Canvas、Element、Outbox events 与 Canvas→WorkItem API 方法 |
| Adapter verification | 🟢 前端定向验证通过 | `pnpm typecheck` 通过；`pnpm exec vitest run src/lib/group/__tests__/worktreeGroupApi.test.ts`：9/9 通过；`git diff --check` 通过 |
| Host session / Group UI | 🟡 未接 | 仓库没有供 Group 路由注入的 JWT session provider；适配器只建立接线接口，页面仍使用 seed/mock；不得将开发用 shared API key 当作用户 JWT |
| 整体 Phase 3E | 🟡 未完成 | 仍缺宿主 token provider 装配、Group Canvas/Task Card 的真实数据投影、Outbox consumer/offset/realtime，以及目标数据库迁移/ACL/RLS 集成验收 |

requirements/basic/detailed design 分别同步至 v2.8/v1.2/v1.5，冻结 provider 边界和失效行为；本计划当时升至 v2.1。此进展没有把 Group 页面或 Phase 3E 标记为生产完成。

### 6.7 本轮阶段结果（Phase 3E Outbox cursor retry helper，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Outbox cursor retry helper | 🟢 代码与 focused verification 完成 | 新增 `CanvasOutboxPoller`：游标只在 projection refresh 成功后前移；refresh 失败保留旧游标并重读同一页；401/403 停止轮询，其他错误采用有界退避；游标只存在实例内存，没有持久 offset，也未接入 Group 页面 |
| Adapter verification | 🟢 通过 | `pnpm typecheck` 通过；`pnpm exec vitest run src/lib/group/__tests__/worktreeGroupApi.test.ts`：10/10 通过，覆盖失败重放与刷新成功后的复合游标推进；`git diff --check` 通过（Git 仅报告既有 LF/CRLF 转换提示） |
| 整体 Phase 3E | 🟡 未完成 | 仍缺宿主 JWT provider 装配、Worktree Group 页面 API projection、持久 consumer offset / realtime、Canvas/Jira relation adapter，以及目标数据库迁移和 ACL/RLS 集成验收 |

本轮没有修改需求、基本设计或 Group API 的授权边界；`CanvasOutboxPoller` 仅实现浏览器侧可复用的 at-least-once polling helper，不宣称是 durable consumer 或实时同步。Phase 3E 与 Phase 3 均保持未完成。

### 6.8 本轮阶段结果（Phase 3E Group 页面授权只读 projection，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Host provider seam | 🟢 注入点完成 | 新增 `WorktreeGroupApiProvider`，由宿主逐请求提供用户 access token、必填且非敏感的 `sessionKey` 和可选 API base URL；sessionKey 变化会重建 API client 并立即隐藏旧 projection；当前 frontend 没有可接入的 JWT session provider，因此应用根布局尚未挂载该 Provider |
| Group read projection | 🟢 页面接线代码完成 | Group 路由在 Provider 存在时从 API 读取经授权的 GroupContext、WorkItem 和当前 Canvas/Elements；只在 Canvas App 请求 Canvas scope；展示 API 只读标志；生命周期写入、Canvas 拖动/删除保持禁用；Provider 缺失时明确使用 mock preview；Provider 存在但请求失败时 fail-closed，不回退 seed |
| Outbox page refresh | 🟡 内存消费切片 | Canvas route 挂载 `CanvasOutboxPoller`；刷新 projection 成功后推进内存复合游标，失败重放；无持久 offset，未接实时推送，页面离开后 consumer 停止；目前取列表首个 Canvas，选择多个 Canvas 的 UI 仍待实现 |
| Verification | 🟢 focused checks 通过 | `pnpm typecheck` 通过；`pnpm exec vitest run src/lib/group/__tests__`：15/15 通过（API adapter 10 项 + projection/provider 5 项）；`git diff --check` 通过，剩余仅既有 LF/CRLF 提示 |
| 整体 Phase 3E | 🟡 未完成 | 需要真实宿主 provider 装配、目标数据库部署、完整数据与 Frame/connector/Element 写入 API、durable consumer/realtime、Canvas/Jira relation adapter，以及 ACL/RLS 集成验收 |

本轮新增认证注入、API DTO 映射、Group 路由只读 projection 和 Canvas outbox refresh 接线，但不把它们标成生产启用：仓库仍无 host session provider，目标 `star_dev` 尚无业务 schema；UI 因而继续默认显示预览。旧 Canvas/Jira 关系和未授权 seed 不会进入 API-backed projection。

### 6.9 本轮阶段结果（Phase 3E 登录会话切换隔离，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Session generation key | 🟢 前端隔离契约完成 | `WorktreeGroupApiProvider` 要求宿主提供非空、非敏感 `sessionKey`；principal 或登录会话变化必须更新 key；新 key 会重建 client，render 当下同步拒绝旧 request key 的数据，返回 loading 后按新 JWT 重新获取投影 |
| Security verification | 🟢 focused checks 通过 | Group projection test 在同一个 token callback 引用下轮换 token 与 session key，验证旧投影立即隐藏、新请求使用新 Bearer；Canvas app test 覆盖 Canvas/Element API projection 与 Outbox poller 启动，scope mismatch 测试确认跨 Project GroupContext fail-closed；`pnpm typecheck` 通过，Group focused Vitest 15/15 通过 |
| Overall blocker | 🟡 未关闭 | 实际宿主登录系统尚未提供/安装，无法核对 sessionKey 与真实 principal/logout 生命周期；真实 DB/API ACL 与端到端会话切换验收待 host integration |

requirements/basic/detailed design 同步为 v2.9/v1.3/v1.8。该 key 只用于客户端实例生命周期，不作为授权凭据、不发送或保存；后端仍必须逐请求验证 JWT。

### 6.10 本轮阶段结果（Phase 3E Canvas→Task Card 操作入口，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Worktree Canvas 初始化 | 🟢 条件式 UI 写路径完成 | Group 页面在真实 provider-backed live projection 且 Worktree 没有 Canvas 时显示创建入口，调用认证、幂等的 `POST /api/v1/worktrees/{worktree_id}/canvases`；未确认重试复用同一 correlation / idempotency key，授权 projection 确认 Canvas 后清除待确认键；失败不回退 seed |
| Canvas 创建任务入口 | 🟢 条件式 UI 写路径完成 | Canvas projection 到达后显示任务创建表单，调用既有 `POST .../canvases/{canvas_id}/work-items`；请求沿用逐次 Bearer token、幂等键和 correlation ID，未确认重试复用完全相同请求体；服务端已有同事务 WorkItem / Multica / Worktree link / Canvas Element + EntityRef / Audit / Outbox 命令 |
| 创建后数据一致性 | 🟢 授权投影刷新 | 任一创建成功后用 refresh key 立即卸载旧投影、重新加载；创建任务后提供 canonical Task Card 深链；API 错误保留错误状态，不在 seed 中伪造成功；Canvas 布局编辑仍只读 |
| Focused verification | 🟢 通过 | 本轮 `pnpm typecheck` 通过；`pnpm exec vitest run src/lib/group/__tests__`：18/18 通过，覆盖 Canvas 初始化、Canvas Document CAS adapter、创建任务 API、会话切换清理与投影刷新；`git diff --check` 待最终复核 |
| 整体 Phase 3E / Phase 3 | 🟡 未完成 | 真实宿主 JWT provider 未装配、目标数据库 migration 未部署；Frame/connector/Element 布局编辑、durable Outbox offset / realtime、Canvas/Jira relation adapter、历史冲突分类和 ACL/RLS 负向集成仍缺 |

此次在 requirements v3.2 §50.5、basic design v1.6 §16、Group detailed design v2.1 §8.1 同步空 Worktree 的 Canvas 初始化、Canvas→Task Card 创建顺序、viewport CAS 保存和真实实施状态。所有写入口均只在真实 provider-backed projection 下出现，不代表 Group 页面已接生产认证，也不关闭 Phase 3。

### 6.11 本轮阶段结果（Phase 3E Canvas viewport CAS 保存，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Viewport 操作 | 🟢 已接入 Group Canvas | live Canvas 的 pan、zoom、wheel 和 fit-to-content 暂存 viewport；只通过显式“保存画布视口”提交，live Group 不写 Zustand seed；提交期间阻断画布交互 |
| Canvas Document CAS | 🟢 前端命令接线完成 | adapter 新增认证 `PUT .../canvases/{canvas_id}/document`，携带当前 Canvas version、viewport、原 frames/connectors、correlation ID 和幂等键；成功后刷新授权投影，失败保留视口并显示错误；Frame/connector 编辑与 Element 写操作未接入 |
| Focused verification | 🟢 通过 | `pnpm typecheck` 通过；`pnpm exec vitest run src/lib/group/__tests__`：18/18 通过，含 Canvas Document route/method/Bearer/idempotency/body contract；最终 `git diff --check` 待复核 |
| 整体 Phase 3E / Phase 3 | 🟡 未完成 | 当前无宿主 JWT provider；目标数据库未部署；浏览器 Document CAS 未在真实登录态实测；仍缺 Frame/connector/Element 布局编辑、durable consumer/realtime、Canvas/Jira relations、RLS/ACL 负向集成和跨 App E2E |

本轮将需求/基本/详细设计同步至 v3.2/v1.6/v2.1；新增 viewport 保存不放宽 Canvas authorization，也不把 Phase 3 标为完成。

### 6.12 本轮阶段结果（Phase 3E Worktree 多 Canvas 选择与创建，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| 多 Canvas 投影 | 🟢 已接入 | Canvas App 加载当前 Worktree 全部授权 Canvas；`canvas_id` 路由选择指定 Canvas，元素读取和 Outbox poller 均绑定所选 ID；无效选择回退到有效授权 Canvas |
| Canvas 创建与切换 | 🟢 已接入 | 空 Worktree 初始化和已有 Canvas 下“新建 Canvas”均使用认证幂等 API；确认响应嵌套为 `canvas.canvas_id` 后切换 URL，并保留命令直至刷新授权投影确认新 ID |
| Focused verification | 🟢 通过 | `pnpm typecheck` 通过；`pnpm exec vitest run src/lib/group/__tests__`：18/18 通过；最终 `git diff --check` 待复核 |
| 整体 Phase 3E / Phase 3 | 🟡 未完成 | 多 Canvas 选择和已有 Task Card 关联入口已接入；仍缺宿主 JWT provider、目标数据库部署、Canvas frame/connector/element 编辑、durable consumer/realtime、Canvas/Jira 关系写入、RLS/ACL 负向集成与跨 App E2E |

本轮同步 requirements v3.3、basic design v1.7、Group detailed design v2.2；本地选择/创建验证不替代真实宿主认证、目标数据库部署或 Phase 3 生产验收。

### 6.13 本轮阶段结果（Phase 3E 已有 Task Card 与 Canvas 原子关联，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| 关联命令 | 🟢 已接入 | 从当前 Worktree 未关联 Task Card 中选择；`POST .../canvases/{canvas_id}/elements` 一次提交 `work_item_card` Element 与 typed EntityRef，服务端在 transaction 中校验 Worktree association 并原子写入 |
| 失败与重试 | 🟢 已接入 | 客户端按 Worktree / Canvas / WorkItem 保留 correlation ID、请求体与 Idempotency-Key；服务端确认成功前保留，切换 Worktree 或登录主体时清理 |
| Focused verification | 🟢 通过 | `pnpm typecheck` 通过；`pnpm exec vitest run src/lib/group/__tests__`：19/19 通过，含 Element 创建 route、Bearer、body 与幂等键 contract；最终 `git diff --check` 待复核 |
| 整体 Phase 3E / Phase 3 | 🟡 未完成 | 已有任务关联入口已移除 blocker；仍缺宿主 JWT provider、目标数据库部署、Canvas frame/connector/element 编辑、durable consumer/realtime、Canvas/Jira 关系写入、RLS/ACL 负向集成和跨 App E2E |

本轮同步 requirements v3.4、basic design v1.8、Group detailed design v2.3。此路径使用当前 Worktree 授权 WorkItem projection，客户端过滤重复卡片，不能替代宿主 provider 与生产数据库验收。

### 6.14 本轮阶段结果（Phase 3E Canvas Element 位置 CAS 保存，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| live Element 拖动 | 🟢 已接入 Group Canvas 条件式交互 | 只允许移动未锁定元素；拖动位置先在 CanvasView 本地预览，松开后通过 Worktree-scoped `PUT .../elements/{element_id}` 保存；不通过 Zustand 修改 live projection |
| CAS、幂等与刷新 | 🟢 前端命令接线完成 | 请求以 Element `expected_version` 为条件，完整保留现有内容、引用、尺寸、锁定与层级，只改坐标；带 `Idempotency-Key` 和 `correlation_id`；成功后刷新授权投影，冲突后也刷新且不覆盖新版本，失败显示错误并保留相同命令重试 |
| 当前验证 | 🟢 静态检查通过 | `pnpm typecheck` 和 `git diff --check` 通过；未增加或运行自动化测试；真实宿主 provider、目标数据库部署和跨成员并发仍未验证 |
| 整体 Phase 3E / Phase 3 | 🟡 未完成 | 仍缺宿主 JWT provider、目标 DB 部署、Frame/connector/内容/删除编辑、durable Outbox offset/realtime、Canvas/Jira 关系命令、ACL/RLS 负向集成和跨 App E2E |

本轮将需求/基本/详细设计同步至 v3.5/v1.9/v2.4。Element update API 虽已有服务端 handler，但没有宿主认证 provider 和目标库部署；此 UI 写路径仍是条件式代码切片，不代表生产能力已启用。

### 6.15 本轮阶段结果（Phase 3F WorkItem lifecycle UI 接线，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Canonical lifecycle projection | 🟢 UI adapter 切片完成 | live WorkItem projection 保留六态 `lifecycle_status`、正整数 `lifecycle_version`、`review_state` 和 `active_worktree_id`；Task Card、Multica 与 Jira 视图均由同一列表 projection 计算状态 |
| Lifecycle commands | 🟢 UI 命令接线完成 | 复用 `POST .../work-items/{work_item_id}/lifecycle`；发送 `expected_version`、`Idempotency-Key` 和 `correlation_id`；动作集合遵循六态迁移，review pending 隐藏完成动作，运行于其它 Worktree 的任务在 UI 禁用本地执行动作；成功与冲突后都重新加载授权 projection |
| 当前验证 | 🟢 静态检查通过 | `pnpm typecheck` 与 `git diff --check` 通过；本轮未新增或运行自动化测试；尚未验证宿主 JWT provider、writer/claim/review gate 服务端行为与目标 PostgreSQL 上的并发写入 |
| Phase 3F 的生产门 | 🟡 未关闭 | Group 页面仍未安装真实 provider，目标 DB/membership/RLS 未部署或完整验收；在线 lifecycle code slice 不等于生产写功能已启用 |

本轮将需求/基本/详细设计同步至 v3.6/v2.0/v2.5。继续阶段将处理 provider 与数据库部署/ACL 验收等环境门，并推进 Task CLI Session API / PTY sandbox、底栏 Chat/LangGraph 与 Plugin runtime；不得把当前前端接线报为生产闭环。

### 6.16 本轮阶段结果（Phase 6 Plugin App 同级导航预览，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| 动态同级导航 | 🟢 Group 页面原型已接入 | Plugins 页面中本地启用的预览插件会作为 Worktree 下的同级导航入口出现；停用后入口移除；插件详情页展示 manifest 能力声明和明确的预览限制 |
| 状态/授权边界 | 🟢 预览边界明确 | 启用状态只保存在页面内存；直达已停用或未知 plugin 路由回退到 Multica；预览页不安装插件、不写 Registry、不授权 capability、不执行插件代码 |
| 生产 Phase 6 | 🟡 未完成 | 尚无 Group App Registry API、发布者信任/签名校验、Project/Worktree plugin binding、capability grant/Gateway、隔离执行器、在途调用 drain/cancel 或撤权实时传播；此 UI 切片不等于热插拔 |
| 当前验证 | 🟢 静态检查通过 | `pnpm typecheck` 与 `git diff --check` 通过；未增加或运行测试套件 |

本轮将 requirements/basic/detailed design 同步至 v3.7/v2.1/v2.6。Phase 4 CLI、Phase 5 Chat/LangGraph 和 Phase 6 Plugin runtime 仍受真实服务端身份/授权/执行依赖阻塞；在这些依赖提供前只继续做不会伪造生产状态的安全切片。

### 6.17 本轮阶段结果（Phase 3E Canvas Frame / visual connector Document 编辑，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Canvas 选择与布局 | 🟢 UI 切片已接入 | Canvas 支持 Shift 多选；可暂存 Frame 新建/删除、标题/几何/演示属性、元素加入/移出 Frame、`kind=free` 视觉连线及其样式。样式变更只改布局，不改变 WorkItem/Jira 关系 |
| Document 草稿与 CAS | 🟢 UI 命令已接入 | viewport、frames、connectors 的完整 draft 保留初始 Canvas version；显式 CAS 携带 `expected_version`、correlation 与稳定幂等键。失败不自动重基；用户可放弃草稿并加载最新投影 |
| Element 文本与删除 | 🟢 条件式 API/UI 切片 | 无实体引用 Text/Sticky Note 使用 Element version CAS；删除接口以幂等事务关闭 Element/EntityRef、推进 Canvas Document version，并清除 Frame membership 与相连 connector；canonical WorkItem 不受删除影响 |
| 当前验证 | 🟢 静态检查通过 | `pnpm typecheck` 与 `git diff --check` 通过；未增加或运行测试套件；目标数据库、宿主 provider 与 ACL/RLS 运行行为未验证 |
| Phase 3 生产门 | 🟡 未关闭 | 目标宿主 provider / DB deployment、membership 与历史分类、ACL/RLS、durable Outbox offset/consumer/realtime、Canvas/Jira relation adapter、其他 Element 内容与并发验收仍未完成 |

本轮将 requirements/basic/detailed design 同步至 v3.9/v2.3/v2.8。Phase 3E 增加 Frame/connector 展示编辑、便笺 Element CAS 与引用安全删除的条件式 UI/API 切片；宿主 provider、目标 DB、ACL/RLS 与 durable consumer/realtime 未就绪时仍不开放生产写路径。

### 6.18 本轮阶段结果（Phase 3E Canvas Element 几何属性 CAS，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Element 尺寸与旋转 | 🟢 条件式 UI/API 切片 | live Canvas 中选择单个未锁定 Element 可编辑宽、高与旋转；提交使用 `update_mode=geometry`，仅允许 geometry 字段，通过 Element version CAS 固定 `expected_version`、`correlation_id` 和幂等键，服务端从锁定行保留其它字段；尺寸范围与后端校验一致 |
| 冲突与身份安全 | 🟢 fail-closed 交互 | 成功或冲突后刷新授权投影；失败保留本地 geometry draft，用户可显式重载；切换用户/Worktree 会清理命令身份和编辑状态；locked Element 无编辑入口 |
| 当前验证 | 🟢 静态检查通过 | `pnpm typecheck` 与 `git diff --check` 通过；未增加或运行测试套件；目标数据库、宿主 provider、ACL/RLS 与真实冲突行为未验证 |
| Phase 3 生产门 | 🟡 未关闭 | 仍缺宿主 JWT provider、目标数据库部署、membership/历史 reconciliation、ACL/RLS 负向验收、durable Outbox consumer/offset/realtime 与 Canvas/Jira relation adapter |

本轮将 requirements/basic/detailed design 同步至 v4.0/v2.4/v2.9。Element 几何编辑成为独立 versioned write slice；Phase 3 仍是未部署实现，不能作为生产验收完成。

### 6.19 本轮阶段结果（Phase 2D Worktree Index API / 归档管理 UI，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Project Index 读取 | 🟢 条件式 API UI 接线 | Worktree Index 在获得 `WorktreeGroupApiProvider` 时调用认证 Project cursor API；校验响应 Project 与每行归属，显示 owner、Agent Session、Runtime、Worktree status、PR、dirty/ahead/behind、health/risk、lock/version，并支持 owner/state/archive 筛选与 cursor 下一页 |
| 管理操作 | 🟢 Archive/restore plan-confirm UI 接线 | live detail 用当前 version 创建短时 `set_archived` plan，校验 plan 与 Worktree 绑定，再经用户二次确认；确认成功后重读授权 Index；现有 AgentSession/Runtime 引用时 UI 预先禁用 archive，服务端仍最终重验。该动作不执行 Git checkout 删除 |
| Preview / fail-closed | 🟢 来源边界明确 | API 错误时显示错误并禁止 seed fallback；当前应用路由树尚未安装宿主 provider，因此默认实际显示本地预览，不能报告为生产 API 已运行 |
| Owner reassignment / create / cleanup | 🟡 未完成 | Owner 转派 UI 仍缺 Project member directory projection；创建/导入需 Repository/Runtime provisioning；物理 cleanup 需 Git lock、Session/Runtime drain 与受审计执行器 |
| 验证 | 🟢 隔离 PostgreSQL 验收通过 | Run、Profile、guard 三 migration 应用成功；guard migration 重复应用；旧 Run 与两类完整 snapshot 接受，5 类负例拒绝；accepted Runs=3（legacy=1、Profile snapshot=2），Profile FK=0；临时数据库清理并复查不存在。目标数据库未触碰。 |

| Phase 2D / Phase 2 总体 | 🟡 未完成 | 宿主 provider、目标数据库部署、membership provisioning/reconciliation、Domain adapter、RLS/ACL 负向与并发运行验收仍是关闭门 |

requirements/basic/detailed design 已同步为 v4.1/v2.5/v3.0。此 UI 切片加强多 Agent Worktree 的集中可观测与显式归档管理，但不代表当前产品路由已有真实认证 provider，也不代表 Git Worktree 的创建、停止或物理清理已打通。

### 6.20 本轮阶段结果（Phase 2D Project member directory / owner reassignment，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Project member directory | 🟢 条件式 API | 新增 `GET /api/v1/projects/{project_id}/members`；必须使用有效 Bearer、`project:read` 和当前 Project membership；响应仅包含有效成员 UUID / role，不查询其它 Project 身份 |
| Owner reassignment | 🟢 条件式 UI/API 接线 | Index 负责人选择仅从上述成员目录取值；`assign_owner` 使用当前 Worktree version、correlation/idempotency key、5 分钟 plan 和显式确认；确认后刷新授权 Index，失败/过期不乐观更新；服务端确认仍重验 manager role、Project membership、owner membership 与 version |
| fail-closed | 🟢 保持 | 成员目录错误时关闭转派入口；provider 缺失时 Index 仍明确显示 preview，不借用本地 seed 的成员或 owner |
| 验证 | 🟢 隔离 PostgreSQL 验收通过 | Run、Profile、guard 三 migration 应用成功；guard migration 重复应用；旧 Run 与两类完整 snapshot 接受，5 类负例拒绝；accepted Runs=3（legacy=1、Profile snapshot=2），Profile FK=0；临时数据库清理并复查不存在。目标数据库未触碰。 |
| Phase 2D / Phase 2 总体 | 🟡 未完成 | 宿主 provider、目标库部署、membership provisioning / 历史 reconciliation、Domain adapter、真实 ACL/RLS 负向与并发验收、Git/Runtime lifecycle 仍是关闭门 |

本轮将需求/基本/详细设计同步至 v4.2/v2.6/v3.1。Index 现在具备 owner directory 与二次确认转派的条件式 UI/API 路径，但当前应用没有安装登录 provider，因而不能宣称用户已能在运行中的产品里真实转派。

### 6.21 本轮阶段结果（Phase 3E Canvas Outbox browser cursor recovery，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Browser cursor recovery | 🟢 条件式客户端 consumer | 新增按 Worktree/Canvas scope 隔离的 `BrowserCanvasEventCursorStore`；游标从 `localStorage` 恢复前进行结构、时间与 UUID 校验；本地存储不可用或数据损坏时回退至 at-least-once 重放；游标不是授权凭据 |
| Cursor commit ordering | 🟢 先刷新后前移 | 每个事件页仍请求受保护 API；仅在刷新当前授权 Canvas projection 成功后写入下一复合游标并推进当前 poller；刷新失败保留旧游标并重放 |
| Verification | 🟢 静态检查通过 | `pnpm typecheck` 与 `git diff --check` 通过；未运行测试；浏览器实际存储、登录态 provider 和目标数据库未验收 |
| Phase 3E / Phase 3 总体 | 🟡 未完成 | 本地恢复游标只覆盖浏览器投影刷新，不是服务端 durable consumer offset；宿主 JWT provider 未装配，目标数据库未部署，缺 NATS / SSE / WebSocket realtime、Canvas/Jira relation adapter、历史 reconciliation 与 ACL/RLS 运行验收 |

本轮将 requirements/basic/detailed design 同步至 v4.3/v2.7/v3.2。Group 路由当前仍 preview，因此恢复游标实现尚未在真实用户会话下运行；不将本地持久化等同于服务端可靠事件消费。

### 6.22 本轮阶段结果（Phase 2C / 3F WorkItem review command，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Review command API | 🟢 条件式 REST 切片 | 新增 `POST .../work-items/{work_item_id}/review`；submit 仅允许当前 active claimant；accept/reject 仅允许当前 Project `tenant_admin` / `project_admin` / `developer` 且不得自我评审；reject 必须带理由；所有动作绑定当前 Worktree association、lifecycle version、Idempotency-Key 与 correlation ID |
| Domain transaction | 🟢 复用现有 WorkItem 表 | 在同一 SQL transaction 锁定 lifecycle、更新 review/status、追加 lifecycle audit、写入 idempotency response；accept 从 `in_progress/pending_review` 转 `completed/accepted`，reject 转 `failed/rejected`，submit 仅更新 review state；无 schema migration |
| Group UI | 🟢 条件式接线 | live projection 下提供提交评审、通过、驳回和驳回理由；成功或冲突刷新授权投影；无宿主 provider 时仍 preview，后端 ACL 是最终裁决 |
| Verification | 🟢 静态编译通过 | `pnpm typecheck`、`cargo check -p star-api-rest --all-targets -j 4` 通过；未运行测试；目标 DB、真实 JWT provider、角色矩阵运行验收未完成 |
| Phase 2C / 3F 总体 | 🟡 未完成 | Review Domain port / Outbox event、Jira alias/sync、metadata edit、provider/DB 部署及跨角色 ACL/RLS 验收仍待完成 |

本轮将 requirements/basic/detailed design 同步至 v4.4/v2.8/v3.3。Review API 目前与其它 Group API 一样依赖真实宿主 JWT 和已部署 PostgreSQL；UI preview 中不会伪造评审状态。

### 6.23 本轮阶段结果（Phase 4A Ed25519 Task CLI grant 验证入口，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Ed25519 grant format | 🟢 独立 signer/verifier 代码切片 | grant 签署完整 `TaskExecutionContext`，使用 `key_id` 和 `star.task-cli.execution-grant.v1` 域分离 payload；issuer signer 接收 PKCS#8 私钥，Runtime verifier 只接收公钥表，支持 key id 轮换且保留 retiring key 至在途 grant 过期（最长五分钟），不共享签名密钥 |
| Fail-closed execution helper | 🟢 验签顺序代码切片 | `prepare_and_consume_verified_task_cli_execution` 先验签，再校验 scope/profile/canonical checkout 并通过 SQLite FULL-sync store 原子消费 nonce；未知 key、损坏签名及任何准备/持久化错误均不会返回可 spawn 请求 |
| Phase 4A / 4B 总体 | 🟡 未完成 | signer 尚未由 REST Session API 使用，Runtime verifier 配置/公钥轮换未部署，当前 ACL 与 Runtime health recheck、Task Session API、spawn-time wiring、PTY、OS-enforced sandbox、TaskRun audit / output redaction 仍未接通；签名不代表运行授权完成 |
| Verification | 🟢 编译与静态检查 | `cargo check -p domain-local-runtime --all-targets -j 4`、改动 Rust 文件 `rustfmt --check` 与 `git diff --check` 通过；未运行测试 |

本轮将 requirements/basic/detailed design 同步到 v4.5/v2.9/v3.4。Phase 4 继续依赖真实 Session API、current ACL / Runtime health、受隔离的 PTY sandbox 与已安装宿主认证 provider；当前实现只封闭签名验证这个执行前门。

### 6.24 本轮阶段结果（Phase 4B1 Task Session start REST seam，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Group REST route | 🟡 fail-closed 接入切片 | 新增 `POST /api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/cli-sessions` 并挂入 Group router；检查 Bearer actor、`agent_session:start`、当前 Project writer membership、Worktree/WorkItem canonical association、`in_progress` claimant、`expected_lifecycle_version`、Runtime binding，并将 Approved Profile ID 与 correlation ID 传给 provisioner；事务结束后再调用 provisioner，避免持有 DB 锁跨进程调用 |
| Provisioner contract | 🟡 注入边界已定义 | `GroupApiState::with_task_cli_session_provisioner` 注入 `TaskCliSessionProvisioner`；默认未配置返回 503。Provisioner 需按 tenant/actor/idempotency key 幂等、对比 request fingerprint，并在 grant/spawn 前重查当前 ACL、任务 version、Runtime health、profile 和 sandbox；当前 main 未装配实现 |
| 成功响应保护 | 🟡 响应契约已实现 | 仅接受 running session receipt 与不超过 60 秒的 ticket expiry；输出 ticket 设置 `Cache-Control: no-store`，response 包含 session/worktree/work-item/runtime/correlation；当前没有真实 provisioner，因此成功 ticket 路径未在运行环境验证 |
| Verification | 🟢 静态编译通过 | `cargo check -p star-api-rest --all-targets -j 4` 通过；本轮未运行测试。编译仍报告仓库其它模块已有 deprecation/unused/dead-code warnings；新增 import warning 已清理 |
| Phase 4B / 全体 Phase | 🟡 未完成 | 仍缺实际 Session provisioner / Ed25519 signer 调用、Runtime verifier 配置与 nonce spawn wiring、实时 ACL/Runtime health 复验、TaskRun Audit、PTY、OS-enforced sandbox、Session status/cancel/reattach API、Task Card UI/session provider、目标 DB/ACL/RLS 与生产身份部署；Phase 5-7 和其余跨 App 验收仍待推进 |

本轮同步 requirements/basic/detailed design 至 v4.6/v3.0/v3.5。该 route 是可注入的受保护控制面边界，不会自行创建会话；无 provisioner 时明确服务不可用。`cargo check -p star-api-rest --all-targets -j 4`、两份 Rust 源文件 `rustfmt --check` 与 `git diff --check` 均通过；未运行测试。编译输出仅见仓库已有 deprecation / unused / dead-code warnings；数据库迁移、运行态 ticket 发放和终端连接未验证。

### 6.25 本轮阶段结果（Phase 4B2 Local Runtime PTY adapter，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Local Runtime PTY adapter | 🟡 交互终端代码切片 | 新增 crate-internal `TaskPtyManager` / `TaskPtySession`：只接收已准备的 `PreparedTaskCliExecution`，使用 `portable-pty` spawn 固定 executable/argv/canonical cwd；清空继承环境后设置 `TERM` 与已批准静态变量；输入上限 64 KiB，cols/rows 范围 1-500，输出经容量 256 的单消费者 FIFO 队列保持带序号原始字节，队列满时 reader 背压而不静默丢弃 attachment 前输出；manager 关闭时终止其子进程。该模块不验证签名、ACL、Runtime health，不写 TaskRun Audit，不持久化 scrollback，不实现 OS sandbox；没有 Session API / WebSocket / Group UI 调用它 |
| Fail-closed 边界 | 🟡 仍未关闭 | REST route 默认没有 provisioner 且返回 503。PTY adapter 明确要求调用方先做实时授权、grant 验签、nonce 消费、审计与 OS sandbox 检查；sandbox 未装配，生产 CLI 仍不能启动 |
| Verification | 🟢 静态编译通过 | `cargo check -p domain-local-runtime --all-targets -j 4` 通过；`rustfmt --edition 2024` 已格式化本轮 Rust 源；未运行测试。依赖锁定 `portable-pty 0.9.0`；本轮不宣称 PTY 行为或 OS sandbox 已经运行验收 |
| Phase 4B / 全体 Phase | 🟡 未完成 | 仍缺 OS-enforced sandbox/path jail、Session provisioner 与 API signer / Runtime verifier 配置、实时 ACL / Runtime health、TaskRun Audit、terminal-stack adapter / durable scrollback、Session status/cancel/reattach、Task Card UI session provider、目标 DB / ACL / RLS / 身份部署；Phase 5-7 未完成 |

本轮同步 requirements/basic/detailed design 至 v4.7/v3.1/v3.6。PTY 只解决交互终端 I/O，不提供 OS 隔离，且当前没有任何 Group API 生产路径调用该 adapter。未运行测试、未应用数据库 migration，也未接入可执行的 Task CLI session。

### 6.26 本轮阶段结果（Phase 4B2 PTY 输出订阅竞态与进程回收，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| PTY 输出 attachment 前缓冲 | 🟢 bounded adapter contract | 将可能在客户端订阅前被 broadcast 丢弃的 PTY output 改为容量 256 的单消费者 FIFO；PTY reader 通过 blocking backpressure 保留顺序并等待 sink 消费。它不等于 durable scrollback，队列只存在于 session 进程内存 |
| Runtime 关闭清理 | 🟢 本地生命周期切片 | `TaskPtyManager` 丢弃时终止仍由其管理的子进程；单个 session 的 terminate 对已退出子进程幂等。尚无服务级 shutdown wiring，也未完成 OS sandbox |
| Verification | 🟢 静态编译通过 | `cargo check -p domain-local-runtime --all-targets -j 4` 通过；未运行测试。CodeRabbit CLI 本机启动失败（`/root/.local/bin/coderabbit: Permission denied`），改由本地 diff/source 复核 |
| Phase 4B / 全体 Phase | 🟡 未完成 | 仍缺 OS-enforced sandbox/path jail、真实 grant/ACL/health/audit provisioner、PTY 到 terminal-stack sink 与 durable scrollback、Session status/cancel/reattach、Task Card UI session provider，以及 Phase 2/3/5/6/7 的生产 provider、数据库、运行时与跨 App 验收 |

本轮同步 requirements/basic/detailed design 至 v4.8/v3.2/v3.7。PTY 输出不再依赖尚不存在的 WebSocket receiver 才能启动；这只消除内存队列未连接导致的首段输出丢弃风险，不表示 attachment / sandbox / durable scrollback 已完成。
### 6.27 本轮阶段结果（Phase 4B3 UI 与 Phase 5 scoped Chat authorization seam，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Phase 4B3 Task Card UI | 🟡 条件式 UI/API 切片 | Task Card 只提交批准的 Profile、lifecycle version、correlation 与幂等键；只有真实 Session receipt 后挂载 xterm；ticket 仅存在页面内存并单次提交首帧，Hello 前不开放输入；无 reattach API 时禁用自动重连。宿主 provider、生产 provisioner、PTY sink/scrollback、OS sandbox、Audit 与 Session status/cancel/reattach 未接 |
| Phase 5 Group Chat API | 🟡 fail-closed 授权入口与加密持久化接收切片 | `/chat/targets`、`/chat/messages` 与 `PgScopedChatWorkflow` 已形成目标过滤、逐目标 GroupContext 预检及 Transcript metadata/受保护 payload/Run/idempotency/outbox/Audit 原子接收；migration 只存 ciphertext + wrapped key，adapter 必须注入 protector；API main 尚未安装 adapter，真实 KMS protector、密钥轮换/销毁、Agent Policy 到期清理与授权导出/删除仍未实现；仍缺 outbox consumer/L0/LangGraph 和 DB/RLS 实际验收，禁止生产部署 |
| Workflow contract | 🟡 仅接口 seam | 注入实现仍需持久化 Transcript/Run intent、校验 actor+key fingerprint、解析 EntityRef、构建 L0 state，并在 checkpoint resume / 每次 capability call 重新授权；目前没有 LangGraph runtime/checkpointer、Transcript store、resume/interrupt 或 streaming UI |
| Verification | 🟢 静态编译通过 | `cargo check -p star-api-rest --all-targets -j 4` 通过；`pnpm typecheck` 通过；Rust `rustfmt --check` 与 `git diff --check` 通过。未运行测试，未验证服务端 provider、PostgreSQL 写入、LangGraph 运行态或浏览器 session |
| Phase 4 / Phase 5 / Phase 6 / Phase 7 | 🟡 未完成 | 4B 运行闭环仍缺 sandbox/provisioner/sink/audit/reattach；5 缺真实 workflow、Transcript、LangGraph/checkpoint、resume 授权与发送 UI；6 缺 Plugin Registry/grant/runtime/hot revoke；7 的跨 App、ACL/RLS、迁移与故障验收尚未开始 |

本轮将 requirements/basic/detailed design 同步至 v5.0/v3.3/v3.8。API 授权 seam 不代表 LangGraph 或聊天运行时已接入；Group 页面底栏仍禁用发送，直到服务端 workflow、Transcript 与目标选择 UI 可用。未运行测试，不修改或推送既有工作树中的其它改动。

### 6.28 本轮阶段结果（Phase 5 GLOBAL Chat 目标目录与选择 UI，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| 目标目录 API | 🟡 授权目录切片 | 新增 `GET /api/v1/worktrees/{worktree_id}/chat/targets`；Bearer actor 必须有 `chat:submit`，在同一 Tenant transaction 锁定并授权路径 Worktree 与 membership 后读取目录；只列当前有效 Project membership 下的非归档 Worktree，最小返回 ID/名称，无 checkout 路径，稳定 UUID keyset，limit 1–100；目录每页重查 membership，POST 仍重新授权目标。自审修正省略 `correlation_id` 时的随机默认值，现回退到 `Idempotency-Key`，同一请求重放 fingerprint 稳定 |
| GLOBAL 目标选择 UI | 🟡 条件式 UI 切片 | Group 页在 GLOBAL scope 使用 live Group API 多选 Worktree，上限 20，按 cursor 加载更多；宿主 provider 缺失显示不可用提示，不退化成 seed 目标；消息输入与发送仍禁用 |
| 文档同步 | 🟢 完成 | requirements v5.1 CHAT-004/AC-CHAT-004；basic design v3.4 §16.7；Group detailed design v3.9 §8.2；本实施计划 phase row、WG-ACC-15 与本节状态一致 |
| Verification | 🟢 静态编译通过 | `cargo check -p star-api-rest --all-targets -j 4`、`pnpm typecheck`、Rust `rustfmt --check`、`git diff --check` 通过。未运行测试，未连接宿主登录 provider，未验证数据库和撤权并发运行态 |
| Phase 5 overall | 🟡 未完成 | 仍缺持久 Transcript/幂等存储、具体 L0 workflow、LangGraph runtime/checkpointer/resume/interrupt、resume/tool-call 再授权执行器、流式 UI 与真实 provider；Phase 4/6/7 及 DB/ACL/RLS 生产门仍未关闭 |

本轮将 requirements/basic/detailed design 同步至 v5.1/v3.4/v3.9。GLOBAL 目录是可分页选择数据而非授权凭证；任何撤权必须由消息提交和后续执行阶段的实时授权检查捕获。

### 6.29 本轮阶段结果（Phase 4B Task CLI Session lifecycle API，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Session status / cancel / reattach API | 🟡 fail-closed route seam | 新增 session GET / DELETE 与 `POST .../{session_id}/attachment-tickets`；分别要求 `agent_session:read` / `agent_session:cancel` / `agent_session:attach` 和非空 `X-Correlation-ID`。每次请求在 tenant transaction 内重验 Project writer membership 与当前 canonical Task/Worktree association，再向 Runtime provisioner 传递完整 tenant/project/repository/worktree/task/actor/runtime/session/correlation context。status 只返回有限 state、exit code、updated_at 且 no-store；cancel 委托幂等操作；reattach 只返回同一 session 的新鲜 ≤60 秒 ticket，归档 Worktree 拒绝 reattach，错绑/不存在统一 404 |
| Runtime adapter contract | 🟡 仅 trait seam | `TaskCliSessionProvisioner` 增加 status/cancel/reattach 必需方法；实现必须再次核对完整 session binding、当前 ACL / Runtime health 和 session 可用状态。仓内无真实实现；默认未注入返回 503，route 不会直接触碰 PTY 或自行签 grant |
| 文档同步 | 🟢 完成 | requirements v5.2 新增 AC-TCI-007；basic design v3.5 与 detailed design v4.0 定义 scope、no-store、脱敏、幂等与新票据规则；本计划 v4.6 同步 Phase 4 行和 release blocker |
| Verification | 🟢 静态编译通过 | `cargo check -p star-api-rest --all-targets -j 4` 通过；两份本轮 Rust 源以 Rust 2024 `rustfmt` 格式化。全仓 `cargo fmt --check` 被非本轮修改文件的格式差异阻断；未运行测试、未验证目标 DB、JWT scope issuer、真实 Runtime provisioner 或 browser reconnect |
| Phase 4 / Phase 5 / Phase 6 / Phase 7 | 🟡 未完成 | Phase 4 仍缺真实 provisioner、grant signer/verifier/nonce spawn wiring、实时 ACL/health、OS sandbox、PTY sink/scrollback、TaskRun Audit/redaction 与 UI reattach 调用；Phase 5 缺 workflow/Transcript/LangGraph/checkpoint/resume/streaming；Phase 6 缺 Registry/grant/runtime/hot revoke；Phase 7 跨 App 与 DB/ACL/RLS/故障验收未关闭 |

本轮把 session 状态读取、取消和恢复连接提升为可授权的 REST 生命周期边界，但没有把 trait seam 标为真实运行能力。旧 ticket 绝不重复交付；恢复连接必须由 provider 对同一 Task/Worktree/session 重新授权并签发新 ticket。

### 6.30 本轮阶段结果（Phase 4B3 Task Card Session 生命周期 UI 接线，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Status / cancel / reattach API client | 🟢 条件式调用切片 | `WorktreeGroupApiClient` 对 status GET、cancel DELETE、reattach POST 使用当前 Bearer token、Worktree/Task/session path scope、独立 `X-Correlation-ID` 和 `cache: no-store`；断线时不复用旧票据 |
| Task Card lifecycle controls | 🟢 条件式 UI 切片 | 可刷新脱敏 session 状态、显式取消；WebSocket 连接失败或断开后显示手动重连；重连先读状态，仅 `running` / `disconnected` 允许申请新票据，并核验 Session/Worktree/Task/Runtime/correlation 绑定与票据 ≤60 秒有效期后重建终端 |
| Session persistence boundary | 🟡 跨页面恢复未实现 | Session ID 与 ticket 仅在当前页面内存；页面刷新后没有可授权的 Session listing/recovery API，UI 明示该限制；ticket 不写入 URL、localStorage 或 sessionStorage |
| Verification | 🟢 focused checks passed | `pnpm typecheck` 通过；`pnpm exec vitest run src/lib/group/__tests__/worktreeGroupApi.test.ts` 15/15 通过；完整 UI 交互和真实 REST provider 尚未运行验证 |
| Phase 4 / Phase 5 / Phase 6 / Phase 7 | 🟡 未完成 | Phase 4 仍缺真实 provisioner、grant signer/verifier/nonce spawn wiring、实时 ACL/health、OS sandbox、PTY sink/scrollback、TaskRun Audit、Session listing/recovery 与 provider/DB/RLS 验收；Phase 5 缺具体 workflow、Transcript、LangGraph/checkpoint/resume/streaming；Phase 6 缺 Registry/grant/runtime/hot revoke；Phase 7 跨 App 与 DB/ACL/RLS/故障验收未关闭 |

本轮将 requirements/basic/detailed design 同步至 v5.3/v3.6/v4.1。界面恢复逻辑只覆盖当前页面仍持有的 session identity，不能替代跨刷新 Session discovery；所有控制请求继续 fail closed，真实 provisioner 未装配时会返回 503。

### 6.31 本轮阶段结果（Phase 4B4 Task CLI Session listing / reload recovery，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Session listing REST | 🟡 fail-closed route seam | 为当前 Worktree/Task 增加 `GET .../cli-sessions?limit=N`；要求 Bearer actor、`agent_session:read`、Project writer membership、canonical Task/Worktree link、Runtime binding 与 `X-Correlation-ID`。limit 默认 20、范围 1–50；provisioner 返回值拒绝超量、重复或 nil session ID；响应 no-store，仅含状态/exit code/更新时间，不含 ticket/output。仓内无 provisioner 实现，未注入时返回 503 |
| Task Card reload discovery | 🟢 条件式 UI/API 切片 | 打开当前 Worktree 的 live Task Card CLI 面板时请求最近 20 项；重载后可识别 Session 并显式查询状态、幂等取消或恢复连接。恢复先查状态，仅 running/disconnected 可继续；每次重新请求新票据并验证 session/Worktree/Task/Runtime/correlation 与 ≤60 秒 expiry。Ticket 仍仅在页面内存，列表只保留脱敏 metadata |
| Verification | 🟢 focused checks passed | `cargo check -p star-api-rest --all-targets -j 4` 通过；`cargo test -p star-api-rest --lib session_list -- -q` 2/2 通过；`pnpm typecheck` 通过；Group API 与终端 focused Vitest 36/36 通过；`git diff --check` 通过。Cargo 有仓库既有 deprecation/unused warnings |
| Phase 4 / Phase 5 / Phase 6 / Phase 7 | 🟡 未完成 | Phase 4 仍缺生产 provisioner、grant signer/runtime verifier/nonce spawn wiring、实时 ACL/health、OS sandbox、PTY sink/scrollback、TaskRun Audit、宿主认证 provider、目标 DB/ACL/RLS 部署与运行验收；Phase 5 缺具体 workflow、Transcript、LangGraph/checkpoint/resume/stream UI；Phase 6 缺 Registry/grant/runtime/hot revoke；Phase 7 跨 App 故障与权限验收未完成 |

本轮将 requirements/basic/detailed design 同步至 v5.4/v3.7/v4.2。页面刷新后的恢复只发现授权列表中仍可见的 session metadata；真正的 Runtime 恢复能力仍依赖生产 provisioner 和实时授权，缺少时接口保持 503。

### 6.32 前阶段结果（Phase 5 LangGraph checkpoint / resume / replay compatibility boundary；当前实现状态由 §6.33 更新，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| 官方 API 语义核对 | 🟢 设计门关闭 | 官方 Python LangGraph docs 核实：`thread_id` 是 checkpointer 的主关联键；interrupt resume 使用 `Command(resume=...)`；checkpoint replay 会重新执行恢复边界后的节点。PostgresSaver 首次 `.setup()` 会建立 checkpoint 表，须由受控 migration/setup 工作执行 |
| 安全与数据所有权 | 🟢 设计约束已冻结 | thread ID 由服务端生成并映射 Chat Session，跟 WorkItem/Task Card/Agent Session 分离；checkpoint 仅保存非敏感 graph state，不保存授权快照/JWT/长期 Secret/plugin grant；每次 start/resume/interrupt/replay/tool call 重做当前授权；有副作用的节点只能调用稳定幂等、可审计的 Domain Command/outbox |
| 当前实现 | 🟡 仅设计完成 | Rust 侧仍只有 `ScopedChatWorkflow` fail-closed trait seam；无 Transcript/run repository、LangGraph runtime/checkpointer、resume/interrupt endpoint、service identity、目标级事件流或 composer 发送 UI；package pin、checkpoint schema isolation/retention 与部署方式尚未冻结 |
| 文档与验证 | 🟢 已同步 | requirements v5.5 / basic design v3.8 / detailed design v4.3 同步 LGS-003；这是官方文档兼容性核对，不是本地 LangGraph 服务的安装、运行或集成测试 |
| Phase 5 / 全部 Phase | 🟡 未完成 | Phase 5 仍需 outbox consumer/L0 dispatch、LangGraph 服务桥接、逐次权限 broker、Postgres checkpoint 隔离/TTL、stream UI 与角色/撤权验收；Phase 2/3/4/6/7 的生产 provider、DB、Runtime、安全隔离、插件 runtime 和跨 App 验收也未关闭 |

LangGraph 官方文档证实 checkpoint 只恢复状态边界，不能保护外部副作用免于重复执行；因此本轮只锁定可实施契约，不将 API 研究表述为运行时实现。此后开发必须先有生产 service identity 与数据库权限模型，不能把 `permission_snapshot_ref` 或 checkpoint 中的 scope 字段当成长期 grant。

### 6.33 前阶段结果（Phase 5 Transcript/Run 持久化与幂等 dispatch intent；当前隐私安全状态由 §6.34 更新，2026-09-29）

| Phase | 结果 | 证据/限制 |
|---|---|---|
| PostgreSQL adapter | 🟢 代码切片完成 | 新增 `PgScopedChatWorkflow`，先锁定 actor/tenant/key 幂等记录；同 fingerprint 返回保存的 queued receipt，不同 fingerprint 冲突；首次请求单事务写 Session + server-generated LangGraph thread ID、user message、queued Run、outbox event、Audit 和 30 天幂等 receipt；事务失败不返回 202 |
| W/T 数据治理 | 🟢 schema 切片完成，运行验收待做 | `group_chat_session`、`group_chat_message`、`group_chat_dispatch_event`、`group_chat_audit_event` 分类 T 并 append-only；`group_chat_run`、`group_chat_idempotency` 分类 W 并显式 30 天 `retention_period` / `expires_at`；六张表均 FORCE RLS；目前无 cleanup scheduler、目标 DB migration 尚未应用，真实 RLS/并发/append-only 验收未跑 |
| L0 / LangGraph execution | 🟡 未完成 | 没有 outbox consumer、Run state worker、LangGraph/checkpointer/resume/interrupt、逐次权限 broker、stream UI 或 production service identity；`main.rs` 未安装 adapter，API 默认仍 503，composer 保持禁用 |
| 验证 | 🟢 隔离 PostgreSQL 验收通过 | Run、Profile、guard 三 migration 应用成功；guard migration 重复应用；旧 Run 与两类完整 snapshot 接受，5 类负例拒绝；accepted Runs=3（legacy=1、Profile snapshot=2），Profile FK=0；临时数据库清理并复查不存在。目标数据库未触碰。 |
| Phase 5 / 全部 Phase | 🟡 未完成 | §6.34 已将 schema 改为 T 元数据 + W ciphertext/wrapped-key 且 adapter 强制 protector 注入；真实 protector、密钥轮换/销毁、期限清理与导出/删除仍未实现；还缺 outbox/L0/LangGraph/checkpoint/stream、Phase 2/3/4/6 生产服务及 Phase 7 跨 App/ACL/RLS/故障验收 |

本切片不会自动发送模型请求：outbox event 只在业务事务提交后存在，生产入口未安装 adapter，且没有消费 worker；配置缺失时现有接口保持 fail closed。当前隐私边界和可继续的实现切片见 §6.34。

### 6.34 本轮阶段结果（Phase 5 Transcript 正文加密边界与保留策略，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Transcript schema | 🟡 已从明文改为密文切片 | `group_chat_message` 是不含正文的 T 元数据；新增 `group_chat_message_payload` W 表，只存 ciphertext、wrapped data key、key ID 与策略期限；独立期限索引允许清理正文载荷而保留审计元数据 |
| API persistence seam | 🟡 protector 必须注入 | `PgScopedChatWorkflow::new` 必须收到 `TranscriptBodyProtector`；每个新消息先经保护接口，空密文/空包装密钥/无效 key ID 或非正 retention 均 fail closed；当前仓库没有真实 KMS/envelope provider，且 main.rs 未安装 adapter |
| 数据治理 | 🟡 规则已同步，执行器未完成 | 遵循既有 Agent Policy：默认最多 90 天，Project Policy 可调整；GLOBAL 多目标由 protector 采用目标策略最短保留期；密钥轮换/销毁、到期清理 worker、授权导出/删除、日志脱敏与真实 RLS 验收仍缺，禁止生产部署 |
| 文档同步 | 🟢 完成 | requirements v5.8 CHAT-006/AC-CHAT-006；basic design v4.1；Group detailed design v4.6；实施计划 v5.2；明确 T 审计不等于聊天正文永久留存 |
| Verification | 🟢 静态编译通过 | Rust 2024 `rustfmt` 与 `cargo check -p star-api-rest --all-targets -j 4` 通过；只见既有 deprecated/unused warnings；未连接数据库、不应用迁移、不运行测试 |
| Phase 5 / 全部 Phase | 🟡 未完成 | 持久化 adapter 仍未接入 production main；缺真实 protector/key lifecycle、payload/run/idempotency cleanup、outbox/L0/LangGraph/checkpoint/resume/stream、目标 DB/RLS/并发验收；Phase 4/6 真实运行能力及 Phase 7 跨 App 故障/ACL 验收也未关闭 |

本轮关闭的是“明文 Transcript 可被接入”的代码/schema 风险边界，不代表聊天正文加密服务、保留清理或生产运行能力已经交付。下一步先实现并验证可信 protector 与期限清理，再接 outbox/L0；不得在 protector 缺失时安装 persistence adapter。
### 6.35 本轮阶段结果（Phase 6 Group App Registry 授权导航 projection API seam，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Registry read API | 🟡 fail-closed seam | 新增 `GET /api/v1/worktrees/{worktree_id}/group-apps`；要求当前 Bearer 与 `worktree:read`，先验证当前 GroupContext，再调用注入的 `GroupAppRegistryProvider`。provider contract 要求其读取事务重验 Project membership/version、Worktree binding、plugin compatibility 与 actor/Worktree grant |
| Projection validation | 🟢 API 输出边界已实现 | 仅输出 plugin ID、manifest version、label、sort order；最多 100 条、ID 唯一/受限并稳定排序；拒绝空/控制字符标签和不合法 provider projection；未配置 provider 返回 503，main 未安装 provider，不影响当前 UI preview-only 标记 |
| 文档同步 | 🟢 完成 | requirements v5.9 PLG-004/AC-PLG-002；basic design v4.2；Group detailed design v4.7；WG-ACC-17；明确没有持久 Registry、grant lifecycle、runtime、disable/uninstall/hot revoke 或前端 live consumer |
| Verification | 🟢 静态编译通过 | Rust 2024 `rustfmt` 与 `cargo check -p star-api-rest --all-targets -j 4` 通过；只有既有 deprecated/unused warnings；未运行测试，不连接或修改数据库 |
| Phase 6 / 全部 Phase | 🟡 未完成 | 目前只有授权 read projection seam；缺 Registry schema/provider、manifest signature/compatibility validation、持久 binding/grant、enable/disable/uninstall commands、capability gateway、runtime isolation、in-flight drain/cancel、热撤权事件、frontend live projection 与运行验收；Phase 2/3/4/5 production dependencies 和 Phase 7 cross-app 验收仍开放 |

该接口只负责读取服务端授权导航投影，不注册或启用插件，也不为任何 capability 授权；只有可信 Registry provider 接入并通过当前权限复验后，才能让生产 UI 消费其响应。

### 6.36 本轮阶段结果（Phase 6 PostgreSQL Registry projection 与 SCD2 schema，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Registry schema | 🟡 migration 已落档 | `db/migrations/2026-09-29-worktree-group-phase-6-plugin-registry.sql` 建立 manifest、Worktree Registry revision、binding、actor `group_app:open` grant 和 Audit 五表；四张 Master 使用 SCD2 有效区间/current-row 唯一约束与只关闭/no-delete trigger，Audit 禁止 UPDATE/DELETE；五表启用并 FORCE RLS。迁移尚未在 PostgreSQL 执行 |
| PostgreSQL provider | 🟡 实现并由 production main 装配 | `PgGroupAppRegistryProvider` 在 tenant/actor-scoped 事务内锁定 membership 与 Worktree binding，比较前序 GroupContext membership version，拒绝已归档 Worktree，再投影当前 verified、Host API version 1 兼容、active 且 actor 有有效 open grant 的最多 100 个 App。Provider 只读，不写 Audit 或授予调用权限 |
| Runtime / trust / UI | 🔴 未完成 | 没有受信任 manifest signature/ingest service、lifecycle writer/admin API、registry version 原子推进命令、tool/data capability gateway、沙箱 runtime、in-flight drain/cancel、撤权事件 consumer 或 Group 页面 live projection；`verification_status='verified'` 不能单独作为信任证明 |
| 文档同步 | 🟢 完成 | requirements v5.10 PLG-005/AC-PLG-003；basic design v4.3；Group detailed design v4.8；WG-ACC-18；按 W/T/M 拆分所有 Registry 表并明确数据库未部署 |
| Verification | 🟡 本轮检查中 | 后续记录 rustfmt、`cargo check -p star-api-rest --all-targets -j 4` 与 `git diff --check` 结果；不连接或修改数据库、不运行测试 |
| Phase 6 / 全部 Phase | 🟡 未完成 | 只读授权 projection 与 schema 已有代码，Phase 6 仍缺 ingest/lifecycle writes、capability enforcement/runtime/hot revoke/UI live consumer 和数据库运行验收；Phase 2/3/4/5 生产依赖及 Phase 7 跨 App 验收仍开放 |

`PgGroupAppRegistryProvider` 已接进 production route state，但数据库 migration 未部署时读取错误映射为非成功响应。migration 通过静态检查或 Rust 编译均不代表 PostgreSQL DDL/RLS 行为已验证；`verified` 只可由后续可信 ingest 写入，不能直接信任普通应用写入。

### 6.37 本轮阶段结果（Phase 6 Group App Registry live UI consumer，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Group UI consumer | 🟢 条件式 live 消费切片 | 有 `WorktreeGroupApiClient` 时请求当前 Worktree 的 `/group-apps`；验证 worktree ID、非负 Registry version、最多 100 条、plugin ID 唯一/受限、label/version 长度和控制字符，并稳定排序。授权列表成为插件同级导航唯一来源；加载、错误、Worktree 或 provider generation 切换时清除旧入口，不回落预览；支持显式重试、30 秒刷新与 tab 可见性刷新。无 API provider 的原型环境保留标记的本地预览；live 入口只代表 navigation authorization projection，不启动插件执行。
| TypeScript | 🟢 通过 | 在 `frontend/` 执行 `pnpm typecheck`，`tsc --noEmit` 通过；未运行测试。 |
| Rust/API | 🟢 通过 | `rustfmt --edition 2024 --check` 覆盖 Registry provider/API/main 装配文件；`cargo check -p star-api-rest --all-targets -j 4` 通过，存在仓库既有 deprecation/unused warnings；未运行测试。 |
| Diff / database | 🟡 受限 | `git diff --check` 通过。Plugin migration 未在 PostgreSQL 执行，真实 RLS、migration 部署和 API 运行验收未发生。 |
| 文档同步 | 🟢 完成 | requirements v5.11、basic design v4.4、Group detailed design v4.9；新增 WG-ACC-19。 |
| Phase 6 / Phase 7 | 🟡 未关闭 | Phase 6 仍缺目标 DB/RLS、受信任 manifest ingest、Registry lifecycle writer、capability gateway/runtime、撤权/drain/cancel 和真实宿主认证 provider；Phase 7 跨 App 生产验收尚未开始，不能由静态类型与编译检查替代。 |

当前应用没有宿主会话 provider，故运行时仍会落在明确标记的本地预览；生产 main 虽已装配 PostgreSQL read provider，但 migration 和 trust/lifecycle/runtime 尚未部署。
### 6.38 本轮阶段结果（Phase 6 Registry 投影校验与聚焦测试，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Projection validator | 🟢 完成 | 抽取 `projectGroupAppsForNavigation` 纯函数；仅接受匹配 Worktree、UUID correlation、非负安全整数 Registry version、最多 100 条唯一受限 ID、有效长度且无控制字符的 manifest version/label 与 int32 sort order；按 sort order/plugin ID 稳定排序。无效输入返回 null，由调用端维持 fail closed。 |
| 聚焦测试 | 🟢 通过 | `pnpm exec vitest run src/lib/group/__tests__/groupAppsRegistry.test.ts`：1 file / 3 tests passed，覆盖有效排序、跨 Worktree 拒绝、重复 ID、控制字符、坏 UUID/Registry version/sort order 与 100 条上限。 |
| TypeScript / diff | 🟢 通过 | `pnpm typecheck` (`tsc --noEmit`) 与 `git diff --check` 通过。 |
| 文档同步 | 🟢 完成 | requirements v5.12、basic design v4.5、Group detailed design v4.10；扩充 WG-ACC-19。 |
| Phase 6 / Phase 7 | 🟡 未关闭 | 目标 DB/RLS、宿主 GroupAccessTokenProvider、可信 manifest trust root/ingest、lifecycle writer、capability gateway/runtime 和 hot revoke/drain/cancel 未部署；Phase 7 的 cross-app/ACL/failure production验收尚未开始。 |

聚焦测试只证明客户端边界校验，不证明 PostgreSQL migration/RLS、真实身份授权、插件执行或热撤权。

### 6.39 本轮阶段结果（Phase 7 本地跨模块回归基线，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Group 与 terminal 前端回归 | 🟢 通过 | 修正 Group Canvas fixture，使 Element projection 含 API 契约要求的 `version`；`pnpm exec vitest run src/lib/group/__tests__ src/components/terminal/TerminalSplitPane.test.tsx src/lib/terminal/wsClient.test.ts`：5 files / 44 tests passed。 |
| REST / Plugin / Canvas API 单测 | 🟢 通过 | `cargo test -p star-api-rest --lib -j 4`：75/75 passed，覆盖 Canvas cursor、CLI Session list 边界及现有 API 契约。首次 Windows link 阶段遇到 LNK1104，单独重试后通过。 |
| Local Runtime 与 terminal-stack 单测 | 🟢 通过 | `cargo test -p domain-local-runtime -p terminal-stack --lib -j 4`：domain-local-runtime 168/168、terminal-stack 82/82 passed，包含单次 ticket 与首帧授权解析边界。 |
| 静态检查 | 🟢 通过 | `pnpm typecheck` 与 `git diff --check` 通过；PowerShell 环境未安装 `rtk`，此次使用原生命令运行检查。 |
| Phase 7 生产验收 | 🟡 未开始 | 这些测试是本机单元/前端回归，不是跨服务 E2E；未连接真实宿主身份、目标 PostgreSQL、RLS 角色或生产 Local Runtime。Phase 2–6 的授权、迁移、事件消费、LangGraph、Plugin runtime blockers 仍阻止生产验收。 |

此阶段建立了可重复的本地回归基线，并修复了与严格 Canvas Element version 校验不一致的测试 fixture。它不能证明跨 App 生产行为；Phase 7 仍需等 Phase 2–6 的运行依赖就绪后执行完整矩阵。

### 6.40 本轮阶段结果（Phase 5/6 隔离 PostgreSQL migration、RLS 与 trigger 验证，2026-09-29）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Phase 5 Chat migration | 🟢 隔离库验证通过 | 在 `star_worktree_phase_validation_20260929` 执行 `2026-09-29-worktree-group-phase-5-chat-store.sql`，并再次重复执行通过；catalog 确认 7 张 `group_chat_*` 表均启用并 FORCE RLS。 |
| Phase 6 Plugin migration | 🟢 隔离库验证通过 | 同一隔离库执行 `2026-09-29-worktree-group-phase-6-plugin-registry.sql` 并重复执行通过；catalog 确认 5 张 `group_app_*` 表均启用并 FORCE RLS。 |
| RLS 与不可变性 | 🟢 隔离事务验证通过 | `SET ROLE star_app` 并仅在测试事务内临时授予 schema/table 权限；Chat tenant+actor 可见性、Plugin tenant scope 与无写 policy 拒绝，以及 Chat/Plugin Master/Audit 的 UPDATE/DELETE trigger 均符合预期；测试事务 rollback，fixtures 和临时 grants 未保留。 |
| Runtime SQL grants | 🔴 未配置 | RLS policy 不赋予 schema `USAGE` 或表权限。隔离库 `star_app` 无 `multica` / `plugin` schema `USAGE`，`star_app_role` 未建立；目标环境的实际应用身份和角色配置尚未确认。不得把本地临时 grants 当成部署权限。 |
| Phase 5/6 与 Phase 7 | 🟡 未关闭 | 以上只验证迁移可重放及策略/触发器行为，不代表目标 DB 已部署、生产 service role 可连接、真实 endpoint 可以读写或 Phase 7 E2E 通过。依赖显式 role bootstrap/grants、真实身份、其余 Phase runtime 与跨 App 故障/权限矩阵。 |

此验证不修改 `star_dev` 或生产目标库。下一道数据库门是先从宿主部署配置确认 runtime identities，再形成最小逐表授权清单并在目标环境以实际应用身份复验；身份未确认前不把 GRANT 写给猜测角色。

### 6.41 本轮阶段结果（开发 Group Guard 守门 #33，2026-09-30）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Cargo source ownership | 🟢 规则收敛 | Canvas = `crates/canvas-*` + `crates/domain-canvas`；Domain = `crates/domain-*`（排除 `domain-canvas`）+ `crates/star-*`。共享 API / application / infrastructure 与 adapters 不由此规则隐式归组。 |
| 守门 #33 实装 | 🟢 实现完成 | `group_guard.py` 从完整 Cargo metadata 读取 workspace resolved dependency graph，只检查 Canvas/Domain 两组包间直接边；cargo metadata 非零、超时、JSON/graph 缺失均 fail closed；frontend/core 显示 N/A。 |
| 聚焦验证 | 🟢 通过 | `python -m pytest scripts/automation/__tests__/test_group_guard.py -q`：20/20 通过；覆盖零跨组边、Canvas→Domain 与 Domain→Canvas 违规、`domain-canvas` ownership、共享包过滤、metadata / workspace node / dependency target 缺失及 #34 Group ID 错配非零退出。真实 `cargo metadata` 图对 Canvas 与 Domain 源包检查结果为 0 条跨组边。 |
| 静态检查 | 🟢 通过 | `python -m py_compile scripts/automation/group_guard.py scripts/automation/__tests__/test_group_guard.py` 与 `git diff --check` 通过。 |
| 产品 GroupContext / 全部 Phase | 🟡 未关闭 | 开发守门 #33 不代表宿主认证、产品 GroupContext、目标数据库/RLS、CLI runtime、LangGraph/Chat、Plugin runtime 或 Phase 7 生产验收完成。 |

### 6.42 本轮阶段结果（Phase 2D Worktree Git retention-lock observation slice，2026-09-30）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Observer contract | 🟢 API seam 完成 | `WorktreeGitLockObserver` 接收已授权 query，绑定 tenant/project/repository/worktree/runtime；`locked / unlocked / unknown` 与持久化 `locked` 分开。该 Git Worktree retention lock 保护 Git 管理记录，不表示 Agent 活跃或编辑互斥。缺 provider/runtime、调用失败、单项超时（2 秒）、无时间戳、超过 30 秒或未来时间戳均归一为 unknown。 |
| Index bounded fan-out | 🟢 代码完成 | 每页最多 8 路并发、总观测预算 3 秒；预算耗尽后未完成项目保留 unknown，避免宿主 Runtime 停顿拖住整页。 |
| Worktree Index UI | 🟢 条件式消费完成 | Project Index 与详情分别显示 Git observation 状态/时间和持久化 `locked` 标记；只有 `source=host_runtime` 且时间戳有效、未超前并在 30 秒内的新鲜观测才显示 locked/unlocked；缺 source、无效/过期/未来时间戳或旧投影按 unknown 显示。 |
| 聚焦验证 | 🟢 本地通过 | 隔离 Cargo target 下 `cargo test -p star-api-rest --lib -j 4`：81/81 通过；`pnpm typecheck` 通过；Group/terminal Vitest 46/46（新增来源、无效/缺失/过期/未来时间戳 fail-closed 用例）；改动 Rust `rustfmt --edition 2024 --check` 与 `git diff --check` 通过。 |
| Phase 2D / Phase 7 生产门 | 🟡 未关闭 | `main.rs` 尚未安装真实 Host Runtime observer；Worktree create/import、独立 Agent 活跃状态与 session drain、lock 下的 plan-confirm physical cleanup、目标 DB/RLS 与 ACL/Runtime 集成、跨 App 生产验收未完成。观察接口自身不能执行或授权删除；fresh unlocked 也不能替代 Agent drain。 |

### 本轮修订（2026-10-01）

### 6.43 本轮阶段结果（Phase 2D Project Repository discovery + Index create/import UI，2026-09-30）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| API contract | 🟢 条件式 API seam 完成 | 新增 Project Repository discovery、import candidate、create、import 四条路由；Repository 读取要求 project:read + membership，仅返回 ID/name/default branch；写操作校验 Bearer actor、worktree:create scope、当前 Project membership 与 writer role；候选上限 50；Idempotency-Key、request fingerprint 与 correlation_id 进入可信 provider 命令；响应仅返回 202 receipt，不含主机路径。 |
| 输入与 UI 边界 | 🟢 条件式消费完成 | create/import/query DTO 拒绝未知字段；客户端不能提交 checkout path、repository URL 或 Git command；branch/base_ref 做 ref 校验，导入只接 opaque candidate_id。Index 控件对 Repository、candidate、receipt 使用精确字段 allowlist 并校验关联，受理后刷新；role/认证/API/额外字段错误均关闭写操作、不回退 seed。 |
| Provider / durable state | 🟡 未完成 | GroupApiState 可注入 ProjectWorktreeLifecycleProvider；未注入时所有接口返回 503。生产 adapter 仍需从权威 Project-Repository binding 解析仓库和路径，并在副作用前重新授权；在 202 前持久化 operation、Worktree membership、Audit 与 Outbox。 |
| 生产 Index / Phase 2D / Phase 7 | 🟡 未完成 | UI 控件已经接线，但当前生产 main 未装配 lifecycle/Host Runtime provider，Project Repository SoR 与宿主认证未接通，因此创建/导入不会在生产成功。独立 Agent/session 活跃状态、drain、物理 cleanup、目标 DB/ACL/RLS 验收与 Phase 7 跨 App 验收仍未完成。 |
| 聚焦验证 | 🟢 本地通过 | `cargo check -p star-api-rest --all-targets -j 4` 通过；`cargo test -p star-api-rest --lib worktree_lifecycle::tests -j 4` 8/8；`pnpm typecheck` 通过；Worktree Lifecycle UI + Group API Vitest 22/22；无关 crate 有既存 deprecation/unused warnings。 |

### 6.44 本轮阶段结果（Phase 2D 授权 Project 目录与生产选择器，2026-09-30）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| 授权目录 API | 🟢 条件式读接口完成 | 新增 `GET /api/v1/projects?limit=&cursor=`；校验 actor 与 `project:read`，设置事务级 tenant scope，从有效 `permission.project_role_binding` 读取当前用户 Project ID/role，UUID keyset cursor，最多 200 条，`Cache-Control: no-store`。没有 Project master/name 表，因此不伪造名称。 |
| 生产 Project Selector | 🟢 条件式 UI 完成 | Group API 存在时只消费服务端授权目录；校验精确字段、UUID、role、重复 ID 与游标，提供续页/重试。深链在目录加载完之前不会被误判无权；接口错误不回退本地 seed。API session generation 切换时 Project directory、Worktree Index 与 member role 投影立即隐藏旧 session 内容。无宿主 API 的原型继续以明确标签展示本地预览。 |
| 剩余 Phase 2D / 生产启用 | 🟡 未完成 | 需接入宿主认证 provider、目标 DB 与实际 membership provisioning；Project 名称和 Project→Repository 关系的 SoR、生产 lifecycle/Host Runtime provider、durable writer、历史归属 reconciliation、Git/Agent 独立活跃状态与 drain/物理 cleanup 仍缺；必须完成 RLS/ACL 负向集成验收。Phase 3-7 的已知生产 blockers 仍按上方 phase 表保持开放。 |
| 聚焦验证 | 🟢 本地通过 | 前端 selector/API/lifecycle 测试 26/26，`pnpm typecheck` 通过；`cargo test -p star-api-rest --lib -j 4` 90/90，`cargo check -p star-api-rest --all-targets -j 4` 通过；三份改动 Rust 文件的 rustfmt check 与 `git diff --check` 通过。编译输出只含其它 crate/API auth 测试的既存 deprecation/unused warnings。 |

### 6.45 本轮阶段结果（Phase 9B2C Worktree archive-confirm Hook gate，2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| policy/evaluator 接线 | 🟢 条件式代码切片完成 | archive confirm 在最终授权事务内读取并验证 Project baseline/Worktree overlay，生成当前 Rust Hook evaluation；缺 Project baseline、不可用 policy、Runtime unhealthy、执行未 drain、活跃 Run/lease/claim/process、locked/unknown Git retention lock 均由 builtin evaluator fail closed。 |
| Runtime/Git 预检 | 🟢 有界 observer seam 完成 | 数据库事务外先最多等待 2 秒获取 Git lock observation；只有新鲜 Unlocked 才请求最多 2 秒 Runtime drain/readiness，避免 lock conflict 时停止 Agent。返回后重开短事务，再锁定 plan/Worktree、检查 Project/Repository/Runtime/lifecycle version、当前 actor manager 权限、policy 与 observation freshness。readiness observer 收到稳定 operation ID、plan correlation 和期望 lifecycle version；admission fence expiry 必须为最终 archive mutation 留至少 5 秒，并在写入前重检。生产 provider 必须保持 fence 至命令完成，且其期限要和目标数据库有界事务时限一起验收。 |
| 决策和审计 | 🟢 条件式实现 | Allow 才执行 archive row 状态更新；Deny 把计划置 cancelled；RequireHuman/Defer 记录未执行 decision projection 并保留待处理计划。当前投影写入 `worktree_management_audit`，不能替代 Phase 9D 通用 append-only Hook RunEvent/outbox/BI。 |
| 生产启用/cleanup | 🟡 未完成 | `GroupApiState` 有可注入 readiness observer seam，但 production main 尚未安装 provider，缺失时 API 返回 503；目标 DB migration/runtime grants/API RLS integration 未验收，DB archive 不执行物理 checkout 删除，Host Runtime cleanup/provider 仍需另行实现。事务期限与 admission fence 的联合运行保证仍待 production adapter 验收。 |
| 聚焦验证 | 🟢 本地通过 | `cargo check -p star-api-rest --lib -j 4` 通过；`cargo test -p star-api-rest --lib -j 4` 103/103；`cargo clippy -p star-api-rest --lib -j 4` exit 0，REST crate 报告 51 条 warning，但定向扫描确认改动的 `group_api.rs`、`hook_policies.rs`、`worktrees.rs` 无 Clippy warning。workspace 仍有既存弃用/unused/style/large-error warnings；rustfmt check 通过，`git diff --check` 在提交前复核。 |

导航约束不变：Settings 侧栏提供“高级设置”主入口，内部 Hooks 是与 Skills/MCP/Plugins 并列的 tab（`/settings/advanced/hooks`）；Hooks 不进入 Worktree Group App 树。Worktree Index 只呈现有效策略和决策状态。Phase 9C UI 代码切片已加入但 session Provider/生产 API 验收仍开放，9D RunEvent/outbox/BI、9E Agent/Loop 与目标环境验收继续开放。

### 6.46 本轮阶段结果（Phase 9C Advanced Settings Hooks tab，2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| 导航层级 | 🟢 代码切片完成 | 沿用 ULYS-235 的既有决策：Settings 侧栏只有“高级设置”入口；进入后 Skills、Hooks、MCP、Plugins 是同层 tabs。Hooks canonical route 为 `/settings/advanced/hooks`，不是 Worktree Group App，也没有 Hooks 独立顶级项。`star-nav-store:v2`→v3 一次性迁移保留旧侧栏顺序/折叠/项目 scope，并补入高级设置；迁移后仍尊重用户移除该入口的选择。 |
| Policy Builder | 🟢 条件式 UI 代码完成 | Project/Worktree 授权选择、typed restrictive rule editor、Draft CAS、publish role gate、rollback 与策略配置 Audit；缺 API session 时明确 fail closed，不读取 seed、不显示假策略、不允许写入。当前范围仅 Worktree archive/cleanup。 |
| API 对接 | 🟡 客户端契约已加，运行时未接通 | 增加 Project/Worktree policy read/draft/publish/rollback 方法；应用 root Providers 尚未注入 Group API token/session generation，故没有真实授权读取或发布验收。生产目标 DB/RLS/grants 也未安装。 |
| RunEvent/BI | 🟡 9D 首个切片 | 新增 Worktree archive append-only HookEvent ledger、同确认事务写入、`hook:read` Project 授权 keyset API 和 Advanced Settings → Hooks 页执行事件面板；只覆盖 `worktree_archive`，coverage=`partial`、百分比 `null`。9C 右栏仍只显示策略配置 Audit；Run-linked producers、真实 Outbox delivery state、BI read model/授权下钻与 Run Detail/Quality & Improvement consumer 尚未实现。Run 仍是每次尝试独立身份，Worktree 是可选 execution reference，Project BI 留在 Quality & Improvement。 |
| 聚焦验证 | 🟡 有边界通过 | `pnpm exec vitest run` 对 Hooks/导航、偏好迁移与 Group API 的四个定向文件 38/38 通过；`pnpm exec tsc --noEmit` 通过。`pnpm typecheck` 在当前 PowerShell 环境报 `tsc is not recognized`，直接调用本地 TSC 已验证。`pnpm exec next build` 的 production bundle compile 成功，但全站静态生成在未修改的 `/worktree` 页因 `useSearchParams()` 缺少 Suspense boundary 失败；此阻断与本次新增 route 不同。 |

**本阶段结论**：Advanced Settings 导航和 Hooks 可视策略 UI 已形成可测试代码切片，不能标为生产 9C 验收完成。关闭条件为装配 app session Provider、在目标 DB/RLS 下验证策略 read/write/publish/rollback、浏览器端 route/accessibility/E2E，并提供策略审计与 9D execution-event 的明确分界测试。

Hooks 导航基线已经在 `AGENTS.md` 与 ULYS-235 明确规定：主入口“高级设置”，内层并列 tabs 为 Skills、Hooks、MCP、Plugins；不得将 Hooks 添加到 Project→Worktree→Apps 树。

### 6.47 本轮阶段结果（Phase 9D HookEvent 事件源、读取 API 与 Hooks 页面事件面板，2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| 存储 schema | 🟡 migration 已写、隔离验证通过 | `db/migrations/2026-10-01-multica-hook-execution-event.sql` 增加 Transaction/append-only `multica.hook_execution_event`，无 Worktree/Run 外键以保留生命周期历史，设置 tenant FORCE RLS、bounded JSON 与 cursor/metric indexes。一次性 PostgreSQL 18 集群重复应用成功；租户隔离和 UPDATE/DELETE/TRUNCATE 拒绝通过。目标 DB/runtime role grants/RLS integration 未验收。 |
| archive producer | 🟡 条件式代码通过 | Worktree archive evaluation 测量 evaluator latency，将 stable decision/reason、policy/evaluator versions、digest 与 freshness timestamps 写入该 ledger；事件与 Worktree mutation/audit 同事务，写失败 fail closed。仅 Allow 通过最终授权与新鲜度复核后记录；Deny/RequireHuman/Defer 随管理审计提交。`star-api-rest` library tests 106/106 通过。 |
| read API | 🟡 条件式 API 代码通过 | 新增 `GET /api/v1/projects/{project_id}/hook-events`，复用 `hook:read` + membership + tenant RLS，Project-bound `(occurred_at,event_id)` cursor，limit 上限 100，allowlisted projection/no-store。返回 partial coverage 与 null percentage；这只是 ledger 读取源，不提供 Outbox consumer 状态。 |
| Advanced Settings UI | 🟡 受控事件面板代码通过 | 既有 `/settings/advanced/hooks` tab 增加 Project-scoped、每页 30 条且页面最多保留 300 条的 Hook 执行事件列表，配置 Audit 与 execution event 分开；Project/session 切换清除旧数据，coverage=`partial`/比例 unknown 明示。测试使用显式 Provider；app root 尚未装配真实认证 session。 |
| 与 RunEvent/BI 的关系 | 🟡 后续接线开放 | `task_execution_run_event` 保持 task/run FK，仅承载 Run-linked Hook facts；Worktree lifecycle ledger 承载独立 archive Hook。Run/tool/validation/review producers、联合 BI read model、聚合公式、Run Detail/Quality & Improvement drilldown 均未完成；未采集 phase 是 unknown，不能按 0 处理。 |
| 验证与启用 | 🟡 本地切片通过、生产未验收 | Rust focused tests 106/106、前端 Hooks/API 两个定向文件 25/25、`pnpm exec tsc --noEmit` 通过；SQL 在一次性 PostgreSQL 18 集群重复应用并验证 FORCE RLS、租户隔离及 append-only 拒绝。目标 DB、host session Provider、生产 Runtime readiness provider 与物理 checkout cleanup 未验收，不能宣称 Phase 9D 或生产 Hook execution 完成。 |
### 6.48 本轮阶段结果（Phase 9D Project Hook summary API，2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Project-scoped API | 🟡 条件式 Rust API 已实现 | GET /api/v1/projects/{project_id}/hook-events/summary?window_days=；复用 hook:read、Project membership 与 tenant RLS，默认窗口 30 天、边界 1–90 天、no-store。目标数据库、runtime grants/RLS 与 host session Provider 未验收。 |
| 指标 contract | 🟡 source-only metric v1 | hook_execution_summary_v1 按 phase/decision 聚合 event_count、run_linked_event_count、timeout_count、duration_total_ms、average_duration_ms 与 latest time；响应携带公式、窗口与 partial/null coverage。当前 run_linked 只表示 Hook ledger 的 run_id 非空行，不是 task_execution_run_event join。 |
| Coverage | 🟡 明确 partial/unknown | 当前仅 instrument worktree_archive；coverage percentage 为 null，run_outcome_join 为 not_available；未接入 phase 和质量 outcome 均 unknown，不按零或完整覆盖率解释。 |
| 验证 | 🟢 隔离 PostgreSQL 验收通过 | Run、Profile、guard 三 migration 应用成功；guard migration 重复应用；旧 Run 与两类完整 snapshot 接受，5 类负例拒绝；accepted Runs=3（legacy=1、Profile snapshot=2），Profile FK=0；临时数据库清理并复查不存在。目标数据库未触碰。 |
| 后续门 | 🟡 Phase 9D/生产未关闭 | 接入 Run-linked producers、跨两类 event source 的去重/Run outcome join、Outbox delivery state、版本化 denominator/cohort 与 Quality & Improvement/Run Detail UI；在目标 DB/RLS/grants 和真实 auth Provider 下验收后才可关闭。 |

### 6.49 本轮阶段结果（Phase 9D Hook summary UI consumer，2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Advanced Settings consumer | 🟡 条件式前端代码已实现 | 原 `/settings/advanced/hooks` 标签中增加 7/30/90 天窗口、总量指标和 phase/decision 表；Project/session/API client 变化时清除旧投影。与用户此前 ULYS-235 的高级设置并列标签要求一致，不新增 Worktree/主侧栏导航项。 |
| coverage 语义 | 🟡 显式 partial/unknown | 只展示 `hook_execution_summary_v1` 已记录 archive ledger；Run ID 行数不宣称 RunEvent join，Run outcome 标记尚未接入；无记录只表示当前已接入账本在窗口内无行。 |
| 验证与启用 | 🟡 未运行测试 | 本轮未运行 tests；目标宿主 auth Provider、目标 DB/RLS/grants、完整 Run-linked producers、Outbox 和 BI read model/UI 仍未完成。 |
| 后续门 | 🟡 Phase 9D/生产未关闭 | 先接 Run-linked Hook producers 与稳定去重/Run outcome join，再实现 Outbox delivery state、完整 coverage/cohort 与 Quality & Improvement/Run Detail 授权下钻。 |

### 6.50 本轮阶段结果（Phase 9D-4 双来源 Hook summary read model，2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| 双来源读取与去重 | 🟡 summary v2 代码已实现 | 在 1–90 天 Project 窗口内合并 Hook ledger 与字段完整的 `hook_evaluated` TaskRunEvent；RunEvent 镜像以 `(tenant_id,event_id)` 去重，producer 必须共享 event ID；新增 Project/time/event-type 索引支持有界扫描。 |
| Run 状态关联 | 🟡 read-model join 已实现 | 通过 tenant/project/work_item/run 完整键读取每个 Run 最新 execution_state，按 phase/decision 展示状态计数；`no_samples/partial/complete` 只描述 join 样本关系，不代表 Run terminal outcome 或全阶段 Hook coverage。 |
| 不完整数据与 coverage | 🟡 可见缺口 | 缺 phase/decision/duration/timeout 且无 ledger 镜像的 RunEvent 不纳入指标，单独报告 excluded 数；known producer 仍只有 `worktree_archive`，Run producer 未接、coverage percentage=null、其他阶段 unknown。 |
| Advanced Settings 位置 | 🟢 延用既有决策 | v2 汇总仍显示在 `/settings/advanced/hooks` 的 Hooks tab 内容区；它属于「高级设置」父入口下的并列标签，未添加 Worktree 或主侧栏导航节点。 |
| 验证 | 🟢 隔离 PostgreSQL 验收通过 | Run、Profile、guard 三 migration 应用成功；guard migration 重复应用；旧 Run 与两类完整 snapshot 接受，5 类负例拒绝；accepted Runs=3（legacy=1、Profile snapshot=2），Profile FK=0；临时数据库清理并复查不存在。目标数据库未触碰。 |
| 后续门 | 🟡 Phase 9D 未关闭 | 接入 Run admission/tool/validation/review producers 与事务内双写；补齐 Outbox delivery state、版本化 denominator/cohort、Run Detail/Quality & Improvement 下钻；完成目标 DB/RLS/grants 与真实 auth Provider 验收。 |

### 6.51 本轮阶段结果（Phase 9D-5a phase-scoped native evaluator contract，2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Typed phase model | 🟡 evaluator API v2 已支持两类 phase | HookRule 可选 phase 将规则绑定到 BeforeRunAdmission 或 BeforeWorktreeArchiveCleanup；未标注规则只适用于 archive/cleanup，不跨 phase。当前 evaluator 不宣称其他 planned phases 已接入。 |
| 旧策略兼容 | 🟢 archive policy digest 保持兼容 | evaluator API v1 仅接受无 phase 字段的旧 policy 并只支持 archive/cleanup；v2 可用 phase-scoped rules。Run admission 当前仅接受 ActorAuthorized、LifecycleVersionMatches、RuntimeHealthy，archive-only facts 在验证时拒绝。 |
| Builder 与发布边界 | 🟢 保持高级设置内并列标签且服务端 fail closed | HookRule phase selector 位于既有 Settings → Advanced Settings → Hooks 内容区；Run admission 显示为 disabled，Project/Worktree publish 与 rollback normalization 拒绝无 producer 支撑的规则并返回 hook_phase_producer_unavailable。ULYS-235 的 Skills/Hooks/MCP/Plugins 局部标签条不变，也未增加 Worktree 导航节点。 |
| 验证 | 🟢 隔离 PostgreSQL 验收通过 | Run、Profile、guard 三 migration 应用成功；guard migration 重复应用；旧 Run 与两类完整 snapshot 接受，5 类负例拒绝；accepted Runs=3（legacy=1、Profile snapshot=2），Profile FK=0；临时数据库清理并复查不存在。目标数据库未触碰。 |
| 未关闭门 | 🟡 Phase 9D 继续开放 | Run admission producer/readiness provider、Run start transaction dual-write、tool/validation/review producers、Outbox delivery、BI cohort/denominator 与 Quality & Improvement/Run Detail drilldown、目标 DB/RLS/grants 与真实 auth Provider 尚未验收。 |

### 6.52 本轮阶段结果（Phase 9D-5b conditional Run admission producer，2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Runtime readiness 与 fence | 🟡 条件式 seam 已接入 | `TaskCliSessionProvisioner::supports_run_admission()` 默认 false；显式 producer 最多 2 秒在事务外返回 readiness/fence。readiness ≤5 秒、非空 fence、提交余量 >5 秒、TTL ≤30 秒；fence 绑定 Task/Worktree/Runtime/profile/request fingerprint，必须由 adapter 在 spawn 前一次性消费与重验。当前无生产 adapter，能力未开启。 |
| Run + Hook 原子写入 | 🟡 REST producer contract 已实现 | final short transaction 重授权、重读 Worktree/Task/lifecycle/effective policy 并运行 `BeforeRunAdmission` evaluator；Allow 写 immutable HookSet snapshot、Run/start event、Hook ledger 与 Run `hook_evaluated`；Deny 仅写 `run_id/work_item_id=NULL` 的 ledger，不创建 Run。ledger 与 RunEvent 显式共享同一 `event_id`，满足 summary v2 `(tenant_id,event_id)` 去重。Runtime readiness 等待不持 DB row lock。 |
| Policy UI 与 BI coverage | 🟢 使用同一服务端 capability | GET policy 的 `producer_capabilities.run_admission` 控制 Builder phase option；publish/rollback normalization 对 capability=false fail closed；event list 与 summary `instrumented_phases` 随 capability 切换。Hooks 仍为 ULYS-235 Advanced Settings → Hooks 并列 tab，非 Worktree 节点；目前实际 capability=false，所以 UI 禁用且 Run coverage unknown。 |
| 验证 | 🟢 隔离 PostgreSQL 验收通过 | Run、Profile、guard 三 migration 应用成功；guard migration 重复应用；旧 Run 与两类完整 snapshot 接受，5 类负例拒绝；accepted Runs=3（legacy=1、Profile snapshot=2），Profile FK=0；临时数据库清理并复查不存在。目标数据库未触碰。 |
| 未关闭门 | 🟡 Phase 9D 未关闭 | 生产 provisioner/runtime fence consume、真实 auth provider、目标 DB migration/grants/RLS/API integration、独立 Outbox delivery、tool/validation/review producers、完整 BI denominator/cohort 与 Run Detail/Quality & Improvement 下钻仍未验收；本切片不宣称生产 Run spawn 已由 Hook 保护。 |


### 6.53 本轮阶段结果（Phase 9E-1 immutable Agent Execution Profile contract，2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Profile 类型与不可变包装 | 🟢 9E-1/9E-2 domain core 已实现 | 新增 domain-agent execution_profile：schema v1 Draft/Document/Verified wrapper 与 bounded current dependency resolver；固定 tenant/project/worktree scope、provider/skill version/digest、grant/HookSet/Worktree 状态，无 fallback；借用式输出不复制大字段。 |
| provider 与跨能力约束 | 🟢 fail-closed field verification | Agent/Memory/Skill/ContextAssembler/Validation/LoopPolicy provider 的 version/digest/capabilities 固定；列表排序唯一；所有 provider/skill capability 必须为 grant snapshot 子集；Validation provider ID 与 Agent provider 分离；Memory 必须显式 Disabled 或有界 Enabled，Unavailable 拒绝。 |
| 上下文与循环/资源预算 | 🟢 单 Run hard ceilings | Profile ≤65,536 bytes，Skill ≤128、capability ≤64、criteria ≤256；Context ≤64 MiB/16,777,216 tokens/4,096 sources；Memory ≤4,096 items/16 MiB/4,194,304 tokens/10 年；Run ≤8 GiB RSS、24 小时 CPU/runtime、256 children/tools、100,000 calls、128 MiB output、16 MiB event buffer。Project/主机聚合预算与设备性能实测仍属后续 scheduler/Phase 12。 |
| scope 与历史 digest | 🟢 API contract 已实现 | Worktree profile 只能精确匹配请求 Worktree；Project profile 仅能在同 tenant/project 的 Worktree 使用；修改 payload 后 verify 检测 digest mismatch。每次实际 create/resume 仍须在 Run command 中重授权并复核 grant expiry/provider/lifecycle。 |
| 验证 | 🟢 隔离 PostgreSQL 验收通过 | Run、Profile、guard 三 migration 应用成功；guard migration 重复应用；旧 Run 与两类完整 snapshot 接受，5 类负例拒绝；accepted Runs=3（legacy=1、Profile snapshot=2），Profile FK=0；临时数据库清理并复查不存在。目标数据库未触碰。 |
| 高级设置导航 | 🟢 既有 ULYS-235 保持 | Hooks 是 Advanced Settings 页面内容区内与 Skills/MCP/Plugins 并列的 tab；主侧栏只承载高级设置父入口，Worktree Index 只投影 effective HookSet 与状态，Worktree 树不新增 Hook 节点。仓库 AGENTS.md 已有此规则，无需重复新增。 |
| 未关闭门 | 🟡 Phase 9E/9D 与生产验收开放 | 9E-3 Profile Master/SCD2 + Audit migration substrate 已新增但未部署目标 DB；9E-4A Run snapshot guard migration 隔离 PostgreSQL 验收通过，尚未部署目标 DB；发布/读取 API、生产 Run writer、current provider catalog adapter、资源原子 reservation 与 DB/RLS runtime 驗收仍开放。Rust-owned CLI adapter、Automation occurrence worker/fencing、Schedule/Cron rule、Loop checkpoint/runtime、跨 Project 有界公平调度也未完成；Phase 9D production provisioner/fence consume、真实 auth、Outbox 与完整 BI 仍开放。 |

### 6.54 本轮阶段结果（Phase 9E-2 bounded Execution Profile resolver，2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| 当前 Provider/Skill 匹配 | 🟢 bounded fail-closed resolver | `ExecutionProfileResolver` 先拒绝乱序/重复/无效/超限 catalog（Provider ≤256、Skill ≤4,096），再精确匹配 ID/version、provider implementation/config digest、capability support、Skill content digest 和 revoke/available 状态；不回退到其它版本。 |
| scope/grant/HookSet/lifecycle 复核 | 🟢 admission facts contract | 匹配 tenant/project/worktree，current grant set/version/capabilities/expiry 必须完全等于 profile snapshot 且未过期；effective HookSet ID/version/digest 必须一致；Worktree 非 Active 时拒绝。caller 必须先完成 actor/GroupContext ACL；这不是 auth provider 或 Run writer。 |
| Rust 前端/并行开销 | 🟢 借用式返回与有界目录 | Resolver 返回 `ResolvedAgentExecutionProfile<'_>` 借用，避免复制 Profile/Context/Skill 内容；catalog 校验线性扫描 bounded 条目，依赖查找使用 binary search；resolver 不持锁。共享 resource reservation、公平排队和设备级总 RSS 仍在后续 scheduler/Phase 12。 |
| 验证 | 🟢 隔离 PostgreSQL 验收通过 | Run、Profile、guard 三 migration 应用成功；guard migration 重复应用；旧 Run 与两类完整 snapshot 接受，5 类负例拒绝；accepted Runs=3（legacy=1、Profile snapshot=2），Profile FK=0；临时数据库清理并复查不存在。目标数据库未触碰。 |
| 未关闭门 | 🟡 后续 9E/9D 与生产验收开放 | Profile registry/SCD2 与授权 API、Run snapshot 原子 writer、provider/Skill 生产 catalog、quota reservation、CLI adapter/进程树 cleanup、Schedule occurrence/fencing、Loop checkpoint/runtime、BI/evidence 生产联动、目标 DB/RLS/grants/auth 均未接通。 |

ULYS-235 导航保持：Hooks 仍是 Settings 高级设置页面内容区与 Skills/MCP/Plugins 并列的 tab，不是主导航或 Worktree 节点。

### 6.55 本轮阶段结果（Phase 9E-3 Profile Master/SCD2 持久化基底，2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Profile 主记录 | 🟢 additive PostgreSQL migration | 新增 `multica.agent_execution_profile` Master，稳定 profile ID、Project/Worktree scope、连续 revision、schema version、active/disabled 状态、verified document JSONB、SHA-256 digest、操作者与 SCD2 有效区间；数据库将 envelope scope/version/digest 与 JSON document 字段对照。 |
| 历史与并发写保护 | 🟢 SCD2 guard contract | 事务级 advisory lock 串行化同 tenant/profile 的写入；只能关闭当前 revision 一次后追加连续 successor，禁止 scope 改绑、历史字段 UPDATE 与物理 DELETE；current revision 唯一。 |
| 审计与租户隔离 | 🟢 append-only Audit + FORCE RLS | 新增 publish/rollback/enable/disable Audit Transaction；目标/source revision FK 与 scope/state guard、4 KiB 脱敏 details；Audit 拒绝 UPDATE/DELETE/TRUNCATE；Profile/Audit 均启用并 FORCE tenant RLS。 |
| Run 快照边界 | 🟡 writer 接入仍待后续阶段 | Phase 8A 已有 nullable Run `execution_profile_id/version/digest/snapshot` 列；本 migration 不写 Run、不创建 API、不部署目标 DB。配对 migration 验证中，先写入 Profile v1 的 Run snapshot，再关闭 v1 并追加 v2 后，Run 内 JSON 与 digest 仍匹配 v1；生产 writer 仍须原子写入且保持对 Profile Master 无 FK。 |
| W/T/M 分类 | 🟢 100% 新表分类 | Profile 为 Master (M)，Audit 为 Transaction (T)；无临时 Draft 表，因此没有遗漏 Work TTL。详细设计表从 Work 4 / Master 2 / Transaction 6 更新到 Work 4 / Master 3 / Transaction 7。 |
| 验证 | 🟢 隔离 PostgreSQL 验收通过 | Run、Profile、guard 三 migration 应用成功；guard migration 重复应用；旧 Run 与两类完整 snapshot 接受，5 类负例拒绝；accepted Runs=3（legacy=1、Profile snapshot=2），Profile FK=0；临时数据库清理并复查不存在。目标数据库未触碰。 |

### 6.56 本轮阶段结果（Phase 9E-4A Run/Profile snapshot 数据库不变量，2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| 完整快照约束 | 🟢 additive migration 重复应用通过 | 2026-10-01 Run Profile snapshot guards migration 要求 Profile ID/version/digest/document 全空或全有；保留无 Profile 的既有 Run。 |
| Run envelope 一致性 | 🟢 正/负向 PostgreSQL 验收通过 | 无 Profile 的旧 Run、Project scoped 与 Worktree scoped 完整 snapshot 插入成功；部分 tuple、tenant/project/Worktree scope 与 digest 不匹配的 5 类负例均被拒绝；Run 到 Profile Master 的 FK 数为 0。 |
| 生命周期与集成边界 | 🟡 尚未接生产 writer | 迁移依赖 Phase 8A Run schema；Profile API、resolver current catalog、Run writer、目标 DB部署与资源原子 reservation 仍未完成。 |
| 验证 | 🟢 隔离 PostgreSQL 验收通过 | Run、Profile、guard 三 migration 应用成功；guard migration 重复应用；旧 Run 与两类完整 snapshot 接受，5 类负例拒绝；accepted Runs=3（legacy=1、Profile snapshot=2），Profile FK=0；临时数据库清理并复查不存在。目标数据库未触碰。 |


### 6.57 本轮阶段结果（Phase 9E-4B1 current Profile read API，2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| 有界 current Profile 列表 | 🟢 REST 代码切片 | `GET /api/v1/worktrees/{worktree_id}/execution-profiles` 要求 `worktree:read`，在 tenant RLS transaction 中验证 Worktree/有效 Project binding 和 active Project membership；只返回当前 active Project Profile 与当前 Worktree Profile 的 metadata，默认 20、最大 50、UUID keyset cursor，不返回 JSON document。 |
| current Profile 详情 | 🟢 Rust read-time verifier | `GET /api/v1/worktrees/{worktree_id}/execution-profiles/{profile_id}` 在同一 scope 授权边界中读取当前 document，调用 `decode_and_verify`，核对 tenant/project/worktree/schema/digest 后执行 `validate_for_scope`；历史、disabled 与其他 Worktree Profile 不暴露。 |
| 缓存与执行边界 | 🟢 fail-closed contract | 两 endpoint 都设置 `Cache-Control: no-store`。读取 Profile 不会验证当前 Provider/Skill/Grant/HookSet catalog，不发布 Profile、不创建 Run，也不执行资源 reservation；Run admission 必须继续调用 9E-2 resolver 并重授权。 |
| 验证 | 🟢 定向 Rust 测试通过 | `cargo test -p star-api-rest --lib group_api::execution_profiles::tests -j 4`：4 passed；crate test binary 完成编译。原始 Cargo.lock 在测试自动解析 240 个依赖后恢复，未提交锁文件 churn。SQL/真实 HTTP Auth/目标 DB RLS/grants 未做集成验收；保留已有用户文件 `Cargo.lock.phase9d-backup`。 |
| 后续门 | 🟡 未完成 | Profile publish/rollback/disable API 已由 §6.58 的 9E-4B2 代码切片覆盖；current Provider/Skill/Grant registry adapter、生产 Run snapshot writer、同事务 quota reservation、真实身份/RLS 与目标 DB 部署、Rust CLI adapter、Schedule occurrence、Engineering Loop 与公平调度继续开放。ULYS-235 Hooks 仍是 Settings“高级设置”内容区并列 tab，不进入 Worktree 树。 |

### 6.58 本轮阶段结果（Phase 9E-4B2 Profile lifecycle write API，2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Project/Worktree Profile lifecycle routes | 🟢 有界 typed REST 代码切片 | 新增两条 `POST .../execution-profiles/{profile_id}/lifecycle` 路径；请求 ≤67,584 bytes，typed Profile document ≤65,536 bytes，Serde 拒绝未知字段。支持 publish/disable/reenable/rollback。Project/Worktree profile scope 从 URL 和当前绑定解析，document 不能改 scope。 |
| 身份、授权与 CAS | 🟢 fail-closed route gate | 要求 `execution-profile:publish`、tenant RLS、当前 Project membership 与 `tenant_admin` / `project_admin` 角色；操作前锁 profile ID 并比较 `expected_current_version`，stale write 拒绝；Worktree endpoint 只改精确 Worktree scope。真实 JWT/OAuth scope provisioning 尚未接入。 |
| SCD2 与审计 | 🟢 同事务代码路径 | Publish 先 Rust verify；状态操作与 rollback 重验 stored document；事务在 SCD2 trigger 同键 advisory lock 后锁 current row，关闭旧 revision、插入 next successor 和 append-only Audit 后一次提交；错误整笔回滚。Rollback 以新 active revision 表示，不复活/覆盖历史行。 |
| 内存与响应 | 🟢 单请求 body bound | request body 硬上限 67,584 bytes；没有 provider/外部 I/O 在事务锁内；响应只回 digest 与小型 revision receipt，所有匹配路由的响应 no-store。真实 API 并发上限、DB pool 队列仍由服务主机配置控制。 |
| 验证 | 🟢 targeted Rust tests 通过 | `cargo check -p star-api-rest --all-targets -j 4` 通过；`cargo test -p star-api-rest --lib group_api::execution_profile_admin::tests -j 4` 重试后 3 passed、0 failed（首次链接收到 Windows `LNK1104`，检查无同名运行进程后重试成功）；rustfmt check 与 `git diff --check` 通过。Cargo.lock 的 240-package 自动解析 churn 已恢复，保留 `Cargo.lock.phase9d-backup`。 |
| 后续门 | 🟡 未完成 | 真实 Auth scope issuance、隔离 PostgreSQL 并发/CAS/trigger/Audit/RLS 验收、目标 DB/grants、Profile 可视 UI、current Provider/Skill/Grant catalog、Run snapshot writer、资源 reservation 与 production CLI/Loop/Schedule runtime 仍开放。ULYS-235 Hooks 保持 Settings“高级设置”内容区并列 tab，不进入 Worktree 树。 |

### 6.59 本轮阶段结果（Phase 9E-4B3 Profile HookSet identity bridge，2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Effective policy load | 🟢 已实现 identity seam | 在调用方授权的当前事务以 `FOR SHARE` 读取 Project baseline 与可选 Worktree overlay；校验继承 policy-set ID、Project version/rules 与 effective document digest。缺 baseline 返回 None，继承不一致拒绝。 |
| Profile HookSet snapshot | 🟢 已实现映射 | overlay 存在时使用 Worktree policy-set ID，否则 Project policy-set ID；effective version 与 lowercase digest 来自 Rust verified snapshot。旧 evaluator 调用仍走 policy-only compatibility wrapper。 |
| 验证 | 🟢 定向验证通过 | `cargo test -p star-api-rest --lib group_api::hook_policies::tests::execution_profile_hook_set_uses_effective_overlay_identity_and_digest -j 4`：1 passed（首次链接遇 LNK1104；确认无同名进程后重试成功）；`cargo check -p star-api-rest --all-targets -j 4` 通过；rustfmt check 与 `git diff --check` 通过。Cargo.lock 因当前 workspace resolver 自动展开 240 个 package，恢复生成 churn；保留既有未跟踪 `Cargo.lock.phase9d-backup`。 |
| 导航决策 | 🟡 源码/测试断言一致，动态未验证 | ULYS-235 Hooks 入口是 Settings“高级设置”内容区 tab，与 Skills/MCP/Plugins 并列；不新增 Worktree app 节点。Main/Project sidebar scope toggle 沿用既有 sidebar 需求，属于外层导航。对应 Vitest 因本 worktree 未安装前端依赖而无法启动。 |
| 后续门 | 🟡 未完成 | identity bridge 不等于 Run admission。当前 Provider/Skill/Grant catalog adapter、生产 Run snapshot writer/同事务 Hook ledger + RunEvent、actor/Auth provider、目标 DB/RLS/grants、资源 reservation、Rust CLI adapter、Schedule occurrence 与 Engineering Loop runtime 仍开放。Run writer 必须重授权并在最终事务内重读 effective HookSet。 |

该阶段只关闭 Profile/Run snapshot 所需的 effective HookSet 身份映射，不宣称 9E 整体或跨 App 生产闭环已完成。

### 6.60 本轮阶段结果（Phase 9E-4B4 Task Card CLI Profile identity audit，2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| 身份边界审计 | 🟢 设计已收口 | `TaskCliSessionStartBody`、readiness/start command 目前含 `approved_launch_profile_id`，其语义是 Runtime executable/argv/environment launch policy；AgentExecutionProfile 是单独的 Agent/Memory/Skill/Context/Validation/Loop/HookSet/grant/resource document。不得将两者共用 ID 或从客户端 seed 推导。 |
| 当前 Run writer 差距 | 🟡 已明确阻断 | `record_cli_task_run` 写 Task/acceptance 与 `hook_set_snapshot`，未写 Profile ID/version/digest/document；CLI request fingerprint 没有 AgentExecutionProfile ID；domain resolver 还依赖未接入的当前 Provider/Skill/Grant catalog。该结论来自当前实现读取；本阶段未改运行时代码，也未声称 Run/Profile 已闭环。 |
| 阶段状态与后续顺序 | 🟢 C1-C5 条件式切片已实现 | C1 Profile picker + request identity/fingerprint 版本兼容；C2 bounded catalog snapshot/revision fence；C3 条件式 Run/Profile/Hook/RunEvent/resource reservation writer；C4 双 Profile 一次性 spawn-fence contract/Run projection；C5 shared DTO、signature v2 与 Local Runtime 原子 consume foundation。后续进入生产适配阶段：接入权威 catalog/auth/profile sources、C3 reservation 与 Run writer、C5 consumer 到 provisioner/OS sandbox、Runtime outcome/BI，并完成目标 DB/RLS、撤权、并发、崩溃恢复和容量验收。所有 production gates 通过前新 Run admission/spawn 保持关闭，旧 Run 查询和 replay 保留。 |
| 性能/并行约束 | 🟢 已纳入设计 | profile 单文档 ≤65,536 bytes、Provider ≤256、Skill ≤4,096 项；锁外准备 bounded catalogs，锁内仅执行有界 verify、scope/version 比较和 SQL 写入，继续使用借用式 verified Profile，避免在并行 Run 中复制 Profile/catalog；资源额度由 admission 原子预约。 |
| 导航约束 | 🟢 保持已确认决策 | Hooks 是 Settings“高级设置”内容区内与 Skills/MCP/Plugins 并列的 tab；Main/Project scope、Worktree Index 是外层导航，Hook 配置不作为 Worktree 子节点。 |
| 文档同步 | 🟢 完成 | Requirements v5.33/AC-AEC-014、Basic Design v5.29 §16.16、Task DD v1.7 §14.11.8 同步了身份差异、fail-closed 与 9E-4C1..C4 的阶段依赖。 |
| 未关闭门 | 🟡 继续执行 | 本阶段仅完成代码审计与跨文档设计同步；Profile picker、幂等兼容实现、provider/skill/grant persistence/adapters、Run snapshot writer、同事务 quota reservation、Runtime fence、真实 Auth/DB/RLS/目标部署、CLI/Schedule/Loop runtime 与 BI 生产闭环仍未完成。 |

该阶段防止将 Local Runtime 的命令启动授权误作 Agent 执行栈版本。9E-4B4 阶段结束时，下一项工作为 9E-4C1；其实现结果见 §6.61。在 C2-C4 实现与验证前，不开放新的 Profile-bound Run。

### 6.61 本轮阶段结果（Phase 9E-4C1 Task Card Profile picker 与 request identity，2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Profile 输入与选择 | 🟢 已接入，保持 fail closed | Task Card CLI preview 读取当前 Worktree 有界 Profile metadata 并要求显式选择；验证 ID、scope、revision/schema version、digest 与重复项。最多使用 50 项；存在下一页、空列表、无效响应或读取失败时禁用 start，不自动选第一个或 seed/default。Approved Launch Profile 与 AgentExecutionProfile 分开呈现。 |
| REST identity 与 fingerprint | 🟢 已接入 | start body 和 readiness/start command 增加独立 `execution_profile_id`。选择 Profile 的新请求将 `cli_session_start_v2` 与 Profile ID 纳入 request fingerprint；缺 Profile 字段时保留旧序列化 tuple 和历史 fingerprint。新 Run 在 readiness 前要求有效 ID 及显式 Profile-bound provisioner capability。 |
| Run admission 安全门 | 🟢 默认关闭 | `supports_profile_bound_run_admission()` 默认 false，生产 provisioner 尚未装配；Hook/Run admission capability 同时要求该能力。当前代码不允许新 Run 绕过 Profile snapshot writer/C2-C4 完成门。 |
| 验证 | 🟢 定向静态检查通过 | `cargo check -p star-api-rest --all-targets -j 4` 通过；rustfmt check、`git diff --check` 与直接 `node .\node_modules\typescript\bin\tsc --noEmit` 通过。`npm run typecheck` 的 shim 解析失败，直接调用本地 TypeScript compiler 完成同一检查。未运行单元测试。Cargo check 生成约 240 package 的 Cargo.lock resolver churn，已恢复 tracked lock，并保留既有未跟踪 `Cargo.lock.phase9d-backup`。 |
| 文档同步 | 🟢 完成 | Requirements v5.34/AEC-014/AC-AEC-015、Basic Design v5.30 §16.16、Task DD v1.8 §14.11.8 与本节同步 C1 输入、幂等和 fail-closed 边界；ULYS-235 Hooks 继续是高级设置内容区并列 tab，不进入 Worktree 树。 |
| 未关闭门 | 🟡 生产集成仍开放 | 权威 Provider/Skill/Grant 与 Approved Launch Profile sources；真实 Auth/Project ACL 与目标 DB/RLS grants；C3 Run/reservation writer 的生产部署与 Reservation activate/reject/release；C5 consume foundation 接入 production provisioner/OS sandbox/spawn；Runtime outcome/Outbox/BI；并行 admission、撤权、失败恢复、TTL/capacity 与端到端验收。上述门未通过前 capability 保持 false。 |

本阶段只增加 profile selection identity 和历史幂等兼容，不表示 Profile-bound Run admission 已可用。

### 6.62 Phase 9E-4C2 bounded current catalog snapshot 与 revision fence（2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Reference-scoped snapshot | 🟢 Rust contract 已实现 | 新增 `CurrentExecutionCatalogSnapshot`：只承载 Profile 引用的最多 5 个 Provider、128 个 Skill 与 current GrantSet；Arc-backed immutable slices、估算载荷 ≤1 MiB，校验 scope/identity/revisions/digest/order/bounds。Resolver 新入口 `resolve_current_snapshot` 要求当前存储 revision 与 snapshot fence 完全匹配，并保留 borrowed Profile/catalog。 |
| Revision fence | 🟢 Rust contract 已实现 | Fence 绑定 tenant/Project/Worktree、Profile ID/version/digest、Provider/Skill revisions、GrantSet ID/version 和观察时间；TTL ≤5 秒，Run admission transaction 至少剩 1 秒。过期、revision drift、错 scope/profile 或剩余时间不足拒绝。来源每次 mutation 同事务增加 monotonic revision 是适配器强制契约。 |
| Admission capability | 🟢 默认关闭 | `TaskCliSessionProvisioner::supports_current_execution_catalogs()` 默认为 false，start route 与 `run_admission_producer_available` 都要求显式支持 current catalog 能力；既有 Profile-bound 与 Run-writer capability 仍分别检查。 |
| 数据库 substrate | 🟢 migration 切片已实现 | `2026-10-01-multica-agent-execution-catalog.sql` 建立 Provider/Skill/GrantSet SCD2 Master、normalized capability relations、Provider/Skill revision projection 与 append-only audit；tenant FORCE RLS、SCD2 guard、同事务 child guard、deferred capability-count guard 与 revision/audit triggers 已在隔离 PostgreSQL 18 实测。 |
| Current SQL adapter | 🟢 bounded read/recheck 切片已实现 | `star-api-rest::execution_catalogs` 按已验证 Profile IDs 查询 Provider/Skill、能力行与当前 GrantSet；载入 Skill labels 前以数据库聚合拒绝预计 heap >384 KiB；preflight/final transaction 使用 REPEATABLE READ 与 Profile/revision/Grant `FOR SHARE`，resolver recheck fence 与有效 HookSet；Arc-backed snapshot随 readiness command传递。final writer 尚未写 Profile snapshot。 |
| 生产数据源/运行门 | 🔴 未关闭，阻断新 Run | 数据表及 REST adapter 已存在，但 Provider/Skill/Grant publisher/source mutation API、真实 production catalog 装载、目标 DB/RLS grants 与 Runtime provisioner 未部署；进程内 ProviderRegistry/CLI SkillRegistry 不能作为授权来源。Catalog capability、Profile-bound Run 与 Run writer 保持 fail closed。 |
| 导航与 Hook 设置 | 🟢 决策已核对 | 沿用 ULYS-235 既有决策：父入口为 Settings → Advanced Settings（`/settings/advanced`），Hooks 规范路由 `/settings/advanced/hooks`，作为页面内容区与 Skills/MCP/Plugins 并列的 tab；Worktree Index/Run Detail/BI 显示 effective state 并深链回该 tab。该需求已有记录，不新增 Worktree tree node。 |
| 验证范围 | 🟢 targeted 验证通过；环境验收开放 | PG18 migration 重复应用、Provider insert/close revision+audit、deferred count mismatch reject 通过；`cargo test -p domain-agent --lib -j 4 --locked` 133/133，`cargo test -p star-api-rest --lib -j 4 --locked` 118/118，`cargo check -p star-api-rest --lib -j 4 --locked` 通过；REST test 的首次 Windows linker LNK1104 在确认无残留进程后重试通过。目标 DB/RLS grants、mutation API 与真实 provider/runtime 并发竞态仍未验证。 |
| 后续门 | 🟡 9E-4C3 开放 | 完成目录 publish/source mutation API 和生产 adapter/部署验收后，继续 9E-4C3：同一短事务重授权、调用 resolver，并原子写 Run 自包含 Profile/Task/Hook/RunEvent/BI 与 resource reservation；9E-4C4 再接双 Profile identity Runtime spawn fence。目标 DB/Auth/RLS、资源 reservation、production CLI/Schedule/Loop 与完整 BI 仍开放。 |

该切片完成 C2 migration/read/recheck 代码与隔离/包级验证，但不代表 catalog publishers、目标数据库授权或生产 Runtime 已部署，也不代表新 Profile-bound Run 已开放。Hooks 继续沿用用户既有高级设置并列 tab 需求。

### 6.63 Phase 9E-4C3 Run/Profile snapshot 与 Project-wide resource reservation（2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Run snapshot writer | 🟢 条件式代码切片 | 新 Run 保存经验证的 AgentExecutionProfile ID/version/digest/document、ResourceBudget 与 Engineering Loop snapshot；兼容旧 Run snapshot 与既有 idempotency replay。Profile snapshot scope/digest 与最终 readiness fence 不符时回滚。 |
| Project quota 与 admission | 🟢 migration/Rust slice 已实现 | Project quota 是无默认 seed 的 Master/SCD2 + append-only Audit。Admission 使用 checked integer conversion，同时验证 per-Run ceiling 与该 Project 所有 Worktree 下未过期 pending/active reservation 聚合；无 active quota、超额、算术边界错误或 fence 失效均 fail closed。 |
| REPEATABLE READ 并发保护 | 🟢 allocation epoch 写入已加入 | 仅锁 quota/reservation 行无法刷新 waiter 的事务快照；因此在读取 usage 前按 tenant/Project 原子 upsert `project_execution_resource_admission_lock`。竞争事务的 stale snapshot 写入收到 SQLSTATE 40001 并映射到 conflict，必须整体重试。此锁表为 30 天过期的 W 状态，租户 maintenance cleanup 尚未装配。 |
| 原子事件关联 | 🟢 代码/SQL slice 已实现 | Reservation pending row、append-only reservation event 与 `task_execution_run_event` 共用 `event_id`、Run/Task/actor/correlation 与受限 details，并与 Run snapshot 位于同一事务。reservation failure 回滚新 Run；命中已有 idempotency Run 时在 reservation 前返回，不会重复占额。reserved maxima 与 Runtime observed metrics 明确分开。 |
| Run/Reservation lifecycle | 🟡 Runtime gate 未关闭 | pending lease 不晚于 readiness admission fence；迁移 guard 支持 pending→active→released 和 append-only transition ledger，但 production Runtime 尚未接 consume/reject fence、activation/release、expired lease reconciliation 与进程树回收。Profile-bound provisioner capability 仍默认 false。 |
| W/T/M、RLS 与 schema | 🟢 隔离迁移与并发 smoke 已验证 | Quota Master、Quota Audit Transaction、Reservation Work、Reservation Event Transaction 与 Admission Lock Work 均启用 tenant FORCE RLS；无默认 quota。PG18 migration 重复应用及 Run budget/shared event/RLS/SCD2/append-only 检查通过；两个独立 `psql` REPEATABLE READ 会话竞争同一 allocation epoch，等待者收到 SQLSTATE 40001，只有 winner 的 epoch=1 提交。 |
| 测试与编译 | 🟡 定向验证通过，最终复跑受 linker 阻断 | allocation epoch 加入后 `star-api-rest --lib` 121/121 tests 通过，含 3 个资源预算边界 tests；最终将 serialization error mapping 扩展至 quota/reservation statements 后，`cargo check -p star-api-rest --lib -j 4 --offline` 与 changed-file rustfmt 通过。再次运行 tests 时 Rust test crate 编译完成，但 `link.exe` 因 LNK1104 无法覆盖共享 target 下的 `star_api_rest-…exe`，故这次没有执行测试。临时移除 `star-desktop` workspace member 后原 Cargo.toml/Cargo.lock 均恢复；原 `--locked` workspace 解析仍有 Wry/objc2 冲突，本阶段不宣称 workspace 全量通过。 |
| 生产 gate 与后续阶段 | 🟡 C3 代码完成，production 未完成 | Catalog publisher/生产 Provider-Skill-Grant 装载、Host Auth/Project ACL、目标 DB/runtime grants、epoch/reservation TTL cleanup、Runtime activation/release、Outbox/Run outcome BI 与真实 CLI provisioner 未连接，所以不能开启新的 Profile-bound Run。下一实现门为 9E-4C4 双 Approved Launch Profile + AgentExecutionProfile spawn fence；先完成本表所列生产前置与并发实测，再更新 capability gate。Hooks 导航继续按 ULYS-235 保留在 Settings → Advanced Settings → 内容区 Hooks tab。 |

### 6.64 Phase 9E-4C4 双 Profile 一次性 Runtime spawn fence（2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Typed fence binding | 🟢 REST contract 已实现 | `TaskRunSpawnFenceBinding` 限定 tenant/actor/Project/repository/Worktree/Task/Runtime/lifecycle、双 Profile ID/version/digest、catalog revision tuple、effective HookSet、ResourceBudget 与 request fingerprint；只复制 revision tuple 和有界预算，不复制 Arc catalog entries。domain-separated v1 digest 覆盖完整 typed binding；REST 重建并逐字段匹配 readiness 返回值。 |
| Runtime consume contract | 🟡 trait gate / production 未实现 | Provisioner contract 要求 Runtime 在 process create 前 compare-and-consume 单次 fence，复验当前 ACL/scope/lifecycle、两份 Profile revisions、catalog/HookSet 和预算，再处理 reservation lifecycle。已有 `domain-local-runtime::task_execution` 会验证签名 grant、scope、Launch Profile ID/字段、canonical checkout 与 nonce，但签名 context 没有 C4 双 Profile/revision/fence claims，且 helper 未被生产 provisioner 调用；它不是该 consumer。目前没有生产 fence store、Launch Profile authority/provider、Runtime consumer 或 OS spawn adapter，`supports_profile_bound_run_admission()` 继续默认 false。 |
| Run snapshot 与 BI projection | 🟢 migration/API/UI projection slice | 新 migration 为 Run 增加 Approved Launch Profile ID/version/digest 与 `spawn_fence_binding_digest` all-null-or-complete guard；新 Run writer 同 C3 Run/Hook/RunEvent/reservation 事务写入；Task Run list/detail API 与 TaskRunHistoryPanel 展示两类 Profile identity 和 fence binding digest，不落 opaque fence ID。该 digest 记录准入绑定，不代表 fence 已消费或进程成功。 |
| Replay/expiry/fail-closed | 🟢 contract 已显式定义 | 新 admission 必须具备 fresh/bounded Runtime fence，错绑/过期/不健康/缺字段均拒绝；无 fence 的 idempotent replay 只返回/恢复既有 Run/session，不能创建新进程。Runtime consume receipt/outcome event 与 reservation activate/release 尚未实现。 |
| Hook 导航 | 🟢 已核对 | ULYS-235 保持 Settings 主导航“高级设置”父入口，Hooks 规范路由 `/settings/advanced/hooks` 位于页面内容区并与 Skills/MCP/Plugins 并列为 tab；不作为独立主导航项，也不属于 Worktree 树节点。Worktree/Run/BI 只展示 effective strategy/result 并深链回此 tab。 |
| 验证与生产前置 | 🟡 Rust 定向编译通过；生产门未关闭 | 三个受影响 Rust 源文件的 targeted `rustfmt --check --edition 2021` 与 `git diff --check` 通过。常规 `cargo check -p star-api-rest --lib -j 4 --offline` 因 workspace 内 star-desktop/Wry 的 objc2 0.6.3/0.6.4 锁版本冲突而失败；临时排除该 workspace member 后同一 package check 通过（16.33s，只有既有非阻断 warning），检查生成的 Cargo.lock resolver churn 已恢复为原样，Cargo.toml 也已还原。当前 worktree 没有安装 TypeScript compiler，故未能 typecheck TaskRunHistoryPanel/API type；本轮不运行测试，PG migration 未执行。生产还需真实 Auth/Project ACL、Approved Launch Profile authoritative source、catalog publisher/data、target DB/RLS grants、Runtime fence consume/OS spawn、reservation lifecycle/TTL cleanup 与 Outbox/BI。 |


### 6.65 Phase 9E-4C5 Runtime 双 Profile fence 消费基础设施（2026-10-01）

| 子阶段 | 结果 | 证据/限制 |
|---|---|---|
| Shared fence contract | 🟢 已实现 | 将 C4 binding/profile/catalog/HookSet/ResourceBudget envelope 移到 `star-dto::task_run`，REST 与 Local Runtime 共用同一 strict type 与 v1 digest implementation；没有复制 Provider/Skill catalog entries。 |
| Signed grant binding | 🟢 已实现 | `TaskExecutionContext.spawn_fence` 在 None 时从 JSON 编码跳过并继续使用 grant signature v1；携带 fence 时改用 signature v2。签名/验证前先校验 fence binding、定长 digest 和 ≤30 秒窗口；旧通用 CLI consumer 拒绝带 fence 的 grant，避免绕过专用 consumer。旧 grant 的缺字段 decode 默认 None。 |
| Runtime consume validation | 🟢 基础 helper 已实现 | 验证签名、最大 30 秒 issue/expiry、current full binding equality、scope 与 approved launch profile revision/digest，然后返回 bounded prepared spawn envelope；调用方仍须先实时重验 ACL/authority/Runtime health/Reservation。helper 不创建 OS process，也未接入 production provisioner。 |
| Atomic replay receipt | 🟢 本地实现 | SQLite FULL WAL/`BEGIN IMMEDIATE` 在一个 transaction 写 grant nonce 与 fence ID receipt；重复 fence 回滚 nonce；过期超过 5 分钟 skew window 后清理，硬上限 50,000 records，达到上限或 SQLite 故障 fail closed。BI Outcome 事件仍未写入。 |
| 验证 | 🟢 定向检查通过 | `cargo test -p domain-local-runtime -p star-dto --lib --offline -j 4`：Local Runtime 174/174、star-dto 63/63 通过，覆盖 legacy v1 compatibility、signature v2 fence tamper rejection、超长 digest 在签名序列化前拒绝、fence drift/expiry、legacy entrypoint rejection 与 nonce/fence atomic replay rollback；`cargo check -p star-api-rest --lib --offline -j 4` 通过，focused `run_admission_requires_fresh_readiness_and_a_bounded_fence` 1/1 通过，只有已有 dependency warnings；受影响 Rust 文件 rustfmt check 与 `git diff --check` 通过。标准 workspace resolve 仍受既有 star-desktop/Wry `objc2 0.6.3` vs `0.6.4` 锁版本冲突影响；为定向检查临时从 workspace 排除 star-desktop，结束后精确恢复 Cargo.toml/Cargo.lock。没有启用 producer capability，也未宣称全 workspace 通过。 |
| 未关闭 production gates | 🟡 继续 fail closed | 尚无 production caller、live ACL/Approved Launch Profile authority/catalog/HookSet providers、target DB/RLS grants、Run/Reservation writer integration、Reservation activate/reject/release、OS sandbox/spawn adapter、Runtime outcome event/BI。完整 adapter 与重放并发/失败恢复/撤权/预算边界验收前，`supports_profile_bound_run_admission()` 继续默认 false。 |

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
| v1.1 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Phase 3C Canvas Document viewport/frames/connectors 的 Worktree-scoped SCD2/CAS API 与 additive migration；区分 3D UI 认证/API 接线、Canvas→WorkItem 原子命令、Outbox consumer 和 DB/ACL 验收门；需求、基本设计、详细设计同步到 v2.5/v0.6/v0.7 | 继续推进所有 Phase，先补齐 Canvas Document 持久化边界 |
| v1.2 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Phase 4A TaskExecutionContext、五分钟 grant 校验、approved launch profile 与 Runtime mount/canonical checkout 对照 helper；同步 3C/4A 验证状态与剩余 session/API/数据库门 | 继续推进所有 Phase，开始实现任务卡内 CLI 执行信任边界 |
| v1.3 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 细化 Phase 4A 可信 grant 来源、profile 非敏感静态环境、独立 secret capability 与 checkout TOCTOU 限制；记录 3C/4A 定向编译和自审状态 | 继续推进所有 Phase，完成当前代码切片的安全边界复核 |
| v1.4 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将 Phase 3 拆分为 3D 原子 Canvas→WorkItem 创建命令和 3E Group UI / Outbox / realtime；同步 requirements v2.6、basic design v0.7、detailed design v1.0 | 继续推进 Canvas 与 Task Card 双向互动 |
| v1.5 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Phase 4A 原子 nonce consume helper/SQLite WAL ledger 与 Work retention；细化 Phase 4B 仍缺 Task Card API、PTY/input、Audit 和 UI 接线；同步 basic design v0.8、detailed design v1.1 | 继续推进 Task Card CLI 的短期授权边界 |
| v1.6 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 盘点 terminal-stack / Local Runtime / Group UI 的实际边界；将 Phase 4B 拆为 session grant/API、Runtime PTY + 单次首帧 WebSocket attachment、Task Card UI/Audit/reattach；同步 basic design v0.9 与 detailed design v1.2 | 继续推进到所有 Phase 完成，先冻结 CLI 安全 attachment contract |
| v1.7 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Phase 4B protected WebSocket route seam、可注入 terminal sink、Local Runtime SHA-256 ticket ledger（Group/Task/Session/policy binding、≤60 秒 TTL、原子单次消费）和 browser ticket-first 客户端；记录 Rust 定向测试与 frontend 当前阻塞，并继续列明 Session API / PTY / sandbox / ACL / UI 接线未完成 | 继续推进到所有 Phase 完成，落实首帧 ticket 与安全连接路径 |
| v1.8 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修复 MSW 覆盖 WebSocket 测试替身的问题，覆盖 ticket-first 授权输入与断线重连取新 ticket；无 Session 的 CLI preview 明确保持断开；补 Monaco peer dependency 与全量 TypeScript 类型问题；记录 33/33 定向测试和 `pnpm typecheck` 通过，并保留服务端 CLI blockers | 继续完成 Phase 4B 浏览器传输验证并推进剩余 Phase |
| v1.9 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 推进 Phase 3E Canvas Outbox 授权读取切片：复核当前 Worktree scope、使用有界复合游标、仅返回 metadata；同步 requirements v2.7、basic design v1.1 与 Group detailed design v1.4，并保留 UI/consumer/realtime/DB 验收 blocker | 继续推进到所有 Phase 完成，先补充 Canvas Outbox 的授权读取能力 |
| v2.0 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 纠正 PostgreSQL 实际端口为 5555；确认 `star_dev` 为空；在隔离 validation DB 成功执行 Phase 2B-2D / 3B / 3C / 3E migrations，并验证 Canvas Outbox RLS、append-only trigger、复合游标并列排序与索引访问；更新 Phase 3E 当前门状态 | 继续关闭 Phase 3E 的数据库运行证据并推进所有 Phase |
| v2.1 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增显式 JWT provider 的 Group API client adapter 和 9 项定向测试；同步 requirements v2.8、basic design v1.2、DD-WORKTREE-GROUP-001 v1.5；明确 API adapter 已可供宿主接入而 Group 页面仍 mock，consumer/realtime 与部署验收未完成 | 推进 Phase 3E，先落实可注入的用户认证 API 传输契约 |
| v2.2 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 为前端 API adapter 增加 Outbox cursor retry helper；验证 projection refresh 失败会重放事件页，成功后按复合游标前进；记录 helper 未接 Group UI 且无持久 offset；adapter 测试增至 10/10 | 继续推进 Phase 3E 的安全消费切片 |
| v2.3 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增宿主 JWT provider 注入 seam、GroupContext/WorkItem/Canvas 的只读 API projection、fail-closed UI 和 Canvas route 本地持久游标刷新；focused Vitest 13/13 与 typecheck 通过；明确无实际宿主 provider、durable offset、realtime、写接线和 DB 部署 | 继续推进 Phase 3E 页面 API 接线 |
| v2.4 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补充 session generation key，保证 principal/session 切换时重建 API client 并在新 projection 到达前清除旧数据；requirements/basic/detailed design 同步至 v2.9/v1.3/v1.8；更新当前 blocker | Phase 3E 登录身份切换安全复核 |
| v2.5 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | requirements/basic/detailed design 同步至 v3.0/v1.4/v1.9；将认证态 Canvas→Task Card 创建入口、刷新投影与任务卡深链列入 Phase 3E；更新 Canvas 布局写、宿主 provider、DB、durable Outbox/realtime 与 Canvas/Jira blocker | Phase 3E 完成首条 UI 到同事务 WorkItem + Canvas 命令的跨 App 写路径 |
| v2.6 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | requirements/basic/detailed design 同步至 v3.1/v1.5/v2.0；在 Group 页面为空 Worktree 添加认证、幂等的 Canvas 初始化入口，成功刷新投影后才开放任务卡创建；Phase 3 blocker 保持并加入多 Canvas 选择与最终验证状态 | Phase 3E 自审发现没有 Canvas 的 Worktree 无法进入可交互协作闭环 |
| v2.7 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | requirements/basic/detailed design 同步至 v3.2/v1.6/v2.1；Group Canvas pan/zoom/fit 可暂存并通过带 expected_version 与幂等键的 Document CAS 显式保存；Focused Group/API 验证 18/18；Frame/connector/Element 编辑与 Phase 3 生产接入仍未完成 | Phase 3E 接通 Canvas viewport CAS 保存 UI 并记录余下 Phase blocker |
| v2.8 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 完成 Worktree scoped 多 Canvas 列表、`canvas_id` 路由选择、无效 ID 授权回退与幂等创建后切换；需求/基本/详细设计同步到 v3.3/v1.7/v2.2；`pnpm typecheck` 与 Group/API 18/18 通过；Phase 3 blocker 去除多 Canvas selector | 继续推进 Phase 3E，并完成一个 Worktree 管理多个协作 Canvas 的 UI 切片 |
| v2.9 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Group Canvas 新增将当前 Worktree 未关联 Task Card 放入 Canvas 的认证幂等入口；使用单请求原子创建 Element + typed EntityRef，刷新 projection 并提供 Task Card 深链；requirements/basic/detailed design 同步至 v3.4/v1.8/v2.3；`pnpm typecheck` 与 Group/API 19/19 通过；Phase 3 仍未完成 | 继续推进到所有 Phase，补齐 Canvas 与既有任务卡的双向互动路径 |
| v3.0 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | live Canvas 增加未锁定 Element 的位置拖动与版本 CAS 写入；冲突刷新授权投影，重复请求复用 correlation / idempotency identity；设计同步至 requirements/basic/detailed v3.5/v1.9/v2.4；静态验证待完成，Phase 3 仍受宿主认证、目标数据库、关系编辑、realtime 与 ACL 验收阻塞 | 继续推进所有 Phase，完成 Canvas 元素与任务卡的布局互动切片 |
| v3.1 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将 canonical WorkItem lifecycle transition API 接入 Multica、Jira、Task Card 在线视图；projection 带 lifecycle version/review state/active Worktree，命令带 expected version、幂等与 correlation，成功/冲突均刷新授权投影；requirements/basic/detailed 同步至 v3.6/v2.0/v2.5 | 继续推进到所有 Phase 完成，先打通任务管理同级 App 的共同状态写入 |
| v3.2 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将已启用的插件预览动态呈现为 Worktree 同级导航入口，并加入只读的插件预览页；同步 requirements/basic/detailed 至 v3.7/v2.1/v2.6；明确 Phase 6 真实 Registry、capability grant、运行时与热撤权尚未完成 | 继续推进到所有 Phase 完成，落实插件同级 App 导航切片并保留生产边界 |
| v3.3 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | CanvasView 支持多元素选择；Group Canvas 将 Frame 创建/删除、元素归入 Frame、纯视觉连线创建/删除暂存进固定版本 Document draft，并通过既有 CAS endpoint 显式保存；同步 requirements/basic/detailed 至 v3.8/v2.2/v2.7；保留 provider、DB、realtime 与其他布局编辑 blocker | 继续推进 Phase 3，完成 Canvas 与任务卡布局互动的可保存切片 |
| v3.4 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Frame 几何/演示属性和 connector 展示样式 Document CAS UI、Text/Sticky Note 内容 Element CAS、authenticated Element delete；服务端删除同事务清理 Frame/connector 引用并保留 canonical WorkItem；同步 requirements/basic/detailed 至 v3.9/v2.3/v2.8 | 继续 Phase 3E，推进 Canvas 属性编辑和安全删除闭环 |
| v3.5 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加未锁定 Canvas Element 的尺寸/旋转 version CAS UI 与 position/geometry/content 字段级命令；服务端拒绝模式外字段并从当前锁定行保留其它属性；requirements/basic/detailed design 同步至 v4.0/v2.4/v2.9；`pnpm typecheck` 通过，目标 DB / 宿主 provider / ACL/RLS 运行验收未做 | 继续 Phase 3E，补齐任务卡画布元素的几何属性编辑与字段级写边界 |
| v3.6 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Worktree Index 条件式消费授权 cursor API，展示 owner/Agent/Runtime/风险/锁信号并支持归档记录筛选；接通 archive/restore plan-confirm 二次确认与成功后刷新；同步 requirements/basic/detailed design 至 v4.1/v2.5/v3.0；`pnpm typecheck` 通过，宿主 provider、目标 DB、member directory 与真实 ACL/RLS 验收仍缺 | 继续解决多 Agent Worktree 管理不可控问题，推进 Project Worktree Index 从 seed 预览接向授权投影 |
| v3.7 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加当前 Project 有效成员目录 API 与 Worktree owner reassignment UI；负责人只可从授权目录选择，经带 version/correlation/idempotency 的短时 plan-confirm 后刷新授权 Index；requirements/basic/detailed design 同步至 v4.2/v2.6/v3.1；宿主 provider、DB 部署与 ACL/RLS 验收仍缺 | 继续推进全部 Phase，优先让多 Agent Worktree 责任归属可控、可审计 |
| v3.8 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Canvas Outbox 浏览器本地复合游标的 scope 隔离、恢复校验及先刷新后持久化顺序；同步 requirements/basic/detailed design 至 v4.3/v2.7/v3.2；保留服务端 consumer offset、realtime、宿主 provider、DB 与 ACL/RLS 生产门 | 继续完成所有 Phase，推进 Phase 3E 页面重载后的投影刷新恢复 |
| v3.9 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 WorkItem review submit/accept/reject REST command 与 Task Card 条件式 UI：claimant 提交、非 claimant reviewer 决策、驳回理由、Worktree/version/idempotency 守门和 append-only audit；同步 requirements/basic/detailed design 至 v4.4/v2.8/v3.3；静态 typecheck 与 cargo check 通过，测试未运行，DB/provider/真实 ACL 仍未验收 | 继续推进 Phase 2C / 3F，补齐 canonical WorkItem 的 review gate 操作闭环 |
| v4.0 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Ed25519 signed Task CLI grant：API issuer 私钥与 Local Runtime 公钥分离、key id 轮换、域分离签名 payload，以及先验签再做执行上下文检查和单次 nonce 消费的 helper；同步 requirements/basic/detailed design 至 v4.5/v2.9/v3.4；Rust 编译与静态检查通过，未运行测试；Session API、current ACL/Runtime health、PTY 与 OS sandbox 仍未接通 | 继续推进 Phase 4，开始接通 authenticated Task Session API 与受限 Local Runtime 执行入口 |
| v4.1 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Task Session start REST route 与 GroupApiState provisioner injection contract；验证 Project membership、Worktree/WorkItem link、claimant/lifecycle version、Runtime binding、Idempotency-Key、request fingerprint、correlation 与 no-store ≤60 秒 ticket 响应；默认未配置 provisioner 返回 503；同步 requirements/basic/detailed design 至 v4.6/v3.0/v3.5；定向 cargo check 通过，未运行测试；真实 signer/runtime/ACL/PTY/sandbox/UI 仍缺 | 用户要求继续推进所有 Phase，进入 Phase 4B1 Session API 控制面接线 |
| v4.2 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 Local Runtime `TaskPtyManager` 交互 PTY adapter：固定 prepared executable/argv/cwd、清空继承环境、限流输入与终端尺寸、序号化原始输出、退出观察及 terminate；同步 requirements/basic/detailed design 至 v4.7/v3.1/v3.6；`cargo check -p domain-local-runtime --all-targets -j 4` 通过、未运行测试；OS sandbox、grant/ACL/audit 与真实 REST/terminal-stack/UI wiring 仍缺 | 用户要求继续推进所有 Phase，推进 Phase 4B2 并保持 sandbox fail-closed |
| v4.3 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修正 PTY 输出从无订阅 broadcast 改为单消费者有界 FIFO 背压队列；增加 manager shutdown 子进程终止与 session terminate 幂等处理；同步 requirements/basic/detailed design 至 v4.8/v3.2/v3.7；cargo check 通过、未运行测试；生产 provisioner / sandbox / sink / scrollback 与其他 Phase blocker 仍未关闭 | 用户要求继续推进全部 Phase；Phase 4B2 自审发现输出订阅竞态并补齐生命周期边界 |
| v4.4 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements/basic/detailed design 至 v5.0/v3.3/v3.8；记录 Phase 4B3 卡内 CLI ticket-first xterm UI；新增 Phase 5 scoped Chat API 的 WORKTREE/GLOBAL target guard、逐目标 GroupContext preflight、EntityRef scope、幂等 fingerprint 与 fail-closed workflow seam；保留 LangGraph/Transcript/stream UI、Phase 4 运行闭环、Phase 6 Plugin runtime 与 Phase 7 跨 App 验收 blocker；cargo check、pnpm typecheck、rustfmt 与 diff-check 静态验证通过，未运行测试 | 继续推进所有 Phase，补齐 Task Card 终端 UI 并开始 Phase 5 服务端 scope/target 授权 |
| v4.5 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加受权 GLOBAL Chat 目标目录及 Group 底栏多选分页；服务端只返回当前有效 Project membership 所属的非归档 Worktree 最小投影，并在单一 tenant transaction 内锁定当前路径 Worktree/membership；提交时仍逐目标重新授权；修正省略 correlation 时重放 fingerprint 不稳定的边界，默认 correlation 回退为幂等键；同步 requirements/basic/detailed design 至 v5.1/v3.4/v3.9；cargo check、pnpm typecheck、rustfmt 与 diff-check 通过，未运行测试；Phase 5 workflow/LangGraph/Transcript/stream 与 Phase 4/6/7 production blockers 保留 | 继续推进所有 Phase，接具体 Chat workflow 和持久 Transcript |
| v4.6 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Task CLI Session status/cancel/reattach REST route 与 Runtime trait contract，逐请求复验 Project writer membership / canonical Task/Worktree link，status 脱敏 no-store、cancel 幂等、reattach 新签短时单次 ticket；同步 requirements/basic/detailed design 至 v5.2/v3.5/v4.0；`cargo check -p star-api-rest --all-targets -j 4` 通过，未运行测试；真实 provider、DB / scope issuer / sandbox / sink / audit / UI reconnect 与 Phase 5-7 blockers 仍未关闭 | 继续推进所有 Phase，先建立 Task CLI Session 的状态与恢复控制面 |
| v4.7 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将 Task Card 状态刷新、幂等取消与手动断线恢复接入 lifecycle API；重连前检查 session 状态并验证新 ticket binding/expiry；同步 requirements/basic/detailed design 至 v5.3/v3.6/v4.1；`pnpm typecheck` 通过、定向 API Vitest 15/15 通过；明确跨页面 Session listing/recovery 与所有生产 Runtime/DB/ACL blockers 仍未关闭 | 继续推进所有 Phase，完成 Phase 4B3 Task Card Session lifecycle UI 切片 |
| v4.8 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增授权 Session listing GET 与 Task Card 刷新后列表发现/手动恢复；limit 默认 20、上限 50、脱敏 no-store、无 ticket/output，恢复需当前状态检查和新票据；Rust 定向 check + 2 个 listing 单测、`pnpm typecheck`、Group API/terminal Vitest 36/36 与 diff-check 通过；生产 provisioner/认证/DB/OS sandbox/audit 与 Phase 5-7 仍未关闭 | 继续推进全部 Phase，完成 Phase 4B4 Session listing/recovery 可交互切片 |
| v4.9 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 基于 Context7 对照当前官方 LangGraph Python 文档，确定 server-generated thread ID、`Command(resume=...)`、checkpoint replay re-execution、每次 resume/tool call 重新授权、幂等 Domain Command/outbox 与 PostgresSaver setup 门；同步 requirements/basic/detailed 为 v5.5/v3.8/v4.3；此项是兼容性设计核对，未安装/运行 LangGraph，Phase 5 runtime/Transcript/UI 与其他 Phase blocker 继续开放 | 继续推进所有 Phase，先冻结 Phase 5 checkpoint 与 replay 的安全边界 |
| v5.1 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements/basic/detailed 至 v5.7/v4.0/v4.5；发现 Group Chat 正文明文 `TEXT` 与 Agent Policy 不符，新增加密/保留/密钥轮换与销毁/导出删除验收门，并标记当前 migration 不可生产部署；Phase 5 与所有 Phase 仍未完成 | Transcript 持久化自审发现敏感正文缺少既有 AI Prompt/Response 数据治理 |
| v5.2 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements/basic/detailed 至 v5.8/v4.1/v4.6；Transcript schema 拆为 T metadata 与 W 密文载荷，强制 protector 注入；新增 WG-ACC-16，cargo check 通过，真实 KMS/清理/导出删除和 DB RLS 仍未验收 | 根据 CHAT-006 收紧正文持久化边界并记录 Phase 5 当前生产门禁 |
| v5.3 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements/basic/detailed 至 v5.9/v4.2/v4.7；增加 `/group-apps` fail-closed read projection/provider seam，复验当前 membership/binding/grant，验证最小唯一导航列表；真实 Registry、grant/runtime/hot revoke、UI consumer 和 Phase 7 验收仍缺 | 继续 Phase 6，先封闭生产 Worktree 同级插件入口的服务端授权读取边界 |
| v5.4 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | requirements/basic/detailed 同步至 v5.10/v4.3/v4.8；新增 Plugin Registry 五表 migration、Master SCD2/no-delete + Audit append-only/FORCE RLS、production main 装配的只读 PostgreSQL provider 与 WG-ACC-18；明确未应用数据库、manifest trust/ingest、lifecycle writer、capability runtime/hot revoke、UI consumer 和 Phase 7 仍未完成；验证状态如 §6.36 | 继续推进所有 Phase，先将 Phase 6 从 API contract 推到持久化授权 projection |
| v5.5 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements/basic/Group detailed design 至 v5.11/v4.4/v4.9；完成 Group UI 对服务端插件授权导航 projection 的条件式 live 消费，加入加载/错误/provider generation fail closed、显式与周期刷新以及 WG-ACC-19；typecheck、rustfmt、cargo check 和 diff-check 通过；数据库、manifest trust/lifecycle/runtime、热撤权与 Phase 7 仍未完成，状态见 §6.37 | 继续推进全部 Phase，关闭可在本仓完成的 Phase 6 导航消费者切片并更新跨 App 验收门 |
| v5.6 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements/basic/Group detailed design 至 v5.12/v4.5/v4.10；抽取 Group App Registry projection validator，拒绝跨 Worktree/错误 UUID/版本/字段/重复 ID/超限 payload 并稳定排序；聚焦测试 3/3、pnpm typecheck 和 diff-check 通过；迁移、可信 ingest、lifecycle/runtime、真实 RLS 与 Phase 7 仍未完成，见 §6.38 | 继续推进所有 Phase，先补强 Phase 6 投影边界并形成可重复的聚焦测试 |
| v5.7 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修正 Group 详细设计首页的基本设计引用并升至 v4.11；建立 Phase 7 本地回归基线：前端 44/44、REST 75/75、Local Runtime 168/168、terminal-stack 82/82，pnpm typecheck 与 diff-check 通过；明确这些不代替宿主身份、目标数据库/RLS 和跨 App 生产验收 | 继续推进全部 Phase，以当前 Group/terminal/API 改动验证本地回归并校正设计文档引用 |
| v5.8 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | requirements/basic/detailed 同步到 v5.13/v4.6/v4.12；Phase 5/6 migrations 在隔离 PostgreSQL 库可重复执行，12 表 FORCE RLS、租户/actor policy 与 append-only trigger 事务测试通过；明确 RLS 不授予 SQL role privileges，并新增目标 runtime role/grants blocker 与 WG-ACC-20 验收契约 | 隔离数据库验证发现 schema/table grants 与 FORCE RLS 是互相独立的生产门禁 |
| v5.9 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将开发守门 #33 从 placeholder 改为 Cargo resolved graph 双向边检查；收敛 Canvas/Domain source ownership 与 shared adapter 范围，并让 #34 显式 Group ID 错配返回非零；聚焦验证 20/20、真实 workspace 0 条跨组边、py_compile 与 diff-check 通过。Phase 产品能力仍受宿主身份、目标 DB/runtime grants、Local Runtime、LangGraph、Plugin runtime 与 Phase 7 生产验收阻塞 | 用户继续推进 Worktree-first Group Apps 全部 Phase；先补完当前分支可落地的跨组依赖守门 |
| v5.10 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements/basic/detailed 至 v5.14/v4.7/v4.13；Phase 2D 增加 Git lock observer seam、freshness/timeout/bounded fan-out 与 unknown-safe Index UI；`locked` 持久字段不再代表实时 Git lock；生产 main 未安装 observer，agent/session drain、create/import、物理清理、目标环境与 Phase 7 生产验收仍未完成；验证结果见 §6.42 | 继续推进全部 Phase，优先把 Worktree Index 的锁状态做成可区分、可观测且不误导清理决策的信号 |
| v5.11 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 自审修正 Group Index 消费端：必须同时满足 host_runtime 来源与有效、新鲜时间戳，missing/unavailable/stale/future 均回退 unknown；详细设计链接 Git 官方 retention-lock 语义；Rust 81/81、`pnpm typecheck`、Group/terminal Vitest 46/46、rustfmt 和 diff-check 通过；Phase 2D Host Runtime provider、Agent drain、目标环境与 Phase 7 blockers 继续开放 | Phase 2D UI 边界复核发现应由客户端再次 fail closed 处理无效或陈旧来源 |
| v5.12 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修正本实施计划首页引用，使其指向当前 requirements v5.14、basic design v4.7 与 Worktree Group detailed design v4.13；Phase 实施与外部 blockers 状态未改变 | 最近同步需求与设计版本后，实施计划首页仍保留旧的 basic/detailed 版本号 |
| v5.13 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | requirements/basic/detailed design 升至 v5.15/v4.8/v4.14；新增 Project-scoped create/import/candidate API contract 与 7 个 focused Rust tests；客户端 path/URL 被严格拒绝、缺 provider 返回 503；生产 provider、Index UI、目标 DB/ACL/RLS 与 Phase 2D/Phase 7 验收仍开放，结果见 §6.43 | 继续完成全部 Phase，先推进 Worktree Index 安全生命周期写入口 |
| v5.14 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | requirements/basic/detailed design 同步升至 v5.16/v4.9/v4.15；新增脱敏 Project Repository 查询、Index create/import 控件与严格投影/receipt allowlist；`pnpm typecheck`、Worktree Lifecycle UI + Group API Vitest 22/22、REST 定向 check 和 lifecycle tests 8/8 通过；宿主认证、Project-Repository SoR、生产 provider/durable writer、目标 DB/ACL/RLS、Agent drain、cleanup 和 Phase 7 验收继续开放，见 §6.43 | 继续完成全部 Phase，推进 Worktree Index 安全生命周期 UI 消费 |
| v5.15 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | requirements/basic/detailed design 同步升至 v5.17/v5.0/v4.16；新增 `/api/v1/projects` 当前用户授权目录与生产 Project Selector，支持 UUID cursor 分页、role/字段校验、深链等待和失败重试，API session generation 更换时同步隐藏旧 Project/Index/member-role projection；前端 26/26、typecheck、star-api-rest lib 90/90、定向 cargo check、rustfmt 与 diff-check 通过；Project/Repository SoR、宿主身份、目标 DB/RLS、生产 lifecycle provider、其他 Phase runtime 与 Phase 7 验收仍开放，见 §6.44 | 继续推进全部 Phase，先消除生产 Project Index 入口对本地 seed 的依赖并关闭跨 session 旧投影窗口 |

| v5.18 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 对齐 requirements v5.19/basic design v5.3/Group DD v4.19/Task DD v0.5/Hook SRS-BD-DD v0.5；增加 Phase 8A/8B 到 Phase 9-13 的范围、依赖与验收门；标明 Profile/Loop/Hook/BI/Benchmark/Rust desktop 尚未实现，外部身份/DB/RLS blockers 继续开放 | 用户要求推进所有 Phase 并同步文档，同时补齐 Rust Hook 高级设置导航与 Run/BI/Loop 架构 |
| v5.19 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.20/basic design v5.4/Group DD v4.20/Task DD v0.6；将 Phase 8A 隔离 migration 验证与 Phase 8B 有界 Run list/detail API、Task Card 历史面板记为条件式实现；目标数据库/RLS、Contract commands、非 CLI producer、Phase 9-13 和生产跨 App/性能验收继续开放 | Phase 8B Run 查询/UI 代码落地并纠正计划中的旧“未实现”状态 |
| v5.20 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步基本设计 v5.5；新增 Phase 9A Rust-native archive/cleanup evaluator core 的实现证据与未接线边界；将 Phase 9 拆为 evaluator、policy/domain gate、Advanced Settings tab、BI 与 Agent/Loop 五个可验收切片；保留 root lockfile Wry/objc2 冲突和 star-desktop 截断占位作为 workspace validation blockers；Hook SRS/BD/DD 的既有 ULYS-235 导航契约保持 v0.5.1 | 用户重申 Hooks 属于既有 Advanced Settings 标签栏并要求继续完成 Phase |
| v5.21 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步基本设计 v5.6；记录 9B1 有界 JSON policy loader、typed validation、digest verification 和不可变 runtime snapshot 的 13 个单测/Clippy 证据；拆分 9B2 持久策略发布与 Worktree lifecycle gate；明确 Advanced Settings → Hooks 是 ULYS-235 既有并列标签结构且当前 UI route 仍待 9C | 用户再次指出 Hooks 应位于高级设置选项卡，并推进 Phase 9 的 Rust 实装 |
| v5.22 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步基本设计 v5.7 / Hook DD v0.5.2；记录 9B2A policy W/T/M migration 结构及 PostgreSQL unavailable 的验证限制；将 store/publish API 和 lifecycle command gate 分成 9B2B/C；保持 ULYS-235 Advanced Settings 并列 tab 为现有导航约束 | 继续推进 Hook policy persistence，避免把未应用 migration 表述为已部署 |
| v5.23 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步基本设计 v5.8 / Hook DD v0.5.3；补入一次性 PostgreSQL 18 上的 migration 双次应用、FORCE RLS、Project/Worktree 重基与 Audit 约束场景证据；区分隔离验证与目标 DB 部署/runtime grants；保持 ULYS-235 Advanced Settings 独立并列 tab 约束 | 完成 9B2A 隔离数据库验证并更新实施状态 |
| v5.24 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步基本设计 v5.9、Group DD v4.21、Hook DD v0.5.4；记录 9B2B scoped policy API、CAS/TTL、admin publish/rollback、16-document memory batches、事务内 overlay rebase、99 个 library tests；保留目标 DB/grants/API RLS integration、9B2C-9E blocker 与 ULYS-235 Advanced Settings 并列 tab 导航 | 完成 9B2B policy API 条件式代码切片并同步 Phase 状态 |
| v5.25 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录当前 workspace lock 的 `--locked` 重新解析限制与既存 Wry/objc2 依赖冲突；仅补直接依赖边，未提交 240-package 全量解析 churn；保留 9B2B 测试证据、目标 DB/RLS blocker、后续 Phase 与 ULYS-235 Advanced Settings 并列 tab 导航 | 提交前复核发现原始 workspace lock 不能在 `--locked` 下完成解析 |
| v5.26 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 basic design v5.10、Group DD v4.22、Hook DD v0.5.5；记录 9B2C archive-confirm gate、锁外 bounded Runtime/Git 预检、锁内 reauthorization/effective policy evaluation/version-freshness CAS 与 102 REST library tests/check/Clippy evidence；明确 provider 缺失 503、target DB/RLS、physical cleanup、RunEvent/outbox/BI 与 9C UI blockers；保持 Hooks Advanced Settings 并列 tab | 推进 9B2C 并将外部等待从 Worktree database-lock window 移出 |
| v5.27 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 basic design v5.11、Group DD v4.23、Hook DD v0.5.6；记录 archive admission fence expiry、至少 5 秒提交余量与写入前重检、103 REST library tests；明确生产 provider/事务期限联合验收、目标 DB/RLS、physical cleanup、RunEvent/outbox/BI 与 9C UI blockers；Hooks 保持 Advanced Settings 并列 tab | 补强 archive 命令期间 admission fence，并同步设计/实施边界 |
| v5.28 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 basic design v5.12 与 Hook SRS/BD/DD v0.5.2/v0.5.2/v0.5.7；记录 Advanced Settings 主入口及 Hooks/Skills/MCP/Plugins 并列 tabs、9C typed policy Builder/API client 条件式代码和 fail-closed session 缺口；补入前端定向测试、TypeScript 与全站 build 限制 | 用户再次确认 Hooks 是高级设置中的选项卡，并要求持续推进 Run/BI 架构 |
| v5.29 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 明确 Phase 9C 导航是 ULYS-235 的既有决策；新增旧版 `star-nav-store:v2` 一次性迁移，保留侧栏偏好并补入“高级设置”，后续仍尊重用户移除；导航、迁移、Builder 与 Group API 聚焦验证 38/38，TypeScript 通过；宿主认证/生产 API、全站 `/worktree` build 阻断及 9D-9E 缺口保持显式 | 用户提醒 Hooks 应沿用此前规定，作为高级设置中的一个选项卡 |
| v5.30 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Phase 9D `hook_execution_event` migration、archive producer 与 Project-authorized keyset API 的阶段范围；区分独立 lifecycle event ledger 与有 Run/Task FK 的 RunEvent；声明当前仅 archive phase、coverage partial/null，Rust/SQL 验证、目标 DB/grants、Run producers、Outbox state、BI read model/UI consumer 均开放；修正文档 cross-reference | 推进 Hook execution event / BI 接缝实现 |
| v5.31 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将已实现的 Project-scoped Hook execution event panel 纳入 Advanced Settings → Hooks 阶段边界；同步 Rust 106/106、前端 targeted 25/25、TypeScript、隔离 PostgreSQL FORCE RLS/append-only 验证；明确真实 app auth Provider、目标 DB/RLS/grants、Run-linked producer、Outbox delivery state 和 BI read model 仍待验收 | 完成 9D Hooks tab 事件面板/API contract 并同步验证事实 |
| v5.32 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Project-scoped Hook summary API 与 hook_execution_summary_v1 phase/decision 计数，固定 1–90 天窗口、公式及 partial/null coverage；只统计当前 archive ledger，不宣称 RunEvent/outcome join、完整 BI、Outbox、目标 DB/RLS 或 Quality & Improvement UI 已实现；记录 targeted cargo check 通过且未运行 tests | 推进 Phase 9D 第一个可复算的 Hook BI 汇总切片 |
| v5.33 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 在既有 Advanced Settings Hooks 标签接入有界 summary card，选择 7/30/90 天并按 phase/decision 呈现；同步 Hook SRS/BD/DD 与 overall basic design；保持 archive-only partial/unknown、Run outcome、真实 auth/DB 与完整 BI 未关闭状态 | Phase 9D-3 summary API consumer 完成代码切片，同时复核 ULYS-235 标签导航层级 |
| v5.34 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步需求 v5.21、基本设计 v5.17、Hook SRS/BD/DD v0.5.4/v0.5.6/v0.5.12；新增 summary v2 双来源 event_id 去重、最新 Run 状态关联、不完整 projection 计数、专项时间索引和独立 coverage 语义；明确 Run producer、Outbox、完整 BI、目标 DB/RLS/grants 与 host auth Provider 未完成 | Phase 9D-4 read model 支持跨账本和 Run state join，同时复核 Hooks 的 Advanced Settings 导航归属 |
| v5.35 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将 Phase 9D-4 实际验证结果入档：TypeScript、rustfmt、diff-check 与临时排除 star-desktop 的 star-api-rest lib 编译通过，未运行 tests、migration 未在目标 DB 执行；确认下一阶段须先扩展仅支持 archive 的 native typed evaluator，避免伪造 Run-linked Hook event | Phase 9D-4 本地代码验证完成，复核 Run producer 的 evaluator 依赖边界 |
| v5.36 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Phase 9D-5a typed evaluator API v2、BeforeRunAdmission/archive phase isolation、legacy v1 archive digest compatibility、fact allowlist、UI disabled 与 Project/Worktree publish/rollback fail-closed gate；同步 requirements v5.22、basic v5.18、Hook SRS/BD/DD v0.5.5/v0.5.7/v0.5.13，并确认 Hooks 仍在 ULYS-235 Advanced Settings 内容区并列标签 | 用户再次指出 Hooks 应是高级设置选项卡，同时继续推进 Run/BI 架构 Hook 阶段 |
| v5.37 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.23、basic v5.19、Hook SRS/BD/DD v0.5.6/v0.5.8/v0.5.14 与 Task DD v0.7；记录 9D-5b readiness/fence、同 event_id Hook ledger/RunEvent 双写、动态 capability/coverage、110 REST + 17 domain + 4 frontend tests 与 TypeScript 通过；注明无 production adapter、目标 DB/auth/Outbox/full BI 仍开放，Advanced Settings → Hooks 标签位置保持不变 | 将 phase-scoped evaluator 推进到 Run admission REST/事务数据流并复核 BI event identity |

| v5.38 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.24、basic design v5.20、Task DD v0.8；记录 Phase 9E-1 Rust Profile immutable verifier、scope/canonical digest/grant checks、explicit Memory、Context preservation、independent Validation 与 per-Run memory/CPU/queue ceilings；定向 domain-agent 124/124 测试通过，workspace lock conflict 与目标环境/provider/Run persistence blockers 保留；确认 ULYS-235 高级设置并列标签不变 | 用户指出 Hooks 入口应遵循既有 Advanced Settings 导航需求，并继续推进所有 Phase |
| v5.39 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.25、basic design v5.21、Task DD v0.9；补齐 ContextAssembler/LoopPolicy versioned provider+digest+grant snapshot 与 Profile Master/Run admission snapshot 边界；provider grant 负向验证通过；ULYS-235 Hooks 仍是 Advanced Settings 内容区并列 tab | Phase 9E-1 自审补齐上下文与循环实现版本引用，继续推进 Run admission |
| v5.40 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.26、basic design v5.22、Task DD v1.0；记录 bounded resolver 对当前 provider/skill/grant/HookSet/Worktree 精确复核、borrowed return、129/129 定向测试与现有 workspace resolver blocker；保留 DB registry/Run writer/shared quota/runtime/production auth 缺口；确认 ULYS-235 导航不变 | 推进 Phase 9E-2 当前执行依赖解析与安全准入 seam |
| v5.41 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.27、basic design v5.23、Task DD v1.1；新增 Profile Master/SCD2 + append-only Audit migration substrate、Scope/digest/schema consistency、FORCE RLS 与 Run self-contained snapshot boundary；临时 PostgreSQL 双次应用、3 revision/3 Audit、8 类负例、tenant RLS 隔离通过；组合 Run + Profile migration 验证 v1 Run JSON/digest 在 Profile v2 successor 后保持不变；临时 DB/角色清理；保留目标 DB/API/生产 Run writer/resource reservation 与 9D/9E runtime blockers；ULYS-235 Advanced Settings tab 导航保持 | Phase 9E-3 为 Run Profile snapshot 建立持久化基底并完成隔离 schema 与历史快照组合验收 |
| v5.42 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.28、basic design v5.24、Task DD v1.2；新增 Run Profile snapshot all-or-none 与 envelope scope/digest guard migration；明确无 Profile FK 与旧 Run 兼容；SQL 静态审查完成，隔离 PostgreSQL 验收待办，Profile API/生产 Run writer/resource reservation/9D runtime 仍开放；ULYS-235 Advanced Settings tab 导航保持 | 修复 Run Profile 可部分为空或与任务 scope 不匹配的持久化风险 |

| v5.43 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.29、basic design v5.25、Task DD v1.3；完成 Phase 9E-4A guard migration 隔离 PostgreSQL 双次应用、legacy Run、Project/Worktree snapshot、5 类负例与 no-FK 验收并清理临时库；保留目标 DB/Profile API/生产 Run writer/resource reservation/9D runtime blockers；ULYS-235 Advanced Settings tab 导航保持 | Phase 9E-4A 验收闭环 |
| v5.44 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.30、basic v5.26、Task DD v1.4；增加 9E-4B1 有界 Worktree current Profile list/detail GET 与 Rust read-time verifier，4 个单测通过；明确 SQL/Auth/RLS 集成、Profile publish/current registry、Run writer 与资源 reservation 仍未完成；确认 ULYS-235 Hooks 是高级设置内容区并列标签 | 9E-4B1 Profile read API 代码切片与验证完成 |
| v5.45 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.31、basic v5.27、Task DD v1.5；记录 9E-4B2 typed lifecycle routes、67,584-byte body bound、admin auth/CAS、SCD2 successor 与 same-transaction append-only Audit；`cargo check --all-targets` 与 3 个 targeted tests 通过，首次 Windows LNK1104 后重试成功，Cargo.lock churn 已恢复并保留 backup；真实 Auth/DB/RLS/target deployment/current catalog/Run writer/resource reservation 仍开放；ULYS-235 高级设置 Hooks 并列 tab 导航保持 | 实现 Profile publish/disable/reenable/rollback 生命周期最小切片 |
| v5.46 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.32、basic design v5.28、Task DD v1.6；落地 Phase 9E-4B3 effective Project/Worktree HookSet identity bridge，定向测试 1/1 通过（首次链接遇 LNK1104，重试成功）与 star-api-rest all-targets check 通过；明确 Run writer/catalog/auth/DB/RLS/resource reservation/CLI/Loop/Schedule 仍开放；复核 ULYS-235 Advanced Settings tab 与既有侧栏 scope 导航 | 将 verified Hook policy identity 接入 AgentExecutionProfile admission 路径 |
| v5.47 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.33、basic design v5.29、Task DD v1.7；记录 Phase 9E-4B4 CLI Run identity 审计：Approved Launch Profile 与 AgentExecutionProfile 独立，当前 request/fence/writer 缺 Profile identity/snapshot；定义 9E-4C1..C4 的选择、catalog fence、原子 Run/resource writer 与双身份 spawn fence；确认 Hooks 高级设置并列 tab 决策 | Run admission 源码审计发现 Launch Profile 与 Agent Profile 尚未打通 |
| v5.48 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.34、basic design v5.30、Task DD v1.8；记录 9E-4C1 CLI Profile picker、API identity、versioned fingerprint/legacy replay，默认关闭 Profile-bound producer 与 C2-C4 blockers；Rust all-targets check、rustfmt/diff-check 通过，npm typecheck 入口受损、未运行 tests；恢复 Cargo.lock resolver churn 并保留既有 backup；Hooks 高级设置 tab 要求保持 | 完成 Phase 9E-4C1 并核对用户重申的 ULYS-235 导航需求 |
| v5.49 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.35、basic v5.31、Task DD v1.9；加入 C2 reference-scoped Arc catalog snapshot、≤5 秒 revision fence、1 秒 transaction margin 与默认关闭 catalog capability；如实记录缺少权威 Provider/Skill/Grant store、revision SQL adapter，故 C2 production gate 未关闭；明确 ULYS-235 Hooks 规范路由 `/settings/advanced/hooks` 属高级设置并列 tab | 推进 9E-4C2，并确认 Hooks 导航沿用用户既有需求 |
| v5.50 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.36、basic v5.32、Task DD v1.10；落地 Provider/Skill/GrantSet SCD2 migration、normalized capability/append-only audit、bounded SQL reader 与 final REPEATABLE READ revision/Profile/Grant/HookSet recheck；isolated PG18 mutation checks、133 domain tests、118 REST tests 与 REST lib check 通过；publisher、目标 DB/RLS grants、Runtime adapter 与 C3 atomic Run/resource writer 仍未关闭；ULYS-235 Hooks 保持 `/settings/advanced/hooks` 高级设置并列 tab | 完成 Phase 9E-4C2 当前目录 migration/read/recheck 切片 |


| v5.51 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.37、basic v5.33、Task DD v1.11；补充 Skill capability rows 拉取前的数据库侧 ≤384 KiB heap 预算，并同步 current catalog reader、C1/C2 实现状态与 fail-closed production gates；沿用 ULYS-235 `/settings/advanced/hooks` 高级设置并列 tab | 收敛 Phase 9E-4C2 内存预算和导航文档状态 |
| v5.52 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.38、basic design v5.34、Task SRS v0.3 与 DD v1.12；记录 C3 Run Profile/Task/Hook/RunEvent/BI/resource reservation 原子 writer、Project 跨 Worktree quota、REPEATABLE READ allocator epoch 并发保护、121 REST tests、最终 `cargo check` 与双会话 SQLSTATE 40001 smoke；说明最终测试复跑被 Windows linker LNK1104 阻断，以及仍关闭的 Runtime/catalog/Auth/target DB/BI gates；明确 maintenance cleanup 待装配、reservation maxima 不是实测值，并保留 ULYS-235 Advanced Settings Hooks tab | 完成 Phase 9E-4C3 Run 与资源 admission 代码切片 |
| v5.53 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.39、basic design v5.35、Task SRS v0.4 与 DD v1.13；记录 C4 typed dual-Profile one-time fence binding、domain-separated digest、Approved Launch Profile identity/Run Detail projection 和 fail-closed Runtime consumer contract；声明 production launch-profile authority、Runtime consume/OS spawn、reservation lifecycle、catalog/auth/target DB/BI 仍未完成；明确 Hooks 仍为 Advanced Settings 内容区并列 tab | 完成 Phase 9E-4C4 双 Profile spawn-fence contract 与 Run audit slice |
| v5.54 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.40、basic design v5.36、Task SRS v0.5 与 DD v1.14；核实现存 Local Runtime grant/profile/path/nonce 基础校验未消费 C4 fence；将 ULYS-235 精确记录为 Settings 主导航 Advanced Settings 父入口下 `/settings/advanced/hooks` 并列 tab；保留 C4 production Runtime/capability fail-closed 状态 | 用户重申 Hooks 属于高级设置选项卡，并要求保留既有导航层级与路径 |
| v5.55 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.41/basic design v5.37/Task DD v1.15；记录 shared typed fence、signature v2 与 v1 legacy grant compatibility、Runtime current-binding validation、同事务 nonce/fence consume、50,000 receipt cap/TTL cleanup 与当前定向验证；明确无 production caller、OS spawn/Reservation/BI，capability 保持关闭 | 完成 Phase 9E-4C5 Runtime fence consume foundation 并准备推送 dev 供用户预览 |
