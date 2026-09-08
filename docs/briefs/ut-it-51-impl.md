# UT-IT-51 派单 Brief — Ops Console §2 UT + §3 IT 派生缺口 51 项端到端实装

> **版本**: v0.1
> **作者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 (per 9/8 15:19 JST 第 6 次强化 Mavis 全权代理)
> **日期**: 2026-09-08 17:25 JST
> **触发**: 2026-09-08 17:20 JST 用户发令"实施ut测试" + 17:22 JST ask_user `ask_da44a894737108b4cd54bc8a` 3 拍板 (51 项派生 / F-02 log AI / 派子代理)
> **守門**: 9 v20 子代理 dispatch 必先 brief, 1 R-05 不 push (推 origin 必先 ask_user), 26 v26 merge main 必 PR 流程

## 0. 拍板实证 (per ask_user `ask_da44a894737108b4cd54bc8a`)

| # | 维度 | 拍板 |
|---|---|---|
| Q1 | UT 实施范围 | **§2 UT + §3 IT 全部派生缺口 51 项** (估 ~1.5M-2M tokens, [M] 子项) |
| Q2 | UT 起点模块 | **F-02 log AI 模块** (跟 F-05 ops-log.sql 3 表 DDL 强联动, MVP 最容易闭环) |
| Q3 | UT 交付流程 | **派子代理** (worktree + brief + 子代理 + owner check 5/5 + PR + merge, 跟 F-01/F-02/F-03/F-04/Test-Design/F-05 实战模式同) |

## 1. 目标

基于 TEST-DESIGN-OPS-001 v0.2 §2 UT + §3 IT 派生缺口 51 项, 端到端实装 UT/IT 测试代码, 跟现有 41/41 lib test + 15/15 IT 1:1 对齐 + 派生缺口补齐. F-05 ops-log.sql 3 表 DDL 落地后 IT `it_ops_log_ddl_wtm_coverage` 必跑通.

## 2. 范围

### 2.1 In-Scope (51 项派生缺口)

**§2 UT 派生缺口 26 项** (per TEST-DESIGN §2.3 列表, 跨 4 模块):
- **F-02 log AI 模块 (3 项, 跟 F-05 强联动, 优先)**:
  1. `log_analysis_stub_confidence_above_threshold_no_review` — confidence > 0.5 不需要 review
  2. `log_upload_missing_content_field_returns_400` — 缺 content 字段返 400
  3. `log_upload_invalid_level_filter_returns_400` — level_filter="FOO" 返 400
- **F-01 cluster 模块 (派生)**:
  4. `cluster_canary_invalid_weight_returns_400` — canary_weight > 100 返 400
  5. `cluster_rollback_invalid_action_returns_400` — action_type 不在 list 返 400
  6. `cluster_status_missing_release_returns_404` — 不存在 release 返 404
  7. `cluster_list_pagination_works` — pagination 边界
- **F-03 metrics 模块 (派生)**:
  8. `metrics_summary_filters_by_tenant` — tenant_id 隔离 RLS 验证
  9. `metrics_summary_returns_empty_when_no_data` — 空数据状态
  10. `metrics_summary_handles_division_by_zero` — 边界
  11. `metrics_summary_unit_validation` — unit 字段值验证
- **F-04 docs 模块 (派生)**:
  12. `doc_scanner_handles_empty_directory` — 空目录边界
  13. `doc_scanner_skips_hidden_files` — . 开头的隐藏文件
  14. `doc_category_from_path_handles_unknown` — 未知路径归 "Other"
  15. `doc_ref_title_handles_special_characters` — unicode 边界
  16. `doc_scanner_respects_max_depth` — 递归深度限制
- **ops_ai 模块 (派生)**:
  17. `ladder_retries_3_times_then_gives_up` — Ladder 重试边界
  18. `ladder_skips_disabled_channels` — 跳过 disabled 通道
  19. `mock_channel_always_succeeds` — mock 通道 100% 成功
  20. `openai_stub_returns_401_without_api_key` — 缺 key 401
  21. `anthropic_stub_returns_503_on_rate_limit` — 限流 503
- **ops_api 模块 (派生)**:
  22. `api_healthz_returns_200_with_version` — healthz 含版本
  23. `api_readyz_returns_503_when_db_down` — db 挂掉 503
  24. `api_correlation_id_echoes_back` — correlation_id 回显
  25. `api_request_id_generation` — 自动生成 request_id
  26. `api_cors_preflight_options` — CORS preflight 处理
- **error 模块 (派生)**:
  - (error 模块已 3/3 测 1:1, 派生 0 项, 计入 baseline)

**§3 IT 派生缺口 23 项** (per TEST-DESIGN §3.3 列表):
- **F-02 log AI IT (派生)**:
  27. `it_log_upload_invalid_level_filter_returns_400` (跟 F-05 联动, 跟 UT #3 配对)
  28. `it_log_analysis_returns_anomaly_on_error_log` — ERROR log 触发 anomaly
  29. `it_log_analysis_returns_no_anomaly_on_info_log` — INFO log 不触发
  30. `it_log_query_log_records_failed_request` — 失败请求审计
- **F-01 cluster IT (派生)**:
  31. `it_cluster_canary_validates_weight_range` — 边界 0-100
  32. `it_cluster_rollback_persists_audit_log` — audit log 落 audit_event
  33. `it_cluster_status_handles_concurrent_requests` — 并发 100
  34. `it_cluster_ddl_wtm_coverage` (跟 F-01 DDL 联动) — 已 1:1 ✅
- **F-03 metrics IT (派生)**:
  35. `it_metrics_summary_filters_by_tenant_id` — tenant 隔离
  36. `it_metrics_summary_handles_star_telemetry_failure` — telemetry 挂 fallback
  37. `it_metrics_summary_handles_empty_metrics_config` — 空配置
  38. `it_ops_metrics_ddl_wtm_coverage` — 已 1:1 ✅
- **F-04 docs IT (派生)**:
  39. `it_docs_list_walkdir_real_scan` — walkdir 真实跑 (跟 F-04 强联动)
  40. `it_docs_list_walkdir_scans_only_docs_subdirs` — 仅扫 docs/ 子目录
  41. `it_docs_list_handles_path_too_long` — 长路径
  42. `it_ops_log_ddl_wtm_coverage` — **F-05 联动**, 验证 db/migrations/2026-09-08-ops-log.sql 3 表 100% 覆盖 (F-05 PR #31 已落地, 跟 §2 IT 配对)
- **跨模块 IT (派生)**:
  43. `it_healthz_returns_200_with_db_ping` — healthz 验 db
  44. `it_readyz_returns_503_when_db_down` — db 挂 readyz 503
  45. `it_correlation_id_propagates_through_axum` — axum 中间件
  46. `it_rate_limit_returns_429_after_threshold` — 限流
  47. `it_request_size_limit_rejects_oversized_payload` — 1MB 限制
  48. `it_concurrent_requests_dont_corrupt_state` — 并发 1000
  49. `it_graceful_shutdown_drains_in_flight_requests` — 优雅关闭
- **DB 集成 IT (派生, sqlx + testcontainers)**:
  50. `it_real_pg_apply_ddl_smoke` — F-02 ops-log.sql + F-01 cluster.sql + F-03 metrics.sql 真实 PG 跑通
  51. `it_rls_13_categories_enforced` — 6 表 RLS 13 類验证 (per SRS-001 §8.2)

**累计 51 项派生缺口 (子代理必逐项实装, 不漏)**.

### 2.2 Out-of-Scope (per 守門 #1 R-05 + 守門 #11 + 守門 #23)

- 真实 K8s/Helm 集群 (走 helm_canary_mock.sh subprocess)
- 真实 LLM OpenAI/Anthropic (走 ai_log_mock.sh + ai_stub.rs)
- 真实 PG 跑 51 项全套 (1 项 [M] 子项 for sqlx + testcontainers, 估 ~200K token)
- 5 域 Lead 真人到位追溯签字
- E2E 浏览器自动化 (Playwright vs Cypress 待 DDD Review 拍板)
- 性能 bench log_upload P95 实证 (per TEST-DESIGN §4.6 缺口 #1, F-05 后续 sprint)

## 3. 守门实证 (per 守門 #1 + #1 v25 + #7 v3 + #9 v20 + #11 + #13)

子代理每 commit 必跑:
1. `cargo check -p star-ops --all-targets -j 4` → 0 err
2. `cargo test -p star-ops --lib -j 4` → 41+51 = 92/92 pass (含 51 项派生缺口, 设计书 baseline 41 测不破)
3. `cargo test -p star-ops --tests -j 4` → 56+23 = 79/79 IT pass (含 23 项 IT 派生, 1 项 F-05 ops-log DDL 联动)
4. `cargo fmt -p star-ops --check` → 0 错 (新文件 0 diff)
5. `cargo clippy -p star-ops --all-targets -j 4` → 0 err (新测 0 advisory)

## 4. 51 项派生缺口分组 (估 ~1.5M-2M tokens 累计)

| 阶段 | 范围 | 测数 | 估 token |
|---|---|---|---|
| **Phase 1** | F-02 log AI UT (3 项) + IT (4 项) = 7 项 (跟 F-05 强联动) | 7 | ~300K |
| **Phase 2** | F-01 cluster UT (4) + IT (3, 已有 1:1) = 7 项 | 7 | ~250K |
| **Phase 3** | F-03 metrics UT (4) + IT (3, 已有 1:1) = 7 项 | 7 | ~250K |
| **Phase 4** | F-04 docs UT (5) + IT (3, 已有 1:1) = 8 项 | 8 | ~300K |
| **Phase 5** | ops_ai UT (5) + IT (2) = 7 项 | 7 | ~250K |
| **Phase 6** | ops_api UT (5) + IT (4) + error (0) = 9 项 | 9 | ~300K |
| **Phase 7** | 跨模块 IT (7) + DB 集成 IT (2, sqlx + testcontainers) = 9 项 | 9 | ~400K |
| **累计** | 51 项 | 51 | **~2.05M** |

## 5. 7 commit 链 (跟 TEST-DESIGN 6 commit 模式同, +1 phase 7 DB 集成)

| # | 标题 | 内容 | 估 token |
|---|---|---|---|
| wt1 | feat(test): Phase 1 F-02 log AI UT 3 项 + IT 4 项 = 7 项 (跟 F-05 ops-log.sql 3 表联动) | F-02 模块派生缺口, 跟 F-05 强联动 | ~300K |
| wt2 | feat(test): Phase 2 F-01 cluster UT 4 项 + IT 3 项 (跟 helm_canary_mock.sh subprocess) | F-01 模块派生 | ~250K |
| wt3 | feat(test): Phase 3 F-03 metrics UT 4 项 + IT 3 项 (跟 star-telemetry 复用) | F-03 模块派生 | ~250K |
| wt4 | feat(test): Phase 4 F-04 docs UT 5 项 + IT 3 项 (跟 walkdir 真实扫描) | F-04 模块派生 | ~300K |
| wt5 | feat(test): Phase 5 ops_ai UT 5 项 + Phase 6 ops_api UT 5 项 = 10 项 (Ladder + 6-field) | Hybrid AI + ops_api 派生 | ~300K |
| wt6 | feat(test): Phase 6 ops_api IT 4 项 + Phase 7 跨模块 IT 7 项 = 11 项 (healthz/readyz/RateLimit/并发) | 跨模块 IT 派生 | ~300K |
| wt7 | feat(test): Phase 7 DB 集成 IT 2 项 (sqlx + testcontainers 真实 PG, 验证 6 ops 表 DDL 100% 覆盖 + RLS 13 類) + docs(phase) PHASE-UT-IT-51-REPORT v0.1 (7 段 per AGENTS.md §3) + PR 描述 | DB 集成 + 报告 | ~250K |
| **累计** | | | **~2.05M** |

## 6. owner evidence check 5/5 准备 (per 守門 #9 主体)

1. **51 项派生缺口全实装** (`cargo test -p star-ops --lib | grep "test result" | Measure-Object` 跟 baseline 41 测加 51 派生 = 92/92)
2. **5 cargo test 守门全 PASS** (lib 92/92 + tests 79/79 + check 0 err + fmt 0 错 + clippy 0 err)
3. **F-05 ops-log.sql 3 表 DDL 联动跑通** (it_ops_log_ddl_wtm_coverage PASS, 跟 PR #31 联动)
4. **8 ahead of origin/main** (1 brief + 7 wt commit 链)
5. **20 维守門 0 违反** + **3 缺口显式标** (per 守門 #11 缺标比错标, DDD Review 必查)

## 7. 已知缺口 (per 守門 #11 缺标比错标, 子代理必标)

1. **真实 PG 容器化跑 6 表 DDL** ([M] 子项 sqlx + testcontainers, 估 ~200K token, Phase 7 wt7 部分实装)
2. **RLS 13 類 cross-tenant 隔离** ([M] 子项, MVP 阶段 TODO)
3. **log_upload_bench P95** (per TEST-DESIGN §4.6 缺口 #1, F-05 后续 sprint)
4. **E2E 浏览器自动化** (per TEST-DESIGN §4.6 缺口 #2, owner 拍板 Playwright/Cypress)
5. **5 域 Lead 真人到位追溯签字** (per 守門 #14 v2 拍板 D 维持)

**DDD Review 必查**: 缺口 #1 (real PG 容器化) + #2 (RLS 13 類) + #3 (log_upload_bench).

## 8. 引用文档 (git 实证可查)

- `D:\Star\.worktrees\wt-ops-ut-it-51\docs\test-design\TEST-DESIGN-OPS-001.md` v0.2 (75KB, 10 章节, §2 UT 派生 26 + §3 IT 派生 23 缺口列表)
- `D:\Star\.worktrees\wt-ops-ut-it-51\docs\requirements\SRS-STAR-OPS-001.md` v0.1 (22.8KB, §8.1 6 表 W/T/M + §8.2 RLS 13 類)
- `D:\Star\.worktrees\wt-ops-ut-it-51\docs\basic-design\OPS-BASIC-DESIGN-001.md` v0.1 (18KB, 4 模块 + 错误码 6-field)
- `D:\Star\.worktrees\wt-ops-ut-it-51\docs\detailed-design\OPS-DETAILED-DESIGN-001.md` v0.1 (45.7KB, 5 模块 + Hybrid AI Ladder)
- `D:\Star\.worktrees\wt-ops-ut-it-51\db\migrations\2026-09-08-ops-cluster.sql` (F-01 2 T 表, 物理删除禁止 + audit trigger + RLS 13 類)
- `D:\Star\.worktrees\wt-ops-ut-it-51\db\migrations\2026-09-08-ops-log.sql` (F-05 3 表, 1 T + 2 W TTL 7d/30d)
- `D:\Star\.worktrees\wt-ops-ut-it-51\db\migrations\2026-09-08-ops-metrics.sql` (F-03 1 M SCD2)
- `D:\Star\.worktrees\wt-ops-ut-it-51\crates\star-ops\src\` (14 文件, 41/41 lib test + 15/15 IT baseline)
- `D:\Star\.worktrees\wt-ops-ut-it-51\docs\briefs\test-design-ops-001.md` v0.1 (TEST-DESIGN brief)
- `D:\Star\.worktrees\wt-ops-ut-it-51\docs\briefs\ops-f0{1,2,3,4}-*.md` (F-01..F-04 brief 4 份)
- 6 PR merge: [#23 MVP](https://github.com/UlyssesLeoLee/Star/pull/23) / [#25 F-02](https://github.com/UlyssesLeoLee/Star/pull/25) / [#27 F-01](https://github.com/UlyssesLeoLee/Star/pull/27) / [#28 F-03](https://github.com/UlyssesLeoLee/Star/pull/28) / [#29 F-04](https://github.com/UlyssesLeoLee/Star/pull/29) / [#30 TEST-DESIGN](https://github.com/UlyssesLeoLee/Star/pull/30) / [#31 F-05](https://github.com/UlyssesLeoLee/Star/pull/31)
- 8 commit hash (brief 8325cce + 6 TEST-DESIGN 59a38bd/9cb3bd5/aba3825/3e29678/90fef91/6c310b3 + WBS v0.13 fff73c1 + self-review 029cc9f + F-05 31cb163 + WBS v0.14 59573bd)
- `AGENTS.md` §4 守門 20 维 (本次 0 违反)

---

**Status**: 🟡 brief 落档, 等 owner push origin, 派 worker 子代理 (估 ~2.05M tokens / 60-90 min, 7 commit 链 + 1 报告).

**不推 origin, 不派子代理** (per 守門 #1 反转 8/30 拍板 + 守門 #9 v20 子代理 dispatch 必先 brief + owner evidence check 5/5).
