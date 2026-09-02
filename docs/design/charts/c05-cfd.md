# C05 Cumulative Flow Diagram (CFD) 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira CFD](https://support.atlassian.com/jira-software-cloud/docs/view-the-cumulative-flow-diagram/) | **需求**: [§3.5](../../requirements/charts-and-reports.md#35-c05--cumulative-flow-diagram-cfd) | **Spec**: [P0 #5](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c05_cfd.rs` + `frontend/src/components/charts/Chart05Cfd.tsx`
> **工期**: 2d

---

## 1. 业务定义

**项目内每天各状态 (To Do / In Progress / Done) 的 issue 数量堆叠面积图**, 反映工作流瓶颈。

---

## 2. 数据源

| 实体 | 字段 |
|---|---|
| `WorkItem` | `workitem_id, status, status_changed_at, created_at, deleted_at` |
| `WorkflowStatus` | `status_id, name, category (todo/in_progress/done)` |

**SQL** (每日快照聚合):
```sql
-- 用 window function 算每日各状态计数
WITH date_series AS (
    SELECT generate_series($start_date::date, $end_date::date, '1 day')::date AS day
),
status_per_day AS (
    SELECT
        d.day,
        wish.category,
        COUNT(wish.workitem_id) AS count
    FROM date_series d
    LEFT JOIN work_item_status_history wish
        ON wish.entered_at::date <= d.day
        AND (wish.exited_at IS NULL OR wish.exited_at::date > d.day)
    LEFT JOIN workflow_status ws ON wish.status_id = ws.status_id
    WHERE wish.tenant_id = $1
    GROUP BY d.day, ws.category
)
SELECT day, category, count
FROM status_per_day
ORDER BY day;
```

---

## 3. 数据 Schema (TS)

```typescript
export interface CfdData {
  date_range: { start: string; end: string };
  status_categories: string[];  // ['todo', 'in_progress', 'in_review', 'done']
  series: Array<{
    day: string;
    counts: Record<string, number>;  // { todo: 30, in_progress: 12, done: 50 }
  }>;
  total: number;  // 每日总和恒等于 issue 总数
}
```

---

## 4. 渲染逻辑

### 4.1 Recharts 组件

| 元素 | 组件 | 备注 |
|---|---|---|
| 整体 | `<AreaChart>` | 堆叠面积 |
| 各状态 | `<Area dataKey="counts.todo">` 等 | 多个堆叠 |
| Stacked | `<Area stackId="1">` | 全部 stackId 相同 |
| 关键线 | `<ReferenceLine>` (Done 顶部) | 显示总吞吐量 |

```tsx
<AreaChart data={data.series}>
  <CartesianGrid strokeDasharray="3 3" />
  <XAxis dataKey="day" />
  <YAxis label={{ value: 'issues', angle: -90 }} />
  <Tooltip content={<CfdTooltip />} />
  <Legend />
  {data.status_categories.map(cat => (
    <Area
      key={cat}
      type="monotone"
      dataKey={`counts.${cat}`}
      stackId="1"
      name={t(`chart.c05.category.${cat}`)}
      fill={CATEGORY_COLORS[cat]}
      stroke={CATEGORY_COLORS[cat]}
    />
  ))}
</AreaChart>
```

### 4.2 颜色

| Category | 浅色 | 深色 |
|---|---|---|
| todo | `#94a3b8` (slate) | `#64748b` |
| in_progress | `#3b82f6` (blue) | `#60a5fa` |
| in_review | `#a855f7` (purple) | `#c084fc` |
| done | `#10b981` (emerald) | `#34d399` |

> 顺序: 浅 → 深, 顶部为 done (突出吞吐量)

### 4.3 交互

| 交互 | 行为 |
|---|---|
| 悬停 | tooltip: 日期 / 各状态计数 / 总数 |
| 切换状态 | legend 点击切换显示/隐藏 |
| 时间窗 | 7/30/90/365 天 / 自定义 |
| 拖动 | Brush 缩放 |
| 导出 | CSV/PNG/PDF |

---

## 5. 配置项

| 字段 | 默认 | 含义 |
|---|---|---|
| `time_range` | `{mode: 'LastNDays', n_days: 30}` | 时间窗 |
| `show_categories` | 全部 | 显示哪些 category |
| `color_scheme` | `'default'` | 色盲切换 |
| `show_total_line` | `false` | 是否画总数线 |

---

## 6. 边界与异常

| 边界 | 处理 |
|---|---|
| 0 issue | 仅画 X 轴, 显示 "无数据" |
| 自定义 workflow | 取当前 active status 作为 category |
| Workflow 变更 (新增状态) | 旧数据按新 category 映射, 显示断点 |
| 软删除 issue | 从计数中排除 (per `deleted_at IS NULL`) |
| 大量 issue (> 10K) | 自动按周聚合, 减少点 |

---

## 7. 性能

- 每日每状态聚合, 时间窗 30 天 × 4 category = 120 数据点
- 时间窗 365 天 = 1460 数据点 (Recharts 仍轻松)
- 性能预算: < 1s query, < 500ms render

### 7.1 索引要求

```sql
CREATE INDEX idx_wish_tenant_entered
    ON work_item_status_history(tenant_id, entered_at);

CREATE INDEX idx_wish_tenant_exited
    ON work_item_status_history(tenant_id, exited_at)
    WHERE exited_at IS NOT NULL;
```

---

## 8. 测试用例

```rust
#[tokio::test]
async fn test_cfd_total_invariant() {
    // 每日各 category 总和恒等于 issue 总数
}

#[tokio::test]
async fn test_cfd_workflow_change() {
    // 新增状态后, 旧数据正确映射
}

#[tokio::test]
async fn test_cfd_time_range_365d() {
    // 365 天大数据量, 性能 < 1s
}
```

---

## 9. i18n

```json
{
  "chart.c05.title": "累积流图 (CFD)",
  "chart.c05.x_axis": "日期",
  "chart.c05.y_axis": "Issue 数",
  "chart.c05.category.todo": "待办",
  "chart.c05.category.in_progress": "进行中",
  "chart.c05.category.in_review": "评审中",
  "chart.c05.category.done": "完成"
}
```

---

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 | 2026-09-02 10:04 JST |
