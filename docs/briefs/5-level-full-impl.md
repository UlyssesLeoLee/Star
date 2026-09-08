# 5-LEVEL-FULL 派单 Brief — Ops Console TEST-DESIGN 5 级别全闭环 (§4 E2E + §5 PT + §6 UAT)

> **版本**: v0.1
> **作者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 (per 9/8 15:19 JST 第 6 次强化 Mavis 全权代理)
> **日期**: 2026-09-08 19:55 JST
> **触发**: 2026-09-08 19:50 JST 用户发令"继续推进测试到 uat 完成" + 19:52 JST ask_user `ask_dd75ea9ea02de7a368133200` 2 拍板 (5 级别全闭环 / 派子代理) + 19:53 JST ask_user `ask_ed07c5b49e82886c449a43df` Playwright 拍板
> **守門**: 9 v20 子代理 dispatch 必先 brief, 1 R-05 不 push, 26 v26 merge main 必 PR 流程

## 0. 拍板实证

| ask_user | 维度 | 拍板 |
|---|---|---|
| `ask_dd75ea9ea02de7a368133200` Q1 | UAT 范围 | **§4 E2E + §5 PT + §6 UAT 5 级别全闭环** (估 ~2.5M-4M tokens, 7-9 commit 链) |
| `ask_dd75ea9ea02de7a368133200` Q2 | 交付形式 | **派子代理** (worktree + brief + 子代理 + owner check 5/5 + PR + merge) |
| `ask_ed07c5b49e82886c449a43df` | E2E 浏览器选型 | **Playwright** (跨浏览器 + TypeScript 生态跟 ops-console 对齐) |

## 1. 目标

实施 TEST-DESIGN-OPS-001 v0.2 5 级别全闭环最后 3 章节 (§4 E2E + §5 PT + §6 UAT), 跟现有 51 项 UT-IT-51 派生 + 5 IT-5-GAPS 缺口 (累计 67/67 lib + 48/48 IT = 115/115 tests pass) 联动闭环. 关闭 §14.10.4 缺口 #2/#5 全部 + §4.6 缺口 #1/#2 + §6.6 已知.

## 2. 范围

### 2.1 In-Scope (3 章节 5 级别全闭环)

**§4 E2E 端到端测试** (Playwright 实施, 估 ~1.5M-2M tokens, 3-4 commit 链):
- §4.1 浏览器自动化范围 (Playwright + Chromium / Firefox / WebKit 跨浏览器)
- §4.2 4 tab 端到端 (Cluster / Log AI / Metrics / Docs × 10 端点 + i18n 3 语言 zh-CN/en/ja)
- §4.3 i18n 3 语言端到端验证
- §4.4 错误码 6-field 闭环 (per 守門 #6 v2)
- §4.5 16 守门规则清单 (per AGENTS.md §4)
- §4.6 已知缺口 (per 守門 #11 缺标比错标, 5 已知: log_upload_bench P95 / E2E 浏览器选型 / F-02 ops-log.sql 5 表 / RLS 13 類验证 / 5 域 Lead 真人到位)

**§5 PT 性能测试** (跟 F-05 联动, 估 ~500K-800K tokens, 2 commit 链):
- §5.1 性能测试目标 (守門 #7 v3 P95<200ms 硬约束)
- §5.2 已有 bench 3 个 (cluster_bench P95=49ms / metrics_bench P95=0.83μs / docs_bench P95=3.8ms)
- §5.3 **新增 log_upload_bench** (跟 F-05 联动, 估 P95 < 200ms 实证)
- §5.4 容量规划 (100 / 1000 / 5000 用户, 跟守門 #1 NFR-OP-005)
- §5.5 16 守门规则清单

**§6 UAT 验收测试** (估 ~500K-1M tokens, 2-3 commit 链):
- §6.1 验收目标 (8 AC + 4 类功能 + 5 维 NFR + 5 错误码 6-field)
- §6.2 验收用例矩阵 (4 tab × 10 端点 = 40 测, 跟 §4 E2E 联动)
- §6.3 验收环境 (4 环境: dev / staging / prod / canary)
- §6.4 验收标准 (DOD: Definition of Done, 跟 5 域 Lead 签字栏联动)
- §6.5 RACI 角色与责任 (5 域 Lead Mavis 临时代签, 真人到位追溯签字)
- §6.6 已知缺口 (per 守門 #11 缺标比错标)

**8 commit 链估 (~2.5M-4M tokens 累计)**:
- wt1: §4.1+§4.2 Playwright setup + 4 tab 端到端 (~600K)
- wt2: §4.3+§4.4+§4.5 i18n + 6-field + 16 守门 (~600K)
- wt3: §5.1+§5.2+§5.3 性能测试 + log_upload_bench (~500K)
- wt4: §5.4+§5.5 容量规划 + 16 守门 (~300K)
- wt5: §6.1+§6.2+§6.3 验收目标 + 用例矩阵 + 环境 (~400K)
- wt6: §6.4+§6.5+§6.6 DOD + RACI + 缺口 (~400K)
- wt7: docs(phase): PHASE-5-LEVEL-FULL-REPORT v0.1 (7 段 per AGENTS.md §3) + PR 描述 (~200K)

### 2.2 Out-of-Scope (per 守門 #1 R-05 + 守門 #11 + 守門 #23 + 守門 #26 v26)

- 真实 K8s/Helm 集群 (走 helm_canary_mock.sh subprocess, per 守門 #1 R-05 mock 路径)
- 真实 LLM OpenAI/Anthropic (走 ai_log_mock.sh + ai_stub.rs, per 守門 #23)
- 真实 PG 部署 prod (per IT-5-GAPS 缺口 #1 [M] 子项待 owner 拍板)
- 5 域 Lead 真人到位追溯签字 (per 守門 #14 v2 拍板 D 维持)
- 业务代码修改 (per UT-IT-51 owner evidence check 5b 守門 #11 派生)
- Cypress 实施 (per拍板 e2e-stack 选 Playwright, Cypress 备选后续 sprint)
- ops_api.rs + ops_domain/* + ops_ai/* 业务代码 (per 守門 #11 缺标比错标 = 不动 baseline 67 lib + 48 IT)

## 3. 守门实证 (per 守門 #1 + #1 v25 + #7 v3 + #9 v20 + #11 + #13)

子代理每 commit 必跑:
1. `cargo check -p star-ops --all-targets -j 4` → 0 err
2. `cargo test -p star-ops --lib -j 4` → 67/67 pass (baseline + 派生不破, 设计书不动业务代码)
3. `cargo test -p star-ops --tests -j 4` → 48/48 IT pass (baseline + 派生不破, IT-5-GAPS 5 缺口 + UT-IT-51 27 派生)
4. `cargo bench -p star-ops --bench docs_bench -- --quick` → 3.8ms P95 (跟 F-04 实证)
5. `cd frontend && pnpm install --frozen-lockfile && & ".\node_modules\.bin\tsc.cmd" --noEmit` → 0 错 (跟 F-02/F-03/F-04 实证)
6. `cd frontend && npx playwright test` (新增 §4 E2E) → 0 错 (跨浏览器覆盖)
7. `cargo fmt -p star-ops --check` + `cargo clippy -p star-ops --all-targets -j 4` → 0 err

## 4. 7 commit 链 (跟 UT-IT-51 7 commit 模式同)

| # | 标题 | 内容 | 估 token |
|---|---|---|---|
| wt1 | feat(e2e): §4.1+§4.2 Playwright setup + 4 tab × 10 端点端到端 (跨 Chromium/Firefox/WebKit) | §4 E2E 主体 | ~600K |
| wt2 | feat(e2e): §4.3+§4.4+§4.5 i18n 3 语言 + 6-field 错误码 + 16 守门 | §4 E2E 完善 | ~600K |
| wt3 | feat(pt): §5.1+§5.2+§5.3 log_upload_bench P95 < 200ms 实证 (跟 F-05 联动) | §5 PT 主体 | ~500K |
| wt4 | feat(pt): §5.4+§5.5 容量规划 (100/1000/5000 用户) + 16 守门 | §5 PT 完善 | ~300K |
| wt5 | feat(uat): §6.1+§6.2+§6.3 验收目标 + 用例矩阵 + 4 环境 (dev/staging/prod/canary) | §6 UAT 主体 | ~400K |
| wt6 | feat(uat): §6.4+§6.5+§6.6 DOD + RACI 5 域 Lead 临时代签 + 已知缺口 | §6 UAT 完善 | ~400K |
| wt7 | docs(phase): PHASE-5-LEVEL-FULL-REPORT v0.1 (7 段 per AGENTS.md §3) + PR 描述 | 报告 + 收官 | ~200K |
| **累计** | | | **~3.0M** |

## 5. owner evidence check 5/5 准备 (per 守門 #9 主体)

1. **5 级别全闭环 3 章节落档** (7 commit 链 + 1 brief = 8 ahead origin/main)
2. **5 cargo + 1 frontend 守门全 PASS** (check 0 err + lib 67/67 + tests 48/48 + bench P95 < 200ms + tsc 0 错 + playwright 0 错)
3. **§4 E2E Playwright 跑通** (跨浏览器 3 个 + 4 tab × 10 端点 = 40 测, 跟 ops-console TypeScript 栈对齐)
4. **§5 PT log_upload_bench 跑通** (跟 F-05 ops-log.sql 3 表联动, P95 < 200ms)
5. **§6 UAT 验收矩阵 + 5 域 Lead 签字栏** (4 验收环境 + RACI + DOD, 跟守門 #14 v2 拍板 D 维持)
6. **20 维守門 0 违反** + **5 已知缺口显式标** (per 守門 #11 缺标比错标, DDD Review 必查)

## 6. 已知缺口 (per 守門 #11 缺标比错标, 子代理必标)

1. **Playwright 浏览器 binary 下载** ([M] 子项, 估 ~500K token, MVP 阶段 chromium-only 推荐, 跨浏览器 留给 CI 跑)
2. **真实 PG prod 部署** (per IT-5-GAPS 缺口 #1 P0 [M] 子项)
3. **RLS 13 類性能 1000+ 并发** (per IT-5-GAPS 缺口 #2 P0 [M] 子项)
4. **Hybrid AI 真实 LLM 通道** (OpenAI/Anthropic, per §14.10.4 缺口 #2 [M] 子项)
5. **5 域 Lead 真人到位追溯签字** (per 守門 #14 v2 拍板 D 维持)

**DDD Review 必查**: 缺口 #1 (Playwright 跨浏览器) + #2 (prod PG) + #3 (RLS perf) + #4 (Hybrid AI real).

## 7. 引用文档 (git 实证可查)

- `D:\Star\.worktrees\wt-ops-5-level-full\docs\test-design\TEST-DESIGN-OPS-001.md` v0.2 (75KB, 10 章节, §4 E2E + §5 PT + §6 UAT 3 章节 详细)
- `D:\Star\.worktrees\wt-ops-5-level-full\docs\requirements\SRS-STAR-OPS-001.md` v0.1 (8 AC + 4 类功能 + 5 维 NFR + 5 错误码 6-field)
- `D:\Star\.worktrees\wt-ops-5-level-full\docs\basic-design\OPS-BASIC-DESIGN-001.md` v0.1
- `D:\Star\.worktrees\wt-ops-5-level-full\docs\detailed-design\OPS-DETAILED-DESIGN-001.md` v0.1
- `D:\Star\.worktrees\wt-ops-5-level-full\db\migrations\` (3 ops SQL: F-01 2 T + F-05 3 W/T + F-03 1 M SCD2)
- `D:\Star\.worktrees\wt-ops-5-level-full\frontend\src\app\ops\` (4 tab × 10 端点 + i18n 3 语言, 跟 §4 E2E 联动)
- `D:\Star\.worktrees\wt-ops-5-level-full\crates\star-ops\benches\` (3 bench: cluster/metrics/docs + 新增 log_upload_bench)
- `D:\Star\.worktrees\wt-ops-5-level-full\docs\briefs\it-5-gaps-impl.md` v0.1 (IT-5-GAPS brief, 5 缺口)
- `D:\Star\.worktrees\wt-ops-5-level-full\docs\briefs\ut-it-51-impl.md` v0.1 (UT-IT-51 brief, 51 派生缺口)
- 9 PR merge 链接: [#23](https://github.com/UlyssesLeoLee/Star/pull/23) / [#25](https://github.com/UlyssesLeoLee/Star/pull/25) / [#27](https://github.com/UlyssesLeoLee/Star/pull/27) / [#28](https://github.com/UlyssesLeoLee/Star/pull/28) / [#29](https://github.com/UlyssesLeoLee/Star/pull/29) / [#30 TEST-DESIGN](https://github.com/UlyssesLeoLee/Star/pull/30) / [#31 F-05](https://github.com/UlyssesLeoLee/Star/pull/31) / [#32 UT-IT-51](https://github.com/UlyssesLeoLee/Star/pull/32) / [#33 IT-5-GAPS](https://github.com/UlyssesLeoLee/Star/pull/33)
- 9 commit hash (PR merge + WBS): `97810c0d` `472bab2` `d8e916e` `8a08756` `73623a7` `74582a7` `31cb163` `92bbcd6` `77be968` + WBS `fff73c1` `029cc9f` `59573bd` `25f806e` `2c20a26`
- `AGENTS.md` §4 守門 20 维 (本次 0 违反)

---

**Status**: 🟡 brief 落档, 等 owner push origin, 派 worker 子代理 (估 ~3.0M tokens / 60-120 min, 7 commit 链 + 1 报告).

**不推 origin, 不派子代理** (per 守門 #1 反转 8/30 拍板 + 守門 #9 v20 子代理 dispatch 必先 brief + owner evidence check 5/5).
