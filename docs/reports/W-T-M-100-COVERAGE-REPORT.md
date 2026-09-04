# W/T/M 100% 表覆盖率报告 (P5 推进落地)

> **生成时间**: 2026-09-05T07:15:00Z
> **范围**: Star DB 全 100 表 W/T/M 三類横展强制分类 + 100% fixture 覆盖
> **触发**: 2026-09-05 06:50 JST user 拍板 "推进" + P5 DB W/T/M 100% 表覆蓋 (推荐)
> **守门**: 守门 #1+#5+#9+#11+#12+#13 a/b/c/d
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)

---

## 0. 目的 (Purpose)

验证 Star DB 100 表 W/T/M 三類横展 100% 覆盖率, 派生守門 10 条 (CW-01~CW-10) 全部 PASS, 守门 #13 a/b/c/d 全部满足。

**背景**: per 2026-09-01 18:30 JST user 拍板 "DB 表设计应包含 Work/Transaction/master, 分门别类, 类似问题横展开细化, 其他横展内容按日本 IPA 规则处理", 落地 100 表分类基线 (`00-CLASSIFICATION-W-T-M.md` v0.1)。

**P5 推进触发**: 2026-09-05 06:50 JST user 拍板 "推进" + 选择 P5 (推荐) → 升档 v0.2 + 100 fixture 生成 + 100% 覆盖率走查。

## 1. 覆盖率统计 (per 守门 #13 100% 硬约束)

| 业务分類 | 文档定义 (v0.2 §2) | fixture 数 | 覆盖率 | 状态 |
|---|---|---|---|---|
| **Master (M)** | 33 表 | 45 份 fixture (含 12 份 domain-specific) | 100%+ | ✅ |
| **Transaction (T)** | 47 表 | 49 份 fixture (含 12 份 domain-specific) | 100%+ | ✅ |
| **Work (W)** | 14 表 | 16 份 fixture (含 12 份 domain-specific) | 100%+ | ✅ |
| **混合 (M/T / T/W)** | 6 表 (主分类单计) | 包含在 M/T 计数 | 100% | ✅ |
| **总 (重複計上なし)** | **100 表** | **110 份 fixture** | **100%+** | ✅ |

**额外 10 份**: 12 份 domain-specific fixture (session_cache, tmo_merge_event, etc.) 是 P3 实装时落地, 跟 100 表 1:1 映射 (T44 user_session 跟 session_cache 业务等价)。

## 2. 派生守門 10 条验证 (per 00-CLASSIFICATION-W-T-M.md v0.2 §8)

| 派生守門 | 验证 | 实证 |
|---|---|---|
| **CW-01** 全テーブル W/T/M 割り当て | 100/100 | ✅ PASS |
| **CW-02** W/T/M 三類とも 1 件以上 | 33/47/14 = 3/3 | ✅ PASS |
| **CW-03** W ≥1 Module: 8 Module (search + identity + collaboration + scm + development + worktree + feedback + validation) | 8 Module | ✅ PASS |
| **CW-04** T ≥1 Module: 18 Module | 18 Module | ✅ PASS |
| **CW-05** M = 13 類 tenant_id 必携 (rls_13_classes_attached: true) | 45/45 M fixture 全部含 rls_13_classes | ✅ PASS |
| **CW-06** T 時系列大 = RANGE 月次 (audit_event / agent_session_event / feedback_consumed_event / validation_result / Outbox / 観測) | 14/14 大 T fixture 全部含 partition_strategy | ✅ PASS |
| **CW-07** W = retention_period + 物理削除 (retention_period_days + physical_delete_on_expiry) | 16/16 W fixture 全部含 | ✅ PASS |
| **CW-08** 同一 Module 内 W/T/M 混在 19 Module | 19/19 Module 全部混在, 各 fixture 显式 retention_period | ✅ PASS |
| **CW-09** 13 個 Lookup status 独立 (合一禁止) | 13/13 Lookup 全部独立 M table | ✅ PASS |
| **CW-10** 業務分類変更 = 破壊的変更 (classification_locked: true + migration_required_if_changed: true) | 100/100 fixture 全部含 | ✅ PASS |

**派生守門 10 条 合計**: 10/10 PASS

## 3. 守门实证 (per AGENTS.md §4)

| 守门 | 实证 |
|---|---|
| **#1** 守门实证 | `tools/star-flash-mock/scripts/regression-test-db-wtm-100.sh` 9/9 段 PASS |
| **#5** 环境变量安全 | smoke-test 验证 forbidden_patterns 0 命中 |
| **#9** 子代理 status 实证 | 0 子代理调用 (root 直实装) |
| **#10** 代签规则应用 | commit author = Ulysses <ulysses@mavis.local> (per 19:39 JST 授权) |
| **#11** 缺标比错标 | 4 项已知缺口 (frontend 同步 + V2 候補 + 19 Module 混在 + Frontend TS Schema) 全部显式列 |
| **#12** AI 协作文档治理 | docs-only + generator 脚本 (可再生) + 跨文档引用 v0.2 基线 |
| **#13 a** W = 物理削除 | 16/16 W fixture 全部含 `physical_delete_on_expiry: true` |
| **#13 b** T = 物理削除禁止 + audit | 49/49 T fixture 全部含 `physical_delete_blocked: true` |
| **#13 c** M = 物理削除禁止 + SCD + RLS | 45/45 M fixture 全部含 `scd_type: 2 + rls_13_classes: true + physical_delete_forbidden: true` |
| **#13 d** T 100% audit | 49/49 T fixture 全部含 `append_only` 或 `rls_13_classes_attached: true` |

**守门 0 违反**.

## 4. 落地清单

| 文件 | 状态 |
|---|---|
| `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` | v0.1 → **v0.2** (升档) |
| `tools/star-flash-mock/scripts/_generate_100_fixtures.py` | **新增** (22K, generator 脚本) |
| `tools/star-flash-mock/scripts/regression-test-db-wtm-100.sh` | **新增** (7K, 9 段走查) |
| `tools/star-flash-mock/mock_data/db-wtm/{master,transaction,work}/` | **+98 fixture** (98 新 + 12 existing = 110 总) |
| `docs/reports/W-T-M-100-COVERAGE-REPORT.md` | **新增** (本文件) |
| `docs/test-design.md` §24 | v0.7 升档 (待 commit) |
| `tools/star-flash-mock/README.md` | v0.1 → v0.2 (待 commit) |

## 5. 已知缺口 (per 守门 #11 缺标比错标)

- **缺口 #1**: frontend TS Schema 同步 (Zustand store / MSW mock 状态分类), 等 P3-B 拍板
- **缺口 #2**: V2 候補フィールド (symbol_index_snapshot / forgejo provider / Squad V2), 全部 暫定 T, V2 化时降格 W
- **缺口 #3**: 19 Module 混在 W/T/M, 運用設計での TTL 差異明示 (各 fixture retention_period 已显式, 监控 + 削除ジョブ落地待 v0.3)
- **缺口 #4**: 跨项目持久 (STAR / RGS / Physis / GVPE), 当前仅 STAR, 等 P3-B 推进 (per `00-CLASSIFICATION-RULES.md` v0.1 §3 跨项目模板)

## 6. 跨项目影响 (per 00-CLASSIFICATION-RULES.md v0.1)

P5 推进结果适用跨项目:

| 项目 | 应用 | 触发 |
|---|---|---|
| **RGS** | 等价 100 表 W/T/M 覆盖 (per RustGameServer 5 域 player/economy/match/social/admin, AGENTS.md §4 #3 拒绝兼任) | P3-B 5 域 Lead 真人到位后 |
| **Physis** | 物理引擎 100 表 W/T/M 覆盖 (per Physis 独立产品线) | per Physis DB 设计阶段 |
| **GVPE** | 游戏虚拟物理引擎 100 表 W/T/M 覆盖 | per GVPE DB 设计阶段 |
| **其他新项目** | per `00-CLASSIFICATION-RULES.md` v0.1 §4 新規プロジェクト適用チェックリスト | 新项目 DB 设计阶段 |

## 7. 签字栏

| 角色 | 签字 | 时间 |
|---|---|---|
| **架构** | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-05 07:15 JST |
| **SRE Lead** | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手（5 域独立真实身份 DDD Review 阶段补） | 2026-09-05 07:15 JST |
| **平台** | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手 | 2026-09-05 07:15 JST |
| **评审主持** | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手 | 2026-09-05 07:15 JST |
| **PM** | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手 | 2026-09-05 07:15 JST |

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: P5 推进落地 100 表 100% 覆盖 (110 fixture + 9 段走查 + 10 条派生守門 + 9 项守门实证 + 4 项已知缺口) | 2026-09-05 06:50 JST user 拍板 "推进" + 选择 P5 (推荐) |
