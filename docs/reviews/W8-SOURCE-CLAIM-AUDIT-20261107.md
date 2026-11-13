# W8 Source Position Claim Lane — Integration Readiness Audit

> **状态：HOLD / NOT READY FOR INTEGRATION。** 本报告仅审计 Git、MAGOS 登记、Orca worktree/terminal 状态与现有阶段报告；未读取 W8 源码、SQL 或数据库，也未作代码语义审查。
> 审计基线：本 lane dev@8c505f36f7679a349d20d3d5b86676cf5e70a393；候选 UlyssesLeoLee/w8-source-position-claim-fix-20261025@fceda75fb554069450214a2a9c0cef8c98a16c4c。
> 用户数据边界：候选工作树中的未跟踪 docs/briefs/WBS-W8-DECODE-FIX-20261026.md 仅由 Git/MAGOS 状态枚举发现；未读取、修改、移动或删除。

## §0 目的与边界

核对 W8 source-position claim 候选是否满足进入本地 dev 的前置条件，特别是登记范围、分支基线、工作树归属与报告中记录的运行验证状态。结论只针对可观测的 Git/MAGOS/Orca 元数据，不替代代码审查或 PostgreSQL/RLS/ACK 验证。

本轮只写入本 lane 登记的本报告；没有改动候选、其他 W8 worktree、dev、main、WBS、远端或既有用户内容。

## §1 审计矩阵

| 检查项 | 观测结果 | 集成影响 |
|---|---|---|
| 本审计 lane | codex/w8-source-claim-audit-20261107；基线 dev@8c505f36；初始状态仅有本 lane 已登记 brief 与 dispatch JSONL 未跟踪 | 本报告只在自己的隔离 worktree 提交 |
| 候选 Git 状态 | UlyssesLeoLee/w8-source-position-claim-fix-20261025@fceda75；tracked 状态未见修改，另有未跟踪 docs/briefs/WBS-W8-DECODE-FIX-20261026.md | 未跟踪内容未经审阅，必须保留；不能把当前工作树视为完全 clean |
| 候选与 dev 关系 | git rev-list --left-right --count dev...candidate 为 402 9；候选相对 dev 有 9 个提交、dev 相对候选有 402 个提交 | 候选基线陈旧且高度分叉；需先建立依赖图并在隔离候选上重放/复验 |
| Orca base/lineage | Orca 显示候选 baseRef 为 codex/wbs-w8-transport-remediation-20261025，父 worktree 是 C:/Users/leo19/.codex/worktrees/wbs-w8-transport-remediation/Star | 此 lane 是 W8 transport worktree 的派生候选，不是从当前 dev 直接开始 |
| 文件范围 | git diff --name-status dev...candidate 列出 18 个 tracked 路径；当前 MAGOS manifest 登记 5 个路径，另有 13 个 tracked 路径不在 manifest 中 | manifest drift 阻止全 lane 集成；未经重新登记的改动不得随 lane 进入 |
| MAGOS drift | magos_guard.py list --drift --json 对候选列出 14 个 drift 路径：13 个在分支差异中的 tracked 文件，以及上述 1 个未跟踪 brief。已登记路径为 shared-position-claim migration、PG arbitration test、phase report、high-isolation brief、dispatch JSONL | 必须先解决登记漂移；不得通过扩大提交或 cherry-pick 绕过 manifest 检查 |
| Orca 活动状态 | worktree ps 对候选显示 workspaceStatus=in-progress，hasHostSidebarActivity=false、liveTerminalCount=0、hasAttachedPty=false、unverifiableTerminalCount=0、agents=[]；定向 terminal list 也返回 0 项 | 查询时未观察到活跃 agent/terminal；该快照不消除未跟踪文件或登记风险 |
| 阶段报告 | PHASE-WBS-W8-SOURCE-CLAIM-FIX-20261025.md 将 W8 标记为 OPEN；最新记录加入高隔离 exact-replay/ACK 回归，但专用 PostgreSQL endpoints 未配置，相关 ignored DB suite 为 NOT_RUN | 编译/静态门不能替代实际 PostgreSQL、RLS 与 ACK 证据 |

### Drift 路径摘要

MAGOS 显示未登记漂移：Cargo.lock、Cargo.toml、crates/bi-native-event-adapter/Cargo.toml、crates/bi-native-event-adapter/src/lib.rs、crates/bi-owner-runtime/src/task_event_inbox.rs、crates/bi-owner-runtime/src/task_event_inbox_postgres_tests.rs、crates/bi-owner-runtime/src/task_event_quarantine.rs、crates/bi-owner-runtime/src/task_event_receiver.rs、crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs、db/migrations/2026-10-24-bi-task-event-source-origin.sql、db/migrations/2026-10-24-platform-native-transport-lease-order.sql、docs/briefs/WBS-W8-DECODE-FIX-20261026.md、docs/briefs/WBS-W8-TRANSPORT-REMEDIATION-20261025.md、docs/reports/PHASE-WBS-W8-TRANSPORT-REMEDIATION-20261025.md。

## §2 验证摘要

| 命令 / 来源 | 结果 |
|---|---|
| rtk git status --porcelain=v1 --branch --untracked-files=normal（候选） | branch 正确；1 个未跟踪 brief；没有读取该 brief 内容 |
| rtk git rev-list --left-right --count dev...UlyssesLeoLee/w8-source-position-claim-fix-20261025 | 402 9 |
| rtk git diff --name-status dev...UlyssesLeoLee/w8-source-position-claim-fix-20261025 | 18 个 tracked 路径；路径清单只用于范围比对，未打开源码/SQL |
| rtk orca worktree show / rtk orca worktree ps / 定向 rtk orca terminal list | 候选 owner 元数据、父 worktree 与终端状态已核对；查询时无活跃 terminal/agent |
| magos_guard.py list --drift --json | active claim 有 5 条登记路径及 14 条 drift 路径 |
| 既有阶段报告 | 最新文档状态仍为 W8 OPEN；目标 PostgreSQL endpoints 未配置，PG suite NOT_RUN |
| 代码语义审查 / SQL 语义审查 / 数据库验证 | 本审计未执行；不据此给出通过结论 |
| 本报告 rtk git diff --check | 提交前执行，结果见 lane commit |

Remote freshness：未 fetch；报告只依据本地 refs 与最近一次更新的远端跟踪 refs，不能证明远端当前状态。没有检查或修改远端 PR。

## §3 已知缺口

1. W8 lane 当前登记范围与 branch diff 不一致：13 个 tracked 改动路径没有在当前 manifest 中，另有 1 个未跟踪 brief。
2. 候选从旧 W8 transport lane 派生，与当前 dev 有显著历史分叉（402 behind / 9 ahead）；提交拓扑不证明这些改动在当前架构下语义独立或兼容。
3. 当前报告未审阅上述 13 个路径的代码/SQL，也未重新运行 Rust tests、PostgreSQL/RLS、commit-before-ACK、TLS/mTLS、恢复或 settlement 验收。
4. 阶段报告记录专用 PostgreSQL endpoints 未配置、ignored suite NOT_RUN；不存在本轮新的 DB 通过证据。
5. 当前 Orca 查询时无活跃 agent/terminal，但工作树仍 in-progress，并存在未跟踪 brief；不能据此自动归档或清理。

## §4 子代理失败接手清单

本审计未向下派发子代理。检查期间未发现需要接手的子代理输出。候选已有的阶段报告属于历史证据，尚未由本审计重新验证。

## §5 守门规则

1. 不修改、覆盖、移动、清理或删除候选及其他 W8 worktree 的任何文件。
2. 只允许本报告在当前 audit lane 中被写入；不改 WBS、dev、main 或远端。
3. manifest drift 未解决前，不集成整个候选 lane，也不以 cherry-pick 绕过登记门。
4. 分支需先同步到当前 dev 基线并重新审阅；若为公开/依赖来源，先协调依赖方，避免改写共享历史。
5. 必须独立检查未登记文件的语义、依赖与兼容性后，才能决定分拆或登记；本报告不授权更改 manifest。
6. PostgreSQL/RLS/commit-before-ACK 等运行证据缺失时，状态保持 OPEN/NOT RUN。
7. 当前用户未跟踪 brief 原样保留；不得把其存在描述成可丢弃垃圾。
8. 只根据获取到的本地 Git refs 判断；未 fetch 不声称远端最新。
9. 本报告不把 Git/Orca 元数据结论外推为代码正确性或生产可用性。

## §6 签批栏

| 角色 | 状态 |
|---|---|
| 架构 | 架构师（Mavis 接手审核）；只读集成准备审计，未批准集成 |
| SRE Lead | Mavis 接手审核；PostgreSQL/RLS/网络验收未运行 |
| 平台 | Mavis 接手审核；manifest drift 与旧基线分叉待处理 |
| 评审主持 | Mavis 接手审核；未执行源码/SQL 语义审查 |
| PM | Mavis 接手审核；W8 保持 OPEN，当前候选 NOT READY |

审批者：架构师（Mavis 接手审核）；未作人工部署或生产验收声明。
修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核。

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新建只读集成准备审计；记录 MAGOS drift、与 dev 分叉、Orca owner/terminal 快照及 PostgreSQL 缺口；不审阅候选源码或 SQL。 | W8 source-position claim lane 集成前状态盘点 |
