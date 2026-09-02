# C08 Throughput Report 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira Throughput Report](https://support.atlassian.com/jira-software-cloud/docs/view-the-throughput-report/) | **需求**: [§3.8](../../requirements/charts-and-reports.md#38-c08--throughput-report) | **Spec**: [P1 #9](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c08_throughput.rs` + `frontend/src/components/charts/Chart08Throughput.tsx`
> **工期**: 1d

---

## 1. 业务定义

**单位时间内完成的 issue 数 (折线/柱)** + 移动平均叠加。

---

## 2. 数据源

| 实体 | 字段 |
|---|---|
| `WorkItem` | `resolved_at, tenant_id, project_id` |

**SQL** (按周/月分桶):
```sql
SELECT
    date_trunc($1, resolved_at) AS bucket,  -- 'week' or 'month'
    COUNT(*) AS count
FROM work_item
WHERE tenant_id = $2 AND project_id = $3
  AND resolved_at BETWEEN $start AND $end
GROUP BY bucket
ORDER BY bucket;
```

---

## 3. 数据 Schema (TS)

```typescript
export interface ThroughputData {
  granularity: 'day' | 'week' | 'month';
  series: Array<{
    bucket: string;     // "2026-W36" (week) or "2026-09" (month)
    count: number;
  }>;
  moving_avg: Array<{
    bucket: string;
    avg: number;        // 3 期移动平均
  }>;
  stats: {
    total: number;
    avg: number;
    std_dev: number;
  };
}
```

---

## 4. 渲染逻辑

```tsx
<ComposedChart data={mergedSeries}>
  <CartesianGrid strokeDasharray="3 3" />
  <XAxis dataKey="bucket" />
  <YAxis label={{ value: 'issues', angle: -90 }} />
  <Tooltip />
  <Legend />
  <Bar dataKey="count" name={t('chart.c08.series.count')} fill="#3b82f6" fillOpacity={0.6} />
  <Line type="monotone" dataKey="avg" name={t('chart.c08.series.moving_avg')} stroke="#10b981" strokeWidth={2} dot={false} />
</ComposedChart>
```

## 5. 配置项

| 字段 | 默认 | 含义 |
|---|---|---|
| `granularity` | `'week'` | `'day'` / `'week'` / `'month'` |
| `moving_avg_window` | `3` | 移动平均窗口 (期数) |
| `time_range` | `{LastNDays: 90}` | 时间窗 |

## 6. 边界

| 边界 | 处理 |
|---|---|
| 0 数据 | "无数据" |
| 单期数据 | 不画移动平均 |
| 移动平均窗口 > 数据期 | 自动缩小窗口 |

## 7. 性能

- 90 天 / 7 = 13 周, 移动平均 13 点
- 性能预算: < 200ms

## 8. 测试

```rust
#[test]
fn test_throughput_weekly_buckets() {}
#[test]
fn test_moving_avg_calculation() {}
```

## 9. i18n

```json
{
  "chart.c08.title": "吞吐量报告",
  "chart.c08.series.count": "完成数",
  "chart.c08.series.moving_avg": "移动平均",
  "chart.c08.granularity.day": "天",
  "chart.c08.granularity.week": "周",
  "chart.c08.granularity.month": "月"
}
```

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 | 2026-09-02 10:04 JST |
