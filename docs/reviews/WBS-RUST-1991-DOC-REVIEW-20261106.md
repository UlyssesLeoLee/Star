# WBS Rust 1.99.1 / Vercel 文档候选审查

**审查日期：** 2026-11-06
**候选：** `codex/star-rust-1991-baseline-20261105@be154efb6cff5dd6eea6c85f5c5a5ccc00d8ccd6`
**目标基线：** `dev@59c63d1b272258e7820e2fdf71c62c206afd686a`
**结论：** **SELECTIVE ADAPT**。Rust 1.99.1 对齐是有效且领先的文档更新；本次不建议合并整个来源分支，也不建议直接用其旧版 DD/TD 覆盖 VCD0 canonical DD/TD。

## 1. 范围与证据

本次只读检查 brief 登记的七个来源文档、两条本地分支历史、来源分支对两个关联 WBS 文件的差异与当前 `dev` 基线。七文件 diff 为 **110 insertions / 13 deletions**：五份既有 Vercel 文档仅改 Rust 版本引用；另新增基线 brief 与阶段报告。WBS 变更仅作为相关追踪证据审阅，不在本 lane 修改或提交。代码、目标 Vercel 账户、远端 CI、数据库、镜像构建或发布行为均未由本审查执行或验证。

**分支级处置：HOLD 整个来源分支。** Rust 1.99.1 runtime/config 子集已由 `ebdde739` 集成到 `dev`；该提交是当前 `dev` 的祖先，包含 17 个工具链、CI、Cargo 与镜像配置路径。本次文档差异是另一交付单元，不应重复引入 runtime 改动或整分支合并。

主要对照物：

- VCD0 文档 lane：`codex/vcd0-canonical-detail-test-20261024@4f0ceae9818b7bd2b3b25d9164d6e43303121689`。其 DD/TD 由 `8019ec30` 建立，TD 由 `c7065825` 修订，DD 由 `4f0ceae9` 补齐审查缺口；这些是与来源分支 DD/TD 不同的文档候选。
- VCD0 审查史：`VCD0-DD-COMPATIBILITY-20261024.md`、`VCD0-TD-COMPATIBILITY-20261024.md`、`VCD0-BASIC-ADR-COMPATIBILITY-20261024.md`。这些审查针对更早的未批准草案；其拒绝项不能自动移植到后续 canonical 文档，但其 fail-closed、所有者边界、需求追踪和未运行状态仍是有效审查标准。
- 当前根 Cargo workspace package metadata：`Cargo.toml:182-187` 声明 `rust-version = "1.99.1"`、`license = "Apache-2.0"`；根 `LICENSE:1-3` 是 MIT。稳定版 WBS `docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md:4` 仍明示 NO-GO，版本目标 v1.0.6 未获发布放行。

## 2. 文件逐项结论

| 来源文件 | 精确差异 / 证据 | 结论 | 处理建议 |
|---|---|---|---|
| `docs/requirements/SRS-STAR-VERCEL-DEPLOY-001.md` | 1.99.0 → 1.99.1：仓库基线 `:39`、平台镜像能力门 `:103`、NFR-VC-08 `:204`、GAP-03 `:248`。FR-VC-002/AC-VC-007/GAP-03 仍把目标 Vercel build 当未验证门。 | **ADAPT** | 接受版本更新；保留目标构建、Rust Runtime/Streaming、数据库与 Owner 的未知状态。不得把本地 1.99.1 build 当 Vercel 通过。 |
| `docs/adr/vercel/ADR-STAR-VERCEL-DEPLOY-001.md` | VCD-2 隔离项目检查版本由 1.99.0 改为 1.99.1，`:49`；失败或未知仍关闭 Rust 执行。 | **ADAPT** | 接受该版本点更新；不得据此改变 VCD-2 进入 VCD-3 的门槛。 |
| `docs/basic-design/vercel/BD-STAR-VERCEL-DEPLOY-001.md` | 工具链表 `:168` 与 VCD-2 验证范围 `:173` 更新到 1.99.1；两处仍要求目标平台实测。 | **ADAPT** | 接受版本更新；保留“验证前无 Rust 生产 handler”与既有 Star Owner/执行面边界。 |
| `docs/detailed-design/vercel/DD-STAR-VERCEL-DEPLOY-001.md` | 候选文件 `:123`、`:310` 把 VCD-2 / open gate 从 1.99.0 改为 1.99.1。VCD0 canonical DD `:123`、`:310` 仍写 1.99.0；其 `:62-67`、`:170` 保留 chat SSE 路由未决并令 VCD-0 OPEN。 | **ADAPT** | 只吸收版本差异到经审查的 canonical DD；不要用此来源文件覆盖 VCD0 DD。保留 chat SSE、身份/Origin/Gateway 和 Owner 映射门。 |
| `docs/test-design/vercel/TD-STAR-VERCEL-DEPLOY-001.md` | 候选 T-028 `:127`、NFR-VC-08 `:238`、GAP-03 `:246`、已知缺口 `:284` 更新到 1.99.1。VCD0 canonical TD 的同类检查仍用 1.99.0：T-028 `:127`、NFR `:228`、GAP-03 `:236`、known gap `:274`；测试仍 NOT_RUN。 | **ADAPT** | 将 1.99.1 更新完整重放到 canonical TD 的所有活动引用；历史修订记录可保留原版本。保持 T-028、全部 AC 与 VCD-0 状态 NOT_RUN/OPEN。 |
| `docs/briefs/STAR-RUST-1991-BASELINE-20261105.md` | 新增简明范围、非目标和验收条件；明确不关闭 hosted CI、Vercel/Neon、Multipass/K3s、发布或执行门。 | **KEEP** | 保留为版本对齐工作的 brief；作为计划记录，不作为验证或发布证明。 |
| `docs/reports/PHASE-STAR-RUST-1991-BASELINE-20261105.md` | 新增实现与验证报告 `§1-§3`，并明确 workspace / hosted / Vercel / Neon / Multipass/K3s / release gates open，`:35`。其中 `:29` 总结剩余 1.99.0 仅历史/fixture/revision；这不涵盖尚未集成的 VCD0 canonical DD/TD 当前内容。 | **ADAPT** | 保留原测试记录为来源报告，但不得将其描述为独立复验。集成记录需明确 VCD0 DD/TD 仍有活动 1.99.0 引用，且版本改动本身没有关闭 VCD-0 或 v1.0.6 发布门。 |

### 2.1 关联 WBS 差异（来源范围外，只读）

| 文件 / 来源提交 | 差异与状态判断 | 结论 | 最小摘取建议 |
|---|---|---|---|
| `docs/implementation-plans/WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md` / `be154efb` | 3 行更新：Resource bounds `:103`、WLD-7 `:119` 和 VCD-2 `:122` 的活动 Rust 基线改为 1.99.1；VCD-2 仍需目标账户/套餐构建、依赖/包体、DB/RLS、RSS/并发、streaming/WebSocket 证据，未知失败继续禁用 Rust handler。 | **KEEP（路径级）** | 与 Vercel SRS/ADR/BD 的版本更新一致。解除来源 claim 并重新登记后，只摘取此文件差异；不要合入 `be154efb` 的全部 16 个文档路径。 |
| `docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md` / `c640eb34` | 5 行新增/修改：新增 `:3` 的 2026-11-05 基线快照，更新 W2 `:41`、W4 `:43`、W7 `:46` 与后续步骤的活动基线到 1.99.1，同时保留旧 1.99.0 测试失败为历史证据。文件仍称 W1 BLOCKED（根 Cargo Apache-2.0/MIT 冲突）且整体 NO-GO（`:3-4`）；没有宣称 v1.0.6 发布或 hosted gates 通过。 | **KEEP（路径级）** | `c640eb34` 同时修改 CI、Cargo、Docker、runner 等 34 个路径，禁止整提交 cherry-pick。协调者报告该来源目前仍持有 stable-WBS MAGOS claim；待 claim 释放后再登记并选择性应用此文件差异。本审查不释放或绕过 claim。 |

## 3. 设计与验收追踪

1. 该版本调整对应 Vercel SRS `NFR-VC-08`、`GAP-03`、`FR-VC-002` 与 `AC-VC-007`；VCD-2 必须在目标账户/套餐验证 build root、依赖闭包、包体、Rust Runtime 限额与回滚。文件中的 1.99.1 只更新待验收版本，不证明 Vercel build 已运行。
2. ADR/BD/DD/TD 共同保持“Vercel 页面/BFF 可选、Rust HTTP adapter 条件启用、Star Owner 和执行面继续权威”的结构。它与 Project/Run/Worktree/Branch 及 Multipass/K3s 自有运行面约束相容；本审查未发现对外文案引入竞争产品品牌。
3. 当前 VCD-0 canonical DD/TD 是另一个文档来源，不同于 Rust 来源分支的较旧 DD/TD。VCD0 分支最新 DD/TD 仍有 1.99.0 活动引用，并保留 chat SSE 路由差异、真实 Gateway/身份/Origin 契约和目标账户测试缺口。兼容性审查的早期 C-1/VCD-R1 关于第二套 Owner/数据库/执行面，已通过后续 canonical 单 Owner 文档的边界而被吸收为守门；不可把早期草案的拒绝描述成当前 canonical DD/TD 的全部状态。
4. 当前 `dev` 的 VCD-2 WBS 仍写 Rust 1.99.0（`docs/implementation-plans/WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md` §VCD-2）。来源分支另以 `be154efb` 更新该 WBS；稳定版 WBS 则由 `c640eb34` 更新。两处内容与 1.99.1 文档对齐，但需尊重各自 MAGOS claim，并按路径选择性集成；不合入两个提交中的无关改动。协调者报告 `c640eb34` 来源仍持有 stable-WBS claim，目前不应尝试集成该路径。

## 4. 许可证与发布主张风险

- 用户选择的根许可证为 MIT；实际根 `LICENSE` 正文是 MIT，但 workspace metadata 仍是 Apache-2.0（`Cargo.toml:187`）。这不推翻 1.99.1 工具链设置，却阻断将包元数据、SBOM、notice 与发行闭包视作法律身份已统一。W1 保持 BLOCKED。
- 稳定版 WBS 顶部 `:4` 为 NO-GO，且用户给出的 v1.0.6 是版本目标，不是已批准 tag。来源 brief/report 未声称发布完成；本审查不批准 tag、release、artifact、deployment 或 push。
- 阶段报告记录的 registry digest、CI action SHA、本机编译与 Python suite 结果属于来源报告陈述。本审查未重跑这些检查，也未核验 GitHub/容器 Registry 当前状态；它们不得被扩大为 workspace、hosted、Vercel、Multipass/K3s 或稳定发布通过。

## 5. 最小选择性集成建议

1. 接受 SRS、ADR、BD 的 1.99.1 变更；这三份更新不改变架构/门禁。
2. 不复制来源 DD/TD 整份文件。先审查/选定 VCD0 canonical DD/TD，再把 1.99.1 更新应用到 DD 的 VCD-2 与开放门，以及 TD 的 T-028、NFR-VC-08、GAP-03、known gap 和相关追踪文字；另更新 VCD-2 / 稳定版 WBS 中仍活动的 1.99.0 引用。
3. 保留 brief。阶段报告只作为来源阶段证据；若其入 dev，应限定它覆盖的验证命令、时间和路径，不把已有结果由本次只读审查背书。
4. 发布继续 NO-GO，直到 Cargo/LICENSE metadata、完整分发许可证闭包、VCD-0/VCD-2、hosted CI、目标数据库/RLS、最终镜像/SBOM、签名、公证、安装升级/回滚和 accountable release owner 的最终验收全部解决。

## 6. 限制

只读比较了指定来源和 VCD0 worktree 的文档及本地 Git 历史。没有切换、编辑、复制、rebase、merge 或清理任一来源 worktree/branch；没有读取 D:\\Star 的用户脏文件。未运行任何 Cargo、Python、Vercel、PostgreSQL、K3s、容器构建或发布测试。没有刷新远端 refs，也没有独立认证来源阶段报告里记录的 CI、registry、toolchain 或测试结果。
