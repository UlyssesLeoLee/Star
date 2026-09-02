// frontend/src/components/charts/Chart02Burnup.tsx
'use client';

/**
 * C02 Burnup Chart — Recharts 实现 (per docs/design/charts/c02-burnup.md v1.0)
 * 累积完成 + 范围阶梯 (stepAfter)
 */

import {
  LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend,
  ResponsiveContainer,
} from 'recharts';
import { useTranslation } from '@/i18n/hooks';

interface Point { x: string; y: number }
export interface BurnupData {
  series: { actual: Point[]; scope?: Point[] };
  summary: { completed_sp: number; total_sp: number; completion_ratio: number };
}

export function Chart02Burnup({ data, height = 400 }: { data: BurnupData; height?: number }) {
  const { t } = useTranslation();
  const merged = data.series.actual.map((p, i) => ({
    x: p.x,
    actual: p.y,
    scope: data.series.scope?.[i]?.y ?? null,
  }));

  return (
    <ResponsiveContainer width="100%" height={height} data-testid="chart-c02-burnup">
      <LineChart data={merged} margin={{ top: 20, right: 30, left: 20, bottom: 20 }}>
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis dataKey="x" tick={{ fontSize: 12 }} />
        <YAxis label={{ value: t('chart.c02.y_axis.sp'), angle: -90, position: 'insideLeft', fontSize: 12 }} />
        <Tooltip />
        <Legend />
        <Line type="monotone" dataKey="actual" name={t('chart.c02.series.actual')} stroke="#3b82f6" strokeWidth={2} dot={{ r: 3 }} />
        {data.series.scope && (
          <Line type="stepAfter" dataKey="scope" name={t('chart.c02.series.scope')} stroke="#94a3b8" strokeDasharray="5 5" dot={false} />
        )}
      </LineChart>
    </ResponsiveContainer>
  );
}
