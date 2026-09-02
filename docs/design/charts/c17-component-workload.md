# C17 Workload by Component 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira Workload by Component](https://support.atlassian.com/jira-software-cloud/docs/) | **需求**: [§3.17](../../requirements/charts-and-reports.md#317-c17--workload-by-component) | **Spec**: [P2 #17](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c17_component_workload.rs` + `frontend/src/components/charts/Chart17ComponentWorkload.tsx`
> **工期**: 1.5d (复用 C16)

---

## 1. 业务定义

**按 component 分组的 open issue 数 (横向 Bar)**, 与 C16 同构, 维度替换为 component。

## 2-4. 复用 C16

**差异**:
- SQL: `JOIN work_item_component ON ... GROUP BY component`
- TS schema: `component_id` / `component_name`
- 颜色: 与 component 关联, 走 component 自定义色

## 5. 配置项

| 字段 | 默认 | 含义 |
|---|---|---|
| `top_n` | `20` | top N component |
| `stack_mode` | `'stack'` | |
| `status_filter` | `'open'` | |

## 6-7. 同 C16

## 8. 测试

```rust
#[test]
fn test_component_workload_grouping() {}
```

## 9. i18n

```json
{
  "chart.c17.title": "按模块工作量",
  "chart.c17.unassigned": "未分类",
  "chart.c17.other_components": "其他 {n} 模块"
}
```

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 | 2026-09-02 10:04 JST |
