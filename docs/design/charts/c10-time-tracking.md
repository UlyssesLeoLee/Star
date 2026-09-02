# C10 Time Tracking Report 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira Time Tracking Report](https://support.atlassian.com/jira-software-cloud/docs/view-the-time-tracking-report/) | **需求**: [§3.10](../../requirements/charts-and-reports.md#310-c10--time-tracking-report) | **Spec**: [P1 #11](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c10_time_tracking.rs` + `frontend/src/components/charts/Chart10TimeTracking.tsx`
> **工期**: 2d

---

## 1. 业务定义

**估时 vs 已记录 vs 剩余时间** 报告, 按用户/项目/issue 分组。

---

## 2. 数据源

| 实体 | 字段 |
|---|---|
| `WorkItem` | `original_estimate_seconds, time_spent_seconds, remaining_estimate_seconds` |
| `WorkLog` | `worklog_id, issue_id, time_spent_seconds, author_id, started_at` |

**SQL** (按用户聚合):
```sql
SELECT
    u.user_id, u.name, u.avatar_url,
    SUM(wi.original_estimate_seconds) AS original,
    SUM(wi.time_spent_seconds) AS spent,
    SUM(wi.remaining_estimate_seconds) AS remaining
FROM work_item wi
JOIN work_item_assignee wia ON wi.workitem_id = wia.workitem_id
JOIN "user" u ON wia.user_id = u.user_id
WHERE wi.tenant_id = $1 AND wi.project_id = $2
GROUP BY u.user_id, u.name, u.avatar_url
ORDER BY spent DESC;
```

---

## 3. 数据 Schema (TS)

```typescript
export interface TimeTrackingData {
  granularity: 'user' | 'project' | 'issue';
  rows: Array<{
    id: string;                    // user_id / project_id / issue_key
    name: string;
    avatar_url?: string;
    original_seconds: number;
    spent_seconds: number;
    remaining_seconds: number;
    progress: number;              // 0-1
  }>;
  summary: {
    total_original: number;
    total_spent: number;
    total_remaining: number;
    overall_progress: number;
  };
}
```

## 4. 渲染逻辑

混合 Table + 摘要 Bar:

```tsx
<Table>
  <thead>
    <tr>
      <th>{t('chart.c10.column.name')}</th>
      <th>{t('chart.c10.column.original')}</th>
      <th>{t('chart.c10.column.spent')}</th>
      <th>{t('chart.c10.column.remaining')}</th>
      <th>{t('chart.c10.column.progress')}</th>
    </tr>
  </thead>
  <tbody>
    {data.rows.map(r => (
      <tr key={r.id}>
        <td>{r.name}</td>
        <td>{formatDuration(r.original_seconds)}</td>
        <td>{formatDuration(r.spent_seconds)}</td>
        <td>{formatDuration(r.remaining_seconds)}</td>
        <td>
          <ProgressBar value={r.progress} />
          <span className="text-xs">{(r.progress * 100).toFixed(0)}%</span>
        </td>
      </tr>
    ))}
  </tbody>
</Table>
```

## 5. 配置项

| 字段 | 默认 | 含义 |
|---|---|---|
| `granularity` | `'user'` | `'user'` / `'project'` / `'issue'` |
| `time_range` | `{LastNDays: 30}` | 时间窗 (基于 worklog.started_at) |
| `top_n` | `50` | 限制行数 |
| `show_progress_bar` | `true` | 显示进度条 |

## 6. 边界

| 边界 | 处理 |
|---|---|
| 0 数据 | "无数据" |
| 所有 original = 0 | 不显示进度条 (无法计算) |
| spent > original | ⚠ 警告 (超时) |
| 列表 > 1000 | 分页 |

## 7. 性能

- 数据量: 取决于时间窗 + 用户数
- 性能预算: < 500ms (含 worklog join)

### 7.1 索引

```sql
CREATE INDEX idx_wi_tenant_project ON work_item(tenant_id, project_id);
CREATE INDEX idx_wia_user ON work_item_assignee(user_id);
CREATE INDEX idx_wl_started ON work_log(started_at) WHERE started_at IS NOT NULL;
```

## 8. 测试

```rust
#[test]
fn test_time_tracking_user_aggregation() {}
#[test]
fn test_overspent_detection() {}
```

## 9. i18n

```json
{
  "chart.c10.title": "时间跟踪报告",
  "chart.c10.granularity.user": "按用户",
  "chart.c10.granularity.project": "按项目",
  "chart.c10.granularity.issue": "按 Issue",
  "chart.c10.column.original": "原估时",
  "chart.c10.column.spent": "已用时",
  "chart.c10.column.remaining": "剩余",
  "chart.c10.column.progress": "进度",
  "chart.c10.warning.overspent": "超时"
}
```

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 | 2026-09-02 10:04 JST |
