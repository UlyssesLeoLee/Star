# C09 Forecast Chart 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira Forecast (官方未提供, V1 创新)](https://support.atlassian.com/jira-software-cloud/docs/) | **需求**: [§3.9](../../requirements/charts-and-reports.md#39-c09--forecast-chart) | **Spec**: [P1 #10](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c09_forecast.rs` + `frontend/src/components/charts/Chart09Forecast.tsx`
> **工期**: 2d

---

## 1. 业务定义

基于历史 Velocity / Throughput, **预测 Sprint 或项目完成日期**, 含 80%/95% 置信带。

---

## 2. 数据源

| 实体 | 字段 |
|---|---|
| `Sprint` | `sprint_id, completed_sp, committed_sp, start_date, end_date, status` |
| `WorkItem` (current sprint) | `sprint_id, remaining_sp (sum)` |

## 3. 预测方法

### 3.1 简单平均 (default)

```
predicted_velocity = mean(last N completed sprints)
predicted_completion_date = today + remaining_sp / predicted_velocity * sprint_duration
```

### 3.2 滚动平均

```
predicted_velocity = mean(last 3 completed sprints)
```

### 3.3 线性回归

```python
# 简单线性回归 y = ax + b
x = [1, 2, 3, ..., N]  # sprint index
y = [sprint.completed_sp for sprint in last_N_sprints]
slope, intercept = linear_regression(x, y)
predicted_velocity = slope * (N + 1) + intercept
```

### 3.4 置信带 (80% / 95%)

```python
std_dev = std(last_N completed sprints)
confidence_80 = predicted_velocity ± 1.28 * std_dev / sqrt(N)  # z-score 80%
confidence_95 = predicted_velocity ± 1.96 * std_dev / sqrt(N)  # z-score 95%
```

---

## 4. 数据 Schema (TS)

```typescript
export interface ForecastData {
  historical: {
    sprints: Array<{ name: string; completed_sp: number; committed_sp: number }>;
    avg_velocity: number;
    std_dev: number;
  };
  forecast: {
    method: 'simple_avg' | 'rolling_avg' | 'linear_regression';
    predicted_velocity: number;
    confidence_80: [number, number];  // [low, high]
    confidence_95: [number, number];
    predicted_completion_date: string;  // ISO
    remaining_sprints: number;
  };
  current: {
    sprint_name: string;
    remaining_sp: number;
    end_date: string;
  };
}
```

---

## 5. 渲染逻辑

```tsx
<ComposedChart data={mergedData}>
  <CartesianGrid strokeDasharray="3 3" />
  <XAxis type="number" dataKey="sprint_index" />
  <YAxis label={{ value: 'SP', angle: -90 }} />
  <Tooltip />
  <Legend />
  <Bar dataKey="completed_sp" name="Historical" fill="#3b82f6" />
  <Line type="monotone" dataKey="predicted" name="Forecast" stroke="#f59e0b" strokeDasharray="5 5" />
  {/* 95% confidence band */}
  <Area type="monotone" dataKey="upper_95" name="95% Upper" fill="#f59e0b" fillOpacity={0.1} stroke="none" />
  <Area type="monotone" dataKey="lower_95" name="95% Lower" fill="#f59e0b" fillOpacity={0.1} stroke="none" />
</ComposedChart>
```

## 6. 配置项

| 字段 | 默认 | 含义 |
|---|---|---|
| `method` | `'rolling_avg'` | 预测方法 |
| `history_window` | `6` | 用最近 N 个 Sprint |
| `confidence_levels` | `[80, 95]` | 显示哪些置信带 |
| `target_sprint_id` | (current) | 目标 Sprint |

## 7. 边界与异常

| 边界 | 处理 |
|---|---|
| < 3 历史 Sprint (per INV-REPORT-15) | 不预测, 显示 "需要更多历史" |
| 0 remaining_sp | 完成日期 = today |
| 预测日期 > Sprint.end_date + 30d | ⚠ 警告, 大概率延期 |
| std_dev > avg | 置信带过宽, 提示 "数据波动大, 预测不可靠" |

## 8. 性能

- 6 Sprint × 3 methods = 18 预测计算
- 性能预算: < 300ms

## 9. 测试

```rust
#[test]
fn test_forecast_min_3_sprints() {
    // < 3 应返回 Err(NeedMoreHistory)
}

#[test]
fn test_simple_avg_prediction() {
    // 历史 30, 40, 50 SP, avg = 40, remaining 80, 预测 2 Sprint
}

#[test]
fn test_linear_regression() {
    // y = 2x + 10, 验证斜率/截距
}

#[test]
fn test_confidence_bands() {
    // std_dev = 5, N = 6, 80% = ±2.6
}
```

## 10. i18n

```json
{
  "chart.c09.title": "预测",
  "chart.c09.method.simple_avg": "简单平均",
  "chart.c09.method.rolling_avg": "滚动平均",
  "chart.c09.method.linear_regression": "线性回归",
  "chart.c09.predicted_completion": "预计完成: {date}",
  "chart.c09.confidence_80": "80% 置信区间",
  "chart.c09.confidence_95": "95% 置信区间",
  "chart.c09.empty.less_than_3": "至少需要 3 个已完成 Sprint",
  "chart.c09.warning.high_variance": "数据波动大, 预测不可靠"
}
```

## 11. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 (含 3 种预测方法 + 置信带算法) | 2026-09-02 10:04 JST |
