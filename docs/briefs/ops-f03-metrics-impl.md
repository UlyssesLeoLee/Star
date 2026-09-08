# Brief: F-03 Ops Metrics 端到端实装 (per WBS §14.10.2 + 拍板 9/8 15:14 JST)

> **状态**: 🟡 Brief v0.1
> **拍板**: 2026-09-08 15:14 JST 用户发令"继续, 完成所有任务后merge到main" (F-03 优先于 F-04 拍, per token-OLU 估 400K > 200K)
> **wt-branch**: `wt-ops-f03-metrics`
> **base**: `main @ d8e916e` (per PR #27 squash F-01 + 22 commit fast-forward 后)
> **触发**: 2026-09-08 15:11 JST PR #27 squash merge 后, 累计 3/4 子项 75% 收官, 拍 F-03 继续
> **关联**: [WBS §14.10.2 4 子项端到端](../../reports/STAR-P3-WBS-001.md) · [OPS-DETAILED §3 Hybrid AI + metrics](../../detailed-design/OPS-DETAILED-DESIGN-001.md) · [OPS-BASIC §3.3 F-03 Metrics + §3.4 F-04 Docs](../../basic-design/OPS-BASIC-DESIGN-001.md) · [SRS §4 F-03 §8.1 11 表 W/T/M](../../requirements/SRS-STAR-OPS-001.md) · [ADR-0048 framework 锁 axum 0.8](../../architecture/2026-08-26-upgrade/adr/0048-star-warehouse-axum-lock.md) · [PR #27 F-01 squash d8e916e](https://github.com/UlyssesLeoLee/Star/pull/27) · [PR #25 F-02 squash 472bab2](https://github.com/UlyssesLeoLee/Star/pull/25) · [PR #23 MVP squash 97810c0d](https://github.com/UlyssesLeoLee/Star/pull/23) · [守門 #9 子代理 RPC 不可靠实证](../../../AGENTS.md) · [brief F-02](../../briefs/ops-f02-log-ai-impl.md) · [brief F-01](../../briefs/ops-f01-cluster-update-impl.md)

---

## 1. 目标 (Objective)

实装 `star-ops` 端 `/api/ops/metrics/summary` 端到端, 让运维数据 (5 KPI 卡片 + 趋势) 真实化, UI `/ops/page.tsx` metrics tab 接真实 API 返 200 + 解析 + 渲染. 走守门 #13 (W/T/M 100% 覆盖) + 守门 #5 v2 (API key 安全) + 守门 #1 v25 (CI cargo test 单 crate).

**范围** (per WBS §14.10.2 F-03 估 400K token):
- 1 后端实装 (metrics.rs 真实, 接 star-telemetry 复用 + 1 endpoint 真实化)
- 1 前端实装 (MetricsTab.tsx 5 KPI 胶囊 + 趋势占位)
- 1 表 DDL 雏形 (ops_metrics_config M 类 SCD2 已有 SRS-001 §4, 本 PR 落 DDL 雏形)
- 1 IT 跨 crate 雏形 (axum oneshot + star-telemetry mock)
- 1 PT 雏形 (criterion bench P95 < 200ms 守门)
- 1 i18n 3 语言 (zh-CN/en/ja) metrics 文案 (F-02 已落 opsConsole.metricsTitle/MetricsKPI, 复用)
- 6 commit 链 + 1 PR 描述 + 1 PHASE 报告

**累计 12 表 W/T/M** (F-03 1 + F-02 3 + F-01 2 + 既有 6 = 12 表, 守门 #13 100% 覆盖)

## 2. 范围 (Scope)

### 2.1 In-Scope (F-03 端到端)

**后端**:
- `crates/star-ops/src/ops_domain/metrics.rs`: 真实化 `summary()` async, 调 `star-telemetry` 复用 (已存在 `crates/star-telemetry/`)
- `crates/star-ops/src/ops_api.rs`: metrics_summary handler 真实 (调 metrics.rs summary)
- `crates/star-ops/Cargo.toml`: 加 `star-telemetry = { path = "../star-telemetry" }` (复用)
- 1 表 DDL 雏形: `db/migrations/2026-09-08-ops-metrics.sql` (ops_metrics_config M 类 SCD2, 守门 #13 累计 12 表 W/T/M 100%)
- IT 雏形: `crates/star-ops/tests/it_metrics_summary.rs` (axum oneshot + star-telemetry mock 跑通)
- PT 雏形: `crates/star-ops/benches/metrics_bench.rs` (criterion P95 < 200ms)

**前端**:
- `frontend/src/app/ops/components/MetricsTab.tsx` 新建 (5 KPI 胶囊: cpu_avg / mem_avg / active_tasks / mcp_qps / llm_token_daily + 趋势占位)
- `frontend/src/app/ops/page.tsx` 改 metrics TabsContent 用 MetricsTab
- i18n 3 语言 (zh-CN/en/ja) 加 metrics 文案: `metricsCard.cpu`, `metricsCard.mem`, `metricsCard.tasks`, `metricsCard.mcp`, `metricsCard.llm`, `metrics.trend` (F-02 已落 `opsConsole.metricsTitle/MetricsKPI` 复用)
- `frontend/src/lib/ops-api.ts` 扩 metrics 1 endpoint (已有 8 endpoint, 跟 F-01 同 pattern)

**文档**:
- `docs/reports/PHASE-F03-METRICS-REPORT.md` v0.1 (7 段 per AGENTS.md §3)
- `docs/reports/PR-F03-METRICS-001.md` (PR 描述)

### 2.2 Out-of-Scope (不修, 跟 F-01/F-02 实证同)

- 真实 Prometheus / Grafana 集成 (实装阶段 owner 拍板)
- 真实 star-telemetry instrumentation 全 5 KPI 实测数据 (MVP mock 5 KPI 落档)
- log AI 端到端 (F-02 已收官 PR #25)
- cluster update 端到端 (F-01 已收官 PR #27)
- 文档扫描 (F-04 后续)
- OAuth 2.0 / mTLS — 复用 star-context::ActorContext (MVP auth stub)
- 5 域 Lead RACI 分配 — 临时代签, 真人到位后追溯

## 3. 实施步骤 (6 commit 链)

| # | commit | 标题 | 估 token | 关键守門 |
|---|---|---|---|---|
| 1 | `wt1` | feat(ops): Cargo.toml 加 star-telemetry 复用 (F-03 端到端, per SRS-001 §4 F-03) | 50K | #4.2 唯一实施入口 + ADR-0048 + F-01 同 pattern (不加 kube) |
| 2 | `wt2` | feat(ops-domain): metrics.rs 真实化 (调 star-telemetry 复用 5 KPI) | 120K | #1 R-05 (mock 路径) + #5 v2 + #13 累计 |
| 3 | `wt3` | feat(ops-api): metrics_summary handler 真实 + ops-api.ts 扩 1 endpoint + i18n 3 语言 | 80K | #5 v2 + #1 v25 + #6 v2 |
| 4 | `wt4` | feat(db): 1 表 DDL 雏形 (ops_metrics_config M SCD2, per 守門 #13 12 表 W/T/M 100%) | 40K | #13 100% 覆盖 + #13 c SCD2 |
| 5 | `wt5` | test(ops): IT 跨 crate (axum oneshot + star-telemetry mock) + criterion bench P95 < 200ms | 60K | #1 v25 + #1 v3 + #7 v3 |
| 6 | `wt6` | docs(phase): PHASE-F03-METRICS-REPORT v0.1 (7 段) + PR-F03-METRICS-001 描述 | 50K | #12 + #21 v21 |
| **累计** | | | **~400K (严控)** | |

每个 commit 必先 `git log -p --follow <file>` 实证 worktree commit 在 chain 上 (per 守門 #9 主体), author = Ulysses (per 守門 #10). commit message 含 守門编号引用 (跟 F-01/F-02 实证同).

## 4. 守門 (per AGENTS.md §4 累积规 v1-v26 + F-01/F-02 实证同)

每 commit 必跑 6 项 (Windows PowerShell 7, 跟 F-01 实证 5/5 同):

```powershell
# 1. cargo check -p star-ops (单 crate per 守門 #1 v25)
cargo check -p star-ops --all-targets -j 4  # 0 err

# 2. cargo test -p star-ops (单 crate 跳过 workspace per 守門 #1 v25 + v26)
cargo test -p star-ops --lib -j 4  # 全 pass, 含 IT 雏形

# 3. cargo fmt + clippy
cargo fmt -p star-ops --check
cargo clippy -p star-ops --all-targets -j 4  # 0 err (advisory per 守門 #7 v3)

# 4. workspace 必跑 (per 守門 #1 v1 派生, ~1m 19s 实证)
cargo check --workspace --all-targets -j 4  # 0 err

# 5. python mock 跑通 (F-03 不需 subprocess, 跳过)

# 6. frontend typecheck 我改的文件 0 错
cd frontend && npx tsc --noEmit 2>&1 | Select-String -Pattern "src/app/ops|src/lib/ops-api|i18n/dictionary|i18n/zh-CN|i18n/en.ts|i18n/ja.ts"
# 期望: 0 错
```

**20 维守門** (跟 F-01 实证同, 累计 + F-03 派生): #1 4 守門 + #1 v19/v25 + #3 #4 #5 v2 #6 v2 #7 v3 #9 #10 #11 #12 #13 + #14 v2 + #19 v19 #21 v23 #24 v2 #25 v25 #26 v26.

**12 表 W/T/M 累计 100% 覆盖** (F-03 1 + F-02 3 + F-01 2 + 既有 6 = 12 表, per 守門 #13).

## 5. 子代理 brief 规则 (per 守門 #9 v20 + v3 实证, F-01 owner 接手 5/5 实证经验)

### 5.1 派前必做 (owner 必先)

1. 落档本 brief (本文件, 已落档)
2. worktree commit brief + push origin (per 守門 #9 v20, 必先 `git log -p --follow docs/briefs/ops-f03-metrics-impl.md` 实证)
3. 派 worker 子代理 (run_in_background=true), 引用本 brief 路径
4. 子代理 status="succeeded" ≠ 实际成功, **owner 必在子代理返回后跑 owner evidence check** (per 守門 #9 主体实证 + F-01 owner 接手 wt2-wt8 5/5 实证经验, 拒绝子代理"5 min 快速成功" report)

### 5.2 子代理不能擅自做的 (F-01 实证同)

- 推 origin (必 owner 拍板 per 守門 #1 反转 8/30 拍板)
- merge main (必 PR 流程 per 守門 #26 v26)
- 改 守門 20 维任何一条 (必 owner 拍板)
- 跳过 cargo check 守门 (必跑 0 err)
- 跳过 IT/PT 验证 (必跑 跨 crate IT + P95 守门)
- 编造历史 (守門 #1 禁回溯)
- 6 commit 缺 1 → **重做, 不接受 5 commit "差不多"** (F-02 850K 略超 6% 可接受, F-03 严控 400K)
- 真实 Prometheus / Grafana 集成 (仅 mock 路径 per 守門 #1 R-05)
- 引入 kube (F-01 教训: kube 切生产才要, MVP 不需要)

### 5.3 owner evidence check (per 守門 #9 主体)

子代理 status=succeeded 后, owner 必**实际跑** (F-01 实证 5 项):

```powershell
# 1. 6 commit 在 wt-ops-f03-metrics branch 上
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' log --oneline main..wt-ops-f03-metrics | Measure-Object -Line  # 必 7 (brief + 6 F-03)

# 2. 跨 crate 实证
cd D:\Star\.worktrees\wt-ops-f03-metrics
cargo check -p star-ops --all-targets -j 4  # 必 0 err
cargo test -p star-ops --lib -j 4  # 必 33+ pass
cargo test -p star-ops --tests -j 4  # 必 lib + IT pass
cargo bench -p star-ops --bench metrics_bench -- --quick  # P95 < 200ms 实证
cargo fmt -p star-ops --check
cargo clippy -p star-ops --all-targets -j 4
cargo check --workspace --all-targets -j 4  # 0 err (~1m 19s 实证)

# 3. star-telemetry 复用实证 (F-03 关键)
# 跟 F-02 mock subprocess 不同, F-03 直接调 star-telemetry 5 KPI 函数
cargo test -p star-telemetry --lib -j 4  # 必 跑通 (跟 star-ops 共享 telemetry)

# 4. worktree commit 在 origin 远端
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' ls-remote origin wt-ops-f03-metrics
# 期望: 1 row hash (brief 已 push per 守門 #9 v20)

# 5. frontend typecheck (F-03 2 新文件 0 错)
cd frontend && npx tsc --noEmit 2>&1 | Select-String -Pattern "src/app/ops/components/MetricsTab|src/lib/ops-api|i18n/dictionary|i18n/zh-CN|i18n/en.ts|i18n/ja.ts"
# 期望: 0 错
```

## 6. 输出格式 (跟 F-01/F-02 实证同, per pre-pr-review skill)

```
# F-03 metrics 端到端实装报告 (worker 子代理 output)

## 1. 6 commit 链 (per `git log wt-ops-f03-metrics`)
## 2. 守門实证 (cargo check / test / fmt / clippy / workspace / typecheck 输出)
## 3. 20 维守門 0 违反
## 4. 跟既有 star-ops 兼容性 (15 文件清单 + 12 REST 端点 + Hybrid AI 4 級 Ladder)
## 5. 12 表 W/T/M 覆盖 (F-03 1 + F-02 3 + F-01 2 + 既有 6 = 12 表 100%, per 守門 #13)
## 6. 已知缺口 (per 守門 #11 缺标比错标, 至少 5 项)
## 7. owner evidence check 准备
## 8. 待 PR 描述 (`docs/reports/PR-F03-METRICS-001.md` 摘要)
```

## 7. 失败处理 (per 守門 #9 主体实证 + F-01 owner 接手 5/5 经验)

- cargo check 任何 0 err 不通过 → **修, 不跳过**
- cargo test 任何 fail → **修, 不跳过**
- 20 维守門任何违反 → **修, 不跳过** (per 守門 #11 缺标比错标)
- 推 origin → **不推**, 留给 owner 拍板 (per 守門 #1 反转 8/30 拍板)
- merge main → **不 merge**, 留给 owner 拍板 (per 守門 #26 v26)
- 编造历史 (无 git 实证) → **绝对禁止** (per 守門 #1 禁回溯)
- 6 commit 缺 1 → **重做, 不接受 5 commit "差不多"** (F-03 严控 400K)
- token 超 400K 估 → **停下来, 报告 owner, 不擅自扩 scope** (F-01 600K 严控实证, F-02 850K 略超 6% 可接受, F-03 严守 400K)
- 真实 Prometheus / Grafana → **不调**, 仅 mock (per 守門 #1 R-05)
- 引入 kube → **不引** (F-01 教训)

## 8. 时间预算

F-01 实证 owner 接手 5/5 + 6 commit 链 ~25-40 min. 子代理 ~30-50 min. **总估 60-90 min** (F-03 估 400K 比 F-01 600K 小 33%, 时间预算 75% 缩).

子代理 status=succeeded ≠ 实际成功, owner 必 evidence check (per 守門 #9 主体 10 background task ERR_CONNECTION_CLOSED 教训 + F-01 owner 接手 5/5 实证).

## 9. 签字栏

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 JST |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |

## 10. 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 brief 落档 (10 节 + 6 commit 链 + 守门规则 + 子代理规则 + 失败处理) | 2026-09-08 15:14 JST 用户发令"继续, 完成所有任务后merge到main" (F-03 优先于 F-04 拍) |

## 11. 引用文档

- `docs/reports/STAR-P3-WBS-001.md` v0.11 §14.10.2 + §15 累计
- `docs/requirements/SRS-STAR-OPS-001.md` v0.1 §4 F-03 + §10.2 + §8.1 11 表
- `docs/basic-design/OPS-BASIC-DESIGN-001.md` v0.1 §3.3 F-03 Metrics + §3.4 F-04 Docs
- `docs/detailed-design/OPS-DETAILED-DESIGN-001.md` v0.1 §3 Hybrid AI + metrics 集成
- `docs/architecture/2026-08-26-upgrade/adr/0048-star-warehouse-axum-lock.md` v0.1
- `docs/reports/PHASE-F01-CLUSTER-UPDATE-REPORT.md` v0.1 (F-01 7 段模式参考)
- `docs/reports/PHASE-F02-LOG-AI-REPORT.md` v0.1 (F-02 7 段模式参考)
- `docs/briefs/ops-f01-cluster-update-impl.md` v0.1 (F-01 brief 模式参考)
- `docs/briefs/ops-f02-log-ai-impl.md` v0.1 (F-02 brief 模式参考)
- `crates/star-telemetry/` (复用, 已有 5 KPI 函数)
- `crates/star-ops/src/ops_domain/metrics.rs` (F-03 改, 现有 5 KPI stub)
- `crates/star-ops/src/ops_api.rs` 8 REST stub (F-03 改 metrics_summary 真实)
- `crates/star-ops/Cargo.toml` (F-03 加 star-telemetry dep)
- `frontend/src/app/ops/page.tsx` 4 tab 骨架 (F-03 改 metrics TabsContent)
- `frontend/src/lib/i18n/{dictionary,zh-CN,en,ja}.ts`
- `db/migrations/2026-09-08-ops-cluster.sql` (F-01 DDL 模式参考)
- `AGENTS.md` §4 + §4.1 累积规 v1-v26 + §6 ADR 索引
