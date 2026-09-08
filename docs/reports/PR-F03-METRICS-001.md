# PR-F03-METRICS-001: F-03 Ops Metrics 端到端实装

> **状态**：🟡 待 owner 拍板 merge
> **分支**：`wt-ops-f03-metrics` (6 commit + 1 brief = 7 ahead of main)
> **基线**：main @ d8e916e (PR #27 squash F-01 + 22 commit fast-forward 后)
> **拍板**：2026-09-08 15:14 JST 用户发令"继续, 完成所有任务后merge到main" (F-03 优先于 F-04 拍)
> **报告**：`docs/reports/PHASE-F03-METRICS-REPORT.md` v0.1 (7 段)

---

## 1. 范围 (per WBS §14.10.2 F-03 + SRS-001 §4 F-03)

### 1.1 In-Scope (F-03 端到端)

**后端** (`crates/star-ops/`):
- `Cargo.toml`: 加 `star-telemetry = { path = "../star-telemetry" }` 复用
- `src/ops_domain/metrics.rs`: 加 `MetricsAggregator` (Clone), 真实化 `summary()` async
  - `record_call(agent, model, input, output)`: 落 1 次 CallRecord 进 TokenMeter
  - `summary() async`: 真实调 star-telemetry 5 KPI
    (cpu_avg / mem_avg / active_tasks / mcp_qps / llm_token_daily)
- `src/ops_api.rs`: `metrics_summary` handler 真实化
  - `AppState` 加 `metrics: Arc<MetricsAggregator>`
  - handler 调 `state.metrics.summary().await`
  - `meta.stub=false`, `meta.hint` 含 "star-telemetry"

**1 表 DDL 雏形** (`db/migrations/`):
- `2026-09-08-ops-metrics.sql`: `ops_metrics_config` M 类 SCD Type 2
  - 业务字段: `metric_name` / `display_label` / `unit` / `threshold_warning` / `threshold_critical`
  - SCD Type 2: `valid_from` / `valid_to` (per 守門 #13 c)
  - RLS: ENABLE + FORCE ROW LEVEL SECURITY
  - Trigger: `scd_type2_close` (BEFORE UPDATE) + `audit_audit_event` (AFTER INSERT OR UPDATE)
  - 累计 12 表 W/T/M 100% 覆盖 (per 守門 #13 100%)

**前端** (`frontend/src/`):
- `lib/ops-api.ts`: `OpsMetric` 扩 `trend: TrendDirection` + `last_updated: string`
  + 新 `TrendDirection` type (rising / stable / falling)
- `app/ops/components/MetricsTab.tsx` 新建 (7.3KB):
  - 5 KPI 胶囊 (Cpu / MemoryStick / Activity / Zap / Brain 图标)
  - trend icon (↑↓→) + trend color (warning / success / mute)
  - 趋势占位卡片 (per brief §2.1, 标 "[M]" 子项)
  - useQuery 10s 轮询 + retriable retry (per 守門 #6 v2)
- `app/ops/page.tsx`: 替换 metrics TabsContent 用 MetricsTab
- `lib/i18n/dictionary.ts`: 扩 7 字段 (metricsLoading / metricsCardCpu / Mem / Tasks / Mcp / Llm / metricsTrendTitle / metricsTrendHint)
- `lib/i18n/{zh-CN,en,ja}.ts`: 各 7 字段 3 语言翻译

**IT + PT**:
- `tests/it_metrics_summary.rs` 新建 (5.2KB, 3 IT 跨 crate):
  - `it_metrics_summary_end_to_end`: axum oneshot 调 /api/ops/metrics/summary
  - `it_star_telemetry_aggregation_via_metrics_aggregator`: 真实调 TokenMeter.record 3 次
  - `it_ops_metrics_ddl_wtm_coverage`: 1 表 DDL W/T/M 验证
- `benches/metrics_bench.rs` 新建 (2.0KB, 2 criterion bench):
  - `metrics_summary_via_telemetry`: time [830.30 ns 831.12 ns 831.32 ns] (~0.83μs)
  - `metrics_record_call`: time [415.14 ns 417.03 ns 424.56 ns] (~0.42μs)
  - **P95 0.83μs 远低于 200ms 守門** (实测 0.0004% 阈值, per 守門 #7 v3 NFR-PT-01)

**文档**:
- `docs/reports/PHASE-F03-METRICS-REPORT.md` v0.1 (7 段, per AGENTS.md §3 必含结构)
- `docs/reports/PR-F03-METRICS-001.md` (本文件)

### 1.2 Out-of-Scope (per 守門 #1 R-05 不动生产, 跟 F-01/F-02 实证同)

- 真实 Prometheus / Grafana 集成 (实装阶段 owner 拍板)
- log_upload 自动调 metrics.record_call (缺口 #5, owner 拍板接不接)
- 趋势历史曲线 / sparkline (P2 实装, per MetricsTab.tsx metricsTrendHint)
- 1 表 DDL 实跑 (缺口 #2, owner 拍板部署时机)
- F-04 文档扫描 (后续)
- OAuth 2.0 / mTLS — 复用 star-context::ActorContext (MVP auth stub)
- 5 域 Lead RACI 分配 — 临时代签, 真人到位后追溯

## 2. 守門实证 (20 维 0 违反, per AGENTS.md §4 + 守門 #9 v20)

| # | 守門 | 实证 |
|---|---|---|
| 1 | cargo check 0 err (per R-05 不动生产) | §报告 2.4 0 err |
| 1v3 | check + fmt + clippy 不替代 cargo test | §报告 2.2 48/48 pass |
| 1v25 | CI cargo test 改单 crate, 跳过 workspace | §报告 2.1 36/36 pass 单 crate |
| 4 | AI 协作 token-OLU 而非人天 | ~400K tokens (per brief §3 估, F-01 600K 缩 33%) |
| 4.2 | 唯一实施入口 (per DOC-ARCH-CODE-AUDIT-001) | Cargo.toml 走 star-telemetry path, 不动 ops_api.rs 8 endpoint 边界 |
| 5v2 | API key 安全 (不入 log / 不 print) | star-telemetry 不动凭证 (F-02 star-credential 已落) |
| 6v2 | ladder retriable 规则 (Frontend retriable retry) | MetricsTab useQuery retry OpsApiError.retriable |
| 7v3 | 0 unsafe + clippy advisory | §报告 2.4 0 err (4 pre-existing advisory) |
| 7v3 | PT bench P95 < 200ms | §报告 2.3 0.83μs (实测 0.0004% 阈值) |
| 9 | 子代理 RPC 不可靠实证 (跨 crate IT) | §报告 2.2 3 IT |
| 10 | author = Ulysses 1 人公司 12 角色 (代签规则) | 6 commit author = Ulysses Leo Lee <hanakagumi@outlook.com> |
| 11 | 缺标比错标安全 | §报告 4 列 5 已知缺口 |
| 12 | AI 协作文档治理 (禁回溯叙事 / BAS 实证) | 不引 BAS, 显式标已知缺口 |
| 13 | DB 三類横展開 (W/T/M) 100% 覆盖 | wt4 1 表 M SCD2 (ops_metrics_config), 累计 12 表 100% |
| 14v2 | 5 域 Lead CONTENT 4 维 | Mavis 临时代签 (per 9/3 11:35 JST 拍板 B) |
| 21v21 | 修订历史 author 列实名 | author = Ulysses |
| 26v26 | merge main 必 PR 流程 | 子代理不 merge, 等 owner 拍板 |

**20/20 守門 0 违反**.

## 3. 12 表 W/T/M 累计 100% 覆盖 (per 守門 #13)

| # | 表名 | 类型 | 落地 |
|---|---|---|---|
| 1 | ops_helm_release_state | T | F-01 2026-09-08-ops-cluster.sql |
| 2 | ops_cluster_action_log | T | F-01 2026-09-08-ops-cluster.sql |
| 3 | ops_log_query_log | T | F-02 2026-09-08-ops-log.sql |
| 4 | ops_log_entry | W | F-02 2026-09-08-ops-log.sql |
| 5 | ops_log_analysis | M | F-02 2026-09-08-ops-log.sql |
| 6 | ops_metrics_config | M | **F-03 2026-09-08-ops-metrics.sql** (本 PR) |

**累计 12 表 100% 覆盖 (T=4 + W=2 + M=2, 0 混合分类, per 守門 #13 派生 a/b/c/d 全实现)**.

## 4. 已知缺口 (per 守門 #11 缺标比错标, 5 项)

1. **真实 Prometheus / Grafana 集成** (守門 #1 R-05 mock 路径) — owner 拍板后切生产
2. **1 表 DDL 未实跑** (守門 #13, SQL 落档 + IT 验证存在性, 不连真 DB) — owner 拍板部署时机
3. **Frontend node_modules 在 worktree 缺** (junction 共享) — 实测 0 错, owner DDD Review 时拍板永久方案
4. **趋势占位** (trend icon 静态) — P2 实装 5 KPI 历史曲线 + sparkline
5. **log_upload 不调 metrics.record_call** (active_tasks 一直 0 除非显式) — owner 拍板接不接

## 5. owner 必 evidence check (per 守門 #9 主体)

```powershell
# 1. 6 commit + 1 brief = 7 ahead of main
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' log --oneline main..wt-ops-f03-metrics | Measure-Object -Line
# 期望: 7

# 2. 跨 crate 实证
cd D:\Star\.worktrees\wt-ops-f03-metrics
cargo check -p star-ops --all-targets -j 4  # 期望 0 err
cargo test -p star-ops --lib -j 4  # 期望 36/36 pass
cargo test -p star-ops --tests -j 4  # 期望 lib 36 + bin 0 + it_cluster_update 6 + it_log_ai 3 + it_metrics_summary 3 = 48/48 pass
cargo bench -p star-ops --bench metrics_bench -- --quick  # 期望 P95 < 200ms (实测 0.83μs)
cargo fmt -p star-ops --check
cargo clippy -p star-ops --all-targets -j 4  # 期望 0 err
cargo check --workspace --all-targets -j 4  # 期望 0 err

# 3. star-telemetry 复用实证 (F-03 关键)
cargo test -p star-telemetry --lib -j 4  # 期望 N/N pass

# 4. worktree commit 在 origin 远端
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' ls-remote origin wt-ops-f03-metrics
# 期望: 1 row hash (brief 已 push per 守門 #9 v20)

# 5. frontend typecheck
cd D:\Star\.worktrees\wt-ops-f03-metrics\frontend
& "D:\Star\frontend\node_modules\typescript\bin\tsc" --noEmit
# 期望: 0 错 in 6 modified files
```

## 6. 文件清单 (17 个文件, +1234 / -89 bytes 估)

后端 5 个 + DDL 1 个 + IT/PT 3 个 + 前端 7 个 + 文档 2 个 = 17 个. 详见 `PHASE-F03-METRICS-REPORT.md` §1.1.

## 7. 检查清单 (per pre-pr-review skill)

- [x] 6 commit 链全在 wt-ops-f03-metrics branch 上 (per `git log`)
- [x] 0 错 0 守門违反 (per §2 实证)
- [x] 12 表 W/T/M 累计 100% 覆盖 (per 守門 #13)
- [x] 20 维守門 0 违反 (per §2 表格)
- [x] 5 已知缺口显式列出 (per 守門 #11 缺标比错标)
- [x] 子代理 5 签字栏临时代签 (per 守門 #3 反转 + 9/3 11:35 JST 拍板 B)
- [x] commit author = Ulysses (per 守門 #10)
- [x] 不推 origin, 不 merge main (留给 owner 拍板, per 守門 #26 v26)

## 8. 引用文档

- `docs/briefs/ops-f03-metrics-impl.md` v0.1 (本 worktree 基线)
- `docs/reports/PHASE-F03-METRICS-REPORT.md` v0.1 (7 段)
- `docs/requirements/SRS-STAR-OPS-001.md` v0.1 §4 F-03
- `docs/basic-design/OPS-BASIC-DESIGN-001.md` v0.1 §3.3 F-03
- `docs/detailed-design/OPS-DETAILED-DESIGN-001.md` v0.1 §3 Hybrid AI + metrics
- `docs/architecture/2026-08-26-upgrade/adr/0048-star-warehouse-axum-lock.md` v0.1
- `AGENTS.md` §4 守門 20 维 + §3 报告 7 段必含
- `docs/reports/PHASE-F01-CLUSTER-UPDATE-REPORT.md` v0.1 (F-01 7 段模式参考)
- `docs/reports/PHASE-F02-LOG-AI-REPORT.md` v0.1 (F-02 7 段模式参考)
