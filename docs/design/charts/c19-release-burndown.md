# C19 Release Burndown 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira Release Burndown](https://support.atlassian.com/jira-software-cloud/docs/view-the-release-burndown/) | **需求**: [§3.19](../../requirements/charts-and-reports.md#319-c19--release-burndown) | **Spec**: [P2 #19](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c19_release_burndown.rs` + `frontend/src/components/charts/Chart19ReleaseBurndown.tsx`
> **工期**: 1.5d (复用 C01)

---

## 1. 业务定义

**Version 发布前的剩余 issue 数随时间下降**, 与 C01 同构, 差异在 scope (Version 而非 Sprint) + X 轴终点 (release_due_date)。

## 2-4. 复用 C01

**差异**:
- X 轴终点: `version.release_due_date` (而非 sprint.end_date)
- SQL: `WHERE wi.fix_version_id = $1` (而非 sprint_id)
- TS schema: `version` 替代 `sprint`

## 5. 配置项

| 字段 | 默认 | 含义 |
|---|---|---|
| `y_axis_unit` | `'issue_count'` | `'issue_count'` (默认) / `'sp'` |
| `show_ideal_line` | `true` | |
| `show_release_date_marker` | `true` | |

## 6. 边界

| 边界 | 处理 |
|---|---|
| Version 未设 release_due_date | 不显示 |
| Version 已发布 | 折叠, 显示 "已发布" 标签 |
| 0 issue | "无 issue" |
| release_due_date < today | ⚠ 已逾期 |

## 7. 性能

- 性能预算: < 200ms (复用 C01 缓存)

## 8. 测试

```rust
#[test]
fn test_release_burndown_vs_sprint() {
    // 验证 X 轴终点是 release_due_date
}
```

## 9. i18n

```json
{
  "chart.c19.title": "发布燃尽",
  "chart.c19.release_date": "发布日期",
  "chart.c19.released": "已发布",
  "chart.c19.overdue": "已逾期"
}
```

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 | 2026-09-02 10:04 JST |
