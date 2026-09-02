# C01 Burndown Chart 详细设计

> **状态**: Draft v1.0 (2026-09-02)
> **对标**: [Jira Burndown Chart](https://support.atlassian.com/jira-software-cloud/docs/view-and-understand-the-burndown-chart/)
> **需求基线**: [docs/requirements/charts-and-reports.md §3.1](../../requirements/charts-and-reports.md#31-c01--burndown-chart)
> **基本设计**: [docs/basic-design/charts-and-reports.md §5.3](../../basic-design/charts-and-reports.md#53-22-图表-data-schema-ts-模板) (TimeSeriesData 模板)
> **Spec**: [docs/specs/domain-report-spec.md v1.0 §7 (P0 #1)](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现路径**:
> - 后端: `crates/domain-report/src/charts/c01_burndown.rs`
> - 前端: `frontend/src/components/charts/Chart01Burndown.tsx`
> **估算工期**: 2d (P0 第 1 批)

---

## 0. 文档说明

本文档是 C01 Burndown Chart 的**生产级详细设计**,覆盖数据查询、渲染、配置、边界、性能、测试。

---

## 1. 业务定义

### 1.1 用途

Sprint 期内,**剩余 Story Points (或剩余 issue 数)随时间的下降趋势**。
- 理想线 vs 实际线对比
- 反映 Sprint 进度健康度
- 支持 scope change (范围调整) 可视化

### 1.2 用户场景

- **Scrum Master**: 每日查看 Sprint 进度, 提前识别风险
- **团队成员**: 看到自己贡献的进展
- **PM / Stakeholder**: 评估 Sprint 能否按期完成

### 1.3 何时显示

- 任何 active 或已结束 ≤ 30 天的 Sprint
- 不显示: 计划中或已结束 > 30 天的 Sprint

---

## 2. 数据源

### 2.1 实体 & 字段

| 实体 | 来源 | 关键字段 |
|---|---|---|
| `Sprint` | domain-planning | `sprint_id`, `start_date`, `end_date`, `total_sp`, `scope_change_log[]` |
| `WorkItem` | domain-work-item | `workitem_id`, `sprint_id`, `story_points`, `completed_at`, `in_progress_at` |

### 2.2 SQL 查询模板 (PostgreSQL)

```sql
-- 1. Sprint 元数据
SELECT
    s.sprint_id, s.tenant_id, s.project_id,
    s.start_date, s.end_date, s.total_sp,
    s.scope_change_log  -- JSONB: [{at, delta_sp, reason}]
FROM sprint s
WHERE s.sprint_id = $1
  AND s.tenant_id = $2
  AND s.is_current = TRUE;

-- 2. 每日完成 SP (聚合)
SELECT
    date_trunc('day', wi.completed_at AT TIME ZONE 'UTC') AS day,
    SUM(wi.story_points) AS completed_sp,
    COUNT(*) AS completed_count
FROM work_item wi
WHERE wi.sprint_id = $1
  AND wi.tenant_id = $2
  AND wi.completed_at IS NOT NULL
GROUP BY day
ORDER BY day ASC;

-- 3. Scope change 时间线
SELECT
    (jsonb_array_elements(s.scope_change_log)->>'at')::timestamptz AS change_at,
    (jsonb_array_elements(s.scope_change_log)->>'delta_sp')::integer AS delta_sp
FROM sprint s
WHERE s.sprint_id = $1;
```

### 2.3 数据延迟

- 5min Redis TTL (per INV-REPORT-02)
- work-item 写操作触发 cache invalidate (per basic-design §7.1)

---

## 3. 数据 Schema (TS 详细)

```typescript
// frontend/src/lib/chart-data-schema.ts (Burndown 专用)

export interface BurndownData {
  sprint: {
    sprint_id: string;
    name: string;
    start_date: string;       // ISO 8601
    end_date: string;
    total_sp: number;         // 初始范围 SP
    working_days: string[];   // 排除周末
  };
  series: {
    ideal: TimeSeriesPoint[];  // 理想线
    actual: TimeSeriesPoint[]; // 实际线
  };
  scope_changes: ScopeChange[];
  summary: {
    remaining_sp: number;
    completed_sp: number;
    completed_issues: number;
    total_issues: number;
    predicted_completion_sp: number;  // 预测最终完成 SP
    on_track: boolean;                // 是否按计划
  };
}

export interface TimeSeriesPoint {
  x: string;                  // ISO date "2026-09-02"
  y: number;                  // 剩余 SP
}

export interface ScopeChange {
  at: string;                 // ISO datetime
  delta_sp: number;           // 正数=增加, 负数=减少
  reason: string;
  new_total_sp: number;
}
```

### 3.1 字段含义

| 字段 | 单位 | 取值范围 | 说明 |
|---|---|---|---|
| `sprint.start_date` | ISO date | ≥ today - 90d | Sprint 开始日期 |
| `sprint.end_date` | ISO date | ≤ today + 30d | Sprint 结束日期 |
| `sprint.total_sp` | SP | 0-10000 | 初始总范围 (后续被 scope_change 修改) |
| `series.ideal[].y` | SP | 0-total_sp | 当日理论剩余 SP |
| `series.actual[].y` | SP | 0-total_sp | 当日实际剩余 SP (含 scope 调整) |
| `scope_changes[].delta_sp` | SP | -1000 ~ +1000 | 范围调整量 |
| `summary.predicted_completion_sp` | SP | 0-total_sp*1.5 | 线性外推 |
| `summary.on_track` | boolean | - | actual[-1] ≤ ideal[-1] * 1.1 |

---

## 4. 渲染逻辑

### 4.1 Recharts 组件选择

| 元素 | 组件 | 理由 |
|---|---|---|
| 整体 | `<ResponsiveContainer>` | 自适应宽度 |
| 理想线 | `<Line type="monotone" strokeDasharray="5 5">` | 虚线区分 |
| 实际线 | `<Line type="monotone">` | 实线 + 圆点 |
| Scope change 标记 | `<ReferenceLine strokeDasharray="3 3">` (垂直) | 与 X 轴交点 |
| Y 轴 | `<YAxis domain={[0, 'dataMax + 10%']}>` | 留白 |
| X 轴 | `<XAxis dataKey="x" tickFormatter={formatDay}>` | 日期格式化 |
| Tooltip | `<Tooltip content={<CustomTooltip />}>` | 自定义多行 |
| Legend | `<Legend>` | 切换显示 |

### 4.2 完整渲染代码骨架

```tsx
// frontend/src/components/charts/Chart01Burndown.tsx
'use client';

import { ResponsiveContainer, LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ReferenceLine } from 'recharts';
import { BurndownData } from '@/lib/chart-data-schema';

export function Chart01Burndown({ data, config }: { data: BurndownData; config: ChartConfig }) {
  const merged = mergeSeries(data);  // ideal + actual 同 x 对齐
  const { y_axis_unit } = config;     // 'sp' | 'issue_count'

  return (
    <ResponsiveContainer width="100%" height={400}>
      <LineChart data={merged} margin={{ top: 20, right: 30, left: 20, bottom: 20 }}>
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis
          dataKey="x"
          tickFormatter={(d) => formatDay(d, config.locale)}
          label={{ value: t('chart.c01.x_axis'), position: 'insideBottom', offset: -10 }}
        />
        <YAxis
          label={{ value: t(`chart.c01.y_axis.${y_axis_unit}`), angle: -90, position: 'insideLeft' }}
          domain={[0, (dataMax: number) => Math.ceil(dataMax * 1.1)]}
        />
        <Tooltip content={<BurndownTooltip unit={y_axis_unit} />} />
        <Legend />
        {config.show_ideal_line && (
          <Line
            type="monotone"
            dataKey="ideal"
            name={t('chart.c01.series.ideal')}
            stroke="#94a3b8"
            strokeDasharray="5 5"
            dot={false}
          />
        )}
        <Line
          type="monotone"
          dataKey="actual"
          name={t('chart.c01.series.actual')}
          stroke="#3b82f6"
          strokeWidth={2}
          dot={{ r: 4 }}
          activeDot={{ r: 6 }}
        />
        {data.scope_changes.map((sc, i) => (
          <ReferenceLine
            key={i}
            x={sc.at.split('T')[0]}
            stroke="#f59e0b"
            strokeDasharray="3 3"
            label={{ value: `±${sc.delta_sp}`, position: 'top', fill: '#f59e0b' }}
          />
        ))}
        <ReferenceLine
          x={data.sprint.end_date.split('T')[0]}
          stroke="#ef4444"
          label={{ value: t('chart.c01.sprint_end'), position: 'top' }}
        />
      </LineChart>
    </ResponsiveContainer>
  );
}

function mergeSeries(data: BurndownData) {
  // 合并 ideal + actual 到同一数组, Recharts 要求
  const allDays = new Set([
    ...data.series.ideal.map((p) => p.x),
    ...data.series.actual.map((p) => p.x),
  ]);
  return Array.from(allDays)
    .sort()
    .map((x) => ({
      x,
      ideal: data.series.ideal.find((p) => p.x === x)?.y ?? null,
      actual: data.series.actual.find((p) => p.x === x)?.y ?? null,
    }));
}
```

### 4.3 颜色 / 主题

| 元素 | 浅色主题 | 深色主题 | 色盲友好 (Protanopia) |
|---|---|---|---|
| Ideal line | `#94a3b8` (slate-400) | `#64748b` | 同 (虚线区分) |
| Actual line | `#3b82f6` (blue-500) | `#60a5fa` | `#f59e0b` (amber) |
| Scope change | `#f59e0b` (amber-500) | `#fbbf24` | `#10b981` (emerald) |
| Sprint end | `#ef4444` (red-500) | `#f87171` | `#8b5cf6` (violet) |

> 颜色对色盲友好参考: [Wong palette](https://www.nature.com/articles/nmeth.1618)

### 4.4 交互

| 交互 | 行为 |
|---|---|
| 悬停 tooltip | 显示: 日期 / 理想 SP / 实际 SP / 当日完成 SP / 范围变化 |
| 点击数据点 | 跳转当日 issue 列表 (filter by date) |
| 切换 Y 轴单位 | sp ↔ issue count (config.y_axis_unit) |
| 切换 series | 显示/隐藏 ideal/actual (config.show_ideal_line) |
| 缩放 | Recharts Brush 组件, 限定 [sprint.start, sprint.end+30d] |
| 导出 | 按钮组 (CSV / PNG / PDF) |

### 4.5 自定义 Tooltip

```tsx
function BurndownTooltip({ active, payload, unit }: any) {
  if (!active || !payload?.length) return null;
  const point = payload[0].payload;
  return (
    <div className="rounded border bg-white p-2 shadow dark:bg-zinc-800">
      <div className="font-semibold">{point.x}</div>
      <div className="text-sm">
        <span className="text-slate-500">理想: </span>
        {point.ideal} {unit === 'sp' ? 'SP' : 'issues'}
      </div>
      <div className="text-sm">
        <span className="text-blue-500">实际: </span>
        {point.actual} {unit === 'sp' ? 'SP' : 'issues'}
      </div>
    </div>
  );
}
```

---

## 5. 配置项 (ChartConfig 字段映射)

| 字段 | 类型 | 默认值 | 含义 |
|---|---|---|---|
| `time_range` | TimeRange | `{mode: 'ThisSprint'}` | 仅 Sprint 期 |
| `y_axis_unit` | YAxisUnit | `'sp'` | `'sp'` / `'issue_count'` |
| `show_ideal_line` | bool | `true` | 是否显示理想线 |
| `show_scope_changes` | bool | `true` | 是否标记 scope change |
| `show_sprint_end_marker` | bool | `true` | 是否标记 end_date |
| `log_scale` | bool | `false` | Y 轴 log (一般不用) |
| `color_scheme` | ColorScheme | `'default'` | 色盲切换 |
| `locale` | Locale | `'en-US'` | 影响日期/数字格式 |

---

## 6. 边界与异常

| 边界 | 处理 |
|---|---|
| Sprint 未开始 | 仅显示 ideal 线, actual 全 null, 文案 "Sprint 尚未开始" |
| Sprint 已结束 > 30d | 不显示 (前端 filter), 服务端返回 404 |
| total_sp = 0 | 显示 "无规划范围" 提示, 隐藏线条 |
| 所有 issue 已完成 (Sprint 末) | ideal 终点 = 0, actual 终点 = 0 |
| 实际完成 > 范围 (scope 增加) | actual 线反映 total_sp 调整, 不画负值 |
| 周末/节假日 | ideal 线跳过非工作日 (per `sprint.working_days`) |
| Scope change > 10 次 | 仅显示最近 10 次 (per 需求 §3.1) |
| 未来日期已写入 completed_at | 数据异常, 显示警告 + Audit 记录 |
| 多个 Sprint 并行 | 取 sprint_id 显式指定的 Sprint, 不推断 |
| Cache miss + DB 慢 | 异步生成 + 显示 "loading" 状态 (≤ 3s) |

---

## 7. 性能

### 7.1 数据量

- 单 Sprint issue 数: 通常 < 200, 极少 > 1000
- 时间点: Sprint 平均 14 天 = 14 个点
- 总体: < 500 数据点 (Recharts 性能充裕)

### 7.2 性能预算

| 阶段 | 预算 |
|---|---|
| SQL 查询 | < 200ms (含聚合) |
| Redis 命中 | < 50ms |
| Recharts 渲染 (FCP) | < 500ms |
| 切换 Y 轴单位 | < 100ms (纯前端) |

### 7.3 采样策略

- 数据点 < 1000: 不采样
- 数据点 ≥ 1000: 不适用 (Burndown 不会出现)
- > 10K: 服务端拒绝 (per INV-REPORT-10)

---

## 8. 测试用例

### 8.1 单元测试 (Rust)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_burndown_normal_case() {
        let sprint = Sprint { total_sp: 100, start_date: today(), end_date: today() + 14, ..default() };
        let issues = vec![
            issue_with(20.0, today() + 1),
            issue_with(30.0, today() + 3),
            issue_with(10.0, today() + 5),
        ];
        let data = generate_burndown(&sprint, &issues, &ChartConfig::default()).await.unwrap();
        assert_eq!(data.series.actual.len(), 3);
        assert_eq!(data.summary.remaining_sp, 40.0);
        assert!(data.summary.on_track);
    }

    #[tokio::test]
    async fn test_burndown_with_scope_change() {
        // ...
    }

    #[tokio::test]
    async fn test_burndown_zero_total_sp() {
        let sprint = Sprint { total_sp: 0, ..default() };
        let data = generate_burndown(&sprint, &[], &ChartConfig::default()).await;
        assert!(data.is_ok());  // 边界 case 不报错
    }

    #[tokio::test]
    async fn test_burndown_cache_invalidation() {
        // ...
    }

    #[tokio::test]
    async fn test_burndown_rls_enforced() {
        // 跨租户访问应失败
    }
}
```

### 8.2 集成测试 (Frontend)

```typescript
// Chart01Burndown.test.tsx
import { render, screen } from '@testing-library/react';
import { Chart01Burndown } from './Chart01Burndown';

const mockData: BurndownData = {
  sprint: { sprint_id: 's1', name: 'Sprint 1', start_date: '2026-09-01', end_date: '2026-09-15', total_sp: 100, working_days: [...] },
  series: {
    ideal: [{ x: '2026-09-01', y: 100 }, { x: '2026-09-15', y: 0 }],
    actual: [{ x: '2026-09-01', y: 100 }, { x: '2026-09-03', y: 70 }],
  },
  scope_changes: [{ at: '2026-09-05T10:00:00Z', delta_sp: -20, reason: 'Removed story', new_total_sp: 80 }],
  summary: { remaining_sp: 70, completed_sp: 30, completed_issues: 3, total_issues: 10, predicted_completion_sp: 90, on_track: true },
};

test('renders ideal and actual lines', () => {
  render(<Chart01Burndown data={mockData} config={{ y_axis_unit: 'sp', show_ideal_line: true, ... }} />);
  // 验证 Line 组件被渲染, 颜色正确
});

test('toggles y-axis unit', () => {
  // ...
});

test('handles zero total_sp', () => {
  // ...
});
```

### 8.3 E2E (Playwright)

```typescript
// e2e/c01-burndown.spec.ts
test('User can view burndown for active sprint', async ({ page }) => {
  await page.goto('/reports/c01-burndown');
  await page.waitForSelector('[data-testid="chart-c01-burndown"]');
  await expect(page.getByText('Sprint 1')).toBeVisible();
  // 验证 ideal + actual 线渲染
});

test('Scope change marker visible', async ({ page }) => {
  // ...
});
```

---

## 9. 国际化 (i18n)

### 9.1 文本资源 key

```json
// frontend/src/i18n/zh-CN.json
{
  "chart.c01.title": "燃尽图",
  "chart.c01.x_axis": "日期",
  "chart.c01.y_axis.sp": "剩余 SP",
  "chart.c01.y_axis.issue_count": "剩余 issue 数",
  "chart.c01.series.ideal": "理想",
  "chart.c01.series.actual": "实际",
  "chart.c01.sprint_end": "Sprint 结束",
  "chart.c01.empty.no_sprint": "Sprint 尚未开始",
  "chart.c01.empty.zero_sp": "无规划范围",
  "chart.c01.tooltip.ideal": "理想",
  "chart.c01.tooltip.actual": "实际",
  "chart.c01.tooltip.scope_change": "范围调整",
  "chart.c01.export.csv": "导出 CSV",
  "chart.c01.export.png": "导出 PNG",
  "chart.c01.export.pdf": "导出 PDF"
}
```

### 9.2 复数处理

- `剩余 {n} SP` (zh-CN 无复数变化)
- `Remaining {n, plural, one {# SP} other {# SPs}}` (en-US)

### 9.3 数字/日期格式

- 跟随 `config.locale`:
  - `zh-CN`: `2026-09-02`
  - `en-US`: `09/02/2026`
  - `ja-JP`: `2026/09/02`
- 通过 `date-fns/locale` 实现

---

## 10. 可访问性 (a11y)

| 元素 | a11y 属性 |
|---|---|
| 整体 | `role="img"` `aria-label={t('chart.c01.title')}` |
| 理想线 | `aria-hidden="true"` (Tooltip 文本已含) |
| 实际线 | 同上 |
| Scope change 标记 | `aria-label={t('chart.c01.tooltip.scope_change')}` |
| Tooltip | 键盘可达 (`tabindex="0"`) |

---

## 11. 与其他图表 / 模块关系

- **C02 Burnup**: 共用 Sprint 元数据, 仅数据点计算不同 (累积 vs 剩余)
- **C03 Velocity**: Burndown × N Sprint, 聚合跨 Sprint
- **C09 Forecast**: 复用 historical Burndown 数据预测
- **domain-dashboard chart-* Gadget**: 引用 C01 即可作为 Gadget 嵌入

---

## 12. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 业务定义 + 数据源 + TS schema + Recharts 渲染 + 配置 + 边界 + 性能 + 测试 + i18n + a11y | 2026-09-02 10:04 JST Ulysses 拍板 "图表对标 Jira" |

---

> **审批者**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-02
> **per AGENTS.md §1.0 用户授权升级 v0.5 + 8/27 19:39/20:56/21:59 JST 三次强化**: Mavis 接手默认代签 Ulysses 无需再问
