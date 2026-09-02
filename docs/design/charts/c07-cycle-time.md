# C07 Cycle Time Report 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira Cycle Time Report](https://support.atlassian.com/jira-software-cloud/docs/view-the-cycle-time-report/) | **需求**: [§3.7](../../requirements/charts-and-reports.md#37-c07--cycle-time-report) | **Spec**: [P0 #7](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c07_cycle_time.rs` + `frontend/src/components/charts/Chart07CycleTime.tsx`
> **工期**: 1.5d

---

## 1. 业务定义

**周期时间的分布 (直方图) + 50/85/95 百分位**, 评估团队交付速度稳定性。

---

## 2. 数据源

同 C06, 计算每个 issue 的 cycle_time_days。

---

## 3. 数据 Schema (TS)

```typescript
export interface CycleTimeData {
  buckets: Array<{
    range_start: number;     // 0
    range_end: number;       // 1
    count: number;           // 12
    label: string;           // "0-1d"
  }>;
  percentiles: {
    p50: number;
    p85: number;
    p95: number;
  };
  stats: {
    total_count: number;
    median: number;
    mean: number;
    std_dev: number;
  };
  bucket_size: number;       // 1 / 3 / 7 days
}
```

---

## 4. 渲染逻辑

### 4.1 Recharts 组件

| 元素 | 组件 |
|---|---|
| 直方图 | `<BarChart>` |
| 桶 | `<Bar dataKey="count">` |
| 百分位线 | `<ReferenceLine>` (3 条) |

```tsx
<BarChart data={data.buckets}>
  <CartesianGrid strokeDasharray="3 3" />
  <XAxis dataKey="label" />
  <YAxis label={{ value: t('chart.c07.y_axis'), angle: -90 }} />
  <Tooltip content={<BucketTooltip />} />
  <Bar dataKey="count" fill="#3b82f6" />
  <ReferenceLine x={findBucketByPercentile(data.buckets, data.percentiles.p50)}
    stroke="#10b981" label={{ value: '50%', position: 'top' }} />
  <ReferenceLine x={findBucketByPercentile(data.buckets, data.percentiles.p85)}
    stroke="#f59e0b" label={{ value: '85%', position: 'top' }} />
  <ReferenceLine x={findBucketByPercentile(data.buckets, data.percentiles.p95)}
    stroke="#ef4444" label={{ value: '95%', position: 'top' }} />
</BarChart>
```

### 4.2 颜色

| 桶 | 颜色 |
|---|---|
| 主桶 | `#3b82f6` (blue-500) |
| p50 线 | `#10b981` (green) |
| p85 线 | `#f59e0b` (amber) |
| p95 线 | `#ef4444` (red) |

### 4.3 交互

- 切换桶大小 (1/3/7 天)
- 切换: 显示所有 / 仅已完成 / 仅未完成
- 鼠标悬停: 桶区间 + 计数
- 导出 CSV

---

## 5. 桶自适应算法

```rust
pub fn adaptive_bucket_size(data_count: usize) -> u32 {
    match data_count {
        0..=49 => 1,    // 1 天桶
        50..=499 => 3,  // 3 天桶
        _ => 7,         // 7 天桶
    }
}
```

per 需求 §3.7: 桶选择自适应, 50 以下 1 天桶, 50+ 3 天桶, 500+ 7 天桶。

---

## 6. 边界与异常

| 边界 | 处理 |
|---|---|
| 0 数据 | 仅画 X 轴, "无数据" |
| 所有 cycle_time 相同 | 单一桶, 显示 "所有周期时间 = X 天" |
| 异常长 cycle_time (> 365d) | 单独 "30d+" 桶, 标 ⚠ |
| 1 个 issue | 单一桶, 仅显示该 issue 周期 |

---

## 7. 性能

- 数据量: 90 天 × 10-50 issue/天 = 900-4500 点
- 直方图分桶后: 通常 5-20 个桶
- 性能预算: < 300ms query, < 500ms render

---

## 8. 测试用例

```rust
#[test]
fn test_cycle_time_buckets() {
    let cycle_times = vec![/* 50 issues */];
    let data = generate_cycle_time(&cycle_times, &ChartConfig::default()).unwrap();
    // bucket_size = 1, 至少 1 个桶
}

#[test]
fn test_adaptive_bucket_size() {
    assert_eq!(adaptive_bucket_size(10), 1);
    assert_eq!(adaptive_bucket_size(100), 3);
    assert_eq!(adaptive_bucket_size(1000), 7);
}

#[test]
fn test_percentile_values() {
    let values: Vec<f64> = (1..=100).map(|i| i as f64).collect();
    let data = generate_cycle_time(&values, &ChartConfig::default()).unwrap();
    assert_eq!(data.percentiles.p50, 50.5);  // linear interp
    assert_eq!(data.percentiles.p85, 85.15);
    assert_eq!(data.percentiles.p95, 95.05);
}
```

---

## 9. i18n

```json
{
  "chart.c07.title": "周期时间报告",
  "chart.c07.x_axis": "周期时间",
  "chart.c07.y_axis": "Issue 数",
  "chart.c07.bucket.days": "天",
  "chart.c07.percentile.p50": "50% 在 {n} 天内完成",
  "chart.c07.percentile.p85": "85% 在 {n} 天内完成",
  "chart.c07.percentile.p95": "95% 在 {n} 天内完成"
}
```

---

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 | 2026-09-02 10:04 JST |
