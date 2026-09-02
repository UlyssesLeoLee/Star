# C22 Recently Created 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira Recently Created Issues](https://support.atlassian.com/jira-software-cloud/docs/view-the-recently-created-issues-report/) | **需求**: [§3.22](../../requirements/charts-and-reports.md#322-c22--recently-created) | **Spec**: [P2 #22](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c22_recently_created.rs` + `frontend/src/components/charts/Chart22RecentlyCreated.tsx`
> **工期**: 1d (Table 模板)

---

## 1. 业务定义

**最近创建的 issue 列表 (分页 + 排序 + 跳转)**, 不算严格图表 (Table 模板)。

---

## 2. 数据源

| 实体 | 字段 |
|---|---|
| `WorkItem` | `key, title, type, priority, reporter, assignee, created_at, status` |

**SQL** (分页):
```sql
SELECT
    wi.key, wi.title, wi.issue_type, wi.priority, wi.status, wi.created_at,
    rep.name AS reporter_name, rep.avatar_url AS reporter_avatar,
    asg.name AS assignee_name, asg.avatar_url AS assignee_avatar
FROM work_item wi
LEFT JOIN "user" rep ON wi.reporter_id = rep.user_id
LEFT JOIN work_item_assignee wia ON wi.workitem_id = wia.workitem_id
LEFT JOIN "user" asg ON wia.user_id = asg.user_id
WHERE wi.tenant_id = $1
  AND wi.project_id = $2
  AND wi.created_at >= $start
ORDER BY wi.created_at DESC
LIMIT $page_size OFFSET $offset;
```

## 3. 数据 Schema (TS)

```typescript
export interface RecentlyCreatedData {
  rows: Array<{
    key: string;
    title: string;
    type: string;
    priority: string;
    status: string;
    reporter?: { name: string; avatar_url: string };
    assignee?: { name: string; avatar_url: string };
    created_at: string;
  }>;
  pagination: {
    page: number;
    page_size: number;
    total: number;
    total_pages: number;
  };
  time_range: { start: string; end: string };
}
```

## 4. 渲染逻辑

```tsx
<Table>
  <thead>
    <tr>
      <th>{t('chart.c22.column.key')}</th>
      <th>{t('chart.c22.column.title')}</th>
      <th>{t('chart.c22.column.type')}</th>
      <th>{t('chart.c22.column.priority')}</th>
      <th>{t('chart.c22.column.status')}</th>
      <th>{t('chart.c22.column.reporter')}</th>
      <th>{t('chart.c22.column.assignee')}</th>
      <th>{t('chart.c22.column.created_at')}</th>
    </tr>
  </thead>
  <tbody>
    {data.rows.map(r => (
      <tr key={r.key} onClick={() => router.push(`/issues/${r.key}`)} className="cursor-pointer hover:bg-zinc-50">
        <td><a className="text-blue-500">{r.key}</a></td>
        <td>{r.title}</td>
        <td><TypeBadge type={r.type} /></td>
        <td><PriorityBadge priority={r.priority} /></td>
        <td><StatusPill status={r.status} /></td>
        <td>{r.reporter && <UserAvatar user={r.reporter} />}</td>
        <td>{r.assignee && <UserAvatar user={r.assignee} />}</td>
        <td>{formatRelativeTime(r.created_at)}</td>
      </tr>
    ))}
  </tbody>
</Table>
<Pagination {...data.pagination} />
```

## 5. 配置项

| 字段 | 默认 | 含义 |
|---|---|---|
| `time_range` | `{LastNDays: 7}` | 时间窗 |
| `page_size` | `50` | 每页 |
| `sort_by` | `'created_at desc'` | |
| `filter_by_type` | `[]` | 类型过滤 |
| `filter_by_priority` | `[]` | 优先级过滤 |

## 6. 边界

| 边界 | 处理 |
|---|---|
| 0 数据 | "无新建 issue" |
| 大量 (> 10000) | 分页 + 提示用户缩小时间窗 |
| 超大标题 | 截断 + tooltip |

## 7. 性能

- 7 天 × 50/页 = 50 行
- 性能预算: < 300ms

### 7.1 索引

```sql
CREATE INDEX idx_wi_tenant_created ON work_item(tenant_id, created_at DESC);
```

## 8. 测试

```rust
#[test]
fn test_recently_created_sort() {}
#[test]
fn test_recently_created_pagination() {}
```

## 9. i18n

```json
{
  "chart.c22.title": "最近创建",
  "chart.c22.column.key": "Key",
  "chart.c22.column.title": "标题",
  "chart.c22.column.type": "类型",
  "chart.c22.column.priority": "优先级",
  "chart.c22.column.status": "状态",
  "chart.c22.column.reporter": "报告人",
  "chart.c22.column.assignee": "经办人",
  "chart.c22.column.created_at": "创建时间",
  "chart.c22.empty": "无新建 issue"
}
```

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 | 2026-09-02 10:04 JST |
