// frontend/src/components/charts/Chart08Throughput.tsx
'use client';

/** C08 Throughput (per docs/design/charts/c08-throughput.md) */
import { ComposedChart, Bar, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import { useTranslation } from '@/i18n/hooks';

interface Bucket { bucket: string; count: number; avg?: number }
export interface ThroughputData { series: Bucket[]; moving_avg: Bucket[]; stats: { total: number; avg: number; std_dev: number } }

export function Chart08Throughput({ data, height = 400 }: { data: ThroughputData; height?: number }) {
  const { t } = useTranslation();
  const merged = data.series.map((s, i) => ({ ...s, avg: data.moving_avg[i]?.avg ?? null }));
  return (
    <ResponsiveContainer width="100%" height={height} data-testid="chart-c08-throughput">
      <ComposedChart data={merged}>
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis dataKey="bucket" />
        <YAxis label={{ value: t('chart.c08.y_axis'), angle: -90, position: 'insideLeft' }} />
        <Tooltip />
        <Legend />
        <Bar dataKey="count" name={t('chart.c08.series.count')} fill="#3b82f6" fillOpacity={0.6} />
        <Line type="monotone" dataKey="avg" name={t('chart.c08.series.moving_avg')} stroke="#10b981" strokeWidth={2} dot={false} />
      </ComposedChart>
    </ResponsiveContainer>
  );
}
