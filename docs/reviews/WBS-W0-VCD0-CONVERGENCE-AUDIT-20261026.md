# WBS-W0 VCD-0 分支收敛审计报告

> 日期：2026-10-26
> 审计基线：`dev` @ `2fa28ecc3cb51d9c87badbd1a2a289fa41753f40`
> 范围：W0 brief 指定的六条 VCD-0 分支、对应 Worktree 与当前 Vercel 设计文档
> 结论：**不建议从这六条分支向当前 dev 再合入设计文件；VCD-0 仍 OPEN。**

## §0 目的与范围

依据 `docs/briefs/WBS-W0-VCD0-CONVERGENCE-AUDIT-20261026.md`，只读核对六条候选分支相对当前 `dev` 的提交、最终文件内容、相关评审证据、架构一致性、可能丢失的材料及后续集成建议。本报告不修改来源分支/Worktree、MAGOS claims 或产品文件；未运行测试。

## §1 分支、内容与保留矩阵

| 分支（tip） | 相对 dev ahead | Git / Orca 状态 | VCD-0 差异与建议 |
|---|---:|---|---|
| `codex/vcd0-canonical-basic-20261024` (`673e698d8ffcc474`) | 0 | Git clean；Orca `in-progress`、未归档；终端列表为空；owner 未提供 | 其基本设计 blob 与 dev 相同（`e79747e3…`），没有独有产品内容；不需合入。保留 Worktree，owner 状态仍 UNKNOWN。 |
| `codex/vcd0-canonical-detail-test-20261024` (`4f0ceae9818b7bd2`) | 5 | Git clean；Orca `in-progress`、未归档；终端列表为空；owner 未提供 | 分支最终 DD/TD 分别为 v0.1，dev 已为 DD v0.3/TD v0.4。分支 TD 丢失 noVNC 独立覆盖与契约批准后的强制 Origin 负向验证；DD 也缺少后续路由澄清。不要 cherry-pick 或整支合并。 |
| `codex/vcd0-td049-remediate` (`4e559dcd131e3d29`) | 5 | Git clean；Orca `in-progress`、未归档；终端列表为空；owner 未提供 | 最终 TD 与 dev 完全相同（blob `cd202ed9…`）；分支 DD 为较旧 v0.1（blob `92a6b0ed…`，dev 为 `aea8a8f8…`）。TD049 remediation brief 是独有的工作范围记录，但其设计结果已经在 dev；不需再合入产品文件。 |
| `codex/wbs-vcd0-basic-review-20261024` (`98c1c7ebadef2ee1`) | 2 | Git clean；Orca `in-progress`、未归档；终端列表为空；owner 未提供 | BD R2 review report 与 dev 完全相同（blob `684b8027…`）。唯一新增的 brief 只是已完成评审任务的指令记录；没有需补入的设计内容。 |
| `codex/wbs-vcd0-detailed-review-20261024` (`89c25f55475a36b4`) | 0 | Git clean；Orca `in-progress`、未归档；终端列表为空；owner 未提供 | 分支 tip 已在 dev 历史内；无领先提交，不需合入。 |
| `codex/wbs-vcd0-test-review-20261024` (`d0cc294261f396cb`) | 0 | Git clean；Orca `in-progress`、未归档；终端列表为空；owner 未提供 | 分支 tip 已在 dev 历史内；无领先提交，不需合入。 |

Git diff 同时显示，多条较早分支的完整 tree 相对 dev 缺少后来加入的 WBS、UI、WLD1、W8 与自动化审计材料。**禁止按分支 tip 做整支合并**，否则会以旧 tree 删除大量当前 dev 文件。这里的 ahead 数仅说明提交图关系，不代表这些提交的最终内容优于当前基线。

Orca 对六个精确 Worktree selector 均返回 `workspaceStatus: in-progress`、`isArchived: false`，没有 owner 字段、PR 或 issue；终端 inventory 均为 `terminals: []`，且 `local` host 未被省略。空终端列表不能证明没有活动聊天/agent owner，因此六个 Worktree 都应保留，不能归档或删除。Git 状态均为 clean。MAGOS claim listing 中仍可见 canonical basic、canonical detail/test、basic review、detailed review 与 test review 的文件级 claims；td049 remediation 未观察到 claim。此报告不清理或重置任何 claim。

## §2 当前 dev 的文件与验证证据

当前 canonical 文档状态为：SRS v0.4（Draft、0 implementation）、ADR v0.2（Draft、VCD-0 OPEN）、BD v0.3（Draft、VCD-0 OPEN、NOT_RUN）、DD v0.3（Draft、VCD-0 OPEN、NOT_RUN）、TD v0.4（Draft，所有用例 NOT_RUN）。BD 保留 Project → Engineering Run → Worktree → Branch 与可脱离远端 Branch 的 Run 身份；Vercel 只作可选 Web/BFF 呈现路径，Star Owner、执行面、Hook、BI、Worktree、Multipass/K3s 的权威边界仍由 Star 控制。

关键对照点：

- `docs/basic-design/vercel/BD-STAR-VERCEL-DEPLOY-001.md` §0、§1.2、§5（尤其第 4、32、137–138 行附近）保留 OPEN/NOT_RUN、hostless Run 和 chat SSE 路由未决事实。
- `docs/detailed-design/vercel/DD-STAR-VERCEL-DEPLOY-001.md` §2.3、§7、§12（第 62–66、170–177、313 行附近）未发明 Gateway 握手或路由，明确保持 unavailable/NOT_RUN。
- `docs/test-design/vercel/TD-STAR-VERCEL-DEPLOY-001.md` §4.7（第 161–175 行附近）已覆盖 noVNC；Origin 拒绝测试在真实契约获批后为必测项；当前 T-049 仍 NOT_RUN。
- `docs/reviews/VCD0-DD-COMPATIBILITY-20261024.md` §DD-R3-01..05 列明 DB owner/连接池/迁移映射、chat SSE 路由、浏览器通道清单、Gateway session/Origin/授权/撤权协议以及文档追踪差异。
- `docs/reviews/VCD0-TD-COMPATIBILITY-20261024.md` §R3-01..04 确认 TD v0.4 修复 noVNC 覆盖，但 remote file-browser 的 Vercel 范围和真实 Gateway 契约仍未解决。

仅执行 Git 图/文件 blob/diff、Worktree 状态与 Orca 只读查询；未运行测试、编译、账户检查、网络探测、数据库操作或部署。以上验证结果全部为 **NOT_RUN**，不能据此关闭 VCD-0。CodeGraph 对该 Worktree 未建立索引；本报告不作任何源代码调用关系或代码图结论。

## §3 已知缺口与可能丢失的内容

1. **VCD-0 仍 OPEN。** SRS/ADR/BD/DD/TD 尚未统一 chat SSE 的最终路由；真实 Gateway 握手、Origin allow/deny、当前授权与撤权传播、事件/帧、生命周期和恢复契约缺失。
2. **通道清单未闭合。** 当前 SRS 的 repository inventory 包含 remote file-browser，而 Vercel TD 的 T-049 没有该通道；noVNC/Canvas 的现有与目标范围也需要统一。不得把排除某通道默认为已批准决定。
3. **Owner/持久化映射仍待对齐。** DD R3 指出 FR-VC-009..012、TD T-038/T-039/T-042 与“不得建立第二套 Owner/数据库/迁移权威”的边界仍有冲突；需确定 Star Owner、RLS 事务上下文、连接池及迁移身份。
4. **追踪状态有漂移。** DD R3 指出部分 SRS header 和 WBS 版本引用需要随 canonical 文档收敛更新。
5. **历史资料风险。** basic review brief 与 TD049 remediation brief 在来源分支有独有文本；它们记录任务范围，不构成新的设计或实现。若后续清理分支，应先把本审计的结论及必要的来源追溯记录保存在 dev。当前 dev 已含相应评审结果和最终 TD 设计内容；没有发现需要从这些分支恢复的产品设计正文。
6. **所有执行/运行证据缺失。** Vercel 账户、部署、Rust runtime、DB/RLS、性能与业务行为均未在本审计中验证。

## §4 子代理失败与未验证项

本审计 lane 未派发 child agents，也未调用外部写操作，因此不存在由本 lane 派生的子代理失败接手项。Orca 的 owner 字段不可用；终端为空不能替代 owner 证明。MAGOS claims 只作只读盘点，未授权或执行任何 claim 清理。所有 runtime、账户、数据库、构建、测试及部署门仍未验证。

## §5 守门与集成建议

- 六条分支当前都**不需要向 dev cherry-pick/merge 设计文件**；保持 `dev` 当前 canonical 版本。
- 不以 ahead commit 数作为集成批准；不得整支合并任何存在大范围旧 tree 删除的分支。
- 在 Worktree owner 未知且 Orca 仍标记 `in-progress` 时保留所有六个 Worktree/branch。终端为空不足以解除保留门禁。
- 不清理 MAGOS claims；先由其 owner 完成任务结案与 claim 生命周期管理。
- 先在 SRS/ADR/BD/DD/TD 中关闭 DB Owner 映射、浏览器通道清单、chat SSE 路由/Gateway 契约和追踪漂移，再进行 VCD-0 最终批准；在此之前 T-049 与 VCD-0 保持 NOT_RUN/OPEN。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 / 本次审计自审 | Mavis 接手 agent per DEC-008；证据范围与“无合并”结论已核对 |
| SRE Lead | 不适用于只读分支/文档审计；未作运行批准 |
| 平台 | 不适用于只读分支/文档审计；未作部署批准 |
| 评审主持 | 由 parent agent 对本报告做最终独立复核 |
| PM | 不适用于本次分支审计；未批准 VCD-0 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-10-26 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 对 W0 指定的六条 VCD-0 分支进行提交、文件、架构、Worktree owner/terminal 与集成建议审计；结论为当前 dev 已包含需要的最终设计文本，不建议再合并分支；VCD-0 仍 OPEN。 |
