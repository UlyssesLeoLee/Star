# PHASE-VCD0-BD-REVIEW-R2-20261024

> **结论：READY（仅可选择性纳入 ADR/基本设计草案）**
> **状态：VCD-0 OPEN；设计仍为 Draft；部署、运行时、账号及性能验收 NOT_RUN**
> **审阅基线：** SRS v0.4、WBS v4.18、三份 VCD-0 兼容性审查，以及候选 BD commit `06a66685` 和 ADR commit `bfbfcd69`
> **修订人：** Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> **日期：** 2026-10-24

## §0 目的

对 VCD-0 候选基本设计 v0.2 与 ADR v0.1 做独立第二轮文档审阅，核对两个已知 P2 设计问题、Star Owner/Preview 边界、需求 ID 和当前 WBS。结论只决定这两份草案是否可选择性纳入 dev 供后续评审；不表示批准方案、关闭 VCD-0 或提供部署/平台运行证据。本审阅未读取或修改 `D:\Star` 用户草稿。

## §1 改动与审阅矩阵

| 审阅项 | 候选证据 | 结论 |
|---|---|---|
| 无宿主 Engineering Run | BD §§1、1.2：第 32、63–68 行；Owner 表第 77 行；授权流程第 89 行；UI 行为第 133 行 | **通过。** Run 可在无远端宿主、repository 或 Branch 时创建、授权、列出和管理；远端关联是可空可变元数据，provider 操作才额外校验 provider 权限。Worktree 仍是 Run 下的本地 checkout。 |
| NFR-VC-01 延迟范围与可追踪性 | SRS v0.4 第 197 行；BD §§6.1.1、8.1：第 151–156、213 行 | **通过基本设计修复。** 设计明确 p95 < 2 s、BFF 到响应返回的测量边界、同步授权/Owner 事务范围、异步执行排除项、按操作族测量方式及 NOT_RUN 条件；没有把现有 AC 当成性能通过证据。 |
| Owner/执行面边界 | BD 第 12、26–31、46–58、74–79、92–98 行；ADR §§2、4：第 15–19、35 行 | **通过。** Star Owner 保留身份授权、业务事实、幂等、审计、回执和 Outbox；BFF 不代替 Owner、不直写数据库；Rust HTTP function 仅是经过独立门禁后的薄型 adapter。 |
| Preview 与 Production 隔离 | BD §§3.4、4：第 108–127 行；ADR 第 19、40、47–49 行 | **通过设计层。** 未信任 Preview 不得访问生产凭据/数据/副作用；缺少隔离证据时 fail closed。实际账号与环境隔离尚未验证。 |
| Worktree-first 和既有编号 | BD 第 61、68 行及 §8.1 第 201–218 行；SRS v0.4 第 145–204、222–259 行 | **通过。** 导航保持 Project → Engineering Run → Worktree → Branch；使用 SRS 已有 FR-VC-001..021、NFR-VC-01..08、AC-VC-001..013、GAP-01..11，并将 Star Run 约束单列；未见新建需求或验收编号。 |
| ADR 与 BD 一致性 | ADR §§2–5：第 15–53 行；BD §§1–5、8：第 24–140、189–218 行 | **未发现直接矛盾。** 两者都将 Vercel 限定为可选呈现/BFF 路径、保留 Star Owner 和执行权威、隔离 Preview，并将 Rust functions 置于 VCD-2/VCD-3 条件门后。ADR 没有复述 BD 的 hostless Run 细节或 NFR-VC-01 测量定义；该细节在基本设计中明确，不构成当前决策冲突。 |
| Draft、状态与运行证据 | BD 第 3–5、189–199、220–245 行；ADR 第 3–7、55–63 行 | **通过。** 两份文档标为草案/未批准、VCD-0 OPEN；实现、部署、账号、性能和 runtime 状态均未宣称通过。 |
| 产品命名 | 候选 BD 与 ADR 全文 | **通过。** 未发现对标/仿制文案或竞争产品名称；服务托管平台按实际名称说明。 |

## §2 验证摘要

- 已审阅候选 BD commit `06a66685e78fb0ad7772da4390ecf356d7bd8170`。该提交只修改 BD，将 hostless Run 约束、NFR-VC-01 范围与证据、架构约束追踪补入 v0.2；未改变 SRS 或验收编号。
- 已审阅 ADR `bfbfcd69` 中的 v0.1 草案，以及当前 dev 的 SRS v0.4、WBS v4.18 和 `VCD0-BASIC-ADR-COMPATIBILITY-20261024.md`。
- 这是文档审阅。未运行产品测试、部署、平台账号验证、Rust runtime 或性能测试；这些门保持 NOT_RUN。
- 本报告仅做聚焦的 diff/空白检查；未运行代码格式化、构建或测试。

## §3 已知缺口与后续项

1. **P2 — WBS 的 VCD-1 验收尚未显式写出 NFR-VC-01。** 候选 BD 第 155、213 行将 p95 操作族测量安排到 VCD-1/VCD-2；当前 dev WBS 第 57 行的 VCD-1 验收列出 AC-VC-001..006、013，但没有明确要求记录 p95。WBS 第 58 行的 VCD-2 验收已包含 latency。纳入设计时应同步补足 VCD-1 的适用操作族和 p95 证据门，不能因此提前把 NFR 标为通过。
2. **WBS 快照需随集成更新。** 当前 dev WBS 第 56 行仍记录 canonical ADR/BD/DD/TD 缺失。纳入 ADR/BD 后，应更新 VCD-0 当前证据；DD/TD 仍未完成前，VCD-0 继续 OPEN。
3. ADR 保持 v0.1 草案；其未逐字重复 BD 的 hostless Run 与 p95 细节。后续 ADR 修订可增加指向 BD §§1.2、6.1.1 的约束引用，避免两份设计长期漂移；当前没有发现互相冲突的陈述。
4. Owner API、host-session issuer/audience、真实 Preview 隔离、目标账号/套餐、Rust runtime 资格、数据库/RLS、p95 实测、Cron/rollback 行为和生产部署均未验证。VCD-0 不得据本报告关闭。

## §4 子代理失败接手清单

本 lane 是只读独立审阅；没有委派子代理，也没有子代理失败需要接手。仅本报告为本 lane 产出。

## §5 守门规则检查

| # | 守门项 | 结果 |
|---|---|---|
| 1 | 以当前 dev SRS/WBS 作为需求与阶段基线 | 通过；SRS v0.4、WBS v4.18。 |
| 2 | 候选来源按具体提交审查，不整分支照单接纳 | 通过；范围为 BD `06a66685` 和 ADR `bfbfcd69`。 |
| 3 | 远端宿主/Branch 不可成为 Engineering Run 身份或授权前提 | 通过；BD 第 32、65–68、77、89、133 行。 |
| 4 | 导航与所有权遵守 Project → Run → Worktree → Branch | 通过；BD 第 61、68 行。 |
| 5 | Star Owner 是业务授权和持久事实权威 | 通过；BD 第 26–30、74–79、92–98 行。 |
| 6 | BFF 不得代授权、直写业务表或创建第二 Owner | 通过；BD 第 74–79、92–98 行。 |
| 7 | Preview 与 Production 的密钥、数据及副作用隔离 | 设计通过；环境实测仍 NOT_RUN。 |
| 8 | 运行面缺少授权、Owner 契约或隔离证明时 fail closed | 通过；BD 第 27、89、98、114、120–125 行。 |
| 9 | 不以 mock、HTTP 202 或部署成功冒充业务完成 | 通过；BD 第 12、30、90、97、114 行。 |
| 10 | Rust HTTP functions 保持条件式、无状态、Owner-backed | 通过；ADR 第 17、47–50 行，BD 第 76、170–175 行。 |
| 11 | 不将 Agent、CLI、Hook、Canvas WSS 或 Worktree 文件系统迁入托管函数 | 通过；BD 第 28、56–58、76、135–140 行。 |
| 12 | 遵守 Multipass/K3s 的两记录上限及受控管理边界 | 通过；BD 第 53–58、140、177–179 行；ADR 第 53 行。 |
| 13 | NFR-VC-01 采用明确测量边界和专门证据 | 设计通过；实际测量 NOT_RUN；WBS VCD-1 gate 待同步。 |
| 14 | FR/NFR/GAP/AC 只引用 SRS 既有 ID | 通过；BD §8.1 第 205–218 行。 |
| 15 | 不把通用平台支持推导为 Rust 专属能力 | 通过；BD 第 20、172–175 行，ADR 第 41、49 行。 |
| 16 | 草案、VCD-0、运行时与部署状态准确标注 | 通过；均为 Draft/OPEN/NOT_RUN。 |
| 17 | 不读取/覆盖用户主工作区或无关用户数据 | 通过；未读取或修改 `D:\Star`。 |

## §6 签批栏

| 角色 | 结论/签批 |
|---|---|
| 架构 | Mavis 接手审核：READY for selective integration；不代表 ADR/BD 获批，VCD-0 保持 OPEN。 |
| SRE Lead | 部署、可用性与运行证据：NOT_RUN；本报告不作运行签批。 |
| 平台 | 账号、套餐与 Rust Runtime 证据：NOT_RUN；本报告不作平台签批。 |
| 评审主持 | 独立文档第二轮审阅完成；保留 §3 的 WBS 跟进项。 |
| PM | 不批准阶段关闭；DD/TD 与 VCD-0 验收仍待完成。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 第二轮核验 hostless Run 与 NFR-VC-01 修复；确认 BD/ADR 可选择性纳入，并记录 WBS 验收同步项；VCD-0 保持 OPEN。 | `WBS-VCD0-BD-REVIEW-R2-20261024` brief。 |
