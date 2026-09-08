# PR-IT-5-GAPS-001 — IT-5-GAPS 端到端实装 (5 缺口实装, 67+48 tests pass, 5 已知缺口 DDD Review 必查)

> **作者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 (per 9/8 15:19 JST 第 6 次强化 Mavis 全权代理)
> **日期**: 2026-09-08 JST
> **worktree**: `wt-ops-it-5-gaps` (6 commit 链, 1 brief + 5 wt commit)
> **基点**: `c828df5` (brief) + 5 wt commit (`ce2cacc` + `63aacb6` + `7e8704f` + `28c6f5d` + wt5)
> **基线 commit**: `c828df5`
> **守門**: 9 v20 子代理 dispatch 必先 brief, 1 R-05 不 push, 26 v26 merge main 必 PR 流程, 11 缺标比错标

## 0. 拍板实证 (per ask_user `ask_7f0b0267f4d54280ccb3a524`)

| # | 维度 | 拍板 |
|---|---|---|
| Q1 | 解决遗留 | **仅 cleanup worktree + branch** (5 缺口留给本 brief 实施, 7 worktree + 4 origin/wt-ops-* branch 清理完成 per worktree list 实证 25→15 worktrees + remote prune) |
| Q2 | IT 测试范围 | **§3 IT 5 缺口实装** (跟 IT scope 强联动, 估 ~1.5M-2M tokens, 5 commit 链) |
| Q3 | 交付形式 | **派子代理** (worktree + brief + 子代理 + owner check 5/5 + PR + merge) |

## 1. 改动总览

5 缺口端到端实装 (跟现有 67/67 lib + 42/42 IT baseline 1:1 对齐 + 5 缺口补齐 → **48 IT**):

| 缺口 | IT 测名 | 文件 | commit | 关键守門 |
|---|---|---|---|---|
| #1 真实 PG 容器化 | `it_real_pg_apply_ddl_smoke_real_pg` | `crates/star-ops/tests/it_db_integration.rs` | `ce2cacc` | #1 R-05 + #5 v2 + #13 + #DB-13 c |
| #2 RLS 13 類 cross-tenant | `it_rls_13_categories_enforced_real_pg` | `crates/star-ops/tests/it_db_integration.rs` | `63aacb6` | #1 R-05 + #DB-13 c + #DB-13 CW-05 |
| #3 Ladder L2 fallback | `it_ladder_l2_fallback_when_mock_fails` | `crates/star-ops/tests/it_log_ai.rs` | `7e8704f` | #6 v2 + #23 |
| #4 rate limit middleware | `it_rate_limit_middleware_60_rpm` | `crates/star-ops/tests/it_cross_module.rs` | `28c6f5d` | #6 v2 RATE_LIMITED retriable |
| #5 graceful shutdown | `it_graceful_shutdown_drains_in_flight` + `it_graceful_shutdown_blocks_new_requests_after_signal` | `crates/star-ops/tests/it_cross_module.rs` | (wt5) | #1 R-05 + #11 + #26 |

**关键变更**:
- `Cargo.toml`: +2 dev-deps (sqlx + tokio-util)
- `Cargo.lock`: auto
- 3 个 IT test 文件: +5 新 IT 测 (缺口 #1 #2 #3 #4 #5)
- `docs/reports/PHASE-IT-5-GAPS-REPORT.md`: 7 段 per AGENTS.md §3 模板 (新建)
- `docs/reports/PR-IT-5-GAPS-001.md`: 本文件 (新建)

## 2. 5 项守门实证 (per IT-5-GAPS-IMPL brief §3)

| # | 守门 | 验证方式 | 实证 |
|---|---|---|---|
| 1 | `cargo check` 0 err | `cargo check -p star-ops --all-targets -j 4` | ✅ 0 err |
| 2 | `cargo test --lib` 100% pass | `cargo test -p star-ops --lib -j 4` | ✅ 67/67 pass (41 baseline + 26 派生) |
| 3 | `cargo test --tests` 100% pass | `cargo test -p star-ops --tests -j 4` | ✅ 48/48 pass (15 baseline + 27 派生 + 5 缺口新增) |
| 4 | `cargo fmt` 0 错 | `cargo fmt -p star-ops --check` | ✅ 0 diff |
| 5 | `cargo clippy` 0 advisory (新测) | `cargo clippy -p star-ops --all-targets -j 4` | ✅ 0 advisory |

## 3. 5 已知缺口 (per 守門 #11 缺标比错标, DDD Review 必查)

| # | 缺口 | 等级 | 缓解 |
|---|---|---|---|
| #1 | 真实 PG 容器化部署 prod | P0 | WSL PG 路径 + testcontainers CI 端 (per 守門 #1 R-05) |
| #2 | RLS 13 類 性能 | P0 | NOBYPASSRLS 用户模拟, 6 表 13 類 隔离验证, 1000+ 并发待 [M] |
| #3 | Ladder L2/L3 真实 LLM | P1 | stub 阶段 no_network_mode=true (per 守門 #23 + #25 v25) |
| #4 | rate limit per actor user_id | P1 | per IP 60 req/min, prod 走 Redis (per 守門 #1 R-05) |
| #5 | graceful shutdown in-flight count + timeout | P1 | axum with_graceful_shutdown, 50 in-flight 全 200 |

5 项 [M] 子项 = DDD Review 必查.

## 4. 5 域 Lead 签字栏 (per 守門 #14 v2 拍板 D + 9/8 15:19 JST 第 6 次强化 Mavis 全权代理)

| 角色 | R | A | C | I | 责任人（真人到位前 Mavis 临时代签） | 签字日期 |
|---|---|---|---|---|---|---|
| **架构师** | ✅ | ✅ | — | — | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 JST |
| **SRE Lead** | ⏳ | ⏳ | — | — | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| **平台 Lead** | ⏳ | ⏳ | — | — | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| **评审主持** | ⏳ | ⏳ | — | — | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| **PM** | ⏳ | ⏳ | — | — | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |

## 5. 5 项守门 (per IT-5-GAPS-IMPL brief §3 守门实证)

- ✅ **缺口 #1 真实 PG 容器化跑通**: `cargo test -p star-ops --test it_db_integration it_real_pg_apply_ddl_smoke_real_pg` PASS (跟 6 ops 表 DDL 联动)
- ✅ **缺口 #2 RLS 13 類 隔离**: `cargo test -p star-ops --test it_db_integration it_rls_13_categories_enforced_real_pg` PASS
- ✅ **缺口 #3 Ladder L2 fallback**: `cargo test -p star-ops --test it_log_ai it_ladder_l2_fallback_when_mock_fails` PASS
- ✅ **缺口 #4 rate limit**: `cargo test -p star-ops --test it_cross_module it_rate_limit_middleware_60_rpm` PASS
- ✅ **缺口 #5 graceful shutdown**: `cargo test -p star-ops --test it_cross_module it_graceful_shutdown_drains_in_flight` PASS

## 6. 引用文档 (git 实证可查)

- 完整报告: `docs/reports/PHASE-IT-5-GAPS-REPORT.md` (本 PR 主报告)
- Brief: `docs/briefs/it-5-gaps-impl.md` v0.1
- 前置报告: `docs/reports/PHASE-UT-IT-51-REPORT.md` v0.1
- 设计文档: SRS-001 v0.1 + BAS-001 v0.1 + DDS-001 v0.1 + TEST-DESIGN-OPS-001 v0.2
- 3 DDL 文件: `db/migrations/2026-09-08-ops-{cluster,log,metrics}.sql`
- 8 PR 引用: #23 MVP + #25 F-02 + #27 F-01 + #28 F-03 + #29 F-04 + #30 TEST-DESIGN + #31 F-05 + #32 UT-IT-51
- 5 域 Lead 签字栏: per 守門 #14 v2

## 7. owner 拍板后续 (子代理不主动, per 守門 #1 R-05 + #26 v26)

- [ ] **owner 拍板推 origin** (`git push origin wt-ops-it-5-gaps`)
- [ ] **owner 开 PR** (`gh pr create --base main --head wt-ops-it-5-gaps`)
- [ ] **owner merge main** (review + 5 域 Lead 真人到位后追溯签字)
- [ ] **DDD Review 必查 5 已知缺口** (per §3)
- [ ] **WBS v0.16 升版** 累计 93/108 → 98/108 (+5 子项)

**不推 origin, 不开 PR, 不 merge main** (per 守門 #1 R-05 反转 8/30 拍板 + 守門 #26 v26 + 守門 #9 v20 子代理 dispatch 必先 brief + owner evidence check 5/5).
