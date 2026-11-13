# PHASE-VCD0-DD-REVIEW-R2-20261024

> **审查结论：BLOCKED（暂不纳入 canonical design set）**
> **范围：** 对候选 `docs/detailed-design/vercel/DD-STAR-VERCEL-DEPLOY-001.md` 做只读独立审查。
> **候选版本：** `v0.1`，来源 worktree HEAD `c7065825a987090be1801eff6120c3dc8ec616b1`；文件 SHA-256 `C63528F1EA632CB861B05A8AE25835B647497A92E4D718E6E920C2E2AF75F3DC`。
> **对照：** 当前 dev 的 SRS v0.4 / WBS VCD-0 行、候选 ADR 与基本设计、VCD0-DD 兼容性审查、测试设计 v0.3。
> **状态：** VCD-0 保持 OPEN；所有部署、账号、runtime、生产行为及测试均 NOT_RUN。

## §0 目的

判断候选详细设计能否作为 VCD-0 设计集的集成输入，并列出阻止集成的具体差异。本报告只评审文档；不修改候选设计、SRS、基本设计、ADR、WBS 或产品代码。

## §1 发现矩阵

| ID | 严重度 | 路径与证据 | 发现及处理要求 |
|---|---|---|---|
| DD-R2-01 | **P1** | 候选 DD §5.2，`DD-STAR-VERCEL-DEPLOY-001.md:124-130`、追踪 `:228-231`；当前 SRS `SRS-STAR-VERCEL-DEPLOY-001.md:167-170`；dev WBS `WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md:56` | SRS 的 FR-VC-009..012 明确要求托管 PostgreSQL、事务级 RLS/连接池、独立迁移作业及连接预算；DD 则规定数据库归现有 Star Owner、Vercel 不得直连任意 DB，Provider/RLS/迁移流程仍未裁决。这个边界符合“不增加第二数据库权威”，但没有明确说明这些 FR 是由现有 Star Owner 的 PostgreSQL/迁移流程满足、被条件化，还是需要修订 SRS。仅将这些 FR 映射到 §5.2 和测试编号不足以形成可判定的设计追踪。**集成前**需明确：只允许现有 Star Owner 管理的存储及其迁移流程；若其无法满足 FR，列出需修改的 SRS/AC，而不得默认为 Vercel 新建数据库、Owner 或迁移权威。 |
| DD-R2-02 | **P2** | 候选 DD §7 与 §11，`:153-164`、`:227`、`:232`、`:261-262`；候选 TD v0.3，`TD-STAR-VERCEL-DEPLOY-001.md:163-167`、`:179-180`、`:205` | TD v0.3 的 T-049 已将 Star Gateway 握手、允许的 Preview Origin、当前 Project/Run/capability 授权、撤权行为及协议数据流/生命周期列为正向测试前置门，并覆盖 terminal/ARG/Canvas WebSocket 与 chat SSE。DD §7 只概述部分长连接留在 Star Gateway，未记录该未决契约，也没有列出 chat SSE 路径；§11 的 FR-VC-008、FR-VC-013 与 AC-VC-005/006 追踪均未包含 T-049。**集成前**需在 DD 明确引用真实 Gateway 契约；若尚不存在，则把 handshake/Origin/授权/撤权/生命周期列为 UNKNOWN/OPEN、保持相应能力关闭，并将 T-049 补入 FR/AC 追踪。不得在设计中臆造新的握手协议。 |

### 正向核对

- DD 明确保留 Star Owner、授权、持久回执、审计和 Outbox/Inbox 为业务权威；未引入第二 Owner、数据库权威、worker/scheduler 或 VM 执行面。
- Cron 被限定为 best-effort 唤醒；每次 trigger/retry/resume 固定 `run_as_actor_id` 并重新授权；Run/Infrastructure Owner 的容量预留分属各自事务；Rust Runtime 未知能力保持 UNKNOWN。
- Preview 隔离、回滚后 Schedule 对账、有限请求/数据库预算、SBOM 与失败关闭边界均已写入。文档状态 Draft、VCD-0 OPEN、AC 未验证的表述一致；未发现竞争产品对标或仿制文案。

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| 文档与追踪矩阵只读核对 | 完成；发现 DD-R2-01、DD-R2-02 |
| 报告 whitespace/diff 检查 | 写入后执行；只检查本报告 |
| 测试、编译、静态质量门 | 未运行（本 brief 禁止） |
| Vercel 账号、Preview、部署、Rust Runtime、数据库、生产行为 | 未验证；均 NOT_RUN |

审查使用的 dev checkout 为 `c66ee1e96ac17bae3019ec91063a998c543069cd`；候选 ADR/BD 与 TD 均只读。对这些静态材料的核对不构成运行或部署证据。

## §3 已知缺口

1. DD-R2-01 未解决前，无法证明 FR-VC-009..012 与现有 Owner/数据库权威兼容。
2. DD-R2-02 未补齐前，DD、TD 对长连接的实现边界及验收追踪不完整；T-049 必须保持 NOT_RUN。
3. 目标账号/套餐、真实宿主 session 与 Run 授权映射、Preview 隔离、Rust Runtime 功能与资源、PostgreSQL/RLS、Owner Schedule intake、成本及运维证据仍未验证。
4. 本审查只覆盖指定候选详细设计，不代表完整 VCD-0 套件终审。

## §4 子代理失败接手清单

本审查 lane 未派发子代理；无失败任务需要接手。候选设计文件及其他 lane 产物未修改。

## §5 守门规则

1. VCD-0 保持 **OPEN**；本报告不是批准或关闭证据。
2. DD-R2-01、DD-R2-02 处理并复审前，详细设计不得作为最终 canonical VCD-0 输入。
3. 所有部署、账号、数据库、Rust Runtime、Preview 与生产行为保持 **NOT_RUN**；不得从文档审查推导能力已实现。
4. 不增加 Owner、数据库/迁移权威、后台 worker/scheduler 或替代执行面。
5. T-049 仅使用已核实的 Star Gateway 契约；未知握手、授权或 Origin 策略时 fail closed。
6. 本 lane 未运行测试、编译或部署验证；未读取、编辑或清理 `D:\Star`。

## §6 审查栏

| 角色 | 结论 |
|---|---|
| 独立详细设计审查 | **BLOCKED**；存在 1 项 P1、1 项 P2，见 §1 |
| 架构 / SRE Lead / 平台 / 评审主持 / PM | 本报告范围未代替这些角色的最终整套 VCD-0 审批 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容与触发 |
|---|---|---|---|
| v0.1 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 根据已提交 review brief 对 VCD-0 候选详细设计进行独立只读复核；记录数据库需求归属与 T-049 追踪缺口；无测试或运行态声明。 |
\n