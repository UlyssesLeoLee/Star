# C12 SLA Compliance Report 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira SLA (V1 创新, 官方无)](https://support.atlassian.com/jira-software-cloud/docs/) | **需求**: [§3.12](../../requirements/charts-and-reports.md#312-c12--sla-compliance) | **Spec**: [P1 #13](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c12_sla_compliance.rs` + `frontend/src/components/charts/Chart12SlaCompliance.tsx`
> **工期**: 2d

---

## 1. 业务定义

**SLA 命中率 (%) 随时间变化**, 按优先级叠加, 监控服务承诺履行。

---

## 2. 数据源

| 实体 | 字段 |
|---|---|
| `SlaDefinition` | `sla_id, project_id, priority, target_resolution_hours` |
| `WorkItem` | `resolved_at, created_at, priority, project_id, tenant_id` |

**SLA 命中判定**:
```rust
fn is_sla_met(work_item: &WorkItem, sla: &SlaDefinition) -> bool {
    let resolution_hours = (work_item.resolved_at - work_item.created_at).num_hours();
    resolution_hours <= sla.target_resolution_hours as i64
}
```

**SQL**:
```sql
SELECT
    date_trunc('day', wi.resolved_at) AS day,
    wi.priority,
    COUNT(*) AS total,
    SUM(CASE WHEN EXTRACT(EPOCH FROM (wi.resolved_at - wi.created_at)) / 3600.0 <= sla.target_resolution_hours THEN 1 ELSE 0 END) AS met
FROM work_item wi
JOIN sla_definition sla ON sla.project_id = wi.project_id AND sla.priority = wi.priority
WHERE wi.tenant_id = $1
  AND wi.resolved_at BETWEEN $start AND $end
  AND sla.is_current = TRUE
GROUP BY day, wi.priority
ORDER BY day;
```

## 3. 数据 Schema (TS)

```typescript
export interface SlaComplianceData {
  series: Array<{
    day: string;
    priorities: Record<string, { met: number; total: number; compliance: number /* 0-1 */ }>;
  }>;
  summary: {
    overall_compliance: number;
    by_priority: Record<string, number>;
    breaches: number;  // 未命中数
  };
  sla_definitions: Array<{
    priority: string;
    target_hours: number;
  }>;
}
```

## 4. 渲染逻辑

```tsx
<LineChart data={data.series}>
  <CartesianGrid strokeDasharray="3 3" />
  <XAxis dataKey="day" />
  <YAxis domain={[0, 100]} tickFormatter={(v) => `${v}%`} label={{ value: 'SLA %', angle: -90 }} />
  <Tooltip content={<SlaTooltip />} />
  <Legend />
  {priorities.map(p => (
    <Line
      key={p}
      type="monotone"
      dataKey={`priorities.${p}.compliance`}
      name={t(`chart.c12.priority.${p}`)}
      stroke={PRIORITY_COLORS[p]}
      strokeWidth={2}
    />
  ))}
  <ReferenceLine y={95} stroke="#10b981" strokeDasharray="5 5" label="Target 95%" />
</LineChart>
```

## 5. 颜色

| Priority | 颜色 |
|---|---|
| highest | `#ef4444` (red) |
| high | `#f59e0b` (amber) |
| medium | `#3b82f6` (blue) |
| low | `#94a3b8` (slate) |

## 6. 配置项

| 字段 | 默认 | 含义 |
|---|---|---|
| `sla_id` | (按 project) | SLA 定义 ID |
| `target_line` | `95` | 目标线 (%) |
| `time_range` | `{LastNDays: 30}` | 时间窗 |

## 7. 边界

| 边界 | 处理 |
|---|---|
| 无 SLA 定义 | 提示 "请先在项目设置定义 SLA" |
| 0 resolved issue | "无数据" |
| 命中率 < 50% | 标 ⚠ |
| 未命中数 > 0 | 红色高亮 + 数字 |

## 8. 性能

- 30 天 × 4 priority = 120 数据点
- 性能预算: < 300ms

## 9. 测试

```rust
#[test]
fn test_sla_met_simple() {
    let wi = work_item_with_resolution(10);  // 10 hours
    let sla = sla_with_target(24);
    assert!(is_sla_met(&wi, &sla));
}

#[test]
fn test_sla_unmet() {
    let wi = work_item_with_resolution(30);
    let sla = sla_with_target(24);
    assert!(!is_sla_met(&wi, &sla));
}

#[test]
fn test_compliance_calculation() {
    // 10 total, 8 met → 80%
}
```

## 10. i18n

```json
{
  "chart.c12.title": "SLA 合规报告",
  "chart.c12.priority.highest": "最高",
  "chart.c12.priority.high": "高",
  "chart.c12.priority.medium": "中",
  "chart.c12.priority.low": "低",
  "chart.c12.target_line": "目标 {n}%",
  "chart.c12.summary.overall": "整体合规率: {n}%",
  "chart.c12.summary.breaches": "未命中: {n}"
}
```

## 11. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 | 2026-09-02 10:04 JST |
