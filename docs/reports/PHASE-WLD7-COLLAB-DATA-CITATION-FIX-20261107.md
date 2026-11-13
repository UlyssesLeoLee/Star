# PHASE-WLD7-COLLAB-DATA-CITATION-FIX-20261107

## §0 目的

仅同步 WLD-7 owner-local 数据设计的当前测试设计引用，使其指向 test design v2.6；不更改逻辑数据模型或实现状态。

## §1 改动矩阵

| 文件 | 改动 | 状态 |
|---|---|---|
| `docs/data-design/WLD-7-COLLABORATION-DATA-DESIGN.md` | 当前上游引用从 test design v1.7 更新至 v2.6；保留 v0.4 历史内容并追加 v0.5 说明 | 已完成 |
| `docs/briefs/WLD7-COLLAB-DATA-CITATION-FIX-20261107.md` | 记录本 lane 的目标、范围及验证限制 | 已登记 |
| `docs/reports/WBS-DISPATCH-WLD7-COLLAB-DATA-CITATION-FIX-20261107.jsonl` | 记录 brief 创建与 worker 派发 | 已登记 |
| 本报告 | 记录范围、验证边界及未完成状态 | 已完成 |

## §2 验证摘要

- `git diff --cached --check`：通过（无 whitespace error）。
- 精确变更路径清单核对：通过；暂存路径恰好为 brief 登记的四个 manifest 文件。
- 应用、Cargo、数据库、云端、VM/K3s、构建及性能验证：未运行，且不属于本 lane。
- WLD-7 实现与目标数据库验证：`OPEN / NOT_RUN`。

## §3 已知缺口

- 本文件描述逻辑目标 schema；迁移尚未生成，目标数据库尚未部署或验证。
- WLD-7 实时协作实现及目标数据库验收仍为 `OPEN / NOT_RUN`。
- v0.4 历史记录仍引用当时的 test design v1.7；本次只更新当前文件头引用，不改写历史叙述。

## §4 子代理失败接手清单

- 当前 lane 未发现子代理失败或未回收产物。
- 协调者需复核本 lane 的四文件差异及边界陈述；本报告不代替集成审查。

## §5 守门规则

1. 只改 brief 中登记的四个文件。
2. 不修改 collaboration spec、test design、WBS 或其他源文档。
3. 保留 v0.4 及此前修订历史，不将历史引用改写为当前版本。
4. 不把逻辑目标 schema 描述成已迁移、已部署或已验证。
5. 不运行未列出的应用或基础设施验证，不把 `NOT_RUN` 写成通过。
6. 本 lane 不执行 rebase、merge、push、release 或 worktree 清理。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构师（Mavis 接手审核） | 待协调者终审 |
| SRE Lead | 不适用：仅文档引用修订 |
| 平台 | 不适用：仅文档引用修订 |
| 评审主持 | 待协调者复核 |
| PM | 待协调者复核 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增本阶段记录；静态文档检查通过，待协调者复核 | WLD-7 当前测试设计引用同步 |
