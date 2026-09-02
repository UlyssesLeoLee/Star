# C03 Velocity Chart 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira Velocity Chart](https://support.atlassian.com/jira-software-cloud/docs/view-the-velocity-chart/) | **需求**: [§3.3](../../requirements/charts-and-reports.md#33-c03--velocity-chart) | **Spec**: [P0 #3](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c03_velocity.rs` + `frontend/src/components/charts/Chart03Velocity.tsx`
> **工期**: 1.5d

---

## 1. 业务定义

**跨多个 Sprint 团队承诺 SP vs 完成 SP 对比** + 平均完成线, 用于评估团队稳定输出能力。

---

## 2. 数据源

| 实体 | 字段 |
|---|---|
| `Sprint` | `sprint_id, name, committed_sp, completed_sp, start_date, end_date, status` |

**SQL**:
```sql
SELECT
    s.sprint_id, s.name, s.committed_sp, s.completed_sp,
    s.start_date, s.end_date, s.status
FROM sprint s
WHERE s.project_id = $1
  AND s.status IN ('completed', 'active')
  AND s.tenant_id = $2
  AND s.is_current = TRUE
ORDER BY s.start_date DESC
LIMIT $3;  -- config.top_n_sprints (默认 6)
```

---

## 3. 数据 Schema (TS)

```typescript
export interface VelocityData {
  sprints: Array<{
    sprint_id: string;
    name: string;
    start_date: string;
    end_date: string;
    status: 'completed' | 'active' | 'planned';
    committed_sp: number;
    completed_sp: number | null;  // null = active sprint
  }>;
  average_completed_sp: number;     // 历史平均
  average_committed_sp: number;
  trend: 'increasing' | 'decreasing' | 'stable';
}
```

---

## 4. 渲染逻辑

### 4.1 Recharts 组件

| 元素 | 组件 | 备注 |
|---|---|---|
| Committed (承诺) | `<Bar dataKey="committed_sp">` | 蓝色, 半透明 |
| Completed (完成) | `<Bar dataKey="completed_sp">` | 绿色, 实心 |
| Average line | `<ReferenceLine y={average_completed_sp} stroke="#94a3b8" strokeDasharray="5 5">` | 水平虚线 |

```tsx
<BarChart data={data.sprints} barCategoryGap="20%">
  <CartesianGrid strokeDasharray="3 3" />
  <XAxis dataKey="name" />
  <YAxis label={{ value: 'SP', angle: -90 }} />
  <Tooltip content={<VelocityTooltip />} />
  <Legend />
  <Bar dataKey="committed_sp" name={t('chart.c03.series.committed')} fill="#3b82f6" fillOpacity={0.5} />
  <Bar dataKey="completed_sp" name={t('chart.c03.series.completed')} fill="#10b981" />
  <ReferenceLine y={data.average_completed_sp} stroke="#94a3b8" strokeDasharray="5 5"
    label={{ value: t('chart.c03.average', { n: data.average_completed_sp }), position: 'right' }} />
</BarChart>
```

### 4.2 颜色

| 元素 | 浅色 | 深色 |
|---|---|---|
| Committed | `#3b82f6` (alpha 0.5) | `#60a5fa` |
| Completed | `#10b981` | `#34d399` |
| Average line | `#94a3b8` | `#64748b` |

### 4.3 交互

| 交互 | 行为 |
|---|---|
| 悬停 | tooltip: Sprint 名称 / 承诺 / 完成 / 完成率 |
| 切换 series | 显示/隐藏 committed / completed |
| 切换时间窗 | top_n_sprints (3/6/12/all) |
| 点击 bar | 跳转 Sprint detail |
| 趋势显示 | 右上角 badge: 📈 / 📉 / ➡️ |

---

## 5. 配置项

| 字段 | 默认 | 含义 |
|---|---|---|
| `top_n_sprints` | `6` | 显示最近 N 个 Sprint |
| `show_committed` | `true` | 显示承诺柱 |
| `show_average_line` | `true` | 显示平均线 |
| `show_trend` | `true` | 显示趋势 badge |
| `color_scheme` | `'default'` | 色盲切换 |

---

## 6. 边界与异常

| 边界 | 处理 |
|---|---|
| < 2 个已完成 Sprint | 不画平均线, 显示提示 "需要更多历史数据" |
| Active Sprint | committed_sp 有值, completed_sp = null, 半透明 |
| 所有 Sprint committed=0 | 显示 "未规划" 提示 |
| Trend 计算 | 最近 3 vs 前 3 的 avg, 差异 > 10% 才标记 |

---

## 7. 性能

- 数据量: top_n 默认 6, 极少 > 20
- 性能预算: < 200ms query, < 500ms render

---

## 8. 测试用例

```rust
#[tokio::test]
async fn test_velocity_avg_calculation() {}

#[tokio::test]
async fn test_velocity_active_sprint_null_completed() {}

#[tokio::test]
async fn test_velocity_trend_detection() {
    // 最近 3 avg = 30, 前 3 avg = 20, 应标 increasing
}
```

---

## 9. i18n

```json
{
  "chart.c03.title": "速度图",
  "chart.c03.series.committed": "承诺",
  "chart.c03.series.completed": "完成",
  "chart.c03.average": "平均: {n} SP",
  "chart.c03.trend.increasing": "上升",
  "chart.c03.trend.decreasing": "下降",
  "chart.c03.trend.stable": "稳定",
  "chart.c03.empty.less_than_2": "需要至少 2 个已完成 Sprint"
}
```

---

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 | 2026-09-02 10:04 JST |
