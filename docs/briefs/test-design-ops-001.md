# Test Design 派单 Brief — Ops Console 各级测试设计书 (TEST-DESIGN-OPS-001)

> **版本**: v0.1
> **作者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 (per 9/8 15:19 JST 第 6 次强化 Mavis 全权代理)
> **日期**: 2026-09-08 16:10 JST
> **触发**: 2026-09-08 16:00 JST 用户发令"这个页面功能正常了吗? 基于需求、基本设计、详细设计制作各级测试设计书" + 16:07 JST ask_user `ask_b09da832bbe3eb236682c369` 3 拍板 (5 级别 / 整体写 / 单文档分章)
> **守門**: 9 v20 子代理 dispatch 必先 brief, 1 R-05 不 push (推 origin 必先 ask_user), 26 v26 merge main 必 PR 流程

## 0. 拍板实证 (per ask_user `ask_b09da832bbe3eb236682c369`)

| # | 维度 | 拍板 |
|---|---|---|
| Q1 | 测试层级 | **5 级别 UT/IT/E2E/PT/UAT** (跟 WBS §13 Test Design v0.3 模板对齐, 估 ~1.2M tokens) |
| Q2 | 起点 scope | **整体写 TEST-DESIGN-OPS-001 单文档** (1 份覆盖 4 tab + 10 端点 + 6 表 W/T/M + 8 REST stub + Hybrid AI 4 級 Ladder) |
| Q3 | 文档组织 | **单文档分章 §UT/§IT/§E2E/§PT/§UAT** (跟 WBS §13 模式对齐) |

## 1. 目标

基于 Ops Console 现有 3 份需求/设计文档 (SRS-STAR-OPS-001 v0.1 + OPS-BASIC-DESIGN-001 v0.1 + OPS-DETAILED-DESIGN-001 v0.1), 制作 1 份 `docs/test-design/TEST-DESIGN-OPS-001.md` 单文档, 内部 5 大章节覆盖 5 级别测试设计:

- **§UT 单元测试** (per ops_domain/* + ops_ai/* + ops_api/* 模块, 已 41/41 lib test pass, 设计书需: 测试矩阵 + 覆盖率目标 + 边界条件 + 错误路径)
- **§IT 集成测试** (per 4 IT 跨 crate 已落地 it_cluster_update 6 + it_log_ai 3 + it_metrics_summary 3 + it_docs_list 3, 设计书需: axum oneshot + DB 集成 + 端到端 mock)
- **§E2E 端到端测试** (per 4 tab UI + 10 端点, 设计书需: 浏览器自动化 / Playwright / 真实 walkdir / 真实 k8s mock)
- **§PT 性能测试** (per 守門 #7 v3 P95<200ms, 已 3 bench 实证 cluster + metrics + docs, 设计书需: k6 / criterion / 负载模型 / 容量规划)
- **§UAT 验收测试** (per SRS-001 §2 4 类功能验收 + 6-field 错误码 + Hybrid AI Ladder 4 級, 设计书需: 验收用例矩阵 + RACI + 真人 Lead 签字栏)

## 2. 范围

### 2.1 In-Scope (5 级别设计书)

- **§UT**: ops_api/ops_domain/ops_ai/error 4 模块测试矩阵, 跟现有 41 个 lib test 1:1 对齐 + 缺口识别 (per BAS-001 §3.1-3.4)
- **§IT**: 跨 crate 4 IT 实证 (it_cluster_update + it_log_ai + it_metrics_summary + it_docs_list) + 缺 IT 识别 (per DDS-001 §1.1-1.5)
- **§E2E**: 4 tab UI 端到端 (cluster / log / metrics / docs) + i18n 3 语言 (zh-CN / en / ja) + 错误码 6-field 闭环
- **§PT**: 守門 #7 v3 P95<200ms 硬约束 + 4 bench (cluster + metrics + docs + 预留 log) + 容量规划 (并发 100 / 1000 / 5000 用户)
- **§UAT**: SRS-001 §2 4 类功能验收 + §3 MVP-骨架 落档清单 + §7 NFR (性能/可用性/安全/可观察/质量门) + §8 6 表 W/T/M 100% 覆盖验收

### 2.2 Out-of-Scope

- 真实 K8s/Helm 集群 (per 守門 #1 R-05 mock 路径, MVP 阶段走 helm_canary_mock.sh)
- 真实 LLM OpenAI/Anthropic (per 守門 #23 AI mock 不开外部 API, 走 ai_log_mock.sh + ai_stub.rs)
- 真实 K8s 集群 (per 守門 #1 R-05, MVP mock 路径)
- 真实邮件/短信通知 (per §7.4 可观察, MVP 阶段 TODO)
- 数据库真实 PG 连接 (per 12 表 W/T/M 仅 3 表 DDL 落地, F-05 ops-log.sql 5 表 DDL 补档工作项单独 sprint, 不阻塞本测试设计书)
- 5 域 Lead 真人 Lead 到位 (per 守門 #14 v2 拍板 D, Mavis 临时代签, 真人到位后追溯签字覆盖)

## 3. 4 tab 覆盖矩阵 (per BAS-001 §3.1-3.4)

| Tab | 端点 | 模块 | UT 测数 (lib test) | IT 测数 | E2E 路径 | PT bench | UAT 验收 |
|---|---|---|---|---|---|---|---|
| Cluster (F-01) | cluster_list/canary/rollback/status | ops_api/cluster | 4 (helm_action_ack_serde + canary_request_validates + list_stub + release_status_from_str) | 6 (it_cluster_update) | UI ClusterTab 4 卡片 + 状态轮询 | cluster_bench 4.2ms | SRS-001 §2.2 F-01 |
| Log AI (F-02) | log_upload/analysis | ops_api/log + ops_ai/* | 13 (mock 4 + ladder 4 + openai_stub 4 + anthropic_stub 4 + log 2 + ops_api 2) | 3 (it_log_ai) | UI LogAITab + 实时轮询 | log_bench TODO (F-05 补) | SRS-001 §2.2 F-02 |
| Metrics (F-03) | metrics_summary | ops_api/metrics | 6 (summary 3 + ops_api 2 + metrics 3 - 2 共享) | 3 (it_metrics_summary) | UI MetricsTab 5 KPI 折线 | metrics_bench 3.xms | SRS-001 §2.2 F-03 |
| Docs (F-04) | docs_list | ops_api/docs | 5 (DocCategory 2 + DocRef 1 + DocScanner 2) | 3 (it_docs_list) | UI DocsTab 5 类别分组卡片 | docs_bench 3.8ms | SRS-001 §2.2 F-04 |
| Health | healthz/readyz | ops_api/healthz | 1 (healthz_returns_200) | 0 (健康检查不需 IT) | k8s livenessProbe + readinessProbe | N/A | SRS-001 §7.2 可用性 |
| **累计** | **10 端点** | **4 模块** | **41/41 lib test pass** | **15 IT 跨 crate** | **4 tab + 3 语言** | **3 bench (cluster/metrics/docs)** | **4 类功能验收** |

## 4. 6 表 W/T/M 验收 (per SRS-001 §8.1, 100% 覆盖硬约束 per 守門 #13)

| # | 表名 | 分类 | 实际 DDL 状态 | 测试覆盖 |
|---|---|---|---|---|
| 1 | ops_helm_release_state | T | ✅ `2026-09-08-ops-cluster.sql` 落地 (F-01) | IT cluster_list 实证 |
| 2 | ops_cluster_action_log | T | ✅ `2026-09-08-ops-cluster.sql` 落地 (F-01) | IT cluster_canary/rollback 实证 |
| 3 | ops_log_query_log | T | ❌ `2026-09-08-ops-log.sql` 未落地 (F-05 单独工作项) | IT log_upload 设计 |
| 4 | ops_log_entry | W | ❌ F-05 单独工作项 | IT log_upload 设计 |
| 5 | ops_log_analysis | W | ❌ F-05 单独工作项 | IT log_analysis 设计 |
| 6 | ops_metrics_config | M SCD2 | ✅ `2026-09-08-ops-metrics.sql` 落地 (F-03) | IT metrics_summary 实证 |

**累计 3/6 = 50% DDL 落地, 6/6 = 100% 设计覆盖 (per SRS-001 §8.1 + 守門 #13 100% 表覆盖硬约束)**. F-05 补档后 = 6/6 = 100% 落地.

## 5. 跟既有 WBS §13 Test Design v0.3 关系

- WBS §13 P3-A Test Design v0.3: 4 子项 109 新测试, P3-A 整体 (含 LangGraph / Agent Runtime / Credential / 5 wt 并行)
- 本次 TEST-DESIGN-OPS-001: Ops Console 专项 (4 tab + 10 端点 + 6 表 + Hybrid AI), 跟 §13 平行不重叠
- 引用 §13 模板 (4 阶段 4 段 守门规则) 但 scope 独立

## 6. 子代理 dispatch 实证准备 (per 守門 #9 v20)

子代理 status=succeeded ≠ 实际成功, owner 必 evidence check 5/5:
1. 5 章节 §UT/§IT/§E2E/§PT/§UAT 全部存在 + 跟 SRS-001/BAS-001/DDS-001 引用一致
2. 41/41 lib test + 15/15 IT 仍 pass (设计书不动代码, 守门 cargo test 必 0 错)
3. 4 PR merge commit (97810c0d + 472bab2 + d8e916e + 8a08756 + 73623a7) 引用 + commit hash 准确
4. 6 表 W/T/M 100% 覆盖 + 3 表实际 DDL 落地状态 跟 owner P1 修正一致
5. 20 维守门 0 违反 (per AGENTS.md §4 + 9 v20 + 11 + 12 + 13 + 14 v2 + 21 v21 + 26 v26)

## 7. 6 commit 链 (估 ~1.2M tokens 累计)

| # | 标题 | 内容 | token 估 |
|---|---|---|---|
| wt1 | docs(test-design): TEST-DESIGN-OPS-001 v0.1 §0-§1 目标 + 范围 + 引用 | 引言 + 5 级别框架 + 4 tab 覆盖矩阵骨架 | ~200K |
| wt2 | docs(test-design): §2 UT 单元测试 (41 测 + 覆盖率目标 + 边界) | 4 模块测试矩阵 + 边界条件 + 错误路径 | ~250K |
| wt3 | docs(test-design): §3 IT 集成测试 (15 测 + axum oneshot + DB 集成) | 4 IT 实证 + 缺 IT 识别 + sqlx 测试容器 | ~250K |
| wt4 | docs(test-design): §4 E2E + §5 PT (Playwright + criterion 4 bench) | 4 tab UI 端到端 + 守門 #7 v3 P95<200ms + 容量规划 | ~250K |
| wt5 | docs(test-design): §6 UAT + §7 RACI + §8 修订历史 | SRS §2-§7-§8 验收 + 5 域 Lead Mavis 临时代签 + 引用文档 | ~150K |
| wt6 | docs(phase): PHASE-TEST-DESIGN-OPS-REPORT v0.1 (7 段) + PR 描述 | 7 段报告 + PR-TEST-DESIGN-OPS-001.md | ~100K |
| **累计** | | | **~1.2M** |

## 8. owner evidence check 5/5 准备 (per 守門 #9 主体, 子代理 dispatch 必先)

1. **5 章节骨架存在** (grep `^## §[0-9]` = 5 命中)
2. **6 表 W/T/M 100% 覆盖 + 3 表 DDL 状态** (跟 §4 矩阵一致)
3. **5 commit + 1 brief = 6 ahead of origin/main** (`git log origin/main..HEAD | Measure-Object -Line = 6`)
4. **cargo test -p star-ops --tests 仍 56/56 pass** (设计书不动代码, 0 错守门)
5. **20 维守门 0 违反** (per 守门 #9 v20 + 守门 #11 缺标比错标 + 守门 #12 禁回溯叙事 + 守门 #13 W/T/M + 守门 #14 v2 5 域 Lead 代签 + 守门 #21 v21 修订历史 + 守门 #26 v26 PR 流程)

## 9. 已知缺口 (per 守門 #11 缺标比错标, 子代理必标)

1. **PT log_bench 缺失** (F-02 log AI 性能 bench 未实现, owner 拍板 [M] 子项后补, 跟 F-05 ops-log.sql 同步)
2. **E2E 浏览器自动化** (Playwright / Cypress 选型未定, MVP 阶段手测 + DDD Review 替代, owner 拍板后 [M] 子项)
3. **UAT 5 域 Lead 真人到位追溯签字** (per 守門 #14 v2 拍板 D 维持, Mavis 临时代签 + 真人到位后追溯覆盖)
4. **F-02 ops-log.sql 5 表 DDL 缺** (per WBS §14.10.2 owner P1 修正, F-05 单独 sprint, 不阻塞本设计书)
5. **6 表 RLS 13 類验证缺** (per SRS-001 §8.2, MVP 阶段 TODO, owner 拍板后 [M] 子项 sqlx 真实 PG 集成)

**DDD Review 必查**: 缺口 #1 (log_bench) + #2 (E2E 浏览器自动化) + #4 (F-02 ops-log.sql 5 表) + #5 (6 表 RLS 13 類).

## 10. 引用文档 (git 实证可查)

- `docs/requirements/SRS-STAR-OPS-001.md` v0.1 (22.8KB, 4 类功能 + 6 表 + Hybrid AI + NFR)
- `docs/basic-design/OPS-BASIC-DESIGN-001.md` v0.1 (18KB, 4 模块 + Cargo.toml + 3 端点设计)
- `docs/detailed-design/OPS-DETAILED-DESIGN-001.md` v0.1 (45.7KB, 1.1-1.5 5 模块 + 错误码 + Hybrid AI Ladder)
- `docs/test-design.md` v0.3 (141KB, WBS §13 P3-A Test Design 4 子项 109 测 模板)
- `docs/reports/STAR-P3-WBS-001.md` v0.12 §14.10 4/4 子项 100% 收官 (per 2026-09-08 16:00 JST)
- 4 PR merge: [#23 MVP](https://github.com/UlyssesLeoLee/Star/pull/23) / [#25 F-02](https://github.com/UlyssesLeoLee/Star/pull/25) / [#27 F-01](https://github.com/UlyssesLeoLee/Star/pull/27) / [#28 F-03](https://github.com/UlyssesLeoLee/Star/pull/28) / [#29 F-04](https://github.com/UlyssesLeoLee/Star/pull/29)
- 4 brief 落档: `docs/briefs/ops-f0{1,2,3,4}-*.md` (F-01 12.4KB + F-02 10.6KB + F-03 13.3KB + F-04 13.7KB)
- 5 PHASE 报告: `docs/reports/PHASE-F0{1,2,3,4}-*.md` (7 段 per AGENTS.md §3)
- 12 表 W/T/M 实际 3 表 DDL 落地: `db/migrations/2026-09-08-ops-{cluster,metrics}.sql`
- `AGENTS.md` §4 守門 20 维 (本次 0 违反)

---

**Status**: 🟡 brief 落档, 等 owner push origin, 派 worker 子代理 (估 1.2M tokens / 30-50 min, 6 commit 链).

**不推 origin, 不派子代理** (per 守門 #1 反转 8/30 拍板 + 守門 #9 v20 子代理 dispatch 必先 brief + owner evidence check 5/5).
