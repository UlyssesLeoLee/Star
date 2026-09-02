# C16 Assignee Workload 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira Assignee Workload](https://support.atlassian.com/jira-software-cloud/docs/view-the-assignee-workload-report/) | **需求**: [§3.16](../../requirements/charts-and-reports.md#316-c16--assignee-workload) | **Spec**: [P2 #16](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c16_assignee_workload.rs` + `frontend/src/components/charts/Chart16AssigneeWorkload.tsx`
> **工期**: 1.5d

---

## 1. 业务定义

**每个 assignee 当前 open issue 数 (横向 Bar)**, 可按状态堆叠。

---

## 2. 数据源

| 实体 | 字段 |
|---|---|
| `WorkItem` | `status, workitem_id, tenant_id, project_id` |
| `WorkItemAssignee` | `workitem_id, user_id` |
| `User` | `user_id, name, avatar_url` |

**SQL**:
```sql
SELECT
    u.user_id, u.name, u.avatar_url,
    wi.status,
    COUNT(*) AS count
FROM work_item wi
JOIN work_item_assignee wia ON wi.workitem_id = wia.workitem_id
JOIN "user" u ON wia.user_id = u.user_id
WHERE wi.tenant_id = $1
  AND wi.project_id = $2
  AND wi.status NOT IN ('done', 'closed', 'resolved')
GROUP BY u.user_id, u.name, u.avatar_url, wi.status
ORDER BY count DESC
LIMIT $3;  -- top_n
```

## 3. 数据 Schema (TS)

```typescript
export interface AssigneeWorkloadData {
  rows: Array<{
    user_id: string;
    name: string;
    avatar_url: string;
    by_status: Record<string, number>;  // { todo: 5, in_progress: 2, in_review: 1 }
    total: number;
  }>;
  stack_mode: 'stack' | 'group' | 'none';
}
```

## 4. 渲染逻辑

```tsx
<BarChart data={data.rows} layout="vertical">
  <CartesianGrid strokeDasharray="3 3" />
  <XAxis type="number" label={{ value: 'issues', position: 'insideBottom' }} />
  <YAxis type="category" dataKey="name" width={120}>
    <LabelList dataKey="total" position="right" />
  </YAxis>
  <Tooltip content={<AssigneeTooltip />} />
  <Legend />
  {statusCategories.map(sc => (
    <Bar
      key={sc}
      dataKey={`by_status.${sc}`}
      stackId="a"
      name={t(`chart.c16.status.${sc}`)}
      fill={STATUS_COLORS[sc]}
    />
  ))}
</BarChart>
```

## 5. 配置项

| 字段 | 默认 | 含义 |
|---|---|---|
| `top_n` | `20` | top N 用户 |
| `stack_mode` | `'stack'` | `'stack'` / `'group'` / `'none'` |
| `status_filter` | `'open'` | `'open'` / `'all'` |
| `sort_by` | `'total'` | `'total'` / `'in_progress'` / `'name'` |

## 6. 边界

| 边界 | 处理 |
|---|---|
| 0 数据 | "无数据" |
| 用户数 > top_n | 仅 top N + "其他" |
| 单用户单状态 | 单 Bar |

## 7. 性能

- 性能预算: < 300ms

## 8. 测试

```rust
#[test]
fn test_assignee_workload_grouping() {}
#[test]
fn test_top_n_limit() {}
```

## 9. i18n

```json
{
  "chart.c16.title": "经办人工作量",
  "chart.c16.status.todo": "待办",
  "chart.c16.status.in_progress": "进行中",
  "chart.c16.status.in_review": "评审中",
  "chart.c16.stack_mode.stack": "堆叠",
  "chart.c16.stack_mode.group": "分组",
  "chart.c16.other_users": "其他 {n} 人"
}
```

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 | 2026-09-02 10:04 JST |
