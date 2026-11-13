# WBS-VCD0-CURRENT-DEV-INTEGRATION-REVIEW-20261101

**状态：ADAPT — 不接受 pinned candidate 作为当前 canonical VCD-0 文档包。** 本结论只评审设计与测试文档，不是合并授权；当前 dev 已有更新版本和新的依赖文档，后续若继续 VCD-0，应以当前 dev 为基线修订并重新审阅。

> **审查快照：** 下文固定比较的是候选 `4f0ceae9818b7bd2b3b25d9164d6e43303121689` 与当时的 dev `a272e9794e1a1975781720c9b3775c914f47f546`。本次集成前，dev 已推进至 `85ef753d6d20d2d156569597f114bb3974be3864`，其中含 W0 Web、Desktop 与 Canvas 的 Git-tag 版本绑定实现。ADAPT 结论只保留为已记录的候选审查决策；当前 canonical Vercel DD/TD 如需继续实施仍须以最新 dev 重新核对并更新。

## §0 目的

独立评估分支 codex/vcd0-canonical-detail-test-20261024 的精确候选提交 4f0ceae9818b7bd2b3b25d9164d6e43303121689，是否可纳入当前 dev 基线 a272e9794e1a1975781720c9b3775c914f47f546 的 Vercel 设计/测试与 Star Stable Release 文档集。评审只读；未修改候选、要求/设计/WBS，也未执行 cherry-pick、merge 或 hosted validation。

## §1 基线与候选范围

| 项目 | 固定证据 |
|---|---|
| 候选 | branch codex/vcd0-canonical-detail-test-20261024，HEAD 4f0ceae9818b7bd2b3b25d9164d6e43303121689；其父提交为 2690a33a3d953fb43bb893ccf3ac5f4ccd62b5bb。 |
| 当前 dev | a272e9794e1a1975781720c9b3775c914f47f546；与候选的 merge-base 为 890d570e657c125a7786acf3faca4415311b8dde。 |
| 候选拥有的文档 | docs/detailed-design/vercel/DD-STAR-VERCEL-DEPLOY-001.md；docs/test-design/vercel/TD-STAR-VERCEL-DEPLOY-001.md。 |
| 当前 dev 的文档漂移 | 当前 dev 已有 Vercel ADR、246 行 Vercel basic design 与 149 行 Stable Release WBS；候选版本中未包含这些文件。当前 DD 为 v0.3、TD 为 v0.4；候选两个文件头部仍写 v0.1。当前 dev 还把 noVNC 与 chat SSE 的路由/Origin 负向验收门补进 TD-049。参见两版本间针对上述路径的 git diff。 |

## §2 验收矩阵

| 审查问题 | 结论 | 文件与章节证据 |
|---|---|---|
| Project → Run → Worktree → Branch、Star 授权与 provider linkage | 部分符合。候选 DD §1.2/§7 保留导航和 Star 授权权威（DD 行 21–33、166–170），TD-014/016 检查授权读取和越权拒绝（TD 行 101–104）。但它未把当前 Project Owner 的 0..N repository、无 Repo/Host/Branch 的 hostless Run、远端关联可选，以及解绑前迁移/退役串进 Vercel acceptance。当前 SRS-WLD1 §4.1/§5/§7（行 46–53、66–79、96–107）与 BD-WLD1 §3.4（行 79–85）定义该模型；当前 Vercel BD §1.2/§5（行 89–90、133–140）明确 provider 专属操作须另行授权、无远端宿主关联的 Run 仍可管理。WBS-VCD0-CONTRACT-GAPS §1 P1（行 22）要求将既有 WLD1 AC 接入 Vercel trace；候选 T-014 未覆盖这些场景。 |
| Vercel、Neon 与 Multipass/K3s 部署边界 | 部分符合。候选把 Vercel 限定为 Web/BFF 并把 Agent、Hook、Worktree filesystem、Multipass/K3s 留在 Star execution plane（DD 行 27–33；TD 行 21–25、41–45）。候选没有提 Neon；其 DD §5.2 将具体数据库 provider、Owner 映射、迁移身份与连接预算留为 UNKNOWN/fail-closed（行 132–143）。当前已接受的目标选择是 Vercel profile 使用 Neon、K3s profile 使用 Multipass VM PostgreSQL，但当前 Vercel SRS §2.2/§8 GAP-06 仍记为待官方复核（行 47、251），当前 basic design §6.3 也保留选择门。须同步目标选择与未验证的平台/Owner 事实，不能把 Neon 写成新的 Owner 或 Vercel 直连权限。 |
| 用户触发的 push 协作、双边草稿、冲突及 counterpart diff | 缺失。候选将长连接放在 Star Gateway，并为通道授权/撤权/故障恢复设计 T-049；但候选 DD §2.3（行 62–66）与 TD §4.7/T-049（行 161–167）没有 owner commit 唤醒、正常路径无固定轮询、双方持久草稿、共同 conflict_id、按参与者授权读取 bounded diff、重连 cursor catch-up 的验收。对候选两份文件的定向关键词检索未发现 conflict、counterpart、diff、event-push、owner-commit 或 cursor-catch-up 命中；命中仅为文档 Draft 状态与函数 long-poll 限制。当前 collaboration spec INV-CB-07/08（行 62–63、§13）以及 WBS-W16-EVENT-PUSH-AUDIT §1/§3（行 18、24、26、30、38、52、57–62）将这些定义为现行契约，并确认现存固定轮询与持久跨实例协作仍有缺口。当前 WBS-VCD0-CONTRACT-GAPS §1 P1（行 24）明确指出 T-049 不等于 durable collaboration acceptance。 |
| Security、授权、migration/RLS、事务与幂等 | 设计门禁较完整，验收计划未执行。DD §3、§5.2、§8–§10 保留当前授权、预览隔离、secret 与 fail-closed 边界。TD T-014–019、T-037–042 覆盖授权拒绝、Owner receipt、SET LOCAL/RLS、迁移边界、Outbox 与未知写入后的原幂等键查询（行 101–106、143–148）。但需将其和当前 WLD1 Owner/provider contracts 及 Neon profile 对齐，并将 Inbox 重复投递/恢复与 durable collaboration relay 明确连到现有测试，不得用 Gateway 建连成功代替 Owner 持久事件证据。 |
| 资源预算、rollback、Hosted proof、能力状态 | 设计状态正确保持条件化。TD T-028–036 计划测量 Rust runtime/包体/时延/资源（行 127–135），T-043–048 计划验证 Preview 隔离、promotion、回滚、观测及 SBOM/license（行 154–159），所有用例初始为 NOT_RUN（行 5、31、77）。报告不得把这些计划写成实现或平台通过证据。 |
| Stable Release 范围、版本与 MIT | 未见直接矛盾，但候选未追踪到当前稳定版门禁。当前 WBS §已确认发行决策规定 Desktop、Web/Vercel、三份 Canvas 镜像，Git tag 唯一版本源 v1.0.6，Star 自有项目选 MIT（行 12–18）；W0、W1 与 W6 的版本、license/NOTICE、Hosted Vercel deployment/consumer/rollback 状态仍 PARTIAL/BLOCKED/OPEN（行 28–29、34）。候选的测试/设计不能关闭这些门，也未绑定 tag-derived Web version、stable artifact、MIT metadata 或真实 Vercel deployment。Multipass guest image 明确不属于公开发行制品，Multipass/K3s 保留为测试/执行基础设施。 |

## §3 必须调整后再评审

1. 以 a272e9794e1a1975781720c9b3775c914f47f546 的 SRS、ADR、basic design、DD/TD 与当前 Stable Release WBS 为基线；修正候选 stale v0.1 headers、旧 WBS pointer 和 traceability。保留当前 dev 已加入的 noVNC、chat SSE 最终路径未定以及 Origin 负向检查必须执行的门禁。
2. 按 WBS-VCD0-CONTRACT-GAPS §1 P1 将现有 WLD1 FR/AC 映射进 Vercel 授权读取 acceptance；补测 hostless/no-Repo Run、同 Project 多 repository、可选 provider linkage、repository retiring/migrate-before-unlink 与无关联空态。只引用既有 WLD1/Vercel ID，不新增需求编号。
3. 将 Neon 写为用户选择的 Vercel hosted database profile；单独记录平台产品事实复核、Star Owner/API 实际运行位置、是否经 Owner API、roles/grants、事务 SET LOCAL/RLS、pooler/连接预算与独立 migration 身份仍未验证。任何门未证实前继续禁止 Vercel 直接业务 DB 写入、DDL、Owner 绕过或执行能力。
4. 把 Vercel collaboration UI/transport 明确绑定到现行 Star resource-owner event contract。明确用户提交只通过 Owner transaction 产生 revision/draft/conflict/Outbox 事实；实时 wake-up 是有界 push；固定周期轮询不是正常路径；重连/gap 才做授权 cursor catch-up。将 durable event relay、双边 draft recovery、相同 conflict_id、参与者范围授权的 counterpart diff、revoke/restart/gap 作为协作验收引用。保持 T-049 只验证 Vercel 通道握手/Origin/授权/撤权/可用性，不冒充端到端协作验收。
5. 给出 owner-local Inbox 重复投递、处理后确认、崩溃恢复、durable cursor 与幂等 receipt 的可观察证据要求；Vercel transport response 不等于 Outbox/Inbox 已 durable commit 或业务完成。
6. 在 Stable Release trace 中只引用当前 W0/W1/W3/W6/W9 gates：版本只从 Git tag v1.0.6 派生，项目许可证决策是 MIT，真实 Vercel Hosted run、consumer、artifact/provenance、rollback 未证明则 release 保持 OPEN。不得新建第二版本源或把设计态写成 release-ready。

## §4 可保留的设计内容

- Vercel 作为受保护 Web presentation/BFF surface；Star Owner 保持授权、业务事实、审计、持久 receipt、Outbox/Inbox 和 Schedule 真相。
- 401/403 区分、当前权限复验、写请求幂等恢复、Preview 与 Production 隔离及 secret 不进入客户端/log/trace。
- Cron 仅 best-effort wake-up，不占资源、不创建 Run/Executor；Multipass/K3s、sandbox、Hook、Worktree 与 Agent execution 留在 Star 自有面。
- 数据库/Rust runtime/hosted deployment 未验证时 fail closed；所有 test case 保持 NOT_RUN，直到相应 owner、账户、fixture 与平台证据到位。

## §5 决策与集成边界

**Recommendation: ADAPT.** 候选的安全边界和分层验收框架有复用价值，但 4f0ceae 的文档头部/trace 落后于当前 dev，且缺少现行 WLD1、Neon profile 与 W16 push-collaboration acceptance。不要把该 pinned snapshot 当作可直接合入的 canonical 文档；任何后续修改须在已登记 lane 中按当前 dev 同步后另行审阅。此审查不执行、不授权 merge/cherry-pick，也不代表 Stable Release W6/W0–W9 关闭。

## §6 证据与限制

本评审只对固定 Git 对象中的文档做定向读取与差异检查。未运行 Rust/Node tests、build、数据库测试、Vercel/Neon/K3s/Multipass 操作、GitHub hosted workflow、部署或发布。候选文件的关键词搜索仅能说明限定两份文档未明确写出对应协作验收，不代表全仓负面证明。CodeGraph 本轮未提供 generation 与 coverage API；没有声称图索引完整。CodeRabbit CLI shim 返回 Permission denied，未产生独立审阅结果。

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 固定 Vercel candidate `4f0ceae` 与当时 dev `a272e979` 的只读差异审查，决定 ADAPT | WBS VCD0 integration review |
| v0.2 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 标明当前 dev 已前进到 `85ef753d`，此报告的候选比较仍是历史快照，后续实施以最新 canonical 文档重新审查 | W0 版本绑定 lanes 已集成 |
