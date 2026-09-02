# C20 Time in Status Report 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira Time Since Status Report](https://support.atlassian.com/jira-software-cloud/docs/view-the-time-since-status-report/) | **需求**: [§3.20](../../requirements/charts-and-reports.md#320-c20--time-in-status) | **Spec**: [P2 #20](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c20_time_in_status.rs` + `frontend/src/components/charts/Chart20TimeInStatus.tsx`
> **工期**: 2d

---

## 1. 业务定义

**每个状态平均停留时间 (横向 Bar)**, 识别流程瓶颈。

---

## 2. 数据源

| 实体 | 字段 |
|---|---|
| `WorkItemStatusHistory` | `workitem_id, status, entered_at, exited_at` |
| `WorkflowStatus` | `status_id, name, category` |

**SQL** (已 exit 状态, per INV-REPORT-20):
```sql
SELECT
    ws.name AS status,
    AVG(EXTRACT(EPOCH FROM (wish.exited_at - wish.entered_at)) / 86400.0) AS avg_days,
    COUNT(*) AS sample_count
FROM work_item_status_history wish
JOIN workflow_status ws ON wish.status_id = ws.status_id
WHERE wish.tenant_id = $1
  AND wish.exited_at IS NOT NULL
  AND wish.entered_at >= $start
GROUP BY ws.name
ORDER BY avg_days DESC;
```

## 3. 数据 Schema (TS)

```typescript
export interface TimeInStatusData {
  rows: Array<{
    status: string;
    category: string;        // 'todo' / 'in_progress' / 'done'
    avg_days: number;
    median_days: number;
    sample_count: number;
  }>;
  exclude_current: true;     // INV-REPORT-20
}
```

## 4. 渲染逻辑

```tsx
<BarChart data={data.rows} layout="vertical">
  <CartesianGrid strokeDasharray="3 3" />
  <XAxis type="number" label={{ value: t('chart.c20.x_axis'), position: 'insideBottom' }} />
  <YAxis type="category" dataKey="status" width={120} />
  <Tooltip content={<TISTooltip />} />
  <Bar dataKey="avg_days" fill="#3b82f6">
    {data.rows.map((r, i) => (
      <Cell key={i} fill={CATEGORY_COLORS[r.category]} />
    ))}
    <LabelList dataKey="avg_days" position="right" formatter={(v) => `${v.toFixed(1)}d`} />
  </Bar>
</BarChart>
```

## 5. 颜色

| Category | 颜色 |
|---|---|
| todo | `#94a3b8` |
| in_progress | `#3b82f6` |
| in_review | `#a855f7` |
| done | `#10b981` |

## 6. 配置项

| 字段 | 默认 | 含义 |
|---|---|---|
| `time_range` | `{LastNDays: 90}` | 时间窗 (基于 entered_at) |
| `show_median` | `false` | 是否叠加中位 |
| `min_sample_count` | `5` | 过滤样本 < 5 的状态 |
| `exclude_current` | `true` | INV-REPORT-20 默认 |

## 7. 边界

| 边界 | 处理 |
|---|---|
| 0 已 exit 状态 | "无数据" |
| 单状态 | 单 Bar |
| exited_at = entered_at (同进同出) | 排除 (无意义) |
| exited_at < entered_at | 数据异常, 排除 + Audit |
| sample < 5 | 灰显, 提示 "样本不足" |

## 8. 性能

### 8.1 索引

```sql
CREATE INDEX idx_wish_exited ON work_item_status_history(tenant_id, exited_at)
    WHERE exited_at IS NOT NULL;
```

- 性能预算: < 300ms

## 9. 测试

```rust
#[test]
fn test_time_in_status_excludes_current() {
    // 当前仍在的状态不应计入
}

#[test]
fn test_time_in_status_zero_duration_excluded() {
    // entered = exited 排除
}
```

## 10. i18n

```json
{
  "chart.c20.title": "状态停留时间",
  "chart.c20.x_axis": "平均天数",
  "chart.c20.empty.no_data": "无状态变更历史",
  "chart.c20.low_sample": "样本不足 ({n})"
}
```

## 11. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 | 2026-09-02 10:04 JST |
