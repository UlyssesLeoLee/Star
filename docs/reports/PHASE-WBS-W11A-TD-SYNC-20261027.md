# PHASE-WBS-W11A-TD-SYNC-20261027

## §0 目的

同步 WLD1 测试设计，使其覆盖当前 v0.4 需求/基本设计/详细设计中的 hostless Run、可选 Infrastructure placement、V2/V3 Schedule execution boundary，以及 Neon 与 Multipass VM 的独立验收约束。本 lane 只同步测试设计及阶段报告；W11-A 保持 Partial/Open。

## §1 改动矩阵

| 范围 | 改动 | 状态 |
|---|---|---|
| TD 元数据与追踪矩阵 | 引用 SRS/BD/DD v0.4；新增 W11-A 需求和验收映射 | 完成 |
| Unit/contract | 增加 Hostless Run、Repository/placement 独立性、no-placement、generation pinning、V2/V3 来源/actor/attribution/attempt/digest 和迁移 quarantine 用例 | 完成 |
| PostgreSQL/API/deployment | 增加 V2 不变性、V3 additive schema、owner/RLS/grants、并发幂等和双目标部署验收 | 完成 |
| UI/状态 | 对齐 Project → Engineering Run → Worktree → Branch；明确 Branch 是本地 checkout state；保持 Partial/Open | 完成 |
| 阶段报告 | 记录静态审查范围、未运行项及下游阻塞 | 完成 |

## §2 验证摘要

仅进行静态文档一致性审阅；未运行 Cargo、测试、数据库、迁移、API、Neon、VM、Multipass、K3s 或运行时验证。本报告不宣称任何测试或数据库门禁通过。TD 中列出的所有案例仍是待实施/待执行验收。

## §3 已知缺口

- W11-A 仍为 Partial/Open。
- DD-LOCAL-INFRASTRUCTURE-001.md 在主工作树有用户已有移动/修改状态；本 lane 未读取、复制或修改该文件，相关设计同步未完成。
- WBS 更新不属于本 lane；后续需由协调者在对应 lane 核对和同步。
- V3 schema/migration、owner roles、RLS/grants、V2 回放兼容、V3 attempt 并发、Neon 和 Multipass VM 目标证据均未实现或验证。
- Multipass inventory 的两实例限制、含 Stopped/Deleted 项目计数、禁止自动创建/purge 是验收条件，不表示已检查本机 inventory。

## §4 交接/失败检查清单

| 项 | 交接状态 |
|---|---|
| 测试设计对齐 v0.4 WLD1 SRS/BD/DD | 已写入 TD，待协调者审阅 |
| V2 不变性与 V3 additive 路径 | 已列测试，需实现/数据库 lane 验收 |
| 目标数据库分离 | Neon 与 Multipass VM 分列，不能互相替代证据 |
| DD-LOCAL 与 WBS 联动 | 未完成，保留为 Partial/Open |
| 实现与 runtime 验证 | 本 lane 未执行，需后续授权 lane 完成 |
| dispatcher brief/audit | 保留原文；audit 未手改 |

## §5 守门规则

1. Engineering Run 可以无远端 Branch 身份创建、列出和授权。
2. 导航顺序固定为 Project → Engineering Run → Worktree → Branch。
3. Worktree 下的 Branch 只表示本地 checkout ref/state。
4. Project Repository association 与 Run Infrastructure placement 是独立关系。
5. 未选择 placement 不创建 binding、reservation 或 infrastructure demand。
6. 已选择 placement 必须 pin Profile revision/digest、Host ID/generation、cluster ID/generation。
7. V2 保持 branch_id 必填和 Schedule-only；Manual 对 V2 fail closed。
8. V3 以 tagged Schedule/Manual origin 表达触发来源。
9. V3 execution actor 从 owner 授权上下文派生，不信任客户端 actor 字段。
10. V3 schedule correlation 仅作可选因果归因，不作为身份或授权事实。
11. 每个 V3 attempt 身份唯一；相同 digest 重放原 receipt，不同 digest 冲突。
12. V2/V3 幂等空间相互隔离，不跨版本重放。
13. 迁移失败、legacy row 含义不明或授权已撤销时 quarantine；不改写旧事实。
14. Neon 与 Multipass VM 的角色、RLS、migration 和授权证据分别验证。
15. Multipass inventory 上限为两个，计入 Stopped/Deleted；不得自动创建或 purge。
16. 计划测试不得表述为已运行或已通过。
17. W11-A 只有在文档依赖和 WBS 依赖均闭合、实现验收有证据后才可关闭。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 | Mavis 接手审核；本报告仅记录文档同步 |
| SRE Lead | 待后续目标环境验收 |
| 平台 | 待后续数据库/RLS/placement 实现验收 |
| 评审主持 | 待集成审阅 |
| PM | W11-A Partial/Open |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-10-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 首次记录 W11-A TD 静态同步范围、验证限制和未完成依赖。 |
