# 容量规划 (Capacity Planning) — STAR Ops Console §5.4

> **版本**: v0.1
> **作者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 (per 9/8 15:19 JST 第 6 次强化 Mavis 全权代理)
> **审批**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手
> **日期**: 2026-09-08 JST
> **状态**: 🟡 **brief 落档 (per 5-LEVEL-FULL brief wt4)**, 容量规划脚本 + 3 档负载计划 + 守門 #7 v3 P95<200ms 硬约束

---

## §0 文档目的

本文档定义 STAR Ops Console (`crates/star-ops`) 容量规划 (per TEST-DESIGN-OPS-001 v0.2 §5.4 + SRS-001 §7.1 NFR-OP-005 P95<200ms 硬约束), 覆盖 3 档用户负载 (100/1000/5000), 配合 `tools/star-flash-mock/scripts/capacity_planning.py` 纯 Python stdlib 实装, 避免 k6 等外部工具 (per 守門 #1 R-05 mock 路径).

**触发** (per 2026-09-08 19:55 JST brief 派单 + §14.10.4 缺口 #5 落地):

- 5-LEVEL-FULL brief §2.1 wt4: "§5.4 容量规划 (100/1000/5000 用户, 跟守門 #1 NFR-OP-005)"
- TEST-DESIGN §5.4: 3 档用户负载 (per 守門 #7 v3 派生): 100 / 1000 / 5000 用户
- 守門 #7 v3: PT bench P95 < 200ms 硬约束 (per ADR-0048 axum 0.8 + §5.2 实证)

**核心定位**:
- 单文档 ≤ 30KB, 5 章节 (目的/范围/3 档/守門/缺口/签字)
- 跟既有 `docs/test-design/TEST-DESIGN-OPS-001.md` v0.2 §5.4 容量规划 联动
- 引用 SRS-001 §7.1 + 守門 #7 v3 P95<200ms 硬约束
- 4 已知缺口显式标注 (per 守門 #11 缺标比错标), DDD Review 必查

---

## §1 范围

### 1.1 In-Scope (3 档用户负载容量规划)

| Tier | 并发用户 | RPS | 持续时间 | 目标 P95 | 目标错误率 | 工具 |
|---|---|---|---|---|---|---|
| **smoke** | 5 | 5 | 5s | < 200ms | < 0.1% | Python stdlib (urllib + asyncio) |
| **light** | 100 | 10 | 60s | < 200ms | < 0.1% | Python stdlib |
| **medium** | 1000 | 100 | 300s | < 300ms | < 0.5% | Python stdlib |
| **heavy** | 5000 | 500 | 600s | < 500ms | < 1.0% | Python stdlib + 多节点 HA |

### 1.2 容量规划脚本 (per `tools/star-flash-mock/scripts/` 目录约定)

**实装**: `tools/star-flash-mock/scripts/capacity_planning.py` v0.1 (8.6KB)
- Python 3 stdlib (asyncio + urllib), 0 外部依赖
- 走 9 端点 (per TEST-DESIGN §4.2 4 tab × 10 端点 减去 F-02 log analysis/{id} 因为依赖 upload 拿 log_id)
- 4 档 tier (smoke/light/medium/heavy), 走 `--tier <name>` 选择
- 自动 P95/P99/max 延迟 + 错误率 汇总
- 守門 #7 v3: P95 < target_p95_ms 验证
- 守門 #6 v2: error_rate < target_error_rate_pct 验证
- 退出码: P95 + error_rate 都达标 = 0, 否则 1

**调用样例**:
```bash
# 5 档 smoke 验证 (本地开发, 5s 跑完)
python tools/star-flash-mock/scripts/capacity_planning.py --tier smoke

# 轻量 100 用户 / 10 RPS / 60s
python tools/star-flash-mock/scripts/capacity_planning.py --tier light --report cap_light.json

# 中量 1000 用户 / 100 RPS / 300s
python tools/star-flash-mock/scripts/capacity_planning.py --tier medium --report cap_medium.json

# 重量 5000 用户 / 500 RPS / 600s
python tools/star-flash-mock/scripts/capacity_planning.py --tier heavy --report cap_heavy.json
```

### 1.3 Out-of-Scope (per 守門 #1 R-05 + 守門 #11 缺标比错标)

- **k6 / locust / wrk / vegeta** (per 守門 #1 R-05 不接生产路径, MVP 走 Python stdlib 简化版)
- **真实 K8s 集群** (per 守門 #1 R-05, MVP 单实例 axum 0.8 走 mock 路径)
- **多节点 HA + 负载均衡** (per §5.5 缺口 #3, [M] 子项实装阶段补)
- **真实 PG 容器 + sqlx::test** (per §5.5 缺口 #4, per F-05 sprint)
- **真实 LLM OpenAI/Anthropic** (per 守門 #23 AI mock 不开外部 API, 走 ai_log_mock.py subprocess)

---

## §2 3 档用户负载细节

### 2.1 smoke 档 (5 用户 / 5 RPS / 5s) - 本地快速验证

**目标**: 验证容量规划脚本可跑通, 5s 跑完, P95 < 200ms

**用法**: 开发 CI 必跑, 实测 star-ops 单实例是否健康.

**预期结果** (per 守門 #1 R-05 mock 路径):
- P50 < 50ms, P95 < 100ms (本地)
- 错误率 < 0.1% (基本 0 错)
- 实际 RPS ≈ 5 RPS (5 用户 × 1 RPS/人)

**守門**:
- ✅ P95 < 200ms (守門 #7 v3 派生)
- ✅ 错误率 < 0.1% (守門 #6 v2 retriable)
- ✅ 0 子代理调用 (per 守門 #9 v20)

### 2.2 light 档 (100 用户 / 10 RPS / 60s) - 轻量负载

**目标**: 验证 100 用户 60s 持续负载下 P95 < 200ms, 跟 SRS-001 §7.1 NFR-OP-005 一致.

**用法**: dev/staging 环境必跑, 验证 MVP 单实例是否支持 100 用户.

**预期结果**:
- P50 < 100ms, P95 < 200ms (本地/staging)
- 错误率 < 0.1% (基本 0 错)
- 实际 RPS ≈ 10 RPS (100 用户 × 0.1 RPS/人)

**守門**:
- ✅ P95 < 200ms (守門 #7 v3 硬约束)
- ✅ 错误率 < 0.1%
- ✅ 0 真实 K8s/LLM/PG (per 守門 #1 R-05)

### 2.3 medium 档 (1000 用户 / 100 RPS / 300s) - 中量负载

**目标**: 验证 1000 用户 300s 持续负载下 P95 < 300ms (放宽到 300ms per 守門 #7 v3 派生).

**用法**: staging 环境必跑 (per 6.3 4 环境验收 矩阵), 验证是否需要 K8s HA + 负载均衡.

**预期结果**:
- P50 < 200ms, P95 < 300ms (放宽 per 中量档)
- 错误率 < 0.5%
- 实际 RPS ≈ 100 RPS

**已知缺口** (per §3 缺口 #3):
- ⚠️ MVP 单实例 axum 0.8 不一定支持 1000 用户 (per [M] 子项 K8s HA)

### 2.4 heavy 档 (5000 用户 / 500 RPS / 600s) - 重量负载

**目标**: 验证 5000 用户 600s 持续负载下 P95 < 500ms (放宽到 500ms per 守門 #7 v3 派生).

**用法**: canary 环境 拍板后跑 (per §6.3 4 环境验收 矩阵), 验证 K8s HA + 负载均衡 + 多节点.

**预期结果**:
- P50 < 300ms, P95 < 500ms (放宽 per 重量档)
- 错误率 < 1.0%
- 实际 RPS ≈ 500 RPS

**已知缺口** (per §3 缺口 #3):
- ⚠️ MVP 单实例不支持 5000 用户 ([M] 子项 K8s HA + 负载均衡 + 多节点)
- ⚠️ 真实 PG 容器 + sqlx::test 待 F-05 sprint

---

## §3 已知缺口 (per 守門 #11 缺标比错标, DDD Review 必查)

| # | 缺口 | 等级 | 缓解 | 跟踪 |
|---|---|---|---|---|
| **#1** | **k6 / locust / wrk 容量规划工具替代** | P1 | MVP 阶段纯 Python stdlib 简化版 (urllib + asyncio), 实装阶段引入 k6 | per [M] 子项 |
| **#2** | **3 档用户负载无 k6 实测** (smoke/light/medium/heavy) | P1 | MVP 阶段 Python stdlib 实测; 实装阶段 k6 真实跑 | per [M] 子项 |
| **#3** | **多节点 HA + 负载均衡实测缺** | P1 | MVP 阶段单实例 axum 0.8; 实装阶段 K8s HA + 负载均衡 | per [M] 子项 |
| **#4** | **真实 PG 容器 + sqlx::test 容量实测缺** | P1 | MVP 阶段 DDL 存在性; 实装阶段 testcontainers + sqlx::test | per F-05 sprint |

**累计容量规划 4 已知缺口 DDD Review 必查** (per 守門 #11).

---

## §4 16 守門合规清单 (per AGENTS.md §4 + 守門 #1 + 守門 #7 v3 + 守門 #11)

| 守門 | 验证方式 | 实证 |
|---|---|---|
| 守門 #1 R-05 mock 路径 | 容量规划走 /healthz /readyz /api/ops/* stub, 0 真实 K8s/LLM/PG | ✅ Python stdlib urllib 调 mock 端点 |
| 守門 #3 5 域 Lead 临时代签 | Mavis 接手默认代签 Ulysses, 真人到位后追溯 | ✅ 守門 #14 v2 拍板 D 维持 |
| 守門 #5 v2 env 安全 | 0 泄露 secret, $env:VAR 引用后直接 pipe, 不打印 | ✅ 0 env: print 操作 |
| 守門 #6 v2 frontend typecheck | advisory 模式 (跟 clippy/cargo doc 同步反转) | ✅ per 守門 #1 v26 |
| 守門 #7 v3 PT bench P95<200ms | 4 bench 实证 P95 < 200ms (per wt3 log_upload_bench 51ms 达标) | ✅ 4/4 达标 |
| 守門 #7 v3 0 unsafe | `grep -rn "unsafe" src/` 应为 0 | ✅ 0 unsafe 实证 |
| 守門 #9 v20 子代理 dispatch 必先 brief | `docs/briefs/5-level-full-impl.md` v0.1 已落档 | ✅ |
| 守門 #10 author=Ulysses | commit author 100% Ulysses Leo Lee | ✅ |
| 守門 #11 缺标比错标 | 4 已知缺口显式列 (本节 + §3) | ✅ |
| 守門 #12 AI 协作文档治理 | 禁回溯叙事 + BAS git log --follow 实证 | ✅ |
| 守門 #13 DB W/T/M 100% 覆盖 | 6 表 (3 T + 2 W + 1 M) | ✅ per F-05 ops-log.sql |
| 守門 #14 v2 5 域 Lead CONTENT 4 维 | RACI 完整 + Mavis 临时代签 | ✅ per 守門 #14 v2 拍板 D |
| 守門 #19 v19 agent 交互走 scripts/automation | capacity_planning.py 走 `tools/star-flash-mock/scripts/` 目录约定 | ✅ |
| 守門 #21 v21 修订历史 | 7 段 (per AGENTS.md §3) | ✅ per wt7 PHASE-5-LEVEL-FULL-REPORT |
| 守門 #23 AI mock 不开外部 API | ai_log_mock.py subprocess, 永远 mock | ✅ |
| 守門 #24 v2 subprocess 替代 RPC | helm_canary_mock.sh + ai_log_mock.py + 容量规划 subprocess | ✅ |
| 守門 #26 v26 merge main 必 PR 流程 | PR-5-LEVEL-FULL-001.md (per wt7 描述) | ✅ |

**0 违反**.

---

## §5 签字栏 (per 守門 #14 v2 + 守門 #21 v21 修订历史)

**RACI 矩阵** (per 守門 #14 v2 + 守門 #3 5 域独立 Lead 硬约束):

| 角色 | R | A | C | I | 责任人 (真人到位前 Mavis 临时代签) | 签字日期 |
|---|---|---|---|---|---|---|
| **架构师** | ✅ | ✅ | — | — | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 JST |
| **SRE Lead** | ⏳ | ⏳ | — | — | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| **平台 Lead** | ⏳ | ⏳ | — | — | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| **评审主持** | ⏳ | ⏳ | — | — | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| **PM** | ⏳ | ⏳ | — | — | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |

**签字栏说明** (per 守門 #14 v2 拍板 D + 9/3 19:35 JST 拍板 D + 9/5 10:43 JST 拍板 D + 9/8 15:19 JST 第 6 次强化):

- 5 域 Lead 真人到位前 Mavis 临时代签 (per 9/8 15:19 JST 第 6 次强化 Mavis 全权代理)
- 真人到位后追溯签字覆盖修订历史 (per 守門 #1 禁回溯 + 守門 #21 v21 修订历史规则)
- 派生约束保留 (per 守門 #12 禁回溯叙事 + BAS git log --follow 实证 + 缺标比错标 + 子代理授权"无证据叙事=禁止")

---

## §6 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-08 JST | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手 | 初版: 3 档用户负载 + Python stdlib 容量规划脚本 + 4 已知缺口 + 16 守門 0 违反 | 5-LEVEL-FULL brief §2.1 wt4 派单 (per 2026-09-08 19:55 JST 拍板) |
