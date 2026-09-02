# C04 Sprint Report 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira Sprint Report](https://support.atlassian.com/jira-software-cloud/docs/view-the-sprint-report/) | **需求**: [§3.4](../../requirements/charts-and-reports.md#34-c04--sprint-report) | **Spec**: [P0 #4](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c04_sprint_report.rs` + `frontend/src/components/charts/Chart04SprintReport.tsx`
> **工期**: 1d

---

## 1. 业务定义

**Sprint 完成度摘要**: 分"本期完成 / 上期完成(延期) / 未完成" 三组列表, 不是严格图表 (Table 模板)。

---

## 2. 数据源

| 实体 | 字段 |
|---|---|
| `Sprint` | `sprint_id, start_date, end_date, name` |
| `WorkItem` | `sprint_id, completed_at, status, key, title, type, priority, assignee` |

**分类规则**:
- **本期完成** = `completed_at ∈ [sprint.start, sprint.end]`
- **延期完成** = `completed_at ∈ [sprint.start, sprint.end]` 但 `sprint_id` 来源上一 Sprint
- **未完成** = Sprint 范围内但 `completed_at IS NULL` 且 `status != 'done'`

**SQL**:
```sql
-- 三组 issue
(SELECT 'completed' AS group_type, key, title, type, priority, assignee, completed_at
 FROM work_item
 WHERE sprint_id = $1 AND completed_at BETWEEN $start AND $end)
UNION ALL
(SELECT 'carry_over' AS group_type, ... -- 上期转本期)
UNION ALL
(SELECT 'incomplete' AS group_type, ...
 WHERE sprint_id IN ($1, $prev) AND completed_at IS NULL AND status != 'done')
```

---

## 3. 数据 Schema (TS)

```typescript
export interface SprintReportData {
  sprint: SprintMeta;
  prev_sprint?: SprintMeta;  // 用于延期判定
  groups: {
    completed: IssueRow[];       // 本期完成
    carry_over: IssueRow[];      // 延期完成
    incomplete: IssueRow[];      // 未完成
  };
  summary: {
    completed_count: number;
    carry_over_count: number;
    incomplete_count: number;
    completed_sp: number;
    carried_sp: number;
    remaining_sp: number;
  };
}

export interface IssueRow {
  key: string;            // "PROJ-123"
  title: string;
  type: string;
  priority: string;
  assignee?: { id: string; name: string; avatar_url: string };
  completed_at?: string;
  story_points?: number;
}
```

---

## 4. 渲染逻辑

### 4.1 组件选择

不是图表, 用 Table + 摘要卡片。

```tsx
<div className="grid grid-cols-4 gap-4 mb-4">
  <SummaryCard label={t('chart.c04.summary.completed')} value={data.summary.completed_count} tone="ok" />
  <SummaryCard label={t('chart.c04.summary.carry_over')} value={data.summary.carry_over_count} tone="warn" />
  <SummaryCard label={t('chart.c04.summary.incomplete')} value={data.summary.incomplete_count} tone="err" />
  <SummaryCard label={t('chart.c04.summary.completed_sp')} value={data.summary.completed_sp} tone="info" suffix="SP" />
</div>

<Tabs>
  <Tab label={`完成 (${data.groups.completed.length})`}>
    <IssueTable rows={data.groups.completed} />
  </Tab>
  <Tab label={`延期 (${data.groups.carry_over.length})`}>
    <IssueTable rows={data.groups.carry_over} />
  </Tab>
  <Tab label={`未完成 (${data.groups.incomplete.length})`}>
    <IssueTable rows={data.groups.incomplete} />
  </Tab>
</Tabs>
```

### 4.2 IssueTable 组件

```tsx
function IssueTable({ rows }: { rows: IssueRow[] }) {
  return (
    <table className="w-full">
      <thead>
        <tr>
          <th>{t('chart.c04.column.key')}</th>
          <th>{t('chart.c04.column.title')}</th>
          <th>{t('chart.c04.column.type')}</th>
          <th>{t('chart.c04.column.priority')}</th>
          <th>{t('chart.c04.column.assignee')}</th>
          <th>{t('chart.c04.column.sp')}</th>
        </tr>
      </thead>
      <tbody>
        {rows.map(r => (
          <tr key={r.key} onClick={() => router.push(`/issues/${r.key}`)} className="cursor-pointer hover:bg-zinc-50">
            <td><a className="text-blue-500">{r.key}</a></td>
            <td>{r.title}</td>
            <td><TypeBadge type={r.type} /></td>
            <td><PriorityBadge priority={r.priority} /></td>
            <td>{r.assignee && <UserAvatar user={r.assignee} />}</td>
            <td>{r.story_points ?? '-'}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
```

### 4.3 颜色 / 主题

| Group | 颜色 | Tone |
|---|---|---|
| 完成 | 绿 (`#10b981`) | ok |
| 延期 | 黄 (`#f59e0b`) | warn |
| 未完成 | 红 (`#ef4444`) | err |

---

## 5. 配置项

| 字段 | 默认 | 含义 |
|---|---|---|
| `default_tab` | `'completed'` | 默认展开 tab |
| `show_summary_cards` | `true` | 显示摘要卡片 |
| `page_size` | `50` | 列表分页 |
| `sort_by` | `'completed_at'` | 排序字段 |

---

## 6. 边界与异常

| 边界 | 处理 |
|---|---|
| 空 Sprint | 全部 group 都空, 显示 "无数据" |
| 没有 prev_sprint | carry_over = 0, 不显示该 tab |
| 列表 > 1000 | 分页, 默认 50/页 |
| Issue 跨 Sprint 来回转移 | 取 first sprint_id 为准 |

---

## 7. 性能

- 数据量: 单 Sprint 通常 < 200 issue, 极少 > 1000
- 性能预算: < 500ms query (含 3 个 UNION)

---

## 8. 测试用例

```rust
#[tokio::test]
async fn test_sprint_report_grouping() {
    // 3 完成 / 2 延期 / 5 未完成 → 验证 3 个 group 数量
}

#[tokio::test]
async fn test_sprint_report_carry_over_detection() {}

#[tokio::test]
async fn test_sprint_report_no_prev_sprint() {}
```

---

## 9. i18n

```json
{
  "chart.c04.title": "Sprint 报告",
  "chart.c04.summary.completed": "本期完成",
  "chart.c04.summary.carry_over": "延期完成",
  "chart.c04.summary.incomplete": "未完成",
  "chart.c04.summary.completed_sp": "完成 SP",
  "chart.c04.column.key": "Key",
  "chart.c04.column.title": "标题",
  "chart.c04.column.type": "类型",
  "chart.c04.column.priority": "优先级",
  "chart.c04.column.assignee": "经办人",
  "chart.c04.column.sp": "SP"
}
```

---

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 | 2026-09-02 10:04 JST |
