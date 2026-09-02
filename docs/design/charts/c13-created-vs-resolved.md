# C13 Created vs Resolved Chart 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira Created vs Resolved Chart](https://support.atlassian.com/jira-software-cloud/docs/view-the-created-vs-resolved-chart/) | **需求**: [§3.13](../../requirements/charts-and-reports.md#313-c13--created-vs-resolved-chart) | **Spec**: [P0 #8](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c13_created_vs_resolved.rs` + `frontend/src/components/charts/Chart13CreatedVsResolved.tsx`
> **工期**: 1d

---

## 1. 业务定义

**每天新建 issue 数 vs 解决 issue 数双线对比**, 识别 backlog 增长/收缩趋势。

---

## 2. 数据源

| 实体 | 字段 |
|---|---|
| `WorkItem` | `created_at, resolved_at, project_id, tenant_id` |

**SQL**:
```sql
WITH date_series AS (
    SELECT generate_series($start::date, $end::date, '1 day')::date AS day
),
created_per_day AS (
    SELECT date_trunc('day', created_at)::date AS day, COUNT(*) AS created
    FROM work_item
    WHERE tenant_id = $1 AND project_id = $2 AND created_at BETWEEN $start AND $end
    GROUP BY day
),
resolved_per_day AS (
    SELECT date_trunc('day', resolved_at)::date AS day, COUNT(*) AS resolved
    FROM work_item
    WHERE tenant_id = $1 AND project_id = $2 AND resolved_at BETWEEN $start AND $end
    GROUP BY day
)
SELECT
    ds.day,
    COALESCE(c.created, 0) AS created,
    COALESCE(r.resolved, 0) AS resolved
FROM date_series ds
LEFT JOIN created_per_day c ON ds.day = c.day
LEFT JOIN resolved_per_day r ON ds.day = r.day
ORDER BY ds.day;
```

---

## 3. 数据 Schema (TS)

```typescript
export interface CreatedVsResolvedData {
  series: Array<{
    day: string;
    created: number;
    resolved: number;
  }>;
  summary: {
    total_created: number;
    total_resolved: number;
    net_change: number;        // resolved - created
    avg_daily_created: number;
    avg_daily_resolved: number;
    backlog_trend: 'growing' | 'shrinking' | 'stable';
  };
  time_granularity: 'day' | 'week' | 'month';
}
```

---

## 4. 渲染逻辑

### 4.1 Recharts 组件

```tsx
<LineChart data={data.series}>
  <CartesianGrid strokeDasharray="3 3" />
  <XAxis dataKey="day" />
  <YAxis label={{ value: 'issues', angle: -90 }} />
  <Tooltip content={<CvRTooltip />} />
  <Legend />
  <Line type="monotone" dataKey="created" name={t('chart.c13.series.created')} stroke="#3b82f6" strokeWidth={2} dot={{ r: 3 }} />
  <Line type="monotone" dataKey="resolved" name={t('chart.c13.series.resolved')} stroke="#10b981" strokeWidth={2} dot={{ r: 3 }} />
</LineChart>
```

### 4.2 颜色

| Series | 浅色 | 深色 |
|---|---|---|
| Created | `#3b82f6` (blue) | `#60a5fa` |
| Resolved | `#10b981` (emerald) | `#34d399` |

### 4.3 交互

- 切换粒度 (天/周/月)
- 切换 series 显示
- 缩放 (Brush)
- 导出

---

## 5. 配置项

| 字段 | 默认 | 含义 |
|---|---|---|
| `time_range` | `{mode: 'LastNDays', n_days: 30}` | 时间窗 |
| `granularity` | `'day'` | `'day'` / `'week'` / `'month'` |
| `show_summary` | `true` | 显示摘要卡片 (net change) |
| `color_scheme` | `'default'` | 色盲切换 |

---

## 6. 边界与异常

| 边界 | 处理 |
|---|---|
| 0 数据 | "无数据" 提示 |
| 时间窗 < 7 天 | 不允许切到 week 粒度 |
| 单日 spike (> 3x avg) | tooltip 标 ⚠ 提示 |
| 所有天数 0 | 退化线, 显示 "无活动" |

---

## 7. 性能

- 30 天 × 2 series = 60 数据点
- 365 天 = 730 点
- 性能预算: < 200ms query, < 400ms render

### 7.1 索引

```sql
CREATE INDEX idx_wi_tenant_created ON work_item(tenant_id, created_at);
CREATE INDEX idx_wi_tenant_resolved ON work_item(tenant_id, resolved_at) WHERE resolved_at IS NOT NULL;
```

---

## 8. 测试用例

```rust
#[test]
fn test_created_vs_resolved_basic() {
    // 3 天各 5 新建 3 解决, net = -2 (growing backlog)
    let data = generate_cvr(&sprint, &issues).await.unwrap();
    assert_eq!(data.summary.backlog_trend, "growing");
}

#[test]
fn test_cvr_granularity_week() {
    // 切换 week 后合并 7 天数据
}
```

---

## 9. i18n

```json
{
  "chart.c13.title": "新建 vs 解决",
  "chart.c13.x_axis": "日期",
  "chart.c13.y_axis": "Issue 数",
  "chart.c13.series.created": "新建",
  "chart.c13.series.resolved": "解决",
  "chart.c13.summary.net_change": "净变化",
  "chart.c13.summary.trend.growing": "Backlog 增长",
  "chart.c13.summary.trend.shrinking": "Backlog 收缩",
  "chart.c13.summary.trend.stable": "Backlog 稳定",
  "chart.c13.granularity.day": "天",
  "chart.c13.granularity.week": "周",
  "chart.c13.granularity.month": "月"
}
```

---

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 | 2026-09-02 10:04 JST |
