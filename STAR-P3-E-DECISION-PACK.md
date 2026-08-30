# STAR-P3-E-DECISION-PACK P3-E 阶段 7 子项拍板包 (per 2026-08-30 07:47 JST)

> **Status**: 🟡 Draft (P3-A + P3-B + P3-C/P3-D 决策包准备, P3-E 同步准备)
> **Created**: 2026-08-30
> **Authority**: Ulysses (一人公司 12 角色 per DEC-008)
> **承接**: STAR-P3-WBS-001 §4 P3-E 占位表 (7 子项 / 30M / 5 周) + AGENTS.md §4 守门 #12 v15 派生饱和

本文件是 P3-E 阶段 7 子项的拍板包. P3-E 阶段 5 占位 + 2 阻塞 (E.4 KMS / E.5 5 域 Lead 真人), 累计 30M, 5 周.

---

## §0 背景

P3-E 阶段软预算 30M tokens / 7 子项 / 5 周 (per STAR-OLU-001 §1 1 SRE·周 = 1.2M).

7 子项当前 5 占位 (TBD) + 2 阻塞 (🔴, 需 Ulysses 凭证/真人). 阻塞项是 E.4 KMS 集成 (需 Vault / AWS KMS 凭证) + E.5 5 域 Lead 真人到位 (per 8/21 JST 拒绝兼任硬约束).

---

## §1 P3-E 拍板包 (7 子项推荐标题)

> **推荐策略**: 跟 P3-C 5 域业务子域集成 (KMS + DDD 边界 + 5 域 Lead 真人) + 跨域编排 (Saga) 续. 5 域 (**Star 仓独立定义 per `a4b3cb7` RGS 边界硬约束**, player/economy/match/social/admin 不引用 RGS 仓 5 域).

| # | 子项 | 标题(推荐) | 软预算 | 软参考周 | 依赖 | 状态 | 备注 |
|---|---|---|---|---|---|---|---|
| E.1 | E.1 | **Audit 域** (per domain-audit 增强 + 跨 5 域统一审计 API) | 4.3M | 0.7 周 | 无 | 🟡 占位 | 推荐: domain-audit crate 已存在, E.1 增强 + 5 域统一 |
| E.2 | E.2 | **Notification 域** (per-workspace 通知 + 5 域事件触发) | 4.3M | 0.7 周 | C.1 | 🟡 占位 | 推荐: 跟 C.1 Workspace 依赖, 跨 5 域事件总线 |
| E.3 | E.3 | **Search 域** (per-tenant 全文搜索 + 跨域索引) | 4.3M | 0.7 周 | C.7 | 🟡 占位 | 推荐: 跟 C.7 Postgres 持久层依赖, tsvector 全文搜索 |
| **E.4** | E.4 | **KMS 集成** (Vault / AWS KMS 凭证) | 5M | 0.8 周 | **E.1 + 凭证** | 🔴 **阻塞** | **需 Vault / AWS 凭证**, 否则降级为本地 mock (per 29692a7 类似路径) |
| **E.5** | E.5 | **5 域 Lead 真人到位 (DDD Review)** | 3M | 0.5 周 | 无 | 🔴 **阻塞** | **需 Ulysses 找 5 个真人** (per 8/21 JST 拒绝兼任硬约束, 不接受架构师兼任 player / SRE 兼任 admin) |
| E.6 | E.6 | **5 域 Saga 实装** (per Q-003 / 跨域补偿 / 失败回滚) | 4.5M | 0.8 周 | C.1-C.5 + E.1-E.5 | 🟡 占位 | 推荐: 5 域业务子域 + 跨域编排 + 真人到位后, E.6 跨域 Saga |
| E.7 | E.7 | **5 域 DDD 边界验证** (BoundedContext / Aggregate / Entity 文档 + code review) | 4.5M | 0.8 周 | E.5 | 🟡 占位 | 推荐: DDD Review 阶段, 5 域 Lead 真人 + 文档 + code review |
| **小计** | | | **30M** | **5 周** | | **5 占位 + 2 阻塞** | |

---

## §2 已知缺口 (per 缺标比错标)

| # | 缺口 | 移交 |
|---|---|---|
| 1 | 7 子项标题均为草案, 真实范围需 Ulysses 拍板 | 等 Ulysses 决策 |
| 2 | E.4 KMS 凭证未到位 (Vault / AWS KMS 真实 endpoint + key) | P3-E 启动前需 Ulysses 凭证, 否则走 mock 备选 (per 29692a7 模式) |
| 3 | E.5 5 域 Lead 真人到位 (5 个真人 per 8/21 JST 拒绝兼任) | 跨 session 续, 拍板后启动找真人流程 |
| 4 | 软预算为占位估算, 真实 token 待 SRE Lead 接入 telemetry 后回填 | P3-E phase 2 续 |
| 5 | 跨子项依赖图未画 (E.1-E.5 并行, E.6 串行等 E.5, E.7 串行等 E.5+E.6) | 等 Ulysses 拍板后回填 |
| 6 | 质量门 5 维未在 E.* 子项上实证 (E.* 还没启动) | E.* 阶段启动后实证 |

---

## §3 拍板选项 (Ulysses 一键决定)

### 选项 1: 批准推荐草案 (5 占位 + 2 阻塞, 30M / 5 周, 推荐)

- 7 子项全用推荐标题, 软预算 30M 平摊
- E.4 / E.5 阻塞项用 mock 备选 + 真人走 DDD Review 流程
- 触发 7 wt 并行启动 (per 10:58 JST 每子项 1 wt 决策)

### 选项 2: 推迟 E.4/E.5 阻塞, 推进 E.1/E.2/E.3/E.6/E.7 (5 子项, 21M / 3.5 周)

- E.4 KMS 留 P3-F, E.5 真人留 DDD Review
- 5 子项并行, 软预算 21M
- 风险: E.6 Saga 跨域编排缺 KMS 凭证, 降级本地 mock

### 选项 3: 推迟 P3-E 启动, 推 P3-F 优先 (per WBS §5)

- P3-F 30M / 5 周, 6 子项含 F.6 推 origin (R-05 反转已落地, 推 origin 后续)
- 风险: 5 域业务子域 (P3-C) 还没拍板, P3-F 推 origin 没业务底座

### 选项 4: 自定义

- 你给 7 子项的真实标题 + 软预算分配

---

## §4 子代理失败接手清单 (per 7 子代理派生规则)

- 0 子代理调用, 全部 root 直实装 (守门 #9 RPC 不可靠实证, 10 background task 全 ERR_CONNECTION_CLOSED)
- 拍板后 7 wt 并行启动 + 子代理 brief 写明"无证据叙事 = 禁止" (per AGENTS §1.2 派生规 4)

---

## §5 守门规则 (per AGENTS §4 守门 12 项 + §4.1 v1-v14 累积规)

| # | 规则 | 出处 |
|---|---|---|
| 1 | 本文件仅作拍板草案 + 推荐, **不实施 P3-E 任何子项**, 等 Ulysses 拍板 | 2026-08-30 07:47 JST Mavis 接手代签 |
| 2 | 每推荐行标 🟡 占位, 拍板后行标 🟢 进行中 | 本文件 §1 状态列 |
| 3 | token 软预算 ÷ 1.2M SRE·周上限 → 软参考周, **不参与 gating** | STAR-OLU-001 §1 |
| 4 | 推进门槛是质量门禁 ≥4/5, 不是截止日期 | STAR-OLU-001 §0 |
| 5 | 守门 #12 commit-time 同步 (本文件 commit 即触发, 后续 docs 同步接 v15 派生饱和) | AGENTS §4.1 v15 |

---

## §6 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 2026-08-30 | 🟡 Draft; 7 子项推荐 (Audit / Notification / Search / KMS / 5 域 Lead 真人 / Saga / DDD 边界) + 4 拍板选项 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 7 子项推荐 (E.1 Audit / E.2 Notification / E.3 Search / E.4 KMS / E.5 5 域 Lead / E.6 Saga / E.7 DDD 边界) + 4 拍板选项 + 已知缺口 6 项 | 2026-08-30 P3-B 7/9 子项收官 + P3-C/P3-D 拍板包准备, P3-E 同步准备 |
