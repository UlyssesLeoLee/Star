# WBS-WLD7-CANVAS-COMPAT-20261026

> 状态：只读文档兼容性审计完成；候选内容部分可用，但当前不能原样集成。运行时与性能验收仍 OPEN/NOT_RUN。

## 0. 目的与审计边界

核对 Canvas A12 候选分支是否与本地最新 dev 中的协作需求、基本/详细设计、owner-local 数据设计和测试设计相容。只审查文档，不修改候选或规范正文，不运行测试，不把文档目标写成已实现能力。

## 1. 基线与候选差异

- 本审计 lane 基于本地 dev@7a82c098fe1dc021c8702e468768d68e8b0ee03e。
- 候选：`codex/wld7-canvas-a12-alignment-20261025`@5e5463bb12cf34a53d779e63f7ddc1b139236b76；merge-base 为 6b7543e17fd0f64f23f94ddea9807293167843aa。审计时 dev 比候选多 43 个提交，候选多 1 个提交；本地远端引用新鲜度未知。
- 候选唯一提交相对 merge-base 修改三份专题设计：`docs/requirements/SRS-CANVAS-AGENT-001.md`、`docs/design/BD-CANVAS-AGENT-001.md`、`docs/design/DD-CANVAS-AGENT-001.md`。候选 worktree 另有一个未跟踪 brief，不属于该提交。
- 候选把原先 A12 的传输/CRDT/LWW 假设标作历史，新增 owner/outbox、revision fence、RunContext 授权、作者私有草稿、sealed conflict、双边提醒和 OT 显式解决建议。设计-only、polling/process-local 以及 WLD-7/PT-09 未验收的标注是合适的。
- 候选没有修改测试设计；因此其 trace 必须继续指向当前 dev 测试设计，而不能沿用旧版引用。

## 2. 发现矩阵

| ID / 分类 | 证据 | 影响与建议 |
|---|---|---|
| C1 — stale / 交叉引用 | 候选 BD `§1.5` 与权威引用行 27、217，及 DD 行 34，仍引用协作 spec v1.1、owner-local data v0.3、test design v1.6，并只列 AC-COLLAB-01–08。候选 SRS 行 54 同样列旧数据/测试版本和 AC 01–08。当前 dev 分别是 `domain-collaboration-spec.md` v1.2（行 3、384）、`WLD-7-COLLABORATION-DATA-DESIGN.md` v0.4（行 3–5）和 `TEST-DESIGN-WORKTREE-CANVAS-001.md` v1.7（行 3–5）；当前协作 spec 已有 AC-COLLAB-09（行 367）。 | 将引用更新到 spec v1.2、数据设计 v0.4、测试设计 v1.7，并将 AC-COLLAB-09 纳入 A12 冲突解决追踪。候选 SRS v1.4、BD/DD v0.2 是此候选提出的修订版本；应在 rebase 后核对修订履历和所有反向引用。 |
| C2 — conflicting / 一秒语义 | 候选 BD 行 226、DD 行 1705 与 1713、SRS 行 315/958 把“≤1000ms”表述为两次 server commit time 的间隔；BD 行 230 又把它表述为提案到达时间，造成文内不一致。当前 requirements v7.07（`docs/requirements.md:5`）将 ≤1000ms 限定为完整、摘要有效的重叠提案到达刺激，第二份进入 owner 处理后立即进入 resource-fence 冲突事务；同一行另将 1 秒 p99 定义为冲突提交到双方提醒/首个有界差异可读的送达目标。当前 spec v1.2 §13（`docs/specs/domain-collaboration-spec.md:343`）明确 arrival skew 只是测试刺激，任何更长间隔的有效语义重叠仍须检测。 | 将候选中的所有“commit time 相差 ≤1000ms”改为“完整、digest-verified 的重叠提案在服务端 ≤1000ms 内到达；第二份进入 owner 后立即处理”。保留更长间隔也按 revision/path 检测的规则，并把 1 秒 p99 送达指标与冲突判定分开。 |
| C3 — incomplete / 用户选择与 CAS | 候选 DD 行 1703 允许在显式 resolve 时提出 OT 建议，行 1706 写当前 revision + 幂等 CAS；但候选三份变更文件均未包含 AC-COLLAB-09，也没有 E2E-COLLAB-10/11。当前 AC-COLLAB-09（requirements 行 3835；spec 行 367）要求无副作用预览、用户明确选择、成功的当前 revision owner CAS 才能更改 canonical revision；测试设计 v1.7 行 354–355 的 E2E-COLLAB-10/11 分别要求“OT 可变换也必须产生语义冲突”和取消/确认/stale CAS 路径，stale 时保留双方稿与冲突。 | 补齐显式选择结果、当前 owner revision CAS、取消不产生副作用、stale CAS 保留原两稿/冲突，并映射 AC-COLLAB-09 与 E2E-COLLAB-10/11。不能只把“OT 是可选建议”视为该门已覆盖。 |
| C4 — compatible / 方向一致 | 候选 BD 行 221–230、DD 行 1702–1716 和 SRS 行 958、962 确认 owner-local business truth、资源范围 fan-out、draft/conflict 持久化、无 LWW/共享 CRDT、授权复验和失败关闭，并声明 runtime 仍未验收。当前 spec v1.2 行 317、323、339 与 requirements v7.08 对 owner 事务、推送和草稿边界给出一致方向。 | 保留这些概念；对齐当前 owner-local 数据结构与新测试 ID 后再择取。候选对当前实现状态的 fail-closed 标注可保留。 |

## 3. 结论与最小后续修改

**结论：部分兼容，不建议原样 cherry-pick/合并候选三份文件。** 候选的 owner/outbox、授权边界和双方草稿原则值得保留，但当前版本引用、≤1000ms 语义和 OT 明确选择/CAS 测试追踪尚未满足 dev 的最新契约。

建议在候选分支 rebase 到最新 dev 后，仅对上述三份专题设计做定向修订：

1. 更新 spec/data/test 版本引用和 AC-COLLAB-09 映射。
2. 全文统一为“到达刺激 + 第二份 owner 处理立即冲突”，删除用 commit-time 差值定义一秒冲突的表述。
3. 把 E2E-COLLAB-10/11 与 AC-COLLAB-09 映射到 SRS/BD/DD 的冲突解决流程。
4. 更新修订履历，说明这里只完成设计对齐；数据库迁移、真实事件唤醒/跨实例推送、授权与撤权、恢复以及 PT-09 仍需单独实施和验证。

当前 dev 的 SRS-CANVAS-AGENT-001 文件头/修订表仍显示 v1.2，而候选新增 v1.4。该版本递增可作为候选改动审阅，但 rebase 时应确认中间修订历史和上位文档引用没有遗漏；本报告不替版本 owner 决定编号。

## 4. 验证摘要

| 项目 | 结果 |
|---|---|
| 候选 HEAD、merge-base、提交差异与三份变更文件 | 已只读核对 |
| 当前 requirements/spec/data/test 版本与 AC/E2E 语义 | 已只读核对 |
| Cargo / 单元测试 / PostgreSQL / UI / 实时运行 / PT-09 | NOT_RUN；本任务为文档审计 |

## 5. 已知缺口与处理边界

- 本报告未修改规范正文，也未审阅三份变更范围之外的历史文案。
- 设计正确不代表运行时已经实现；当前测试设计 v1.7 标注相关用例尚未实现/执行，WLD-7 与 PT-09 仍 OPEN/NOT_RUN。
- 不对候选 branch 做快进或整分支合并；审计时它落后 dev 43 个提交，须在整合时重新检查基线与精确差异。

## 6. 审核与修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-10-26 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 对比 Canvas A12 候选与本地 dev 当前协作契约；列出可保留、必须修正和未覆盖的内容；未运行测试。 |
