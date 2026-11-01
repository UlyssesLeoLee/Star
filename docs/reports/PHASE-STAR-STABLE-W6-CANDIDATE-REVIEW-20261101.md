# PHASE-STAR-STABLE-W6-CANDIDATE-REVIEW-20261101

**结论：ADAPT。** 此阶段仅完成固定提交的只读文档验收审阅；没有修改或集成候选，不构成 hosted release acceptance。

> **基线更新（2026-11-01）：** 本报告的候选差异审查快照是 `dev@a272e9794e1a1975781720c9b3775c914f47f546`。当前本地 dev 已推进至 `85ef753d6d20d2d156569597f114bb3974be3864`，并集成 W0 Web、Desktop 与 Canvas 的 Git-tag 版本绑定实现。该集成没有提供 Neon/Owner 运行证据、Vercel hosted deployment、consumer/rollback 或当前 canonical DD/TD 的新一轮逐项审查；本报告的 ADAPT 决策维持。

## §0 目的

按 brief docs/briefs/W6-VCD0-CANDIDATE-REVIEW-20261101.md，对 Vercel canonical detail/test candidate 进行独立只读审阅，并形成可追溯的 WBS 决策。评审目标为分支 codex/vcd0-canonical-detail-test-20261024 的精确 HEAD 4f0ceae9818b7bd2b3b25d9164d6e43303121689，与当前 dev 基线 a272e9794e1a1975781720c9b3775c914f47f546 比较。

## §1 评审矩阵与文件范围

| 范围 | 结论 | 证据 / 后续要求 |
|---|---|---|
| 被审候选 | 只读 | docs/detailed-design/vercel/DD-STAR-VERCEL-DEPLOY-001.md 与 docs/test-design/vercel/TD-STAR-VERCEL-DEPLOY-001.md，均取候选 SHA 4f0ceae9818b7bd2b3b25d9164d6e43303121689，不以 branch label 代替对象核验。 |
| 基线漂移 | 不可 drop-in | 当前 dev 已补 ADR、246 行 basic design 与 Stable Release WBS；dev DD/TD 为 v0.3/v0.4，候选头部仍写 v0.1。dev TD-049 增加 noVNC 与 chat SSE 路由/Origin 负向验收门。候选 revision history 和正文未跟上当前 canonical 文档。 |
| Project/Run 模型 | 部分保留 | 候选 DD §1.2/§7 保留 Star 授权和 Project → Run → Worktree → Branch 语义（行 21–33、166–170），TD T-014/T-016 覆盖授权读取与跨范围拒绝（行 101–104）。需补 WLD1 0..N repositories、no-Repo/Host/Branch 的 hostless Run、provider optionality 与 migrate-before-unlink。当前 SRS-WLD1 §4.1/§5/§7（行 46–53、66–79、96–107）与 BD-WLD1 §3.4（行 79–85）定义该模型；当前 Vercel BD §1.2/§5（行 89–90、133–140）明确 provider 专属操作另行授权、无远端关联的 Run 仍可管理。WBS-VCD0-CONTRACT-GAPS §1 P1 指明 T-014/AC-VC-005 缺少这些链接（行 22）。 |
| Neon 与 Multipass/K3s | 部分保留 | 候选 DD §1.2/§5.2 将执行面留给 Star，并把 DB/Owner/RLS/迁移映射 fail closed（行 27–33、132–143）；TD 明确 Multipass/K3s 是 Star 路径而非 Vercel 执行面（行 21–25、41–45）。文档完全未提 Neon。当前 VCD0 contract-gap review 将 Neon 选为 Vercel profile 的目标、K3s 使用 Multipass VM PostgreSQL，但 Owner/API/roles/RLS/pooler/migration mapping 仍开（该 review §1 P1，行 21）。当前 Vercel SRS 的 GAP-06 仍待官方复核（行 47、251）；需同步“目标已选”与“运行事实未验证”。 |
| Push collaboration | 缺少 Vercel 侧业务验收 | 候选 DD §2.3 与 TD T-049 验证 Star Gateway 长连接、身份、Origin、撤权与恢复（DD 行 62–66；TD 行 161–167），但没有 owner-commit push、无固定轮询、双方持久草稿、共同 conflict_id、participant-authorized counterpart diff 或 cursor catch-up。当前 collaboration spec INV-CB-07/08 和 §13 定义上述契约；WBS-W16-EVENT-PUSH-AUDIT §1/§3 与 WBS-VCD0-CONTRACT-GAPS §1 P1 说明 T-049 不等于 durable collaboration acceptance（WBS VCD0 review 行 24）。需引用既有协作验收，不能扩展 BFF 成业务队列。 |
| Security、DB/事务、资源、rollback 与测试状态 | 框架较完整，仍是计划 | TD T-004–013 覆盖 BFF/secret/SBOM；T-014–019 覆盖 session/授权；T-037–042 覆盖 owner receipt、RLS、migration、Outbox 与幂等恢复；T-032–036、T-043–048 覆盖资源/运行限制、Preview、rollback、观测与许可。文档声明所有用例 NOT_RUN（TD 行 5、31、77）；Outbox/receipt 计划不覆盖完整的跨实例 collaboration relay 与两边草稿闭环。 |
| Stable Release 约束 | 未见直接冲突，不代表门关闭 | 当前 Stable Release WBS §已确认决策为 Desktop、Web/Vercel、三份 Canvas 镜像，Git tag 唯一版本源 v1.0.6，Star 自有项目许可证 MIT（行 12–18）。W0/W1 与 W6 仍未关闭；W6 要求真实 Vercel deployment、consumer 与 rollback 证据（行 28–29、34）。候选不得把设计/静态检查写为 hosted proof。Multipass guest image 不在公开发行范围；Multipass/K3s 留作测试/执行基础设施（WBS 行 15、28）。 |

## §2 验证摘要

- 核实候选 branch HEAD、commit parent、当前 dev HEAD 与 merge-base；候选 SHA 与 brief 相同。
- 比较 candidate 与 dev 中 DD/TD、ADR、basic design 和 Stable Release WBS。结果显示当前 dev 已有多个候选未包含的 canonical 文档，并已有更新后的 DD/TD 版本与 T-049 修订。
- 读取候选 DD/TD 中架构不变量、Owner/授权边界、数据库与迁移门、Schedule/Cron、测试状态、rollback、SBOM、开放条件与 revision history；读取当前 Vercel SRS/basic design、WLD1/VCD0 gap reviews、协作规范/W16 审计和 Stable Release WBS。
- 对候选两份文档执行定向关键词检索；未发现 conflict、counterpart、diff、event-push、owner-commit、cursor-catch-up、轮询或推送等协作语义命中。命中仅为文档 Draft 状态与函数 long-poll 限制。此结果只说明候选 Vercel 文档没有显式追踪这些协作验收，不代表全仓负面证明。
- 未运行产品测试、Rust/Node build、数据库/Neon 验证、Vercel deployment、Multipass/K3s 操作、hosted CI、release、tag 或 push。CodeRabbit CLI shim 调用返回 Permission denied，因此没有独立 CodeRabbit 结果。
- CodeGraph 查询没有 coverage/generation API；本轮是定向文档审查，不声称索引或代码关系完整。

实际证据命令（所有 Git 读取均经 RTK）：

- rtk proxy git status --short --branch
- rtk proxy git rev-parse HEAD
- rtk proxy git log -1 --format='%H %s' dev
- rtk proxy git log -1 --format='%H %s' codex/vcd0-canonical-detail-test-20261024
- rtk proxy git show -s --format='%H%n%P%n%cs%n%s' 4f0ceae9818b7bd2b3b25d9164d6e43303121689
- rtk proxy git show -s --format='%H%n%P%n%cs%n%s' a272e9794e1a1975781720c9b3775c914f47f546
- rtk proxy git merge-base a272e9794e1a1975781720c9b3775c914f47f546 4f0ceae9818b7bd2b3b25d9164d6e43303121689
- rtk proxy git diff --stat 4f0ceae9818b7bd2b3b25d9164d6e43303121689 a272e9794e1a1975781720c9b3775c914f47f546 -- docs/requirements/SRS-STAR-VERCEL-DEPLOY-001.md docs/basic-design/vercel/BD-STAR-VERCEL-DEPLOY-001.md docs/adr/vercel/ADR-STAR-VERCEL-DEPLOY-001.md docs/detailed-design/vercel/DD-STAR-VERCEL-DEPLOY-001.md docs/test-design/vercel/TD-STAR-VERCEL-DEPLOY-001.md docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md
- rtk proxy git diff --unified=4 4f0ceae9818b7bd2b3b25d9164d6e43303121689 a272e9794e1a1975781720c9b3775c914f47f546 -- docs/detailed-design/vercel/DD-STAR-VERCEL-DEPLOY-001.md docs/test-design/vercel/TD-STAR-VERCEL-DEPLOY-001.md
- rtk proxy git show 4f0ceae9818b7bd2b3b25d9164d6e43303121689:docs/detailed-design/vercel/DD-STAR-VERCEL-DEPLOY-001.md
- rtk proxy git show a272e9794e1a1975781720c9b3775c914f47f546:docs/requirements/SRS-STAR-VERCEL-DEPLOY-001.md
- rtk proxy git show a272e9794e1a1975781720c9b3775c914f47f546:docs/basic-design/vercel/BD-STAR-VERCEL-DEPLOY-001.md
- rtk proxy git show a272e9794e1a1975781720c9b3775c914f47f546:docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md
- rtk proxy git grep -n -i -E 'conflict|draft|counterpart|diff|event.?push|owner.commit|cursor.catch.up|poll|轮询|推送|草稿|冲突|差异' 4f0ceae9818b7bd2b3b25d9164d6e43303121689 -- docs/detailed-design/vercel/DD-STAR-VERCEL-DEPLOY-001.md docs/test-design/vercel/TD-STAR-VERCEL-DEPLOY-001.md
- rtk proxy git grep -n -i 'Neon' a272e9794e1a1975781720c9b3775c914f47f546 -- docs/requirements/SRS-STAR-VERCEL-DEPLOY-001.md docs/basic-design/vercel/BD-STAR-VERCEL-DEPLOY-001.md docs/adr/vercel/ADR-STAR-VERCEL-DEPLOY-001.md docs/reviews/WBS-VCD0-CONTRACT-GAPS-20261026.md docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md
- rtk proxy git show a272e9794e1a1975781720c9b3775c914f47f546:docs/reviews/WBS-VCD0-CONTRACT-GAPS-20261026.md
- rtk proxy git grep -n -E '^\\| W6 ' a272e9794e1a1975781720c9b3775c914f47f546 -- docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md
- coderabbit --version; coderabbit auth status

CodeRabbit 命令尝试未能启动，错误为 /root/.local/bin/coderabbit: Permission denied。

## §3 已知缺口

1. Candidate DD/TD v0.1 文档头与 current dev DD v0.3/TD v0.4 不一致；candidate TD 未含 noVNC 与最新 chat SSE/Origin 门禁。候选 references 仍指向旧 execution-closure WBS，不能作为当前 canonical 包直接接受。
2. Project native ownership、0..N repositories、hostless Run、provider optionality 和 migrate-before-unlink 未进入 hosted AC-VC-005/T-014 trace。
3. Neon profile 已作为架构目标选择记录在当前 VCD0 gap review，但 Vercel SRS GAP-06/basic design/DD 的平台事实、Owner placement、API/DB access、RLS、pooler 与独立 migration identity 文档不一致或未验证。需要同步，且 fail-closed 继续有效。
4. Vercel 文档缺少与 current W16 push collaboration 的端到端引用；T-049 是 Gateway/channel security plan，不证明 owner wake-up、durable cross-instance delivery、双方草稿、conflict_id、授权 diff、cursor recovery 或相应 SLO。
5. TD 中有 RLS、迁移、Owner receipt、Outbox 和幂等计划，但没有将 owner-local Inbox crash/retry/durable delivery 与协作 relay profile 作为明确验收链路；任何持久化证据缺失均保持 NOT_RUN。
6. Stable Release W0/W1/W6 等仍 PARTIAL/BLOCKED/OPEN。候选未声明版本或 MIT 释放门，但必须追踪当前 canonical Git-tag-only v1.0.6 与 MIT 决策，不可把测试设计或静态配置当作真实 Vercel release proof。
7. 本审查未验证任何账户、Neon 数据库、迁移、授权 API、hosted build、deployment、consumer、rollback、K3s/Multipass 或 GitHub protection 实况。

## §4 子代理失败接手清单

- 本 lane 未再派生子代理；没有失败子代理、RPC 失败或遗留修改需要接手。
- 父协调者提供了 CodeGraph source query 背景；因本任务是文档审查且未获得 graph coverage/generation API，没有转派图查询或声称图结果完整。

## §5 守门规则

1. 只按 brief 固定的 candidate SHA 与 dev SHA 做判断，不以 branch 名称代替提交核验。
2. 只读候选和当前基线；不改 candidate、要求/basic design/WBS 或其他人的文档。
3. 不执行 cherry-pick、merge、rebase、push、tag、release 或 deployment。
4. Vercel 仅为 Web/BFF surface；不让 BFF 成为身份、Project/Run authorization 或业务事实 owner。
5. Run 的创建、授权、列表与管理不依赖远端 repository/branch/provider identity；provider link 是可选元数据与单独授权。
6. hosted web path 必须使用当前 Star session 和 owner authorization；未验证 session/Run scope 时 fail closed。
7. 不以客户端 header、当前选中 UI state、Vercel token 或出口 IP 替代 Star authorization。
8. Vercel Preview 与 Production 数据、secret、Owner mutation、Schedule trigger 必须隔离；secret 不进 bundle、URL、terminal、log 或 trace。
9. 不由 Vercel 建立第二业务 Owner、业务数据库、迁移 authority、常驻 worker 或 in-memory business queue。
10. Neon 作为目标 profile 与 Neon/Owner/RLS/connection/migration runtime 事实分开；实际映射未证明即禁用相关路径。
11. Owner transaction、Outbox、Inbox、receipt、idempotency 与 provider-neutral durable delivery 保持明确边界；HTTP success 不代表命令完成。
12. collaboration 正常路径采用 owner commit 唤醒的有界 push；固定周期轮询不是实时协作验收替代。
13. 双方 draft、共同 conflict_id、participant authorization、counterpart diff、cursor catch-up 与 revoke/restart/gap recovery 需独立闭环。
14. 401 与 403 语义分离；不对状态未知的副作用请求更换幂等键或盲目重放。
15. Multipass/K3s 与 Host/cluster identity、两实例总量门禁、Star Runtime/Sandbox/Hooks 保持在 Star 控制面；Vercel 不接触 host shell、daemon/socket 或 cluster credentials。
16. 所有 candidate TD cases 初始 NOT_RUN；未有真实 Vercel/Neon/Owner/CI 证据不得升级状态。
17. Stable Release 目标为 Git tag 唯一版本源 v1.0.6，Star 项目选择 MIT；W0–W9 未闭合即不可发布，Multipass guest image 不是公开制品。

## §6 签字栏

| 角色 | 审批者与评语 |
|---|---|
| 架构 | 架构师（Mavis 接手审核）— 保留 Star Owner 与 Vercel fail-closed 边界，候选需适配当前 WLD1/W16 contracts。 |
| SRE Lead | 架构师（Mavis 接手审核）— 没有 hosted deployment、runtime、database 或 rollback 证据；相关门保持 OPEN。 |
| 平台 | 架构师（Mavis 接手审核）— 当前 dev DD/TD 已较候选更新；Neon 目标映射仍需文档与运行证据。 |
| 评审主持 | 架构师（Mavis 接手审核）— 评审范围、固定提交和限制已记录；不批准集成或发布。 |
| PM | 架构师（Mavis 接手审核）— 推荐 ADAPT；Stable Release W0–W9 与 Vercel W6 未关闭。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 对 pinned Vercel candidate 与 current dev/Stable Release 基线做只读验收；建议 ADAPT，记录 WLD1、Neon profile、W16 push collaboration、版本/MIT 与 hosted evidence 缺口。未运行测试或 hosted validation；未集成候选。 | brief docs/briefs/W6-VCD0-CANDIDATE-REVIEW-20261101.md |
| v0.2 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补记 W0 Web/Desktop/Canvas 版本绑定已合入当前 dev 的基线更新，并明确 ADAPT 仅对应原候选审查快照 | rebase 前的当前 dev 基线复核 |
