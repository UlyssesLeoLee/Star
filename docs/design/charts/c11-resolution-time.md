# C11 Resolution Time Report 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira Resolution Time Report](https://support.atlassian.com/jira-software-cloud/docs/view-the-resolution-time-report/) | **需求**: [§3.11](../../requirements/charts-and-reports.md#311-c11--resolution-time-report) | **Spec**: [P1 #12](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c11_resolution_time.rs` + `frontend/src/components/charts/Chart11ResolutionTime.tsx`
> **工期**: 1.5d

---

## 1. 业务定义

**解决时间 (resolution time) 按优先级/类型/经办人分组的平均/中位对比**。

---

## 2. 数据源

| 实体 | 字段 |
|---|---|
| `WorkItem` | `priority, issue_type, resolved_at, created_at, assignee_id` |

**SQL**:
```sql
SELECT
    priority,  -- or issue_type
    AVG(EXTRACT(EPOCH FROM (resolved_at - created_at)) / 86400.0) AS avg_days,
    percentile_cont(0.5) WITHIN GROUP (ORDER BY EXTRACT(EPOCH FROM (resolved_at - created_at)) / 86400.0) AS median_days,
    COUNT(*) AS count
FROM work_item
WHERE tenant_id = $1 AND project_id = $2
  AND resolved_at IS NOT NULL
GROUP BY priority
ORDER BY priority;
```

## 3. 数据 Schema (TS)

```typescript
export interface ResolutionTimeData {
  group_by: 'priority' | 'type' | 'assignee';
  rows: Array<{
    group: string;       // "high" / "Task" / "user_123"
    avg_days: number;
    median_days: number;
    count: number;
  }>;
  summary: {
    overall_avg: number;
    overall_median: number;
  };
}
```

## 4. 渲染逻辑

分组 Bar (Avg + Median 并列):

```tsx
<BarChart data={data.rows}>
  <CartesianGrid strokeDasharray="3 3" />
  <XAxis dataKey="group" />
  <YAxis label={{ value: t('chart.c11.y_axis'), angle: -90 }} />
  <Tooltip content={<RTooltip />} />
  <Legend />
  <Bar dataKey="avg_days" name={t('chart.c11.series.avg')} fill="#3b82f6" />
  <Bar dataKey="median_days" name={t('chart.c11.series.median')} fill="#10b981" />
</BarChart>
```

## 5. 配置项

| 字段 | 默认 | 含义 |
|---|---|---|
| `group_by` | `'priority'` | `'priority'` / `'type'` / `'assignee'` |
| `time_range` | `{LastNDays: 90}` | 时间窗 |
| `show_count_badge` | `true` | 显示 count 数字 |

## 6. 边界

| 边界 | 处理 |
|---|---|
| 0 resolved issue | "无数据" |
| 单组 | 单 Bar, 不分组 |
| group 数 > 20 | 仅显示 top 20 + "其他" |

## 7. 性能

- 性能预算: < 300ms (PostgreSQL percentile_cont 加速)

## 8. 测试

```rust
#[test]
fn test_resolution_time_grouping() {}
#[test]
fn test_median_calculation() {}
```

## 9. i18n

```json
{
  "chart.c11.title": "解决时间报告",
  "chart.c11.y_axis": "天数",
  "chart.c11.series.avg": "平均",
  "chart.c11.series.median": "中位",
  "chart.c11.group.priority": "按优先级",
  "chart.c11.group.type": "按类型",
  "chart.c11.group.assignee": "按经办人"
}
```

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 | 2026-09-02 10:04 JST |
