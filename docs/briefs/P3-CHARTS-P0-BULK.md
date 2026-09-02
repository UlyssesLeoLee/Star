# Brief: P3-CHARTS-P0-BULK

**Agent**: Mavis 自写 (无子代理 dispatch, per 守门 #9 + 守门 #20 子代理 status 实证不可靠)
**Phase**: P3-CHARTS-P0-BULK (子 brief, 父 = P3-CHARTS-P0 v0.1)
**Created**: 2026-09-02 14:07 JST
**Status**: 🟡 running (per docs/automation-design.md §2.3 [P] 判定)

---

## 0. 触发

per 2026-09-02 14:06 JST Ulysses 拍板 (ask_user 3 维):
- target = A P0 阶段 2 (剩余 7 图表)
- depth = I 完整可跑 (同 P0 阶段 1)
- commit-strategy = i 7 图表 × 1 commit (7 commits)

依赖:
- 父 brief: docs/briefs/P3-CHARTS-P0.md v0.1
- 父 commit: d6d8631 ([CHARTS-P0] 22 文件 v1.0)
- C01 模板: crates/domain-report/src/domain/c01_burndown.rs (v0.1.1 修复后)
- 22 详细设计: docs/design/charts/c0X-*.md (P0 批 8 份已有, 7 份 = C02-C05, C06, C07, C13)

base: d6d8631

---

## 1. 7 图表实施清单 (P0 批剩余)

| 顺序 | Chart ID | 名称 | 详细设计 | 工期估计 | 复用 C01 模板 |
|---|---|---|---|---|---|
| 1 | C02 | Burnup | docs/design/charts/c02-burnup.md | 1d | ~70% 复用 (累积 vs 剩余) |
| 2 | C03 | Velocity | docs/design/charts/c03-velocity.md | 1.5d | ~50% 复用 (Bar + RefLine) |
| 3 | C04 | Sprint Report | docs/design/charts/c04-sprint-report.md | 1d | 表格型, 新模板 |
| 4 | C05 | CFD | docs/design/charts/c05-cfd.md | 2d | ~60% 复用 (AreaChart) |
| 5 | C06 | Control Chart | docs/design/charts/c06-control-chart.md | 3d | ~50% 复用 (Modified Z-Score 算法独立) |
| 6 | C07 | Cycle Time | docs/design/charts/c07-cycle-time.md | 1.5d | ~70% 复用 (Histogram + 百分位) |
| 7 | C13 | Created vs Resolved | docs/design/charts/c13-created-vs-resolved.md | 1d | ~80% 复用 (双线) |

**P0 阶段 2 总工期**: ~11d = ~1.5 SRE·周 (per STAR-OLU-001 1.2M tokens/SRE·周)

---

## 2. 阶段拆分 (7 commits)

| Commit | 内容 | 估 +行数 |
|---|---|---|
| d6d8631 | (父 commit) P0 阶段 1 基础设施 + C01 | +3363 |
| **(本 brief 7 commits)** | | |
| commit #1 | C02 Burnup: 后端 + 前端 + 5 单元 + 1 集成 | ~+650 |
| commit #2 | C03 Velocity: 后端 + 前端 + 5 单元 + 1 集成 | ~+700 |
| commit #3 | C04 Sprint Report: 后端 + 前端 + 5 单元 + 1 集成 | ~+600 |
| commit #4 | C05 CFD: 后端 + 前端 + 5 单元 + 1 集成 | ~+700 |
| commit #5 | C06 Control Chart: 后端 + 前端 + 5 单元 + 1 集成 + Modified Z-Score | ~+800 |
| commit #6 | C07 Cycle Time: 后端 + 前端 + 5 单元 + 1 集成 | ~+700 |
| commit #7 | C13 Created vs Resolved: 后端 + 前端 + 5 单元 + 1 集成 | ~+650 |

**P0 阶段 2 累计估**: +4800 行, 7 commits

---

## 3. 每 commit 交付清单 (7 图表统一模板)

```
crates/domain-report/src/domain/c0X_<name>.rs       # 后端算法 + SQL + Port stub
crates/domain-report/tests/c0X_<name>_test.rs      # 5 单元 + 1 集成
crates/domain-report/src/domain/mod.rs             # 加 pub mod c0X_<name>;
crates/domain-report/src/lib.rs                    # 在 generate_p0 里 match C0X 分支
frontend/src/components/charts/Chart0X<Name>.tsx   # Recharts 组件
frontend/src/lib/chart-data-schema.ts              # 加 Chart0X<Name>Data 接口
frontend/src/i18n/charts/zh-CN.json                # 加 chart.c0X.* keys
```

---

## 4. 守门 (per 父 brief §2)

### 4.1 每次 commit 前必跑

- `cargo check --workspace --lib -p domain-report` 0 err
- `cargo clippy --workspace --lib -p domain-report` 0 err
- `cargo test -p domain-report` 全过 (累计 ≥ 12 + 新图 6 = ≥18 测试)
- 上一 commit 守门 OK

### 4.2 7 commit 全部完成后

- 累计 ≥ 12 (C01 阶段 1) + 7 × 6 = **54 测试全过**
- 22 详细设计 vs 实际实现 一致性 check
- docs 同步: registry.md (新增 charts_p0_bulk.py 行) + automation-design.md §4.11 加 sub-row

### 4.3 守门 #1 v19 派生 (per 父 brief)

- ✅ 命中 R/V/S/A 4 维
- ✅ 必先本 brief 落档 (docs/briefs/P3-CHARTS-P0-BULK.md)
- ✅ commit message 含 scripts/automation/charts_p0_bulk.py 路径
- ✅ docs 同步

---

## 5. 已知缺口 (per 父 brief §3, 阶段 2 增量)

1. **7 图表全 P0 完 = P0 阶段 2 收官**, 剩余 14 图表 (P1 7 + P2 7) 留阶段 3/4
2. **C06 Modified Z-Score** 算法 (per docs/design/charts/c06-control-chart.md §5.1) 实装
3. **C04 Sprint Report** 是 Table 模板 (非严格图表), Recharts 用 Table + 摘要卡片
4. **CFD (C05)** 用 AreaChart + stackId, Recharts 支持
5. **跨 session 续风险** (per AGENTS.md §4 #17 H2 实证): 估 1-2 session, 中间 commit checkpoint

---

## 6. git 实证要求 (per 父 brief §4)

- commit author: `Ulysses <ulysses@mavis.local>`
- 每 commit message 含本 brief 路径 `docs/briefs/P3-CHARTS-P0-BULK.md`
- 每 commit message 含脚本相对路径 `scripts/automation/charts_p0_bulk.py`
- 修订人 / 审批者 按 AGENTS.md §2.2 / §2.3 代签

---

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-02 14:07 JST | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 7 图表 × 1 commit, 每图 ~650 行估 | 2026-09-02 14:06 JST Ulysses 拍板 A+I+i |
