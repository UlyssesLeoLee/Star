# STAR-P3-5-DOMAIN-LEAD-REVIEW-PROTOCOL 5 域 Lead review 协议 (6 章节 × 6 份报告 = 36 项 review)

> **Status**: 🟡 Draft v0.1 (等 5 域 Lead 真人到位, 按本协议 review 6 份 P3 报告 + 5 域 DDD 边界 docs)
> **Created**: 2026-08-30 10:45 JST
> **Authority**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代签
> **承接**: STAR-P3-5-DOMAIN-LEAD-PROC.md v0.2 §3 步骤 3 真人到位 review 模板 + STAR-P3-DDD-REVIEW-PHASE.md §2 步骤 3

本文件是 5 域 Lead 真人到位后的 review 协议. 6 章节 review checklist × 6 份 P3 报告 = 36 项 review 项. 每域真人 30-60 分钟, 5 域 = 2.5-5 小时 (per STAR-OLU-001 §6 质量门 5 维).

---

## §0 背景

P3 全 5 阶段 56/64 子项实质收官 87.5% (per 当前 main HEAD `65c43e7`). 6 份 P3 报告签字栏全部架构师代签 (per `ec6dee0` 选项 4 应急). 5 域 Lead 真人到位后, 按本协议 review 6 份 P3 报告 + 5 域 DDD 边界 docs 5 份, 签字栏 #1 追溯签字覆盖应急代签, 提升 P3 阶段从 4/5 质量门到 5/5 (per `STAR-P3-DDD-REVIEW-PHASE.md` §3).

---

## §1 Review 6 章节 (per 1 份报告)

每份 P3 报告 7 段结构 (§0 目的 / §1 改动矩阵 / §2 验证摘要 / §3 已知缺口 / §4 子代理失败接手清单 / §5 守门规则 / §6 签字栏 / §7 修订历史), 真人 review 重点关注 6 章节 (skip §7 修订历史因为是历史 meta):

### 1.1 §0 目的

- [ ] **目标明确**: 报告承接哪个拍板包 / 拍板结果
- [ ] **触发事件**: 跨 session 续做 / Ulysses 拍板 / 子项收官触发
- [ ] **范围清晰**: 子项清单 (batch 收官 vs 单子项)

### 1.2 §1 改动矩阵

- [ ] **文件路径正确**: `crates/domain-*/src/lib.rs` / `frontend/src/app/projects/page.tsx` 等具体路径
- [ ] **行数与改动匹配**: 新增行 / 改动行 vs 报告声称一致
- [ ] **commit short hash 实证**: 每子项有 `commit XXXXXX` 短码作为证据
- [ ] **Cargo crate 注册 / workspace members** (新增 crate 时)

### 1.3 §2 验证摘要

- [ ] **守门 #1 cargo check**: 0 err, 跨 stage 缓存命中
- [ ] **守门 #1 tsc --noEmit**: 0 错 (主仓已实证)
- [ ] **守门 #1 release cargo test**: 41/41 crate 0 fail
- [ ] **守门 #1 域内 cargo test**: 新 crate 单测 100% pass
- [ ] **守门 #9 author + secret 实证**: author=Ulysses, secret 0 hit
- [ ] **守门 #12 docs 同步 6 维度**: PHASE 报告 + AGENTS.md + WBS + README + CHANGELOG + docs/architecture

### 1.4 §3 已知缺口 (per 缺标比错标)

- [ ] **缺口列完整**: 列了真人到位 / 凭证 / 集成测试等阻塞项
- [ ] **缺口移交明确**: 跨 session 续做 / Ulysses 找真人 / 等真凭证
- [ ] **不掩盖**: 真人到位 / 凭证缺失 / 测试未跑 都明确列
- [ ] **不夸大**: 已知缺口 vs 已落地严格区分

### 1.5 §4 子代理失败接手清单 (per 7 子代理派生规则)

- [ ] **0 子代理调用**: 全部 root 直实装 (守门 #9 RPC 不可靠实证)
- [ ] **git log --follow 实证**: 不 commit 散落子代理产出
- [ ] **git 实证可查**: 守门 #1+#9+#12+#8+#15 跨 stage 全过

### 1.6 §6 签字栏

- [ ] **5 角色完整**: 架构 / SRE / 平台 / 评审 / PM
- [ ] **签字日 2026-08-30**: 跨 session 续做日期
- [ ] **签字人架构师代签 (Mavis 接手)**: per `ec6dee0` 选项 4 应急
- [ ] **签字栏 #1 追溯签字覆盖**: 真人到位后签名替换 "架构师代签"

---

## §2 Review 6 份 P3 报告 (6 章节 × 6 份 = 36 项)

### 2.1 PHASE-P3-C1-IMPL-REPORT.md (5.3KB, C.1 Workspace 域 收官)

- [ ] §0: 承接 STAR-P3-C-DECISION-PACK.md C.1 拍板, 触发 2026-08-30 08:18 JST
- [ ] §1: `crates/domain-workspace/src/lib.rs` 增强, commit `f93d909` 实证
- [ ] §2: cargo check 0 err, tsc 0 错, release cargo test 41/41 crate 0 fail
- [ ] §3: 列 5 域 Lead 真人到位 / Postgres 持久层 / 跨域 Saga 等缺口
- [ ] §4: 0 子代理调用, root 直实装
- [ ] §6: 5 角色签字栏 + 架构师代签 (per `ec6dee0`)

### 2.2 PHASE-P3-C2-C5-IMPL-REPORT.md (5.7KB, C.2-C.5 4 子项 batch)

- [ ] §0: 承接 C.2-C.5 拍板, 触发 2026-08-30 08:27 JST
- [ ] §1: `domain-project` + `domain-identity` + `domain-work-item` + `domain-workflow` 4 crate 增强, commit `81de99a`
- [ ] §2: cargo check 0 err (9.95s 缓存命中)
- [ ] §3: 列 5 域 Lead / 4 域实装跨 crate 集成 / Saga 跨域编排 等缺口
- [ ] §4: 0 子代理调用
- [ ] §6: 5 角色 + 架构师代签

### 2.3 PHASE-P3-C6-C8-IMPL-REPORT.md (5.4KB, C.6-C.8 3 子项 batch)

- [ ] §0: 承接 C.6-C.8 拍板, 触发 2026-08-30 08:30 JST
- [ ] §1: `star-saga` + `infrastructure` + `domain-tenant` 3 crate 增强, commit `25d086e`
- [ ] §2: cargo check 0 err
- [ ] §3: 列 5 域 Lead / 跨域 Saga / DDD 边界 等缺口
- [ ] §4: 0 子代理调用
- [ ] §6: 5 角色 + 架构师代签

### 2.4 PHASE-P3-D1-D7-IMPL-REPORT.md (5.2KB, D.1-D.7 7 子项 batch)

- [ ] §0: 承接 D.1-D.7 拍板, 触发 2026-08-30 08:32 JST
- [ ] §1: w28 切 HubCliRuntime 入口 + 跨平台 e2e + Playwright + realFetch wrapper + 3 handler real-mode + markdownlint+cargo doc CI + UserMenu 状态条 7 子项, commit `8ace1d5` + merge `55006a0`
- [ ] §2: cargo check 0 err (8.38s, 19 warning pre-existing)
- [ ] §3: D.2 跨平台 e2e + D.6 markdownlint + cargo doc CI 真实 runner 配置 stub
- [ ] §4: 0 子代理调用
- [ ] §6: 5 角色 + 架构师代签

### 2.5 PHASE-P3-E1-E4-IMPL-REPORT.md (6.1KB, E.1-E.4 4 子项 batch)

- [ ] §0: 承接 E.1-E.4 拍板, 触发 2026-08-30 08:36 JST
- [ ] §1: `domain-audit` + `domain-notification` + `domain-search` 3 域 + `crates/domain-kms` 新建 (LocalMockKms + 5 不变量 + 3 单测), commit `5ea9611` + merge `d2e2a99`
- [ ] §2: cargo check 0 err (0.80s cache 命中, 42/42 crate)
- [ ] §3: E.4 KMS 真凭证路径 / E.5 真人 / E.6 Saga / E.7 DDD 边界 等缺口
- [ ] §4: 0 子代理调用
- [ ] §6: 5 角色 + 架构师代签

### 2.6 PHASE-P3-F1-F5-IMPL-REPORT.md (6.6KB, F.2-F.5 4 子项 batch)

- [ ] §0: 承接 F.2-F.5 拍板, 触发 2026-08-30 08:55 JST
- [ ] §1: `frontend/e2e/cross-domain-5b.spec.ts` 3 Playwright test + `CHANGELOG.md` + `docs/architecture/cross-domain-5b-mermaid.md` + `docs/governance/P3-quality-gate-5d.md` 4 deliverable, commit `6c1bd6c` + merge `93512a9`
- [ ] §2: cargo check 0 err (0.48s cache 命中, P3-F 不增新 crate)
- [ ] §3: F.1 5 域 Lead 真人 / F.2 真实 e2e 需 5 域 Lead 真人 + dev server / F.5 质量门 5/5 实证 等缺口
- [ ] §4: 0 子代理调用
- [ ] §6: 5 角色 + 架构师代签

---

## §3 5 域 Lead review 5 域 DDD 边界 docs (6 章节 × 5 份 = 30 项)

### 3.1 `docs/ddd/01-player-bc.md` (7.4KB, player 域)

- [ ] §1: BoundedContext 业务子域 + Aggregate Root 划分合理
- [ ] §2: User / Workspace / Device 3 Aggregate 字段类型 / 索引 / 约束
- [ ] §3: 7 pub + 3 sub 跨域事件 schema + at-least-once / exactly-once 投递
- [ ] §4: Cargo crate 引用 (domain-identity / domain-workspace / domain-tenant) 散落 vs 独立 crate 拍板
- [ ] §5: 已知缺口 #1-#4 完整 (INV-ID-02 3-tuple 详细字段 / Device 三重绑定 / 跨域事件投递 / 域边界划分)
- [ ] §6: 签字栏 #1 player 域 Lead 真人签字 (覆盖 "架构师代签")

### 3.2 `docs/ddd/02-economy-bc.md` (9.2KB, economy 域)

- [ ] §1: BoundedContext 业务子域 + Aggregate Root 划分 (BillingAccount / Subscription / Invoice / ApiKey)
- [ ] §2: 4 Aggregate 字段类型 / 索引 / 约束
- [ ] §3: 7 pub + 4 sub 跨域事件 schema
- [ ] §4: Cargo crate 引用 (domain-project 含计费 / domain-cli 含 ApiKey 双模式) 散落 vs 独立 `domain-economy` crate 拍板
- [ ] §5: 已知缺口 #1-#6 完整
- [ ] §6: 签字栏 #1 economy 域 Lead 真人签字

### 3.3 `docs/ddd/03-match-bc.md` (8.8KB, match 域)

- [ ] §1: BoundedContext 业务子域 (workflow / 状态机 / saga 编排)
- [ ] §2: Workflow / WorkflowInstance / SagaInstance 3 Aggregate 字段类型 / 索引 / 约束
- [ ] §3: 7 pub + 5 sub 跨域事件 schema (含 WorkflowStarted 5 域协同)
- [ ] §4: Cargo crate 引用 (domain-workflow + domain-work-item + star-saga) 散落 vs 独立 `domain-match` crate 拍板
- [ ] §5: 已知缺口 #1-#5 完整 (含 E.6 Saga 详细补偿机制)
- [ ] §6: 签字栏 #1 match 域 Lead 真人签字 (含 E.6 详细补偿机制)

### 3.4 `docs/ddd/04-social-bc.md` (8.9KB, social 域)

- [ ] §1: BoundedContext 业务子域 (collaboration / 通知 / 评论)
- [ ] §2: Notification / Comment / Mention 3 Aggregate 字段类型 / 索引 / 约束
- [ ] §3: 7 pub + 12 sub 跨域事件 schema (NotificationDispatched 5 域全部监听)
- [ ] §4: Cargo crate 引用 (domain-notification + domain-comment + domain-collaboration) 散落 vs 独立 `domain-social` crate 拍板
- [ ] §5: 已知缺口 #1-#5 完整 (含 5 域 notification template 12 订阅事件文案 + i18n)
- [ ] §6: 签字栏 #1 social 域 Lead 真人签字

### 3.5 `docs/ddd/05-admin-bc.md` (10.3KB, admin 域)

- [ ] §1: BoundedContext 业务子域 (RBAC / permission / tenant)
- [ ] §2: Tenant / Permission / Role / KmsKey 4 Aggregate 字段类型 / 索引 / 约束
- [ ] §3: 8 pub + 8 sub 跨域事件 schema
- [ ] §4: Cargo crate 引用 (domain-tenant + domain-permission + domain-kms + domain-audit 跨域拥有) 散落 vs 独立 `domain-admin` crate 拍板
- [ ] §5: 已知缺口 #1-#6 完整 (含 E.4 KMS 真凭证路径 + ABAC conditions + Tenant isolation_mode + KMS 轮换策略)
- [ ] §6: 签字栏 #1 admin 域 Lead 真人签字 (含 E.4 KMS 真凭证)

---

## §4 Review 时间预算

- **6 份 P3 报告 × 6 章节 = 36 项 review 项**: 30 分钟 (每项 ~50 秒)
- **5 份 DDD 边界 docs × 6 章节 = 30 项 review 项**: 50 分钟 (每项 ~100 秒, docs 更密)
- **签字栏 #1 追溯签字覆盖应急代签**: 10 分钟
- **5 域 Lead 各自 review 自己的 1 域 + 跨域 1 域**: 5 域 × 80 分钟 = 6.7 小时

**总时间预算**: 7-8 小时 (5 域 Lead 协作, 可并行, 5 域 × 90 分钟/域 + 整合 1 小时)

---

## §5 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 2026-08-30 | 🟡 Draft v0.1; 5 域 Lead review 协议 6 章节 × 6 份报告 = 36 项 + 5 份 docs = 30 项, 共 66 项 review, 7-8 小时时间预算 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |

---

## §6 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 5 域 Lead review 协议 (6 章节 × 6 份报告 + 5 份 docs = 66 项 review 项) + 7-8 小时时间预算 | 2026-08-30 10:45 JST Ulysses 指令"全做" 5 套推进触发 |
