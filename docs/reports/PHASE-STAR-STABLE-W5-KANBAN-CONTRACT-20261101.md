# PHASE-STAR-STABLE-W5-KANBAN-CONTRACT-20261101

## §0 目的

修复稳定版 W5 前端基线中 Kanban 默认列快照过期，以及空 W/T/M swimlane 仍渲染空容器的问题。本 lane 基于 `dev@41f84d407e0c245af88ab29938e8dda57c67af94`，仅处理登记的 Kanban 实现/测试、dispatcher 资料与阶段报告；不修改共享 WBS、锁文件、其他 lane、`main` 或 `dev`。

当前领域类型 `WorkItemStatus` 有六个合法状态：`todo`、`in_progress`、`review`、`blocked`、`done`、`wontfix`。默认看板固定显示五列：`todo`、`in_progress`、`review`、`blocked`、`done`；`wontfix` 仍为合法终态，但不属于默认列。该结果与 `ProjectsClient` 的五列顺序和 `KANBAN_COLUMNS` 实现一致。看板组件接收 `board.columns`，因此仍可渲染特定 Board 定义的其他列。

## §1 改动矩阵

| 文件 | 改动 | 状态 |
|---|---|---|
| `frontend/src/components/board/KanbanBoard.tsx` | 移除组件注释中的固定四列说法；过滤空 W/T/M 分组，使没有卡片的 swimlane 不再产生空容器；保持状态列与 drop 行为不变。 | 已修改 |
| `frontend/src/mocks/__tests__/kanban.test.ts` | 断言五个默认列及 `blocked` 的位置，不再依据旧四列断言。 | 已修改 |
| `frontend/src/mocks/__tests__/__snapshots__/snapshot.test.ts.snap` | 把 `blocked` 纳入 `KANBAN_COLUMNS` 快照。 | 已修改 |
| `frontend/src/mocks/__tests__/kanban.test.ts` Cypher header | 记录测试文件直接验证 `KANBAN_COLUMNS` 的关系并刷新来源摘要。 | 已刷新 |
| `docs/briefs/STAR-STABLE-W5-KANBAN-CONTRACT-20261101.md` | 记录 lane 范围、基线、验证与限制。 | dispatcher 已生成 |
| `docs/reports/WBS-DISPATCH-STAR-STABLE-W5-KANBAN-CONTRACT-20261101.jsonl` | 记录真实 dispatcher brief 和本次验证事件。 | 本报告提交时生成 |
| `docs/reports/STAR-STABLE-W5-KANBAN-CONTRACT-20261101.log` | dispatcher 自动生成的 brief 审计记录。仓库以 `*.log` 忽略此文件，因此按已登记 manifest 精确 force-stage 该路径。 | 已纳入 lane manifest |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| 基线 focused suite | 复现 5 项失败、11 项通过：列数为 4 的断言失败，列序和 snapshot 缺少 `blocked`；两个 W/T/M 测试暴露空分组容器仍被渲染。 |
| `rtk proxy npm test -- --run src/components/board/KanbanBoard.test.tsx src/mocks/__tests__/kanban.test.ts src/mocks/__tests__/snapshot.test.ts` | PASS；3 个文件、16 项测试通过；Node `v22.19.0`、npm `10.9.3`、Vitest `5.0.1`。 |
| 依赖准备 | lane 原本没有 `frontend/node_modules`；为在本 lane 上实际执行测试，临时将其 junction 到 dev worktree 的同版本依赖目录。junction 是测试环境辅助项，不在 Git diff 内，完成后移除。 |
| `rtk proxy git diff --check` | PASS；无空白错误。Git 提示 snapshot 文件未来可能转换 LF/CRLF；本次 diff 仅增加一行，没有格式化整个快照。 |
| `rtk proxy python <cgg_guard.py> --changed --stamp` | PASS；2 个变更源文件通过 CypherGraph Guardian。 |
| CodeRabbit 独立审查 | 未完成；两个版本查询均被 CLI shim `/root/.local/bin/coderabbit` 的 `Permission denied` 阻断，因此没有自动化审查结论。 |
| 全量 Web suite / TypeScript typecheck / Playwright E2E / Node 24 hosted run | 本 lane 未运行；focused suite 结果不能替代这些发行门。 |

第一次运行 CypherGraph Guardian 时发现 `kanban.test.ts` 没有 header；按现有直接 import 与 assertion 增加 `TESTS` 关系后复跑通过。第一次启动测试时 lane 没有依赖目录，Vitest 因而无法解析；通过上述临时 junction 使用锁定依赖后，命令在本 lane 通过。

## §3 已知缺口

1. 现有历史 UI/设计文档仍可见四列描述；它们不在本 lane 登记文件中，因此本报告没有改写这些文档。协调者需把对应需求/设计文档同步到当前五列默认契约，再关闭 W5 的文档一致性门。
2. `wontfix` 是领域允许的 WorkItem 状态，但不在默认 Kanban 列。当前实现与其他 UI 已如此定义；是否需要在用户自定义 Board 中显示该列，仍由 Board 配置决定，不由默认列列表推导。
3. 本次只验证 KanbanBoard、默认列契约和 mock snapshot 的 focused tests；没有运行全量前端套件、typecheck、Playwright、Desktop embedded frontend 或 Node 24 hosted pipeline。
4. 本次没有新增任务数据或生产数据路径，fixture 仍是组件测试使用的本地对象；本结果不证明后端任务 provider 或生产看板数据已可用。
5. dispatcher `.log` 会被全局 `*.log` 忽略；本 lane 只因 MAGOS 清单明确登记该文件而精确 force-stage 它，没有加入其他 ignored 文件。
6. CodeRabbit CLI shim 无法启动；结果仍需协调者人工审阅，不能标记为独立自动审查通过。

## §4 子代理失败接手清单

- 本 lane 没有再派生子代理；无失败、超时或未接收结果需要接手。
- 本结果等待协调者独立审阅、WBS 同步与最终 lane 集成；worker 未写入 `dev`。

## §5 守门规则

1. 仅在 `kanban-contract` 隔离 worktree 修改登记的 Kanban 与本 lane 文档文件。
2. 默认列以当前 `KANBAN_COLUMNS` 与 `ProjectsClient` 一致性为依据，不沿用旧测试里的四列常量。
3. `blocked` 保留在默认列中；列顺序保持 `todo → in_progress → review → blocked → done`。
4. `wontfix` 保留为合法领域状态，不擅自加入默认列。
5. Board 组件继续按输入 `board.columns` 渲染列，不把五列限制硬编码到任意 Board。
6. 空 W/T/M swimlane 不渲染，避免空白区域；有卡片的分组顺序仍为 W、T、M。
7. 未分类 WorkItem 的展示保持现有路径，不改变其分类/状态语义。
8. 卡片拖放仍以目标 status 调用现有 `onTransition`，本修复不改状态机。
9. 组件测试 fixture 继续使用本地测试对象，不引入生产数据，也不创建全局任务记录。
10. 不恢复、添加或复制旧演示/mock 任务。
11. 不修改 `package.json`、lockfile、依赖版本或 Vitest 配置。
12. 不修改共享 WBS、其他 lane、`main`、`dev` 或远端 refs。
13. 测试文件 Cypher header 仅记录可由 import/assertion 验证的 `KANBAN_COLUMNS` 关系。
14. `cgg_guard.py --changed --stamp` 必须通过后再提交源改动。
15. focused Kanban/mock tests 与 `git diff --check` 必须通过。
16. 未运行的全量 suite、typecheck、E2E、Node 24 hosted checks 必须继续标记为未验证；本 lane 不关闭 W5。
17. 不 push、不创建 tag/release、不部署，也不清理其他分支或 worktree。

## §6 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | Mavis 接手审核：五列默认契约与六态领域类型区分明确；历史设计文档同步仍待协调者处理。 |
| SRE Lead | Mavis 接手审核：本地 focused suite 通过；全量、Node 24 hosted 和 E2E 尚未验证。 |
| 平台 | Mavis 接手审核：Cypher header gate 与 diff whitespace 检查通过。 |
| 评审主持 | Mavis 接手审核：实现仅过滤空 swimlane，不改变用户状态机或 drag/drop 行为；等待独立 coordinator review。 |
| PM | Mavis 接手审核：本 lane 局部完成；稳定版总体状态保持 NO-GO，W5 仍 OPEN。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修正过期的四列断言/快照；按现有 UI 契约隐藏空 swimlane，并记录 focused 验证和发行门限制。 | W5 前端测试复现 Kanban 列契约与空 swimlane 失败 |
