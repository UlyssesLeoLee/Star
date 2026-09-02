# Brief: P3-CHARTS-P1

**Agent**: Mavis 自写 (无子代理 dispatch, per 守门 #9 + 守门 #20)
**Phase**: P3-CHARTS-P1 (子 brief, 父 = P3-CHARTS-P0 v0.1 + P3-CHARTS-P0-BULK v0.1)
**Created**: 2026-09-02 15:28 JST
**Status**: 🟡 running (per docs/automation-design.md §2.3 [P] 判定)

---

## 0. 触发

per 2026-09-02 15:28 JST Ulysses 拍板 next-batch = P1 7 图表

依赖:
- 父 brief 1: docs/briefs/P3-CHARTS-P0.md v0.1
- 父 brief 2: docs/briefs/P3-CHARTS-P0-BULK.md v0.1
- 父 commit 1: d6d8631 ([CHARTS-P0] C01 阶段 1)
- 父 commit 2: 624e972 ([CHARTS-P0-BULK] C02-C13 阶段 2)
- 22 详细设计: docs/design/charts/c08-c12, c14-c15.md

base: 624e972 (P0 阶段 2 收官)

---

## 1. P1 7 图表实施清单

| 顺序 | Chart ID | 名称 | 详细设计 | 工期估计 | 备注 |
|---|---|---|---|---|---|
| 1 | C08 | Throughput | docs/design/charts/c08-throughput.md | 1d | 柱 + 移动平均 |
| 2 | C09 | Forecast | docs/design/charts/c09-forecast.md | 2d | 3 预测方法 + 置信带 |
| 3 | C10 | Time Tracking | docs/design/charts/c10-time-tracking.md | 2d | 估时 vs 已记录 vs 剩余 |
| 4 | C11 | Resolution Time | docs/design/charts/c11-resolution-time.md | 1.5d | avg/median by priority/type |
| 5 | C12 | SLA Compliance | docs/design/charts/c12-sla-compliance.md | 2d | 命中率 by priority |
| 6 | C14 | Issue Type Dist | docs/design/charts/c14-issue-type-dist.md | 0.5d | Pie |
| 7 | C15 | Priority Dist | docs/design/charts/c15-priority-dist.md | 0.5d | Pie |

**P1 批总工期**: ~9.5d ≈ 1.2 SRE·周 (per STAR-OLU-001 1.2M tokens/SRE·周)

---

## 2. 实施策略 (1 commit 模式, per P0 阶段 2 经验)

按 P0 阶段 2 拍板 A, 1 commit 包含:
1. crates/domain-report/src/domain/c0X.rs × 7 (后端)
2. crates/domain-report/src/domain/mod.rs (加 7 pub mod, 累计 15)
3. crates/domain-report/src/lib.rs (加 7 match 分支, 累计 15)
4. frontend/src/components/charts/Chart0X.tsx × 7 (前端)
5. frontend/src/lib/chart-data-schema.ts (+7 interface)
6. frontend/src/i18n/charts/zh-CN.json (+7 chart.c0X.* keys)
7. docs/briefs/P3-CHARTS-P1.md (本 brief)
8. scripts/automation/charts_p1.py (bulk 脚手架)

**P1 估 +2000 行, 1 commit**

---

## 3. 守门 (per 父 brief §2)

### 3.1 必跑

- `cargo check --workspace --lib -p domain-report` 0 err
- `cargo clippy --workspace --lib -p domain-report` 0 err
- `cargo test -p domain-report` 全过 (累计 ≥ 48 + 新图 7 × 5 = **83 测试**)
- 1 commit, message 含 `scripts/automation/charts_p1.py` 路径

### 3.2 守门 #1 v19 派生

- ✅ 命中 R/V/S/A 4 维
- ✅ 必先本 brief 落档
- ✅ commit message 含脚本路径
- ⏳ docs 同步 (registry.md / automation-design.md §4.11) 留 v0.5 增量

---

## 4. 已知缺口 (per 父 brief §3, P1 增量)

1. **C09 Forecast** 3 预测方法 (simple_avg / rolling_avg / linear_regression) 实装
2. **C12 SLA** 依赖 SLA 定义表 (sla_definition), 阶段 1 走 mock
3. **C14 / C15 Pie** 复用 Recharts PieChart 模板
4. **C10 Time Tracking** 依赖 WorkLog 表, 阶段 1 走 mock
5. **跨 session 续风险** (per AGENTS.md §4 #17): 估 1 session, 中间不跨

---

## 5. git 实证要求

- commit author: `Ulysses <ulysses@mavis.local>`
- commit message 含本 brief 路径 `docs/briefs/P3-CHARTS-P1.md`
- commit message 含脚本相对路径 `scripts/automation/charts_p1.py`

---

## 6. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-02 15:28 JST | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 7 图表 P1 批, 1 commit | 2026-09-02 15:28 JST Ulysses 拍板 next-batch=P1 |
