# C15 Priority Distribution 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira Pie Chart (Priority)](https://support.atlassian.com/jira-software-cloud/docs/view-the-pie-chart/) | **需求**: [§3.15](../../requirements/charts-and-reports.md#315-c15--priority-distribution) | **Spec**: [P1 #15](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c15_priority_dist.rs` + `frontend/src/components/charts/Chart15PriorityDist.tsx`
> **工期**: 0.5d (复用 C14 Pie 模板)

---

## 1. 业务定义

**按 priority 分组的占比 Pie**。

---

## 2-4. 与 C14 高度同构

复用 C14 全部代码, 仅维度从 `issue_type` 改为 `priority`, 颜色重定义。

**差异**:
- SQL: `GROUP BY priority`
- TS schema: `slices[].priority` 替代 `type`
- 颜色:

| Priority | 颜色 |
|---|---|
| highest | `#7c2d12` (深红) |
| high | `#ef4444` (红) |
| medium | `#f59e0b` (琥珀) |
| low | `#3b82f6` (蓝) |
| lowest | `#94a3b8` (灰) |

## 5. 配置项

| 字段 | 默认 | 含义 |
|---|---|---|
| `status_filter` | `'open'` | 默认只看 open |
| `donut_mode` | `true` | donut |
| `show_legend` | `true` | |

## 6. 边界

同 C14

## 7. 性能

< 100ms (单 SQL)

## 8. 测试

```rust
#[test]
fn test_priority_dist_grouping() {}
```

## 9. i18n

```json
{
  "chart.c15.title": "优先级分布",
  "chart.c15.priority.highest": "最高",
  "chart.c15.priority.high": "高",
  "chart.c15.priority.medium": "中",
  "chart.c15.priority.low": "低",
  "chart.c15.priority.lowest": "最低"
}
```

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 | 2026-09-02 10:04 JST |
