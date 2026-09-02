# C21 Heatmap (活跃度) 详细设计

> **状态**: Draft v1.0 (2026-09-02) | **对标**: [Jira Heat Map](https://support.atlassian.com/jira-software-cloud/docs/view-the-heat-map/) | **需求**: [§3.21](../../requirements/charts-and-reports.md#321-c21--heatmap-活跃度) | **Spec**: [P2 #21](../../specs/domain-report-spec.md#8-22-图表实施分批-per-需求-11-risk-chart-01)
> **实现**: `crates/domain-report/src/charts/c21_heatmap.rs` + `frontend/src/components/charts/Chart21Heatmap.tsx`
> **工期**: 3d (Recharts 无原生, 自研 SVG, per RISK-CHART-09)

---

## 1. 业务定义

**周 × 小时 矩阵, 值 = 新建/解决 issue 数, 色阶展示**, 识别团队活跃时段。

---

## 2. 数据源

| 实体 | 字段 |
|---|---|
| `WorkItem` | `created_at, resolved_at, tenant_id, project_id` |

**SQL** (服务端预聚合到 7×24, per INV-REPORT-17):
```sql
-- Created 矩阵
SELECT
    EXTRACT(ISODOW FROM created_at) AS weekday,  -- 1-7
    EXTRACT(HOUR FROM created_at) AS hour,        -- 0-23
    COUNT(*) AS count
FROM work_item
WHERE tenant_id = $1 AND project_id = $2
  AND created_at >= $start
  AND created_at AT TIME ZONE $tz BETWEEN $start AND $end
GROUP BY weekday, hour;
-- 结果填入 7×24 矩阵
```

> 注: SQL 需按 timezone 转换, 用 `AT TIME ZONE 'Asia/Tokyo'` 语法。

## 3. 数据 Schema (TS)

```typescript
export interface HeatmapData {
  x_categories: string[];  // ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'] or ['0', '1', ..., '23']
  y_categories: string[];  // ['Week 1', ..., 'Week N'] or hours
  values: number[][];      // values[y_idx][x_idx]
  color_scale: {
    min: number;
    max: number;
    scheme: 'viridis' | 'blues' | 'custom';
  };
  value_type: 'created' | 'resolved';
  timezone: string;        // IANA
  layout: 'day_hour' | 'week_hour' | 'day_week';
}
```

## 4. 渲染逻辑

> **Recharts 无原生 Heatmap**, 走**自研 SVG** (per RISK-CHART-09)。

```tsx
function Chart21Heatmap({ data }: { data: HeatmapData }) {
  const cellSize = 24;
  const colorScale = useColorScale(data.color_scale);

  return (
    <svg width={(data.x_categories.length + 1) * cellSize} height={(data.y_categories.length + 1) * cellSize}>
      {/* Y 轴标签 */}
      {data.y_categories.map((y, yi) => (
        <text key={y} x={0} y={(yi + 1.5) * cellSize} className="text-xs">
          {y}
        </text>
      ))}
      {/* X 轴标签 */}
      {data.x_categories.map((x, xi) => (
        <text key={x} x={(xi + 1.5) * cellSize} y={cellSize * 0.7} textAnchor="middle" className="text-xs">
          {x}
        </text>
      ))}
      {/* 单元格 */}
      {data.values.map((row, yi) =>
        row.map((v, xi) => (
          <rect
            key={`${yi}-${xi}`}
            x={(xi + 1) * cellSize}
            y={(yi + 1) * cellSize}
            width={cellSize}
            height={cellSize}
            fill={colorScale(v)}
            stroke="#fff"
            strokeWidth={1}
          >
            <title>{`${data.y_categories[yi]} × ${data.x_categories[xi]}: ${v}`}</title>
          </rect>
        ))
      )}
      {/* 图例 */}
      <Legend colorScale={colorScale} min={data.color_scale.min} max={data.color_scale.max} />
    </svg>
  );
}
```

## 5. 色阶算法 (d3-scale 兼容)

```typescript
import { scaleSequential } from 'd3-scale';
import { interpolateBlues, interpolateViridis } from 'd3-scale-chromatic';

const colorScales = {
  viridis: interpolateViridis,
  blues: interpolateBlues,
  custom: (t: number) => `rgba(59, 130, 246, ${t})`,  // 蓝色透明
};

function useColorScale(scale: ColorScale) {
  return (value: number) => {
    const t = (value - scale.min) / (scale.max - scale.min || 1);
    return colorScales[scale.scheme](Math.max(0, Math.min(1, t)));
  };
}
```

## 6. 配置项

| 字段 | 默认 | 含义 |
|---|---|---|
| `layout` | `'day_hour'` | `'day_hour'` / `'week_hour'` / `'day_week'` |
| `value_type` | `'created'` | `'created'` / `'resolved'` |
| `timezone` | `'UTC'` | IANA (用户时区) |
| `time_range` | `{LastNDays: 30}` | 时间窗 |
| `color_scheme` | `'blues'` | 色阶 (含色盲友好 viridis) |

## 7. 边界

| 边界 | 处理 |
|---|---|
| 0 issue | 全 0 矩阵, 浅色 |
| 单小时单天 | 单 cell 高亮 |
| 跨时区 | 按用户时区聚合, 文档化说明 |

## 8. 性能

- 服务端预聚合 7×24 = 168 cell, 极快
- 性能预算: < 100ms query, < 200ms render

### 8.1 d3-scale-chromatic 添加到 package.json

per ask_user tech-stack=A Recharts, Heatmap 自研 SVG **仍需 d3-scale-chromatic** 作为色阶算法:
```json
"dependencies": {
  "d3-scale": "^4.0.2",
  "d3-scale-chromatic": "^3.1.0"
}
```

## 9. 测试

```rust
#[test]
fn test_heatmap_aggregation() {
    // 7×24 = 168 cell 全部填充
}

#[test]
fn test_heatmap_timezone() {
    // UTC vs Asia/Tokyo 转换
}
```

## 10. i18n

```json
{
  "chart.c21.title": "活跃度热力图",
  "chart.c21.value_type.created": "新建",
  "chart.c21.value_type.resolved": "解决",
  "chart.c21.layout.day_hour": "按天 × 小时",
  "chart.c21.layout.week_hour": "按周 × 小时",
  "chart.c21.layout.day_week": "按天 × 周",
  "chart.c21.day.mon": "周一",
  "chart.c21.day.tue": "周二",
  "chart.c21.day.wed": "周三",
  "chart.c21.day.thu": "周四",
  "chart.c21.day.fri": "周五",
  "chart.c21.day.sat": "周六",
  "chart.c21.day.sun": "周日"
}
```

## 11. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版 (自研 SVG + d3-scale-chromatic 色阶) | 2026-09-02 10:04 JST |
