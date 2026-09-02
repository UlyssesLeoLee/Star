# C18 Version Workload Report 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira Version Workload Report](https://support.atlassian.com/jira-software-cloud/docs/view-the-version-workload-report/) | **需求**: [§3.18](../../requirements/charts-and-reports.md#318-c18--version-workload) | **Spec**: [P2 #18](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c18_version_workload.rs` + `frontend/src/components/charts/Chart18VersionWorkload.tsx`
> **工期**: 1.5d

---

## 1. 业务定义

**按 Version 分组的 issue 数 (按完成状态堆叠)**, 含已发布/未发布分组。

---

## 2. 数据源

| 实体 | 字段 |
|---|---|
| `Version` | `version_id, name, released, release_date` |
| `WorkItem` | `fix_version_id, status, story_points` |

**SQL**:
```sql
SELECT
    v.version_id, v.name, v.released, v.release_date,
    wi.status,
    SUM(wi.story_points) AS sp_sum,
    COUNT(*) AS count
FROM version v
LEFT JOIN work_item wi ON wi.fix_version_id = v.version_id
WHERE v.tenant_id = $1 AND v.project_id = $2
GROUP BY v.version_id, v.name, v.released, v.release_date, wi.status
ORDER BY v.released DESC, v.release_date DESC;
```

## 3. 数据 Schema (TS)

```typescript
export interface VersionWorkloadData {
  versions: Array<{
    version_id: string;
    name: string;
    released: boolean;
    release_date?: string;
    by_status: Record<string, { count: number; sp_sum: number }>;
    total_count: number;
    total_sp: number;
  }>;
  group_by_released: 'separated' | 'merged';
}
```

## 4. 渲染逻辑

```tsx
<BarChart data={data.versions} layout="vertical">
  <CartesianGrid strokeDasharray="3 3" />
  <XAxis type="number" />
  <YAxis type="category" dataKey="name" width={150} />
  <Tooltip content={<VersionTooltip />} />
  <Legend />
  {statusCategories.map(sc => (
    <Bar key={sc} dataKey={`by_status.${sc}.count`} stackId="a" fill={STATUS_COLORS[sc]}
      name={t(`chart.c18.status.${sc}`)} />
  ))}
</BarChart>
```

## 5. 颜色 / 状态

| Status | 颜色 |
|---|---|
| done | `#10b981` (green) |
| in_progress | `#3b82f6` (blue) |
| todo | `#94a3b8` (slate) |
| unresolved | `#cbd5e1` (light slate) |

## 6. 配置项

| 字段 | 默认 | 含义 |
|---|---|---|
| `group_by_released` | `'separated'` | 已发布/未发布分组 |
| `unit` | `'count'` | `'count'` / `'sp'` |
| `top_n` | `20` | top N version |
| `include_released` | `true` | 含已发布 |

## 7. 边界

| 边界 | 处理 |
|---|---|
| 0 version | "无版本" |
| 已发布但无 issue | 显示 0 |
| 未发布 + 无 issue | 不显示 |

## 8. 性能

- 性能预算: < 300ms

## 9. 测试

```rust
#[test]
fn test_version_workload_aggregation() {}
```

## 10. i18n

```json
{
  "chart.c18.title": "版本工作量报告",
  "chart.c18.released": "已发布",
  "chart.c18.unreleased": "未发布",
  "chart.c18.status.done": "完成",
  "chart.c18.status.in_progress": "进行中",
  "chart.c18.status.todo": "待办",
  "chart.c18.unit.count": "按计数",
  "chart.c18.unit.sp": "按 SP"
}
```

## 11. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 | 2026-09-02 10:04 JST |
