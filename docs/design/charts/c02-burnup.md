# C02 Burnup Chart 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira Burnup Chart](https://support.atlassian.com/jira-software-cloud/docs/view-the-burnup-chart/) | **需求**: [§3.2](../../requirements/charts-and-reports.md#32-c02--burnup-chart) | **Spec**: [P0 #2](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c02_burnup.rs` + `frontend/src/components/charts/Chart02Burnup.tsx`
> **工期**: 1d (复用 C01 大量代码)

---

## 1. 业务定义

Sprint 期内,**累积完成 SP 上升趋势 + Sprint 范围调整线**。Burndown 互补图, 更适合反映 scope 变化频繁的 Sprint。

---

## 2. 数据源

| 实体 | 字段 |
|---|---|
| `Sprint` | `sprint_id, start_date, end_date, total_sp, scope_change_log[]` |
| `WorkItem` | `sprint_id, story_points, completed_at` |

**SQL**:
```sql
-- 累积完成 SP (cumulative)
SELECT
    day,
    SUM(sp) OVER (ORDER BY day) AS cumulative_sp
FROM (
    SELECT date_trunc('day', completed_at) AS day, SUM(story_points) AS sp
    FROM work_item
    WHERE sprint_id = $1 AND completed_at IS NOT NULL
    GROUP BY day
) t;
```

---

## 3. 数据 Schema (TS)

```typescript
export interface BurnupData {
  sprint: SprintMeta;
  series: {
    actual: TimeSeriesPoint[];      // 累积完成
    scope: TimeSeriesPoint[];       // 范围阶梯
  };
  scope_changes: ScopeChange[];
  summary: {
    completed_sp: number;
    total_sp: number;
    completion_ratio: number;       // 0-1
  };
}
```

---

## 4. 渲染逻辑

### 4.1 Recharts 组件

| 元素 | 组件 | 备注 |
|---|---|---|
| Actual (累积完成) | `<Line type="monotone">` | 主线, 实线 |
| Scope (范围) | `<Line type="stepAfter">` | 阶梯式, 范围变更时跳跃 |
| Scope change 标记 | `<ReferenceLine>` (垂直) | 黄色虚线 |

### 4.2 关键差异 vs C01

- Scope 线用 `type="stepAfter"` 而非单调, 反映范围调整的瞬时跳跃
- Actual 线是 `cumulative` 而非 `remaining` (与 C01 互补)
- 范围变更时 Scope 线垂直跳变, Actual 线不变 (反映真实完成量)

```tsx
<Line type="monotone" dataKey="actual" stroke="#3b82f6" strokeWidth={2} dot={{ r: 4 }} />
<Line type="stepAfter" dataKey="scope" stroke="#94a3b8" strokeDasharray="5 5" dot={false} />
```

### 4.3 颜色 / 主题

| 元素 | 浅色 | 深色 |
|---|---|---|
| Actual | `#3b82f6` | `#60a5fa` |
| Scope | `#94a3b8` | `#64748b` |

### 4.4 交互

同 C01 (悬停 tooltip / 切换显示 / 缩放 / 导出)

---

## 5. 配置项

| 字段 | 默认 | 含义 |
|---|---|---|
| `y_axis_unit` | `'sp'` | `'sp'` / `'issue_count'` |
| `show_scope_line` | `true` | 是否显示范围线 |
| `show_scope_changes` | `true` | 是否标记变化 |
| `color_scheme` | `'default'` | 色盲切换 |

---

## 6. 边界与异常

| 边界 | 处理 |
|---|---|
| Sprint 未开始 | actual 全 null, scope 持平 |
| 范围 0 → 50 (新增) | scope 阶梯跳到 50 |
| 范围 50 → 30 (移除) | scope 阶梯降到 30 |
| 实际完成 < 范围 (未完成) | actual 始终 ≤ scope |
| 实际完成 > 范围 (超额) | 显示警告 ⚠, 实际值超 scope |
| 范围变更 > 10 次 | 仅显示最近 10 次 |

---

## 7. 性能

- 数据点 < 200 (Sprint 14 天 × 14 issue)
- 复用 C01 缓存, 缓存 key: `report:{tenant}:{report_id}:c02_burnup`
- 性能预算同 C01

---

## 8. 测试用例

```rust
#[tokio::test]
async fn test_burnup_cumulative_sum() {
    // 完成 3 issue, sp 20+30+10=60, 累积第 3 天 = 60
    let data = generate_burnup(&sprint, &issues).await.unwrap();
    assert_eq!(data.series.actual[2].y, 60.0);
}

#[tokio::test]
async fn test_burnup_scope_step() {
    // 范围 100 → 80, scope 线应在变更日 step
}

#[tokio::test]
async fn test_burnup_zero_total() {}

#[tokio::test]
async fn test_burnup_overshoot() {
    // 实际 110 > 范围 100, 应警告
}
```

---

## 9. i18n

```json
{
  "chart.c02.title": "燃起图",
  "chart.c02.x_axis": "日期",
  "chart.c02.y_axis.sp": "累积完成 SP",
  "chart.c02.series.actual": "实际完成",
  "chart.c02.series.scope": "Sprint 范围",
  "chart.c02.tooltip.scope_change": "范围调整: ±{n} SP"
}
```

---

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 | 2026-09-02 10:04 JST Ulysses 拍板 |
