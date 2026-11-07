# PHASE-WLD7-TEST-CITATION-ALIGNMENT-20261107

## §0 目的

以变更前的 domain collaboration spec v1.5 与 owner-local data design v0.7 为基线，核对其中指向 test design v2.6 的 WLD-7 验收锚点，并与当前 v2.8 比较。最终源文档升级至 spec v1.6/data v0.8。仅处理登记的六个文档路径；本阶段只更新引用和如实说明状态，不实现运行时功能。

## §1 改动矩阵

| 范围 | 结果 |
|---|---|
| domain-collaboration-spec.md | Version/reference synchronized to v1.6: active data-design ref v0.8 and test-design v2.8 §4.8/§5.2/§6.2.5; clarify partial evidence; append revision history |
| WLD-7-COLLABORATION-DATA-DESIGN.md | Version v0.8: update test-design citation to v2.8 E2E-COLLAB-09 and spec cross-reference to v1.6; append revision history |
| lane brief / phase report / review / dispatch | 记录逐锚点比较、结论、静态检查和未解决的 §9.5 交叉引用 |
| test-design、WBS、SRS、BD、DD、代码 | 未修改 |

## §2 验证摘要

基线为指定 worktree 的 dev@22b92756ad762f27297c493760d95435ef98f2d1；历史 v2.6 快照取自 6634a455bc475a66128390d36d956c47b659fe9a，当前 test design 为 v2.8。

- 比较了 §4.8 的 WLD-7 用例 E2E-COLLAB-01..11：11 行完全相同。
- 比较了 §6.2.5 的 AC-COLLAB-01..09 映射：9 行完全相同。
- 比较了 §5.2 的 PT-09：内容完全相同。
- v2.8 把 §4.8 计划数从 23 增至 24，增加的 E2E-COLLAB-24 属于 W16；§4.8 状态段明确指出一个 PG18.6 设备撤销/SSE 定向测试不代表 WLD-7 完整多用户 revision-push E2E 通过。
- §9.5 的 WLD-7 来源仍列 collaboration spec v1.0 与 data design v0.2；该过期来源版本在 v2.6 与 v2.8 均存在。test-design 文件归另一 lane 所有，本阶段未修改。
- rtk git diff --check 通过；手工审阅差异仅包含已登记文件。未运行 tests、Cargo、数据库、VM、云、构建或部署。

## §3 已知缺口

- WLD-7 多用户 revision push、持久双方草稿/冲突、跨 gateway 丢失通知恢复仍 OPEN/NOT_RUN。
- Owner migrations、grants、RLS、目标 PostgreSQL 证据和 PT-09 性能实测仍未完成。
- test-design §9.5 的 spec/data 版本交叉引用应由其 owner 后续修正；本 lane 无权改 test-design。
- 本阶段不证明任何生产运行时闭环，也不改变现有实现状态。

## §4 子代理失败接手清单

本 lane 未派出子代理；没有子代理失败或待接手事项。

## §5 守门规则

1. 仅写入本 brief 登记的六个路径。
2. 不根据版本号推断语义变化，逐行比较了被引用的 WLD-7 锚点。
3. v2.6 与 v2.8 的 E2E-COLLAB-01..11 对照一致。
4. v2.6 与 v2.8 的 AC-COLLAB-01..09 映射一致。
5. PT-09 的验收内容未改变。
6. 新增的 E2E-COLLAB-24 属于 W16，不冒充 WLD-7 新用例。
7. PG18.6 单项设备撤销/SSE 结果不标记为完整 WLD-7 通过。
8. WLD-7 状态保持 OPEN/NOT_RUN。
9. 不宣称目标 schema 已迁移或部署。
10. 不宣称 grants、RLS 或目标 PostgreSQL 验收通过。
11. 不宣称 PT-09 或资源/性能阈值已经实测。
12. 不修改 test-design、WBS、SRS、BD、DD 或代码。
13. 不运行测试、Cargo、数据库、VM、云、构建或部署命令。
14. 不修改 main、远端 refs 或其他 worktree。
15. 不合并、不清理、不 push。
16. §9.5 的过期 WLD-7 来源引用记录为后续项，未在无权路径上修正。

## §6 签字栏

| 角色 | 责任人 | 状态 |
|---|---|---|
| 修订人 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 已完成指定文档引用核对与修订 |
| 架构审核 | 架构师（Mavis 接手 agent per DEC-008） | 协调者集成前复核本 lane commit |
| SRE Lead | Mavis 接手 | 无运行环境或生产验收声明 |
| 平台 | Mavis 接手 | 未执行平台、VM、数据库或部署操作 |
| 评审主持 / PM | Mavis 接手 | 范围、状态与后续项已记录 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 建立 WLD-7 test design v2.6→v2.8 精确锚点比较记录；同步两个当前引用，保留未实现/未验收状态。 |
