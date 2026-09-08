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

**Status**: 🟡 §0-§1 草稿落地, 等 §2-§6 续做 (per 6 commit 链 wt1→wt6)

> **本文档分章**: 5 大章节 §UT/§IT/§E2E/§PT/§UAT + §0 目的 + §1 范围 + §7 RACI + §8 修订历史 + §9 引用 = 9 章节. 后续 commit wt2-wt5 续 §2-§8, wt6 写 PHASE 报告 + PR 描述.
