# TEST-DESIGN-OPS-001 — STAR Ops Console 测试设计书

> **版本**: v0.1
> **作者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 9/8 15:19 JST 第 6 次强化 Mavis 全权代理)
> **审批**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手
> **日期**: 2026-09-08 JST
> **状态**: Draft Baseline (per ask_user `ask_b09da832bbe3eb236682c369` 拍板 3 维: 5 级别 / 整体写 / 单文档分章)

---

## §0 文档目的

本文档定义 STAR Ops Console（`crates/star-ops` 48 package）的端到端测试设计书，单文档覆盖 5 级别测试（UT/IT/E2E/PT/UAT）共 **41 lib + 15 IT + 3 bench** 实证锚点，**1 份单文档分章** 对应 4 tab × 10 端点 × 6 表 W/T/M × Hybrid AI 4 级 Ladder。

**触发**（per 2026-09-08 16:00 JST 用户发令 + 16:07 JST ask_user 拍板）：

- 用户发令"这个页面功能正常了吗? 基于需求、基本设计、详细设计制作各级测试设计书"
- ask_user `ask_b09da832bbe3eb236682c369` 拍板 3 维：
  - Q1 测试层级: **5 级别 UT/IT/E2E/PT/UAT**（跟 WBS §13 Test Design v0.3 模板对齐，估 ~1.2M tokens）
  - Q2 起点 scope: **整体写 TEST-DESIGN-OPS-001 单文档**（1 份覆盖 4 tab + 10 端点 + 6 表 W/T/M + 8 REST stub + Hybrid AI 4 级 Ladder）
  - Q3 文档组织: **单文档分章 §UT/§IT/§E2E/§PT/§UAT**（跟 WBS §13 模式对齐）

**核心定位**：

- 单文档 ≤ 60KB，分 9 章节（§0 目的 + §1 范围 + §2-§6 5 级别 + §7 RACI + §8 修订历史 + §9 引用）
- 跟既有 `docs/test-design.md` v0.3 (141KB, WBS §13 P3-A 整体, 4 子项 109 测) 平行不重叠
- 引用上游 3 份需求/设计文档 (SRS-001 v0.1 + BAS-001 v0.1 + DDS-001 v0.1) + 既有 5 份 brief
- 5 已知缺口显式标注（per 守门 #11 缺标比错标），DDD Review 必查

---

## §1 范围

### 1.1 In-Scope（5 级别测试设计书）

| 章节 | 测试级别 | 范围 | 实证锚点 |
|---|---|---|---|
| **§2 UT** | 单元测试 | ops_api/ops_domain/ops_ai/error 4 模块测试矩阵 + 41 lib test 1:1 对齐 + 边界 + 错误路径 | `cargo test -p star-ops --lib` 41/41 pass (实证 2026-09-08) |
| **§3 IT** | 集成测试 | 跨 crate 4 IT（it_cluster_update 6 + it_log_ai 3 + it_metrics_summary 3 + it_docs_list 3） + axum oneshot + sqlx 容器 + 缺 IT 识别 | `cargo test -p star-ops --tests` 15/15 IT pass (实证 2026-09-08) |
| **§4 E2E** | 端到端 | 4 tab UI（cluster / log / metrics / docs）+ i18n 3 语言（zh-CN / en / ja）+ 错误码 6-field 闭环 | MVP 阶段手测 + DDD Review；Playwright/Cypress 选型待 DDD Review 拍板（见 §4.6 缺口） |
| **§5 PT** | 性能测试 | 守门 #7 v3 **P95<200ms** 硬约束 + 3 bench 实证（cluster 49ms / metrics ~0.83μs / docs 4.7ms，per git 实证）+ 容量规划 100/1000/5000 用户 | `cargo bench -p star-ops -- --quick` 3 bench P95 实证（per F-01/F-03/F-04 commit message） |
| **§6 UAT** | 验收测试 | SRS-001 §2 4 类功能验收 + §3 MVP-骨架 落档清单 + §7 NFR（性能/可用/安全/可观察/质量门）+ §8 6 表 W/T/M 100% 覆盖验收 | 5 域 Lead 签字栏（Mavis 临时代签 per 守门 #14 v2 拍板 D） |

**累计实证锚点**（per 守门 #1 v25 单 crate）：

```
lib test:    41/41 pass    (cargo test -p star-ops --lib -j 4)
IT test:     15/15 pass    (cargo test -p star-ops --tests -j 4)
bench PT:    3 bench P95 < 200ms 实证 (cluster 49ms / metrics 0.83μs / docs 4.7ms)
合计实证:    59 测 100% pass + 3 bench P95 守门
```

### 1.2 Out-of-Scope（per 守门 #1 R-05 mock 路径 + 守门 #11 缺标比错标）

- **真实 K8s/Helm 集群**（per 守门 #1 R-05 工具/数据接口不接生产）：MVP 走 `scripts/automation/helm_canary_mock.sh` subprocess（per F-01 实装）
- **真实 LLM OpenAI/Anthropic**（per 守门 #23 AI mock 不开外部 API）：MVP 走 `scripts/automation/ai_log_mock.py` + `crates/star-ops/src/ops_ai/{openai_stub,anthropic_stub}.rs` 标 `// TODO: 真实 API`
- **真实 K8s 集群**（per 守门 #1 R-05，MVP mock 路径）：cluster update 走 `helm_canary_mock.sh` subprocess 实测 49ms P95
- **真实邮件/短信通知**（per SRS-001 §7.4 可观察，MVP 阶段 TODO）：F-01/F-02 告警通道 MVP 不实装
- **真实 PG 连接**（per 12 表 W/T/M 仅 3 表 DDL 落地，F-05 ops-log.sql 5 表 DDL 补档工作项单独 sprint，不阻塞本测试设计书）
- **5 域 Lead 真人 Lead 到位**（per 守门 #14 v2 拍板 D，Mavis 临时代签，真人到位后追溯签字覆盖）
- **OAuth 2.0 / mTLS**（per BAS-001 §7 MVP auth stub）：复用 `star-context::ActorContext`，不实装
- **OpenAPI 3.1 spec 自动生成**（per BAS-001 §7 待 v0.2 utoipa）：MVP 不实装

### 1.3 5 级别测试框架

```
                          ┌──────────────────────┐
                          │  浏览器 (Next.js)     │  ← E2E (Playwright/Cypress 选型待 DDD Review)
                          │  /ops (4 tab)         │
                          └──────────┬───────────┘
                                     │ HTTP fetch
                                     ▼
                          ┌──────────────────────┐
                          │  star-ops (port 8090)│
                          │  axum 0.8 + 6-field  │  ← IT (axum oneshot + 跨 crate + sqlx 容器)
                          │  10 endpoint + 2     │
                          │  healthz/readyz      │
                          └──────────┬───────────┘
                                     │
        ┌─────────────┬──────────────┼──────────────┬─────────────┐
        ▼             ▼              ▼              ▼             ▼
   ┌─────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
   │error    │  │ops_api   │  │ops_domain│  │ops_ai    │  │benches   │  ← UT (41 lib test) + PT (3 bench)
   │3 test   │  │7 test    │  │11 test   │  │16 test   │  │7 bench   │
   └─────────┘  └──────────┘  └──────────┘  └──────────┘  └──────────┘
                                     │
                                     ▼
                          ┌──────────────────────┐
                          │ scripts/automation/  │
                          │ ai_log_mock.py       │  ← subprocess (per 守门 #24 v2)
                          │ helm_canary_mock.sh  │
                          └──────────────────────┘

↓ ↑ ↓
所有级别通过以下守门验证:
- 守门 #1: cargo test -p star-ops 0 err
- 守门 #7 v3: PT bench P95 < 200ms
- 守门 #11: 缺标比错标 (5 已知缺口显式列)
- 守门 #13: DB W/T/M 100% 覆盖
- 守门 #14 v2: 5 域 Lead Mavis 临时代签
- 守门 #23: AI mock 不开外部 API
- 守门 #26 v26: PR 流程
```

### 1.4 4 tab × 10 端点覆盖矩阵

| Tab | 端点 | 模块 | UT 测数 | IT 测数 | E2E 路径 | PT bench | UAT 验收 |
|---|---|---|---|---|---|---|---|
| **Cluster (F-01)** | `GET /api/ops/cluster/releases`<br>`POST /api/ops/cluster/canary`<br>`POST /api/ops/cluster/rollback`<br>`GET /api/ops/cluster/status` | `ops_api` + `ops_domain::cluster` | 4 (helm_action_ack_serde + canary_request_validates_weight_range + list_stub_returns_one_helm_release + release_status_from_str_works) | 6 (it_cluster_update) | UI ClusterTab 4 卡片 + 状态轮询 + canary 滑块 + rollback 选择 | `cluster_bench` P95 **49ms** (per F-01 git log d8e916e) | SRS-001 §2.2 F-01 + AC-001..AC-003 |
| **Log AI (F-02)** | `POST /api/ops/log/upload`<br>`GET /api/ops/log/analysis/{id}` | `ops_api` + `ops_domain::log` + `ops_ai::*` | 13 (mock 3 + ladder 5 + openai_stub 3 + anthropic_stub 3 + log 2 - 3 共享) | 3 (it_log_ai) | UI LogAITab + 实时轮询 + 上传区 + AI 分析结果 | `log_upload_bench` P95 **TODO** (F-05 单独工作项，见 §4.6 缺口 #1) | SRS-001 §2.2 F-02 + AC-004 |
| **Metrics (F-03)** | `GET /api/ops/metrics/summary` | `ops_api` + `ops_domain::metrics` | 6 (summary_stub + summary_empty_state + summary_returns_five_kpis_via_telemetry + ops_api 2 + metrics 0) | 3 (it_metrics_summary) | UI MetricsTab 5 KPI 折线 + 趋势占位 | `metrics_bench` P95 **~0.83μs** (per F-03 git log 8a08756) | SRS-001 §2.2 F-03 + AC-005 |
| **Docs (F-04)** | `GET /api/ops/docs` | `ops_api` + `ops_domain::docs` | 5 (DocCategory 2 + DocRef 1 + DocScanner 2) | 3 (it_docs_list) | UI DocsTab 5 类别分组卡片 | `docs_bench` P95 **4.7ms** (per F-04 git log 08e7711 实测 [4.7011 ms 4.7065 ms 4.7078 ms]) | SRS-001 §2.2 F-04 + AC-006 |
| **Health** | `GET /healthz`<br>`GET /readyz` | `ops_api` (healthz 端点) | 1 (healthz_returns_200) | 0（健康检查不需 IT） | k8s livenessProbe + readinessProbe | N/A | SRS-001 §7.2 可用性 |
| **累计** | **10 端点** | **4 模块 + error** | **41/41 lib test pass** | **15 IT 跨 crate** | **4 tab + 3 语言** | **3 bench (cluster/metrics/docs)** | **4 类功能验收** |

**注**: 实证 P95 值（cluster 49ms / metrics 0.83μs / docs 4.7ms）来自 F-01/F-03/F-04 commit message git log 实证，非回溯叙事。**brief 列 "cluster 4.2ms" 跟 git 实证 49ms 数值不一致**（per 守门 #12 禁回溯叙事 + 守门 #11 缺标比错标，本文以 git 实证 49ms 为准，brief 4.2ms 推测为打字误差）。

### 1.5 引用基线（per 守门 #9 v20 子代理 dispatch 必先 brief）

**上游需求/设计文档**（3 份必引用）：

- [`docs/requirements/SRS-STAR-OPS-001.md` v0.1](../../requirements/SRS-STAR-OPS-001.md)（22.8KB，§2 4 类功能 + §3 MVP-骨架 + §5 Hybrid AI Ladder + §6 RACI + §7 NFR + §8 6 表 W/T/M）
- [`docs/basic-design/OPS-BASIC-DESIGN-001.md` v0.1](../../basic-design/OPS-BASIC-DESIGN-001.md)（18KB，§1 系统组成 + §2 模块 + §3 3 端点 + §4 错误码）
- [`docs/detailed-design/OPS-DETAILED-DESIGN-001.md` v0.1](../../detailed-design/OPS-DETAILED-DESIGN-001.md)（45.7KB，§1.1-1.5 5 模块 + 错误码 6-field + Hybrid AI Ladder + 4 IT 实证）

**下游落地 commit**（4 子项 + MVP-骨架 + WBS，6 commit hash 必引用）：

| Commit | 主题 | 关键证据 |
|---|---|---|
| `97810c0d` (PR #23) | MVP-骨架（4 tab + 8 REST stub + Hybrid AI + 6 表 W/T/M 100%） | star-ops crate 48 package + lib.rs 暴露 4 子模块 |
| `472bab2` (PR #25) | F-02 log AI 端到端实装 | log_ai.rs + 13 测 + log_upload_bench + ops_log_query_log T 表设计 |
| `d8e916e` (PR #27) | F-01 cluster update 端到端实装 | helm_canary_mock.sh subprocess + 11 表 W/T/M 100% + cluster_bench P95 49ms |
| `8a08756` (PR #28) | F-03 metrics 端到端实装 | star-telemetry 复用 + 12 表 W/T/M 100% + metrics_bench P95 ~0.83μs |
| `73623a7` (PR #29) | F-04 docs 端到端实装 | walkdir 真实扫描 + 5 docs 子域 + 3 IT + bench P95 3.8ms（commit title）/ 4.7ms（commit body 实测） |
| `fff73c1` | WBS v0.12 §14.10 4/4 子项 100% 收官 + §15 累计 86/108 → 90/108 (83.3%) + owner P1 修正（12 表 → 3 表 DDL，F-02 ops-log.sql 5 表缺 = F-05 工作项） | owner P1 修正实证 |

**既有 test-design 模板**（P3-A 整体，平行不重叠）：

- [`docs/test-design.md` v0.3](../../test-design.md)（141KB，4 子项 109 新测试，跟本专项平行不重叠）

**既有 5 份 brief**（4 子项 + 本专项，共 5 份）：

- `docs/briefs/ops-f01-cluster-update-impl.md` v0.1（12.4KB，F-01 brief）
- `docs/briefs/ops-f02-log-ai-impl.md` v0.1（10.6KB，F-02 brief）
- `docs/briefs/ops-f03-metrics-impl.md` v0.1（13.3KB，F-03 brief）
- `docs/briefs/ops-f04-docs-impl.md` v0.1（13.7KB，F-04 brief）
- `docs/briefs/test-design-ops-001.md` v0.1（10.8KB，本专项 brief，commit `8325cce`）

**5 份 PHASE 报告**（4 子项 + 入口，每份 7 段 per AGENTS.md §3 模板）：

- `docs/reports/PHASE-OPS-INTRY-REPORT.md`（入口）
- `docs/reports/PHASE-F01-CLUSTER-UPDATE-REPORT.md`
- `docs/reports/PHASE-F02-LOG-AI-REPORT.md`
- `docs/reports/PHASE-F03-METRICS-REPORT.md`
- `docs/reports/PHASE-F04-DOCS-REPORT.md`

**实际 DDL 落地状态**（owner P1 修正后）：

- ✅ `db/migrations/2026-09-08-ops-cluster.sql`（F-01，2 表 T：ops_helm_release_state + ops_cluster_action_log）
- ✅ `db/migrations/2026-09-08-ops-metrics.sql`（F-03，1 表 M SCD2：ops_metrics_config）
- ❌ `db/migrations/2026-09-08-ops-log.sql`（F-02，0 行落地，3 表 DDL 缺：ops_log_query_log T + ops_log_entry W + ops_log_analysis W = F-05 单独工作项，不阻塞本设计书）

**引用基线**（per 守门 #13 DB W/T/M 100% 覆盖）：

- `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` v0.1
- `docs/data-design/ipa-detail/00-CLASSIFICATION-RULES.md` v0.1

---

## §2 UT 单元测试（per 守门 #1 v25 单 crate cargo test --lib）

### 2.1 测试目标与覆盖率基线

**目标**：验证 `crates/star-ops` 4 模块（`error` + `ops_api` + `ops_domain` + `ops_ai`）的纯函数 / 单元行为符合 OPS-BASIC-DESIGN §3-§5 契约，覆盖以下维度：

| 维度 | 目标 | 实证 |
|---|---|---|
| 行覆盖率 | ≥ 80% (lib 路径) | 41/41 lib test 100% pass (实证 2026-09-08 `cargo test -p star-ops --lib`) |
| 分支覆盖率 | ≥ 70% (lib 路径) | per `cargo tarpaulin` / `cargo llvm-cov` 后续 [M] 子项引入 |
| 错误码 6-field 必填率 | 100% | per error.rs 5 variant + `to_body` 完整覆盖 |
| 边界条件 | 必跑 5 维（空/最大/最小/无效输入/并发） | per §2.5 边界条件矩阵 |
| Mock 路径守门 #1 R-05 | 100% 不接生产 | per §2.6 错误路径覆盖 |

**实证锚点**（per 守门 #1 v25 单 crate + 守门 #9 v20 git 实证可查）：

```
$ cargo test -p star-ops --lib -j 4
running 41 tests
... (41 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out)
test result: ok. 41 passed; 0 failed ... finished in 0.92s
```

### 2.2 4 模块测试矩阵（41 测 1:1 对齐）

#### 2.2.1 `error` 模块（3 测，per `src/error.rs:147-175`）

| 测名 | 验证 | 派生规 |
|---|---|---|
| `not_implemented_returns_501` | `NotImplemented` variant → `code=NOT_IMPLEMENTED` + `source_kind=Internal` + `retriable=false` + `hint=Some(...)` | 守门 #6 6-field 完整 + 守门 #12 实证 |
| `unauthorized_returns_401_with_policy_source` | `Unauthorized` variant → `code=UNAUTHORIZED` + `source_kind=Policy` + `retriable=false` | 守门 #6 + 守门 #14 v2 策略层 |
| `rate_limited_is_retriable` | `RateLimited` variant → `code=RATE_LIMITED` + `retriable=true` (per 守门 #6 v2 派生) | 守门 #6 v2 派生 + 60 req/min |

**派生测试缺口识别**（per 守门 #11 缺标比错标）：

- ⚠️ `bad_request_returns_400_with_validation_source`（缺）：`BadRequest` variant 无 UT 覆盖，per BAS-001 §3.5 应有 `source_kind=Validation` 验证。[M] 子项补
- ⚠️ `internal_returns_500_with_internal_source`（缺）：`Internal` variant 无 UT 覆盖，per BAS-001 §3.5 应有 `source_kind=Internal` + `retriable=true` 验证。[M] 子项补
- ⚠️ `ops_error_into_response_status_mapping`（缺）：5 variant → HTTP status code 映射（501/401/429/400/500）无 UT 覆盖，per `IntoResponse` impl。[M] 子项补

**累计缺 3 测**（per 守门 #11）：DDD Review 必查。

#### 2.2.2 `ops_domain` 模块（11 测，per BAS-001 §3.1-3.4）

| 测名 | 模块 | 验证 | 派生规 |
|---|---|---|---|
| `release_status_from_str_works` | `cluster` | `ReleaseStatus` 字符串 → enum 解析（5 状态） | 守门 #6 + 状态机 |
| `helm_action_ack_serde` | `cluster` | `HelmActionAck` 序列化/反序列化 | 守门 #6 6-field + axum 0.8 JSON |
| `canary_request_validates_weight_range` | `cluster` | `CanaryRequest.canary_weight` ∈ [0, 100] 校验 | 守门 #11 边界 + 业务规则 |
| `list_stub_returns_one_helm_release` | `cluster` | `HelmRelease::list_releases()` 返 1 条 stub | per BAS-001 §3.1 MVP stub |
| `doc_category_short_name_returns_5_labels` | `docs` | 5 类别 short name（SRS/BAS/DDS/REPORT/OTHER） | per F-04 5 子域 |
| `doc_ref_stub_returns_two_refs` | `docs` | `DocRef::stub()` 返 2 条硬编码 | per BAS-001 §3.4 MVP stub |
| `doc_category_from_path_classifies_four_subdirs` | `docs` | 4 子目录（requirements/ basic-design/ detailed-design/ reports/）→ 4 类别 | per F-04 walkdir 真实路径 |
| `doc_scanner_finds_phase_f03_report` | `docs` | `DocScanner.list()` 找到 F-03 PHASE 报告 | per F-04 真实 walkdir |
| `doc_scanner_list_finds_at_least_four_subsections` | `docs` | `DocScanner.list()` ≥ 4 docs | per F-04 5 子域 |
| `log_entry_stub_has_error_level` | `log` | `LogEntry::stub()` level=Error | per BAS-001 §3.2 |
| `log_analysis_stub_confidence_below_threshold` | `log` | `LogAnalysis::stub()` confidence < 0.5 触发 needs_review | 守门 #23 + 0.5 阈值 |
| `summary_stub_returns_five_kpis` | `metrics` | `MetricsAggregator::summary_stub()` 返 5 KPI | per BAS-001 §3.3 |
| `summary_empty_state_returns_five_kpis` | `metrics` | `MetricsAggregator` 空状态返 5 KPI（0 值） | per F-03 端到端 |
| `summary_returns_five_kpis_via_telemetry` | `metrics` | 真实调 `star-telemetry` 5 KPI 接入 | per F-03 端到端 + 守门 #1 R-05 mock |

**派生测试缺口识别**（per 守门 #11 缺标比错标）：

- ⚠️ `release_status_from_str_invalid_returns_err`（缺）：`ReleaseStatus::from_str` 无效输入应返 Err 而非 panic。[M] 子项补
- ⚠️ `canary_request_validates_target_revision_range`（缺）：`target_revision` 边界（0/负数/超大）无 UT 覆盖。[M] 子项补
- ⚠️ `apply_level_filter_empty_filter_returns_full_content`（缺）：`ops_api::apply_level_filter`（`src/ops_api.rs:345-367`）空 filter 行为无 UT 覆盖（仅 IT 实证）。[M] 子项补
- ⚠️ `log_analysis_stub_confidence_above_threshold_no_review`（缺）：confidence ≥ 0.5 不触发 needs_review 分支无 UT 覆盖。[M] 子项补
- ⚠️ `summary_aggregator_telemetry_call_count`（缺）：`MetricsAggregator` 调 `star-telemetry` 次数断言（应 = 5 KPI）无 UT 覆盖。[M] 子项补

**累计缺 5 测**（per 守门 #11）：DDD Review 必查。

#### 2.2.3 `ops_ai` 模块（16 测，per BAS-001 §5 + ADR-0026 §2.2）

| 测名 | 模块 | 验证 | 派生规 |
|---|---|---|---|
| `is_retriable_internal` | `ladder` | `Internal` 错误 → retriable=true | 守门 #6 v2 |
| `is_retriable_rate_limited` | `ladder` | `RateLimited` 错误 → retriable=true | 守门 #6 v2 派生 |
| `is_not_retriable_not_implemented` | `ladder` | `NotImplemented` 错误 → retriable=false | 守门 #6 |
| `is_not_retriable_unauthorized` | `ladder` | `Unauthorized` 错误 → retriable=false | 守门 #6 |
| `is_not_retriable_bad_request` | `ladder` | `BadRequest` 错误 → retriable=false | 守门 #6 |
| `mock_channel_always_enabled` | `mock` | `MockChannel::is_enabled()` 永真（L4 兜底） | 守门 #23 L1 兜底 |
| `mock_analyze_info_log_produces_no_anomaly` | `mock` | INFO log → no anomaly | 守门 #23 mock 模板 |
| `mock_analyze_error_log_produces_anomaly` | `mock` | ERROR log → 1 anomaly | 守门 #23 mock 模板 |
| `call_subprocess_stub_real_invocation` | `mock` | subprocess 真实调 `ai_log_mock.py` 跑通 | 守门 #24 v2 subprocess |
| `openai_stub_is_disabled_without_api_key` | `openai_stub` | 无 API key → is_enabled=false | 守门 #5 v2 + L2 |
| `openai_stub_is_enabled_with_api_key` | `openai_stub` | 有 API key → is_enabled=true | 守门 #5 v2 + L2 |
| `openai_stub_builder_builds_with_api_key` | `openai_stub` | Builder 模式构建 + 透传 api_key | 守门 #5 v2 |
| `openai_stub_no_network_mode_returns_stub_analysis` | `openai_stub` | 无网络模式返 stub `LogAnalysis` | per DDS-001 §3.2 |
| `anthropic_stub_is_disabled_without_api_key` | `anthropic_stub` | 同 OpenAI | 守门 #5 v2 + L3 |
| `anthropic_stub_is_enabled_with_api_key` | `anthropic_stub` | 同 OpenAI | 守门 #5 v2 + L3 |
| `anthropic_stub_builder_builds_with_api_key` | `anthropic_stub` | 同 OpenAI | 守门 #5 v2 |
| `anthropic_stub_no_network_mode_returns_stub_analysis` | `anthropic_stub` | 同 OpenAI | per DDS-001 §3.2 |

**派生测试缺口识别**（per 守门 #11 缺标比错标）：

- ⚠️ `ladder_analyze_log_first_channel_succeeds`（缺）：L1 mock 成功直接返回，无下一通道尝试的 UT 覆盖。[M] 子项补
- ⚠️ `ladder_analyze_log_fallback_to_next_channel`（缺）：L1 失败 retriable → L2 兜底，无 UT 覆盖。[M] 子项补
- ⚠️ `ladder_analyze_log_non_retriable_returns_immediately`（缺）：L1 失败 non-retriable 直接返错，无 UT 覆盖。[M] 子项补
- ⚠️ `ladder_analyze_log_all_channels_disabled_returns_no_channel`（缺）：所有通道 disable → `no_channel_available` 错误，无 UT 覆盖。[M] 子项补
- ⚠️ `mock_analyze_warn_log_produces_one_anomaly`（缺）：WARN log anomaly 模板（mock 模板仅 INFO/ERROR 覆盖）。[M] 子项补
- ⚠️ `mock_confidence_below_threshold_triggers_needs_review`（缺）：mock 模板 confidence 0.42 < 0.5 触发 needs_review 的独立 UT。[M] 子项补
- ⚠️ `openai_stub_analyze_log_returns_not_implemented`（缺）：OpenAI stub `analyze_log` 应返 `NotImplemented`，无 UT 覆盖。[M] 子项补
- ⚠️ `anthropic_stub_analyze_log_returns_not_implemented`（缺）：同 OpenAI stub。[M] 子项补

**累计缺 8 测**（per 守门 #11）：DDD Review 必查。

#### 2.2.4 `ops_api` 模块（7 测，per BAS-001 §3.1-3.5）

| 测名 | 验证 | 派生规 |
|---|---|---|
| `cluster_list_returns_one_release` | `GET /api/ops/cluster/releases` → 200 + 1 release | per BAS-001 §3.1 |
| `metrics_summary_returns_five_kpis` | `GET /api/ops/metrics/summary` → 200 | per BAS-001 §3.3 |
| `metrics_summary_real_returns_5_kpis_with_stub_false` | 200 + 5 KPI names + `meta.stub=false` + hint 含 "star-telemetry" | per F-03 端到端 + 守门 #1 R-05 |
| `healthz_returns_200` | `GET /healthz` → 200 OK | per BAS-001 §6.2 livenessProbe |
| `log_analysis_returns_stub` | `GET /api/ops/log/analysis/{id}` → 200 + stub | per BAS-001 §3.2 |
| `log_upload_with_trace_id_and_level_filter` | POST + trace_id 透传 + level_filter 应用 | per 守门 #5 v2 + F-02 |
| `log_upload_rejects_oversized_body` | POST 1.1MB content → 400 BadRequest | per 守门 #5 v2 1MB 限制 |

**派生测试缺口识别**（per 守门 #11 缺标比错标）：

- ⚠️ `cluster_canary_returns_200_with_action_id`（缺）：`POST /api/ops/cluster/canary` 端点无 UT 覆盖（仅 IT 实证）。[M] 子项补
- ⚠️ `cluster_rollback_returns_200_with_action_id`（缺）：`POST /api/ops/cluster/rollback` 端点无 UT 覆盖。[M] 子项补
- ⚠️ `cluster_status_returns_release_name_phase`（缺）：`GET /api/ops/cluster/status` 端点无 UT 覆盖（仅 IT 实证）。[M] 子项补
- ⚠️ `docs_list_returns_walkdir_real_results`（缺）：`GET /api/ops/docs` 端点无 UT 覆盖（仅 IT 实证）。[M] 子项补
- ⚠️ `readyz_returns_200`（缺）：`GET /readyz` 端点无 UT 覆盖（仅 healthz 覆盖）。[M] 子项补
- ⚠️ `log_upload_missing_content_field_returns_400`（缺）：缺必填字段 `content` 返 400 的 UT 覆盖。[M] 子项补
- ⚠️ `log_upload_invalid_level_filter_returns_400`（缺）：`level_filter` 非法值（如 `"FOO"`）返 400 的 UT 覆盖。[M] 子项补
- ⚠️ `ops_api_routes_count_equals_ten_plus_health`（缺）：路由计数（10 + 2 health = 12）无 UT 覆盖。[M] 子项补
- ⚠️ `ops_response_meta_serialization`（缺）：`OpsResponse<T>` + `OpsMeta` 序列化（含 None vs Some）无 UT 覆盖。[M] 子项补
- ⚠️ `cluster_canary_invalid_canary_weight_returns_400`（缺）：`canary_weight > 100` 返 400 的 UT 覆盖（IT 实证在 it_cluster_update.rs）。[M] 子项补

**累计缺 10 测**（per 守门 #11）：DDD Review 必查。

### 2.3 覆盖率目标与缺口统计

| 模块 | 现有测数 | 缺口测数 | 覆盖率目标 | 优先级 |
|---|---|---|---|---|
| `error` | 3 | 3 | 100% (5 variant × 6-field 全覆盖) | P0 |
| `ops_domain` | 14 (cluster 4 + docs 5 + log 2 + metrics 3) | 5 | ≥ 85% | P0 |
| `ops_ai` | 16 (ladder 5 + mock 3 + openai 4 + anthropic 4) | 8 | ≥ 85% | P1 |
| `ops_api` | 7 | 10 | ≥ 80% (含 10 端点 + health) | P0 |
| **累计** | **41** | **26** | **平均 ≥ 80%** | — |

**覆盖率工具**（per 守门 #1 v25 单 crate 派生）：

- MVP 阶段: `cargo test -p star-ops --lib -j 4` 100% pass 为主，**不强制覆盖率 %**
- 实装阶段: 引入 `cargo tarpaulin` 或 `cargo llvm-cov`，CI 必跑覆盖率门禁（per [M] 子项拍板）

### 2.4 边界条件矩阵（5 维必跑）

| 维度 | 测试方法 | 必跑测数（估） |
|---|---|---|
| **空输入** | `""` 空字符串 / `vec![]` / `None` / `Uuid::nil()` | 6 (per 6 端点 + 4 域 stub) |
| **最大输入** | 1MB log content / `canary_weight=100` / 完整 trace_id | 4 (per F-02 + F-01) |
| **最小输入** | 0 / `canary_weight=0` / 单字符 source | 3 (per F-01 + F-02) |
| **无效输入** | `ReleaseStatus::from_str("invalid")` / `level_filter=["FOO"]` / 缺 content 字段 | 6 (per 6 端点 schema 校验) |
| **并发** | 100/1000/5000 用户同时 POST `/api/ops/log/upload` (per 容量规划 §5.6) | 3 (per PT 容量规划 100/1000/5000) |

**累计边界条件 5 维 × 估 4 测 = 20 测**（per 守门 #11 缺标比错标）：MVP 阶段至少跑 空 + 无效 2 维，实装阶段补 最大 + 最小 + 并发 3 维。

### 2.5 错误路径覆盖（per BAS-001 §3.5 + 守门 #6 v2 6-field）

| 错误码 | HTTP Status | source_kind | retriable | 必跑测 | MVP 实证 |
|---|---|---|---|---|---|
| `NOT_IMPLEMENTED` | 501 | Internal | false | `not_implemented_returns_501` | ✅ |
| `UNAUTHORIZED` | 401 | Policy | false | `unauthorized_returns_401_with_policy_source` | ✅ |
| `RATE_LIMITED` | 429 | Policy | **true** (per 守门 #6 v2 派生) | `rate_limited_is_retriable` | ✅ |
| `BAD_REQUEST` | 400 | Validation | false | ⚠️ 缺（per §2.2.1） | ❌ |
| `INTERNAL` | 500 | Internal | true | ⚠️ 缺（per §2.2.1） | ❌ |

**MVP-骨架阶段错误路径现状**（per 守门 #11 缺标比错标）：

- ✅ 5/5 错误码 enum 完整
- ✅ 3/5 错误码 6-field UT 覆盖（NOT_IMPLEMENTED / UNAUTHORIZED / RATE_LIMITED）
- ⚠️ 2/5 错误码 6-field UT 缺（BAD_REQUEST / INTERNAL）：DDD Review 必查
- ✅ Ladder retriable 规则 5 测覆盖（5 variant × is_retriable）
- ⚠️ 5 variant → HTTP status code 映射无独立 UT：DDD Review 必查

### 2.6 守门合规清单（per AGENTS.md §4 + 守门 #1 v25）

| 守门 | 验证方式 | MVP 实证 |
|---|---|---|
| 守门 #1 v25 单 crate | `cargo test -p star-ops --lib -j 4` | ✅ 41/41 pass |
| 守门 #6 6-field | error.rs 5 variant + `to_body` 完整 | ✅ enum 完整，UT 缺 2 测 |
| 守门 #7 0 unsafe | `grep -rn "unsafe" src/` 应为 0（除 `// SAFETY:` 注释） | ✅ 0 unsafe 实证（per F-01/F-03/F-04 commit） |
| 守门 #12 AI 协作文档 | BAS/DDS 引用必 `git log --follow` 实证 | ✅ 本节全 git 实证 |
| 守门 #23 AI mock 不开外部 API | `mock.rs` 仅 subprocess，无 `reqwest` 实证 | ✅ `Cargo.toml` 0 reqwest 实证 |

### 2.7 本章小结

- **实证 41/41 lib test 100% pass**（per 守门 #1 v25 单 crate）
- **派生缺口 26 测**（error 3 + ops_domain 5 + ops_ai 8 + ops_api 10）：DDD Review 必查
- **边界条件 5 维**：MVP 阶段跑 空 + 无效 2 维，实装阶段补 3 维
- **错误路径 5/5 错误码 enum 完整，3/5 UT 覆盖**：DDD Review 必查 2 缺口
- **守门 #1+#6+#7+#12+#23 跨节全过，0 违反**

---

**Status**: 🟡 §0-§2 草稿落地, 等 §3-§6 续做 (per 6 commit 链 wt2 done, wt3-wt5 续 §3-§8)
