# Brief: EX-07 — 4 层统一可观测性 (Python)

> **状态**: 🟡 Brief v0.1
> **拍板**: per `ask_2e8740e6779ac6a8d854c590` 4 推荐项
> **wt-branch**: `wt-ex-07-observability`
> **base**: `main` (阶段 3 merge 后)
> **关联**: [03-detailed-design.md §1.1 + §2.4](../architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md) · [PHASE §1.1 EX-07](../../reports/PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md) · [守门 #22 控制台不污染](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) · [守门 #23 AI mock](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md)

---

## 1. 目标

在 `scripts/automation/exclusion/` 落档 3 源文件 + 扩展 `scripts/automation/console_server.py` 加 8 端点, 实现 4 层统一可观测性 (5 Prometheus 锁 metric + 锁泄漏告警 24h 1h 阈值 + 24h idempotency_keys 归档 cron).

## 2. 范围

### 2.1 In-Scope

- `scripts/automation/exclusion/lock_metric_exporter.py` — `LockMetricExporter` 类 (5 Prometheus 指标 per `02 §7.4`)
  - `star_lock_acquired_total` (counter, 按 tier/result)
  - `star_lock_wait_seconds` (histogram)
  - `star_lock_hold_seconds` (histogram)
  - `star_lock_timeout_total` (counter, 按 tier)
  - `star_idempotency_dedup_total` (counter, 按 key_type)
- `scripts/automation/exclusion/lock_leak_alerter.py` — `LockLeakAlerter` 类 (24h 1h 阈值 + Slack 告警)
- `scripts/automation/exclusion/archive_cron.py` — `ArchiveCronJob` 类 (24h 把 `idempotency_keys` 移到 `idempotency_keys_archive`)
- `scripts/automation/console_server.py` 扩展 — 加 8 端点 `/api/exclusion/*`:
  - `GET /api/exclusion/metrics` — 返回 5 锁 metric
  - `GET /api/exclusion/locks` — 当前 4 层持锁列表
  - `GET /api/exclusion/locks/{trace_id}` — 单 trace 4 层锁链
  - `GET /api/exclusion/audit` — 锁审计查询
  - `POST /api/exclusion/force-release` — admin 强制释放
  - `GET /api/exclusion/leak-alerts` — 锁泄漏告警列表
  - `POST /api/exclusion/archive-now` — 手动触发归档
  - `GET /api/exclusion/health` — 健康检查
- 1 集成测试 (per `02 §7.2 IT-07` 锁 metric 暴露 + `IT-08` 锁泄漏告警)
- 2 UT (per `02 §7.1` EX-07 部分)

### 2.2 Out-of-Scope

- ❌ 5 表 DDL (EX-01 范围)
- ❌ 任何 backend Rust / TypeScript 代码
- ❌ Slack 真实告警 (mock 实现, 真实 Slack webhook 后续 SRE Lead 配置)
- ❌ Grafana dashboard (后续 SRE Lead 评审 per G-EI-06)

## 3. 已知缺口

- Slack 真实 webhook 未配置, mock 实现 (per 守门 #23 AI mock + G-EI-06 SRE Lead 拍板)
- 锁泄漏告警阈值 24h 1h (per G-EI-05) 需 SRE Lead 拍板, 暂用 1h 保守值
- Grafana dashboard 未实装 (per G-EI-06)
- 真实 PG 实例缺失, IT 用 mock asyncpg pool

## 4. 守门

1. 守门 #1 Python 4 步
2. 守门 #5 env 安全: 不打印 Slack webhook URL
3. 守门 #6 PowerShell
4. 守门 #9 v3: 派 worker 子代理 (8 端点扩展量大)
5. 守门 #10 author=Ulysses
6. 守门 #11 缺标比错标
7. 守门 #19 v19: 走 `scripts/automation/exclusion/`
8. **守门 #22 控制台不污染**: console_server.py 扩展后必跑 `cargo check --workspace --lib` 0 err 实证 (Python 不进 main 编译链, 但实证 0 错)
9. **守门 #23 AI mock**: Slack 告警走 mock webhook, 不开真实 Slack API

## 5. 依赖

### 5.1 上游

- EX-01 (5 表存在)
- EX-02 (star-mutex LockAuditLogger 已落)
- EX-03 (DispatchLockManager 已落)
- EX-04 (SubAgentLock + LockWatcher 已落)
- EX-05 (UI 显示锁状态已落)
- EX-06 (star-mcp 16 tool 幂等已落)

### 5.2 下游

- EX-08 (集成测试 + 压测) 验证 5 metric 暴露

## 6. 交付物

| # | 路径 | 描述 |
|---|---|---|
| 1 | `scripts/automation/exclusion/lock_metric_exporter.py` | LockMetricExporter 5 metric (~5KB) |
| 2 | `scripts/automation/exclusion/lock_leak_alerter.py` | LockLeakAlerter 24h 1h (~3KB) |
| 3 | `scripts/automation/exclusion/archive_cron.py` | ArchiveCronJob 24h (~2KB) |
| 4 | `scripts/automation/console_server.py` 扩展 | 加 8 端点 (在现有 8080 FastAPI 扩展) (~5KB patch) |
| 5 | `scripts/automation/exclusion/tests/test_lock_metric_exporter.py` | UT (~2KB) |
| 6 | `scripts/automation/exclusion/tests/test_lock_leak_alerter.py` | UT (~2KB) |
| 7 | `scripts/automation/exclusion/tests/it_lock_metrics.py` | IT-07 (~2KB) |
| 8 | `scripts/automation/exclusion/tests/it_lock_leak_alert.py` | IT-08 (~2KB) |
| 9 | `docs/briefs/ex-07-observability.md` | 本 brief |

**总 9 文件, ~23KB raw**

## 7. 验收

- [ ] `python -m py_compile scripts/automation/exclusion/{lock_metric_exporter,lock_leak_alerter,archive_cron}.py` 0 err
- [ ] `python -m pytest scripts/automation/exclusion/tests/` 100% pass (4/4 UT + 2/2 IT)
- [ ] `cargo check --workspace --lib` 0 err (守门 #22 实证, console_server 是 Python 跟 main 编译隔离, 验证 main 编译未污染)
- [ ] `python -c "import requests; r = requests.get('http://localhost:8080/api/exclusion/metrics'); assert r.status_code == 200"` 8 端点全部 200 (需要 console_server.py 启动)
- [ ] `git log -p --follow scripts/automation/exclusion/lock_metric_exporter.py` 实证类完整
- [ ] commit author = Ulysses per 守门 #10

## 8. 实施路径

### 8.1 派 worker 子代理 (守门 #9 v3 推荐)

1. 创建 wt (post 阶段 3 merge): `git worktree add ../.worktrees/wt-ex-07-observability -b wt-ex-07-observability main`
2. 派 worker 子代理: prompt 含本 brief + 守门 + 路径
3. worker 在 wt 内写 9 文件 + 实证守门 #22 (cargo check 0 err 实证 console 不污染 main)
4. Mavis 端实证: `git log -p --follow` 实证各文件完整
5. 切回 main + `git merge --no-ff wt-ex-07-observability`
6. 阶段 4 跨 2 wt cargo check + test 实证 (per `ask_2e8740e6779ac6a8d854c590` 拍板 D 选项 1: 阶段 merge 前 cargo check + 阶段后 cargo test)

### 8.2 worker 失败接手

- worker status="succeeded" 但 git log 没看到 commit → Mavis 直接接手
- worker RPC 失败 → Mavis 直接接手

## 9. 风险

- 守门 #22 console_server.py 扩展后 cargo check 必须 0 err → 实装前 Mavis 端用 mock 验证 Python 不进 main
- 8 端点 E2E 测需 console_server.py 启动 → 测用 subprocess + port 8080 验证
- 5 metric Prometheus 暴露格式 → 走 `prometheus_client` Python 库标准格式

## 10-11. 签字 / 修订历史

5 角色 Mavis 临时代签 + v0.1 初稿.
