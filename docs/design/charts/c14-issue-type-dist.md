# C14 Issue Type Distribution 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira Pie Chart (Issue Type)](https://support.atlassian.com/jira-software-cloud/docs/view-the-pie-chart/) | **需求**: [§3.14](../../requirements/charts-and-reports.md#314-c14--issue-type-distribution) | **Spec**: [P1 #14](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c14_issue_type_dist.rs` + `frontend/src/components/charts/Chart14IssueTypeDist.tsx`
> **工期**: 0.5d (复用 Pie 模板)

---

## 1. 业务定义

**按 issue type 分组的占比 Pie**, 含中心总数 + 标签。

---

## 2. 数据源

| 实体 | 字段 |
|---|---|
| `WorkItem` | `issue_type, tenant_id, project_id` |

**SQL**:
```sql
SELECT issue_type, COUNT(*) AS count
FROM work_item
WHERE tenant_id = $1 AND project_id = $2
  AND status NOT IN ('deleted')
GROUP BY issue_type
ORDER BY count DESC;
```

## 3. 数据 Schema (TS)

```typescript
export interface IssueTypeDistData {
  slices: Array<{
    type: string;          // "Task" / "Bug" / "Story"
    count: number;
    percentage: number;    // 0-1
  }>;
  total: number;
  status_filter?: string;  // 区分 open / closed
}
```

## 4. 渲染逻辑

Recharts `<PieChart>` + 自定义中心标签:

```tsx
<PieChart>
  <Pie
    data={data.slices}
    dataKey="count"
    nameKey="type"
    cx="50%"
    cy="50%"
    innerRadius={60}     // donut
    outerRadius={100}
    paddingAngle={2}
    onClick={(slice) => router.push(`/issues?type=${slice.type}`)}
  >
    {data.slices.map((s, i) => (
      <Cell key={i} fill={TYPE_COLORS[s.type] || '#94a3b8'} />
    ))}
  </Pie>
  <Tooltip content={<PieTooltip />} />
  <Legend />
  {/* 中心总数 */}
  <text x="50%" y="48%" textAnchor="middle" className="text-2xl font-bold">
    {data.total}
  </text>
  <text x="50%" y="55%" textAnchor="middle" className="text-sm fill-zinc-500">
    {t('chart.c14.total')}
  </text>
</PieChart>
```

## 5. 颜色

| Type | 颜色 |
|---|---|
| Bug | `#ef4444` (red) |
| Story | `#10b981` (emerald) |
| Task | `#3b82f6` (blue) |
| Epic | `#a855f7` (purple) |
| Subtask | `#94a3b8` (slate) |
| 其他 | `#cbd5e1` |

## 6. 配置项

| 字段 | 默认 | 含义 |
|---|---|---|
| `status_filter` | `'all'` | `'all'` / `'open'` / `'closed'` |
| `donut_mode` | `true` | donut 模式 (有内径) |
| `show_legend` | `true` | 显示图例 |
| `min_slice_percentage` | `2` | 合并小于 2% 的 slice 为 "其他" |

## 7. 边界

| 边界 | 处理 |
|---|---|
| 0 issue | "无数据" |
| 1 type | 整个圆, 中心显示 100% |
| > 10 type | top 10 + "其他" |
| Slice < 2% | 合并到 "其他" |

## 8. 性能

- 单 SQL, 快速
- 性能预算: < 100ms

## 9. 测试

```rust
#[test]
fn test_issue_type_dist_basic() {}
#[test]
fn test_min_slice_merge() {
    // < 2% 合并为 "其他"
}
```

## 10. i18n

```json
{
  "chart.c14.title": "问题类型分布",
  "chart.c14.total": "总数",
  "chart.c14.type.bug": "Bug",
  "chart.c14.type.story": "Story",
  "chart.c14.type.task": "Task",
  "chart.c14.type.epic": "Epic",
  "chart.c14.type.subtask": "子任务",
  "chart.c14.type.other": "其他"
}
```

## 11. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 | 2026-09-02 10:04 JST |
