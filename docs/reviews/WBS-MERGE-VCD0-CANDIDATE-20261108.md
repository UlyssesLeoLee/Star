# VCD-0 Vercel 详细设计与测试设计候选审查

**审查基线：** 本地 `dev@6ad1a00f746234d5e296f942cc67f20f917d1609`
**候选：** `codex/vcd0-canonical-detail-test-20261024@4f0ceae9818b7bd2b3b25d9164d6e43303121689`
**结论：** **Selective ADAPT / 来源提交 HOLD**。保留其安全和验收原则；没有候选提交适合直接摘取。候选分支及工作树不作修改。

## §0 目的

按用户确认的 Project/Run/Worktree 架构、Vercel 使用 Neon、Rust 1.99.1 基线、当前授权边界及事件驱动协作要求，审查候选 DD/TD 是否比当前 dev 更新或提供可独立集成的内容。本审查仅覆盖已登记文档，不运行测试。

## §1 候选范围与提交映射

以共同祖先比较（`dev...candidate`），候选领先 5 个提交，候选独有文件共 4 个：

| 提交 | 文件变化 | 审查结果 |
|---|---|---|
| `8019ec30` — add canonical detailed and test designs | 新增 Vercel DD、TD | 设计初稿；现已被 dev 中更新版替代 |
| `1e99a9f6` — prepare TD P1 remediation | 新增 TD 修复 brief | 一次性任务说明，不应并入规范 |
| `c7065825` — address VCD-0 gateway test review | 修改 TD，增加并修订 T-049 | 有可复用的测试门思路；当前 TD 已进一步收敛 |
| `2690a33a` — define VCD-0 DD remediation scope | 新增 DD 修复 brief | 一次性任务说明，不应并入规范 |
| `4f0ceae9` — close VCD-0 detailed design review gaps | 修改 DD，补充 FR-VC-009..012 Owner/数据/RLS/迁移映射及 fail-closed | 有价值的责任划分已出现在当前 DD，候选版本已过时 |

候选新增/修改的唯一规范路径是 `docs/detailed-design/vercel/DD-STAR-VERCEL-DEPLOY-001.md` 与 `docs/test-design/vercel/TD-STAR-VERCEL-DEPLOY-001.md`。候选头部仍标 DD v0.1 / TD v0.1，修订历史却分别到 v0.2 / v0.3；当前 dev 已有 DD v0.3、TD v0.4。候选工作树在审查时为干净状态；未对其执行写操作。

## §2 按设计状态分类

| 分类 | 证据与判断 | 处置 |
|---|---|---|
| **当前有效且已吸收** | 候选强调 Star Owner 保有授权/事实、BFF 不得成为第二授权或业务事实源、Preview 隔离、Cron 仅作 best-effort 唤醒、未知能力 fail closed。当前 DD v0.3 §1–§3、§5–§7 已保留并细化这些原则；§6 还规定每次 trigger/retry/resume 复验不可变 `run_as_actor_id`。 | 继续以 dev 现行 DD 为准，不回放候选提交。 |
| **可适配但不是候选独有** | 候选 DD v0.2 将 Owner、事务级 `SET LOCAL`/RLS、迁移身份、连接预算及未批准时禁用数据库/执行能力逐项映射（候选 §5.2）。dev DD v0.3 §5.2 已有相同责任和 fail-closed 表，且补上目标数据库作业授权与池预算证据要求。 | 后续只基于现行 Star owner 契约补证，不照抄旧映射或推定 Provider/endpoint。 |
| **过时或未覆盖最新决定** | 候选 DD/TD 以 Rust 1.99.0 为最低版本；用户已指定 1.99.1。当前 Vercel SRS v0.4、BD v0.3、DD v0.3、TD v0.4 也仍写 1.99.0，故是当前文档的同步缺口，不是合并候选可解决的问题。Vercel DD §5.2 仍把具体 Provider 标作 UNKNOWN；用户已选择 Vercel 使用 Neon，但该选择不证明 Neon 项目、区域、角色、Owner API、RLS 或连接池已落地。 | 新的 Vercel 文档同步应将 Neon 记为目标 Provider/profile、将 Rust 1.99.1 记为基线，同时保留实际运行能力为 NOT_RUN/UNKNOWN，直到有证据。 |
| **与当前架构不一致或仍未对齐** | 当前 Vercel BD §1.2 已明确 hostless Engineering Run 与可选远端元数据；当前需求 WTG-027 与 WLD-1 详细设计规定 Star 原生 Project owner、Project 关联 0..N Repository、retiring 冻结新引用并先迁移/退役 Run/Worktree 引用。候选没有提供这些决定的完整映射。 | Vercel SRS/BD/DD/TD 后续须引用现行 Project/Repository 与 hostless Run 契约；不可将远端 Branch/Repository 提升为 Run 身份前提。 |
| **协作验收缺口** | 当前需求已规定持久事件驱动推送、语义重叠立即冲突、草稿保全及对方修改可读。候选 T-049 只覆盖通道连接/授权/撤权/恢复，未覆盖事件推送、双方草稿与冲突可见性。通用 Canvas 测试设计已有 WLD-7 计划用例，但 Vercel/Neon 路径仍需按该设计单独验收。 | 在后续 Vercel TD 中关联 WLD-7 的双 actor、事件唤醒、草稿/冲突与 Neon 路径用例；不把连接成功等同协作通过。 |
| **需保持关闭的路由歧义** | 候选 DD §2.3 承认 SRS FR-VC-013 允许 Next Route Handler 转发 chat SSE，而候选 T-049 又要求通过 Star Gateway 测 chat SSE，候选内部仍不一致。当前 dev DD §2.3/§7 与 ADR 继续将此列为未决契约，并要求未批准前 unavailable；当前 TD v0.4 的 T-049 按批准路由验收。候选 TD 也缺少当前 TD 明列的 noVNC 通道。 | 不从候选选择 Gateway 或 Next 路由；先统一 SRS/ADR/BD/DD/TD 与真实 Gateway/session 契约，再启用。测试矩阵以当前 TD v0.4 为基线。 |

## §3 安全的选择与适配

1. 不 cherry-pick 五个候选提交，也不合并整支。DD/TD 是同一旧设计线中的连续文件版本，两个 brief 是 10 月一次性修复任务，没有可独立保留的实现或契约提交。
2. 以当前 dev 的 Vercel SRS/BD/DD/TD 及 `DD-WLD1-REPOSITORY-BOUNDARY-001.md` 为唯一改稿基线；同步 Neon 目标、Rust 1.99.1、Star 原生 Project owner、0..N Repository/解绑迁移、hostless Run、当前授权和 WLD-7 验收映射。
3. 保持 Owner/API/Neon/RLS/连接池、目标构建器、真实 Gateway 握手、chat SSE 路由、撤权和协作行为的真实环境验证未完成状态。静态设计不升级为部署或运行证据。

## §4 验证摘要与已知缺口

- 只读确认候选独有 5 个提交、4 个路径；当前规范版本为 SRS v0.4、BD v0.3、DD v0.3、TD v0.4。
- 未运行构建、测试或部署；遵从 brief 范围。当前 TD 的全部 Vercel 用例仍为 NOT_RUN，VCD-0 仍 OPEN。
- 待补：将 Rust 1.99.1 和 Neon 选择同步到 Vercel 文档；将 0..N/先迁移解绑及 hostless Run 映射到 Vercel 验收；增加 Vercel/Neon 上事件推送、冲突与双方草稿可见的验收追踪；先裁决 chat SSE 路径并批准真实连接契约。
- 本审查不更改候选分支/工作树、canonical 设计文件、实现、main 或远端。

## §5 决定

**Selectively adapt principles; HOLD all five candidate commits.** 当前 dev 已包含比候选更新的设计；本轮没有适合合入的候选文件或提交。后续若同步 Vercel 架构，应新建基于当前 dev 的文档 lane，仅按 §3 最小更新，完成独立审查后再决定是否集成。VCD-0、Vercel hosted/runtime、Neon/Owner/RLS、Gateway 和协作验收均不得标为完成。

## §6 审查签署

| 角色 | 结论 |
|---|---|
| 架构 / 独立审查 | Mavis 接手审核 — ADAPT 原则，HOLD 候选提交 |
| 范围与保留 | 仅审查指定候选 DD/TD；候选与其他工作树保持原样 |
| 验证责任 | 无测试或运行环境结论；后续执行 owner 按批准 WBS 提供实证 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-11-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 对 VCD-0 候选 5 个提交及 DD/TD 与当前 dev、用户决策进行定向审查；记录选择性适配/HOLD 结论、当前同步缺口和未验证门禁。 |
