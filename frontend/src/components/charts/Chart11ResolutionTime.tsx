// frontend/src/components/charts/Chart11ResolutionTime.tsx
'use client';

/** C11 Resolution Time (per docs/design/charts/c11-resolution-time.md) */
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import { useTranslation } from '@/i18n/hooks';

interface GroupRow { group: string; avg_days: number; median_days: number; count: number }
export interface ResolutionTimeData { group_by: 'priority' | 'type' | 'assignee'; rows: GroupRow[] }

export function Chart11ResolutionTime({ data, height = 400 }: { data: ResolutionTimeData; height?: number }) {
  const { t } = useTranslation();
  return (
    <ResponsiveContainer width="100%" height={height} data-testid="chart-c11-resolution-time">
      <BarChart data={data.rows}>
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis dataKey="group" />
        <YAxis label={{ value: t('chart.c11.y_axis'), angle: -90, position: 'insideLeft' }} />
        <Tooltip />
        <Legend />
        <Bar dataKey="avg_days" name={t('chart.c11.series.avg')} fill="#3b82f6" />
        <Bar dataKey="median_days" name={t('chart.c11.series.median')} fill="#10b981" />
      </BarChart>
    </ResponsiveContainer>
  );
}
