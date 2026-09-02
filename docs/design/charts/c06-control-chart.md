# C06 Control Chart 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira Control Chart](https://support.atlassian.com/jira-software-cloud/docs/view-the-control-chart/) | **需求**: [§3.6](../../requirements/charts-and-reports.md#36-c06--control-chart) | **Spec**: [P0 #6](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c06_control_chart.rs` + `frontend/src/components/charts/Chart06ControlChart.tsx`
> **工期**: 3d (高风险, 异常检测算法)

---

## 1. 业务定义

**每个 issue 完成时的"周期时间"散点图, 叠加 ±3σ 控制线检测异常**。识别流程异常, 改进预测。

---

## 2. 数据源

| 实体 | 字段 |
|---|---|
| `WorkItem` | `workitem_id, key, completed_at, first_in_progress_at, cycle_time` |

**SQL**:
```sql
SELECT
    workitem_id, key, title,
    EXTRACT(EPOCH FROM (completed_at - first_in_progress_at)) / 86400.0 AS cycle_time_days,
    completed_at
FROM work_item
WHERE tenant_id = $1
  AND completed_at IS NOT NULL
  AND first_in_progress_at IS NOT NULL
  AND completed_at >= $start_date
  AND completed_at <= $end_date
ORDER BY completed_at;
```

---

## 3. 数据 Schema (TS)

```typescript
export interface ControlChartData {
  data_points: Array<{
    workitem_id: string;
    key: string;             // "PROJ-123"
    title: string;
    cycle_time_days: number;
    completed_at: string;     // ISO
    anomaly: boolean;        // 是否超 ±3σ
    z_score: number;         // 标准分数
  }>;
  reference_lines: Array<{
    y_value: number;
    label: string;           // "Median", "70%", "85%", "95%", "+3σ", "-3σ"
    style: 'solid' | 'dashed' | 'dotted';
  }>;
  stats: {
    median: number;
    p70: number;
    p85: number;
    p95: number;
    mean: number;
    std_dev: number;
  };
}
```

---

## 4. 渲染逻辑

### 4.1 Recharts 组件

| 元素 | 组件 | 备注 |
|---|---|---|
| 散点 | `<ScatterChart>` | 主图 |
| 正常点 | `<Scatter fill="#3b82f6">` | 蓝色 |
| 异常点 | `<Scatter fill="#ef4444">` | 红色高亮 |
| Median / 百分位 | `<ReferenceLine>` | 水平 |
| ±3σ | `<ReferenceLine stroke="#ef4444" strokeDasharray="5 5">` | 水平 |

```tsx
<ResponsiveContainer>
  <ScatterChart margin={{ top: 20, right: 30, left: 20, bottom: 20 }}>
    <CartesianGrid />
    <XAxis
      type="number"
      dataKey="completed_at"
      domain={['dataMin', 'dataMax']}
      tickFormatter={(d) => formatDate(d, config.locale)}
      name={t('chart.c06.x_axis')}
    />
    <YAxis
      type="number"
      dataKey="cycle_time_days"
      scale={config.log_scale ? 'log' : 'linear'}
      domain={[0, 'dataMax + 10%']}
      label={{ value: t('chart.c06.y_axis'), angle: -90 }}
    />
    <Tooltip content={<ControlChartTooltip />} />
    <Scatter
      name="Normal"
      data={normalPoints}
      fill="#3b82f6"
      onClick={(p) => router.push(`/issues/${p.key}`)}
    />
    <Scatter name="Anomaly" data={anomalyPoints} fill="#ef4444" />
    {data.reference_lines.map(line => (
      <ReferenceLine
        y={line.y_value}
        stroke="#94a3b8"
        strokeDasharray={line.style === 'dashed' ? '5 5' : line.style === 'dotted' ? '2 2' : ''}
        label={{ value: line.label, position: 'right' }}
      />
    ))}
  </ScatterChart>
</ResponsiveContainer>
```

### 4.2 颜色

| 元素 | 浅色 | 深色 |
|---|---|---|
| 正常点 | `#3b82f6` | `#60a5fa` |
| 异常点 (超 3σ) | `#ef4444` | `#f87171` |
| Median | `#94a3b8` | `#64748b` |
| 百分位 (70/85/95) | `#cbd5e1` | `#475569` |
| ±3σ 控制线 | `#ef4444` (虚线) | `#f87171` |

### 4.3 交互

| 交互 | 行为 |
|---|---|
| 悬停正常点 | tooltip: key / 周期 / 完成日 |
| 悬停异常点 | tooltip + z-score + ⚠ 警告 |
| 点击点 | 跳转 issue detail |
| 切换 log/linear | Y 轴切换 |
| 切换 series | 显示/隐藏异常点 |
| 缩放 | Brush |

---

## 5. 异常检测算法 (核心)

### 5.1 算法: Modified Z-Score (Iglewicz-Hoaglin)

不用传统 Z-Score (受异常值影响大), 采用 **Modified Z-Score**:

```
M_i = 0.6745 * (x_i - median) / MAD
```

其中 `MAD = median(|x_i - median|)`。

**判定**: `|M_i| > 3.5` 视为异常 (per Iglewicz & Hoaglin 1993)。

```rust
pub fn detect_anomalies(cycle_times: &[f64]) -> Vec<AnomalyResult> {
    let n = cycle_times.len();
    if n < 10 {
        return vec![AnomalyResult::default(); n];  // INV-REPORT-14: < 10 不画控制线
    }

    let median = percentile(cycle_times, 50.0);
    let deviations: Vec<f64> = cycle_times.iter().map(|x| (x - median).abs()).collect();
    let mad = percentile(&deviations, 50.0);

    if mad == 0.0 {
        return vec![AnomalyResult::default(); n];  // 所有值相同
    }

    cycle_times.iter().map(|&x| {
        let m = 0.6745 * (x - median) / mad;
        AnomalyResult {
            z_score: m,
            anomaly: m.abs() > 3.5,
        }
    }).collect()
}

fn percentile(values: &[f64], p: f64) -> f64 {
    // numpy.percentile linear interpolation
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let rank = (p / 100.0) * (sorted.len() as f64 - 1.0);
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    if lower == upper {
        sorted[lower]
    } else {
        sorted[lower] + (rank - lower as f64) * (sorted[upper] - sorted[lower])
    }
}
```

### 5.2 百分位计算

70% / 85% / 95% 走 `percentile` 函数 (linear interpolation, per numpy)。

---

## 6. 配置项

| 字段 | 默认 | 含义 |
|---|---|---|
| `time_range` | `{mode: 'LastNDays', n_days: 90}` | 时间窗 |
| `anomaly_threshold` | `3.5` | Modified Z 阈值 (Iglewicz-Hoaglin) |
| `show_anomaly_only` | `false` | 仅显示异常点 |
| `log_scale` | `false` | Y 轴 log |
| `percentiles` | `[50, 70, 85, 95]` | 显示哪些百分位线 |
| `min_data_points` | `10` | INV-REPORT-14 阈值 |

---

## 7. 边界与异常

| 边界 | 处理 |
|---|---|
| < 10 数据点 | 不画控制线, 仅画散点, 提示 "需要更多数据" |
| 所有 cycle_time 相同 | MAD=0, 不画控制线 |
| 异常值过多 (> 20%) | 提示 "数据分布异常, 请检查" |
| cycle_time < 0 | 数据异常, 排除 (理论上不可能) |
| cycle_time > 365 天 | 显示但加 ⚠ 提示, 可能是 stale issue |
| 缺失 first_in_progress_at | 排除 (无法计算 cycle_time) |

---

## 8. 性能

- 数据点: 90 天 × 10-50 issue/天 = 900-4500 点
- 异常检测: O(n log n), 极快
- Recharts Scatter: < 5000 点性能良好
- 性能预算: < 500ms query, < 800ms render

---

## 9. 测试用例

```rust
#[test]
fn test_modified_z_score_normal() {
    let cycle_times = vec![3.0, 4.0, 5.0, 3.5, 4.5, 5.5, 3.2, 4.8, 5.2, 4.0];
    let results = detect_anomalies(&cycle_times);
    // 所有 |M| < 3.5, 不应异常
    assert!(results.iter().all(|r| !r.anomaly));
}

#[test]
fn test_modified_z_score_outlier() {
    let mut cycle_times = vec![3.0; 20];
    cycle_times.push(100.0);  // 明显异常
    let results = detect_anomalies(&cycle_times);
    assert!(results.last().unwrap().anomaly);
}

#[test]
fn test_control_chart_min_10_points() {
    // < 10 点不画控制线
    let cycle_times = vec![3.0, 4.0, 5.0];
    let results = detect_anomalies(&cycle_times);
    assert!(results.iter().all(|r| r.z_score == 0.0));
}

#[test]
fn test_percentile_linear_interp() {
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    assert_eq!(percentile(&values, 50.0), 3.0);
    assert_eq!(percentile(&values, 25.0), 2.0);
    assert_eq!(percentile(&values, 75.0), 4.0);
}
```

---

## 10. i18n

```json
{
  "chart.c06.title": "控制图",
  "chart.c06.x_axis": "完成日期",
  "chart.c06.y_axis": "周期时间 (天)",
  "chart.c06.reference.median": "中位",
  "chart.c06.reference.p70": "70%",
  "chart.c06.reference.p85": "85%",
  "chart.c06.reference.p95": "95%",
  "chart.c06.reference.plus_3sigma": "+3σ",
  "chart.c06.reference.minus_3sigma": "-3σ",
  "chart.c06.tooltip.anomaly": "⚠ 异常 (z={z})",
  "chart.c06.empty.less_than_10": "至少需要 10 个完成 issue"
}
```

---

## 11. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 (含 Modified Z-Score 算法) | 2026-09-02 10:04 JST |
