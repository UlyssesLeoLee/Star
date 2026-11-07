# PHASE — WBS Candidate Revalidation (Current dev)

## 0. 目的

按当前 dev@e2393618d5d9cf71181bdadf78b5b4663850e4cf 重新核对三个仍未收敛的 VCD-0/WLD1 候选，防止旧设计、错位引用或未登记文件被整分支带入 dev。复核结果供选择性集成使用，不代表目标 DB、Hosted deployment 或产品运行闭环已通过。

## 1. 改动与结论矩阵

| 来源 lane | 复核范围 | 结论 | 集成决策 |
|---|---|---|---|
| codex/wbs-vcd0-basic-review-20261024@98c1c7eb | 两份已登记 brief/review | 登记内容已在 dev；review Git blob 完全相同。来源分支另有两份未登记文件 | 不摘取；不合并登记外文件 |
| codex/vcd0-canonical-detail-test-20261024@4f0ceae9 | VCD-0 DD/TD 与两份修复 brief | 候选版本、路由范围、noVNC 与安全负向测试落后于当前 dev；部分 owner mapping 已在 dev | 拒绝整分支与 cherry-pick；VCD-0 继续 OPEN，TD 继续 NOT_RUN |
| codex/wbs-wld1-boundary-review-20261025@de2efaad | Repository boundary review | owner/cardinality/迁移方向有用；引用过期，且当前 WBS 状态与当前设计内容不一致 | ADAPT selectively；保留未决项及 provider fail-closed 边界，不合并旧快照 |

## 2. 验证摘要

- 当前 dev checkout 在本次审阅开始时为干净状态，HEAD 为 e2393618d5d9cf71181bdadf78b5b4663850e4cf；本 lane 从该基线创建。
- 三份候选分别处于独立、无活动终端的来源 worktree。三个 reviewer 均为只读；未编辑来源 branch/worktree。
- VCD0 Basic 已登记 review 文件与 dev 内容比对相同，Git blob ID 为 34673af9e9db56764b2cbe975b5df89705a01fe2。
- 当前 VCD-0 SRS v0.4、DD v0.3、TD v0.4 仍为 Draft；TD 测试状态仍为 NOT_RUN。
- WLD1 provider 界面与 fail-closed default 由当前 dev 源码复核；实际 PostgreSQL、RLS、授权与 writer integration 本阶段未运行。
- 本阶段未运行产品测试、部署或数据库测试；本报告不得标成实现验收通过。

## 3. 已知缺口

1. VCD-0 的 Vercel production、Neon/Owner runtime、Hosted build、consumer、跨服务协作与 rollback 仍 OPEN/NOT_RUN。
2. WLD1 的持久 Project owner writer/API/schema、grant/RLS、Repository association revision/CAS、引用 reconciliation 与 owner-local audit/Outbox 尚未运行验收。
3. Current WBS 的 WLD1 状态行与当前已定义的详细设计存在文档漂移。
4. Group WBS 主文件与受保护 Rust 候选登记重叠；本 lane 未改写该主 WBS。需在解除重叠后更新。
5. main 中 DDD 审计文档内容已在 dev；它不构成此三个候选的理由或依赖。

## 4. 子代理与失败接手

- 三个只读 reviewer 完成，分别负责 VCD0 Basic、VCD0 DD/TD、WLD1 boundary。
- WLD1 reviewer 在初次探测时查询了两个不存在的 D:/Star 文档路径；均返回 path-not-found，未读取用户脏内容。之后仅用隔离 worktree 中的 git show 比对 dev commit，并给出精确证据。
- 未派发实现 worker；无未收回代码或测试结果。

## 5. 守门规则

1. 只以当次核验的 dev SHA 作目标，不引用过期远端快照证明当前状态。
2. 只集成 manifest 与审查明确允许的文件。
3. 不将登记外文件随整分支带入。
4. 不 cherry-pick 已在 dev 且 blob 相同的内容。
5. 不将旧候选 DD/TD 覆盖更新版本。
6. 不新增未经 SRS 批准的 FR/AC/NFR 编号。
7. 不把通用平台能力推断为 Rust Runtime 能力。
8. Vercel 不拥有 Run 授权、执行 worker、Worktree、Agent sandbox 或持久业务事实。
9. Cron 只作 best-effort wake-up；每次无人值守运行必须重新授权。
10. Neon、Owner、PostgreSQL/RLS 未运行时保持 OPEN/NOT_RUN。
11. Project→Repository 0..N 与先迁移后解绑的顺序保持不变；禁止先解绑后迁移。
12. Worktree lifecycle provider 缺失时继续 fail closed。
13. 迁移目标、权威引用源和跨 owner 事务边界未冻结时不开放解绑。
14. 历史 WBS 行保持历史含义，不因状态过期而篡改历史叙述。
15. WBS、SRS、BD、DD、TD 版本与行引用必须对照当前 dev 重查。
16. 保留 D:/Star 的用户脏文件及受保护 W8/Rust 候选 worktrees。
17. 不 push、不改 main、不声明本次为 runtime 或发布验收。

## 6. 评审与签批

| 角色 | 结果 |
|---|---|
| 架构评审 | 架构师（Mavis 接手 agent per DEC-008）：有条件通过文档复核；不接受旧候选实现 |
| SRE/Infra | 未执行环境部署；Vercel、Neon 与 Multipass/K3s runtime 保持 OPEN |
| 平台 | 不采纳未验证的 Rust Runtime 或第二 execution-plane 主张 |
| 评审主持 | Mavis 接手：批准选择性适配结论，拒绝整分支合并 |
| PM | 版本仍为草案；当前 W6/WLD1 闭环未完成 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 首次记录三个候选在 current dev 的审查结论、选择性集成决定、未运行验收和 WBS 冲突 | 用户要求并行推进 WBS 并只把经审查内容 rebase/merge 到 dev |
