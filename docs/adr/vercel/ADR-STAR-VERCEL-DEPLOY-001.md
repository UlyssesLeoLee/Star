# ADR-STAR-VERCEL-DEPLOY-001 — Vercel as an Optional Web Hosting Path

> **状态：草案 v0.2，未批准；VCD-0 OPEN**
> **决策所有者：架构师（Mavis 接手审核）**
> **上游：** `docs/requirements/SRS-STAR-VERCEL-DEPLOY-001.md` v0.4；`docs/implementation-plans/WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md` v4.19
> **关联：** `docs/reviews/VCD0-BASIC-ADR-COMPATIBILITY-20261024.md`、`VCD0-DD-COMPATIBILITY-20261024.md`、`VCD0-TD-COMPATIBILITY-20261024.md`
> **日期：** 2026-10-24

## 1. 背景与问题

Star 需要评估 Web 呈现面是否可部署到 Vercel，同时避免把具有状态、权限、执行或本地资源责任的能力迁入无状态函数。Vercel Runtime、目标账号/套餐、数据可达性与生产环境隔离仍有待验证；本 ADR 记录的是待批准的架构边界，不是部署或运行证据。

## 2. 决策提案

**提议将 Vercel 作为可选的 Web 呈现与同源 BFF 托管路径。** 第一阶段只评估 Next.js 呈现面、受保护的 Preview/Production 发布与受约束的同源代理。Star 现有 Owner 仍是身份授权、业务事实、Schedule、Agent/Task 执行、Hook、BI、Canvas 实时协作、本地 Worktree 文件系统及 Multipass/K3s 操作的唯一权威。

Rust HTTP Functions 不是本阶段的默认部署范围。只有 VCD-2 在目标账号/套餐上完成隔离可行性验证，并通过 WLD-0 与相关 WLD-6 的 Owner/授权/持久化门后，VCD-3 才能逐个评审薄型、无状态、allowlist HTTP adapter。adapter 只能调用现有 Star Owner 契约；不得形成第二套业务 Owner、数据库权威、迁移权威、worker、scheduler 或 VM 执行面。

Vercel Preview 与 Production 必须隔离凭据、数据和副作用。未信任 Preview 不得访问生产密钥、数据、Owner 写入或 Schedule 触发权限。Preview 构建成功不代表业务集成可用；不支持或未验证的能力须明确禁用并 fail closed。

## 3. 考虑过的方案

| 方案 | 说明 | 结果 |
|---|---|---|
| 保持全部 Web 与执行能力只运行在现有 Star 环境 | 不增加 Vercel 部署路径 | 可作为 VCD-1 失败或隔离门未通过时的回退；无法验证用户提出的托管 Web 路径。 |
| Vercel 承载受保护的 Web 呈现面与同源 BFF；Rust HTTP adapter 另设条件门 | Web 与 Star 权威 Owner/执行面分离；Rust Functions 仅在独立验证和 Owner 评审后逐步考虑 | **本 ADR 提案。** 保持边界清晰并可按阶段停止。 |
| 将完整服务端、数据和执行面迁入 Vercel | 将业务 Owner、持久化、后台任务、Agent、长连接和基础设施操作交由平台函数承载 | 拒绝：与 SRS v0.4 及现有 Star Owner、运行时、Multipass/K3s、原生 Hook 和本地 Worktree 约束冲突，也缺少平台能力证据。 |

## 4. 结果与约束

### 正向结果

- Web 呈现可以独立验证和回滚，且不要求迁移 Star 的业务事实或执行面。
- 失败的 Rust Runtime 验证不会阻断受保护的前端托管路径，也不会自动触发迁移。
- 所有写入仍由 Star Owner 复验授权、在本 Owner 事务内持久化，并按既有 Outbox/Inbox 契约推进。

### 代价与风险

- BFF 与 Star API 之间增加网络、身份传递、区域和可用性依赖；VCD-1 必须验证真实 host-session 契约与 Run/Worktree 授权。
- 预览环境必须严格隔离；任何无法证明的凭据或数据隔离都阻止 Preview 访问生产 Owner。
- Rust Runtime 仍为 Beta；其专属 WebSocket/Streaming、资源限额和目标账号行为未证实。通用 Functions WebSocket GA 不能推导 Rust Runtime 支持。
- Cron 是 best-effort 唤醒信号，可能漏发、重复或重叠，失败不自动重试；它不是 Schedule 事实源或执行器。
- Vercel 并行保留 Star 现有 K8s/Helm/GitOps/Multipass 路径；本决策不删除、不改写或替换这些资产。

## 5. 强制决策门

1. **VCD-0：** 完成并批准与 SRS v0.4 对齐的 ADR、基本设计、详细设计与测试设计；在此之前不得将 Vercel 部署标为已批准或已完成。
2. **VCD-1：** 验证受保护前端 Preview/Production、真实登录和 Run 授权、同源 BFF、安全环境变量、构建门、Preview 无生产密钥/数据/副作用及回滚。
3. **VCD-2：** 只在无生产数据、无业务写入的隔离项目验证 Rust 1.99.0、构建根、包体、目标套餐资源、并发、RSS、冷启动、日志/指标、Streaming/WebSocket（若需要）、数据库连接/RLS 和回滚。未知或失败即保持 Rust 执行能力关闭。
4. **VCD-3：** 只有 VCD-2 PASS、VCD-1 完成、WLD-0 授权/Owner 契约通过，以及产生事件时相关 WLD-6 通过后，才评审逐个 allowlist 的薄型 Owner-backed HTTP adapter。每个 endpoint 需遵守现有 Owner 授权、幂等、持久回执、审计和 Outbox/Inbox 语义。
5. **VCD-4：** 对 Cron 漏发/重复/重叠/失败与回滚后的定义恢复执行对账；Star 的持久 Schedule 与 Owner 授权仍是权威。无人值守触发按固化的 `run_as_actor_id` 实时复验权限；每次触发/重试/恢复解析当时有效的 `RunInfrastructureBinding`。

不得从通用平台表推导 Rust 专属资源承诺，不得用 mock/进程内队列、部署成功或函数响应替代 Owner 持久回执或 Agent 执行验证。测试阶段 Multipass inventory 最多保留两个记录；必须通过带跨进程锁的受控 launcher 创建，inventory 未知/已满时 fail closed，不自动 purge，也不以 Docker Desktop 为前置条件。

## 6. 状态、证据与后续

本文是 VCD-0 的**未批准决策提案**。chat SSE 的最终路径尚未选择：SRS 允许受限 Next Route Handler 透传，其他路径须由对应 Star 服务契约明确支持；SRS/ADR/BD/DD/TD 和真实 session、授权/撤权、Origin/跨站防护、streaming、生命周期契约统一并验证前，该能力保持关闭。SRS/WBS 中定义的需求与门槛是追踪基线；平台资料陈述仅表示在 SRS 标注日期核对过的公开平台事实，不代表目标账号验证。部署、登录、Preview 隔离、Rust 构建/运行、数据库/RLS、Cron 和生产行为均 **NOT_RUN**。VCD-0 只有在本 ADR 与对应 BD/DD/TD 完成一致性评审并得到批准后才可关闭。

## 7. 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 基于 SRS v0.4 和三份 VCD-0 兼容性审查创建本决策提案；保留可选 Web/BFF 路径，排除未批准的第二 Owner/数据库/worker/scheduler/VM 执行架构；不声明批准或实测。 |
| v0.2 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将 chat SSE 最终路由及其真实 session/授权/Origin/streaming/lifecycle 契约列为 VCD-0 OPEN 决策门，明确 SRS 允许 Next Route Handler 并不代表已选定或验证；其他 VCD 门仍 NOT_RUN。 |
