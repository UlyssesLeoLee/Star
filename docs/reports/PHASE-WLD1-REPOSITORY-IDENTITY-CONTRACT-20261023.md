# PHASE-WLD1 Repository Identity Contract 文档切片报告

- 版本：v0.1
- 日期：2026-10-23
- 阶段：WBS-WLD-NEXT-20261023 / WLD-1 repository identity contract
- 基线：`ac73efe51897522215f96559e33e716ad551577d`
- 分支：`codex/wbs-wld1-repository-contract`
- 状态：**设计合同已补齐；Repository master、Project link persistence、Run binding writer、Host root registry 与 lifecycle provider 均未实现/未验收。**

## §0 目的

将 Project→Repository、Engineering Run→Repository 与可信本地 repository root 的身份及 owner 边界写入需求、基本设计、详细设计和追踪文档，并与 [`PHASE-WLD1-REPOSITORY-DIRECTORY-20261023.md`](PHASE-WLD1-REPOSITORY-DIRECTORY-20261023.md) 的源审计对齐。本切片只提出并追踪目标契约，不实现数据库或运行时行为。

## §1 改动矩阵

| 文件 | 改动 | 状态 |
|---|---|---|
| `docs/requirements.md` | 新增 WTG-026 / AC-WTG-013；定义 SCM master、Project authorization link、Run nullable binding、Host root registration 的 owner 与 fail-closed 验收；修订 v6.93 | 已编辑 |
| `docs/requirements/SRS-WORKTREE-CANVAS-001.md` | 升至 v1.5；增加 WLD-1 身份来源需求补充，明确旧 126 FR/NFR 计数不含新增跨领域要求 | 已编辑 |
| `docs/basic-design.md` | 升至 v6.96；新增 §16.77 owner/身份边界、当前审计差距和运行准入门 | 已编辑 |
| `docs/design/BD-WORKTREE-CANVAS-001.md` | 升至 v1.5；增加 D-REPO-001 与 §1.6 Repository/root 所有权决策 | 已编辑 |
| `docs/design/DD-WORKTREE-GROUP-001.md` | 升至 v5.18；增加 §6.1.3 候选查询、建议性 binding API、root generation 与副作用前授权流程 | 已编辑 |
| `docs/design/TRACEABILITY-WORKTREE-CANVAS-001.md` | 升至 v1.3；新增 AC-WTG-013 与 WTG-026 独立 trace addendum；将旧 43 AC 统计标注为历史统计 | 已编辑 |
| 本报告 | 记录本切片目的、证据、限制和门禁 | 新增 |

## §2 验证摘要

- 基线 HEAD 与分支在开始时分别为 `ac73efe51897522215f96559e33e716ad551577d` 与 `codex/wbs-wld1-repository-contract`；初始工作树干净。
- 已复核已合入的 WLD-1 repository directory audit 与六份 manifest 文档；父级提供的源证据包括：无 canonical PostgreSQL Repository master/adapter、Run repository identity 可空、无持久 Host root registry、进程内 path map 不是 SoR、lifecycle provider 默认未安装且缺失返回 503。
- Markdown 目标检查：变更行中涉及的 16 个相对链接均解析到现存文件；新增 WTG-026 / AC-WTG-013 与 §50.3.4 / §50.8AV / SRS §16 / BD §1.6 / Group DD §6.1.3 / Trace §16 交叉引用均存在。
- `rtk git diff --check`：通过，无空白/补丁格式错误。
- 改动范围检查：六个 manifest 文档 + 本报告；未编辑代码、迁移、WBS 或其他路径。
- 未运行测试、Cargo、数据库或运行时检查；本切片没有代码/迁移改动，不把设计描述当作实现验证。

## §3 已知缺口与外部交付依赖

1. SCM owner 尚无经证实的持久 Repository master 与生产查询 adapter；需补齐跨宿主稳定 RepositoryId 和无 Branch repository 的完整列举来源。
2. Project↔Repository association 的权威持久存储、revision/current-state、审计及可撤销授权 writer 尚未证明。
3. Engineering Run Directory 的 nullable Run→Repository binding 写 API、CAS/幂等、Audit/Outbox、授权 grants/RLS 及迁移仍需实现/验证。
4. Host Infrastructure owner 的持久 HostEnvironment + RepositoryId + generation root registry、path verification/currentness/revocation 和最小访问接口尚未实现。
5. 新的 Run-scoped candidate/binding API 与 Worktree provider composition 未实现。建议接口路径在详细设计中明确标为 proposal；不得误认为路由已存在。
6. 未来至少需要 disposable PostgreSQL 与真实 HTTP 负向验收：Project/Run 授权、跨 Project/tenant、NULL binding、撤权、旧 generation、root path 不泄漏、provider 缺失及重放/并发。
7. 迁移期间 Project-scoped repository-list/create/import seam 保持兼容描述；实际 Worktree create/import/CLI 仍受显式 Run binding、trusted root registration 与 provider readiness 门控，缺失时 fail closed。

## §4 子代理失败接手清单

本 lane 未派发子代理。没有未接手的子代理产出或失败项；未改 WBS、代码、migration 或其它 lane 文件。

## §5 守门规则核对

| # | 核对项 | 结果 |
|---|---|---|
| 1 | 仅编辑 brief manifest 六份文档并新增指定报告 | 通过；最终 status 仅含指定七个路径 |
| 2 | 保留 Project→Run→Worktree→Branch 与 hostless Run | 通过；NULL Repository 仅阻断 checkout/Worktree 执行，不隐藏 Run |
| 3 | SCM master、Project authorization、Run binding、Host root、Worktree checkout 的身份边界独立 | 设计合同已落档；实现未验收 |
| 4 | Project candidate visibility 不等于 Run binding 或 Worktree capability | 通过；两者分为授权读取与显式 owner mutation |
| 5 | Run-scoped candidate query 检查当前 Project membership + Run read grant | 设计合同已落档；API 未实现 |
| 6 | repository_id NULL 不从 Project default、remote Branch 或旧 Worktree 推断 | 通过 |
| 7 | raw host path 不进入浏览器、Agent、Plugin 或任务卡 CLI | 通过；来源只由 Host capability 解析 |
| 8 | 缺 root/provider/grant 时 fail closed | 通过；当前 provider 缺失 503 的源证据已注明 |
| 9 | 报告不把 provider、schema、migration 或 production DB 验收说成已完成 | 通过 |
| 10 | 文档链接、ID、版本和追踪关系同步 | 通过；16 个变更相对链接、WTG-026 / AC-WTG-013 和目标章节引用均已复核 |
| 11 | 不运行无关测试或数据库操作 | 通过；本任务为 docs-only |
| 12 | 只提交 manifest 与报告；不 rebase/merge/push/cleanup | 通过；仅显式暂存 brief 的七个路径，集成由协调者负责 |
| 13 | 新增面向产品的文案不包含竞争产品品牌或仿制宣传 | 通过；本切片新增文本使用中性领域名称 |
| 14 | 外部身份/授权来源依赖明确列出，不虚构 Star 内部 owner | 通过；SCM、Project、Run Directory、Host Infrastructure 分责并标注待交付来源 |
| 15 | 主导航与 hostless Run 既有决策保持一致 | 通过；Project → Run → Worktree → Branch 顺序及 nullable Run binding 已复核 |
| 16 | 不把隔离审计/fixture 结论扩大为目标数据库或生产运行事实 | 通过；未连接数据库/Host Runtime，所有实现依赖仍标开放 |
| 17 | 当前分支只交付本 WLD-1 设计合同，不替协调者改 WBS 或处理其他 lane | 通过；未编辑 WBS、代码或其他 lane 路径 |

## §6 签字栏

| 角色 | 审阅结论 |
|---|---|
| 架构 | Mavis 接手审核：owner 边界与 hostless Run 语义已补齐；source gap 仍开放 |
| SRE Lead | Mavis 接手审核：未连接目标 PostgreSQL/Host Runtime，不声称部署验证 |
| 平台 | Mavis 接手审核：candidate/binding/provider seam 明确，写侧仍 fail closed |
| 评审主持 | Mavis 接手审核：设计、需求与追踪已同步；实现验收待后续 WBS lane |
| PM | Mavis 接手审核：本切片不标记 Worktree create/import/CLI 商用闭环可用 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容与触发 |
|---|---|---|---|
| v0.1 | 2026-10-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 按 WLD-1 Repository Directory source audit 补齐 SCM/Project/Run/Host identity contract、候选与 binding 差异和 fail-closed 验收；仅 docs-only，不声称实现或数据库部署 |
