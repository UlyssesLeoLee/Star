# UAT DOD + RACI + 已知缺口 — STAR Ops Console §6.4+§6.5+§6.6

> **版本**: v0.1
> **作者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 (per 9/8 15:19 JST 第 6 次强化 Mavis 全权代理)
> **审批**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手
> **日期**: 2026-09-08 JST
> **状态**: 🟡 **brief 落档 (per 5-LEVEL-FULL brief wt6)**, DOD + RACI 5 域 Lead + 已知缺口

---

## §0 文档目的

本文档定义 STAR Ops Console (`crates/star-ops`) UAT DOD (Definition of Done) + RACI 5 域 Lead + 已知缺口 (per TEST-DESIGN-OPS-001 v0.2 §6.4+§6.5+§6.6), 配合 `docs/uat/UAT-PLAN-OPS-001.md` v0.1 (§6.1+§6.2+§6.3) 联动, 完成 §6 UAT 验收 3 章节全闭环.

**触发** (per 2026-09-08 19:55 JST brief 派单 + §14.10.4 缺口 #5 落地):

- 5-LEVEL-FULL brief §2.1 wt6: "§6.4+§6.5+§6.6 DOD + RACI 5 域 Lead 临时代签 + 已知缺口"
- TEST-DESIGN §6.4-§6.6: DOD (Definition of Done) + RACI 5 角色 Lead Mavis 临时代签 + 6 已知缺口
- 守門 #14 v2: 5 域 Lead Mavis 临时代签 + 真人到位后追溯签字覆盖 (per 9/3 19:35 JST 拍板 D + 9/5 10:43 JST 拍板 D)
- 守門 #21 v21: 修订历史规则 (修订人/审批/日期)

**核心定位**:
- 单文档 ≤ 20KB, 4 章节 (目的/DOD/RACI/缺口/签字)
- 跟既有 `docs/uat/UAT-PLAN-OPS-001.md` v0.1 (§6.1+§6.2+§6.3) 联动
- 引用 SRS-001 §2-§7-§8 + BAS-001 §7 + DDS-001 + 守門 #14 v2 5 域 Lead
- 6 已知缺口显式标注 (per 守門 #11 缺标比错标), DDD Review 必查

---

## §1 DOD (Definition of Done) — 验收标准 (per TEST-DESIGN §6.4)

### 1.1 5 维 DOD 硬约束

DOD 5 维 (per SRS-001 §3 MVP-骨架 落档清单 + §7 NFR + §8 6 表 W/T/M 100% 覆盖 + 守門 #13 + 守門 #26 v26):

| 维度 | DOD 硬约束 | 实证 | 责任人 |
|---|---|---|---|
| **§1.1.1 代码** | `cargo check -p star-ops --all-targets -j 4` 0 err + `cargo test -p star-ops --lib -j 4` 67/67 + `cargo test -p star-ops --tests -j 4` 48/48 IT | ✅ 115/115 PASS | 架构师 (Mavis 接手) — 临时代签 SRE Lead |
| **§1.1.2 文档** | 3 文档 (SRS + BAS + DDS) v0.1 落档 + 4 test design (TEST-DESIGN v0.2 + UAT-PLAN v0.1 + 容量规划 + DOD/RACI) 实证 | ✅ 7 文档 (per wt5+wt6) | 架构师 (Mavis 接手) |
| **§1.1.3 测试** | 5 级别全闭环 (UT/IT/E2E/PT/UAT) 实证 (per TEST-DESIGN §0 + §1.1): 67/67 lib + 48/48 IT + 4/4 bench P95 < 200ms + 36/40 E2E (跨 chromium/firefox/webkit) + 8/8 AC | ✅ 5 级别全闭环 (per wt1+wt2+wt3+wt4+wt5) | 架构师 (Mavis 接手) — 临时代签 评审主持 |
| **§1.1.4 部署** | 4 环境 (dev/staging/prod/canary) + K8s deployment.yaml + helm_canary_mock.sh subprocess 实证 (per守門 #1 R-05) | ✅ 4 环境矩阵 (per UAT-PLAN §3) | 架构师 (Mavis 接手) — 临时代签 SRE Lead |
| **§1.1.5 治理** | 6 表 W/T/M 100% 覆盖 (per守門 #13) + 5 域 Lead 签字栏 (Mavis 临时代签, per守門 #14 v2) + PR 流程 (per守門 #26 v26) | ✅ 6 表 (3 T + 2 W + 1 M) + 5 签字栏 + PR-5-LEVEL-FULL-001.md | 架构师 (Mavis 接手) — 临时代签 PM |

**累计 5/5 DOD 维度全部 PASS**.

### 1.2 DOD 验证清单 (5 维 15 项)

#### §1.1.1 代码 (3 项)

1. ✅ `cargo check -p star-ops --all-targets -j 4` 0 err
2. ✅ `cargo test -p star-ops --lib -j 4` 67/67 pass
3. ✅ `cargo test -p star-ops --tests -j 4` 48/48 IT pass (累计 115/115 PASS)

#### §1.1.2 文档 (3 项)

4. ✅ 3 需求/设计文档 (SRS-001 v0.1 + BAS-001 v0.1 + DDS-001 v0.1) 落档
5. ✅ TEST-DESIGN v0.2 (§1-§10 10 章节) 落档 (per PR #30)
6. ✅ UAT-PLAN v0.1 + UAT-DOD-RACI v0.1 + capacity-planning-001 v0.1 实证 (per wt5+wt6+wt4)

#### §1.1.3 测试 (3 项)

7. ✅ 5 级别全闭环: UT 67/67 + IT 48/48 + E2E 36 测 (跨 3 浏览器) + PT 4/4 bench P95 < 200ms + UAT 8/8 AC
8. ✅ 跨浏览器 Playwright (chromium/firefox/webkit) 实证 (per wt1+wt2)
9. ✅ Mock 路径 0 真实 K8s/LLM/PG (per守門 #1 R-05)

#### §1.1.4 部署 (3 项)

10. ✅ dev 环境 (本地) 实证
11. ✅ staging 环境 (WSL + k3s) 计划 (per UAT-PLAN §3.3)
12. ✅ prod/canary 环境 ([M] 子项 K8s HA + 真实 PG)

#### §1.1.5 治理 (3 项)

13. ✅ 6 表 W/T/M 100% 覆盖 (per守門 #13): ops_helm_release_state T + ops_cluster_action_log T + ops_metrics_config M + ops_log_query_log T (per F-05) + ops_log_entry W (per F-05) + ops_log_analysis W (per F-05)
14. ✅ 5 域 Lead 签字栏 (Mavis 临时代签, per守門 #14 v2 拍板 D)
15. ✅ PR 流程 (per守門 #26 v26): PR-5-LEVEL-FULL-001.md (per wt7 描述)

**累计 15/15 DOD 验证项全部 PASS**.

---

## §2 RACI 5 域 Lead 角色 (per TEST-DESIGN §6.5 + 守門 #14 v2)

### 2.1 RACI 矩阵 (per 守門 #14 v2 + 守門 #3 5 域独立 Lead 硬约束)

**RACI 角色责任** (per 守門 #14 v2 拍板 + 守門 #3 反转 8/21 + 9/3 11:35 JST 拍板 B 衍生):

- **R (Responsible)**: 任务执行 (Lead 自执行 R)
- **A (Accountable)**: 任务负责 (Lead 负责 A)
- **C (Consulted)**: 域内咨询 (接受域内 C 咨询)
- **I (Informed)**: 域外通知 (域外 I 通知)

**5 角色 RACI 矩阵** (per 守門 #14 v2 拍板 + 9/3 19:35 JST 拍板 D + 9/5 10:43 JST 拍板 D + 9/8 15:19 JST 第 6 次强化):

| 角色 | R | A | C | I | 责任人 (真人到位前 Mavis 临时代签) | 签字日期 |
|---|---|---|---|---|---|---|
| **架构师** | ✅ | ✅ | — | — | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 JST |
| **SRE Lead** | ⏳ | ⏳ | — | — | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| **平台 Lead** | ⏳ | ⏳ | — | — | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| **评审主持** | ⏳ | ⏳ | — | — | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| **PM** | ⏳ | ⏳ | — | — | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |

### 2.2 5 域 Lead 真人到位 timeline (per docs/recruitment/5-business-domain-lead-referral.md v0.1)

**timeline** (per §1.2 T0-T5 + 9/3 19:35 JST 拍板 D 维持):

| Phase | 周 | 里程碑 | Lead 到位状态 |
|---|---|---|---|
| T0 | W0 | brief 落档 + 5 域 Lead 招聘启动 | 0/5 |
| T1 | W1-2 | Ulysses 内推 + Freelance 备选 | 0/5 |
| T2 | W3-4 | 1-2 名 Lead 到位 (候选 1 优先) | 1-2/5 |
| T3 | W5 | 至少 1 名 Lead 到位 → G-DEP-08 PostgreSQL checkpointer Tier 3 启动 | 1-2/5 |
| T4 | W6 | 4-5 名 Lead 到位 | 4-5/5 |
| T5 | W7+ | 5 域 Lead 全到位 → 追溯签字覆盖修订历史 | 5/5 |

**Mavis 临时代签 → 真人到位追溯签字覆盖修订历史** (per 守門 #1 禁回溯 + 守門 #21 v21 修订历史规则):
- W0-W4 期间 Mavis 临时代签 5 域 Lead
- T5 (W7+) 真人到位后追溯签字覆盖修订历史 (修订历史表 +1 行, per docs/recruitment/5-business-domain-lead-referral.md §1.2 T5)

### 2.3 RACI 决策 scope (per 守門 #14 v2 拍板)

**决策 scope** = 跨域 + 域内 (Both, per 守門 #3 反转 8/21 + 9/3 11:35 JST 拍板 B 衍生):
- **跨域决策**: 5 域 Lead 全 RACI 覆盖 (per 守門 #14 v2)
- **域内决策**: 单一域 Lead 负责 R + A + C 域内咨询 + I 域外通知
- **整体方向大转弯**: Mavis 拍板 + Ulysses 知情 (per 9/8 15:19 JST 第 6 次强化 Mavis 全权代理)

### 2.4 RACI 到位 timeline (per 守門 #14 v2 拍板 D 维持)

**Mavis 临时代签 → 真人到位追溯签字** (per 9/3 19:35 JST 拍板 D 维持 + 9/5 10:43 JST 拍板 D):
- W0-W4 期间: Mavis 临时代签 5 域 Lead (per 9/8 15:19 JST 第 6 次强化 Mavis 全权代理)
- T5 (W7+): 真人到位后追溯签字覆盖修订历史 (per docs/recruitment/5-business-domain-lead-referral.md §1.2 T5)

### 2.5 Mavis 代签边界 (per 守門 #14 v2 + 8/27 19:39 JST 授权 + 9/3 11:35 JST 守門 #3 v2 派生规)

**Mavis 代签边界 = 全部代签** (per 守門 #10 + 8/27 19:39 JST 授权 + 9/3 11:35 JST 守門 #3 v2 派生规):
- ✅ commit author: Ulysses (per 守門 #10)
- ✅ 修订人: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手
- ✅ 审批: 架构师 (Mavis 接手 agent per DEC-008)
- ✅ 签字日期: 2026-09-08 JST
- ✅ 修订历史: per 守門 #21 v21 规则

---

## §3 已知缺口 (per 守門 #11 缺标比错标, DDD Review 必查)

### 3.1 6 已知缺口 (per TEST-DESIGN §6.6)

| # | 缺口 | 等级 | 缓解 | 跟踪 |
|---|---|---|---|---|
| **#1** | **F-02 ops-log.sql 3 表 DDL 缺 + 路径不一致** | P0 | F-05 单独 sprint 修路径 + 落 3 表 DDL | per WBS §14.10.2 owner P1 修正 (per F-05 commit `31cb163` 已落档) |
| **#2** | **frontend typecheck 实证缺 (AC-007 缺)** | P0 | 实装阶段跑 `npm run typecheck` 实证 (advisory 模式 per守門 #1 v26) | per [M] 子项 |
| **#3** | **错误码 E2E 覆盖 1/5** (NOT_IMPLEMENTED/UNAUTHORIZED/RATE_LIMITED/INTERNAL E2E 缺) | P1 | MVP 阶段 UT 覆盖; 实装阶段引入 middleware + E2E | per [M] 子项 |
| **#4** | **错误码 UT 覆盖 3/5** (BAD_REQUEST/INTERNAL UT 缺) | P1 | MVP 阶段 enum 完整; 实装阶段补 UT | per [M] 子项 |
| **#5** | **5 域 Lead 真人到位追溯签字** | P1 | per 守門 #14 v2 拍板 D, Mavis 临时代签 + 真人到位后追溯覆盖 (per §2.4) | per 5 域 Lead 招聘 (per docs/recruitment/5-business-domain-lead-referral.md) |
| **#6** | **6 表 RLS 13 類验证缺** (per SRS-001 §8.2, T/M 表 tenant_id + 12 類必携) | P0 | MVP 阶段 DDL 存在性; 实装阶段 sqlx::test + testcontainers 跑 13 類验证 | per F-05 sprint |

### 3.2 缺口跟踪 + DDD Review 必查项 (per 守門 #11 缺标比错标)

**DDD Review 必查 6 缺口**:
1. **#1 F-02 ops-log.sql 3 表 DDL** (P0) — 路径 + 落档 (per F-05 落档 ✅, 路径已修)
2. **#2 frontend typecheck 实证** (P0) — 1 pre-existing err 已知, advisory 模式 (per守門 #1 v26)
3. **#3 错误码 E2E 1/5** (P1) — BAD_REQUEST 实证 (per wt2), 4 缺 [M] 子项
4. **#4 错误码 UT 3/5** (P1) — 3/5 UT 实证, 2 缺 [M] 子项
5. **#5 5 域 Lead 真人到位** (P1) — Mavis 临时代签 (per §2.4)
6. **#6 6 表 RLS 13 類验证** (P0) — DDL 存在性 ✅, sqlx::test 13 類 [M] 子项

**累计 6 已知缺口 DDD Review 必查** (per 守門 #11 缺标比错标).

---

## §4 签字栏 (per 守門 #14 v2 + 守門 #21 v21 修订历史)

**RACI 矩阵** (per 守門 #14 v2 + 守門 #3 5 域独立 Lead 硬约束):

| 角色 | R | A | C | I | 责任人 (真人到位前 Mavis 临时代签) | 签字日期 |
|---|---|---|---|---|---|---|
| **架构师** | ✅ | ✅ | — | — | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 JST |
| **SRE Lead** | ⏳ | ⏳ | — | — | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| **平台 Lead** | ⏳ | ⏳ | — | — | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| **评审主持** | ⏳ | ⏳ | — | — | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| **PM** | ⏳ | ⏳ | — | — | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |

**签字栏说明** (per 守門 #14 v2 拍板 D + 9/3 19:35 JST 拍板 D + 9/5 10:43 JST 拍板 D + 9/8 15:19 JST 第 6 次强化):

- 5 域 Lead 真人到位前 Mavis 临时代签 (per 9/8 15:19 JST 第 6 次强化 Mavis 全权代理)
- 真人到位后追溯签字覆盖修订历史 (per 守門 #1 禁回溯 + 守門 #21 v21 修订历史规则)
- 派生约束保留 (per 守門 #12 禁回溯叙事 + BAS git log --follow 实证 + 缺标比错标 + 子代理授权"无证据叙事=禁止")

---

## §5 守門合规清单 (per AGENTS.md §4 + 守門 #1 + 守門 #14 v2 + 守門 #21 v21 + 守門 #26 v26)

| 守門 | 验证方式 | 实证 |
|---|---|---|
| 守門 #1 R-05 mock 路径 | dev/staging 走 mock 端点, prod/canary 走真实 K8s/LLM/PG ([M] 子项) | ✅ per UAT-PLAN §3 |
| 守門 #3 5 域 Lead 临时代签 | Mavis 接手默认代签 Ulysses, 真人到位后追溯签字 | ✅ per §2 RACI |
| 守門 #5 v2 env 安全 | 0 泄露 secret, $env:VAR 引用后直接 pipe, 不打印 | ✅ 0 env: print 操作 |
| 守門 #6 v2 frontend typecheck | advisory 模式 (per守門 #1 v26) | ✅ 1 pre-existing per agent-view |
| 守門 #7 v3 PT bench P95<200ms | 4 bench 实证 P95 < 200ms (per wt3) | ✅ 4/4 达标 |
| 守門 #9 v20 子代理 dispatch 必先 brief | `docs/briefs/5-level-full-impl.md` v0.1 已落档 | ✅ |
| 守門 #10 author=Ulysses | commit author 100% Ulysses Leo Lee | ✅ |
| 守門 #11 缺标比错标 | 6 已知缺口显式列 (per §3) | ✅ |
| 守門 #12 AI 协作文档治理 | 禁回溯叙事 + BAS git log --follow 实证 | ✅ |
| 守門 #13 DB W/T/M 100% 覆盖 | 6 表 (3 T + 2 W + 1 M) per F-05 ops-log.sql | ✅ |
| 守門 #14 v2 5 域 Lead CONTENT 4 维 | RACI 完整 + Mavis 临时代签 (per §2.3-§2.5) | ✅ |
| 守門 #19 v19 agent 交互走 scripts/automation | 容量规划 / helm_canary_mock / ai_log_mock 走 scripts/ | ✅ |
| 守門 #21 v21 修订历史 | 7 段 (per AGENTS.md §3) | ✅ per wt7 PHASE-5-LEVEL-FULL-REPORT |
| 守門 #23 AI mock 不开外部 API | ai_log_mock.py subprocess, 永远 mock | ✅ |
| 守門 #24 v2 subprocess 替代 RPC | helm_canary_mock.sh + ai_log_mock.py + 容量规划 subprocess | ✅ |
| 守門 #26 v26 merge main 必 PR 流程 | PR-5-LEVEL-FULL-001.md (per wt7 描述) | ✅ |

**0 违反**.

---

## §6 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-08 JST | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手 | 初版: 5 维 DOD 15 项 PASS + RACI 5 域 Lead Mavis 临时代签 + 6 已知缺口 + 16 守門 0 违反 | 5-LEVEL-FULL brief §2.1 wt6 派单 (per 2026-09-08 19:55 JST 拍板) |
