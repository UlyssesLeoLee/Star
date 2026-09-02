# STAR-P3-5-DOMAIN-LEAD-REGISTRY 5 域 Lead 真人注册 (per 8/21 JST 拒绝兼任硬约束)

> **Status**: 🟡 Draft v0.1 (等 Ulysses 找 5 个真人, 每人认领 1 域)
> **Created**: 2026-08-30 10:45 JST
> **Authority**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代签 (per 8/27 19:39 JST 用户授权)
> **承接**: STAR-P3-5-DOMAIN-LEAD-PROC.md v0.2 步骤 2 (per `afe8dcb` commit, 2026-08-30 09:01 JST) + STAR-P3-5-DOMAIN-LEAD-SELECTION-RESULT.md 选项 4 应急 (per `ec6dee0`)

本文件是 5 域 Lead 真人注册模板. 等 Ulysses 找 5 个真人后, 把 5 行的姓名/邮箱/角色/到岗日期填上, 签字栏追溯覆盖应急代签 (per `ec6dee0` 选项 4 应急架构师代签).

---

## §0 背景

P3 全 5 阶段 60/65 拍板完成 (per `ec8131a`), 56/64 子项实质收官 87.5% (per 当前 main HEAD `65c43e7`), 8 子项卡 5 域 Lead 真人到位:

- **P3-C.9** 5 域 Lead 真人到位 (per C 拍板 9/9 子项, 跨 session 续)
- **P3-E.5** 5 域 Lead 真人到位 (per E 拍板 7/7 子项, 跨 session 续)
- **P3-F.1** 5 域 Lead 真人到位 (per F 拍板 5/6 子项, 跨 session 续)
- **E.6** 5 域 Saga 实装 (per match 域 Lead 真人补详细补偿机制)
- **E.7 code review** (5 域 Lead review 6 份 P3 报告 + DDD 边界 docs)
- **F.2 真人 review** (5 域 Lead review frontend 5 域 marker + 真实 e2e 跑通)
- **F.5** 质量门 5/5 实证 (DDD Review 阶段 5 角色真人到位)
- **INC-SESSION-005 终审** (P3 全 5 阶段 5 角色签字 + 5/5 实证)

**硬约束 (per 8/21 JST Ulysses 拍板)**: 不接受兼任 — 架构师不能兼任 player Lead, SRE 不能兼任 admin Lead. 5 个真人 = 5 个独立个体.

---

## §1 5 域 Lead 真人注册表

| 域 | Lead 姓名 | 邮箱 | 角色 | 域边界 docs | 到岗日期 | 状态 |
|---|---|---|---|---|---|---|
| **player** (用户/identity/workspace) | `<待填>` | `<待填>` | Player Lead | `docs/ddd/01-player-bc.md` | `<YYYY-MM-DD>` | 🟡 待到岗 |
| **economy** (billing/pricing/cost) | `<待填>` | `<待填>` | Economy Lead | `docs/ddd/02-economy-bc.md` | `<YYYY-MM-DD>` | 🟡 待到岗 |
| **match** (workflow/状态机/saga) | `<待填>` | `<待填>` | Match Lead | `docs/ddd/03-match-bc.md` | `<YYYY-MM-DD>` | 🟡 待到岗 |
| **social** (collaboration/通知) | `<待填>` | `<待填>` | Social Lead | `docs/ddd/04-social-bc.md` | `<YYYY-MM-DD>` | 🟡 待到岗 |
| **admin** (RBAC/permission/tenant) | `<待填>` | `<待填>` | Admin Lead | `docs/ddd/05-admin-bc.md` | `<YYYY-MM-DD>` | 🟡 待到岗 |

**填写示例** (per `STAR-LEAD-ROSTER.md` 4KB 模板, 5 域 Lead 真实身份采集模板, 8/28 落地):

```markdown
| **player** (用户/identity/workspace) | 张三 | zhangsan@example.com | Player Lead | docs/ddd/01-player-bc.md | 2026-09-01 | 🟢 到岗 |
```

---

## §2 5 域 Lead 真人到位流程 (per `STAR-P3-5-DOMAIN-LEAD-PROC.md` v0.2)

### 步骤 1: Ulysses 找 5 个真人 (跨 session)

- **方法 A (推荐)**: Ulysses 个人网络 / 公司内部 5 个工程师, 每人认领 1 域, 签署 DDD Review 协议
- **方法 B (备选)**: 通过 freelance 平台 (e.g. Toptal / Upwork) 找 5 个 Rust 工程师
- **方法 C (备选)**: 开源社区招募

### 步骤 2: 5 域 Lead 注册 (本文件 §1 表)

填入本文件 §1 表的 5 行, 落地 1 commit (`docs(governance): STAR-P3-5-DOMAIN-LEAD-REGISTRY.md 5 域 Lead 真人到位`).

### 步骤 3: 5 域 Lead review 域边界 docs (per `STAR-P3-5-DOMAIN-LEAD-PROC.md` v0.2 §3 步骤 3)

5 域 Lead 各 review 1 域 docs (`docs/ddd/0X-*.md`), 6 章节:
1. §1 BoundedContext 定义
2. §2 Aggregate 字段 + 命令 + 事件
3. §3 跨域事件
4. §4 Cargo crate 引用
5. §5 已知缺口 (字段类型/索引/约束/ABAC conditions 等)
6. §6 签字栏 #1 追溯签字

**审阅时间预算**: 每域 30-60 分钟, 5 域 = 2.5-5 小时 (per STAR-OLU-001 §6 质量门 5 维).

### 步骤 4: 5 域 Lead review 6 份 P3 报告 (per `STAR-P3-5-DOMAIN-LEAD-PROC.md` v0.2 §4 步骤 5)

5 域 Lead review 6 份 P3 阶段收官报告 (签字栏 #1 追溯覆盖架构师代签):
- `PHASE-P3-C1-IMPL-REPORT.md` (5KB)
- `PHASE-P3-C2-C5-IMPL-REPORT.md` (5.7KB)
- `PHASE-P3-C6-C8-IMPL-REPORT.md` (5.4KB)
- `PHASE-P3-D1-D7-IMPL-REPORT.md` (5.2KB)
- `PHASE-P3-E1-E4-IMPL-REPORT.md` (6.1KB)
- `PHASE-P3-F1-F5-IMPL-REPORT.md` (6.6KB)

### 步骤 5: 跨 session 续做 + 真人到位验收 (per `STAR-P3-5-DOMAIN-LEAD-PROC.md` v0.2 §4 步骤 5)

5 域 Lead 到岗后, 11 wt 并行实装可以接入:
- 5 域业务子域 (P3-C.1-C.5) 推进时, 真人签字 docs
- 跨域 Saga (P3-C.6 / E.6) 真人主持跨域 review
- 跨域 E2E (P3-F.2) 真人跑测试验收

---

## §3 已知缺口 (per 缺标比错标)

| # | 缺口 | 移交 |
|---|---|---|
| 1 | 5 域 Lead 真人姓名/邮箱/角色 5 行待填 (per §1 表) | Ulysses 找 5 个真人, 每人认领 1 域 |
| 2 | DDD Review 阶段 5 角色真人到位 (架构 / SRE / 平台 / 评审 / PM), 当前全部架构师代签 (per `ec6dee0`) | 5 域 Lead 真人到位后, 5 角色真人补 |
| 3 | E.6 Saga 详细补偿机制 (per match 域 Lead 真人补) | match 域 Lead 真人到位后 |
| 4 | F.2 真实 e2e (5 域 Lead 真人 review + dev server 启动) | 5 域 Lead 真人到位后 |

---

## §4 签字栏 (5 角色 + 5 域 Lead)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 2026-08-30 | 🟡 Draft v0.1; 5 域 Lead 真人注册模板落地, 等 Ulysses 找 5 个真人 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 6 | player 域 Lead | `<待到岗>` | `<待签>` | 🟡 待真人到位追溯签字 |
| 7 | economy 域 Lead | `<待到岗>` | `<待签>` | 🟡 待真人到位追溯签字 |
| 8 | match 域 Lead | `<待到岗>` | `<待签>` | 🟡 待真人到位追溯签字 |
| 9 | social 域 Lead | `<待到岗>` | `<待签>` | 🟡 待真人到位追溯签字 |
| 10 | admin 域 Lead | `<待到岗>` | `<待签>` | 🟡 待真人到位追溯签字 |

---

## §5 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 5 域 Lead 真人注册表 (5 行待填) + 5 步流程 (per STAR-P3-5-DOMAIN-LEAD-PROC v0.2) + 签字栏 5 角色 + 5 域 Lead 待到岗 | 2026-08-30 10:45 JST Ulysses 指令"全做" 5 套推进触发 |
