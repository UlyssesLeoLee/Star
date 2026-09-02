// frontend/src/components/charts/Chart07CycleTime.tsx
'use client';

/**
 * C07 Cycle Time Chart — Recharts BarChart + ReferenceLine (per docs/design/charts/c07-cycle-time.md v1.0)
 * 直方图 + 50/85/95 百分位线
 */

import {
  BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend,
  ReferenceLine, ResponsiveContainer,
} from 'recharts';
import { useTranslation } from '@/i18n/hooks';

interface Bucket { range_start: number; range_end: number; count: number; label: string }
export interface CycleTimeData {
  buckets: Bucket[];
  percentiles: { p50: number; p85: number; p95: number };
}

export function Chart07CycleTime({ data, height = 400 }: { data: CycleTimeData; height?: number }) {
  const { t } = useTranslation();
  return (
    <ResponsiveContainer width="100%" height={height} data-testid="chart-c07-cycle-time">
      <BarChart data={data.buckets}>
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis dataKey="label" />
        <YAxis label={{ value: t('chart.c07.y_axis'), angle: -90, position: 'insideLeft' }} />
        <Tooltip />
        <Legend />
        <Bar dataKey="count" name={t('chart.c07.y_axis')} fill="#3b82f6" />
        <ReferenceLine x={findBucketLabel(data, data.percentiles.p50)} stroke="#10b981" label={{ value: '50%', position: 'top' }} />
        <ReferenceLine x={findBucketLabel(data, data.percentiles.p85)} stroke="#f59e0b" label={{ value: '85%', position: 'top' }} />
        <ReferenceLine x={findBucketLabel(data, data.percentiles.p95)} stroke="#ef4444" label={{ value: '95%', position: 'top' }} />
      </BarChart>
    </ResponsiveContainer>
  );
}

function findBucketLabel(data: CycleTimeData, value: number): string {
  const bucket = data.buckets.find(b => value >= b.range_start && value < b.range_end);
  return bucket?.label || '';
}
