# PHASE-WBS-W11A-HOSTLESS-BINDING-20261027

> 状态：**Partial / Open**。本报告记录 W11-A 的文档设计增补及后续审查修订，不代表代码、数据库迁移、WBS 或详细基础设施设计已闭合。
> 基线：dev `4f629ce5103fb4a9fb67a775341475ef66bbf2ea`。
> 工作分支：`codex/wbs-w11a-hostless-binding-20261027`。
> 范围：只同步已登记的 SRS、BD、DD 与两份 W/T/M 文档；未触及 WBS 和用户正在移动的 `DD-LOCAL-INFRASTRUCTURE-001.md`。

## §0 目的

把 hostless Engineering Run 和可选基础设施放置纳入 WLD1 的一致设计：Run identity 与创建、授权、列表不依赖远端 Cloud Branch；Repository binding 和 Infrastructure placement 相互独立；只有显式选择 placement 才创建 RunInfrastructureBinding。V3 demand 同时定义 Schedule 与手动 TaskExecutionRun 的 tagged trigger origin、attempt 唯一性与回放；兼容仍需 Branch 且 Schedule-only 的 V2 契约。

本阶段为文档设计，不更改 Rust、SQL、迁移、API 实现、WBS、目标数据库或 VM。因 `DD-LOCAL-INFRASTRUCTURE-001.md` 和 WBS 同步门未完成，W11-A 明确保持 Partial / Open。

## §1 改动矩阵

| 文件 | 变化 | 结果与边界 |
|---|---|---|
| `docs/requirements/SRS-WLD1-REPOSITORY-BOUNDARY-001.md` | v0.4；明确 Run identity、可选 bindings、V3 Schedule/Manual origin、固定 execution actor、attempt 唯一性与 V2/V3 replay 目标 | 需求设计已同步；V3 未实现，TD-WLD1 尚未同步 |
| `docs/basic-design/BD-WLD1-REPOSITORY-BOUNDARY-001.md` | v0.4；补充 owner、hostless placement、tagged origin、owner-derived actor 授权、attempt/digest/replay 及缺口 | 基本设计已同步；仍需 DD-LOCAL/WBS/TD 对齐 |
| `docs/detailed-design/DD-WLD1-REPOSITORY-BOUNDARY-001.md` | v0.4；补充 API、V3 Schedule/Manual variants、execution actor、optional attribution、attempt identity/uniqueness、digest/replay 和 additive migration | 详细设计目标已写；没有 SQL/API 实施 |
| `docs/data-design/phase9f4d-run-owner-wtm.md` | v1.10；增加 V3 Schedule/Manual Outbox/Inbox、owner-derived actor 与 attempt/replay 设计目标 | 现有两张 T 表分类和历史证据保持不变 |
| `docs/data-design/phase9f4d-infrastructure-owner-wtm.md` | v0.16；增加逻辑 V3 Schedule/Manual schema target、execution actor 与 W/T 目标分类 | 当前 20 张表的 4 W / 10 T / 6 M 与旧验证快照保持不变；新表未创建 |
| `docs/reports/PHASE-WBS-W11A-HOSTLESS-BINDING-20261027.md` | 新增本阶段报告 | 仅报告已登记 lane 结果 |
| `docs/briefs/WBS-W11A-HOSTLESS-BINDING-20261027.md` | 未修改 | 保留 dispatcher 生成的原始 brief |
| WBS、`DD-LOCAL-INFRASTRUCTURE-001.md` | 未修改 | 分别由并行 W14 lane 所有；用户 main worktree 的文件移动/删除内容受保留门禁保护 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| SRS/BD/DD/W/T/M 版本、Run identity、placement、V2/V3 术语交叉核对 | 已按文档 diff 静态检查；V3 均标为设计目标，当前实现状态明确为未验证 |
| 当前 V2 source contract | parent 的 CodeGraph 定向查询指出 Run owner demand、V2 snapshot、Host demand、consumer 与 Outbox 均要求 Branch，且 V2 validator 只接受 `run_origin=schedule` 并校验 Rule/revision/occurrence；直接核对 migration 确认 Outbox/Inbox 的 `branch_id UUID NOT NULL`、`schema_version = 2`、非零 Branch 与复合 FK。Run owner 的 SELECT projection 暴露 `automation_rule_id`、`automation_rule_version`、`automation_occurrence_id`。CodeGraph 覆盖不完整，不作全仓穷尽声明 |
| Trigger/attempt 文档一致性 | 五份设计/W-T/M 文档均包含 `trigger_origin`、Schedule rule/revision/occurrence、Manual command、owner-derived `execution_actor_id`、positive attempt number、Schedule-only V2 与 optional schedule attribution；Report guard count 为 17 |
| 文档差异检查 | `rtk git diff --check`：通过（检查 tracked 文档 diff）；PowerShell 静态检查确认 manifest 内 7 份文件无行尾空白，报告 §0–§7 全部存在 |
| Markdown / 链接检查 | 对目标文件做静态阅读与版本/字段交叉核对；未运行专用 Markdown linter 或完整链接检查 |
| Cargo fmt/check/clippy/test、前端测试/E2E | 未运行；本 lane 明确禁止运行这些门禁 |
| SQL migration、PostgreSQL/RLS/grants、Neon、Multipass VM、K3s、运行时与性能 | 未运行；不构成通过声明 |

验证边界：本次只证明文档文本与 diff 可检查；不证明 API、worker、数据库结构、授权、Outbox/Inbox consumer、容量保留、K3s readiness 或任务执行闭环已实现。

## §3 已知缺口

1. `docs/detailed-design/DD-LOCAL-INFRASTRUCTURE-001.md` 仍待同步。MAGOS 登记时发现 `D:\Star` main worktree 对该文件存在用户未提交的移动/删除；本 lane 按保留要求没有读取、复制、覆盖、暂存或清理它，也未写入新目标路径。
2. WBS 文件由并行 W14 lane 持有，本 lane 未修改。其 W11-A 任务状态、依赖、验收准则及本次 V3 决策需在该 lane 完成后由 root 协调同步。
3. `TD-WLD1-REPOSITORY-BOUNDARY-001.md` 未在登记清单中，仍为 v0.2；V3/V2 兼容、迁移失败/隔离、跨版本幂等及 hostless API 验收需后续纳入测试设计。
4. 当前实现仍是 V2：已有 Rust/SQL 路径要求 `branch_id`；本 lane 未修改代码或迁移。需要新增 owner 端 V3 wire、存储、canonicalizer、reader/writer、RLS/grants 与兼容部署流程。
5. V2 Run-owner demand path 对 `run_origin=schedule` 作硬校验；Manual TaskExecutionRun demand 必须 fail closed，直到 V3 两端与持久化门部署完成。当前物理字段使用 `automation_*` 命名，V3 canonical domain uses `schedule_rule_*` names。
6. V3 关系名是数据设计目标，不是已审定 DDL。落地前需核对完整 FK/owner/RLS/grant/retention 闭包，并明确 Schedule occurrence、Manual command、attempt、demand、reservation/result 的一致性；不得仅按本文名称直接生成生产 migration。
7. V3 canonical JSON/CBOR 的规范化字节格式、optional/null/omitted 规则、Unicode 与数值边界、digest 算法和密钥/签名需求仍待接口/安全设计冻结。`execution_actor_id` 必须由 owner 从固定 Schedule Rule run-as 或 Manual command 认证上下文解析并在每次 attempt 重验；schedule attribution 只作 causal provenance，不改变 Manual actor/授权。
8. Migration 只能迁移由权威 Project/Repository owner 证明且当前有效的远端 metadata。模糊、撤权、失效或无法证明的旧记录必须隔离；不能推断默认 Branch、Schedule origin 或生成 sentinel。
9. Neon 与 Multipass VM PostgreSQL 的目标数据库 RLS/grants、并发撤权、owner Outbox/Inbox 与运行验证仍未执行；测试阶段 Multipass inventory 上限仍为 2，本文没有启动或清理 VM。
10. 没有验证 Run/Infrastructure worker、双层容量预留、K3s API/node/CRI、隔离 Runtime sandbox、Schedule/Manual execution、BI/Benchmark、取消/恢复、原生 Hook 或可用任务卡 CLI 的产品闭环。

## §4 子代理失败接手清单

| 项目 | 状态 | 交接动作 |
|---|---|---|
| 本 W11-A 文档 lane | 完成本登记文档增补；未提交 | root 按 diff 审查后决定是否纳入 dev |
| WBS 文件 | 由并行 W14 remediation lane 持有；不是本 lane 的失败项 | W14 完成后由 root 对齐 W11-A 状态、验收和依赖，不要覆盖其工作 |
| `DD-LOCAL-INFRASTRUCTURE-001.md` | 因用户 main worktree 的未提交移动/删除而被排除 | 保留原状态；待用户移动完成后由 root 安排单独同步与一致性审查 |
| 代码/SQL/数据库验收 | 本 lane 未派发、未实施、未运行 | 按 WBS 后续阶段派发独立 implementation 与 verifier；V2 replay compatibility 必须先覆盖 |

本 lane 未创建子代理，因此没有子代理 RPC/实现失败需要回收；不把并行 lane 的 WBS 工作当作本 lane 的产出。

## §5 守门规则

1. 仅在已登记隔离 worktree 中工作，不触碰 `D:\Star`、dev checkout 或其他 worktree。
2. 只编辑 brief 清单内文档和本阶段报告；不得改 WBS、`DD-LOCAL-INFRASTRUCTURE-001.md` 或新增目标路径。
3. 不提交、不 rebase、不 merge、不清理分支/worktree。
4. Run identity 固定为 tenant + Project + Run；创建、授权、列表不查询远端 Branch。
5. Remote Repository/provider/Branch 是可选版本化 provenance，不作为 Run 父级、授权前提或本地 ref 替代。
6. RunRepositoryBinding 和 RunInfrastructureBinding 分离；两者都不从 Project/default Repository 自动推导。
7. 未选择 placement 时不创建 Infrastructure binding 或 capacity demand；不以 nil UUID、空值伪装或虚构 Branch 作 sentinel。
8. Placement 必须固定 Profile revision/digest、HostEnvironment ID+generation、K3s cluster ID+generation、namespace、runtime profile 与预算。
9. 需要指定环境的执行在每次准入、retry/resume 均须重新授权、复验 binding 与 readiness；任一未知或缺失都 fail closed。
10. Worktree local checkout ref 仅属于 Worktree scope，不能升级成 Run identity 或远端 Branch。
11. V3 使用独立 schema/wire/digest domain 和 version-scoped idempotency；trigger tag、required origin identity、owner-derived `execution_actor_id`、attempt identity/number、optional schedule attribution/remote provenance presence 与安全敏感 placement 纳入规范化摘要。
12. V2 历史只按原 decoder/canonicalizer/digest/Branch/Schedule origin replay；V2 Manual fail closed；禁止清空、补造或迁移重哈希，也不跨 V2/V3 dedupe。
13. V3 Schedule key 是 rule ID + immutable revision + occurrence；Manual key 是 command ID；Manual schedule attribution 仅是被验证的历史因果记录且不改变 origin/authorization。nil Schedule UUID 禁止。
14. Run 与 Infrastructure 各守 owner-local Inbox/Outbox 事务；Infrastructure 不直接写 Run 表，Run Inbox 提交后才确认运输层消息。
15. V3 每个 attempt 使用唯一 TaskExecutionRun ID 与正数 attempt number；same attempt/same digest 重放 receipt、digest 冲突拒绝、新 attempt 使用新 identity 并保留根 origin。
16. 保持 tenant FORCE RLS、least-privilege grants、append-only Transaction、短 TTL Work/清理证据边界；保留 Multipass test cap=2 且不 purge；设计文本不能替代 DB/VM 验收。
17. 不把文档静态检查描述为编译、测试、数据库、VM、运行时、性能或商业产品闭环通过；历史验证结果不得升级为本次验证。

## §6 签字栏

| 角色 | 审阅责任 | 状态与边界 |
|---|---|---|
| 架构 | Mavis 接手 agent 终审 | 本报告交 root 审阅；W11-A 未关闭 |
| SRE Lead | 核对部署、恢复、资源与可观测性设计 | 本 lane 未执行部署或运行验收；不表示基础设施批准 |
| 平台 | 核对 owner、schema、RLS、版本与迁移边界 | 本 lane 只写设计文本；V3 DDL/RLS/grants 未实施 |
| 评审主持 | 核对需求、基本设计、详细设计、W/T/M 一致性 | 当前登记文档已同步；DD-LOCAL/WBS/TD gate 仍开放 |
| PM | 核对范围、WBS 状态与用户目标 | 由 root 在 WBS 同步后终审；本报告不将阶段标记为完成 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.2 | 2026-10-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 回应审查，明确 V3 Schedule/Manual tagged origin、owner-derived execution actor、optional schedule attribution、attempt 唯一性与 digest/replay；记录 V2 Schedule-only 门及未验证边界；阶段保持 Partial / Open。 | root 对 occurrence trigger 来源、V2 run_origin 与 manual TaskExecutionRun 范围的交叉审查。 |
| v0.1 | 2026-10-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 W11-A hostless Run/可选 placement 文档增补、V2/V3 基础兼容决策、未验证边界及 DD-LOCAL/WBS/TD 遗留门；阶段保持 Partial / Open。 | root 派发的 W11-A hostless Run / 可选 Infrastructure placement 文档同步任务。 |
