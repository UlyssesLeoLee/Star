// frontend/src/components/charts/Chart06ControlChart.tsx
'use client';

/**
 * C06 Control Chart — Recharts ScatterChart + ReferenceLine (per docs/design/charts/c06-control-chart.md v1.0)
 */

import {
  ScatterChart, Scatter, XAxis, YAxis, CartesianGrid, Tooltip, Legend,
  ReferenceLine, ResponsiveContainer,
} from 'recharts';
import { useTranslation } from '@/i18n/hooks';

interface ControlPoint { key: string; cycle_time_days: number; completed_at: string; anomaly: boolean; z_score: number }
interface RefLine { y_value: number; label: string; style: string }
export interface ControlChartData {
  data_points: ControlPoint[];
  reference_lines: RefLine[];
  stats: { median: number; p70: number; p85: number; p95: number; mean: number; std_dev: number };
}

export function Chart06ControlChart({ data, height = 400 }: { data: ControlChartData; height?: number }) {
  const { t } = useTranslation();
  const normal = data.data_points.filter(p => !p.anomaly).map((p, i) => ({ idx: i, y: p.cycle_time_days, z: p.z_score, key: p.key }));
  const anomaly = data.data_points.filter(p => p.anomaly).map((p, i) => ({ idx: i, y: p.cycle_time_days, z: p.z_score, key: p.key }));

  return (
    <ResponsiveContainer width="100%" height={height} data-testid="chart-c06-control-chart">
      <ScatterChart margin={{ top: 20, right: 30, left: 20, bottom: 20 }}>
        <CartesianGrid />
        <XAxis type="number" dataKey="idx" name={t('chart.c06.x_axis')} />
        <YAxis type="number" dataKey="y" name={t('chart.c06.y_axis')} />
        <Tooltip cursor={{ strokeDasharray: '3 3' }} />
        <Legend />
        <Scatter name={t('chart.c06.series.normal')} data={normal} fill="#3b82f6" />
        <Scatter name={t('chart.c06.series.anomaly')} data={anomaly} fill="#ef4444" />
        {data.reference_lines.map((line, i) => (
          <ReferenceLine
            key={i}
            y={line.y_value}
            stroke="#94a3b8"
            strokeDasharray={line.style === 'dashed' ? '5 5' : line.style === 'dotted' ? '2 2' : ''}
            label={{ value: line.label, position: 'right' }}
          />
        ))}
      </ScatterChart>
    </ResponsiveContainer>
  );
}
